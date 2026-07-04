#!/bin/bash
# Tests for scripts/audit-eval-check.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK_SCRIPT="$SCRIPT_DIR/audit-eval-check.sh"
FAILURES=0

pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; FAILURES=$((FAILURES + 1)); }
run_test() { echo "TEST: $1"; }

# --- Test 1: no log file yet -> SKIP: 0/5 ---
run_test "missing log file reports SKIP: 0/5"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
OUTPUT=$("$CHECK_SCRIPT" wiki-audit)
if [ "$OUTPUT" = "SKIP: 0/5" ]; then
  pass "reports SKIP: 0/5 for missing log"
else
  fail "expected 'SKIP: 0/5', got: $OUTPUT"
fi
rm -rf "$TMPDIR"

# --- Test 2: 3 run entries, no marker -> SKIP: 3/5 ---
run_test "3 run entries with no marker reports SKIP: 3/5"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
for i in 1 2 3; do
  echo '{"type":"run"}' >> "$TMPDIR/wiki-audit.jsonl"
done
OUTPUT=$("$CHECK_SCRIPT" wiki-audit)
if [ "$OUTPUT" = "SKIP: 3/5" ]; then
  pass "reports SKIP: 3/5 for 3 run entries"
else
  fail "expected 'SKIP: 3/5', got: $OUTPUT"
fi
rm -rf "$TMPDIR"

# --- Test 3: exactly 5 run entries -> TRIGGER ---
run_test "5 run entries with no marker reports TRIGGER"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
for i in 1 2 3 4 5; do
  echo '{"type":"run"}' >> "$TMPDIR/wiki-audit.jsonl"
done
OUTPUT=$("$CHECK_SCRIPT" wiki-audit)
if [ "$OUTPUT" = "TRIGGER" ]; then
  pass "reports TRIGGER for 5 run entries"
else
  fail "expected 'TRIGGER', got: $OUTPUT"
fi
rm -rf "$TMPDIR"

# --- Test 4: marker resets the count ---
run_test "eval_marker resets the count for subsequent runs"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
for i in 1 2 3 4 5; do
  echo '{"type":"run"}' >> "$TMPDIR/wiki-audit.jsonl"
done
echo '{"type":"eval_marker","runs_covered":5}' >> "$TMPDIR/wiki-audit.jsonl"
echo '{"type":"run"}' >> "$TMPDIR/wiki-audit.jsonl"
echo '{"type":"run"}' >> "$TMPDIR/wiki-audit.jsonl"
OUTPUT=$("$CHECK_SCRIPT" wiki-audit)
if [ "$OUTPUT" = "SKIP: 2/5" ]; then
  pass "reports SKIP: 2/5 after marker resets count"
else
  fail "expected 'SKIP: 2/5', got: $OUTPUT"
fi
rm -rf "$TMPDIR"

# --- Test 5: unknown skill name is rejected ---
run_test "unknown skill name is rejected"
if "$CHECK_SCRIPT" not-a-real-skill 2>/dev/null; then
  fail "expected non-zero exit for unknown skill name"
else
  pass "exits non-zero for unknown skill name"
fi

# --- Test 6: partial/word-component skill name is rejected (not just wholesale garbage) ---
run_test "partial skill name (word component of a valid one) is rejected"
if "$CHECK_SCRIPT" wiki 2>/dev/null; then
  fail "expected non-zero exit for partial skill name 'wiki'"
else
  pass "exits non-zero for partial skill name 'wiki'"
fi

echo ""
if [ "$FAILURES" -eq 0 ]; then
  echo "All tests passed."
  exit 0
else
  echo "$FAILURES test(s) failed."
  exit 1
fi
