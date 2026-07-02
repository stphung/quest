# The Vessel — Act 2

**Issue:** Post-Loom endgame progression

## Overview

After completing all 28 Woven Patterns, reaching Ascension X, conquering Zone 50, and spending 100,000 PR, the player discovers that their branch of Yggdrasil is dying. A beacon on a distant living branch calls to them. The Loom's woven reality becomes the hull of a ship — the Vessel — and the player launches into the void between worlds. This begins Act 2: the game shifts from zone-based combat to a voyage through cosmic space, with the ship as the new "character."

## Narrative Foundation

**Yggdrasil is dying.** The player has been fighting on a single branch of the World Tree. Zone 50 (The Origin Thread) reveals the truth: the branch is withering from the root. The fractures, the void, the corruption — all symptoms of a dying limb.

**The beacon.** After Z50 falls, a signal appears from impossibly far away — another branch, still alive. The Loom resonates with it. The 28 Woven Patterns weren't just sustaining reality locally; they were unconsciously answering a call.

**The ship IS the Loom.** The Loom's woven fate becomes the literal hull of the Vessel. The Deep's Gateway was digging toward the roots. The patterns were the blueprint. Everything the player built was preparation for this moment. 100,000 PR fuels the transformation — reality itself is burned as fuel to reshape the Loom into something that can travel between branches.

**The destination.** A living branch of Yggdrasil, 10,000 light-years away. Visible on sensors from launch. The gate to Act 3 (colony/settlement — future content).

## Launch Requirements

All four must be met:

| Requirement | Purpose |
|-------------|---------|
| 28 Woven Patterns complete | The Loom is fully built — becomes the hull |
| Ascension X | Maximum power — the character is ready |
| Zone 50 conquered | The Origin Thread reveals the dying branch |
| 100,000 PR spent | Fuel for the transformation |

The 100,000 is **spent from prestige rank in a single all-or-nothing burn at launch confirmation** (see the launch-gate spec). Rank never freezes and there is no partial banking — the gate is simply holding 100,000 PR at once, and the hero fights at full prestige bonuses for the entire wait. WR→PR is `PR/hr = WR × (1 + WR/100)` (`src/loom/logic.rs`), plus Power Cores at up to 48 PR/day:

| Situation | PR/day income | Time until P100,000 |
|-----------|--------------|---------------------|
| Veteran already holding 100k+ | — | Immediate — launch the moment the signal appears |
| At the gate near P50k, typical Loom (50 WR/hr) | ~1,850 | ~27 days |
| At the gate near P50k, maxed Loom (131 WR/hr) | ~7,320 | ~7 days |

So the construction watch runs **zero days to about a month**. Upgrading extractors toward L20 is the big income lever during the wait. (Challenge wins grant only 1-2 PR each — negligible at these rates.)

## Mode Shift

Launching the Vessel is an **alternate game mode**. The player is no longer fighting in zones. The entire UI shifts to the voyage. The old world runs in the background as a supply line but is not directly playable.

**Act 1 carryover:** Clean break. Only PR transmissions carry over. The 100,000 PR IS the carryover — everything the player built was consumed to create the ship. No God Items, Haven bonuses, or Ascension multipliers transfer. The Vessel is a fresh start.

## The Vessel as Character

The ship replaces the hero as the thing you grow and upgrade.

### Ship Stats

Stats come from three sources that stack: base ship level (XP from combat), room bonuses, and crew assignments.

| Stat | Analogue | Governs |
|------|----------|---------|
| Firepower | Power | Auto-combat damage against void hostiles |
| Hull | HP/Defense | Ship health, passive damage resistance |
| Engines | Speed | Distance traveled per day, evasion in combat |
| Sensors | Discovery | Detection range for encounters, derelicts, resources |

**Stat growth layers:**
1. **Base stats** — the ship gains XP from combat encounters and levels up, increasing base stats (like the hero)
2. **Room bonuses** — each built and upgraded room adds to relevant stats (Weapons Bay adds Firepower, Hull Plating adds Hull, etc.)
3. **Crew assignments** — crew members stationed in a room boost its output based on their skills. A gunner in an upgraded Weapons Bay significantly amplifies Firepower.

### Crew (5-8 members)

