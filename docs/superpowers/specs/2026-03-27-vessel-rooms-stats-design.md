# Vessel Room System & Ship Stats

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 3 of 7
**Depends on:** Sub-project 2 (Voyage Shell)

## Overview

The ship's power comes from four multiplicative layers: base ship stats (from XP/leveling), room bonuses (from built and upgraded rooms), crew assignments (from stationed crew members), and component bonuses (from items installed in room slots). Rooms are the core build system — ~20 types unlocked at distance milestones, built into slots, upgraded with levels and components, constrained by Reactor power.

There is no free scaling from distance — all power is earned through rooms, crew, components, and ship XP. The Rune Array room can optionally convert old-world Transmissions into combat bonuses, but this costs a room slot.

## Ship Stats

| Stat | Combat Role | Non-Combat Role |
|------|------------|-----------------|
| Firepower | Damage dealt to enemies | — |
| Hull | HP pool + defense value | — |
| Engines | Evasion (dodge chance) | Distance traveled per day |
| Sensors | Crit chance | Detection range for encounters/derelicts |

## Ship Stat Formula

Each stat is computed independently through the same pipeline:

```
Final stat = base_stat × room_multiplier × crew_multiplier × component_multiplier
```

- **base_stat**: From ship level (gained via combat XP). Starts at 10, grows per level.
- **room_multiplier**: `1.0 + sum of all active room contributions for that stat`. A level 5 Weapons Bay adds +0.50 to the Firepower room multiplier.
- **crew_multiplier**: `1.0 + crew bonus`. Based on specialty match and skill level of crew assigned to relevant rooms.
- **component_multiplier**: `1.0 + sum of stat component bonuses` from component slots in relevant rooms.

Example: Base Firepower 30 × Room 1.5 × Crew 1.3 × Components 1.2 = 70 Firepower.

## Combat Pipelines

### Attack Pipeline (ship → enemy)

```
1. Final Firepower (base × room × crew × component)
2. + Rune Array flat bonus (if built; converts Transmissions to flat damage)
3. - Enemy defense
4. Min 1
5. Crit check: chance = 5% + (Sensors / (Sensors + enemy_stealth)) × 25%
   Cap: 30%. Crit multiplier: 2.0x (components can increase)
6. Apply damage to enemy HP
```

### Defense Pipeline (enemy → ship)

```
1. Enemy base damage
2. - Hull defense (Final Hull stat used as defense value)
3. Min 1
4. Evasion check: dodge_chance = Engines / (Engines + enemy_accuracy)
   Cap: 50%. Dodge negates entire attack.
5. Apply damage to current Hull HP
```

### Key Differences from Act 1

- **No prestige/ascension multiplier** — clean break, ship earns all power
- **No hull regen between fights** — damage persists, must be repaired via Workshop/events
- **Evasion replaces damage reduction** — Engines-based dodge instead of flat DR
- **Sensors drive crits** — gives exploration stat a combat role
- **Rune Array is optional** — old-world Transmissions only become combat power if you invest a room slot

## Ship Leveling

The ship gains XP from combat encounters (like the hero). XP curve: `500 × level^1.3` per level. No max level cap — diminishing returns serve as the natural ceiling.

Per level, base stats increase:
- Firepower: +2
- Hull: +3 (HP scales slightly faster)
- Engines: +1 (speed is powerful, grows slowly)
- Sensors: +1

## Reactor Power Budget

The Reactor is a room that produces power points. All other rooms consume power.

### Power Production

Reactor at level N produces: `10 + (N-1) × 5` power points.

| Reactor Level | Power Produced |
|---------------|----------------|
| 1 | 10 |
| 3 | 20 |
| 5 | 30 |
| 7 | 40 |
| 10 | 55 |

### Power Consumption

Each room has a base power cost by category that increases with room level.

**Base costs by category:**

| Category | Rooms | Base Cost |
|----------|-------|-----------|
| Core | Reactor, Hull Plating, Engines, Weapons Bay | 5 |
| Survival | Fuel Refinery, Cargo Hold, Life Support | 3 |
| Exploration | Sensors Array, Shuttle Bay, Cartography Deck | 4 |
| Crew/Narrative | Quarters, Medbay, Shrine | 2 |
| Production | Forge, Garden, Workshop | 3 |
| Special | Rune Array, Void Lens, Fate Loom | 6 |

**Level scaling:** Room power cost = `base_cost + (level - 1)`. A level 5 Weapons Bay costs `5 + 4 = 9` power.

