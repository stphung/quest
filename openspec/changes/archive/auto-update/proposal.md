> **Backported** from pre-OpenSpec design docs (docs/archive/plans/2026-02-02-auto-update-design.md, docs/archive/plans/2026-02-02-auto-update-implementation.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

Quest supports self-updating via a CLI command. On game startup, it checks for updates and displays a notification if one is available. The user runs `quest update` to download and install.

## What Changes

- Introduced/changed `auto-update` — see `design.md` (source: `2026-02-02-auto-update-design.md`).

## Capabilities

### Modified Capabilities
- (tooling / no capability): auto-update

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
