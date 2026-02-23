//! Check-in event system for The Deep.
//!
//! Events fire at progress milestones during missions, presenting the player with
//! choices gated by squad archetype. Auto-resolve always picks the safest option
//! after a 2-hour timeout.
//!
//! # Structure
//!
//! - Static event templates live in this file, indexed by `EventTemplateId`.
//! - `generate_mission_events()` selects and schedules events when a mission starts.
//! - `tick_mission_events()` checks triggers and applies auto-resolve on each tick.
//! - `resolve_event_choice()` applies the player's explicit choice.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::RngExt;

use super::types::{
    CheckInEvent, EventChoice, LayerTier, MercArchetype, Mission, MissionStatus, MissionType,
};

// ── Auto-resolve timeout ──────────────────────────────────────────────────────

/// Wall-clock duration before an unresolved event is auto-resolved.
pub const AUTO_RESOLVE_TIMEOUT_HOURS: i64 = 2;

// ── Event scheduling progress points ─────────────────────────────────────────

/// Progress fractions at which events fire for each mission type / layer tier.
///
/// Returns a `&'static [f64]` slice of trigger points in ascending order.
pub fn event_trigger_points(mission_type: MissionType, tier: LayerTier) -> &'static [f64] {
    match mission_type {
        MissionType::SupplyRun | MissionType::Construction(_) => &[],
        MissionType::Recon => &[0.50],
        MissionType::Expedition => &[0.33, 0.66],
        MissionType::Breakthrough => match tier {
            LayerTier::Shallows | LayerTier::Warrens => &[0.25, 0.50, 0.75],
            LayerTier::Hollows => &[0.25, 0.50, 0.75, 0.90],
            LayerTier::SunkenReach => &[0.20, 0.45, 0.70, 0.90],
            LayerTier::Abyss | LayerTier::Void => &[0.15, 0.35, 0.55, 0.75, 0.90],
        },
    }
}

// ── Event category ────────────────────────────────────────────────────────────

/// Category used to ensure diversity — no two consecutive events of the same category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    Hazard,
    Combat,
    Obstacle,
    Discovery,
    BossApproach,
}

// ── Event tags (for event chaining) ──────────────────────────────────────────

/// Tags accumulated on a mission by player choices.  Later events may check these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTag {
    TookRisk,
    ExploredSidePath,
    AlertedEnemies,
    HarvestedEnergy,
    UsedArcanePower,
}

// ── Static event template ─────────────────────────────────────────────────────

/// Fully resolved event template ready for use.  All choices are pre-defined.
struct EventTemplate {
    category: EventCategory,
    title: &'static str,
    description: &'static str,
    /// All player-visible choices for this event.
    choices: &'static [ChoiceTemplate],
    /// Index of the default safe choice used by auto-resolve (must be non-gated, non-risky).
    auto_resolve_index: usize,
    /// If a player selects the "risky" choice and it succeeds, apply this tag to the mission.
    risk_success_tag: Option<EventTag>,
    /// If a player selects the "risky" choice and it fails, apply this tag.
    risk_failure_tag: Option<EventTag>,
}

struct ChoiceTemplate {
    label: &'static str,
    required_archetype: Option<MercArchetype>,
    /// Seconds added to mission duration on this choice (0 = no delay).
    time_delta_secs: i64,
    is_risky: bool,
    /// Marks awarded when picking this choice (bonus income).
    bonus_marks: u32,
    /// Marks spent when picking this choice (cost).
    mark_cost: u32,
    /// For risky choices: base success probability (0.0–1.0).
    risky_success_chance: f64,
    /// For risky choices: injury probability on failure.
    risky_injury_on_fail: f64,
    /// Whether picking this choice triggers an event chain.
    unlocks_bonus_event: bool,
    /// Optional: effective Power modifier for the rest of the mission (1.0 = no change).
    power_modifier: f64,
}

impl ChoiceTemplate {
    const fn safe(
        label: &'static str,
        archetype: Option<MercArchetype>,
        time_secs: i64,
        bonus_marks: u32,
        mark_cost: u32,
    ) -> Self {
        Self {
            label,
            required_archetype: archetype,
            time_delta_secs: time_secs,
            is_risky: false,
            bonus_marks,
            mark_cost,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        }
    }

    const fn risky(
        label: &'static str,
        time_secs: i64,
        success_chance: f64,
        injury_on_fail: f64,
        bonus_marks: u32,
        chain: bool,
    ) -> Self {
        Self {
            label,
            required_archetype: None,
            time_delta_secs: time_secs,
            is_risky: true,
            bonus_marks,
            mark_cost: 0,
            risky_success_chance: success_chance,
            risky_injury_on_fail: injury_on_fail,
            unlocks_bonus_event: chain,
            power_modifier: 1.0,
        }
    }
}

// ── The Shallows (L1-3) ───────────────────────────────────────────────────────

