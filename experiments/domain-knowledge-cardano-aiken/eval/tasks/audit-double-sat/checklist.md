# audit-double-sat — anti-pattern checklist

| # | Item |
|---|------|
| V1 | Vulnerability identified: outputs are filtered by address only, not by per-input tag |
| F1 | Fix tags each beneficiary output with the spending input's `OutputReference` via inline datum |
| F2 | Fix uses `if..is`/`when..is` to soft-cast output datum (NOT `expect`) — foreign outputs may have other datum shapes |
| F3 | `own_ref` parameter is actually used (no longer prefixed `_`) |
| C1 | `SwapDatum` and `swap` validator name unchanged |
| H1 | All stdlib imports valid |
| S1 | No `todo` placeholders |
