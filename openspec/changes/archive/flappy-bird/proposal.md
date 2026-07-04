> **Backported** from pre-OpenSpec design docs (docs/design/flappy-bird-architecture.md, docs/design/flappy-bird.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

This document describes how to integrate a real-time action minigame (Flappy Bird) into Quest's existing 100ms tick-based game loop. The key challenge is running Flappy Bird at 30+ FPS while keeping all other game systems (combat, fishing, dungeons) at their normal 100ms tick rate.

## What Changes

- Introduced/changed `flappy-bird` — see `design.md` (source: `flappy-bird-architecture.md`).
- Introduced/changed `flappy-bird` — see `design.md` (source: `flappy-bird.md`).

## Capabilities

### Modified Capabilities
- `challenges`: historical design/plan for this feature

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
