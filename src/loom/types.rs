#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

/// Maximum number of shuttles a player can build (balance cap).
pub const MAX_SHUTTLES: usize = 5;

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

    /// Returns the fixed index (0-5) for direct array access.
    pub fn index(self) -> usize {
        match self {
            NodeId::EmberSpindle => 0,
            NodeId::ReflectionLens => 1,
            NodeId::VoidCondenser => 2,
            NodeId::MemoryArchive => 3,
            NodeId::SilenceWell => 4,
            NodeId::ResonanceForge => 5,
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

/// Unified address for any node in the Loom — either a fixed Extractor or a built Shuttle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoomNodeRef {
    /// One of the 6 fixed extractor nodes.
    Extractor(NodeId),
    /// A player-built shuttle, identified by index in `LoomPersistent::shuttles`.
    Shuttle(usize),
}

/// A player-built processing node that runs a single locked recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shuttle {
    /// First input resource for this shuttle's locked recipe.
    pub input_a: Resource,
    /// Second input resource for this shuttle's locked recipe.
    pub input_b: Resource,
    /// The nature catalyst for this shuttle's recipe.
    pub nature: NodeNature,
    /// Output resource produced.
    pub output: Resource,
    /// Output amount multiplier from the recipe.
    #[serde(default = "default_shuttle_amount")]
    pub amount: f64,
    /// Recipe tier (1, 2, or 3).
    #[serde(default = "default_shuttle_tier")]
    pub tier: u8,
    /// Current buffer level (holds output resource).
    #[serde(default)]
    pub buffer: f64,
    /// Buffer capacity.
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    /// Shuttle level (for future upgrades).
    #[serde(default = "default_node_level")]
    pub level: u32,
    /// Whether this shuttle is stalled (missing inputs or buffer full).
    #[serde(default)]
    pub stalled: bool,
    /// Whether currently under construction.
    #[serde(default)]
    pub under_construction: bool,
    /// Seconds remaining for construction (wall-clock time).
    #[serde(default, alias = "construction_ticks_remaining")]
    pub construction_secs_remaining: f64,
    /// Sources for input A — extractors or lower-tier shuttles.
    #[serde(default)]
    pub sources_a: Vec<LoomNodeRef>,
    /// Sources for input B — extractors or lower-tier shuttles.
    #[serde(default)]
    pub sources_b: Vec<LoomNodeRef>,
    /// Per-shuttle output rate tracker (transient, not serialized).
    #[serde(skip)]
    pub output_rate_tracker: RateTracker,
}

impl Shuttle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_a: Resource,
        input_b: Resource,
        nature: NodeNature,
        output: Resource,
        amount: f64,
        tier: u8,
        sources_a: Vec<LoomNodeRef>,
        sources_b: Vec<LoomNodeRef>,
    ) -> Self {
        Self {
            input_a,
            input_b,
            nature,
            output,
            amount,
            tier,
            buffer: 0.0,
            buffer_capacity: 500.0,
            level: 1,
            stalled: false,
            under_construction: false,
            construction_secs_remaining: 0.0,
            sources_a,
            sources_b,
            output_rate_tracker: RateTracker::new(),
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
    /// Whether the node is currently upgrading (locked out from production).
    #[serde(default)]
    pub upgrading: bool,
    /// Remaining seconds until upgrade completes. Ticked down each game tick.
    #[serde(default)]
    pub upgrade_remaining_secs: f64,
}

fn default_node_level() -> u32 {
    1
}

fn default_buffer_capacity() -> f64 {
    250.0
}

fn default_shuttle_amount() -> f64 {
    1.0
}

fn default_shuttle_tier() -> u8 {
    1
}

fn default_base_rate() -> f64 {
    25.0
}

fn default_time_warp() -> f64 {
    1.0
}

impl LoomNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            level: 1,
            unlocked: false,
            buffer: 0.0,
            buffer_capacity: 250.0, // 10 hours at 25/hr base
            base_rate: 25.0,
            stalled: false,
            unlock_progress: 0.0,
            upgrading: false,
            upgrade_remaining_secs: 0.0,
        }
    }
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
    /// Narrative flavor text displayed when this pattern is active.
    #[serde(default)]
    pub flavor: String,
    /// Eternal patterns never complete — they act as endgame resource sinks.
    #[serde(default)]
    pub eternal: bool,
}

