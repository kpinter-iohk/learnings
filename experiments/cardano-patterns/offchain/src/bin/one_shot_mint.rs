//! Off-chain builder for the one-shot mint pattern.
//!
//! The minting policy is parameterized by a specific UTxO (the "seed").
//! That UTxO must be consumed in the same transaction that mints, so the
//! policy can fire exactly once. After it fires, the seed is gone and
//! no future transaction can satisfy the policy.
//!
//! Build & run:
//!   cargo run --bin one_shot_mint

use cardano_serialization_lib as csl;

// ---------------------------------------------------------------------------
// 1. The seed UTxO. In production this comes from a wallet's coin selection.
// ---------------------------------------------------------------------------

const SEED_TXID_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";
const SEED_OUTPUT_INDEX: u32 = 0;

/// Compiled bytecode of the validator with the seed already applied.
///
/// Produced by:
///   aiken blueprint apply \
///       --module one_shot --validator one_shot \
///       <seed-as-plutus-data-cbor>
///
/// The hex below corresponds to seed = (txid 0x00..01, output_index 0).
/// Re-running `aiken blueprint apply` with a different seed yields a
/// different bytecode and therefore a different policy id.
const APPLIED_POLICY_HEX: &str = "58b10101003229800aba2aba1aab9faab9eaab9dab9a48888896600264646644b30013370e900018031baa00189991198008009bac300b30093754601600c6eb8c024c01cdd5000912cc00400629422b30013375e601660126ea8c02c00403a29462660040046018002803900a459005180380098039804000980380098019baa0078a4d1365640044c126d8798258200000000000000000000000000000000000000000000000000000000000000001000001";

fn main() {
    // ---- 2. Load the parameterized policy ----------------------------------
    let bytes = hex::decode(APPLIED_POLICY_HEX).expect("policy hex");
    let policy = csl::PlutusScript::new_v3(bytes);
    let policy_id: csl::ScriptHash = policy.hash();
    println!("policy_id = {}", policy_id.to_hex());

    // ---- 3. Build the mint transaction -------------------------------------
    let tx = build_mint_tx(&policy, &policy_id);
    println!("tx body cbor = {}", hex::encode(tx.body().to_bytes()));
}

fn build_mint_tx(policy: &csl::PlutusScript, policy_id: &csl::ScriptHash) -> csl::Transaction {
    let cfg = csl::TransactionBuilderConfigBuilder::new()
        .fee_algo(&csl::LinearFee::new(
            &csl::BigNum::from(44u64),
            &csl::BigNum::from(155_381u64),
        ))
        .pool_deposit(&csl::BigNum::from(500_000_000u64))
        .key_deposit(&csl::BigNum::from(2_000_000u64))
        .max_value_size(5_000)
        .max_tx_size(16_384)
        .coins_per_utxo_byte(&csl::BigNum::from(4_310u64))
        .ex_unit_prices(&csl::ExUnitPrices::new(
            &csl::UnitInterval::new(&csl::BigNum::from(577u64), &csl::BigNum::from(10_000u64)),
            &csl::UnitInterval::new(&csl::BigNum::from(721u64), &csl::BigNum::from(10_000_000u64)),
        ))
        .build()
        .unwrap();
    let mut builder = csl::TransactionBuilder::new(&cfg);

    let recipient = demo_address();

    // ---- 4. Spend the seed UTxO -------------------------------------------
    // This is the linchpin: the policy requires this exact input. Consuming
    // it here is what allows the mint to succeed. After this transaction,
    // the seed is gone and no future transaction can satisfy the policy.
    let mut inputs = csl::TxInputsBuilder::new();
    let seed_input = csl::TransactionInput::new(
        &csl::TransactionHash::from_hex(SEED_TXID_HEX).unwrap(),
        SEED_OUTPUT_INDEX,
    );
    inputs
        .add_regular_input(
            &recipient,
            &seed_input,
            &csl::Value::new(&csl::BigNum::from(10_000_000u64)),
        )
        .unwrap();
    builder.set_inputs(&inputs);

    // ---- 5. Mint one token under the parameterized policy ------------------
    // The validator ignores the redeemer (it only inspects the input list),
    // so any value works. We use Unit (Constr 0 []) by convention.
    let asset_name = csl::AssetName::new(b"OneShot".to_vec()).unwrap();
    let redeemer = csl::Redeemer::new(
        &csl::RedeemerTag::new_mint(),
        &csl::BigNum::zero(),
        &csl::PlutusData::new_empty_constr_plutus_data(&csl::BigNum::zero()),
        &csl::ExUnits::new(
            &csl::BigNum::from(500_000u64),
            &csl::BigNum::from(200_000_000u64),
        ),
    );
    let witness =
        csl::MintWitness::new_plutus_script(&csl::PlutusScriptSource::new(policy), &redeemer);
    let mut mint = csl::MintBuilder::new();
    mint.add_asset(&witness, &asset_name, &csl::Int::new(&csl::BigNum::from(1u64)))
        .unwrap();
    builder.set_mint_builder(&mint);

    // ---- 5b. Collateral. Plutus scripts require a collateral input — a
    //          plain UTxO that the chain seizes if the script fails. In a
    //          wallet this is a dedicated reserved UTxO; here we use a
    //          throwaway second input owned by the same address.
    let mut collateral = csl::TxInputsBuilder::new();
    let collateral_input = csl::TransactionInput::new(
        &csl::TransactionHash::from_hex(
            "0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap(),
        0,
    );
    collateral
        .add_regular_input(
            &recipient,
            &collateral_input,
            &csl::Value::new(&csl::BigNum::from(5_000_000u64)),
        )
        .unwrap();
    builder.set_collateral(&collateral);

    // ---- 6. Send the minted token (plus minimum ADA) to the recipient ------
    let mut assets = csl::Assets::new();
    assets.insert(&asset_name, &csl::BigNum::from(1u64));
    let mut multiasset = csl::MultiAsset::new();
    multiasset.insert(policy_id, &assets);
    let mut value = csl::Value::new(&csl::BigNum::from(2_000_000u64));
    value.set_multiasset(&multiasset);

    let output = csl::TransactionOutputBuilder::new()
        .with_address(&recipient)
        .next()
        .unwrap()
        .with_value(&value)
        .build()
        .unwrap();
    builder.add_output(&output).unwrap();

    // ---- 7. Balance change and assemble ------------------------------------
    builder
        .calc_script_data_hash(&csl::TxBuilderConstants::plutus_conway_cost_models())
        .unwrap();
    builder.add_change_if_needed(&recipient).unwrap();
    builder.build_tx().unwrap()
}

fn demo_address() -> csl::Address {
    // Throwaway keys for a self-contained demo. A real builder would receive
    // these from a wallet.
    let payment = csl::PublicKey::from_bytes(&[1u8; 32]).unwrap();
    let stake = csl::PublicKey::from_bytes(&[2u8; 32]).unwrap();
    csl::BaseAddress::new(
        csl::NetworkInfo::testnet_preview().network_id(),
        &csl::Credential::from_keyhash(&payment.hash()),
        &csl::Credential::from_keyhash(&stake.hash()),
    )
    .to_address()
}
