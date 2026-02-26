use super::mercenaries::generate_starter_roster;
use super::types::{
    deep_discovery_chance, DeepState, MercStatus, Mission, MissionStatus, MissionType,
};
use chrono::Utc;
use rand::Rng;
use rand::RngExt;

/// Attempts to discover The Deep.
///
/// Returns `true` if The Deep was discovered this tick.
/// Sets `deep.persistent.discovered = true` and initialises the first roster
/// with 3 starter mercenaries via [`generate_starter_roster`].
///
/// Mirrors the pattern used by `try_discover_haven()` and
/// `try_discover_soulforge()`.
pub fn try_discover_deep<R: Rng>(deep: &mut DeepState, prestige_rank: u32, rng: &mut R) -> bool {
    if deep.persistent.discovered {
        return false;
    }
    let chance = deep_discovery_chance(prestige_rank);
    if chance <= 0.0 {
        return false;
    }
    if rng.random::<f64>() >= chance {
        return false;
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
    // Seed starting Warband Marks — higher guild ranks get more to match increased costs.
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

    true
}

/// Check if the story chain should advance and optionally award fragments.
///
/// Called after rift resonance is incremented (during prestige).
/// Returns the new story stage if it advanced, or None.
pub fn advance_deep_story(deep: &mut DeepState, prestige_rank: u32) -> Option<u8> {
    if deep.persistent.discovered {
        return None;
    }

    // Award fragments based on resonance
    deep.maybe_award_rift_fragment();

    // Check stage progression
    deep.check_story_progression(prestige_rank)
}

/// Complete the discovery via the story chain. Called when the player
/// presses [D] after stage 4 is reached (The Entrance).
pub fn complete_story_discovery<R: Rng>(deep: &mut DeepState, rng: &mut R) {
    if deep.persistent.discovered || deep.persistent.deep_story_stage < 4 {
        return;
    }
    deep.persistent.discovered = true;
    deep.persistent.deep_story_stage = 5;
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        rng,
    );
    deep.prestige.roster.extend(starters);
    deep.prestige.available_missions =
        super::missions::generate_mission_pool(&deep.persistent, rng);
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

#[cfg(test)]
mod tests {
    use super::super::types::{
        DeepState, MercArchetype, MercStatus, Mercenary, DEEP_MIN_PRESTIGE_RANK,
    };
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn seeded_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_try_discover_deep_blocked_below_min_prestige() {
        let mut rng = seeded_rng();
        let mut deep = DeepState::new();

        for _ in 0..10_000 {
            assert!(!try_discover_deep(
                &mut deep,
                DEEP_MIN_PRESTIGE_RANK - 1,
                &mut rng
            ));
        }
        assert!(!deep.persistent.discovered);
    }

    #[test]
    fn test_try_discover_deep_blocked_when_already_discovered() {
        let mut rng = seeded_rng();
        let mut deep = DeepState::new();
        deep.persistent.discovered = true;

        for _ in 0..1_000 {
            assert!(!try_discover_deep(
                &mut deep,
                DEEP_MIN_PRESTIGE_RANK,
                &mut rng
            ));
        }
    }

    #[test]
    fn test_try_discover_deep_eventually_discovers() {
        let mut rng = seeded_rng();
        let mut deep = DeepState::new();

        let mut discovered = false;
        for _ in 0..500_000 {
            if try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK, &mut rng) {
                discovered = true;
                break;
            }
        }
        assert!(discovered, "Expected discovery within 500k ticks at P15");
        assert!(deep.persistent.discovered);
    }

    #[test]
    fn test_discovery_initialises_starter_roster() {
        let mut rng = seeded_rng();
        let mut deep = DeepState::new();

        let mut discovered = false;
        for _ in 0..500_000 {
            if try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK, &mut rng) {
                discovered = true;
                break;
            }
        }
        assert!(discovered);
        assert_eq!(
            deep.prestige.roster.len(),
            3,
            "Starter roster must have exactly 3 mercs"
        );

        let archetypes: Vec<MercArchetype> =
            deep.prestige.roster.iter().map(|m| m.archetype).collect();
        assert!(
            archetypes.contains(&MercArchetype::Vanguard),
            "Starter roster must include a Vanguard"
        );
        assert!(
            archetypes.contains(&MercArchetype::Scout),
            "Starter roster must include a Scout"
        );
        assert!(
            archetypes.contains(&MercArchetype::Medic),
            "Starter roster must include a Medic"
        );

        for merc in &deep.prestige.roster {
            assert_eq!(merc.level, Mercenary::BASE_LEVEL);
            assert_eq!(merc.missions_completed, 0);
            // First Orders auto-queues all 3 starter mercs on a mission.
            assert!(
                matches!(merc.status, MercStatus::OnMission(_)),
                "Starter mercs should be on First Orders mission, got {:?}",
                merc.status
            );
        }

        // Verify First Orders mission was queued.
        assert_eq!(
            deep.prestige.active_missions.len(),
            1,
            "First Orders mission should be queued"
        );
        assert!(
            deep.prestige.active_missions[0].is_first_orders,
            "Queued mission should be First Orders"
        );
        assert!(
            deep.persistent.first_orders_queued,
            "first_orders_queued flag should be set"
        );
    }

    #[test]
    fn test_discovery_probability_in_range() {
        let mut rng = seeded_rng();
        let mut discoveries = 0u32;
        let trials = 500_000u32;

        for _ in 0..trials {
            let mut deep = DeepState::new();
            if try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK, &mut rng) {
                discoveries += 1;
            }
        }

        // Expected: ~7 per 500k ticks (0.000014 base chance).
        // Allow a wide band for probabilistic tests.
        assert!(
            (1..=50).contains(&discoveries),
            "Expected ~7 discoveries per 500k ticks, got {}",
            discoveries
        );
    }

    #[test]
    fn test_merc_id_counter_increments_on_discovery() {
        let mut rng = seeded_rng();
        let mut deep = DeepState::new();

        for _ in 0..500_000 {
            if try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK, &mut rng) {
                break;
            }
        }

        assert_eq!(
            deep.persistent.merc_id_counter, 3,
            "Three starter mercs should have consumed IDs 1, 2, 3"
        );
        let ids: Vec<u64> = deep.prestige.roster.iter().map(|m| m.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}
