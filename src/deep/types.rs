#![allow(dead_code)] // Functions wired into the game loop incrementally
//! The Deep — Mercenary Expedition System data structures.
//!
//! Two-tier persistence model (both persist across prestiges):
//! - **Account-level**: Guild rank, layer breakthroughs, infrastructure, familiarity
//! - **Operational**: Mercenaries, active missions, Warband Marks

use crate::achievements::AchievementId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serde helper: serialize `HashMap<u64, Mercenary>` as `Vec<Mercenary>` for
/// backward-compatible JSON saves.
mod roster_serde {
    use super::Mercenary;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;

    pub fn serialize<S: Serializer>(
        map: &HashMap<u64, Mercenary>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let vec: Vec<&Mercenary> = map.values().collect();
        vec.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<u64, Mercenary>, D::Error> {
        let vec = Vec::<Mercenary>::deserialize(deserializer)?;
        Ok(vec.into_iter().map(|m| (m.id, m)).collect())
    }
}

// ── Discovery & Gate ─────────────────────────────────────────────────────────

/// Minimum prestige rank required to discover The Deep.
pub use crate::core::constants::DEEP_MIN_PRESTIGE_RANK;
/// The layer where the Gateway is located.
pub const GATEWAY_LAYER: u32 = 30;

// ── Layer Tier Bounds ─────────────────────────────────────────────────────────

pub const SHALLOWS_LAYERS: std::ops::RangeInclusive<u32> = 1..=3;
pub const WARRENS_LAYERS: std::ops::RangeInclusive<u32> = 4..=7;
pub const HOLLOWS_LAYERS: std::ops::RangeInclusive<u32> = 8..=12;
pub const SUNKEN_REACH_LAYERS: std::ops::RangeInclusive<u32> = 13..=18;
pub const ABYSS_LAYERS: std::ops::RangeInclusive<u32> = 19..=25;
// The Void is layer 26 and beyond (open-ended).
pub const VOID_START_LAYER: u32 = 26;

// ── Guild Rank Constants ──────────────────────────────────────────────────────

/// All five guild ranks, each with their roster cap, concurrent-mission cap,
/// and the deepest layer that must be broken through to reach this rank.
pub const GUILD_RANK_STATS: [(u32, u32, Option<u32>); 5] = [
    // (max_roster, concurrent_missions, required_breakthrough_layer)
    (5, 1, None),      // Rank 1 – Freelancers  (discovery unlocks)
    (7, 2, Some(3)),   // Rank 2 – Company
    (9, 2, Some(7)),   // Rank 3 – Battalion
    (12, 3, Some(13)), // Rank 4 – Legion
    (15, 4, Some(19)), // Rank 5 – Vanguard
];

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Progression tier of a layer.  Determines hazard types, flavor, and difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerTier {
    Shallows,    // L1–3
    Warrens,     // L4–7
    Hollows,     // L8–12
    SunkenReach, // L13–18
    Abyss,       // L19–25
    Void,        // L26+
}

impl LayerTier {
    /// Compute tier from a 1-based layer index.
    pub fn from_layer(layer: u32) -> Self {
        match layer {
            1..=3 => LayerTier::Shallows,
            4..=7 => LayerTier::Warrens,
            8..=12 => LayerTier::Hollows,
            13..=18 => LayerTier::SunkenReach,
            19..=25 => LayerTier::Abyss,
            _ => LayerTier::Void,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            LayerTier::Shallows => "The Shallows",
            LayerTier::Warrens => "The Warrens",
            LayerTier::Hollows => "The Hollows",
            LayerTier::SunkenReach => "The Sunken Reach",
            LayerTier::Abyss => "The Abyss",
            LayerTier::Void => "The Void",
        }
    }
}

/// Mercenary archetype — determines stat distribution and unlocks event options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MercArchetype {
    /// Frontline tank.  Primary: STR/CON.  Reduces squad casualties.
    Vanguard,
    /// Recon specialist.  Primary: DEX/WIS.  Better auto-resolve, reveals events early.
    Scout,
    /// Elemental specialist.  Primary: INT.  Counters environmental hazards.
    Arcanist,
    /// Healer.  Primary: WIS/CON.  Reduces injury severity and prevents merc loss.
    Medic,
    /// Trap/obstacle specialist.  Primary: DEX/INT.  Speeds missions, unlocks alternate routes.
    Saboteur,
}

impl MercArchetype {
    pub const ALL: &'static [MercArchetype] = &[
        MercArchetype::Vanguard,
        MercArchetype::Scout,
        MercArchetype::Arcanist,
        MercArchetype::Medic,
        MercArchetype::Saboteur,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            MercArchetype::Vanguard => "Vanguard",
            MercArchetype::Scout => "Scout",
            MercArchetype::Arcanist => "Arcanist",
            MercArchetype::Medic => "Medic",
            MercArchetype::Saboteur => "Saboteur",
        }
    }

    /// Base Power, Resilience, Expertise distribution for a fresh recruit.
    /// Returns (power, resilience, expertise).
    pub fn base_stats(self) -> (u32, u32, u32) {
        match self {
            MercArchetype::Vanguard => (14, 12, 4), // high power + resilience
            MercArchetype::Scout => (8, 10, 12),    // high expertise
            MercArchetype::Arcanist => (10, 6, 14), // highest expertise, fragile
            MercArchetype::Medic => (6, 14, 10),    // highest resilience
            MercArchetype::Saboteur => (10, 8, 12), // balanced expertise
        }
    }
}

/// Current availability status of a mercenary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MercStatus {
    /// Ready to be assigned to a mission.
    Available,
    /// Assigned to an active mission (mission id stored for lookup).
    OnMission(u64),
    /// Injured — cannot take missions for the given number of missions.
    Injured { missions_remaining: u32 },
    /// Permanently lost (kept in roster only for death notification; purged after acknowledged).
    Lost,
}

/// Type of mission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    /// 2–4h, no risk, cleared layers only.  Resource farming and safe merc XP.
    SupplyRun,
    /// 4–8h, low risk, frontier layer.  Builds familiarity, reveals check-in events.
    Recon,
    /// 8–16h, medium risk, frontier layer.  Primary progression missions.
    Expedition,
    /// 18–24h, high risk, frontier layer (once per layer).  Unlocks the next layer.
    Breakthrough,
    /// 4–8h, no risk, cleared layers only.  Builds infrastructure.
    Construction(Infrastructure),
    /// 24h, Layer 30 only.  Opens the Gateway.  One-time mission.
    GatewayExpedition,
}