/// A single requirement within a woven pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequirement {
    pub resource: Resource,
    /// Minimum production rate (units/hr) that must be sustained.
    #[serde(default)]
    pub required_rate: f64,
    /// Total seconds the rate must be sustained to complete this requirement.
    #[serde(default)]
    pub sustain_duration_secs: f64,
    /// Seconds sustained so far (timer advances when rate >= threshold, pauses otherwise).
    #[serde(default)]
    pub sustained_secs: f64,
    /// Whether this individual requirement is complete (locks when sustain timer finishes).
    #[serde(default)]
    pub completed: bool,
    /// Legacy field — total amount needed (accumulated totals system). Kept for serde compat.
    #[serde(default, alias = "rate_per_hour")]
    pub amount: f64,
    /// Legacy field — accumulated production so far. Kept for serde compat.
    #[serde(default)]
    pub accumulated: f64,
}

/// Current save format version. Bump when breaking changes require a fresh start.
pub const LOOM_SAVE_VERSION: u32 = 2;

/// All persistent Loom state (saved to loom.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomPersistent {
    /// Save format version. Old saves (version 0/1) are auto-reset on load.
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub discovered: bool,
    #[serde(default)]
    pub archetype: Option<LoomArchetype>,
    #[serde(default = "default_nodes")]
    pub nodes: Vec<LoomNode>,
    #[serde(default)]
    pub codex: Vec<CodexEntry>,
    #[serde(default)]
    pub active_pattern: usize,
    #[serde(default)]
    pub patterns: Vec<WovenPattern>,
    /// Legacy field — previously held global resource stockpiles. Kept for serde compat.
    #[serde(default, skip_serializing)]
    pub _stockpiles_legacy: HashMap<Resource, f64>,
    #[serde(default)]
    pub second_node_unlock_elapsed: Option<f64>,
    /// Player-built shuttles (recipe-locked processing nodes).
    #[serde(default)]
    pub shuttles: Vec<Shuttle>,
    /// Unix timestamp of last WR→PR grant (wall-clock, like Power Cores).
    #[serde(default)]
    pub wr_pr_last_granted_at: i64,
    /// Pattern milestones awaiting consumption by the tick pipeline.
    /// Set by offline resolution or tick_stages; consumed by tick.rs to emit events.
    #[serde(skip)]
    pub pending_pattern_milestones: Vec<super::milestones::PatternMilestone>,
}

impl LoomPersistent {
    /// Total number of Woven Patterns (excludes eternal patterns).
    pub fn woven_pattern_count(&self) -> usize {
        self.patterns.iter().filter(|p| !p.eternal).count()
    }

    /// Number of completed Woven Patterns (excludes eternal patterns).
    pub fn completed_pattern_count(&self) -> usize {
        self.patterns
            .iter()
            .filter(|p| p.completed && !p.eternal)
            .count()
    }

    /// Maximum number of Shuttles the player can build.
    /// Scales with completed Woven Patterns up to MAX_SHUTTLES.
    pub fn max_shuttles(&self) -> usize {
        let patterns = self.completed_pattern_count();
        let slots = match patterns {
            0 => 0,
            1..=3 => 1,
            4..=7 => 2,
            8..=11 => 3,
            12..=14 => 4,
            _ => MAX_SHUTTLES, // 5 at 15+ patterns
        };
        slots.min(MAX_SHUTTLES)
    }
}

fn default_nodes() -> Vec<LoomNode> {
    NodeId::ALL.iter().map(|&id| LoomNode::new(id)).collect()
}

