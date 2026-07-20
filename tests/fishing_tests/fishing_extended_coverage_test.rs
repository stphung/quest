//! Coverage gap tests for the fishing module.
//!
//! This file targets areas not covered by fishing_core_coverage_test.rs:
//!
//! - fishing/drops.rs constants: verifying the exact drop chance values and
//!   their monotonic ordering (Common <= Uncommon < Rare < Epic < Legendary).
//! - fishing/rank.rs edge cases: tier-boundary thresholds (Grandmaster 2000
//!   fish), exact-threshold carry-over to zero, and the rank 29->30 boundary
//!   which is the last step before the base cap.
//! - fishing/generation.rs: roll_fish_rarity rank scaling (higher rank
//!   produces proportionally fewer Common fish), generate_fish XP bounds, and
//!   spot name / session size invariants.
//! - FishingState::fish_required_for_rank: Mythic/Transcendent tier values
//!   and the beyond-max fallback.
//! - fishing/discovery.rs: probability that a fresh state can discover a spot
//!   within a reasonable number of calls (~5% per call).

#![allow(clippy::field_reassign_with_default)]

use quest::core::constants::{
    FISHING_DROP_CHANCE_COMMON, FISHING_DROP_CHANCE_EPIC, FISHING_DROP_CHANCE_LEGENDARY,
    FISHING_DROP_CHANCE_RARE, FISHING_DROP_CHANCE_UNCOMMON,
};
use quest::fishing::{
    check_rank_up_with_max, generate_fish, generate_fishing_session, roll_fish_rarity,
    try_discover_fishing, FishRarity, FishingPhase, FishingState, SPOT_NAMES,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =============================================================================
// Helpers
// =============================================================================

fn rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

use super::helpers::fresh_state;

fn fresh_fishing_state(rank: u32, fish_toward: u32) -> FishingState {
    FishingState {
        rank,
        total_fish_caught: 0,
        fish_toward_next_rank: fish_toward,
        legendary_catches: 0,
        leviathan_encounters: 0,
        ..Default::default()
    }
}

// =============================================================================
// fishing/drops.rs — constant values match documented design
// =============================================================================

#[test]
fn test_fishing_drop_chance_common_is_five_percent() {
    assert!(
        (FISHING_DROP_CHANCE_COMMON - 0.05).abs() < f64::EPSILON,
        "Common fish drop chance should be exactly 0.05 (5%), got {}",
        FISHING_DROP_CHANCE_COMMON
    );
}

#[test]
fn test_fishing_drop_chance_uncommon_is_five_percent() {
    assert!(
        (FISHING_DROP_CHANCE_UNCOMMON - 0.05).abs() < f64::EPSILON,
        "Uncommon fish drop chance should be exactly 0.05 (5%), got {}",
        FISHING_DROP_CHANCE_UNCOMMON
    );
}

#[test]
fn test_fishing_drop_chance_rare_is_fifteen_percent() {
    assert!(
        (FISHING_DROP_CHANCE_RARE - 0.15).abs() < f64::EPSILON,
        "Rare fish drop chance should be exactly 0.15 (15%), got {}",
        FISHING_DROP_CHANCE_RARE
    );
}

#[test]
fn test_fishing_drop_chance_epic_is_thirty_five_percent() {
    assert!(
        (FISHING_DROP_CHANCE_EPIC - 0.35).abs() < f64::EPSILON,
        "Epic fish drop chance should be exactly 0.35 (35%), got {}",
        FISHING_DROP_CHANCE_EPIC
    );
}

#[test]
fn test_fishing_drop_chance_legendary_is_seventy_five_percent() {
    assert!(
        (FISHING_DROP_CHANCE_LEGENDARY - 0.75).abs() < f64::EPSILON,
        "Legendary fish drop chance should be exactly 0.75 (75%), got {}",
        FISHING_DROP_CHANCE_LEGENDARY
    );
}

// =============================================================================
// fishing/drops.rs — drop chances are monotonically non-decreasing by rarity
// =============================================================================

#[test]
fn test_fishing_drop_chances_monotonically_non_decreasing_by_rarity() {
    // Common and Uncommon are both 5% — that is the one exception where two
    // adjacent rarities share the same rate (both sit at the "rare enough to
    // warrant a chance but not worth tuning separately" threshold).
    let chances = [
        ("Common", FISHING_DROP_CHANCE_COMMON),
        ("Uncommon", FISHING_DROP_CHANCE_UNCOMMON),
        ("Rare", FISHING_DROP_CHANCE_RARE),
        ("Epic", FISHING_DROP_CHANCE_EPIC),
        ("Legendary", FISHING_DROP_CHANCE_LEGENDARY),
    ];
    // Common and Uncommon may be equal (both 5%), rest must be strictly increasing
    assert!(
        chances[0].1 <= chances[1].1,
        "{} ({}) should be <= {} ({})",
        chances[0].0,
        chances[0].1,
        chances[1].0,
        chances[1].1
    );
    for window in chances.windows(2).skip(1) {
        assert!(
            window[0].1 < window[1].1,
            "{} ({}) should be strictly less than {} ({})",
            window[0].0,
            window[0].1,
            window[1].0,
            window[1].1
        );
    }
}

#[test]
fn test_fishing_drop_chances_are_valid_probabilities() {
    for (name, chance) in [
        ("Common", FISHING_DROP_CHANCE_COMMON),
        ("Uncommon", FISHING_DROP_CHANCE_UNCOMMON),
        ("Rare", FISHING_DROP_CHANCE_RARE),
        ("Epic", FISHING_DROP_CHANCE_EPIC),
        ("Legendary", FISHING_DROP_CHANCE_LEGENDARY),
    ] {
        assert!(
            (0.0..=1.0).contains(&chance),
            "{} drop chance {} is not in [0, 1]",
            name,
            chance
        );
    }
}

// =============================================================================
// fishing/rank.rs — check_rank_up_with_max: Grandmaster tier (rank 29->30)
// =============================================================================

#[test]
fn test_check_rank_up_grandmaster_tier_boundary_29_to_30() {
    // Rank 29 is in the Grandmaster tier, requires 2000 fish
    let mut state = fresh_fishing_state(29, 2000);
    let result = check_rank_up_with_max(&mut state, 30);
    assert!(
        result.is_some(),
        "Rank 29->30 should succeed with exactly 2000 fish (Grandmaster tier)"
    );
    assert_eq!(state.rank, 30);
    assert_eq!(
        state.fish_toward_next_rank, 0,
        "Exact threshold should leave zero excess"
    );
}

#[test]
fn test_check_rank_up_grandmaster_insufficient_does_not_rank_up() {
    // 1999 is one short of the 2000 needed for Grandmaster tier
    let mut state = fresh_fishing_state(29, 1999);
    let result = check_rank_up_with_max(&mut state, 30);
    assert!(
        result.is_none(),
        "1999/2000 fish should not trigger rank-up"
    );
    assert_eq!(state.rank, 29, "Rank should remain 29");
    assert_eq!(
        state.fish_toward_next_rank, 1999,
        "Fish count should be unchanged"
    );
}

// =============================================================================
// fishing/rank.rs — fish_required_for_rank: Mythic tier (31-35)
// =============================================================================

#[test]
fn test_fish_required_for_mythic_tier() {
    assert_eq!(FishingState::fish_required_for_rank(31), 4_000);
    assert_eq!(FishingState::fish_required_for_rank(32), 6_000);
    assert_eq!(FishingState::fish_required_for_rank(33), 10_000);
    assert_eq!(FishingState::fish_required_for_rank(34), 16_000);
    assert_eq!(FishingState::fish_required_for_rank(35), 25_000);
}

// =============================================================================
// fishing/rank.rs — fish_required_for_rank: Transcendent tier (36-40)
// =============================================================================

#[test]
fn test_fish_required_for_transcendent_tier() {
    assert_eq!(FishingState::fish_required_for_rank(36), 40_000);
    assert_eq!(FishingState::fish_required_for_rank(37), 65_000);
    assert_eq!(FishingState::fish_required_for_rank(38), 100_000);
    assert_eq!(FishingState::fish_required_for_rank(39), 160_000);
    assert_eq!(FishingState::fish_required_for_rank(40), 250_000);
}

#[test]
fn test_fish_required_beyond_max_rank_uses_fallback() {
    // Ranks beyond 40 return the same value as rank 40 (250_000)
    assert_eq!(FishingState::fish_required_for_rank(41), 250_000);
    assert_eq!(FishingState::fish_required_for_rank(99), 250_000);
}

// =============================================================================
// fishing/generation.rs — roll_fish_rarity: rank 1 is predominantly Common
// =============================================================================

#[test]
fn test_roll_fish_rarity_rank_1_mostly_common() {
    let mut r = rng(1001);
    let trials = 1000;
    let mut common_count = 0u32;

    for _ in 0..trials {
        if roll_fish_rarity(1, &mut r) == FishRarity::Common {
            common_count += 1;
        }
    }

    // Base Common chance is 60%; expect well above 50% at rank 1
    let common_rate = common_count as f64 / trials as f64;
    assert!(
        common_rate > 0.50,
        "At rank 1 Common fish should dominate (>50%), got {:.2}%",
        common_rate * 100.0
    );
}

// =============================================================================
// fishing/generation.rs — roll_fish_rarity: high rank increases non-Common fish
// =============================================================================

#[test]
fn test_roll_fish_rarity_rank_30_has_fewer_common_than_rank_1() {
    let trials = 2000;

    let mut r1 = rng(2001);
    let common_rank1 = (0..trials)
        .filter(|_| roll_fish_rarity(1, &mut r1) == FishRarity::Common)
        .count();

    let mut r30 = rng(2001); // same seed to isolate the rank effect
    let common_rank30 = (0..trials)
        .filter(|_| roll_fish_rarity(30, &mut r30) == FishRarity::Common)
        .count();

    assert!(
        common_rank30 < common_rank1,
        "Rank 30 should produce fewer Common fish than rank 1 \
         (rank1={}, rank30={})",
        common_rank1,
        common_rank30
    );
}

#[test]
fn test_roll_fish_rarity_rank_40_has_fewer_common_than_rank_20() {
    let trials = 2000;

    let mut r20 = rng(3001);
    let common_rank20 = (0..trials)
        .filter(|_| roll_fish_rarity(20, &mut r20) == FishRarity::Common)
        .count();

    let mut r40 = rng(3001);
    let common_rank40 = (0..trials)
        .filter(|_| roll_fish_rarity(40, &mut r40) == FishRarity::Common)
        .count();

    assert!(
        common_rank40 < common_rank20,
        "Rank 40 should produce fewer Common fish than rank 20 \
         (rank20={}, rank40={})",
        common_rank20,
        common_rank40
    );
}

// =============================================================================
// fishing/generation.rs — roll_fish_rarity: every rarity is reachable at rank 1
// =============================================================================

#[test]
fn test_roll_fish_rarity_all_rarities_reachable_at_rank_1() {
    let mut r = rng(4001);
    let mut seen = [false; 5];

    for _ in 0..50_000 {
        let rarity = roll_fish_rarity(1, &mut r);
        seen[rarity as usize] = true;
        if seen.iter().all(|&s| s) {
            break;
        }
    }

    assert!(seen[0], "Common should be reachable at rank 1");
    assert!(seen[1], "Uncommon should be reachable at rank 1");
    assert!(seen[2], "Rare should be reachable at rank 1");
    assert!(seen[3], "Epic should be reachable at rank 1");
    assert!(seen[4], "Legendary should be reachable at rank 1");
}

// =============================================================================
// fishing/generation.rs — generate_fish: XP bounds by rarity
// =============================================================================

#[test]
fn test_generate_fish_common_xp_within_documented_range() {
    let mut r = rng(5001);
    for _ in 0..100 {
        let fish = generate_fish(FishRarity::Common, &mut r);
        assert!(
            (50..=100).contains(&fish.xp_reward),
            "Common fish XP should be 50-100, got {}",
            fish.xp_reward
        );
        assert_eq!(fish.rarity, FishRarity::Common);
    }
}

#[test]
fn test_generate_fish_uncommon_xp_within_documented_range() {
    let mut r = rng(5002);
    for _ in 0..100 {
        let fish = generate_fish(FishRarity::Uncommon, &mut r);
        assert!(
            (150..=250).contains(&fish.xp_reward),
            "Uncommon fish XP should be 150-250, got {}",
            fish.xp_reward
        );
        assert_eq!(fish.rarity, FishRarity::Uncommon);
    }
}

#[test]
fn test_generate_fish_rare_xp_within_documented_range() {
    let mut r = rng(5003);
    for _ in 0..100 {
        let fish = generate_fish(FishRarity::Rare, &mut r);
        assert!(
            (400..=600).contains(&fish.xp_reward),
            "Rare fish XP should be 400-600, got {}",
            fish.xp_reward
        );
        assert_eq!(fish.rarity, FishRarity::Rare);
    }
}

#[test]
fn test_generate_fish_epic_xp_within_documented_range() {
    let mut r = rng(5004);
    for _ in 0..100 {
        let fish = generate_fish(FishRarity::Epic, &mut r);
        assert!(
            (1000..=1500).contains(&fish.xp_reward),
            "Epic fish XP should be 1000-1500, got {}",
            fish.xp_reward
        );
        assert_eq!(fish.rarity, FishRarity::Epic);
    }
}

#[test]
fn test_generate_fish_legendary_xp_within_documented_range() {
    let mut r = rng(5005);
    for _ in 0..100 {
        let fish = generate_fish(FishRarity::Legendary, &mut r);
        assert!(
            (3000..=5000).contains(&fish.xp_reward),
            "Legendary fish XP should be 3000-5000, got {}",
            fish.xp_reward
        );
        assert_eq!(fish.rarity, FishRarity::Legendary);
    }
}

#[test]
fn test_generate_fish_name_is_non_empty() {
    let mut r = rng(5006);
    for rarity in [
        FishRarity::Common,
        FishRarity::Uncommon,
        FishRarity::Rare,
        FishRarity::Epic,
        FishRarity::Legendary,
    ] {
        let fish = generate_fish(rarity, &mut r);
        assert!(
            !fish.name.is_empty(),
            "Generated fish name should not be empty for {:?}",
            rarity
        );
    }
}

// =============================================================================
// fishing/generation.rs — generate_fishing_session: invariants on the session
// =============================================================================

#[test]
fn test_generate_fishing_session_total_fish_in_range() {
    let mut r = rng(6001);
    for _ in 0..200 {
        let session = generate_fishing_session(&mut r);
        assert!(
            (3..=8).contains(&session.total_fish),
            "Session total_fish should be 3-8, got {}",
            session.total_fish
        );
    }
}

#[test]
fn test_generate_fishing_session_starts_in_casting_phase() {
    let mut r = rng(6002);
    for _ in 0..50 {
        let session = generate_fishing_session(&mut r);
        assert_eq!(
            session.phase,
            FishingPhase::Casting,
            "New session should always start in Casting phase"
        );
    }
}

#[test]
fn test_generate_fishing_session_spot_name_is_from_known_list() {
    let mut r = rng(6003);
    for _ in 0..200 {
        let session = generate_fishing_session(&mut r);
        assert!(
            SPOT_NAMES.contains(&session.spot_name.as_str()),
            "Session spot '{}' should be one of the documented spot names",
            session.spot_name
        );
    }
}

#[test]
fn test_generate_fishing_session_starts_with_no_fish_caught() {
    let mut r = rng(6004);
    let session = generate_fishing_session(&mut r);
    assert!(
        session.fish_caught.is_empty(),
        "New session should have no fish caught yet"
    );
    assert!(
        session.items_found.is_empty(),
        "New session should have no items found yet"
    );
}

#[test]
fn test_generate_fishing_session_ticks_remaining_is_positive() {
    let mut r = rng(6005);
    for _ in 0..50 {
        let session = generate_fishing_session(&mut r);
        assert!(
            session.ticks_remaining > 0,
            "New session should have positive ticks_remaining, got {}",
            session.ticks_remaining
        );
    }
}

// =============================================================================
// fishing/discovery.rs — discovery rate is approximately 5% per call
// =============================================================================

#[test]
fn test_discover_fishing_probability_approximately_five_percent() {
    let trials = 500u32;
    let mut discoveries = 0u32;

    for seed in 0..trials as u64 {
        let mut state = fresh_state();
        // Ensure no blocking conditions
        state.active_fishing = None;
        state.active_dungeon = None;

        let mut r = rng(seed + 70000);
        if try_discover_fishing(&mut state, &mut r).is_some() {
            discoveries += 1;
        }
    }

    let rate = discoveries as f64 / trials as f64;
    // Documented as 5% per kill; allow wide tolerance for statistical variance
    assert!(
        (0.02..=0.09).contains(&rate),
        "Fishing discovery rate {:.4} should be approximately 5% (0.02-0.09 acceptable range)",
        rate
    );
}

// =============================================================================
// fishing/rank.rs — check_rank_up_with_max: multiple sequential rank-ups
// =============================================================================

#[test]
fn test_sequential_rank_ups_correctly_track_state() {
    // Rank 1 -> 2 -> 3 in sequence, each with exact fish
    let mut state = FishingState {
        rank: 1,
        total_fish_caught: 0,
        fish_toward_next_rank: 100, // Enough for rank 1->2
        legendary_catches: 0,
        leviathan_encounters: 0,
        ..Default::default()
    };

    // First rank-up: 1 -> 2
    let msg1 = check_rank_up_with_max(&mut state, 40);
    assert!(msg1.is_some(), "First rank-up should succeed");
    assert_eq!(state.rank, 2);
    assert_eq!(state.fish_toward_next_rank, 0);

    // Add fish for next rank-up (rank 2 is still Novice tier, needs 100)
    state.fish_toward_next_rank = 100;
    let msg2 = check_rank_up_with_max(&mut state, 40);
    assert!(msg2.is_some(), "Second rank-up should succeed");
    assert_eq!(state.rank, 3);
    assert_eq!(state.fish_toward_next_rank, 0);

    // Add fish for rank 3->4 (Novice tier, still 100)
    state.fish_toward_next_rank = 150; // 50 excess
    let msg3 = check_rank_up_with_max(&mut state, 40);
    assert!(msg3.is_some(), "Third rank-up should succeed");
    assert_eq!(state.rank, 4);
    assert_eq!(
        state.fish_toward_next_rank, 50,
        "Excess 50 should carry over"
    );
}

#[test]
fn test_rank_up_message_contains_tier_name_at_mythic_boundary() {
    // Rank 30->31 crosses from Grandmaster to Mythic tier (requires max_rank 40)
    let mut state = fresh_fishing_state(30, 2000);
    let msg = check_rank_up_with_max(&mut state, 40).expect("Should rank up to Mythic tier");
    assert!(
        msg.contains("Fishing rank up!"),
        "Message should contain 'Fishing rank up!'"
    );
    assert!(
        msg.contains("rank 31"),
        "Message should state the new rank number"
    );
    // "Titan's Bane" is the name for rank 31 (first Mythic rank)
    assert!(
        msg.contains("Titan's Bane"),
        "Message should contain rank name 'Titan\\'s Bane' for rank 31"
    );
}

// =============================================================================
// FishingState — fish_required_for_rank monotonically increases across tiers
// =============================================================================

#[test]
fn test_fish_required_increases_across_tier_boundaries() {
    // Tier transition points: ranks 6, 11, 16, 21, 26, 31, 36
    let tier_first_ranks = [1u32, 6, 11, 16, 21, 26, 31, 36];
    let requirements: Vec<u32> = tier_first_ranks
        .iter()
        .map(|&r| FishingState::fish_required_for_rank(r))
        .collect();

    for window in requirements.windows(2) {
        assert!(
            window[0] <= window[1],
            "Fish requirements should be non-decreasing across tier boundaries, \
             but found {} > {}",
            window[0],
            window[1]
        );
    }
}