impl MissionType {
    pub fn display_name(self) -> &'static str {
        match self {
            MissionType::SupplyRun => "Supply Run",
            MissionType::Recon => "Recon",
            MissionType::Expedition => "Expedition",
            MissionType::Breakthrough => "Breakthrough",
            MissionType::Construction(_) => "Construction",
            MissionType::GatewayExpedition => "Gateway Expedition",
        }
    }

    /// Risk tier: 0 = safe, 1 = low, 2 = medium, 3 = high.
    pub fn risk_tier(self) -> u8 {
        match self {
            MissionType::SupplyRun | MissionType::Construction(_) => 0,
            MissionType::Recon => 1,
            MissionType::Expedition => 2,
            MissionType::Breakthrough | MissionType::GatewayExpedition => 3,
        }
    }

    /// Nominal duration range in seconds (wall-clock).
    pub fn duration_range_secs(self) -> (u64, u64) {
        match self {
            MissionType::SupplyRun => (3600, 21600),            // 1h-6h
            MissionType::Recon => (3600, 36000),                // 1h-10h
            MissionType::Expedition => (10800, 108000),         // 3h-30h
            MissionType::Breakthrough => (14400, 144000),       // 4h-40h
            MissionType::Construction(_) => (7200, 72000),      // 2h-20h
            MissionType::GatewayExpedition => (172800, 172800), // 48h
        }
    }

    /// Maximum number of check-in events that can fire during the mission.
    pub fn max_events(self) -> u32 {
        match self {
            MissionType::SupplyRun | MissionType::Construction(_) => 0,
            MissionType::Recon => 1,
            MissionType::Expedition => 2,
            MissionType::Breakthrough | MissionType::GatewayExpedition => 5,
        }
    }
}

/// Lifecycle state of a mission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionStatus {
    /// Mission accepted but not yet started (e.g., a future start-time feature).
    Preparing,
    /// Mission is running — timer ticking.
    Active,
    /// A check-in event has fired and is awaiting player response.
    EventPending,
    /// Mission timer completed and rewards have been calculated.
    Completed,
    /// Mission failed (squad wiped or abandoned).
    Failed,
}

/// Outcome of a completed mission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionOutcome {
    /// Full rewards awarded.
    Success,
    /// Reduced rewards; possible injuries.
    PartialSuccess,
    /// Minimal rewards; injuries or merc loss.
    Failure,
}

/// Persistent infrastructure built on a cleared layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Infrastructure {
    /// Reduces mission duration on this layer by 25%.
    Outpost,
    /// Supply run missions on this layer yield bonus resources.
    SupplyCache,
    /// Boosts familiarity and improves auto-resolve outcomes.
    Watchtower,
    /// Unlocks a shortcut — expeditions can skip this layer when pushing deeper.
    Bridge,
}

impl Infrastructure {
    pub const ALL: &'static [Infrastructure] = &[
        Infrastructure::Outpost,
        Infrastructure::SupplyCache,
        Infrastructure::Watchtower,
        Infrastructure::Bridge,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Infrastructure::Outpost => "Outpost",
            Infrastructure::SupplyCache => "Supply Cache",
            Infrastructure::Watchtower => "Watchtower",
            Infrastructure::Bridge => "Bridge",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Infrastructure::Outpost => "Reduces mission duration on this layer by 25%.",
            Infrastructure::SupplyCache => "Supply runs yield bonus Warband Marks and resources.",
            Infrastructure::Watchtower => {
                "Boosts familiarity (+40); improves auto-resolve outcomes."
            }
            Infrastructure::Bridge => {
                "Unlocks a shortcut; -2% mission duration per bridged layer (max -30%)."
            }
        }
    }

    /// Single-width Unicode symbol for compact UI display.
    ///
    /// Uses standard-width Unicode symbols (not emoji) to avoid inconsistent
    /// rendering widths across terminal emulators.
    pub fn icon(self) -> char {
        match self {
            Infrastructure::Outpost => '\u{2691}', // ⚑ (black flag — claimed territory)
            Infrastructure::SupplyCache => '\u{25c8}', // ◈ (diamond in box — treasure)
            Infrastructure::Watchtower => '\u{25ce}', // ◎ (bullseye — observation)
            Infrastructure::Bridge => '\u{21d2}',  // ⇒ (double arrow — passage)
        }
    }

    /// Duration reduction applied to missions on this layer (0.0–1.0).
    pub fn duration_reduction(self) -> f64 {
        match self {
            Infrastructure::Outpost => 0.25,
            _ => 0.0,
        }
    }
}

// ── Core Data Structs ──────────────────────────────────────────────────────────

/// A single mercenary in the player's roster.
///
/// Resets on prestige — mercs represent one generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mercenary {
    /// Unique id within this prestige cycle (monotonically assigned).
    pub id: u64,
    pub name: String,
    pub archetype: MercArchetype,
    /// Raw combat effectiveness.  Scales with level and archetype.
    pub power: u32,
    /// Survival chance when things go wrong.
    pub resilience: u32,
    /// Bonus to missions matching their specialty archetype.
    pub expertise: u32,
    /// Level gained from completing missions; resets on prestige.
    pub level: u32,
    /// Total missions this merc has completed (informational).
    pub missions_completed: u32,
    pub status: MercStatus,
}

impl Mercenary {
    /// Base level for all freshly recruited mercs.
    pub const BASE_LEVEL: u32 = 1;

    /// XP-style missions required to gain a level (simple linear scaling).
    pub fn missions_to_next_level(level: u32) -> u32 {
        3 + level * 2
    }

    /// Whether this mercenary is available for assignment.
    pub fn is_available(&self) -> bool {
        matches!(self.status, MercStatus::Available)
    }

    /// Effective power accounting for level bonuses.
    pub fn effective_power(&self) -> u32 {
        self.power + (self.level.saturating_sub(1)) * 3
    }

    /// Effective resilience accounting for level bonuses.
    pub fn effective_resilience(&self) -> u32 {
        self.resilience + (self.level.saturating_sub(1)) * 2
    }
}

/// A single underground layer.
///
/// Persistent state lives in `LayerRecord` (account-level).
/// This struct is the combined view used at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// 1-based layer index.
    pub index: u32,
    /// Computed tier based on index.
    pub tier: LayerTier,
    /// Short thematic name (e.g., "Ashrift Hollow").
    pub theme_name: String,
    /// Difficulty rating (1.0 = base; scales with depth).
    pub difficulty: f64,
    /// Familiarity with this layer (0–100).  Increases from completing missions here.
    pub familiarity: u8,
    /// Infrastructure pieces that have been built on this layer.
    pub infrastructure: Vec<Infrastructure>,
    /// Whether the breakthrough mission for this layer has been completed.
    pub cleared: bool,
}

