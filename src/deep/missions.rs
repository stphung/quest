#![allow(dead_code)] // Functions wired into the game loop incrementally
//! Mission lifecycle management for The Deep.
//!
//! Covers:
//! - Mission pool generation (3–5 available missions based on guild rank)
//! - Squad assignment validation (power threshold, capacity)
//! - Mission start: sets wall-clock timing, deducts costs, generates events
//! - Mission ticking: fires events at progress milestones, auto-resolves expired events
//! - Mission resolution: computes outcome and rewards, applies injuries/losses
//! - Offline resolution: fast-forwards on game load
//!
//! All functions follow the Haven/Soulforge pattern: no global state, explicit
//! params, `rng: &mut impl Rng` passed through from the caller.

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, RngExt};

use super::economy::{
    compute_mark_reward, merc_xp_per_mission, merc_xp_to_next_level, mission_launch_cost,
    outcome_mark_multiplier, stormglass_reward, xp_reward, MarkRewardParams,
};
use super::events::{generate_mission_events_with_names, tick_mission_events, EventTickResult};
use super::layers::{
    apply_duration_modifiers, apply_familiarity_gain, base_mission_duration_secs,
    mark_layer_cleared, mission_power_threshold, watchtower_auto_resolve_bonus, DurationModifiers,
};
use super::mercenaries::{injure_merc, mark_merc_lost};
use super::types::{
    effective_concurrent_missions, AvailableMission, DeepPersistent, DeepPrestige, GuildRank,
    Infrastructure, LayerTier, MercArchetype, MercStatus, Mission, MissionOutcome, MissionResult,
    MissionStatus, MissionType, WarbandLogEntry,
};

// ── Mission Pool Generation ────────────────────────────────────────────────────

/// Number of available missions shown at each guild rank.
///
/// More missions become available at higher ranks.
pub fn available_mission_count(guild_rank: GuildRank) -> usize {
    match guild_rank.0 {
        1 | 2 => 3,
        3 | 4 => 4,
        _ => 5,
    }
}

/// Generate the set of available missions shown in the mission pool.
///
/// Produces a mix of mission types based on what's currently accessible:
/// - SupplyRun/Construction only on cleared layers
/// - Recon/Expedition/Breakthrough only on the frontier or cleared layers
///
/// The pool always includes at least one SupplyRun (safe option) when cleared
/// layers exist.
pub fn generate_mission_pool(
    persistent: &DeepPersistent,
    rng: &mut impl Rng,
) -> Vec<AvailableMission> {
    let guild_rank = persistent.guild_rank;
    let count = available_mission_count(guild_rank);
    let frontier = persistent.frontier_layer();
    let has_cleared_layers = persistent.layers.iter().any(|l| l.cleared);

    let mut pool = Vec::with_capacity(count);

    // Always offer a Supply Run on the frontier or the most recently cleared layer,
    // or layer 1 if nothing is cleared yet.
    let supply_run_layer = if has_cleared_layers {
        persistent
            .layers
            .iter()
            .filter(|l| l.cleared)
            .map(|l| l.index)
            .max()
            .unwrap_or(1)
    } else {
        1
    };
    pool.push(generate_available_mission(
        MissionType::SupplyRun,
        supply_run_layer,
        persistent,
        rng,
    ));

    // Add Recon on the frontier if we have room.
    if pool.len() < count {
        pool.push(generate_available_mission(
            MissionType::Recon,
            frontier,
            persistent,
            rng,
        ));
    }

    // Add an Expedition on the frontier.
    if pool.len() < count {
        pool.push(generate_available_mission(
            MissionType::Expedition,
            frontier,
            persistent,
            rng,
        ));
    }

    // Add a Breakthrough on the frontier (once per layer; check not already cleared).
    let frontier_cleared = persistent
        .layer_record(frontier)
        .map(|r| r.cleared)
        .unwrap_or(false);

    if pool.len() < count && !frontier_cleared {
        pool.push(generate_available_mission(
            MissionType::Breakthrough,
            frontier,
            persistent,
            rng,
        ));
    }

    // Fill remaining slots with Construction missions on cleared layers,
    // or extra Supply Runs on deeper cleared layers if no infra slots remain.
    if pool.len() < count && has_cleared_layers {
        // Find a cleared layer that still has buildable infrastructure.
        let construction_target = persistent
            .layers
            .iter()
            .filter(|l| l.cleared && l.infrastructure.len() < Infrastructure::ALL.len())
            .map(|l| l.index)
            .next();

        if let Some(target_layer) = construction_target {
            // Pick a random infrastructure type not yet built.
            let built = &persistent
                .layers
                .iter()
                .find(|l| l.index == target_layer)
                .map(|l| l.infrastructure.clone())
                .unwrap_or_default();

            let available_infra: Vec<Infrastructure> = Infrastructure::ALL
                .iter()
                .filter(|&&i| !built.contains(&i))
                .copied()
                .collect();

            if !available_infra.is_empty() {
                let infra_index = rng.random_range(0..available_infra.len());
                pool.push(generate_available_mission(
                    MissionType::Construction(available_infra[infra_index]),
                    target_layer,
                    persistent,
                    rng,
                ));
            }
        }
    }

    pool.truncate(count);
    pool
}

/// Refresh interval for the mission pool in seconds (6 hours).
///
/// The pool is regenerated when it is empty OR when this many seconds have
/// elapsed since the last refresh, whichever comes first.
pub const POOL_REFRESH_INTERVAL_SECS: i64 = 6 * 3600;

/// Check whether the mission pool needs refreshing and regenerate it if so.
///
/// The pool refreshes when any of these conditions are true:
/// 1. `available_missions` is empty (player has accepted all missions), OR
/// 2. `pool_refreshed_at` is `None` (never been explicitly set), OR
/// 3. At least `POOL_REFRESH_INTERVAL_SECS` (6h) have elapsed since the last refresh.
///
/// Returns `true` if the pool was refreshed (signals the caller to set
/// `deep_changed` and persist state to disk).
///
/// # Design notes
/// - `now` is passed explicitly to allow deterministic testing without wall-clock dependency.
/// - Called once per tick from stage 11c in `core/tick.rs` (after mission ticking).
/// - Also called from `main_helpers/offline.rs` on game load for immediate catch-up.
/// - `pool_refreshed_at` defaults to `None` for old saves, triggering an immediate refresh.
pub fn maybe_refresh_mission_pool(
    prestige: &mut DeepPrestige,
    persistent: &DeepPersistent,
    now: DateTime<Utc>,
    rng: &mut impl Rng,
) -> bool {
    let pool_empty = prestige.available_missions.is_empty();
    let pool_stale = match prestige.pool_refreshed_at {
        None => true,
        Some(refreshed_at) => (now - refreshed_at).num_seconds() >= POOL_REFRESH_INTERVAL_SECS,
    };

    if pool_empty || pool_stale {
        prestige.available_missions = generate_mission_pool(persistent, rng);
        prestige.pool_refreshed_at = Some(now);
        true
    } else {
        false
    }
}

