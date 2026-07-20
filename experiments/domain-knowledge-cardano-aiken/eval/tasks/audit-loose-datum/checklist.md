# audit-loose-datum — anti-pattern checklist

| # | Item |
|---|------|
| V1 | Vulnerability identified: `None -> True` branch (or equivalent) lets anyone spend without a datum |
| F1 | Fix rejects all missing-datum cases (either via `expect Some(d)` or explicit `None -> False`) |
| F2 | Fix preserves owner-signature requirement when datum is present |
| C1 | `VaultDatum` and `vault` validator name unchanged |
| H1 | All stdlib imports valid |
| S1 | No `todo` placeholders |
