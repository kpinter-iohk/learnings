use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval.{Finite, Interval, IntervalBound}
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

    // Threshold signature check: at least `threshold` of `signers` are present
    // in `tx.extra_signatories`.
    let signed_count =
      list.count(datum.signers, fn(key) { list.has(tx.extra_signatories, key) })
    let threshold_met = signed_count >= datum.threshold

    // Time check: the lower bound of the transaction's validity range must
    // be at least `frozen_until`. Same semantics as a vesting validator —
    // a finite lower bound `lo` proves "current time >= lo".
    let Interval { lower_bound: IntervalBound { bound_type, .. }, .. } =
      tx.validity_range
    let freeze_elapsed =
      when bound_type is {
        Finite(lo) -> lo >= datum.frozen_until
        _ -> False
      }

    threshold_met? && freeze_elapsed?
  }
}
