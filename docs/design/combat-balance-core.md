# Combat Balance Core: Enemy Stats, Boss Multipliers, and Fight Duration

## Status: PROPOSED
## Author: Game Designer #1
## Related: Issue #123 (Combat Balance Overhaul)

---

## Problem Summary

The current combat system has a fundamental design flaw: **enemy stats are derived from the player's stats**, making leveling meaningless for combat power. Specifically:

```
Current formulas (src/combat/types.rs):
  Enemy HP     = player_max_hp * variance(0.8-1.2) * zone_mult * boss_mult
  Enemy Damage = (player_max_hp / 7) * variance(0.8-1.2) * zone_mult * boss_mult
```

This causes:
1. **No power growth**: A level 1 player and a level 100 player face proportionally identical challenges
2. **Boss death spiral**: Zone/boss multipliers stack on top of already player-scaled values, creating unkillable bosses (~0.1% success rate)
3. **Uniform fight duration**: Every fight feels the same regardless of zone or level
4. **65% of P0 players stuck in Zone 1**: Leveling provides no combat advantage, only boss multipliers make bosses harder

---

## Design Philosophy

### Core Principle: Zone-Based Static Scaling

Enemies should have **fixed stats determined by their zone and subzone**, independent of the player. This creates a natural difficulty curve where:

- **Under-leveled**: Content is genuinely hard (longer fights, risk of death)
- **At-level**: Content is appropriately challenging (5-10s fights, bosses require some luck)
- **Over-leveled**: Content becomes easy (quick kills, trivial bosses)

This is the standard idle RPG approach and is critical for the prestige loop to feel rewarding.

### Design Targets

| Encounter Type | Fight Duration (at-level) | Player Win Rate (at-level) |
|---|---|---|
| Normal mob | 5-8 seconds (3-5 exchanges) | ~95% |
| Subzone boss | 10-15 seconds (6-10 exchanges) | ~60-70% |
| Zone boss | 15-25 seconds (10-16 exchanges) | ~30-40% |
| Dungeon elite | 8-12 seconds | ~80% |
| Dungeon boss | 12-20 seconds | ~50-60% |

"At-level" means the player's level is at the zone's `min_level` with average attribute distribution and no equipment bonuses.

---

## Player Power Reference

Before defining enemy stats, we need to model what players look like at each zone boundary.

### Attribute Growth Model (P0, no equipment)

At P0, attribute cap = 20. Level-ups give +3 random points across 6 attributes.

| Level | Avg Attribute | CON Modifier | STR Modifier | Max HP | Phys Dmg | Total Dmg | Defense |
|---|---|---|---|---|---|---|---|
| 1 | 10 | 0 | 0 | 50 | 5 | 10 | 0 |
| 5 | 12 | +1 | +1 | 60 | 7 | 14 | 1 |
| 10 | 14 | +2 | +2 | 70 | 9 | 18 | 2 |
| 15 | 16 | +3 | +3 | 80 | 11 | 22 | 3 |
| 20 | 18 | +4 | +4 | 90 | 13 | 26 | 4 |
| 25 | 20 (cap) | +5 | +5 | 100 | 15 | 30 | 5 |

Note: At P0 cap (attr 20), all characters converge to the same stats regardless of level beyond 25. This is intentional -- it creates a "soft ceiling" that encourages prestige.

### Prestige Power Scaling

Each prestige rank adds +5 to attribute cap. By P5 (cap=45), a maxed character has:
- CON 45, mod +17: HP = 50 + 170 = 220
- STR 45, mod +17: Phys = 5 + 34 = 39
- Total damage: ~78 (phys + magic)
- DEX 45, mod +17: Defense = 17

This means prestige players are dramatically more powerful -- by design. They should trivialize earlier zones and only struggle with prestige-gated content.

### Reference Player Stats by Zone Entry Level

These are the expected player stats when first entering each zone (P0 for zones 1-2, appropriate prestige for later zones). Equipment bonuses add roughly +20-50% to base stats.

