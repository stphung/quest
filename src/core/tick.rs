//! Extracted game tick logic — the central per-tick orchestration function.
//!
//! This module contains the `game_tick()` function that processes a single
//! 100ms game tick, updating combat, fishing, dungeon, challenges, achievements,
//! and play time. It returns a [`TickResult`] describing what happened so the
//! presentation layer (main.rs) can update the UI without game logic depending
//! on any UI types.

use super::tick_stages;
pub use super::tick_types::{TickEvent, TickResult};
use crate::achievements::Achievements;
use crate::challenges::ActiveMinigame;
use crate::combat::events::CombatBonuses;
use crate::combat::logic::update_combat;
use crate::core::constants::{HAVEN_MIN_PRESTIGE_RANK, TICKS_PER_SECOND, TICK_INTERVAL_MS};
use crate::core::game_logic::spawn_enemy_if_needed;
use crate::core::game_state::GameState;
use crate::haven::Haven;
use crate::stormglass::sigils::SigilBonuses;
use rand::Rng;

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
    let mut haven_bonuses = haven.compute_bonuses();
    let sigil_bonuses = SigilBonuses::compute(&state.storm_sigils);

    // Inject sigil bonuses into haven_bonuses for fields that flow through
    // process_item_drop (drop_rate) and process_fishing_tick (fishing speed)
    haven_bonuses.drop_rate_percent += sigil_bonuses.drop_rate_percent;
    haven_bonuses.fishing_timer_reduction += sigil_bonuses.fishing_speed_percent;

    // ── 1. Process challenge AI thinking ────────────────────────
    match &mut state.active_minigame {
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

    // ── 2. Try challenge discovery (skipped during Chrono Surge) ─
    if !state.chrono_surge_active {
        let haven_discovery = haven_bonuses.challenge_discovery_percent;
        if let Some(challenge_type) =
            crate::challenges::menu::try_discover_challenge_with_haven(state, rng, haven_discovery)
        {
            let icon = challenge_type.icon();
            let flavor = challenge_type.discovery_flavor();
            result.events.push(TickEvent::ChallengeDiscovered {
                challenge_type,
                message: format!("{} {}", icon, flavor),
                follow_up: format!("{} Press [Tab] to view pending challenges", icon),
            });
        }
    }

    // ── 3. Sync player max HP with cached derived stats ─────────
    if state.derived_stats_dirty {
        state.recalculate_derived_stats(&enhancement.levels);
        state.recalculate_prestige_bonuses();
    }
    let derived = state.cached_derived_stats;
    state.combat_state.update_max_hp(derived.max_hp);

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
    let prestige_combat = state.cached_prestige_bonuses;
    let combat_bonuses = CombatBonuses {
        // Haven bonuses
        hp_regen_percent: haven_bonuses.hp_regen_percent,
        hp_regen_delay_reduction: haven_bonuses.hp_regen_delay_reduction,
        damage_percent: haven_bonuses.damage_percent + sigil_bonuses.damage_percent,
        crit_chance_percent: haven_bonuses.crit_chance_percent
            + sigil_bonuses.crit_chance_percent
            + prestige_combat.crit_chance,
        double_strike_chance: haven_bonuses.double_strike_chance
            + sigil_bonuses.double_strike_percent,
        xp_gain_percent: haven_bonuses.xp_gain_percent + sigil_bonuses.xp_percent,
        // God item bonuses
        early_damage_percent: crate::god_items::equipped_god_item_damage_percent(&state.equipment),
        damage_reduction_percent: crate::god_items::equipped_god_item_dr(&state.equipment)
            + sigil_bonuses.damage_reduction_percent,
        attack_speed_percent: crate::god_items::equipped_god_item_attack_speed_percent(
            &state.equipment,
        ) + sigil_bonuses.attack_speed_percent,
        regen_reduction_percent: crate::god_items::equipped_god_item_regen_reduction_percent(
            &state.equipment,
        ) + sigil_bonuses.regen_delay_percent,
        // Prestige flat bonuses
        flat_damage: prestige_combat.flat_damage,
        flat_defense: prestige_combat.flat_defense,
        flat_hp: prestige_combat.flat_hp,
    };
    // Apply flat HP bonus to combat max HP (not in DerivedStats to avoid enemy scaling)
    if combat_bonuses.flat_hp > 0 {
        let boosted_max = derived.max_hp + combat_bonuses.flat_hp;
        state.combat_state.update_max_hp(boosted_max);
    }
    // Apply sigil max HP% bonus on top of current max HP
    if sigil_bonuses.max_hp_percent > 0.0 {
        let current_max = state.combat_state.player_max_hp;
        let boosted = (current_max as f64 * (1.0 + sigil_bonuses.max_hp_percent / 100.0)) as u32;
        state.combat_state.update_max_hp(boosted);
    }
    let combat_events = update_combat(
        rng,
        state,
        delta_time,
        &combat_bonuses,
        achievements,
        &derived,
    );

    tick_stages::process_combat_events(
        state,
        combat_events,
        &haven_bonuses,
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
    *tick_counter += 1;
    if *tick_counter >= TICKS_PER_SECOND {
        state.play_time_seconds += 1;
        if state.combat_seconds_this_tick {
            state.xp_rate_samples.push_back(state.xp_this_second);
            if state.xp_rate_samples.len() > crate::core::constants::XP_RATE_WINDOW_SECONDS {
                state.xp_rate_samples.pop_front();
            }
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
        *tick_counter = 0;
    }

    // ── 9. Collect achievement notifications ────────────────────
    tick_stages::collect_achievement_events(achievements, &mut result);

    // ── 10. Haven discovery check ────────────────────────────────
    // Independent roll per tick, only when eligible (P10+, no active content)
    if !haven.discovered
        && state.prestige_rank >= HAVEN_MIN_PRESTIGE_RANK
        && state.active_dungeon.is_none()
        && state.active_fishing.is_none()
        && state.active_minigame.is_none()
        && crate::haven::try_discover_haven(haven, state.prestige_rank, rng)
    {
        // Track Haven discovery achievement
        achievements.on_haven_discovered(Some(&state.character_name));
        result.events.push(TickEvent::HavenDiscovered);
        result.haven_changed = true;
        if !debug_mode {
            result.achievements_changed = true;
        }
    }

    // ── 11. Soulforge discovery check ────────────────────────────
    // Independent roll per tick, only when eligible (P15+, no active content)
    if !enhancement.discovered
        && state.prestige_rank >= crate::enhancement::SOULFORGE_MIN_PRESTIGE_RANK
        && state.active_dungeon.is_none()
        && state.active_fishing.is_none()
        && state.active_minigame.is_none()
        && crate::enhancement::try_discover_soulforge(enhancement, state.prestige_rank, rng)
    {
        // Track Soulforge discovery achievement
        achievements.on_soulforge_discovered(Some(&state.character_name));
        result.events.push(TickEvent::SoulforgeDiscovered);
        result.enhancement_changed = true;
        if !debug_mode {
            result.achievements_changed = true;
        }
    }

    // ── 11b. Deep discovery ───────────────────────────────────────
    // Discovery is triggered by defeating The Expanse cycle boss.
    // No per-tick random roll.

    // ── 11c. Deep mission ticking ──────────────────────────────────
    // Tick active missions (wall-clock based), resolve completions,
    // and fire achievement handlers.
    if deep.persistent.discovered {
        let now = chrono::Utc::now();
        let pending_before = deep.prestige.pending_results.len();
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
            achievements.on_deep_mission_complete(Some(&state.character_name));
        }
        for layer in &summary.breakthroughs {
            achievements.on_deep_breakthrough(*layer, Some(&state.character_name));
        }
        for _ in 0..summary.mercs_lost {
            achievements.on_deep_merc_lost(Some(&state.character_name));
        }
        if summary.gateway_opened {
            achievements.on_deep_gateway_opened(Some(&state.character_name));
        }

        if (summary.missions_completed > 0 || summary.mercs_lost > 0) && !debug_mode {
            result.achievements_changed = true;
        }

        // Emit tick events for newly completed missions
        for pending in deep.prestige.pending_results.iter().skip(pending_before) {
            if let Some(ref res) = pending.result {
                let outcome_str = match res.outcome {
                    crate::deep::MissionOutcome::Success => "Success",
                    crate::deep::MissionOutcome::PartialSuccess => "Partial Success",
                    crate::deep::MissionOutcome::Failure => "Failure",
                };
                result.events.push(TickEvent::DeepMissionComplete {
                    message: format!(
                        "\u{1F4DC} Mission complete: {} ({})",
                        pending.mission_type.display_name(),
                        outcome_str
                    ),
                });
            }
        }

        // Check whether the mission pool needs a 6-hour refresh.
        if crate::deep::missions::maybe_refresh_mission_pool(
            &mut deep.prestige,
            &deep.persistent,
            now,
            rng,
        ) {
            result.deep_changed = true;
        }
    }

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
