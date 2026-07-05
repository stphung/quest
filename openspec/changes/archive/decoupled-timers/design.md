> Backported design record. Sources: docs/design/decoupled-timers-architecture.md, docs/design/decoupled-timers-game-design.md, docs/design/decoupled-timers-ux.md.

## decoupled-timers-architecture.md

# Decoupled Attack Timers: Technical Architecture

## Status: Design Document
## Author: System Architect
## Date: 2026-02-11

---

## 1. Overview

Decouple the single shared `attack_timer` in `CombatState` into independent player and enemy attack timers. Currently, combat uses a single timer that triggers both player and enemy attacks in the same tick. This design separates them so each combatant attacks on their own cadence.

## 2. Current Architecture

### CombatState (src/combat/types.rs:206-218)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub current_enemy: Option<Enemy>,
    pub player_current_hp: u64,
    pub player_max_hp: u64,
    pub attack_timer: f64,        // <-- single shared timer
    pub regen_timer: f64,
    pub is_regenerating: bool,
    #[serde(skip)]
    pub visual_effects: Vec<VisualEffect>,
    #[serde(skip)]
    pub combat_log: VecDeque<CombatLogEntry>,
}
```

### Current Combat Flow (src/combat/logic.rs:62-289)

```
update_combat(delta_time):
  if is_regenerating -> handle regen, return
  if no enemy -> return
  attack_timer += delta_time
  if attack_timer >= ATTACK_INTERVAL / attack_speed_multiplier:
    attack_timer = 0.0
    player attacks (with crit, double strike, haven bonuses)
    if enemy dies -> emit event, enter regen, return
    enemy attacks back (with defense, reflection)
    if player dies -> emit event, handle death
```

Key observation: the enemy's "attack" is not timer-driven; it is an immediate counter-attack within the same threshold check as the player attack. There is no concept of enemy attack speed.

### Timer Reset Points

- **New enemy spawned** (src/core/game_logic.rs:131,153): `attack_timer = 0.0`
- **Attack fires** (src/combat/logic.rs:114): `attack_timer = 0.0`
- **CombatState::new()** (src/combat/types.rs:232): `attack_timer: 0.0`

### Constants (src/core/constants.rs)

```rust
pub const ATTACK_INTERVAL_SECONDS: f64 = 1.5;
```

### UI Usage (src/ui/combat_scene.rs:131)

```rust
let next_attack = ATTACK_INTERVAL_SECONDS - game_state.combat_state.attack_timer;
```

The UI shows a countdown to the next attack based on the single timer.

---

## 3. Proposed Struct Changes

### CombatState

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub current_enemy: Option<Enemy>,
    pub player_current_hp: u64,
    pub player_max_hp: u64,

    /// Player's independent attack timer. Accumulates delta_time each tick.
    /// Player attacks when this reaches the effective player attack interval.
    pub player_attack_timer: f64,

    /// Enemy's independent attack timer. Accumulates delta_time each tick.
    /// Enemy attacks when this reaches the effective enemy attack interval.
    #[serde(default)]
    pub enemy_attack_timer: f64,

    pub regen_timer: f64,
    pub is_regenerating: bool,

    #[serde(skip)]
    pub visual_effects: Vec<VisualEffect>,
    #[serde(skip)]
    pub combat_log: VecDeque<CombatLogEntry>,
}
```

**Rationale for keeping `attack_timer` name as `player_attack_timer`:** Clarity. Both timers live in `CombatState` (not on `Enemy`) because the enemy is transient -- enemies are created/destroyed frequently. The timer state is a property of the combat encounter, not the enemy entity.

### Enemy Struct

The `Enemy` struct does **not** change. Enemy attack speed is derived from zone/boss data, not stored on each enemy instance. This avoids serialization changes to every save that contains an enemy.

### New Constants (src/core/constants.rs)

```rust
/// Base attack interval for enemies (before zone/boss modifiers)
pub const ENEMY_ATTACK_INTERVAL_SECONDS: f64 = 2.0;
```

The player constant remains as `ATTACK_INTERVAL_SECONDS = 1.5`.

### Enemy Attack Speed Derivation

Enemy attack speed is **not** stored anywhere -- it is calculated at combat time based on contextual data:

