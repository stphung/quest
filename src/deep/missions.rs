//! Mission system for The Deep.
//!
//! Handles mission creation, validation, launch, resolution, and reward
//! calculation. All time handling uses injected Unix timestamps (`i64`) so
//! that every function is deterministically testable without calling
//! `Utc::now()`.

use rand::{Rng, RngExt};

use crate::deep::layers::{available_mission_types, layer_difficulty};
use crate::deep::types::{
    ActiveMission, CompletedMission, EventType, InfrastructureType, LayerState, Mercenary,
    MercStatus, MissionEvent, MissionResult, MissionType, TheDeepState,
};

// =========================================================================
// Duration
// =========================================================================

/// Returns the wall-clock mission duration in seconds.
///
/// Base ranges:
/// - `SupplyRun`     — 7200–14400 s  (2–4 h)
/// - `Recon`         — 14400–28800 s (4–8 h)
/// - `Expedition`    — 28800–57600 s (8–16 h)
/// - `Breakthrough`  — 64800–86400 s (18–24 h)
/// - `Construction`  — 14400–28800 s (4–8 h)
///
/// An [`InfrastructureType::Outpost`] on the target layer reduces the
/// duration by 25 %.
///
/// Note: Saboteur reduction (−15 %) is applied at launch time by the caller
/// so that squad composition is not baked into the mission template.
pub fn calculate_mission_duration_secs<R: Rng>(
    mission_type: MissionType,
    layer_id: u8,
    state: &TheDeepState,
    rng: &mut R,
) -> u64 {
    let (min, max) = match mission_type {
        MissionType::SupplyRun => (7_200u64, 14_400),
        MissionType::Recon => (14_400, 28_800),
        MissionType::Expedition => (28_800, 57_600),
        MissionType::Breakthrough => (64_800, 86_400),
        MissionType::Construction => (14_400, 28_800),
    };

    let base: u64 = rng.random_range(min..=max);

    // Apply outpost reduction if built on this layer.
    let has_outpost = state
        .account
        .layers
        .iter()
        .find(|l| l.layer_number == layer_id)
        .is_some_and(|l| l.infrastructure.contains(&InfrastructureType::Outpost));

    if has_outpost {
        base * 3 / 4
    } else {
        base
    }
}

// =========================================================================
// Cost
// =========================================================================

/// Returns the Warband Marks cost to launch a mission.
///
/// Formula per type:
/// - `SupplyRun`    — `20 + layer_id * 2`
/// - `Recon`        — `30 + layer_id * 3`
/// - `Expedition`   — `50 + layer_id * 5`
/// - `Breakthrough` — `80 + layer_id * 8`
/// - `Construction` — `0` (infrastructure cost is separate)
pub fn calculate_mission_cost(mission_type: MissionType, layer_id: u8) -> u32 {
    let id = layer_id as u32;
    match mission_type {
        MissionType::SupplyRun => 20 + id * 2,
        MissionType::Recon => 30 + id * 3,
        MissionType::Expedition => 50 + id * 5,
        MissionType::Breakthrough => 80 + id * 8,
        MissionType::Construction => 0,
    }
}

// =========================================================================
// Reward
// =========================================================================

/// Returns the Warband Marks reward for a completed mission.
///
/// Base reward per type and result:
///
/// | Type          | Success                  | Partial      | Failure   |
/// |---------------|--------------------------|--------------|-----------|
/// | SupplyRun     | `35 + layer * 4`         | 70 %         | 30 %      |
/// | Recon         | `40 + layer * 4`         | 70 %         | 30 %      |
/// | Expedition    | `80 + layer * 10`        | 70 %         | 30 %      |
/// | Breakthrough  | `200 + layer * 15`       | 70 %         | 30 %      |
/// | Construction  | `0`                      | `0`          | `0`       |
pub fn calculate_mission_reward(
    mission_type: MissionType,
    layer_id: u8,
    result: MissionResult,
) -> u32 {
    let id = layer_id as u32;
    let full = match mission_type {
        MissionType::SupplyRun => 35 + id * 4,
        MissionType::Recon => 40 + id * 4,
        MissionType::Expedition => 80 + id * 10,
        MissionType::Breakthrough => 200 + id * 15,
        MissionType::Construction => return 0,
    };

    match result {
        MissionResult::Success => full,
        MissionResult::PartialSuccess => full * 70 / 100,
        MissionResult::Failure => full * 30 / 100,
    }
}

