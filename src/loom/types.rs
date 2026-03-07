#![allow(dead_code)]
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

/// Unified address for any node in the Loom — either a fixed Extractor or a built Refinery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoomNodeRef {
    /// One of the 6 fixed extractor nodes.
    Extractor(NodeId),
    /// A player-built refinery, identified by index in `LoomPersistent::refineries`.
    Refinery(usize),
}

/// A player-built processing node that runs a single locked recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinery {
    /// First input resource for this refinery's locked recipe.
    pub input_a: Resource,
    /// Second input resource for this refinery's locked recipe.
    pub input_b: Resource,
    /// The nature catalyst for this refinery's recipe.
    pub nature: NodeNature,
    /// Output resource produced.
    pub output: Resource,
    /// Output amount multiplier from the recipe.
    pub amount: f64,
    /// Recipe tier (1, 2, or 3).
    pub tier: u8,
    /// Current buffer level (holds output resource).
    #[serde(default)]
    pub buffer: f64,
    /// Buffer capacity.
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    /// Refinery level (for future upgrades).
    #[serde(default = "default_node_level")]
    pub level: u32,
    /// Whether this refinery is stalled (missing inputs or buffer full).
    #[serde(default)]
    pub stalled: bool,
    /// Whether currently under construction.
    #[serde(default)]
    pub under_construction: bool,
    /// Ticks remaining for construction.
    #[serde(default)]
    pub construction_ticks_remaining: u32,
}

impl Refinery {
    pub fn new(
        input_a: Resource,
        input_b: Resource,
        nature: NodeNature,
        output: Resource,
        amount: f64,
        tier: u8,
    ) -> Self {
        Self {
            input_a,
            input_b,
            nature,
            output,
            amount,
            tier,
            buffer: 0.0,
            buffer_capacity: 20.0,
            level: 1,
            stalled: false,
            under_construction: false,
            construction_ticks_remaining: 0,
        }
    }
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
    /// Accumulated unlock progress in hours (contributed by unlocked neighbors).
    /// When this reaches the threshold (2.0 hours), the node unlocks.
    #[serde(default)]
    pub unlock_progress: f64,
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
            unlock_progress: 0.0,
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
    pub from: LoomNodeRef,
    pub to: LoomNodeRef,
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
    #[serde(default)]
    pub completed: bool,
}

/// A single requirement within a woven pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequirement {
    pub resource: Resource,
    /// Total amount of this resource needed to complete the pattern.
    #[serde(alias = "rate_per_hour")]
    pub amount: f64,
    /// Accumulated production so far.
    #[serde(default)]
    pub accumulated: f64,
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
    /// Player-built refineries (recipe-locked processing nodes).
    #[serde(default)]
    pub refineries: Vec<Refinery>,
}

impl LoomPersistent {
    /// Maximum number of Refineries the player can build.
    /// Equal to the number of completed Woven Patterns.
    pub fn max_refineries(&self) -> usize {
        self.patterns.iter().filter(|p| p.completed).count()
    }
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
            refineries: Vec::new(),
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
    /// Scroll offset for the Codex view (number of lines scrolled down).
    pub codex_scroll: usize,
}

impl LoomUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            view: LoomView::FlowView,
            selected_node: 0,
            selected_pipe: 0,
            selected_archetype: 0,
            codex_scroll: 0,
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

    #[test]
    fn test_loom_node_ref_equality() {
        let ext_a = LoomNodeRef::Extractor(NodeId::EmberSpindle);
        let ext_b = LoomNodeRef::Extractor(NodeId::EmberSpindle);
        let ref_a = LoomNodeRef::Refinery(0);
        let ref_b = LoomNodeRef::Refinery(0);
        let ref_c = LoomNodeRef::Refinery(1);
        assert_eq!(ext_a, ext_b);
        assert_eq!(ref_a, ref_b);
        assert_ne!(ext_a, ref_a);
        assert_ne!(ref_a, ref_c);
    }

    #[test]
    fn test_refinery_new() {
        let r = Refinery::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
        );
        assert_eq!(r.input_a, Resource::Ember);
        assert_eq!(r.input_b, Resource::VoidEssence);
        assert_eq!(r.nature, NodeNature::Heat);
        assert_eq!(r.output, Resource::ForgedLight);
        assert!((r.amount - 1.0).abs() < 0.001);
        assert_eq!(r.tier, 1);
        assert!(!r.stalled);
        assert!((r.buffer - 0.0).abs() < 0.001);
        assert!((r.buffer_capacity - 20.0).abs() < 0.001);
        assert_eq!(r.level, 1);
    }

    #[test]
    fn test_loom_state_default_has_empty_refineries() {
        let state = LoomState::new();
        assert!(state.persistent.refineries.is_empty());
    }

    #[test]
    fn test_refinery_limit_zero_with_no_patterns() {
        let state = LoomState::new();
        assert_eq!(state.persistent.max_refineries(), 0);
    }

    #[test]
    fn test_pattern_requirement_fields() {
        let req = PatternRequirement {
            resource: Resource::Ember,
            amount: 5.0,
            accumulated: 0.0,
        };
        assert_eq!(req.resource, Resource::Ember);
        assert!((req.amount - 5.0).abs() < 1e-9);
        assert!((req.accumulated - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_woven_pattern_no_timer_fields() {
        let pattern = WovenPattern {
            index: 0,
            name: "Test".to_string(),
            requirements: vec![],
            completed: false,
        };
        assert!(!pattern.completed);
        assert_eq!(pattern.index, 0);
    }
}