```rust
/// Calculates the effective enemy attack interval for the current encounter.
/// Called each tick in update_combat() (cheap computation).
fn effective_enemy_attack_interval(state: &GameState) -> f64 {
    let base = ENEMY_ATTACK_INTERVAL_SECONDS;

    // Bosses attack faster than regular enemies
    let boss_modifier = if state.zone_progression.fighting_boss {
        0.8  // 20% faster
    } else if state.active_dungeon.as_ref()
        .and_then(|d| d.current_room())
        .map_or(false, |r| matches!(r.room_type, RoomType::Elite | RoomType::Boss))
    {
        0.85  // 15% faster for dungeon elites/bosses
    } else {
        1.0
    };

    // Higher zones have slightly faster enemies
    let zone_modifier = 1.0 - (state.zone_progression.current_zone_id as f64 - 1.0) * 0.02;
    let zone_modifier = zone_modifier.max(0.7); // Cap at 30% reduction

    base * boss_modifier * zone_modifier
}
```

This keeps the calculation stateless and avoids needing to store speed data in `Enemy` or migrate save data for enemies.

---

## 4. Serde Migration Strategy

### Problem

Old saves have `"attack_timer": <value>` but no `"enemy_attack_timer"` field.

### Solution: Rename + Default

Use serde's `alias` and `default` attributes:

```rust
/// Player's independent attack timer
#[serde(alias = "attack_timer")]
pub player_attack_timer: f64,

/// Enemy's independent attack timer (new field, defaults to 0.0)
#[serde(default)]
pub enemy_attack_timer: f64,
```

**How this works:**

1. **Old save loaded:** JSON contains `"attack_timer": 0.7`. Serde matches `alias = "attack_timer"` and deserializes into `player_attack_timer = 0.7`. `enemy_attack_timer` is absent, so `#[serde(default)]` yields `0.0`.

2. **New save loaded:** JSON contains `"player_attack_timer": 0.7, "enemy_attack_timer": 0.3`. Both deserialize directly.

3. **New save written:** Uses the new field names `player_attack_timer` and `enemy_attack_timer`. The old `attack_timer` key is never written again.

**No data loss.** The player's timer progress is preserved. The enemy timer starts fresh at 0.0, which is correct -- the enemy simply gets a brief grace period on load.

### Test: Backward Compatibility

Add to `src/character/manager.rs` tests and `src/core/game_state.rs` tests:

```rust
#[test]
fn test_old_save_with_attack_timer_loads_as_player_attack_timer() {
    let json = serde_json::json!({
        // ... minimal save fields ...
        "combat_state": {
            "player_max_hp": 50,
            "player_current_hp": 50,
            "current_enemy": null,
            "is_regenerating": false,
            "regen_timer": 0.0,
            "attack_timer": 1.2,  // OLD field name
            "combat_log": []
        },
        // ... rest of minimal save ...
    });

    let loaded: GameState = serde_json::from_value(json).unwrap();
    assert!((loaded.combat_state.player_attack_timer - 1.2).abs() < f64::EPSILON);
    assert!((loaded.combat_state.enemy_attack_timer - 0.0).abs() < f64::EPSILON);
}
```

Also update `test_serialization_default_fields_from_old_json` in `game_state.rs` and the `test_minimal_v2_save_still_loads` tests in `manager.rs` to use the old `attack_timer` key, confirming migration works.

---

## 5. update_combat() Refactored Pseudocode

