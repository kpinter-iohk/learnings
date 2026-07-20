# Task: fix a security vulnerability in a swap validator

You are auditing an Aiken spending validator. The validator has a known security flaw. Find it and fix it.

## Vulnerable validator

```aiken
use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/address
use cardano/assets
use cardano/transaction.{OutputReference, Transaction}

pub type SwapDatum {
  beneficiary: VerificationKeyHash,
  price: Int,
}

validator swap {
  spend(
    datum_opt: Option<SwapDatum>,
    _redeemer: Data,
    _own_ref: OutputReference,
    self: Transaction,
  ) {
    expect Some(datum) = datum_opt
    let beneficiary_address = address.from_verification_key(datum.beneficiary)
    let user_outputs =
      list.filter(self.outputs, fn(o) { o.address == beneficiary_address })
    let total_paid =
      list.foldl(
        user_outputs,
        0,
        fn(o, total) { total + assets.lovelace_of(o.value) },
      )
    total_paid >= datum.price
  }
}
```

## The vulnerability

When **two or more UTxOs** sitting at this script's address are spent in the **same transaction**, this validator runs once per spent UTxO but each instance inspects the same transaction-wide list of outputs. A single payment of `price` lovelace to the beneficiary satisfies the check for **every** spent UTxO simultaneously, allowing an attacker to drain multiple UTxOs while paying only once.

## Required fix

Modify the validator so that each spent UTxO requires its **own dedicated payment output**, identified by the input's `OutputReference`. Specifically:

- Each beneficiary output must carry an inline datum whose value is the `OutputReference` of the input being spent (`own_ref`).
- Outputs whose datum does not match `own_ref` (or has no datum / a different shape) must not count toward `total_paid`.

The `SwapDatum` type and the `swap` validator name must stay the same. Do not change the spend handler signature.

## Required interface (unchanged)

```aiken
pub type SwapDatum {
  beneficiary: VerificationKeyHash,
  price: Int,
}

validator swap {
  spend(
    datum_opt: Option<SwapDatum>,
    _redeemer: Data,
    own_ref: OutputReference,
    self: Transaction,
  ) {
    // your fixed implementation
  }
}
```

(Note that the original code used `_own_ref` to indicate the parameter was unused. Your fix must use it, so drop the underscore.)

## Output

Write your complete fixed validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/audit-double-sat/submissions/SUBMISSION.ak`

Do not write any tests of your own. Do not run `aiken check`. Stop after writing the file.
