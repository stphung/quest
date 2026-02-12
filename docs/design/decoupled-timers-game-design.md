# Decoupled Attack Timers: Game Design

## Overview

Replace the single shared `attack_timer` with independent player and enemy attack timers. Currently, both combatants share one timer (1.5s) -- the player always attacks first, and the enemy counter-attacks in the same tick. This change lets each side attack on its own schedule, creating more dynamic combat pacing.

## Current System Summary

- **Single timer**: `CombatState.attack_timer` accumulates delta_time each tick (100ms).
- **Trigger**: When timer >= `ATTACK_INTERVAL_SECONDS / attack_speed_multiplier` (base 1.5s), both attack.
- **Order**: Player always attacks first. Enemy counter-attacks immediately after in the same tick.
- **Attack speed**: Player equipment's AttackSpeed affix reduces the shared interval. Enemy has no speed stat.
- **Regen phase**: After a kill, `is_regenerating` blocks all combat for 2.5s (base).

## Design Decisions

### 1. Player Timer

**Base interval**: 1.5s (unchanged from `ATTACK_INTERVAL_SECONDS`).

**Speed modifier**: `effective_interval = 1.5 / attack_speed_multiplier` (same formula as today). Player attack speed continues to come from equipment AttackSpeed affixes and is calculated in `DerivedStats`.

No change to the player attack formula. The player timer works exactly like the current shared timer but only triggers the player's attack.

### 2. Enemy Timer

**Base interval**: 2.0s.

Enemies attack slower than the player by default. This reflects the idle RPG philosophy: the player should feel powerful, and faster player attacks make equipment attack speed bonuses more rewarding by comparison.

**New constant**: `ENEMY_ATTACK_INTERVAL_SECONDS: f64 = 2.0`

**No enemy attack speed stat**. Enemies do not have individual speed values. All enemies within a tier share the same base interval, modified only by tier. This keeps enemy data simple (no new field on the `Enemy` struct for speed) and avoids per-enemy balance tuning.

### 3. Enemy Attack Speed Scaling by Tier

Enemy attack interval decreases (enemies attack faster) in harder content:

| Context | Interval | Rationale |
|---------|----------|-----------|
| Normal mobs (overworld) | 2.0s | Baseline. Comfortable pacing for idle watching. |
| Subzone bosses | 1.8s | Slightly faster to add pressure. |
| Zone bosses | 1.5s | Match the player's base speed -- bosses feel dangerous. |
| Dungeon Combat rooms | 2.0s | Same as overworld mobs. |
| Dungeon Elite rooms | 1.6s | Elites are aggressive and dangerous. |
| Dungeon Boss rooms | 1.4s | Dungeon bosses hit hard and fast. |

These values are constants, not per-enemy fields. The combat logic determines the enemy's interval from the combat context (dungeon room type, boss flag, zone boss flag).

**Helper function** (pseudocode):
```
fn enemy_attack_interval(state) -> f64:
    if in dungeon:
        match room_type:
            Boss  -> 1.4
            Elite -> 1.6
            _     -> 2.0
    else if zone_progression.fighting_boss:
        if is_zone_boss -> 1.5
        else            -> 1.8
    else:
        2.0
```

### 4. Simultaneous Attack Resolution

When both timers expire on the same tick (both >= their respective intervals):

1. **Player attacks first**. This preserves the existing behavior where the player has initiative. In an idle RPG, the player should feel proactive, not reactive.
2. **If the enemy dies from the player's attack**, the enemy does NOT get a final counter-attack. The kill is clean.
3. **If the enemy survives**, the enemy then attacks.

This is consistent with the current system's feel (player always strikes first) while being a natural extension to independent timers.

### 5. Regen Phase Interaction

**No change to regen behavior**. When `is_regenerating` is true:
- Both attack timers are paused (neither accumulates).
- When regen completes and a new enemy spawns, both timers reset to 0.0.

This maintains the current design where regen is a full combat pause.

### 6. Timer Reset on New Enemy

When a new enemy spawns (after regen, or entering a new dungeon room):
- **Player timer**: Reset to 0.0.
- **Enemy timer**: Reset to 0.0.

Both start fresh. No advantage carries over between encounters.

### 7. Timer Reset on Player Death

When the player dies:
- Both timers reset to 0.0 (same as current behavior with the single timer).
- In overworld: boss encounter resets as before.
- In dungeon: dungeon exits as before.

