# meta-audit: Evaluating and Improving the Audit Skills

## Problem

Quest has five domain audit skills (`perf-audit`, `test-audit`, `doc-audit`, `wiki-audit`,
`dependency-audit`) plus an `audit` orchestrator that runs all five in parallel. Each run
produces real, valuable findings — but nothing looks back at *how well* a skill has been
performing over time. Two failure modes go unnoticed today:

1. **Finding accuracy drift.** An audit agent can assert a wrong "correct value" and a
   downstream fix agent will faithfully apply it. This already happened during the
   2026-07-03 wiki-audit: an agent described Vault Warden as a code-breaking game from its
   name alone; a later fix agent caught the mistake only by independently re-reading
   `src/challenges/vault_warden/`. Nothing forces that kind of re-check after the fact.
2. **Scope staleness.** Each skill's `SKILL.md` hardcodes which files/pages/modules its
   agents check (e.g. wiki-audit's four agents each list explicit wiki pages). As the
   codebase grows — new wiki pages, new `src/*/` modules, new test directories — those
   lists silently fall behind, and new areas go unaudited without anyone noticing.

`meta-audit` is a new skill that evaluates the five domain skills against both failure
modes and safely improves their `SKILL.md` files over time.

## Goals

- Detect confirmed inaccuracies in past audit findings by re-deriving the correct value
  independently from source, as of the commit the original audit inspected.
- Detect scope staleness: files/pages/modules that exist today but aren't covered by any
  agent in a skill's `SKILL.md`, and scope entries that no longer exist.
- Apply safe fixes (add/remove a scope entry, add a guardrail for a confirmed recurring
  false-positive pattern) automatically; flag anything that changes audit strategy more
  substantially for human review — same split doc-audit/wiki-audit already use for their
  own domain fixes.
- Keep the per-run cost near zero. Only the periodic deep-eval spawns extra agents.

## Non-goals

- Evaluating cost/efficiency (agent count, tokens, wall-clock) — not in scope for v1.
- Evaluating the `audit` orchestrator itself, or `meta-audit` evaluating itself
  (no self-referential recursion in v1).
- Adding `meta-audit` to the `audit` orchestrator's parallel fan-out — it stays a
  separate, independently-triggered skill.
- A UI/dashboard. Output is a report in a PR body, same as every other audit skill.

## Architecture

A new skill directory: `.claude/skills/meta-audit/`

```
.claude/skills/meta-audit/
├── SKILL.md                        # deep-eval flow (this is the user/auto-invocable skill)
└── history/
    ├── perf-audit.jsonl
    ├── test-audit.jsonl
    ├── doc-audit.jsonl
    ├── wiki-audit.jsonl
    └── dependency-audit.jsonl
```

Two new helper scripts under `scripts/` (deterministic, not LLM-hand-edited, so the log
can't be silently corrupted by a sloppy append):

- `scripts/audit-eval-log.sh <skill-name> <json-file>` — validates the file is well-formed
  JSON (`jq empty`), then appends it as one line to
  `.claude/skills/meta-audit/history/<skill-name>.jsonl`. Fails loudly (non-zero exit) on
  invalid JSON so the calling agent notices immediately instead of corrupting the log.
- `scripts/audit-eval-check.sh <skill-name>` — counts `"type": "run"` entries in that
  skill's log since the last `"type": "eval_marker"` entry (or since file start if none
  exist). Prints `TRIGGER` if the count is >= 5, otherwise `SKIP: <n>/5`.

Both the log directory and the two scripts are committed to git, same as any other repo
file — history is durable and diffable across commits.

## Log schema

One JSONL file per audited skill. Two entry types.

**`run`** — appended by the audited skill itself at the end of every run:

```json
{
  "type": "run",
  "date": "2026-07-03",
  "commit_sha": "0f9aa90...",
  "pr_url": "https://github.com/stphung/quest/pull/643",
  "agent_count": 4,
  "scope": ["Combat.md", "Zones-and-Progression.md", "Prestige.md", "Equipment.md", "..."],
  "findings": [
    {
      "location": "Soulforge.md:26-28",
      "claim": "Soul Tithe unavailable for +8/+9/+10",
      "correct_value": "Available at 25/85/750 PR (ENHANCEMENT_SOUL_TITHE_COSTS)",
      "severity": "HIGH",
      "category": "stale-mechanic",
      "auto_fixed": true
    }
  ]
}
```

`commit_sha` is always a commit in the main `quest` repo (never the `quest.wiki`
submodule) — every finding's `correct_value`, including wiki-audit's, is ultimately
derived from `src/` source, which only lives in the main repo. It's found via
`git merge-base HEAD origin/main` — the commit `main` was at when the audit's agents
did their read-only cross-referencing, i.e. the exact state every finding's
`correct_value` describes. It is deliberately NOT the branch's own HEAD (`git rev-parse
HEAD`) and NOT the eventual PR merge commit: for skills that modify the audited code
itself (e.g. perf-audit, test-audit), the merge commit contains the *fix*, not the
pre-fix state a finding describes. This is critical for re-verification, since it lets
`meta-audit` check the claim against source *as it was*, not against however the
codebase looks by the time the deep-eval runs. Without it, a re-check months later can't
tell "the original audit was wrong" apart from "the game changed since then."

A `run` entry is logged even when `findings` is empty (an "all clear" run, e.g. PR #593)
— it still counts toward the 5-run threshold in `audit-eval-check.sh`.

**`eval_marker`** — appended by `meta-audit` itself when a deep-eval for that skill
completes:

```json
{
  "type": "eval_marker",
  "date": "2026-07-10",
  "pr_url": "https://github.com/stphung/quest/pull/650",
  "runs_covered": 5
}
```

This is the only state `audit-eval-check.sh` needs — no separate counter file to keep in
sync.

## Changes to the five existing audit skills

Each of `perf-audit`, `test-audit`, `doc-audit`, `wiki-audit`, `dependency-audit` gets:

**1. A scope-universe comment**, placed right under the frontmatter, giving `meta-audit` a
mechanically checkable definition of "everything that could be in scope" for that skill:

| Skill | scope-universe |
|-------|-----------------|
| wiki-audit | `quest.wiki/*.md` |
| doc-audit | `CLAUDE.md`, `docs/*.md`, `src/*/CLAUDE.md`, `README.md` |
| test-audit | `tests/*_tests/` |
| perf-audit | `src/*/` (top-level module directories only — perf-audit's scope is thematic, not exhaustive; staleness here means "a new module exists with no agent assigned to it," not "every file must be listed") |
| dependency-audit | *(none — scope is inherently `Cargo.toml`'s current contents, which can't go stale by definition)* |

**2. A new `## Log This Run` section**, inserted right after the existing `## Output`
section — it can't go before `## Output`, because `## Output` is where `/ship` runs and
creates the PR, and `Log This Run` needs that PR's URL to already exist:

```markdown
## Log This Run

After verification passes, record this run for `meta-audit`:

1. Build a JSON summary: date, current commit SHA, PR URL, agent count, the scope
   actually covered, and every finding (location, claim, correct value, severity,
   category, whether auto-fixed).
2. `scripts/audit-eval-log.sh wiki-audit <path-to-summary.json>`
3. `scripts/audit-eval-check.sh wiki-audit` — if it prints `TRIGGER`, invoke the
   `meta-audit` skill for `wiki-audit` as a follow-up after this run's own PR is up.
```

(Skill name substituted per file.)

## `meta-audit` skill flow

Invoked two ways: automatically by a domain skill when its threshold triggers, or
on-demand ("evaluate wiki-audit", "how are the audits doing" → runs the same Phase 1-6
flow independently once per skill, in parallel, each producing its own PR).

**Phase 1 — Load history.** Read the target skill's `.jsonl`. Take every `run` entry
since the last `eval_marker` (or all of them if none exist yet).

**Phase 2 — Adversarial re-verification (parallel agents).** For each finding in scope
(capped at 20 most-recent if there are more, to bound cost — log if capped), spawn one
fresh agent that is given *only* the `location` and `claim` — never the logged
`correct_value` — and told to independently derive the correct value from source **as of
`commit_sha`**, reading historical file contents read-only via `git show <commit_sha>:<path>`
(never `git checkout`, which would mutate the shared working tree). Diff its
independently-derived answer against the logged `correct_value`. A mismatch is a
confirmed inaccuracy in the original audit.

**Phase 3 — Scope-staleness check.** For every skill with a `scope-universe` manifest,
enumerate the glob/paths it defines against the current repo tree, and compare against
the union of every `Scope:` line declared by that skill's Phase‑1 agents. Report:
- **Scope gaps** — items in the universe not covered by any agent (e.g. a new wiki page,
  a new `src/*/` module, a new `tests/*_tests/` directory).
- **Scope rot** — items an agent lists that no longer exist (renamed/removed files).

**Phase 4 — Synthesize.** Group findings by recurrence: a false-positive *pattern* that
happened more than once (e.g. "inferred game mechanic from a name instead of reading
source") is a stronger signal to add a guardrail than a one-off. Rank scope gaps and
confirmed inaccuracies by how long they've been present (older = higher priority).

**Phase 5 — Fix (same split every other audit skill uses).**
- *Auto-fix:* add a missing path to the right agent's `Scope:` line; remove a stale path;
  append a targeted guardrail row to the skill's anti-patterns table for a confirmed
  recurring false-positive category.
- *Flag for review:* anything that would change agent count, phase structure, or the
  auto-fix/flag-for-review policy itself.

**Phase 6 — Ship.** The `eval_marker` needs the PR's own URL, which doesn't exist until
the PR is created, so the order is: (1) branch, commit the `SKILL.md` fix(es), and open
the PR first (same pattern as every other audit skill — no direct commits to `main`); PR
body includes the full report: confirmed inaccuracies, scope gaps/rot, what was
auto-fixed, what's flagged; (2) only now, with a real PR URL in hand, append the
`eval_marker` entry; (3) commit that marker addition as a follow-up commit on the same
still-open branch before it merges.

## Historical backfill

Waiting for five fresh runs per skill before the first useful deep-eval is a long cold
start. Instead, backfill each skill's log now from data that already exists:

- **wiki-audit**: one `run` entry from today's 2026-07-03 audit (PR #643) — the full
  findings list from that session is already known.
- **doc-audit**: `run` entries reconstructed from PR bodies of #637, #632, #627, #617,
  #613 (each already contains a structured findings breakdown, per the existing PR
  convention — see #637's body for the shape).
- **test-audit**: reconstructed from #639, #622, #593's PR bodies.
- **perf-audit** / **dependency-audit**: backfill whatever PRs exist; if fewer than 5,
  the first deep-eval simply runs on-demand with a smaller sample rather than waiting for
  the auto-trigger.

This is a one-time manual pass as part of implementing this skill, not an ongoing
mechanism. `commit_sha` for backfilled entries follows the same rule as live runs: for
doc-audit/wiki-audit, whose audited PRs never touch the `src/` code their findings
describe, the PR's merge commit and its pre-fix parent commit are equivalent, so either
works. For perf-audit/test-audit, whose audited PRs *do* modify the code being
evaluated, the merge commit contains the fix rather than the pre-fix state a finding
describes — so backfilled `commit_sha` must be the PR's parent (pre-fix) commit instead.

## Testing / validation

- Manually invoke `meta-audit` for `wiki-audit` right after backfilling its log with
  today's run, confirm Phase 2 correctly re-derives the Soulforge Soul Tithe costs and
  the Vault Warden mechanic from source and matches the logged `correct_value` (both were
  correct by the time the PR merged, so this should come back clean — the run itself is
  the test of the re-verification pipeline, not an expectation of finding a bug).
- Deliberately seed one wrong `correct_value` into a copy of the wiki-audit log and
  confirm Phase 2 flags it as a mismatch (validates the adversarial check actually
  catches errors, not just rubber-stamps).
- Confirm `audit-eval-check.sh` returns `SKIP: n/5` correctly as entries accumulate and
  `TRIGGER` exactly at the 5th `run` entry since the last marker.
- Confirm `audit-eval-log.sh` rejects malformed JSON with a non-zero exit and does not
  touch the log file.

## Open questions / future work

- Threshold (`N = 5`) is a single global default. If `dependency-audit` runs far less
  often than `wiki-audit` in practice, a per-skill threshold might be worth revisiting
  once real cadence data exists.
- `perf-audit`'s scope-universe is thematic (module-level), not exhaustive like the
  others — worth revisiting if it produces too much noise once real data comes in.
- Whether `meta-audit` should eventually be folded into the `audit` orchestrator once it's
  proven out is deferred; keeping it separate now avoids coupling an unproven mechanism to
  the existing stable orchestrator.
