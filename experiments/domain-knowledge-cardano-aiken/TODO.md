# TODO / handoff — domain-knowledge primer experiment

A short handoff for resuming this work after a break. Skim this file, then read whichever section matches what you want to do next.

---

## Where we are

The original plan (from the first conversation) was:

1. Survey prior work — **done**
2. Research format + write the primer + run an A/B — **done**, validated
3. Generalize into a reusable skill for any niche domain — **done** (`SKILL_PROMPT.md`)

Three A/B experiments have run on this directory, all single-shot, no iteration:

| Experiment | Model | doc-off | doc-on | Notes |
|---|---|---|---|---|
| Greenfield (write from scratch) | Sonnet 4.6 | 0 / 17 tests | 17 / 17 | Dominant failure: stdlib drift (`aiken/list` → `aiken/collection/list`). |
| Greenfield | Opus 4.7 | 11 / 17 | 17 / 17 | Knows current stdlib paths; tripped on `Interval<Int>` (non-generic in Aiken). |
| Brownfield (extend / fix existing code) | Opus 4.7 | 41 / 42 | 42 / 42 | Surface signal nearly tied. Doc's value shifts to *latent* defects: doc-off uses `expect` for soft-casts where `if..is` is required (§5.1 / §5.2 anti-pattern). Verified via one probe test. |

Bottom line conclusion: the doc reliably prevents compile-time hallucination on a niche language, and it consistently teaches idioms that survive unanticipated input shapes. On a strong baseline model (Opus) doing brownfield work, the marginal value is qualitative robustness rather than test-pass uplift.

Full per-task narrative in `eval/RESULTS.md`.

---

## What's still open

Listed in rough priority order. Each item references files / commands you'd touch.

### High-value next moves

- **Run with iteration enabled** for the brownfield audit-double-sat task to test whether self-correction closes the latent-defect gap. Current eval forbids `aiken check` in the sub-agent. Removing that constraint asks: would doc-off agents notice their `expect` would crash if they could see test results? Likely yes for the obvious test, but the foreign-datum probe might still fail because the test suite has to include the foreign-datum case for it to surface.

- **Try the `SKILL_PROMPT.md` on a second domain** to validate the procedure generalizes. Candidate domains: Roc lang web framework, Rust embedded HAL (STM32 specifically — niche peripheral docs), the Aiken governance subset that this primer doesn't cover. Picking the second domain is itself a design decision; pick something where you can author 2-3 compile-checkable tasks.

- **Format ablation** — hold content constant, vary the doc structure. The current layered format (preamble → mental model → structure → syntax → example → anti-patterns → stdlib appendix) was a hypothesis, not a measurement. Compare against Q&A-only, anti-pattern-first, or bullets-only formats on the same tasks. Bounded by Opus already at 17/17 ceiling on greenfield — would need to either weaken to Sonnet or move to harder tasks.

### Robustness items

- N = 1 per cell across all experiments. Variance unknown. Re-running each cell 3-5 times with different seeds would establish whether the qualitative findings are stable or seed-noise.
- No Haiku 4.5 run. Cheap to add; would amplify doc uplift signal since Haiku has older training cutoff than Opus.
- Audit tasks gave the agent the vuln description in the prompt. A genuine *find-the-bug-blind* eval would test a different capability.

### Cleanup / nice-to-have

- The OUTLINE.md is a mid-process artifact; could be deleted or archived.
- The `eval/run-arm.sh` harness doesn't yet handle the case where a sub-agent writes a multi-line `use` statement — single-line is assumed. Hasn't bitten yet but could.

---

## How to pick up

### Prerequisites

```bash
aiken --version    # should print v1.1.22 or newer
python3 --version  # any 3.x
```

If `aiken` isn't installed, the toolchain is at `~/.aiken/bin/aiken` (managed by `aikup`). Reinstall with:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://install.aiken-lang.org | sh
aikup install
```

### Re-run an existing eval cell

The harness takes a task name and an arm name:

```bash
cd /home/kpi/dev/learnings/experiments/domain-knowledge-cardano-aiken/eval