The Vessel carries a small, tight-knit crew. Every member matters — losing one is significant.

**Recruitment:** Crew are found during the voyage — at trading posts, rescued from derelicts, discovered in anomalies. You don't start with a full crew.

**Crew members have:**
- A name and background (generated)
- A role/specialty (Gunner, Engineer, Navigator, Medic, Scholar, etc.)
- A skill level that improves with time stationed in a room
- Status: Active, Injured, or Lost

**Assignment:** Each room can have one crew member assigned. A crew member's specialty determines how much they boost the room. A Gunner in Weapons Bay gives a large bonus; a Gunner in the Garden gives a small one. Unassigned rooms still function at base level.

**Crew needs:** Supplies drain proportional to crew count. Quarters provide rest (morale). Medbay recovers injured crew. Losing crew to events or combat is permanent — replacement requires finding someone new.

### Ship Rooms (~20 total)

Room slots unlock at geometrically spaced distance milestones — roughly one every 1-2 weeks of wall-clock under the speed curve (see rooms spec). What you build in each slot is your choice.

**Room upgrade system:** Two axes of improvement:
1. **Levels (1-10)** — spend salvage/materials to level up a room. Each level increases its base stat contribution. Straightforward resource investment.
2. **Component slots (2-3 per room)** — each room has slots for components found from encounters, derelicts, and events. Components modify what the room does — add effects, boost specific stats, enable new capabilities. Discovery-driven, not grindy.

Levels are the floor (reliable growth from resources), components are the ceiling (exciting finds that change how a room works).

**Core Systems:**
- **Reactor** — power generation, limits how many rooms can be active
- **Hull Plating** — ship HP and passive defense
- **Engines** — speed (distance/tick and evasion)
- **Weapons Bay** — auto-combat damage

**Survival:**
- **Fuel Refinery** — converts raw void matter into fuel, slowing the drain
- **Cargo Hold** — resource storage capacity
- **Life Support** — crew capacity ceiling and efficiency

**Exploration:**
- **Sensors Array** — detection range for encounters, derelicts, resources
- **Shuttle Bay** — send smaller craft to investigate things you pass
- **Cartography Deck** — maps ahead, reveals what's coming

**Crew/Narrative:**
- **Quarters** — crew rest, morale, passive bonuses
- **Medbay** — crew recovery after combat/events
- **Shrine** — Norse themed, provides blessings/buffs (mythology tie-in)

**Production:**
- **Forge** — craft upgrades and components from salvage
- **Garden/Hydroponics** — passive supply generation
- **Workshop** — repair hull over time

**Special/Late-Game:**
- **Rune Array** — channel old-world PR transmissions more efficiently
- **Void Lens** — see further, unlock hidden encounters
- **Fate Loom** — miniature loom, weave minor patterns for ship bonuses

## Core Loop

Three layers running simultaneously, at different rhythms:

### Layer 1: Auto-Combat (distance-based)

The ship encounters hostiles in the void and fights them automatically, like zone combat. Frequency scales with distance traveled — faster engines mean more encounters per day. Enemies scale with distance from origin. Loot is salvage, materials, and occasionally components for room upgrades. The ship gains XP from combat, leveling up base stats.

### Layer 2: Resource Management (continuous)

Three resources drain over time:
- **Fuel** — consumed by engines. Run out = drift.
- **Hull integrity** — degrades from combat and void hazards. Reaches 0 = drift.
- **Supplies** — consumed by crew (proportional to crew count). Run out = efficiency penalties, morale drop.

Resources are replenished by harvesting (void matter), salvaging (combat loot), and production rooms (Refinery, Garden, Workshop).

### Layer 3: Decision Events (time-based, wall-clock)

Every few hours, a narrative event fires regardless of distance:
- Derelict ships to explore (loot, components, sometimes crew)
- Distress signals to answer or ignore (risk/reward)
- Spatial anomalies to study or avoid (Sensors reveal hidden options)
- Trading posts / wayfarers to barter with
- Crew morale situations
- Norse mythological encounters (echoes of the old world)

Choices depend on ship capabilities (Sensors reveal hidden options, Shuttle Bay enables boarding, Shrine provides blessings). Auto-resolve picks the safe path if the player doesn't respond — the safe path always avoids resource loss but may miss rewards.

