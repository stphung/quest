pub mod discovery;
pub mod logic;
pub mod patterns;
pub mod persistence;
pub mod pipes;
pub mod recipes;
pub mod types;

pub use discovery::complete_discovery;
#[allow(unused_imports)]
pub use logic::{
    archetype_nodes, check_node_stall, codex_hint_indices, effective_buffer_capacity,
    effective_node_base_rate, effective_pipe_bandwidth, loom_external_bonuses,
    loom_production_bonus, node_conversion_multiplier, node_effective_rate, node_level_multiplier,
    node_native_resource, node_neighbor_unlock_count, node_neighbor_unlock_speed_multiplier,
    node_neighbors, node_throughput_multiplier, node_upgrade_cost, node_upgrade_cost_multiplier,
    process_reactions, resonance_early_feedback_active, select_archetype, tick_base_production,
    tick_loom_staggered_unlock, tick_neighbor_unlocking, tick_stall_detection, try_upgrade_node,
    LoomExternalBonuses, SECOND_NODE_UNLOCK_SECONDS,
};
#[allow(unused_imports)]
pub use patterns::{
    active_pattern_requirement_status, active_pattern_requirements_met, all_patterns_complete,
    tick_pattern_sustain,
};
#[allow(unused_imports)]
pub use persistence::{load_loom, loom_save_path, save_loom};
#[allow(unused_imports)]
pub use pipes::{
    adjust_split_ratio, build_pipe, demolish_pipe, incoming_pipe_count, normalize_split_ratios,
    outgoing_pipe_count, pipe_exists, pipe_flow_rate, tick_pipe_construction, tick_pipe_flow,
    total_split_ratio, upgrade_pipe, PipeError, PIPE_BUILD_COST_T1, PIPE_CONSTRUCTION_TICKS,
    PIPE_UPGRADE_COSTS,
};
#[allow(unused_imports)]
pub use types::{
    CodexEntry, LoomArchetype, LoomNode, LoomNodeRef, LoomPersistent, LoomState, LoomUiState,
    LoomView, NodeId, NodeNature, PatternRequirement, Pipe, PipeTier, Refinery, Resource,
    WovenPattern,
};
