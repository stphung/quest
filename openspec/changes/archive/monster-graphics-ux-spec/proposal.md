> **Backported** from pre-OpenSpec design docs (docs/design/monster-graphics-ux-spec.md). Archived for historical design rationale; current behavior lives in `openspec/specs/`.

## Why

This document defines the visual design system for enemy monsters in Quest's terminal-based combat view. It addresses five critical gaps in the current implementation: limited sprite variety (6 templates for 55+ enemy types), monochromatic rendering (all enemies Color::Red), no visual tier differentiation, fragile name-based sprite matching, and no zone identity in monster visuals.

## What Changes

- Introduced/changed `monster-graphics-ux-spec` — see `design.md` (source: `monster-graphics-ux-spec.md`).

## Capabilities

### Modified Capabilities
- `combat`: historical design/plan for this feature

## Impact

Historical feature work. See `design.md` for the full design record and `tasks.md` for the implementation plan (where one was recorded).
