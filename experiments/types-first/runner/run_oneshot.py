#!/usr/bin/env python3
"""
Oneshot runner: single `claude --print` call per cell, no tools, no iteration.

Each cell:
  1. Render the appropriate <treatment>_oneshot.txt prompt with the task spec.
  2. Call claude with empty system prompt, no tools, in /tmp.
  3. Extract the largest rust code block as lib.rs.
  4. Run cargo build / test / clippy in the eval workspace.
  5. Save metrics.json.

Outputs land in:
  generations/oneshot/<task>/<treatment>/run_<N>/
    prompt.txt, response.txt, lib.rs, metrics.json
  results/oneshot.csv
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from run import (
    EXP_ROOT, MODEL, TASKS, TREATMENTS,
    code_metrics, evaluate_code, setup_eval_workspace,
)

CLI_CWD = Path("/tmp/types-first-oneshot")
CODE_FENCE = re.compile(r"```(?:rust|rs)?\s*\n(.*?)```", re.DOTALL)


@dataclass
class Result:
    task: str
    treatment: str
    run: int
    compile_ok: bool
    tests_passed: int
    tests_total: int
    clippy_warnings: int | None
    loc: int
    unwrap_count: int
    panic_count: int
    todo_count: int
    unsafe_count: int
    response_chars: int
    code_chars: int
    duration_s: float


def render_prompt(treatment: str, task: str) -> str:
    template = (EXP_ROOT / "prompts" / f"{treatment}_oneshot.txt").read_text()
    spec = (EXP_ROOT / "tasks" / task / "spec.md").read_text()
    return template.replace("<<TASK_SPEC>>", spec.strip())


def call_claude(prompt: str, timeout_s: int = 600) -> str:
    CLI_CWD.mkdir(parents=True, exist_ok=True)
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
        cwd=CLI_CWD,
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"claude rc={proc.returncode}: {proc.stderr[-500:]}")
    return proc.stdout


def extract_code(response: str) -> str | None:
    blocks = CODE_FENCE.findall(response)
    if not blocks:
        return None
    largest = max(blocks, key=len).strip()
    return largest + "\n" if largest else None


def gen_dir(task: str, tr: str, run: int) -> Path:
    return EXP_ROOT / "generations" / "oneshot" / task / tr / f"run_{run}"


def already_done(task: str, tr: str, run: int) -> bool:
    return (gen_dir(task, tr, run) / "metrics.json").exists()


def run_one(task: str, tr: str, run: int, dry_run: bool) -> Result | None:
    d = gen_dir(task, tr, run)
    d.mkdir(parents=True, exist_ok=True)

    prompt = render_prompt(tr, task)
    (d / "prompt.txt").write_text(prompt)

    if dry_run:
        print(f"  [dry-run] {task}/{tr}/run_{run}: {len(prompt)} chars")
        return None

    started = time.time()
    print(f"  -> {task}/{tr}/run_{run}: calling claude...", flush=True)
    response = call_claude(prompt)
    duration = time.time() - started
    (d / "response.txt").write_text(response)

    code = extract_code(response)
    if code is None:
        print("     ! no code block found")
        result = Result(
            task=task, treatment=tr, run=run,
            compile_ok=False, tests_passed=0, tests_total=0,
            clippy_warnings=None, loc=0,
            unwrap_count=0, panic_count=0, todo_count=0, unsafe_count=0,
            response_chars=len(response), code_chars=0,
            duration_s=duration,
        )
    else:
        (d / "lib.rs").write_text(code)
        ev = evaluate_code(task, code)
        cm = code_metrics(code)
        result = Result(
            task=task, treatment=tr, run=run,
            compile_ok=ev.compile_ok,
            tests_passed=ev.tests_passed, tests_total=ev.tests_total,
            clippy_warnings=ev.clippy_warnings,
            loc=cm.loc, unwrap_count=cm.unwrap_count,
            panic_count=cm.panic_count, todo_count=cm.todo_count,
            unsafe_count=cm.unsafe_count,
            response_chars=len(response), code_chars=len(code),
            duration_s=duration,
        )

    (d / "metrics.json").write_text(json.dumps(asdict(result), indent=2))
    print(
        f"     compile={result.compile_ok} "
        f"tests={result.tests_passed}/{result.tests_total} "
        f"clippy={result.clippy_warnings} loc={result.loc} ({result.duration_s:.0f}s)"
    )
    return result


def write_csv(results: list[Result]) -> Path:
    out = EXP_ROOT / "results" / "oneshot.csv"
    out.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = list(asdict(results[0]).keys()) if results else []
    with out.open("w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        for r in results:
            w.writerow(asdict(r))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--task", choices=TASKS + ["all"], default="all")
    ap.add_argument("--treatment", choices=TREATMENTS + ["all"], default="all")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true")
    args = ap.parse_args()

    tasks = TASKS if args.task == "all" else [args.task]
    treatments = TREATMENTS if args.treatment == "all" else [args.treatment]

    if not args.dry_run:
        from shutil import which
        if which("claude") is None:
            print("ERROR: 'claude' CLI not on PATH", file=sys.stderr)
            return 1

    total = len(tasks) * len(treatments) * args.runs
    print(f"Oneshot sweep: {total} cell(s)")

    results: list[Result] = []
    for task in tasks:
        print(f"\n== {task} ==")
        if not args.dry_run:
            setup_eval_workspace(task)
        for tr in treatments:
            for r in range(args.runs):
                if not args.force and already_done(task, tr, r):
                    print(f"  skip {task}/{tr}/run_{r}")
                    cached = json.loads(
                        (gen_dir(task, tr, r) / "metrics.json").read_text()
                    )
                    results.append(Result(**cached))
                    continue
                try:
                    res = run_one(task, tr, r, args.dry_run)
                    if res is not None:
                        results.append(res)
                except Exception as e:
                    print(f"     ! {type(e).__name__}: {e}", file=sys.stderr)

    if results and not args.dry_run:
        path = write_csv(results)
        print(f"\nResults: {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
