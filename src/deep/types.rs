//! The Deep — core data structures for the mercenary expedition system.
//!
//! An endgame (P15+) account-level system where players recruit and manage a
//! mercenary company, sending squads on long-duration wall-clock missions that
//! push deeper into a vast underground structure called The Deep.
//!
//! ## Persistence boundary
//!
//! The prestige reset boundary is encoded in the type system rather than
//! relying on a reset function to zero the right fields:
//!
//! - [`DeepAccountState`] — survives every prestige (guild rank, layer progress,
//!   infrastructure, lifetime stats)
//! - [`DeepRunState`] — one mercenary company per prestige cycle; replaced on
//!   prestige with a fresh default
//! - [`TheDeepState`] — top-level wrapper holding both, plus the discovery flag
//!
//! ## Time handling
//!
//! No chrono calls are made inside this module. Unix timestamps (`i64`) are
//! stored as raw values. The caller (main.rs) injects the current time so that
//! all deep logic remains deterministically testable.

use serde::{Deserialize, Serialize};
use std::fmt;

// =========================================================================
// Balance constants
// =========================================================================

// -- Supply Run rewards & costs --
/// Minimum Warband Marks reward from a successful Supply Run.
pub const SUPPLY_RUN_REWARD_MIN: u32 = 35;
/// Maximum Warband Marks reward from a successful Supply Run.
pub const SUPPLY_RUN_REWARD_MAX: u32 = 55;
/// Minimum Warband Marks cost to launch a Supply Run.
pub const SUPPLY_RUN_COST_MIN: u32 = 20;
/// Maximum Warband Marks cost to launch a Supply Run.
pub const SUPPLY_RUN_COST_MAX: u32 = 30;

// -- Breakthrough rewards & costs --
/// Minimum cost to launch a Breakthrough mission (scales up with layer).
pub const BREAKTHROUGH_COST_MIN: u32 = 80;
/// Maximum cost to launch a Breakthrough mission (scales up with layer).
pub const BREAKTHROUGH_COST_MAX: u32 = 150;
/// Marks reward for a Breakthrough on Layer 1-3 (The Shallows).
pub const BREAKTHROUGH_REWARD_SHALLOWS: u32 = 200;
/// Marks reward for a Breakthrough on Layer 4-7 (The Warrens).
pub const BREAKTHROUGH_REWARD_WARRENS: u32 = 250;
/// Marks reward for a Breakthrough on Layer 8+ (The Hollows and beyond).
pub const BREAKTHROUGH_REWARD_HOLLOWS_PLUS: u32 = 350;

// -- Merc recruitment costs --
/// Cost to recruit a random-archetype mercenary.
pub const RECRUIT_COST_BASIC_MIN: u32 = 30;
/// Cost to recruit a random-archetype mercenary (upper bound).
pub const RECRUIT_COST_BASIC_MAX: u32 = 50;
/// Cost to recruit a mercenary of a specific archetype.
pub const RECRUIT_COST_SPECIFIC_MIN: u32 = 60;
/// Cost to recruit a mercenary of a specific archetype (upper bound).
pub const RECRUIT_COST_SPECIFIC_MAX: u32 = 100;
/// Cost to recruit a premium (high-stat) mercenary.
pub const RECRUIT_COST_PREMIUM_MIN: u32 = 100;
/// Cost to recruit a premium (high-stat) mercenary (upper bound).
pub const RECRUIT_COST_PREMIUM_MAX: u32 = 150;

// -- Infrastructure costs (Warband Marks, via construction mission) --
/// Construction cost for an Outpost on a cleared layer.
pub const INFRA_COST_OUTPOST: u32 = 100;
/// Construction cost for a Supply Cache on a cleared layer.
pub const INFRA_COST_SUPPLY_CACHE: u32 = 100;
/// Construction cost for a Watchtower on a cleared layer.
pub const INFRA_COST_WATCHTOWER: u32 = 150;
/// Construction cost for a Bridge on a cleared layer.
pub const INFRA_COST_BRIDGE: u32 = 200;

