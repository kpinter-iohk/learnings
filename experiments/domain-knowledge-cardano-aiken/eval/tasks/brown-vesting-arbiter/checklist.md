# brown-vesting-arbiter — anti-pattern checklist

| # | Item |
|---|------|
| C1 | `VestingDatum` extended with `arbiter: VerificationKeyHash` field; existing 3 fields preserved with same names/types |
| C2 | Validator name `vesting`, spend handler signature unchanged |
| C3 | Arbiter check is OR-combined with existing owner/beneficiary logic, not gated by any other condition |
| C4 | Datum `Option` extraction preserved (`expect Some(d)` or equivalent) |
| H1 | All stdlib imports remain valid (no module-path or function-name hallucinations) |
| S1 | No `todo` placeholders |
