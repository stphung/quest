# meta-audit Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `meta-audit` skill that evaluates the five domain audit skills (`perf-audit`, `test-audit`, `doc-audit`, `wiki-audit`, `dependency-audit`) for finding accuracy and scope staleness, and safely improves their `SKILL.md` files over time.

**Architecture:** Two small deterministic bash scripts (`scripts/audit-eval-log.sh`, `scripts/audit-eval-check.sh`) back a per-skill JSONL history log under `.claude/skills/meta-audit/history/`. Each of the five domain skills gets a `## Log This Run` step appended and (for four of them) a `scope-universe` HTML comment. The new `meta-audit/SKILL.md` reads that history, re-verifies past findings against source, checks scope drift, and ships safe fixes via PR. Historical PRs are backfilled into the log now so the mechanism has real data from day one instead of a cold start.

**Tech Stack:** Bash, `jq`, Markdown (skill definitions), git.

**Design doc:** `docs/superpowers/specs/2026-07-03-meta-audit-skill-design.md`

---

## Task 1: `scripts/audit-eval-log.sh` (with tests)

**Files:**
- Create: `scripts/audit-eval-log.test.sh`
- Create: `scripts/audit-eval-log.sh`

- [ ] **Step 1: Write the test script**

Create `scripts/audit-eval-log.test.sh`:

```bash
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
```

- [ ] **Step 2: Make it executable and run it to confirm it fails**

```bash
chmod +x scripts/audit-eval-log.test.sh
./scripts/audit-eval-log.test.sh
```

Expected: `bash: scripts/audit-eval-log.sh: No such file or directory` (or similar) — the script under test doesn't exist yet.

- [ ] **Step 3: Write `scripts/audit-eval-log.sh`**

```bash
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
```

- [ ] **Step 4: Make it executable and run the tests**

```bash
chmod +x scripts/audit-eval-log.sh
./scripts/audit-eval-log.test.sh
```

Expected: `All tests passed.` with 7 `PASS:` lines and exit code 0.

- [ ] **Step 5: Commit**

```bash
git add scripts/audit-eval-log.sh scripts/audit-eval-log.test.sh
git commit -m "feat(meta-audit): add audit-eval-log.sh with tests"
```

---

## Task 2: `scripts/audit-eval-check.sh` (with tests)

**Files:**
- Create: `scripts/audit-eval-check.test.sh`
- Create: `scripts/audit-eval-check.sh`

- [ ] **Step 1: Write the test script**

Create `scripts/audit-eval-check.test.sh`:

```bash
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
```

- [ ] **Step 2: Make it executable and run it to confirm it fails**

```bash
chmod +x scripts/audit-eval-check.test.sh
./scripts/audit-eval-check.test.sh
```

Expected: `bash: scripts/audit-eval-check.sh: No such file or directory` (or similar).

- [ ] **Step 3: Write `scripts/audit-eval-check.sh`**

```bash
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
```

- [ ] **Step 4: Make it executable and run the tests**

```bash
chmod +x scripts/audit-eval-check.sh
./scripts/audit-eval-check.test.sh
```

Expected: `All tests passed.` with 6 `PASS:` lines and exit code 0.

- [ ] **Step 5: Commit**

```bash
git add scripts/audit-eval-check.sh scripts/audit-eval-check.test.sh
git commit -m "feat(meta-audit): add audit-eval-check.sh with tests"
```

---

## Task 3: Create the `meta-audit` skill

**Files:**
- Create: `.claude/skills/meta-audit/SKILL.md`
- Create: `.claude/skills/meta-audit/history/perf-audit.jsonl` (empty)
- Create: `.claude/skills/meta-audit/history/test-audit.jsonl` (empty)
- Create: `.claude/skills/meta-audit/history/doc-audit.jsonl` (empty)
- Create: `.claude/skills/meta-audit/history/wiki-audit.jsonl` (empty)
- Create: `.claude/skills/meta-audit/history/dependency-audit.jsonl` (empty)

- [ ] **Step 1: Create the empty history log files**

Git doesn't track empty directories, so create the (empty) log files now — Task 4-8 append `## Log This Run` sections that reference these exact paths, and Tasks 9-13 populate them via `audit-eval-log.sh`.

```bash
mkdir -p .claude/skills/meta-audit/history
touch .claude/skills/meta-audit/history/perf-audit.jsonl
touch .claude/skills/meta-audit/history/test-audit.jsonl
touch .claude/skills/meta-audit/history/doc-audit.jsonl
touch .claude/skills/meta-audit/history/wiki-audit.jsonl
touch .claude/skills/meta-audit/history/dependency-audit.jsonl
```

- [ ] **Step 2: Write `.claude/skills/meta-audit/SKILL.md`**

```markdown
---
name: meta-audit
description: Evaluates the accuracy and scope coverage of the other audit skills (perf-audit, test-audit, doc-audit, wiki-audit, dependency-audit) by re-verifying past findings against source and checking for scope drift, then safely improves their SKILL.md files. Use when asked to evaluate audit skill performance, or auto-triggered by a domain audit skill after its 5th run since the last evaluation.
---

# Meta-Audit: Evaluating the Audit Skills

Re-verifies past findings from the five domain audit skills against source (as of the
commit each finding was made against) and checks whether each skill's declared scope has
fallen behind the current codebase. Auto-fixes safe improvements; flags anything that
would change audit strategy.

## When to Use

- Auto-triggered: a domain audit skill's `## Log This Run` step reports `TRIGGER` from
  `scripts/audit-eval-check.sh <skill-name>` (5 runs accumulated since the last evaluation)
- On-demand: "evaluate wiki-audit", "how are the audits doing" (runs all five independently)

## Inputs

