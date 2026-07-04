> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-28-scaffold-wiring-design.md, docs/plans/2026-02-28-scaffold-wiring-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

**Goal:** Complete the structural overhaul scaffold from PR #424 by wiring all 6 facades, switching main.rs to TickContext, populating sub-structs with real types, and adding custom serde for backward-compatible saves.

## What Changes

- Introduced/changed `scaffold-wiring` — see `design.md` (source: `2026-02-28-scaffold-wiring-design.md`).

## Capabilities

### Modified Capabilities
- (tooling / no capability): scaffold-wiring

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
