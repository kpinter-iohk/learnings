# audit-datum-continuity — anti-pattern checklist

| # | Item |
|---|------|
| V1 | Vulnerability identified: `out_state.authority` not compared to `in_state.authority` |
| F1 | Fix adds explicit equality check on `authority` between input state and continuing-output state |
| F2 | Counter increment check (`out_state.counter == in_state.counter + 1`) preserved |
| F3 | Authority-signature check preserved |
| C1 | `StateDatum` and `state_machine` validator name unchanged |
| H1 | All stdlib imports valid |
| S1 | No `todo` placeholders |
