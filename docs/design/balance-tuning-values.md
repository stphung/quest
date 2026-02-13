# Balance Tuning: Proposed Values

## Design Goals

| Metric | Current | Target |
|--------|---------|--------|
| P0 Zone 1 mob fight duration | ~4.5s (3 hits) | 5-10s (5-6 hits) |
| P0 death rate (1hr) | 0 | 15-20 deaths/hr |
| Dungeon failure rate | ~0% | 10-20% |
| Prestige flat_damage at P5 | 5 | 15 |
| Prestige flat_damage at P20 | 12 | 40 |
| Boss fights (at-level) | trivial | 10-20s, occasional death |

## 1. Zone Enemy Stats (ZONE_ENEMY_STATS)

Multiplier: ~1.8x HP, ~1.4x DMG, ~1.5x DEF across all zones. Steps scaled proportionally.

### Current Values
```
Zone  1: (30,  5,  5,  1,  0, 0)   // Meadow
Zone  2: (50,  8,  9,  2,  1, 1)   // Dark Forest
Zone  3: (90, 12, 16,  3,  4, 1)   // Mountain Pass
Zone  4: (120, 15, 22, 4,  7, 2)   // Ancient Ruins
Zone  5: (170, 18, 30, 5, 11, 2)   // Volcanic Wastes
Zone  6: (210, 22, 38, 6, 15, 3)   // Frozen Tundra
Zone  7: (270, 25, 48, 7, 20, 3)   // Crystal Caverns
Zone  8: (320, 30, 56, 8, 24, 4)   // Sunken Kingdom
Zone  9: (380, 35, 66, 9, 30, 4)   // Floating Isles
Zone 10: (450, 40, 78, 10, 36, 5)  // Storm Citadel
Zone 11: (520, 45, 88, 12, 42, 5)  // The Expanse
```

### Proposed Values
```
Zone  1: (55,  9,  7,  2,  0, 0)   // Meadow
Zone  2: (90, 14, 13,  3,  2, 1)   // Dark Forest
Zone  3: (160, 22, 22, 4,  6, 2)   // Mountain Pass
Zone  4: (215, 27, 31, 6, 10, 3)   // Ancient Ruins
Zone  5: (305, 32, 42, 7, 16, 3)   // Volcanic Wastes
Zone  6: (380, 40, 53, 8, 22, 4)   // Frozen Tundra
Zone  7: (485, 45, 67, 10, 29, 4)  // Crystal Caverns
Zone  8: (575, 54, 78, 11, 35, 6)  // Sunken Kingdom
Zone  9: (685, 63, 92, 13, 43, 6)  // Floating Isles
Zone 10: (810, 72, 109, 14, 52, 7) // Storm Citadel
Zone 11: (935, 81, 123, 17, 60, 7) // The Expanse
```

### Zone 1 Math (P0 Level 1)
- Player: 10 total damage, 50 HP, 0 defense, 5% crit, 1.5s attack interval
- Old mob: 30 HP / 5 DMG / 0 DEF -> 3 hits = 4.5s, takes 10 damage (20% HP)
- New mob: 55 HP / 7 DMG / 0 DEF -> 6 hits = 9s, takes ~28 damage (56% HP)
- Result: fights last 2x longer, death is possible if unlucky or fighting boss

## 2. Boss Multipliers

### Current Values
```
SUBZONE_BOSS_MULTIPLIERS:  (2.5, 1.3, 1.5)
ZONE_BOSS_MULTIPLIERS:     (4.0, 1.6, 2.0)
DUNGEON_ELITE_MULTIPLIERS: (1.5, 1.2, 1.3)
DUNGEON_BOSS_MULTIPLIERS:  (2.5, 1.4, 1.5)
```

### Proposed Values
```
SUBZONE_BOSS_MULTIPLIERS:  (3.0, 1.5, 1.8)
ZONE_BOSS_MULTIPLIERS:     (5.0, 1.8, 2.5)
DUNGEON_ELITE_MULTIPLIERS: (2.2, 1.5, 1.6)
DUNGEON_BOSS_MULTIPLIERS:  (3.5, 1.8, 2.0)
```

### Rationale
- Subzone boss: 3.0x HP means ~18 hits = 27s at level, with 1.5x damage they deal real threat
- Zone boss: 5.0x HP is a genuinely long fight (35+ seconds), 1.8x damage creates death risk
- Dungeon elite: 2.2x HP (was 1.5x) makes elites feel like real guardians, not slightly tougher mobs
- Dungeon boss: 3.5x HP (was 2.5x) combined with 1.8x damage creates 10-20% failure rate