// =========================================================================
// Mission creation helpers
// =========================================================================

/// Generates the check-in event schedule for a mission.
///
/// Count by type:
/// - SupplyRun:    0–1 events
/// - Recon:        0–1 events  (kept consistent with SupplyRun for simplicity)
/// - Expedition:   1–2 events
/// - Breakthrough: 2–4 events
/// - Construction: 0 events
fn generate_events<R: Rng>(mission_type: MissionType, rng: &mut R) -> Vec<MissionEvent> {
    let count: usize = match mission_type {
        MissionType::SupplyRun => rng.random_range(0..=1),
        MissionType::Recon => rng.random_range(1..=2),
        MissionType::Expedition => rng.random_range(1..=2),
        MissionType::Breakthrough => rng.random_range(2..=4),
        MissionType::Construction => 0,
    };

    // Distribute events across available trigger points (25, 50, 75).
    let mut trigger_points: Vec<u8> = vec![25, 50, 75];
    // Shuffle so we pick a random subset when count < 3.
    for i in (1..trigger_points.len()).rev() {
        let j = rng.random_range(0..=i);
        trigger_points.swap(i, j);
    }
    trigger_points.truncate(count);
    trigger_points.sort_unstable();

    let event_types = [
        EventType::CaveIn,
        EventType::Ambush,
        EventType::FloodedPassage,
        EventType::AncientDoor,
        EventType::Tremor,
    ];

    trigger_points
        .into_iter()
        .map(|pct| {
            let idx = rng.random_range(0..event_types.len());
            MissionEvent {
                trigger_at_percent: pct,
                event_type: event_types[idx],
                resolved: false,
                resolution: None,
            }
        })
        .collect()
}

/// Creates an [`ActiveMission`] with computed duration, cost, and check-in events.
///
/// The returned mission has `squad` set to an empty `Vec` — assign squad members
/// before calling [`start_mission`].
pub fn create_mission<R: Rng>(
    mission_type: MissionType,
    layer_id: u8,
    state: &TheDeepState,
    now_unix: i64,
    rng: &mut R,
) -> ActiveMission {
    // Use a simple incrementing ID based on mission count across all state.
    let id = (state.run.active_missions.len() + state.run.completed_missions.len() + 1) as u32;

    let duration_secs = calculate_mission_duration_secs(mission_type, layer_id, state, rng);
    let cost = calculate_mission_cost(mission_type, layer_id);
    let events = generate_events(mission_type, rng);

    ActiveMission {
        id,
        mission_type,
        layer: layer_id,
        squad: Vec::new(),
        start_time: now_unix,
        duration_secs,
        cost,
        events,
        events_resolved: 0,
    }
}

// =========================================================================
// Mission launch
// =========================================================================

/// Validates and launches a mission, deducting Marks and marking mercs as
/// [`MercStatus::OnMission`].
///
/// Validation rules (in order):
/// 1. Player can afford the mission cost.
/// 2. Active mission slots are not full (determined by guild rank).
/// 3. Every merc in `squad_ids` is present in the roster and [`MercStatus::Ready`].
///
/// On success the mission is pushed into `state.run.active_missions` and the
/// squad mercs are set to [`MercStatus::OnMission`].
pub fn start_mission(
    state: &mut TheDeepState,
    mut mission: ActiveMission,
    squad_ids: Vec<u32>,
) -> Result<(), &'static str> {
    // 1. Afford check.
    if state.run.warband_marks < mission.cost {
        return Err("insufficient Warband Marks");
    }

    // 2. Mission slot check.
    let slots = state.account.guild_rank.mission_slots();
    if state.run.active_missions.len() >= slots {
        return Err("no mission slots available");
    }

    // 3. Squad availability check.
    for &id in &squad_ids {
        let merc = state
            .run
            .mercenaries
            .iter()
            .find(|m| m.id == id)
            .ok_or("mercenary not found in roster")?;
        if merc.status != MercStatus::Ready {
            return Err("mercenary is not available");
        }
    }

    // All checks passed — commit.
    state.run.warband_marks -= mission.cost;
    mission.squad = squad_ids.clone();

    for id in squad_ids {
        if let Some(merc) = state.run.mercenaries.iter_mut().find(|m| m.id == id) {
            merc.status = MercStatus::OnMission;
        }
    }

    state.run.active_missions.push(mission);
    Ok(())
}

