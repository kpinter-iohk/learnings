# Types-First Experiment

Comparing Rust code generation quality between two prompting strategies:

- **Direct.** Implement the task.
- **Design-first.** Step 1: design types and signatures with `todo!()` bodies. Step 2: implement.

## Layout

```
tasks/{scripting,job_queue,roman}/
  spec.md      task spec given to Claude verbatim (replaces <<TASK_SPEC>> in the prompt)
  tests.rs     pre-written integration tests (copied into each generation's eval workspace)

prompts/
  direct.txt           treatment A template
  design_first.txt     treatment B template

runner/run.py          end-to-end runner: prompt → API → extract → cargo build/test/clippy → CSV

build/{task}/          per-task cargo workspace, target/ shared across runs
generations/{task}/{treatment}/run_{N}/   prompt, response, lib.rs, metrics.json
results/results.csv    aggregate output
```

## Running

The runner shells out to the `claude` CLI in headless mode (`--print`) with maximum-isolation flags: empty system prompt, no tools, no session persistence, no slash commands, model pinned to `claude-opus-4-7`. Each call runs from `/tmp/types-first-runs/` so no project files or `CLAUDE.md` leak into the model's context.

```sh
# dry run — render all prompts but don't call the model
python3 runner/run.py --dry-run

# full sweep: 3 tasks × 2 treatments × 3 runs = 18 generations
python3 runner/run.py

# subset
python3 runner/run.py --task roman --treatment direct --runs 1

# already-completed cells are skipped; --force re-runs them
python3 runner/run.py --force
```

### Methodology caveats vs. direct API

We're going through the CLI (not the raw API) because direct API access wasn't available. This means:
- Temperature is whatever the CLI defaults to (not exposed as a flag).
- The CLI may inject a small amount of context the API call wouldn't (we mitigate with `--system-prompt ""`, `--tools ""`, and a clean cwd).

Both treatments are subject to the same conditions, so the comparison remains valid; the absolute numbers may differ from a pure-API run.

## Hypothesis (in brief)

- **B1 — Behavioral correctness.** Design-first passes more tests on average. Confounded with token budget.
- **B2 — Type-system usage in implementation.** Design-first produces richer types (more enums, more newtypes, fewer unwraps) even after stripping the design preamble. Tested on the implementation alone.
- **D1–D4** — Compile rate, cyclomatic complexity, run-to-run variance, clippy count: descriptive only.

See git history for the full design discussion.

## Metrics produced

Per generation (`metrics.json`, also a row in `results.csv`):

- `compile_ok` — `cargo build --lib` returncode == 0
- `tests_passed` / `tests_total` — parsed from `cargo test` summary line
- `clippy_warnings` — count of warning-level diagnostics from `cargo clippy --lib -- -W clippy::pedantic`
- `loc` — non-empty lines in the generated `lib.rs`
- `unwrap_count` / `panic_count` / `todo_count` / `unsafe_count` — regex counts in `lib.rs`
- `response_chars` / `code_chars` — full response length and extracted-code length
- `duration_s` — wall-clock for the whole cell (API call + cargo runs)

The B2 rubric (type richness, error discipline, idiomaticity) is not computed here — that's a second-pass blinded scoring step that runs against the saved `lib.rs` files after generation.
