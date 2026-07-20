use aiken/collection/list as eval_list
use cardano/address.{Address, Script}
use cardano/assets
use cardano/assets.{AssetName, PolicyId}
use cardano/transaction.{Input, NoDatum, Output, OutputReference, Transaction, placeholder} as eval_tx_module



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
// ============================================================
// Appended test suite — do not modify.
// ============================================================


const eval_policy_id: PolicyId =
  #"aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"
const eval_token_name: AssetName = #"4d59544f4b454e"
const eval_other_name: AssetName = #"4f54484552"

fn eval_param_utxo() -> OutputReference {
  OutputReference {
    transaction_id: #"abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1",
    output_index: 0,
  }
}

fn eval_other_utxo() -> OutputReference {
  OutputReference {
    transaction_id: #"def456def456def456def456def456def456def456def456def456def456def4",
    output_index: 7,
  }
}

fn eval_dummy_output() -> Output {
  Output {
    address: Address {
      payment_credential: Script(
        #"aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
      ),
      stake_credential: None,
    },
    value: assets.zero,
    datum: NoDatum,
    reference_script: None,
  }
}

fn eval_input_with(out_ref: OutputReference) -> Input {
  Input { output_reference: out_ref, output: eval_dummy_output() }
}

fn eval_tx(
  inputs: List<OutputReference>,
  mint_value: assets.Value,
) -> Transaction {
  Transaction {
    ..eval_tx_module.placeholder,
    inputs: eval_list.map(inputs, eval_input_with),
    mint: mint_value,
  }
}

test eval_param_consumed_one_token_passes() {
  one_shot_nft.mint(
    eval_param_utxo(),
    eval_token_name,
    Void,
    eval_policy_id,
    eval_tx(
      [eval_param_utxo()],
      assets.from_asset(eval_policy_id, eval_token_name, 1),
    ),
  )
}

test eval_param_not_consumed_fails() fail {
  one_shot_nft.mint(
    eval_param_utxo(),
    eval_token_name,
    Void,
    eval_policy_id,
    eval_tx(
      [eval_other_utxo()],
      assets.from_asset(eval_policy_id, eval_token_name, 1),
    ),
  )
}

test eval_two_tokens_fails() fail {
  one_shot_nft.mint(
    eval_param_utxo(),
    eval_token_name,
    Void,
    eval_policy_id,
    eval_tx(
      [eval_param_utxo()],
      assets.from_asset(eval_policy_id, eval_token_name, 2),
    ),
  )
}

test eval_wrong_name_fails() fail {
  one_shot_nft.mint(
    eval_param_utxo(),
    eval_token_name,
    Void,
    eval_policy_id,
    eval_tx(
      [eval_param_utxo()],
      assets.from_asset(eval_policy_id, eval_other_name, 1),
    ),
  )
}

test eval_extra_asset_under_same_policy_fails() fail {
  one_shot_nft.mint(
    eval_param_utxo(),
    eval_token_name,
    Void,
    eval_policy_id,
    eval_tx(
      [eval_param_utxo()],
      assets.merge(
        assets.from_asset(eval_policy_id, eval_token_name, 1),
        assets.from_asset(eval_policy_id, eval_other_name, 1),
      ),
    ),
  )
}
