# Task: fix a security flaw in a vault validator

You are auditing an Aiken spending validator. The validator has a security flaw. Find it and fix it.

## Vulnerable validator

```aiken
use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{OutputReference, Transaction}

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
      None -> True
    }
  }
}
```

## The intended semantics

The vault should authorize spending **only** when:

- the UTxO has a datum specifying an owner, **and**
- that owner's key is in the transaction's signatories.

If the datum is missing or the owner did not sign, spending must be rejected.

## Output

Fix the validator so the intended semantics are upheld. The `VaultDatum` type and the `vault` validator name must stay the same.

Write your fixed validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/audit-loose-datum/submissions/SUBMISSION.ak`

Do not write any tests of your own. Do not run `aiken check`. Stop after writing the file.
