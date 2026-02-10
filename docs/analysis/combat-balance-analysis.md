# Combat Balance Analysis

*Generated: 2026-02-09*
*Simulator: PR #115 (CombatEngine)*

## Executive Summary

The game's combat balance is fundamentally broken due to **dynamic enemy scaling**. Enemies scale with player stats, making it impossible to out-level content. Combined with harsh boss multipliers, this creates a situation where:

- Boss success rate is **~0.1%** regardless of player level or prestige
- After **14 hours of play**, 65% of P0 players are still stuck in Zone 1
- Leveling provides no power advantage — a level 500 character faces the same relative difficulty as level 10

## The Core Problem

### Dynamic Enemy Scaling

From `src/combat/types.rs`:
```rust
// Enemy HP: 80-120% of player HP, scaled by zone/subzone
let max_hp = player_max_hp * hp_variance * zone_multiplier
```

Enemies are generated relative to the player's current stats. As you level up and gain HP/damage, enemies gain proportionally. **You can never become stronger than the content.**

### Boss Multipliers

On top of dynamic scaling, bosses apply additional multipliers:

| Boss Type | HP Multiplier | Damage Multiplier |
|-----------|---------------|-------------------|
| Subzone Boss | 2.0x | 1.5x |
| Zone Boss | 3.0x | 2.0x |

A zone boss has **3x the HP and 2x the damage** of already-scaled enemies.

## Simulation Data

### Methodology

- Simulator uses `CombatEngine` (same code as real game)
- 20 runs per prestige level
- 500,000 ticks per run (~14 hours of game time)
- Tracked: zone reached, boss kills, boss deaths, fight duration

### Zone Progression by Prestige (% of runs ending in each zone)

```
         Z1    Z2    Z3    Z4    Z5    Z6
    ┌──────────────────────────────────────
P0  │  65%   35%    -     -     -     -
P1  │  60%   40%    -     -     -     -
P2  │  70%   30%    -     -     -     -
P3  │  60%   40%    -     -     -     -
P4  │  50%   50%    -     -     -     -
P5  │  15%   50%   15%   20%    -     -
P6  │  40%   30%   10%   20%    -     -
P7  │  10%   50%   40%    -     -     -
P8  │  30%   30%   30%   10%    -     -
P9  │  30%   40%   20%   10%    -     -
P10 │  10%   40%   35%   10%    5%    -
```

**Prestige Gates:**
- P0-P4: Max Zone 2
- P5-P9: Max Zone 4  
- P10-P14: Max Zone 6
- P15-P19: Max Zone 8
- P20+: Max Zone 10

### Boss Success Rate

| Prestige | Boss Kills | Boss Deaths | Success Rate |
|----------|------------|-------------|--------------|
| P0 | ~3 | ~5,531 | 0.05% |
| P5 | ~10 | ~7,182 | 0.14% |
| P10 | ~15 | ~9,820 | 0.15% |

**Boss success rate stays at ~0.1% regardless of prestige level.**

### Combat Timing

| Prestige | Avg Fight Duration | Design Target |
|----------|-------------------|---------------|
| P0 | 0.5s (5 ticks) | 5-10 seconds |
| P5 | 0.4s (4 ticks) | 5-10 seconds |
| P10 | 0.3s (3 ticks) | 5-10 seconds |

Fights are **10-20x faster than designed**, meaning very quick deaths.

### Death Statistics (P0, 500K ticks)

- Average Deaths: 53,393
- Boss Deaths: 5,531 (10%)
- Regular Deaths: 47,862 (90%)
- Deaths per Kill: ~0.96 (dying almost every fight)

## Why Prestige Doesn't Help

Prestige provides:
1. **XP Multiplier**: `1.0 + 0.5 * rank^0.7` (P10 = 3.5x XP)
2. **Zone Access**: Higher prestige unlocks more zones
3. **Attribute Caps**: `20 + (prestige * 5)` max per stat

What prestige does **NOT** provide:
- Combat advantage (enemies scale with you)
- Higher boss success rate
- Lower death rate

The XP multiplier helps you level faster, but leveling doesn't help because enemies scale. You just die faster at higher levels.

## The Player Experience

### At P0 (New Player)

1. Start in Zone 1, Subzone 1
2. Need to kill 10 mobs to spawn boss
3. Die ~9 times while getting those 10 kills
4. Face boss → 99.95% chance of death
5. Boss death resets kill counter to 0
6. Repeat for ~5,500 boss attempts before one success
7. After 14 hours: 65% chance still in Zone 1

### At P10 (Experienced Player)

1. Start in Zone 1 with 3.5x XP multiplier
2. Level up faster → higher stats
3. Face boss → still 99.85% chance of death (enemies scaled)
4. After 14 hours: 10% chance still in Zone 1, 5% reached Zone 5

## Design Intent vs Reality

### Intended Flow
1. Fight through Zone 1-2
2. Hit prestige gate at Zone 2
3. Prestige at level 10-25
4. Return stronger, push further
5. Repeat until P20, then clear all zones

### Actual Flow
1. Fight in Zone 1
2. Die repeatedly to first boss
3. Never reach prestige gate
4. Quit

## Recommended Fixes

### Option A: Cap Enemy Scaling

```rust
// Cap enemy scaling at some maximum
let capped_hp = player_max_hp.min(ENEMY_HP_CAP);
let max_hp = capped_hp * hp_variance * zone_multiplier;
```

This allows high-level players to overpower early zones.

### Option B: Reduce Boss Multipliers

| Boss Type | Current | Proposed |
|-----------|---------|----------|
| Subzone Boss | 2.0x HP, 1.5x Dmg | 1.3x HP, 1.2x Dmg |
| Zone Boss | 3.0x HP, 2.0x Dmg | 1.5x HP, 1.3x Dmg |

### Option C: Level-Based Damage Bonus

```rust
// Player damage scales with level beyond base stats
let level_bonus = 1.0 + (level as f64 * 0.005); // +0.5% per level
```

At level 100, player deals 50% more damage than enemy scaling expects.

### Option D: Increase Fight Duration

Current enemy damage is too high, causing 0.3-0.5s fights. Reducing enemy damage by 50-70% would achieve the 5-10 second design target and give players more chances to crit/win.

### Option E: Remove Boss Reset

Currently, dying to a boss resets your kill counter. Removing this would make progress more consistent, even if slow.

## Conclusion

The combat system is working as coded but not as intended. The dynamic scaling creates a treadmill where progress is nearly impossible. Prestige provides psychological progression (bigger numbers, more XP) without actual gameplay progression (clearing zones).

The fix requires breaking the scaling symmetry — either cap enemy scaling, reduce boss difficulty, or give players a level-based advantage that enemies don't receive.

---

*See also: GitHub Issue #122*
