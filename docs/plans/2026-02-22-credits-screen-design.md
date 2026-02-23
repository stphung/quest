# Credits Screen Design

**Issue:** #299
**Date:** 2026-02-22

## Overview

A simple credits screen overlay accessible from the main game screen via `[C]` keybind. Displays contributors and technologies in an RPG-themed modal dialog.

## Visual Design

```
╔══════════════════════════════════════╗
║           ⚔  Q U E S T  ⚔           ║
║      A Terminal-Based Idle RPG       ║
║                                      ║
║  ─── Forged By ───────────────────   ║
║  Steven Phung (@stphung)     Creator ║
║  DH (@dhsu)             Contributor  ║
║                                      ║
║  ─── Built With ──────────────────   ║
║  Rust · Ratatui · Crossterm          ║
║                                      ║
║             [Esc] Close              ║
╚══════════════════════════════════════╝
```

## Implementation

- **Keybind:** `[C]` on the main game screen opens the credits overlay
- **Close:** `[Esc]` or `[C]` closes it
- **Overlay type:** New `GameOverlay::Credits` variant
- **UI file:** `src/ui/credits_scene.rs` — centered modal following the same pattern as `help_overlay.rs`
- **Theming:** Uses the existing `render_themed_block` / `themed_border_color` system so the border matches the player's unlocked border style
- **Colors:** Cyan border and title (matching help overlay), bold section headers, DarkGray close hint

## Files to Change

1. `src/input/types.rs` — Add `GameOverlay::Credits` variant
2. `src/input/mod.rs` — Handle `[C]` keybind in `handle_base_game`, handle Esc dismiss in overlay priority chain
3. `src/ui/credits_scene.rs` — New file: `draw_credits_overlay(frame)`
4. `src/ui/mod.rs` — Register `pub mod credits_scene`
5. `src/main_helpers/overlay.rs` — Render `GameOverlay::Credits` in `draw_game_overlays`
