> Backported design record. Sources: docs/plans/2026-02-16-haven-scene-fx-design.md.

## 2026-02-16-haven-scene-fx-design.md

# Haven Scene FX Effects Design

## Overview

Add subtle, cozy scene_fx-powered visual effects to the Haven skill tree overlay. Convert `render_haven_tree` from pure Ratatui widgets to a SceneCell buffer with a warm hearth glow backdrop. The goal is warmth without intensity — a cozy cabin fireplace, not an industrial forge.

## Approach

**Full Scene Buffer Replacement (Approach A):** Rewrite `render_haven_tree` to use a `SceneCell` buffer, consistent with `soulforge_scene.rs` and `fishing_scene.rs`. The entire overlay renders into a buffer, then flushes with `render_buffer()`.

## Scope

**In scope:** `render_haven_tree` (the main skill tree + detail panel view).

**Out of scope:** Modals (build confirmation, forge confirmation, discovery modal, vault selection) stay as plain Ratatui widgets. No room-reactive effects — the backdrop is static regardless of which room is selected.

## Warm Hearth Backdrop

### Background Gradient

Bottom rows warm amber `(60, 35, 15)` fading to near-black `(10, 8, 6)` at top. Much dimmer than Soulforge's `(180, 60, 20)` — cozy, not dramatic.

### Drifting Motes

4-6 particles drifting upward very slowly (~0.3x Soulforge speed). Characters: `·`, `·`. Warm amber/brown tones `(140, 90, 30)` at bottom fading to `(60, 35, 15)` as they rise. No bright whites, yellows, or fancy characters (`*`, `✦`) — just soft warm dots.

### No Shimmer

Unlike Soulforge, no heat shimmer effect. Haven should feel still and calm.

### Selected Row Highlight

Selected skill tree row gets a subtle warm highlight bg `(30, 22, 12)`.

## Layout Preservation

The existing layout structure stays identical:

```
┌─ Haven ─────────────────────────────────────┐
│ [Summary bar: active bonuses]               │
│ ┌─Skill Tree──────┐┌─Room Detail──────────┐ │
│ │  ★★· Hearthstone ││ Description...       │ │
│ │  ★·· Armory      ││ Bonuses: T1/T2/T3   │ │
│ │  ...             ││ Requirements:        │ │
│ │                  ││ Cost: X PR           │ │
│ └──────────────────┘└──────────────────────┘ │
│ [↑/↓] Navigate  [Enter] Build  [Esc] Close  │
└──────────────────────────────────────────────┘
```

- Outer `Block` (Cyan border, " Haven " title) stays as a Ratatui widget
- Everything inside `inner` renders into the SceneCell buffer
- Sub-panel borders rendered via `put_cell` with box-drawing characters and DarkGray fg
- All text rendered via `put_text()` helper functions

## Text Readability

All text keeps existing fg colors (Cyan, White, Green, Yellow, DarkGray, Red). The dark backdrop ensures high contrast. No changes to the color scheme — only the background changes from black to warm gradient.

## Technical Details

### scene_fx utilities used

- `SceneCell` buffer for the full overlay inner area
- `render_buffer()` to flush to frame
- `put_cell()` for character placement
- `hash2d()` for deterministic mote placement
- `lerp_rgb()` / `lerp_channel()` for gradient interpolation
- `current_millis()` for animation timing

### Contrast with Soulforge

| Property | Soulforge | Haven |
|----------|-----------|-------|
| Bottom gradient | `(180, 60, 20)` bright orange | `(60, 35, 15)` dim amber |
| Top gradient | `(15, 8, 5)` | `(10, 8, 6)` |
| Ember/mote count | 8-12 | 4-6 |
| Particle speed | Normal | ~0.3x |
| Particle chars | `·`, `•`, `*`, `✦` | `·`, `•` |
| Heat shimmer | Yes | No |
| Spark effects | Yes (on strikes) | No |
| Phase variations | 5 phases, dramatic | Static, calm |

## Files Modified

- `src/ui/haven_scene.rs` — Major rewrite of `render_haven_tree` and its sub-functions to SceneCell buffer rendering

## Files Not Modified

- `src/ui/scene_fx.rs` — Existing utilities are sufficient
- `src/haven/types.rs` — No type changes needed
- `src/haven/logic.rs` — No logic changes
- Modal functions in `haven_scene.rs` — Stay as Ratatui widgets
