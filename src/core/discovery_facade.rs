#![allow(dead_code)]

use rand::{Rng, RngExt};

use crate::core::constants::{DUNGEON_DISCOVERY_CHANCE, FISHING_DISCOVERY_CHANCE};

/// Explicit inputs for discovery rolls.
pub struct DiscoveryInput {
    pub prestige_rank: u32,
    pub character_level: u32,
    pub current_zone_id: u32,
    pub has_active_dungeon: bool,
    pub has_active_fishing: bool,
    pub has_active_minigame: bool,
}

/// Result of discovery rolls for a single tick.
#[derive(Debug, Default)]
pub struct DiscoveryResult {
    pub dungeon_discovered: bool,
    pub fishing_spot_discovered: bool,
}

/// Facade: roll for discoveries with explicit inputs.
///
/// Only returns whether discoveries happened. The caller is responsible
/// for creating the actual dungeon/fishing session.
pub fn roll_discoveries_facade<R: Rng>(input: &DiscoveryInput, rng: &mut R) -> DiscoveryResult {
    let mut result = DiscoveryResult::default();

    // Dungeon discovery: blocked by active dungeon
    if !input.has_active_dungeon {
        result.dungeon_discovered = rng.random::<f64>() < DUNGEON_DISCOVERY_CHANCE;
    }

    // Fishing discovery: blocked by active fishing AND active dungeon
    // (matches try_discover_fishing which checks both conditions)
    if !input.has_active_fishing && !input.has_active_dungeon {
        result.fishing_spot_discovered = rng.random::<f64>() < FISHING_DISCOVERY_CHANCE;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn default_input() -> DiscoveryInput {
        DiscoveryInput {
            prestige_rank: 0,
            character_level: 1,
            current_zone_id: 1,
            has_active_dungeon: false,
            has_active_fishing: false,
            has_active_minigame: false,
        }
    }

    #[test]
    fn active_dungeon_blocks_dungeon_discovery() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut input = default_input();
        input.has_active_dungeon = true;

        for _ in 0..1000 {
            let result = roll_discoveries_facade(&input, &mut rng);
            assert!(!result.dungeon_discovered);
        }
    }

    #[test]
    fn active_dungeon_blocks_fishing_discovery() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut input = default_input();
        input.has_active_dungeon = true;

        for _ in 0..1000 {
            let result = roll_discoveries_facade(&input, &mut rng);
            assert!(!result.fishing_spot_discovered);
        }
    }

    #[test]
    fn active_fishing_blocks_fishing_discovery() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut input = default_input();
        input.has_active_fishing = true;

        for _ in 0..1000 {
            let result = roll_discoveries_facade(&input, &mut rng);
            assert!(!result.fishing_spot_discovered);
        }
    }

    #[test]
    fn no_active_content_allows_discovery_rolls() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let input = default_input();

        // Should not panic; just exercise the code path
        let mut any_dungeon = false;
        let mut any_fishing = false;
        for _ in 0..1000 {
            let result = roll_discoveries_facade(&input, &mut rng);
            if result.dungeon_discovered {
                any_dungeon = true;
            }
            if result.fishing_spot_discovered {
                any_fishing = true;
            }
        }

        // With 1000 rolls at 1% and 5%, we should see at least one of each
        assert!(any_dungeon, "Expected at least one dungeon discovery in 1000 rolls at 1%");
        assert!(any_fishing, "Expected at least one fishing spot discovery in 1000 rolls at 5%");
    }
}
