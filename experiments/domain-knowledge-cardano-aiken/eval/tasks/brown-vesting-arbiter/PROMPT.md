# Task: extend vesting validator with an arbiter

You are extending an existing Aiken vesting validator for the Cardano blockchain.

## Existing validator

This is the current code. It allows the owner to always spend, or the beneficiary to spend after a lock time:

```aiken
use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/interval
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
    expect Some(datum) = datum_opt
    let owner_signed = list.has(tx.extra_signatories, datum.owner)
    let beneficiary_signed = list.has(tx.extra_signatories, datum.beneficiary)
    let time_reached = interval.is_entirely_after(tx.validity_range, datum.lock_until - 1)
    owner_signed || (beneficiary_signed && time_reached)
  }
}
```

## Required change

Add a third party — an **arbiter** — who can spend at any time regardless of the lock or other signers. The arbiter is intended as a recovery key.

## Required interface (tests will reference these exact names)

```aiken
pub type VestingDatum {
  lock_until: Int,
  owner: VerificationKeyHash,
  beneficiary: VerificationKeyHash,
  arbiter: VerificationKeyHash,
}

validator vesting {
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _utxo: OutputReference,
    tx: Transaction,
  ) {
    // your implementation
  }
}
```

Semantics:
- Owner signs → spend authorized (any time).
- Beneficiary signs AND time has reached `lock_until` → spend authorized.
- Arbiter signs → spend authorized (any time, no other conditions).
- Otherwise → spend rejected.

## Output

Write your complete validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/brown-vesting-arbiter/submissions/SUBMISSION.ak`

Do not write any tests of your own. Do not run `aiken check`. Stop after writing the file.
