#![allow(dead_code)]
use crate::achievements::Achievements;
use crate::character::derived_stats::DerivedStats;
use crate::combat::events::{CombatBonuses, CombatEvent};
use crate::combat::types::CombatState;
use crate::core::game_state::GameState;
use rand::Rng;

/// Explicit inputs for the combat update facade.
/// These fields represent the decomposed API surface that combat needs.
/// During migration, the facade still takes `&mut GameState` and delegates
/// to `update_combat()`. Future work: extract `update_combat_core()`.
pub struct CombatInput<'a> {
    pub combat_state: &'a mut CombatState,
    pub bonuses: &'a CombatBonuses,
    pub derived: &'a DerivedStats,
    pub prestige_rank: u32,
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
            &mut rng, &mut state, 0.1, &bonuses, &mut achievements, &derived,
        );
        assert!(events.is_empty(), "No events when no enemy");
    }

    #[test]
    fn test_combat_facade_with_enemy() {
        use crate::combat::enemy_generation::generate_enemy_for_current_zone;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("CombatTest".to_string(), 0);
        let enemy = generate_enemy_for_current_zone(1, 1);
        state.combat_state.current_enemy = Some(enemy);
        state.combat_state.player_current_hp = 100;
        let bonuses = CombatBonuses::default();
        let derived = DerivedStats::default();
        let mut achievements = crate::achievements::Achievements::default();

        // Run enough ticks for an attack to happen
        for _ in 0..20 {
            let _events = update_combat_facade(
                &mut rng, &mut state, 0.1, &bonuses, &mut achievements, &derived,
            );
        }
        // Should have processed some combat -- just verify no panic
    }
}
