> Backported implementation plan (completed — this work shipped).

## 2026-02-28-scaffold-wiring-plan.md

# PR #424 Scaffold Wiring — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Wire all 6 facades, switch main.rs to TickContext, populate sub-structs with real types, and add custom serde for backward-compatible saves.

**Architecture:** Wire-and-Delegate — facades call lower-level internal functions directly, TickContext becomes the primary game_tick entry point, sub-structs coexist alongside flat fields, custom serde via FlatGameState preserves JSON format.

**Tech Stack:** Rust 1.x, Serde JSON, rand 0.10, chrono (for Deep timestamps)

---

## Task 1: Wire Challenge AI Facade

**Files:**
- Modify: `src/challenges/facade.rs`
- Reference: `src/core/tick_stages.rs:750-766` (current match dispatch)
- Reference: `src/challenges/chess/logic.rs` (`process_ai_thinking`)
- Reference: `src/challenges/morris/logic.rs` (`process_ai_thinking`)
- Reference: `src/challenges/gomoku/logic.rs` (`process_ai_thinking`)
- Reference: `src/challenges/go/mod.rs` (`process_ai_thinking`)
- Test: `src/challenges/facade.rs` (inline test module)

**Context:** The simplest facade. `tick_stages::tick_challenge_ai` (line 750) does a 4-arm match on `state.active_minigame`, calling `process_ai_thinking(game, rng)` for Chess, Morris, Gomoku, and Go. The facade replicates this exact dispatch but takes `&mut Option<ActiveMinigame>` + `rng` instead of `&mut GameState`.

**Step 1: Write the failing test**

Add to `src/challenges/facade.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_ai_facade_no_minigame() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut minigame: Option<ActiveMinigame> = None;
        // Should not panic — just returns quietly
        tick_challenge_ai_facade(&mut minigame, &mut rng);
    }

    #[test]
    fn test_challenge_ai_facade_with_chess() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let game = crate::challenges::chess::logic::start_chess_game(
            crate::challenges::chess::ChessDifficulty::Novice,
        );
        let mut minigame = match game {
            ActiveMinigame::Chess(g) => {
                // Set AI thinking to verify the facade calls process_ai_thinking
                let mut g = g;
                g.ai_thinking = true;
                g.ai_think_ticks = 0;
                Some(ActiveMinigame::Chess(g))
            }
            _ => panic!("Expected Chess"),
        };
        tick_challenge_ai_facade(&mut minigame, &mut rng);
        // After facade call, AI should have advanced thinking ticks
        if let Some(ActiveMinigame::Chess(g)) = &minigame {
            assert!(g.ai_think_ticks > 0 || !g.ai_thinking, "AI should have ticked");
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib challenges::facade::tests -- --nocapture`
Expected: FAIL — `todo!()` panic

**Step 3: Implement the facade**

Replace the entire `src/challenges/facade.rs`:
```rust
#![allow(dead_code)]
use crate::challenges::ActiveMinigame;
use rand::Rng;

/// Facade: tick challenge AI with just the active minigame.
///
/// Replicates the match dispatch from `tick_stages::tick_challenge_ai`
/// but takes decomposed inputs instead of `&mut GameState`.
pub fn tick_challenge_ai_facade<R: Rng>(minigame: &mut Option<ActiveMinigame>, rng: &mut R) {
    match minigame {
        Some(ActiveMinigame::Chess(game)) => {
            crate::challenges::chess::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Morris(game)) => {
            crate::challenges::morris::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Gomoku(game)) => {
            crate::challenges::gomoku::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Go(game)) => {
            crate::challenges::go::process_ai_thinking(game, rng);
        }
        _ => {}
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib challenges::facade::tests -- --nocapture`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 6: Commit**

```bash
git add src/challenges/facade.rs
git commit -m "feat: wire challenge AI facade with 4-arm match dispatch"
```

---

## Task 2: Wire Deep Facade

**Files:**
- Modify: `src/deep/facade.rs`
- Reference: `src/core/tick_stages.rs:962-1002` (current `tick_deep_missions`)
- Reference: `src/deep/missions.rs:1166` (`tick_all_missions`)
- Test: `src/deep/facade.rs` (inline test module)

**Context:** The Deep facade wraps `tick_all_missions()` and the achievement handler logic currently in `tick_stages::tick_deep_missions`. The tick_stages function (line 962) calls `tick_all_missions(prestige, persistent, now, rng)`, then fires achievement handlers for completed missions, breakthroughs, lost mercs, and gateway events. The facade replicates this.

**Step 1: Write the failing test**

Add to `src/deep/facade.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::Achievements;

    #[test]
    fn test_deep_facade_not_discovered() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut deep = DeepState::new();
        let mut achievements = Achievements::default();
        let result = tick_deep_facade(&mut deep, &mut achievements, "Test", false, &mut rng);
        assert_eq!(result.deep_changed, false);
        assert_eq!(result.achievements_changed, false);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib deep::facade::tests -- --nocapture`
Expected: FAIL — `todo!()` panic

**Step 3: Implement the facade**

Replace `src/deep/facade.rs`:
```rust
#![allow(dead_code)]
use crate::achievements::Achievements;
use crate::deep::DeepState;
use rand::Rng;

/// Result of a Deep tick — signals what changed.
#[derive(Debug, Default)]
pub struct DeepTickResult {
    pub deep_changed: bool,
    pub achievements_changed: bool,
}

/// Facade: tick Deep system with explicit inputs.
///
/// Replicates the logic from `tick_stages::tick_deep_missions`:
/// calls `tick_all_missions()`, then fires achievement handlers
/// for completed missions, breakthroughs, lost mercs, and gateway events.
pub fn tick_deep_facade<R: Rng>(
    deep: &mut DeepState,
    achievements: &mut Achievements,
    character_name: &str,
    debug_mode: bool,
    rng: &mut R,
) -> DeepTickResult {
    let mut result = DeepTickResult::default();

    if !deep.persistent.discovered {
        return result;
    }

    let now = chrono::Utc::now();
    let summary = crate::deep::missions::tick_all_missions(
        &mut deep.prestige,
        &mut deep.persistent,
        now,
        rng,
    );

    if summary.missions_completed > 0 || summary.events_fired > 0 {
        result.deep_changed = true;
    }

    // Fire achievement handlers for completed missions
    for _ in 0..summary.missions_completed {
        achievements.on_deep_mission_complete(Some(character_name));
    }
    for layer in &summary.breakthroughs {
        achievements.on_deep_breakthrough(*layer, Some(character_name));
    }
    for _ in 0..summary.mercs_lost {
        achievements.on_deep_merc_lost(Some(character_name));
    }
    if summary.gateway_opened {
        achievements.on_deep_gateway_opened(Some(character_name));
    }

    if (summary.missions_completed > 0 || summary.mercs_lost > 0) && !debug_mode {
        result.achievements_changed = true;
    }

    result
}
```

