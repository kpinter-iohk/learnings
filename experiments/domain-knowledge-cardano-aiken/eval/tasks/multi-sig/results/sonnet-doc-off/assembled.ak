use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction, placeholder} as eval_tx_module



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
// ============================================================
// Appended test suite — do not modify.
// ============================================================


const eval_k1: VerificationKeyHash =
  #"11111111111111111111111111111111111111111111111111111111"
const eval_k2: VerificationKeyHash =
  #"22222222222222222222222222222222222222222222222222222222"
const eval_k3: VerificationKeyHash =
  #"33333333333333333333333333333333333333333333333333333333"
const eval_k4: VerificationKeyHash =
  #"44444444444444444444444444444444444444444444444444444444"
const eval_k5: VerificationKeyHash =
  #"55555555555555555555555555555555555555555555555555555555"
const eval_outsider: VerificationKeyHash =
  #"99999999999999999999999999999999999999999999999999999999"

fn eval_3_of_5_datum() -> MultiSigDatum {
  MultiSigDatum {
    signers: [eval_k1, eval_k2, eval_k3, eval_k4, eval_k5],
    threshold: 3,
  }
}

fn eval_utxo() -> OutputReference {
  OutputReference { transaction_id: "", output_index: 0 }
}

fn eval_tx(signers: List<VerificationKeyHash>) -> Transaction {
  Transaction { ..eval_tx_module.placeholder, extra_signatories: signers }
}

test eval_3_of_5_exactly_3_signers_passes() {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_k3]),
  )
}

test eval_3_of_5_all_5_signers_passes() {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_k3, eval_k4, eval_k5]),
  )
}

test eval_3_of_5_only_2_signers_fails() fail {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2]),
  )
}

test eval_3_of_5_zero_signers_fails() fail {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([]),
  )
}

test eval_outsiders_dont_count() fail {
  // 2 listed signers + 3 outsiders should still fail a 3-of-5 multisig
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_outsider, eval_outsider, eval_outsider]),
  )
}

test eval_missing_datum_fails() fail {
  multi_sig.spend(
    None,
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_k3]),
  )
}
