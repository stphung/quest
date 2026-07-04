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

For each finding, spawn one fresh **Explore** agent given **only** its `location` and `claim` —
never its logged `correct_value`. If `location` names multiple files, the agent should verify the claim against all of them. Task: independently derive what the correct value
actually was **as of that finding's `commit_sha`**, reading source read-only via
`git show <commit_sha>:<path>` only. Never run `git checkout`, `git switch`, `git reset`,
`git stash`, or edit any file — nothing in this phase may change what's in the working tree.
The agent reports its own independently-derived value plus the source location it based that on.

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

1. Append an `eval_marker` entry to the target skill's log:
   ```bash
   echo '{"type":"eval_marker","date":"<today>","pr_url":"<this PR>","runs_covered":<n>}' > /tmp/marker.json
   scripts/audit-eval-log.sh <skill-name> /tmp/marker.json
   ```
2. Branch, commit the `SKILL.md` fix(es) and the updated history log, open a PR (same
   pattern as every other audit skill — no direct commits to `main`).
3. PR body includes the full report: confirmed inaccuracies (with citations), scope
   gaps/rot, what was auto-fixed, what's flagged for review.

## Output

Report the PR URL and final status when done (use `/ship` skill).
