use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval
use cardano/transaction.{OutputReference, Transaction}

pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
}

validator vesting {
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    expect Some(datum) = datum_opt
    let signed_by_owner = list.has(tx.extra_signatories, datum.owner)
    let signed_by_beneficiary =
      list.has(tx.extra_signatories, datum.beneficiary)
    let time_reached =
      interval.is_entirely_after(tx.validity_range, datum.lock_until - 1)
    signed_by_owner? || (signed_by_beneficiary? && time_reached?)
  }
}
