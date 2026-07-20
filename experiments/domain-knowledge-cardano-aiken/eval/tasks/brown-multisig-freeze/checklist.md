# brown-multisig-freeze — anti-pattern checklist

| # | Item |
|---|------|
| C1 | `MultiSigDatum` extended with `frozen_until: Int`; existing fields preserved |
| C2 | Validator name and signature unchanged |
| C3 | Both threshold AND time-freeze must pass — AND-combined, not OR |
| A1 | Uses `aiken/interval` (`is_entirely_after` or equivalent) — not raw bound destructuring |
| A2 | Time direction correct (lower bound of validity_range >= frozen_until) |
| C4 | Datum `Option` extraction preserved |
| H1 | All stdlib imports valid |
| S1 | No `todo` placeholders |
