> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-15-loot-ticker-design.md, docs/plans/2026-02-15-loot-ticker-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

Replace the current vertical Loot panel with a 1-row horizontal scrolling ticker (stock-ticker style) that continuously displays recent game events. This saves ~5 rows of vertical space and gives the combat log full width.

## What Changes

- Introduced/changed `loot-ticker` — see `design.md` (source: `2026-02-15-loot-ticker-design.md`).

## Capabilities

### Modified Capabilities
- (tooling / no capability): loot-ticker

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
