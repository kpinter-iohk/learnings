use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction, placeholder} as eval_tx_module



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
// ============================================================
// Appended test suite — do not modify.
// ============================================================


const eval_owner: VerificationKeyHash =
  #"11111111111111111111111111111111111111111111111111111111"
const eval_stranger: VerificationKeyHash =
  #"99999999999999999999999999999999999999999999999999999999"

fn eval_datum() -> VaultDatum {
  VaultDatum { owner: eval_owner }
}

fn eval_utxo() -> OutputReference {
  OutputReference { transaction_id: "", output_index: 0 }
}

fn eval_tx(signers: List<VerificationKeyHash>) -> Transaction {
  Transaction { ..eval_tx_module.placeholder, extra_signatories: signers }
}

test eval_owner_signs_with_datum_passes() {
  vault.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_owner]),
  )
}

test eval_stranger_signs_fails() fail {
  vault.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_stranger]),
  )
}

test eval_no_signers_fails() fail {
  vault.spend(
    Some(eval_datum()),
    Void,
    eval_utxo(),
    eval_tx([]),
  )
}

// The vulnerability the fix must close:
// vulnerable code returns True for None datum, allowing anyone to spend.
test eval_missing_datum_owner_signs_fails() fail {
  vault.spend(
    None,
    Void,
    eval_utxo(),
    eval_tx([eval_owner]),
  )
}

test eval_missing_datum_stranger_signs_fails() fail {
  vault.spend(
    None,
    Void,
    eval_utxo(),
    eval_tx([eval_stranger]),
  )
}

test eval_missing_datum_no_signers_fails() fail {
  vault.spend(None, Void, eval_utxo(), eval_tx([]))
}