/// Build an `AvailableMission` for a given type and layer.
fn generate_available_mission(
    mission_type: MissionType,
    layer: u32,
    persistent: &DeepPersistent,
    rng: &mut impl Rng,
) -> AvailableMission {
    let tier = LayerTier::from_layer(layer);
    let layer_record = persistent.layer_record(layer);
    let familiarity = layer_record.map(|r| r.familiarity).unwrap_or(0);
    let has_outpost = layer_record
        .map(|r| r.has_infrastructure(Infrastructure::Outpost))
        .unwrap_or(false);

    let saboteur_present = false; // Pool generation doesn't know squad yet.
    let bridge_layers = (1..layer)
        .filter(|l| {
            persistent
                .layer_record(*l)
                .map(|r| r.has_infrastructure(Infrastructure::Bridge))
                .unwrap_or(false)
        })
        .count() as u32;
    let mods = DurationModifiers {
        has_outpost,
        familiarity,
        has_saboteur: saboteur_present,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers,
    };
    let base = base_mission_duration_secs(tier, mission_type);
    let duration_secs = apply_duration_modifiers(base, &mods);

    let min_power = mission_power_threshold(layer, mission_type);
    let marks_cost = mission_launch_cost(mission_type, layer);

    // Randomly pick a recommended/required archetype based on layer tier.
    let required_archetype = pick_required_archetype(mission_type, tier, rng);
    let recommended_archetype = pick_recommended_archetype(tier, rng);

    let description = mission_description(mission_type, layer);

    AvailableMission {
        mission_type,
        layer,
        duration_secs,
        min_squad_power: min_power,
        required_archetype,
        recommended_archetype,
        marks_cost,
        description: description.to_string(),
    }
}

fn pick_required_archetype(
    mission_type: MissionType,
    tier: LayerTier,
    _rng: &mut impl Rng,
) -> Option<MercArchetype> {
    // Breakthrough missions require higher tiers to have specific archetypes.
    match (mission_type, tier) {
        (MissionType::Breakthrough, LayerTier::Hollows)
        | (MissionType::Breakthrough, LayerTier::SunkenReach)
        | (MissionType::Breakthrough, LayerTier::Abyss)
        | (MissionType::Breakthrough, LayerTier::Void) => Some(MercArchetype::Medic),
        _ => None,
    }
}

fn pick_recommended_archetype(tier: LayerTier, rng: &mut impl Rng) -> Option<MercArchetype> {
    // Recommend archetypes relevant to common hazards for this tier.
    let candidates: &[MercArchetype] = match tier {
        LayerTier::Shallows => &[MercArchetype::Scout, MercArchetype::Vanguard],
        LayerTier::Warrens => &[MercArchetype::Saboteur, MercArchetype::Vanguard],
        LayerTier::Hollows => &[MercArchetype::Arcanist, MercArchetype::Scout],
        LayerTier::SunkenReach => &[
            MercArchetype::Arcanist,
            MercArchetype::Saboteur,
            MercArchetype::Medic,
        ],
        LayerTier::Abyss | LayerTier::Void => &[MercArchetype::Arcanist, MercArchetype::Medic],
    };
    let idx = rng.random_range(0..candidates.len());
    Some(candidates[idx])
}

fn mission_description(mission_type: MissionType, layer: u32) -> &'static str {
    let tier = LayerTier::from_layer(layer);
    match (mission_type, tier) {
        (MissionType::GatewayExpedition, _) => "Breach the sealed gateway at the root of The Deep.",
        (MissionType::SupplyRun, _) => "Recover resources from previously cleared sections.",
        (MissionType::Recon, LayerTier::Shallows) => {
            "Survey the Shallows for intel and entry points."
        }
        (MissionType::Recon, LayerTier::Warrens) => "Map the warren tunnels and catalogue threats.",
        (MissionType::Recon, LayerTier::Hollows) => {
            "Probe the Hollow's crystal formations and stalker nests."
        }
        (MissionType::Recon, LayerTier::SunkenReach) => {
            "Survey flooded vaults and note guardian positions."
        }
        (MissionType::Recon, _) => "Gather intelligence on the deep structure ahead.",
        (MissionType::Expedition, LayerTier::Shallows) => {
            "Push through the Shallows, securing resources."
        }
        (MissionType::Expedition, LayerTier::Warrens) => {
            "Penetrate the Warren tunnels and neutralise threats."
        }
        (MissionType::Expedition, LayerTier::Hollows) => {
            "Advance through the Hollows under heavy hazard."
        }
        (MissionType::Expedition, LayerTier::SunkenReach) => {
            "Navigate the submerged Sunken Reach corridors."
        }
        (MissionType::Expedition, _) => "Mount a full expedition deeper into the structure.",
        (MissionType::Breakthrough, _) => {
            "Defeat the guardian and break through to the next layer."
        }
        (MissionType::Construction(Infrastructure::Outpost), _) => {
            "Establish a forward outpost to reduce future mission times."
        }
        (MissionType::Construction(Infrastructure::SupplyCache), _) => {
            "Cache supplies to improve resource yields on this layer."
        }
        (MissionType::Construction(Infrastructure::Watchtower), _) => {
            "Build a watchtower to improve intel and auto-resolve quality."
        }
        (MissionType::Construction(Infrastructure::Bridge), _) => {
            "Construct a shortcut bridge to bypass this layer."
        }
    }
}

// ── Squad Validation ───────────────────────────────────────────────────────────

/// Errors returned when squad assignment fails validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SquadAssignmentError {
    /// No mercs selected.
    EmptySquad,
    /// One or more selected mercs are not available.
    MercNotAvailable(u64),
    /// Combined squad power is below the mission threshold.
    InsufficientPower { required: u32, actual: u32 },
    /// A required archetype is missing from the squad.
    MissingRequiredArchetype(MercArchetype),
    /// Too many concurrent missions already running.
    ConcurrentMissionLimit,
    /// Insufficient Warband Marks to cover the launch cost.
    InsufficientMarks { required: u32, available: u32 },
}

/// Validate that the given squad can be assigned to the available mission.
///
/// Does NOT mutate state — purely checks preconditions.
pub fn validate_squad_assignment(
    available: &AvailableMission,
    merc_ids: &[u64],
    prestige: &DeepPrestige,
    persistent: &DeepPersistent,
    is_free_daily_supply_run: bool,
) -> Result<(), SquadAssignmentError> {
    if merc_ids.is_empty() {
        return Err(SquadAssignmentError::EmptySquad);
    }

    // Check concurrent mission limit.
    let active_count = prestige.active_mission_count() as u32;
    if active_count
        >= effective_concurrent_missions(persistent.guild_rank, persistent.deepest_layer_reached)
    {
        return Err(SquadAssignmentError::ConcurrentMissionLimit);
    }

    // Collect squad archetypes and validate availability.
    let mut total_power = 0u32;
    let mut archetypes: Vec<MercArchetype> = Vec::new();

    for &id in merc_ids {
        let merc = prestige
            .find_merc(id)
            .ok_or(SquadAssignmentError::MercNotAvailable(id))?;
        if !merc.is_available() {
            return Err(SquadAssignmentError::MercNotAvailable(id));
        }
        total_power += merc.effective_power();
        archetypes.push(merc.archetype);
    }

    // Power check.
    if total_power < available.min_squad_power {
        return Err(SquadAssignmentError::InsufficientPower {
            required: available.min_squad_power,
            actual: total_power,
        });
    }

    // Required archetype check.
    if let Some(required) = available.required_archetype {
        if !archetypes.contains(&required) {
            return Err(SquadAssignmentError::MissingRequiredArchetype(required));
        }
    }

    // Marks cost check (skip if this is the free daily supply run).
    if !is_free_daily_supply_run
        && available.marks_cost > 0
        && prestige.warband_marks < available.marks_cost
    {
        return Err(SquadAssignmentError::InsufficientMarks {
            required: available.marks_cost,
            available: prestige.warband_marks,
        });
    }

    Ok(())
}

// ── Mission Start ─────────────────────────────────────────────────────────────

