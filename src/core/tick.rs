//! Extracted game tick logic — the central per-tick orchestration function.
//!
//! This module contains the `game_tick()` function that processes a single
//! 100ms game tick, updating combat, fishing, dungeon, challenges, achievements,
//! and play time. It returns a [`TickResult`] describing what happened so the
//! presentation layer (main.rs) can update the UI without game logic depending
//! on any UI types.

use super::tick_context::TickContext;
use super::tick_stages;
pub use super::tick_types::{TickEvent, TickResult};
use crate::achievements::Achievements;
use crate::core::constants::TICK_INTERVAL_MS;
use crate::core::game_logic::spawn_enemy_if_needed;
use crate::core::game_state::GameState;
use crate::haven::Haven;
use rand::Rng;

/// New entry point using TickContext. Delegates to the existing game_tick().
pub fn game_tick_with_context<R: Rng>(ctx: &mut TickContext, rng: &mut R) -> TickResult {
    game_tick(
        ctx.state,
        ctx.tick_counter,
        ctx.haven,
        ctx.enhancement,
        ctx.deep,
        ctx.achievements,
        ctx.debug_mode,
        rng,
    )
}

/// Processes a single 100ms game tick.
///
/// Updates game state (combat, fishing, dungeon, challenges, achievements,
/// play time) and returns a [`TickResult`] describing what happened.
///
/// # Arguments
/// - `state` — Mutable game state (character, combat, zones, equipment, etc.)
/// - `tick_counter` — Counts ticks for play-time tracking (10 ticks = 1 second).
///   Caller owns this counter across ticks.
/// - `haven` — Mutable Haven state for bonus calculations and discovery.
/// - `enhancement` — Mutable Enhancement state for soulforge discovery.
/// - `deep` — Mutable Deep state for mercenary expedition discovery.
/// - `achievements` — Mutable achievement state for unlock tracking.
/// - `debug_mode` — When true, suppresses achievement/haven-save signals.
/// - `rng` — Random number generator (any `impl Rng`). Pass
///   `&mut rand::rng()` in production, or a seeded
///   `rand_chacha::ChaCha8Rng` in tests for deterministic behavior.
///
/// # Returns
/// A [`TickResult`] containing all events and flags. The caller (main.rs)
/// is responsible for:
/// - Mapping events to combat log entries via `add_log_entry()`
/// - Creating `VisualEffect` objects for [`TickEvent::PlayerAttack`] events
/// - Updating `visual_effects` lifetimes
/// - Persisting achievements to disk when `achievements_changed` is true
/// - Persisting Haven to disk when `haven_changed` is true
/// - Persisting Enhancement to disk when `enhancement_changed` is true
/// - Persisting Deep to disk when `deep_changed` is true
/// - Showing the Leviathan encounter modal when `leviathan_encounter` is `Some`
/// - Showing achievement modal overlay when `achievement_modal_ready` is non-empty
#[allow(clippy::too_many_arguments)]
pub fn game_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    enhancement: &mut crate::enhancement::EnhancementProgress,
    deep: &mut crate::deep::DeepState,
    achievements: &mut Achievements,
    debug_mode: bool,
    rng: &mut R,
) -> TickResult {
    let mut result = TickResult::default();
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    // ── 0. Compute merged Haven + Sigil bonuses ─────────────────
    let (haven_bonuses, sigil_bonuses) = tick_stages::compute_merged_bonuses(haven, state);

    // ── 1. Process challenge AI thinking ────────────────────────
    tick_stages::tick_challenge_ai(state, rng);

    // ── 2. Try challenge discovery (skipped during Chrono Surge) ─
    tick_stages::tick_challenge_discovery(state, &haven_bonuses, rng, &mut result);

    // ── 3. Sync player max HP with cached derived stats ─────────
    tick_stages::sync_derived_stats(state, enhancement);

    // ── 4. Update dungeon exploration ───────────────────────────
    tick_stages::process_dungeon_events(state, delta_time, &haven_bonuses, &mut result, rng);

    // ── 5. Update fishing (mutually exclusive with combat) ──────
    if tick_stages::process_fishing_tick(
        state,
        tick_counter,
        delta_time,
        &haven_bonuses,
        achievements,
        debug_mode,
        &mut result,
        rng,
    ) {
        // Fishing was active — skip combat, collect achievements and return
        tick_stages::collect_achievement_events(achievements, &mut result);
        return result;
    }

    // ── 6. Combat ───────────────────────────────────────────────
    tick_stages::run_combat(
        state,
        delta_time,
        &haven_bonuses,
        &sigil_bonuses,
        achievements,
        deep,
        debug_mode,
        &mut result,
        rng,
    );

    // ── 6b. Decay HUD flash timers ──────────────────────────────
    state.combat_state.tick_hud(delta_time);

    // ── 7. Spawn enemy if needed ────────────────────────────────
    spawn_enemy_if_needed(state);

    // ── 8. Update play time ─────────────────────────────────────
    tick_stages::update_play_time(state, tick_counter);

    // ── 9. Collect achievement notifications ────────────────────
    tick_stages::collect_achievement_events(achievements, &mut result);

    // ── 10. Haven discovery check ────────────────────────────────
    tick_stages::tick_haven_discovery(state, haven, achievements, debug_mode, &mut result, rng);

    // ── 11. Soulforge discovery check ────────────────────────────
    tick_stages::tick_soulforge_discovery(
        state,
        enhancement,
        achievements,
        debug_mode,
        &mut result,
        rng,
    );

    // ── 11b. Deep discovery ───────────────────────────────────────
    // Discovery is triggered by defeating The Expanse cycle boss.
    // No per-tick random roll.

    // ── 11c. Deep mission ticking ──────────────────────────────────
    tick_stages::tick_deep_missions(state, deep, achievements, debug_mode, &mut result, rng);

    // ── 12. Achievement modal accumulation ────────────────────────
    if achievements.is_modal_ready() {
        result.achievement_modal_ready = achievements.take_modal_queue();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::DeepState;
    use crate::enhancement::EnhancementProgress;
    use crate::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_game_tick_returns_empty_result_for_idle_state() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            false,
            &mut rng,
        );

        // A fresh state with no enemy should just spawn an enemy
        assert!(state.combat_state.current_enemy.is_some());
        // tick_counter should have incremented
        assert_eq!(tick_counter, 1);
        // No leviathan encounter
        assert!(result.leviathan_encounter.is_none());
    }

    #[test]
    fn test_game_tick_increments_play_time() {
        let mut state = GameState::new("Time Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let initial_time = state.play_time_seconds;

        for _ in 0..10 {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep,
                &mut achievements,
                false,
                &mut rng,
            );
        }

        assert_eq!(state.play_time_seconds, initial_time + 1);
        assert_eq!(tick_counter, 0);
    }

    #[test]
    fn test_game_tick_spawns_enemy() {
        let mut state = GameState::new("Spawn Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        assert!(state.combat_state.current_enemy.is_none());

        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            false,
            &mut rng,
        );

        assert!(state.combat_state.current_enemy.is_some());
    }

    #[test]
    fn test_game_tick_with_context_matches_direct_call() {
        use crate::core::tick_context::TickContext;

        let mut state1 = GameState::new("Test1".to_string(), 0);
        let mut state2 = state1.clone();
        let mut tc1: u32 = 0;
        let mut tc2: u32 = 0;
        let mut haven1 = Haven::default();
        let mut haven2 = haven1.clone();
        let mut enh1 = EnhancementProgress::new();
        let mut enh2 = enh1.clone();
        let mut deep1 = DeepState::new();
        let mut deep2 = deep1.clone();
        let mut ach1 = Achievements::default();
        let mut ach2 = ach1.clone();
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let result1 = game_tick(
            &mut state1,
            &mut tc1,
            &mut haven1,
            &mut enh1,
            &mut deep1,
            &mut ach1,
            false,
            &mut rng1,
        );

        let mut ctx = TickContext {
            state: &mut state2,
            tick_counter: &mut tc2,
            haven: &mut haven2,
            enhancement: &mut enh2,
            deep: &mut deep2,
            achievements: &mut ach2,
            debug_mode: false,
        };
        let result2 = game_tick_with_context(&mut ctx, &mut rng2);

        assert_eq!(result1.events.len(), result2.events.len());
    }

    #[test]
    fn test_game_tick_combat_produces_events() {
        use crate::character::attributes::AttributeType;
        use crate::character::derived_stats::DerivedStats;

        let mut state = GameState::new("Combat Test".to_string(), 0);
        state.attributes.set(AttributeType::Strength, 50);
        state.attributes.set(AttributeType::Intelligence, 50);
        let derived =
            DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
        state.combat_state.update_max_hp(derived.max_hp);
        state.combat_state.player_current_hp = state.combat_state.player_max_hp;

        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let mut all_events = Vec::new();
        for _ in 0..5000 {
            let result = game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep,
                &mut achievements,
                false,
                &mut rng,
            );
            all_events.extend(result.events);

            // Stop after first enemy defeated
            if all_events
                .iter()
                .any(|e| matches!(e, TickEvent::EnemyDefeated { .. }))
            {
                break;
            }
        }

        assert!(
            all_events
                .iter()
                .any(|e| matches!(e, TickEvent::EnemyDefeated { .. })),
            "Should have an EnemyDefeated event"
        );
    }
}