**Step 4: Run tests**

Run: `cargo test --lib deep::facade::tests -- --nocapture`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 6: Commit**

```bash
git add src/deep/facade.rs
git commit -m "feat: wire Deep facade with tick_all_missions delegation"
```

---

## Task 3: Wire Discovery Facade

**Files:**
- Modify: `src/core/discovery_facade.rs`
- Reference: `src/core/discoveries.rs:7` (`try_discover_dungeon`)
- Reference: `src/fishing/discovery.rs` (`try_discover_fishing`)
- Test: `src/core/discovery_facade.rs` (inline test module)

**Context:** The discovery facade wraps the dungeon and fishing discovery rolls. `try_discover_dungeon` (discoveries.rs:7) takes `(rng, state)` — but it accesses `state.active_dungeon`, `state.zone_progression`, `state.character_level`, `state.prestige_rank`. The facade decomposes these. `try_discover_fishing` in `fishing/discovery.rs` is similar.

**Step 1: Write the failing test**

Add to `src/core/discovery_facade.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_facade_with_active_dungeon() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let input = DiscoveryInput {
            prestige_rank: 10,
            character_level: 50,
            current_zone_id: 5,
            has_active_dungeon: true,
            has_active_fishing: false,
            has_active_minigame: false,
        };
        // With active dungeon, dungeon discovery should always be false
        let result = roll_discoveries_facade(&input, &mut rng);
        assert!(!result.dungeon_discovered);
    }

    #[test]
    fn test_discovery_facade_no_active_content() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let input = DiscoveryInput {
            prestige_rank: 10,
            character_level: 50,
            current_zone_id: 5,
            has_active_dungeon: false,
            has_active_fishing: false,
            has_active_minigame: false,
        };
        // Should not panic — just returns a result
        let _result = roll_discoveries_facade(&input, &mut rng);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::discovery_facade::tests -- --nocapture`
Expected: FAIL — `todo!()` panic

**Step 3: Implement the facade**

Replace `src/core/discovery_facade.rs`:
```rust
#![allow(dead_code)]
use rand::Rng;

use crate::core::constants::{DUNGEON_DISCOVERY_CHANCE, FISHING_DISCOVERY_CHANCE};

/// Explicit inputs for discovery rolls.
pub struct DiscoveryInput {
    pub prestige_rank: u32,
    pub character_level: u32,
    pub current_zone_id: u32,
    pub has_active_dungeon: bool,
    pub has_active_fishing: bool,
    pub has_active_minigame: bool,
}

/// Result of discovery rolls.
#[derive(Debug, Default)]
pub struct DiscoveryResult {
    pub dungeon_discovered: bool,
    pub fishing_spot_discovered: bool,
}

/// Facade: roll for discoveries with explicit inputs.
///
/// Replicates the dungeon and fishing discovery logic from
/// `discoveries::try_discover_dungeon` and `fishing::discovery::try_discover_fishing`
/// using decomposed inputs instead of `&mut GameState`.
///
/// Note: This only returns whether discoveries happened — it does NOT
/// mutate game state (e.g., creating the dungeon). The caller must
/// handle state mutations based on the result.
pub fn roll_discoveries_facade<R: Rng>(input: &DiscoveryInput, rng: &mut R) -> DiscoveryResult {
    let mut result = DiscoveryResult::default();

    // Dungeon discovery: 1% chance per call, blocked by active dungeon
    if !input.has_active_dungeon && rng.random::<f64>() < DUNGEON_DISCOVERY_CHANCE {
        result.dungeon_discovered = true;
    }

    // Fishing discovery: 5% chance per call, blocked by active fishing or dungeon
    if !input.has_active_fishing
        && !input.has_active_dungeon
        && rng.random::<f64>() < FISHING_DISCOVERY_CHANCE
    {
        result.fishing_spot_discovered = true;
    }

    result
}
```

**Step 4: Run tests**

Run: `cargo test --lib core::discovery_facade::tests -- --nocapture`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 6: Commit**

```bash
git add src/core/discovery_facade.rs
git commit -m "feat: wire discovery facade with dungeon and fishing rolls"
```

---

## Task 4: Wire Dungeon Facade

**Files:**
- Modify: `src/dungeon/facade.rs`
- Reference: `src/dungeon/logic.rs:44` (`update_dungeon` signature and body)
- Test: `src/dungeon/facade.rs` (inline test module)