static FLOODED_PASSAGE: EventTemplate = EventTemplate {
    category: EventCategory::Obstacle,
    title: "FLOODED PASSAGE",
    description: "The path ahead is submerged in dark water. Something moves beneath the surface.",
    choices: &[
        ChoiceTemplate::safe("Wade through carefully", None, 2 * 3600, 0, 0),
        ChoiceTemplate::risky("Swim across quickly", 0, 0.65, 0.20, 0, true),
        ChoiceTemplate::safe(
            "Find higher ground",
            Some(MercArchetype::Scout),
            1 * 3600,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: Some(EventTag::TookRisk),
    risk_failure_tag: None,
};

static CRUMBLING_DESCENT: EventTemplate = EventTemplate {
    category: EventCategory::Obstacle,
    title: "CRUMBLING DESCENT",
    description: "A long stone stairway descends into darkness. Cracks spider across every step. One wrong move and the whole thing comes down.",
    choices: &[
        ChoiceTemplate::safe("Descend one at a time", None, 2 * 3600, 0, 0),
        ChoiceTemplate::safe(
            "Reinforce and cross",
            Some(MercArchetype::Saboteur),
            30 * 60,
            0,
            20,
        ),
        ChoiceTemplate::risky("Jump to the landing", 0, 0.60, 0.25, 0, false),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static CRUMBLING_BRIDGE: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "CRUMBLING BRIDGE",
    description: "A stone bridge spans a chasm filled with stagnant water. The supports are cracked. It might hold — or it might not.",
    choices: &[
        ChoiceTemplate::safe("Cross slowly, single-file", None, 90 * 60, 0, 0),
        ChoiceTemplate::safe("Find another way around", None, 3 * 3600, 0, 0),
        ChoiceTemplate::safe(
            "Reinforce with magic",
            Some(MercArchetype::Arcanist),
            0,
            0,
            15,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static SUBTERRANEAN_NEST: EventTemplate = EventTemplate {
    category: EventCategory::Combat,
    title: "SUBTERRANEAN NEST",
    description: "Your squad stumbles into a nest of pale, eyeless creatures. They skitter away from the light but there are dozens of them blocking the tunnel ahead.",
    choices: &[
        ChoiceTemplate::safe("Wait for them to disperse", None, 2 * 3600, 0, 0),
        ChoiceTemplate::risky("Push through aggressively", 0, 0.70, 0.15, 0, false),
        ChoiceTemplate::safe(
            "Shield wall advance",
            Some(MercArchetype::Vanguard),
            30 * 60,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// Boss approach for Shallows
static SHALLOWS_BOSS: EventTemplate = EventTemplate {
    category: EventCategory::BossApproach,
    title: "THE GUARDIAN'S CHAMBER",
    description: "A massive stone guardian blocks the descent. Its eyes glow with cold blue light.",
    choices: &[
        ChoiceTemplate::safe("Direct assault", None, 0, 0, 0),
        ChoiceTemplate {
            label: "Tactical approach",
            required_archetype: Some(MercArchetype::Scout),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.15, // +15% effective Power
        },
        ChoiceTemplate::safe("Retreat", None, 0, 0, 0), // index 2 = abort
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// ── The Warrens (L4-7) ────────────────────────────────────────────────────────

static TUNNEL_AMBUSH: EventTemplate = EventTemplate {
    category: EventCategory::Combat,
    title: "AMBUSH IN THE WARRENS",
    description: "Shapes materialize from alcoves in the tunnel walls. Your squad is surrounded on three sides.",
    choices: &[
        ChoiceTemplate::safe(
            "Defensive formation",
            Some(MercArchetype::Vanguard),
            2 * 3600,
            0,
            0,
        ),
        ChoiceTemplate::risky("Fight through", 0, 0.65, 0.30, 60, false),
        ChoiceTemplate {
            label: "Set trap and counter-ambush",
            required_archetype: Some(MercArchetype::Saboteur),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 30,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        },
        // Hidden safe fallback (index 3): always available, worst option
        ChoiceTemplate::safe("Retreat and find another route", None, 3 * 3600, 0, 0),
    ],
    auto_resolve_index: 3, // non-gated safe fallback
    risk_success_tag: None,
    risk_failure_tag: None,
};

static POISON_GAS_VENT: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "TOXIC VENT",
    description: "A fissure in the tunnel floor belches noxious green vapor. The gas is heavier than air, pooling in the corridor ahead.",
    choices: &[
        ChoiceTemplate::safe("Wait for the vent to subside", None, 3 * 3600, 0, 0),
        ChoiceTemplate::safe(
            "Purify the air",
            Some(MercArchetype::Arcanist),
            0,
            0,
            25,
        ),
        ChoiceTemplate::risky("Rush through with held breath", 0, 0.60, 0.35, 0, false),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: Some(EventTag::TookRisk), // failed rush weakens squad
};

static COLLAPSING_TUNNEL: EventTemplate = EventTemplate {
    category: EventCategory::Obstacle,
    title: "CAVE-IN AHEAD",
    description: "Your squad encounters a collapsed tunnel blocking the main path. Rubble fills the corridor ceiling-to-floor.",
    choices: &[
        ChoiceTemplate::safe(
            "Dig through",
            Some(MercArchetype::Vanguard),
            3 * 3600,
            0,
            0,
        ),
        ChoiceTemplate::safe(
            "Find alternate route",
            Some(MercArchetype::Saboteur),
            0,
            0,
            0,
        ),
        ChoiceTemplate::safe(
            "Blast through",
            Some(MercArchetype::Arcanist),
            1 * 3600,
            0,
            30,
        ),
        // Non-gated fallback — always available
        ChoiceTemplate::safe("Dig through (slow)", None, 4 * 3600, 0, 0),
    ],
    auto_resolve_index: 3,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static FORK_IN_PATH: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "BRANCHING TUNNELS",
    description: "The tunnel splits into three passages. One is wide and well-traveled. One is narrow and dark. One has faint markings on the walls.",
    choices: &[
        ChoiceTemplate::safe("Take the wide passage", None, 0, 0, 0),
        ChoiceTemplate::risky("Explore the narrow path", 1 * 3600, 0.65, 0.15, 40, true),
        ChoiceTemplate {
            label: "Follow the markings",
            required_archetype: Some(MercArchetype::Scout),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 20,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        },
    ],
    auto_resolve_index: 0,
    risk_success_tag: Some(EventTag::ExploredSidePath),
    risk_failure_tag: None,
};

// Boss approach for Warrens
static WARRENS_BOSS: EventTemplate = EventTemplate {
    category: EventCategory::BossApproach,
    title: "THE OVERSEER'S DEN",
    description: "The Warrens Overseer is a massive burrowing creature. Its lair is a tangle of tunnels within tunnels.",
    choices: &[
        ChoiceTemplate::safe("Direct assault", None, 0, 0, 0),
        ChoiceTemplate {
            label: "Collapse tunnels",
            required_archetype: Some(MercArchetype::Saboteur),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.20,
        },
        ChoiceTemplate {
            label: "Lure and trap",
            required_archetype: Some(MercArchetype::Scout),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 30,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.10,
        },
        ChoiceTemplate::safe("Retreat", None, 0, 0, 0),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// ── The Hollows (L8-12) ───────────────────────────────────────────────────────

static TOXIC_SPORE_CLOUD: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "TOXIC SPORE CLOUD",
    description: "Dense clouds of luminous spores fill the cavern. Your squad is choking. The spores corrode metal and burn exposed skin.",
    choices: &[
        ChoiceTemplate {
            label: "Push through with cloth masks",
            required_archetype: None,
            time_delta_secs: 3 * 3600,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 0.95, // -5% power debuff for rest of mission
        },
        ChoiceTemplate::safe(
            "Arcane Barrier",
            Some(MercArchetype::Arcanist),
            0,
            0,
            30,
        ),
        ChoiceTemplate::safe(
            "Find ventilation shaft",
            Some(MercArchetype::Scout),
            1 * 3600,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static CRYSTAL_FORMATION: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "UNSTABLE CRYSTAL FORMATION",
    description: "A cluster of glowing crystals pulses with contained energy. Beautiful — and potentially volatile.",
    choices: &[
        ChoiceTemplate::safe("Navigate carefully", None, 2 * 3600, 0, 0),
        ChoiceTemplate {
            label: "Harvest crystal energy",
            required_archetype: Some(MercArchetype::Arcanist),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 50,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: true, // triggers HarvestedEnergy chain
            power_modifier: 1.0,
        },
        ChoiceTemplate::risky("Smash through", 0, 0.60, 0.25, 30, false),
    ],
    auto_resolve_index: 0,
    risk_success_tag: Some(EventTag::HarvestedEnergy),
    risk_failure_tag: None,
};

static HOLLOW_STALKERS: EventTemplate = EventTemplate {
    category: EventCategory::Combat,
    title: "HOLLOW STALKERS",
    description: "Three eyeless predators emerge from the shadows. They've been tracking your squad for hours.",
    choices: &[
        ChoiceTemplate::safe(
            "Defensive formation",
            Some(MercArchetype::Vanguard),
            2 * 3600,
            0,
            0,
        ),
        ChoiceTemplate::risky("Fight through", 0, 0.65, 0.0, 60, false), // injury tracked via failure_tag
        ChoiceTemplate {
            label: "Set trap and ambush",
            required_archetype: Some(MercArchetype::Saboteur),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 30,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        },
        ChoiceTemplate::safe("Retreat slowly", None, 3 * 3600, 0, 0),
    ],
    auto_resolve_index: 3,
    risk_success_tag: None,
    risk_failure_tag: Some(EventTag::AlertedEnemies),
};

static ABYSSAL_ECHO: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "ECHOES FROM BELOW",
    description: "A deep vibration rolls through the cavern. The walls pulse with faint light. Something vast stirs in the layers below, and its attention has brushed against your squad.",
    choices: &[
        ChoiceTemplate::safe("Wait for it to pass", None, 2 * 3600, 0, 0),
        ChoiceTemplate::safe(
            "Ward the squad",
            Some(MercArchetype::Arcanist),
            0,
            0,
            40,
        ),
        ChoiceTemplate::safe(
            "Medicate the shaken",
            Some(MercArchetype::Medic),
            30 * 60,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// Boss approach for Hollows
static HOLLOWS_BOSS: EventTemplate = EventTemplate {
    category: EventCategory::BossApproach,
    title: "THE SENTINEL'S CHAMBER",
    description: "A massive stone sentinel blocks the descent. Your squad is battered but determined.",
    choices: &[
        ChoiceTemplate::safe("Direct assault", None, 0, 0, 0),
        ChoiceTemplate {
            label: "Tactical approach",
            required_archetype: Some(MercArchetype::Scout),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.15,
        },
        ChoiceTemplate {
            label: "Arcane disruption",
            required_archetype: Some(MercArchetype::Arcanist),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 40,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.20,
        },
        ChoiceTemplate::safe("Retreat", None, 0, 0, 0),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// ── The Sunken Reach (L13-18) ─────────────────────────────────────────────────

static FLOODED_VAULT: EventTemplate = EventTemplate {
    category: EventCategory::Obstacle,
    title: "THE DROWNED VAULT",
    description: "An ancient vault lies submerged beneath black water. The mission path runs through it. The water is cold, deep, and wrong.",
    choices: &[
        ChoiceTemplate::risky("Swim through", 0, 0.55, 0.30, 0, false),
        ChoiceTemplate::safe(
            "Drain the vault",
            Some(MercArchetype::Saboteur),
            2 * 3600,
            0,
            50,
        ),
        ChoiceTemplate::safe(
            "Ward against the water",
            Some(MercArchetype::Arcanist),
            1 * 3600,
            0,
            40,
        ),
        ChoiceTemplate::safe("Find a way around", None, 4 * 3600, 0, 0),
    ],
    auto_resolve_index: 3,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static CORRUPTED_ARTIFACT: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "CORRUPTED RELIC",
    description: "A pedestal holds a dark artifact radiating waves of distortion. Your equipment buzzes. The air tastes like copper.",
    choices: &[
        ChoiceTemplate::safe("Leave it alone", None, 0, 0, 0),
        ChoiceTemplate {
            label: "Purify and claim",
            required_archetype: Some(MercArchetype::Arcanist),
            time_delta_secs: 1 * 3600,
            is_risky: false,
            bonus_marks: 80,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: true,
            power_modifier: 1.0,
        },
        ChoiceTemplate::risky("Grab and go", 0, 0.55, 0.40, 60, false),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static SUNKEN_GUARDIAN: EventTemplate = EventTemplate {
    category: EventCategory::Combat,
    title: "THE SUNKEN GUARDIAN WAKES",
    description: "A massive stone figure rises from the flooded floor. Ancient runes flare to life across its body. It blocks the only passage forward.",
    choices: &[
        ChoiceTemplate::risky("Fight the Guardian", 0, 0.50, 0.35, 0, false),
        ChoiceTemplate::safe(
            "Disable its runes",
            Some(MercArchetype::Arcanist),
            1 * 3600,
            0,
            60,
        ),
        ChoiceTemplate::safe(
            "Shield wall and endure",
            Some(MercArchetype::Vanguard),
            3 * 3600,
            0,
            0,
        ),
        ChoiceTemplate::safe(
            "Find the bypass tunnel",
            Some(MercArchetype::Scout),
            2 * 3600,
            0,
            0,
        ),
        ChoiceTemplate::safe("Wait for it to power down", None, 5 * 3600, 0, 0),
    ],
    auto_resolve_index: 4,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static TIDAL_SURGE: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "TIDAL SURGE",
    description: "A rhythmic booming echoes from below. The water level is rising — fast. Your squad has minutes before this section floods completely.",
    choices: &[
        ChoiceTemplate::safe("Climb to high ground", None, 2 * 3600, 0, 0),
        ChoiceTemplate::risky("Sprint through before flood", 0, 0.65, 0.25, 0, false),
        ChoiceTemplate::safe(
            "Seal the water source",
            Some(MercArchetype::Saboteur),
            30 * 60,
            0,
            30,
        ),
        ChoiceTemplate::safe(
            "Stabilize the wounded",
            Some(MercArchetype::Medic),
            1 * 3600,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// Boss approach for Sunken Reach
static SUNKEN_REACH_BOSS: EventTemplate = EventTemplate {
    category: EventCategory::BossApproach,
    title: "THE DROWNED KING",
    description: "The Drowned King sits on a throne of coral and bone in a flooded throne room. The water here obeys its will.",
    choices: &[
        ChoiceTemplate {
            label: "Direct assault",
            required_archetype: None,
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 0.90, // fighting in water is hard
        },
        ChoiceTemplate {
            label: "Drain the chamber",
            required_archetype: Some(MercArchetype::Saboteur),
            time_delta_secs: 1 * 3600,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        },
        ChoiceTemplate {
            label: "Arcane counter",
            required_archetype: Some(MercArchetype::Arcanist),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 60,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.15,
        },
        ChoiceTemplate::safe(
            "Medical preparation",
            Some(MercArchetype::Medic),
            30 * 60,
            0,
            0,
        ),
        ChoiceTemplate::safe("Retreat", None, 0, 0, 0),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// ── The Abyss (L19-25) ────────────────────────────────────────────────────────

static ABYSSAL_RIFT: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "ABYSSAL RIFT",
    description: "A crack in reality shimmers before the squad. Strange energies seep through. The air hums with power — and danger.",
    choices: &[
        ChoiceTemplate::safe("Seal the rift", None, 2 * 3600, 0, 0),
        ChoiceTemplate::risky("Harvest the energy", 0, 0.55, 0.30, 80, true),
        ChoiceTemplate {
            label: "Channel through",
            required_archetype: Some(MercArchetype::Arcanist),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 50,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        },
    ],
    auto_resolve_index: 0,
    risk_success_tag: Some(EventTag::HarvestedEnergy),
    risk_failure_tag: None,
};

static VOID_SWARM: EventTemplate = EventTemplate {
    category: EventCategory::Combat,
    title: "VOID SWARM",
    description: "The darkness ahead isn't empty — it's alive. A swarm of void fragments converges on your position, each one a splinter of unmade reality.",
    choices: &[
        ChoiceTemplate::safe(
            "Turtle formation",
            Some(MercArchetype::Vanguard),
            3 * 3600,
            0,
            0,
        ),
        ChoiceTemplate::safe(
            "Purging fire",
            Some(MercArchetype::Arcanist),
            0,
            0,
            70,
        ),
        ChoiceTemplate::risky("Sprint through the swarm", 0, 0.50, 0.40, 0, false),
        ChoiceTemplate::safe("Slow retreat", None, 4 * 3600, 0, 0),
    ],
    auto_resolve_index: 3,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static REALITY_FRACTURE: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "REALITY FRACTURE",
    description: "Space folds. The tunnel ahead exists in two states simultaneously — one path leads forward, the other loops back on itself. Your squad can feel their thoughts splitting.",
    choices: &[
        ChoiceTemplate {
            label: "Anchor with willpower",
            required_archetype: None,
            time_delta_secs: 3 * 3600,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 0.95, // minor disorientation debuff
        },
        ChoiceTemplate {
            label: "Navigate the fracture",
            required_archetype: Some(MercArchetype::Scout),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 30,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0,
        },
        ChoiceTemplate::safe(
            "Stabilize reality",
            Some(MercArchetype::Arcanist),
            1 * 3600,
            0,
            50,
        ),
        ChoiceTemplate::safe(
            "Meditate through it",
            Some(MercArchetype::Medic),
            2 * 3600,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0, // non-gated safe fallback (with minor debuff)
    risk_success_tag: None,
    risk_failure_tag: None,
};

static WHISPERING_DARK: EventTemplate = EventTemplate {
    category: EventCategory::Hazard,
    title: "THE WHISPERING DARK",
    description: "In the deepest reaches, the darkness whispers. Not sound — thought. It offers knowledge, power, a way forward. All it asks is a small price.",
    choices: &[
        ChoiceTemplate::safe("Ignore the whispers", None, 2 * 3600, 0, 0),
        ChoiceTemplate::risky("Listen and bargain", 0, 0.50, 0.50, 100, false),
        ChoiceTemplate::safe(
            "Ward against intrusion",
            Some(MercArchetype::Arcanist),
            0,
            0,
            80,
        ),
        ChoiceTemplate::safe(
            "Medic counter-measure",
            Some(MercArchetype::Medic),
            1 * 3600,
            0,
            0,
        ),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// Boss approach for Abyss
static ABYSS_BOSS: EventTemplate = EventTemplate {
    category: EventCategory::BossApproach,
    title: "THE VOID WARDEN",
    description: "Something vast and formless guards the threshold between layers. It has no shape, no weakness you can see. But it can be broken — everything down here can be broken, if you push hard enough.",
    choices: &[
        ChoiceTemplate {
            label: "Direct assault",
            required_archetype: None,
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 0.85, // void saps strength
        },
        ChoiceTemplate {
            label: "Void ward",
            required_archetype: Some(MercArchetype::Arcanist),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 80,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.0, // counters the void debuff
        },
        ChoiceTemplate {
            label: "Find the anchor point",
            required_archetype: Some(MercArchetype::Scout),
            time_delta_secs: 0,
            is_risky: false,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.0,
            risky_injury_on_fail: 0.0,
            unlocks_bonus_event: false,
            power_modifier: 1.10,
        },
        ChoiceTemplate::safe(
            "Medical preparation",
            Some(MercArchetype::Medic),
            30 * 60,
            0,
            0,
        ),
        ChoiceTemplate {
            label: "All-out assault",
            required_archetype: None,
            time_delta_secs: 0,
            is_risky: true,
            bonus_marks: 0,
            mark_cost: 0,
            risky_success_chance: 0.70,
            risky_injury_on_fail: 0.20,
            unlocks_bonus_event: false,
            power_modifier: 1.25, // glass cannon
        },
        ChoiceTemplate::safe("Retreat", None, 0, 0, 0),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

// ── Event pools by tier ───────────────────────────────────────────────────────

static SHALLOWS_POOL: [&EventTemplate; 4] = [
    &FLOODED_PASSAGE,
    &CRUMBLING_DESCENT,
    &CRUMBLING_BRIDGE,
    &SUBTERRANEAN_NEST,
];
static WARRENS_POOL: [&EventTemplate; 4] = [
    &TUNNEL_AMBUSH,
    &POISON_GAS_VENT,
    &COLLAPSING_TUNNEL,
    &FORK_IN_PATH,
];
static HOLLOWS_POOL: [&EventTemplate; 4] = [
    &TOXIC_SPORE_CLOUD,
    &CRYSTAL_FORMATION,
    &HOLLOW_STALKERS,
    &ABYSSAL_ECHO,
];
static SUNKEN_REACH_POOL: [&EventTemplate; 4] = [
    &FLOODED_VAULT,
    &CORRUPTED_ARTIFACT,
    &SUNKEN_GUARDIAN,
    &TIDAL_SURGE,
];
// Void reuses Abyss templates (scaled by caller)
static ABYSS_POOL: [&EventTemplate; 4] = [
    &ABYSSAL_RIFT,
    &VOID_SWARM,
    &REALITY_FRACTURE,
    &WHISPERING_DARK,
];

/// Non-boss events available for a given tier (in the order they're defined above).
fn tier_event_pool(tier: LayerTier) -> &'static [&'static EventTemplate] {
    match tier {
        LayerTier::Shallows => &SHALLOWS_POOL,
        LayerTier::Warrens => &WARRENS_POOL,
        LayerTier::Hollows => &HOLLOWS_POOL,
        LayerTier::SunkenReach => &SUNKEN_REACH_POOL,
        LayerTier::Abyss | LayerTier::Void => &ABYSS_POOL,
    }
}

fn tier_boss_template(tier: LayerTier) -> &'static EventTemplate {
    match tier {
        LayerTier::Shallows => &SHALLOWS_BOSS,
        LayerTier::Warrens => &WARRENS_BOSS,
        LayerTier::Hollows => &HOLLOWS_BOSS,
        LayerTier::SunkenReach => &SUNKEN_REACH_BOSS,
        LayerTier::Abyss | LayerTier::Void => &ABYSS_BOSS,
    }
}

// ── Void scaling helpers ──────────────────────────────────────────────────────

/// Layer-dependent scale factors for Void layers (L26+).
///
/// Returns (time_scale, mark_cost_scale, injury_scale) for a given layer.
fn void_scale(layer: u32) -> (f64, f64, f64) {
    if layer <= 25 {
        return (1.0, 1.0, 1.0);
    }
    let levels_above = (layer - 25) as f64;
    let time_scale = 1.0 + levels_above * 0.10;
    let mark_scale = 1.0 + levels_above * 0.15;
    let injury_scale = 1.0 + levels_above * 0.02;
    (time_scale, mark_scale, injury_scale)
}

// ── Conversion: EventTemplate -> CheckInEvent ─────────────────────────────────

fn template_to_check_in_event(template: &EventTemplate, layer: u32, now: DateTime<Utc>) -> CheckInEvent {
    let (time_scale, mark_scale, _injury_scale) = void_scale(layer);

    let choices: Vec<EventChoice> = template
        .choices
        .iter()
        .map(|c| {
            let scaled_time = (c.time_delta_secs as f64 * time_scale) as i64;
            let _scaled_cost = (c.mark_cost as f64 * mark_scale) as u32;
            EventChoice {
                label: c.label.to_string(),
                required_archetype: c.required_archetype,
                time_delta_secs: scaled_time,
                is_risky: c.is_risky,
                unlocks_bonus_event: c.unlocks_bonus_event,
            }
        })
        .collect();

    CheckInEvent {
        title: template.title.to_string(),
        description: template.description.to_string(),
        choices,
        auto_resolve_choice: template.auto_resolve_index,
        archetype_bonus: template
            .choices
            .iter()
            .find(|c| c.required_archetype.is_some())
            .and_then(|c| c.required_archetype),
        fired_at: now,
        resolved_choice: None,
    }
}

// ── Public: generate mission events ──────────────────────────────────────────

/// Generate and pre-schedule all check-in events for a newly started mission.
///
/// Events are stored in `mission.events` with `resolved_choice = None`.
/// They will be triggered progressively as the mission advances.
///
/// For Breakthrough missions the last event is always a BossApproach event.
/// Earlier events are drawn from the tier pool with category diversity (no two
/// consecutive events of the same category).
pub fn generate_mission_events(
    mission_type: MissionType,
    layer: u32,
    squad_archetypes: &[MercArchetype],
    rng: &mut impl Rng,
) -> Vec<CheckInEvent> {
    let tier = LayerTier::from_layer(layer);
    let triggers = event_trigger_points(mission_type, tier);
    if triggers.is_empty() {
        return Vec::new();
    }

    let pool = tier_event_pool(tier);
    let is_breakthrough = mission_type == MissionType::Breakthrough;
    let now = Utc::now();

    let mut events = Vec::with_capacity(triggers.len());
    let mut last_category: Option<EventCategory> = None;
    // Track which pool indices we've used to avoid exact repetition.
    let mut used: Vec<usize> = Vec::new();

    for (i, _trigger) in triggers.iter().enumerate() {
        let is_last = i == triggers.len() - 1;

        let template: &EventTemplate = if is_last && is_breakthrough {
            tier_boss_template(tier)
        } else {
            // Pick from pool: prefer different category, prefer unused, then any.
            let candidate = pool
                .iter()
                .enumerate()
                .filter(|(idx, t)| {
                    t.category != EventCategory::BossApproach
                        && Some(t.category) != last_category
                        && !used.contains(idx)
                })
                .map(|(_, t)| *t)
                .next()
                .or_else(|| {
                    // Fallback: allow same category but avoid exact repeat
                    pool.iter()
                        .enumerate()
                        .filter(|(idx, t)| {
                            t.category != EventCategory::BossApproach && !used.contains(idx)
                        })
                        .map(|(_, t)| *t)
                        .next()
                })
                .unwrap_or(pool[rng.random_range(0..pool.len())]);
            candidate
        };

        // Mark as used (find index in pool).
        if let Some(idx) = pool.iter().position(|t| std::ptr::eq(*t, template)) {
            used.push(idx);
        }
        last_category = Some(template.category);

        // Filter choices to those available given the squad composition.
        // We keep all choices in the event — availability is validated at response time.
        // However, we must ensure auto_resolve_choice is always a non-gated option.
        let mut event = template_to_check_in_event(template, layer, now);

        // Sanity: ensure auto_resolve_choice is not archetype-gated.
        // If it is (shouldn't happen by design), fall back to first non-gated choice.
        if event
            .choices
            .get(event.auto_resolve_choice)
            .and_then(|c| c.required_archetype)
            .is_some()
        {
            if let Some(safe_idx) = event
                .choices
                .iter()
                .position(|c| c.required_archetype.is_none() && !c.is_risky)
            {
                event.auto_resolve_choice = safe_idx;
            }
        }

        // Determine archetype_bonus: the archetype that provides the best gated option,
        // prioritising those present in the squad.
        event.archetype_bonus = template
            .choices
            .iter()
            .filter_map(|c| c.required_archetype)
            .find(|a| squad_archetypes.contains(a))
            .or_else(|| {
                template
                    .choices
                    .iter()
                    .filter_map(|c| c.required_archetype)
                    .next()
            });

        events.push(event);
    }

    events
}

// ── Outcome resolution helpers ────────────────────────────────────────────────

/// Result of resolving a single check-in event choice.
#[derive(Debug, Clone)]
pub struct EventResolution {
    /// Seconds to add to the mission end time (may be 0).
    pub time_delta_secs: i64,
    /// Warband Marks awarded immediately.
    pub bonus_marks: u32,
    /// Warband Marks spent immediately.
    pub mark_cost: u32,
    /// Whether a squad member should be rolled for injury.
    pub may_injure: bool,
    /// Base injury probability (0.0 = no risk).
    pub injury_chance: f64,
    /// Effective Power modifier to apply for the rest of the mission.
    pub power_modifier: f64,
    /// Whether this choice triggers a chain event tag.
    pub chain_tag: Option<EventTag>,
    /// True when this resolution came from the auto-resolve timer.
    pub was_auto_resolved: bool,
}

/// Resolve a player-selected or auto-resolved choice for an event.
///
/// Returns `None` if the choice index is out of bounds or the choice requires
/// an archetype that the squad doesn't have.
///
/// # Auto-resolve
///
/// Pass `choice_index = None` to trigger the auto-resolve path (safest option,
/// never picks risky or archetype-gated choices).
pub fn resolve_event(
    event: &mut CheckInEvent,
    choice_index: Option<usize>, // None = auto-resolve
    squad_archetypes: &[MercArchetype],
    rng: &mut impl Rng,
) -> Option<EventResolution> {
    // Determine which choice to apply.
    let effective_index = match choice_index {
        Some(idx) => {
            // Validate: index in bounds.
            if idx >= event.choices.len() {
                return None;
            }
            // Validate: if gated, squad must have the archetype.
            if let Some(required) = event.choices[idx].required_archetype {
                if !squad_archetypes.contains(&required) {
                    return None;
                }
            }
            idx
        }
        None => event.auto_resolve_choice,
    };

    let was_auto_resolved = choice_index.is_none();
    event.resolved_choice = Some(effective_index);

    let choice = &event.choices[effective_index];

    // Find the corresponding static template data for outcome calculation.
    // We use the EventChoice data we have, supplemented by template data for
    // risky outcomes which aren't stored on CheckInEvent directly.
    // Look up risky data from the static template pools by matching title.
    let (risky_success_chance, risky_injury_on_fail, risk_success_tag, chain_tag_on_success) =
        find_template_risky_data(&event.title, effective_index);

    let (time_delta, bonus_marks, mark_cost, may_injure, injury_chance, power_modifier, chain_tag) =
        if choice.is_risky && !was_auto_resolved {
            // Roll the risky outcome.
            let success = rng.random::<f64>() < risky_success_chance;
            if success {
                (
                    choice.time_delta_secs,
                    // Risky successes may have bonus marks baked into template; use 0
                    // if EventChoice doesn't carry marks (it does via bonus_marks field
                    // but CheckInEvent only stores the choice label).
                    0u32, // resolved via template below
                    0u32,
                    false,
                    0.0f64,
                    1.0f64,
                    if choice.unlocks_bonus_event {
                        chain_tag_on_success
                    } else {
                        None
                    },
                )
            } else {
                (
                    choice.time_delta_secs + 3600, // failure adds 1h extra delay
                    0u32,
                    0u32,
                    true,
                    risky_injury_on_fail,
                    1.0f64,
                    None,
                )
            }
        } else {
            // Safe choice.
            (
                choice.time_delta_secs,
                0u32,
                0u32,
                false,
                0.0f64,
                1.0f64, // power_modifier stored in template, not CheckInEvent
                None,
            )
        };

    // Retrieve power_modifier and bonus_marks from the static template.
    let (tpl_power_mod, tpl_bonus_marks) =
        find_template_choice_extras(&event.title, effective_index);

    Some(EventResolution {
        time_delta_secs: time_delta,
        bonus_marks: bonus_marks + tpl_bonus_marks,
        mark_cost,
        may_injure,
        injury_chance,
        power_modifier: if tpl_power_mod != 1.0 {
            tpl_power_mod
        } else {
            power_modifier
        },
        chain_tag: chain_tag.or(risk_success_tag.filter(|_| !was_auto_resolved)),
        was_auto_resolved,
    })
}

// ── Template lookup helpers ───────────────────────────────────────────────────
// These functions look up static template data by event title string.
// We do this rather than storing raw template references in CheckInEvent
// (which is serialized) to keep the save format clean.

/// All event templates in one slice for lookup.
static ALL_TEMPLATES: [&EventTemplate; 25] = [
    &FLOODED_PASSAGE,
    &CRUMBLING_DESCENT,
    &CRUMBLING_BRIDGE,
    &SUBTERRANEAN_NEST,
    &SHALLOWS_BOSS,
    &TUNNEL_AMBUSH,
    &POISON_GAS_VENT,
    &COLLAPSING_TUNNEL,
    &FORK_IN_PATH,
    &WARRENS_BOSS,
    &TOXIC_SPORE_CLOUD,
    &CRYSTAL_FORMATION,
    &HOLLOW_STALKERS,
    &ABYSSAL_ECHO,
    &HOLLOWS_BOSS,
    &FLOODED_VAULT,
    &CORRUPTED_ARTIFACT,
    &SUNKEN_GUARDIAN,
    &TIDAL_SURGE,
    &SUNKEN_REACH_BOSS,
    &ABYSSAL_RIFT,
    &VOID_SWARM,
    &REALITY_FRACTURE,
    &WHISPERING_DARK,
    &ABYSS_BOSS,
];

fn all_templates() -> &'static [&'static EventTemplate] {
    &ALL_TEMPLATES
}

fn find_template_by_title(title: &str) -> Option<&'static EventTemplate> {
    all_templates()
        .iter()
        .find(|t| t.title == title)
        .copied()
}

fn find_template_risky_data(
    title: &str,
    choice_index: usize,
) -> (f64, f64, Option<EventTag>, Option<EventTag>) {
    let Some(template) = find_template_by_title(title) else {
        return (0.65, 0.20, None, None);
    };
    let Some(choice) = template.choices.get(choice_index) else {
        return (0.65, 0.20, None, None);
    };
    (
        choice.risky_success_chance,
        choice.risky_injury_on_fail,
        template.risk_failure_tag,
        template.risk_success_tag,
    )
}

fn find_template_choice_extras(title: &str, choice_index: usize) -> (f64, u32) {
    let Some(template) = find_template_by_title(title) else {
        return (1.0, 0);
    };
    let Some(choice) = template.choices.get(choice_index) else {
        return (1.0, 0);
    };
    (choice.power_modifier, choice.bonus_marks)
}

// ── Public: mission event ticking ─────────────────────────────────────────────

/// Events produced while ticking mission events.
#[derive(Debug, Clone)]
pub struct EventTickResult {
    /// Events that newly became pending this tick (player should be notified).
    pub newly_pending: Vec<usize>, // indices into mission.events
    /// Events that were auto-resolved this tick.
    pub auto_resolved: Vec<(usize, EventResolution)>,
}

/// Check mission event triggers and apply auto-resolve on expired pending events.
///
/// Call this each tick (100ms) while the game is running, and also on game load
/// for offline resolution.
///
/// Returns what changed so the caller can update mission status and rewards.
pub fn tick_mission_events(
    mission: &mut Mission,
    squad_archetypes: &[MercArchetype],
    now: DateTime<Utc>,
    rng: &mut impl Rng,
) -> EventTickResult {
    let progress = mission.progress(now);
    let mut result = EventTickResult {
        newly_pending: Vec::new(),
        auto_resolved: Vec::new(),
    };

    // Only process events on active missions.
    if !matches!(
        mission.status,
        MissionStatus::Active | MissionStatus::EventPending
    ) {
        return result;
    }

    // Walk pending_event_index forward: fire scheduled events whose trigger has passed.
    while mission.pending_event_index < mission.events.len() {
        let event = &mission.events[mission.pending_event_index];
        // Events are pre-scheduled — we derive trigger progress from scheduling position.
        // The trigger point is stored implicitly via the number of events and their order.
        // We computed trigger points in generate_mission_events(); here we map by index.
        let tier = LayerTier::from_layer(mission.layer);
        let triggers = event_trigger_points(mission.mission_type, tier);
        let trigger_progress = triggers
            .get(mission.pending_event_index)
            .copied()
            .unwrap_or(1.0);

        if event.resolved_choice.is_some() {
            // Already resolved — advance.
            mission.pending_event_index += 1;
            continue;
        }

        if progress < trigger_progress {
            // Not yet reached — stop checking.
            break;
        }

        // Event should fire now (or should have fired while offline).
        // If the event hasn't been given a fired_at time, set it.
        // fired_at is set at creation to Utc::now() in template_to_check_in_event,
        // but we need the actual trigger time. We set it here on first encounter.
        // Since fired_at is always set, check if it's the initial creation time
        // by seeing if resolved_choice is still None and status isn't EventPending.
        if mission.status != MissionStatus::EventPending {
            // First time we see this unresolved event at/past its trigger.
            // Compute the actual trigger time (not `now`) so offline/fast-forward
            // scenarios can immediately auto-resolve events that fired in the past.
            let total_duration_secs = (mission.ends_at - mission.started_at)
                .num_seconds()
                .max(1) as f64;
            let trigger_time = mission.started_at
                + Duration::seconds((trigger_progress * total_duration_secs) as i64);
            // Use whichever is earlier: computed trigger time or now (can't fire in the future)
            let actual_fired_at = trigger_time.min(now);
            mission.events[mission.pending_event_index].fired_at = actual_fired_at;
            mission.status = MissionStatus::EventPending;
            result.newly_pending.push(mission.pending_event_index);
            // Don't break — fall through to check auto-resolve immediately.
        }

        // Event is already pending (or just fired) — check for auto-resolve timeout.
        let pending_since = mission.events[mission.pending_event_index].fired_at;
        let elapsed = now - pending_since;
        if elapsed >= Duration::hours(AUTO_RESOLVE_TIMEOUT_HOURS) {
            // Auto-resolve.
            let event_mut = &mut mission.events[mission.pending_event_index];
            if let Some(resolution) = resolve_event(event_mut, None, squad_archetypes, rng) {
                result
                    .auto_resolved
                    .push((mission.pending_event_index, resolution));
                mission.pending_event_index += 1;
                // Resume mission — clear EventPending status and continue loop
                // to process any remaining events that may have also elapsed.
                mission.status = MissionStatus::Active;
                continue; // Don't break — keep processing events in the while loop.
            }
        }
        // Event needs player input (not yet auto-resolved) — stop here.
        break;
    }

    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn seeded_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    // ── event_trigger_points ─────────────────────────────────────────────────

    #[test]
    fn test_supply_run_has_no_events() {
        assert!(
            event_trigger_points(MissionType::SupplyRun, LayerTier::Shallows).is_empty()
        );
    }

    #[test]
    fn test_construction_has_no_events() {
        use super::super::types::Infrastructure;
        assert!(
            event_trigger_points(
                MissionType::Construction(Infrastructure::Outpost),
                LayerTier::Warrens
            )
            .is_empty()
        );
    }

    #[test]
    fn test_recon_has_one_event_at_50_percent() {
        let pts = event_trigger_points(MissionType::Recon, LayerTier::Shallows);
        assert_eq!(pts.len(), 1);
        assert!((pts[0] - 0.50).abs() < 1e-9);
    }

    #[test]
    fn test_expedition_has_two_events() {
        let pts = event_trigger_points(MissionType::Expedition, LayerTier::Hollows);
        assert_eq!(pts.len(), 2);
        assert!(pts[0] < pts[1]);
    }

    #[test]
    fn test_breakthrough_shallows_has_three_events() {
        let pts = event_trigger_points(MissionType::Breakthrough, LayerTier::Shallows);
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn test_breakthrough_abyss_has_five_events() {
        let pts = event_trigger_points(MissionType::Breakthrough, LayerTier::Abyss);
        assert_eq!(pts.len(), 5);
    }

    #[test]
    fn test_trigger_points_are_ascending() {
        for tier in [
            LayerTier::Shallows,
            LayerTier::Warrens,
            LayerTier::Hollows,
            LayerTier::SunkenReach,
            LayerTier::Abyss,
        ] {
            let pts = event_trigger_points(MissionType::Breakthrough, tier);
            for window in pts.windows(2) {
                assert!(
                    window[0] < window[1],
                    "Trigger points must be strictly ascending for {:?}",
                    tier
                );
            }
        }
    }

    // ── generate_mission_events ───────────────────────────────────────────────

    #[test]
    fn test_generate_events_supply_run_returns_empty() {
        let mut rng = seeded_rng();
        let events = generate_mission_events(MissionType::SupplyRun, 1, &[], &mut rng);
        assert!(events.is_empty());
    }

    #[test]
    fn test_generate_events_expedition_returns_two() {
        let mut rng = seeded_rng();
        let events = generate_mission_events(
            MissionType::Expedition,
            4,
            &[MercArchetype::Vanguard],
            &mut rng,
        );
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_generate_events_breakthrough_shallows_last_is_boss_approach() {
        let mut rng = seeded_rng();
        let events = generate_mission_events(MissionType::Breakthrough, 1, &[], &mut rng);
        assert_eq!(events.len(), 3);
        // Last event should have BossApproach title from shallows boss template.
        let last = &events[events.len() - 1];
        assert_eq!(last.title, "THE GUARDIAN'S CHAMBER");
    }

    #[test]
    fn test_generate_events_auto_resolve_choice_is_always_non_gated() {
        let mut rng = seeded_rng();
        for layer in [1u32, 4, 8, 13, 19] {
            for mission_type in [
                MissionType::Recon,
                MissionType::Expedition,
                MissionType::Breakthrough,
            ] {
                let events = generate_mission_events(mission_type, layer, &[], &mut rng);
                for event in &events {
                    let auto_choice = &event.choices[event.auto_resolve_choice];
                    assert!(
                        auto_choice.required_archetype.is_none(),
                        "Auto-resolve choice must not require an archetype (layer {}, {:?})",
                        layer,
                        mission_type,
                    );
                }
            }
        }
    }

    #[test]
    fn test_generate_events_all_choices_have_labels() {
        let mut rng = seeded_rng();
        let events = generate_mission_events(
            MissionType::Breakthrough,
            8,
            &[MercArchetype::Arcanist],
            &mut rng,
        );
        for event in &events {
            for choice in &event.choices {
                assert!(!choice.label.is_empty(), "Choice label must not be empty");
            }
        }
    }

    // ── resolve_event ─────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_auto_resolve_picks_safe_choice() {
        let mut rng = seeded_rng();
        let mut events = generate_mission_events(MissionType::Expedition, 1, &[], &mut rng);
        let event = &mut events[0];
        let original_auto = event.auto_resolve_choice;

        let resolution = resolve_event(event, None, &[], &mut rng);
        assert!(resolution.is_some());
        let res = resolution.unwrap();
        assert!(res.was_auto_resolved);
        assert!(!res.may_injure, "Auto-resolve must never cause injury");
        assert_eq!(event.resolved_choice, Some(original_auto));
    }

    #[test]
    fn test_resolve_player_choice_gated_without_archetype_returns_none() {
        let mut rng = seeded_rng();
        // Generate Flooded Passage which has a Scout-gated option at index 2.
        // We need an event where index 2 is gated.
        let mut event = template_to_check_in_event(&FLOODED_PASSAGE, 1, Utc::now());
        let gated_index = event
            .choices
            .iter()
            .position(|c| c.required_archetype.is_some())
            .unwrap();

        // No Scout in squad — should fail.
        let res = resolve_event(&mut event, Some(gated_index), &[], &mut rng);
        assert!(
            res.is_none(),
            "Should reject gated choice when archetype not in squad"
        );
    }

    #[test]
    fn test_resolve_player_choice_gated_with_archetype_succeeds() {
        let mut rng = seeded_rng();
        let mut event = template_to_check_in_event(&FLOODED_PASSAGE, 1, Utc::now());
        let gated_index = event
            .choices
            .iter()
            .position(|c| c.required_archetype == Some(MercArchetype::Scout))
            .unwrap();

        let res = resolve_event(
            &mut event,
            Some(gated_index),
            &[MercArchetype::Scout],
            &mut rng,
        );
        assert!(res.is_some(), "Gated choice should succeed with matching archetype");
        assert_eq!(event.resolved_choice, Some(gated_index));
    }

    #[test]
    fn test_resolve_out_of_bounds_returns_none() {
        let mut rng = seeded_rng();
        let mut event = template_to_check_in_event(&FLOODED_PASSAGE, 1, Utc::now());
        let res = resolve_event(&mut event, Some(999), &[], &mut rng);
        assert!(res.is_none());
    }

    #[test]
    fn test_auto_resolve_never_picks_risky_choice() {
        let mut rng = seeded_rng();
        // Run auto-resolve many times — should never produce may_injure = true.
        for seed in 0u64..100 {
            let mut rng2 = ChaCha8Rng::seed_from_u64(seed);
            let mut events =
                generate_mission_events(MissionType::Expedition, 5, &[], &mut rng2);
            for event in &mut events {
                let res = resolve_event(event, None, &[], &mut rng);
                if let Some(r) = res {
                    assert!(
                        !r.may_injure,
                        "Auto-resolve must never injure mercs (seed {})",
                        seed
                    );
                }
            }
        }
    }

    // ── void_scale ────────────────────────────────────────────────────────────

    #[test]
    fn test_void_scale_no_scaling_at_layer_25() {
        let (ts, ms, is) = void_scale(25);
        assert!((ts - 1.0).abs() < 1e-9);
        assert!((ms - 1.0).abs() < 1e-9);
        assert!((is - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_void_scale_increases_past_layer_25() {
        let (ts26, ms26, _) = void_scale(26);
        let (ts30, ms30, _) = void_scale(30);
        assert!(ts26 > 1.0);
        assert!(ts30 > ts26);
        assert!(ms30 > ms26);
    }

    // ── template completeness ─────────────────────────────────────────────────

    #[test]
    fn test_all_templates_have_non_gated_safe_fallback() {
        for template in all_templates() {
            let non_gated_safe = template
                .choices
                .iter()
                .any(|c| c.required_archetype.is_none() && !c.is_risky);
            assert!(
                non_gated_safe,
                "Template '{}' must have at least one non-gated, non-risky choice",
                template.title
            );
        }
    }

    #[test]
    fn test_all_templates_auto_resolve_index_in_bounds() {
        for template in all_templates() {
            assert!(
                template.auto_resolve_index < template.choices.len(),
                "auto_resolve_index {} out of bounds for '{}' ({} choices)",
                template.auto_resolve_index,
                template.title,
                template.choices.len()
            );
        }
    }

    #[test]
    fn test_all_templates_auto_resolve_choice_is_not_gated() {
        for template in all_templates() {
            let choice = &template.choices[template.auto_resolve_index];
            assert!(
                choice.required_archetype.is_none(),
                "auto_resolve_choice for '{}' must not require an archetype",
                template.title
            );
        }
    }

    #[test]
    fn test_all_templates_auto_resolve_choice_is_not_risky() {
        for template in all_templates() {
            let choice = &template.choices[template.auto_resolve_index];
            assert!(
                !choice.is_risky,
                "auto_resolve_choice for '{}' must not be risky",
                template.title
            );
        }
    }

    #[test]
    fn test_boss_templates_exist_for_all_tiers() {
        for tier in [
            LayerTier::Shallows,
            LayerTier::Warrens,
            LayerTier::Hollows,
            LayerTier::SunkenReach,
            LayerTier::Abyss,
            LayerTier::Void,
        ] {
            let boss = tier_boss_template(tier);
            assert_eq!(boss.category, EventCategory::BossApproach);
            assert!(!boss.title.is_empty());
        }
    }
}