### 8. Weapon-Blocked Boss (Zone 10)

When the player's attack is blocked (no Stormbreaker against Zone 10 final boss):
- Player timer fires but deals no damage (current behavior).
- Enemy timer fires independently. The enemy attacks on its own schedule (1.5s for a zone boss).
- This makes the weapon-blocked fight feel more punishing -- the player takes damage on the enemy's faster schedule while dealing none. This reinforces the "go get Stormbreaker" signal.

## Balance Analysis

### DPS Impact

**Current system** (shared 1.5s timer):
- Player and enemy both attack every 1.5s (modified by player attack speed).
- With 50% attack speed bonus: both attack every 1.0s.
- Enemy DPS = `enemy_damage / shared_interval`

**New system** (independent timers):
- Player attacks every `1.5 / attack_speed_multiplier` seconds.
- Normal mob attacks every 2.0s.
- With 50% player attack speed: player attacks every 1.0s, mob still attacks every 2.0s.

**Net effect on normal mobs**: Enemies deal ~25% less DPS than before (2.0s interval vs 1.5s). Player DPS is unchanged. This makes overworld grinding safer and faster, which is desirable for an idle game.

**Net effect on bosses**: Zone bosses attack at 1.5s (same as before), so boss fights feel the same difficulty. Subzone bosses at 1.8s are slightly easier. Dungeon bosses at 1.4s are harder. This creates better difficulty differentiation.

### Attack Speed Affix Value

Currently, player AttackSpeed also speeds up when the enemy attacks (since they share a timer). After decoupling, AttackSpeed only benefits the player's attack rate, not enemy rate.

**This is a slight nerf to AttackSpeed's survivability value** (before, faster attacks also meant enemies hit you sooner, but you killed them sooner -- net positive). After decoupling, faster player attacks still kill enemies sooner (reducing total hits taken), but enemies attack on their own schedule regardless.

**No rebalancing needed**. The item scoring weight for AttackSpeed (1.2x) is already the second-lowest offensive weight. The affix remains valuable for DPS. The survivability interaction was not intuitive to players anyway -- making attack speed "purely offensive" is cleaner.

### Fight Duration Impact

Average fight duration against normal mobs with base stats:
- **Current**: Both attack every 1.5s. Enemy dies in N player attacks. Player takes N enemy hits.
- **New**: Player attacks every 1.5s, enemy attacks every 2.0s. Enemy still dies in N player attacks. Player takes fewer enemy hits (roughly 75% as many). Fights end slightly sooner in real time since enemy counter-attacks no longer pad each round.

Boss fights: Roughly unchanged for zone bosses (1.5s enemy interval). Dungeon bosses attack faster (1.4s) so fights are slightly more dangerous, which is intentional for the hardest content.

### Offline Progression

Offline XP simulation does not model individual combat ticks -- it estimates kills based on time. **No change needed** for offline progression. The offline system already uses an abstract model (`OFFLINE_MULTIPLIER = 0.25`) that doesn't depend on attack timer specifics.

## New Constants

```rust
// Enemy attack timing
pub const ENEMY_ATTACK_INTERVAL_SECONDS: f64 = 2.0;
pub const ENEMY_BOSS_ATTACK_INTERVAL_SECONDS: f64 = 1.8;
pub const ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS: f64 = 1.5;
pub const ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS: f64 = 1.6;
pub const ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS: f64 = 1.4;
```

## Summary of Changes

| Aspect | Before | After |
|--------|--------|-------|
| Timer count | 1 shared | 2 independent (player + enemy) |
| Player base interval | 1.5s | 1.5s (unchanged) |
| Enemy base interval | 1.5s (shared) | 2.0s (mobs), 1.4-1.8s (bosses/elites) |
| Attack speed affix | Speeds shared timer | Speeds player timer only |
| Attack order (same tick) | Player first, always | Player first, always |
| Enemy kills player's attack | N/A (simultaneous) | Enemy does NOT counter-attack after dying |
| Regen phase | Blocks shared timer | Blocks both timers |
| Enemy struct change | None | None (interval from context, not per-enemy) |
| Offline progression | No change | No change |
| Save compatibility | attack_timer field | Split into player_attack_timer + enemy_attack_timer (serde(default) for compat) |