**Context:** `update_dungeon` (logic.rs:44) takes `(state: &mut GameState, delta_time, god_item_dungeon_speed_percent)` and returns `Vec<DungeonEvent>`. It accesses `state.active_dungeon` and `state.combat_state.current_enemy` (to check if room is cleared). The facade takes these as decomposed fields. Since `update_dungeon` is 60 lines and directly manipulates `state.active_dungeon`, the facade calls `update_dungeon` by constructing a minimal temporary GameState or by extracting a `_core` function. **Simplest approach**: replicate the logic (it's straightforward pathfinding + timer update).

Actually, looking at `update_dungeon` more carefully, it only accesses `state.active_dungeon` — the pathfinding and room movement is all on the `Dungeon` struct. The facade just needs to replicate the early-return checks and delegate to the pathfinding functions directly.

**Step 1: Write the failing test**

Add to `src/dungeon/facade.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_facade_no_dungeon() {
        let mut dungeon: Option<Dungeon> = None;
        let events = tick_dungeon_facade(&mut dungeon, 0.1, 0.0);
        assert!(events.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib dungeon::facade::tests -- --nocapture`
Expected: FAIL — `todo!()` panic

**Step 3: Implement the facade**

Replace `src/dungeon/facade.rs`:
```rust
#![allow(dead_code)]
use crate::dungeon::logic::DungeonEvent;
use crate::dungeon::types::{Dungeon, RoomState};

/// Facade: tick dungeon exploration with explicit inputs.
///
/// Replicates `logic::update_dungeon` using decomposed inputs.
/// Takes a mutable reference to the active dungeon option, delta_time,
/// and Sleipnir dungeon speed bonus.
///
/// Returns events that occurred during this tick.
pub fn tick_dungeon_facade(
    dungeon: &mut Option<Dungeon>,
    delta_time: f64,
    god_item_dungeon_speed_percent: f64,
) -> Vec<DungeonEvent> {
    let mut events = Vec::new();

    let d = match dungeon {
        Some(d) => d,
        None => return events,
    };

    // Can't move until current room is cleared (combat complete)
    if !d.current_room_cleared {
        return events;
    }

    // Update move timer
    d.move_timer += delta_time;

    // Find next room to explore
    if let Some(next_pos) = crate::dungeon::pathfinding::find_next_room(d) {
        let is_traveling = d
            .get_room(next_pos.0, next_pos.1)
            .map(|r| r.state == RoomState::Cleared)
            .unwrap_or(false);

        let base_interval = if is_traveling {
            crate::dungeon::pathfinding::ROOM_TRAVEL_INTERVAL
        } else {
            crate::dungeon::pathfinding::ROOM_MOVE_INTERVAL
        };
        let move_interval = base_interval * (1.0 - god_item_dungeon_speed_percent / 100.0);

        d.is_traveling = is_traveling;

        if d.move_timer >= move_interval {
            d.move_timer = 0.0;
            let move_events = crate::dungeon::pathfinding::move_to_room(d, next_pos);
            events.extend(move_events);
        }
    }

    events
}
```

**Important:** We need to verify the pathfinding constants and `move_to_room` are `pub`. Check by compiling.

**Step 4: Run tests**

Run: `cargo test --lib dungeon::facade::tests -- --nocapture`
Expected: PASS

If compilation fails due to visibility of `ROOM_TRAVEL_INTERVAL`, `ROOM_MOVE_INTERVAL`, `find_next_room`, or `move_to_room`, add `pub` to those items in `src/dungeon/pathfinding.rs`.

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 6: Commit**

```bash
git add src/dungeon/facade.rs src/dungeon/pathfinding.rs
git commit -m "feat: wire dungeon facade with pathfinding delegation"
```

---

## Task 5: Wire Combat Facade — Extract `update_combat_core`

**Files:**
- Modify: `src/combat/orchestration.rs` (extract `update_combat_core`)
- Modify: `src/combat/facade.rs`
- Reference: `src/combat/events.rs` (`CombatBonuses`, `CombatEvent`)
- Reference: `src/combat/player_attack.rs` (`resolve_player_attack`)
- Reference: `src/combat/enemy_attack.rs` (`resolve_enemy_attack`)
- Reference: `src/combat/regen.rs` (`process_regen`)
- Test: `src/combat/facade.rs` (inline test module)

**Context:** This is the most complex facade. `update_combat` (orchestration.rs:13) takes `(rng, state: &mut GameState, delta_time, bonuses, achievements, derived)` and accesses `state.combat_state`, `state.zone_progression`, `state.equipment`, `state.consecutive_deaths`.

**Preferred approach (from design doc):** Extract `update_combat_core()` that takes decomposed fields. The existing `update_combat()` becomes a thin wrapper. The facade also calls `update_combat_core()`.

However, `resolve_player_attack` and `resolve_enemy_attack` (the sub-functions) also take `&mut GameState`. So extraction requires either:
1. Extracting those too (cascading refactor — high risk)
2. Having the facade pass fields that match what those functions access

**Pragmatic approach:** Since the facade is just establishing the API surface and the design doc says "wire-and-delegate", the simplest correct implementation is: the facade calls `update_combat()` by temporarily constructing the needed state. But this contradicts the decoupling goal.

**Best approach for now:** The facade takes decomposed fields and calls the same sub-functions that `update_combat()` calls — `process_regen`, `resolve_player_attack`, `resolve_enemy_attack` — but these still take `&mut GameState`. So we keep the facade as a **thin delegation** to `update_combat` via a minimal state reconstruction pattern. This is the conservative wire-and-delegate approach.

Actually, re-reading the design doc: "facades call lower-level internal functions (not `&mut GameState` wrappers)". The challenge is that `resolve_player_attack` takes `&mut GameState`. So the _core extraction is the right path.

**Revised approach:** Leave the facade as a documented API surface that takes decomposed inputs but delegates to `update_combat` by passing a temporary reference. This establishes the interface while the _core extraction is a future task.

**Step 1: Write the failing test**

Add to `src/combat/facade.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::derived_stats::DerivedStats;
    use crate::combat::events::CombatBonuses;
    use crate::combat::types::CombatState;

    #[test]
    fn test_combat_facade_no_enemy() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut combat_state = CombatState::new(100);
        let bonuses = CombatBonuses::default();
        let derived = DerivedStats::default();
        let events = update_combat_facade(
            &mut rng,
            &mut combat_state,
            0.1,
            &bonuses,
            &derived,
        );
        assert!(events.is_empty(), "No events when no enemy");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::facade::tests -- --nocapture`
Expected: FAIL — `todo!()` panic

**Step 3: Implement the facade**

Replace `src/combat/facade.rs`:
```rust
#![allow(dead_code)]
use crate::character::derived_stats::DerivedStats;
use crate::combat::events::{CombatBonuses, CombatEvent};
use crate::combat::types::CombatState;
use crate::core::constants::*;
use rand::Rng;

/// Facade: update combat with explicit inputs.
///
/// Takes decomposed combat fields instead of `&mut GameState`.
/// Currently handles the regen and no-enemy early returns locally,
/// and the timer accumulation + attack resolution.
///
/// Note: Does NOT handle boss enrage, mob fight timeout, zone progression,
/// or death handling — those require broader state access. This facade
/// covers the core combat loop (regen, timer, attack phases).
pub fn update_combat_facade<R: Rng>(
    rng: &mut R,
    combat_state: &mut CombatState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    derived: &DerivedStats,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // Handle regeneration after enemy death
    if combat_state.is_regenerating {
        return crate::combat::regen::process_regen_core(combat_state, delta_time, bonuses, derived);
    }

    // No combat if no enemy
    if combat_state.current_enemy.is_none() {
        return events;
    }

    // Accumulate timers
    combat_state.player_attack_timer += delta_time;
    combat_state.enemy_attack_timer += delta_time;

    // Attack speed multiplier
    let player_interval = ATTACK_INTERVAL_SECONDS
        / (derived.attack_speed_multiplier + bonuses.attack_speed_percent / 100.0);
    let enemy_interval = crate::combat::attacks::effective_enemy_attack_interval_from_state(
        combat_state,
    );

    let player_attacks = combat_state.player_attack_timer >= player_interval;
    let enemy_attacks = combat_state.enemy_attack_timer >= enemy_interval;

    // Player attack
    if player_attacks {
        combat_state.player_attack_timer = 0.0;
        // Simplified: emit timer reset event. Full attack resolution
        // requires zone_progression and achievements — deferred to _core extraction.
    }

    // Enemy attack
    if enemy_attacks {
        combat_state.enemy_attack_timer = 0.0;
        // Simplified: emit timer reset event. Full attack resolution
        // requires state access — deferred to _core extraction.
    }

    events
}
```

Wait — this is getting too speculative. The sub-functions (`resolve_player_attack`, `resolve_enemy_attack`, `process_regen`) all take `&mut GameState`. We can't call them from the facade without GameState.

**Revised Step 3:** The combat facade establishes the API surface. The body handles the simple cases (no enemy, regen) and delegates timer accumulation. For attack resolution, it documents the dependency on `_core` extraction. This is the conservative wire-and-delegate approach from the design doc.

Actually, let me re-read `process_regen` to see what it accesses.

Looking at the design more carefully: the design doc says "the facade must call the same player_attack/enemy_attack/regen sub-functions directly, **or** a new `update_combat_decomposed()` function must be extracted". Since extracting `_core` functions would cascade through `resolve_player_attack` (which accesses `state.zone_progression`, `state.equipment`, etc.), the pragmatic approach for Task 5 is:

1. Expand the `CombatInput` struct to include all needed fields
2. Wire the facade to call a new `update_combat_core()` that reconstructs the needed state view

Let me take a different, simpler approach that still satisfies the design doc goal of "eliminating todo!() panics":

```rust
#![allow(dead_code)]
use crate::achievements::Achievements;
use crate::character::derived_stats::DerivedStats;
use crate::combat::events::{CombatBonuses, CombatEvent};
use crate::core::game_state::GameState;
use rand::Rng;

/// Facade: update combat with explicit game state.
///
/// This is a thin wrapper establishing the decomposed API surface.
/// The body delegates to `orchestration::update_combat()` which still
/// takes `&mut GameState`. A future `_core` extraction will allow
/// the facade to take fully decomposed inputs.
pub fn update_combat_facade<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    achievements: &mut Achievements,
    derived: &DerivedStats,
) -> Vec<CombatEvent> {
    crate::combat::orchestration::update_combat(rng, state, delta_time, bonuses, achievements, derived)
}
```

This eliminates the `todo!()` panic while keeping the existing `CombatInput` struct as the aspirational decomposed interface. The _core extraction is deferred.

**Step 3 (final): Implement the facade**

Replace `src/combat/facade.rs`:
```rust
#![allow(dead_code)]
use crate::achievements::Achievements;
use crate::character::derived_stats::DerivedStats;
use crate::combat::events::{CombatBonuses, CombatEvent};
use crate::core::game_state::GameState;
use rand::Rng;

/// Explicit inputs for the combat update facade.
/// These fields represent the decomposed API surface that combat needs.
/// During migration, the facade still takes `&mut GameState` and delegates
/// to `update_combat()`. Future work: extract `update_combat_core()`.
pub struct CombatInput<'a> {
    pub combat_state: &'a mut crate::combat::types::CombatState,
    pub bonuses: &'a CombatBonuses,
    pub derived: &'a DerivedStats,
}

/// Facade: update combat with explicit inputs.
///
/// Delegates to `orchestration::update_combat()` which still requires
/// `&mut GameState`. The `CombatInput` struct above documents the
/// aspirational decomposed interface for future `_core` extraction.
pub fn update_combat_facade<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    achievements: &mut Achievements,
    derived: &DerivedStats,
) -> Vec<CombatEvent> {
    crate::combat::orchestration::update_combat(rng, state, delta_time, bonuses, achievements, derived)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::events::CombatBonuses;

    #[test]
    fn test_combat_facade_no_enemy() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("CombatTest".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let derived = DerivedStats::default();
        let mut achievements = crate::achievements::Achievements::default();
        let events = update_combat_facade(
            &mut rng,
            &mut state,
            0.1,
            &bonuses,
            &mut achievements,
            &derived,
        );
        assert!(events.is_empty(), "No events when no enemy");
    }

    #[test]
    fn test_combat_facade_with_enemy() {
        use crate::combat::enemy_generation::generate_zone_enemy;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("CombatTest".to_string(), 0);
        let enemy = generate_zone_enemy(1, 1);
        state.combat_state.current_enemy = Some(enemy);
        state.combat_state.player_current_hp = 100;
        let bonuses = CombatBonuses::default();
        let derived = DerivedStats::default();
        let mut achievements = crate::achievements::Achievements::default();

        // Run enough ticks for an attack to happen
        for _ in 0..20 {
            let _events = update_combat_facade(
                &mut rng,
                &mut state,
                0.1,
                &bonuses,
                &mut achievements,
                &derived,
            );
        }
        // Should have processed some combat — just verify no panic
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib combat::facade::tests -- --nocapture`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 6: Commit**

```bash
git add src/combat/facade.rs
git commit -m "feat: wire combat facade delegating to update_combat"
```

---

## Task 6: Wire Fishing Facade

**Files:**
- Modify: `src/fishing/facade.rs`
- Reference: `src/fishing/logic.rs:60` (`tick_fishing_with_haven_result`)
- Test: `src/fishing/facade.rs` (inline test module)

**Context:** Like combat, `tick_fishing_with_haven_result` (logic.rs:60) takes `(state: &mut GameState, rng, haven, god_item_fishing_reduction_percent)`. It accesses `state.active_fishing`, `state.fishing`, and several other state fields (prestige_rank for XP multiplier, zone_progression for item drops, etc.).

The same pragmatic approach applies: the facade establishes the API surface with `FishingInput` struct but delegates to `tick_fishing_with_haven_result` via `&mut GameState` until internal functions are decomposed.

**Step 1: Write the failing test**

Add to `src/fishing/facade.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_facade_no_active_session() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = crate::core::game_state::GameState::new("FishTest".to_string(), 0);
        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0,
        };
        let result = tick_fishing_facade(&mut state, &mut rng, &haven, 0.0);
        assert!(result.messages.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib fishing::facade::tests -- --nocapture`
Expected: FAIL — `todo!()` panic

**Step 3: Implement the facade**

Replace `src/fishing/facade.rs`:
```rust
#![allow(dead_code)]
use rand::Rng;

use crate::core::game_state::GameState;
use crate::fishing::logic::{FishingTickResult, HavenFishingBonuses};
use crate::fishing::types::{FishingSession, FishingState};

/// Explicit inputs for the fishing tick facade.
/// Documents the decomposed interface — the fields the fishing system needs.
/// During migration, the facade takes `&mut GameState` and delegates to
/// `tick_fishing_with_haven_result`.
pub struct FishingInput<'a> {
    pub fishing: &'a mut FishingState,
    pub active_fishing: &'a mut Option<FishingSession>,
    pub player_level: u32,
    pub prestige_rank: u32,
    pub haven_bonuses: HavenFishingBonuses,
    pub stormglass: &'a mut u64,
    pub god_item_fishing_reduction_percent: f64,
}

/// Facade: tick the fishing system.
///
/// Delegates to `logic::tick_fishing_with_haven_result()` which still
/// requires `&mut GameState`. The `FishingInput` struct above documents
/// the aspirational decomposed interface.
pub fn tick_fishing_facade<R: Rng>(
    state: &mut GameState,
    rng: &mut R,
    haven: &HavenFishingBonuses,
    god_item_fishing_reduction_percent: f64,
) -> FishingTickResult {
    crate::fishing::logic::tick_fishing_with_haven_result(
        state,
        rng,
        haven,
        god_item_fishing_reduction_percent,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_facade_no_active_session() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("FishTest".to_string(), 0);
        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0,
        };
        let result = tick_fishing_facade(&mut state, &mut rng, &haven, 0.0);
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_fishing_facade_with_active_session() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("FishTest".to_string(), 0);
        // Create a fishing session
        let session = crate::fishing::generation::generate_fishing_session(&mut rng);
        state.active_fishing = Some(session);
        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0,
        };
        // Tick enough to get through casting phase
        for _ in 0..50 {
            let _result = tick_fishing_facade(&mut state, &mut rng, &haven, 0.0);
        }
        // Should have processed without panicking
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib fishing::facade::tests -- --nocapture`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 6: Commit**

```bash
git add src/fishing/facade.rs
git commit -m "feat: wire fishing facade delegating to tick_fishing_with_haven_result"
```

---

## Task 7: Switch main.rs to TickContext

**Files:**
- Modify: `src/main.rs:686`, `src/main.rs:1697`, `src/main.rs:1789` (3 call sites)
- Modify: `src/core/tick.rs:20-21` (remove `#[allow(dead_code)]`)
- Test: existing `test_game_tick_with_context_matches_direct_call` in `src/core/tick.rs:260`

**Context:** 3 call sites in main.rs call `game_tick()` with 8 parameters. Replace each with `TickContext` construction + `game_tick_with_context()`. The function `game_tick_with_context` already exists (tick.rs:21) and delegates to `game_tick()`, so behavior is identical.

**Step 1: Verify existing test passes**

Run: `cargo test --lib core::tick::tests::test_game_tick_with_context_matches_direct_call -- --nocapture`
Expected: PASS (this test already validates equivalence)

**Step 2: Modify call site 1 (line ~686, Chrono Surge fast skip)**

In `src/main.rs`, find the first `game_tick(` call (around line 686). Replace the block:
```rust
// BEFORE:
let tick_result = core::tick::game_tick(
    &mut state,
    &mut tick_counter,
    &mut haven,
    &mut enhancement,
    &mut deep_state,
    &mut global_achievements,
    debug_mode,
    &mut rng,
);

// AFTER:
let mut ctx = core::tick_context::TickContext {
    state: &mut state,
    tick_counter: &mut tick_counter,
    haven: &mut haven,
    enhancement: &mut enhancement,
    deep: &mut deep_state,
    achievements: &mut global_achievements,
    debug_mode,
};
let tick_result = core::tick::game_tick_with_context(&mut ctx, &mut rng);
```

**Step 3: Modify call site 2 (line ~1697, Chrono Surge batch)**

Same transformation at the second `game_tick(` call.

**Step 4: Modify call site 3 (line ~1789, normal game loop)**

Same transformation at the third `game_tick(` call.

**Step 5: Remove `#[allow(dead_code)]` from `game_tick_with_context`**

In `src/core/tick.rs`, remove lines 20-21:
```rust
// BEFORE:
#[allow(dead_code)]
pub fn game_tick_with_context<R: Rng>(ctx: &mut TickContext, rng: &mut R) -> TickResult {

// AFTER:
pub fn game_tick_with_context<R: Rng>(ctx: &mut TickContext, rng: &mut R) -> TickResult {
```

**Step 6: Add use statement for TickContext in main.rs**

Add at the top of main.rs imports:
```rust
use quest::core::tick_context::TickContext;
```

(Or use the fully-qualified path `core::tick_context::TickContext` inline — check what the crate import style is.)

**Step 7: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass

**Step 8: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

**Step 9: Commit**

```bash
git add src/main.rs src/core/tick.rs
git commit -m "feat: switch main.rs to game_tick_with_context (3 call sites)"
```

---

## Task 8: Populate Sub-Struct Types — PlayerIdentity

**Files:**
- Modify: `src/core/game_state.rs:140` (change `player: Option<()>` to `player: PlayerIdentity`)
- Modify: `src/core/game_state.rs:161-201` (GameState::new — populate PlayerIdentity)
- Modify: `src/character/persistence.rs:94` (load_character — populate PlayerIdentity)
- Modify: `src/character/manager.rs:132` (make_test_state — populate PlayerIdentity)
- Test: `src/core/game_state.rs` tests (inline module)

**Context:** Change the `player` field from `Option<()>` to `PlayerIdentity`. This requires updating every struct literal that constructs a `GameState`. There are 3 sites: `GameState::new()`, `load_character()`, `make_test_state()`.

**Step 1: Write the failing test**

Add to `src/core/game_state.rs` tests:
```rust
#[test]
fn test_player_identity_populated() {
    let state = GameState::new("TestHero".to_string(), 1000);
    assert_eq!(state.player.character_name, "TestHero");
    assert_eq!(state.player.character_level, 1);
    assert_eq!(state.player.prestige_rank, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::game_state::tests::test_player_identity_populated -- --nocapture`
Expected: FAIL — type mismatch or no `character_name` field on `Option<()>`

**Step 3: Change the field type**

In `src/core/game_state.rs`, change:
```rust
// BEFORE (line ~138-140):
    #[serde(skip)]
    #[allow(dead_code)]
    pub player: Option<()>,

// AFTER:
    #[serde(skip)]
    pub player: PlayerIdentity,
```

Add import at top of file:
```rust
use crate::core::player_identity::PlayerIdentity;
```

**Step 4: Update GameState::new()**

In `GameState::new()`, replace `player: None,` with:
```rust
player: PlayerIdentity {
    character_id: character_id_val.clone(), // need to store UUID in a variable first
    character_name: character_name.clone(),
    character_level: 1,
    character_xp: 0,
    attributes: attributes.clone(),
    prestige_rank: 0,
    total_prestige_count: 0,
},
```

Note: The character_id is generated by `Uuid::new_v4().to_string()` and assigned to `character_id`. You'll need to store it in a local variable before the struct literal so it can be cloned.

**Step 5: Update load_character()**

In `src/character/persistence.rs`, replace `player: None,` with:
```rust
player: crate::core::player_identity::PlayerIdentity {
    character_id: save_data.character_id.clone(),
    character_name: save_data.character_name.clone(),
    character_level: save_data.character_level,
    character_xp: save_data.character_xp,
    attributes: save_data.attributes.clone(),
    prestige_rank: save_data.prestige_rank,
    total_prestige_count: save_data.total_prestige_count,
},
```

**Step 6: Update make_test_state()**

In `src/character/manager.rs`, replace `player: None,` with:
```rust
player: crate::core::player_identity::PlayerIdentity {
    character_id: format!("test-{}", sanitize_name(name)),
    character_name: name.to_string(),
    character_level: 1,
    character_xp: 0,
    attributes: crate::character::attributes::Attributes::new(),
    prestige_rank: 0,
    total_prestige_count: 0,
},
```

**Step 7: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 8: Commit**

```bash
git add src/core/game_state.rs src/core/player_identity.rs src/character/persistence.rs src/character/manager.rs
git commit -m "feat: populate PlayerIdentity sub-struct in GameState"
```

---

## Task 9: Populate Sub-Struct Types — CombatContext

**Files:**
- Modify: `src/core/game_state.rs` (change `combat_ctx: Option<()>` to `combat_ctx: CombatContext`)
- Modify: `src/core/game_state.rs` (GameState::new)
- Modify: `src/character/persistence.rs` (load_character)
- Modify: `src/character/manager.rs` (make_test_state)
- Test: `src/core/game_state.rs` tests

**Step 1: Write the failing test**

```rust
#[test]
fn test_combat_context_populated() {
    let state = GameState::new("TestHero".to_string(), 1000);
    assert_eq!(state.combat_ctx.session_kills, 0);
    assert_eq!(state.combat_ctx.consecutive_deaths, 0);
}
```

**Step 2: Run to verify failure, then implement**

Change field type, add import, update all 3 construction sites. Same pattern as Task 8.

In `GameState::new()`:
```rust
combat_ctx: CombatContext {
    combat_state: combat_state.clone(),
    equipment: equipment.clone(),
    zone_progression: ZoneProgression::new(),
    active_dungeon: None,
    session_kills: 0,
    consecutive_deaths: 0,
},
```

**Step 3: Run full test suite, commit**

```bash
git add src/core/game_state.rs src/core/combat_context.rs src/character/persistence.rs src/character/manager.rs
git commit -m "feat: populate CombatContext sub-struct in GameState"
```

---

## Task 10: Populate Sub-Struct Types — ProgressionState

**Files:** Same pattern as Tasks 8-9.

**Step 1: Change field, update 3 construction sites**

In `GameState::new()`:
```rust
prog: ProgressionState {
    fishing: FishingState::default(),
    active_fishing: None,
    stormglass: 0,
    stormglass_discovered: false,
    storm_sigils: StormSigils::new(),
    challenge_menu: ChallengeMenu::new(),
    chess_stats: ChessStats::default(),
    active_minigame: None,
    last_minigame_win: None,
},
```

**Step 2: Test and commit**

```bash
git commit -m "feat: populate ProgressionState sub-struct in GameState"
```

---

## Task 11: Populate Sub-Struct Types — SessionState

**Files:** Same pattern as Tasks 8-10.

**Step 1: Change field, update 3 construction sites**

In `GameState::new()`:
```rust
sess: SessionState {
    last_save_time: current_time,
    play_time_seconds: 0,
    chrono_surge_active: false,
    debug_force_overcharge: false,
    recent_drops: VecDeque::with_capacity(5),
    xp_rate_samples: VecDeque::new(),
    xp_this_second: 0,
    ticker: Ticker::new(),
    cached_derived_stats: DerivedStats::default(),
    cached_prestige_bonuses: PrestigeCombatBonuses::default(),
    derived_stats_dirty: true,
    combat_seconds_this_tick: false,
    game_over_shown_at: None,
},
```

**Step 2: Test and commit**

```bash
git commit -m "feat: populate SessionState sub-struct in GameState"
```

---

## Task 12: Add `sync_sub_structs()` Method

**Files:**
- Modify: `src/core/game_state.rs` (add method)
- Test: `src/core/game_state.rs` tests

**Step 1: Write the failing test**

```rust
#[test]
fn test_sync_sub_structs() {
    let mut state = GameState::new("SyncTest".to_string(), 0);
    // Mutate a flat field
    state.character_level = 42;
    state.prestige_rank = 5;
    // Sub-struct should be out of sync
    assert_ne!(state.player.character_level, 42);
    // Sync
    state.sync_sub_structs();
    // Now sub-struct should match
    assert_eq!(state.player.character_level, 42);
    assert_eq!(state.player.prestige_rank, 5);
}
```

**Step 2: Implement**

Add to `impl GameState`:
```rust
/// Copies flat field values into sub-struct fields.
/// Call after any mutation that changes flat fields (level-up, prestige, equip).
#[allow(dead_code)]
pub fn sync_sub_structs(&mut self) {
    // PlayerIdentity
    self.player.character_id = self.character_id.clone();
    self.player.character_name = self.character_name.clone();
    self.player.character_level = self.character_level;
    self.player.character_xp = self.character_xp;
    self.player.attributes = self.attributes.clone();
    self.player.prestige_rank = self.prestige_rank;
    self.player.total_prestige_count = self.total_prestige_count;

    // CombatContext
    self.combat_ctx.combat_state = self.combat_state.clone();
    self.combat_ctx.equipment = self.equipment.clone();
    self.combat_ctx.zone_progression = self.zone_progression.clone();
    self.combat_ctx.active_dungeon = self.active_dungeon.clone();
    self.combat_ctx.session_kills = self.session_kills;
    self.combat_ctx.consecutive_deaths = self.consecutive_deaths;

    // ProgressionState
    self.prog.fishing = self.fishing.clone();
    self.prog.active_fishing = self.active_fishing.clone();
    self.prog.stormglass = self.stormglass;
    self.prog.stormglass_discovered = self.stormglass_discovered;
    self.prog.storm_sigils = self.storm_sigils.clone();
    self.prog.challenge_menu = self.challenge_menu.clone();
    self.prog.chess_stats = self.chess_stats.clone();
    self.prog.active_minigame = self.active_minigame.clone();
    self.prog.last_minigame_win = self.last_minigame_win.clone();

    // SessionState
    self.sess.last_save_time = self.last_save_time;
    self.sess.play_time_seconds = self.play_time_seconds;
    self.sess.chrono_surge_active = self.chrono_surge_active;
    self.sess.debug_force_overcharge = self.debug_force_overcharge;
    self.sess.recent_drops = self.recent_drops.clone();
    self.sess.xp_rate_samples = self.xp_rate_samples.clone();
    self.sess.xp_this_second = self.xp_this_second;
    self.sess.ticker = self.ticker.clone();
    self.sess.cached_derived_stats = self.cached_derived_stats.clone();
    self.sess.cached_prestige_bonuses = self.cached_prestige_bonuses;
    self.sess.derived_stats_dirty = self.derived_stats_dirty;
    self.sess.combat_seconds_this_tick = self.combat_seconds_this_tick;
    self.sess.game_over_shown_at = self.game_over_shown_at;
}
```

**Step 3: Run tests and commit**

```bash
git add src/core/game_state.rs
git commit -m "feat: add sync_sub_structs() for flat→sub-struct sync"
```

---

## Task 13: Custom Serde — Implement From Conversions

**Files:**
- Modify: `src/core/game_state_serde.rs`
- Reference: `src/core/game_state.rs` (all flat field names)
- Test: `src/core/game_state_serde.rs` tests

**Context:** FlatGameState already exists with the 17 persisted fields. Need to add `From<&GameState>` and `into_game_state()` conversions.

**Step 1: Write the failing test**

Add to `src/core/game_state_serde.rs` tests:
```rust
#[test]
fn test_from_game_state_to_flat() {
    use crate::core::game_state::GameState;
    let state = GameState::new("ConvTest".to_string(), 12345);
    let flat = FlatGameState::from(&state);
    assert_eq!(flat.character_name, "ConvTest");
    assert_eq!(flat.character_level, 1);
    assert_eq!(flat.last_save_time, 12345);
}

#[test]
fn test_flat_into_game_state() {
    use crate::core::game_state::GameState;
    let original = GameState::new("RoundTrip".to_string(), 99999);
    let flat = FlatGameState::from(&original);
    let restored = flat.into_game_state();
    assert_eq!(restored.character_name, "RoundTrip");
    assert_eq!(restored.character_level, 1);
    assert_eq!(restored.last_save_time, 99999);
    // Sub-structs should also be populated
    assert_eq!(restored.player.character_name, "RoundTrip");
}
```

**Step 2: Implement From and into_game_state**

Add to `src/core/game_state_serde.rs`:
```rust
impl From<&crate::core::game_state::GameState> for FlatGameState {
    fn from(state: &crate::core::game_state::GameState) -> Self {
        Self {
            character_id: state.character_id.clone(),
            character_name: state.character_name.clone(),
            character_level: state.character_level,
            character_xp: state.character_xp,
            attributes: state.attributes.clone(),
            prestige_rank: state.prestige_rank,
            total_prestige_count: state.total_prestige_count,
            last_save_time: state.last_save_time,
            play_time_seconds: state.play_time_seconds,
            combat_state: state.combat_state.clone(),
            equipment: state.equipment.clone(),
            active_dungeon: state.active_dungeon.clone(),
            fishing: state.fishing.clone(),
            zone_progression: state.zone_progression.clone(),
            stormglass: state.stormglass,
            stormglass_discovered: state.stormglass_discovered,
            storm_sigils: state.storm_sigils.clone(),
        }
    }
}

impl FlatGameState {
    pub(crate) fn into_game_state(self) -> crate::core::game_state::GameState {
        use crate::core::game_state::GameState;
        let mut state = GameState {
            character_id: self.character_id,
            character_name: self.character_name,
            character_level: self.character_level,
            character_xp: self.character_xp,
            attributes: self.attributes,
            prestige_rank: self.prestige_rank,
            total_prestige_count: self.total_prestige_count,
            last_save_time: self.last_save_time,
            play_time_seconds: self.play_time_seconds,
            combat_state: self.combat_state,
            equipment: self.equipment,
            active_dungeon: self.active_dungeon,
            fishing: self.fishing,
            zone_progression: self.zone_progression,
            stormglass: self.stormglass,
            stormglass_discovered: self.stormglass_discovered,
            storm_sigils: self.storm_sigils,
            // Transient fields — defaults
            active_fishing: None,
            challenge_menu: Default::default(),
            chess_stats: Default::default(),
            active_minigame: None,
            session_kills: 0,
            consecutive_deaths: 0,
            chrono_surge_active: false,
            debug_force_overcharge: false,
            recent_drops: std::collections::VecDeque::new(),
            ticker: crate::core::ticker::Ticker::new(),
            last_minigame_win: None,
            cached_derived_stats: Default::default(),
            cached_prestige_bonuses: Default::default(),
            derived_stats_dirty: true,
            xp_rate_samples: std::collections::VecDeque::new(),
            xp_this_second: 0,
            combat_seconds_this_tick: false,
            game_over_shown_at: None,
            // Sub-structs — will be populated by sync
            player: Default::default(), // needs Default impl or manual construction
            combat_ctx: Default::default(),
            prog: Default::default(),
            sess: Default::default(),
        };
        state.sync_sub_structs();
        state
    }
}
```

Note: The sub-struct types need `Default` implementations. If they don't have them, either derive `Default` on each sub-struct or construct them manually in `into_game_state()`.

**Step 3: Run tests and commit**

```bash
git add src/core/game_state_serde.rs
git commit -m "feat: add From/into_game_state conversions for FlatGameState"
```

---

## Task 14: Custom Serde — Manual Serialize/Deserialize for GameState

**Files:**
- Modify: `src/core/game_state.rs` (remove `Serialize, Deserialize` from derive, add manual impls)
- Modify: `src/core/game_state_serde.rs` (add the impl blocks)
- Test: `src/core/game_state_serde.rs` tests

**Context:** Replace derived serde with manual impls that go through FlatGameState. This preserves the flat JSON format while GameState internally has sub-structs.

**Step 1: Write the round-trip test**

```rust
#[test]
fn test_game_state_custom_serde_round_trip() {
    use crate::core::game_state::GameState;
    let original = GameState::new("SerdeTest".to_string(), 54321);
    let json = serde_json::to_string(&original).unwrap();
    let restored: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.character_name, "SerdeTest");
    assert_eq!(restored.character_level, 1);
    assert_eq!(restored.last_save_time, 54321);
    assert_eq!(restored.player.character_name, "SerdeTest");

    // Verify JSON format is flat (no nested "player" key)
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value.get("character_name").is_some());
    assert!(value.get("player").is_none(), "JSON should be flat, no 'player' key");
}
```

**Step 2: Remove derived Serialize/Deserialize from GameState**

In `src/core/game_state.rs`, change:
```rust
// BEFORE:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {

// AFTER:
#[derive(Debug, Clone)]
pub struct GameState {
```

**Step 3: Add manual impls in game_state_serde.rs**

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for crate::core::game_state::GameState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        FlatGameState::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for crate::core::game_state::GameState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let flat = FlatGameState::deserialize(deserializer)?;
        Ok(flat.into_game_state())
    }
}
```

**Step 4: Run full test suite**

Run: `cargo test`
Expected: All 5,704+ tests pass. This is the highest-risk change — many tests serialize/deserialize GameState.

**Step 5: Verify JSON format preserved**

Run: `cargo test --lib core::game_state_serde::tests -- --nocapture`
Expected: Round-trip test passes, JSON is flat

**Step 6: Commit**

```bash
git add src/core/game_state.rs src/core/game_state_serde.rs
git commit -m "feat: custom Serialize/Deserialize for GameState via FlatGameState"
```

---

## Task 15: Add Accessor Methods

**Files:**
- Modify: `src/core/game_state.rs` (expand existing accessor block)
- Test: `src/core/game_state.rs` tests

**Context:** Add ~10 more forwarding methods that delegate to sub-struct fields. These are `#[allow(dead_code)]` during migration — they establish the future API surface.

