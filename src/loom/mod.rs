pub mod discovery;
pub mod logic;
pub mod patterns;
pub mod persistence;
pub mod recipes;
pub mod types;

pub use discovery::complete_discovery;
#[allow(unused_imports)]
pub use logic::{
    archetype_nodes, node_conversion_multiplier, node_effective_rate, node_level_multiplier,
    node_native_resource, node_neighbor_unlock_count, node_neighbor_unlock_speed_multiplier,
    node_neighbors, node_throughput_multiplier, node_upgrade_cost, node_upgrade_cost_multiplier,
    resonance_early_feedback_active, select_archetype, tick_base_production,
    tick_loom_staggered_unlock, tick_neighbor_unlocking, try_upgrade_node,
    SECOND_NODE_UNLOCK_SECONDS,
};
#[allow(unused_imports)]
pub use patterns::{
    active_pattern_requirement_status, active_pattern_requirements_met, all_patterns_complete,
    tick_pattern_sustain,
};
#[allow(unused_imports)]
pub use persistence::{load_loom, loom_save_path, save_loom};
#[allow(unused_imports)]
pub use types::{
    CodexEntry, LoomArchetype, LoomNode, LoomPersistent, LoomState, LoomUiState, LoomView, NodeId,
    NodeNature, PatternRequirement, Pipe, PipeTier, Resource, WovenPattern,
};
