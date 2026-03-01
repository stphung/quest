//! The Deep — Mercenary Expedition System.
//!
//! An endgame (P15+) system where players recruit and manage a mercenary company,
//! sending squads on long-duration missions (2–24 hours, real-time wall-clock) that
//! push deeper into a vast underground structure.
//!
//! # Persistence Model
//!
//! Two-tier persistence, mirroring the pattern used by Haven and Soulforge:
//!
//! | State | Where | Survives prestige? |
//! |---|---|---|
//! | [`DeepPersistent`] | `~/.quest/deep.json` | Yes |
//! | [`DeepPrestige`] | character save | No |
//!
//! Both tiers are combined in [`DeepState`] for convenience.
//!
//! # Module Layout
//!
//! ```text
//! src/deep/
//! ├── mod.rs          — Public API re-exports (this file)
//! ├── types.rs        — All data structures, enums, and helper methods
//! ├── generation.rs   — Merc, mission, and event generation
//! ├── logic.rs        — Mission ticking, event resolution, squad validation
//! ├── persistence.rs  — Save/load from ~/.quest/deep.json
//! └── discovery.rs    — Discovery roll logic
//! ```

pub mod discovery;
pub mod economy;
pub mod events;
pub mod facade;
pub mod layers;
pub mod mercenaries;
pub mod missions;
pub mod narratives;
pub mod persistence;
pub mod types;

// Re-export the full public surface so callers use `deep::Foo` not `deep::types::Foo`.
// Allow unused while consumers (tick.rs, UI, tests) are still being implemented.
#[allow(unused_imports)]
pub use types::{
    effective_concurrent_missions,
    // Structs
    AvailableMission,
    CheckInEvent,
    DeepPersistent,
    DeepPrestige,
    DeepState,
    DeepUiState,
    // Enums
    DeepView,
    EventChoice,
    GenerationRecord,
    GuildRank,
    Infrastructure,
    Layer,
    LayerRecord,
    LayerTier,
    MercArchetype,
    MercStatus,
    Mercenary,
    Mission,
    MissionOutcome,
    MissionResult,
    MissionStatus,
    MissionType,
    RecruitPool,
    WarbandLogEntry,
    DEEP_MIN_PRESTIGE_RANK,
    GATEWAY_LAYER,
};

pub use discovery::complete_discovery;

// Economy re-exports
#[allow(unused_imports)]
pub use economy::{
    base_marks_earned, base_xp_reward, compute_mark_reward, guild_upgrade_cost,
    marks_variance_multiplier, merc_xp_per_mission, merc_xp_to_next_level, mission_launch_cost,
    outcome_mark_multiplier, recruit_quality_distribution, stormglass_reward,
    try_upgrade_guild_rank, xp_reward, GuildUpgradeError, MarkRewardParams, RecruitQuality,
};

// Layers re-exports
#[allow(unused_imports)]
pub use layers::{
    apply_familiarity_gain, build_infrastructure, familiarity_gain, infrastructure_build_cost,
    is_frontier_layer, is_safe_layer, layer_power_thresholds, mark_layer_cleared,
    mission_duration_secs, mission_power_threshold, watchtower_auto_resolve_bonus,
    FamiliarityLevel, InfrastructureBuildError, LayerPowerThresholds,
};
#[allow(unused_imports)]
pub use mercenaries::{
    apply_merc_xp, available_mercs, generate_merc_name, generate_mercenary, generate_recruit_pool,
    generate_starter_roster, injure_merc, mark_merc_lost, purge_lost_mercs, recruit_pool_size,
    roll_recruit_cost, roll_recruit_quality, roster_has_capacity, stats_at_level, tick_merc_injury,
    xp_to_next_level, InjurySeverity, MercQuality,
};
#[allow(unused_imports)]
pub use persistence::{deep_save_path, load_deep, save_deep};

// Events re-exports
#[allow(unused_imports)]
pub use events::{
    event_trigger_points, generate_mission_events, generate_mission_events_with_names,
    resolve_event, tick_mission_events, EventCategory, EventResolution, EventTag, EventTickResult,
    AUTO_RESOLVE_TIMEOUT_HOURS,
};

// Missions re-exports
#[allow(unused_imports)]
pub use missions::{
    available_mission_count, daily_supply_run_resets_at, effective_duration_secs,
    generate_mission_pool, is_daily_supply_run_available, maybe_refresh_mission_pool,
    maybe_refresh_recruit_pool, resolve_mission, resolve_offline_missions, run_softlock_safeguards,
    start_mission, tick_all_missions, tick_mission, validate_squad_assignment, MissionTickSummary,
    OfflineResolutionSummary, SquadAssignmentError, POOL_REFRESH_INTERVAL_SECS,
};