| Zone | Entry Level | Prestige | HP (base) | Total Dmg (base) | Defense (base) |
|---|---|---|---|---|---|
| 1 Meadow | 1 | P0 | 50 | 10 | 0 |
| 2 Dark Forest | 10 | P0 | 70 | 18 | 2 |
| 3 Mountain Pass | 25 | P5 | 130 | 38 | 8 |
| 4 Ancient Ruins | 40 | P5 | 160 | 48 | 12 |
| 5 Volcanic Wastes | 55 | P10 | 220 | 64 | 18 |
| 6 Frozen Tundra | 70 | P10 | 260 | 76 | 22 |
| 7 Crystal Caverns | 85 | P15 | 320 | 92 | 28 |
| 8 Sunken Kingdom | 100 | P15 | 370 | 106 | 32 |
| 9 Floating Isles | 115 | P20 | 440 | 124 | 38 |
| 10 Storm Citadel | 130 | P20 | 500 | 140 | 44 |
| 11 The Expanse | 150 | P20+ | 560+ | 156+ | 50+ |

---

## New Enemy Stat Formulas

### Core Formula: Zone-Based Static Stats

Replace the current `generate_enemy_with_multiplier(player_max_hp, ...)` with a formula based purely on zone and subzone:

```
Enemy Base HP     = ZONE_BASE_HP[zone_id] + (subzone_depth - 1) * ZONE_HP_STEP[zone_id]
Enemy Base Damage = ZONE_BASE_DMG[zone_id] + (subzone_depth - 1) * ZONE_DMG_STEP[zone_id]
Enemy Base Defense = ZONE_BASE_DEF[zone_id] + (subzone_depth - 1) * ZONE_DEF_STEP[zone_id]

Actual HP     = Enemy Base HP * variance(0.9, 1.1)
Actual Damage = Enemy Base Damage * variance(0.9, 1.1)
Actual Defense = Enemy Base Defense (no variance, keep it predictable)
```

Variance is reduced from 0.8-1.2 to 0.9-1.1 to make combat more predictable and balance-testable.

### Zone Enemy Stat Table

These values are tuned so that an at-level player with no equipment kills a normal mob in 4-6 player attacks (6-9 seconds at 1.5s interval) while surviving 4-7 enemy attacks (8-14 seconds at 2.0s interval).

**Design constraint**: Player should always be able to survive long enough to kill the enemy at-level. Enemy effective DPS < Player effective DPS for normal mobs.

| Zone | Base HP | HP Step | Base Dmg | Dmg Step | Base Def | Def Step |
|---|---|---|---|---|---|---|
| 1 Meadow | 30 | 5 | 5 | 1 | 0 | 0 |
| 2 Dark Forest | 50 | 8 | 9 | 2 | 1 | 1 |
| 3 Mountain Pass | 90 | 12 | 16 | 3 | 4 | 1 |
| 4 Ancient Ruins | 120 | 15 | 22 | 4 | 7 | 2 |
| 5 Volcanic Wastes | 170 | 18 | 30 | 5 | 11 | 2 |
| 6 Frozen Tundra | 210 | 22 | 38 | 6 | 15 | 3 |
| 7 Crystal Caverns | 270 | 25 | 48 | 7 | 20 | 3 |
| 8 Sunken Kingdom | 320 | 30 | 56 | 8 | 24 | 4 |
| 9 Floating Isles | 380 | 35 | 66 | 9 | 30 | 4 |
| 10 Storm Citadel | 450 | 40 | 78 | 10 | 36 | 5 |
| 11 The Expanse | 520 | 45 | 88 | 12 | 42 | 5 |

### Worked Example: Zone 1, Subzone 1 (Sunny Fields)

At-level player (Level 1, P0, no equipment):
- Player: HP=50, Dmg=10, Def=0, Atk interval=1.5s
- Enemy: HP=30(+/-10%), Dmg=5(+/-10%), Def=0, Atk interval=2.0s

Player DPS: 10 / 1.5 = 6.67/s. Time to kill enemy: 30 / 6.67 = **4.5 seconds** (3 attacks)
Enemy DPS: 5 / 2.0 = 2.5/s. Damage taken in 4.5s: ~11 HP. Player survives easily with 39 HP remaining.

### Worked Example: Zone 1, Subzone 3 (Mushroom Caves) - Normal Mob

At-level player (Level ~7, P0):
- Player: HP=60, Dmg=14, Def=1, Atk interval=1.5s
- Enemy: HP=40(+/-10%), Dmg=7(+/-10%), Def=0

Player effective DPS: 14 / 1.5 = 9.3/s. Time to kill: 40 / 9.3 = **4.3 seconds** (3 attacks)
Enemy effective DPS: (7-1) / 2.0 = 3.0/s. Damage taken: ~13 HP. Player at 47 HP. Comfortable.