```
fn update_combat(state, delta_time, haven, achievements) -> Vec<CombatEvent>:
    events = []

    // --- Phase 0: Regen (unchanged) ---
    if is_regenerating:
        handle_regen(state, delta_time, haven)
        return events

    if no enemy:
        return events

    // --- Phase 1: Accumulate both timers ---
    state.combat_state.player_attack_timer += delta_time
    state.combat_state.enemy_attack_timer += delta_time

    let derived = DerivedStats::calculate(...)
    let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier
    let enemy_interval = effective_enemy_attack_interval(state)

    // --- Phase 2: Determine who attacks this tick ---
    let player_attacks = player_attack_timer >= player_interval
    let enemy_attacks = enemy_attack_timer >= enemy_interval

    // --- Phase 3: Player attack (if ready) ---
    if player_attacks:
        state.combat_state.player_attack_timer = 0.0

        // Existing player attack logic (weapon block check, crit, double strike, haven bonuses)
        // ... (unchanged) ...

        if enemy dies:
            state.combat_state.enemy_attack_timer = 0.0  // Reset for next enemy
            // emit kill event, enter regen
            return events

    // --- Phase 4: Enemy attack (if ready) ---
    if enemy_attacks:
        state.combat_state.enemy_attack_timer = 0.0

        // Enemy damage calculation (defense reduction, reflection)
        let enemy_damage = enemy.damage.saturating_sub(derived.defense)
        state.combat_state.player_current_hp -= enemy_damage
        events.push(EnemyAttack { damage: enemy_damage })

        // Damage reflection
        if derived.damage_reflection_percent > 0.0 && enemy_damage > 0:
            reflected = (enemy_damage * reflection_pct / 100) as u64
            enemy.take_damage(reflected)
            // Check if reflection killed the enemy
            if !enemy.is_alive():
                // emit kill event, enter regen
                return events

        // Check player death
        if !player_alive:
            // Existing death handling (dungeon exit, boss reset, etc.)
            state.combat_state.player_attack_timer = 0.0
            state.combat_state.enemy_attack_timer = 0.0

    return events
```

### Key Behavioral Changes

1. **Player and enemy can attack on different ticks.** Previously, both always acted in the same tick. Now, a tick may have only a player attack, only an enemy attack, both, or neither.

2. **Same-tick ordering: player first.** When both timers fire on the same tick, the player attacks first. This preserves the existing "player advantage" behavior and avoids the feel-bad scenario of dying just before your attack would have killed the enemy.

3. **Enemy attacks are no longer contingent on player attacking.** Enemies attack on their own timer even if the player hasn't reached their threshold yet.

4. **Regen still blocks ALL combat.** When `is_regenerating` is true, neither timer advances. This is intentional -- regen is a brief pause between encounters.

5. **Both timers reset on new enemy spawn.** When `spawn_enemy()` or `spawn_dungeon_enemy()` runs, both `player_attack_timer` and `enemy_attack_timer` reset to 0.0.

6. **Both timers reset on player death.** This prevents the enemy from getting a "free" attack immediately on the next encounter after death.

---

## 6. Timer Reset Matrix

| Event | player_attack_timer | enemy_attack_timer |
|---|---|---|
| New enemy spawned | 0.0 | 0.0 |
| Player attack fires | 0.0 | unchanged |
| Enemy attack fires | unchanged | 0.0 |
| Enemy dies | unchanged (enters regen) | 0.0 |
| Player dies | 0.0 | 0.0 |
| Regen completes | unchanged | unchanged |
| CombatState::new() | 0.0 | 0.0 |
| Load from save | preserved | preserved (0.0 for old saves) |

---

## 7. Files That Need Changes

### Core Changes (combat logic)

| File | Change | Description |
|---|---|---|
| `src/combat/types.rs` | Rename field, add field | `attack_timer` -> `player_attack_timer`, add `enemy_attack_timer` |
| `src/combat/logic.rs` | Refactor `update_combat()` | Split timer accumulation and threshold checks as described in Section 5 |
| `src/core/constants.rs` | Add constant | `ENEMY_ATTACK_INTERVAL_SECONDS: f64 = 2.0` |
| `src/core/game_logic.rs` | Update timer resets | Lines 131, 153: reset both timers on enemy spawn |

### UI Changes

| File | Change | Description |
|---|---|---|
| `src/ui/combat_scene.rs` | Update timer display | Line 131: use `player_attack_timer` instead of `attack_timer`. Add enemy attack countdown display. |

### Test Updates

| File | Change | Description |
|---|---|---|
| `src/combat/logic.rs` (tests) | Rename field references | ~40 occurrences of `attack_timer` -> `player_attack_timer` in test code |
| `src/character/manager.rs` (tests) | Keep old JSON keys | Existing save compat tests should still use `"attack_timer"` in JSON to test migration |
| `src/core/game_state.rs` (tests) | Update compat test | `test_serialization_default_fields_from_old_json` should test old key migration |

### No Changes Needed