/// Start a mission: deduct costs, assign squad, and generate scheduled events.
///
/// Returns the new `Mission` — the caller is responsible for pushing it into
/// `prestige.active_missions` and updating merc statuses.
///
/// # Panics
///
/// Panics only if `merc_ids` references a merc not in `prestige.roster` —
/// callers should run `validate_squad_assignment` first.
pub fn start_mission(
    available: &AvailableMission,
    merc_ids: &[u64],
    prestige: &mut DeepPrestige,
    persistent: &mut DeepPersistent,
    is_free_daily_supply_run: bool,
    now: DateTime<Utc>,
    rng: &mut impl Rng,
) -> Mission {
    // Deduct marks cost.
    if !is_free_daily_supply_run && available.marks_cost > 0 {
        prestige.warband_marks = prestige.warband_marks.saturating_sub(available.marks_cost);
    }

    // Compute actual duration (with infrastructure modifiers).
    let layer_record = persistent.layer_record(available.layer);
    let familiarity = layer_record.map(|r| r.familiarity).unwrap_or(0);
    let has_outpost = layer_record
        .map(|r| r.has_infrastructure(Infrastructure::Outpost))
        .unwrap_or(false);

    let squad_archetypes: Vec<MercArchetype> = merc_ids
        .iter()
        .filter_map(|&id| prestige.find_merc(id).map(|m| m.archetype))
        .collect();

    let has_saboteur = squad_archetypes.contains(&MercArchetype::Saboteur);
    let saboteur_is_veteran = merc_ids
        .iter()
        .filter_map(|&id| prestige.find_merc(id))
        .any(|m| m.archetype == MercArchetype::Saboteur && m.level >= 10);

    let total_power: u32 = merc_ids
        .iter()
        .filter_map(|&id| prestige.find_merc(id))
        .map(|m| m.effective_power())
        .sum();
    let threshold = mission_power_threshold(available.layer, available.mission_type);
    let is_overpowered = total_power >= (threshold * 3 / 2); // ≥150% of threshold

    let bridge_layers = (1..available.layer)
        .filter(|l| {
            persistent
                .layer_record(*l)
                .map(|r| r.has_infrastructure(Infrastructure::Bridge))
                .unwrap_or(false)
        })
        .count() as u32;

    let tier = LayerTier::from_layer(available.layer);
    let base_duration = base_mission_duration_secs(tier, available.mission_type);
    let mods = DurationModifiers {
        has_outpost,
        familiarity,
        has_saboteur,
        saboteur_is_veteran,
        is_overpowered,
        bridge_layers,
    };
    let duration_secs = apply_duration_modifiers(base_duration, &mods);

    let ends_at = now + Duration::seconds(duration_secs as i64);

    // Gather squad merc names for personalised event descriptions.
    let squad_names: Vec<String> = merc_ids
        .iter()
        .filter_map(|&id| prestige.find_merc(id).map(|m| m.name.clone()))
        .collect();

    // Generate check-in events (pre-scheduled at fraction points).
    let events = generate_mission_events_with_names(
        available.mission_type,
        available.layer,
        &squad_archetypes,
        &squad_names,
        rng,
    );

    // Assign mission id.
    let mission_id = persistent.next_mission_id();

    // Mark mercs as on-mission.
    for &id in merc_ids {
        if let Some(merc) = prestige.find_merc_mut(id) {
            merc.status = MercStatus::OnMission(mission_id);
        }
    }

    Mission {
        id: mission_id,
        mission_type: available.mission_type,
        layer: available.layer,
        squad: merc_ids.to_vec(),
        started_at: now,
        ends_at,
        events,
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    }
}

// ── Mission Ticking ────────────────────────────────────────────────────────────

/// Tick a single active mission: advance events, auto-resolve expired events.
///
/// Call every game tick (100ms) while the game is running.
///
/// Returns the `EventTickResult` so the caller can update mission status,
/// apply time deltas from event resolutions, and notify the player.
pub fn tick_mission(
    mission: &mut Mission,
    prestige: &DeepPrestige,
    now: DateTime<Utc>,
    rng: &mut impl Rng,
) -> EventTickResult {
    let squad_archetypes: Vec<MercArchetype> = mission
        .squad
        .iter()
        .filter_map(|&id| prestige.find_merc(id).map(|m| m.archetype))
        .collect();

    tick_mission_events(mission, &squad_archetypes, now, rng)
}

/// Tick all active missions in `prestige`.
///
/// Resolves events and, for missions that have elapsed, moves them to
/// `prestige.pending_results` (completed missions awaiting player acknowledgement).
///
/// Returns a summary of what changed.
pub fn tick_all_missions(
    prestige: &mut DeepPrestige,
    persistent: &mut DeepPersistent,
    now: DateTime<Utc>,
    rng: &mut impl Rng,
) -> MissionTickSummary {
    let mut summary = MissionTickSummary::default();
    let mut completed_ids: Vec<u64> = Vec::new();

    for mission in &mut prestige.active_missions {
        if !matches!(
            mission.status,
            MissionStatus::Active | MissionStatus::EventPending
        ) {
            continue;
        }

        // Tick events.
        let squad_archetypes: Vec<MercArchetype> = mission
            .squad
            .iter()
            .filter_map(|&id| {
                prestige
                    .roster
                    .iter()
                    .find(|m| m.id == id)
                    .map(|m| m.archetype)
            })
            .collect();

        let event_result = tick_mission_events(mission, &squad_archetypes, now, rng);
        if !event_result.newly_pending.is_empty() {
            summary.events_fired += event_result.newly_pending.len();
        }
        if !event_result.auto_resolved.is_empty() {
            summary.events_auto_resolved += event_result.auto_resolved.len();
            // Apply time deltas from auto-resolved events.
            for (_, resolution) in &event_result.auto_resolved {
                if resolution.time_delta_secs != 0 {
                    let delta = Duration::seconds(resolution.time_delta_secs);
                    mission.ends_at += delta;
                }
            }
        }

        // If the mission timer has elapsed but there are still pending events
        // (e.g. events that fire at progress milestones which were reached at the
        // same tick as completion), force auto-resolve all remaining unresolved events
        // without applying time deltas — the mission has already ended.
        if mission.is_time_elapsed(now) {
            while mission.pending_event_index < mission.events.len() {
                let event = &mission.events[mission.pending_event_index];
                if event.resolved_choice.is_some() {
                    mission.pending_event_index += 1;
                    continue;
                }
                // Auto-resolve this event. Do NOT apply time deltas because the
                // mission timer has already elapsed.
                let event_mut = &mut mission.events[mission.pending_event_index];
                if super::events::resolve_event(event_mut, None, &squad_archetypes, rng).is_some() {
                    summary.events_auto_resolved += 1;
                }
                mission.pending_event_index += 1;
            }
            // Clear EventPending status so the completion check passes.
            if matches!(mission.status, MissionStatus::EventPending) {
                mission.status = MissionStatus::Active;
            }
        }

        // Check if the mission timer has elapsed (and no pending events).
        if mission.is_time_elapsed(now) && !matches!(mission.status, MissionStatus::EventPending) {
            completed_ids.push(mission.id);
        }
    }

    // Resolve completed missions.
    for id in completed_ids {
        if let Some(idx) = prestige.active_missions.iter().position(|m| m.id == id) {
            let mut mission = prestige.active_missions.remove(idx);
            resolve_mission(&mut mission, prestige, persistent, rng);
            summary.missions_completed += 1;

            // Populate achievement signals from the resolved mission.
            if let Some(ref result) = mission.result {
                summary.mercs_lost += result.lost_mercs.len();
                if matches!(mission.mission_type, MissionType::Breakthrough)
                    && matches!(result.outcome, MissionOutcome::Success)
                {
                    summary.breakthroughs.push(mission.layer);
                }
                if matches!(mission.mission_type, MissionType::GatewayExpedition)
                    && matches!(result.outcome, MissionOutcome::Success)
                {
                    summary.gateway_opened = true;
                }
            }

            prestige.pending_results.push(mission);
        }
    }

    summary
}

