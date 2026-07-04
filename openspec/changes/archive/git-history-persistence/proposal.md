> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-22-git-history-persistence-design.md, docs/plans/2026-02-22-git-history-persistence-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

Add git-based versioning to the existing `~/.quest/` save directory so players can restore to previous save points. Significant game events trigger git commits with descriptive messages. Players browse and restore from a two-panel in-game Timeline Browser.

## What Changes

- Introduced/changed `git-history-persistence` — see `design.md` (source: `2026-02-22-git-history-persistence-design.md`).

## Capabilities

### Modified Capabilities
- `time-vault`: historical design/plan for this feature

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
