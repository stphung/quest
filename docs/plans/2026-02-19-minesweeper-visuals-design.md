# Minesweeper Visual Upgrade Design

**Date:** 2026-02-19
**Status:** Implemented

## Summary

Upgrade the Trap Detection (minesweeper) minigame visuals from plain ASCII characters to Unicode symbols with checkerboard shading and background coloring for improved readability and visual appeal.

## Changes

### Symbol Upgrades

| Element | Before | After | Notes |
|---------|--------|-------|-------|
| Hidden cell | `#` (Gray) | `■` (checkerboard Gray/Rgb(120,120,130)) | Alternating colors based on `(row + col) % 2` |
| Flag | `F` (Red) | `⚑` (Red) | Unicode flag symbol |
| Mine/Trap | `*` (Red) | `☠` (Red) | Skull and crossbones |
| Empty | `.` (DarkGray) | `·` (DarkGray) | Middle dot for cleaner look |
| Numbers 1-8 | Unchanged | Unchanged | Same colors as before |

### Background Shading

- **Hidden/flagged cells**: Dark blue-gray background `Rgb(40, 40, 50)` distinguishes unrevealed area from revealed cells
- **Revealed cells**: Default terminal background (no bg set)
- **Cursor**: Upgraded from `bg(DarkGray)` to `bg(Yellow) + Bold` for higher visibility

### Implementation Details

- `get_cell_display()` signature changed to accept `(row, col)` parameters for checkerboard pattern calculation
- Background colors applied in `render_grid()` based on `cell.revealed` state
- Cursor background override applied after cell background (takes precedence)
- Legend in info panel updated to show new Unicode symbols
- All Unicode characters use `\u{xxxx}` escape sequences in source code

## Files Changed

- `src/ui/minesweeper_scene.rs` — All visual changes contained in this single file
