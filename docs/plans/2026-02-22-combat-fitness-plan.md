# Combat Fitness System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prevent death loops where players die repeatedly to regular mobs or get stuck in unwinnable fights, by detecting these situations and automatically retreating the player to a safe zone.

**Architecture:** Two detection triggers (consecutive deaths >= 3, mob fight timeout >= 30s) that invoke a shared retreat function. The retreat function finds the last zone where the player defeated a boss and calls `travel_to()`. New `CombatEvent::CombatRetreat` and `TickEvent::CombatRetreat` variants carry the retreat message through the event pipeline.

**Tech Stack:** Rust, existing combat/zone/tick event pipeline.

---

### Task 1: Add constants

**Files:**
- Modify: `src/core/constants.rs:93-94` (after `KILLS_FOR_BOSS_RETRY`)

**Step 1: Add the two new constants**

Add after `KILLS_FOR_BOSS_RETRY` (line 94):

```rust
// Combat fitness: death loop and stalemate prevention
pub const DEATH_LOOP_THRESHOLD: u32 = 3;
pub const MOB_FIGHT_TIMEOUT_SECONDS: f64 = 30.0;
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles without errors

**Step 3: Commit**

```bash
git add src/core/constants.rs
git commit -m "feat(combat): add death loop and mob fight timeout constants (#320)"
```

---

### Task 2: Add `consecutive_deaths` field to `GameState`

**Files:**
- Modify: `src/core/game_state.rs:92-94` (near `session_kills`)

**Step 1: Write the failing test**

Add to `src/core/game_state.rs` in the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn test_consecutive_deaths_transient() {
    let mut gs = GameState::new("Hero".to_string(), 0);
    assert_eq!(gs.consecutive_deaths, 0);

    gs.consecutive_deaths = 5;
    let json = serde_json::to_string(&gs).unwrap();
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.consecutive_deaths, 0); // transient, not saved
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_consecutive_deaths_transient 2>&1 | tail -5`
Expected: FAIL — field `consecutive_deaths` not found

**Step 3: Add the field**

Add `consecutive_deaths` as a transient field to `GameState` struct, near `session_kills` (line ~93):

```rust
    /// Consecutive deaths to regular mobs without a kill (transient, for death loop detection)
    #[serde(skip)]
    pub consecutive_deaths: u32,
```

Initialize it in `GameState::new()` (in the `Self { ... }` block, near `session_kills: 0`):

```rust
    consecutive_deaths: 0,
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib test_consecutive_deaths_transient 2>&1 | tail -5`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/game_state.rs
git commit -m "feat(combat): add consecutive_deaths transient field to GameState (#320)"
```

---

### Task 3: Add `current_fight_elapsed` field to `CombatState`

**Files:**
- Modify: `src/combat/types.rs:77-106` (`CombatState` struct)

**Step 1: Write the failing test**

Add to `src/combat/types.rs` in the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn test_current_fight_elapsed_defaults_to_zero() {
    let cs = CombatState::new(50);
    assert!((cs.current_fight_elapsed - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_current_fight_elapsed_is_transient() {
    let mut cs = CombatState::new(50);
    cs.current_fight_elapsed = 25.0;
    let json = serde_json::to_string(&cs).unwrap();
    let loaded: CombatState = serde_json::from_str(&json).unwrap();
    assert!((loaded.current_fight_elapsed - 0.0).abs() < f64::EPSILON);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib test_current_fight_elapsed 2>&1 | tail -5`
Expected: FAIL — field not found

**Step 3: Add the field**

Add to `CombatState` struct (after `boss_fight_timer`, line ~101):

```rust
    /// Accumulates time (seconds) the player has been fighting the current regular mob.
    /// Used for mob fight timeout detection. Resets when a new enemy spawns.
    #[serde(skip)]
    pub current_fight_elapsed: f64,
```

Initialize in `CombatState::new()` (after `boss_fight_timer: 0.0`):

```rust
    current_fight_elapsed: 0.0,
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib test_current_fight_elapsed 2>&1 | tail -5`
Expected: PASS

**Step 5: Commit**

```bash
git add src/combat/types.rs
git commit -m "feat(combat): add current_fight_elapsed transient field to CombatState (#320)"
```

---

### Task 4: Add `CombatEvent::CombatRetreat` and `TickEvent::CombatRetreat`

**Files:**
- Modify: `src/combat/events.rs:49-94` (`CombatEvent` enum)
- Modify: `src/core/tick_types.rs:20-197` (`TickEvent` enum)

**Step 1: Add `CombatEvent::CombatRetreat`**

Add to `CombatEvent` enum in `src/combat/events.rs` (after `SubzoneBossDefeated`):

```rust
    /// Player was overwhelmed and auto-retreated to a safe zone.
    CombatRetreat {
        zone_name: String,
    },
```

