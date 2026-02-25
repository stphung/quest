use super::constants::*;
use super::game_state::GameState;
use crate::deep::DeepState;
use rand::{Rng, RngExt};

/// Attempts to discover a dungeon after killing an enemy
/// Returns true if a dungeon was discovered and entered
pub fn try_discover_dungeon<R: Rng>(rng: &mut R, state: &mut GameState) -> bool {
    // Don't discover if already in a dungeon
    if state.active_dungeon.is_some() {
        return false;
    }

    if rng.random::<f64>() >= DUNGEON_DISCOVERY_CHANCE {
        return false;
    }

    // Discover dungeon!
    // Prestige affects dungeon quality (size, rewards), not discovery rate
    let zone_id = state.zone_progression.current_zone_id;
    let dungeon = crate::dungeon::generation::generate_dungeon(
        state.character_level,
        state.prestige_rank,
        zone_id,
    );
    state.active_dungeon = Some(dungeon);

    true
}

/// Attempts to discover The Deep.
///
/// Thin wrapper over [`crate::deep::try_discover_deep`] so callers can import
/// all discovery rolls from a single module.  Returns `true` if The Deep was
/// discovered this tick.
#[allow(dead_code)]
pub fn try_discover_deep<R: Rng>(deep: &mut DeepState, prestige_rank: u32, rng: &mut R) -> bool {
    crate::deep::try_discover_deep(deep, prestige_rank, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn seeded_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_try_discover_dungeon_skips_when_in_dungeon() {
        let mut rng = seeded_rng();
        let mut state = GameState::new("Test Hero".to_string(), 0);

        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0, 1));

        for _ in 0..100 {
            assert!(!try_discover_dungeon(&mut rng, &mut state));
        }
    }

    #[test]
    fn test_try_discover_dungeon_probability() {
        let mut rng = seeded_rng();
        let mut discoveries = 0;
        let trials = 3_000;

        for _ in 0..trials {
            let mut state = GameState::new("Test Hero".to_string(), 0);
            if try_discover_dungeon(&mut rng, &mut state) {
                discoveries += 1;
            }
        }

        assert!(
            (6..=75).contains(&discoveries),
            "Expected ~30 discoveries (1%), got {}",
            discoveries
        );
    }

    #[test]
    fn test_try_discover_dungeon_creates_valid_dungeon() {
        let mut rng = seeded_rng();
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.character_level = 10;
        state.prestige_rank = 1;

        let mut discovered = false;
        for _ in 0..1000 {
            if try_discover_dungeon(&mut rng, &mut state) {
                discovered = true;
                break;
            }
            state.active_dungeon = None;
        }

        if discovered {
            let dungeon = state.active_dungeon.as_ref().unwrap();
            assert!(!dungeon.grid.is_empty());
            assert_eq!(dungeon.player_position, dungeon.entrance_position);
        }
    }

    #[test]
    fn test_discovery_blocked_during_active_dungeon() {
        let mut rng = seeded_rng();
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0, 1));

        for _ in 0..100 {
            assert!(!try_discover_dungeon(&mut rng, &mut state));
        }
    }
}
