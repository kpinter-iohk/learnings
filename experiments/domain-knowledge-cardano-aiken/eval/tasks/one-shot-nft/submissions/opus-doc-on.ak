use aiken/collection/dict
use aiken/collection/list
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    let Transaction { inputs, mint, .. } = tx

    // The parameter UTxO must be consumed in this transaction.
    let is_param_utxo_consumed =
      list.any(inputs, fn(input) { input.output_reference == utxo_ref })

    // Inspect the assets minted under this policy. There must be exactly
    // one asset entry, with the expected token name and quantity 1.
    expect [Pair(minted_name, minted_quantity)] =
      mint |> assets.tokens(policy_id) |> dict.to_pairs()

    is_param_utxo_consumed? && (minted_name == token_name)? && (minted_quantity == 1)?
  }
}
