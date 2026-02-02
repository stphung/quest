//! Fish generation and fishing session creation.
//!
//! Handles rarity rolling, fish generation, and fishing session initialization.

#![allow(dead_code)]

use crate::fishing::{CaughtFish, FishRarity, FishingPhase, FishingSession};
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
    // Calculate how many bonus tiers we get (1 per 5 ranks, starting from rank 5)
    let bonus_tiers = rank.saturating_sub(1) / 5;

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

/// Phase timing constants (at 100ms tick rate)
pub const CASTING_TICKS_MIN: u32 = 5;     // 0.5s minimum cast
pub const CASTING_TICKS_MAX: u32 = 15;    // 1.5s maximum cast
pub const WAITING_TICKS_MIN: u32 = 10;    // 1.0s minimum wait (quick bite!)
pub const WAITING_TICKS_MAX: u32 = 80;    // 8.0s maximum wait (patient fishing)
pub const REELING_TICKS_MIN: u32 = 5;     // 0.5s minimum reel (easy catch)
pub const REELING_TICKS_MAX: u32 = 30;    // 3.0s maximum reel (fighter!)

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
}
