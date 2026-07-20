// ============================================================
// Appended test suite — do not modify.
// ============================================================

use aiken/interval.{Interval}
use aiken/interval
use cardano/transaction.{placeholder} as eval_tx_module

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
const eval_frozen_until: Int = 5_000_000

fn eval_3_of_5_datum() -> MultiSigDatum {
  MultiSigDatum {
    signers: [eval_k1, eval_k2, eval_k3, eval_k4, eval_k5],
    threshold: 3,
    frozen_until: eval_frozen_until,
  }
}

fn eval_utxo() -> OutputReference {
  OutputReference { transaction_id: "", output_index: 0 }
}

fn eval_tx(
  signers: List<VerificationKeyHash>,
  range: Interval,
) -> Transaction {
  Transaction {
    ..eval_tx_module.placeholder,
    extra_signatories: signers,
    validity_range: range,
  }
}

test eval_threshold_met_after_freeze_passes() {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_k3], interval.after(eval_frozen_until + 1000)),
  )
}

test eval_all_signers_after_freeze_passes() {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx(
      [eval_k1, eval_k2, eval_k3, eval_k4, eval_k5],
      interval.after(eval_frozen_until + 1000),
    ),
  )
}

test eval_threshold_met_before_freeze_fails() fail {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_k3], interval.before(eval_frozen_until - 1000)),
  )
}

test eval_all_signers_before_freeze_fails() fail {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx(
      [eval_k1, eval_k2, eval_k3, eval_k4, eval_k5],
      interval.before(eval_frozen_until - 1000),
    ),
  )
}

test eval_below_threshold_after_freeze_fails() fail {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2], interval.after(eval_frozen_until + 1000)),
  )
}

test eval_outsiders_dont_count() fail {
  multi_sig.spend(
    Some(eval_3_of_5_datum()),
    Void,
    eval_utxo(),
    eval_tx(
      [eval_k1, eval_k2, eval_outsider, eval_outsider, eval_outsider],
      interval.after(eval_frozen_until + 1000),
    ),
  )
}

test eval_missing_datum_fails() fail {
  multi_sig.spend(
    None,
    Void,
    eval_utxo(),
    eval_tx([eval_k1, eval_k2, eval_k3], interval.after(eval_frozen_until + 1000)),
  )
}