impl Default for LoomPersistent {
    fn default() -> Self {
        Self {
            version: LOOM_SAVE_VERSION,
            discovered: false,
            archetype: None,
            nodes: default_nodes(),
            codex: Vec::new(),
            active_pattern: 0,
            patterns: Vec::new(),
            _stockpiles_legacy: HashMap::new(),
            second_node_unlock_elapsed: None,
            shuttles: Vec::new(),
            wr_pr_last_granted_at: 0,
            pending_pattern_milestones: Vec::new(),
        }
    }
}

/// Top-level Loom state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomState {
    pub persistent: LoomPersistent,
    /// Per-resource rolling rate trackers (transient, not serialized).
    #[serde(skip)]
    pub rate_trackers: HashMap<Resource, RateTracker>,
    /// Debug time warp multiplier (1.0 = normal, 10/100/1000 = accelerated). Not saved.
    #[serde(skip, default = "default_time_warp")]
    pub time_warp: f64,
    /// Signals the UI to rebuild the graph layout (set by tick-path logic, consumed by UI).
    #[serde(skip)]
    pub graph_dirty: bool,
    /// Wall-clock timestamp of the last Loom tick, for computing real elapsed time.
    /// Transient — initialized on first tick after load.
    #[serde(skip)]
    pub last_tick_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl LoomState {
    pub fn new() -> Self {
        Self {
            persistent: LoomPersistent::default(),
            rate_trackers: HashMap::new(),
            time_warp: 1.0,
            graph_dirty: false,
            last_tick_at: None,
        }
    }
}

impl Default for LoomState {
    fn default() -> Self {
        Self::new()
    }
}

/// Which step of the shuttle build flow the player is on.
#[derive(Debug, Clone)]
pub enum BuildStep {
    /// Selecting a recipe from the filtered list. `cursor` indexes into the available recipes.
    SelectRecipe { cursor: usize },
    /// Selecting sources for input A. `toggle[i]` = whether source i is selected.
    SelectSourcesA { cursor: usize, toggle: Vec<bool> },
    /// Selecting sources for input B.
    SelectSourcesB { cursor: usize, toggle: Vec<bool> },
    /// Confirm build — shows summary and expected throughput.
    Confirm,
    /// Build is blocked — shows why (e.g., need more patterns).
    Blocked { message: String },
}

/// State for the multi-step shuttle build flow.
#[derive(Debug, Clone)]
pub struct BuildState {
    pub step: BuildStep,
    pub tier: u8,
    /// Index into `all_recipes()` for the selected recipe.
    pub recipe_index: usize,
    /// Available recipes for current tier (indices into all_recipes()).
    pub available_recipes: Vec<usize>,
    /// Eligible source nodes for input A.
    pub eligible_sources_a: Vec<LoomNodeRef>,
    /// Eligible source nodes for input B.
    pub eligible_sources_b: Vec<LoomNodeRef>,
    /// Selected sources for input A (populated after SelectSourcesA step).
    pub selected_sources_a: Vec<LoomNodeRef>,
    /// Selected sources for input B (populated after SelectSourcesB step).
    pub selected_sources_b: Vec<LoomNodeRef>,
}

/// Runtime-only UI state (not serialized).
#[derive(Debug)]
pub struct LoomUiState {
    pub open: bool,
    /// Currently selected node in the graph view (None = no selection).
    pub selected_graph_node: Option<petgraph::stable_graph::NodeIndex>,
    /// Per-edge animation phase for flowing particle effects.
    pub particle_phases: HashMap<petgraph::stable_graph::EdgeIndex, f64>,
    /// Frame counter for throbber animation (incremented each render call).
    pub throbber_frame: u32,
    /// Active build flow state, if any.
    pub build: Option<BuildState>,
    /// Cached production graph (rebuilt when graph_dirty flag is set).
    pub loom_graph: Option<super::graph::LoomGraph>,
    /// Cached layout for the production graph.
    pub loom_layout: Option<super::layout::LoomLayout>,
    /// UI-side dirty flag for graph rebuild (e.g., after window resize).
    pub graph_dirty: bool,
    /// True when waiting for second D press to confirm shuttle demolish.
    pub demolish_pending: bool,
}

