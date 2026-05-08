#!/usr/bin/env python3
"""
LLM-as-judge rubric scorer.

Scores every saved final lib.rs across both modes (oneshot, agentic),
both treatments (direct, design_first), and all 3 runs per cell.

Each call is independent: a fresh `claude --print` with empty system prompt,
no tools, in a clean cwd. The judge sees only the task spec and the code —
no treatment label, no run number, no metadata. Scoring is on four 0..3
dimensions defined in RUBRIC.

Outputs:
  generations/<mode>/<task>/<treatment>/run_<N>/judge_prompt.txt
  generations/<mode>/<task>/<treatment>/run_<N>/judge_response.txt
  generations/<mode>/<task>/<treatment>/run_<N>/judge.json
  results/judge.csv
"""

from __future__ import annotations

import argparse
import csv
import json
import random
import re
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path

EXP_ROOT = Path(__file__).resolve().parent.parent
TASKS = ["scripting", "job_queue", "roman", "bank_ledger", "levenshtein"]
TREATMENTS = ["direct", "design_first"]
MODES = ["oneshot", "agentic"]
MODEL = "claude-opus-4-7"

JUDGE_CWD = Path("/tmp/types-first-judge")

RUBRIC = """\
SCORING RUBRIC

Score the code on four dimensions, each integer from 0 to 3. Be calibrated
and strict — most production code is not 3/3. Score the code as written,
not what you imagine it could become.

== type_expressiveness ==
How well does the code encode the domain in Rust's type system?
- 0: Heavy stringly-typed or raw-primitive use where a domain type fits
     (String error messages instead of typed variants, raw u64 for IDs,
     booleans for multi-state things).
- 1: Some types but inconsistent — typed where convenient, untyped where lazy.
- 2: Mostly typed: enums for discrete states, newtypes for IDs; some
     opportunities still missed.
- 3: Thorough domain modeling; every meaningful distinction has a type;
     no avoidable string-typed or boolean-tagged data.

== error_handling ==
How disciplined is error handling?
- 0: Library code uses unwrap(), panic!, or unimplemented! for situations
     that should produce errors.
- 1: Returns Result, but error type is String or has uninformative variants.
- 2: Typed error enum with distinguishing variants; library mostly avoids
     panicking; occasional unwrap/expect without documented reason.
- 3: Typed error enum with variants that meaningfully distinguish failure
     modes; library never panics; any expect/unwrap has a documented reason.

== api_ergonomics ==
How polished is the public API?
- 0: Public surface leaks internals; missing common derives; no Display
     where useful; no doc comments on public items.
- 1: API works but choices are awkward (e.g., Box<dyn Error>, inconsistent
     naming, sparse derives).
- 2: Clean public surface; sensible derives; doc comments on most public items.
- 3: Polished — narrow public surface, complete derives, every public item
     documented, naming consistent with std conventions.

== idiomaticity ==
Does this read like idiomatic Rust?
- 0: Java-in-Rust. Manual loops where iterators fit; no ? operator; no
     derives where they apply; aggressive cloning instead of borrowing.
- 1: Some idioms but inconsistent. Knows Result/Option but doesn't chain well.
- 2: Generally idiomatic. Uses ?, iterator methods, derives; some
     non-idiomatic choices remain.
- 3: Reads like idiomatic Rust written by an experienced practitioner.
     matches! where appropriate, expressive iterators, consistent error
     handling, no unnecessary clones."""

JUDGE_PROMPT = """You are an expert Rust reviewer scoring a single library implementation against a spec.

The code implements this spec:

---
{spec}
---

The code:

```rust
{code}
```

{rubric}

Output a single JSON object only — no prose, no markdown fences. The JSON must have exactly these keys with integer scores 0..3:

{{
  "type_expressiveness": {{"score": <int>, "justification": "<one sentence>"}},
  "error_handling":      {{"score": <int>, "justification": "<one sentence>"}},
  "api_ergonomics":      {{"score": <int>, "justification": "<one sentence>"}},
  "idiomaticity":        {{"score": <int>, "justification": "<one sentence>"}}
}}"""

JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


@dataclass
class JudgeResult:
    mode: str
    task: str
    treatment: str
    run: int
    type_expressiveness: int
    error_handling: int
    api_ergonomics: int
    idiomaticity: int
    total: int
    justifications: dict = field(default_factory=dict)


def render_prompt(task: str, code: str) -> str:
    spec = (EXP_ROOT / "tasks" / task / "spec.md").read_text().strip()
    return JUDGE_PROMPT.format(spec=spec, code=code, rubric=RUBRIC)


def call_judge(prompt: str, timeout_s: int = 300) -> str:
    JUDGE_CWD.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [
            "claude", "--print",
            "--system-prompt", "",
            "--tools", "",
            "--no-session-persistence",
            "--disable-slash-commands",
            "--permission-mode", "bypassPermissions",
            "--model", MODEL,
            "--output-format", "text",
            prompt,
        ],
        cwd=JUDGE_CWD,
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"judge call rc={proc.returncode}: {proc.stderr[-500:]}")
    return proc.stdout


