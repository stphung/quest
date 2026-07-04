#!/bin/bash
# Appends a single JSON entry to a meta-audit history log.
# Usage: scripts/audit-eval-log.sh <skill-name> <json-file>
#
# The json-file must contain one JSON object with a "type" field of "run" or
# "eval_marker". Fails loudly (non-zero exit) and leaves the log untouched on
# any validation failure, so a malformed entry never corrupts the log.
#
# Set META_AUDIT_HISTORY_DIR to override the log directory (used by tests).
set -euo pipefail

VALID_SKILLS="perf-audit test-audit doc-audit wiki-audit dependency-audit"

if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed." >&2
  exit 1
fi

SKILL_NAME="${1:-}"
JSON_FILE="${2:-}"

if [ -z "$SKILL_NAME" ] || [ -z "$JSON_FILE" ]; then
  echo "Usage: $0 <skill-name> <json-file>" >&2
  exit 1
fi

if ! echo "$VALID_SKILLS" | grep -qw "$SKILL_NAME"; then
  echo "Error: unknown skill '$SKILL_NAME'. Must be one of: $VALID_SKILLS" >&2
  exit 1
fi

if [ ! -f "$JSON_FILE" ]; then
  echo "Error: file not found: $JSON_FILE" >&2
  exit 1
fi

if ! jq empty "$JSON_FILE" 2>/dev/null; then
  echo "Error: $JSON_FILE is not valid JSON. Log NOT written." >&2
  exit 1
fi

ENTRY_TYPE=$(jq -r '.type // empty' "$JSON_FILE")
if [ "$ENTRY_TYPE" != "run" ] && [ "$ENTRY_TYPE" != "eval_marker" ]; then
  echo "Error: JSON \"type\" field must be \"run\" or \"eval_marker\", got: '${ENTRY_TYPE}'. Log NOT written." >&2
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
LOG_DIR="${META_AUDIT_HISTORY_DIR:-$REPO_ROOT/.claude/skills/meta-audit/history}"
LOG_FILE="$LOG_DIR/${SKILL_NAME}.jsonl"

mkdir -p "$LOG_DIR"
jq -c . "$JSON_FILE" >> "$LOG_FILE"

LINE_COUNT=$(wc -l < "$LOG_FILE" | tr -d ' ')
echo "Logged 1 entry to $LOG_FILE (now $LINE_COUNT lines)"
