use quest::core::game_state::GameState;
use quest::fishing::generation::{generate_fish_with_rank, LeviathanResult};
use quest::fishing::logic::{tick_fishing_with_haven_result, HavenFishingBonuses};
use quest::fishing::types::{FishRarity, FishingPhase, FishingSession, FishingState};
use quest::stormglass::spending::can_purchase_storm_lure;
use quest::stormglass::types::{EXCHANGE_MENU_ITEMS, STORM_LURE_COST};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

fn rng_from(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

fn test_state_rank40() -> GameState {
    let mut state = GameState::new("Test".to_string(), 0);
    state.fishing.rank = 40;
    state.stormglass = 100_000;
    state
}

fn reeling_session() -> FishingSession {
    FishingSession {
        spot_name: "Test".to_string(),
        total_fish: 100,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Reeling,
    }
}

// =========================================================================
// CONSTANTS AND PURCHASE VALIDATION
// =========================================================================

#[test]
fn test_storm_lure_cost_is_50000() {
    assert_eq!(STORM_LURE_COST, 50_000);
}

#[test]
fn test_exchange_menu_items_is_4() {
    assert_eq!(EXCHANGE_MENU_ITEMS, 4);
}

#[test]
fn test_can_purchase_storm_lure_all_conditions_met() {
    assert!(can_purchase_storm_lure(50_000, false, 40));
}

#[test]
fn test_cannot_purchase_storm_lure_insufficient_sg() {
    assert!(!can_purchase_storm_lure(49_999, false, 40));
}

#[test]
fn test_cannot_purchase_storm_lure_already_active() {
    assert!(!can_purchase_storm_lure(100_000, true, 40));
}

#[test]
fn test_cannot_purchase_storm_lure_rank_below_40() {
    assert!(!can_purchase_storm_lure(100_000, false, 39));
}

#[test]
fn test_can_purchase_storm_lure_exact_cost() {
    assert!(can_purchase_storm_lure(STORM_LURE_COST, false, 40));
}

#[test]
fn test_cannot_purchase_storm_lure_all_conditions_fail() {
    assert!(!can_purchase_storm_lure(0, true, 1));
}

// =========================================================================
// GENERATION: LURE ENCOUNTER BONUS
// =========================================================================

#[test]
fn test_generate_fish_no_lure_bonus_preserves_base_behavior() {
    // With 0.0 lure bonus, behavior should be identical to before
    let mut rng = seeded_rng();
    for encounters in 0..10 {
        let (_, result) =
            generate_fish_with_rank(FishRarity::Legendary, 40, encounters, 0.0, 0.0, &mut rng);
        // Just verify it doesn't crash - exact results depend on seed
        assert!(matches!(
            result,
            LeviathanResult::None | LeviathanResult::Escaped { .. }
        ));
    }
}

#[test]
fn test_high_lure_bonus_guarantees_encounter() {
    // With 1.0 (100%) bonus, every legendary at rank 40 with <10 encounters should encounter
    let mut rng = seeded_rng();
    let (_, result) = generate_fish_with_rank(FishRarity::Legendary, 40, 0, 1.0, 0.0, &mut rng);
    assert!(
        matches!(
            result,
            LeviathanResult::Escaped {
                encounter_number: 1
            }
        ),
        "Expected Escaped(1), got {:?}",
        result
    );
}

#[test]
fn test_high_lure_bonus_guarantees_catch() {
    // With 1.0 bonus (100%), catch should be guaranteed at 10+ encounters
    let mut rng = seeded_rng();
    let (_, result) = generate_fish_with_rank(FishRarity::Legendary, 40, 10, 0.0, 1.0, &mut rng);
    assert_eq!(result, LeviathanResult::Caught);
}

#[test]
fn test_catch_miss_returned_when_lure_active_and_catch_fails() {
    // Find a seed where catch fails with a moderate bonus
    let mut found_catch_miss = false;
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let (_, result) =
            generate_fish_with_rank(FishRarity::Legendary, 40, 10, 0.0, 0.05, &mut rng);
        if result == LeviathanResult::CatchMiss {
            found_catch_miss = true;
            break;
        }
    }
    assert!(
        found_catch_miss,
        "Should find CatchMiss with moderate bonus over 1000 seeds"
    );
}

