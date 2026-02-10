//! Fish generation and fishing session creation.
//!
//! Handles rarity rolling, fish generation, and fishing session initialization.

#![allow(dead_code)]

use super::types::{CaughtFish, FishRarity, FishingPhase, FishingSession};
use crate::items::Item;
use rand::Rng;

/// Fishing spot names for session generation.
pub const SPOT_NAMES: [&str; 8] = [
    "Crystal Lake",
    "Misty Pond",
    "Rushing Creek",
    "Coral Shallows",
    "Abyssal Rift",
    "Moonlit Bay",
    "Serpent Cove",
    "Whispering Falls",
];

/// Common fish names.
pub const FISH_NAMES_COMMON: [&str; 5] = ["Minnow", "Carp", "Perch", "Bluegill", "Sunfish"];

/// Uncommon fish names.
pub const FISH_NAMES_UNCOMMON: [&str; 5] = ["Trout", "Bass", "Catfish", "Walleye", "Crappie"];

/// Rare fish names.
pub const FISH_NAMES_RARE: [&str; 5] = ["Salmon", "Pike", "Sturgeon", "Muskie", "Steelhead"];

/// Epic fish names.
pub const FISH_NAMES_EPIC: [&str; 5] = ["Marlin", "Swordfish", "Barracuda", "Tuna", "Mahi-mahi"];

/// Legendary fish names.
pub const FISH_NAMES_LEGENDARY: [&str; 5] = [
    "Kraken Spawn",
    "Sea Serpent",
    "Leviathan Fry",
    "Abyssal Eel",
    "Phantom Whale",
];

/// The legendary Storm Leviathan - only appears at max rank (40).
/// Catching this fish is required to forge the Stormbreaker.
pub const STORM_LEVIATHAN: &str = "Storm Leviathan";

/// XP reward ranges by rarity (min, max).
const XP_REWARDS: [(u32, u32); 5] = [
    (50, 100),    // Common
    (150, 250),   // Uncommon
    (400, 600),   // Rare
    (1000, 1500), // Epic
    (3000, 5000), // Legendary
];

/// Base rarity chances (percentages out of 100).
/// Common: 60%, Uncommon: 25%, Rare: 10%, Epic: 4%, Legendary: 1%
const BASE_CHANCES: [f64; 5] = [60.0, 25.0, 10.0, 4.0, 1.0];

/// Rank bonus per 5 ranks (percentages).
/// Every 5 ranks: -2% Common, +1% Uncommon, +0.5% Rare, +0.3% Epic, +0.2% Legendary
const RANK_BONUS_PER_5: [f64; 5] = [-2.0, 1.0, 0.5, 0.3, 0.2];

/// Rolls a fish rarity based on the player's fishing rank.
///
/// Base chances: Common 60%, Uncommon 25%, Rare 10%, Epic 4%, Legendary 1%
/// Every 5 ranks: -2% Common, +1% Uncommon, +0.5% Rare, +0.3% Epic, +0.2% Legendary
pub fn roll_fish_rarity(rank: u32, rng: &mut impl Rng) -> FishRarity {
    // Calculate how many bonus tiers we get (1 per 5 ranks: rank 5, 10, 15, ...)
    let bonus_tiers = rank / 5;

    // Calculate adjusted chances
    let mut chances = BASE_CHANCES;
    for (i, bonus) in RANK_BONUS_PER_5.iter().enumerate() {
        chances[i] += bonus * bonus_tiers as f64;
    }

    // Ensure Common doesn't go below a minimum (prevents negative)
    chances[0] = chances[0].max(10.0);

    // Roll a number from 0.0 to 100.0
    let roll: f64 = rng.gen_range(0.0..100.0);

    // Determine rarity based on cumulative chances (check rarest first)
    let mut cumulative = 100.0;

    // Legendary threshold
    cumulative -= chances[4];
    if roll >= cumulative {
        return FishRarity::Legendary;
    }

    // Epic threshold
    cumulative -= chances[3];
    if roll >= cumulative {
        return FishRarity::Epic;
    }

    // Rare threshold
    cumulative -= chances[2];
    if roll >= cumulative {
        return FishRarity::Rare;
    }

    // Uncommon threshold
    cumulative -= chances[1];
    if roll >= cumulative {
        return FishRarity::Uncommon;
    }

    // Default to Common
    FishRarity::Common
}

