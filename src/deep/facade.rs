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
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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
}
