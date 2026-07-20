# brown-nft-burn — anti-pattern checklist

| # | Item |
|---|------|
| C1 | Validator signature unchanged: `one_shot_nft(utxo_ref, token_name)` with `mint(_r, policy_id, tx)` |
| C2 | Existing mint behavior preserved (positive qty requires param consumed + qty == 1 + correct name) |
| C3 | Burn case allows qty == -1 of correct name WITHOUT param check |
| C4 | Rejects burn qty != -1 and burn of wrong name |
| C5 | No-op (qty == 0) rejected |
| A1 | No invented redeemer dispatch (the existing signature has no typed redeemer; quantity-based dispatch is required) |
| A2 | Uses `assets.tokens` / `assets.flatten` / `assets.quantity_of` correctly (no destructuring `Value`) |
| H1 | All stdlib imports valid |
| S1 | No `todo` placeholders |
