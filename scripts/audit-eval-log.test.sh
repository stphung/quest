#!/bin/bash
# Tests for scripts/audit-eval-log.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_SCRIPT="$SCRIPT_DIR/audit-eval-log.sh"
FAILURES=0

pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; FAILURES=$((FAILURES + 1)); }
run_test() { echo "TEST: $1"; }

# --- Test 1: valid "run" entry appends correctly ---
run_test "valid run entry appends and reports correct count"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
echo '{"type":"run","date":"2026-07-03","commit_sha":"abc123","pr_url":"https://example.com/pr/1","agent_count":4,"scope":["A.md"],"findings":[]}' > "$TMPDIR/entry.json"
OUTPUT=$("$LOG_SCRIPT" wiki-audit "$TMPDIR/entry.json" 2>&1)
if [ -f "$TMPDIR/wiki-audit.jsonl" ] && [ "$(wc -l < "$TMPDIR/wiki-audit.jsonl" | tr -d ' ')" = "1" ]; then
  pass "log file has 1 line after first entry"
else
  fail "expected 1 line in log file, got: $(cat "$TMPDIR/wiki-audit.jsonl" 2>/dev/null || echo MISSING)"
fi
if echo "$OUTPUT" | grep -q "now 1 lines"; then
  pass "reports correct line count"
else
  fail "expected 'now 1 lines' in output, got: $OUTPUT"
fi
rm -rf "$TMPDIR"

# --- Test 2: invalid JSON is rejected, log untouched ---
run_test "invalid JSON is rejected and does not create a log file"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
echo 'not valid json{{{' > "$TMPDIR/bad.json"
if "$LOG_SCRIPT" wiki-audit "$TMPDIR/bad.json" 2>/dev/null; then
  fail "expected non-zero exit for invalid JSON"
else
  pass "exits non-zero for invalid JSON"
fi
if [ -f "$TMPDIR/wiki-audit.jsonl" ]; then
  fail "log file should not have been created for invalid JSON"
else
  pass "log file not created for invalid JSON"
fi
rm -rf "$TMPDIR"

# --- Test 3: JSON with missing/wrong "type" is rejected ---
run_test "JSON with missing type field is rejected"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
echo '{"date":"2026-07-03"}' > "$TMPDIR/no-type.json"
if "$LOG_SCRIPT" wiki-audit "$TMPDIR/no-type.json" 2>/dev/null; then
  fail "expected non-zero exit for missing type field"
else
  pass "exits non-zero for missing type field"
fi
rm -rf "$TMPDIR"

# --- Test 4: unknown skill name is rejected ---
run_test "unknown skill name is rejected"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
echo '{"type":"run"}' > "$TMPDIR/entry.json"
if "$LOG_SCRIPT" not-a-real-skill "$TMPDIR/entry.json" 2>/dev/null; then
  fail "expected non-zero exit for unknown skill name"
else
  pass "exits non-zero for unknown skill name"
fi
rm -rf "$TMPDIR"

# --- Test 4b: partial skill name is rejected (word boundary bug fix) ---
run_test "partial skill name (missing -audit suffix) is rejected"
TMPDIR=$(mktemp -d)
export META_AUDIT_HISTORY_DIR="$TMPDIR"
echo '{"type":"run"}' > "$TMPDIR/entry.json"
if "$LOG_SCRIPT" wiki "$TMPDIR/entry.json" 2>/dev/null; then
  fail "expected non-zero exit for partial skill name 'wiki' (not 'wiki-audit')"
else
  pass "exits non-zero for partial skill name 'wiki'"
fi
if [ ! -f "$TMPDIR/wiki.jsonl" ]; then
  pass "does not create wiki.jsonl for partial skill name"
else
  fail "should not have created wiki.jsonl log file for partial skill name"
fi
rm -rf "$TMPDIR"

# --- Test 5: missing arguments is rejected ---
run_test "missing arguments is rejected"
if "$LOG_SCRIPT" wiki-audit 2>/dev/null; then
  fail "expected non-zero exit for missing json-file argument"
else
  pass "exits non-zero for missing json-file argument"
fi

echo ""
if [ "$FAILURES" -eq 0 ]; then
  echo "All tests passed."
  exit 0
else
  echo "$FAILURES test(s) failed."
  exit 1
fi
