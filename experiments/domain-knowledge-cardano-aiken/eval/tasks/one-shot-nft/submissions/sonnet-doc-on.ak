use aiken/collection/dict
use aiken/collection/list
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    let minted_tokens =
      tx.mint
        |> assets.tokens(policy_id)
        |> dict.to_pairs()

    let utxo_consumed =
      list.any(tx.inputs, fn(input) { input.output_reference == utxo_ref })

    let exactly_one_nft =
      when minted_tokens is {
        [Pair(name, quantity)] -> name == token_name && quantity == 1
        _ -> False
      }

    utxo_consumed? && exactly_one_nft?
  }
}