// -- Outpost passive income (Marks/day, scales by layer tier) --
/// Daily passive income from an Outpost on Layers 1-3 (The Shallows).
pub const OUTPOST_INCOME_SHALLOWS: u32 = 15;
/// Daily passive income from an Outpost on Layers 4-7 (The Warrens).
pub const OUTPOST_INCOME_WARRENS: u32 = 20;
/// Daily passive income from an Outpost on Layer 8+ (The Hollows and beyond).
pub const OUTPOST_INCOME_HOLLOWS_PLUS: u32 = 25;

// -- Prestige recovery --
/// Free starter mercs on each new prestige run = min(this, guild_rank.roster_cap()).
/// Rank 1 (cap 5) → 5 free; Rank 2 (cap 7) → 5 free + recruit 2; etc.
pub const FREE_STARTER_MERCS: usize = 5;

// =========================================================================
// Top-level state
// =========================================================================

/// Top-level Deep state, saved to ~/.quest/deep.json.
///
/// Wraps the two sub-states whose lifetimes differ at the prestige boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheDeepState {
    /// Whether the player has discovered The Deep.
    pub discovered: bool,
    /// Data that persists across every prestige (guild rank, layers, lifetime stats).
    pub account: DeepAccountState,
    /// Data for the current prestige cycle; replaced with a fresh default on prestige.
    pub run: DeepRunState,
}

impl TheDeepState {
    pub fn new() -> Self {
        Self {
            discovered: false,
            account: DeepAccountState::new(),
            run: DeepRunState::new(),
        }
    }
}

impl Default for TheDeepState {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Account-level state (persists across prestiges)
// =========================================================================

/// Data that persists across every prestige reset.
///
/// Contains: guild rank, per-layer progress (cleared/familiarity/infrastructure),
/// and lifetime statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepAccountState {
    /// Persistent guild rank — carries over to every new run.
    pub guild_rank: GuildRank,
    /// Per-layer state (cleared, familiarity, infrastructure).
    /// Indexed by layer number; grows as the player pushes deeper.
    pub layers: Vec<LayerState>,
    /// Lifetime Warband Marks earned across all prestige cycles.
    pub total_marks_earned: u64,
}

impl DeepAccountState {
    pub fn new() -> Self {
        Self {
            guild_rank: GuildRank::default(),
            layers: Vec::new(),
            total_marks_earned: 0,
        }
    }
}

impl Default for DeepAccountState {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Run-level state (resets on prestige)
// =========================================================================

/// Data for one prestige cycle's mercenary company.
///
/// Replaced with a fresh `DeepRunState::new()` on each prestige. The player
/// starts each run at zero marks with an empty roster and fresh recruitment pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepRunState {
    /// Current warband marks (reset on prestige).
    pub warband_marks: u32,
    /// Active mercenary roster (reset on prestige).
    pub mercenaries: Vec<Mercenary>,
    /// Missions currently in progress (reset on prestige).
    pub active_missions: Vec<ActiveMission>,
    /// Completed missions awaiting reward collection.
    pub completed_missions: Vec<CompletedMission>,
    /// Recruitment pool available for hiring (refreshes daily or on prestige).
    pub recruitment_pool: Vec<Mercenary>,
    /// Date string (YYYY-MM-DD) of the last recruitment pool refresh.
    /// Comparison with the current date is done by the caller — not here.
    pub recruitment_refresh_date: Option<String>,
}

impl DeepRunState {
    pub fn new() -> Self {
        Self {
            warband_marks: 0,
            mercenaries: Vec::new(),
            active_missions: Vec::new(),
            completed_missions: Vec::new(),
            recruitment_pool: Vec::new(),
            recruitment_refresh_date: None,
        }
    }
}

impl Default for DeepRunState {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Mercenaries
// =========================================================================

/// A single mercenary in the player's roster or recruitment pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mercenary {
    /// Unique identifier within a prestige cycle.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Archetype determines stat distribution and event options.
    pub archetype: MercArchetype,
    /// Level 1-10, gained from completing missions; resets on prestige.
    pub level: u8,
    /// Combat effectiveness (derived from archetype + level).
    pub power: u32,
    /// Survival chance when things go wrong.
    pub resilience: u32,
    /// Current availability status.
    pub status: MercStatus,
    /// Number of missions this merc has completed in this prestige cycle.
    pub missions_completed: u32,
    /// Missions remaining before an injured merc returns to duty.
    /// 0 when not injured. Set by `injure_mercenary`, decremented by `recover_mercenaries`.
    #[serde(default)]
    pub injury_cooldown: u8,
}

