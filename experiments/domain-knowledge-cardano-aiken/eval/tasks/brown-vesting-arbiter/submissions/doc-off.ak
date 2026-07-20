use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval
use cardano/transaction.{OutputReference, Transaction}

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
    owner_signed || arbiter_signed || (beneficiary_signed && time_reached)
  }
}
