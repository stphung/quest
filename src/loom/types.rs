use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Which archetype the player chose at Loom unlock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoomArchetype {
    BurnBright, // Ember Spindle + Void Condenser
    ReachWide,  // Reflection Lens + Memory Archive
    RunDeep,    // Silence Well + Resonance Forge
}

/// The six node identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeId {
    EmberSpindle,
    ReflectionLens,
    VoidCondenser,
    MemoryArchive,
    SilenceWell,
    ResonanceForge,
}

impl NodeId {
    pub const ALL: [NodeId; 6] = [
        NodeId::EmberSpindle,
        NodeId::ReflectionLens,
        NodeId::VoidCondenser,
        NodeId::MemoryArchive,
        NodeId::SilenceWell,
        NodeId::ResonanceForge,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            NodeId::EmberSpindle => "Ember Spindle",
            NodeId::ReflectionLens => "Reflection Lens",
            NodeId::VoidCondenser => "Void Condenser",
            NodeId::MemoryArchive => "Memory Archive",
            NodeId::SilenceWell => "Silence Well",
            NodeId::ResonanceForge => "Resonance Forge",
        }
    }

    pub fn nature(&self) -> NodeNature {
        match self {
            NodeId::EmberSpindle => NodeNature::Heat,
            NodeId::ReflectionLens => NodeNature::Form,
            NodeId::VoidCondenser => NodeNature::Void,
            NodeId::MemoryArchive => NodeNature::Pattern,
            NodeId::SilenceWell => NodeNature::Stillness,
            NodeId::ResonanceForge => NodeNature::Vibration,
        }
    }
}

/// Node natures — the hidden ingredient in combinatorial recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeNature {
    Heat,
    Form,
    Void,
    Pattern,
    Stillness,
    Vibration,
}

/// The six base resources, each tied to a node, plus confluence and reaction products.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    // Base resources
    Ember,
    Reflection,
    VoidEssence,
    Memory,
    Silence,
    Resonance,
    // Confluence resources
    ForgedLight,
    EchoGlass,
    StillbornSong,
    // Reaction products
    CondensedEmber,
    EmberEcho,
    PurifiedVoid,
    WovenReality,
}

/// A single processing node in the Loom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomNode {
    pub id: NodeId,
    #[serde(default = "default_node_level")]
    pub level: u32,
    #[serde(default)]
    pub unlocked: bool,
    #[serde(default)]
    pub buffer: f64,
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    #[serde(default = "default_base_rate")]
    pub base_rate: f64,
    #[serde(default)]
    pub stalled: bool,
}

fn default_node_level() -> u32 {
    1
}

fn default_buffer_capacity() -> f64 {
    20.0
}

fn default_base_rate() -> f64 {
    5.0
}

impl LoomNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            level: 1,
            unlocked: false,
            buffer: 0.0,
            buffer_capacity: 20.0, // 4 hours at 5/hr base
            base_rate: 5.0,
            stalled: false,
        }
    }
}

/// Pipe bandwidth tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipeTier {
    T1, // 5/hr
    T2, // 12/hr
    T3, // 25/hr
    T4, // 50/hr
}

impl PipeTier {
    pub fn bandwidth(&self) -> f64 {
        match self {
            PipeTier::T1 => 5.0,
            PipeTier::T2 => 12.0,
            PipeTier::T3 => 25.0,
            PipeTier::T4 => 50.0,
        }
    }
}

/// A directional pipe between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipe {
    pub from: NodeId,
    pub to: NodeId,
    pub tier: PipeTier,
    /// What fraction of the source node's output goes through this pipe (0.0-1.0).
    pub split_ratio: f64,
    #[serde(default)]
    pub under_construction: bool,
    #[serde(default)]
    pub construction_ticks_remaining: u32,
}

/// A discovered recipe in the codex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub inputs: Vec<Resource>,
    pub node_nature: NodeNature,
    pub output: Resource,
    pub output_amount: f64,
    #[serde(default)]
    pub discovered: bool,
}

/// Woven Pattern — a progression milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WovenPattern {
    pub index: u32,
    pub name: String,
    pub requirements: Vec<PatternRequirement>,
    pub sustain_seconds: u32,
    #[serde(default)]
    pub sustained_seconds: u32,
    #[serde(default)]
    pub completed: bool,
}

/// A single requirement within a woven pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequirement {
    pub resource: Resource,
    pub rate_per_hour: f64,
}

/// All persistent Loom state (saved to loom.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomPersistent {
    #[serde(default)]
    pub discovered: bool,
    #[serde(default)]
    pub archetype: Option<LoomArchetype>,
    #[serde(default = "default_nodes")]
    pub nodes: Vec<LoomNode>,
    #[serde(default)]
    pub pipes: Vec<Pipe>,
    #[serde(default)]
    pub codex: Vec<CodexEntry>,
    #[serde(default)]
    pub active_pattern: usize,
    #[serde(default)]
    pub patterns: Vec<WovenPattern>,
    #[serde(default)]
    pub stockpiles: HashMap<Resource, f64>,
    #[serde(default)]
    pub second_node_unlock_elapsed: Option<f64>,
}

fn default_nodes() -> Vec<LoomNode> {
    NodeId::ALL.iter().map(|&id| LoomNode::new(id)).collect()
}

impl Default for LoomPersistent {
    fn default() -> Self {
        Self {
            discovered: false,
            archetype: None,
            nodes: default_nodes(),
            pipes: Vec::new(),
            codex: Vec::new(),
            active_pattern: 0,
            patterns: Vec::new(),
            stockpiles: HashMap::new(),
            second_node_unlock_elapsed: None,
        }
    }
}

/// Top-level Loom state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomState {
    pub persistent: LoomPersistent,
}

impl LoomState {
    pub fn new() -> Self {
        Self {
            persistent: LoomPersistent::default(),
        }
    }
}

impl Default for LoomState {
    fn default() -> Self {
        Self::new()
    }
}

/// Which view the Loom UI is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoomView {
    ArchetypeSelection,
    FlowView,
    ListDetail,
    Codex,
}

/// Runtime-only UI state (not serialized).
#[derive(Debug)]
pub struct LoomUiState {
    pub open: bool,
    pub view: LoomView,
    pub selected_node: usize,
    pub selected_pipe: usize,
    pub selected_archetype: usize,
}

impl LoomUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            view: LoomView::FlowView,
            selected_node: 0,
            selected_pipe: 0,
            selected_archetype: 0,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }
}

impl Default for LoomUiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loom_state_default() {
        let state = LoomState::new();
        assert!(!state.persistent.discovered);
        assert_eq!(state.persistent.nodes.len(), 6);
        assert!(state.persistent.pipes.is_empty());
        assert!(state.persistent.codex.is_empty());
        assert_eq!(state.persistent.active_pattern, 0);
    }
}