/// Returns the fish name array for a given rarity.
fn get_fish_names(rarity: FishRarity) -> &'static [&'static str; 5] {
    match rarity {
        FishRarity::Common => &FISH_NAMES_COMMON,
        FishRarity::Uncommon => &FISH_NAMES_UNCOMMON,
        FishRarity::Rare => &FISH_NAMES_RARE,
        FishRarity::Epic => &FISH_NAMES_EPIC,
        FishRarity::Legendary => &FISH_NAMES_LEGENDARY,
    }
}

/// Returns the XP reward range (min, max) for a given rarity.
fn get_xp_range(rarity: FishRarity) -> (u32, u32) {
    XP_REWARDS[rarity as usize]
}

/// Generates a fish of the specified rarity.
///
/// XP rewards by rarity:
/// - Common: 50-100
/// - Uncommon: 150-250
/// - Rare: 400-600
/// - Epic: 1000-1500
/// - Legendary: 3000-5000
pub fn generate_fish(rarity: FishRarity, rng: &mut impl Rng) -> CaughtFish {
    let names = get_fish_names(rarity);
    let name = names[rng.gen_range(0..names.len())].to_string();

    let (min_xp, max_xp) = get_xp_range(rarity);
    let xp_reward = rng.gen_range(min_xp..=max_xp);

    CaughtFish {
        name,
        rarity,
        xp_reward,
    }
}

/// XP reward for the Storm Leviathan (significantly higher than normal legendary)
const STORM_LEVIATHAN_XP: (u32, u32) = (10000, 15000);

/// Minimum fishing rank required to encounter the Storm Leviathan
const LEVIATHAN_MIN_RANK: u32 = 40;

/// Number of encounters required before the Leviathan can be caught
const LEVIATHAN_REQUIRED_ENCOUNTERS: u8 = 10;

/// Chance to catch the Leviathan after completing all encounters (25%)
const LEVIATHAN_CATCH_CHANCE: f64 = 0.25;

/// Marker value indicating the Storm Leviathan has been caught.
/// Used to prevent double-catching in the same tick (e.g., with double fish bonus).
pub const LEVIATHAN_CAUGHT_MARKER: u8 = 255;

/// Progressive encounter chances for the Storm Leviathan hunt.
/// The beast learns and becomes harder to find with each encounter.
/// Total of 10 encounters needed, taking roughly a month of casual play.
const LEVIATHAN_ENCOUNTER_CHANCES: [f64; 10] = [
    0.08,   // Encounter 1: 8%   - "Ripples"
    0.06,   // Encounter 2: 6%   - "The Shadow"
    0.05,   // Encounter 3: 5%   - "Emergence"
    0.04,   // Encounter 4: 4%   - "Known"
    0.03,   // Encounter 5: 3%   - "First Strike"
    0.02,   // Encounter 6: 2%   - "Fury"
    0.015,  // Encounter 7: 1.5% - "Blood in Water"
    0.01,   // Encounter 8: 1%   - "The Long Night"
    0.005,  // Encounter 9: 0.5% - "Exhaustion"
    0.0025, // Encounter 10: 0.25% - "Legend"
];

/// Result of a Storm Leviathan roll during fishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeviathanResult {
    /// No Leviathan appeared (normal fish)
    None,
    /// Leviathan appeared but escaped (encounter, not caught yet)
    Escaped { encounter_number: u8 },
    /// Leviathan was finally caught after all encounters
    Caught,
}

