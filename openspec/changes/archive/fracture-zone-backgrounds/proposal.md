> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-28-fracture-zone-backgrounds-design.md, docs/plans/2026-02-28-fracture-zone-backgrounds-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

All 19 fracture zones (12-30) currently fall through to `config_fallback()` in `zone_bg.rs`, rendering the same generic grey background. Each zone has rich narrative identity that should be reflected in its combat scene background.

## What Changes

- Introduced/changed `fracture-zone-backgrounds` — see `design.md` (source: `2026-02-28-fracture-zone-backgrounds-design.md`).

## Capabilities

### Modified Capabilities
- `zones`: historical design/plan for this feature

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
