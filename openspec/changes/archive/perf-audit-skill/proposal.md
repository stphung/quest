> **Backported** from pre-OpenSpec design docs (docs/plans/2026-03-08-perf-audit-skill-design.md, docs/plans/2026-03-08-perf-audit-skill-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

A multi-agent skill that audits the game for runtime performance bottlenecks, auto-fixes safe patterns, and leaves behind profiling infrastructure (criterion benchmarks + simulator profiling).

## What Changes

- Introduced/changed `perf-audit-skill` — see `design.md` (source: `2026-03-08-perf-audit-skill-design.md`).

## Capabilities

### Modified Capabilities
- (tooling / no capability): perf-audit-skill

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
