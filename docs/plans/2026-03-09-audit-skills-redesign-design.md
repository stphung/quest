# Audit Skills Redesign: audit-docs + audit-wiki

## Overview

Rename and redesign `update-docs` and `update-wiki` skills to `audit-docs` and `audit-wiki`, matching the structure and quality of `perf-audit` and `test-audit`. Replace vague scope tiers and abstract roles with fixed agent counts, specific file scopes, anti-pattern tables, and auto-fix guardrails.

## audit-docs

### Process Flow

```
Phase 1: Parallel Audit (5 agents, read-only)
  ├── Agent 1: Root CLAUDE.md + docs/*.md
  ├── Agent 2: Core engine modules (core, combat, character, zones)
  ├── Agent 3: Content system modules (dungeon, fishing, items, challenges)
  ├── Agent 4: Progression system modules (haven, enhancement, ascension, stormglass, deep, power_cores, god_items, loom)
  └── Agent 5: Infrastructure modules (ui, input, utils, main_helpers, achievements, history)

Phase 2: Fix (sequential)
  ├── Auto-fix safe patterns
  └── Flag risky changes for user review

Phase 3: Verify
  ├── make check
  ├── Every src/*/ has CLAUDE.md (except bin/)
  ├── Every file listed in CLAUDE.md exists on disk
  └── Report summary
```

### Anti-Patterns

| Pattern | Severity | Example | Fix |
|---------|----------|---------|-----|
| Missing file in inventory | HIGH | New `.rs` file not listed in CLAUDE.md | Add to file table |
| Stale constant value | HIGH | CLAUDE.md says 1800 but code says 900 | Update to match source |
| Missing module CLAUDE.md | HIGH | `src/foo/` exists with no CLAUDE.md | Create from template |
| Listed file doesn't exist | MEDIUM | CLAUDE.md lists `bar.rs` but it was deleted/renamed | Remove or rename |
| Stale type/enum variant | MEDIUM | New enum variant not documented | Add to types table |
| Stale dependency version | LOW | `Cargo.toml` says 0.30 but docs say 0.28 | Update version |

### Guardrails

**Auto-fix (no approval needed):**
- Add missing files to inventory tables
- Update constant values to match source
- Fix stale dependency versions
- Add missing enum variants to type docs

**Flag for user review:**
- Removing documented files/sections
- Rewriting architectural descriptions
- Creating new CLAUDE.md files from scratch

## audit-wiki

### Process Flow

```
Phase 1: Parallel Audit (4 agents, read-only)
  ├── Agent 1: Core gameplay (Combat, Zones, Prestige, Equipment)
  ├── Agent 2: Discovery systems (Dungeons, Fishing, Haven, Soulforge)
  ├── Agent 3: Late-game systems (Challenges, The-Deep, Stormglass, Achievements)
  └── Agent 4: Guides & meta (Home, Getting-Started, Strategy-Guide, Stormbreaker-Path, Controls-and-UI)

Phase 2: Fix (sequential)
  ├── Auto-fix safe patterns
  └── Flag risky changes for user review

Phase 3: Verify
  ├── All wiki pages match source constants
  ├── No broken cross-links
  └── Report summary
```

### Anti-Patterns

| Pattern | Severity | Example | Fix |
|---------|----------|---------|-----|
| Missing game system | HIGH | Loom not documented in wiki | Create or add section |
| Stale numbers/constants | HIGH | Wiki says "15% drop rate" but code says 20% | Update to match code |
| Missing zone/achievement/item | MEDIUM | New zones added but not in wiki | Add entries |
| Broken cross-links | MEDIUM | `[[Page Name]]` points to renamed page | Fix link |
| Stale strategy advice | LOW | Guide recommends outdated build order | Update advice |

### Guardrails

**Auto-fix (no approval needed):**
- Update constant values to match source code
- Fix broken cross-links
- Add missing zones/achievements/items to existing tables

**Flag for user review:**
- Creating new wiki pages from scratch
- Removing documented sections
- Changing strategy/gameplay advice

## Output

Both skills report the PR URL and final status when done (use `/ship` skill).

## What Changes from Current Skills

| Aspect | Current | New |
|--------|---------|-----|
| Name | `update-docs` / `update-wiki` | `audit-docs` / `audit-wiki` |
| Agents | Vague scope tiers (small/medium/large) | Fixed agent count with specific file scopes |
| Patterns | No anti-pattern tables | Severity-ranked pattern tables |
| Guardrails | None | Auto-fix vs flag-for-review |
| Roles | Abstract ("sys-architects", "product managers") | Explore agents with concrete scopes |
| Output | Generic | Ships via `/ship` skill |

### Removed
- Abstract role names and team composition sections
- Scope tier logic
- Multi-purpose instructions (always run full audit)
