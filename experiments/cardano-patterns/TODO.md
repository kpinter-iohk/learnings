# Cardano Patterns — Status & Resume Notes

Article + experiment exploring six smart-contract patterns specific to Cardano,
using Ethereum as a foil. Started 2026-05.

## File map

```
cardano-patterns.qmd                        article (repo root)
cardano-patterns.css                        styles
experiments/cardano-patterns/
  TODO.md                                   this file
  onchain/
    aiken.toml                              v1.1.21, Plutus V3, stdlib v3.1.0
    validators/one_shot.ak                  pattern 1: validator + unit test
    plutus.json                             compiled blueprint
  offchain/
    Cargo.toml                              cardano-serialization-lib 13.2.1
    src/bin/one_shot_mint.rs                pattern 1: real CSL tx builder
```

## Where we are

Pattern 1 (one-shot minting) is done end-to-end:

- Aiken validator compiles, unit test passes (`aiken check` in `onchain/`).
- Rust tx builder compiles and runs (`cargo run --bin one_shot_mint` in `offchain/`).
  - Prints policy id `5c8559f2…46cd0` (matches Aiken's hash).
  - Produces a fully-balanced tx body CBOR.
- Article section renders cleanly (intro, Problem / Solution / Where used,
  Aiken code, pseudo-Rust off-chain, one-liner essence, Ethereum foil).

Patterns 2–6 are not written. The "What's next" section at the end of the
article lists them.

## Locked decisions

Carry these forward; do not relitigate without reason.

- **Tone.** Prose with flow, not telegraph. Sentences connect. Antecedents
  concrete. Open with motivating frame ("We want to X…"). The fix-up in
  Pattern 1's Problem and Solution is the reference.
- **No jargon at first use.** Concept first, name second. Glossary at end of
  article maps plain words to standard terms.
- **Audience.** Beginners to both Ethereum and Cardano. Not Cardano devs.
- **Section structure per pattern.** Problem → Solution → Where used →
  on-chain Aiken → off-chain pseudo-Rust → "in essence" pseudocode →
  Ethereum foil. Each pattern follows this skeleton.
- **Aiken code.** Real, compiles via `aiken check`, lives under
  `onchain/validators/<pattern>.ak`. At least one unit test per validator.
- **Off-chain.** Pseudo-Rust in the article (clean, no `csl::` noise, no
  `unwrap`, no `BigNum::from`). Real CSL code under
  `offchain/src/bin/<pattern>.rs` for verification. Article has a callout
  establishing this convention at the top of Pattern 1's off-chain section.
- **Ethereum foil.** Short, prose-mostly, sometimes absent. Framed as
  "what each Cardano trick recovers; what Ethereum pays for not needing it."
- **Widget convention.** Single-line HTML comment in the qmd, grep-able:
  `<!-- WIDGET_IDEA — label — concept it teaches — mechanic sketch -->`.
  Discuss before building any widget. The first attempt at a checkbox-toggle
  widget for Pattern 1 was removed as underwhelming.

## Remaining patterns (in pedagogical order)

Problem / Solution / Where-used text for all six is locked in the conversation
history. Re-use verbatim when writing each section.

2. **State-thread token.** Updatable on-chain state without storage slots. Uses
   one-shot mint to create a unique stamp; require it to travel on every state
   update. Authenticity check = stamp presence.
3. **Forwarding token-creation rules.** Minting policy that delegates entirely
   to a spending script that runs in the same transaction. Both scripts share
   transaction fate.
4. **Cooperative arguments (Leader / Follower).** Avoid N×N script work when N
   coins at one address are spent together. One coin's redeemer is `Leader`
   and does the full check; the others are `Follower` and just confirm a
   leader exists.
5. **Withdraw-zero.** Use the staking system as a once-per-transaction hook.
   Register a script as a staking address; withdraw 0 ADA in the transaction;
   the staking script fires exactly once.
6. **Reference scripts (CIP-33).** Point at a script's bytecode instead of
   inlining it on every transaction. Bytecode lives in one UTxO; every
   future tx references it.

## Open widget markers

```
grep -n WIDGET_IDEA cardano-patterns.qmd
```

Current markers (Pattern 1):

- *Seed lifecycle* — three-state ledger view (before / during / after the
  mint) showing why "one-shot" is structural, not enforced.
- *Parameter → policy ID* — seed picker, live bytecode and policy id.
  Reinforces that two different seeds give two different policies.

Both deferred for discussion before any building.

## How to resume

1. Read the current article. `quarto preview cardano-patterns.qmd --port 4444`
   then open <http://localhost:4444/cardano-patterns.html>.
2. Verify experiments still compile.
   ```
   cd experiments/cardano-patterns/onchain && aiken check
   cd experiments/cardano-patterns/offchain && cargo check
   ```
3. Pick the next pattern (start with #2 — state-thread token).
4. Per-pattern recipe:
   - Write Aiken validator at `onchain/validators/<pattern>.ak` with a unit
     test; run `aiken check` and `aiken build`.
   - Write Rust tx builder at `offchain/src/bin/<pattern>.rs`; verify
     `cargo run --bin <pattern>` works end-to-end.
   - Add the article section: Problem / Solution / Where used (locked text)
     → Aiken code → pseudo-Rust excerpts → "in essence" pseudocode →
     Ethereum foil.
   - Drop `WIDGET_IDEA` markers wherever visualization would clarify
     something prose can't. Do not build widgets.
   - Re-render with `quarto render cardano-patterns.qmd`.

## Gotchas worth keeping

- `aiken blueprint apply` is interactive — it needs piped input
  (`echo "" | aiken blueprint apply ...`) or a pty (`script -c '...' /dev/null`).
  Without one of these, it exits 1 with no output and no error.
- The Aiken validator/module names for `--module`/`--validator` flags are the
  *short* names (`one_shot` not `one_shot.one_shot.mint`).
- In CSL `TransactionBuilder`, call `calc_script_data_hash` **before**
  `add_change_if_needed`, not after — otherwise the change calculation
  doesn't include the script-data-hash size and the fee comes up short.
- CSL needs `TxBuilderConstants::plutus_conway_cost_models()` for Plutus V3.
  The default (`plutus_default_cost_models` → vasil) does not include V3 and
  panics with "Missing cost model for language version: Language(PlutusV3)".
- Plutus mints require a collateral input (a non-script UTxO the chain can
  seize if the script fails). Without one, CSL refuses to build.

## Process notes

- The locked Problem/Solution/Where-used text for the six patterns took
  several rewrites to land — the failure mode was telegraph-style prose.
  Read Pattern 1 in the rendered article for the reference tone.
- The user prefers exploration and decisions before building. When in
  doubt about scope, format, or framing, surface the choice and ask.
- The user is sophisticated; do not pad explanations. But "no fluff" does
  not mean "fragmented sentences" — prose has to flow.