One or more target skill names: `perf-audit`, `test-audit`, `doc-audit`, `wiki-audit`,
`dependency-audit`. When invoked for "all five," repeat the full flow below independently
per skill (they don't share state or need to run in a particular order).

## Phase 1: Load History

Read `.claude/skills/meta-audit/history/<skill-name>.jsonl`. Take every `"type": "run"`
entry after the last `"type": "eval_marker"` entry (or every `run` entry if no marker
exists yet). If there are zero `run` entries to evaluate, stop and report "nothing to
evaluate yet."

## Phase 2: Adversarial Re-Verification (Parallel Agents, Read-Only)

Collect every finding across the loaded runs (cap at the 20 most recent if there are
more — note in the final report how many were skipped and why).

For each finding, spawn one fresh agent given **only** its `location` and `claim` —
never its logged `correct_value`. Task: independently derive what the correct value
actually was **as of that finding's `commit_sha`**, reading source read-only via
`git show <commit_sha>:<path>` (never `git checkout` — this must never mutate the shared
working tree). The agent reports its own independently-derived value plus the source
location it based that on.

Diff the agent's answer against the logged `correct_value`. A mismatch is a confirmed
inaccuracy in the original audit — record it as `{finding, original_correct_value,
re_derived_value, agent_citation}`.

## Phase 3: Scope-Staleness Check

Only for skills with a `<!-- meta-audit scope-universe: ... -->` comment near the top of
their `SKILL.md` (currently: wiki-audit, doc-audit, test-audit, perf-audit —
dependency-audit has none, and this phase is a no-op for it).

1. Read the scope-universe comment and enumerate what it currently matches in the repo
   (e.g. `ls quest.wiki/*.md`, `ls src/*/`, `find tests -maxdepth 1 -type d -name '*_tests'`).
2. Read the target skill's `## Phase 1` section and collect every `Scope:` line across
   all its agents into one set.
3. Diff the two sets:
   - **Scope gap** — in the universe but not covered by any agent's `Scope:` line.
   - **Scope rot** — named in a `Scope:` line but doesn't exist in the universe anymore.

## Phase 4: Synthesize

Group confirmed inaccuracies (Phase 2) by category. A category that recurs more than
once across the evaluated runs is a stronger signal than a one-off — call this out
explicitly in the report, since it justifies adding a guardrail rather than shrugging
off a single mistake.

Rank scope gaps/rot (Phase 3) — a gap that's existed across multiple backfilled runs
(the file/module existed before several of the evaluated runs and was still never
picked up) ranks higher than one that just appeared.

## Phase 5: Fix

### Auto-fix (no user approval needed)
- Add a missing path to the correct agent's `Scope:` line (scope gap)
- Remove a path from a `Scope:` line that no longer exists (scope rot)
- Append one row to the skill's anti-patterns/findings table documenting a *confirmed,
  recurring* false-positive category, with a one-line guardrail (e.g. "verify a
  mechanic from its source implementation, not its name — see history for a past miss")

### Flag for user review
- Any change to a skill's agent count or Phase structure
- Any change to what a skill treats as auto-fixable vs. flag-for-review
- A confirmed inaccuracy that isn't part of a recurring pattern (a one-off mistake
  doesn't justify rewriting the skill — report it, but don't act on it)

## Phase 6: Ship

The `eval_marker` needs the PR's own URL, which doesn't exist until the PR is created —
so commit the fix(es) and open the PR first, then append the marker as a follow-up
commit onto that same still-open branch before it merges:

1. Branch, commit the `SKILL.md` fix(es) and the updated history log, open a PR (same
   pattern as every other audit skill — no direct commits to `main`). PR body includes
   the full report: confirmed inaccuracies (with citations), scope gaps/rot, what was
   auto-fixed, what's flagged for review.
2. Now that the PR exists, append the `eval_marker` entry with its real URL:
   ```bash
   echo '{"type":"eval_marker","date":"<today>","pr_url":"<the PR URL from step 1>","runs_covered":<n>}' > /tmp/marker.json
   scripts/audit-eval-log.sh <skill-name> /tmp/marker.json
   ```
3. Commit and push this marker addition as one more commit on the same PR branch
   (multiple commits on one PR are normal — squash-merge collapses them at merge time).

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/meta-audit/
git commit -m "feat(meta-audit): add meta-audit skill definition"
```

---

## Task 4: Wire up `wiki-audit`

**Files:**
- Modify: `.claude/skills/wiki-audit/SKILL.md`

- [ ] **Step 1: Add the scope-universe comment**

Find this text near the top of `.claude/skills/wiki-audit/SKILL.md`:

```markdown
---
name: wiki-audit
description: Multi-agent wiki audit — finds stale numbers, missing systems, and broken links in the player-facing wiki. Use when wiki is stale, after game changes, or before releases.
---

# Audit Player-Facing Wiki
```

Replace it with:

```markdown
---
name: wiki-audit
description: Multi-agent wiki audit — finds stale numbers, missing systems, and broken links in the player-facing wiki. Use when wiki is stale, after game changes, or before releases.
---

<!-- meta-audit scope-universe: quest.wiki/*.md -->

# Audit Player-Facing Wiki
```

- [ ] **Step 2: Add the "Log This Run" section**

Find this text near the end of `.claude/skills/wiki-audit/SKILL.md`:

```markdown
# Update submodule pointer in main repo
git add quest.wiki
git commit -m "docs: update quest.wiki submodule pointer"
```

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

Replace it with:

```markdown
# Update submodule pointer in main repo
git add quest.wiki
git commit -m "docs: update quest.wiki submodule pointer"
```

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 4,
     "scope": ["Combat.md", "Equipment.md"],
     "findings": [
       {
         "location": "Combat.md:42",
         "claim": "...",
         "correct_value": "...",
         "severity": "HIGH",
         "category": "stale-constant",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh wiki-audit /tmp/wiki-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh wiki-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `wiki-audit` next. If it
   prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.
```

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/wiki-audit/SKILL.md
git commit -m "feat(meta-audit): wire wiki-audit into meta-audit logging"
```

---

## Task 5: Wire up `doc-audit`

**Files:**
- Modify: `.claude/skills/doc-audit/SKILL.md`

- [ ] **Step 1: Add the scope-universe comment**

Find this text near the top of `.claude/skills/doc-audit/SKILL.md`:

```markdown
---
name: doc-audit
description: Multi-agent developer documentation audit — finds stale constants, missing files, and outdated types across CLAUDE.md files, README.md, and docs/. Use when docs are stale, after adding features, or before releases.
---

# Audit Developer Documentation
```

Replace it with:

```markdown
---
name: doc-audit
description: Multi-agent developer documentation audit — finds stale constants, missing files, and outdated types across CLAUDE.md files, README.md, and docs/. Use when docs are stale, after adding features, or before releases.
---

<!-- meta-audit scope-universe: CLAUDE.md docs/*.md src/*/CLAUDE.md README.md -->

# Audit Developer Documentation
```

- [ ] **Step 2: Add the "Log This Run" section**

Find this text near the end of `.claude/skills/doc-audit/SKILL.md`:

```markdown
| Key Constants | If applicable |
| Adding / Extending | If applicable |

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

Replace it with:

```markdown
| Key Constants | If applicable |
| Adding / Extending | If applicable |

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 6,
     "scope": ["CLAUDE.md", "src/combat/CLAUDE.md"],
     "findings": [
       {
         "location": "src/combat/CLAUDE.md:42",
         "claim": "...",
         "correct_value": "...",
         "severity": "HIGH",
         "category": "stale-constant",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh doc-audit /tmp/doc-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh doc-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `doc-audit` next. If it
   prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.
```

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/doc-audit/SKILL.md
git commit -m "feat(meta-audit): wire doc-audit into meta-audit logging"
```

---

## Task 6: Wire up `test-audit`

**Files:**
- Modify: `.claude/skills/test-audit/SKILL.md`

- [ ] **Step 1: Add the scope-universe comment**

Find this text near the top of `.claude/skills/test-audit/SKILL.md`:

```markdown
---
name: test-audit
description: Multi-agent test health audit — finds flaky tests and performance bottlenecks by area, auto-fixes safe patterns, and verifies with 10x runs. Use when tests are flaky, slow, or before releases.
---

# Test Health Audit
```

Replace it with:

```markdown
---
name: test-audit
description: Multi-agent test health audit — finds flaky tests and performance bottlenecks by area, auto-fixes safe patterns, and verifies with 10x runs. Use when tests are flaky, slow, or before releases.
---

<!-- meta-audit scope-universe: tests/*_tests/ -->

# Test Health Audit
```

- [ ] **Step 2: Add the "Log This Run" section**

Find this text near the end of `.claude/skills/test-audit/SKILL.md`:

```markdown
| Ignoring Monte Carlo tests as "probably fine" | Check margins are generous (2-5x expected range) |
| Skipping the 10x verification | Flakiness is probabilistic; 1 run proves nothing |

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

Replace it with:

```markdown
| Ignoring Monte Carlo tests as "probably fine" | Check margins are generous (2-5x expected range) |
| Skipping the 10x verification | Flakiness is probabilistic; 1 run proves nothing |

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 3,
     "scope": ["tests/game_tick_tests/", "tests/combat_tests/"],
     "findings": [
       {
         "location": "tests/combat_tests/foo_test.rs:42",
         "claim": "...",
         "correct_value": "...",
         "severity": "HIGH",
         "category": "unseeded-rng",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh test-audit /tmp/test-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh test-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `test-audit` next. If it
   prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/test-audit/SKILL.md
git commit -m "feat(meta-audit): wire test-audit into meta-audit logging"
```

---

## Task 7: Wire up `perf-audit`

**Files:**
- Modify: `.claude/skills/perf-audit/SKILL.md`

- [ ] **Step 1: Add the scope-universe comment**

Find this text near the top of `.claude/skills/perf-audit/SKILL.md`:

```markdown
---
name: perf-audit
description: Multi-agent performance audit — finds and fixes hot-path bottlenecks, adds criterion benchmarks and simulator profiling. Use when game feels slow, after adding features, or to establish performance baselines.
---

# Performance Audit
```

Replace it with:

```markdown
---
name: perf-audit
description: Multi-agent performance audit — finds and fixes hot-path bottlenecks, adds criterion benchmarks and simulator profiling. Use when game feels slow, after adding features, or to establish performance baselines.
---

<!-- meta-audit scope-universe: src/*/ (thematic — flags a new top-level module with no agent assigned, not every file) -->

# Performance Audit
```

- [ ] **Step 2: Add the "Log This Run" section**

Find this text near the end of `.claude/skills/perf-audit/SKILL.md`:

```markdown
1. `make check` must pass
2. `cargo bench` runs without errors
3. Report summary of: findings, auto-fixes applied, items flagged for review, benchmark baselines

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

Replace it with:

```markdown
1. `make check` must pass
2. `cargo bench` runs without errors
3. Report summary of: findings, auto-fixes applied, items flagged for review, benchmark baselines

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 3,
     "scope": ["src/core/", "src/ui/"],
     "findings": [
       {
         "location": "src/core/tick_stages.rs:331",
         "claim": "...",
         "correct_value": "...",
         "severity": "MEDIUM",
         "category": "per-tick-allocation",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh perf-audit /tmp/perf-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh perf-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `perf-audit` next. If it
   prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/perf-audit/SKILL.md
git commit -m "feat(meta-audit): wire perf-audit into meta-audit logging"
```

---

## Task 8: Wire up `dependency-audit`

**Files:**
- Modify: `.claude/skills/dependency-audit/SKILL.md`

No scope-universe comment for this one — its scope is `Cargo.toml`'s current contents, which can't go stale by definition (see design doc's Open Questions).

- [ ] **Step 1: Add the "Log This Run" section**

Find this text near the end of `.claude/skills/dependency-audit/SKILL.md`:

```markdown
| Security advisories | N | list |
| Flagged for review | N | list |

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

Replace it with:

```markdown
| Security advisories | N | list |
| Flagged for review | N | list |

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 2,
     "scope": ["Cargo.toml"],
     "findings": [
       {
         "location": "Cargo.toml:uuid",
         "claim": "...",
         "correct_value": "...",
         "severity": "LOW",
         "category": "patch-bump",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh dependency-audit /tmp/dependency-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh dependency-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `dependency-audit` next. If
   it prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.
```

- [ ] **Step 2: Commit**

```bash
git add .claude/skills/dependency-audit/SKILL.md
git commit -m "feat(meta-audit): wire dependency-audit into meta-audit logging"
```

---

## Task 9: Backfill `wiki-audit.jsonl`

**Files:**
- Modify: `.claude/skills/meta-audit/history/wiki-audit.jsonl` (via script, not direct edit)

This backfills the 2026-07-03 wiki-audit session (merged as PR #643) using its real,
already-known findings — no reconstruction needed, this session did the audit itself.
`commit_sha` is `0f9aa90` (main's HEAD when that audit's agents ran).

- [ ] **Step 1: Write the backfill JSON**

Create `/tmp/backfill-wiki-audit-643.json`:

```json
{
  "type": "run",
  "date": "2026-07-03",
  "commit_sha": "0f9aa90",
  "pr_url": "https://github.com/stphung/quest/pull/643",
  "agent_count": 4,
  "scope": ["Achievements.md", "Ascension.md", "Challenges.md", "Combat.md", "Controls-and-UI.md", "Dungeons.md", "Equipment.md", "Fishing.md", "Getting-Started.md", "Haven.md", "Home.md", "Loom-of-Worlds.md", "Power-Cores.md", "Prestige.md", "Soulforge.md", "Stormbreaker-Path.md", "Stormglass.md", "Strategy-Guide.md", "The-Deep.md", "Zones-and-Progression.md"],
  "findings": [
    {"location": "Getting-Started.md:111, Strategy-Guide.md:77,472", "claim": "Challenge minigame count documented as 12", "correct_value": "14 (src/challenges/menu.rs ChallengeType has 14 variants)", "severity": "HIGH", "category": "stale-count", "auto_fixed": true},
    {"location": "Strategy-Guide.md:365", "claim": "Fracture Z30 enemies ~700x stronger than Z11", "correct_value": "~7,500x (1.6^19 per FRACTURE_ZONE_STAT_MULTIPLIER)", "severity": "MEDIUM", "category": "stale-derived-number", "auto_fixed": true},
    {"location": "Challenges.md:170,152", "claim": "Shard Fusion win targets 512/1024/2048/4096", "correct_value": "256/512/1024/2048 (shard_fusion/types.rs)", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "Challenges.md:40,152", "claim": "Rune Deciphering slots 4-6", "correct_value": "3-5 (rune/types.rs num_slots())", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "Power-Cores.md achievement table", "claim": "Power Core I/II/V/VI points 25/50/100/250", "correct_value": "10/25/250/500 (achievements/data.rs)", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "Power-Cores.md", "claim": "Power Core achievements categorized as Progression", "correct_value": "Categorized as The Deep (AchievementCategory::Deep)", "severity": "MEDIUM", "category": "wrong-category", "auto_fixed": true},
    {"location": "Achievements.md Deep section", "claim": "Power Core achievements missing from Achievements page", "correct_value": "Added 6 Power Core achievement rows", "severity": "MEDIUM", "category": "missing-entries", "auto_fixed": true},
    {"location": "The-Deep.md Mission Types table, 'The Gateway' section", "claim": "Gateway Expedition duration 48h", "correct_value": "72h / 3 days (GatewayExpedition => 259_200 secs)", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "The-Deep.md Duration Modifiers + Familiarity tables", "claim": "Familiarity reductions Mapped -10%, Familiar -20%, Mastered -30%", "correct_value": "-15%/-30%/-45% (duration_factor())", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "The-Deep.md Duration Modifiers table + Advanced Tips", "claim": "Saboteur in squad -10%/-15%, Overpowered squad >150% power -10%, 30-minute mission duration floor", "correct_value": "None of these exist in effective_duration_secs(); real pipeline is Familiarity x Outpost x Bridge (-2%/bridged layer, capped -30%), which was missing from the docs entirely", "severity": "MEDIUM", "category": "fabricated-mechanic", "auto_fixed": true},
    {"location": "Soulforge.md enhancement table + Soul Tithe section", "claim": "Soul Tithe unavailable for +8/+9/+10 (no guaranteed path)", "correct_value": "Available at 25/85/750 PR respectively (ENHANCEMENT_SOUL_TITHE_COSTS)", "severity": "HIGH", "category": "stale-mechanic", "auto_fixed": true},
    {"location": "Dungeons.md Room Types table", "claim": "Combat 60% / Treasure 20% spawn rate", "correct_value": "Treasure is a fixed count by dungeon size (1-8); Combat is all remaining rooms — no probability roll exists", "severity": "MEDIUM", "category": "mischaracterized-mechanic", "auto_fixed": true},
    {"location": "Equipment.md Item Rarity table", "claim": "Attribute Rolls column implies rarity increases attribute count (1 to 4-6 attributes)", "correct_value": "Every item always rolls 1-3 attributes regardless of rarity; the 1/1-2/2-3/3-4/4-6 values are base attribute value at ilvl 10, not count (items/generation.rs)", "severity": "MEDIUM", "category": "mischaracterized-mechanic", "auto_fixed": true},
    {"location": "Loom-of-Worlds.md pattern arc headers", "claim": "Patterns grouped as Teaching Arc / Mastery Arc / Endgame Arc", "correct_value": "In-game chapter names: Chapter I: The Awakening / II: The Deepening / III: The Unraveling (loom/discovery.rs)", "severity": "LOW", "category": "stale-naming", "auto_fixed": true},
    {"location": "Loom-of-Worlds.md Woven Patterns intro", "claim": "28 total patterns, no mention of a 29th", "correct_value": "A 29th pattern, The Eternal Weave, exists as a never-completing endgame sink, excluded from the 28 completable count", "severity": "LOW", "category": "missing-mechanic", "auto_fixed": true},
    {"location": "Loom-of-Worlds.md Extractor Upgrades", "claim": "50% of the Extractor's buffer is consumed on upgrade", "correct_value": "50% of buffer capacity is consumed, not current buffer contents (logic.rs:155)", "severity": "LOW", "category": "imprecise-wording", "auto_fixed": true},
    {"location": "Controls-and-UI.md minigame controls section", "claim": "Only 12 of 14 minigames have documented controls (Runic Lights, Vault Warden missing)", "correct_value": "Added control sections for both, verified against src/challenges/runic_lights/ and src/challenges/vault_warden/ input handling", "severity": "MEDIUM", "category": "missing-entries", "auto_fixed": true}
  ]
}
```

- [ ] **Step 2: Run the log script**

```bash
scripts/audit-eval-log.sh wiki-audit /tmp/backfill-wiki-audit-643.json
```

Expected: `Logged 1 entry to .../wiki-audit.jsonl (now 1 lines)`

- [ ] **Step 3: Verify the threshold**

```bash
scripts/audit-eval-check.sh wiki-audit
```

Expected: `SKIP: 1/5`

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/meta-audit/history/wiki-audit.jsonl
git commit -m "chore(meta-audit): backfill wiki-audit history from PR #643"
```

---

## Task 10: Backfill `doc-audit.jsonl`

**Files:**
- Modify: `.claude/skills/meta-audit/history/doc-audit.jsonl` (via script, not direct edit)

Backfills 5 historical doc-audit PRs, reconstructed from their PR bodies (each already
contains a structured findings breakdown). `commit_sha` for each is that PR's merge
commit.

- [ ] **Step 1: Write the 5 backfill JSON files**

Create `/tmp/backfill-doc-audit-613.json`:

```json
{
  "type": "run",
  "date": "2026-07-02",
  "commit_sha": "87185aef40e2c761b0bd28dc91f3fef5360acf55",
  "pr_url": "https://github.com/stphung/quest/pull/613",
  "agent_count": 5,
  "scope": ["CLAUDE.md", "docs/secondary-systems.md", "docs/balancing.md", "docs/system-design.md", "docs/core-systems.md", "docs/challenge-minigames.md", "src/*/CLAUDE.md"],
  "findings": [
    {"location": "docs/secondary-systems.md, docs/balancing.md, docs/system-design.md", "claim": "Ascension VII-X costs documented as 500 + 75*(level-6) (~575-800 PR)", "correct_value": "LOOM_ASCENSION_COSTS = [1500, 4000, 8000, 15000] (src/ascension/types.rs)", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "docs/core-systems.md", "claim": "MAX_ASCENSION_LEVEL documented as 6", "correct_value": "10", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "docs/challenge-minigames.md", "claim": "Runic Lights and Vault Warden missing; challenge count 12; discovery weight 218", "correct_value": "14 challenges; discovery weight 256", "severity": "HIGH", "category": "missing-entries", "auto_fixed": true},
    {"location": "docs (Deep power thresholds)", "claim": "L19/L25 rows and Void formula stale (+80/layer over base 930)", "correct_value": "+60/layer over base 700 (src/deep/layers.rs)", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "docs (Deep familiarity)", "claim": "Familiarity duration factors -10/-20/-30%", "correct_value": "-15/-30/-45% (duration_factor())", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "src/deep/CLAUDE.md", "claim": "apply_duration_modifiers, MIN_MISSION_DURATION_SECS (30-min floor), Saboteur/Overpower duration modifiers documented as real APIs", "correct_value": "None of these exist; real pipeline is effective_duration_secs (Familiarity, Outpost -25%, Bridge -2%/layer cap -30%)", "severity": "HIGH", "category": "phantom-api", "auto_fixed": true},
    {"location": "various CLAUDE.md counts", "claim": "achievements 232/213, titles 63, TickEvent variants 45, update-check interval 30min", "correct_value": "achievements 240, titles 64, TickEvent variants 48, update-check interval 15min", "severity": "HIGH", "category": "stale-count", "auto_fixed": true},
    {"location": "docs/challenge-minigames.md Shard Fusion", "claim": "Shard Fusion targets 512-4096", "correct_value": "256-2048", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "achievements/CLAUDE.md", "claim": "VaultWardenJourneyman worth 15 points (off-tier, not fixed — flagged for maintainer)", "correct_value": "N/A — flagged for review, not corrected in docs pass", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false}
  ]
}
```

Create `/tmp/backfill-doc-audit-617.json`:

```json
{
  "type": "run",
  "date": "2026-07-02",
  "commit_sha": "5b4e8a9b7f8d1ecb4af9a5fc618cb5e0f4a934a3",
  "pr_url": "https://github.com/stphung/quest/pull/617",
  "agent_count": 5,
  "scope": ["CLAUDE.md", "docs/core-systems.md", "docs/system-design.md", "src/*/CLAUDE.md"],
  "findings": [
    {"location": "CLAUDE.md docs", "claim": "PowerCoreState struct documented", "correct_value": "Doesn't exist — state lives on DeepPersistent.power_core_last_granted", "severity": "HIGH", "category": "phantom-api", "auto_fixed": true},
    {"location": "character/CLAUDE.md", "claim": "create_character(name) function documented", "correct_value": "Creation flows through creation.rs::process_creation_input()", "severity": "HIGH", "category": "phantom-api", "auto_fixed": true},
    {"location": "combat/CLAUDE.md", "claim": "Divine Bulwark DR documented in player's outgoing damage pipeline; ascension_multiplier missing from Combat Flow", "correct_value": "Bulwark DR only applies to enemy defense (incoming) pipeline; ascension_multiplier added to Combat Flow", "severity": "HIGH", "category": "wrong-pipeline", "auto_fixed": true},
    {"location": "docs/core-systems.md, docs/system-design.md", "claim": "Tick pipeline documented as old 14-stage", "correct_value": "Actual ~21-stage pipeline (Loom tick, HUD decay, Deep/fracture/pattern stages, Power Cores)", "severity": "HIGH", "category": "stale-pipeline", "auto_fixed": true},
    {"location": "zones/CLAUDE.md", "claim": "ZoneProgression.defeated_bosses/unlocked_zones documented as Vec", "correct_value": "Actually BTreeSet", "severity": "MEDIUM", "category": "stale-type", "auto_fixed": true},
    {"location": "ascension/CLAUDE.md", "claim": "can_ascend/ascend signatures missing completed_patterns parameter", "correct_value": "Added missing parameter to documented signatures", "severity": "MEDIUM", "category": "stale-signature", "auto_fixed": true},
    {"location": "docs (challenge constants)", "claim": "Rune challenge slot count 4-6, minigame handler count 12", "correct_value": "3-5 slots, 14 handlers", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "root CLAUDE.md skills table", "claim": "Missing add-challenge/balance-sim/clean-workspace skills", "correct_value": "Added all 3 to skills table", "severity": "MEDIUM", "category": "missing-section", "auto_fixed": true}
  ]
}
```

Create `/tmp/backfill-doc-audit-627.json`:

```json
{
  "type": "run",
  "date": "2026-07-02",
  "commit_sha": "640b4e9a1f4138682df682328a17525eb0e144c1",
  "pr_url": "https://github.com/stphung/quest/pull/627",
  "agent_count": 5,
  "scope": ["CLAUDE.md", "src/*/CLAUDE.md"],
  "findings": [
    {"location": "character/CLAUDE.md", "claim": "perform_prestige() documented as preserving equipment", "correct_value": "Code does a complete equipment wipe (prestige_actions.rs:42); partial preservation only via perform_prestige_with_vault()", "severity": "HIGH", "category": "wrong-mechanic", "auto_fixed": true},
    {"location": "character/CLAUDE.md DerivedStats", "claim": "DerivedStats documented with a 'prestige multiplier from CHA' field", "correct_value": "No such struct field exists — computed separately via core::xp::prestige_multiplier()", "severity": "MEDIUM", "category": "phantom-api", "auto_fixed": true},
    {"location": "combat/CLAUDE.md", "claim": "boss_weapon_blocked() cited as living in zones/progression.rs", "correct_value": "Actually in zones/gates.rs", "severity": "MEDIUM", "category": "stale-citation", "auto_fixed": true},
    {"location": "history/CLAUDE.md", "claim": "validate_token() function documented", "correct_value": "Doesn't exist — real functions are github_get_username() and github_list_repos()", "severity": "MEDIUM", "category": "phantom-api", "auto_fixed": true},
    {"location": "loom/CLAUDE.md", "claim": "29th 'eternal' Loom pattern undocumented", "correct_value": "Documented discovery.rs eternal_pattern() as a never-completing endgame sink", "severity": "LOW", "category": "missing-mechanic", "auto_fixed": true},
    {"location": "root CLAUDE.md CI/CD section", "claim": "PR checks documented as running via scripts/ci-checks.sh", "correct_value": "PR checks run as six independent jobs in ci.yml; ci-checks.sh only mirrors them for local make check; noted a real coverage-exclusion drift between the two", "severity": "MEDIUM", "category": "stale-process-doc", "auto_fixed": true},
    {"location": "achievements/CLAUDE.md", "claim": "VaultWardenJourneyman worth 15 points doesn't match documented 5/10/25 tier pattern (flagged, not a docs fix)", "correct_value": "N/A — flagged for maintainer, balance call not docs call", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false}
  ]
}
```

Create `/tmp/backfill-doc-audit-632.json`:

```json
{
  "type": "run",
  "date": "2026-07-03",
  "commit_sha": "fe03234e65aaa3b82c11f08a619d95af8230afbc",
  "pr_url": "https://github.com/stphung/quest/pull/632",
  "agent_count": 6,
  "scope": ["CLAUDE.md", "README.md", ".claude/skills/doc-audit/SKILL.md"],
  "findings": [
    {"location": "CLAUDE.md tagline", "claim": "Tagline didn't reflect 50 zones + layered endgame systems", "correct_value": "Updated tagline", "severity": "LOW", "category": "stale-description", "auto_fixed": true},
    {"location": "CLAUDE.md module table, Key Constants, Simulators", "claim": "Vessel/Voyage (Act 2) landed in #628/#630 but completely undocumented", "correct_value": "Added module table entry, Key Constants (250k PR launch burn, 28 patterns + Ascension X, ACT2_ENABLED kill-switch, QUEST_ACT2=1 override), voyage_simulator entry", "severity": "HIGH", "category": "missing-module-doc", "auto_fixed": true},
    {"location": "CLAUDE.md damage pipeline description", "claim": "Bulwark DR documented as applying in player damage output pipeline", "correct_value": "Applies in enemy->player defense (incoming) pipeline (combat/player_attack.rs vs enemy_attack.rs:41-46)", "severity": "HIGH", "category": "wrong-pipeline", "auto_fixed": true},
    {"location": "README.md zones/fishing/offline/drops", "claim": "10 zones, 30 fishing ranks, 50% offline rate, drop rates 30%+5%", "correct_value": "50 zones, 40 fishing ranks, 25% offline rate, drop rates 15%+1% capped 25%", "severity": "HIGH", "category": "stale-constant", "auto_fixed": true},
    {"location": "README.md Prestige section", "claim": "XP multiplier documented as 1.5x compounding per rank; tier ladder order wrong", "correct_value": "1 + 0.5 x rank^0.7 diminishing returns; base 20 + 5/rank attribute caps; ladder is Diamond(P5)->Emerald->Sapphire->Ruby->Obsidian->Celestial(P10)", "severity": "HIGH", "category": "wrong-mechanic", "auto_fixed": true},
    {"location": "README.md Controls", "claim": "Quit key documented as Q", "correct_value": "Esc, on both screens; documented full real key map", "severity": "MEDIUM", "category": "wrong-keybinding", "auto_fixed": true},
    {"location": "README.md Zones", "claim": "Zone 11 / Fracture zone range undocumented or wrong", "correct_value": "Zone 11 is The Expanse (unlocks after Zone 10 at P25); Fracture zones are 12-30", "severity": "MEDIUM", "category": "stale-constant", "auto_fixed": true},
    {"location": "README.md Items", "claim": "Attribute-range table with numbers matching nothing in code; affix counts wrong (Rare/Epic)", "correct_value": "Removed unsubstantiated table; corrected affix counts (Rare 2-3, Epic 3-4)", "severity": "MEDIUM", "category": "stale-constant", "auto_fixed": true},
    {"location": "README.md Features list", "claim": "8 shipped systems missing from Features list (Soulforge, Ascension, Deep, Loom, Stormglass, Power Cores, Time Vault)", "correct_value": "Added all 8; confirmed dark-shipped Vessel deliberately not advertised", "severity": "HIGH", "category": "missing-section", "auto_fixed": true},
    {"location": ".claude/skills/doc-audit/SKILL.md", "claim": "README.md not covered by any audit skill", "correct_value": "Added Agent 6 — README (player-facing) to doc-audit's Phase 1", "severity": "MEDIUM", "category": "scope-gap", "auto_fixed": true}
  ]
}
```

Create `/tmp/backfill-doc-audit-637.json`:

```json
{
  "type": "run",
  "date": "2026-07-03",
  "commit_sha": "ed7bb2a32ad4d2c295733e7179853eb16911dae3",
  "pr_url": "https://github.com/stphung/quest/pull/637",
  "agent_count": 6,
  "scope": ["CLAUDE.md", "docs/*.md", "src/*/CLAUDE.md", "README.md"],
  "findings": [
    {"location": "src/vessel/ (no CLAUDE.md)", "claim": "Module had zero documentation", "correct_value": "Added full CLAUDE.md (files, key types, kill-switch mechanics, integration points, constants, gotchas)", "severity": "HIGH", "category": "missing-module-doc", "auto_fixed": true},
    {"location": "various CLAUDE.md", "claim": "TickEvent 48 variants, AchievementId 213, title count 63, zone count 30 (docs/system-design.md), AchievementCategory 8 (missing Loom)", "correct_value": "TickEvent 50, AchievementId 240, title count 64, zone count 50, AchievementCategory 9 (Loom added)", "severity": "HIGH", "category": "stale-count", "auto_fixed": true},
    {"location": "docs (simulator CSV)", "claim": "CSV simulator output documented with 11 columns", "correct_value": "19 columns documented", "severity": "MEDIUM", "category": "stale-constant", "auto_fixed": true},
    {"location": "haven/enhancement/ascension/stormglass/power_cores/god_items/achievements/deep CLAUDE.md", "claim": "Stale file/function attributions from the old core/game_logic.rs split", "correct_value": "Updated to tick_stages.rs/xp.rs/power_rating.rs/offline.rs/enemy_spawning.rs/discoveries.rs", "severity": "MEDIUM", "category": "stale-citation", "auto_fixed": true},
    {"location": "input/CLAUDE.md, ui/CLAUDE.md, main_helpers/CLAUDE.md", "claim": "Vessel/Act 2 wiring (voyage_input.rs, vessel_scene.rs, voyage_scene.rs, GameOverlay::Vessel/VesselDiscovery, V hotkey, Act 2 tick stage) undocumented", "correct_value": "Documented all of the above", "severity": "HIGH", "category": "missing-section", "auto_fixed": true},
    {"location": "README.md Fracture zone range", "claim": "Internal contradiction: 11-30 in one place, 12-30 in another", "correct_value": "Fixed to consistently read 12-30", "severity": "MEDIUM", "category": "internal-contradiction", "auto_fixed": true},
    {"location": "README.md Stormglass, Character Select controls, project structure tree", "claim": "Stormglass earning description wrong; misleading per-challenge prestige gate note; missing bug-report hotkey; project tree missing main_helpers/ and bin/", "correct_value": "Corrected Stormglass description, removed misleading note, added hotkey and tree entries", "severity": "MEDIUM", "category": "stale-description", "auto_fixed": true},
    {"location": "src/deep/CLAUDE.md", "claim": "Referenced nonexistent 'Abyssal' item affixes", "correct_value": "Removed fabricated integration point", "severity": "MEDIUM", "category": "fabricated-content", "auto_fixed": true}
  ]
}
```

- [ ] **Step 2: Run the log script for each, in date order**

```bash
scripts/audit-eval-log.sh doc-audit /tmp/backfill-doc-audit-613.json
scripts/audit-eval-log.sh doc-audit /tmp/backfill-doc-audit-617.json
scripts/audit-eval-log.sh doc-audit /tmp/backfill-doc-audit-627.json
scripts/audit-eval-log.sh doc-audit /tmp/backfill-doc-audit-632.json
scripts/audit-eval-log.sh doc-audit /tmp/backfill-doc-audit-637.json
```

Expected: 5 lines each ending `(now N lines)` for N = 1, 2, 3, 4, 5.

- [ ] **Step 3: Verify the threshold**

```bash
scripts/audit-eval-check.sh doc-audit
```

Expected: `TRIGGER` (exactly 5 runs backfilled, no marker yet — this is expected and
means doc-audit is immediately eligible for its first real deep-eval once this plan
ships; that's a deliberate side effect of backfilling 5 full runs, not a bug).

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/meta-audit/history/doc-audit.jsonl
git commit -m "chore(meta-audit): backfill doc-audit history from PRs #613,617,627,632,637"
```

---

## Task 11: Backfill `test-audit.jsonl`

**Files:**
- Modify: `.claude/skills/meta-audit/history/test-audit.jsonl` (via script, not direct edit)

- [ ] **Step 1: Write the 3 backfill JSON files**

Create `/tmp/backfill-test-audit-593.json`:

```json
{
  "type": "run",
  "date": "2026-04-15",
  "commit_sha": "851bed7fb0ef929d84646722497c117442529d4d",
  "pr_url": "https://github.com/stphung/quest/pull/593",
  "agent_count": 3,
  "scope": ["tests/game_tick_tests/", "tests/combat_tests/", "tests/character_tests/", "tests/zone_tests/", "tests/fishing_tests/", "tests/deep_tests/", "tests/haven_tests/", "tests/enhancement_tests/", "tests/stormglass_tests/", "tests/item_tests/", "tests/achievement_tests/", "tests/history_tests/", "tests/misc_tests/"],
  "findings": []
}
```

Create `/tmp/backfill-test-audit-622.json`:

```json
{
  "type": "run",
  "date": "2026-07-02",
  "commit_sha": "e7d548fc1b53b6dcebf0bfdf69f0c537476ca95f",
  "pr_url": "https://github.com/stphung/quest/pull/622",
  "agent_count": 3,
  "scope": ["tests/game_tick_tests/", "tests/combat_tests/", "tests/character_tests/", "tests/zone_tests/", "tests/fishing_tests/", "tests/deep_tests/", "tests/haven_tests/", "tests/enhancement_tests/", "tests/stormglass_tests/", "tests/item_tests/", "tests/achievement_tests/", "tests/history_tests/", "tests/misc_tests/"],
  "findings": [
    {"location": "tests/game_tick_tests/game_loop_orchestration_test.rs", "claim": "Two Haven-discovery tests looped 100x over P=0-by-structure outcomes", "correct_value": "Trimmed to 5 iterations", "severity": "LOW", "category": "excessive-loop", "auto_fixed": true},
    {"location": "tests/item_tests/item_generation_scoring_test.rs", "claim": "test_boss_drop_not_common_nor_mythic looped 500x over a structurally-impossible arm", "correct_value": "Trimmed to 20", "severity": "LOW", "category": "excessive-loop", "auto_fixed": true},
    {"location": "tests/item_tests/item_combat_coverage_test.rs", "claim": "Two P0-prestige-gate discovery tests looped 100x", "correct_value": "Trimmed to 10", "severity": "LOW", "category": "excessive-loop", "auto_fixed": true},
    {"location": "tests/item_tests/item_pipeline_test.rs", "claim": "test_try_drop_item_produces_valid_equippable_items looped 1000x at ~20% drop rate", "correct_value": "Trimmed to 100 (odds of zero hits ~1 in 5 billion at 100 trials)", "severity": "LOW", "category": "excessive-loop", "auto_fixed": true},
    {"location": "tests/misc_tests/prestige_cycle_test.rs", "claim": "Dead safety assertion tick < 199_999 could never fire given actual loop bound of 20_000", "correct_value": "Changed to tick < 19_999 so it's a real guard again", "severity": "MEDIUM", "category": "dead-assertion", "auto_fixed": true},
    {"location": "production drop/dungeon/enemy-gen functions", "claim": "try_drop_from_mob, try_drop_from_boss, generate_dungeon/generate_maze, enemy-generation call rand::rng() internally instead of accepting injectable RNG (flagged, not fixed)", "correct_value": "N/A — requires threading &mut impl Rng through production signatures; flagged for follow-up", "severity": "MEDIUM", "category": "flagged-not-fixed", "auto_fixed": false},
    {"location": "tests/deep_tests/deep_mission_test.rs::test_offline_resolution_leaves_running_mission_active", "claim": "Test is tautological because resolve_offline_missions() uses real Utc::now() internally (flagged, not fixed)", "correct_value": "N/A — flagged for follow-up", "severity": "MEDIUM", "category": "flagged-not-fixed", "auto_fixed": false},
    {"location": "game_tick_tests, enhancement_tests, stormglass_tests (~40 loops)", "claim": "5,000-20,000 iteration brute-force loops to hit rare events instead of known-good seeds (flagged, not fixed)", "correct_value": "N/A — shrinking safely requires running code to find good seeds per-test; flagged for follow-up given scope", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false}
  ]
}
```

Create `/tmp/backfill-test-audit-639.json`:

```json
{
  "type": "run",
  "date": "2026-07-03",
  "commit_sha": "d4992a665d98d260666771f77847848b7ae4f80b",
  "pr_url": "https://github.com/stphung/quest/pull/639",
  "agent_count": 3,
  "scope": ["tests/game_tick_tests/", "tests/combat_tests/", "tests/character_tests/", "tests/zone_tests/", "tests/fishing_tests/", "tests/deep_tests/", "tests/haven_tests/", "tests/enhancement_tests/", "tests/stormglass_tests/", "tests/item_tests/", "tests/achievement_tests/", "tests/history_tests/", "tests/misc_tests/"],
  "findings": [
    {"location": "haven tests: test_is_modal_ready_before_500ms_returns_false", "claim": "Relies on real elapsed wall-clock time between unlock() and readiness check being under 500ms", "correct_value": "Now explicitly sets accumulation_start, matching sibling test pattern in same file", "severity": "HIGH", "category": "timing-race", "auto_fixed": true},
    {"location": "test_full_pipeline_generate_power_equip, test_dungeon_size_scaling", "claim": "Used unseeded RNG for item power / dungeon size comparisons", "correct_value": "Replaced with seeded RNG or a direct assertion proven safe by the generation formula", "severity": "HIGH", "category": "unseeded-rng", "auto_fixed": true},
    {"location": "several 'structurally blocked gate' tests", "claim": "Iteration counts 100/500/1000 on preconditions checked before any RNG roll (P=0 by code, not probability)", "correct_value": "Reduced to 5-100 iterations, matching convention elsewhere in these files", "severity": "LOW", "category": "excessive-loop", "auto_fixed": true},
    {"location": "Monte Carlo Haven-discovery test", "claim": "800 ticks per seed gave a loose confidence margin", "correct_value": "Bumped to 1000 ticks per seed for ~98% single-shot expected-discovery margin", "severity": "MEDIUM", "category": "tight-probabilistic-margin", "auto_fixed": true}
  ]
}
```

- [ ] **Step 2: Run the log script for each, in date order**

```bash
scripts/audit-eval-log.sh test-audit /tmp/backfill-test-audit-593.json
scripts/audit-eval-log.sh test-audit /tmp/backfill-test-audit-622.json
scripts/audit-eval-log.sh test-audit /tmp/backfill-test-audit-639.json
```

Expected: 3 lines each ending `(now N lines)` for N = 1, 2, 3.

- [ ] **Step 3: Verify the threshold**

```bash
scripts/audit-eval-check.sh test-audit
```

Expected: `SKIP: 3/5`

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/meta-audit/history/test-audit.jsonl
git commit -m "chore(meta-audit): backfill test-audit history from PRs #593,622,639"
```

---

## Task 12: Backfill `perf-audit.jsonl`

**Files:**
- Modify: `.claude/skills/meta-audit/history/perf-audit.jsonl` (via script, not direct edit)

- [ ] **Step 1: Write the 3 backfill JSON files**

Create `/tmp/backfill-perf-audit-562.json`:

```json
{
  "type": "run",
  "date": "2026-03-15",
  "commit_sha": "6c231aba7bf7725a0f6018011c43df9a3916daf9",
  "pr_url": "https://github.com/stphung/quest/pull/562",
  "agent_count": 3,
  "scope": ["src/core/", "src/ui/", "src/achievements/", "src/deep/"],
  "findings": [
    {"location": "run_combat, process_dungeon_events, process_fishing_tick", "claim": "6 per-tick linear equipment scans for god item bonuses", "correct_value": "Cached in new CachedGodItemBonuses struct on GameState, invalidated alongside derived stats", "severity": "MEDIUM", "category": "linear-scan", "auto_fixed": true},
    {"location": "process_zone_achievements", "claim": "Linear zone lookup by name on boss defeat", "correct_value": "O(1) via new old_zone_id field on BossDefeatResult variants", "severity": "LOW", "category": "linear-scan", "auto_fixed": true},
    {"location": "current_enemy_name.clone() per combat event", "claim": "Clone flagged", "correct_value": "Deferred — only cloned when events actually fire", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false},
    {"location": "fishing message format!() allocations", "claim": "Allocations flagged", "correct_value": "Deferred — only when fishing active", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false}
  ]
}
```

Create `/tmp/backfill-perf-audit-565.json`:

```json
{
  "type": "run",
  "date": "2026-03-15",
  "commit_sha": "9f6e508981a4740e6c2e71755a9b6219458d9379",
  "pr_url": "https://github.com/stphung/quest/pull/565",
  "agent_count": 3,
  "scope": ["src/core/", "src/ui/", "src/achievements/", "src/deep/"],
  "findings": [
    {"location": "tick.rs:186-187", "claim": "loom_zone_cap_for_patterns(completed_pattern_count()) called unconditionally every tick, iterating 28 patterns", "correct_value": "Only recomputed when loom_changed is true", "severity": "MEDIUM", "category": "redundant-computation", "auto_fixed": true},
    {"location": "tick_stages.rs:335", "claim": "Enemy name clone on combat events flagged", "correct_value": "Acceptable — only on events, not per-tick", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false},
    {"location": "tick_stages.rs:741,818", "claim": "ascension_combat_multiplier() called twice per tick flagged", "correct_value": "Trivially cheap (2 comparisons) — no fix needed", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false},
    {"location": "benches/game_tick.rs", "claim": "Benchmarks used deprecated game_tick()", "correct_value": "Modernized to game_tick_with_context(), added LoomState to benchmark setup", "severity": "LOW", "category": "stale-benchmark", "auto_fixed": true}
  ]
}
```

Create `/tmp/backfill-perf-audit-595.json`:

```json
{
  "type": "run",
  "date": "2026-04-15",
  "commit_sha": "1dd73953d818a2ddc0c73f67661a6920967e3361",
  "pr_url": "https://github.com/stphung/quest/pull/595",
  "agent_count": 3,
  "scope": ["src/core/", "src/ui/", "src/achievements/", "src/deep/"],
  "findings": [
    {"location": "tick_stages.rs:331 process_combat_events", "claim": "Unconditional String clone of enemy name before empty event loop check", "correct_value": "Clone lazily per-arm, eliminating allocation on ticks with no combat events", "severity": "MEDIUM", "category": "per-tick-allocation", "auto_fixed": true},
    {"location": "unlock.rs:57 check_milestones", "claim": "Early-exit pattern checked", "correct_value": "Already optimized — no change needed", "severity": "LOW", "category": "no-change-needed", "auto_fixed": false},
    {"location": "tick_stages.rs:935,1045", "claim": "chrono::Utc::now() called twice per tick", "correct_value": "Separate fn boundaries — unfixable without an API change; flagged", "severity": "LOW", "category": "flagged-not-fixed", "auto_fixed": false},
    {"location": "src/ui/ render path", "claim": "Per-frame format!() allocations checked", "correct_value": "No hot-path format!() found in render path", "severity": "LOW", "category": "no-change-needed", "auto_fixed": false}
  ]
}
```

- [ ] **Step 2: Run the log script for each, in date order**

```bash
scripts/audit-eval-log.sh perf-audit /tmp/backfill-perf-audit-562.json
scripts/audit-eval-log.sh perf-audit /tmp/backfill-perf-audit-565.json
scripts/audit-eval-log.sh perf-audit /tmp/backfill-perf-audit-595.json
```

Expected: 3 lines each ending `(now N lines)` for N = 1, 2, 3.

- [ ] **Step 3: Verify the threshold**

```bash
scripts/audit-eval-check.sh perf-audit
```

Expected: `SKIP: 3/5`

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/meta-audit/history/perf-audit.jsonl
git commit -m "chore(meta-audit): backfill perf-audit history from PRs #562,565,595"
```

---

## Task 13: Backfill `dependency-audit.jsonl`

**Files:**
- Modify: `.claude/skills/meta-audit/history/dependency-audit.jsonl` (via script, not direct edit)

Only one clean historical dependency-audit run was found in git history (PR #563). Per
the design doc, fewer than 5 backfilled runs is expected and fine — the first deep-eval
for this skill will just run on-demand with a smaller sample rather than waiting for the
auto-trigger.

- [ ] **Step 1: Write the backfill JSON**

Create `/tmp/backfill-dependency-audit-563.json`:

```json
{
  "type": "run",
  "date": "2026-03-15",
  "commit_sha": "27d5038456a7611bf3ac1517c6ef66b3b91a11ae",
  "pr_url": "https://github.com/stphung/quest/pull/563",
  "agent_count": 2,
  "scope": ["Cargo.toml", "Cargo.lock"],
  "findings": [
    {"location": "Cargo.toml uuid", "claim": "uuid pinned at 1.21, 1.22 available (compatible)", "correct_value": "Bumped to 1.22", "severity": "LOW", "category": "patch-bump", "auto_fixed": true},
    {"location": "Cargo.toml criterion", "claim": "criterion pinned at 0.5, latest 0.8.2 (major version, breaking API)", "correct_value": "N/A — flagged for review; upgrading requires updating benches/game_tick.rs", "severity": "MEDIUM", "category": "flagged-not-fixed", "auto_fixed": false}
  ]
}
```

- [ ] **Step 2: Run the log script**

```bash
scripts/audit-eval-log.sh dependency-audit /tmp/backfill-dependency-audit-563.json
```

Expected: `Logged 1 entry to .../dependency-audit.jsonl (now 1 lines)`

- [ ] **Step 3: Verify the threshold**

```bash
scripts/audit-eval-check.sh dependency-audit
```

Expected: `SKIP: 1/5`

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/meta-audit/history/dependency-audit.jsonl
git commit -m "chore(meta-audit): backfill dependency-audit history from PR #563"
```

---

## Task 14: Final verification

**Files:** none (verification only)

- [ ] **Step 1: Re-run both test scripts to confirm nothing regressed**

```bash
./scripts/audit-eval-log.test.sh && ./scripts/audit-eval-check.test.sh
```

Expected: `All tests passed.` from both, exit code 0.

- [ ] **Step 2: Confirm the threshold for every skill matches the backfilled counts**

```bash
for s in perf-audit test-audit doc-audit wiki-audit dependency-audit; do
  echo "$s: $(scripts/audit-eval-check.sh "$s")"
done
```

Expected:
```
perf-audit: SKIP: 3/5
test-audit: SKIP: 3/5
doc-audit: TRIGGER
wiki-audit: SKIP: 1/5
dependency-audit: SKIP: 1/5
```

- [ ] **Step 3: Confirm every backfilled log line is valid JSON**

```bash
for f in .claude/skills/meta-audit/history/*.jsonl; do
  echo "=== $f ==="
  jq empty "$f" && echo "OK: all lines valid" || echo "FAIL: invalid JSON in $f"
done
```

Expected: `OK: all lines valid` for all 5 files (note: `jq empty` on a multi-line file
with one JSON object per line validates each line's syntax as `jq` streams the input).

- [ ] **Step 4: Run `make check`**

```bash
make check
```

Expected: all checks pass. This change touches no Rust code, so this mainly confirms
nothing was accidentally broken (e.g. a stray file didn't trip `cargo fmt --check` or
similar) — but it's the project's standard pre-push gate, so run it anyway.

- [ ] **Step 5: Commit anything outstanding, if `make check` needed fixes**

```bash
git status -s
```

If clean, nothing to do. If `make check` required a fix, commit it separately with a
clear message before moving to Task 15.

---

## Task 15: Ship

**Files:** none (this task pushes and opens the PR)

- [ ] **Step 1: Push the branch**

```bash
git push -u origin HEAD
```

- [ ] **Step 2: Open the PR**

```bash
gh pr create --title "feat: add meta-audit skill for evaluating audit skill performance" --body "$(cat <<'EOF'
## Summary

Adds `meta-audit`, a new skill that evaluates the five domain audit skills (`perf-audit`,
`test-audit`, `doc-audit`, `wiki-audit`, `dependency-audit`) for finding accuracy and
scope staleness, and safely improves their `SKILL.md` files over time.

- Two new deterministic scripts (`scripts/audit-eval-log.sh`, `scripts/audit-eval-check.sh`)
  back a per-skill JSONL history log under `.claude/skills/meta-audit/history/`, with test
  coverage in `scripts/*.test.sh`.
- Each of the five domain skills gets a new `## Log This Run` step and (for four of them)
  a `scope-universe` comment `meta-audit` uses to detect scope drift mechanically.
- The new `meta-audit/SKILL.md` re-verifies past findings against source as of the commit
  they were made against (never mutating the working tree), checks for scope gaps/rot,
  and ships safe fixes via PR — same auto-fix/flag-for-review split every other audit
  skill already uses.
- Backfilled real historical data (15 run entries across 4 skills, reconstructed from
  past audit PR bodies) so the mechanism has data from day one instead of a cold start.
  `doc-audit` immediately crosses the 5-run threshold as a result — its first deep-eval
  can run right after this merges.

Design doc: `docs/superpowers/specs/2026-07-03-meta-audit-skill-design.md`
Plan: `docs/superpowers/plans/2026-07-03-meta-audit-skill.md`

## Test plan

- [x] `./scripts/audit-eval-log.test.sh` — 6 assertions pass
- [x] `./scripts/audit-eval-check.test.sh` — 5 assertions pass
- [x] Every backfilled history file is valid line-delimited JSON
- [x] `scripts/audit-eval-check.sh <skill>` reports the expected threshold state for all 5 skills
- [x] `make check` passes

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Report the PR URL to the user**

This plan does not auto-merge. Report the PR URL and let the user decide whether to
`/ship` it (enable automerge + watch CI) or review it manually first.

---

## Post-Merge Validation (not part of the automated task list above)

The design's Testing/Validation section calls for two additional checks that spend real
agent tokens by actually invoking `meta-audit`'s Phase 2 (adversarial re-verification).
These are deliberately **not** automated tasks above — they should be run consciously by
the user after this PR merges, not silently executed mid-plan:

1. **End-to-end proof.** Once merged, invoke the `meta-audit` skill on-demand for
   `wiki-audit` (its 1 backfilled run doesn't need to hit the 5-run auto-trigger — the
   on-demand path in `meta-audit/SKILL.md`'s `## When to Use` evaluates whatever `run`
   entries exist). This re-verifies all 17 backfilled findings from PR #643 against
   source at commit `0f9aa90` and should come back clean — both the Soulforge Soul Tithe
   costs and the Vault Warden mechanic were correct by the time that PR merged, so a
   clean result validates the re-verification pipeline works, not that it found a new bug.
2. **Adversarial sanity check.** Copy `.claude/skills/meta-audit/history/wiki-audit.jsonl`
   to a scratch location, edit one finding's `correct_value` to something deliberately
   wrong (e.g. change the Soul Tithe +10 cost from `750 PR` to `75 PR`), then run
   `meta-audit`'s Phase 2 against that scratch copy for just that one finding. Confirm it
   flags a mismatch — this proves the adversarial check actually catches errors instead
   of rubber-stamping whatever was logged.