impl Layer {
    /// Compute the cumulative duration reduction from all built infrastructure.
    pub fn total_duration_reduction(&self) -> f64 {
        self.infrastructure
            .iter()
            .map(|i| i.duration_reduction())
            .sum::<f64>()
            .min(0.75) // cap at 75% reduction
    }

    /// Whether a given infrastructure piece has been built on this layer.
    pub fn has_infrastructure(&self, infra: Infrastructure) -> bool {
        self.infrastructure.contains(&infra)
    }

    /// Count of available infrastructure slots (each layer supports all 4 types once).
    pub fn available_infrastructure_slots(&self) -> usize {
        Infrastructure::ALL.len() - self.infrastructure.len()
    }
}

/// A choice option within a check-in event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoice {
    /// Short label shown in the UI (e.g., "Dig through").
    pub label: String,
    /// Optional archetype whose presence unlocks this choice.
    pub required_archetype: Option<MercArchetype>,
    /// Time added/subtracted to the mission duration in seconds (negative = speeds up).
    pub time_delta_secs: i64,
    /// Whether this choice carries a risk of injury or merc loss.
    pub is_risky: bool,
    /// Index of the choice in this event's list that becomes available on success
    /// (used for event chaining where a risky choice unlocks a bonus option later).
    pub unlocks_bonus_event: bool,
    /// Percentage chance (0-100) that this risky choice results in injury/loss.
    #[serde(default)]
    pub risk_percent: Option<u8>,
}

/// A check-in event that fires mid-mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInEvent {
    /// Narrative title shown in the UI (e.g., "CAVE-IN AHEAD").
    pub title: String,
    /// Narrative body text.
    pub description: String,
    /// All possible choices, including archetype-gated ones.
    pub choices: Vec<EventChoice>,
    /// Index of the choice that auto-resolve uses when the player does not respond.
    /// Always safe (no risk of merc loss).
    pub auto_resolve_choice: usize,
    /// Which archetype gets a bonus to outcomes when present (informational for UI).
    pub archetype_bonus: Option<MercArchetype>,
    /// Wall-clock time when this event fired.  Used to compute time-to-respond.
    pub fired_at: DateTime<Utc>,
    /// Index of the choice the player selected, or None if unresolved.
    pub resolved_choice: Option<usize>,
}

impl CheckInEvent {
    /// Effective choice index: player's selection if any, otherwise auto-resolve.
    pub fn effective_choice(&self) -> usize {
        self.resolved_choice.unwrap_or(self.auto_resolve_choice)
    }

    /// Whether this event has been resolved (by the player or automatically).
    pub fn is_resolved(&self) -> bool {
        self.resolved_choice.is_some()
    }
}

/// A log entry for a completed mission, stored in the warband log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarbandLogEntry {
    pub mission_name: String,
    pub layer: u32,
    pub outcome: MissionOutcome,
    pub marks_earned: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Record of a completed generation's accomplishments, stored permanently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub generation: u32,
    pub marks_earned: u32,
    pub missions_completed: u32,
    pub mercs_lost: u32,
    pub deepest_layer_reached: u32,
    pub gateway_opened_this_generation: bool,
}

/// Rewards produced by a completed mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionResult {
    pub outcome: MissionOutcome,
    /// Warband Marks earned.
    pub marks_earned: u32,
    /// XP to award to the character.
    pub xp_earned: u32,
    /// Item ilvl for any dropped items (None if no items).
    pub item_ilvl: Option<u32>,
    /// Merc ids that were injured as a result of this mission.
    pub injured_mercs: Vec<u64>,
    /// Merc ids that were permanently lost.
    pub lost_mercs: Vec<u64>,
    /// Level-ups earned by each surviving squad member (merc_id -> levels_gained).
    pub merc_level_ups: Vec<(u64, u32)>,
    /// Whether a danger bonus was applied to merc XP (from risky event choices).
    #[serde(default)]
    pub danger_bonus_xp: bool,
}

/// An active or recently completed mission.
///
/// Resets on prestige.  Wall-clock time is used for all timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    /// Unique id (monotonically assigned per prestige cycle).
    pub id: u64,
    pub mission_type: MissionType,
    /// 1-based target layer index.
    pub layer: u32,
    /// Ids of mercs assigned to this squad.
    pub squad: Vec<u64>,
    /// Wall-clock time the mission started.
    pub started_at: DateTime<Utc>,
    /// Wall-clock time the mission is expected to complete (accounts for infrastructure bonuses).
    pub ends_at: DateTime<Utc>,
    /// Events scheduled and/or fired during this mission.
    pub events: Vec<CheckInEvent>,
    /// Index of the next event not yet fired (events[pending_event_index] fires next).
    pub pending_event_index: usize,
    pub status: MissionStatus,
    /// Populated when mission completes.
    pub result: Option<MissionResult>,
    /// Whether this is the auto-queued "First Orders" starter mission.
    #[serde(default)]
    pub is_first_orders: bool,
}

impl Mission {
    /// Progress as a fraction 0.0..=1.0, clamped.
    pub fn progress(&self, now: DateTime<Utc>) -> f64 {
        let total = (self.ends_at - self.started_at).num_seconds().max(1) as f64;
        let elapsed = (now - self.started_at).num_seconds().max(0) as f64;
        (elapsed / total).clamp(0.0, 1.0)
    }

    /// Whether the timer has elapsed (mission is ready to resolve).
    pub fn is_time_elapsed(&self, now: DateTime<Utc>) -> bool {
        now >= self.ends_at
    }

    /// Number of unresolved pending events.
    pub fn unresolved_event_count(&self) -> usize {
        self.events.iter().filter(|e| !e.is_resolved()).count()
    }

    /// Whether this mission has a pending event awaiting player input.
    pub fn has_pending_event(&self) -> bool {
        matches!(self.status, MissionStatus::EventPending)
    }
}

/// Available mission in the mission pool (not yet accepted).
///
/// Regenerated at each pool refresh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableMission {
    pub mission_type: MissionType,
    pub layer: u32,
    /// Nominal duration in seconds (may be reduced further by infrastructure when accepted).
    pub duration_secs: u64,
    /// Minimum combined squad Power required.
    pub min_squad_power: u32,
    /// Archetype that is required for this mission (player may still attempt without it).
    pub required_archetype: Option<MercArchetype>,
    /// Archetype whose presence improves outcomes.
    pub recommended_archetype: Option<MercArchetype>,
    /// Warband Marks cost to send the mission (infrastructure or supply costs).
    pub marks_cost: u32,
    /// Short description shown in the mission list.
    pub description: String,
}

