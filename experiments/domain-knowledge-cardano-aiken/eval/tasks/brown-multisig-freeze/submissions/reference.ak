use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval
use cardano/transaction.{OutputReference, Transaction}

pub type MultiSigDatum {
  signers: List<VerificationKeyHash>,
  threshold: Int,
  frozen_until: Int,
}

validator multi_sig {
  spend(
    datum_opt: Option<MultiSigDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    expect Some(datum) = datum_opt
    let signed_count =
      list.count(datum.signers, fn(key) { list.has(tx.extra_signatories, key) })
    let threshold_met = signed_count >= datum.threshold
    let unfrozen =
      interval.is_entirely_after(tx.validity_range, datum.frozen_until - 1)
    threshold_met? && unfrozen?
  }
}
