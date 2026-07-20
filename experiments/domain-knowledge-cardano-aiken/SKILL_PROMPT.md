# Skill: bootstrap domain knowledge primer for a niche domain

This is the generalized procedure for producing a `domain_knowledge.md` for any niche programming domain — meant to be injected into a sub-agent to bootstrap a better mental model. Validated on Cardano/Aiken (see `eval/RESULTS.md` in this directory): single-shot Sonnet 4.6 went from 0 / 17 tests passing without the doc to 17 / 17 with it.

To install as a Claude Code skill: drop this file as `SKILL.md` inside `~/.claude/skills/domain-bootstrap/` (or any directory of your choice).

---

## When to invoke

Use this skill when:
- The target domain has known thin training coverage (recent language, niche framework, specialized DSL).
- You expect to reuse the resulting doc across multiple sub-agent sessions.
- You can articulate a small set of compile-checkable or behavior-checkable tasks to validate the doc against (~3 tasks is enough for a clean signal).

Do **not** use this skill for:
- Domains already well-covered in training data (popular languages, mainstream frameworks).
- Conceptual / discursive domains where there is no objective correctness signal (rules-of-thumb advice, philosophy of design).

## Inputs

Ask the user for:
1. **Domain name** (e.g., "Cardano smart contract design in Aiken", "rust embedded HAL for STM32", "Roc lang web framework basics").
2. **Concrete task class** the resulting sub-agents will perform (e.g., "writing validators", "writing kernel drivers"). One sentence.
3. **Authoritative sources** for the domain — official docs URL, canonical repo, key blog posts. If user doesn't know, surface them via search.
4. **Toolchain availability** — is there a compiler or test runner that lets us verify outputs behaviorally? (Strongly preferred over LLM-judge eval.)

## Procedure

### Phase 1 — Prior work survey (skip if you've done it for this domain)

Spawn one general-purpose research agent. Brief:
- Find any existing LLM-context docs for the domain (llms.txt, AGENTS.md, .cursorrules, CLAUDE.md, community skill files).
- Find any documented LLM failure modes specific to the domain — blog posts, issue trackers, "LLM-keeps-doing-X" complaints.
- Report under 400 words. If a substantial existing doc already exists and seems strong, stop here and use it.

### Phase 2 — Gather source material

Spawn one general-purpose research agent. Brief:
- Visit each authoritative source URL.
- Distill into a single `RESEARCH.md` file with these sections:
  - **Mental model** — what makes this domain conceptually distinct from adjacent domains the LLM might confuse it with.
  - **Syntax cheatsheet** — exact syntax forms with one-line examples, focused on what differs from cousin languages.
  - **API / library anchor list** — the actual module/function/symbol names that exist. This is anti-hallucination grounding. Be exhaustive for the most-used 5–10 modules.
  - **Validator-structure-equivalent** — if the domain has a particular code structure (script structure, plugin shape, etc.), document it here.
  - **Anti-pattern catalog** — each entry with name, mistake, why-it's-wrong, fix. Source: official "common pitfalls" docs, FAQ pages, security writeups.
  - **Worked example** — one complete unit of code (not a snippet) with annotations.
- Output as `RESEARCH.md` in the experiment directory. Cite every block.

### Phase 3 — Write `domain_knowledge.md`

Use a layered format (validated on Cardano/Aiken):

```
0. How to use this document (preamble — ~25 lines)
1. Mental model (~80 lines)
2. Code structure (~100 lines)
3. Syntax cheatsheet (~120 lines)
4. Worked example (~80 lines)
5. Anti-pattern catalog (~250 lines)
6. API / library anchor list (~80 lines, appendix)
A. Toolchain / project layout (~40 lines, appendix)
```

Target: 600–1000 lines. Distill ~50% from RESEARCH.md verbatim quotes to directive prose.

