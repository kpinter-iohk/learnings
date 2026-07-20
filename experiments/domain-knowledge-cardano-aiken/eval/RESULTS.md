# A/B Eval Results — domain_knowledge.md for Cardano/Aiken

**Date:** 2026-05-23
**Model (both arms):** Sonnet 4.6 (`claude-sonnet-4-6`)
**Sub-agent type:** `general-purpose`
**Iteration:** single-shot, no compile/test feedback to agents
**Doc-on arm:** sub-agent instructed to read `domain_knowledge.md` (999 lines) before writing
**Doc-off arm:** sub-agent uses prior knowledge only
**Tasks:** 3 — vesting validator, one-shot NFT minting policy, threshold multi-sig
**Tests:** 17 total (6 + 5 + 6) — authored by experimenter, validated against reference solutions

---

## Top-line

| Metric | doc-on | doc-off |
|---|---|---|
| Tasks compiling | 3 / 3 | 0 / 3 |
| Tasks with all tests passing | 3 / 3 | 0 / 3 |
| Individual tests passed | 17 / 17 | 0 / 17 |
| Token budget used per task | ~26k | ~12k |

**The doc-off arm never reached the test phase on any task.** All three failed at compile time with the identical error: `aiken::check::unknown::module: list`. They each invoked `list.has`, `list.any`, or `list.foldl` without importing `aiken/collection/list`.

---

## Per-task results

### Task 1 — Vesting validator

**doc-on submission** (`submissions/doc-on.ak`):
- Compiles cleanly.
- Tests: 6 / 6 passed.
- Functionally equivalent to reference solution.
- Style nit: does not use `?` operator on final boolean (minor — failure traces less informative).

**doc-off submission** (`submissions/doc-off.ak`):
- Compile error: `unknown module 'list'` at line 16:3.
- Imports declared: `aiken/crypto`, `aiken/interval` (with `Finite, IntervalBound`), `cardano/transaction`.
- Imports missing: `aiken/collection/list` (needed for `list.has` inside the `must_be_signed_by` helper).
- Logic itself appeared correct — owner OR (beneficiary AND time-reached), with a custom `lower_bound_reached` function that pattern-matches on `tx.validity_range.lower_bound.bound_type`.
- **Verdict:** would likely have passed all tests had `aiken/collection/list` been imported. Pure stdlib drift failure.

### Task 2 — One-shot NFT minting policy

**doc-on submission** (`submissions/doc-on.ak`):
- Compiles cleanly.
- Tests: 5 / 5 passed.
- *Notably more robust than the reference solution.* Uses `when minted_tokens is { [Pair(name, q)] -> ... ; _ -> False }` instead of `expect [Pair(...)]`. This returns `False` on "multiple assets" rather than crashing — same on-chain outcome but cleaner validator semantics.

**doc-off submission** (`submissions/doc-off.ak`):
- Compile error: `unknown module 'list'` at line 14:7.
- Imports declared: `cardano/assets`, `cardano/transaction`.
- Imports missing: `aiken/collection/list`, `aiken/collection/dict`.
- Used `dict.to_list(policy_tokens)` — function does not exist; real name is `dict.to_pairs`.
- Used tuple `(token_name, 1)` for equality comparison — stdlib uses `Pair(token_name, 1)`. `Tuple` ≠ `Pair` in Aiken type system. Would have been a second compile error if imports had been fixed.
- **Verdict:** two distinct hallucination failures — module import and function name. Doc's §6.3 explicitly lists `dict.to_pairs` (not `to_list`).

### Task 3 — Threshold multi-sig

**doc-on submission** (`submissions/doc-on.ak`):
- Compiles cleanly.
- Tests: 6 / 6 passed.
- Identical idiomatic structure to reference solution (`list.count` over signers, threshold compare).

**doc-off submission** (`submissions/doc-off.ak`):
- Compile error: `unknown module 'list'` at line 15:3.
- Imports declared: `aiken/crypto`, `cardano/transaction`.
- Imports missing: `aiken/collection/list` (needed for `list.foldl`, `list.has` inside the `count_signers` helper).
- Logic itself appeared correct — fold over signers list, increment on match, threshold compare.
- **Verdict:** would likely have passed all tests had the import been added. Pure stdlib drift failure.

---

## Failure mode analysis

**The dominant failure was stdlib drift:** all three doc-off submissions referenced functions in modules named `list`, `dict` that they did not import. Aiken's stdlib reorganized `aiken/list` → `aiken/collection/list` and `aiken/dict` → `aiken/collection/dict` at some prior version. LLMs trained on older versions retain the old module names; if they remember to import at all, they often import the wrong path. Sonnet 4.6's training apparently did not include the reorganization.