## Distance and Progression

**Destination:** 10,000 light-years.

**Duration target: ~8 months of engaged idle play** (240 ± 60 days). Active optimizers land closer to 5-6 months; fully passive play drifts toward 12. This is the master pacing constant — the speed curve, milestone spacing, and enemy scaling all derive from it, and the vessel simulator (planned alongside sub-project 4) validates changes against it.

**Speed** is derived from the ship's final Engines stat (formula and anchors in the rooms spec): `speed_ly_per_day = Engines² / 1,000`, capped at 100 ly/day. Intended trajectory:

| Phase | Speed | Wall-clock |
|-------|-------|-----------|
| Launch | 0.1 ly/day (Engines 10) | day 0 |
| Early voyage | ~1 ly/day (Engines ~32) | ~month 2 |
| Mid voyage | ~10-20 ly/day (Engines ~100-140) | ~month 4 |
| Cruise plateau | 60-100 ly/day (Engines 250-316) | final ~3 months |

Speed roughly doubles every ~2 weeks through the ramp, then plateaus. The design principle: **distance is exponential, wall-clock is linear.**

**Distance milestones are spaced geometrically** (~×1.5-1.6 apart), so under the exponential speed curve each unlock arrives roughly every 1-2 weeks of wall-clock regardless of raw distance:
- New room slots (16 unlocks from 10 ly to 9,000 ly — see rooms spec)
- New encounter types and enemy tiers
- Story beats and narrative revelations about the destination
- New room types available to build

**Scaling:** Enemies, resource scarcity, and event complexity scale with distance on sublinear power curves (distance spans three orders of magnitude — linear scaling would explode). The void gets harder the further you go.

## Supply Line from Home

The old world doesn't disappear. The Loom, Power Cores, and WR->PR conversion keep running as a background supply line.

**Transmissions:** PR generated by the old world trickles to the ship as a raw resource. Rate diminishes with distance:
- 0-1,000 ly: 100% of old-world PR generation arrives
- 1,000-3,000 ly: 50%
- 3,000-7,000 ly: 25%
- 7,000-10,000 ly: 10%

This creates a natural bridge: early voyage is subsidized by old infrastructure, late voyage the ship must be self-sustaining.

Transmissions can be used as a universal crafting/upgrade material or converted into the ship's native resources via the Rune Array room.

## Failure and Recovery

**Drift state:** If hull reaches 0 or fuel runs out, the ship doesn't die. It enters drift — a low-power recovery mode. The ship stops moving. Old-world transmissions slowly restore fuel/hull. The player can also sacrifice distance (backtrack) to reach safer space.

Drift is a setback, not permadeath. It creates tension without punishment.

## Theme

**Norse roots, cosmic ocean.** The ship is built from the World Tree's woven fate. Its rooms have Norse names. The Shrine channels the old gods. But what you sail through is alien and unknowable — the space between branches of Yggdrasil is not empty. It's full of things that don't belong to any mythology. The familiar Norse elements are the player's anchor in an unfamiliar void.

The destination — the living branch — grows clearer on sensors as you approach. What you find there is Act 3.

## Sub-Project Specs

| Sub-Project | Spec | Status |
|-------------|------|--------|
| 1. Launch Gate & Vessel Overlay | [vessel-launch-gate-design.md](2026-03-27-vessel-launch-gate-design.md) | Designed |
| 2. Mode Transition & Basic Voyage Shell | [vessel-mode-transition-design.md](2026-03-27-vessel-mode-transition-design.md) | Designed |
| 3. Room System & Ship Stats | [vessel-rooms-stats-design.md](2026-03-27-vessel-rooms-stats-design.md) | Designed |
| 4. Auto-Combat | [vessel-combat-design.md](2026-03-27-vessel-combat-design.md) | Designed |
| 5. Crew System | [vessel-crew-design.md](2026-03-27-vessel-crew-design.md) | Designed |
| 6. Decision Events | — | Not started |
| 7. Supply Line | — | Not started |

## Future: Act 3 (Colony/Settlement)

Not designed yet. The destination is the gate. Arriving at the living branch opens a new phase: building a settlement, establishing a foothold, discovering a new world. The ship becomes the seed of a colony. This is future content — the Vessel spec only covers the voyage.
