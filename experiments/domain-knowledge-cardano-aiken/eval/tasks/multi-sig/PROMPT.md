# Task: Threshold multi-signature validator

You are writing an Aiken spending validator for the Cardano blockchain.

## Behavior

A spending validator that authorizes a transaction when at least `threshold` of the keys in `signers` (from the datum) are also present in `tx.extra_signatories`.

Examples:
- A 3-of-5 multi-sig: `signers` has 5 keys, `threshold = 3`. Spend authorized if at least 3 of those 5 keys signed the transaction.
- A 2-of-3 multi-sig: `signers` has 3 keys, `threshold = 2`. Spend authorized if at least 2 of those 3 keys signed.

Signers from outside the datum's `signers` list do not count toward the threshold.

## Required interface

```aiken
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
    // your implementation here
  }
}
```

You may add a fallback handler if you wish but it is not required.

## Output

Write your complete validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/multi-sig/submissions/SUBMISSION.ak`

Do **not** write any tests of your own — a separate test suite will be appended automatically. Your file should contain only imports, type definitions, helper functions, and the `validator multi_sig { ... }` block. Stop after writing the file; do not run `aiken check` yourself.