**Step 2: Add `TickEvent::CombatRetreat`**

Add to `TickEvent` enum in `src/core/tick_types.rs` (after `PlayerDiedInDungeon`):

```rust
    /// Player was overwhelmed and retreated to a safer zone.
    CombatRetreat { zone_name: String, message: String },
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles (may have unused warnings, that's OK)

**Step 4: Commit**

```bash
git add src/combat/events.rs src/core/tick_types.rs
git commit -m "feat(combat): add CombatRetreat event variants (#320)"
```

---

### Task 5: Implement the retreat logic in `combat/orchestration.rs`

This is the core implementation. The retreat logic lives here because it's analogous to `resolve_boss_enrage` which is already in this file.

**Files:**
- Modify: `src/combat/orchestration.rs`

**Step 1: Write the failing integration test**

Create test in `tests/combat_submodules_test.rs` (append to end of file):

```rust
// ═══════════════════════════════════════════════════════════════════
// Combat Fitness: Death Loop Prevention (#320)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_mob_fight_timeout_triggers_retreat() {
    let mut state = state_with_enemy(99999, 1, 0); // Unkillable mob, low damage
    let mut rng = seeded_rng();
    let mut achievements = Achievements::new();
    let d = derived(&state);
    let bonuses = default_bonuses();

    // Simulate fight lasting beyond MOB_FIGHT_TIMEOUT_SECONDS (30s)
    // Each tick = 0.1s, so 301 ticks = 30.1s
    let mut retreat_triggered = false;
    for _ in 0..301 {
        let events = update_combat(&mut rng, &mut state, 0.1, &bonuses, &mut achievements, &d);
        for e in &events {
            if matches!(e, CombatEvent::CombatRetreat { .. }) {
                retreat_triggered = true;
            }
        }
        if retreat_triggered {
            break;
        }
    }

    assert!(retreat_triggered, "Mob fight timeout should trigger retreat");
    assert!(
        state.combat_state.current_enemy.is_none(),
        "Enemy should be cleared after retreat"
    );
}

#[test]
fn test_mob_fight_timeout_does_not_apply_to_bosses() {
    let mut state = state_with_enemy(99999, 1, 0);
    state.zone_progression.fighting_boss = true; // Mark as boss fight
    let mut rng = seeded_rng();
    let mut achievements = Achievements::new();
    let d = derived(&state);
    let bonuses = default_bonuses();

    // Simulate 30+ seconds — should NOT trigger mob timeout (boss has its own enrage)
    let mut mob_retreat_triggered = false;
    for _ in 0..310 {
        let events = update_combat(&mut rng, &mut state, 0.1, &bonuses, &mut achievements, &d);
        for e in &events {
            if matches!(e, CombatEvent::CombatRetreat { .. }) {
                mob_retreat_triggered = true;
            }
        }
        if mob_retreat_triggered {
            break;
        }
    }

    assert!(
        !mob_retreat_triggered,
        "Mob fight timeout should not trigger for boss fights"
    );
}

#[test]
fn test_consecutive_deaths_triggers_retreat() {
    let mut rng = seeded_rng();
    let mut achievements = Achievements::new();
    let bonuses = default_bonuses();

    // Set up a player who will die quickly: enemy does huge damage
    let mut state = state_with_enemy(100, 9999, 0);
    state.consecutive_deaths = DEATH_LOOP_THRESHOLD - 1; // One more death = retreat
    let d = derived(&state);

    // Run combat until the player dies
    let mut retreat_triggered = false;
    for _ in 0..100 {
        let events = update_combat(&mut rng, &mut state, 0.1, &bonuses, &mut achievements, &d);
        for e in &events {
            if matches!(e, CombatEvent::CombatRetreat { .. }) {
                retreat_triggered = true;
            }
        }
        if retreat_triggered {
            break;
        }
    }

    assert!(
        retreat_triggered,
        "Should retreat after reaching death loop threshold"
    );
    assert_eq!(state.consecutive_deaths, 0, "Deaths should reset after retreat");
}

#[test]
fn test_consecutive_deaths_reset_on_kill() {
    let mut rng = seeded_rng();
    let mut achievements = Achievements::new();
    let bonuses = default_bonuses();

    // Player with 2 consecutive deaths, fighting a weak enemy they can kill
    let mut state = state_with_weak_enemy();
    state.consecutive_deaths = 2;

    // Advance combat until the enemy dies
    for _ in 0..100 {
        let events = update_combat(&mut rng, &mut state, 0.1, &bonuses, &mut achievements, &derived(&state));
        let enemy_died = events.iter().any(|e| matches!(e, CombatEvent::EnemyDied { .. } | CombatEvent::SubzoneBossDefeated { .. }));
        if enemy_died {
            break;
        }
    }

    assert_eq!(
        state.consecutive_deaths, 0,
        "Consecutive deaths should reset when enemy is killed"
    );
}

