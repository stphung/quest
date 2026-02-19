// Re-export offline progression types for backwards compatibility
pub use super::offline::{calculate_offline_xp, process_offline_progression, OfflineReport};

// Re-export XP system types for backward compatibility
pub use super::xp::{
    apply_tick_xp, combat_kill_xp, distribute_level_up_points, prestige_multiplier,
    xp_for_next_level, xp_gain_per_tick,
};

// Re-export enemy spawning for backward compatibility
pub use super::enemy_spawning::spawn_enemy_if_needed;

// Re-export dungeon discovery for backward compatibility
pub use super::discoveries::try_discover_dungeon;

#[cfg(test)]
mod tests {
    /// Verify that all re-exports from game_logic.rs resolve correctly.
    /// This test ensures backward compatibility after extraction.
    #[test]
    fn test_reexports_resolve() {
        // XP system
        let _xp = super::xp_for_next_level(1);
        let _mult = super::prestige_multiplier(0, 0);
        let _rate = super::xp_gain_per_tick(0, 0, 0);

        // Enemy spawning - just verify the function exists (needs GameState to call)
        let _fn_ptr: fn(&mut crate::core::game_state::GameState) = super::spawn_enemy_if_needed;

        // Dungeon discovery - verify the function exists
        let _fn_ptr: fn(&mut crate::core::game_state::GameState) -> bool =
            super::try_discover_dungeon;

        // Offline progression - verify the type exists
        fn _accepts_report(_r: super::OfflineReport) {}
    }
}
