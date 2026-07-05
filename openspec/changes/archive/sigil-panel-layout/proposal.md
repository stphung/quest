> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-21-sigil-panel-layout-design.md, docs/plans/2026-02-21-sigil-panel-layout-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

Storm Sigils are currently rendered as a sub-panel nested inside the Equipment block in `stats_equipment.rs`. This is thematically wrong — sigils are permanent soul-inscribed progression bonuses, not gear. They deserve their own section.

## What Changes

- Introduced/changed `sigil-panel-layout` — see `design.md` (source: `2026-02-21-sigil-panel-layout-design.md`).

## Capabilities

### Modified Capabilities
- `stormglass`: historical design/plan for this feature

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
