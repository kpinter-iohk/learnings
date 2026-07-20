use aiken/collection/dict
use aiken/collection/list
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    expect [Pair(minted_name, minted_qty)] =
      tx.mint |> assets.tokens(policy_id) |> dict.to_pairs()
    let name_ok = minted_name == token_name
    if minted_qty > 0 {
      let utxo_consumed =
        list.any(tx.inputs, fn(input) { input.output_reference == utxo_ref })
      name_ok? && (minted_qty == 1)? && utxo_consumed?
    } else if minted_qty < 0 {
      name_ok? && (minted_qty == -1)?
    } else {
      False
    }
  }
}