/// Determines a mercenary's stat distribution, role, and event unlock options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MercArchetype {
    /// Frontline tank — reduces squad casualties.
    Vanguard,
    /// Recon specialist — reveals events earlier, better auto-resolve.
    Scout,
    /// Healer — reduces injury severity, prevents merc loss.
    Medic,
    /// Trap and obstacle specialist — speeds missions, unlocks alternate routes.
    Saboteur,
    /// Elemental specialist — counters environmental hazards.
    Arcanist,
}

impl fmt::Display for MercArchetype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MercArchetype::Vanguard => "Vanguard",
            MercArchetype::Scout => "Scout",
            MercArchetype::Medic => "Medic",
            MercArchetype::Saboteur => "Saboteur",
            MercArchetype::Arcanist => "Arcanist",
        };
        write!(f, "{}", name)
    }
}

/// Current availability status of a mercenary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MercStatus {
    /// Available to assign to missions.
    Ready,
    /// Deployed on an active mission — cannot be reassigned.
    OnMission,
    /// Temporarily unavailable after injury (1-2 missions).
    Injured,
    /// Permanently gone — removed from roster.
    Lost,
}

// =========================================================================
// Missions
// =========================================================================

/// The type of a mission, determining duration, risk, and available layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    /// 2-4h, no risk, cleared layers only — resource farming, safe merc XP.
    SupplyRun,
    /// 4-8h, low risk, frontier layer — gather intel, reveal events.
    Recon,
    /// 8-16h, medium risk, frontier layer — main progression.
    Expedition,
    /// 18-24h, high risk, frontier layer (once) — unlock next layer.
    Breakthrough,
    /// 4-8h, no risk, cleared layers only — build infrastructure.
    Construction,
}

/// An active mission in progress.
///
/// Time is represented as raw Unix timestamps (`i64`). The caller injects the
/// current timestamp so that all mission logic is testable with fixed values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMission {
    /// Unique mission identifier.
    pub id: u32,
    /// Type of this mission.
    pub mission_type: MissionType,
    /// Which layer this mission is on.
    pub layer: u8,
    /// Mercenary IDs assigned to this mission.
    pub squad: Vec<u32>,
    /// Unix timestamp (seconds) when the mission was started.
    /// Injected by the caller; never set by calling `Utc::now()` in this module.
    pub start_time: i64,
    /// Total mission duration in seconds (wall-clock time).
    pub duration_secs: u64,
    /// Warband Marks cost to launch this mission.
    pub cost: u32,
    /// Scheduled check-in events for this mission.
    pub events: Vec<MissionEvent>,
    /// How many events have been resolved so far.
    pub events_resolved: usize,
}

/// A check-in event that fires at a scheduled point during a mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionEvent {
    /// When this event triggers, as percent through mission duration (25, 50, 75).
    pub trigger_at_percent: u8,
    /// What kind of event occurred.
    pub event_type: EventType,
    /// Whether the player has already resolved this event.
    pub resolved: bool,
    /// How the event was resolved (set when resolved = true).
    pub resolution: Option<EventResolution>,
}

/// Types of check-in events that can occur during a mission.
/// Placeholder variants — will be expanded with flavor text by game designers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Collapsed tunnel blocking the main path.
    CaveIn,
    /// Hostile creatures or rival faction encountered.
    Ambush,
    /// Water-filled passage requiring alternate approach.
    FloodedPassage,
    /// Sealed ancient door with unknown contents beyond.
    AncientDoor,
    /// Ground shaking — structural instability threat.
    Tremor,
}

/// How a check-in event was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventResolution {
    /// Resolved safely with no complications.
    Safe,
    /// Resolved using an archetype-specific ability.
    Archetype,
    /// Risky resolution chosen — higher reward, potential injury.
    Risky,
    /// Auto-resolved while player was away (always picks safest option).
    AutoResolved,
}

