use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use cardano/address
use cardano/assets
use cardano/transaction.{InlineDatum, OutputReference, Transaction}

pub type SwapDatum {
  beneficiary: VerificationKeyHash,
  price: Int,
}

validator swap {
  spend(
    datum_opt: Option<SwapDatum>,
    _redeemer: Data,
    own_ref: OutputReference,
    self: Transaction,
  ) {
    expect Some(datum) = datum_opt
    let beneficiary_address = address.from_verification_key(datum.beneficiary)
    let user_outputs_restricted =
      list.filter(
        self.outputs,
        fn(output) {
          when output.datum is {
            InlineDatum(output_datum) ->
              if output_datum is tag: OutputReference {
                and {
                  output.address == beneficiary_address,
                  own_ref == tag,
                }
              } else {
                False
              }
            _ -> False
          }
        },
      )
    let total_paid =
      list.foldl(
        user_outputs_restricted,
        0,
        fn(o, total) { total + assets.lovelace_of(o.value) },
      )
    total_paid >= datum.price
  }
}
