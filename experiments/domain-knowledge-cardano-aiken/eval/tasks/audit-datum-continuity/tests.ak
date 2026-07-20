// ============================================================
// Appended test suite — do not modify.
// ============================================================

use cardano/address.{Address, Script}
use cardano/assets
use cardano/transaction.{Input, NoDatum, Output, placeholder} as eval_tx_module

const eval_authority: VerificationKeyHash =
  #"11111111111111111111111111111111111111111111111111111111"
const eval_attacker: VerificationKeyHash =
  #"99999999999999999999999999999999999999999999999999999999"
const eval_script_hash: ByteArray =
  #"aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"

fn eval_own_ref() -> OutputReference {
  OutputReference {
    transaction_id: #"abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1",
    output_index: 0,
  }
}

fn eval_own_address() -> Address {
  Address {
    payment_credential: Script(eval_script_hash),
    stake_credential: None,
  }
}

fn eval_in_state(counter: Int) -> StateDatum {
  StateDatum { authority: eval_authority, counter: counter }
}

fn eval_continuing_output(state: StateDatum) -> Output {
  Output {
    address: eval_own_address(),
    value: assets.zero,
    datum: eval_tx_module.InlineDatum(state),
    reference_script: None,
  }
}

fn eval_own_input() -> Input {
  Input {
    output_reference: eval_own_ref(),
    output: Output {
      address: eval_own_address(),
      value: assets.zero,
      datum: NoDatum,
      reference_script: None,
    },
  }
}

fn eval_tx(
  signers: List<VerificationKeyHash>,
  outputs: List<Output>,
) -> Transaction {
  Transaction {
    ..eval_tx_module.placeholder,
    inputs: [eval_own_input()],
    outputs: outputs,
    extra_signatories: signers,
  }
}

test eval_authority_signs_counter_incs_passes() {
  state_machine.spend(
    Some(eval_in_state(0)),
    Void,
    eval_own_ref(),
    eval_tx(
      [eval_authority],
      [
        eval_continuing_output(
          StateDatum { authority: eval_authority, counter: 1 },
        ),
      ],
    ),
  )
}

test eval_authority_signs_counter_wrong_fails() fail {
  state_machine.spend(
    Some(eval_in_state(0)),
    Void,
    eval_own_ref(),
    eval_tx(
      [eval_authority],
      [
        eval_continuing_output(
          StateDatum { authority: eval_authority, counter: 5 },
        ),
      ],
    ),
  )
}

// The vulnerability: authority is changed between input and output.
// Vulnerable code passes because counter is +1 and authority signs.
// Fixed code must catch the authority change and reject.
test eval_authority_changed_fails() fail {
  state_machine.spend(
    Some(eval_in_state(0)),
    Void,
    eval_own_ref(),
    eval_tx(
      [eval_authority],
      [
        eval_continuing_output(
          StateDatum { authority: eval_attacker, counter: 1 },
        ),
      ],
    ),
  )
}

test eval_stranger_signs_fails() fail {
  state_machine.spend(
    Some(eval_in_state(0)),
    Void,
    eval_own_ref(),
    eval_tx(
      [eval_attacker],
      [
        eval_continuing_output(
          StateDatum { authority: eval_authority, counter: 1 },
        ),
      ],
    ),
  )
}

test eval_no_continuing_output_fails() fail {
  state_machine.spend(
    Some(eval_in_state(0)),
    Void,
    eval_own_ref(),
    eval_tx([eval_authority], []),
  )
}

test eval_missing_datum_fails() fail {
  state_machine.spend(
    None,
    Void,
    eval_own_ref(),
    eval_tx(
      [eval_authority],
      [
        eval_continuing_output(
          StateDatum { authority: eval_authority, counter: 1 },
        ),
      ],
    ),
  )
}
