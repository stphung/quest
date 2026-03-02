---
name: update-docs
description: Use when documentation is stale, after adding features or modules, before releases, or when CLAUDE.md files and docs/ need to match the current codebase. Use when root CLAUDE.md is bloated, modules lack CLAUDE.md files, or docs have content duplication.
---

# Update Developer Documentation

Audit and update all developer documentation (CLAUDE.md files + docs/) for both structural health and content accuracy.

## When to Use

- After landing new features, modules, or challenges
- When constants, types, or signatures have changed
- Before a release
- Root CLAUDE.md exceeds ~300 lines or has per-module file listings
- New modules exist without CLAUDE.md files
- When asked to update or audit docs

**For player-facing wiki:** Use the `update-wiki` skill instead.

## Phase 1: Structural Health Check

Check documentation architecture before auditing content.

```bash
# Root CLAUDE.md size
wc -l CLAUDE.md

# Module CLAUDE.md coverage
for dir in src/*/; do
  module=$(basename "$dir")
  if [ -f "${dir}CLAUDE.md" ]; then echo "✓ $module"
  else echo "✗ $module (MISSING)"; fi
done
```

**Red flags:**
- Root > 300 lines with per-module file listings → restructure into navigation hub
- Modules missing CLAUDE.md → create from template (see `claude-md-template.md` in this skill directory)
- Content duplicated between root and module docs → move detail to module, keep summary in root

**Root CLAUDE.md target structure (~250 lines):**
- Build/Run, Workflow, CI/CD, Skills table (keep as-is)
- Module navigation table (one row per module, links to CLAUDE.md)
- Common Patterns, Key Constants, Combat Mechanics (condensed summaries)
- Dependencies

**If structural issues found:** Fix them first (Phase 2 content audit runs against correct structure).

## Phase 2: Content Accuracy Audit

1. Check when docs were last modified: `git log --format='%ai' -1 -- docs/ src/*/CLAUDE.md CLAUDE.md`
2. List commits since: `git log --oneline --since="<date>" main`
3. Categorize: new modules, new files, updated constants, structural changes

### What to audit against:
- `git log` since last doc update
- Actual source files for constants, types, signatures
- `Cargo.toml` for dependency versions
- Test file counts and test counts

### Files to check:

**docs/ (7 files):**
- `system-design.md`, `core-systems.md`, `secondary-systems.md`, `balancing.md`
- `challenge-minigames.md`, `decisions.md`, `infrastructure.md`

**CLAUDE.md files (21+):**
- Root `CLAUDE.md` — module table, constants, dependencies
- `src/*/CLAUDE.md` — per-module file inventories, types, constants

### Update rules:
- Read each file FIRST before editing
- Keep edits minimal — match existing style
- Cross-reference source code for accurate values
- Never modify .rs source files

## Phase 3: Execute Updates

### Scope tiers:

**Small** (1-3 items, single module): Handle directly.

**Medium** (4-8 items, 2-3 modules): Spawn 1 Explore agent to audit, then update directly.

**Large** (9+ items, 4+ modules, or new modules added): Launch parallel agents:
- **Agent 1**: New module docs (read source files, follow `claude-md-template.md`)
- **Agent 2-3**: Existing doc updates (absorb content, fix stale values)
- **Agent 4**: Root CLAUDE.md rewrite (if structural changes needed)

All agents edit different files — safe to parallelize.

## Phase 4: Verify and Commit

1. `make check` passes
2. Every `src/*/` directory has a CLAUDE.md (except `bin/`)
3. Root CLAUDE.md under 300 lines (if restructured)
4. Every file listed in any CLAUDE.md exists on disk
5. Module navigation table covers all modules
6. Create branch `docs/update-<topic>`, commit, push, PR

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
