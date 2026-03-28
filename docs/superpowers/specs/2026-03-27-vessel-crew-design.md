# Vessel Crew System

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 5 of 7
**Depends on:** Sub-project 3 (Room System & Ship Stats)

## Overview

The Vessel carries 5-8 named crew members, each a unique individual with a specialty, a ship trait, and a room trait. Crew are found during the voyage and assigned to rooms to boost their output. Every crew member matters — losing one is significant.

## Crew Member Definition

Each crew member has:

- **Name** — generated (first name + epithet, like Deep mercs)
- **Specialty** (1 of 6) — determines room match bonus
- **Ship trait** (1 of 6) — passive bonus, always active
- **Room trait** (1 of 6) — bonus that only affects their assigned room
- **Skill level** (1-10) — grows passively over time while stationed in a room
- **Status** — Active, Injured, or Lost

### Specialties

Each specialty has primary rooms (high multiplier) and secondary rooms (moderate multiplier). All other rooms get a small mismatch multiplier.

| Specialty | Primary Rooms | Secondary Rooms |
|-----------|--------------|-----------------|
| Weapons | Weapons Bay | Hull Plating |
| Engineering | Reactor, Workshop | Engines |
| Navigation | Engines, Cartography Deck | Sensors Array |
| Medicine | Medbay, Life Support | Garden |
| Lore | Shrine, Fate Loom | Void Lens |
| Salvage | Forge, Shuttle Bay | Cargo Hold |

### Ship Traits (always active, regardless of room assignment)

| Trait | Effect |
|-------|--------|
| Lucky | +5% component drop rate |
| Resourceful | +10% salvage from encounters |
| Cautious | Ship takes 5% less hull damage |
| Inspiring | All other crew gain skill 10% faster |
| Vigilant | +5% evasion chance |
| Hardy | +5% fuel efficiency |

### Room Traits (only affect the assigned room)

| Trait | Effect |
|-------|--------|
| Industrious | Room output +15% |
| Efficient | Room power cost -1 |
| Precise | Room stat contribution +10% |
| Inventive | Room component effects +20% |
| Tireless | Room functions at full even when ship is over power budget |
| Adaptive | No specialty mismatch penalty in this room |

## Crew Multiplier

The crew multiplier is the third layer in the ship stat formula: `final_stat = base × room_multiplier × crew_multiplier`.

Crew multiplier for a stat = product of all crew contributions to that stat. Most crew only contribute to the stat their assigned room affects.

### Specialty Match Bonus

Based on crew skill level and room match:

| Match | Multiplier at Lv 1 | Multiplier at Lv 10 |
|-------|-------------------|---------------------|
| Primary room | 1.10 | 1.50 |
| Secondary room | 1.05 | 1.25 |
| Other room | 1.02 | 1.10 |
| No crew assigned | 1.00 | — |

Formula: `match_base + (skill_level - 1) × growth_per_level`

| Match | Base | Growth/Level |
|-------|------|-------------|
| Primary | 1.10 | +0.044 |
| Secondary | 1.05 | +0.022 |
| Other | 1.02 | +0.009 |

## Skill Growth

Crew gain skill XP passively while stationed in a room. Time-based, not event-based.

- **Growth rate:** 1 skill level per ~3 days of being stationed (real wall-clock time)
- **XP curve:** `skill_xp_required(level) = 3 × 86400 × level` seconds (3 days × level number)
- Level 1→2: 3 days. Level 9→10: 27 days. Total to max: ~135 days.
- **Unassigned crew don't gain skill.**

Reassigning a crew member to a different room resets nothing — skill level is universal, not per-room.

## Crew Capacity

Base crew capacity: 2. Increased by Life Support room (+1 per level, base level gives +2 capacity for total of 4). Max capacity: 8 (Life Support level 6+).

| Life Support Level | Crew Capacity |
|-------------------|---------------|
| None built | 2 |
| Level 1 | 3 |
| Level 2 | 4 |
| Level 3 | 5 |
| Level 4 | 6 |
| Level 5 | 7 |
| Level 6+ | 8 |

## Recruitment

Crew are found during the voyage through narrative moments, not recruited from a pool. Every recruitment is a story event with a cost or risk.

### Sources

- **Rescue events** — "A drifting pod pings your sensors. Investigate? (costs 20 supplies)" → find a survivor
- **Trading posts** — hire a crew member for salvage (scaling cost with distance)
- **Derelict exploration** — explore a wreck, find someone in stasis
- **Boss rewards** — defeating a Norse boss may trigger a recruitment event with a higher starting skill crew member

### Frequency

Random with pity timer: recruitment events appear organically but with a guarantee of at least one opportunity every 1,500 ly. First opportunity appears within ~200 ly or the first few hours. Total available across the full 10,000 ly voyage: ~8-10 opportunities, so the player can be somewhat selective.

