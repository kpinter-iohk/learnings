use aiken/collection/list as eval_list
use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/address
use cardano/address.{Address, VerificationKey}
use cardano/assets
use cardano/transaction.{InlineDatum, Output, OutputReference, Transaction, placeholder} as eval_tx_module



pub type SwapDatum {
  beneficiary: VerificationKeyHash,
  price: Int,
}

validator swap {
  spend(
    datum_opt: Option<SwapDatum>,
    _redeemer: Data,
    own_ref: OutputReference,
    self: Transaction,
  ) {
    expect Some(datum) = datum_opt
    let beneficiary_address = address.from_verification_key(datum.beneficiary)
    let dedicated_outputs =
      list.filter(
        self.outputs,
        fn(o) {
          if o.address != beneficiary_address {
            False
          } else {
            when o.datum is {
              InlineDatum(d) -> {
                expect tagged_ref: OutputReference = d
                tagged_ref == own_ref
              }
              _ -> False
            }
          }
        },
      )
    let total_paid =
      list.foldl(
        dedicated_outputs,
        0,
        fn(o, total) { total + assets.lovelace_of(o.value) },
      )
    total_paid >= datum.price
  }
}
// ============================================================
// Appended test suite — do not modify.
// ============================================================


const eval_beneficiary: VerificationKeyHash =
  #"11111111111111111111111111111111111111111111111111111111"
const eval_price: Int = 100_000_000

fn eval_datum() -> SwapDatum {
  SwapDatum { beneficiary: eval_beneficiary, price: eval_price }
}

fn eval_ref_1() -> OutputReference {
  OutputReference {
    transaction_id: #"abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1",
    output_index: 0,
  }
}

fn eval_ref_2() -> OutputReference {
  OutputReference {
    transaction_id: #"abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1",
    output_index: 1,
  }
}

fn eval_beneficiary_address() -> Address {
  Address {
    payment_credential: VerificationKey(eval_beneficiary),
    stake_credential: None,
  }
}

fn eval_tagged_output(lovelace: Int, tag: OutputReference) -> Output {
  Output {
    address: eval_beneficiary_address(),
    value: assets.from_lovelace(lovelace),
    datum: InlineDatum(tag),
    reference_script: None,
  }
}

fn eval_untagged_output(lovelace: Int) -> Output {
  Output {
    address: eval_beneficiary_address(),
    value: assets.from_lovelace(lovelace),
    datum: eval_tx_module.NoDatum,
    reference_script: None,
  }
}

fn eval_tx(outputs: List<Output>) -> Transaction {
  Transaction { ..eval_tx_module.placeholder, outputs: outputs }
}

test eval_tagged_full_payment_passes() {
  swap.spend(
    Some(eval_datum()),
    Void,
    eval_ref_1(),
    eval_tx([eval_tagged_output(eval_price, eval_ref_1())]),
  )
}

test eval_no_outputs_fails() fail {
  swap.spend(Some(eval_datum()), Void, eval_ref_1(), eval_tx([]))
}

// The attack scenario the fix must prevent:
// own_ref = R1, but the only payment is tagged with a *different* ref (R2).
// Vulnerable code would still see "100M paid to beneficiary" and pass.
test eval_attack_other_ref_tag_fails() fail {
  swap.spend(
    Some(eval_datum()),
    Void,
    eval_ref_1(),
    eval_tx([eval_tagged_output(eval_price, eval_ref_2())]),
  )
}

// Same family of attack: payment to beneficiary but with NO tag at all.
test eval_untagged_payment_fails() fail {
  swap.spend(
    Some(eval_datum()),
    Void,
    eval_ref_1(),
    eval_tx([eval_untagged_output(eval_price)]),
  )
}

test eval_tagged_underpayment_fails() fail {
  swap.spend(
    Some(eval_datum()),
    Void,
    eval_ref_1(),
    eval_tx([eval_tagged_output(eval_price - 1, eval_ref_1())]),
  )
}

// Tagged payment present, but ALSO an untagged output — total should only
// count the tagged one.
test eval_mixed_only_tagged_counts() {
  swap.spend(
    Some(eval_datum()),
    Void,
    eval_ref_1(),
    eval_tx([
      eval_tagged_output(eval_price, eval_ref_1()),
      eval_untagged_output(eval_price),  // ignored
    ]),
  )
}

// Probe: a foreign output at the beneficiary's address carries an InlineDatum
// of an UNRELATED shape (an Int here). A well-written fix uses soft-cast
// (`if d is OutputReference`) and skips this output. A naive fix that uses
// `expect tagged_ref: OutputReference = d` will crash on this output even
// though there is a perfectly valid tagged payment elsewhere in the tx.
// The expected on-chain behaviour is to skip and succeed.
fn eval_foreign_datum_output(lovelace: Int) -> Output {
  Output {
    address: eval_beneficiary_address(),
    value: assets.from_lovelace(lovelace),
    datum: InlineDatum(42),
    reference_script: None,
  }
}

test eval_foreign_inline_datum_is_skipped() {
  swap.spend(
    Some(eval_datum()),
    Void,
    eval_ref_1(),
    eval_tx([
      eval_foreign_datum_output(eval_price),  // foreign — must be ignored
      eval_tagged_output(eval_price, eval_ref_1()),
    ]),
  )
}
