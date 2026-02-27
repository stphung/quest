use super::mercenaries::generate_starter_roster;
use super::types::{DeepState, MercStatus, Mission, MissionStatus, MissionType};
use chrono::Utc;
use rand::Rng;

/// Complete The Deep discovery. Called when the trigger condition is met.
pub fn complete_discovery<R: Rng>(deep: &mut DeepState, rng: &mut R) {
    if deep.persistent.discovered {
        return;
    }
    deep.persistent.discovered = true;
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        rng,
    );
    deep.prestige.roster.extend(starters);
    deep.prestige.available_missions =
        super::missions::generate_mission_pool(&deep.persistent, rng);
    deep.prestige.pool_refreshed_at = Some(Utc::now());
    deep.prestige.warband_marks = match deep.persistent.guild_rank.0 {
        1 => 50,
        2 => 100,
        3 => 200,
        4 => 350,
        5 => 500,
        _ => 50,
    };

    // Queue the "First Orders" starter mission if never done before
    queue_first_orders(deep);
}

/// Queue the "First Orders" starter mission, putting all starter mercs on it.
///
/// Only runs once per account (tracked by `first_orders_queued`). The mission
/// is a 20-minute Recon on Layer 1 with guaranteed success, awarding +30
/// familiarity and 15 Warband Marks.
fn queue_first_orders(deep: &mut DeepState) {
    if deep.persistent.first_orders_queued {
        return;
    }
    deep.persistent.first_orders_queued = true;

    let squad_ids: Vec<u64> = deep.prestige.roster.iter().map(|m| m.id).collect();
    let mission_id = deep.persistent.next_mission_id();

    // Mark all starter mercs as on-mission
    for merc in deep.prestige.roster.iter_mut() {
        merc.status = MercStatus::OnMission(mission_id);
    }

    let now = Utc::now();
    let first_orders = Mission {
        id: mission_id,
        mission_type: MissionType::Recon,
        layer: 1,
        squad: squad_ids,
        started_at: now,
        ends_at: now + chrono::Duration::minutes(20),
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: true,
    };
    deep.prestige.active_missions.push(first_orders);
}
