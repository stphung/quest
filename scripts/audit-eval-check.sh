#!/bin/bash
# Reports whether a skill's meta-audit deep-eval threshold has been reached.
# Usage: scripts/audit-eval-check.sh <skill-name>
#
# Prints "TRIGGER" if >=5 "run" entries have accumulated since the last
# "eval_marker" entry (or since the start of the log if none exists).
# Otherwise prints "SKIP: <n>/5".
#
# Set META_AUDIT_HISTORY_DIR to override the log directory (used by tests).
set -euo pipefail

THRESHOLD=5

if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed." >&2
  exit 1
fi

SKILL_NAME="${1:-}"

if [ -z "$SKILL_NAME" ]; then
  echo "Usage: $0 <skill-name>" >&2
  exit 1
fi

case "$SKILL_NAME" in
  perf-audit|test-audit|doc-audit|wiki-audit|dependency-audit) ;;
  *)
    echo "Error: unknown skill '$SKILL_NAME'. Must be one of: perf-audit test-audit doc-audit wiki-audit dependency-audit" >&2
    exit 1
    ;;
esac

if [ -z "${META_AUDIT_HISTORY_DIR:-}" ]; then
  REPO_ROOT="$(git rev-parse --show-toplevel)"
fi
LOG_FILE="${META_AUDIT_HISTORY_DIR:-$REPO_ROOT/.claude/skills/meta-audit/history}/${SKILL_NAME}.jsonl"

if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
  echo "SKIP: 0/${THRESHOLD}"
  exit 0
fi

TYPES=$(jq -r '.type' "$LOG_FILE")

LAST_MARKER=$(echo "$TYPES" | grep -n '^eval_marker$' | tail -1 | cut -d: -f1) || true
LAST_MARKER="${LAST_MARKER:-0}"

RUN_COUNT=$(echo "$TYPES" | tail -n "+$((LAST_MARKER + 1))" | grep -c '^run$') || true
RUN_COUNT="${RUN_COUNT:-0}"

if [ "$RUN_COUNT" -ge "$THRESHOLD" ]; then
  echo "TRIGGER"
else
  echo "SKIP: ${RUN_COUNT}/${THRESHOLD}"
fi