### Worked Example: Zone 2, Subzone 1 (Forest Edge) - Normal Mob

At-level player (Level 10, P0):
- Player: HP=70, Dmg=18, Def=2
- Enemy: HP=50, Dmg=9, Def=1

Player effective DPS: (18-1) / 1.5 = 11.3/s. Time to kill: 50 / 11.3 = **4.4s** (3 attacks)
Enemy effective DPS: (9-2) / 2.0 = 3.5/s. Damage taken: ~15 HP. Player at 55 HP. Good.

### Worked Example: Over-leveled (Level 20 in Zone 1)

Player (Level 20, P0):
- Player: HP=90, Dmg=26, Def=4
- Zone 1 Subzone 1 Enemy: HP=30, Dmg=5, Def=0

Player effective DPS: 26 / 1.5 = 17.3/s. Time to kill: 30 / 17.3 = **1.7s** (2 attacks)
Enemy effective DPS: (5-4) / 2.0 = 0.5/s. Damage taken: ~1 HP. Trivial, as intended.

---

## Boss Multipliers

### Subzone Bosses

Subzone bosses guard the transition between subzones. They should be a meaningful check that requires the player to be close to the zone's level range, but not a hard wall.

```
Subzone Boss HP     = Normal Enemy HP * 2.5
Subzone Boss Damage = Normal Enemy Damage * 1.3
Subzone Boss Defense = Normal Enemy Defense * 1.5
Subzone Boss Attack Interval = 1.8s (unchanged from current)
```

### Zone Bosses

Zone bosses guard the transition to the next zone. They should be a hard check that requires the player to be at or slightly above the zone's max level.

```
Zone Boss HP     = Normal Enemy HP * 4.0
Zone Boss Damage = Normal Enemy Damage * 1.6
Zone Boss Defense = Normal Enemy Defense * 2.0
Zone Boss Attack Interval = 1.5s (unchanged from current)
```

### Worked Example: Zone 1 Subzone Boss (Field Guardian)

At-level player (Level ~5, P0):
- Player: HP=60, Dmg=14, Def=1, Atk=1.5s
- Subzone 1 normal mob: HP=30, Dmg=5, Def=0
- Field Guardian: HP=75, Dmg=6, Def=0, Atk=1.8s

Player DPS: 14 / 1.5 = 9.3/s. Time to kill boss: 75 / 9.3 = **8.1 seconds** (6 attacks)
Boss DPS: (6-1) / 1.8 = 2.8/s. Damage taken in 8.1s: ~23 HP. Player at 37 HP. **Win rate: ~75-85%** (variance may kill lower rolls).

### Worked Example: Zone 1 Zone Boss (Sporeling Queen)

At-level player (Level ~10, P0):
- Player: HP=70, Dmg=18, Def=2, Atk=1.5s
- Subzone 3 normal mob: HP=40, Dmg=7, Def=0
- Sporeling Queen: HP=160, Dmg=11, Def=0, Atk=1.5s

Player DPS: 18 / 1.5 = 12.0/s. Time to kill: 160 / 12.0 = **13.3 seconds** (9 attacks)
Boss DPS: (11-2) / 1.5 = 6.0/s. Damage taken in 13.3s: ~80 HP. Player HP=70. **Player dies by default.**

But consider: at level 10, the player should have some equipment from Zone 1 drops. With +20% stats from gear:
- Player: HP=84, Dmg=22, Def=3
- Player DPS: 22 / 1.5 = 14.7/s. Time to kill: 160 / 14.7 = 10.9s (8 attacks)
- Boss effective DPS: (11-3) / 1.5 = 5.3/s. Damage taken: 58 HP. Player at 26 HP. **Win rate: ~35-45%.** Crit luck matters. This is correct for a zone boss.

The player can also over-level (level 12-15) to improve odds significantly. This is the intended path for most players.

### Worked Example: Zone 5 Subzone Boss (Ash Walker Chief)

At-level player (Level 55, P10):
- Player: HP=220, Dmg=64, Def=18
- Subzone 1 normal: HP=170, Dmg=30, Def=11
- Boss: HP=425, Dmg=39, Def=16, Atk=1.8s

