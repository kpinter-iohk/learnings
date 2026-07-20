# Task: add a time-freeze to a threshold multi-sig validator

You are extending an existing Aiken spending validator.

## Existing validator

This is the current code. It authorizes spending when at least `threshold` of `signers` are present in `tx.extra_signatories`:

```aiken
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
    signed_count >= datum.threshold
  }
}
```

## Required change

Add a `frozen_until` field to the datum. Spending should be authorized only when **both**:

- the threshold signature condition is met (as before), **and**
- the transaction's validity range proves that the current time has reached `frozen_until`.

## Required interface (tests will reference these exact names)

```aiken
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
    // your implementation
  }
}
```

The time check semantics are the same as for a vesting validator: the lower bound of `tx.validity_range` must be at least `frozen_until`. Use `aiken/interval`.

## Output

Write your complete validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/brown-multisig-freeze/submissions/SUBMISSION.ak`

Do not write any tests of your own. Do not run `aiken check`. Stop after writing the file.
