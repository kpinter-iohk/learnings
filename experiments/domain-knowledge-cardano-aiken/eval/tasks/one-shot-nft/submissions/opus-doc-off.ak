use aiken/collection/list
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{OutputReference, Transaction}

validator one_shot_nft(utxo_ref: OutputReference, token_name: AssetName) {
  mint(_redeemer: Data, policy_id: PolicyId, tx: Transaction) {
    let Transaction { inputs, mint, .. } = tx

    // Check that the parameter UTxO is being consumed
    let utxo_consumed =
      list.any(inputs, fn(input) { input.output_reference == utxo_ref })

    // Inspect all assets minted under this policy
    // flatten returns a list of (PolicyId, AssetName, Int) triples
    let minted_under_policy =
      assets.flatten(mint)
        |> list.filter(fn(triple) {
            let (pid, _, _) = triple
            pid == policy_id
          })

    // Exactly one asset under this policy: token_name with quantity 1
    expect [(_, name, qty)] = minted_under_policy

    utxo_consumed && name == token_name && qty == 1
  }

  else(_) {
    False
  }
}