Player DPS: (64-16) / 1.5 = 32/s. Time to kill: 425 / 32 = **13.3s** (9 attacks)
Boss effective DPS: (39-18) / 1.8 = 11.7/s. Damage taken: 156 HP. Player at 64 HP. **Win rate: ~60-65%.** Good.

### Boss Multiplier Summary

| Boss Type | HP Mult | Dmg Mult | Def Mult | Atk Interval |
|---|---|---|---|---|
| Subzone Boss | 2.5x | 1.3x | 1.5x | 1.8s |
| Zone Boss | 4.0x | 1.6x | 2.0x | 1.5s |
| Dungeon Elite | 1.5x | 1.2x | 1.3x | 1.6s |
| Dungeon Boss | 2.5x | 1.4x | 1.5x | 1.4s |

Note: Dungeon enemies use the same zone-based static stats as their base, with dungeon-specific multipliers on top.

---

## Defense in the Damage Formula

### Current Formula

```rust
let enemy_damage = enemy.damage.saturating_sub(derived.defense);
```

This is a flat subtraction model. This is fine for the game's design but needs a minimum damage floor to prevent defense from completely negating damage:

```
Effective Damage = max(1, attacker_damage - defender_defense)
```

The current code already has `saturating_sub` which goes to 0, but the minimum should be 1 to prevent complete damage immunity. This needs a code change.

For player attacks vs enemy defense:
```
Player Effective Damage = max(1, player_total_damage - enemy_defense)
```

This matters in later zones where enemies have meaningful defense values.

---

## Fight Duration Analysis by Zone

Using the stat tables above, here are expected fight durations for an at-level player with no equipment:

| Zone | Normal Mob (s) | Subzone Boss (s) | Zone Boss (s) |
|---|---|---|---|
| 1 Meadow | 4-6 | 8-11 | 13-18 |
| 2 Dark Forest | 4-6 | 8-12 | 14-19 |
| 3 Mountain Pass | 5-7 | 9-13 | 15-20 |
| 4 Ancient Ruins | 5-7 | 10-14 | 16-22 |
| 5 Volcanic Wastes | 5-8 | 10-14 | 16-22 |
| 6 Frozen Tundra | 5-8 | 10-15 | 17-23 |
| 7 Crystal Caverns | 5-8 | 11-15 | 18-24 |
| 8 Sunken Kingdom | 6-8 | 11-15 | 18-24 |
| 9 Floating Isles | 6-9 | 12-16 | 19-25 |
| 10 Storm Citadel | 6-9 | 12-16 | 19-25 |

When over-leveled by 5+ levels, normal mob duration drops to 2-3 seconds. When over-leveled by 10+, it drops to 1-2 seconds. This is the core reward loop: grinding levels makes content faster.

---

## Death Penalty

### Current Behavior
- Death to boss: resets `fighting_boss=false` and `kills_in_subzone=0`
- Must kill 10 more mobs to trigger boss again

### Recommendation: Keep with Modification

The kill counter reset is good -- it creates a meaningful cost for death without being punishing. However, 10 kills when each fight takes 5-8s (plus 2.5s regen) means 75-105 seconds to get back to the boss. This is appropriate.

**Change**: After dying to a boss, reduce the required kills to trigger the boss again from 10 to 5 for that specific attempt. This prevents the frustration of a long grind-back after a close loss while still maintaining some cost. Store this as a `boss_retry_kills` field on `ZoneProgression` that resets to the full 10 after defeating the boss or changing zones.

This is a softer penalty that keeps players engaged rather than frustrated, especially important for zone bosses where the win rate is 30-40%.

---

## Over-leveling and Power Curve

### How Leveling Creates Power

Each level gives +3 random attribute points. For combat:
- STR/INT increase damage (+2 per modifier point, i.e., per 2 attribute points)
- CON increases HP (+10 per modifier point)
- DEX increases defense (+1 per modifier) and crit chance (+1% per modifier)

Average per level: +0.5 to each attribute = +0.25 modifier = +0.5 damage, +2.5 HP, +0.25 defense

This means being 5 levels above content gives roughly:
- +2.5 damage, +12.5 HP, +1.25 defense

Against Zone 1 enemies (HP=30, Dmg=5, Def=0), these are significant improvements.

### Equipment Amplification

Equipment drops from killed enemies. Higher-zone equipment has higher ilvl and better stats. A player farming Zone 1 will accumulate ilvl 10 gear that provides an additional ~15-30% boost to base stats. This compounds with level advantages.

