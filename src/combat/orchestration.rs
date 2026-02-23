use crate::character::derived_stats::DerivedStats;
use crate::core::constants::*;
use crate::core::game_state::GameState;
use rand::Rng;

use super::attacks::effective_enemy_attack_interval;
use super::events::{CombatBonuses, CombatEvent};

/// Updates combat state, returns events that occurred
/// `bonuses` contains all combined combat bonuses from Haven, god items, prestige, and sigils.
/// `achievements` is used to check for Stormbreaker achievement (Zone 10 boss)
/// `derived` contains pre-computed derived stats (avoids redundant recalculation)
pub fn update_combat<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // Handle regeneration after enemy death
    if state.combat_state.is_regenerating {
        return super::regen::process_regen(state, delta_time, bonuses, derived);
    }

    // No combat if no enemy
    if state.combat_state.current_enemy.is_none() {
        return events;
    }

    // --- Phase 1: Accumulate timers ---
    state.combat_state.player_attack_timer += delta_time;
    state.combat_state.enemy_attack_timer += delta_time;

    // --- Phase 1b: Boss enrage timer ---
    if state.zone_progression.fighting_boss {
        state.combat_state.boss_fight_timer += delta_time;
        if state.combat_state.boss_fight_timer >= BOSS_ENRAGE_SECONDS {
            events.extend(resolve_boss_enrage(state, achievements));
            return events;
        }
    }

    // --- Phase 1c: Mob fight timeout ---
    if !state.zone_progression.fighting_boss {
        state.combat_state.current_fight_elapsed += delta_time;
        if state.combat_state.current_fight_elapsed >= MOB_FIGHT_TIMEOUT_SECONDS {
            events.extend(resolve_combat_retreat(state));
            return events;
        }
    }

    // Attack speed multiplier: higher = faster attacks
    let player_interval = ATTACK_INTERVAL_SECONDS
        / (derived.attack_speed_multiplier + bonuses.attack_speed_percent / 100.0);
    let enemy_interval = effective_enemy_attack_interval(state);

    // --- Phase 2: Determine who attacks this tick ---
    let player_attacks = state.combat_state.player_attack_timer >= player_interval;
    let enemy_attacks = state.combat_state.enemy_attack_timer >= enemy_interval;

    // --- Phase 3: Player attack (if ready) ---
    if player_attacks {
        let (attack_events, enemy_died) =
            super::player_attack::resolve_player_attack(rng, state, bonuses, achievements, derived);
        events.extend(attack_events);
        if enemy_died {
            return events;
        }
    }

    // --- Phase 4: Enemy attack (if ready) ---
    if enemy_attacks {
        let attack_events =
            super::enemy_attack::resolve_enemy_attack(rng, state, bonuses, achievements, derived);
        events.extend(attack_events);
    }

    events
}

/// Resolves combat retreat: player is overwhelmed and retreats to the last safe zone.
///
/// Called when mob fight timeout or death loop threshold is reached.
/// Finds the highest zone with a defeated boss and travels there.
pub fn resolve_combat_retreat(state: &mut GameState) -> Vec<CombatEvent> {
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

/// Resolves boss enrage: instant kills the player and resets combat state.
///
/// Always retreats the player to subzone 1 of the current zone.
/// For weapon-gated bosses this prevents an infinite loop; for normal bosses
/// it signals the player isn't strong enough yet.
fn resolve_boss_enrage(
    state: &mut GameState,
    achievements: &crate::achievements::Achievements,
) -> Vec<CombatEvent> {
    let weapon_blocked = state
        .zone_progression
        .boss_weapon_blocked(achievements)
        .is_some();
    let enemy_name = state
        .combat_state
        .current_enemy
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "Boss".to_string());

    // Reset combat state
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 0.0;
    state.combat_state.boss_fight_timer = 0.0;
    state.combat_state.current_enemy = None;

    // Retreat to first subzone of current zone
    state.zone_progression.fighting_boss = false;
    state.zone_progression.kills_in_subzone = 0;
    state.zone_progression.current_subzone_id = 1;

    vec![CombatEvent::BossEnrage {
        weapon_blocked,
        enemy_name,
    }]
}