// =========================================================================
// Squad power
// =========================================================================

/// Returns the combined power of a set of mercenaries.
pub fn calculate_squad_power(mercs: &[&Mercenary]) -> u32 {
    mercs.iter().map(|m| m.power).sum()
}

// =========================================================================
// Mission resolution
// =========================================================================

/// Resolves an active mission, computing outcome, rewards, injuries, and
/// familiarity gains.
///
/// Outcome thresholds (squad_power vs layer_difficulty):
/// - `>= difficulty * 0.8` → [`MissionResult::Success`]
/// - `>= difficulty * 0.5` → [`MissionResult::PartialSuccess`]
/// - otherwise             → [`MissionResult::Failure`]
///
/// For [`MissionType::Breakthrough`] success:
/// - Marks the layer as cleared in `state.account.layers`.
/// - If this layer extends the deepest known layer, the layer list is grown.
///
/// Returns the [`CompletedMission`] and removes the mission from
/// `state.run.active_missions`.
///
/// # Panics
///
/// Panics if `mission_index` is out of bounds.
pub fn resolve_mission<R: Rng>(
    state: &mut TheDeepState,
    mission_index: usize,
    rng: &mut R,
) -> CompletedMission {
    let mission = state.run.active_missions.remove(mission_index);
    let layer_id = mission.layer;
    let squad_ids = mission.squad.clone();

    // Compute squad power from current roster state.
    let squad_power: u32 = squad_ids
        .iter()
        .filter_map(|&id| state.run.mercenaries.iter().find(|m| m.id == id))
        .map(|m| m.power)
        .sum();

    let difficulty = layer_difficulty(layer_id);

    let result = if squad_power as f32 >= difficulty as f32 * 0.8 {
        MissionResult::Success
    } else if squad_power as f32 >= difficulty as f32 * 0.5 {
        MissionResult::PartialSuccess
    } else {
        MissionResult::Failure
    };

    let marks_earned = calculate_mission_reward(mission.mission_type, layer_id, result);

    // Determine injuries: on failure or partial success, random mercs may be
    // injured. On success, no injuries.
    let mut injuries: Vec<u32> = Vec::new();
    match result {
        MissionResult::Failure => {
            // 50 % chance each merc is injured on failure.
            for &id in &squad_ids {
                if rng.random_bool(0.5) {
                    injuries.push(id);
                }
            }
        }
        MissionResult::PartialSuccess => {
            // 25 % chance each merc is injured on partial.
            for &id in &squad_ids {
                if rng.random_bool(0.25) {
                    injuries.push(id);
                }
            }
        }
        MissionResult::Success => {}
    }

    // Apply injuries.
    for &id in &injuries {
        if let Some(merc) = state.run.mercenaries.iter_mut().find(|m| m.id == id) {
            merc.status = MercStatus::Injured;
        }
    }

    // Return non-injured squad members to Ready.
    for &id in &squad_ids {
        if !injuries.contains(&id) {
            if let Some(merc) = state.run.mercenaries.iter_mut().find(|m| m.id == id) {
                merc.status = MercStatus::Ready;
                merc.missions_completed += 1;
            }
        }
    }

    // Familiarity gain: success 0.15, partial 0.08, failure 0.03.
    let familiarity_gain = match result {
        MissionResult::Success => 0.15f32,
        MissionResult::PartialSuccess => 0.08,
        MissionResult::Failure => 0.03,
    };

    ensure_layer_exists(&mut state.account.layers, layer_id);
    if let Some(layer) = state
        .account
        .layers
        .iter_mut()
        .find(|l| l.layer_number == layer_id)
    {
        layer.familiarity = (layer.familiarity + familiarity_gain).clamp(0.0, 1.0);

        // Breakthrough success: mark layer cleared.
        if mission.mission_type == MissionType::Breakthrough && result == MissionResult::Success {
            layer.cleared = true;
        }
    }

    // Apply marks.
    state.run.warband_marks += marks_earned;
    state.account.total_marks_earned += marks_earned as u64;

    let events_summary: Vec<String> = mission
        .events
        .iter()
        .map(|e| format!("{:?}", e.event_type))
        .collect();

    CompletedMission {
        mission_type: mission.mission_type,
        layer: layer_id,
        result,
        marks_earned,
        events_summary,
        injuries,
        collected: false,
    }
}