### Prestige Power Spike

Prestiging raises the attribute cap by 5, allowing each attribute to grow further. A P1 character maxing out attributes reaches much higher stats than a P0 character. This creates a dramatic power spike for prestige-gated zones.

---

## Implementation Checklist for Task #4

1. **Add zone stat lookup tables** to `src/core/constants.rs` (or a new `src/combat/balance.rs`)
2. **Replace `generate_enemy_with_multiplier`** to use zone-based static stats instead of `player_max_hp`
3. **Update `generate_zone_enemy`** to use new static tables
4. **Update `generate_subzone_boss`** to use new boss multipliers
5. **Add enemy defense field** -- the `Enemy` struct currently has no `defense` field; add it and use in damage calculations
6. **Add minimum damage floor** of 1 to both player and enemy damage calculations
7. **Update dungeon enemy generation** to use zone-based stats (dungeon in zone N uses zone N stats)
8. **Add boss retry mechanic** (5 kills instead of 10 after boss death) -- optional, lower priority
9. **Update all tests** that depend on current enemy stat formulas

### Constants to Add

```rust
// Zone enemy base stats: (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 11] = [
    (30, 5, 5, 1, 0, 0),     // Zone 1: Meadow
    (50, 8, 9, 2, 1, 1),     // Zone 2: Dark Forest
    (90, 12, 16, 3, 4, 1),   // Zone 3: Mountain Pass
    (120, 15, 22, 4, 7, 2),  // Zone 4: Ancient Ruins
    (170, 18, 30, 5, 11, 2), // Zone 5: Volcanic Wastes
    (210, 22, 38, 6, 15, 3), // Zone 6: Frozen Tundra
    (270, 25, 48, 7, 20, 3), // Zone 7: Crystal Caverns
    (320, 30, 56, 8, 24, 4), // Zone 8: Sunken Kingdom
    (380, 35, 66, 9, 30, 4), // Zone 9: Floating Isles
    (450, 40, 78, 10, 36, 5), // Zone 10: Storm Citadel
    (520, 45, 88, 12, 42, 5), // Zone 11: The Expanse
];

// Boss multipliers: (hp_mult, dmg_mult, def_mult)
pub const SUBZONE_BOSS_MULTIPLIERS: (f64, f64, f64) = (2.5, 1.3, 1.5);
pub const ZONE_BOSS_MULTIPLIERS: (f64, f64, f64) = (4.0, 1.6, 2.0);
pub const DUNGEON_ELITE_MULTIPLIERS: (f64, f64, f64) = (1.5, 1.2, 1.3);
pub const DUNGEON_BOSS_MULTIPLIERS: (f64, f64, f64) = (2.5, 1.4, 1.5);
```

---

## Compatibility Notes

### Save File Compatibility

The `Enemy` struct is serialized in `CombatState`. Adding a `defense` field requires `#[serde(default)]` to maintain backward compatibility with old saves. An enemy loaded from an old save will have defense=0, which is safe.

### Dungeon Enemies

Dungeon enemies currently use generic `generate_enemy(player_max_hp)`, `generate_elite_enemy(player_max_hp)`, and `generate_boss_enemy(player_max_hp)`. These should be updated to use zone-based stats from the zone the dungeon was discovered in. The dungeon should store the `zone_id` it was generated in (it may already have this info via dungeon level).

### Zone 11 (The Expanse)

The Expanse cycles infinitely. Its stats are intentionally higher than Zone 10 to serve as an endless challenge. Since it cycles, players will naturally over-level its content through repeated clears and prestige.

---

## Validation Criteria

The implementation should be validated against these scenarios:

1. **Fresh P0 player in Zone 1**: Can kill normal mobs in 4-6s. Reaches boss after ~2 minutes of farming. Has ~35-45% boss win rate by level 10 with gear.
2. **P0 player grinding Zone 1 at level 15**: Normal mobs die in 2-3s. Zone boss beatable with ~70%+ win rate.
3. **P5 player entering Zone 3**: Normal mobs take 5-7s. Subzone bosses are challenging but beatable at 60-70%.
4. **Over-leveled player in earlier zone**: Previous zone content is trivially easy (1-2s kills).
5. **Boss death recovery**: Takes 40-60 seconds (5 kills at 5-8s + regen) to retry boss.