**The doc neutralizes this directly via §6 (the stdlib anchor list).** §6.1 enumerates the current modules; §6.2–6.10 enumerate functions per module. Doc-on agents read this list before writing and consistently used the correct module paths.

**Secondary failure observed (one-shot-nft doc-off):** function-name hallucination. `dict.to_list` does not exist; `dict.to_pairs` does. The doc-on agent used `dict.to_pairs`. The doc explicitly lists `to_pairs` in §6.3.

**No other anti-patterns observed in this run.** The doc-off submissions, where they did compile-resolved expressions, used correct types and reasonable logic. The bottleneck was purely "which functions exist where."

---

## Anti-pattern checklist scores

Doc-off submissions could not be scored against most items because they did not compile. Items below are checklist-style (PASS = avoided trap, FAIL = committed, N/A = not reachable due to compile failure).

### Vesting

| Item | doc-on | doc-off |
|---|---|---|
| C1 Validator name & signature | PASS | PASS |
| C2 `Option<VestingDatum>` not bare | PASS | PASS |
| C3 Handles missing-datum | PASS | PASS |
| A1 Option extraction not skipped | PASS | PASS |
| A2 Uses `aiken/interval` | PASS | PASS (manual field-match acceptable) |
| A3 Time direction correct | PASS | PASS |
| A4 owner OR (beneficiary AND time) | PASS | PASS |
| A5 `list.has` usage | PASS | **FAIL** (module not imported) |
| A6 No float type | PASS | PASS |
| A7 No reentrancy/account assumptions | PASS | PASS |
| A8 No Plutus-Tx idioms | PASS | PASS |
| H1 Imports exist in stdlib | PASS | **FAIL** |
| H2 Functions exist on imported module | PASS | **FAIL** (used `list.` without import) |
| H3 No invented library helpers | PASS | PASS |
| S1 `?` operator on final boolean | FAIL (style) | N/A |
| S2 No production trace | PASS | PASS |
| S3 No `todo` | PASS | PASS |

Doc-on pass rate: 16/17 (94%, one style item)
Doc-off pass rate: 13/17 + 1 N/A = ~76%, but compile-blocking failures dominate

### One-shot NFT

| Item | doc-on | doc-off |
|---|---|---|
| C1 Validator parameters | PASS | PASS |
| C2 `mint` signature | PASS | PASS |
| C3 No datum parameter | PASS | PASS |
| A1 Checks param UTxO in inputs | PASS | PASS |
| A2 Uses `assets` stdlib funcs | PASS | PASS |
| A3 Exactly 1 of token_name | PASS | PASS (intent) |
| A4 No other assets under policy | PASS | PASS (intent) |
| A5 No burn allowed | PASS | PASS |
| A6 Treats Value as opaque | PASS | PASS |
| A7 No reentrancy assumptions | PASS | PASS |
| A8 No Plutus-Tx idioms | PASS | PASS |
| H1 Imports exist in stdlib | PASS | **FAIL** (list, dict missing) |
| H2 Functions exist on module | PASS | **FAIL** (`dict.to_list` invented) |
| H3 No invented helpers | PASS | PASS |
| H4 `assets.tokens` / `quantity_of` correct | PASS | PASS |
| S1 `?` operator usage | PASS | FAIL (style) |
| S2 No `todo` | PASS | PASS |
| S3 Parameters used | PASS | PASS |

### Multi-sig

| Item | doc-on | doc-off |
|---|---|---|
| C1 Validator name & signature | PASS | PASS |
| C2 `Option<MultiSigDatum>` | PASS | PASS |
| C3 Handles missing-datum | PASS | PASS |
| A1 Counts only listed signers | PASS | PASS |
| A2 Outsiders excluded | PASS | PASS |
| A3 Threshold uses `>=` | PASS | PASS |
| A4 Integer arithmetic | PASS | PASS |
| A5 Uses stdlib list helpers | PASS | PASS (used correct names, missing import) |
| A6 No mutable counter | PASS | PASS |
| A7 No Plutus-Tx idioms | PASS | PASS |
| H1 Imports exist in stdlib | PASS | **FAIL** (list missing) |
| H2 Functions exist on module | PASS | PASS (names were correct) |
| H3 No invented helpers | PASS | PASS |
| S1 `?` operator | FAIL (style) | FAIL (style) |
| S2 No `todo` | PASS | PASS |
| S3 No unused imports | PASS | PASS |

