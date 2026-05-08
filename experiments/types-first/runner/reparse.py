#!/usr/bin/env python3
"""
Re-analyze saved transcripts to capture every Write/Edit to lib.rs and
identify each treatment's "first complete attempt":

  - For direct: the first Write event (the agent's initial implementation).
  - For design-first: the first state without any `todo!()` call (i.e. step 2,
    after the skeleton is filled in — usually the second Write event).

Snapshots are saved as lib_v0.rs, lib_v1.rs, ... in each cell directory.
metrics.json is updated with first_complete_* fields.
"""

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from run import (
    TASKS, TREATMENTS, gen_dir, evaluate_code, code_metrics, setup_eval_workspace,
)


def extract_lib_history(transcript_path: Path) -> list[str]:
    states: list[str] = []
    current: str | None = None
    for line in transcript_path.read_text().splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") != "assistant":
            continue
        for block in ev.get("message", {}).get("content", []):
            if block.get("type") != "tool_use":
                continue
            name = block.get("name", "")
            inp = block.get("input", {}) or {}
            fp = inp.get("file_path", "") or ""
            if not fp.endswith("lib.rs"):
                continue
            if name == "Write":
                current = inp.get("content", "") or ""
                states.append(current)
            elif name == "Edit" and current is not None:
                old = inp.get("old_string", "") or ""
                new = inp.get("new_string", "") or ""
                if inp.get("replace_all"):
                    current = current.replace(old, new)
                else:
                    idx = current.find(old)
                    if idx != -1:
                        current = current[:idx] + new + current[idx + len(old):]
                states.append(current)
    return states


def first_complete(states: list[str], treatment: str) -> tuple[int, str] | None:
    if not states:
        return None
    if treatment == "direct":
        return (0, states[0])
    for i, s in enumerate(states):
        if not re.search(r"\btodo\s*!\s*\(", s):
            return (i, s)
    return None


def main():
    print(f"{'cell':<40} {'versions':>8} {'fc_idx':>6} {'compile':>7} {'tests':>10} {'loc':>5}")
    print("-" * 80)
    for task in TASKS:
        setup_eval_workspace(task)
        for tr in TREATMENTS:
            for run in range(3):
                d = gen_dir(task, tr, run)
                tx = d / "transcript.ndjson"
                if not tx.exists():
                    continue
                states = extract_lib_history(tx)
                for i, s in enumerate(states):
                    (d / f"lib_v{i}.rs").write_text(s)
                fc = first_complete(states, tr)
                if fc is None:
                    print(f"{task}/{tr}/run_{run:<2}: 0 versions extracted")
                    continue
                idx, fc_code = fc
                ev = evaluate_code(task, fc_code)
                cm = code_metrics(fc_code)
                metrics = json.loads((d / "metrics.json").read_text())
                metrics.update({
                    "lib_versions": len(states),
                    "first_complete_idx": idx,
                    "first_complete_compile": ev.compile_ok,
                    "first_complete_tests_passed": ev.tests_passed,
                    "first_complete_tests_total": ev.tests_total,
                    "first_complete_loc": cm.loc,
                    "first_complete_unwrap_count": cm.unwrap_count,
                })
                (d / "metrics.json").write_text(json.dumps(metrics, indent=2))
                cell = f"{task}/{tr}/run_{run}"
                print(f"{cell:<40} {len(states):>8} {idx:>6} {str(ev.compile_ok):>7} "
                      f"{ev.tests_passed:>4}/{ev.tests_total:<4} {cm.loc:>5}")


if __name__ == "__main__":
    main()
