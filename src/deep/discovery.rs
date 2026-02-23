use super::mercenaries::generate_starter_roster;
use super::types::{deep_discovery_chance, DeepState};
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
    let starters =
        generate_starter_roster(deep.persistent.guild_rank, || deep.persistent.next_merc_id(), rng);
    deep.prestige.roster.extend(starters);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{
        DeepState, MercArchetype, MercStatus, Mercenary, DEEP_MIN_PRESTIGE_RANK,
    };
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
            assert!(!try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK - 1, &mut rng));
        }
        assert!(!deep.persistent.discovered);
    }

    #[test]
    fn test_try_discover_deep_blocked_when_already_discovered() {
        let mut rng = seeded_rng();
        let mut deep = DeepState::new();
        deep.persistent.discovered = true;

        for _ in 0..1_000 {
            assert!(!try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK, &mut rng));
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
            assert_eq!(merc.status, MercStatus::Available);
        }
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