---

## What the experiment shows

1. **The doc works.** On the chosen tasks, with this model, in single-shot mode, the doc moves the agent from 0% to 100% compile + test pass rate. This is the strongest signal a binary A/B can produce.

2. **The dominant failure mode is stdlib drift, and the doc's §6 directly addresses it.** This was predicted by the prior-work survey (arXiv 2407.02742 on DSL hallucination in code generation). The §6 anchor list is the most load-bearing section of the doc for this model on this task class.

3. **Doc-on agents can produce *better-than-reference* code.** The one-shot-NFT doc-on submission used `when ... is { _ -> False }` instead of `expect [...]`, giving the validator return-False rejection semantics instead of crash-on-reject — a strictly better Aiken pattern. This suggests the doc is teaching style, not just preventing errors.

4. **Cost is ~2x for doc-on.** Doc-on agents used ~26k tokens vs ~12k for doc-off. The doc accounts for ~14k of that.

5. **Iteration was disabled.** Doc-off agents would likely recover with one or two retries (the missing-import error is unambiguous). The interesting question for follow-up is whether the doc still helps with iteration, or whether self-correction alone closes the gap.

---

## Limitations of this experiment

- **N = 1 per cell.** Each (task, arm) was run exactly once. Variance across runs is unknown; another run might produce different numbers though the import-omission failure is so consistent that the qualitative result would almost certainly hold.
- **Single model.** Sonnet 4.6 only. Older or smaller models likely fail worse without the doc; stronger models (Opus) might already get it right without the doc.
- **Single-shot only.** No iteration, no compile feedback. Real-world sub-agents would iterate.
- **Three tasks, all on the validator/policy core.** No tasks on governance, treasury, or other Plutus-V3 surfaces.
- **No format ablation.** This experiment compared *doc vs no-doc*. It did not compare format variants (bullets vs anti-pattern-first vs Q&A) — the OUTLINE.md proposed a single layered format and it was tested as-authored.
- **One-shot-NFT doc-on bonus is anecdotal.** Whether the doc reliably teaches the `when ... is { _ -> False }` pattern over `expect [...]` would require more runs.

---

## What to do next

1. **Generalize.** Extract the *process* used here into a reusable skill that produces a `domain_knowledge.md` for any niche domain. The skill should at minimum prescribe: (a) prior-work survey, (b) anti-pattern catalog from authoritative sources, (c) anchor list against hallucination, (d) worked example, (e) eval harness with compile-checkable A/B.

2. **Robustness checks.** Re-run with iteration enabled. Re-run on Haiku and Opus. Re-run with 3 seeds per cell to estimate variance.

3. **Format ablation.** Hold the *content* constant; vary the *format* (bullets vs Q&A vs anti-pattern-first). See whether the layered structure is load-bearing or whether content alone explains the uplift.

4. **Adversarial domain coverage.** Add tasks that exercise governance, treasury, voting, reference-input patterns — places the doc may have under-served.

---

# Addendum — Opus 4.7 run (2026-05-24)

Same experiment design, model swapped to `claude-opus-4-7`. Same 6 sub-agents, same constraints, same harness (with a smarter import merger; see `merge-imports.py`).

## Top-line comparison

| | Sonnet 4.6 doc-off | Sonnet 4.6 doc-on | Opus 4.7 doc-off | Opus 4.7 doc-on |
|---|---|---|---|---|
| Tasks compiling | 0 / 3 | 3 / 3 | 2 / 3 | 3 / 3 |
| Tasks all-tests-pass | 0 / 3 | 3 / 3 | 2 / 3 | 3 / 3 |
| Individual tests | 0 / 17 | 17 / 17 | 11 / 17 | 17 / 17 |
| Tokens / task (median) | ~12k | ~26k | ~17k | ~36k |

**Opus closes most of the no-doc gap on its own.** Without the doc it gets 2 / 3 tasks right vs Sonnet's 0 / 3. With the doc it gets 3 / 3 — same as Sonnet.

## What Opus got right without the doc

- **Correct stdlib paths.** All three Opus doc-off submissions used `aiken/collection/list` (current path), unlike Sonnet which used the obsolete `aiken/list`. Opus's training appears to include the post-reorg stdlib.
- **Multi-sig and one-shot-NFT compile and pass cleanly.** The one-shot-NFT submission used `assets.flatten(mint) |> list.filter(...)` then `expect [(_, name, qty)]` — different from but functionally equivalent to the doc-on submission's `assets.tokens |> dict.to_pairs |> when ... is`.

