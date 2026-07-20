# Vesting task — anti-pattern checklist

Score each item as PASS (avoided the trap), FAIL (committed the trap), or N/A. Cite the line(s) in `submissions/<arm>.ak` that demonstrate.

## Core correctness

| # | Item | Reference |
|---|------|-----------|
| C1 | Validator named `vesting` and exposes a `spend` handler matching the signature | §2.1, §2.2 |
| C2 | `datum_opt` parameter typed as `Option<VestingDatum>`, not `VestingDatum` | §1.4, §2.3 |
| C3 | Handles missing-datum case (either `expect Some(d) = datum_opt` or explicit `when` match) | §2.3 |

## Domain-specific anti-patterns

| # | Item | Reference |
|---|------|-----------|
| A1 | Does NOT treat datum as `VestingDatum` directly (must extract from `Option`) | §1.4, §5.2 |
| A2 | Uses `aiken/interval` for time-bound check, not arithmetic on raw bound fields | §1.5, §6.4 |
| A3 | Time check uses semantically correct direction (lower-bound of validity_range vs lock_until) | §5.9 |
| A4 | Owner check is OR-combined with (beneficiary AND time), not AND-combined | task spec |
| A5 | Signatory check uses `list.has(tx.extra_signatories, key)`, not invented function | §6.2 |
| A6 | No integer arithmetic with raw POSIX time except for bounds — no float type used | §5.8 |
| A7 | No reentrancy, mutex, or account-balance assumptions | §1.1 |
| A8 | No use of Plutus-Tx syntax (`PubKeyHash`, `txInfoSignatories`, `mustBeSignedBy`) | §0 |

## Stdlib hallucination

| # | Item | Reference |
|---|------|-----------|
| H1 | All imported modules exist in stdlib §6 | §6 |
| H2 | All called functions exist on their imported module | §6 |
| H3 | No invented helper functions assumed from another library (vodka, mesh) without import | §0 |

## Style / hygiene

| # | Item | Reference |
|---|------|-----------|
| S1 | Uses `?` operator on final boolean to enable failure tracing | §3.7 |
| S2 | No `trace` left in production-style code on a hot path | §5.4 |
| S3 | No `todo` placeholders | §3.7 |

## Scoring summary

Pass rate = (passed items) / (applicable items).