/// Rotating recruit pool — refreshes daily or on prestige.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecruitPool {
    /// Candidates available this cycle.
    pub candidates: Vec<Mercenary>,
    /// Wall-clock time the pool was last refreshed.
    pub refreshed_at: DateTime<Utc>,
    /// Warband Marks cost to recruit each candidate (index-aligned with candidates).
    pub recruit_costs: Vec<u32>,
}

impl RecruitPool {
    /// Whether the pool needs a daily refresh.
    pub fn needs_refresh(&self, now: DateTime<Utc>) -> bool {
        let age_hours = (now - self.refreshed_at).num_hours();
        age_hours >= 24
    }
}

/// Guild rank — persists across prestiges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildRank(pub u8); // 1–5

impl GuildRank {
    pub const MIN: GuildRank = GuildRank(1);
    pub const MAX: GuildRank = GuildRank(5);

    pub fn display_name(self) -> &'static str {
        match self.0 {
            1 => "Freelancers",
            2 => "Company",
            3 => "Battalion",
            4 => "Legion",
            5 => "Vanguard",
            _ => "Unknown",
        }
    }

    pub fn max_roster(self) -> u32 {
        GUILD_RANK_STATS[(self.0 as usize) - 1].0
    }

    pub fn concurrent_missions(self) -> u32 {
        GUILD_RANK_STATS[(self.0 as usize) - 1].1
    }

    /// Deepest layer that must be broken through to *unlock* this rank (None = discovery).
    pub fn required_breakthrough_layer(self) -> Option<u32> {
        GUILD_RANK_STATS[(self.0 as usize) - 1].2
    }

    pub fn can_advance(self) -> bool {
        self.0 < GuildRank::MAX.0
    }

    pub fn next(self) -> Option<GuildRank> {
        if self.can_advance() {
            Some(GuildRank(self.0 + 1))
        } else {
            None
        }
    }
}

impl Default for GuildRank {
    fn default() -> Self {
        GuildRank::MIN
    }
}

/// Effective concurrent missions, accounting for breakthrough bonuses.
/// Breaking through Layer 3 grants +1 concurrent slot even at Rank 1.
pub fn effective_concurrent_missions(guild_rank: GuildRank, deepest_layer_reached: u32) -> u32 {
    let base = guild_rank.concurrent_missions();
    // Bonus: breaking L3 grants a concurrent slot before formal Rank 2
    if guild_rank.0 == 1 && deepest_layer_reached >= 3 {
        base + 1
    } else {
        base
    }
}

// ── Persistent (Account-Level) Record Per Layer ────────────────────────────────

/// Persistent record for a single layer.  Lives inside `DeepPersistent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerRecord {
    /// 1-based layer index.
    pub index: u32,
    /// Familiarity level (0–100).  Increases from completing missions on this layer.
    pub familiarity: u8,
    /// Infrastructure built on this layer (persists across prestiges).
    pub infrastructure: Vec<Infrastructure>,
    /// Whether the breakthrough for this layer has been completed.
    pub cleared: bool,
}

impl LayerRecord {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            familiarity: 0,
            infrastructure: Vec::new(),
            cleared: false,
        }
    }

    /// Whether a given infrastructure piece has been built on this layer.
    pub fn has_infrastructure(&self, infra: Infrastructure) -> bool {
        self.infrastructure.contains(&infra)
    }

    /// Compute the cumulative duration reduction from all built infrastructure.
    pub fn total_duration_reduction(&self) -> f64 {
        self.infrastructure
            .iter()
            .map(|i| i.duration_reduction())
            .sum::<f64>()
            .min(0.75) // cap at 75% reduction
    }
}

// ── Top-Level State ────────────────────────────────────────────────────────────

fn default_fracture_zone_cap() -> u32 {
    11
}

/// Account-level Deep state — **persists across prestiges**.
///
/// Saved to `~/.quest/deep.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepPersistent {
    /// Whether The Deep has been discovered.
    pub discovered: bool,
    /// Current guild rank (1–5).
    pub guild_rank: GuildRank,
    /// Warband Marks cost to upgrade guild rank from current to next rank.
    pub guild_upgrade_cost: u32,
    /// Records for all layers reached so far (index into vec = layer_index - 1).
    pub layers: Vec<LayerRecord>,
    /// Deepest layer the player has ever set foot on (1-based).
    pub deepest_layer_reached: u32,
    /// Monotonically increasing counter to assign unique merc ids across prestiges.
    pub merc_id_counter: u64,
    /// Monotonically increasing counter for mission ids.
    pub mission_id_counter: u64,
    /// Monotonically increasing counter tracking how many prestige cycles have occurred.
    #[serde(default)]
    pub generation_counter: u32,
    /// Whether the Gateway at Layer 30 has been opened.
    #[serde(default)]
    pub gateway_opened: bool,
    /// Whether the "First Orders" starter mission has been auto-queued.
    #[serde(default)]
    pub first_orders_queued: bool,
    /// Records of completed generations (capped at 10).
    #[serde(default)]
    pub generation_records: Vec<GenerationRecord>,
    /// Highest fracture zone the player can access (default 11 = Expanse only).
    #[serde(default = "default_fracture_zone_cap")]
    pub fracture_zone_cap: u32,
    /// Pending fracture region unlock notification (consumed by tick to show modal).
    #[serde(default)]
    pub pending_fracture_region_unlock: Option<crate::zones::FractureRegion>,
    /// Power Core timestamps: maps each core's `AchievementId` to the Unix
    /// timestamp (seconds) when the core last granted a prestige rank.
    /// Missing entries are treated as "never granted" (timestamp = 0).
    #[serde(default)]
    pub power_core_last_granted: HashMap<AchievementId, i64>,
}

impl Default for DeepPersistent {
    fn default() -> Self {
        Self::new()
    }
}

impl DeepPersistent {
    pub fn new() -> Self {
        Self {
            discovered: false,
            guild_rank: GuildRank::MIN,
            guild_upgrade_cost: 500,
            layers: Vec::new(),
            deepest_layer_reached: 0,
            merc_id_counter: 0,
            mission_id_counter: 0,
            generation_counter: 0,
            gateway_opened: false,
            first_orders_queued: false,
            generation_records: Vec::new(),
            fracture_zone_cap: 11,
            pending_fracture_region_unlock: None,
            power_core_last_granted: HashMap::new(),
        }
    }

    /// Get or create a mutable layer record by 1-based index.
    pub fn layer_record_mut(&mut self, index: u32) -> &mut LayerRecord {
        // Grow the vec if needed.
        while self.layers.len() < index as usize {
            let next_index = self.layers.len() as u32 + 1;
            self.layers.push(LayerRecord::new(next_index));
        }
        &mut self.layers[(index - 1) as usize]
    }