#[test]
fn test_retreat_target_is_last_safe_zone() {
    let mut state = state_with_enemy(99999, 1, 0);
    // Player has cleared bosses in zone 3
    state.zone_progression.defeated_bosses = vec![(1, 1), (1, 2), (1, 3), (2, 1), (3, 1)];
    state.zone_progression.current_zone_id = 5;
    state.zone_progression.current_subzone_id = 2;
    state.zone_progression.unlock_zone(3);
    state.zone_progression.unlock_zone(4);
    state.zone_progression.unlock_zone(5);
    let mut rng = seeded_rng();
    let mut achievements = Achievements::new();
    let d = derived(&state);
    let bonuses = default_bonuses();

    // Trigger mob fight timeout
    let mut retreat_zone_name = String::new();
    for _ in 0..310 {
        let events = update_combat(&mut rng, &mut state, 0.1, &bonuses, &mut achievements, &d);
        for e in &events {
            if let CombatEvent::CombatRetreat { zone_name } = e {
                retreat_zone_name = zone_name.clone();
            }
        }
        if !retreat_zone_name.is_empty() {
            break;
        }
    }

    // Should retreat to zone 3 (highest with a defeated boss)
    assert_eq!(state.zone_progression.current_zone_id, 3);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    assert!(!retreat_zone_name.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test combat_submodules_test test_mob_fight_timeout_triggers_retreat 2>&1 | tail -10`
Expected: FAIL — no `CombatRetreat` variant on `CombatEvent` (or no match in `update_combat`)

**Step 3: Implement the retreat logic**

In `src/combat/orchestration.rs`, add the following changes:

1. **Increment `current_fight_elapsed` for non-boss fights** — add after the boss enrage check (line ~44), before attack speed calculation:

```rust
    // --- Phase 1c: Mob fight timeout ---
    if !state.zone_progression.fighting_boss {
        state.combat_state.current_fight_elapsed += delta_time;
        if state.combat_state.current_fight_elapsed >= MOB_FIGHT_TIMEOUT_SECONDS {
            events.extend(resolve_combat_retreat(state));
            return events;
        }
    }
```

2. **Add the `resolve_combat_retreat` function** at the bottom of the file (after `resolve_boss_enrage`):

```rust
/// Resolves combat retreat: player is overwhelmed and retreats to the last safe zone.
///
/// Called when mob fight timeout or death loop threshold is reached.
/// Finds the highest zone with a defeated boss and travels there.
fn resolve_combat_retreat(state: &mut GameState) -> Vec<CombatEvent> {
    // Find last safe zone: highest zone_id with a defeated boss
    let safe_zone_id = state
        .zone_progression
        .defeated_bosses
        .iter()
        .map(|(zone_id, _)| *zone_id)
        .max()
        .unwrap_or(1); // Fall back to Zone 1

    let zone_name = crate::zones::get_zone(safe_zone_id)
        .map(|z| z.name.to_string())
        .unwrap_or_else(|| "Meadow".to_string());

    // Reset combat state
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 0.0;
    state.combat_state.current_fight_elapsed = 0.0;
    state.combat_state.current_enemy = None;

    // Move to safe zone
    state.zone_progression.current_zone_id = safe_zone_id;
    state.zone_progression.current_subzone_id = 1;
    state.zone_progression.kills_in_subzone = 0;
    state.zone_progression.fighting_boss = false;

    // Reset death counter
    state.consecutive_deaths = 0;

    vec![CombatEvent::CombatRetreat { zone_name }]
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --test combat_submodules_test test_mob_fight_timeout 2>&1 | tail -10`
Run: `cargo test --test combat_submodules_test test_consecutive_deaths 2>&1 | tail -10`
Run: `cargo test --test combat_submodules_test test_retreat_target 2>&1 | tail -10`
Expected: All PASS

**Step 5: Commit**

```bash
git add src/combat/orchestration.rs tests/combat_submodules_test.rs
git commit -m "feat(combat): implement mob fight timeout and retreat logic (#320)"
```

---

### Task 6: Increment/reset `consecutive_deaths` on player death and enemy kill

**Files:**
- Modify: `src/combat/enemy_attack.rs:85-125` (player death to regular mob)
- Modify: `src/combat/damage.rs:64-83` (enemy death / kill)

**Step 1: Increment `consecutive_deaths` on regular mob death**

In `src/combat/enemy_attack.rs`, inside the player death block for non-dungeon, non-boss deaths (around line 123, the `else if` that calls `enemy.reset_hp()`), add before `enemy.reset_hp()`:

```rust
                    // Track consecutive deaths for death loop detection
                    state.consecutive_deaths += 1;

                    // Check death loop threshold — trigger retreat
                    if state.consecutive_deaths >= DEATH_LOOP_THRESHOLD {
                        return super::orchestration::resolve_combat_retreat_public(state);
                    }
```

Note: We need to make `resolve_combat_retreat` callable from `enemy_attack.rs`. Add a public wrapper in `orchestration.rs`:

```rust
/// Public wrapper for resolve_combat_retreat, called from enemy_attack.rs
/// when the death loop threshold is reached.
pub fn resolve_combat_retreat_public(state: &mut GameState) -> Vec<CombatEvent> {
    resolve_combat_retreat(state)
}
```

**Step 2: Reset `consecutive_deaths` on enemy kill**

In `src/combat/damage.rs`, inside `handle_enemy_death()`, add after the achievements tracking (line ~73, after `achievements.on_enemy_killed(...)`):

```rust
    // Reset consecutive deaths counter on successful kill
    state.consecutive_deaths = 0;
```

**Step 3: Reset `current_fight_elapsed` on enemy kill**

Also in `handle_enemy_death()`, the `current_fight_elapsed` is already implicitly handled because the enemy is removed (line 76: `state.combat_state.current_enemy = None`). But we should reset it explicitly for clarity. Add near line 78:

```rust
    state.combat_state.current_fight_elapsed = 0.0;
```

**Step 4: Run all combat tests**

Run: `cargo test --test combat_submodules_test 2>&1 | tail -10`
Expected: All PASS

**Step 5: Commit**

```bash
git add src/combat/enemy_attack.rs src/combat/damage.rs src/combat/orchestration.rs
git commit -m "feat(combat): track consecutive deaths and reset on kill (#320)"
```

---

### Task 7: Map `CombatEvent::CombatRetreat` through the tick event pipeline

**Files:**
- Modify: `src/core/tick_stages.rs:329-571` (`process_combat_events`)
- Modify: `src/tick_events.rs:25-533` (`apply_tick_events`)

**Step 1: Map `CombatEvent::CombatRetreat` to `TickEvent::CombatRetreat` in `process_combat_events`**

In `src/core/tick_stages.rs`, add a new match arm in `process_combat_events` (after the `CombatEvent::PlayerDied` arm, around line 506):

```rust
            CombatEvent::CombatRetreat { zone_name } => {
                let message = format!(
                    "\u{1f3c3} Overwhelmed! Retreating to {}...",
                    zone_name
                );
                result.events.push(TickEvent::CombatRetreat {
                    zone_name,
                    message,
                });
            }
```

**Step 2: Handle `TickEvent::CombatRetreat` in `apply_tick_events`**

In `src/tick_events.rs`, add a new match arm (after the `PlayerDied` / `PlayerDiedInDungeon` arm, around line 148):

```rust
            TickEvent::CombatRetreat { zone_name, message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1f3c3}",
                    text: format!("Retreated to {}", zone_name),
                    color: Color::Yellow,
                    bold: false,
                    segments: None,
                });
            }
