use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/transaction.{InlineDatum, OutputReference, Transaction}

pub type StateDatum {
  authority: VerificationKeyHash,
  counter: Int,
}

validator state_machine {
  spend(
    datum_opt: Option<StateDatum>,
    _redeemer: Data,
    own_ref: OutputReference,
    tx: Transaction,
  ) {
    expect Some(in_state) = datum_opt
    let authority_signed = list.has(tx.extra_signatories, in_state.authority)

    expect Some(own_input) =
      list.find(tx.inputs, fn(i) { i.output_reference == own_ref })
    expect Some(out) =
      list.find(tx.outputs, fn(o) { o.address == own_input.output.address })
    expect InlineDatum(out_data) = out.datum
    expect out_state: StateDatum = out_data

    and {
      authority_signed?,
      (out_state.authority == in_state.authority)?,
      (out_state.counter == in_state.counter + 1)?,
    }
  }
}