/// A mission that has completed and is awaiting reward collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedMission {
    /// Type of mission that was run.
    pub mission_type: MissionType,
    /// Which layer the mission was on.
    pub layer: u8,
    /// Overall outcome of the mission.
    pub result: MissionResult,
    /// Warband Marks earned from this mission.
    pub marks_earned: u32,
    /// Summary text for each event that occurred.
    pub events_summary: Vec<String>,
    /// Mercenary IDs that were injured during this mission.
    pub injuries: Vec<u32>,
    /// Whether the player has collected the rewards from this mission.
    pub collected: bool,
}

/// The overall outcome of a completed mission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionResult {
    /// Full rewards earned.
    Success,
    /// Reduced rewards, possible injuries.
    PartialSuccess,
    /// Minimal rewards, injuries or losses possible.
    Failure,
}

// =========================================================================
// Guild Rank
// =========================================================================

/// The player's guild rank, which persists across all prestiges.
///
/// Determines maximum roster size and concurrent mission slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuildRank {
    /// Rank 1 — starting rank, 5 mercs, 1 mission slot.
    #[default]
    Freelancers,
    /// Rank 2 — requires Layer 3 breakthrough.
    Sellswords,
    /// Rank 3 — requires Layer 7 breakthrough.
    Company,
    /// Rank 4 — requires Layer 13 breakthrough.
    Battalion,
    /// Rank 5 (max) — requires Layer 19 breakthrough.
    Legion,
}

impl GuildRank {
    /// Maximum number of mercenaries in the roster at this rank.
    pub fn roster_cap(&self) -> usize {
        match self {
            GuildRank::Freelancers => 5,
            GuildRank::Sellswords => 7,
            GuildRank::Company => 9,
            GuildRank::Battalion => 12,
            GuildRank::Legion => 15,
        }
    }

    /// Number of missions that can run concurrently at this rank.
    pub fn mission_slots(&self) -> usize {
        match self {
            GuildRank::Freelancers => 1,
            GuildRank::Sellswords => 1,
            GuildRank::Company => 2,
            GuildRank::Battalion => 3,
            GuildRank::Legion => 4,
        }
    }

    /// Warband Marks cost to upgrade to the next rank, or None if already max rank.
    pub fn upgrade_cost(&self) -> Option<u32> {
        match self {
            GuildRank::Freelancers => Some(500),
            GuildRank::Sellswords => Some(1_500),
            GuildRank::Company => Some(4_000),
            GuildRank::Battalion => Some(10_000),
            GuildRank::Legion => None,
        }
    }

    /// Layer breakthrough required to be eligible for this rank, or None for Freelancers.
    pub fn required_layer(&self) -> Option<u8> {
        match self {
            GuildRank::Freelancers => None,
            GuildRank::Sellswords => Some(3),
            GuildRank::Company => Some(7),
            GuildRank::Battalion => Some(13),
            GuildRank::Legion => Some(19),
        }
    }

    /// The next guild rank, or None if already at Legion.
    pub fn next(&self) -> Option<GuildRank> {
        match self {
            GuildRank::Freelancers => Some(GuildRank::Sellswords),
            GuildRank::Sellswords => Some(GuildRank::Company),
            GuildRank::Company => Some(GuildRank::Battalion),
            GuildRank::Battalion => Some(GuildRank::Legion),
            GuildRank::Legion => None,
        }
    }

    /// Number of free starter mercenaries provided at the beginning of each prestige run.
    ///
    /// Always `min(FREE_STARTER_MERCS, self.roster_cap())`. Rank 1 (cap 5) gets all
    /// 5 free. Rank 2+ (cap 7+) still gets 5 free and must recruit the rest.
    pub fn free_starter_mercs(&self) -> usize {
        FREE_STARTER_MERCS.min(self.roster_cap())
    }
}

// =========================================================================
// Infrastructure
// =========================================================================

/// Permanent infrastructure that can be built on cleared layers.
///
/// Infrastructure persists across all prestiges and makes future missions easier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfrastructureType {
    /// Reduces mission duration on this layer by 25%.
    Outpost,
    /// Supply run missions yield bonus resources.
    SupplyCache,
    /// Reveals more intel, improves auto-resolve outcomes.
    Watchtower,
    /// Unlocks shortcut — missions can skip this layer when pushing deeper.
    Bridge,
}

