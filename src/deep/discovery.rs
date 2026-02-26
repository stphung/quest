use super::mercenaries::generate_starter_roster;
use super::types::{
    DeepState, MercStatus, Mission, MissionStatus, MissionType, STORY_STAGE_DISCOVERED,
    STORY_STAGE_ENTRANCE,
};
use chrono::Utc;
use rand::Rng;

/// Check if the story chain should advance.
///
/// Called after rift resonance is incremented (during prestige).
/// Returns the new story stage if it advanced, or None.
pub fn advance_deep_story(deep: &mut DeepState, prestige_rank: u32) -> Option<u8> {
    if deep.persistent.discovered {
        return None;
    }

    // Check stage progression
    deep.check_story_progression(prestige_rank)
}

/// Complete the discovery via the story chain. Called when the player
/// presses [D] after stage 4 is reached (The Entrance).
pub fn complete_story_discovery<R: Rng>(deep: &mut DeepState, rng: &mut R) {
    if deep.persistent.discovered || deep.persistent.deep_story_stage < STORY_STAGE_ENTRANCE {
        return;
    }
    deep.persistent.discovered = true;
    deep.persistent.deep_story_stage = STORY_STAGE_DISCOVERED;
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