```

**Step 3: Verify it compiles and all tests pass**

Run: `cargo build 2>&1 | tail -5`
Run: `cargo test 2>&1 | tail -10`
Expected: compiles and all tests pass

**Step 4: Commit**

```bash
git add src/core/tick_stages.rs src/tick_events.rs
git commit -m "feat(combat): map CombatRetreat through tick event pipeline (#320)"
```

---

### Task 8: Reset `current_fight_elapsed` when new enemy spawns

**Files:**
- Modify: `src/core/enemy_spawning.rs` (or wherever `spawn_enemy_if_needed` is)

**Step 1: Find where enemies are spawned**

Check `src/core/enemy_spawning.rs` — the `spawn_enemy_if_needed` function sets `state.combat_state.current_enemy = Some(enemy)`. Add a reset of the fight timer when a new enemy spawns.

**Step 2: Add the reset**

In the `spawn_enemy_if_needed` function, after the enemy is assigned to `state.combat_state.current_enemy`, add:

```rust
    state.combat_state.current_fight_elapsed = 0.0;
```

**Step 3: Verify all tests pass**

Run: `cargo test 2>&1 | tail -10`
Expected: PASS

**Step 4: Commit**

```bash
git add src/core/enemy_spawning.rs
git commit -m "feat(combat): reset fight timer when new enemy spawns (#320)"
```

---

### Task 9: Run full CI checks

**Step 1: Run the full CI check suite**

Run: `make check`
Expected: All 5 checks pass (fmt, clippy, test, build, audit)

**Step 2: Fix any issues**

If clippy or fmt flags anything, fix it and re-run.

**Step 3: Final commit if needed**

```bash
git add -A
git commit -m "chore: fix lint/fmt issues from combat fitness implementation (#320)"
```