| File | Reason |
|---|---|
| `src/combat/types.rs` (Enemy struct) | Enemy attack speed is calculated, not stored |
| `src/character/derived_stats.rs` | No enemy speed field in DerivedStats |
| `src/items/` | No item affixes affect enemy attack speed |
| `src/dungeon/` | Enemy generation unchanged |
| `src/fishing/` | Unrelated system |
| `src/haven/` | Haven bonuses only affect player stats |

---

## 8. Test Migration Guide

### The `force_combat_tick` Helper

The existing test helper at `src/combat/logic.rs:305-312`:

```rust
fn force_combat_tick(state, haven, achievements) -> Vec<CombatEvent> {
    state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
    update_combat(state, 0.1, haven, achievements)
}
```

**Rename to:**

```rust
fn force_player_attack(state, haven, achievements) -> Vec<CombatEvent> {
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = 0.0; // Prevent enemy from attacking
    update_combat(state, 0.1, haven, achievements)
}
```

This forces a player attack while suppressing the enemy attack, which isolates player-attack behavior for testing.

### New Helper: Force Enemy Attack

```rust
fn force_enemy_attack(state, haven, achievements) -> Vec<CombatEvent> {
    state.combat_state.player_attack_timer = 0.0; // Prevent player from attacking
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
    update_combat(state, 0.1, haven, achievements)
}
```

### New Helper: Force Both Attacks

```rust
fn force_both_attacks(state, haven, achievements) -> Vec<CombatEvent> {
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
    update_combat(state, 0.1, haven, achievements)
}
```

### Test-by-Test Migration

Most existing tests that set `attack_timer = ATTACK_INTERVAL_SECONDS` do so to trigger a player attack and expect both player + enemy events. These should switch to `force_both_attacks()` to preserve existing behavior:

| Test Pattern | Old Code | New Code | Rationale |
|---|---|---|---|
| "Force attack to kill enemy" | `attack_timer = ATTACK_INTERVAL` | `force_player_attack()` | Only need player attack; enemy counter-attack not needed when enemy dies |
| "Player dies from enemy attack" | `attack_timer = ATTACK_INTERVAL` | `force_both_attacks()` | Need both attacks to trigger player death from enemy hit |
| "Defense reduces damage" | `attack_timer = ATTACK_INTERVAL` | `force_both_attacks()` | Tests enemy damage reduction, needs enemy to attack |
| "Crit damage check" | `attack_timer = ATTACK_INTERVAL` | `force_player_attack()` | Only testing player crit output |
| "Multi-turn combat" | `attack_timer = ATTACK_INTERVAL` in loop | Both timers in loop | Need both to simulate real combat |

### Specific Tests That Need Careful Attention

1. **`test_update_combat_attack_interval`** (line 357): Currently checks that `events.len() >= 2` (player + enemy). With decoupled timers, this test needs to set both timers to get both events.

2. **`test_player_died_resets`** (line 382): Sets `player_current_hp = 1` and expects enemy to kill player. Must use `force_both_attacks()`.

3. **`test_attack_speed_reduces_interval`** (line 1348): Sets `attack_timer = 1.0` and expects attack with +50% speed (effective interval 1.0s). Change to `player_attack_timer = 1.0`.

4. **`test_attack_speed_normal_interval_without_affix`** (line 1390): Sets `attack_timer = 1.0` and expects NO attack. Change to `player_attack_timer = 1.0`.

5. **`test_regeneration_skips_combat`** (line 716): Sets `attack_timer = ATTACK_INTERVAL_SECONDS` to verify no combat during regen. Should set both timers to verify neither fires during regen.

### New Tests to Add

```rust
#[test]
fn test_enemy_attacks_independently_of_player() {
    // Enemy timer fires but player timer does not
    // Should see EnemyAttack but not PlayerAttack
}

#[test]
fn test_player_attacks_independently_of_enemy() {
    // Player timer fires but enemy timer does not
    // Should see PlayerAttack but not EnemyAttack
}

#[test]
fn test_both_timers_fire_player_goes_first() {
    // Both timers fire on same tick
    // Player attack event should appear before enemy attack event
    // If player kills enemy, no enemy attack should occur
}

#[test]
fn test_enemy_attack_interval_scales_with_zone() {
    // Zone 1 enemy interval > Zone 10 enemy interval
}

#[test]
fn test_boss_enemy_attacks_faster() {
    // Boss encounter has shorter enemy attack interval
}

#[test]
fn test_enemy_timer_resets_on_new_enemy_spawn() {
    // After spawning a new enemy, enemy_attack_timer = 0.0
}

#[test]
fn test_both_timers_reset_on_player_death() {
    // After player death, both timers are 0.0
}

#[test]
fn test_regen_blocks_both_timers() {
    // During regen, neither timer advances
}

#[test]
fn test_old_save_migration_attack_timer_to_player_attack_timer() {
    // Load JSON with "attack_timer" key, verify player_attack_timer has the value
    // and enemy_attack_timer defaults to 0.0
}
```

