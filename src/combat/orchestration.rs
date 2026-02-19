use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::core::constants::*;
use crate::core::game_state::GameState;

use super::attacks::effective_enemy_attack_interval;
use super::events::{CombatEvent, GodItemCombatBonuses, HavenCombatBonuses};

/// Updates combat state, returns events that occurred
/// `haven` contains all Haven bonuses that affect combat
/// `prestige_bonuses` contains flat combat bonuses from prestige rank
/// `achievements` is used to check for Stormbreaker achievement (Zone 10 boss)
/// `derived` contains pre-computed derived stats (avoids redundant recalculation)
pub fn update_combat(
    state: &mut GameState,
    delta_time: f64,
    haven: &HavenCombatBonuses,
    prestige_bonuses: &PrestigeCombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
    god_items: &GodItemCombatBonuses,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // Handle regeneration after enemy death
    if state.combat_state.is_regenerating {
        return super::regen::process_regen(state, delta_time, haven, god_items, derived);
    }

    // No combat if no enemy
    if state.combat_state.current_enemy.is_none() {
        return events;
    }

    // --- Phase 1: Accumulate both timers ---
    state.combat_state.player_attack_timer += delta_time;
    state.combat_state.enemy_attack_timer += delta_time;

    // Attack speed multiplier: higher = faster attacks
    let player_interval = ATTACK_INTERVAL_SECONDS
        / (derived.attack_speed_multiplier + god_items.attack_speed_percent / 100.0);
    let enemy_interval = effective_enemy_attack_interval(state);

    // --- Phase 2: Determine who attacks this tick ---
    let player_attacks = state.combat_state.player_attack_timer >= player_interval;
    let enemy_attacks = state.combat_state.enemy_attack_timer >= enemy_interval;

    // --- Phase 3: Player attack (if ready) ---
    if player_attacks {
        let (attack_events, enemy_died) = super::player_attack::resolve_player_attack(
            state,
            haven,
            prestige_bonuses,
            achievements,
            derived,
            god_items,
        );
        events.extend(attack_events);
        if enemy_died {
            return events;
        }
    }

    // --- Phase 4: Enemy attack (if ready) ---
    if enemy_attacks {
        let attack_events = super::enemy_attack::resolve_enemy_attack(
            state,
            haven,
            prestige_bonuses,
            achievements,
            derived,
            god_items,
        );
        events.extend(attack_events);
    }

    events
}
