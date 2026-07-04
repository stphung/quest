> **Backported** from pre-OpenSpec design docs (docs/plans/2026-02-23-cloud-sync-design.md, docs/plans/2026-02-23-cloud-sync-plan.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

Enable players to back up Time Vault saves to GitHub and restore them on another machine. Uses the existing `git2`-based save history system with a private GitHub repository as the remote.

## What Changes

- Introduced/changed `cloud-sync` — see `design.md` (source: `2026-02-23-cloud-sync-design.md`).

## Capabilities

### Modified Capabilities
- `time-vault`: historical design/plan for this feature

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