---

## 9. UI Impact

### Combat Scene (src/ui/combat_scene.rs)

**Current display (line 131):**
```rust
let next_attack = ATTACK_INTERVAL_SECONDS - game_state.combat_state.attack_timer;
// Shows: "In Combat | Next: 0.8s"
```

**New display:**
```rust
let derived = DerivedStats::calculate_derived_stats(...);
let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
let player_next = player_interval - game_state.combat_state.player_attack_timer;

let enemy_interval = effective_enemy_attack_interval(game_state);
let enemy_next = enemy_interval - game_state.combat_state.enemy_attack_timer;

// Shows: "In Combat | You: 0.8s | Foe: 1.2s"
```

The `effective_enemy_attack_interval()` function will need to be accessible from the UI module. It should be placed in `src/combat/logic.rs` and made `pub`.

### DPS Calculation (src/ui/combat_scene.rs:113)

**Current:**
```rust
let base_dps = derived.total_damage() as f64 / ATTACK_INTERVAL_SECONDS;
```

This remains unchanged -- it is the player's DPS, which still uses the player's attack interval.

---

## 10. Edge Cases

### 1. Enemy Dies from Reflection on Enemy's Own Attack

If the enemy attacks and damage reflection kills the enemy, the enemy should die normally (emit kill event, enter regen). The player's timer is NOT reset -- it continues accumulating toward the next attack that never fires (because regen starts).

### 2. Player Kills Enemy on Player Attack, Enemy Timer Was Also Ready

Player goes first. Enemy dies. Enemy attack is skipped (enemy is dead). This is the correct behavior -- player advantage.

### 3. Both Combatants Would Die on Same Tick

Player attacks first. If player kills enemy, combat ends (player survives). If player does NOT kill enemy, enemy attacks. If enemy kills player, death is handled normally. This preserves existing "player advantage" semantics.

### 4. Very Large Delta Time (Offline Progression)

Offline XP calculation (`calculate_offline_xp` in `game_logic.rs`) does NOT use `update_combat()`. It simulates kills as XP ticks. The attack timers are irrelevant for offline progression. On resume, timers will be wherever they were at save time, which is correct.

### 5. Dungeon Room Transitions

When entering a new dungeon room with an enemy, `spawn_dungeon_enemy()` resets both timers to 0.0. This gives the player a brief grace period in each room.

---

## 11. Balance Implications

| Metric | Before | After | Change |
|---|---|---|---|
| Player attack interval | 1.5s base | 1.5s base | No change |
| Enemy attack interval | 1.5s (same as player) | 2.0s base | Enemies attack 25% slower |
| Boss attack interval | 1.5s | 1.6s (2.0 * 0.8) | Bosses attack slightly slower than player |
| Zone 10 enemy interval | 1.5s | 1.54s (2.0 * 0.82 * 0.94) | High zone enemies attack faster than Zone 1 |
| Effective DPS taken | coupled to player speed | independent | Player takes less damage per second from regular enemies |

The base `ENEMY_ATTACK_INTERVAL_SECONDS = 2.0` is a tuning knob. Adjust this value during playtesting. The game designer's balance document should confirm the final value. The zone and boss modifiers are also tunable.

---

## 12. Implementation Checklist

