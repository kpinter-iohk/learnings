use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    let Transaction { inputs, mint, .. } = tx

    let utxo_consumed =
      list.any(inputs, fn(input) { input.output_reference == utxo_ref })

    let policy_tokens = assets.tokens(mint, policy_id)

    let correct_mint =
      dict.to_list(policy_tokens) == [(token_name, 1)]

    utxo_consumed && correct_mint
  }
}
