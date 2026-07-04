// Re-export offline progression types for backwards compatibility
pub use super::offline::{calculate_offline_xp, process_offline_progression, OfflineReport};

// Re-export XP system types for backward compatibility
pub use super::xp::{
    apply_tick_xp, combat_kill_xp, distribute_level_up_points, prestige_multiplier,
    process_level_ups_from_current_xp, xp_for_next_level, xp_gain_per_tick,
};

// Re-export enemy spawning for backward compatibility
pub use super::enemy_spawning::spawn_enemy_if_needed;

// Re-export dungeon discovery for backward compatibility
pub use super::discoveries::try_discover_dungeon;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_state::GameState;
    use rand::SeedableRng;

    /// Verify that all re-exports from game_logic.rs resolve correctly and
    /// actually execute (not just type-check) so their bodies register as
    /// covered wherever they're invoked via the game_logic:: path.
    #[test]
    fn test_reexports_resolve() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);

        // XP system
        let _xp = xp_for_next_level(1);
        let _mult = prestige_multiplier(0, 0);
        let _rate = xp_gain_per_tick(0, 0, 0);
        let _kill_xp = combat_kill_xp(&mut rng, 0.0, 0.0);

        let mut state = GameState::new("ReexportTest".to_string(), 0);
        let _attrs = distribute_level_up_points(&mut rng, &mut state);
        let (_levelups, _leveled_attrs) = apply_tick_xp(&mut rng, &mut state, 50000.0);
        let _processed = process_level_ups_from_current_xp(&mut rng, &mut state);

        // Enemy spawning - actually invoke it via the re-exported path.
        spawn_enemy_if_needed(&mut state);
        assert!(state.combat_state.current_enemy.is_some());

        // Dungeon discovery - actually invoke it via the re-exported path.
        let _discovered = try_discover_dungeon(&mut rng, &mut state);

        // Offline progression - actually invoke it and use the returned report.
        let report = process_offline_progression(&mut rng, &mut state, 0.0);
        let _also_calculated = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        fn accepts_report(_r: OfflineReport) {}
        accepts_report(report);
    }
}
