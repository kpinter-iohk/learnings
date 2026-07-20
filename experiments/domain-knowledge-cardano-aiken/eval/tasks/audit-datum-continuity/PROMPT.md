# Task: fix a state-machine validator missing a continuity check

You are auditing an Aiken spending validator. The validator implements a counter state machine: when spent, it consumes a state UTxO and creates a new one with an incremented counter. There is a missing check. Find and fix it.

## Vulnerable validator

```aiken
use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{InlineDatum, OutputReference, Transaction}

pub type StateDatum {
  authority: VerificationKeyHash,
  counter: Int,
}

validator state_machine {
  spend(
    datum_opt: Option<StateDatum>,
    _redeemer: Data,
    own_ref: OutputReference,
    tx: Transaction,
  ) {
    expect Some(in_state) = datum_opt
    let authority_signed = list.has(tx.extra_signatories, in_state.authority)

    expect Some(own_input) =
      list.find(tx.inputs, fn(i) { i.output_reference == own_ref })
    expect Some(out) =
      list.find(tx.outputs, fn(o) { o.address == own_input.output.address })
    expect InlineDatum(out_data) = out.datum
    expect out_state: StateDatum = out_data

    authority_signed && out_state.counter == in_state.counter + 1
  }
}
```

## The intended semantics

The `StateDatum` has two fields:

- `authority` is **immutable** — it identifies the principal who controls this state machine and must be preserved exactly across every UTxO transition.
- `counter` is **mutable** but constrained: each transition must increment it by exactly 1.

The current code constrains `counter` correctly but **does not constrain `authority`**, allowing the current authority to transfer the role to a different key in the same transaction. That breaks the contract's identity guarantee.

## Required fix

Add the missing check so that any continuing-output state datum preserves `authority` exactly.

## Required interface (unchanged)

```aiken
pub type StateDatum {
  authority: VerificationKeyHash,
  counter: Int,
}

validator state_machine {
  spend(
    datum_opt: Option<StateDatum>,
    _redeemer: Data,
    own_ref: OutputReference,
    tx: Transaction,
  ) {
    // your fixed implementation
  }
}
```

## Output

Write your complete fixed validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/audit-datum-continuity/submissions/SUBMISSION.ak`

Do not write any tests of your own. Do not run `aiken check`. Stop after writing the file.