    /// Read-only layer record view (returns None if layer not yet reached or index is 0).
    pub fn layer_record(&self, index: u32) -> Option<&LayerRecord> {
        if index == 0 {
            return None;
        }
        self.layers.get((index as usize) - 1)
    }

    /// Frontier layer index: the deepest unlocked-but-not-cleared layer.
    ///
    /// If all known layers are cleared, the frontier is the next undiscovered
    /// layer (`deepest_layer_reached + 1`), with a minimum of 1.
    pub fn frontier_layer(&self) -> u32 {
        self.layers
            .iter()
            .find(|l| !l.cleared)
            .map(|l| l.index)
            .unwrap_or_else(|| (self.deepest_layer_reached + 1).max(1))
    }

    /// Next available merc id (increments counter).
    pub fn next_merc_id(&mut self) -> u64 {
        self.merc_id_counter += 1;
        self.merc_id_counter
    }

    /// Next available mission id (increments counter).
    pub fn next_mission_id(&mut self) -> u64 {
        self.mission_id_counter += 1;
        self.mission_id_counter
    }
}

/// Operational Deep state — mercenaries, missions, and marks.
///
/// Persists across prestiges. Embedded in the character save file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepPrestige {
    /// Current Warband Marks balance.
    pub warband_marks: u32,
    /// The mercenary roster for this prestige cycle.
    #[serde(with = "roster_serde")]
    pub roster: HashMap<u64, Mercenary>,
    /// Active and recently completed missions.
    pub active_missions: Vec<Mission>,
    /// Missions available to start (mission pool).
    pub available_missions: Vec<AvailableMission>,
    /// Rotating recruit pool.
    pub recruit_pool: RecruitPool,
    /// Missions awaiting result collection by the player (completed but not yet shown).
    pub pending_results: Vec<Mission>,
    /// Which generation (prestige cycle) this prestige state belongs to.
    #[serde(default)]
    pub generation_number: u32,
    /// Log of completed missions this prestige cycle.
    #[serde(default)]
    pub warband_log: Vec<WarbandLogEntry>,
    /// Total Warband Marks earned this generation (for generation records).
    #[serde(default)]
    pub total_marks_earned: u32,
    /// Total missions completed this generation (for generation records).
    #[serde(default)]
    pub total_missions_completed: u32,
    /// Total mercs lost this generation (for generation records).
    #[serde(default)]
    pub total_mercs_lost: u32,
    /// Wall-clock time when the available mission pool was last refreshed.
    /// `None` means the pool has never been explicitly refreshed (triggers
    /// an immediate refresh on next check). Defaults to `None` on old saves
    /// for backward compatibility.
    #[serde(default)]
    pub pool_refreshed_at: Option<DateTime<Utc>>,
}

impl Default for DeepPrestige {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            warband_marks: 0,
            roster: HashMap::new(),
            active_missions: Vec::new(),
            available_missions: Vec::new(),
            recruit_pool: RecruitPool {
                candidates: Vec::new(),
                refreshed_at: now,
                recruit_costs: Vec::new(),
            },
            pending_results: Vec::new(),
            generation_number: 0,
            warband_log: Vec::new(),
            total_marks_earned: 0,
            total_missions_completed: 0,
            total_mercs_lost: 0,
            pool_refreshed_at: None,
        }
    }
}

impl DeepPrestige {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a mutable mercenary by id.
    pub fn find_merc_mut(&mut self, id: u64) -> Option<&mut Mercenary> {
        self.roster.get_mut(&id)
    }

    /// Look up a mercenary by id.
    pub fn find_merc(&self, id: u64) -> Option<&Mercenary> {
        self.roster.get(&id)
    }

    /// Look up a mutable active mission by id.
    pub fn find_mission_mut(&mut self, id: u64) -> Option<&mut Mission> {
        self.active_missions.iter_mut().find(|m| m.id == id)
    }

    /// Count of mercs currently available for assignment.
    pub fn available_merc_count(&self) -> usize {
        self.roster.values().filter(|m| m.is_available()).count()
    }

    /// Count of active (in-progress) missions.
    pub fn active_mission_count(&self) -> usize {
        self.active_missions
            .iter()
            .filter(|m| {
                matches!(
                    m.status,
                    MissionStatus::Active | MissionStatus::EventPending
                )
            })
            .count()
    }

    /// Whether any active mission has a pending event needing player input.
    pub fn has_any_pending_event(&self) -> bool {
        self.active_missions.iter().any(|m| m.has_pending_event())
    }

    /// Spend Warband Marks; returns false if insufficient.
    pub fn spend_marks(&mut self, amount: u32) -> bool {
        if self.warband_marks >= amount {
            self.warband_marks -= amount;
            true
        } else {
            false
        }
    }
}

/// Combined runtime Deep state (both tiers in one place for convenience).
///
/// `persistent` is saved to `~/.quest/deep.json`.
/// `prestige` is saved alongside the character save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepState {
    pub persistent: DeepPersistent,
    pub prestige: DeepPrestige,
}

impl DeepState {
    pub fn new() -> Self {
        Self {
            persistent: DeepPersistent::new(),
            prestige: DeepPrestige::new(),
        }
    }

    /// Reset per-prestige state while keeping persistent state intact.
    /// Call this at the start of each prestige cycle.
    pub fn on_prestige(&mut self) {
        self.persistent.generation_counter += 1;

        let record = GenerationRecord {
            generation: self.persistent.generation_counter,
            marks_earned: self.prestige.total_marks_earned,
            missions_completed: self.prestige.total_missions_completed,
            mercs_lost: self.prestige.total_mercs_lost,
            deepest_layer_reached: self.persistent.deepest_layer_reached,
            gateway_opened_this_generation: self.persistent.gateway_opened,
        };
        self.persistent.generation_records.push(record);
        if self.persistent.generation_records.len() > 10 {
            let excess = self.persistent.generation_records.len() - 10;
            self.persistent.generation_records.drain(..excess);
        }

        self.prestige.generation_number = self.persistent.generation_counter;
    }

    /// Whether The Deep is fully available (discovered and at least one mission possible).
    pub fn is_active(&self) -> bool {
        self.persistent.discovered
    }
}

impl Default for DeepState {
    fn default() -> Self {
        Self::new()
    }
}

// ── UI State ──────────────────────────────────────────────────────────────────

/// Sub-view within The Deep overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepView {
    /// Active missions in progress — progress bars, events, results.
    Active,
    /// Available missions — select and assign squad.
    NewMission,
    /// Mercenary roster — stats, status, archetype.
    Roster,
    /// Layer map and infrastructure state.
    Infrastructure,
    /// Check-in event response (shown as modal over Active).
    EventResponse,
    /// Recruit new mercs from the rotating pool.
    Recruit,
}

