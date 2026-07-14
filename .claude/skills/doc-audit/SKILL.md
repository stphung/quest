---
name: doc-audit
description: Multi-agent developer documentation audit — finds stale constants, missing files, and outdated types across CLAUDE.md files, README.md, and docs/. Use when docs are stale, after adding features, or before releases.
---

<!-- meta-audit scope-universe: CLAUDE.md docs/*.md docs/dossiers/*.md src/*/CLAUDE.md README.md -->

# Audit Developer Documentation

Multi-agent audit of developer documentation. Finds stale content, missing files, and outdated types across CLAUDE.md files, README.md, and docs/, then auto-fixes safe patterns.

## When to Use

- After landing new features, modules, or challenges
- When constants, types, or signatures have changed
- Before a release
- New modules exist without CLAUDE.md files
- When asked to audit or update docs

**For the player-facing wiki (`quest.wiki/`):** Use the `wiki-audit` skill instead. The repo `README.md` is covered *here* (Agent 6), not by wiki-audit.

## Phase 1: Parallel Audit (6 Agents, Read-Only)

Spawn 6 Explore agents simultaneously. Each agent cross-references documentation against actual source code to find discrepancies.

**Agent 1 — Root & Architecture**

Scope: `CLAUDE.md`, `docs/*.md` (README, decisions), `docs/dossiers/*.md`

Check: Module navigation table completeness, key constants accuracy, dependency versions, architecture descriptions.

Dossier caveat (`docs/dossiers/*.md`, living docs refreshed manually; the
`README.md` there is exempt): only the **Mechanics & Constants** and **Interrelations**
sections make present-tense claims — cross-reference those against source as usual. The
**Balance Evidence** and **Fun Assessment** sections are *dated snapshots* of past
simulator runs and rubric scores; do not flag them for disagreeing with current source.
A dossier whose `Last refreshed` sha is far behind HEAD while commits have touched its
system's paths gets one MEDIUM "stale dossier — needs a manual refresh"
finding (flag for review, not auto-fixed), not a per-line teardown.

**Agent 2 — Core Engine**

Scope: `src/core/CLAUDE.md`, `src/combat/CLAUDE.md`, `src/character/CLAUDE.md`, `src/zones/CLAUDE.md`

**Agent 3 — Content Systems**

Scope: `src/dungeon/CLAUDE.md`, `src/fishing/CLAUDE.md`, `src/items/CLAUDE.md`, `src/challenges/CLAUDE.md`

**Agent 4 — Progression Systems**

Scope: `src/haven/CLAUDE.md`, `src/enhancement/CLAUDE.md`, `src/ascension/CLAUDE.md`, `src/stormglass/CLAUDE.md`, `src/deep/CLAUDE.md`, `src/power_cores/CLAUDE.md`, `src/god_items/CLAUDE.md`, `src/loom/CLAUDE.md`, `src/vessel/CLAUDE.md`

**Agent 5 — Infrastructure**

Scope: `src/ui/CLAUDE.md`, `src/input/CLAUDE.md`, `src/utils/CLAUDE.md`, `src/main_helpers/CLAUDE.md`, `src/achievements/CLAUDE.md`, `src/history/CLAUDE.md`

**Agent 6 — README (player-facing)**

Scope: `README.md`

Check every player-facing claim against source, not against other docs: the intro tagline and Features list (zone count, rank counts, rates, system roster — new shipped systems must appear), gameplay numbers (drop rates, offline rate, prestige formula, attribute caps, enhancement odds), key bindings in Controls (verify against `src/input/` and `src/main_helpers/update.rs` handlers, not memory), minigame details (board sizes, ELO, rewards), the Game Systems sections, install/update/platform claims, and the project-structure tree. Two README-specific rules: (1) do NOT advertise dark-shipped / kill-switched features (e.g. anything gated behind `vessel::ACT2_ENABLED`) — players can't see them yet; (2) prefer removing a precise-but-wrong number over inventing a new unverified one.

### Anti-Patterns

Each agent searches for:

| Pattern | Severity | Example | Fix |
|---------|----------|---------|-----|
| Missing file in inventory | HIGH | New `.rs` file not listed in CLAUDE.md | Add to file table |
| Stale constant value | HIGH | CLAUDE.md says 1800 but code says 900 | Update to match source |
| Missing module CLAUDE.md | HIGH | `src/foo/` exists with no CLAUDE.md | Create from template |
| Listed file doesn't exist | MEDIUM | CLAUDE.md lists `bar.rs` but it was deleted/renamed | Remove or rename |
| Stale type/enum variant | MEDIUM | New enum variant not documented | Add to types table |
| Stale dependency version | LOW | `Cargo.toml` says 0.30 but docs say 0.28 | Update version |
| Stale README gameplay claim | HIGH | README says "10 zones" / "30 ranks" but code has 50 / 40 | Update to match source |
| Wrong key binding in README | MEDIUM | README says "Q: Quit" but handler binds Esc | Update to actual handler |
| Shipped system missing from README | MEDIUM | Feature landed but Features list / Game Systems silent | Add a bullet or section |
| Dark-shipped feature advertised in README | HIGH | Kill-switched content described as playable | Remove from README (keep in CLAUDE.md) |
| README tree row for a dark-shipped module | MEDIUM | The `vessel/` row churned across three runs: removed as dark-ship (#728), re-added naming Act 2 content (#736), re-neutralized (#741) | Keep the module's row with a neutral, non-advertising description (e.g. "Feature-flagged module (disabled by default)") — that satisfies both the inventory rule and the dark-ship rule; do not delete the row, and never name the gated content |

Each agent produces a ranked report: file, pattern, severity (HIGH/MEDIUM/LOW), current value vs correct value, whether auto-fixable.

## Phase 2: Fix (Sequential)

Spawn fix agents based on audit findings.

### Auto-fix (no user approval needed)

- Add missing files to inventory tables
- Update constant values to match source
- Fix stale dependency versions
- Add missing enum variants to type docs
- Remove entries for deleted/renamed files

### Flag for user review

- Removing documented sections (might be intentional WIP)
- Rewriting architectural descriptions
- Creating new CLAUDE.md files from scratch

## Phase 3: Verify

1. `make check` must pass
2. Every `src/*/` directory has a CLAUDE.md (except `bin/`)
3. Every file listed in any CLAUDE.md exists on disk
4. README.md contains no dark-shipped features and every number in it was verified against source this run
5. Report summary of: findings, auto-fixes applied, items flagged for review

## Module CLAUDE.md Template

See `claude-md-template.md` in this skill directory. Key required sections:

| Section | Required |
|---------|----------|
| Files (table) | Yes |
| Key Types | Yes |
| How It Works | Yes |
| Integration Points | Yes |
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
