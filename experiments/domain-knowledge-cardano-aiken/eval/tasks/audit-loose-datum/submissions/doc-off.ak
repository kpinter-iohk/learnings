use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction}

pub type VaultDatum {
  owner: VerificationKeyHash,
}

validator vault {
  spend(
    datum_opt: Option<VaultDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    when datum_opt is {
      Some(d) -> list.has(tx.extra_signatories, d.owner)
      None -> False
    }
  }
}
