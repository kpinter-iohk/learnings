#!/usr/bin/env python3
"""
Agentic runner for the types-first Rust experiment.

Each cell:
  1. Creates a fresh per-cell cargo workspace at /tmp/types-first-runs/<cell>/
     containing Cargo.toml and an empty src/. No test file — the agent
     should not see our integration tests, only the spec.
  2. Runs `claude -p` with Bash, Edit, Read, Write enabled and stream-json
     output, prompting the agent to implement the spec and verify with
     cargo. The agent decides when to stop.
  3. Saves the full stream as transcript.ndjson.
  4. Parses the stream for: first Write to lib.rs (-> first_lib.rs), cargo
     invocation count, iterations (turns), final cost.
  5. Reads final src/lib.rs from the workspace -> final_lib.rs.
  6. Evaluates BOTH first_lib.rs and final_lib.rs against the pre-written
     test suite at tasks/<task>/tests.rs, in a separate eval workspace
     under build/eval/<task>/.

Outputs land in:
  generations/agentic/<task>/<treatment>/run_<N>/
    prompt.txt              prompt sent to the agent
    transcript.ndjson       full stream-json output
    first_lib.rs            agent's initial Write (if captured)
    final_lib.rs            workspace src/lib.rs at agent stop
    metrics.json            per-run metrics
  results/agentic.csv       aggregate
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import shutil
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path

EXP_ROOT = Path(__file__).resolve().parent.parent
TASKS = ["scripting", "job_queue", "roman", "bank_ledger", "levenshtein"]
TREATMENTS = ["direct", "design_first"]
DEFAULT_RUNS = 3
MODEL = "claude-opus-4-7"

AGENT_CWD_ROOT = Path("/tmp/types-first-runs")

CARGO_TOML_LIB = """\
[package]
name = "solution"
version = "0.0.1"
edition = "2021"

[lib]
path = "src/lib.rs"
"""

CARGO_TOML_EVAL = """\
[package]
name = "solution"
version = "0.0.1"
edition = "2021"

[lib]
path = "src/lib.rs"

