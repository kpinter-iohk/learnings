# Task: One-shot NFT minting policy

You are writing an Aiken minting policy for the Cardano blockchain.

## Behavior

A one-shot minting policy parameterized by a UTxO reference and an asset name. The policy authorizes minting only when **all** of these are true:

- the parameter UTxO (`utxo_ref`) is in the transaction's inputs (i.e. it is being consumed in this same transaction)
- exactly **one** token of `token_name` under this policy is being minted (mint quantity must equal 1)
- **no other assets** are being minted under this policy in the same transaction

Burning is not allowed by this policy (this is a one-shot mint).

## Required interface

```aiken
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    // your implementation here
  }
}
```

You may add a fallback handler if you wish but it is not required.

## Hints about what NOT to do

- A `mint` handler does not receive a datum. You will get a compile error if you try to declare a datum parameter.
- The mint field on `Transaction` is a `Value`. Use `aiken/cardano/assets` stdlib functions to inspect it; do not try to destructure `Value` directly.

## Output

Write your complete validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/one-shot-nft/submissions/SUBMISSION.ak`

Do **not** write any tests of your own — a separate test suite will be appended automatically. Your file should contain only imports, type definitions, helper functions, and the `validator one_shot_nft(...) { ... }` block. Stop after writing the file; do not run `aiken check` yourself.
