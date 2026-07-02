use super::mercenaries::{generate_recruit_pool, generate_starter_roster};
use super::types::DeepState;
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
    deep.session
        .roster
        .extend(starters.into_iter().map(|m| (m.id, m)));
    deep.session.available_missions = super::missions::generate_mission_pool(
        &deep.persistent,
        &deep.session.active_missions,
        rng,
    );
    deep.session.recruit_pool = generate_recruit_pool(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        rng,
    );
    deep.session.pool_refreshed_at = Some(Utc::now());
    deep.session.warband_marks = match deep.persistent.guild_rank.0 {
        1 => 50,
        2 => 100,
        3 => 200,
        4 => 350,
        5 => 500,
        _ => 50,
    };
}