impl LoomUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_graph_node: None,
            particle_phases: HashMap::new(),
            throbber_frame: 0,
            build: None,
            loom_graph: None,
            loom_layout: None,
            graph_dirty: false,
            demolish_pending: false,
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

/// Rolling window rate tracker for measuring resource production over 20 seconds.
///
/// Uses a circular buffer of 200 ticks (at 100ms/tick = 20 seconds).
/// The running sum gives O(1) per-tick updates. Not serialized — on load,
/// it starts empty and ramps up over 20 seconds.
const RATE_WINDOW_SIZE: usize = 200;
const TICKS_PER_HOUR: f64 = 36_000.0;

#[derive(Debug, Clone)]
pub struct RateTracker {
    buffer: VecDeque<f64>,
    sum: f64,
}

impl RateTracker {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            sum: 0.0,
        }
    }

    /// Push one tick's production amount, evicting the oldest value if the window is full.
    pub fn push(&mut self, amount: f64) {
        if self.buffer.len() >= RATE_WINDOW_SIZE {
            if let Some(old) = self.buffer.pop_front() {
                self.sum -= old;
            }
        }
        self.buffer.push_back(amount);
        self.sum += amount;
    }

    /// Returns the estimated production rate per hour based on the rolling window.
    /// Divides by actual sample count (not window capacity) to avoid suppressed rates
    /// during cold start / ramp-up.
    pub fn rate_per_hour(&self) -> f64 {
        let count = self.buffer.len();
        if count == 0 {
            return 0.0;
        }
        (self.sum / count as f64) * TICKS_PER_HOUR
    }
}

