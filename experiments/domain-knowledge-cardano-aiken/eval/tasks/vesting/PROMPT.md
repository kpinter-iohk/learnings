# Task: Vesting validator

You are writing an Aiken validator for the Cardano blockchain.

## Behavior

A spending validator that locks funds with three datum fields: a `lock_until` time (POSIX milliseconds, expressed as an `Int`), an `owner`, and a `beneficiary`. The validator authorizes spending when **either**:

- the `owner` is among the transaction's signatories (the owner can always reclaim), **or**
- the `beneficiary` is among the transaction's signatories **and** the transaction's validity range proves that the current time has reached `lock_until`.

If neither condition is met, the validator must return `False`.

## Required interface

Define the types and validator with these exact names so external tests can reference them:

```aiken
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction}

pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
}

validator vesting {
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    // your implementation here
  }
}
```

You may add a fallback handler if you wish but it is not required.

## What "validity range proves time has reached lock_until" means

The validator can deduce a lower bound on the current time from `tx.validity_range`. If that lower bound is at least `lock_until`, then time has reached `lock_until`. Use `aiken/interval` for this check.

## Output

Write your complete validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/vesting/submissions/SUBMISSION.ak`

Do **not** write any tests of your own — a separate test suite will be appended automatically. Your file should contain only imports, type definitions, helper functions, and the `validator vesting { ... }` block. Stop after writing the file; do not run `aiken check` yourself.