/// Summary of what happened during `tick_all_missions`.
#[derive(Debug, Default, Clone)]
pub struct MissionTickSummary {
    pub missions_completed: usize,
    pub events_fired: usize,
    pub events_auto_resolved: usize,
    /// Layer numbers cleared by successful Breakthrough missions.
    pub breakthroughs: Vec<u32>,
    /// Number of mercenaries permanently lost across all resolved missions.
    pub mercs_lost: usize,
    /// Whether a GatewayExpedition completed successfully this tick.
    pub gateway_opened: bool,
}

// ── Mission Resolution ─────────────────────────────────────────────────────────

/// Compute the mission outcome based on squad power vs. layer difficulty,
/// event choices, and random variance.
///
/// Outcome categories:
/// - Success: squad power ≥ threshold and no significant failures
/// - PartialSuccess: squad power near threshold or some risky choices failed
/// - Failure: squad power well below threshold or multiple critical failures
fn compute_outcome(
    mission: &Mission,
    prestige: &DeepPrestige,
    persistent: &DeepPersistent,
    rng: &mut impl Rng,
) -> MissionOutcome {
    let threshold = mission_power_threshold(mission.layer, mission.mission_type);

    let total_power: u32 = mission
        .squad
        .iter()
        .filter_map(|&id| prestige.find_merc(id))
        .map(|m| m.effective_power())
        .sum();

    // Power ratio: how much of the threshold the squad meets.
    let power_ratio = if threshold == 0 {
        2.0f64 // safe missions always succeed
    } else {
        total_power as f64 / threshold as f64
    };

    // Collect event power modifiers from resolved events.
    let mut combined_power_modifier = 1.0f64;
    for event in &mission.events {
        // We use the effective_choice index to look up the power modifier via the
        // static template. For now, we apply a simple heuristic based on event data.
        if event.is_resolved() {
            let choice_idx = event.effective_choice();
            if let Some(choice) = event.choices.get(choice_idx) {
                // Risky choices that weren't auto-resolved may have already been
                // tracked; non-risky safe fallbacks are neutral (modifier 1.0).
                // We don't have the power_modifier on EventChoice, but the key
                // insight is: if the squad succeeded at risky options, we reward;
                // if they auto-resolved everything, they stay at baseline.
                let _ = choice; // power_modifier is on the static template, not CheckInEvent
            }
        }
    }

    // Count unresolved (auto-resolved) events as slightly negative.
    let auto_resolved_count = mission
        .events
        .iter()
        .filter(|e| e.resolved_choice == Some(e.auto_resolve_choice))
        .count();
    let total_events = mission.events.len();

    // Apply a small penalty per auto-resolved event on Breakthrough.
    // Watchtower on the mission's layer reduces this penalty.
    if matches!(mission.mission_type, MissionType::Breakthrough) && total_events > 0 {
        let has_watchtower = persistent
            .layer_record(mission.layer)
            .map(|r| r.has_infrastructure(Infrastructure::Watchtower))
            .unwrap_or(false);
        let watchtower_bonus = watchtower_auto_resolve_bonus(has_watchtower);
        let auto_fraction = auto_resolved_count as f64 / total_events as f64;
        let penalty_rate = (0.10 - watchtower_bonus).max(0.0);
        combined_power_modifier *= 1.0 - auto_fraction * penalty_rate;
    }

    let effective_ratio = power_ratio * combined_power_modifier;

    // Roll for outcome based on power ratio.
    let roll: f64 = rng.random();

    // Safe missions always succeed.
    if matches!(
        mission.mission_type,
        MissionType::SupplyRun | MissionType::Construction(_)
    ) {
        return MissionOutcome::Success;
    }

    if effective_ratio >= 1.5 {
        // Overpowered: nearly always succeed.
        if roll < 0.95 {
            MissionOutcome::Success
        } else {
            MissionOutcome::PartialSuccess
        }
    } else if effective_ratio >= 1.0 {
        // At or above threshold: usually succeed.
        let success_chance = 0.60 + effective_ratio * 0.25;
        let success_chance = success_chance.clamp(0.60, 0.90);
        if roll < success_chance {
            MissionOutcome::Success
        } else {
            MissionOutcome::PartialSuccess
        }
    } else if effective_ratio >= 0.75 {
        // Below threshold: mostly partial.
        if roll < 0.30 {
            MissionOutcome::Success
        } else if roll < 0.80 {
            MissionOutcome::PartialSuccess
        } else {
            MissionOutcome::Failure
        }
    } else {
        // Well below threshold: mostly failure.
        if roll < 0.50 {
            MissionOutcome::PartialSuccess
        } else {
            MissionOutcome::Failure
        }
    }
}

/// Resolve a completed mission: compute outcome, calculate rewards, apply
/// injuries, and update merc statuses.
///
/// Mutates `mission.result` and updates mercs in `prestige`.
/// Also applies layer state changes (familiarity, clearing) to `persistent`.
pub fn resolve_mission(
    mission: &mut Mission,
    prestige: &mut DeepPrestige,
    persistent: &mut DeepPersistent,
    rng: &mut impl Rng,
) {
    // Special handling for First Orders starter mission
    if mission.is_first_orders {
        resolve_first_orders(mission, prestige, persistent);
        return;
    }

    let outcome = compute_outcome(mission, prestige, persistent, rng);

    // Build reward parameters.
    let layer_record = persistent.layer_record(mission.layer);
    let familiarity = layer_record.map(|r| r.familiarity).unwrap_or(0);
    let has_supply_cache = layer_record
        .map(|r| r.has_infrastructure(Infrastructure::SupplyCache))
        .unwrap_or(false);

    let rng_variance: f64 = rng.random();
    let marks_earned = compute_mark_reward(&MarkRewardParams {
        mission_type: mission.mission_type,
        layer: mission.layer,
        familiarity,
        has_supply_cache,
        outcome: outcome.clone(),
        rng_variance,
    });

    let xp = xp_reward(mission.mission_type, mission.layer, outcome.clone());
    let stormglass = stormglass_reward(mission.mission_type, mission.layer);
    let stormglass_scaled = match outcome {
        MissionOutcome::Success => stormglass,
        MissionOutcome::PartialSuccess => stormglass / 2,
        MissionOutcome::Failure => 0,
    };

    // Determine injuries and losses based on outcome.
    let (injured_mercs, lost_mercs) =
        apply_mission_casualties(mission, prestige, persistent, &outcome, rng);

    // Compute power ratio for danger bonus check.
    let threshold = mission_power_threshold(mission.layer, mission.mission_type);
    let total_power: u32 = mission
        .squad
        .iter()
        .filter_map(|&id| prestige.find_merc(id))
        .map(|m| m.effective_power())
        .sum();
    let power_ratio = if threshold == 0 {
        2.0f64
    } else {
        total_power as f64 / threshold as f64
    };
    let danger_bonus = power_ratio < 1.0;

    // Apply merc XP and level-ups (with danger bonus if underpowered).
    let merc_level_ups = apply_squad_xp(mission, prestige, &outcome, danger_bonus);

    // Update layer familiarity.
    if let Some(record) = persistent
        .layers
        .iter_mut()
        .find(|r| r.index == mission.layer)
    {
        apply_familiarity_gain(record, mission.mission_type);
    } else {
        // Layer not yet in records — create and gain familiarity.
        let record = persistent.layer_record_mut(mission.layer);
        apply_familiarity_gain(record, mission.mission_type);
    }

    // Mark layer cleared on Breakthrough success.
    if matches!(mission.mission_type, MissionType::Breakthrough)
        && matches!(outcome, MissionOutcome::Success)
    {
        mark_layer_cleared(persistent, mission.layer);
    }

    // Gateway Expedition success opens the sealed gateway.
    if matches!(mission.mission_type, MissionType::GatewayExpedition)
        && matches!(outcome, MissionOutcome::Success)
    {
        persistent.gateway_opened = true;
    }

    // Award marks to prestige balance.
    prestige.warband_marks = prestige.warband_marks.saturating_add(marks_earned);

    // Track generation-level stats
    prestige.total_marks_earned += marks_earned;
    prestige.total_missions_completed += 1;

    // Build the result.
    mission.result = Some(MissionResult {
        outcome,
        marks_earned,
        xp_earned: xp,
        stormglass_earned: stormglass_scaled,
        item_ilvl: item_ilvl_for_mission(mission),
        injured_mercs,
        lost_mercs,
        merc_level_ups,
        danger_bonus_xp: danger_bonus,
    });

    // Append to the warband log (keep last 10 entries).
    let log_outcome = mission.result.as_ref().unwrap().outcome.clone();
    prestige.warband_log.push(WarbandLogEntry {
        mission_name: mission.mission_type.display_name().to_string(),
        layer: mission.layer,
        outcome: log_outcome,
        marks_earned,
        timestamp: Utc::now(),
    });
    const MAX_WARBAND_LOG: usize = 10;
    if prestige.warband_log.len() > MAX_WARBAND_LOG {
        let excess = prestige.warband_log.len() - MAX_WARBAND_LOG;
        prestige.warband_log.drain(..excess);
    }

    mission.status = MissionStatus::Completed;
}

