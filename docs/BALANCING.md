# Quest Balancing Guide

How to tune Quest's game economy without breaking progression. This document covers balance philosophy, key levers, danger zones, and testing methodology. Every number in this document is sourced from the codebase (primarily `src/core/constants.rs` and the module CLAUDE.md files).

## Table of Contents

1. [Balance Philosophy](#balance-philosophy)
2. [The Core Loop](#the-core-loop)
3. [Progression Pacing](#progression-pacing)
4. [Key Balance Levers](#key-balance-levers)
5. [System Interactions](#system-interactions)
6. [Danger Zones](#danger-zones)
7. [Progression Simulation Findings](#progression-simulation-findings)
8. [Testing & Validation](#testing--validation)
9. [Common Tuning Scenarios](#common-tuning-scenarios)
10. [Appendix: Current Constants](#appendix-current-constants)

---

## Balance Philosophy

### Idle RPG Principles

Quest is an **idle RPG** -- balance should support:

1. **Meaningful AFK progress** -- Players should feel rewarded for leaving the game running
2. **Active play advantage** -- But active decisions (prestige timing, minigames, Haven) should outpace pure idling
3. **Long-term goals** -- Endgame (Stormbreaker) should take weeks/months, not hours
4. **No hard walls** -- Progress should slow but never stop completely
5. **Prestige feel-good** -- Each reset should feel like a meaningful power boost

### The Golden Ratio

```
Active play should be ~2-3x more efficient than pure idle.
```

This means:
- Winning minigames for prestige ranks beats grinding levels
- Strategic prestige timing beats waiting for max level
- Haven investment pays off over multiple prestiges

### Player Psychology Targets

| Milestone | Target Time | Feel |
|-----------|-------------|------|
| First prestige (P1) | 30-60 min | "I get it now" |
| Haven unlock (P10) | 8-12 hours | "New system!" |
| Stormbreaker | 2-4 weeks | "Finally!" |
| The Expanse cycles | Infinite | "One more run" |

---

## The Core Loop

```
                    CORE PROGRESSION LOOP

    +----------+
    |  Combat  |<-----------------------------+
    |  (Idle)  |                              |
    +----+-----+                              |
         | XP + Items                         |
         v                                    |
    +----------+                              |
    |  Level   |                              |
    |   Up     |                              |
    +----+-----+                              |
         | +3 Attributes                      |
         v                                    |
    +----------+     +----------+             |
    |  Power   |---->|  Zone    |             |
    | Increase |     | Progress |             |
    +----------+     +----+-----+             |
                          | Wall              |
                          v                   |
                    +----------+              |
                    | Prestige |--------------+
                    |  Reset   | (Multiplier boost)
                    +----------+
```

### What Makes This Work

1. **XP scales with prestige** -- Higher prestige = faster XP = faster levels
2. **Attribute caps scale** -- Higher prestige = higher potential power
3. **Zones gate progress** -- Can't rush ahead without prestige investment
4. **Multiplier diminishes** -- Each prestige matters less, preventing runaway

---

## Progression Pacing

### XP Curve

Formula: `xp_needed = 100 * level^1.5`

Constants: `XP_CURVE_BASE = 100.0`, `XP_CURVE_EXPONENT = 1.5`

| Level | XP Needed | Time at P0* | Time at P10* |
|-------|-----------|-------------|--------------|
| 10 | 3,162 | ~5 min | ~1.5 min |
| 50 | 35,355 | ~1 hour | ~20 min |
| 100 | 100,000 | ~3 hours | ~50 min |
| 200 | 283,000 | ~8 hours | ~2.5 hours |

*Approximate, assuming constant combat with average kill XP.

### Prestige Tiers and Level Requirements

| Prestige | Tier Name | Required Level | Multiplier (`1 + 0.5 * rank^0.7`) | Attribute Cap (`20 + 5*rank`) |
|----------|-----------|----------------|--------------------------------------|-------------------------------|
| P0 | None | -- | 1.00x | 20 |
| P1 | Bronze | 10 | 1.50x | 25 |
| P2 | Silver | 25 | 1.81x | 30 |
| P3 | Gold | 50 | 2.08x | 35 |
| P4 | Platinum | 65 | 2.32x | 40 |
| P5 | Diamond | 80 | 2.54x | 45 |
| P6 | Emerald | 90 | 2.75x | 50 |
| P7 | Sapphire | 100 | 2.95x | 55 |
| P8 | Ruby | 110 | 3.14x | 60 |
| P9 | Obsidian | 120 | 3.33x | 65 |
| P10 | Celestial | 130 | 3.51x | 70 |
| P11 | Astral | 140 | 3.68x | 75 |
| P12 | Cosmic | 150 | 3.85x | 80 |
| P13 | Stellar | 160 | 4.01x | 85 |
| P14 | Galactic | 170 | 4.17x | 90 |
| P15 | Transcendent | 180 | 4.33x | 95 |
| P16 | Divine | 190 | 4.48x | 100 |
| P17 | Exalted | 200 | 4.63x | 105 |
| P18 | Mythic | 210 | 4.78x | 110 |
| P19 | Legendary | 220 | 4.93x | 115 |
| P20 | Eternal | 235 | 5.07x | 120 |
| P25 | Eternal | 310 | 5.76x | 145 |
| P30 | Eternal | 385 | 6.41x | 170 |
| P100 | Eternal | 1435 | 13.56x | 520 |

Level requirement formula for P20+: `220 + (rank - 19) * 15`

**Key insight**: The multiplier uses diminishing returns (`rank^0.7`), so later prestiges give smaller percentage gains. Each prestige cycle takes longer, creating the "wall" feeling that makes prestiging satisfying.

### Prestige Multiplier Formula

```
multiplier = 1.0 + 0.5 * rank^0.7
```

Constants: `PRESTIGE_MULT_BASE_FACTOR = 0.5`, `PRESTIGE_MULT_EXPONENT = 0.7`

The prestige multiplier is further enhanced by Charisma:
```
effective_multiplier = base_multiplier + (CHA_modifier * 0.1)
```

Where `CHA_modifier = (CHA_value - 10) / 2` (integer division).

---

## Key Balance Levers

### Lever 1: XP Curve Exponent

```rust
// src/core/constants.rs
XP_CURVE_BASE = 100.0
XP_CURVE_EXPONENT = 1.5
// Formula: xp_needed = 100 * level^1.5
```

| Exponent | Effect |
|----------|--------|
| 1.3 | Faster leveling, shorter prestige cycles |
| **1.5** | **Current** -- balanced idle pacing |
| 1.7 | Slower leveling, more grind per prestige |
| 2.0 | Very slow -- only for hardcore modes |

**When to adjust**: If prestiges feel too fast/slow.

### Lever 2: Prestige Multiplier Formula

```rust
// src/core/constants.rs
PRESTIGE_MULT_BASE_FACTOR = 0.5
PRESTIGE_MULT_EXPONENT = 0.7
// Formula: 1.0 + 0.5 * rank^0.7
```

| Formula | P1 | P10 | P20 | Character |
|---------|-----|------|------|-----------|
| `1 + 0.3 * rank^0.7` | 1.3x | 2.5x | 3.5x | Slower power curve |
| **`1 + 0.5 * rank^0.7`** | **1.5x** | **3.5x** | **5.1x** | **Current** |
| `1 + 0.7 * rank^0.7` | 1.7x | 4.5x | 6.7x | Faster power curve |

**When to adjust**: If prestige feels unrewarding (increase) or trivializes content (decrease).

### Lever 3: Kill XP Range

```rust
// src/core/constants.rs
COMBAT_XP_MIN_TICKS = 200
COMBAT_XP_MAX_TICKS = 400
// Kill XP = xp_per_tick * random(200..=400)
```

| Range | Effect |
|-------|--------|
| 100-200 | More passive-like, longer fights matter less |
| **200-400** | **Current** -- kills are significant |
| 300-600 | Kills dominate, pure idle is weak |

**When to adjust**: If combat feels unrewarding vs pure idling.

### Lever 4: Attribute Scaling

Six attributes with formula: `modifier = (value - 10) / 2` (integer division, min 0)

| Attribute | Effect | Per Modifier Point |
|-----------|--------|--------------------|
| STR | Physical damage | +2 damage (`5 + STR_mod * 2`) |
| DEX | Defense and crit | +1 defense, +1% crit chance (`5% + DEX_mod`) |
| CON | Max HP | +10 HP (`50 + CON_mod * 10`) |
| INT | Magic damage | +2 damage (`5 + INT_mod * 2`) |
| WIS | XP gain | +5% XP per modifier (`1.0 + WIS_mod * 0.05`) |
| CHA | Prestige multiplier | +0.1 per modifier point |

Base attribute value: 10. On level-up: +3 random points distributed among non-capped attributes.

Attribute cap: `BASE_ATTRIBUTE_CAP (20) + ATTRIBUTE_CAP_PER_PRESTIGE (5) * prestige_rank`

### Lever 5: Drop Rates

```rust
// src/core/constants.rs
ITEM_DROP_BASE_CHANCE = 0.15       // 15% per kill
ITEM_DROP_PRESTIGE_BONUS = 0.01   // +1% per prestige rank
ITEM_DROP_MAX_CHANCE = 0.25        // 25% hard cap
```

Mob drop rate formula:
```
base_chance = 0.15 + (prestige_rank * 0.01)
drop_chance = min(base_chance * (1.0 + haven_drop_bonus/100), 0.25)
```

**Mob rarity distribution** (base at P0, no Haven):
- Common: 60%, Magic: 28%, Rare: 10%, Epic: 2%, Legendary: **never**
- Prestige bonus: +1% per rank (capped at 10%) shifts Common downward
- Workshop bonus: shifts distribution toward higher rarities (max 25%)
- Common floor: 20% minimum

**Boss rarity distribution** (fixed, no Haven/prestige bonuses):

| Boss Type | Magic | Rare | Epic | Legendary |
|-----------|-------|------|------|-----------|
| Normal boss | 40% | 35% | 20% | 5% |
| Zone 10 final boss | 20% | 40% | 30% | 10% |

Bosses always drop an item. Bosses never drop Common.

### Lever 6: Haven Bonuses

Each Haven room has tiered values. Bonuses are percentage-based.

| Room | Bonus Type | T1 | T2 | T3 | T4 | Max Tier |
|------|-----------|-----|-----|-----|-----|----------|
| Hearthstone | Offline XP | +25% | +50% | +100% | -- | 3 |
| Armory | Damage | +5% | +10% | +25% | -- | 3 |
| Training Yard | XP Gain | +5% | +10% | +30% | -- | 3 |
| Trophy Hall | Drop Rate | +5% | +10% | +15% | -- | 3 |
| Watchtower | Crit Chance | +5% | +10% | +20% | -- | 3 |
| Alchemy Lab | HP Regen Speed | +25% | +50% | +100% | -- | 3 |
| War Room | Double Strike | +10% | +20% | +35% | -- | 3 |
| Bedroom | Regen Delay Reduction | -15% | -30% | -50% | -- | 3 |
| Garden | Fishing Timer Reduction | -10% | -20% | -40% | -- | 3 |
| Library | Challenge Discovery | +20% | +30% | +50% | -- | 3 |
| Fishing Dock | Double Fish Chance | +25% | +50% | +100% | +10 Max Rank | 4 |
| Workshop | Item Rarity | +10% | +15% | +25% | -- | 3 |
| Vault | Items Preserved on Prestige | 1 | 3 | 5 | -- | 3 |
| Storm Forge | Stormbreaker forging | enabled | -- | -- | -- | 1 |

**Haven prestige rank costs** (costs scale with tree depth):

| Depth | Rooms | T1 | T2 | T3 |
|-------|-------|-----|-----|-----|
| 0 | Hearthstone | 1 | 2 | 3 |
| 1 | Armory, Bedroom | 1 | 3 | 5 |
| 2-3 | Mid-tree rooms | 2 | 4 | 6 |
| 4 | War Room, Vault | 3 | 5 | 7 |

Special costs: Fishing Dock T4 = 10 prestige ranks. Storm Forge = 25 prestige ranks.

**Danger**: Haven bonuses are permanent and cumulative. Small changes compound across all future play.

---

## System Interactions

### Interaction Matrix

```
           | Prestige | Haven  | Items  | Fishing | Challenges
-----------+----------+--------+--------+---------+----------
Prestige   |    -     | Gates  | Reset  | Persist | Rewards PR
Haven      | Currency |   -    | Rarity | Rank Cap| Discovery
Items      | Lost*    | Vault  |   -    | Drops   |    -
Fishing    | Persist  | Dock   | Drops  |    -    | Ranks
Challenges | +Ranks   | Library|   -    | +Ranks  |    -
```

*Items are lost on prestige unless preserved by Vault (1/3/5 items at T1/T2/T3).

### Critical Chains

**1. Prestige -> Haven -> Everything**
```
More prestige -> More Haven rooms -> Permanent bonuses -> Faster prestige
```
This is a **virtuous cycle** -- players who engage with Haven accelerate faster.

**2. Fishing -> Stormbreaker Gate**
```
Fishing Rank 40 -> Storm Leviathan (10 encounters) -> Storm Forge -> Zone 10 boss
```
This chain gates endgame. If fishing is too fast/slow, endgame timing shifts.

**3. Challenges -> Prestige Shortcuts**
```
Minigame wins -> +Prestige ranks -> Skip level grinding
```
Skilled players can prestige faster via minigames.

---

## Danger Zones

### Do NOT Touch Without Testing

| Constant | Risk |
|----------|------|
| `TICK_INTERVAL_MS` (100ms) | Breaks all timing, UI responsiveness |
| `BASE_XP_PER_TICK` (1.0) | Ripples through entire XP economy |
| Zone prestige requirements | Blocks/trivializes content |
| Prestige level requirements | Core progression pacing |
| `MAX_FISHING_RANK` (40) | Breaks Stormbreaker chain |

### High-Impact Changes

| Change | Ripple Effects |
|--------|----------------|
| XP curve exponent | All level timings, prestige pacing |
| Prestige multiplier | Power curve, Haven value |
| Haven T3 bonuses | Endgame power ceiling |
| Challenge prestige rewards | Speedrun strategies |

### Safe to Tune

| Change | Isolated To |
|--------|-------------|
| Fish rarity weights | Fishing feel |
| Enemy name syllables | Flavor only |
| Item affix ranges | Item power variance |
| Dungeon room types | Dungeon variety |
| UI colors/layout | Presentation |

---

## Progression Simulation Findings

Based on analysis of game balance parameters and zone prestige gates, here is what a simulated progression looks like at different prestige milestones.

### P0: Zones 1-2 Only (6 subzones)

At P0 the player has access only to Meadow (Zone 1) and Dark Forest (Zone 2). With a 1.0x multiplier, attribute cap of 20, and ilvl 10-20 gear, the player is stuck in zones 1-2 until they reach level 10 and prestige. There is no zone progression wall per se, just the prestige level requirement.

### P5: Zones 1-4 (12 subzones)

At P5 (Diamond), zones 3-4 (Mountain Pass, Ancient Ruins) unlock. Multiplier is ~2.54x, attribute cap is 45. The player can clear all 12 subzones across zones 1-4 before hitting the P10 gate for zones 5-6. Progression through zones 1-2 is fast; zones 3-4 provide the meaningful challenge at this tier.

### P10: Zones 1-6 (20 subzones), Haven discovered

At P10 (Celestial), zones 5-6 unlock (Volcanic Wastes, Frozen Tundra). Multiplier is ~3.51x, attribute cap is 70. Haven discovery becomes possible (base chance 0.000014/tick). This is the first major "breakout" point -- the player has access to 4-subzone zones for the first time and can start investing prestige ranks into Haven rooms for permanent bonuses. With Haven bonuses stacking, subsequent prestiges accelerate.

### P15: Zones 1-8 (28 subzones)

At P15 (Transcendent), zones 7-8 (Crystal Caverns, Sunken Kingdom) unlock. Multiplier is ~4.33x, attribute cap is 95. Haven should be well-established with several rooms upgraded. The player can clear 28 total subzones before hitting the P20 gate.

### P20: Zones 1-10 (36 subzones), Stormbreaker-blocked

At P20 (Eternal), all zones are unlocked. The player can progress through 35 subzones (zones 1-10, subzones 1-3 of Storm Citadel). However, Zone 10's final boss ("The Undying Storm") requires the Stormbreaker weapon, creating the ultimate gate. To break through, the player must:

1. Max fishing rank to 40 (requires Fishing Dock T4 at 10 prestige ranks to unlock ranks 31-40; ~701,000 total fish)
2. Complete the 10-encounter Storm Leviathan hunt
3. Build the Storm Forge in Haven (25 prestige ranks)
4. Forge Stormbreaker
5. Defeat The Undying Storm

### Post-Game: Zone 11 (The Expanse)

After completing Zone 10, the player unlocks The Expanse (Zone 11), an infinite 4-subzone zone that cycles endlessly. Min level 150, no level cap. This provides infinite post-game content.

---

## Zone Map Reference

| Zone | Name | Prestige Req | Level Range | Subzones | Boss |
|------|------|-------------|-------------|----------|------|
| 1 | Meadow | P0 | 1-10 | 3 | Sporeling Queen |
| 2 | Dark Forest | P0 | 10-25 | 3 | Broodmother Arachne |
| 3 | Mountain Pass | P5 | 25-40 | 3 | Frost Wyrm |
| 4 | Ancient Ruins | P5 | 40-55 | 3 | Lich King's Shade |
| 5 | Volcanic Wastes | P10 | 55-70 | 4 | Infernal Titan |
| 6 | Frozen Tundra | P10 | 70-85 | 4 | The Frozen One |
| 7 | Crystal Caverns | P15 | 85-100 | 4 | Crystal Colossus |
| 8 | Sunken Kingdom | P15 | 100-115 | 4 | The Drowned King |
| 9 | Floating Isles | P20 | 115-130 | 4 | Tempest Lord |
| 10 | Storm Citadel | P20 | 130-150 | 4 | The Undying Storm* |
| 11 | The Expanse | StormsEnd | 150+ | 4 (cycling) | Avatar of Infinity |

*Requires Stormbreaker weapon to defeat final boss.

---

## Item System

### Item Level Scaling

Items scale with zone progression: `ilvl = zone_id * 10`

ilvl multiplier formula: `1.0 + (ilvl - 10) / 30`

| Zone | ilvl | Multiplier | Effect |
|------|------|------------|--------|
| 1 | 10 | 1.0x | Baseline |
| 3 | 30 | 1.67x | Early prestige |
| 5 | 50 | 2.33x | Mid-game |
| 7 | 70 | 3.0x | Late-game |
| 10 | 100 | 4.0x | Endgame |

### Generation Rules by Rarity

Base attribute ranges at ilvl 10 (scaled by ilvl multiplier), 1-3 random attributes:

| Rarity | Base Attr Range | Affixes | At ilvl 10 | At ilvl 100 (4.0x) |
|--------|----------------|---------|------------|---------------------|
| Common | 1 | 0 | 1-3 total | 4-12 total |
| Magic | 1-2 | 1 | 1-6 total | 4-24 total |
| Rare | 2-3 | 2-3 | 2-9 total | 8-36 total |
| Epic | 3-4 | 3-4 | 3-12 total | 12-48 total |
| Legendary | 4-6 | 4-5 | 4-18 total | 16-72 total |

### 9 Affix Types

DamagePercent, CritChance, CritMultiplier, AttackSpeed, HPBonus, DamageReduction, HPRegen, DamageReflection, XPGain

### Equipment Slots

7 slots: Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring

### Fishing Item Drops

Drop chance by fish rarity:
- Common/Uncommon: 5%
- Rare: 15%
- Epic: 35%
- Legendary: 75%

Item rarity matches fish rarity. Item ilvl based on current zone.

---

## Fishing System

### Rank Progression (40 ranks, 8 tiers)

| Tier | Ranks | Fish/Rank | Cumulative Fish | Notes |
|------|-------|-----------|-----------------|-------|
| Novice | 1-5 | 100 | 500 | |
| Apprentice | 6-10 | 200 | 1,500 | |
| Journeyman | 11-15 | 400 | 3,500 | |
| Expert | 16-20 | 800 | 7,500 | |
| Master | 21-25 | 1,500 | 15,000 | |
| Grandmaster | 26-30 | 2,000 | 25,000 | Base max (without Haven) |
| Mythic | 31-35 | 4k-25k | 86,000 | Requires Fishing Dock T4 |
| Transcendent | 36-40 | 40k-250k | 701,000 | Storm Leviathan at rank 40 |

Constants: `BASE_MAX_FISHING_RANK = 30`, `MAX_FISHING_RANK = 40`

### Rarity System

Base catch chances (adjusted by rank):
- Common: 60%, Uncommon: 25%, Rare: 10%, Epic: 4%, Legendary: 1%
- Every 5 ranks: -2% Common, +1% Uncommon, +0.5% Rare, +0.3% Epic, +0.2% Legendary
- Common floor: 10% minimum

XP rewards by rarity:
- Common: 50-100, Uncommon: 150-250, Rare: 400-600, Epic: 1,000-1,500, Legendary: 3,000-5,000

### Fishing Phases

Each fish catch goes through three phases:
- **Casting** (0.5-1.5s): Line being cast
- **Waiting** (1-8s): Waiting for a bite
- **Reeling** (0.5-3s): Fish on the line

Session size: 3-8 fish per session.

### Storm Leviathan Hunt

Progressive 10-encounter hunt, only available at rank 40 on legendary fish catches:

1. Encounter chances (decreasing): 8%, 6%, 5%, 4%, 3%, 2%, 1.5%, 1%, 0.5%, 0.25%
2. After 10 encounters: 25% catch chance per legendary fish
3. Catching awards 10,000-15,000 XP and enables Stormbreaker forging

---

## Combat System

### Timing Constants

| Constant | Value | Notes |
|----------|-------|-------|
| Tick interval | 100ms | 10 ticks/sec |
| Attack interval | 1.5s | 15 ticks between attacks |
| HP regen duration | 2.5s | After kill, full HP restore |
| Autosave | 30s | |
| Update check | 30 min +/- 5 min | |

### Damage Formula

```
physical_damage = 5 + (STR_modifier * 2)
magic_damage = 5 + (INT_modifier * 2)
total_damage = physical_damage + magic_damage
effective_damage = max(1, total_damage - enemy_defense)
```

With Haven Armory bonus: `damage * (1 + armory_percent / 100)`

### Critical Hits

- Base crit chance: 5% + DEX_modifier
- Haven Watchtower bonus adds flat percentage
- Base crit multiplier: 2.0x (enhanced by CritMultiplier affix)

### Double Strike (War Room)

- Chance per attack to hit twice
- Second strike uses same damage but is not a crit

### Death Mechanics

- **Death to zone boss**: Resets encounter (`fighting_boss=false`, `kills_in_subzone=0`), preserves prestige
- **Death in dungeon**: Exits dungeon, no prestige loss, dungeon progress lost

### Boss Spawn

`KILLS_FOR_BOSS = 10` -- after 10 kills in a subzone, the subzone boss spawns.

---

## Offline Progression

### Formula

```
estimated_kills = (elapsed_seconds / 5.0) * OFFLINE_MULTIPLIER
xp_per_kill = xp_per_tick_rate * avg_ticks_per_kill
offline_xp = estimated_kills * xp_per_kill * (1 + haven_offline_bonus / 100)
```

Constants:
- `OFFLINE_MULTIPLIER = 0.25` (25% of online kill rate)
- `MAX_OFFLINE_SECONDS = 604800` (7 days)
- Average ticks per kill: `(200 + 400) / 2 = 300`
- Estimated kill rate: 1 kill every 5 seconds (includes combat + regen time)

### Modifiers (all multiplicative)

- **Prestige multiplier**: `1 + 0.5 * rank^0.7` (plus CHA bonus)
- **WIS modifier**: `1 + WIS_mod * 0.05`
- **Haven Hearthstone**: T1 +25%, T2 +50%, T3 +100% offline XP

### Example: 1 Hour Offline at P0

```
kills = (3600 / 5) * 0.25 = 180
xp_per_kill = 1.0 * 300 = 300
total_xp = 180 * 300 = 54,000
```

At P10 with Hearthstone T3 and WIS +5:
```
kills = 180
prestige_mult = 3.51
wis_mult = 1.25
haven_mult = 2.0
xp_per_kill = 1.0 * 3.51 * 1.25 * 300 = 1,316
total_xp = 180 * 1,316 * 2.0 = 473,760
```

---

## Discovery Chances

| Discovery | Chance | Condition | Notes |
|-----------|--------|-----------|-------|
| Dungeon | 2% per kill | Always | Blocked by active dungeon/fishing |
| Fishing spot | 5% per kill | Always | Blocked by active fishing/dungeon |
| Challenge | 0.0014% per tick | P1+ required | ~2hr average; Haven Library boosts |
| Haven | 0.0014% per tick + 0.0007% per rank above 10 | P10+ required | Only when no active content |

Constants: `DUNGEON_DISCOVERY_CHANCE = 0.02`, `FISHING_DISCOVERY_CHANCE = 0.05`, `CHALLENGE_DISCOVERY_CHANCE = 0.000014`, `HAVEN_DISCOVERY_BASE_CHANCE = 0.000014`, `HAVEN_DISCOVERY_RANK_BONUS = 0.000007`

---

## Dungeon System

### Dungeon Sizes

| Size | Grid | Rooms | Boss XP | Prestige Requirement |
|------|------|-------|---------|---------------------|
| Small | 5x5 | 8-12 | 1,000-1,500 | Any |
| Medium | 7x7 | 15-20 | 2,000-3,000 | Level 25+ or P2+ |
| Large | 9x9 | 25-30 | 4,000-6,000 | Level 75+ or P4+ |
| Epic | 11x11 | 35-45 | 8,000-12,000 | P6+ |
| Legendary | 13x13 | 50-65 | 15,000-25,000 | P8+ |

Size is based on character level and prestige rank (with 20% random variation: 20% chance of one size smaller, 60% expected, 20% one size larger).

### Room Type Distribution

- Combat: 60%
- Treasure: 20% (guaranteed item drop, no combat)
- Elite: 15% (drops boss key)
- Boss: 5% (requires key to enter)

Exactly one Elite and one Boss room per dungeon.

---

## Challenge Minigames

All challenges require P1+ to discover. Discovery is random (~2hr average per challenge). 6 challenge types with 4 difficulty levels each.

### Discovery Weights

| Challenge | Weight | ~Probability |
|-----------|--------|--------------|
| Minesweeper (Trap Detection) | 30 | 27% |
| Rune (Rune Deciphering) | 25 | 23% |
| Gomoku (Five in a Row) | 20 | 18% |
| Morris (Nine Men's Morris) | 15 | 14% |
| Chess | 10 | 9% |
| Go (Territory Control) | 10 | 9% |

### Challenge Rewards

**Chess** (8x8, chess-engine crate):

| Difficulty | ELO | Reward |
|------------|-----|--------|
| Novice | ~500 | +1 Prestige Rank |
| Apprentice | ~800 | +2 Prestige Ranks |
| Journeyman | ~1100 | +3 Prestige Ranks |
| Master | ~1350 | +5 Prestige Ranks |

**Go** (9x9, MCTS AI):

| Difficulty | Simulations | Reward |
|------------|-------------|--------|
| Novice | 500 | +1 Prestige Rank |
| Apprentice | 2,000 | +2 Prestige Ranks |
| Journeyman | 8,000 | +3 Prestige Ranks |
| Master | 20,000 | +5 Prestige Ranks |

**Morris** (Nine Men's Morris, minimax AI):

| Difficulty | Reward |
|------------|--------|
| Novice | +50% level XP |
| Apprentice | +100% level XP |
| Journeyman | +150% level XP |
| Master | +200% level XP, +1 Fish Rank |

**Gomoku** (15x15, minimax depth 2-5):

| Difficulty | Reward |
|------------|--------|
| Novice | +75% level XP |
| Apprentice | +100% level XP |
| Journeyman | +1 Prestige Rank, +50% level XP |
| Master | +2 Prestige Ranks, +100% level XP |

**Minesweeper** (Trap Detection, variable grid):

| Difficulty | Grid | Reward |
|------------|------|--------|
| Novice | varies | +50% level XP |
| Apprentice | varies | +75% level XP |
| Journeyman | varies | +100% level XP |
| Master | varies | +1 Prestige Rank, +200% level XP |

**Rune Deciphering** (Mastermind-style):

| Difficulty | Reward |
|------------|--------|
| Novice | +25% level XP |
| Apprentice | +50% level XP |
| Journeyman | +1 Fish Rank, +75% level XP |
| Master | +1 Prestige Rank, +2 Fish Ranks |

---

## Testing & Validation

### Quick Smoke Test

```bash
cargo run -- --debug
```

Use debug menu (backtick) to:
1. Trigger fishing -- verify rank-up timing
2. Trigger challenges -- verify rewards apply
3. Trigger Haven -- verify discovery and building
4. Trigger dungeon -- verify room clearing

### Progression Simulation

To test XP/prestige pacing without playing:

```rust
fn simulate_progression(prestiges: u32) {
    let mut total_time = 0.0;
    for p in 0..=prestiges {
        let mult = 1.0 + 0.5 * (p as f64).powf(0.7);
        let req_level = get_required_level(p + 1);
        let xp_needed = total_xp_to_level(req_level);
        let time_hours = xp_needed / (mult * 3600.0 * XP_PER_SECOND);
        total_time += time_hours;
        println!("P{}: {}h (cumulative: {}h)", p, time_hours, total_time);
    }
}
```

### Balance Checkpoints

Before shipping balance changes, verify:

- [ ] P1 achievable in 30-60 min
- [ ] P10 achievable in 10-15 hours
- [ ] Stormbreaker requires meaningful fishing investment
- [ ] Haven bonuses feel impactful but not mandatory
- [ ] Minigame rewards are attractive but not required
- [ ] Offline progression provides meaningful but not dominant XP

---

## Common Tuning Scenarios

### "Prestige feels pointless"

**Symptom**: Players don't feel stronger after prestiging.

**Fixes**:
1. Increase prestige multiplier coefficient (0.5 -> 0.6)
2. Increase attribute cap scaling (5 -> 6 per rank)
3. Add more visible power indicators in UI

### "Game is too slow"

**Symptom**: Players quit before P5.

**Fixes**:
1. Lower XP curve exponent (1.5 -> 1.4)
2. Increase kill XP range (200-400 -> 250-500)
3. Lower early prestige level requirements

### "Game is too fast"

**Symptom**: Players hit endgame in days, not weeks.

**Fixes**:
1. Raise XP curve exponent (1.5 -> 1.6)
2. Raise prestige level requirements
3. Lower prestige multiplier coefficient

### "Items don't matter"

**Symptom**: Players ignore equipment.

**Fixes**:
1. Increase affix value ranges
2. Lower base stats, raise item contribution
3. Add more impactful affix types

### "Fishing takes forever"

**Symptom**: Storm Leviathan feels impossibly far.

**Fixes**:
1. Lower fish-per-rank requirements in upper tiers
2. Increase FishingDock bonuses
3. Add more fishing rank rewards from challenges

### "Haven is too expensive"

**Symptom**: Players hoard prestige ranks, never build.

**Fixes**:
1. Lower tier costs (especially T1)
2. Increase bonus values to make investment obvious
3. Add "preview" of bonuses before purchase

---

## Appendix: Current Constants

All constants are defined in `src/core/constants.rs`.

```rust
// Timing
TICK_INTERVAL_MS: u64 = 100;
ATTACK_INTERVAL_SECONDS: f64 = 1.5;
HP_REGEN_DURATION_SECONDS: f64 = 2.5;
AUTOSAVE_INTERVAL_SECONDS: u64 = 30;
UPDATE_CHECK_INTERVAL_SECONDS: u64 = 1800;  // 30 minutes
UPDATE_CHECK_JITTER_SECONDS: u64 = 300;     // +/- 5 minutes

// XP and Leveling
BASE_XP_PER_TICK: f64 = 1.0;
XP_CURVE_BASE: f64 = 100.0;
XP_CURVE_EXPONENT: f64 = 1.5;
COMBAT_XP_MIN_TICKS: u64 = 200;
COMBAT_XP_MAX_TICKS: u64 = 400;
OFFLINE_MULTIPLIER: f64 = 0.25;
MAX_OFFLINE_SECONDS: i64 = 604800;          // 7 days

// Character Attributes
BASE_ATTRIBUTE_VALUE: u32 = 10;
NUM_ATTRIBUTES: usize = 6;
BASE_ATTRIBUTE_CAP: u32 = 20;
ATTRIBUTE_CAP_PER_PRESTIGE: u32 = 5;
LEVEL_UP_ATTRIBUTE_POINTS: u32 = 3;

// Prestige Multiplier
PRESTIGE_MULT_BASE_FACTOR: f64 = 0.5;
PRESTIGE_MULT_EXPONENT: f64 = 0.7;

// Item Drops
ITEM_DROP_BASE_CHANCE: f64 = 0.15;
ITEM_DROP_PRESTIGE_BONUS: f64 = 0.01;
ITEM_DROP_MAX_CHANCE: f64 = 0.25;
MOB_RARITY_PRESTIGE_BONUS_PER_RANK: f64 = 0.01;
MOB_RARITY_PRESTIGE_BONUS_CAP: f64 = 0.10;
ZONE_ILVL_MULTIPLIER: u32 = 10;
ILVL_SCALING_BASE: f64 = 10.0;
ILVL_SCALING_DIVISOR: f64 = 30.0;

// Discovery Chances
DUNGEON_DISCOVERY_CHANCE: f64 = 0.02;
FISHING_DISCOVERY_CHANCE: f64 = 0.05;
CHALLENGE_DISCOVERY_CHANCE: f64 = 0.000014;
HAVEN_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
HAVEN_DISCOVERY_RANK_BONUS: f64 = 0.000007;
HAVEN_MIN_PRESTIGE_RANK: u32 = 10;

// Zone and Fishing
KILLS_FOR_BOSS: u32 = 10;
BASE_MAX_FISHING_RANK: u32 = 30;
MAX_FISHING_RANK: u32 = 40;
```

---

*Balance is never "done" -- it's an ongoing conversation between designer intent and player experience. When in doubt, playtest.*