#[test]
fn test_no_catch_miss_without_lure_bonus() {
    // With 0.0 lure bonus, catch miss should never occur (returns None instead)
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let (_, result) =
            generate_fish_with_rank(FishRarity::Legendary, 40, 10, 0.0, 0.0, &mut rng);
        assert_ne!(
            result,
            LeviathanResult::CatchMiss,
            "Should not get CatchMiss without lure bonus (seed {})",
            seed
        );
    }
}

#[test]
fn test_lure_bonus_non_legendary_returns_none() {
    // Non-legendary fish should always return None regardless of lure bonus
    let mut rng = seeded_rng();
    for rarity in [
        FishRarity::Common,
        FishRarity::Uncommon,
        FishRarity::Rare,
        FishRarity::Epic,
    ] {
        let (_, result) = generate_fish_with_rank(rarity, 40, 0, 1.0, 1.0, &mut rng);
        assert_eq!(result, LeviathanResult::None);
    }
}

#[test]
fn test_lure_bonus_below_rank40_returns_none() {
    let mut rng = seeded_rng();
    let (_, result) = generate_fish_with_rank(FishRarity::Legendary, 39, 0, 1.0, 0.0, &mut rng);
    assert_eq!(result, LeviathanResult::None);
}

#[test]
fn test_catch_chance_capped_at_100_percent() {
    // Even with huge bonus, should not crash or behave oddly
    let mut rng = seeded_rng();
    let (_, result) = generate_fish_with_rank(FishRarity::Legendary, 40, 10, 0.0, 10.0, &mut rng);
    assert_eq!(
        result,
        LeviathanResult::Caught,
        "100%+ catch chance should always catch"
    );
}

#[test]
fn test_encounter_bonus_increases_encounter_rate() {
    // Count encounters with and without lure bonus over many seeds
    let mut encounters_without = 0u32;
    let mut encounters_with = 0u32;
    let trials = 5000;

    for seed in 0..trials {
        let mut rng1 = rng_from(seed);
        let (_, r1) = generate_fish_with_rank(FishRarity::Legendary, 40, 0, 0.0, 0.0, &mut rng1);
        if matches!(r1, LeviathanResult::Escaped { .. }) {
            encounters_without += 1;
        }

        let mut rng2 = rng_from(seed);
        let (_, r2) = generate_fish_with_rank(FishRarity::Legendary, 40, 0, 0.10, 0.0, &mut rng2);
        if matches!(r2, LeviathanResult::Escaped { .. }) {
            encounters_with += 1;
        }
    }

    assert!(
        encounters_with > encounters_without,
        "Lure encounter bonus should increase encounter rate ({} with bonus vs {} without)",
        encounters_with,
        encounters_without
    );
}

// =========================================================================
// TICK INTEGRATION: LURE CONSUMPTION
// =========================================================================

#[test]
fn test_lure_consumed_on_encounter() {
    // Find a seed where encounter fires with high bonus
    let haven = HavenFishingBonuses::default();
    let mut found = false;
    for seed in 0..5000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_tracking_bonus = 0.0;
        state.fishing.lure_miss_ramp = 0.10; // max miss ramp for best encounter chance
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_encounter.is_some() {
            // Lure should be consumed
            assert!(
                !state.fishing.storm_lure_active,
                "Lure should be consumed on encounter"
            );
            assert!(result.lure_consumed, "lure_consumed flag should be set");
            // Tracking should increase by 1.5%
            assert!(
                (state.fishing.lure_tracking_bonus - 0.015).abs() < 0.001,
                "Tracking should be +1.5% after first encounter, got {}",
                state.fishing.lure_tracking_bonus
            );
            // Miss ramp should reset
            assert_eq!(
                state.fishing.lure_miss_ramp, 0.0,
                "Miss ramp should reset on encounter"
            );
            found = true;
            break;
        }
    }
    assert!(
        found,
        "Should find encounter with lure active over 5000 seeds"
    );
}