impl InfrastructureType {
    /// Warband Marks cost to build this infrastructure via a construction mission.
    pub fn cost(&self) -> u32 {
        match self {
            InfrastructureType::Outpost => INFRA_COST_OUTPOST,
            InfrastructureType::SupplyCache => INFRA_COST_SUPPLY_CACHE,
            InfrastructureType::Watchtower => INFRA_COST_WATCHTOWER,
            InfrastructureType::Bridge => INFRA_COST_BRIDGE,
        }
    }

    /// Daily Warband Marks income generated by an Outpost on the given layer number.
    ///
    /// Income scales by layer tier per the balance design. This is a free function
    /// on `InfrastructureType` because only Outposts generate passive income; callers
    /// should gate on `self == InfrastructureType::Outpost` before calling.
    pub fn outpost_daily_income(layer: u8) -> u32 {
        match LayerTier::from_layer(layer) {
            LayerTier::Shallows => OUTPOST_INCOME_SHALLOWS,
            LayerTier::Warrens => OUTPOST_INCOME_WARRENS,
            // Hollows and every deeper tier yield the highest rate
            _ => OUTPOST_INCOME_HOLLOWS_PLUS,
        }
    }

    /// Short description of this infrastructure's effect.
    pub fn description(&self) -> &str {
        match self {
            InfrastructureType::Outpost => "Reduces mission duration on this layer by 25%",
            InfrastructureType::SupplyCache => "Supply run missions yield bonus resources",
            InfrastructureType::Watchtower => "Reveals more intel, improves auto-resolve outcomes",
            InfrastructureType::Bridge => {
                "Unlocks shortcut — missions can skip this layer when pushing deeper"
            }
        }
    }
}

// =========================================================================
// Layer state
// =========================================================================

/// State for a single layer of The Deep.
///
/// Layer state (familiarity, cleared, infrastructure) persists across prestiges
/// inside [`DeepAccountState`]. Maximum 2 infrastructure slots per layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerState {
    /// Which layer number (1-based).
    pub layer_number: u8,
    /// Intel / familiarity with this layer (0.0 = unknown, 1.0 = fully mapped).
    /// Increases from completing missions on this layer.
    pub familiarity: f32,
    /// Whether a breakthrough mission has been completed on this layer.
    /// Cleared layers are available for supply runs and construction.
    pub cleared: bool,
    /// Infrastructure built on this layer (maximum 2 slots).
    pub infrastructure: Vec<InfrastructureType>,
}

// =========================================================================
// Layer tiers
// =========================================================================

/// The thematic tier of a layer, determined by its depth.
///
/// ```text
/// Layer 1-3:   The Shallows     (introductory)
/// Layer 4-7:   The Warrens      (branching tunnels)
/// Layer 8-12:  The Hollows      (open caverns, environmental hazards)
/// Layer 13-18: The Sunken Reach (flooded/corrupted)
/// Layer 19-25: The Abyss        (extreme danger)
/// Layer 26+:   The Void         (infinite scaling endgame)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerTier {
    Shallows,
    Warrens,
    Hollows,
    SunkenReach,
    Abyss,
    Void,
}

impl LayerTier {
    /// Returns the tier for a given layer number (1-based).
    pub fn from_layer(layer: u8) -> LayerTier {
        match layer {
            1..=3 => LayerTier::Shallows,
            4..=7 => LayerTier::Warrens,
            8..=12 => LayerTier::Hollows,
            13..=18 => LayerTier::SunkenReach,
            19..=25 => LayerTier::Abyss,
            _ => LayerTier::Void,
        }
    }
}

impl fmt::Display for LayerTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LayerTier::Shallows => "The Shallows",
            LayerTier::Warrens => "The Warrens",
            LayerTier::Hollows => "The Hollows",
            LayerTier::SunkenReach => "The Sunken Reach",
            LayerTier::Abyss => "The Abyss",
            LayerTier::Void => "The Void",
        };
        write!(f, "{}", name)
    }
}
