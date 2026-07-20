use aiken/crypto.{VerificationKeyHash}
use aiken/interval.{Finite, IntervalBound}
use cardano/transaction.{OutputReference, Transaction}

pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
}

fn must_be_signed_by(tx: Transaction, vkh: VerificationKeyHash) -> Bool {
  list.has(tx.extra_signatories, vkh)
}

fn lower_bound_reached(tx: Transaction, lock_until: Int) -> Bool {
  when tx.validity_range.lower_bound.bound_type is {
    Finite(lower) -> lower >= lock_until
    _ -> False
  }
}

validator vesting {
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    when datum_opt is {
      None -> False
      Some(datum) ->
        if must_be_signed_by(tx, datum.owner) {
          True
        } else {
          must_be_signed_by(tx, datum.beneficiary) && lower_bound_reached(
            tx,
            datum.lock_until,
          )
        }
    }
  }
}
