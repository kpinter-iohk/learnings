use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval.{Finite, Interval, IntervalBound}
use cardano/transaction.{OutputReference, Transaction}

pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
}

fn must_be_signed_by(tx: Transaction, vkh: VerificationKeyHash) -> Bool {
  list.has(tx.extra_signatories, vkh)
}

fn must_start_after(range: Interval<Int>, lock_until: Int) -> Bool {
  when range.lower_bound.bound_type is {
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
    expect Some(datum) = datum_opt

    let owner_signed = must_be_signed_by(tx, datum.owner)
    let beneficiary_signed = must_be_signed_by(tx, datum.beneficiary)
    let time_reached = must_start_after(tx.validity_range, datum.lock_until)

    owner_signed || (beneficiary_signed && time_reached)
  }
}
