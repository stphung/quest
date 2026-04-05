pub mod discovery;
pub mod graph;
pub mod layout;
pub mod logic;
pub mod milestones;
pub mod patterns;
pub mod persistence;
pub mod recipes;
pub mod types;

pub use discovery::complete_discovery;
#[allow(unused_imports)]
pub use logic::{
    archetype_nodes, build_shuttle, check_node_stall, codex_hint_indices, demolish_shuttle,
    effective_buffer_capacity, effective_node_base_rate, eligible_sources_for_tier,
    initialize_loom, loom_external_bonuses, loom_production_bonus, loom_zone_cap_for_patterns,
    node_conversion_multiplier, node_effective_rate, node_level_multiplier, node_native_resource,
    node_neighbor_unlock_count, node_neighbor_unlock_speed_multiplier, node_neighbors,
    node_throughput_multiplier, node_upgrade_cost, node_upgrade_cost_multiplier,
    node_upgrade_duration, resonance_early_feedback_active, select_archetype,
    shuttle_build_cost_public, shuttle_effective_intake_cap, tick_base_production,
    tick_loom_staggered_unlock, tick_neighbor_unlocking, tick_node_upgrades,
    tick_shuttle_construction, tick_shuttle_pull, tick_shuttle_stall_detection,
    tick_stall_detection, try_upgrade_node, unlocked_tiers, upgrade_shuttle, wr_to_pr_per_day,
    LoomExternalBonuses, ShuttleError, ShuttleUpgradeError, MAX_NODE_LEVEL,
    SECOND_NODE_UNLOCK_SECONDS, SHUTTLE_CONSTRUCTION_TICKS,
};
pub use milestones::PatternMilestone;
#[allow(unused_imports)]
pub use patterns::{
    active_pattern_requirement_status, active_pattern_requirements_met, all_patterns_complete,
    tick_pattern_sustain,
};
#[allow(unused_imports)]
pub use persistence::{load_loom, loom_save_path, save_loom};
#[allow(unused_imports)]
pub use types::{
    BuildState, BuildStep, CodexEntry, LoomArchetype, LoomNode, LoomNodeRef, LoomPersistent,
    LoomState, LoomUiState, NodeId, NodeNature, PatternRequirement, RateTracker, Resource, Shuttle,
    WovenPattern, MAX_SHUTTLES,
};
