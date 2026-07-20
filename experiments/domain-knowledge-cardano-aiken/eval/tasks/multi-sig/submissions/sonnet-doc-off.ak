use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction}

pub type MultiSigDatum {
  signers: List<VerificationKeyHash>,
  threshold: Int,
}

fn count_signers(
  signers: List<VerificationKeyHash>,
  extra_signatories: List<VerificationKeyHash>,
) -> Int {
  list.foldl(
    signers,
    0,
    fn(signer, acc) {
      if list.has(extra_signatories, signer) {
        acc + 1
      } else {
        acc
      }
    },
  )
}

validator multi_sig {
  spend(
    datum_opt: Option<MultiSigDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    when datum_opt is {
      None -> False
      Some(datum) -> {
        let valid_count =
          count_signers(datum.signers, tx.extra_signatories)
        valid_count >= datum.threshold
      }
    }
  }
}
