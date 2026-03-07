pub mod discovery;
pub mod logic;
pub mod patterns;
pub mod persistence;
pub mod recipes;
pub mod types;

pub use discovery::complete_discovery;
#[allow(unused_imports)]
pub use logic::{
    archetype_nodes, node_conversion_multiplier, node_neighbor_unlock_count,
    node_neighbor_unlock_speed_multiplier, node_throughput_multiplier,
    node_upgrade_cost_multiplier, resonance_early_feedback_active, select_archetype,
    tick_loom_staggered_unlock, SECOND_NODE_UNLOCK_SECONDS,
};
#[allow(unused_imports)]
pub use persistence::{load_loom, loom_save_path, save_loom};
#[allow(unused_imports)]
pub use types::{
    CodexEntry, LoomArchetype, LoomNode, LoomPersistent, LoomState, LoomUiState, LoomView, NodeId,
    NodeNature, PatternRequirement, Pipe, PipeTier, Resource, WovenPattern,
};
