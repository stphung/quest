//! Shared helpers for combat submodule / damage pipeline tests.

use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::combat::events::{CombatBonuses, CombatEvent};
use quest::combat::logic::update_combat;
use quest::combat::types::Enemy;
use quest::core::constants::*;
use quest::core::game_state::GameState;

/// Creates a fresh, default `GameState` for combat tests.
pub fn fresh_state() -> GameState {
    GameState::new("CombatTest".to_string(), 0)
}

/// Computes derived stats for a state (no set bonuses).
pub fn derived(state: &GameState) -> DerivedStats {
    DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7])
}

/// Default (all-zero) combat bonuses.
pub fn default_bonuses() -> CombatBonuses {
    CombatBonuses::default()
}

/// Builds a seeded RNG for deterministic combat rolls.
pub fn seeded_rng(seed: u64) -> rand_chacha::ChaCha8Rng {
    use rand::SeedableRng;
    rand_chacha::ChaCha8Rng::seed_from_u64(seed)
}

/// Creates a state with an enemy set up for combat.
pub fn state_with_enemy(hp: u64, dmg: u64, def: u64) -> GameState {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(Enemy::new_with_defense(
        "Test Enemy".to_string(),
        hp,
        dmg,
        def,
    ));
    state
}

/// Force a single player attack. Sets player timer to threshold, suppresses enemy timer.
pub fn force_player_attack(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = 0.0;
    let mut achievements = Achievements::default();
    update_combat(rng, state, 0.0, bonuses, &mut achievements, &d, 11, 30)
}

/// Force a single enemy attack. Sets enemy timer to threshold, suppresses player timer.
pub fn force_enemy_attack(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
    let mut achievements = Achievements::default();
    update_combat(rng, state, 0.0, bonuses, &mut achievements, &d, 11, 30)
}
