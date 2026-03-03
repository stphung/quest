# Storm Sigils as Equipment Subsection Design

## Problem

Storm Sigils are a separate bordered panel between hero and equipment panels in the stats panel. This wastes vertical space and visually disconnects sigils from equipment, even though they're both "gear" the player wears.

## Solution

Move sigils into the equipment panel as a subsection below the 7 equipment slots, separated by a titled divider line. Conditionally shown only when sigils are etched (same as today).

## Layout

```
┌─ Equipment ⚡523+45 ────────────┐
│ Weapon: Flaming Sword    Rare T5│
│ Armor:  Iron Plate       Epic T3│
│ Helmet: Steel Helm       Mag  T2│
│ Gloves: Leather Grips    Com  T1│
│ Boots:  Swift Runners    Rare T4│
│ Amulet: Jade Pendant     Epic T6│
│ Ring:   Band of Might    Mag  T2│
├─ ⛆ Storm Sigils (3/5) ─────────┤
│ ⚡ Fury        +5% DMG       A  │
│ 🛡 Bulwark     +3% DEF       B  │
│ · empty                        │
│ 🔒 locked                      │
│ 🔒 locked                      │
└─────────────────────────────────┘
```

When no sigils are etched, the divider and sigil rows are hidden (equipment panel only shows the 7 slots).

## Changes

### `src/ui/stats_panel.rs`
- Remove sigils panel from layout constraints (no more separate `Constraint::Length(7)`)
- Remove `draw_sigils_panel()` call
- Pass `&game_state.storm_sigils` to `draw_equipment_names_only()`
- Equipment panel gains the vertical space previously used by the sigils panel

### `src/ui/stats_equipment.rs`
- Add `storm_sigils: &StormSigils` parameter to `draw_equipment_names_only()`
- After rendering the 7 equipment slots, if `storm_sigils.etched_count() > 0`:
  - Draw a horizontal divider line with `" ⛆ Storm Sigils (N/5) "` title
  - Render 5 sigil slot lines (reuse rendering logic from `stats_sigils.rs`)
- Keep soulforge visual effects on equipment rows only

### `src/ui/stats_sigils.rs`
- Extract per-slot rendering into a `pub(super)` helper function
- Or inline the rendering into `stats_equipment.rs` and remove the standalone panel function

## Out of Scope
- Game logic, sigil data, input handling — unchanged
- M/S tier layouts — unchanged
- Stormglass Exchange overlay — unchanged
- Sigil visual effects (storm-blue tint, heat, motes) — keep on the sigil section