/// Resolve the "First Orders" starter mission with guaranteed success.
///
/// Awards +30 familiarity on Layer 1 and 15 Warband Marks.
fn resolve_first_orders(
    mission: &mut Mission,
    prestige: &mut DeepPrestige,
    persistent: &mut DeepPersistent,
) {
    // Award +30 familiarity on Layer 1
    let record = persistent.layer_record_mut(1);
    record.familiarity = record.familiarity.saturating_add(30).min(100);

    let marks_earned = 15u32;
    prestige.warband_marks += marks_earned;

    mission.result = Some(MissionResult {
        outcome: MissionOutcome::Success,
        marks_earned,
        xp_earned: 0,
        stormglass_earned: 0,
        item_ilvl: None,
        injured_mercs: vec![],
        lost_mercs: vec![],
        merc_level_ups: vec![],
        danger_bonus_xp: false,
    });
    mission.status = MissionStatus::Completed;

    // Free squad
    for &merc_id in &mission.squad {
        if let Some(merc) = prestige.find_merc_mut(merc_id) {
            merc.status = MercStatus::Available;
        }
    }

    // Add to warband log
    prestige.warband_log.push(WarbandLogEntry {
        mission_name: "First Orders".to_string(),
        layer: 1,
        outcome: MissionOutcome::Success,
        marks_earned,
        timestamp: Utc::now(),
    });
}

/// Determine the item ilvl for a completed mission (if it can drop an item).
///
/// Only Expeditions and Breakthroughs can produce items.
/// ilvl = layer_index * 10 (mirrors the zone ilvl formula).
fn item_ilvl_for_mission(mission: &Mission) -> Option<u32> {
    match mission.mission_type {
        MissionType::Expedition | MissionType::Breakthrough | MissionType::GatewayExpedition => {
            Some(mission.layer * 10)
        }
        _ => None,
    }
}

/// Apply casualties (injuries and losses) to the squad based on outcome.
///
/// Returns (injured_ids, lost_ids).
fn apply_mission_casualties(
    mission: &Mission,
    prestige: &mut DeepPrestige,
    _persistent: &DeepPersistent,
    outcome: &MissionOutcome,
    rng: &mut impl Rng,
) -> (Vec<u64>, Vec<u64>) {
    let mut injured = Vec::new();
    let mut lost = Vec::new();

    // Safe missions never cause casualties.
    if matches!(
        mission.mission_type,
        MissionType::SupplyRun | MissionType::Construction(_)
    ) {
        release_squad_from_mission(mission, prestige);
        return (injured, lost);
    }

    let risk_tier = mission.mission_type.risk_tier();

    // Medic squad bonus: if squad contains a Medic, all other members get
    // a 20% injury reduction (the Medic's healing triage during the mission).
    let has_medic = mission.squad.iter().any(|&id| {
        prestige
            .find_merc(id)
            .map(|m| m.archetype == MercArchetype::Medic)
            .unwrap_or(false)
    });

    for &id in &mission.squad {
        let Some(merc) = prestige.find_merc(id) else {
            continue;
        };
        let resilience = merc.effective_resilience();

        // Base injury probability from outcome and risk tier.
        let base_injury_chance = match outcome {
            MissionOutcome::Success => 0.0,
            MissionOutcome::PartialSuccess => 0.15 + risk_tier as f64 * 0.10,
            MissionOutcome::Failure => 0.35 + risk_tier as f64 * 0.15,
        };

        // Resilience reduces injury chance (cap at 40%).
        let resilience_factor = 1.0 - (resilience as f64 / 100.0).min(0.40);

        // Medic triage bonus: 20% reduction for non-Medic squad members.
        let medic_factor = if has_medic && merc.archetype != MercArchetype::Medic {
            0.80
        } else {
            1.0
        };

        let injury_chance =
            (base_injury_chance * resilience_factor * medic_factor).clamp(0.0, 0.80);

        if rng.random::<f64>() < injury_chance {
            // Determine if injury or loss.
            let loss_chance = match outcome {
                MissionOutcome::Failure => 0.15 + risk_tier as f64 * 0.10,
                _ => 0.05,
            };

            if rng.random::<f64>() < loss_chance {
                lost.push(id);
                if let Some(m) = prestige.find_merc_mut(id) {
                    mark_merc_lost(m);
                }
                prestige.total_mercs_lost += 1;
            } else {
                injured.push(id);
                if let Some(m) = prestige.find_merc_mut(id) {
                    injure_merc(m, super::mercenaries::InjurySeverity::Moderate, rng);
                }
            }
        }
    }

    // Release non-lost, non-injured mercs from mission.
    for &id in &mission.squad {
        if !injured.contains(&id) && !lost.contains(&id) {
            if let Some(merc) = prestige.find_merc_mut(id) {
                if matches!(merc.status, MercStatus::OnMission(_)) {
                    merc.status = MercStatus::Available;
                }
            }
        }
    }

    (injured, lost)
}

/// Release all squad members from `OnMission` status back to `Available`.
///
/// Used for safe missions that never cause casualties.
fn release_squad_from_mission(mission: &Mission, prestige: &mut DeepPrestige) {
    for &id in &mission.squad {
        if let Some(merc) = prestige.find_merc_mut(id) {
            if matches!(merc.status, MercStatus::OnMission(_)) {
                merc.status = MercStatus::Available;
            }
        }
    }
}