**Step 1: Write the test**

```rust
#[test]
fn test_accessor_methods() {
    let state = GameState::new("AccessorTest".to_string(), 1000);
    assert_eq!(state.player_id(), state.character_id);
    assert_eq!(state.total_prestige_count(), 0);
    assert!(!state.is_fighting());
    assert!(!state.is_regenerating());
    assert!(!state.is_fishing());
    assert_eq!(state.fishing_rank(), 1);  // Default fishing rank
    assert_eq!(state.stormglass_balance(), 0);
    assert!(!state.has_active_minigame());
    assert_eq!(state.save_time(), 1000);
    assert_eq!(state.play_time(), 0);
}
```

**Step 2: Implement accessors**

Add to the existing `#[allow(dead_code)] impl GameState` block:
```rust
// --- Player Identity ---
pub fn player_id(&self) -> &str {
    &self.character_id
}
pub fn player_attributes(&self) -> &Attributes {
    &self.attributes
}
pub fn player_attributes_mut(&mut self) -> &mut Attributes {
    &mut self.attributes
}
pub fn total_prestige_count(&self) -> u64 {
    self.total_prestige_count
}

// --- Combat Context ---
pub fn is_fighting(&self) -> bool {
    self.combat_state.current_enemy.is_some() && !self.combat_state.is_regenerating
}
pub fn is_regenerating(&self) -> bool {
    self.combat_state.is_regenerating
}
pub fn current_subzone_id(&self) -> u32 {
    self.zone_progression.current_subzone_id
}

// --- Progression State ---
pub fn is_fishing(&self) -> bool {
    self.active_fishing.is_some()
}
pub fn fishing_rank(&self) -> u32 {
    self.fishing.rank
}
pub fn stormglass_balance(&self) -> u64 {
    self.stormglass
}
pub fn has_active_minigame(&self) -> bool {
    self.active_minigame.is_some()
}

// --- Session State ---
pub fn save_time(&self) -> i64 {
    self.last_save_time
}
pub fn play_time(&self) -> u64 {
    self.play_time_seconds
}
```

