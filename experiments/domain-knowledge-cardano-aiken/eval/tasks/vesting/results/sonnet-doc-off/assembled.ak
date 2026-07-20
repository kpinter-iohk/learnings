use aiken/crypto.{VerificationKeyHash}
use aiken/interval
use aiken/interval.{Finite, Interval, IntervalBound}
use cardano/transaction.{OutputReference, Transaction, placeholder} as eval_tx_module



pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
}

fn must_be_signed_by(tx: Transaction, vkh: VerificationKeyHash) -> Bool {
  list.has(tx.extra_signatories, vkh)
}

fn lower_bound_reached(tx: Transaction, lock_until: Int) -> Bool {
  when tx.validity_range.lower_bound.bound_type is {
    Finite(lower) -> lower >= lock_until
    _ -> False
  }
}

validator vesting {
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    when datum_opt is {
      None -> False
      Some(datum) ->
        if must_be_signed_by(tx, datum.owner) {
          True
        } else {
          must_be_signed_by(tx, datum.beneficiary) && lower_bound_reached(
            tx,
            datum.lock_until,
          )
        }
    }
  }
}
// ============================================================
// Appended test suite — do not modify.
// ============================================================


const eval_owner_key: VerificationKeyHash =
  #"11111111111111111111111111111111111111111111111111111111"
const eval_beneficiary_key: VerificationKeyHash =
  #"22222222222222222222222222222222222222222222222222222222"
const eval_stranger_key: VerificationKeyHash =
  #"33333333333333333333333333333333333333333333333333333333"
const eval_lock_until: Int = 1_000_000

fn eval_datum() -> VestingDatum {
  VestingDatum {
    lock_until: eval_lock_until,
    owner: eval_owner_key,
    beneficiary: eval_beneficiary_key,
  }
}

fn eval_utxo() -> OutputReference {
  OutputReference { transaction_id: "", output_index: 0 }
}

fn eval_tx(signers: List<VerificationKeyHash>, range: Interval) -> Transaction {
  Transaction {
    ..eval_tx_module.placeholder,
    extra_signatories: signers,
    validity_range: range,
  }
}

test eval_owner_alone_before_lock_passes() {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_owner_key], interval.before(eval_lock_until - 1000)),
  )
}

test eval_beneficiary_after_lock_passes() {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_beneficiary_key], interval.after(eval_lock_until + 1000)),
  )
}

test eval_beneficiary_before_lock_fails() fail {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_beneficiary_key], interval.before(eval_lock_until - 1000)),
  )
}

test eval_no_signers_fails() fail {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([], interval.after(eval_lock_until + 1000)),
  )
}

test eval_stranger_after_lock_fails() fail {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_stranger_key], interval.after(eval_lock_until + 1000)),
  )
}

test eval_missing_datum_fails() fail {
  vesting.spend(
    None,
    Void,
    eval_utxo(),
    eval_tx([eval_owner_key], interval.everything),
  )
}