/// Apply XP to squad members and compute level-ups.
///
/// Returns a list of (merc_id, levels_gained) for the notification display.
/// When `danger_bonus` is true (squad underpowered, power_ratio < 1.0), merc XP
/// is multiplied by 1.5x as a reward for the risky rush.
fn apply_squad_xp(
    mission: &Mission,
    prestige: &mut DeepPrestige,
    outcome: &MissionOutcome,
    danger_bonus: bool,
) -> Vec<(u64, u32)> {
    let base_xp = merc_xp_per_mission(mission.mission_type, mission.layer);
    let danger_mult = if danger_bonus { 1.5 } else { 1.0 };
    let xp =
        (base_xp as f64 * outcome_mark_multiplier(outcome.clone()) * danger_mult).round() as u32;

    let mut level_ups = Vec::new();

    for &id in &mission.squad {
        let Some(merc) = prestige.find_merc_mut(id) else {
            continue;
        };

        // Don't give XP to lost mercs.
        if matches!(merc.status, MercStatus::Lost) {
            continue;
        }

        merc.missions_completed += 1;

        let mut levels_gained = 0u32;
        let _ = merc_xp_to_next_level(merc.level); // for future XP-based leveling

        // Level-up check: use missions_completed as proxy for accumulated XP.
        // Each `missions_to_next_level(level)` completed missions earns one level.
        let missions_needed = super::types::Mercenary::missions_to_next_level(merc.level);
        if missions_needed > 0 && merc.missions_completed % missions_needed == 0 {
            merc.level += 1;
            levels_gained += 1;
        }
        let _ = xp; // xp tracked per-merc for future granular system

        if levels_gained > 0 {
            level_ups.push((id, levels_gained));
        }
    }

    level_ups
}

// ── Offline Resolution ─────────────────────────────────────────────────────────

/// Resolve all active missions that should have completed while the game was closed.
///
/// Called on game load. Fast-forwards mission timers and resolves events/outcomes
/// using the same logic as the normal tick path.
///
/// Returns a summary of what was resolved for display in the post-load notification.
pub fn resolve_offline_missions(
    prestige: &mut DeepPrestige,
    persistent: &mut DeepPersistent,
    rng: &mut impl Rng,
) -> OfflineResolutionSummary {
    let now = Utc::now();
    let mut summary = OfflineResolutionSummary::default();

    let mut completed_ids: Vec<u64> = Vec::new();

    for mission in &mut prestige.active_missions {
        if !matches!(
            mission.status,
            MissionStatus::Active | MissionStatus::EventPending
        ) {
            continue;
        }

        // Auto-resolve all pending events that fired while offline.
        let squad_archetypes: Vec<MercArchetype> = mission
            .squad
            .iter()
            .filter_map(|&id| {
                prestige
                    .roster
                    .iter()
                    .find(|m| m.id == id)
                    .map(|m| m.archetype)
            })
            .collect();

        // Auto-resolve all remaining events by advancing time to now.
        // tick_mission_events handles the 2h timeout check; on load we call it
        // once with `now` which may be many hours after the event fired.
        let tick_result = tick_mission_events(mission, &squad_archetypes, now, rng);
        summary.events_auto_resolved += tick_result.auto_resolved.len();

        // Apply time deltas.
        for (_, resolution) in &tick_result.auto_resolved {
            if resolution.time_delta_secs != 0 {
                let delta = Duration::seconds(resolution.time_delta_secs);
                mission.ends_at += delta;
            }
        }

        // If the mission completed while offline, queue it for resolution.
        if mission.is_time_elapsed(now) && !matches!(mission.status, MissionStatus::EventPending) {
            completed_ids.push(mission.id);
        }
    }

    // Resolve completed missions.
    for id in completed_ids {
        if let Some(idx) = prestige.active_missions.iter().position(|m| m.id == id) {
            let mut mission = prestige.active_missions.remove(idx);
            resolve_mission(&mut mission, prestige, persistent, rng);
            summary.missions_resolved += 1;
            if let Some(ref result) = mission.result {
                summary.total_marks_earned += result.marks_earned;
                summary.total_xp_earned += result.xp_earned;
            }
            prestige.pending_results.push(mission);
        }
    }

    summary
}

/// Summary of what was resolved during offline fast-forward on game load.
#[derive(Debug, Default, Clone)]
pub struct OfflineResolutionSummary {
    /// Number of missions that completed while offline.
    pub missions_resolved: usize,
    /// Number of check-in events auto-resolved during offline catch-up.
    pub events_auto_resolved: usize,
    /// Total Warband Marks awarded from completed missions.
    pub total_marks_earned: u32,
    /// Total character XP awarded.
    pub total_xp_earned: u32,
}

// ── Free Daily Supply Run ──────────────────────────────────────────────────────

/// Whether the daily free supply run slot has been used today (UTC calendar day).
///
/// Callers should persist this flag alongside `DeepPrestige`.
/// This function is a helper for calculating the reset; the actual flag is
/// managed by the caller.
pub fn daily_supply_run_resets_at(last_used_at: DateTime<Utc>) -> DateTime<Utc> {
    // Reset at midnight UTC the next calendar day.
    let day = last_used_at
        .date_naive()
        .succ_opt()
        .unwrap_or(last_used_at.date_naive());
    day.and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc())
        .unwrap_or(last_used_at + Duration::hours(24))
}