/// Generates a fish with rank awareness and Leviathan hunt progression.
///
/// At rank 40+, legendary fish have a progressive chance to trigger a Storm Leviathan
/// encounter. The beast must be encountered 10 times before it can be caught.
/// Each encounter has a decreasing chance, making the hunt take roughly a month.
pub fn generate_fish_with_rank(
    rarity: FishRarity,
    rank: u32,
    leviathan_encounters: u8,
    rng: &mut impl Rng,
) -> (CaughtFish, LeviathanResult) {
    // Early exit: Leviathan only appears for legendary fish at rank 40+
    if rarity != FishRarity::Legendary || rank < LEVIATHAN_MIN_RANK {
        return (generate_fish(rarity, rng), LeviathanResult::None);
    }

    // Already caught (marker set) - no more Leviathan encounters
    if leviathan_encounters == LEVIATHAN_CAUGHT_MARKER {
        return (generate_fish(rarity, rng), LeviathanResult::None);
    }

    // After 10 encounters, chance to catch
    if leviathan_encounters >= LEVIATHAN_REQUIRED_ENCOUNTERS {
        if rng.gen::<f64>() < LEVIATHAN_CATCH_CHANCE {
            let xp_reward = rng.gen_range(STORM_LEVIATHAN_XP.0..=STORM_LEVIATHAN_XP.1);
            return (
                CaughtFish {
                    name: STORM_LEVIATHAN.to_string(),
                    rarity: FishRarity::Legendary,
                    xp_reward,
                },
                LeviathanResult::Caught,
            );
        }
        return (generate_fish(rarity, rng), LeviathanResult::None);
    }

    // Progressive encounter roll
    let encounter_chance = LEVIATHAN_ENCOUNTER_CHANCES[leviathan_encounters as usize];
    if rng.gen::<f64>() < encounter_chance {
        let fish = generate_fish(rarity, rng);
        return (
            fish,
            LeviathanResult::Escaped {
                encounter_number: leviathan_encounters + 1,
            },
        );
    }

    (generate_fish(rarity, rng), LeviathanResult::None)
}

/// Checks if a caught fish is the Storm Leviathan.
pub fn is_storm_leviathan(fish: &CaughtFish) -> bool {
    fish.name == STORM_LEVIATHAN && fish.rarity == FishRarity::Legendary
}

/// Phase timing constants (at 100ms tick rate)
pub const CASTING_TICKS_MIN: u32 = 5; // 0.5s minimum cast
pub const CASTING_TICKS_MAX: u32 = 15; // 1.5s maximum cast
pub const WAITING_TICKS_MIN: u32 = 10; // 1.0s minimum wait (quick bite!)
pub const WAITING_TICKS_MAX: u32 = 80; // 8.0s maximum wait (patient fishing)
pub const REELING_TICKS_MIN: u32 = 5; // 0.5s minimum reel (easy catch)
pub const REELING_TICKS_MAX: u32 = 30; // 3.0s maximum reel (fighter!)

/// Generates a new fishing session with a random spot and fish count.
///
/// - Random spot name from SPOT_NAMES
/// - Random total_fish count: 3-8
/// - fish_caught and items_found start empty
/// - Starts in Casting phase
pub fn generate_fishing_session(rng: &mut impl Rng) -> FishingSession {
    let spot_name = SPOT_NAMES[rng.gen_range(0..SPOT_NAMES.len())].to_string();
    let total_fish = rng.gen_range(3..=8);

    FishingSession {
        spot_name,
        total_fish,
        fish_caught: Vec::new(),
        items_found: Vec::<Item>::new(),
        ticks_remaining: rng.gen_range(CASTING_TICKS_MIN..=CASTING_TICKS_MAX),
        phase: FishingPhase::Casting,
    }
}

/// Returns random ticks for the casting phase.
pub fn roll_casting_ticks(rng: &mut impl Rng) -> u32 {
    rng.gen_range(CASTING_TICKS_MIN..=CASTING_TICKS_MAX)
}

/// Returns random ticks for the waiting (bite) phase.
pub fn roll_waiting_ticks(rng: &mut impl Rng) -> u32 {
    rng.gen_range(WAITING_TICKS_MIN..=WAITING_TICKS_MAX)
}

