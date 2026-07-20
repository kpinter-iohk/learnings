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
    let tagged_outputs =
      list.filter(
        self.outputs,
        fn(o) {
          when o.datum is {
            InlineDatum(output_datum) ->
              if output_datum is tag_ref: OutputReference {
                and {
                  o.address == beneficiary_address,
                  tag_ref == own_ref,
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
        tagged_outputs,
        0,
        fn(o, total) { total + assets.lovelace_of(o.value) },
      )
    (total_paid >= datum.price)?
  }
}