1. [ ] Add `ENEMY_ATTACK_INTERVAL_SECONDS` to `src/core/constants.rs`
2. [ ] Rename `attack_timer` to `player_attack_timer` in `CombatState`, add `enemy_attack_timer` with `#[serde(default)]` and `#[serde(alias = "attack_timer")]`
3. [ ] Update `CombatState::new()` to initialize both timers
4. [ ] Add `pub fn effective_enemy_attack_interval(state: &GameState) -> f64` to `src/combat/logic.rs`
5. [ ] Refactor `update_combat()` to use two independent timer checks (Section 5)
6. [ ] Update `spawn_enemy()` and `spawn_dungeon_enemy()` in `src/core/game_logic.rs` to reset both timers
7. [ ] Update death handling to reset both timers
8. [ ] Update `src/ui/combat_scene.rs` to show both countdowns
9. [ ] Rename all test references from `attack_timer` to `player_attack_timer`
10. [ ] Migrate `force_combat_tick` to `force_player_attack` / `force_both_attacks` / `force_enemy_attack`
11. [ ] Add new tests (Section 8)
12. [ ] Add backward compatibility test for old save migration
13. [ ] Update existing backward compat tests in `manager.rs` to keep old `"attack_timer"` JSON keys
14. [ ] Run `make check` to verify all tests pass

## decoupled-timers-game-design.md

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

## decoupled-timers-ux.md

# UX Design: Decoupled Attack Timers

## Context

Currently, combat uses a single shared `attack_timer`. When it fires, the player attacks first, then the enemy attacks back in the same tick. This creates a rigid, turn-based feel where attacks are always paired.

The proposed change introduces independent attack timers for the player and the enemy, allowing them to attack at different rates. This document provides UX recommendations for how to communicate this change to the player through the terminal UI.

## Current Combat UI Layout

```
+-- Combat ----------------------------------------+
| Player HP: 45/50       [==============--------]  |  <- 1 line
|                                                   |
|                    /\_/\                          |
|                   ( o.o )                         |  <- sprite area
|                    > ^ <                          |
|                   Meadow Beetle                   |
|                                                   |
| Meadow Beetle: 30/40  [===========---------]     |  <- 1 line
| * In Combat | Next: 0.8s | DPS: 12                |  <- 1 line status
+---------------------------------------------------+
```

The bottom info panel shows a combat log (right half) with color-coded entries:
- Green: player attacks
- Yellow+bold: player crits
- Red: enemy attacks

## Recommendation 1: Dual Timer Display in the Status Bar

**Change**: Replace the single "Next: 0.8s" countdown with two compact countdowns showing both timers.

**Current status line (in combat)**:
```
* In Combat | Next: 0.8s | DPS: 12
```

**Proposed status line (in combat)**:
```
* In Combat | You: 0.4s  Foe: 1.1s | DPS: 12
```

**Rationale**: The status bar is already 1 line and already shows a single timer. Splitting it into two labeled countdowns is minimal visual change, easy to scan, and directly communicates the core mechanical difference. The player immediately sees that the two timers count down independently.

**Color coding**:
- "You: 0.4s" in green (matches player attack log color)
- "Foe: 1.1s" in red (matches enemy attack log color)
- The color reinforces which timer belongs to whom without needing extra labels

**When an attack is imminent** (under 0.3s remaining), the number could flash with the BOLD modifier to create a subtle anticipation cue.

## Recommendation 2: Do NOT Add Timer Progress Bars

**Recommendation**: Do not add gauge/progress bars for attack timers next to the HP bars.

**Rationale**: Adding progress bars would consume vertical space in an already constrained layout (the combat area has `Min(5)` for the sprite, plus 1 line each for player HP, enemy HP, and status). Progress bars would either shrink the sprite area or require expanding the combat panel. The status bar countdown is sufficient -- this is an idle game where precise timer tracking is informational, not interactive. Players do not need to time actions against the timer.

## Recommendation 3: Visual Effects Need No Timing Changes

**Current behavior**: `DamageNumber`, `AttackFlash`, and `HitImpact` effects are lifetime-based (created on the event, fade over their `max_lifetime`). They are independent of the attack timer.

**Recommendation**: Keep the visual effect system as-is. Since effects are already event-driven (spawned when a `CombatEvent` fires), they will naturally work with decoupled timers. When the player attacks, player effects fire. When the enemy attacks on its own timer, enemy effects fire. No overlapping issue arises because effects are rendered per-frame based on their remaining lifetime.

