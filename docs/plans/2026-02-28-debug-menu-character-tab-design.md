# Debug Menu "Character" Tab — Design Document

**Date:** 2026-02-28
**Goal:** Add fast travel, prestige granting, and level granting to the debug menu to accelerate game testing.

---

## Overview

Add a new `DebugCategory::Character` tab to the debug menu containing 16 actions for manipulating character state: zone travel (11 zones), prestige increments (+1/+5/+10), and level increments (+10/+50).

## New Debug Category

**Tab position:** After "Deep", before "Borders" in the tab bar.

### Zone Travel (11 actions)

One action per zone: "Travel to Meadow (Zone 1)" through "Travel to The Expanse (Zone 11)".

Each travel action:
1. Clears `active_dungeon` and `active_fishing` (can't travel mid-activity)
2. Auto-sets `prestige_rank = max(current, zone.prestige_requirement)` if needed
3. Calls `reset_for_prestige()` to recalculate `unlocked_zones` when prestige changes
4. Calls `travel_to(zone_id, 1)` to move to subzone 1 of the target zone
5. Recalculates derived stats and prestige bonuses
6. Returns status message, e.g., "Traveled to Volcanic Wastes (P10)"

Zone prestige requirements for reference:
- Zones 1-2: P0
- Zones 3-4: P5
- Zones 5-6: P10
- Zones 7-8: P15
- Zones 9-10: P20
- Zone 11 (The Expanse): P0 (achievement-gated, not prestige-gated)

### Prestige Increments (3 actions)

- "+1 Prestige Rank"
- "+5 Prestige Ranks"
- "+10 Prestige Ranks"

Each action:
1. Increments `prestige_rank` by the specified amount
2. Recalculates attribute cap (`20 + rank * 5`)
3. Calls `reset_for_prestige()` to unlock newly accessible zones
4. Recalculates derived stats and prestige bonuses

### Level Increments (2 actions)

- "+10 Levels"
- "+50 Levels"

Each action:
1. Loops through target number of level-ups
2. Per level: sets XP to meet `xp_for_next_level`, increments `character_level`, distributes +3 attribute points via `distribute_level_up_points()`
3. Recalculates derived stats after all levels granted

## Integration Points

### Files to modify
- `src/utils/debug_menu.rs` — Add `DebugCategory::Character`, new `DebugAction` variants, `CHARACTER_ACTIONS` array, and 16 handler functions
- `src/ui/debug_menu_scene.rs` — No changes needed (renders dynamically from category data)

### Dependencies used
- `zones::data::get_all_zones()` — Zone names and prestige requirements
- `zones::advancement::travel_to()` — Zone travel
- `zones::advancement::reset_for_prestige()` — Zone unlock recalculation
- `core::xp::xp_for_next_level()` — XP curve for level grants
- `core::xp::distribute_level_up_points()` — Attribute distribution on level-up
- `character::derived_stats` — Stat recalculation

## Design Decisions

- **Auto-adjust prestige on travel** rather than ignoring gates. Keeps game state consistent for accurate testing.
- **Increment buttons** (+1/+5/+10/+10/+50) rather than fixed presets. Stackable and flexible for reaching any target state.
- **Single "Character" tab** groups all state manipulation. Existing tabs stay focused on their domains (World = discovery triggers, Resources = currency grants).
- **Clear active content on travel.** Traveling while in a dungeon or fishing would leave orphaned state. Safest to clear.
