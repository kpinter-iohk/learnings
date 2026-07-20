use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval
use aiken/interval.{Interval}
use cardano/transaction.{OutputReference, Transaction, placeholder} as eval_tx_module



pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
  arbiter: VerificationKeyHash,
}

validator vesting {
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    expect Some(datum) = datum_opt
    let owner_signed = list.has(tx.extra_signatories, datum.owner)
    let beneficiary_signed = list.has(tx.extra_signatories, datum.beneficiary)
    let arbiter_signed = list.has(tx.extra_signatories, datum.arbiter)
    let time_reached =
      interval.is_entirely_after(tx.validity_range, datum.lock_until - 1)
    arbiter_signed || owner_signed || (beneficiary_signed && time_reached)
  }
}
// ============================================================
// Appended test suite — do not modify.
// ============================================================


const eval_owner: VerificationKeyHash =
  #"11111111111111111111111111111111111111111111111111111111"
const eval_beneficiary: VerificationKeyHash =
  #"22222222222222222222222222222222222222222222222222222222"
const eval_arbiter: VerificationKeyHash =
  #"33333333333333333333333333333333333333333333333333333333"
const eval_stranger: VerificationKeyHash =
  #"99999999999999999999999999999999999999999999999999999999"
const eval_lock_until: Int = 1_000_000

fn eval_datum() -> VestingDatum {
  VestingDatum {
    lock_until: eval_lock_until,
    owner: eval_owner,
    beneficiary: eval_beneficiary,
    arbiter: eval_arbiter,
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

test eval_owner_alone_anytime_passes() {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_owner], interval.before(eval_lock_until - 1000)),
  )
}

test eval_beneficiary_after_lock_passes() {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_beneficiary], interval.after(eval_lock_until + 1000)),
  )
}

test eval_beneficiary_before_lock_fails() fail {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_beneficiary], interval.before(eval_lock_until - 1000)),
  )
}

test eval_arbiter_alone_before_lock_passes() {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_arbiter], interval.before(eval_lock_until - 1000)),
  )
}

test eval_arbiter_alone_after_lock_passes() {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_arbiter], interval.after(eval_lock_until + 1000)),
  )
}

test eval_no_signers_fails() fail {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([], interval.everything),
  )
}

test eval_stranger_alone_fails() fail {
  vesting.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_stranger], interval.everything),
  )
}

test eval_missing_datum_fails() fail {
  vesting.spend(
    None,
    Void,
    eval_utxo(),
    eval_tx([eval_owner], interval.everything),
  )
}
