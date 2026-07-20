# Multi-sig task — anti-pattern checklist

Score each item as PASS (avoided the trap), FAIL (committed the trap), or N/A. Cite the line(s) in `submissions/<arm>.ak` that demonstrate.

## Core correctness

| # | Item | Reference |
|---|------|-----------|
| C1 | Validator named `multi_sig` and exposes a `spend` handler matching the signature | §2.1, §2.2 |
| C2 | `datum_opt` parameter typed as `Option<MultiSigDatum>` | §1.4, §2.3 |
| C3 | Handles missing-datum case (either `expect Some(d) = datum_opt` or `when` match) | §2.3 |

## Domain-specific anti-patterns

| # | Item | Reference |
|---|------|-----------|
| A1 | Counts only signers that appear BOTH in datum's `signers` AND in `tx.extra_signatories` | task spec |
| A2 | Outsider signers (not in `signers`) do not increase the count | task spec |
| A3 | Threshold comparison uses `>=`, not `==` (i.e. accepts more than threshold) | task spec |
| A4 | Uses integer arithmetic (no float, no rational unless deliberately) | §5.8 |
| A5 | Uses stdlib `aiken/collection/list` functions (filter, count, foldl, etc.) | §6.2 |
| A6 | No mutable counter, no accumulator side-effects, no global state assumption | §1.3 |
| A7 | No Plutus-Tx idioms (`PubKeyHash`, `txSignedBy`, `mustBeSignedBy`) | §0 |

## Stdlib hallucination

| # | Item | Reference |
|---|------|-----------|
| H1 | All imported modules exist in stdlib §6 | §6 |
| H2 | All called functions exist on their imported module | §6 |
| H3 | No invented helpers without explicit import | §0 |

## Style / hygiene

| # | Item | Reference |
|---|------|-----------|
| S1 | Uses `?` operator on final boolean to enable failure tracing | §3.7 |
| S2 | No `todo` placeholders | §3.7 |
| S3 | No unused imports (warnings allowed but not errors) | — |

## Scoring summary

Pass rate = (passed items) / (applicable items).
