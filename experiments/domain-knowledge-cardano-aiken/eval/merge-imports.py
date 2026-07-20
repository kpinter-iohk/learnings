#!/usr/bin/env python3
"""
Merge `use` statements from one or more Aiken source files.

Groups by module path, unions named imports, preserves bare imports and aliases.
Resolves the case where two `use aiken/foo.{X}` lines both bring `X` into scope
unqualified (Aiken treats this as a duplicate-import error).

Usage:
  merge-imports.py FILE1 [FILE2 ...]

Emits to stdout: deduped use statements first, then the non-`use` content of
each file in order.
"""

import re
import sys
from collections import defaultdict

# use foo/bar
# use foo/bar.{X, Y}
# use foo/bar as baz
# use foo/bar.{X, Y} as baz
USE_RE = re.compile(
    r"^use\s+([A-Za-z_][\w/]*?)(?:\.\{([^}]*)\})?(?:\s+as\s+(\w+))?\s*$"
)


def main():
    if len(sys.argv) < 2:
        sys.exit("usage: merge-imports.py FILE1 [FILE2 ...]")

    # For each module path: {names: set, alias: str|None, bare: bool, bare_alias: str|None}
    # Aliased and unaliased named imports both bring names into unqualified scope,
    # so they share one `names` set. The alias (if any) attaches to the merged form.
    modules = defaultdict(
        lambda: {"names": set(), "alias": None, "bare": False, "bare_alias": None}
    )
    body_chunks = []
    weird_uses = []

    for path in sys.argv[1:]:
        with open(path) as f:
            non_use_lines = []
            for line in f:
                raw = line.rstrip("\n")
                stripped = raw.strip()
                if stripped.startswith("use "):
                    m = USE_RE.match(stripped)
                    if not m:
                        # Multi-line or unusual — preserve verbatim
                        weird_uses.append(raw)
                        continue
                    mod = m.group(1)
                    names_str = m.group(2)
                    alias = m.group(3)
                    if names_str:
                        names = {n.strip() for n in names_str.split(",") if n.strip()}
                        modules[mod]["names"] |= names
                        if alias:
                            modules[mod]["alias"] = alias
                    else:
                        if alias:
                            modules[mod]["bare_alias"] = alias
                        else:
                            modules[mod]["bare"] = True
                else:
                    non_use_lines.append(raw)
            body_chunks.append("\n".join(non_use_lines))

    out_lines = []

    # Emit weird (unparsed) use lines first to be safe
    out_lines.extend(weird_uses)

    for mod in sorted(modules.keys()):
        info = modules[mod]
        # Aliased bare (`use mod as foo`) and bare (`use mod`) coexist in Aiken
        # without conflict — emit both if both are needed.
        if info["bare_alias"]:
            out_lines.append(f"use {mod} as {info['bare_alias']}")
        if info["bare"]:
            out_lines.append(f"use {mod}")
        # Single named-import line per module: merged name set + optional alias.
        # This collapses what would otherwise be two `use mod.{...}` lines
        # (one aliased, one not) into a single line, avoiding the duplicate-import
        # error that Aiken raises when the same name is brought into scope twice.
        if info["names"]:
            names_str = ", ".join(sorted(info["names"]))
            line = f"use {mod}.{{{names_str}}}"
            if info["alias"]:
                line += f" as {info['alias']}"
            out_lines.append(line)

    out_lines.append("")
    out_lines.append("")
    for chunk in body_chunks:
        out_lines.append(chunk)

    print("\n".join(out_lines))


if __name__ == "__main__":
    main()
