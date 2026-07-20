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
    let MultiSigDatum { signers, threshold } = datum
    let signed_count =
      list.foldl(
        signers,
        0,
        fn(signer, acc) {
          if list.has(tx.extra_signatories, signer) {
            acc + 1
          } else {
            acc
          }
        },
      )
    (threshold >= 0)? && (signed_count >= threshold)?
  }
}
