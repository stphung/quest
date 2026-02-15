# Update Documentation

Audit and update all documentation (docs/ and CLAUDE.md files) to match the current codebase.

## Phase 1: Assess Scope

Run a quick audit to determine how much has changed since docs were last updated.

### Steps:
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

## Phase 3: Verify and Commit

1. Run `cargo test` — confirm all tests pass (no source files accidentally changed)
2. Run `cargo clippy --all-targets -- -D warnings` — confirm clean
3. Create a branch `docs/update-<topic>` (since main is protected)
4. Commit with message: `docs: update design docs and CLAUDE.md for <summary>`
5. Push and create PR
