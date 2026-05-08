#!/usr/bin/env python3
"""
Mechanical idiom-compliance metrics over saved lib.rs files.

For each (mode, task, treatment, run), compute:

  doc_pct           — % of public items with a /// doc comment immediately above
                       (Rust API Guidelines C-DOC: every public item should be documented)
  derive_debug_pct  — % of public structs/enums with #[derive(... Debug ...)]
                       (C-COMMON-TRAITS recommends Debug on most types)
  lib_panics        — count of unwrap()/panic!()/todo!()/unimplemented!() OUTSIDE
                       #[cfg(test)] blocks (clippy::unwrap_used; library code shouldn't
                       panic for callable inputs)
  expects           — count of .expect( OUTSIDE test blocks (separate because expect
                       with a documented reason is sometimes idiomatic)
  newtype_count     — count of `pub struct X(<primitive>);` newtypes (idiom for
                       wrapping primitive IDs and units)
  clippy_per_100loc — clippy::pedantic warnings per 100 LOC (already collected;
                       normalized for size)

Outputs results/idiom_metrics.csv and prints an aggregate table.
No LLM calls — purely textual analysis.
"""

import csv
import json
import re
from collections import defaultdict
from pathlib import Path
from statistics import mean

EXP_ROOT = Path(__file__).resolve().parent.parent
TASKS = ["scripting", "job_queue", "roman", "bank_ledger", "levenshtein"]
TREATMENTS = ["direct", "design_first"]
MODES = ["oneshot", "agentic"]


def strip_test_blocks(code: str) -> str:
    """Remove `#[cfg(test)]` items (mod tests, fn ...) by brace-matching."""
    out: list[str] = []
    pos = 0
    while pos < len(code):
        idx = code.find("#[cfg(test)]", pos)
        if idx == -1:
            out.append(code[pos:])
            break
        out.append(code[pos:idx])
        brace_idx = code.find("{", idx)
        if brace_idx == -1:
            out.append(code[idx:])
            break
        depth = 1
        i = brace_idx + 1
        while i < len(code) and depth > 0:
            c = code[i]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
            i += 1
        pos = i
    return "".join(out)


PUB_ITEM_RE = re.compile(
    r"^\s*pub\s+(fn|struct|enum|trait|const|static|type|mod)\s+\w+"
)
PUB_TYPE_RE = re.compile(r"^\s*pub\s+(struct|enum)\s+\w+")
DERIVE_RE = re.compile(r"^\s*#\[derive\(([^)]*)\)\]")


def _has_doc_above(lines: list[str], i: int) -> bool:
    """True if a /// comment precedes lines[i] (skipping blank/attribute lines)."""
    j = i - 1
    while j >= 0:
        s = lines[j].strip()
        if s == "":
            j -= 1
            continue
        if s.startswith("#["):
            j -= 1
            continue
        if s.startswith("///") or s.startswith("//!"):
            return True
        return False
    return False


def _derive_above(lines: list[str], i: int) -> str | None:
    """Return the contents of a #[derive(...)] preceding lines[i], or None."""
    j = i - 1
    while j >= 0:
        s = lines[j].strip()
        if s == "" or s.startswith("///") or s.startswith("//!"):
            j -= 1
            continue
        m = DERIVE_RE.match(lines[j])
        if m:
            return m.group(1)
        if s.startswith("#["):
            j -= 1
            continue
        return None
    return None


def doc_coverage(code: str) -> tuple[int, int]:
    code = strip_test_blocks(code)
    lines = code.split("\n")
    documented = total = 0
    for i, line in enumerate(lines):
        if not PUB_ITEM_RE.match(line):
            continue
        total += 1
        if _has_doc_above(lines, i):
            documented += 1
    return documented, total


def derive_debug_coverage(code: str) -> tuple[int, int]:
    code = strip_test_blocks(code)
    lines = code.split("\n")
    has = total = 0
    for i, line in enumerate(lines):
        if not PUB_TYPE_RE.match(line):
            continue
        total += 1
        d = _derive_above(lines, i) or ""
        if "Debug" in d:
            has += 1
    return has, total


def lib_panic_counts(code: str) -> dict:
    code = strip_test_blocks(code)
    return {
        "unwrap": len(re.findall(r"\.unwrap\s*\(", code)),
        "expect": len(re.findall(r"\.expect\s*\(", code)),
        "panic": len(re.findall(r"\bpanic\s*!\s*\(", code)),
        "todo": len(re.findall(r"\btodo\s*!\s*\(", code)),
        "unimplemented": len(re.findall(r"\bunimplemented\s*!\s*\(", code)),
    }


PRIMITIVE_NEWTYPE_RE = re.compile(
    r"pub\s+struct\s+\w+\s*\(\s*(?:pub\s+)?"
    r"(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64|bool|String)"
    r"\s*\)\s*;"
)


def newtype_count(code: str) -> int:
    return len(PRIMITIVE_NEWTYPE_RE.findall(strip_test_blocks(code)))