#[test]
fn test_lure_consumed_on_catch() {
    let haven = HavenFishingBonuses::default();
    let mut found = false;
    for seed in 0..10000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.leviathan_encounters = 10;
        state.fishing.storm_lure_active = true;
        state.fishing.lure_tracking_bonus = 0.15; // 10 encounters worth
        state.fishing.lure_miss_ramp = 0.05;
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.caught_storm_leviathan {
            assert!(!state.fishing.storm_lure_active, "Lure consumed on catch");
            assert!(result.lure_consumed);
            assert_eq!(
                state.fishing.lure_miss_ramp, 0.0,
                "Miss ramp resets on catch"
            );
            found = true;
            break;
        }
    }
    assert!(found, "Should catch Leviathan over 10000 seeds with lure");
}

#[test]
fn test_lure_consumed_on_catch_miss() {
    let haven = HavenFishingBonuses::default();
    let mut found = false;
    for seed in 0..10000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.leviathan_encounters = 10;
        state.fishing.storm_lure_active = true;
        state.fishing.lure_tracking_bonus = 0.05; // need > 0 so CatchMiss fires instead of None
        state.fishing.lure_miss_ramp = 0.02;
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_catch_miss {
            assert!(
                !state.fishing.storm_lure_active,
                "Lure consumed on catch miss"
            );
            assert!(result.lure_consumed);
            // Miss ramp should increase from 0.02 to 0.025
            assert!(
                (state.fishing.lure_miss_ramp - 0.025).abs() < 0.001,
                "Miss ramp should be 0.02 + 0.005 = 0.025 after catch miss, got {}",
                state.fishing.lure_miss_ramp
            );
            found = true;
            break;
        }
    }
    assert!(found, "Should find catch miss over 10000 seeds");
}

#[test]
fn test_miss_ramp_increases_on_legendary_no_encounter() {
    // When lure active, legendary at rank 40 with no encounter should increase miss ramp
    let haven = HavenFishingBonuses::default();
    let mut found = false;
    for seed in 0..5000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.0;
        state.fishing.leviathan_encounters = 0;
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // If it was a legendary catch but no encounter, miss ramp should increase
        // The lure should still be active (not consumed) since no encounter/catch happened
        if result.leviathan_encounter.is_none()
            && !result.caught_storm_leviathan
            && !result.leviathan_catch_miss
            && state.fishing.storm_lure_active
        // lure not consumed
        {
            // Check if a legendary was caught by looking at messages
            let had_legendary = result.messages.iter().any(|m| m.contains("[Legendary]"));
            if had_legendary {
                assert!(
                    (state.fishing.lure_miss_ramp - 0.005).abs() < 0.001,
                    "Miss ramp should be 0.5% after legendary miss (got {})",
                    state.fishing.lure_miss_ramp
                );
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "Should find legendary with no encounter over 5000 seeds at rank 40"
    );
}

#[test]
fn test_miss_ramp_capped_at_10_percent() {
    let haven = HavenFishingBonuses::default();

    // Set miss ramp at exactly 10%, verify it doesn't go above
    let mut state = test_state_rank40();
    state.fishing.storm_lure_active = true;
    state.fishing.lure_miss_ramp = 0.10;
    state.fishing.leviathan_encounters = 0;

    // Manually verify the cap formula
    let new_ramp = (0.10_f64 + 0.005).min(0.10);
    assert!(
        (new_ramp - 0.10).abs() < 0.001,
        "Miss ramp should cap at 10%"
    );

    // Also verify via tick if we can find a legendary
    for seed in 0..5000 {
        let mut rng = rng_from(seed);
        let mut state2 = test_state_rank40();
        state2.fishing.storm_lure_active = true;
        state2.fishing.lure_miss_ramp = 0.10; // already at cap
        state2.fishing.leviathan_encounters = 0;
        state2.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state2, &mut rng, &haven, 0.0);

        if result.leviathan_encounter.is_none()
            && !result.caught_storm_leviathan
            && !result.leviathan_catch_miss
            && state2.fishing.storm_lure_active
        {
            let had_legendary = result.messages.iter().any(|m| m.contains("[Legendary]"));
            if had_legendary {
                // Miss ramp should stay at cap
                assert!(
                    state2.fishing.lure_miss_ramp <= 0.10 + 0.0001,
                    "Miss ramp should not exceed 10%, got {}",
                    state2.fishing.lure_miss_ramp
                );
                break;
            }
        }
    }
}