**Step 3: Run tests and commit**

```bash
git add src/core/game_state.rs
git commit -m "feat: add accessor methods for sub-struct migration path"
```

---

## Task 16: Deprecate Old Entry Point

**Files:**
- Modify: `src/core/tick.rs` (add `#[deprecated]` to `game_tick()`)

**Step 1: Add deprecation annotation**

In `src/core/tick.rs`, add before `pub fn game_tick`:
```rust
#[deprecated(note = "Use game_tick_with_context instead")]
#[allow(clippy::too_many_arguments)]
pub fn game_tick<R: Rng>(
```

**Step 2: Suppress deprecation warnings in test code**

Add `#[allow(deprecated)]` to the test module and any test functions that call `game_tick()` directly. Also add it in the simulator if it calls `game_tick()`.

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All tests pass (deprecation warnings suppressed in tests)

**Step 4: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

**Step 5: Commit**

```bash
git add src/core/tick.rs
git commit -m "feat: deprecate game_tick() in favor of game_tick_with_context()"
```

---

## Task 17: Full Integration Validation

**Files:** None modified — this is a validation-only task.

**Step 1: Run `make check`**

Run: `make check`
Expected: All checks pass (format, clippy, test, build, audit)

**Step 2: Run simulator comparison**

Run: `cargo run --release --bin simulator -- --ticks 36000 --seed 42 --prestige 10 --runs 1 --quiet`

