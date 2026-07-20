#!/usr/bin/env bash
# Run a single arm of the eval for a single task.
# Usage: run-arm.sh <task-name> <arm-name>
#   task-name = vesting | one-shot-nft | multi-sig
#   arm-name  = doc-on  | doc-off  | reference  (any name)
#
# Pre-condition: tasks/<task>/submissions/<arm>.ak contains the sub-agent's
# validator code. This script:
#   1. Assembles submission + tests.ak into the project's validators/main.ak,
#      hoisting and deduplicating `use` statements to the top of the file
#      (Aiken requires all imports at file head).
#   2. Runs `aiken check`, capturing stdout+stderr.
#   3. Stores results under tasks/<task>/results/<arm>/.

set -euo pipefail

TASK="${1:?task name required}"
ARM="${2:?arm name required}"

ROOT="$(cd "$(dirname "$0")" && pwd)"
TASK_DIR="$ROOT/tasks/$TASK"
SUBMISSION="$TASK_DIR/submissions/$ARM.ak"
TESTS="$TASK_DIR/tests.ak"
RESULTS_DIR="$TASK_DIR/results/$ARM"

if [[ ! -f "$SUBMISSION" ]]; then
  echo "ERROR: submission not found at $SUBMISSION" >&2
  exit 1
fi
if [[ ! -f "$TESTS" ]]; then
  echo "ERROR: tests not found at $TESTS" >&2
  exit 1
fi

mkdir -p "$RESULTS_DIR"

MAIN="$TASK_DIR/validators/main.ak"

# Assemble via the Python merger — handles overlapping named imports from the
# same module (e.g. tests import `aiken/interval.{Interval}` and the submission
# imports `aiken/interval.{Finite, Interval, IntervalBound}`).
python3 "$ROOT/merge-imports.py" "$SUBMISSION" "$TESTS" > "$MAIN"

# Run aiken check
cd "$TASK_DIR"
set +e
# Aiken writes diagnostic errors only to a TTY-capable target; redirected
# stderr/stdout swallow them silently. Use `script` to allocate a pseudo-TTY
# and strip ANSI codes after.
script -qefc "aiken check" /dev/null > "$RESULTS_DIR/check.raw.txt" 2>&1
EXIT_CODE=$?
set -e

# Strip ANSI escape codes and the "Script started/done" wrapper lines.
sed -E 's/\x1B\[[0-9;]*[mGKHF]//g; /^Script (started|done)/d' \
    "$RESULTS_DIR/check.raw.txt" > "$RESULTS_DIR/check.txt"
rm -f "$RESULTS_DIR/check.raw.txt"

cp "$MAIN" "$RESULTS_DIR/assembled.ak"

{
  echo "exit_code=$EXIT_CODE"
  echo "task=$TASK"
  echo "arm=$ARM"
  echo "timestamp=$(date -Iseconds)"
  if grep -q '"summary"' "$RESULTS_DIR/check.txt"; then
    echo "tests_present=yes"
    # Extract test pass/fail counts via grep -A
    grep -E '"(total|passed|failed)"' "$RESULTS_DIR/check.txt" | head -3
  else
    echo "tests_present=no"
  fi
} > "$RESULTS_DIR/meta.txt"

echo "Exit code: $EXIT_CODE"
echo "Results: $RESULTS_DIR"