/// Check if the daily free supply run is available given the last-used timestamp.
pub fn is_daily_supply_run_available(
    last_used_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    match last_used_at {
        None => true,
        Some(used) => now >= daily_supply_run_resets_at(used),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::types::{
        DeepPersistent, DeepPrestige, GuildRank, MercArchetype, MercStatus, Mercenary,
        MissionOutcome, MissionStatus, MissionType,
    };
    use chrono::Utc;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn seeded_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    fn make_merc(id: u64, archetype: MercArchetype, power: u32) -> Mercenary {
        Mercenary {
            id,
            name: format!("Merc_{}", id),
            archetype,
            power,
            resilience: 10,
            expertise: 8,
            level: 1,
            missions_completed: 0,
            status: MercStatus::Available,
        }
    }

    fn make_prestige_with_mercs(mercs: Vec<Mercenary>) -> DeepPrestige {
        let mut p = DeepPrestige::new();
        p.roster = mercs;
        p
    }

    fn make_available_mission(mission_type: MissionType, layer: u32) -> AvailableMission {
        AvailableMission {
            mission_type,
            layer,
            duration_secs: 8 * 3600,
            min_squad_power: 20,
            required_archetype: None,
            recommended_archetype: None,
            marks_cost: 80,
            description: "Test mission".to_string(),
        }
    }

    // ── available_mission_count ───────────────────────────────────────────────

    #[test]
    fn test_available_mission_count_by_rank() {
        assert_eq!(available_mission_count(GuildRank(1)), 3);
        assert_eq!(available_mission_count(GuildRank(2)), 3);
        assert_eq!(available_mission_count(GuildRank(3)), 4);
        assert_eq!(available_mission_count(GuildRank(4)), 4);
        assert_eq!(available_mission_count(GuildRank(5)), 5);
    }

    // ── generate_mission_pool ─────────────────────────────────────────────────

    #[test]
    fn test_generate_mission_pool_returns_correct_count() {
        let mut rng = seeded_rng();
        let persistent = DeepPersistent::new();
        let pool = generate_mission_pool(&persistent, &mut rng);
        let expected = available_mission_count(persistent.guild_rank);
        assert_eq!(pool.len(), expected);
    }

    #[test]
    fn test_generate_mission_pool_includes_supply_run() {
        let mut rng = seeded_rng();
        let persistent = DeepPersistent::new();
        let pool = generate_mission_pool(&persistent, &mut rng);
        assert!(
            pool.iter()
                .any(|m| m.mission_type == MissionType::SupplyRun),
            "Pool must always include a Supply Run"
        );
    }

    #[test]
    fn test_generate_mission_pool_all_have_positive_duration() {
        let mut rng = seeded_rng();
        let persistent = DeepPersistent::new();
        let pool = generate_mission_pool(&persistent, &mut rng);
        for mission in &pool {
            assert!(
                mission.duration_secs > 0,
                "Every mission must have positive duration"
            );
        }
    }

    // ── validate_squad_assignment ─────────────────────────────────────────────

    #[test]
    fn test_validate_squad_empty_squad_fails() {
        let persistent = DeepPersistent::new();
        let prestige = DeepPrestige::new();
        let available = make_available_mission(MissionType::Expedition, 1);
        let result = validate_squad_assignment(&available, &[], &prestige, &persistent, false);
        assert_eq!(result, Err(SquadAssignmentError::EmptySquad));
    }

    #[test]
    fn test_validate_squad_insufficient_power_fails() {
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 5); // too weak
        let prestige = make_prestige_with_mercs(vec![merc]);
        let mut available = make_available_mission(MissionType::Expedition, 1);
        available.min_squad_power = 50;

        let result = validate_squad_assignment(&available, &[1], &prestige, &persistent, false);
        assert_eq!(
            result,
            Err(SquadAssignmentError::InsufficientPower {
                required: 50,
                actual: 5,
            })
        );
    }

    #[test]
    fn test_validate_squad_sufficient_power_succeeds() {
        let persistent = DeepPersistent::new();
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let prestige = make_prestige_with_mercs(vec![merc]);
        let mut available = make_available_mission(MissionType::Expedition, 1);
        available.marks_cost = 0;
        available.min_squad_power = 20;

        let result = validate_squad_assignment(&available, &[1], &prestige, &persistent, false);
        assert!(result.is_ok(), "Should succeed with sufficient power");
    }

    #[test]
    fn test_validate_squad_missing_required_archetype() {
        let persistent = DeepPersistent::new();
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let prestige = make_prestige_with_mercs(vec![merc]);
        let mut available = make_available_mission(MissionType::Breakthrough, 1);
        available.required_archetype = Some(MercArchetype::Medic);
        available.marks_cost = 0;

        let result = validate_squad_assignment(&available, &[1], &prestige, &persistent, false);
        assert_eq!(
            result,
            Err(SquadAssignmentError::MissingRequiredArchetype(
                MercArchetype::Medic
            ))
        );
    }

    #[test]
    fn test_validate_squad_insufficient_marks_fails() {
        let persistent = DeepPersistent::new();
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 10; // Need 80

        let available = make_available_mission(MissionType::Expedition, 1);

        let result = validate_squad_assignment(&available, &[1], &prestige, &persistent, false);
        assert_eq!(
            result,
            Err(SquadAssignmentError::InsufficientMarks {
                required: 80,
                available: 10,
            })
        );
    }

    #[test]
    fn test_validate_squad_free_supply_run_skips_marks_check() {
        let persistent = DeepPersistent::new();
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 0; // No marks

        let mut available = make_available_mission(MissionType::SupplyRun, 1);
        available.marks_cost = 20;
        available.min_squad_power = 20;

        let result = validate_squad_assignment(
            &available,
            &[1],
            &prestige,
            &persistent,
            true, /* free */
        );
        assert!(
            result.is_ok(),
            "Free daily supply run should skip marks check"
        );
    }

    #[test]
    fn test_validate_squad_concurrent_limit_enforced() {
        let mut persistent = DeepPersistent::new();
        // Guild Rank 1 allows 1 concurrent mission.
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);

        // Simulate one active mission already running.
        let now = Utc::now();
        let active = Mission {
            id: 999,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![],
            started_at: now,
            ends_at: now + Duration::hours(3),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };
        prestige.active_missions.push(active);

        let mut available = make_available_mission(MissionType::Recon, 1);
        available.marks_cost = 0;

        let result = validate_squad_assignment(&available, &[1], &prestige, &persistent, false);
        assert_eq!(result, Err(SquadAssignmentError::ConcurrentMissionLimit));
        let _ = persistent.next_mission_id(); // Suppress unused warning
    }

    // ── start_mission ─────────────────────────────────────────────────────────

    #[test]
    fn test_start_mission_deducts_marks() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 200;

        let available = make_available_mission(MissionType::Expedition, 1); // cost 80
        let now = Utc::now();

        let mission = start_mission(
            &available,
            &[1],
            &mut prestige,
            &mut persistent,
            false,
            now,
            &mut rng,
        );

        assert!(prestige.warband_marks < 200, "Marks should be deducted");
        assert_eq!(mission.squad, vec![1]);
        assert_eq!(mission.mission_type, MissionType::Expedition);
    }

    #[test]
    fn test_start_mission_free_supply_run_does_not_deduct_marks() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Scout, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 100;

        let mut available = make_available_mission(MissionType::SupplyRun, 1);
        available.marks_cost = 20;
        let now = Utc::now();

        let _mission = start_mission(
            &available,
            &[1],
            &mut prestige,
            &mut persistent,
            true,
            now,
            &mut rng,
        );
        assert_eq!(
            prestige.warband_marks, 100,
            "Free run should not deduct marks"
        );
    }

    #[test]
    fn test_start_mission_sets_merc_on_mission_status() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Medic, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 200;

        let available = make_available_mission(MissionType::Expedition, 1);
        let now = Utc::now();

        let mission = start_mission(
            &available,
            &[1],
            &mut prestige,
            &mut persistent,
            false,
            now,
            &mut rng,
        );

        let merc_status = &prestige.find_merc(1).unwrap().status;
        assert!(
            matches!(merc_status, MercStatus::OnMission(id) if *id == mission.id),
            "Merc should be on mission"
        );
    }

    #[test]
    fn test_start_mission_has_correct_timing() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 200;

        let available = make_available_mission(MissionType::SupplyRun, 1);
        let now = Utc::now();

        let mission = start_mission(
            &available,
            &[1],
            &mut prestige,
            &mut persistent,
            true,
            now,
            &mut rng,
        );

        assert_eq!(mission.started_at, now);
        assert!(mission.ends_at > now, "ends_at must be after started_at");
        // Supply run on Shallows: base 600s (10min). With overpowered modifier
        // (-10%), can be as low as 540s. Allow up to 2h for higher-tier supply runs.
        let duration = (mission.ends_at - mission.started_at).num_seconds();
        assert!(
            (540..=2 * 3600 + 60).contains(&duration),
            "Duration {} seconds out of expected range",
            duration
        );
    }

    #[test]
    fn test_start_breakthrough_generates_events() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.warband_marks = 500;

        let mut available = make_available_mission(MissionType::Breakthrough, 1);
        available.marks_cost = 150;
        let now = Utc::now();

        let mission = start_mission(
            &available,
            &[1],
            &mut prestige,
            &mut persistent,
            false,
            now,
            &mut rng,
        );

        // Breakthrough on layer 1 (Shallows) should have 3 events.
        assert_eq!(mission.events.len(), 3, "Breakthrough should have 3 events");
        assert_eq!(mission.pending_event_index, 0);
    }

    #[test]
    fn test_start_supply_run_generates_no_events() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);

        let available = make_available_mission(MissionType::SupplyRun, 1);
        let now = Utc::now();

        let mission = start_mission(
            &available,
            &[1],
            &mut prestige,
            &mut persistent,
            true,
            now,
            &mut rng,
        );
        assert!(
            mission.events.is_empty(),
            "Supply run should have no events"
        );
    }

    // ── resolve_mission ───────────────────────────────────────────────────────

    #[test]
    fn test_resolve_supply_run_always_succeeds() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        // Put merc on mission.
        prestige.roster[0].status = MercStatus::OnMission(1);

        let now = Utc::now();
        let mut mission = Mission {
            id: 1,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![1],
            started_at: now - Duration::hours(3),
            ends_at: now - Duration::minutes(1),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };

        resolve_mission(&mut mission, &mut prestige, &mut persistent, &mut rng);

        let result = mission.result.as_ref().unwrap();
        assert_eq!(result.outcome, MissionOutcome::Success);
        assert_eq!(
            result.injured_mercs,
            Vec::<u64>::new(),
            "Supply run should never injure"
        );
        assert_eq!(
            result.lost_mercs,
            Vec::<u64>::new(),
            "Supply run should never lose mercs"
        );
    }

    #[test]
    fn test_resolve_mission_awards_marks() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 100); // very powerful
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.roster[0].status = MercStatus::OnMission(1);
        let initial_marks = prestige.warband_marks;

        let now = Utc::now();
        let mut mission = Mission {
            id: 1,
            mission_type: MissionType::Expedition,
            layer: 1,
            squad: vec![1],
            started_at: now - Duration::hours(10),
            ends_at: now - Duration::minutes(1),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };

        resolve_mission(&mut mission, &mut prestige, &mut persistent, &mut rng);

        assert!(
            prestige.warband_marks > initial_marks,
            "Marks should increase after mission completion"
        );
    }

    #[test]
    fn test_resolve_breakthrough_marks_layer_cleared() {
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 200); // very powerful = likely success
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.roster[0].status = MercStatus::OnMission(1);

        // Use a seeded RNG that will produce a success outcome.
        let mut success_rng = ChaCha8Rng::seed_from_u64(0);

        let now = Utc::now();
        let mut mission = Mission {
            id: 1,
            mission_type: MissionType::Breakthrough,
            layer: 1,
            squad: vec![1],
            started_at: now - Duration::hours(20),
            ends_at: now - Duration::minutes(1),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };

        resolve_mission(
            &mut mission,
            &mut prestige,
            &mut persistent,
            &mut success_rng,
        );

        let result = mission.result.as_ref().unwrap();
        // If succeeded or partially succeeded, layer should be cleared.
        if matches!(
            result.outcome,
            MissionOutcome::Success | MissionOutcome::PartialSuccess
        ) {
            assert!(
                persistent
                    .layer_record(1)
                    .map(|r| r.cleared)
                    .unwrap_or(false),
                "Layer 1 should be cleared after successful breakthrough"
            );
        }
    }

    #[test]
    fn test_resolve_mission_merc_returns_to_available_after_success() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.roster[0].status = MercStatus::OnMission(1);

        let now = Utc::now();
        let mut mission = Mission {
            id: 1,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![1],
            started_at: now - Duration::hours(3),
            ends_at: now - Duration::minutes(1),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };

        resolve_mission(&mut mission, &mut prestige, &mut persistent, &mut rng);

        // After supply run (always success, no casualties), merc should be available.
        let merc_status = &prestige.find_merc(1).unwrap().status;
        assert_eq!(
            *merc_status,
            MercStatus::Available,
            "Merc should be available after safe mission"
        );
    }

    // ── offline resolution ────────────────────────────────────────────────────

    #[test]
    fn test_offline_resolution_resolves_elapsed_missions() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.roster[0].status = MercStatus::OnMission(1);

        let now = Utc::now();
        // Mission that completed 2 hours ago.
        let mission = Mission {
            id: 1,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![1],
            started_at: now - Duration::hours(5),
            ends_at: now - Duration::hours(2),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };
        prestige.active_missions.push(mission);

        let summary = resolve_offline_missions(&mut prestige, &mut persistent, &mut rng);

        assert_eq!(summary.missions_resolved, 1);
        assert!(
            prestige.active_missions.is_empty(),
            "Completed mission should be removed from active"
        );
        assert_eq!(
            prestige.pending_results.len(),
            1,
            "Completed mission should be in pending_results"
        );
    }

    #[test]
    fn test_offline_resolution_does_not_affect_future_missions() {
        let mut rng = seeded_rng();
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 30);
        let mut prestige = make_prestige_with_mercs(vec![merc]);
        prestige.roster[0].status = MercStatus::OnMission(1);

        let now = Utc::now();
        // Mission not yet complete (ends in 5 hours).
        let mission = Mission {
            id: 1,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![1],
            started_at: now - Duration::hours(2),
            ends_at: now + Duration::hours(5),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };
        prestige.active_missions.push(mission);

        let summary = resolve_offline_missions(&mut prestige, &mut persistent, &mut rng);

        assert_eq!(summary.missions_resolved, 0);
        assert_eq!(
            prestige.active_missions.len(),
            1,
            "Future mission should still be active"
        );
    }

    // ── daily supply run ──────────────────────────────────────────────────────

    #[test]
    fn test_daily_supply_run_available_when_never_used() {
        let now = Utc::now();
        assert!(is_daily_supply_run_available(None, now));
    }

    #[test]
    fn test_daily_supply_run_not_available_same_day() {
        let now = Utc::now();
        // Used just now (within same day).
        let used_at = now - Duration::minutes(30);
        assert!(!is_daily_supply_run_available(Some(used_at), now));
    }

    #[test]
    fn test_daily_supply_run_available_next_day() {
        let now = Utc::now();
        // Used over 24 hours ago.
        let used_at = now - Duration::hours(25);
        assert!(is_daily_supply_run_available(Some(used_at), now));
    }

    // ── compute_outcome ───────────────────────────────────────────────────────

    #[test]
    fn test_safe_missions_always_succeed() {
        let mut persistent = DeepPersistent::new();
        let _ = persistent.layer_record_mut(1);
        let merc = make_merc(1, MercArchetype::Vanguard, 1); // very weak
        let prestige = make_prestige_with_mercs(vec![merc]);

        let now = Utc::now();
        let supply_run = Mission {
            id: 1,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![1],
            started_at: now,
            ends_at: now + Duration::hours(2),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };

        // Run many seeds — always expect success.
        for seed in 0u64..20 {
            let mut test_rng = ChaCha8Rng::seed_from_u64(seed);
            let outcome = compute_outcome(&supply_run, &prestige, &persistent, &mut test_rng);
            assert_eq!(
                outcome,
                MissionOutcome::Success,
                "Supply run must always succeed (seed {})",
                seed
            );
        }
        let _ = persistent.frontier_layer(); // Suppress unused warning
    }

    #[test]
    fn test_overpowered_squad_high_success_rate() {
        let now = Utc::now();
        let persistent = DeepPersistent::new();
        // Threshold for Expedition layer 1 = 20. Squad power = 50 (250%).
        let merc = make_merc(1, MercArchetype::Vanguard, 50);
        let prestige = make_prestige_with_mercs(vec![merc]);

        let mission = Mission {
            id: 1,
            mission_type: MissionType::Expedition,
            layer: 1,
            squad: vec![1],
            started_at: now,
            ends_at: now + Duration::hours(10),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        };

        let mut success_count = 0;
        for seed in 0u64..50 {
            let mut test_rng = ChaCha8Rng::seed_from_u64(seed);
            if compute_outcome(&mission, &prestige, &persistent, &mut test_rng)
                == MissionOutcome::Success
            {
                success_count += 1;
            }
        }

        // Overpowered squad should succeed ≥80% of the time.
        assert!(
            success_count >= 40,
            "Overpowered squad should succeed ≥80% ({}/50)",
            success_count
        );
    }
}