impl Default for RateTracker {
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
        assert!(state.persistent.codex.is_empty());
        assert_eq!(state.persistent.active_pattern, 0);
    }

    #[test]
    fn test_loom_node_ref_equality() {
        let ext_a = LoomNodeRef::Extractor(NodeId::EmberSpindle);
        let ext_b = LoomNodeRef::Extractor(NodeId::EmberSpindle);
        let ref_a = LoomNodeRef::Shuttle(0);
        let ref_b = LoomNodeRef::Shuttle(0);
        let ref_c = LoomNodeRef::Shuttle(1);
        assert_eq!(ext_a, ext_b);
        assert_eq!(ref_a, ref_b);
        assert_ne!(ext_a, ref_a);
        assert_ne!(ref_a, ref_c);
    }

    #[test]
    fn test_shuttle_new() {
        let r = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        assert_eq!(r.input_a, Resource::Ember);
        assert_eq!(r.input_b, Resource::VoidEssence);
        assert_eq!(r.nature, NodeNature::Heat);
        assert_eq!(r.output, Resource::ForgedLight);
        assert!((r.amount - 1.0).abs() < 0.001);
        assert_eq!(r.tier, 1);
        assert!(!r.stalled);
        assert!((r.buffer - 0.0).abs() < 0.001);
        assert!((r.buffer_capacity - 500.0).abs() < 0.001);
        assert_eq!(r.level, 1);
        assert_eq!(r.sources_a.len(), 1);
        assert_eq!(r.sources_b.len(), 1);
    }

    #[test]
    fn test_loom_state_default_has_empty_shuttles() {
        let state = LoomState::new();
        assert!(state.persistent.shuttles.is_empty());
    }

    #[test]
    fn test_shuttle_limit_zero_with_no_patterns() {
        let state = LoomState::new();
        assert_eq!(state.persistent.max_shuttles(), 0);
    }

    #[test]
    fn test_pattern_requirement_fields() {
        let req = PatternRequirement {
            resource: Resource::Ember,
            required_rate: 0.0,
            sustain_duration_secs: 0.0,
            sustained_secs: 0.0,
            completed: false,
            amount: 5.0,
            accumulated: 0.0,
        };
        assert_eq!(req.resource, Resource::Ember);
        assert!((req.amount - 5.0).abs() < 1e-9);
        assert!((req.accumulated - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_pattern_requirement_rate_fields() {
        let req = PatternRequirement {
            resource: Resource::Ember,
            required_rate: 25.0,
            sustain_duration_secs: 7200.0,
            sustained_secs: 0.0,
            completed: false,
            amount: 0.0,
            accumulated: 0.0,
        };
        assert_eq!(req.resource, Resource::Ember);
        assert!((req.required_rate - 25.0).abs() < 1e-9);
        assert!((req.sustain_duration_secs - 7200.0).abs() < 1e-9);
        assert!((req.sustained_secs).abs() < 1e-9);
        assert!(!req.completed);
    }

    #[test]
    fn test_woven_pattern_no_timer_fields() {
        let pattern = WovenPattern {
            index: 0,
            name: "Test".to_string(),
            requirements: vec![],
            completed: false,
            flavor: String::new(),
            eternal: false,
        };
        assert!(!pattern.completed);
        assert_eq!(pattern.index, 0);
    }

    #[test]
    fn test_completed_pattern_count_empty() {
        let state = LoomState::new();
        assert_eq!(state.persistent.completed_pattern_count(), 0);
    }

    #[test]
    fn test_completed_pattern_count_some_completed() {
        let mut state = LoomState::new();
        state.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "A".to_string(),
            requirements: vec![],
            completed: true,
            flavor: String::new(),
            eternal: false,
        });
        state.persistent.patterns.push(WovenPattern {
            index: 1,
            name: "B".to_string(),
            requirements: vec![],
            completed: false,
            flavor: String::new(),
            eternal: false,
        });
        assert_eq!(state.persistent.completed_pattern_count(), 1);
    }

    #[test]
    fn test_rate_tracker_new_is_empty() {
        let tracker = RateTracker::new();
        assert!((tracker.rate_per_hour()).abs() < 1e-9);
    }

    #[test]
    fn test_rate_tracker_push_single_value() {
        let mut tracker = RateTracker::new();
        tracker.push(1.0);
        let rate = tracker.rate_per_hour();
        // 1 sample of 1.0 unit per tick → 1.0 * 36000 ticks/hr = 36000/hr
        assert!((rate - 36000.0).abs() < 1e-6, "rate was {}", rate);
    }

    #[test]
    fn test_rate_tracker_full_window_steady() {
        let mut tracker = RateTracker::new();
        let per_tick = 50.0 / 36000.0;
        for _ in 0..600 {
            tracker.push(per_tick);
        }
        let rate = tracker.rate_per_hour();
        assert!((rate - 50.0).abs() < 0.1, "rate was {}", rate);
    }

    #[test]
    fn test_rate_tracker_evicts_old_values() {
        let mut tracker = RateTracker::new();
        for _ in 0..600 {
            tracker.push(1.0);
        }
        for _ in 0..600 {
            tracker.push(0.0);
        }
        assert!((tracker.rate_per_hour()).abs() < 1e-9);
    }

    #[test]
    fn test_rate_tracker_default_matches_new() {
        let tracker = RateTracker::default();
        assert!((tracker.rate_per_hour()).abs() < 1e-9);
    }

    #[test]
    fn test_node_id_name_all_variants() {
        assert_eq!(NodeId::EmberSpindle.name(), "Ember Spindle");
        assert_eq!(NodeId::ReflectionLens.name(), "Reflection Lens");
        assert_eq!(NodeId::VoidCondenser.name(), "Void Condenser");
        assert_eq!(NodeId::MemoryArchive.name(), "Memory Archive");
        assert_eq!(NodeId::SilenceWell.name(), "Silence Well");
        assert_eq!(NodeId::ResonanceForge.name(), "Resonance Forge");
    }

    #[test]
    fn test_node_id_index_all_variants() {
        assert_eq!(NodeId::EmberSpindle.index(), 0);
        assert_eq!(NodeId::ReflectionLens.index(), 1);
        assert_eq!(NodeId::VoidCondenser.index(), 2);
        assert_eq!(NodeId::MemoryArchive.index(), 3);
        assert_eq!(NodeId::SilenceWell.index(), 4);
        assert_eq!(NodeId::ResonanceForge.index(), 5);
    }

    #[test]
    fn test_node_id_nature_all_variants() {
        assert_eq!(NodeId::EmberSpindle.nature(), NodeNature::Heat);
        assert_eq!(NodeId::ReflectionLens.nature(), NodeNature::Form);
        assert_eq!(NodeId::VoidCondenser.nature(), NodeNature::Void);
        assert_eq!(NodeId::MemoryArchive.nature(), NodeNature::Pattern);
        assert_eq!(NodeId::SilenceWell.nature(), NodeNature::Stillness);
        assert_eq!(NodeId::ResonanceForge.nature(), NodeNature::Vibration);
    }

    #[test]
    fn test_node_id_all_constant_has_six_unique_entries() {
        use std::collections::HashSet;
        let set: HashSet<NodeId> = NodeId::ALL.iter().copied().collect();
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn test_loom_archetype_variants_are_distinct() {
        assert_ne!(LoomArchetype::BurnBright, LoomArchetype::ReachWide);
        assert_ne!(LoomArchetype::ReachWide, LoomArchetype::RunDeep);
        assert_eq!(LoomArchetype::BurnBright, LoomArchetype::BurnBright);
    }

    #[test]
    fn test_loom_archetype_serde_round_trip() {
        for archetype in [
            LoomArchetype::BurnBright,
            LoomArchetype::ReachWide,
            LoomArchetype::RunDeep,
        ] {
            let json = serde_json::to_string(&archetype).unwrap();
            let loaded: LoomArchetype = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded, archetype);
        }
    }

    #[test]
    fn test_loom_node_new_defaults() {
        let node = LoomNode::new(NodeId::SilenceWell);
        assert_eq!(node.id, NodeId::SilenceWell);
        assert_eq!(node.level, 1);
        assert!(!node.unlocked);
        assert!((node.buffer - 0.0).abs() < 1e-9);
        assert!((node.buffer_capacity - 250.0).abs() < 1e-9);
        assert!((node.base_rate - 25.0).abs() < 1e-9);
        assert!(!node.stalled);
        assert!((node.unlock_progress - 0.0).abs() < 1e-9);
        assert!(!node.upgrading);
        assert!((node.upgrade_remaining_secs - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_loom_node_deserialize_applies_defaults() {
        // Only `id` is required; every other field should fall back to its
        // #[serde(default = "...")] function.
        let json = r#"{"id":"EmberSpindle"}"#;
        let node: LoomNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.id, NodeId::EmberSpindle);
        assert_eq!(node.level, 1);
        assert!((node.buffer_capacity - 250.0).abs() < 1e-9);
        assert!((node.base_rate - 25.0).abs() < 1e-9);
        assert!(!node.unlocked);
        assert!(!node.stalled);
    }

    #[test]
    fn test_shuttle_deserialize_applies_defaults() {
        // Omit tier, amount, buffer_capacity, level, sources — all have
        // #[serde(default = "...")] or #[serde(default)] fallbacks.
        let json = r#"{
            "input_a": "Ember",
            "input_b": "VoidEssence",
            "nature": "Heat",
            "output": "ForgedLight"
        }"#;
        let shuttle: Shuttle = serde_json::from_str(json).unwrap();
        assert_eq!(shuttle.tier, 1);
        assert!((shuttle.amount - 1.0).abs() < 1e-9);
        assert!((shuttle.buffer_capacity - 250.0).abs() < 1e-9);
        assert_eq!(shuttle.level, 1);
        assert!(shuttle.sources_a.is_empty());
        assert!(shuttle.sources_b.is_empty());
        assert!(!shuttle.under_construction);
        assert!((shuttle.construction_secs_remaining - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_shuttle_construction_secs_remaining_alias() {
        // Legacy save field name should still deserialize via `alias`.
        let json = r#"{
            "input_a": "Ember",
            "input_b": "VoidEssence",
            "nature": "Heat",
            "output": "ForgedLight",
            "construction_ticks_remaining": 42.0
        }"#;
        let shuttle: Shuttle = serde_json::from_str(json).unwrap();
        assert!((shuttle.construction_secs_remaining - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_pattern_requirement_amount_alias() {
        let json = r#"{
            "resource": "Ember",
            "rate_per_hour": 12.5
        }"#;
        let req: PatternRequirement = serde_json::from_str(json).unwrap();
        assert!((req.amount - 12.5).abs() < 1e-9);
    }

    #[test]
    fn test_codex_entry_fields() {
        let entry = CodexEntry {
            inputs: vec![Resource::Ember, Resource::VoidEssence],
            node_nature: NodeNature::Heat,
            output: Resource::ForgedLight,
            output_amount: 2.0,
            discovered: true,
        };
        assert_eq!(entry.inputs.len(), 2);
        assert_eq!(entry.node_nature, NodeNature::Heat);
        assert_eq!(entry.output, Resource::ForgedLight);
        assert!((entry.output_amount - 2.0).abs() < 1e-9);
        assert!(entry.discovered);
    }

    #[test]
    fn test_woven_pattern_count_excludes_eternal() {
        let mut state = LoomState::new();
        state.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Regular".to_string(),
            requirements: vec![],
            completed: false,
            flavor: String::new(),
            eternal: false,
        });
        state.persistent.patterns.push(WovenPattern {
            index: 1,
            name: "Eternal".to_string(),
            requirements: vec![],
            completed: false,
            flavor: String::new(),
            eternal: true,
        });
        assert_eq!(state.persistent.woven_pattern_count(), 1);
    }

    #[test]
    fn test_completed_pattern_count_excludes_eternal_even_when_completed() {
        let mut state = LoomState::new();
        state.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Eternal".to_string(),
            requirements: vec![],
            completed: true,
            flavor: String::new(),
            eternal: true,
        });
        assert_eq!(state.persistent.completed_pattern_count(), 0);
    }

    #[test]
    fn test_max_shuttles_curve_boundaries() {
        let cases: [(usize, usize); 10] = [
            (0, 0),
            (1, 1),
            (3, 1),
            (4, 2),
            (7, 2),
            (8, 3),
            (11, 3),
            (12, 4),
            (14, 4),
            (15, MAX_SHUTTLES),
        ];
        for (completed, expected_slots) in cases {
            let mut state = LoomState::new();
            for i in 0..completed {
                state.persistent.patterns.push(WovenPattern {
                    index: i as u32,
                    name: format!("P{}", i),
                    requirements: vec![],
                    completed: true,
                    flavor: String::new(),
                    eternal: false,
                });
            }
            assert_eq!(
                state.persistent.max_shuttles(),
                expected_slots,
                "completed={} expected slots={}",
                completed,
                expected_slots
            );
        }
    }

    #[test]
    fn test_max_shuttles_never_exceeds_cap_at_high_counts() {
        let mut state = LoomState::new();
        for i in 0..50 {
            state.persistent.patterns.push(WovenPattern {
                index: i,
                name: format!("P{}", i),
                requirements: vec![],
                completed: true,
                flavor: String::new(),
                eternal: false,
            });
        }
        assert_eq!(state.persistent.max_shuttles(), MAX_SHUTTLES);
    }

    #[test]
    fn test_loom_persistent_default_trait() {
        let persistent = LoomPersistent::default();
        assert_eq!(persistent.version, LOOM_SAVE_VERSION);
        assert!(!persistent.discovered);
        assert!(persistent.archetype.is_none());
        assert_eq!(persistent.nodes.len(), 6);
        assert!(persistent.shuttles.is_empty());
    }

    #[test]
    fn test_loom_state_default_trait() {
        let state = LoomState::default();
        assert!(!state.persistent.discovered);
        assert!((state.time_warp - 1.0).abs() < 1e-9);
        assert!(!state.graph_dirty);
        assert!(state.last_tick_at.is_none());
    }

    #[test]
    fn test_loom_state_transient_fields_not_serialized() {
        let mut state = LoomState::new();
        state.time_warp = 5.0;
        state.graph_dirty = true;
        state
            .rate_trackers
            .insert(Resource::Ember, RateTracker::new());

        let json = serde_json::to_string(&state).unwrap();
        let loaded: LoomState = serde_json::from_str(&json).unwrap();

        // Transient fields reset to their defaults on load, regardless of
        // what was set before serialization.
        assert!((loaded.time_warp - 1.0).abs() < 1e-9);
        assert!(!loaded.graph_dirty);
        assert!(loaded.rate_trackers.is_empty());
        assert!(loaded.last_tick_at.is_none());
        // Persistent data survives the round trip.
        assert_eq!(loaded.persistent.nodes.len(), state.persistent.nodes.len());
    }

    #[test]
    fn test_loom_ui_state_new_and_default() {
        let ui = LoomUiState::new();
        assert!(!ui.open);
        assert!(ui.selected_graph_node.is_none());
        assert!(ui.particle_phases.is_empty());
        assert_eq!(ui.throbber_frame, 0);
        assert!(ui.build.is_none());
        assert!(ui.loom_graph.is_none());
        assert!(ui.loom_layout.is_none());
        assert!(!ui.graph_dirty);
        assert!(!ui.demolish_pending);

        let default_ui = LoomUiState::default();
        assert!(!default_ui.open);
    }

    #[test]
    fn test_loom_ui_state_open_method() {
        let mut ui = LoomUiState::new();
        assert!(!ui.open);
        ui.open();
        assert!(ui.open);
    }

    #[test]
    fn test_build_step_variants_construct() {
        let recipe = BuildStep::SelectRecipe { cursor: 0 };
        let sources_a = BuildStep::SelectSourcesA {
            cursor: 1,
            toggle: vec![true, false],
        };
        let sources_b = BuildStep::SelectSourcesB {
            cursor: 2,
            toggle: vec![false],
        };
        let confirm = BuildStep::Confirm;
        let blocked = BuildStep::Blocked {
            message: "need more patterns".to_string(),
        };

        // Exercise Debug + Clone derives on every variant.
        for step in [
            recipe.clone(),
            sources_a.clone(),
            sources_b.clone(),
            confirm.clone(),
            blocked.clone(),
        ] {
            assert!(!format!("{:?}", step).is_empty());
        }

        match blocked {
            BuildStep::Blocked { message } => assert_eq!(message, "need more patterns"),
            _ => panic!("expected Blocked variant"),
        }
    }

    #[test]
    fn test_build_state_construction() {
        let build = BuildState {
            step: BuildStep::Confirm,
            tier: 2,
            recipe_index: 3,
            available_recipes: vec![0, 1, 2],
            eligible_sources_a: vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            eligible_sources_b: vec![LoomNodeRef::Shuttle(0)],
            selected_sources_a: vec![],
            selected_sources_b: vec![],
        };
        assert_eq!(build.tier, 2);
        assert_eq!(build.available_recipes.len(), 3);
        assert!(matches!(build.step, BuildStep::Confirm));
    }

    #[test]
    fn test_resource_and_nodenature_hash_and_eq() {
        use std::collections::HashSet;
        let mut resources = HashSet::new();
        resources.insert(Resource::Ember);
        resources.insert(Resource::Ember);
        resources.insert(Resource::WovenReality);
        assert_eq!(resources.len(), 2);

        assert_eq!(NodeNature::Heat, NodeNature::Heat);
        assert_ne!(NodeNature::Heat, NodeNature::Void);
    }
}