## 3. Prestige Combat Bonus Formulas

### Current Values
```
PRESTIGE_FLAT_DAMAGE_FACTOR:   2.0  (exponent 0.6)
PRESTIGE_FLAT_DEFENSE_FACTOR:  1.0  (exponent 0.55)
PRESTIGE_CRIT_PER_RANK:        0.5  (cap 10.0)
PRESTIGE_FLAT_HP_FACTOR:       5.0  (exponent 0.5)
```

### Proposed Values
```
PRESTIGE_FLAT_DAMAGE_FACTOR:   5.0  (exponent 0.7)
PRESTIGE_FLAT_DEFENSE_FACTOR:  3.0  (exponent 0.6)
PRESTIGE_CRIT_PER_RANK:        0.5  (cap 15.0)
PRESTIGE_FLAT_HP_FACTOR:      15.0  (exponent 0.6)
```

### Bonus Values by Prestige Rank

| Rank | Flat DMG (old/new) | Flat DEF (old/new) | Crit% (old/new) | Flat HP (old/new) |
|------|-------------------:|-------------------:|-----------------:|------------------:|
| P0   | 0 / 0              | 0 / 0              | 0.0 / 0.0       | 0 / 0             |
| P1   | 2 / 5              | 1 / 3              | 0.5 / 0.5       | 5 / 15            |
| P5   | 5 / 15             | 2 / 7              | 2.5 / 2.5       | 11 / 39           |
| P10  | 7 / 25             | 3 / 11             | 5.0 / 5.0       | 15 / 59           |
| P15  | 9 / 33             | 4 / 15             | 7.5 / 7.5       | 19 / 76           |
| P20  | 12 / 40            | 5 / 18             | 10.0 / 10.0     | 22 / 90           |
| P30  | 14 / 54            | 6 / 23             | 10.0 / 15.0     | 27 / 115          |

### Rationale
- Damage: 2.5x increase at P5, 3.3x at P20 -- makes prestige feel rewarding immediately
- Defense: 3.5x increase at P5, 3.6x at P20 -- higher prestige players take much less damage
- Crit cap raised from 10% to 15% -- high-prestige players (P30+) get meaningful extra crit
- HP: 3.5x increase at P5, 4.1x at P20 -- significant survivability boost

## 4. Simulator Validation

All tests run with proposed values temporarily applied to `src/core/constants.rs`.

### Baseline (Current Values)

| Prestige | Duration | Kills | Deaths | Final Level | Final Zone |
|----------|----------|-------|--------|-------------|------------|
| P0       | 1hr      | 471-633 | 0    | 36-42       | 2-3        |
| P10      | 2hr      | 1291-1403 | 0  | 104-111     | 6-4        |
| P20      | 2hr      | 568-1084 | 11-52 | 80-110    | 10-4       |

### Proposed Values

| Prestige | Duration | Kills | Deaths | Final Level | Final Zone | Notes |
|----------|----------|-------|--------|-------------|------------|-------|
| P0       | 1hr      | 383-475 | 17-19 | 32-35     | 2-3        | Deaths now non-zero. Fights ~5.7s avg. |
| P5       | 1hr      | 416-564 | 0-18  | 51-61     | 4-3        | Prestige bonuses provide clear advantage. |
| P10      | 2hr      | 1153-1310 | 0-2 | 97-105    | 6-4        | Nearly invincible in Zones 1-5, challenged in 6+. |
| P15      | 2hr      | 1142-1329 | 7-12 | 110-124   | 8-4        | Consistent moderate deaths in late zones. |
| P20      | 2hr      | 728-1147 | 2-78  | 90-109    | 10-4       | Zone 9-10 genuinely dangerous. High variance. |

### Key Observations

1. **Fight Duration**: P0 Zone 1 fights now average ~5.7s (target 5-10s) vs previous ~3.5s
2. **Death Rate**: P0 has 17-19 deaths/hr (~1 every 3 minutes), creating tension without frustration
3. **Prestige Power**: P5 has 0-18 deaths (variance from zone progression), P10 has 0-2 -- prestige clearly helps
4. **Late-Game Challenge**: P20 in Zones 9-10 shows 2-78 deaths -- end-game zones remain dangerous
5. **Dungeon Deaths**: P0 dungeon death observed at tick 620 (level 5, elite encounter) -- dungeons are risky
6. **Progression Pace**: Kill rates decreased ~15% at P0 (534->440 avg) due to longer fights, feels appropriate