impl DeepView {
    /// The navigable tabs in order.
    pub const TABS: &[DeepView] = &[
        DeepView::Infrastructure,
        DeepView::Active,
        DeepView::NewMission,
        DeepView::Roster,
        DeepView::Recruit,
    ];

    /// Short label for rendering in the tab bar.
    pub fn tab_label(self) -> &'static str {
        match self {
            DeepView::Active => "Active",
            DeepView::NewMission => "Deploy",
            DeepView::Roster => "Roster",
            DeepView::Infrastructure => "Layers",
            DeepView::EventResponse => "Events",
            DeepView::Recruit => "Recruit",
        }
    }

    /// Next tab in the TABS cycle.
    pub fn next_tab(self) -> DeepView {
        let Some(pos) = Self::TABS.iter().position(|&v| v == self) else {
            return self;
        };
        Self::TABS[(pos + 1) % Self::TABS.len()]
    }

    /// Previous tab in the TABS cycle.
    pub fn prev_tab(self) -> DeepView {
        let Some(pos) = Self::TABS.iter().position(|&v| v == self) else {
            return self;
        };
        Self::TABS[(pos + Self::TABS.len() - 1) % Self::TABS.len()]
    }
}

/// Overlay state for The Deep UI (not serialized — pure runtime UI state).
pub struct DeepUiState {
    pub open: bool,
    pub view: DeepView,
    /// Selected index in whatever list is currently shown.
    pub selected_index: usize,
    /// Mission id of the mission whose event the player is responding to.
    pub event_mission_id: Option<u64>,
    /// Selected choice index within the active check-in event.
    pub event_choice_index: usize,
    /// Mission id being configured in the NewMission view.
    pub staging_mission_index: Option<usize>,
    /// Currently selected mercs for the staged mission.
    pub staged_squad: Vec<u64>,
    /// Transient error/info message shown in the footer area. Cleared on next action.
    pub flash_message: Option<String>,
    /// Per-session visit counters for progressive disclosure hints.
    /// Hints fade after N visits (e.g., 3-5). Resets on game restart.
    pub hub_visit_count: u8,
    pub mission_visit_count: u8,
    pub roster_visit_count: u8,
    pub layer_visit_count: u8,
    pub event_visit_count: u8,
    pub recruit_visit_count: u8,
    /// Whether the event response modal is open over the hub.
    pub event_modal_open: bool,
    /// Whether [?] help reference panel is shown.
    pub show_help: bool,
    /// Mercs from last prestige shown in farewell screen: (name, level, missions_completed).
    pub farewell_mercs: Vec<(String, u32, u32)>,
}

impl DeepUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            view: DeepView::Infrastructure,
            selected_index: 0,
            event_mission_id: None,
            event_choice_index: 0,
            staging_mission_index: None,
            staged_squad: Vec::new(),
            flash_message: None,
            hub_visit_count: 0,
            mission_visit_count: 0,
            roster_visit_count: 0,
            layer_visit_count: 0,
            event_visit_count: 0,
            recruit_visit_count: 0,
            event_modal_open: false,
            show_help: false,
            farewell_mercs: Vec::new(),
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.view = DeepView::Infrastructure;
        self.selected_index = 0;
        self.event_mission_id = None;
        self.event_choice_index = 0;
        self.event_modal_open = false;

        self.staging_mission_index = None;
        self.staged_squad.clear();
        self.flash_message = None;
        self.hub_visit_count = self.hub_visit_count.saturating_add(1);
        self.show_help = false;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.view = DeepView::Infrastructure;
        self.selected_index = 0;
        self.event_mission_id = None;
        self.event_choice_index = 0;
        self.event_modal_open = false;

        self.staging_mission_index = None;
        self.staged_squad.clear();
        self.flash_message = None;
        self.show_help = false;
    }
}