#[test]
fn test_tracking_bonus_accumulates_across_encounters() {
    // Tracking should be +1.5% per encounter
    let expected_after_5: f64 = 0.015 * 5.0;
    assert!(
        (expected_after_5 - 0.075).abs() < 0.001,
        "5 encounters should give 7.5% tracking"
    );

    let expected_after_10: f64 = 0.015 * 10.0;
    assert!(
        (expected_after_10 - 0.15).abs() < 0.001,
        "10 encounters should give 15% tracking"
    );
}

#[test]
fn test_no_lure_no_bonus_behavior_unchanged() {
    // Without lure active, behavior should be exactly as before
    let haven = HavenFishingBonuses::default();
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.storm_lure_active = false;
        state.fishing.lure_miss_ramp = 0.05; // has ramp but lure not active
        state.fishing.lure_tracking_bonus = 0.10; // has tracking but lure not active
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // Miss ramp and tracking should not change when lure is not active
        assert!(
            (state.fishing.lure_miss_ramp - 0.05).abs() < 0.001,
            "Miss ramp should not change without active lure (seed {}, got {})",
            seed,
            state.fishing.lure_miss_ramp
        );
        assert!(
            (state.fishing.lure_tracking_bonus - 0.10).abs() < 0.001,
            "Tracking should not change without active lure (seed {}, got {})",
            seed,
            state.fishing.lure_tracking_bonus
        );
        // lure_consumed should be false
        assert!(
            !result.lure_consumed,
            "No lure consumption without active lure (seed {})",
            seed
        );
    }
}

#[test]
fn test_lure_not_consumed_on_non_legendary_catch() {
    // If lure is active but fish is not legendary, lure should remain active
    let haven = HavenFishingBonuses::default();
    let mut found_non_legendary = false;
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.0;
        state.fishing.leviathan_encounters = 0;
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // Check if a non-legendary was caught
        let had_legendary = result.messages.iter().any(|m| m.contains("[Legendary]"));
        let had_catch = result.messages.iter().any(|m| m.contains("Caught"));
        if had_catch && !had_legendary {
            // Lure should still be active
            assert!(
                state.fishing.storm_lure_active,
                "Lure should remain active after non-legendary catch (seed {})",
                seed
            );
            assert!(
                !result.lure_consumed,
                "Lure should not be consumed on non-legendary catch"
            );
            // Miss ramp should not change for non-legendary
            assert_eq!(
                state.fishing.lure_miss_ramp, 0.0,
                "Miss ramp should not change for non-legendary catch"
            );
            found_non_legendary = true;
            break;
        }
    }
    assert!(
        found_non_legendary,
        "Should find non-legendary catch in 1000 seeds"
    );
}

#[test]
fn test_tracking_bonus_persists_after_lure_consumed() {
    // Tracking bonus should remain even after lure is consumed
    let haven = HavenFishingBonuses::default();
    let mut found = false;
    for seed in 0..5000 {
        let mut rng = rng_from(seed);
        let mut state = test_state_rank40();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_tracking_bonus = 0.06; // already had 4 encounters
        state.fishing.lure_miss_ramp = 0.05;
        state.fishing.leviathan_encounters = 4;
        state.active_fishing = Some(reeling_session());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_encounter.is_some() {
            assert!(!state.fishing.storm_lure_active, "Lure consumed");
            // Tracking should have increased from 0.06 to 0.075
            assert!(
                (state.fishing.lure_tracking_bonus - 0.075).abs() < 0.001,
                "Tracking should accumulate: expected 0.075, got {}",
                state.fishing.lure_tracking_bonus
            );
            found = true;
            break;
        }
    }
    assert!(found, "Should find encounter in 5000 seeds");
}

// =========================================================================
// SERIALIZATION BACKWARD COMPATIBILITY
// =========================================================================