Note: The Reactor itself costs 0 power (it produces, doesn't consume).

### Over-Budget Behavior

- Rooms can be **toggled on/off**. Inactive rooms cost 0 power and contribute nothing.
- If total active room cost exceeds Reactor output, the ship is **over-budget**. Over-budget rooms run at 50% effectiveness (stat contributions halved). All active rooms share the penalty equally — there's no priority system.
- The UI clearly shows power used/available and highlights rooms in red when over budget.

## Room System

### Slot Unlocks

~20 slots unlock at distance milestones, roughly 1 per 500 ly:

| Distance | Slots Available | Cumulative |
|----------|----------------|------------|
| 0 ly (launch) | 2 | 2 |
| 500 ly | 1 | 3 |
| 1,000 ly | 1 | 4 |
| 1,500 ly | 1 | 5 |
| ... | 1 per 500 ly | ... |
| 9,000 ly | 1 | 20 |

### Room Type Unlocks

Room types unlock at distance milestones. Not all types available from the start.

| Distance | Room Types Unlocked |
|----------|-------------------|
| 0 ly | Reactor, Engines, Hull Plating, Weapons Bay, Cargo Hold, Quarters |
| 1,000 ly | Fuel Refinery, Life Support, Sensors Array |
| 2,500 ly | Workshop, Garden, Medbay |
| 4,000 ly | Forge, Shuttle Bay, Shrine |
| 6,000 ly | Cartography Deck, Rune Array |
| 8,000 ly | Void Lens, Fate Loom |

### Building a Room

- Select an empty slot
- Choose from unlocked room types
- Pay a salvage cost (scales with room category: Core 100, Survival 60, Exploration 80, Crew 40, Production 60, Special 150)
- Room is built instantly (no construction delay — the Loom already taught patience)

### Rebuilding

Demolishing a room frees the slot but costs 50% of the original build cost in salvage (demolition fee). Components in the room are returned to inventory. The slot becomes empty and can be rebuilt.

### Room Upgrades: Levels

Rooms level from 1 to 10. Each level costs salvage scaling with current level and room category:

```
upgrade_cost = base_build_cost × level × 1.5
```

Each level increases the room's stat multiplier contribution by a fixed amount per room type.

### Room Upgrades: Components

Each room has 2-3 component slots (depending on category):
- Core/Special rooms: 3 slots
- All others: 2 slots

Components are found from combat loot, derelict exploration, and decision events. They modify the room:
- **Stat components** — flat bonus to a specific stat multiplier (e.g. "+0.1 Firepower multiplier")
- **Efficiency components** — reduce power consumption (e.g. "-1 power cost")
- **Special components** — unique effects (e.g. "Weapons Bay fires twice per encounter", "Garden produces fuel as well as supplies")

Components can be swapped freely (no cost to remove/replace). They're an inventory you manage.

## Room Stat Contributions

Each room contributes a multiplier bonus to one or more stats. Base contribution at level 1, scaling with level.

**Per-level multiplier bonus (added to room's contribution per level):**

| Room | Primary Stat | Bonus/Level | Secondary |
|------|-------------|-------------|-----------|
| Weapons Bay | Firepower | +0.10 | — |
| Hull Plating | Hull | +0.10 | — |
| Engines | Engines | +0.10 | — |
| Sensors Array | Sensors | +0.10 | — |
| Fuel Refinery | — | — | Reduces fuel drain by 5%/level |
| Cargo Hold | — | — | +10% resource capacity/level |
| Life Support | — | — | +1 crew capacity/level (base 2) |
| Quarters | — | — | +5% crew effectiveness/level |
| Medbay | — | — | Crew injury recovery speed +10%/level |
| Shrine | All stats | +0.02 | — |
| Forge | — | — | Component crafting (future) |
| Garden | — | — | +5 supplies/day per level |
| Workshop | Hull | +0.03 | +2 hull repair/day per level |
| Shuttle Bay | Sensors | +0.05 | Enables boarding events |
| Cartography Deck | Sensors | +0.05 | Reveals upcoming encounters |
| Rune Array | — | — | +10% transmission efficiency/level |
| Void Lens | Sensors | +0.08 | Unlocks hidden encounters |
| Fate Loom | All stats | +0.03 | Weave minor ship patterns |

A level 5 Weapons Bay contributes: `5 × 0.10 = 0.50` to the Firepower room multiplier. Combined room multiplier for Firepower = `1.0 + sum of all Firepower contributions`.

## Starting State (at launch)

The Vessel starts with:
- 2 room slots filled: **Reactor** (Lv 1) and **Engines** (Lv 1)
- Ship level 1 (base stats: Firepower 10, Hull 10, Engines 10, Sensors 10)
- 0 crew
- Some starting salvage (enough for 2-3 room builds)
- Room types available: Reactor, Engines, Hull Plating, Weapons Bay, Cargo Hold, Quarters

## UI: Room Management

Accessed via `[R]` hotkey from the voyage screen. Shows a grid/list of all slots:

```
┌─ Rooms (3/20 slots) ──── Power: 14/15 ─────────────┐
│                                                      │
│  1. [Reactor]      Lv 3   ⚡ produces 20            │
│  2. [Engines]      Lv 2   ⚡ 6   Eng +0.20          │
│  3. [Weapons Bay]  Lv 1   ⚡ 5   Fpr +0.10  ●●○     │
│  4. (empty)                                          │
│  5. (locked — 1,500 ly)                              │
│  ...                                                 │
│                                                      │
│  Selected: Weapons Bay                               │
│  Firepower: +0.10 multiplier                         │
│  Power cost: 5                                       │
│  Components: [Empty] [Empty] [Empty]                 │
│  Upgrade to Lv 2: 150 Salvage                        │
│                                                      │
│  [U] Upgrade  [T] Toggle  [D] Demolish  [Esc] Back  │
└──────────────────────────────────────────────────────┘
```

## Files

| File | Change |
|------|--------|
| `src/vessel/rooms.rs` | New: room types, stats, costs, power budget, build/demolish/upgrade |
| `src/vessel/components.rs` | New: component types, slot management, inventory |
| `src/vessel/stats.rs` | New: ship stat derivation (base × room × crew) |
| `src/vessel/types.rs` | Modify: add Room, Component, RoomSlot to VesselState |
| `src/ui/vessel_rooms_scene.rs` | New: room management overlay rendering |
| `src/input/vessel_input.rs` | Modify: add [R] hotkey and room management input |

## Testing

- Unit test: stat formula (base × room × crew) produces correct values
- Unit test: Reactor power production at each level
- Unit test: room power cost scales with base + level
- Unit test: over-budget penalty applies 50% to all rooms
- Unit test: room type unlock gating by distance
- Unit test: room slot unlock gating by distance
- Unit test: build cost and demolish refund calculations
- Unit test: upgrade cost formula
- Unit test: room stat contribution scales with level
- Unit test: component slot limits by room category
- Unit test: toggling rooms on/off affects power budget