// =========================================================================
// Available missions pool
// =========================================================================

/// Generates 3–5 missions available to launch.
///
/// Pool composition:
/// - Always includes at least one [`MissionType::SupplyRun`] on a cleared layer
///   (if any cleared layers exist, otherwise on layer 1 as the default).
/// - Includes frontier missions (Recon / Expedition / Breakthrough) on the
///   deepest reached layer based on familiarity thresholds.
/// - Fills remaining slots with varied missions across known layers.
///
/// The returned missions have empty squads and valid costs/durations but are
/// *not* yet added to `state.run.active_missions`.
pub fn available_missions<R: Rng>(
    state: &TheDeepState,
    now_unix: i64,
    rng: &mut R,
) -> Vec<ActiveMission> {
    let mut pool: Vec<ActiveMission> = Vec::new();

    // Determine the deepest cleared layer (or 0 if none).
    let deepest_cleared = state
        .account
        .layers
        .iter()
        .filter(|l| l.cleared)
        .map(|l| l.layer_number)
        .max()
        .unwrap_or(0);

    // Frontier layer: one beyond the deepest cleared, or 1 if nothing cleared.
    let frontier_layer = deepest_cleared.saturating_add(1).max(1);

    // --- Guaranteed SupplyRun on a cleared layer ---
    let supply_layer = if deepest_cleared > 0 {
        deepest_cleared
    } else {
        1
    };
    pool.push(create_mission(
        MissionType::SupplyRun,
        supply_layer,
        state,
        now_unix,
        rng,
    ));

    // --- Frontier missions based on familiarity ---
    let frontier_familiarity = state
        .account
        .layers
        .iter()
        .find(|l| l.layer_number == frontier_layer)
        .map(|l| l.familiarity)
        .unwrap_or(0.0);

    let frontier_cleared = state
        .account
        .layers
        .iter()
        .find(|l| l.layer_number == frontier_layer)
        .is_some_and(|l| l.cleared);

    let frontier_types = available_mission_types(frontier_familiarity, frontier_cleared);

    // Add up to 2 frontier missions.
    let target_count = rng.random_range(3usize..=5);
    for mt in &frontier_types {
        if pool.len() >= target_count {
            break;
        }
        // Skip SupplyRun on frontier — already covered above.
        if *mt == MissionType::SupplyRun {
            continue;
        }
        pool.push(create_mission(*mt, frontier_layer, state, now_unix, rng));
    }

    // --- Fill remaining slots with cleared-layer variety ---
    let cleared_layers: Vec<u8> = state
        .account
        .layers
        .iter()
        .filter(|l| l.cleared)
        .map(|l| l.layer_number)
        .collect();

    while pool.len() < target_count && !cleared_layers.is_empty() {
        let idx = rng.random_range(0..cleared_layers.len());
        let layer = cleared_layers[idx];
        let fill_types = [MissionType::SupplyRun, MissionType::Recon];
        let ft = fill_types[rng.random_range(0..fill_types.len())];
        pool.push(create_mission(ft, layer, state, now_unix, rng));
    }

    // --- Fallback: pad to minimum 3 with additional frontier missions ---
    // This handles the early-game case where no layers are cleared yet.
    // Only SupplyRun and Recon are available at 0.0 familiarity; we may add
    // duplicate types to guarantee the minimum pool size.
    while pool.len() < target_count {
        let fallback_types = [MissionType::SupplyRun, MissionType::Recon];
        let ft = fallback_types[rng.random_range(0..fallback_types.len())];
        pool.push(create_mission(ft, frontier_layer, state, now_unix, rng));
    }

    pool
}

// =========================================================================
// Internal helpers
// =========================================================================

/// Ensures `layers` contains an entry for `layer_number`, growing the vec as
/// needed with default (uncleared, zero-familiarity) entries.
fn ensure_layer_exists(layers: &mut Vec<LayerState>, layer_number: u8) {
    if layers.iter().any(|l| l.layer_number == layer_number) {
        return;
    }
    layers.push(LayerState {
        layer_number,
        familiarity: 0.0,
        cleared: false,
        infrastructure: Vec::new(),
    });
}
