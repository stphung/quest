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