**One refinement**: Consider differentiating the `AttackFlash` effect by source. Currently it uses yellow swords (`"*".repeat(20)`). To help the player visually distinguish simultaneous attacks (which become possible with decoupled timers):
- Player attack flash: keep yellow swords
- Enemy attack flash: use red exclamation marks or a red-tinted variant

This is a minor enhancement. If both attacks happen to land on the same tick, the player will see both flash effects and both log entries, which is acceptable.

## Recommendation 4: Combat Log Entries -- Add Timestamp Context

**Current behavior**: Log entries are color-coded (green = player, red = enemy) with a message string. When attacks were paired, the log naturally read as alternating exchanges.

**With decoupled timers**: Attacks may arrive at different rates, so the log will no longer alternate predictably. For example, a fast-attacking player might show three green entries before a single red entry.

**Recommendation**: No structural change needed. The existing color coding (green vs. red) already differentiates the source clearly. The log scrolls newest-first, so the player can see the natural rhythm of attacks.

**Optional enhancement**: When consecutive entries are from the same source (e.g., three player attacks in a row), a subtle visual separator or grouping is NOT recommended -- it would add visual noise. The color alone is sufficient.

## Recommendation 5: HP Bars Remain Unchanged

**Recommendation**: Keep HP bars exactly as they are. HP bars show current state, not attack timing. They will naturally reflect the new attack cadence as HP changes occur at different rates.

## Recommendation 6: 3D Dungeon View -- No Changes Needed

**Current behavior**: The 3D view (`combat_3d.rs`) renders the enemy sprite centered in the area, or a waiting message when no enemy is present. It does not display timer information.

**Recommendation**: No changes needed. The 3D view is purely a sprite renderer. Timer information is handled by the status bar below it, which applies identically in both dungeon and overworld combat.

## Recommendation 7: DPS Display Adjustment

**Current behavior**: DPS is calculated as `total_damage / ATTACK_INTERVAL_SECONDS`, adjusted for crit. It displays as a single number.

**With decoupled timers**: The player's attack interval may differ from the constant `ATTACK_INTERVAL_SECONDS` (equipment with attack speed affixes already modify this via `attack_speed_multiplier`). The DPS calculation already accounts for this.

**Recommendation**: Keep the single DPS number. It already reflects the player's effective attack rate. If desired, an "Enemy DPS" could be shown, but this adds clutter and is not actionable information in an idle game. The player's HP bar decreasing rate already communicates incoming damage visually.

## Recommendation 8: Regeneration Phase -- No Changes

**Current behavior**: After killing an enemy, the player enters a regen phase (2.5s base) where HP gradually restores. Both timers are irrelevant during this phase.

**Recommendation**: No changes. The status bar already shows "Regenerating..." during this phase. Neither attack timer is relevant, so the dual timer display simply does not render during regen.

## Summary of Changes

| UI Element | Change Required | Details |
|---|---|---|
| Status bar timer | Yes | Split "Next: 0.8s" into "You: 0.4s  Foe: 1.1s" with color coding |
| Attack flash effects | Optional | Differentiate player vs enemy flash color |
| HP bars | No | Already reflect state changes naturally |
| Combat log | No | Color coding already distinguishes source |
| 3D dungeon view | No | Does not display timer info |
| DPS display | No | Already accounts for player attack speed |
| Regen phase | No | Timers not shown during regen |
| Timer progress bars | No (explicitly rejected) | Would consume too much vertical space |

## Implementation Notes for Developers

1. **Status bar**: Modify `draw_combat_status()` in `combat_scene.rs` to read two timer values (e.g., `player_attack_timer` and `enemy_attack_timer`) instead of the single `attack_timer`. Format them side-by-side with colored spans.

2. **Bold flash on imminent attack**: When a timer value is under 0.3s, add `Modifier::BOLD` to its span style to create a subtle urgency cue.

3. **Attack flash differentiation**: In `combat_effects.rs`, the `AttackFlash` variant could carry a `is_player: bool` field. The render method would choose yellow for player, red for enemy.

4. **CombatState changes**: The single `attack_timer: f64` field will be replaced by `player_attack_timer: f64` and `enemy_attack_timer: f64`. The status bar reads these directly.

5. **Backward compatibility**: Use `#[serde(default)]` on new timer fields (as noted in the existing `CombatState` doc comment) so old save files load correctly with timers initialized to 0.0.