### Verbose Fight Analysis (P0 Zone 1)

```
Fight 1 (t=15 to t=90):  75 ticks = 7.5s, 6 player hits, mob did 18 damage (36% HP)
Fight 2 (t=130 to t=205): 75 ticks = 7.5s, 6 hits, mob did 21 damage (42% HP)
Fight 3 (t=245 to t=305): 60 ticks = 6.0s, 5 hits, mob did 21 damage (42% HP)
```

All within the 5-10s target. Player takes 36-42% HP per fight, making consecutive fights without regen dangerous.

## 5. Constants to Change (Summary)

In `src/core/constants.rs`:

| Constant | Current | Proposed |
|----------|---------|----------|
| `ZONE_ENEMY_STATS[0]` | `(30, 5, 5, 1, 0, 0)` | `(55, 9, 7, 2, 0, 0)` |
| `ZONE_ENEMY_STATS[1]` | `(50, 8, 9, 2, 1, 1)` | `(90, 14, 13, 3, 2, 1)` |
| `ZONE_ENEMY_STATS[2]` | `(90, 12, 16, 3, 4, 1)` | `(160, 22, 22, 4, 6, 2)` |
| `ZONE_ENEMY_STATS[3]` | `(120, 15, 22, 4, 7, 2)` | `(215, 27, 31, 6, 10, 3)` |
| `ZONE_ENEMY_STATS[4]` | `(170, 18, 30, 5, 11, 2)` | `(305, 32, 42, 7, 16, 3)` |
| `ZONE_ENEMY_STATS[5]` | `(210, 22, 38, 6, 15, 3)` | `(380, 40, 53, 8, 22, 4)` |
| `ZONE_ENEMY_STATS[6]` | `(270, 25, 48, 7, 20, 3)` | `(485, 45, 67, 10, 29, 4)` |
| `ZONE_ENEMY_STATS[7]` | `(320, 30, 56, 8, 24, 4)` | `(575, 54, 78, 11, 35, 6)` |
| `ZONE_ENEMY_STATS[8]` | `(380, 35, 66, 9, 30, 4)` | `(685, 63, 92, 13, 43, 6)` |
| `ZONE_ENEMY_STATS[9]` | `(450, 40, 78, 10, 36, 5)` | `(810, 72, 109, 14, 52, 7)` |
| `ZONE_ENEMY_STATS[10]` | `(520, 45, 88, 12, 42, 5)` | `(935, 81, 123, 17, 60, 7)` |
| `SUBZONE_BOSS_MULTIPLIERS` | `(2.5, 1.3, 1.5)` | `(3.0, 1.5, 1.8)` |
| `ZONE_BOSS_MULTIPLIERS` | `(4.0, 1.6, 2.0)` | `(5.0, 1.8, 2.5)` |
| `DUNGEON_ELITE_MULTIPLIERS` | `(1.5, 1.2, 1.3)` | `(2.2, 1.5, 1.6)` |
| `DUNGEON_BOSS_MULTIPLIERS` | `(2.5, 1.4, 1.5)` | `(3.5, 1.8, 2.0)` |
| `PRESTIGE_FLAT_DAMAGE_FACTOR` | `2.0` | `5.0` |
| `PRESTIGE_FLAT_DAMAGE_EXPONENT` | `0.6` | `0.7` |
| `PRESTIGE_FLAT_DEFENSE_FACTOR` | `1.0` | `3.0` |
| `PRESTIGE_FLAT_DEFENSE_EXPONENT` | `0.55` | `0.6` |
| `PRESTIGE_CRIT_CAP` | `10.0` | `15.0` |
| `PRESTIGE_FLAT_HP_FACTOR` | `5.0` | `15.0` |
| `PRESTIGE_FLAT_HP_EXPONENT` | `0.5` | `0.6` |

No changes to: `PRESTIGE_CRIT_PER_RANK` (remains 0.5), timing constants, XP formulas, or item drop rates.

## 6. Test Impact Notes

Any test that hard-codes specific enemy stat values or prestige combat bonus values will need updating. Key areas to check:
- Tests in `src/combat/` that create enemies with specific HP/damage values
- Tests in `src/character/prestige.rs` that assert specific bonus values
- Tests in `src/core/` that depend on zone enemy stat tuples
- Integration tests in `tests/` that simulate combat outcomes