/// Returns random ticks for the reeling phase.
pub fn roll_reeling_ticks(rng: &mut impl Rng) -> u32 {
    rng.gen_range(REELING_TICKS_MIN..=REELING_TICKS_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn create_test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(12345)
    }

    #[test]
    fn test_roll_fish_rarity_returns_valid_rarities() {
        let mut rng = create_test_rng();

        // Roll many times and ensure all results are valid
        for _ in 0..1000 {
            let rarity = roll_fish_rarity(1, &mut rng);
            assert!(matches!(
                rarity,
                FishRarity::Common
                    | FishRarity::Uncommon
                    | FishRarity::Rare
                    | FishRarity::Epic
                    | FishRarity::Legendary
            ));
        }
    }

    #[test]
    fn test_rank_bonus_affects_distribution() {
        let mut rng = create_test_rng();
        let iterations = 10000;

        // Test at rank 1 (no bonus)
        let mut rank1_counts = [0u32; 5];
        for _ in 0..iterations {
            let rarity = roll_fish_rarity(1, &mut rng);
            rank1_counts[rarity as usize] += 1;
        }

        // Test at rank 30 (max bonus: 6 tiers * bonuses)
        let mut rank30_counts = [0u32; 5];
        for _ in 0..iterations {
            let rarity = roll_fish_rarity(30, &mut rng);
            rank30_counts[rarity as usize] += 1;
        }

        // At rank 30, we should have:
        // - Fewer common fish than rank 1
        // - More uncommon/rare/epic/legendary than rank 1

        // Common should be lower at rank 30
        assert!(
            rank30_counts[0] < rank1_counts[0],
            "Common fish should be less frequent at rank 30 (rank30: {}, rank1: {})",
            rank30_counts[0],
            rank1_counts[0]
        );

        // Sum of non-common should be higher at rank 30
        let rank1_non_common: u32 = rank1_counts[1..].iter().sum();
        let rank30_non_common: u32 = rank30_counts[1..].iter().sum();
        assert!(
            rank30_non_common > rank1_non_common,
            "Non-common fish should be more frequent at rank 30"
        );
    }

    #[test]
    fn test_generate_fish_returns_correct_xp_range_common() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let fish = generate_fish(FishRarity::Common, &mut rng);
            assert_eq!(fish.rarity, FishRarity::Common);
            assert!(
                fish.xp_reward >= 50 && fish.xp_reward <= 100,
                "Common fish XP {} should be between 50-100",
                fish.xp_reward
            );
        }
    }

    #[test]
    fn test_generate_fish_returns_correct_xp_range_uncommon() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let fish = generate_fish(FishRarity::Uncommon, &mut rng);
            assert_eq!(fish.rarity, FishRarity::Uncommon);
            assert!(
                fish.xp_reward >= 150 && fish.xp_reward <= 250,
                "Uncommon fish XP {} should be between 150-250",
                fish.xp_reward
            );
        }
    }

    #[test]
    fn test_generate_fish_returns_correct_xp_range_rare() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let fish = generate_fish(FishRarity::Rare, &mut rng);
            assert_eq!(fish.rarity, FishRarity::Rare);
            assert!(
                fish.xp_reward >= 400 && fish.xp_reward <= 600,
                "Rare fish XP {} should be between 400-600",
                fish.xp_reward
            );
        }
    }

    #[test]
    fn test_generate_fish_returns_correct_xp_range_epic() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let fish = generate_fish(FishRarity::Epic, &mut rng);
            assert_eq!(fish.rarity, FishRarity::Epic);
            assert!(
                fish.xp_reward >= 1000 && fish.xp_reward <= 1500,
                "Epic fish XP {} should be between 1000-1500",
                fish.xp_reward
            );
        }
    }

    #[test]
    fn test_generate_fish_returns_correct_xp_range_legendary() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let fish = generate_fish(FishRarity::Legendary, &mut rng);
            assert_eq!(fish.rarity, FishRarity::Legendary);
            assert!(
                fish.xp_reward >= 3000 && fish.xp_reward <= 5000,
                "Legendary fish XP {} should be between 3000-5000",
                fish.xp_reward
            );
        }
    }

    #[test]
    fn test_generate_fish_uses_correct_names() {
        let mut rng = create_test_rng();

        // Test each rarity
        for _ in 0..20 {
            let fish = generate_fish(FishRarity::Common, &mut rng);
            assert!(FISH_NAMES_COMMON.contains(&fish.name.as_str()));
        }

        for _ in 0..20 {
            let fish = generate_fish(FishRarity::Uncommon, &mut rng);
            assert!(FISH_NAMES_UNCOMMON.contains(&fish.name.as_str()));
        }

        for _ in 0..20 {
            let fish = generate_fish(FishRarity::Rare, &mut rng);
            assert!(FISH_NAMES_RARE.contains(&fish.name.as_str()));
        }

        for _ in 0..20 {
            let fish = generate_fish(FishRarity::Epic, &mut rng);
            assert!(FISH_NAMES_EPIC.contains(&fish.name.as_str()));
        }

        for _ in 0..20 {
            let fish = generate_fish(FishRarity::Legendary, &mut rng);
            assert!(FISH_NAMES_LEGENDARY.contains(&fish.name.as_str()));
        }
    }

    #[test]
    fn test_generate_fishing_session_returns_valid_fish_count() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let session = generate_fishing_session(&mut rng);
            assert!(
                session.total_fish >= 3 && session.total_fish <= 8,
                "Fish count {} should be between 3-8",
                session.total_fish
            );
        }
    }

    #[test]
    fn test_generate_fishing_session_uses_valid_spot_names() {
        let mut rng = create_test_rng();

        for _ in 0..100 {
            let session = generate_fishing_session(&mut rng);
            assert!(
                SPOT_NAMES.contains(&session.spot_name.as_str()),
                "Spot name '{}' should be in SPOT_NAMES",
                session.spot_name
            );
        }
    }

    #[test]
    fn test_generate_fishing_session_starts_empty() {
        let mut rng = create_test_rng();

        let session = generate_fishing_session(&mut rng);
        assert!(
            session.fish_caught.is_empty(),
            "Session should start with empty fish_caught"
        );
        assert!(
            session.items_found.is_empty(),
            "Session should start with empty items_found"
        );
    }

    #[test]
    fn test_spot_names_constant_has_correct_count() {
        assert_eq!(
            SPOT_NAMES.len(),
            8,
            "SPOT_NAMES should have exactly 8 spots"
        );
    }

    #[test]
    fn test_fish_names_constants_have_correct_counts() {
        assert_eq!(FISH_NAMES_COMMON.len(), 5);
        assert_eq!(FISH_NAMES_UNCOMMON.len(), 5);
        assert_eq!(FISH_NAMES_RARE.len(), 5);
        assert_eq!(FISH_NAMES_EPIC.len(), 5);
        assert_eq!(FISH_NAMES_LEGENDARY.len(), 5);
    }

    // ==================== Storm Leviathan Tests ====================

    #[test]
    fn test_leviathan_encounter_chances_decreasing() {
        // Each encounter chance should be less than or equal to the previous
        for i in 1..LEVIATHAN_ENCOUNTER_CHANCES.len() {
            assert!(
                LEVIATHAN_ENCOUNTER_CHANCES[i] <= LEVIATHAN_ENCOUNTER_CHANCES[i - 1],
                "Encounter {} chance ({}) should be <= encounter {} chance ({})",
                i + 1,
                LEVIATHAN_ENCOUNTER_CHANCES[i],
                i,
                LEVIATHAN_ENCOUNTER_CHANCES[i - 1]
            );
        }
    }

    #[test]
    fn test_leviathan_encounter_chances_valid_range() {
        for (i, chance) in LEVIATHAN_ENCOUNTER_CHANCES.iter().enumerate() {
            assert!(
                *chance > 0.0 && *chance <= 1.0,
                "Encounter {} chance {} should be between 0 and 1",
                i + 1,
                chance
            );
        }
    }

    #[test]
    fn test_generate_fish_with_rank_non_legendary_returns_none() {
        let mut rng = create_test_rng();

        // Non-legendary fish should never trigger Leviathan encounters
        for rarity in [
            FishRarity::Common,
            FishRarity::Uncommon,
            FishRarity::Rare,
            FishRarity::Epic,
        ] {
            for encounters in 0..=10 {
                let (fish, result) = generate_fish_with_rank(rarity, 40, encounters, &mut rng);
                assert_eq!(result, LeviathanResult::None);
                assert_eq!(fish.rarity, rarity);
            }
        }
    }

    #[test]
    fn test_generate_fish_with_rank_low_rank_returns_none() {
        let mut rng = create_test_rng();

        // Ranks below 40 should never trigger Leviathan encounters
        for rank in [1, 10, 20, 30, 39] {
            for _ in 0..100 {
                let (fish, result) =
                    generate_fish_with_rank(FishRarity::Legendary, rank, 0, &mut rng);
                assert_eq!(
                    result,
                    LeviathanResult::None,
                    "Rank {} should not trigger Leviathan",
                    rank
                );
                assert_eq!(fish.rarity, FishRarity::Legendary);
            }
        }
    }

    #[test]
    fn test_generate_fish_with_rank_encounter_increments() {
        // Use a seeded RNG that will produce an encounter
        // We'll run multiple attempts to find one that triggers
        let mut rng = create_test_rng();

        // At rank 40, legendary fish with 0 encounters has 8% chance
        // Run enough times to get some encounters
        let mut encountered = false;
        for _ in 0..1000 {
            let (_, result) = generate_fish_with_rank(FishRarity::Legendary, 40, 0, &mut rng);
            if let LeviathanResult::Escaped { encounter_number } = result {
                assert_eq!(encounter_number, 1, "First encounter should be number 1");
                encountered = true;
                break;
            }
        }
        assert!(
            encountered,
            "Should have encountered Leviathan at least once in 1000 tries at 8% chance"
        );
    }

    #[test]
    fn test_generate_fish_with_rank_catch_after_10_encounters() {
        let mut rng = create_test_rng();

        // With 10 encounters complete, there's a 25% chance to catch
        let mut caught = false;
        for _ in 0..1000 {
            let (fish, result) = generate_fish_with_rank(FishRarity::Legendary, 40, 10, &mut rng);
            if result == LeviathanResult::Caught {
                assert_eq!(fish.name, STORM_LEVIATHAN);
                assert_eq!(fish.rarity, FishRarity::Legendary);
                assert!(
                    fish.xp_reward >= STORM_LEVIATHAN_XP.0
                        && fish.xp_reward <= STORM_LEVIATHAN_XP.1,
                    "Storm Leviathan XP {} should be in range {:?}",
                    fish.xp_reward,
                    STORM_LEVIATHAN_XP
                );
                caught = true;
                break;
            }
        }
        assert!(
            caught,
            "Should have caught Leviathan at least once in 1000 tries at 25% chance"
        );
    }

    #[test]
    fn test_is_storm_leviathan() {
        let leviathan = CaughtFish {
            name: STORM_LEVIATHAN.to_string(),
            rarity: FishRarity::Legendary,
            xp_reward: 12000,
        };
        assert!(is_storm_leviathan(&leviathan));

        // Wrong name
        let fake = CaughtFish {
            name: "Ancient Kraken".to_string(),
            rarity: FishRarity::Legendary,
            xp_reward: 12000,
        };
        assert!(!is_storm_leviathan(&fake));

        // Wrong rarity (shouldn't happen but test anyway)
        let wrong_rarity = CaughtFish {
            name: STORM_LEVIATHAN.to_string(),
            rarity: FishRarity::Epic,
            xp_reward: 12000,
        };
        assert!(!is_storm_leviathan(&wrong_rarity));
    }

    #[test]
    fn test_leviathan_result_equality() {
        assert_eq!(LeviathanResult::None, LeviathanResult::None);
        assert_eq!(LeviathanResult::Caught, LeviathanResult::Caught);
        assert_eq!(
            LeviathanResult::Escaped {
                encounter_number: 5
            },
            LeviathanResult::Escaped {
                encounter_number: 5
            }
        );
        assert_ne!(
            LeviathanResult::Escaped {
                encounter_number: 5
            },
            LeviathanResult::Escaped {
                encounter_number: 6
            }
        );
        assert_ne!(LeviathanResult::None, LeviathanResult::Caught);
    }

    #[test]
    fn test_storm_leviathan_xp_range() {
        // Verify the XP constant is reasonable
        assert!(
            STORM_LEVIATHAN_XP.0 >= 10000,
            "Minimum Leviathan XP should be at least 10000"
        );
        assert!(
            STORM_LEVIATHAN_XP.1 >= STORM_LEVIATHAN_XP.0,
            "Max XP should be >= min XP"
        );
    }

    #[test]
    fn test_leviathan_caught_marker_prevents_double_catch() {
        // Bug fix test: After catching the Storm Leviathan, the caught marker
        // should prevent any further encounters or catches (e.g., from double fish bonus).
        let mut rng = create_test_rng();

        // When encounters is set to the caught marker, no Leviathan events should occur
        for _ in 0..1000 {
            let (fish, result) = generate_fish_with_rank(
                FishRarity::Legendary,
                40, // Max rank
                LEVIATHAN_CAUGHT_MARKER,
                &mut rng,
            );

            assert_eq!(
                result,
                LeviathanResult::None,
                "Caught marker should prevent all Leviathan encounters"
            );
            // Should get a regular legendary fish, not the Storm Leviathan
            assert_ne!(
                fish.name, STORM_LEVIATHAN,
                "Should not catch Storm Leviathan when marker is set"
            );
        }
    }

    #[test]
    fn test_leviathan_caught_marker_value() {
        // Verify the caught marker is distinct from valid encounter counts
        assert_eq!(LEVIATHAN_CAUGHT_MARKER, 255);
        assert!(LEVIATHAN_CAUGHT_MARKER > LEVIATHAN_REQUIRED_ENCOUNTERS);
    }
}