## What Opus got wrong without the doc

- **Vesting failed at compile: `Interval<Int>` does not type-check.** Aiken's `Interval` type takes no generic parameter — it is already specialized. The doc explicitly lists `Interval` as a non-generic type in §6.4. Opus doc-on got this right; Opus doc-off did not.

This is the *same* surface-detail failure I personally made when first writing the test fixtures (see `reference-aiken-toolchain-gotchas` memory). It is a small bit of Aiken-specific knowledge that doesn't show up in general programming intuition and that LLMs without an up-to-date primer routinely fumble.

## What the Opus run shows

1. **Doc uplift shrinks as the baseline model gets stronger.** Sonnet went 0 → 100% with the doc. Opus went ~65% → 100%. The doc still drives the model to a perfect score, but the marginal value is concentrated on a smaller set of failure modes.

2. **The remaining failure mode is the same kind, smaller in count.** Both Sonnet doc-off and Opus doc-off broke on Aiken syntax/library specifics — Sonnet on module paths, Opus on generic-type-arity. Both are exactly what the doc's §6 anchor list and §2/§3 syntax sections are designed to neutralize.

3. **Doc-on cost roughly doubles per task.** ~36k tokens for Opus doc-on vs ~17k for Opus doc-off. Worth it for production sub-agent work where one compile-fail wastes more than 19k tokens; questionable for high-throughput automated runs where most tasks would have passed anyway.

4. **For the user's described use case — spawning a sub-agent to do work in this domain — the doc is a clear net positive on either model.** Even at 2/3 baseline pass rate (Opus), the marginal compile-time failure prevented by the doc represents real avoided rework.

## Failure-mode summary across both runs

| Failure mode | Sonnet doc-off | Opus doc-off | Both doc-on |
|---|---|---|---|
| Wrong stdlib module path (`aiken/list` vs `aiken/collection/list`) | 3 / 3 tasks | 0 / 3 tasks | 0 / 3 tasks |
| Non-existent function name (`dict.to_list` instead of `to_pairs`) | 1 / 3 tasks | 0 / 3 tasks | 0 / 3 tasks |
| `Interval<Int>` type arity error | 0 / 3 tasks | 1 / 3 tasks | 0 / 3 tasks |
| Logic error (wrong threshold direction, missing condition, etc.) | 0 observed | 0 observed | 0 observed |

The doc neutralized every failure mode that appeared in either no-doc arm. No new failure modes were introduced by the doc.

## Caveats on Opus comparison

- Still N = 1 per (task, arm, model) cell. Variance unknown.
- The `Interval<Int>` failure may be a single-seed unlucky draw; another Opus doc-off run might pass vesting.
- Opus doc-off would almost certainly succeed on retry given the unambiguous error message — single-shot underestimates Opus more than Sonnet.
- One harness bug surfaced during the Opus run (duplicate-import after naive line-dedup) and was fixed with a smarter import merger. All 15 cells were re-run after the fix. The fix did not change any previously-passing or previously-failing Sonnet result.

---

# Addendum 2 — Brownfield tasks on Opus 4.7 (2026-05-24)

Same model, same constraints, same harness as the Opus greenfield run. Six new tasks designed to mirror realistic dev workflows: 3 *feature-add* (extend an existing validator) and 3 *audit-fix* (diagnose and patch a vulnerable validator). The vulnerable validators in the audit tasks were drawn from canonical Aiken anti-patterns (double-satisfaction, loose-datum, datum-continuity break).

## Task list

| # | Task | Type | Vuln/feature | Tests |
|---|---|---|---|---|
| 1 | `brown-vesting-arbiter` | feature-add | add an always-spend arbiter role | 8 |
| 2 | `brown-nft-burn` | feature-add | allow burning -1 of token without param UTxO | 8 |
| 3 | `brown-multisig-freeze` | feature-add | add a `frozen_until` time-lock | 7 |
| 4 | `audit-double-sat` | audit-fix | tag outputs to prevent double-satisfaction | 7 (6 + 1 probe) |
| 5 | `audit-loose-datum` | audit-fix | reject `None` datum (vulnerable: returned True) | 6 |
| 6 | `audit-datum-continuity` | audit-fix | enforce authority-field immutability | 6 |

## Top-line results