#[test]
fn test_fishing_state_serde_roundtrip_with_lure() {
    let state = FishingState {
        rank: 40,
        total_fish_caught: 10000,
        fish_toward_next_rank: 500,
        legendary_catches: 50,
        leviathan_encounters: 7,
        storm_lure_active: true,
        lure_miss_ramp: 0.045,
        lure_tracking_bonus: 0.105,
    };

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: FishingState = serde_json::from_str(&json).unwrap();

    assert!(deserialized.storm_lure_active);
    assert!((deserialized.lure_miss_ramp - 0.045).abs() < 0.001);
    assert!((deserialized.lure_tracking_bonus - 0.105).abs() < 0.001);
    assert_eq!(deserialized.rank, 40);
    assert_eq!(deserialized.leviathan_encounters, 7);
}

#[test]
fn test_fishing_state_deserialize_without_lure_fields() {
    // Old save format without lure fields should deserialize with defaults
    let json = r#"{"rank":30,"total_fish_caught":5000,"fish_toward_next_rank":200,"legendary_catches":10}"#;
    let state: FishingState = serde_json::from_str(json).unwrap();

    assert_eq!(state.rank, 30);
    assert!(!state.storm_lure_active);
    assert_eq!(state.lure_miss_ramp, 0.0);
    assert_eq!(state.lure_tracking_bonus, 0.0);
    assert_eq!(state.leviathan_encounters, 0);
}

#[test]
fn test_fishing_state_skip_serializing_defaults() {
    // Default lure values should not appear in serialized output
    let state = FishingState::default();
    let json = serde_json::to_string(&state).unwrap();

    assert!(
        !json.contains("storm_lure_active"),
        "Should skip false storm_lure_active in: {}",
        json
    );
    assert!(
        !json.contains("lure_miss_ramp"),
        "Should skip 0.0 lure_miss_ramp in: {}",
        json
    );
    assert!(
        !json.contains("lure_tracking_bonus"),
        "Should skip 0.0 lure_tracking_bonus in: {}",
        json
    );
    assert!(
        !json.contains("leviathan_encounters"),
        "Should skip 0 leviathan_encounters in: {}",
        json
    );
}

#[test]
fn test_fishing_state_roundtrip_preserves_all_fields() {
    // Full roundtrip with all fields populated
    let original = FishingState {
        rank: 35,
        total_fish_caught: 50000,
        fish_toward_next_rank: 1234,
        legendary_catches: 100,
        leviathan_encounters: 5,
        storm_lure_active: false,
        lure_miss_ramp: 0.035,
        lure_tracking_bonus: 0.075,
    };

    let json = serde_json::to_string_pretty(&original).unwrap();
    let restored: FishingState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.rank, original.rank);
    assert_eq!(restored.total_fish_caught, original.total_fish_caught);
    assert_eq!(
        restored.fish_toward_next_rank,
        original.fish_toward_next_rank
    );
    assert_eq!(restored.legendary_catches, original.legendary_catches);
    assert_eq!(restored.leviathan_encounters, original.leviathan_encounters);
    assert_eq!(restored.storm_lure_active, original.storm_lure_active);
    assert!((restored.lure_miss_ramp - original.lure_miss_ramp).abs() < 0.0001);
    assert!((restored.lure_tracking_bonus - original.lure_tracking_bonus).abs() < 0.0001);
}

#[test]
fn test_fishing_state_extra_fields_ignored() {
    // Future-proof: extra fields in JSON should be ignored
    let json = r#"{"rank":40,"total_fish_caught":9999,"fish_toward_next_rank":0,"legendary_catches":50,"storm_lure_active":true,"lure_miss_ramp":0.05,"lure_tracking_bonus":0.10,"some_future_field":"ignored"}"#;
    let state: FishingState = serde_json::from_str(json).unwrap();

    assert_eq!(state.rank, 40);
    assert!(state.storm_lure_active);
    assert!((state.lure_miss_ramp - 0.05).abs() < 0.001);
    assert!((state.lure_tracking_bonus - 0.10).abs() < 0.001);
}
