//! The Deep — core data structures for the mercenary expedition system.
//!
//! An endgame (P15+) account-level system where players recruit and manage a
//! mercenary company, sending squads on long-duration wall-clock missions that
//! push deeper into a vast underground structure called The Deep.

use serde::{Deserialize, Serialize};
use std::fmt;

// =========================================================================
// Top-level state
// =========================================================================

/// Account-level Deep state, saved to ~/.quest/deep.json.
///
/// Persistence model:
/// - Persists across prestiges: guild_rank, layers (cleared state, infrastructure, familiarity),
///   total_marks_earned
/// - Resets on prestige: mercenaries, active_missions, completed_missions, warband_marks,
///   recruitment_pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheDeepState {
    /// Whether the player has discovered The Deep.
    pub discovered: bool,
    /// Persistent guild rank (persists across prestiges).
    pub guild_rank: GuildRank,
    /// Current warband marks (resets on prestige).
    pub warband_marks: u32,
    /// Active mercenary roster (resets on prestige).
    pub mercenaries: Vec<Mercenary>,
    /// Missions currently in progress (resets on prestige).
    pub active_missions: Vec<ActiveMission>,
    /// Missions completed and awaiting reward collection.
    pub completed_missions: Vec<CompletedMission>,
    /// Per-layer state (familiarity, cleared status, infrastructure — persists across prestiges).
    pub layers: Vec<LayerState>,
    /// Pool of mercenaries available to recruit (refreshes daily or on prestige).
    pub recruitment_pool: Vec<Mercenary>,
    /// Date (YYYY-MM-DD) of last recruitment pool refresh, for daily reset logic.
    pub recruitment_refresh_date: Option<String>,
    /// Lifetime Warband Marks earned across all prestiges.
    pub total_marks_earned: u64,
}

impl TheDeepState {
    pub fn new() -> Self {
        Self {
            discovered: false,
            guild_rank: GuildRank::default(),
            warband_marks: 0,
            mercenaries: Vec::new(),
            active_missions: Vec::new(),
            completed_missions: Vec::new(),
            layers: Vec::new(),
            recruitment_pool: Vec::new(),
            recruitment_refresh_date: None,
            total_marks_earned: 0,
        }
    }
}

impl Default for TheDeepState {
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
    /// Number of missions this merc has completed across this prestige cycle.
    pub missions_completed: u32,
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
    /// Unix timestamp when the mission was started.
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
/// Placeholder variants for now — will be expanded with flavor text by game designers.
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
            InfrastructureType::Outpost => 300,
            InfrastructureType::SupplyCache => 250,
            InfrastructureType::Watchtower => 200,
            InfrastructureType::Bridge => 500,
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
/// Layer state (familiarity, cleared, infrastructure) persists across prestiges.
/// Maximum 2 infrastructure slots per layer.
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