Voice rules:
- Declarative, no hedging. "X is Y", not "X is generally Y in most cases."
- Code blocks first-class — show the right pattern, then explain.
- Each anti-pattern: wrong code → why → right code.
- Section 0 must tell the sub-agent how to use the doc explicitly: "verify-before-recall for X", "do not import patterns from Y", "if you use a function not in §6, stop."

### Phase 4 — Build the eval harness

Three tasks is enough for a clean signal. For each task:
- A `PROMPT.md` describing the expected behavior + the exact public interface (type names, function names) so external tests can reference them.
- A `tests.ak` (or task-appropriate test file) with 5–6 tests covering the behavior. Include positive AND negative cases. Use the toolchain's "expected failure" annotation for negative cases that may legitimately crash rather than return False.
- A `checklist.md` of anti-pattern items to score against — derived directly from the anti-pattern catalog (§5 of the doc).
- A project scaffold (e.g., `aiken new`, `cargo new`, etc.) ready for a sub-agent's code to be dropped in.

Critical harness lesson: write a *reference solution* and run the tests against it before spawning sub-agents. If tests are buggy, the experiment is meaningless. Verify the reference passes all tests first.

### Phase 5 — Spawn the A/B

Six sub-agents in parallel (3 tasks × 2 conditions):
- **Doc-on:** prompt instructs the agent to read `domain_knowledge.md` first.
- **Doc-off:** prompt does not mention the doc; agent uses prior knowledge.

Hard constraints for both arms (failure to enforce contaminates the experiment):
- No web tools (WebSearch, WebFetch).
- No compile/test commands during the run.
- Read only the task prompt (+ doc, for doc-on).
- Write the validator once. Stop. No iteration.

Use the same model for both arms — Sonnet 4.6 was the validated choice; smaller models will show bigger uplift but noisier results.

### Phase 6 — Score

For each submission:
1. Run the toolchain's check command, capture exit code and structured output.
2. Count tests passed / failed.
3. Score against the anti-pattern checklist — PASS / FAIL / N/A per item.
4. Note any *qualitative* differences (e.g., the doc-on output may use idioms the doc-off output does not).

Tabulate doc-on vs doc-off across all three axes.

### Phase 7 — Report

Produce a `RESULTS.md` with:
- Top-line table (compile rate, test pass rate, token cost per arm).
- Per-task narrative.
- Failure mode analysis (which sections of the doc were load-bearing).
- Explicit limitations (N=1 per cell, single model, single-shot, etc.).
- "What to do next" — robustness checks, format ablation, larger task coverage.

## Pitfalls observed

- **Stdlib drift is the #1 failure mode for niche languages.** The most load-bearing section of the doc is the API anchor list. If a stdlib was reorganized post-training-cutoff, doc-off agents will hallucinate the old paths.
- **Aiken-specific:** `aiken check` writes diagnostic errors only to a TTY-capable target. Redirecting stderr swallows them silently. Use `script -qefc "aiken check" /dev/null` in your harness, then strip ANSI codes with `sed`.
- **Cross-file validator references don't work in Aiken** (as of v1.1.22 / stdlib v3.1.0). Concatenate validator + tests into a single `.ak` file with a `use` deduplication pass.
- **Negative tests need `test foo() fail { body }`.** If the validator rejects by crashing (`expect [Pair(...)]` fails to match), `!validator(...)` doesn't capture it.
- **Reference solution validation is non-negotiable.** Skipping it once will burn the eval.

## What this skill does NOT cover

- **Format ablation.** The layered format above is validated; alternatives (bullets-only, Q&A, anti-pattern-first) are untested.
- **Multi-model robustness.** Validated on Sonnet 4.6 only. Older or smaller models likely benefit more; stronger models may already pass without the doc.
- **Iteration enabled.** Single-shot only. Whether the doc still helps when agents can self-correct via compile feedback is open.
- **Domains without compile/run feedback.** This procedure leans heavily on objective verification. For domains where you can only judge outputs subjectively, the eval design needs different scaffolding (rubric + LLM-as-judge, with known judge-shares-training-gap caveat).
