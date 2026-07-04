> **Backported** from pre-OpenSpec design docs (docs/plans/2026-03-03-unified-status-strip-design.md, docs/plans/2026-03-03-unified-status-strip-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

The floating combat text system was designed when HP bars sat adjacent to the enemy sprite. Now that HP bars live in the 2-row status strip at the bottom of the right panel, the floating text over the sprite area is spatially disconnected from the health changes it represents.

## What Changes

- Introduced/changed `unified-status-strip` — see `design.md` (source: `2026-03-03-unified-status-strip-design.md`).

## Capabilities

### Modified Capabilities
- (tooling / no capability): unified-status-strip

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
