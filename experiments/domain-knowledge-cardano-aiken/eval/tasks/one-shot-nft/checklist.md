# One-shot NFT task — anti-pattern checklist

Score each item as PASS (avoided the trap), FAIL (committed the trap), or N/A. Cite the line(s) in `submissions/<arm>.ak` that demonstrate.

## Core correctness

| # | Item | Reference |
|---|------|-----------|
| C1 | Validator parameterized by `(utxo_ref: OutputReference, token_name: AssetName)` | §2.5 |
| C2 | Has a `mint` handler with signature `mint(redeemer: ?, policy_id: PolicyId, self: Transaction)` | §2.2 |
| C3 | Mint handler does NOT declare a datum parameter | §1.4, §5.3 |

## Domain-specific anti-patterns

| # | Item | Reference |
|---|------|-----------|
| A1 | Checks parameter UTxO is in `tx.inputs` (via `list.any` or equivalent) | §5.6 |
| A2 | Inspects `tx.mint` using `assets` stdlib functions, not by destructuring `Value` | §6.10 |
| A3 | Verifies mint quantity equals exactly 1 for `token_name` | task spec |
| A4 | Verifies NO other asset names are minted under this policy | task spec |
| A5 | Does not allow burn (negative quantity) | task spec |
| A6 | Treats `Value` as opaque — uses `tokens`, `quantity_of`, `from_asset`, etc. — does not pattern-match `Value` constructors | §6.10 |
| A7 | No reentrancy, mutex, or global-counter assumptions | §1.1 |
| A8 | No use of Plutus-Tx idioms (`txInfoMint`, `valueOf`, `singleton`) | §0 |

## Stdlib hallucination

| # | Item | Reference |
|---|------|-----------|
| H1 | All imported modules exist in stdlib §6 | §6 |
| H2 | All called functions exist on their imported module | §6 |
| H3 | No invented helpers from external libraries without explicit import | §0 |
| H4 | Uses `assets.tokens` or `assets.quantity_of` correctly | §6.10 |

## Style / hygiene

| # | Item | Reference |
|---|------|-----------|
| S1 | Uses `?` operator on conditions to enable failure tracing | §3.7 |
| S2 | No `todo` placeholders | §3.7 |
| S3 | Validator parameters used (not ignored) | task spec |

## Scoring summary

Pass rate = (passed items) / (applicable items).