./run-arm.sh vesting reference
./run-arm.sh vesting sonnet-doc-on
./run-arm.sh audit-double-sat doc-on
```

It assembles `submissions/<arm>.ak + tests.ak` into the project's `validators/main.ak` (with smart import merging via `merge-imports.py`), runs `aiken check`, and writes results to `tasks/<task>/results/<arm>/`. Read `check.txt` for human-readable output and `meta.txt` for exit code summary.

### Run a fresh sub-agent on an existing task

The exact prompt template for spawning a doc-on sub-agent is in the conversation transcript (search for `HARD CONSTRAINTS`). Key elements:

- Use the `Agent` tool with `subagent_type: "general-purpose"` and `model: "opus"` or `"sonnet"`.
- The agent reads `domain_knowledge.md` (doc-on only) and the task's `PROMPT.md`.
- Override the write path to `tasks/<task>/submissions/<arm>.ak`.
- Forbid network tools, compile commands, and directory exploration.

After the agent writes its submission, call `./run-arm.sh <task> <arm>` to score it.

### Add a new task

1. `aiken new eval/<task-name>` from `eval/tasks/`. Delete the generated `validators/placeholder.ak`.
2. Write `PROMPT.md` (task description, exact interface, write-path).
3. Write `tests.ak` (a hidden test suite — appended to the submission at harness time).
4. Write `checklist.md` (anti-pattern items to score against qualitatively).
5. Write `submissions/reference.ak` (your known-good solution).
6. `./run-arm.sh <task> reference` — **this must pass before spawning sub-agents**. If the reference fails, your tests are buggy and any A/B result will be meaningless.
7. Spawn doc-on and doc-off sub-agents pointing at the new task. Then `./run-arm.sh <task> doc-on` and `./run-arm.sh <task> doc-off`.

### Update the primer

`domain_knowledge.md` is what gets injected into doc-on agents. Edit it and re-run the entire A/B to measure the effect.

`RESEARCH.md` is raw source material; if Aiken or stdlib evolves, refresh `RESEARCH.md` first and then update `domain_knowledge.md` from it.

---

## Reference

### File map

```
domain_knowledge.md          The primer being tested (999 lines).
RESEARCH.md                  Raw source material distilled from aiken-lang.org + IOG cardano-mentor.
OUTLINE.md                   Mid-process design artifact; safe to archive.
SKILL_PROMPT.md              Generalized 7-phase procedure for producing one of these for any niche domain.
                             Install path: ~/.claude/skills/domain-bootstrap/SKILL.md (untested as a CC skill).

eval/
  RESULTS.md                 Full writeup of all three experiments with caveats. Read this first.
  run-arm.sh                 Harness: assemble + aiken check + capture, per (task, arm).
  merge-imports.py           Smart Aiken `use`-statement merger called by run-arm.sh.
  tasks/
    vesting/                 Greenfield: spend validator with time-bound beneficiary.
    one-shot-nft/            Greenfield: minting policy with one-shot UTxO consumption.
    multi-sig/               Greenfield: K-of-N spending validator.
    brown-vesting-arbiter/   Brownfield: extend vesting with arbiter escape hatch.
    brown-nft-burn/          Brownfield: extend one-shot NFT with burning.
    brown-multisig-freeze/   Brownfield: extend multi-sig with time-freeze.
    audit-double-sat/        Audit-fix: tag outputs to prevent double satisfaction.
    audit-loose-datum/       Audit-fix: reject None datum.
    audit-datum-continuity/  Audit-fix: enforce immutable field across state transitions.

  tasks/<task>/
    PROMPT.md                Sub-agent prompt (same for both arms; doc-on adds doc-read instruction).
    tests.ak                 Hidden tests appended at harness time.
    checklist.md             Anti-pattern items for qualitative scoring.
    submissions/             reference.ak + <arm>.ak files written by sub-agents.
    results/<arm>/           check.txt, check.err, assembled.ak, meta.txt.
```

### Aiken toolchain gotchas (rediscovered the hard way)

- **`aiken check` writes errors only to a TTY.** Plain stderr redirection silently swallows compile errors. The harness uses `script -qefc "aiken check" /dev/null` to allocate a pseudo-TTY, then strips ANSI codes.
- **Cross-file validator references don't work** (v1.1.22 + stdlib v3.1.0). Validators must live in the same `.ak` file as their tests. The harness concatenates submission + tests with smart `use`-statement merging.
- **Aiken `Interval` is not generic.** Write `Interval`, not `Interval<Int>`. The doc lists this in §6.4; doc-off agents trip on it.
- **Negative tests need the `fail` modifier.** A validator that rejects via `expect [Pair(...)]` *crashes* rather than returns False — `!validator(...)` doesn't catch that. Use `test foo() fail { body }` for any test that expects rejection.
- **`is_entirely_after(I, X)` is strict.** Interval `[1000, +∞)` is **not** entirely after 1000. Use `is_entirely_after(range, lock_until - 1)` for "lower bound >= lock_until" semantics.
- **Stdlib v3 reorganized modules.** `aiken/list` → `aiken/collection/list`. LLMs trained on older stdlib hallucinate the old paths.

### Memory pointers

These memory files exist on this machine and will be visible to future Claude Code sessions in this project:

- `~/.claude/projects/-home-kpi-dev-learnings/memory/project-domain-knowledge-docs.md` — meta-project tracker.
- `~/.claude/projects/-home-kpi-dev-learnings/memory/reference-aiken-toolchain-gotchas.md` — Aiken-specific quirks captured during this work.
- `~/.claude/projects/-home-kpi-dev-learnings/memory/domain-cardano-aiken.md` — user-domain memory (you work on Cardano/Aiken).
- `~/.claude/projects/-home-kpi-dev-learnings/memory/feedback-no-intermediate-review.md` — don't pause for outline review once an approach is agreed.