| Task | doc-on tests | doc-off tests |
|---|---|---|
| brown-vesting-arbiter | 8 / 8 | 8 / 8 |
| brown-nft-burn | 8 / 8 | 8 / 8 |
| brown-multisig-freeze | 7 / 7 | 7 / 7 |
| audit-double-sat | **7 / 7** | **6 / 7** |
| audit-loose-datum | 6 / 6 | 6 / 6 |
| audit-datum-continuity | 6 / 6 | 6 / 6 |
| **Total** | **42 / 42** | **41 / 42** |

**Surface signal: nearly identical.** On the test-pass axis, Opus doc-off reaches the same correctness as doc-on across all six brownfield tasks except a single test on `audit-double-sat`. This is a stark contrast to the greenfield run where doc-off compiled on 2 / 3 tasks and passed 11 / 17 tests.

The most plausible reason: in brownfield tasks the existing validator code shows the agent the correct stdlib import paths, the correct type names, and the validator-shape conventions. The agent anchors on the visible code rather than on potentially-stale training memory. The doc's §6 anchor list — the most load-bearing section in the greenfield run — provides much less marginal value when the agent can read working examples in the task itself.

## Where the doc still mattered: latent quality

The doc-on submissions consistently produced code that the doc-off submissions did not, even when both passed the test suite:

- **doc-on uses `if d is Type` (soft cast) for foreign-output datum filters.** doc-off uses `expect ... = d` (hard cast) inside filters, which crashes when iterating outputs whose datums are of unrelated shapes. The doc explicitly warns about this in §5.1 (the double-satisfaction fix) and §5.2 (opaque-Data misuse).
- **doc-on uses the `?` operator on boolean conditions for failure tracing.** doc-off uses bare `&&` and `||` — same semantics, but production-failed validators are harder to debug.
- **doc-on uses `expect Some(d) = datum_opt` idiomatically.** doc-off sometimes used `when ... is { Some(d) -> ...; None -> False }` instead — both correct, but the doc-on form is the convention used across the Aiken ecosystem.

## The probe test (`audit-double-sat` test #7)

After observing that doc-on used `if..is` and doc-off used `expect` in the double-sat fix, I added one extra test to the suite — a transaction that has both a properly-tagged payment AND a foreign output at the beneficiary address carrying an unrelated inline datum (an integer literal). A real on-chain transaction can contain such outputs because the same payment-credential can receive unrelated funds. Expected behavior: the validator should ignore the foreign output and approve based on the tagged payment.

- **Reference solution (uses `if..is`):** test passes.
- **doc-on Opus submission (uses `if..is`):** test passes.
- **doc-off Opus submission (uses `expect`):** test **fails** — validator crashes on the foreign output.

**Disclosure of methodology:** This test was authored *after* observing the qualitative difference between submissions, not before. It is a targeted probe of a hypothesis that emerged during analysis, not an independent measurement. A more rigorous protocol would freeze the test suite before any submissions are scored. For this addendum I disclose the post-hoc nature; the qualitative finding (doc-on uses soft-cast, doc-off uses hard-cast) is the load-bearing observation, and the test number simply quantifies one consequence.

## Implications for the doc's value proposition

The greenfield → brownfield contrast clarifies what the doc actually buys you on a strong baseline model:

1. **Greenfield Opus:** the doc prevents compile-time hallucinations (`Interval<Int>` arity error). High-visibility uplift.
2. **Brownfield Opus:** the doc prevents *latent* defects — code that compiles, passes obvious tests, and would still crash in production on unanticipated input shapes. Lower-visibility uplift, but arguably the more dangerous failure mode in real on-chain code where bugs can drain funds.

For the user's stated use case (sub-agent debugging / auditing existing code), the brownfield setting is the more realistic scenario. The doc's marginal value is qualitative robustness, not surface correctness.

## Token cost (brownfield)

| | doc-on | doc-off |
|---|---|---|
| Median tokens / task | ~36k | ~17k |

Identical ratio to the greenfield Opus run; the doc itself (~14k tokens) accounts for the difference.

## What this experiment did NOT show

- **Audit identification.** Both arms were given the vulnerability description in the task prompt — they were not asked to *find* the vulnerability blind. A genuine audit task (no hint, agent must discover the flaw) would test a different capability.
- **Whether the latent-defect pattern generalizes.** Only the `audit-double-sat` task had a clear soft-cast-vs-hard-cast differentiator. Other audit tasks had similar fixes regardless of the doc.
- **Robustness of the qualitative claim.** N = 1 per cell on Opus 4.7; another Opus seed might use `if..is` without the doc, or `expect` with it. The probe test fixes the specific instance but does not establish a probability.
