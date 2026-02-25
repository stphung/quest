---
name: doc-health-audit
description: Use when documentation is stale, after adding features or modules, before releases, or when docs/, CLAUDE.md files, or the player-facing wiki (quest.wiki/) need to match the current codebase. Use when asked to update docs or audit documentation accuracy.
---

# Update Documentation

Audit and update all documentation (docs/, CLAUDE.md files, and player-facing wiki) to match the current codebase.

## When to Use

- After landing new features, modules, or challenges
- When constants, types, or signatures have changed
- Before a release
- When asked to update or audit docs
- When player-facing wiki (`quest.wiki/`) needs updating after game changes

## Phase 1: Assess Scope

1. Check when docs were last modified: `git log --format='%ai' -1 -- docs/ src/*/CLAUDE.md CLAUDE.md`
2. List commits since that date: `git log --oneline --since="<date>" main`
3. Categorize changes into:
   - **New modules/systems** (entirely new directories or major features)
   - **New files in existing modules** (new challenge types, UI scenes, etc.)
   - **Updated constants/values** (rebalanced numbers, renamed things)
   - **Structural changes** (renamed modules, moved files, changed signatures)
4. Count the categories to determine scope

### Scope Tiers:

**Small** (1-3 items, single module affected):
- No team needed
- Handle directly: read the affected doc, make the edit, verify, commit

**Medium** (4-8 items, 2-3 modules affected):
- Spawn 1 Explore agent to audit all docs in parallel
- Then make the updates directly based on audit findings
- Verify with cargo test + clippy

**Large** (9+ items, 4+ modules affected, or any new module added):
- Create a team with dynamic sizing:
  - **1 sys-architect per audit track**: 1 for docs/, 1 for CLAUDE.md files (always 2)
  - **1 tech-writer per 5 files needing updates**: round up (minimum 1, maximum 3)
  - **No reviewers** (docs-only changes don't need QA gates)
- Task structure:
  1. Audit tasks (parallel, one per sys-architect)
  2. Update tasks (parallel, blocked by audits, one per tech-writer)
  3. Final verify task (blocked by all updates): cargo test + clippy to confirm no source changes

## Phase 2: Execute Updates

### What to audit against:
- `git log` since last doc update for new features, modules, challenges, achievements
- Actual source files for accurate constants, types, function signatures
- `Cargo.toml` for dependency versions
- `tests/` directory for test file count
- `cargo test 2>&1 | grep "test result:"` for test counts

### Files to check:

**docs/ (7 files):**
- `system-design.md` — Architecture, tick pipeline, project structure, TickEvent/TickResult
- `core-systems.md` — Tick stages, game mechanics, constants
- `secondary-systems.md` — Achievements, challenges, fishing, haven
- `balancing.md` — Discovery weights, reward tables, constants appendix
- `challenge-minigames.md` — Challenge list, mechanics, difficulties, rewards
- `decisions.md` — Design decision log
- `infrastructure.md` — Debug menu, storage layout, save signals

**CLAUDE.md files (12+):**
- Root `CLAUDE.md` — Module list, project structure, constants, dependencies, test count
- `src/*/CLAUDE.md` — Per-module docs (achievements, challenges, character, combat, core, dungeon, fishing, haven, items, ui, zones, enhancement)
- Check for missing CLAUDE.md in any new module directories

### Update rules:
- Read each file FIRST before editing
- Keep edits minimal and factual — match existing style
- Don't rewrite sections, just add/update what's stale
- Cross-reference source code for accurate values
- Never modify .rs source files

## Phase 3: Verify and Commit (Developer Docs)

1. Run `cargo test` — confirm all tests pass (no source files accidentally changed)
2. Run `cargo clippy --all-targets -- -D warnings` — confirm clean
3. Create a branch `docs/update-<topic>` (since main is protected)
4. Commit with message: `docs: update design docs and CLAUDE.md for <summary>`
5. Push and create PR

## Phase 4: Player-Facing Wiki (`quest.wiki/`)

The GitHub wiki at `quest.wiki/` (git submodule) contains player-facing documentation. After updating developer docs, audit the wiki for staleness.

### Initializing the Wiki Submodule

The wiki must be initialized before it can be read or updated:

```bash
# Check if submodule is initialized (- prefix means not initialized)
git submodule status quest.wiki

# Initialize and clone the wiki submodule
git submodule update --init quest.wiki

# Verify (should show commit hash without - prefix)
git submodule status quest.wiki
```

If `quest.wiki/` exists as a regular directory (not a submodule), remove it first:
```bash
rm -rf quest.wiki
git submodule update --init quest.wiki
```

### Wiki Pages (15):

| Page | Content |
|------|---------|
| `Home.md` | Landing page, system overview table, quick links, installation |
| `Getting-Started.md` | New player guide, character creation, gameplay loop, first prestige |
| `Combat.md` | Damage pipeline, enemy scaling, boss multipliers, XP curve, death |
| `Zones-and-Progression.md` | All 11 zones, subzones, bosses, prestige gates |
| `Prestige.md` | Tier list (Bronze→Eternal), multipliers, combat bonuses, strategy |
| `Equipment.md` | 7 slots, 5 rarities, affixes, drop rates, auto-equip |
| `Dungeons.md` | Discovery, sizes, room types, keys, safe death |
| `Fishing.md` | 40 ranks, 8 tiers, Storm Leviathan hunt |
| `Haven.md` | 14-room skill tree, bonuses per tier, build order recommendations |
| `Soulforge.md` | Enhancement +0 to +10, success rates, costs, strategy |
| `Challenges.md` | All 10 minigames, controls, difficulties, rewards |
| `Achievements.md` | 5 categories, 80+ achievements |
| `Strategy-Guide.md` | 5-phase progression guide (early→endgame), pro tips |
| `Stormbreaker-Path.md` | Step-by-step walkthrough, PR cost breakdown, checklist |
| `Controls-and-UI.md` | Full keyboard reference for every screen and minigame |

### What to check:
- New game systems, challenges, or mechanics not yet documented in the wiki
- Changed constants (discovery rates, rewards, costs, XP curves) that affect player-facing numbers
- New zones, rooms, achievements, or equipment that need wiki coverage
- Cross-links between pages (use `[[Page Name]]` format)

### Wiki update workflow:

**Small** (1-3 pages affected):
- Edit the wiki pages directly, commit and push to `quest.wiki/`

**Large** (new systems, many pages affected):
- Create a team:
  - **2 sys-architects**: Research the codebase, write findings to `_research_*.md` temp files
  - **2 product managers**: Write/update wiki pages using research (split pages between them)
  - **1 game designer**: Write/update strategy and gameplay guidance pages
- Task dependencies:
  1. Research tasks (parallel, sys-architects)
  2. Writing tasks (parallel, blocked by research, PMs + game designer)
  3. Final review (blocked by all writing): cross-link, clean up research files, verify Home.md TOC
- Shut down agents as they complete their tasks

### Wiki update rules:
- Player-facing tone: friendly, engaging, practical tips alongside mechanics
- Use `[[Page Name]]` for cross-references between wiki pages
- Include data tables for numbers-heavy content (zone stats, reward tables, costs)
- Translate code concepts into player language (e.g., "prestige" not "reset")
- Every page should have a "See Also" or "Related Pages" section
- Clean up any `_research_*.md` temp files before pushing
- Commit directly to `quest.wiki/` (wiki repos have no branch protection)
- Push to `origin master` (wiki repos use `master`, not `main`)
