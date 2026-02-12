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
    pub player_current_hp: u32,
    pub player_max_hp: u32,
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
    pub player_current_hp: u32,
    pub player_max_hp: u32,

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
            reflected = (enemy_damage * reflection_pct / 100) as u32
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
