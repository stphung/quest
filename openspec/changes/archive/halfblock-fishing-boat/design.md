> Backported design record. Sources: docs/plans/2026-02-18-halfblock-fishing-boat-design.md.

## 2026-02-18-halfblock-fishing-boat-design.md

# Halfblock Fishing Boat Design

## Goal

Replace the current 7-char ASCII fishing boat with a ~13-char halfblock sailboat for better visibility and visual quality.

## Current State

The boat in `src/ui/fishing_scene.rs` (lines 283-322) uses plain ASCII:
- Hull: `_/___\_` (7 chars)
- Mast: `|` (3 rows)
- Sail: `\` + `\` (2 diagonal chars)
- Flag: `>` (1 char)

It's hard to read against the animated water background.

## Design: Halfblock Sailboat (~13w x 5h)

```
        ▸        ← red pennant
       ╱│        ← sail top + mast
      ╱ │        ← sail body + mast
  ▄▄▟█▀▀█▀█▙▄▄  ← deck with halfblock bow/stern taper
   ▀▀▄████▄▀▀   ← hull bottom with halfblock curves
      ▀▀▀▀      ← keel/waterline blend
```

### Halfblock technique

- `▄` (lower half): fg = hull color, bg = water/sky color — smooth bottom edges
- `▀` (upper half): fg = hull color, bg = water color — smooth top edges
- `▟` / `▙`: quarter-block corners for bow/stern taper
- `█`: solid hull body

### Color palette

| Element | Color | RGB |
|---------|-------|-----|
| Hull body | Dark brown | (94, 58, 36) |
| Hull highlight | Medium brown | (140, 90, 52) |
| Deck | Light wood | (188, 140, 86) |
| Bow/stern taper | Blended with water bg | varies |
| Mast | Golden wood | (236, 180, 108) |
| Sail | Cream/off-white | (245, 235, 215) |
| Pennant | Red | (255, 96, 72) |
| Keel/waterline | Blend hull into water bg | varies |

### Key rendering details

- Halfblock cells at hull edges use the water background color as the "other half" so the boat blends smoothly into the water
- The sail uses `╱` for diagonal lines (box-drawing diagonal)
- Same bobbing animation (`wave_tick * 0.04`)
- Same horizontal position (~35% of width)
- Fishing line still extends from mast tip to bobber
- Mast is taller (4 rows vs 3) to accommodate larger sail

### What changes

- `draw_water_scene()` boat section (lines 283-322) replaced with new halfblock rendering
- Boat wider: 13 chars vs 7
- Boat taller: 5 rows vs 3
- `boat_row` adjusted for new height
- Fishing line origin point adjusted for taller mast

### What stays the same

- All water/sky/cloud/star rendering unchanged
- Bobber mechanics and ripples unchanged
- Fishing phase visual changes unchanged
- Leviathan encounter modal unchanged
