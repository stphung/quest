use crate::character::derived_stats::DerivedStats;
use crate::core::constants::HP_REGEN_DURATION_SECONDS;
use crate::core::game_state::GameState;

use super::events::{CombatBonuses, CombatEvent};

/// Processes HP regeneration state after an enemy kill.
///
/// Returns any `RegenComplete` events that occurred during this tick.
/// The function advances the regen timer, applies gradual HP restoration,
/// and emits periodic heal floats (every 0.5s).
pub(crate) fn process_regen(
    state: &mut GameState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    derived: &DerivedStats,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // HP regen multiplier: higher = faster regen (equipment + haven bonus)
    let total_regen_multiplier =
        derived.hp_regen_multiplier * (1.0 + bonuses.hp_regen_percent / 100.0);

    // Apply hp_regen_delay_reduction and regen_reduction_percent to base regen duration
    let base_regen_duration = HP_REGEN_DURATION_SECONDS
        * (1.0 - bonuses.hp_regen_delay_reduction / 100.0)
        * (1.0 - bonuses.regen_reduction_percent / 100.0);
    let effective_regen_duration = base_regen_duration / total_regen_multiplier;

    state.combat_state.regen_timer += delta_time;

    if state.combat_state.regen_timer >= effective_regen_duration {
        let healed = state
            .combat_state
            .player_max_hp
            .saturating_sub(state.combat_state.regen_start_hp);
        state.combat_state.player_current_hp = state.combat_state.player_max_hp;
        state.combat_state.is_regenerating = false;
        state.combat_state.regen_timer = 0.0;
        if healed > 0 {
            events.push(CombatEvent::RegenComplete { healed });
        }
    } else {
        // Gradual regen
        let regen_progress = state.combat_state.regen_timer / effective_regen_duration;
        let start_hp = state.combat_state.player_current_hp;
        let target_hp = state.combat_state.player_max_hp;
        state.combat_state.player_current_hp =
            start_hp + ((target_hp - start_hp) as f64 * regen_progress) as u64;

        // Emit periodic heal floats every 0.5s so they align with HP bar fill
        const REGEN_FLOAT_INTERVAL: f64 = 0.5;
        let prev_bucket =
            ((state.combat_state.regen_timer - delta_time) / REGEN_FLOAT_INTERVAL) as u32;
        let curr_bucket = (state.combat_state.regen_timer / REGEN_FLOAT_INTERVAL) as u32;
        if curr_bucket > prev_bucket {
            let healed = state
                .combat_state
                .player_current_hp
                .saturating_sub(state.combat_state.regen_start_hp);
            if healed > 0 {
                events.push(CombatEvent::RegenComplete { healed });
                state.combat_state.regen_start_hp = state.combat_state.player_current_hp;
            }
        }
    }

    events
}
