#![allow(dead_code)]

use crate::achievements::Achievements;
use crate::deep::DeepState;
use rand::Rng;

/// Result of a Deep tick, indicating what changed.
#[derive(Debug, Default)]
pub struct DeepTickResult {
    /// Whether Deep state changed (missions completed or events fired).
    pub deep_changed: bool,
    /// Whether achievement state changed (missions completed or mercs lost, non-debug only).
    pub achievements_changed: bool,
}

/// Facade: tick Deep system — resolve missions, fire achievement handlers.
///
/// Replicates the logic from `tick_stages::tick_deep_missions`, calling
/// `tick_all_missions()` and firing achievement side-effects.
pub fn tick_deep_facade<R: Rng>(
    deep: &mut DeepState,
    achievements: &mut Achievements,
    character_name: &str,
    debug_mode: bool,
    rng: &mut R,
) -> DeepTickResult {
    let mut result = DeepTickResult::default();

    if !deep.persistent.discovered {
        return result;
    }

    let now = chrono::Utc::now();
    let summary = crate::deep::missions::tick_all_missions(
        &mut deep.prestige,
        &mut deep.persistent,
        now,
        rng,
    );

    if summary.missions_completed > 0 || summary.events_fired > 0 {
        result.deep_changed = true;
    }

    // Fire achievement handlers for completed missions
    for _ in 0..summary.missions_completed {
        achievements.on_deep_mission_complete(Some(character_name));
    }
    for layer in &summary.breakthroughs {
        achievements.on_deep_breakthrough(*layer, Some(character_name));
    }
    for _ in 0..summary.mercs_lost {
        achievements.on_deep_merc_lost(Some(character_name));
    }
    if summary.gateway_opened {
        achievements.on_deep_gateway_opened(Some(character_name));
    }

    if (summary.missions_completed > 0 || summary.mercs_lost > 0) && !debug_mode {
        result.achievements_changed = true;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::mercenaries::MercQuality;
    use crate::deep::types::{MercStatus, Mercenary, Mission, MissionStatus, MissionType};
    use chrono::{Duration, Utc};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn make_merc(id: u64, power: u32, resilience: u32) -> Mercenary {
        Mercenary {
            id,
            name: format!("Merc_{id}"),
            archetype: crate::deep::types::MercArchetype::Vanguard,
            power,
            resilience,
            expertise: 8,
            level: 1,
            missions_completed: 0,
            quality: MercQuality::Common,
            status: MercStatus::Available,
        }
    }

    fn make_mission(id: u64, mission_type: MissionType, layer: u32, squad: Vec<u64>) -> Mission {
        let now = Utc::now();
        Mission {
            id,
            mission_type,
            layer,
            squad,
            started_at: now - Duration::hours(1),
            ends_at: now - Duration::minutes(1),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        }
    }

    #[test]
    fn test_deep_facade_not_discovered() {
        let mut deep = DeepState::default();
        assert!(!deep.persistent.discovered);

        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = tick_deep_facade(&mut deep, &mut achievements, "TestHero", false, &mut rng);

        assert!(!result.deep_changed);
        assert!(!result.achievements_changed);
    }

    #[test]
    fn test_deep_facade_discovered_no_missions() {
        let mut deep = DeepState::default();
        deep.persistent.discovered = true;

        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = tick_deep_facade(&mut deep, &mut achievements, "TestHero", false, &mut rng);

        // No active missions, so nothing should change
        assert!(!result.deep_changed);
        assert!(!result.achievements_changed);
    }

    #[test]
    fn test_deep_facade_completed_mission_sets_deep_and_achievements_changed() {
        let mut deep = DeepState::default();
        deep.persistent.discovered = true;
        let merc = make_merc(1, 100, 10);
        deep.prestige.roster.insert(1, merc);
        deep.prestige
            .active_missions
            .push(make_mission(1, MissionType::SupplyRun, 1, vec![1]));

        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = tick_deep_facade(&mut deep, &mut achievements, "TestHero", false, &mut rng);

        assert!(result.deep_changed);
        assert!(result.achievements_changed);
        assert!(deep.prestige.active_missions.is_empty());
        assert_eq!(deep.prestige.pending_results.len(), 1);
    }

    #[test]
    fn test_deep_facade_debug_mode_suppresses_achievements_changed() {
        let mut deep = DeepState::default();
        deep.persistent.discovered = true;
        let merc = make_merc(1, 100, 10);
        deep.prestige.roster.insert(1, merc);
        deep.prestige
            .active_missions
            .push(make_mission(1, MissionType::SupplyRun, 1, vec![1]));

        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // debug_mode = true suppresses the achievements_changed save signal even
        // though a mission completed.
        let result = tick_deep_facade(&mut deep, &mut achievements, "TestHero", true, &mut rng);

        assert!(result.deep_changed);
        assert!(!result.achievements_changed);
    }

    #[test]
    fn test_deep_facade_breakthrough_success_clears_layer_and_reports_achievement() {
        let mut deep = DeepState::default();
        deep.persistent.discovered = true;
        // Massively overpowered squad relative to Layer 1's Breakthrough threshold (25)
        // guarantees an effective power ratio well above 1.5, which succeeds on all
        // but a 5% RNG roll — seed 42 lands in the 95% success band.
        let merc = make_merc(1, 10_000, 50);
        deep.prestige.roster.insert(1, merc);
        deep.prestige
            .active_missions
            .push(make_mission(1, MissionType::Breakthrough, 1, vec![1]));

        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = tick_deep_facade(&mut deep, &mut achievements, "TestHero", false, &mut rng);

        assert!(result.deep_changed);
        assert!(
            deep.persistent
                .layer_record(1)
                .map(|r| r.cleared)
                .unwrap_or(false),
            "Breakthrough success should clear Layer 1, confirming the breakthrough \
             summary reached the facade's achievement loop"
        );
    }

    #[test]
    fn test_deep_facade_gateway_expedition_success_opens_gateway() {
        let mut deep = DeepState::default();
        deep.persistent.discovered = true;
        let merc = make_merc(1, 10_000, 50);
        deep.prestige.roster.insert(1, merc);
        deep.prestige.active_missions.push(make_mission(
            1,
            MissionType::GatewayExpedition,
            1,
            vec![1],
        ));

        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = tick_deep_facade(&mut deep, &mut achievements, "TestHero", false, &mut rng);

        assert!(result.deep_changed);
        assert!(
            deep.persistent.gateway_opened,
            "Successful GatewayExpedition should open the gateway, confirming the \
             gateway_opened summary field reached the facade's achievement branch"
        );
    }

    #[test]
    fn test_deep_facade_mercs_lost_reported_to_achievements() {
        let mut deep = DeepState::default();
        deep.persistent.discovered = true;

        // Underpowered, zero-resilience squad against a high-risk Breakthrough at
        // Layer 1: effective power ratio is far below threshold, so the mission
        // resolves as a Failure with a high per-merc loss chance. Bounded seed
        // search (matches the project's established pattern for RNG-dependent
        // branch coverage, e.g. enhancement_test.rs) finds a seed producing at
        // least one lost merc well within a small number of attempts.
        let mut found = false;
        for seed in 0..300u64 {
            let mut deep_trial = DeepState::default();
            deep_trial.persistent.discovered = true;
            deep_trial.prestige.roster.insert(1, make_merc(1, 1, 0));
            deep_trial.prestige.active_missions.push(make_mission(
                1,
                MissionType::Breakthrough,
                1,
                vec![1],
            ));

            let mut achievements = Achievements::default();
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = tick_deep_facade(
                &mut deep_trial,
                &mut achievements,
                "TestHero",
                false,
                &mut rng,
            );

            if matches!(
                deep_trial.prestige.roster.get(&1).map(|m| &m.status),
                Some(MercStatus::Lost)
            ) {
                assert!(result.deep_changed);
                found = true;
                break;
            }
        }
        assert!(
            found,
            "Could not find a seed producing a lost merc within 300 attempts"
        );
    }
}