def lib_path(mode: str, task: str, tr: str, run: int) -> Path:
    base = EXP_ROOT / "generations" / mode / task / tr / f"run_{run}"
    return base / ("final_lib.rs" if mode == "agentic" else "lib.rs")


def metrics_path(mode: str, task: str, tr: str, run: int) -> Path:
    return EXP_ROOT / "generations" / mode / task / tr / f"run_{run}" / "metrics.json"


def main():
    rows: list[dict] = []
    for mode in MODES:
        for task in TASKS:
            for tr in TREATMENTS:
                for r in range(3):
                    lp = lib_path(mode, task, tr, r)
                    if not lp.exists():
                        continue
                    code = lp.read_text()
                    if not code.strip():
                        continue

                    docs, doc_total = doc_coverage(code)
                    deriv, deriv_total = derive_debug_coverage(code)
                    panics = lib_panic_counts(code)
                    newts = newtype_count(code)

                    md = {}
                    mp = metrics_path(mode, task, tr, r)
                    if mp.exists():
                        md = json.loads(mp.read_text())
                    loc = md.get("loc", 0)
                    clippy = md.get("clippy_warnings")

                    rows.append({
                        "mode": mode, "task": task, "treatment": tr, "run": r,
                        "doc_documented": docs, "doc_total": doc_total,
                        "doc_pct": (100 * docs / doc_total) if doc_total else 0.0,
                        "derive_debug": deriv, "derive_total": deriv_total,
                        "derive_pct": (100 * deriv / deriv_total) if deriv_total else 0.0,
                        "lib_panics": (
                            panics["unwrap"] + panics["panic"]
                            + panics["todo"] + panics["unimplemented"]
                        ),
                        "lib_unwrap": panics["unwrap"],
                        "lib_expect": panics["expect"],
                        "newtype_count": newts,
                        "loc": loc,
                        "clippy_warnings": clippy,
                        "clippy_per_100loc": (
                            (100 * clippy / loc) if (loc and clippy is not None) else None
                        ),
                    })

    out = EXP_ROOT / "results" / "idiom_metrics.csv"
    out.parent.mkdir(parents=True, exist_ok=True)
    if rows:
        with out.open("w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
            w.writeheader()
            w.writerows(rows)

    by = defaultdict(list)
    for r in rows:
        by[(r["mode"], r["task"], r["treatment"])].append(r)

    print(
        f'{"mode":<8} {"task":<10} {"treat":<14} '
        f'{"doc%":>5} {"drv%":>5} {"panic":>5} {"unwrp":>5} '
        f'{"newt":>4} {"cl/100":>7} {"loc":>5}'
    )
    print("-" * 80)
    for k in sorted(by):
        rs = by[k]
        d = mean(r["doc_pct"] for r in rs)
        v = mean(r["derive_pct"] for r in rs)
        p = mean(r["lib_panics"] for r in rs)
        u = mean(r["lib_unwrap"] for r in rs)
        n = mean(r["newtype_count"] for r in rs)
        c_per = [r["clippy_per_100loc"] for r in rs if r["clippy_per_100loc"] is not None]
        c = mean(c_per) if c_per else float("nan")
        loc = mean(r["loc"] for r in rs)
        print(
            f'{k[0]:<8} {k[1]:<10} {k[2]:<14} '
            f'{d:>5.0f} {v:>5.0f} {p:>5.1f} {u:>5.1f} '
            f'{n:>4.1f} {c:>7.2f} {loc:>5.0f}'
        )

    print()
    print("=== Direct vs design-first deltas (design − direct) ===")
    print(
        f'{"mode":<8} {"task":<10} {"Δdoc%":>6} {"Δdrv%":>6} '
        f'{"Δpanic":>7} {"Δnewt":>6} {"Δcl/100":>8}'
    )
    print("-" * 60)
    for mode in MODES:
        for task in TASKS:
            d = [r for r in rows if r["mode"] == mode and r["task"] == task and r["treatment"] == "direct"]
            f_ = [r for r in rows if r["mode"] == mode and r["task"] == task and r["treatment"] == "design_first"]
            if not d or not f_:
                continue
            dd = mean(r["doc_pct"] for r in f_) - mean(r["doc_pct"] for r in d)
            dv = mean(r["derive_pct"] for r in f_) - mean(r["derive_pct"] for r in d)
            dp = mean(r["lib_panics"] for r in f_) - mean(r["lib_panics"] for r in d)
            dn = mean(r["newtype_count"] for r in f_) - mean(r["newtype_count"] for r in d)
            d_cl = (
                [r["clippy_per_100loc"] for r in f_ if r["clippy_per_100loc"] is not None],
                [r["clippy_per_100loc"] for r in d if r["clippy_per_100loc"] is not None],
            )
            dc = (mean(d_cl[0]) if d_cl[0] else 0) - (mean(d_cl[1]) if d_cl[1] else 0)
            print(
                f'{mode:<8} {task:<10} {dd:>+6.0f} {dv:>+6.0f} {dp:>+7.1f} '
                f'{dn:>+6.1f} {dc:>+8.2f}'
            )


if __name__ == "__main__":
    main()
