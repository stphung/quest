---
name: audit-docs
description: Multi-agent developer documentation audit — finds stale constants, missing files, and outdated types across CLAUDE.md files and docs/. Use when docs are stale, after adding features, or before releases.
---

# Audit Developer Documentation

Multi-agent audit of developer documentation. Finds stale content, missing files, and outdated types across CLAUDE.md files and docs/, then auto-fixes safe patterns.

## When to Use

- After landing new features, modules, or challenges
- When constants, types, or signatures have changed
- Before a release
- New modules exist without CLAUDE.md files
- When asked to audit or update docs

**For player-facing wiki:** Use the `audit-wiki` skill instead.

## Phase 1: Parallel Audit (5 Agents, Read-Only)

Spawn 5 Explore agents simultaneously. Each agent cross-references documentation against actual source code to find discrepancies.

**Agent 1 — Root & Architecture**

Scope: `CLAUDE.md`, `docs/*.md` (system-design, core-systems, secondary-systems, balancing, challenge-minigames, decisions, infrastructure)

Check: Module navigation table completeness, key constants accuracy, dependency versions, architecture descriptions.

**Agent 2 — Core Engine**

Scope: `src/core/CLAUDE.md`, `src/combat/CLAUDE.md`, `src/character/CLAUDE.md`, `src/zones/CLAUDE.md`

**Agent 3 — Content Systems**

Scope: `src/dungeon/CLAUDE.md`, `src/fishing/CLAUDE.md`, `src/items/CLAUDE.md`, `src/challenges/CLAUDE.md`

**Agent 4 — Progression Systems**

Scope: `src/haven/CLAUDE.md`, `src/enhancement/CLAUDE.md`, `src/ascension/CLAUDE.md`, `src/stormglass/CLAUDE.md`, `src/deep/CLAUDE.md`, `src/power_cores/CLAUDE.md`, `src/god_items/CLAUDE.md`, `src/loom/CLAUDE.md`

**Agent 5 — Infrastructure**

Scope: `src/ui/CLAUDE.md`, `src/input/CLAUDE.md`, `src/utils/CLAUDE.md`, `src/main_helpers/CLAUDE.md`, `src/achievements/CLAUDE.md`, `src/history/CLAUDE.md`

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
4. Report summary of: findings, auto-fixes applied, items flagged for review

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