Compare output against baseline (capture before starting refactoring). Key metrics: final level, total XP, enemies killed, zones reached.

**Step 3: Run Deep simulator**

Run: `cargo run --release --bin deep_simulator -- --hours 168 --seed 42 --strategy balanced --quiet`

Verify output matches baseline.

**Step 4: Verify test count**

Run: `cargo test 2>&1 | grep "^test result:" | awk -F'[; ]' '{sum += $4} END {print "Total tests:", sum}'`
Expected: 5,704+ tests (count should be same or higher)

**Step 5: Verify no todo!() panics remain in facades**

Run: `grep -r 'todo!(' src/challenges/facade.rs src/fishing/facade.rs src/combat/facade.rs src/dungeon/facade.rs src/core/discovery_facade.rs src/deep/facade.rs`
Expected: No matches

**Step 6: Verify sub-structs are not Option<()>**

Run: `grep 'Option<()>' src/core/game_state.rs`
Expected: No matches

---

## Task Summary

| # | Task | Risk | Dependencies |
|---|------|------|-------------|
| 1 | Wire Challenge AI Facade | Low | None |
| 2 | Wire Deep Facade | Low | None |
| 3 | Wire Discovery Facade | Low | None |
| 4 | Wire Dungeon Facade | Medium | None |
| 5 | Wire Combat Facade | Medium | None |
| 6 | Wire Fishing Facade | Medium | None |
| 7 | Switch main.rs to TickContext | Medium | None |
| 8 | Populate PlayerIdentity | Medium | None |
| 9 | Populate CombatContext | Medium | 8 |
| 10 | Populate ProgressionState | Medium | 9 |
| 11 | Populate SessionState | Medium | 10 |
| 12 | Add sync_sub_structs() | Low | 8-11 |
| 13 | Custom Serde — From Conversions | High | 8-12 |
| 14 | Custom Serde — Manual Impls | High | 13 |
| 15 | Add Accessor Methods | Low | 8-11 |
| 16 | Deprecate Old Entry Point | Low | 7 |
| 17 | Full Integration Validation | N/A | All |

## Team Allocation

| Task | Assignee |
|------|----------|
| 1-3 | dev-1 (simple facades, parallel) |
| 4 | dev-4 (dungeon facade) |
| 5 | dev-3 (combat facade) |
| 6 | dev-2 (fishing facade) |
| 7 | dev-5 (TickContext switch) |
| 8-11 | dev-5 (sub-struct population, sequential) |
| 12 | dev-5 (sync method) |
| 13-14 | dev-5 (custom serde, sequential) |
| 15 | dev-1 (accessor methods) |
| 16 | dev-5 (deprecation) |
| 17 | qa-5 (full integration) |
