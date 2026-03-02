---
name: docs-structure-audit
description: Use when root CLAUDE.md is bloated with per-module file listings, modules lack CLAUDE.md files, or documentation duplicates detail between root and module docs. Use after adding new modules, after major restructuring, or when root exceeds ~300 lines.
---

# Documentation Structure Audit

Optimize CLAUDE.md documentation architecture: trim root to a navigation hub, ensure every module has its own CLAUDE.md, eliminate duplication.

**Distinct from `doc-health-audit`:** That skill audits content accuracy (staleness). This skill audits structural architecture (bloat, missing docs, duplication).

## When to Use

- Root CLAUDE.md exceeds ~300 lines with per-module file listings
- New modules exist without CLAUDE.md files
- Per-module details duplicated between root and module docs
- After module renames, splits, or merges
- After major feature additions that created new directories

## Phase 1: Assess Current State

```bash
# Count root lines
wc -l CLAUDE.md

# Check module coverage
for dir in src/*/; do
  module=$(basename "$dir")
  if [ -f "${dir}CLAUDE.md" ]; then echo "✓ $module"
  else echo "✗ $module (MISSING)"; fi
done
```

Identify bloat sources in root: per-module file listings, detailed tables, project structure trees, content that belongs in module docs.

## Phase 2: Plan Changes

### Root CLAUDE.md Target (~250 lines)

**Keep in root (condensed):**
- Build & Run, Development Workflow, CI/CD Pipeline
- Skills table
- Architecture overview (1-2 lines: entry point + main loop)
- Module navigation table (ALL modules with CLAUDE.md links)
- Common Patterns (condensed)
- Key Constants (most critical only)
- Combat/system mechanics (pipeline summaries only)
- Dependencies

**Move to module docs:**
- Per-module file listings → each module's CLAUDE.md
- Detailed tables (zone tiers, item stats, etc.) → relevant module's CLAUDE.md
- Full project structure tree → remove entirely (redundant with module docs)

### Module Navigation Table Format

```markdown
## Modules

| Module | Path | Docs | Purpose |
|--------|------|------|---------|
| Core | `src/core/` | [CLAUDE.md](src/core/CLAUDE.md) | One-line purpose |
| ... | ... | ... | ... |
```

Every module directory under `src/` gets a row. All rows link to their CLAUDE.md.

## Phase 3: Execute with Parallel Agents

All agents edit different files, so they can run in parallel without conflicts.

**Agent 1 — New module docs:** Create CLAUDE.md for each module that lacks one.
- Read ALL source files in the module first
- Follow template from `claude-md-template.md` (co-located in this skill directory)
- Target 60-150 lines per module doc

**Agent 2-3 — Existing doc updates:** Update module docs to absorb content removed from root.
- Read current module doc, identify what's moving from root
- Use Edit to add missing sections (don't rewrite entire files)
- Split across agents by module count

**Agent 4 — Root rewrite:** Restructure root CLAUDE.md.
- Replace file listings with navigation table
- Remove content that moved to module docs
- Condense remaining sections
- Use Write tool (major rewrite, not incremental edits)

**After all agents complete — Fix cross-references:**
- Ensure root navigation table links to all CLAUDE.md files (including newly created ones)

## Phase 4: Verify

1. `make check` passes (docs-only changes shouldn't break anything)
2. Every `src/*/` directory has a CLAUDE.md (except `bin/`)
3. Root CLAUDE.md is under 300 lines
4. Every file listed in any CLAUDE.md actually exists on disk:
   ```bash
   # For each module, glob *.rs and compare against CLAUDE.md file table
   ```
5. Module navigation table in root covers all modules
6. No content duplication between root and module docs (spot-check 5 modules)

## Module CLAUDE.md Template

See `claude-md-template.md` in this skill directory for the standard format. Key required sections:

| Section | Required | Content |
|---------|----------|---------|
| Files | Yes | Table of all .rs files with one-line purposes |
| Key Types | Yes | Primary structs/enums with important fields |
| How It Works | Yes | Core mechanics, mental model |
| Integration Points | Yes | Cross-module calls |
| Key Constants | If applicable | Balance numbers, thresholds |
| Adding / Extending | If applicable | How to extend the module |

Template is flexible — modules can deviate where it doesn't fit.
