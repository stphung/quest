> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-27-structural-overhaul-design.md, docs/plans/2026-02-27-structural-overhaul-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

**Date**: 2026-02-27 **Approach**: B (Module Facades + Decomposed State) **Risk tolerance**: Conservative — preserve all public APIs, existing tests pass unchanged **Scope**: Full structural overhaul (GameState, tick engine, challenges, UI)

## What Changes

- Introduced/changed `structural-overhaul` — see `design.md` (source: `2026-02-27-structural-overhaul-design.md`).

## Capabilities

### Modified Capabilities
- (tooling / no capability): structural-overhaul

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