### Information Reveal

On recruitment, the player sees the crew member's **name and specialty only**. Traits are hidden and reveal over time:

- **Ship trait:** reveals after 1 day aboard the ship
- **Room trait:** reveals after 3 days stationed in any room

Until revealed, traits show as "???" in the crew panel. This creates a "getting to know your crew" arc.

### Capacity and Hard Choices

Recruitment is gated by Life Support crew capacity. If the ship is at capacity when a recruitment event fires:

- The player sees the new person's name and specialty
- They must choose: **dismiss an existing crew member** to make room, or **let the new person go**
- Dismissed crew are gone forever
- This creates meaningful "is this person better than who I have?" decisions

If the ship is below capacity, the player can simply accept or decline.

### Dismissal

Crew can be dismissed at any time from the crew management screen to free capacity. Dismissed crew are gone forever — no undo, no re-recruitment.

## Injuries and Loss

### Injuries

Crew can be injured during combat or dangerous events:
- **Light injury:** 1 day recovery. Crew can't be assigned during recovery.
- **Severe injury:** 3 day recovery.
- **Medbay reduces recovery time** by 10% per level.

Injury chance per combat encounter: 5% base, reduced by Hull stat and Cautious trait.

### Permanent Loss

Crew can be permanently lost from:
- **Catastrophic events** — rare decision events with high-risk choices
- **Hull reaching 0** — 20% chance per crew member of being lost when entering drift from combat

Lost crew are gone forever. This is the primary emotional stakes of the voyage.

### Injury Protection

- Medbay prevents severe injuries from becoming permanent loss (same pattern as Deep Medic archetype)
- Cautious ship trait reduces injury chance
- High Hull stat reduces injury chance

## Supplies

Crew consume supplies proportional to headcount:
- **Drain rate:** 1 supply per crew member per day
- At 6 crew: 6 supplies/day
- Garden room generates supplies to offset this

When supplies hit 0:
- All crew effectiveness drops by 50% (multipliers halved)
- Morale penalty: skill growth pauses
- No crew loss from starvation — they survive but perform poorly

## Starting State

At launch:
- 0 crew aboard
- Crew capacity: 2 (no Life Support built yet)
- First crew member is offered during the first decision event (~500 ly or first few hours)

## UI: Crew Management

Accessed via `[C]` hotkey from the voyage screen:

```
┌─ Crew (3/6 capacity) ────────────────────────────────┐
│                                                       │
│  1. Brynn                                             │
│     Specialty: Navigation  Skill: Lv 4                │
│     Ship: Vigilant (+5% evasion)                      │
│     Room: Precise (+10% stat contribution)            │
│     Assigned: Engines                        [Active] │
│                                                       │
│  2. Kael                                              │
│     Specialty: Weapons     Skill: Lv 2                │
│     Ship: Resourceful (+10% salvage)                  │
│     Room: Industrious (+15% output)                   │
│     Assigned: Weapons Bay                    [Active] │
│                                                       │
│  3. Lyra                                              │
│     Specialty: Lore        Skill: Lv 1                │
│     Ship: Lucky (+5% component drops)                 │
│     Room: Efficient (-1 power)                        │
│     Assigned: (none)                      [Unassigned]│
│                                                       │
│  [A] Assign  [U] Unassign  [D] Dismiss  [Esc] Back   │
└───────────────────────────────────────────────────────┘
```

Assigning shows a room picker with match quality indicators (Primary/Secondary/Other).

## Files

| File | Change |
|------|--------|
| `src/vessel/crew.rs` | New: crew generation, skill growth, injury, recruitment, assignment |
| `src/vessel/types.rs` | Modify: add CrewMember, Specialty, ShipTrait, RoomTrait to VesselState |
| `src/vessel/stats.rs` | Modify: integrate crew multiplier into stat derivation |
| `src/vessel/tick.rs` | Modify: tick skill growth, supply drain from crew |
| `src/ui/vessel_crew_scene.rs` | New: crew management overlay rendering |
| `src/input/vessel_input.rs` | Modify: add [C] hotkey and crew management input |

## Testing

- Unit test: crew multiplier calculation for primary/secondary/other match at various skill levels
- Unit test: skill growth rate (time to level up)
- Unit test: crew capacity scales with Life Support level
- Unit test: supply drain proportional to crew count
- Unit test: injury recovery time reduced by Medbay
- Unit test: crew loss chance on drift
- Unit test: each ship trait applies correct bonus
- Unit test: each room trait applies correct bonus
- Unit test: supplies at 0 halves crew effectiveness
- Unit test: dismiss removes crew permanently