[[test]]
name = "integration"
path = "tests/integration.rs"
"""

TEST_RESULT = re.compile(r"test result: (?:ok|FAILED)\.\s+(\d+) passed;\s+(\d+) failed")


@dataclass
class CodeMetrics:
    loc: int = 0
    unwrap_count: int = 0
    panic_count: int = 0
    todo_count: int = 0
    unsafe_count: int = 0


@dataclass
class EvalMetrics:
    compile_ok: bool = False
    tests_passed: int = 0
    tests_total: int = 0
    clippy_warnings: int | None = None


@dataclass
class Result:
    task: str
    treatment: str
    run: int
    # Final-pass metrics
    compile_ok: bool
    tests_passed: int
    tests_total: int
    clippy_warnings: int | None
    loc: int
    unwrap_count: int
    panic_count: int
    todo_count: int
    unsafe_count: int
    # First-pass metrics (the agent's initial Write before any iteration)
    first_pass_captured: bool
    first_pass_compile: bool
    first_pass_tests_passed: int
    first_pass_tests_total: int
    first_pass_loc: int
    # Process metrics
    iterations: int
    cargo_invocations: int
    agent_self_verified: bool
    total_cost_usd: float
    duration_s: float


# ---------- prompts ----------

def render_prompt(treatment: str, task: str) -> str:
    template = (EXP_ROOT / "prompts" / f"{treatment}_agentic.txt").read_text()
    spec = (EXP_ROOT / "tasks" / task / "spec.md").read_text()
    return template.replace("<<TASK_SPEC>>", spec.strip())


# ---------- agent workspace ----------

def cell_workspace(task: str, treatment: str, run: int) -> Path:
    return AGENT_CWD_ROOT / f"{task}_{treatment}_run_{run}"


def reset_agent_workspace(task: str, treatment: str, run: int) -> Path:
    ws = cell_workspace(task, treatment, run)
    if ws.exists():
        shutil.rmtree(ws)
    (ws / "src").mkdir(parents=True)
    (ws / "Cargo.toml").write_text(CARGO_TOML_LIB)
    return ws


# ---------- claude CLI ----------

def call_claude_agent(prompt: str, cwd: Path, timeout_s: int = 900) -> tuple[str, int]:
    """Run claude --print with tools enabled in cwd; return (stdout, returncode).

    Tools: Bash, Edit, Read, Write. No --system-prompt override (the agent
    needs the default system prompt to use tools effectively — that's part
    of what the experiment measures). No CLAUDE.md leaks because cwd is
    under /tmp/.
    """
    proc = subprocess.run(
        [
            "claude", "--print",
            "--tools", "Bash,Edit,Read,Write",
            "--permission-mode", "bypassPermissions",
            "--no-session-persistence",
            "--disable-slash-commands",
            "--model", MODEL,
            "--output-format", "stream-json",
            "--verbose",
            prompt,
        ],
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )
    return proc.stdout, proc.returncode


# ---------- transcript parsing ----------

def parse_transcript(stdout: str) -> dict:
    """Walk stream-json events; extract first-Write content and process stats."""
    first_lib_content: str | None = None
    cargo_invocations = 0
    num_turns = 0
    total_cost = 0.0
    error_msg: str | None = None

    for line in stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue

        if ev.get("type") == "assistant":
            for block in ev.get("message", {}).get("content", []):
                if block.get("type") != "tool_use":
                    continue
                name = block.get("name", "")
                inp = block.get("input", {}) or {}
                if name == "Write":
                    fp = inp.get("file_path", "") or ""
                    if "lib.rs" in fp and first_lib_content is None:
                        first_lib_content = inp.get("content")
                if name == "Bash":
                    cmd = (inp.get("command", "") or "").strip()
                    if cmd.startswith("cargo ") or cmd.startswith("cargo\t"):
                        cargo_invocations += 1
        elif ev.get("type") == "result":
            num_turns = int(ev.get("num_turns", 0) or 0)
            total_cost = float(ev.get("total_cost_usd", 0.0) or 0.0)
            if ev.get("is_error"):
                error_msg = ev.get("result") or "unknown error"

    return {
        "first_lib_content": first_lib_content,
        "cargo_invocations": cargo_invocations,
        "num_turns": num_turns,
        "total_cost_usd": total_cost,
        "error_msg": error_msg,
    }


# ---------- evaluation ----------

def setup_eval_workspace(task: str) -> Path:
    ws = EXP_ROOT / "build" / "eval" / task
    (ws / "src").mkdir(parents=True, exist_ok=True)
    (ws / "tests").mkdir(parents=True, exist_ok=True)
    (ws / "Cargo.toml").write_text(CARGO_TOML_EVAL)
    (ws / "tests" / "integration.rs").write_text(
        (EXP_ROOT / "tasks" / task / "tests.rs").read_text()
    )
    return ws


def run_cargo(args: list[str], cwd: Path, timeout: int = 300) -> tuple[int, str, str]:
    proc = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    return proc.returncode, proc.stdout, proc.stderr


def parse_test_counts(output: str) -> tuple[int, int]:
    passed = total = 0
    for m in TEST_RESULT.finditer(output):
        p, f = int(m.group(1)), int(m.group(2))
        passed += p
        total += p + f
    return passed, total


def count_clippy_warnings(json_output: str) -> int:
    n = 0
    for line in json_output.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("reason") != "compiler-message":
            continue
        if obj.get("message", {}).get("level") == "warning":
            n += 1
    return n


def evaluate_code(task: str, code: str) -> EvalMetrics:
    if not code or not code.strip():
        return EvalMetrics()
    ws = setup_eval_workspace(task)
    (ws / "src" / "lib.rs").write_text(code)
    m = EvalMetrics()
    rc, _, _ = run_cargo(["cargo", "build", "--lib", "--quiet"], ws)
    m.compile_ok = rc == 0
    if not m.compile_ok:
        return m
    try:
        rc, stdout, _ = run_cargo(
            ["cargo", "test", "--test", "integration", "--quiet", "--no-fail-fast"],
            ws, timeout=300,
        )
        m.tests_passed, m.tests_total = parse_test_counts(stdout)
    except subprocess.TimeoutExpired:
        pass
    rc, stdout, _ = run_cargo(
        ["cargo", "clippy", "--lib", "--quiet", "--message-format=json",
         "--", "-W", "clippy::pedantic"],
        ws,
    )
    m.clippy_warnings = count_clippy_warnings(stdout)
    return m


def code_metrics(code: str) -> CodeMetrics:
    if not code:
        return CodeMetrics()
    return CodeMetrics(
        loc=len([l for l in code.splitlines() if l.strip()]),
        unwrap_count=len(re.findall(r"\.unwrap\s*\(", code)),
        panic_count=len(re.findall(r"\bpanic\s*!", code)),
        todo_count=len(re.findall(r"\btodo\s*!", code)),
        unsafe_count=len(re.findall(r"\bunsafe\b", code)),
    )


# ---------- run loop ----------

def gen_dir(task: str, treatment: str, run: int) -> Path:
    return EXP_ROOT / "generations" / "agentic" / task / treatment / f"run_{run}"


def already_done(task: str, treatment: str, run: int) -> bool:
    return (gen_dir(task, treatment, run) / "metrics.json").exists()


def run_one(task: str, treatment: str, run: int, dry_run: bool) -> Result | None:
    cell_id = f"{task}/{treatment}/run_{run}"
    d = gen_dir(task, treatment, run)
    d.mkdir(parents=True, exist_ok=True)

    prompt = render_prompt(treatment, task)
    (d / "prompt.txt").write_text(prompt)

    if dry_run:
        print(f"  [dry-run] {cell_id}: prompt {len(prompt)} chars")
        return None

    ws = reset_agent_workspace(task, treatment, run)
    print(f"  -> {cell_id}: agent running in {ws}...", flush=True)

    started = time.time()
    stdout, rc = call_claude_agent(prompt, ws)
    duration = time.time() - started
    (d / "transcript.ndjson").write_text(stdout)

    if rc != 0:
        print(f"     ! claude CLI rc={rc}", file=sys.stderr)

    parsed = parse_transcript(stdout)

    # First-pass capture
    first_pass_captured = parsed["first_lib_content"] is not None
    first_code = parsed["first_lib_content"] or ""
    if first_pass_captured:
        (d / "first_lib.rs").write_text(first_code)

    # Final state from disk
    final_path = ws / "src" / "lib.rs"
    final_code = final_path.read_text() if final_path.exists() else ""
    if final_code:
        (d / "final_lib.rs").write_text(final_code)

    # Evaluate first_lib (if captured) — note: no clippy on first-pass to save time
    if first_pass_captured:
        first_eval = evaluate_code(task, first_code)
        first_cm = code_metrics(first_code)
    else:
        first_eval = EvalMetrics()
        first_cm = CodeMetrics()

    # Evaluate final
    final_eval = evaluate_code(task, final_code)
    final_cm = code_metrics(final_code)

    result = Result(
        task=task, treatment=treatment, run=run,
        compile_ok=final_eval.compile_ok,
        tests_passed=final_eval.tests_passed,
        tests_total=final_eval.tests_total,
        clippy_warnings=final_eval.clippy_warnings,
        loc=final_cm.loc, unwrap_count=final_cm.unwrap_count,
        panic_count=final_cm.panic_count, todo_count=final_cm.todo_count,
        unsafe_count=final_cm.unsafe_count,
        first_pass_captured=first_pass_captured,
        first_pass_compile=first_eval.compile_ok,
        first_pass_tests_passed=first_eval.tests_passed,
        first_pass_tests_total=first_eval.tests_total,
        first_pass_loc=first_cm.loc,
        iterations=parsed["num_turns"],
        cargo_invocations=parsed["cargo_invocations"],
        agent_self_verified=parsed["cargo_invocations"] > 0,
        total_cost_usd=parsed["total_cost_usd"],
        duration_s=duration,
    )

    (d / "metrics.json").write_text(json.dumps(asdict(result), indent=2))
    print(
        f"     final: compile={result.compile_ok} "
        f"tests={result.tests_passed}/{result.tests_total} "
        f"clippy={result.clippy_warnings} loc={result.loc} | "
        f"first: compile={result.first_pass_compile} "
        f"tests={result.first_pass_tests_passed}/{result.first_pass_tests_total} | "
        f"turns={result.iterations} cargo={result.cargo_invocations} "
        f"${result.total_cost_usd:.2f} ({result.duration_s:.0f}s)"
    )
    return result


def write_csv(results: list[Result]) -> Path:
    out = EXP_ROOT / "results" / "agentic.csv"
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
    ap.add_argument("--runs", type=int, default=DEFAULT_RUNS)
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
        AGENT_CWD_ROOT.mkdir(parents=True, exist_ok=True)

    total = len(tasks) * len(treatments) * args.runs
    print(f"Agentic sweep: {total} cell(s) "
          f"({len(tasks)} task × {len(treatments)} treatment × {args.runs} run)")

    results: list[Result] = []
    for task in tasks:
        print(f"\n== {task} ==")
        if not args.dry_run:
            setup_eval_workspace(task)
        for treatment in treatments:
            for run in range(args.runs):
                if not args.force and already_done(task, treatment, run):
                    print(f"  skip {task}/{treatment}/run_{run} (already done)")
                    cached = json.loads(
                        (gen_dir(task, treatment, run) / "metrics.json").read_text()
                    )
                    results.append(Result(**cached))
                    continue
                try:
                    r = run_one(task, treatment, run, args.dry_run)
                    if r is not None:
                        results.append(r)
                except Exception as e:
                    print(f"     ! {type(e).__name__}: {e}", file=sys.stderr)

    if results and not args.dry_run:
        path = write_csv(results)
        print(f"\nResults: {path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
