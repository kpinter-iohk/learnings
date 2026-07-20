# Task: add burning to a one-shot NFT minting policy

You are extending an existing Aiken minting policy.

## Existing validator

This is the current code. It allows minting exactly one token of `token_name` when the parameter UTxO is consumed:

```aiken
use aiken/collection/dict
use aiken/collection/list
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    let utxo_consumed =
      list.any(tx.inputs, fn(input) { input.output_reference == utxo_ref })
    expect [Pair(minted_name, minted_qty)] =
      tx.mint |> assets.tokens(policy_id) |> dict.to_pairs()
    utxo_consumed && minted_name == token_name && minted_qty == 1
  }
}
```

## Required change

Allow **burning** the NFT. The validator should authorize a mint action when:

- **Mint case** (positive quantity): the parameter UTxO (`utxo_ref`) is consumed, **exactly one** token of `token_name` is minted under this policy, and no other tokens under this policy are minted. (Existing behavior, preserved.)
- **Burn case** (negative quantity): the burned token is `token_name`, and the burn quantity is exactly `-1`. The parameter UTxO does **not** need to be consumed when burning.
- Anything else (zero quantity, wrong name, multiple assets, wrong quantity): rejected.

## Required interface (unchanged signature)

```aiken
validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    // your implementation
  }
}
```

## Output

Write your complete validator code to:

`/home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval/tasks/brown-nft-burn/submissions/SUBMISSION.ak`

Do not write any tests of your own. Do not run `aiken check`. Stop after writing the file.
