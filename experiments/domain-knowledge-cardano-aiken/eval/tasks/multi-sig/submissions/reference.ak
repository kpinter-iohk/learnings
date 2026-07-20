use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction}

pub type MultiSigDatum {
  signers: List<VerificationKeyHash>,
  threshold: Int,
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
    (signed_count >= datum.threshold)?
  }
}
