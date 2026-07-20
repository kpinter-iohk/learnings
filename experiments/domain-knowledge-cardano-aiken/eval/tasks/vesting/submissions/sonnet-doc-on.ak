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
    let is_owner_signed = list.has(tx.extra_signatories, datum.owner)
    let is_beneficiary_signed = list.has(tx.extra_signatories, datum.beneficiary)
    let is_after_lock_until = interval.is_entirely_after(tx.validity_range, datum.lock_until - 1)
    is_owner_signed || (is_beneficiary_signed && is_after_lock_until)
  }
}