impl Default for DeepUiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── LayerTier ──────────────────────────────────────────────────────────────

    #[test]
    fn test_layer_tier_from_layer_bounds() {
        assert_eq!(LayerTier::from_layer(1), LayerTier::Shallows);
        assert_eq!(LayerTier::from_layer(3), LayerTier::Shallows);
        assert_eq!(LayerTier::from_layer(4), LayerTier::Warrens);
        assert_eq!(LayerTier::from_layer(7), LayerTier::Warrens);
        assert_eq!(LayerTier::from_layer(8), LayerTier::Hollows);
        assert_eq!(LayerTier::from_layer(12), LayerTier::Hollows);
        assert_eq!(LayerTier::from_layer(13), LayerTier::SunkenReach);
        assert_eq!(LayerTier::from_layer(18), LayerTier::SunkenReach);
        assert_eq!(LayerTier::from_layer(19), LayerTier::Abyss);
        assert_eq!(LayerTier::from_layer(25), LayerTier::Abyss);
        assert_eq!(LayerTier::from_layer(26), LayerTier::Void);
        assert_eq!(LayerTier::from_layer(100), LayerTier::Void);
    }

    // ── GuildRank ──────────────────────────────────────────────────────────────

    #[test]
    fn test_guild_rank_display_names() {
        assert_eq!(GuildRank(1).display_name(), "Freelancers");
        assert_eq!(GuildRank(2).display_name(), "Company");
        assert_eq!(GuildRank(3).display_name(), "Battalion");
        assert_eq!(GuildRank(4).display_name(), "Legion");
        assert_eq!(GuildRank(5).display_name(), "Vanguard");
    }

    #[test]
    fn test_guild_rank_roster_caps() {
        assert_eq!(GuildRank(1).max_roster(), 5);
        assert_eq!(GuildRank(2).max_roster(), 7);
        assert_eq!(GuildRank(3).max_roster(), 9);
        assert_eq!(GuildRank(4).max_roster(), 12);
        assert_eq!(GuildRank(5).max_roster(), 15);
    }

    #[test]
    fn test_guild_rank_concurrent_missions() {
        assert_eq!(GuildRank(1).concurrent_missions(), 1);
        assert_eq!(GuildRank(2).concurrent_missions(), 2);
        assert_eq!(GuildRank(3).concurrent_missions(), 2);
        assert_eq!(GuildRank(4).concurrent_missions(), 3);
        assert_eq!(GuildRank(5).concurrent_missions(), 4);
    }

    #[test]
    fn test_guild_rank_advancement() {
        assert!(GuildRank(1).can_advance());
        assert!(GuildRank(4).can_advance());
        assert!(!GuildRank(5).can_advance());
        assert_eq!(GuildRank(1).next(), Some(GuildRank(2)));
        assert_eq!(GuildRank(5).next(), None);
    }

    #[test]
    fn test_guild_rank_required_breakthrough() {
        assert_eq!(GuildRank(1).required_breakthrough_layer(), None);
        assert_eq!(GuildRank(2).required_breakthrough_layer(), Some(3));
        assert_eq!(GuildRank(3).required_breakthrough_layer(), Some(7));
        assert_eq!(GuildRank(4).required_breakthrough_layer(), Some(13));
        assert_eq!(GuildRank(5).required_breakthrough_layer(), Some(19));
    }

    // ── effective_concurrent_missions ────────────────────────────────────────

    #[test]
    fn test_effective_concurrent_missions_bonus_at_rank1_layer3() {
        assert_eq!(effective_concurrent_missions(GuildRank(1), 0), 1);
        assert_eq!(effective_concurrent_missions(GuildRank(1), 2), 1);
        assert_eq!(effective_concurrent_missions(GuildRank(1), 3), 2);
        assert_eq!(effective_concurrent_missions(GuildRank(1), 10), 2);
        // Higher ranks are unaffected
        assert_eq!(effective_concurrent_missions(GuildRank(2), 3), 2);
        assert_eq!(effective_concurrent_missions(GuildRank(3), 7), 2);
        assert_eq!(effective_concurrent_missions(GuildRank(5), 25), 4);
    }

    // ── MercArchetype ─────────────────────────────────────────────────────────

    #[test]
    fn test_merc_archetype_base_stats_are_differentiated() {
        for &archetype in MercArchetype::ALL {
            let (p, r, e) = archetype.base_stats();
            // All stats must be non-zero.
            assert!(p > 0, "{:?} power must be > 0", archetype);
            assert!(r > 0, "{:?} resilience must be > 0", archetype);
            assert!(e > 0, "{:?} expertise must be > 0", archetype);
        }
        // Vanguard should have highest resilience, not Scout.
        let (_, vanguard_resilience, _) = MercArchetype::Vanguard.base_stats();
        let (_, scout_resilience, _) = MercArchetype::Scout.base_stats();
        assert!(vanguard_resilience > scout_resilience);
        // Arcanist should have highest expertise.
        let (_, _, arcanist_expertise) = MercArchetype::Arcanist.base_stats();
        let (_, _, vanguard_expertise) = MercArchetype::Vanguard.base_stats();
        assert!(arcanist_expertise > vanguard_expertise);
    }

    // ── MissionType ───────────────────────────────────────────────────────────

    #[test]
    fn test_mission_type_risk_tiers() {
        assert_eq!(MissionType::SupplyRun.risk_tier(), 0);
        assert_eq!(
            MissionType::Construction(Infrastructure::Outpost).risk_tier(),
            0
        );
        assert_eq!(MissionType::Recon.risk_tier(), 1);
        assert_eq!(MissionType::Expedition.risk_tier(), 2);
        assert_eq!(MissionType::Breakthrough.risk_tier(), 3);
    }

    #[test]
    fn test_mission_type_duration_ranges_are_ordered() {
        // Minimum durations should increase with risk tier (ranges may overlap).
        // Supply Run and Recon share the same minimum at Shallows tier.
        let supply_min = MissionType::SupplyRun.duration_range_secs().0;
        let recon_min = MissionType::Recon.duration_range_secs().0;
        let expedition_min = MissionType::Expedition.duration_range_secs().0;
        let breakthrough_min = MissionType::Breakthrough.duration_range_secs().0;
        assert!(supply_min <= recon_min);
        assert!(recon_min < expedition_min);
        assert!(expedition_min < breakthrough_min);
    }

    #[test]
    fn test_mission_type_max_events() {
        assert_eq!(MissionType::SupplyRun.max_events(), 0);
        assert_eq!(
            MissionType::Construction(Infrastructure::Bridge).max_events(),
            0
        );
        assert_eq!(MissionType::Recon.max_events(), 1);
        assert_eq!(MissionType::Expedition.max_events(), 2);
        assert_eq!(MissionType::Breakthrough.max_events(), 5);
    }

    // ── Infrastructure ────────────────────────────────────────────────────────

    #[test]
    fn test_infrastructure_outpost_reduces_duration() {
        assert!(Infrastructure::Outpost.duration_reduction() > 0.0);
        assert_eq!(Infrastructure::SupplyCache.duration_reduction(), 0.0);
        assert_eq!(Infrastructure::Watchtower.duration_reduction(), 0.0);
        assert_eq!(Infrastructure::Bridge.duration_reduction(), 0.0);
    }

    // ── Layer ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_layer_duration_reduction_caps_at_75_percent() {
        // If every infrastructure type provided a reduction it would be capped.
        let layer = Layer {
            index: 1,
            tier: LayerTier::Shallows,
            theme_name: "Test".to_string(),
            difficulty: 1.0,
            familiarity: 0,
            // Simulate four outposts (normally impossible — just tests the cap).
            infrastructure: vec![
                Infrastructure::Outpost,
                Infrastructure::Outpost,
                Infrastructure::Outpost,
                Infrastructure::Outpost,
            ],
            cleared: false,
        };
        assert_eq!(layer.total_duration_reduction(), 0.75);
    }

    #[test]
    fn test_layer_has_infrastructure() {
        let layer = Layer {
            index: 1,
            tier: LayerTier::Shallows,
            theme_name: "Test".to_string(),
            difficulty: 1.0,
            familiarity: 0,
            infrastructure: vec![Infrastructure::Outpost],
            cleared: false,
        };
        assert!(layer.has_infrastructure(Infrastructure::Outpost));
        assert!(!layer.has_infrastructure(Infrastructure::Watchtower));
    }

    // ── DeepPersistent ────────────────────────────────────────────────────────

    #[test]
    fn test_deep_persistent_default_state() {
        let dp = DeepPersistent::new();
        assert!(!dp.discovered);
        assert_eq!(dp.guild_rank, GuildRank(1));
        assert_eq!(dp.deepest_layer_reached, 0);
        assert!(dp.layers.is_empty());
    }

    #[test]
    fn test_deep_persistent_layer_record_grows_on_demand() {
        let mut dp = DeepPersistent::new();
        let _ = dp.layer_record_mut(3);
        assert_eq!(dp.layers.len(), 3);
        assert_eq!(dp.layers[0].index, 1);
        assert_eq!(dp.layers[2].index, 3);
    }

    #[test]
    fn test_deep_persistent_frontier_layer_starts_at_one() {
        let dp = DeepPersistent::new();
        assert_eq!(dp.frontier_layer(), 1);
    }

    #[test]
    fn test_deep_persistent_frontier_layer_advances_past_cleared() {
        let mut dp = DeepPersistent::new();
        dp.layer_record_mut(1).cleared = true;
        dp.layer_record_mut(2).cleared = true;
        // Layer 3 is not cleared — frontier should be 3.
        let _ = dp.layer_record_mut(3);
        assert_eq!(dp.frontier_layer(), 3);
    }

    #[test]
    fn test_deep_persistent_frontier_layer_when_all_cleared() {
        // Bug regression: when all known layers are cleared, frontier should advance
        // to deepest_layer_reached + 1, not reset to 1.
        let mut dp = DeepPersistent::new();
        dp.layer_record_mut(1).cleared = true;
        dp.layer_record_mut(2).cleared = true;
        dp.layer_record_mut(3).cleared = true;
        dp.deepest_layer_reached = 3;
        // All layers cleared → frontier should be 4 (the next unexplored layer).
        assert_eq!(dp.frontier_layer(), 4);
    }

    #[test]
    fn test_layer_record_index_zero_returns_none() {
        // Bug regression: index 0 previously underflowed usize::MAX via saturating_sub.
        let dp = DeepPersistent::new();
        assert!(dp.layer_record(0).is_none());
    }

    #[test]
    fn test_deep_persistent_id_counters_are_monotonic() {
        let mut dp = DeepPersistent::new();
        let id1 = dp.next_merc_id();
        let id2 = dp.next_merc_id();
        let id3 = dp.next_mission_id();
        assert!(id2 > id1);
        assert_eq!(id3, 1); // mission counter is independent
    }

    // ── DeepPrestige ──────────────────────────────────────────────────────────

    #[test]
    fn test_deep_prestige_spend_marks() {
        let mut dp = DeepPrestige::new();
        dp.warband_marks = 500;
        assert!(dp.spend_marks(200));
        assert_eq!(dp.warband_marks, 300);
        assert!(!dp.spend_marks(400)); // insufficient
        assert_eq!(dp.warband_marks, 300); // unchanged
    }

    // ── DeepState ─────────────────────────────────────────────────────────────

    #[test]
    fn test_deep_state_on_prestige_preserves_prestige_state() {
        let mut ds = DeepState::new();
        ds.persistent.discovered = true;
        ds.persistent.guild_rank = GuildRank(3);
        ds.prestige.warband_marks = 9999;
        ds.on_prestige();
        // Mercs, missions, and marks persist across prestiges.
        assert_eq!(ds.prestige.warband_marks, 9999);
        // Generation counter advances.
        assert_eq!(ds.persistent.generation_counter, 1);
        assert_eq!(ds.prestige.generation_number, 1);
        // Persistent state is untouched.
        assert!(ds.persistent.discovered);
        assert_eq!(ds.persistent.guild_rank, GuildRank(3));
    }

    #[test]
    fn test_deep_state_serde_roundtrip() {
        let mut ds = DeepState::new();
        ds.persistent.discovered = true;
        ds.persistent.guild_rank = GuildRank(2);
        ds.prestige.warband_marks = 1240;

        let json = serde_json::to_string(&ds).unwrap();
        let loaded: DeepState = serde_json::from_str(&json).unwrap();

        assert!(loaded.persistent.discovered);
        assert_eq!(loaded.persistent.guild_rank, GuildRank(2));
        assert_eq!(loaded.prestige.warband_marks, 1240);
    }

    // ── Mercenary helpers ─────────────────────────────────────────────────────

    #[test]
    fn test_mercenary_effective_stats_scale_with_level() {
        let m = Mercenary {
            id: 1,
            name: "Rodgar".to_string(),
            archetype: MercArchetype::Vanguard,
            power: 14,
            resilience: 12,
            expertise: 4,
            level: 3,
            missions_completed: 0,
            status: MercStatus::Available,
        };
        assert!(m.effective_power() > m.power);
        assert!(m.effective_resilience() > m.resilience);
    }

    #[test]
    fn test_mercenary_is_available_only_when_not_on_mission_or_injured() {
        let base = Mercenary {
            id: 1,
            name: "Rodgar".to_string(),
            archetype: MercArchetype::Vanguard,
            power: 14,
            resilience: 12,
            expertise: 4,
            level: 1,
            missions_completed: 0,
            status: MercStatus::Available,
        };
        assert!(base.is_available());
        let on_mission = Mercenary {
            status: MercStatus::OnMission(42),
            ..base.clone()
        };
        assert!(!on_mission.is_available());
        let injured = Mercenary {
            status: MercStatus::Injured {
                missions_remaining: 2,
            },
            ..base.clone()
        };
        assert!(!injured.is_available());
        let lost = Mercenary {
            status: MercStatus::Lost,
            ..base
        };
        assert!(!lost.is_available());
    }

    // ── CheckInEvent ─────────────────────────────────────────────────────────

    #[test]
    fn test_check_in_event_effective_choice_falls_back_to_auto() {
        let event = CheckInEvent {
            title: "Cave-in".to_string(),
            description: "A tunnel collapsed.".to_string(),
            choices: vec![
                EventChoice {
                    label: "Dig through".to_string(),
                    required_archetype: None,
                    time_delta_secs: 3 * 3600,
                    is_risky: false,
                    unlocks_bonus_event: false,
                    risk_percent: None,
                },
                EventChoice {
                    label: "Find alternate route".to_string(),
                    required_archetype: Some(MercArchetype::Saboteur),
                    time_delta_secs: 0,
                    is_risky: true,
                    unlocks_bonus_event: true,
                    risk_percent: Some(30),
                },
            ],
            auto_resolve_choice: 0,
            archetype_bonus: Some(MercArchetype::Saboteur),
            fired_at: Utc::now(),
            resolved_choice: None,
        };
        assert_eq!(event.effective_choice(), 0);
        assert!(!event.is_resolved());
    }

    #[test]
    fn test_check_in_event_resolved_choice_overrides_auto() {
        let mut event = CheckInEvent {
            title: "Cave-in".to_string(),
            description: "A tunnel collapsed.".to_string(),
            choices: vec![
                EventChoice {
                    label: "Dig through".to_string(),
                    required_archetype: None,
                    time_delta_secs: 3 * 3600,
                    is_risky: false,
                    unlocks_bonus_event: false,
                    risk_percent: None,
                },
                EventChoice {
                    label: "Find alternate route".to_string(),
                    required_archetype: Some(MercArchetype::Saboteur),
                    time_delta_secs: 0,
                    is_risky: true,
                    unlocks_bonus_event: true,
                    risk_percent: Some(30),
                },
            ],
            auto_resolve_choice: 0,
            archetype_bonus: Some(MercArchetype::Saboteur),
            fired_at: Utc::now(),
            resolved_choice: None,
        };
        event.resolved_choice = Some(1);
        assert_eq!(event.effective_choice(), 1);
        assert!(event.is_resolved());
    }
}
