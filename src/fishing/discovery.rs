//! Fishing spot discovery logic.

use super::generation as fishing_generation;
use crate::core::constants::FISHING_DISCOVERY_CHANCE;
use crate::core::game_state::GameState;
use rand::{Rng, RngExt};

/// Roll for fishing spot discovery after killing an enemy.
///
/// Returns a discovery message if a spot is found.
///
/// # Conditions
/// - 5% chance per call
/// - Only if no active fishing session
/// - Only if not in a dungeon
pub fn try_discover_fishing(state: &mut GameState, rng: &mut impl Rng) -> Option<String> {
    // Check preconditions
    if state.active_fishing.is_some() {
        return None;
    }
    if state.active_dungeon.is_some() {
        return None;
    }

    // 5% discovery chance
    if rng.random::<f64>() >= FISHING_DISCOVERY_CHANCE {
        return None;
    }

    // Generate new fishing session
    let session = fishing_generation::generate_fishing_session(rng);
    let spot_name = session.spot_name.clone();

    state.active_fishing = Some(session);

    Some(format!("Discovered fishing spot: {}!", spot_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing::types::{FishingPhase, FishingSession};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn create_test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(12345)
    }

    fn create_test_game_state() -> GameState {
        GameState::new("Test Fisher".to_string(), 0)
    }

    #[test]
    fn test_try_discover_fishing_respects_conditions() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // With active fishing, should not discover
        state.active_fishing = Some(FishingSession {
            spot_name: "Existing".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 15,
            phase: FishingPhase::Waiting,
        });

        // Try many times - should never discover when already fishing
        for _ in 0..100 {
            let result = try_discover_fishing(&mut state, &mut rng);
            assert!(result.is_none(), "Should not discover when already fishing");
        }

        // Clear fishing session
        state.active_fishing = None;

        // With active dungeon, should not discover
        state.active_dungeon = Some(crate::dungeon::Dungeon::new(
            crate::dungeon::DungeonSize::Small,
        ));

        for _ in 0..100 {
            let result = try_discover_fishing(&mut state, &mut rng);
            assert!(result.is_none(), "Should not discover when in dungeon");
        }
    }

    #[test]
    fn test_try_discover_fishing_has_5_percent_chance() {
        let mut state = create_test_game_state();

        // Run many trials to verify approximately 5% discovery rate
        let trials = 10000;
        let mut discoveries = 0;

        for seed in 0..trials {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            // Reset state
            state.active_fishing = None;
            state.active_dungeon = None;

            if try_discover_fishing(&mut state, &mut rng).is_some() {
                discoveries += 1;
                // Clear for next trial
                state.active_fishing = None;
            }
        }

        let rate = discoveries as f64 / trials as f64;
        // Allow 1% tolerance (4-6% range)
        assert!(
            (0.04..=0.06).contains(&rate),
            "Discovery rate {} should be approximately 5%",
            rate
        );
    }
}
