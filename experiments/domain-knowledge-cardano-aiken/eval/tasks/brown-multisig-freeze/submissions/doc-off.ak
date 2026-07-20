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
    let signed_count =
      list.count(datum.signers, fn(key) { list.has(tx.extra_signatories, key) })
    let threshold_met = signed_count >= datum.threshold

    let Interval { lower_bound: IntervalBound { bound_type, .. }, .. } =
      tx.validity_range
    let time_reached =
      when bound_type is {
        Finite(lower) -> lower >= datum.frozen_until
        _ -> False
      }

    threshold_met && time_reached
  }
}