def parse_response(response: str) -> tuple[dict, dict] | None:
    m = JSON_BLOCK.search(response)
    if not m:
        return None
    try:
        d = json.loads(m.group(0))
    except json.JSONDecodeError:
        return None
    try:
        scores = {
            k: int(d[k]["score"])
            for k in ("type_expressiveness", "error_handling", "api_ergonomics", "idiomaticity")
        }
        justs = {k: str(d[k].get("justification", "")) for k in scores}
    except (KeyError, ValueError, TypeError):
        return None
    return scores, justs


def code_path(mode: str, task: str, treatment: str, run: int) -> Path:
    base = EXP_ROOT / "generations" / mode / task / treatment / f"run_{run}"
    return base / ("final_lib.rs" if mode == "agentic" else "lib.rs")


def judge_path(mode: str, task: str, treatment: str, run: int) -> Path:
    base = EXP_ROOT / "generations" / mode / task / treatment / f"run_{run}"
    return base / "judge.json"


def score_one(mode: str, task: str, treatment: str, run: int, dry_run: bool) -> JudgeResult | None:
    cp = code_path(mode, task, treatment, run)
    if not cp.exists():
        return None
    code = cp.read_text()
    if not code.strip():
        return None

    cell = cp.parent
    prompt = render_prompt(task, code)
    (cell / "judge_prompt.txt").write_text(prompt)

    if dry_run:
        return None

    print(f"  -> {mode}/{task}/{treatment}/run_{run}: judging...", flush=True)
    started = time.time()
    response = call_judge(prompt)
    (cell / "judge_response.txt").write_text(response)
    parsed = parse_response(response)
    if parsed is None:
        print(f"     ! could not parse judge response", file=sys.stderr)
        return None
    scores, justs = parsed
    result = JudgeResult(
        mode=mode, task=task, treatment=treatment, run=run,
        **scores,
        total=sum(scores.values()),
        justifications=justs,
    )
    (cell / "judge.json").write_text(json.dumps(asdict(result), indent=2))
    print(
        f"     T={result.type_expressiveness} E={result.error_handling} "
        f"A={result.api_ergonomics} I={result.idiomaticity} "
        f"sum={result.total} ({time.time() - started:.0f}s)"
    )
    return result


def write_csv(results: list[JudgeResult]) -> Path:
    out = EXP_ROOT / "results" / "judge.csv"
    out.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "mode", "task", "treatment", "run",
        "type_expressiveness", "error_handling", "api_ergonomics", "idiomaticity", "total",
    ]
    with out.open("w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        for r in results:
            row = {k: getattr(r, k) for k in fieldnames}
            w.writerow(row)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=MODES + ["all"], default="all")
    ap.add_argument("--task", choices=TASKS + ["all"], default="all")
    ap.add_argument("--treatment", choices=TREATMENTS + ["all"], default="all")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true")
    args = ap.parse_args()

    modes = MODES if args.mode == "all" else [args.mode]
    tasks = TASKS if args.task == "all" else [args.task]
    treatments = TREATMENTS if args.treatment == "all" else [args.treatment]

    if not args.dry_run:
        from shutil import which
        if which("claude") is None:
            print("ERROR: 'claude' CLI not on PATH", file=sys.stderr)
            return 1

    # Build randomized work list (reduce systematic-order effects)
    work: list[tuple[str, str, str, int]] = []
    for mode in modes:
        for task in tasks:
            for tr in treatments:
                for r in range(args.runs):
                    if not args.force and judge_path(mode, task, tr, r).exists():
                        continue
                    if not code_path(mode, task, tr, r).exists():
                        continue
                    work.append((mode, task, tr, r))
    random.seed(42)
    random.shuffle(work)

    print(f"Judging {len(work)} file(s) (skipping {(len(modes)*len(tasks)*len(treatments)*args.runs) - len(work)} already done or missing)")

    results: list[JudgeResult] = []
    for mode, task, tr, run in work:
        try:
            r = score_one(mode, task, tr, run, args.dry_run)
            if r is not None:
                results.append(r)
        except Exception as e:
            print(f"     ! {type(e).__name__}: {e}", file=sys.stderr)

    # Reload all available scores so the CSV has the full picture
    all_results: list[JudgeResult] = []
    for mode in modes:
        for task in tasks:
            for tr in treatments:
                for r in range(args.runs):
                    p = judge_path(mode, task, tr, r)
                    if not p.exists():
                        continue
                    d = json.loads(p.read_text())
                    all_results.append(JudgeResult(**d))

    if all_results and not args.dry_run:
        path = write_csv(all_results)
        print(f"\nResults: {path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
