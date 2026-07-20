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
    let dedicated_outputs =
      list.filter(
        self.outputs,
        fn(o) {
          if o.address != beneficiary_address {
            False
          } else {
            when o.datum is {
              InlineDatum(d) -> {
                expect tagged_ref: OutputReference = d
                tagged_ref == own_ref
              }
              _ -> False
            }
          }
        },
      )
    let total_paid =
      list.foldl(
        dedicated_outputs,
        0,
        fn(o, total) { total + assets.lovelace_of(o.value) },
      )
    total_paid >= datum.price
  }
}
