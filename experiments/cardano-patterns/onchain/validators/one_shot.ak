use aiken/collection/list
use cardano/transaction.{OutputReference, Transaction}

validator one_shot(seed: OutputReference) {
  mint(_redeemer: Data, _policy_id: ByteArray, self: Transaction) {
    list.any(self.inputs, fn(input) { input.output_reference == seed })
  }

  else(_) {
    fail
  }
}

test seed_is_consumed() {
  let seed =
    OutputReference {
      transaction_id: #"0000000000000000000000000000000000000000000000000000000000000001",
      output_index: 0,
    }

  let other =
    OutputReference {
      transaction_id: #"0000000000000000000000000000000000000000000000000000000000000002",
      output_index: 0,
    }

  expect [seed, other] |> list.any(fn(r) { r == seed })
  !([other] |> list.any(fn(r) { r == seed }))
}
