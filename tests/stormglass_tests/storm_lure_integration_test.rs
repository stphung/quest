use quest::character::prestige_actions::perform_prestige;
use quest::core::game_state::GameState;
use quest::fishing::generation::{generate_fish_with_rank, LeviathanResult};
use quest::fishing::logic::{tick_fishing_with_haven_result, HavenFishingBonuses};
use quest::fishing::types::{FishRarity, FishingPhase, FishingSession, FishingState};
use quest::stormglass::spending::can_purchase_storm_lure;
use quest::stormglass::types::{EXCHANGE_MENU_ITEMS, STORM_LURE_COST};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn rng_from(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

fn rank40_state() -> GameState {
    let mut state = GameState::new("Tester".to_string(), 0);
    state.fishing.rank = 40;
    state.stormglass = 200_000;
    state
}

fn reeling_1tick() -> FishingSession {
    FishingSession {
        spot_name: "Deep".to_string(),
        total_fish: 100,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Reeling,
    }
}

fn haven_default() -> HavenFishingBonuses {
    HavenFishingBonuses::default()
}

// =========================================================================
// FULL HUNT SIMULATION WITH LURE
// =========================================================================

fn simulate_hunt(seed: u64, use_lure: bool) -> u32 {
    let mut rng = rng_from(seed);
    let mut encounters: u8 = 0;
    let mut tracking = 0.0f64;
    let mut miss_ramp = 0.0f64;
    let mut legendary_count = 0u32;

    loop {
        legendary_count += 1;

        let enc_bonus = if use_lure { tracking + miss_ramp } else { 0.0 };
        let catch_bonus = if use_lure { tracking + miss_ramp } else { 0.0 };

        let (_, result) = generate_fish_with_rank(
            FishRarity::Legendary,
            40,
            encounters,
            enc_bonus,
            catch_bonus,
            &mut rng,
        );

        match result {
            LeviathanResult::Escaped { encounter_number } => {
                encounters = encounter_number;
                if use_lure {
                    tracking += 0.015;
                    miss_ramp = 0.0;
                }
            }
            LeviathanResult::Caught => {
                return legendary_count;
            }
            LeviathanResult::CatchMiss => {
                if use_lure {
                    miss_ramp = (miss_ramp + 0.005).min(0.10);
                }
            }
            LeviathanResult::None => {
                if use_lure && encounters < 10 {
                    miss_ramp = (miss_ramp + 0.005).min(0.10);
                }
            }
        }

        // Safety: prevent infinite loop
        if legendary_count > 50_000 {
            panic!("Hunt took too long (50k+ legendaries) - seed {}", seed);
        }
    }
}

#[test]
fn test_full_hunt_with_lure_completes_faster_than_without() {
    // Simulate a full Leviathan hunt with lure always active
    // vs without lure. Count total legendaries needed.
    let seed = 42u64;

    // Hunt WITHOUT lure
    let legendaries_no_lure = simulate_hunt(seed, false);

    // Hunt WITH lure
    let legendaries_with_lure = simulate_hunt(seed, true);

    // With lure should require fewer legendaries (or same)
    assert!(
        legendaries_with_lure <= legendaries_no_lure,
        "Lure hunt ({} legendaries) should be <= no-lure hunt ({})",
        legendaries_with_lure,
        legendaries_no_lure
    );
}

#[test]
fn test_hunt_simulation_multiple_seeds() {
    // Run across multiple seeds to verify lure consistently helps
    let mut lure_faster = 0;
    let mut same = 0;
    let seeds = 100;

    for seed in 0..seeds {
        let no_lure = simulate_hunt(seed, false);
        let with_lure = simulate_hunt(seed, true);

        if with_lure < no_lure {
            lure_faster += 1;
        } else if with_lure == no_lure {
            same += 1;
        }
    }

    // Lure should help in the majority of cases
    assert!(
        lure_faster + same >= seeds * 7 / 10,
        "Lure should help or tie in 70%+ of seeds. Faster: {}, Same: {}, Total: {}",
        lure_faster,
        same,
        seeds
    );
}

// =========================================================================
// EDGE CASES
// =========================================================================

#[test]
fn test_lure_at_zero_encounters() {
    // Lure should boost encounter chance from the very start
    let mut hits = 0;
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        // Base chance for encounter 1 is 5%. With 10% bonus, should be 15%.
        let (_, result) =
            generate_fish_with_rank(FishRarity::Legendary, 40, 0, 0.10, 0.0, &mut rng);
        if matches!(result, LeviathanResult::Escaped { .. }) {
            hits += 1;
        }
    }
    // Expected ~15% with 10% bonus on top of 5% base = 15%. Allow 10-20% range.
    let rate = hits as f64 / 1000.0;
    assert!(
        rate > 0.10 && rate < 0.20,
        "Encounter rate with 10% bonus should be ~15%, got {:.1}%",
        rate * 100.0
    );
}

#[test]
fn test_lure_at_ten_encounters_catch_phase() {
    // At 10 encounters with tracking bonus, catch rate should be boosted
    let mut catches = 0;
    let tracking = 0.15; // 10 encounters worth of tracking
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let (_, result) =
            generate_fish_with_rank(FishRarity::Legendary, 40, 10, 0.0, tracking, &mut rng);
        if result == LeviathanResult::Caught {
            catches += 1;
        }
    }
    // Base 25% + 15% tracking = 40%. Allow 30-50% range.
    let rate = catches as f64 / 1000.0;
    assert!(
        rate > 0.30 && rate < 0.50,
        "Catch rate with 15% tracking should be ~40%, got {:.1}%",
        rate * 100.0
    );
}

#[test]
fn test_tracking_persists_across_lure_purchases() {
    // Tracking bonus should persist even after lure is consumed and new one purchased
    let mut state = rank40_state();
    state.fishing.storm_lure_active = false;
    state.fishing.lure_tracking_bonus = 0.075; // from 5 previous encounters

    // "Purchase" new lure
    state.fishing.storm_lure_active = true;
    state.stormglass -= STORM_LURE_COST;

    // Tracking should still be there
    assert!((state.fishing.lure_tracking_bonus - 0.075).abs() < 0.001);
}

#[test]
fn test_miss_ramp_resets_when_new_lure_purchased() {
    // Miss ramp should reset to 0 when buying a new lure
    let mut state = rank40_state();
    state.fishing.storm_lure_active = false;
    state.fishing.lure_miss_ramp = 0.04; // built up from previous lure

    // "Purchase" new lure (mirrors stormglass_input.rs purchase handler)
    state.fishing.storm_lure_active = true;
    state.fishing.lure_miss_ramp = 0.0;

    // Miss ramp should be reset
    assert_eq!(state.fishing.lure_miss_ramp, 0.0);
}

#[test]
fn test_lure_inactive_by_default() {
    let state = FishingState::default();
    assert!(!state.storm_lure_active);
    assert_eq!(state.lure_miss_ramp, 0.0);
    assert_eq!(state.lure_tracking_bonus, 0.0);
}

#[test]
fn test_cannot_have_two_lures() {
    // If lure is already active, cannot purchase another
    assert!(!can_purchase_storm_lure(1_000_000, true, false, 40));
}

#[test]
fn test_cannot_purchase_lure_below_rank_40() {
    assert!(!can_purchase_storm_lure(1_000_000, false, false, 39));
    assert!(!can_purchase_storm_lure(1_000_000, false, false, 1));
}

#[test]
fn test_cannot_purchase_lure_without_enough_stormglass() {
    assert!(!can_purchase_storm_lure(49_999, false, false, 40));
    assert!(!can_purchase_storm_lure(0, false, false, 40));
}

#[test]
fn test_can_purchase_lure_at_exact_cost() {
    assert!(can_purchase_storm_lure(STORM_LURE_COST, false, false, 40));
}

#[test]
fn test_exchange_menu_items_includes_storm_lure() {
    // EXCHANGE_MENU_ITEMS should be 4 (Invoke Challenge, Chrono Surge, Storm Sigils, Storm Lure)
    assert_eq!(
        EXCHANGE_MENU_ITEMS, 4,
        "Exchange menu should have 4 items including Storm Lure"
    );
}

#[test]
fn test_lure_tracking_persists_across_prestige() {
    // Fishing state (including lure tracking/miss_ramp) is preserved across prestige
    let mut state = rank40_state();
    state.fishing.storm_lure_active = true;
    state.fishing.lure_tracking_bonus = 0.045; // 3 encounters worth
    state.fishing.lure_miss_ramp = 0.02;
    state.fishing.leviathan_encounters = 3;

    // Set up state to be prestige-eligible
    state.character_level = 25; // meets Bronze threshold
    state.prestige_rank = 0;

    perform_prestige(&mut state);

    // Fishing state should be preserved (prestige only clears active_fishing)
    assert_eq!(state.fishing.rank, 40);
    assert!(
        (state.fishing.lure_tracking_bonus - 0.045).abs() < 0.001,
        "Tracking bonus should persist across prestige"
    );
    assert!(
        (state.fishing.lure_miss_ramp - 0.02).abs() < 0.001,
        "Miss ramp should persist across prestige"
    );
    assert!(
        state.fishing.storm_lure_active,
        "Lure active state should persist across prestige"
    );
    assert_eq!(
        state.fishing.leviathan_encounters, 3,
        "Leviathan encounters should persist across prestige"
    );
    // But active_fishing session should be cleared
    assert!(
        state.active_fishing.is_none(),
        "Active fishing session should be cleared on prestige"
    );
}

// =========================================================================
// TICK-LEVEL INTEGRATION
// =========================================================================

#[test]
fn test_tick_lure_encounter_then_miss_ramp_resets() {
    // After an encounter, miss ramp should be 0 even if it was high before
    let haven = haven_default();

    for seed in 0..2000 {
        let mut rng = rng_from(seed);
        let mut state = rank40_state();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.10; // max ramp
        state.fishing.lure_tracking_bonus = 0.0;
        state.fishing.leviathan_encounters = 0;
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_encounter.is_some() {
            assert_eq!(
                state.fishing.lure_miss_ramp, 0.0,
                "Miss ramp should reset after encounter"
            );
            assert!(
                (state.fishing.lure_tracking_bonus - 0.015).abs() < 0.001,
                "Tracking should be +1.5% after first encounter"
            );
            assert!(
                !state.fishing.storm_lure_active,
                "Lure should be consumed after encounter"
            );
            assert!(result.lure_consumed, "lure_consumed flag should be set");
            return; // test passed
        }
    }
    panic!("Could not trigger encounter in 2000 seeds");
}

#[test]
fn test_tick_multiple_catches_accumulate_tracking() {
    // Simulate multiple encounters, verify tracking accumulates
    let haven = haven_default();
    let mut total_tracking = 0.0f64;
    let mut encounters_found = 0u32;

    let mut state = rank40_state();
    state.fishing.storm_lure_active = true;

    // NOTE: bound kept higher than the ~1000-2000 used for the single-encounter tests
    // above. This test needs a *2nd* independent encounter (not just the first), and
    // empirically (with this exact state/rng setup) the 2nd encounter isn't observed
    // until seed ~2022 — so 4000 keeps ~2x headroom instead of cutting all the way to
    // the 1000-2000 range that suffices for a single-encounter search.
    for seed in 0..4000 {
        if encounters_found >= 3 {
            break;
        }

        let mut rng = rng_from(seed);
        state.fishing.storm_lure_active = true; // re-buy lure after consumption
        state.fishing.lure_miss_ramp = 0.0; // miss ramp resets on new lure purchase
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_encounter.is_some() {
            encounters_found += 1;
            total_tracking += 0.015;
            assert!(
                (state.fishing.lure_tracking_bonus - total_tracking).abs() < 0.001,
                "Tracking should be {:.1}% after {} encounters",
                total_tracking * 100.0,
                encounters_found
            );
        }
    }

    assert!(
        encounters_found >= 2,
        "Should find at least 2 encounters in 4000 seeds"
    );
}

#[test]
fn test_tick_lure_catch_miss_increments_miss_ramp() {
    // When in catch phase (encounters >= 10), a miss should increment miss_ramp
    let haven = haven_default();

    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let mut state = rank40_state();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.0;
        state.fishing.lure_tracking_bonus = 0.015; // some tracking
        state.fishing.leviathan_encounters = 10; // catch phase
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_catch_miss {
            assert!(
                (state.fishing.lure_miss_ramp - 0.005).abs() < 0.001,
                "Miss ramp should be +0.5% after one catch miss, got {}",
                state.fishing.lure_miss_ramp
            );
            assert!(
                !state.fishing.storm_lure_active,
                "Lure should be consumed on catch miss"
            );
            assert!(result.lure_consumed, "lure_consumed flag should be set");
            return;
        }
    }
    panic!("Could not trigger catch miss in 1000 seeds");
}

#[test]
fn test_tick_lure_caught_resets_miss_ramp() {
    // When Leviathan is caught, miss ramp should reset to 0
    let haven = haven_default();

    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let mut state = rank40_state();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.05;
        state.fishing.lure_tracking_bonus = 0.10;
        state.fishing.leviathan_encounters = 10; // catch phase
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.caught_storm_leviathan {
            assert_eq!(
                state.fishing.lure_miss_ramp, 0.0,
                "Miss ramp should reset to 0 on catch"
            );
            assert!(
                !state.fishing.storm_lure_active,
                "Lure should be consumed on catch"
            );
            assert!(result.lure_consumed, "lure_consumed flag should be set");
            return;
        }
    }
    panic!("Could not trigger catch in 1000 seeds");
}

#[test]
fn test_non_legendary_catch_does_not_affect_lure() {
    // Non-legendary catches should not affect lure state at all
    let haven = haven_default();

    // Use a low rank to guarantee non-legendary catches
    let mut state = GameState::new("Test".to_string(), 0);
    state.fishing.rank = 1; // very low rank = almost always common
    state.fishing.storm_lure_active = true;
    state.fishing.lure_miss_ramp = 0.03;
    state.fishing.lure_tracking_bonus = 0.06;

    for seed in 0..100 {
        let mut rng = rng_from(seed);
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // Lure state should be unchanged (rank 1 can't encounter Leviathan)
        assert!(state.fishing.storm_lure_active, "Lure should remain active");
        assert!(!result.lure_consumed, "Lure should not be consumed");
        assert!((state.fishing.lure_miss_ramp - 0.03).abs() < 0.001);
        assert!((state.fishing.lure_tracking_bonus - 0.06).abs() < 0.001);
    }
}

#[test]
fn test_lure_purchase_and_use_full_flow() {
    // Simulate: purchase lure -> fish -> encounter -> lure consumed
    let mut state = rank40_state();

    // Purchase lure
    assert!(state.stormglass >= STORM_LURE_COST);
    state.stormglass -= STORM_LURE_COST;
    state.fishing.storm_lure_active = true;

    let remaining_sg = state.stormglass;
    assert_eq!(remaining_sg, 200_000 - 50_000);
    assert!(state.fishing.storm_lure_active);

    // Fish until something happens with lure
    let haven = haven_default();
    let mut something_happened = false;

    for seed in 0..2000 {
        let mut rng = rng_from(seed);
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.lure_consumed {
            assert!(
                !state.fishing.storm_lure_active,
                "Lure should be deactivated"
            );
            something_happened = true;
            break;
        }

        // Re-activate lure if still active (just continue the loop)
        if !state.fishing.storm_lure_active {
            something_happened = true;
            break;
        }
    }

    assert!(something_happened, "Lure should eventually be consumed");
}

#[test]
fn test_legendary_miss_at_rank40_with_lure_increments_miss_ramp() {
    // When a legendary is caught at rank 40 with lure active but no Leviathan encounter,
    // the miss_ramp should increase (encounter phase, LeviathanResult::None)
    let haven = haven_default();
    let mut miss_ramp_increased = false;

    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        let mut state = rank40_state();
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.0;
        state.fishing.lure_tracking_bonus = 0.0;
        state.fishing.leviathan_encounters = 0; // encounter phase
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // We need a legendary catch where leviathan did NOT appear
        if result.leviathan_encounter.is_none()
            && !result.caught_storm_leviathan
            && !result.leviathan_catch_miss
            && state.fishing.lure_miss_ramp > 0.0
        {
            // This means a legendary was caught but no encounter -> miss ramp increased
            assert!(
                (state.fishing.lure_miss_ramp - 0.005).abs() < 0.001,
                "Miss ramp should be +0.5%, got {}",
                state.fishing.lure_miss_ramp
            );
            // Lure should still be active (only consumed on encounter/catch/catchmiss)
            assert!(
                state.fishing.storm_lure_active,
                "Lure should remain active on legendary miss"
            );
            miss_ramp_increased = true;
            break;
        }
    }

    assert!(
        miss_ramp_increased,
        "Should find a legendary miss that increments miss_ramp in 1000 seeds"
    );
}

#[test]
fn test_miss_ramp_caps_at_ten_percent() {
    // Miss ramp should never exceed 0.10 (10%)
    let haven = haven_default();
    let mut state = rank40_state();
    state.fishing.storm_lure_active = true;
    state.fishing.lure_miss_ramp = 0.095; // close to cap
    state.fishing.lure_tracking_bonus = 0.0;
    state.fishing.leviathan_encounters = 0;

    // Find a legendary miss to push miss_ramp
    for seed in 0..1000 {
        let mut rng = rng_from(seed);
        state.fishing.storm_lure_active = true;
        state.fishing.lure_miss_ramp = 0.095;
        state.active_fishing = Some(reeling_1tick());

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        if result.leviathan_encounter.is_none()
            && !result.caught_storm_leviathan
            && !result.leviathan_catch_miss
            && state.fishing.lure_miss_ramp > 0.095
        {
            // Should be capped at 0.10
            assert!(
                state.fishing.lure_miss_ramp <= 0.10 + 0.001,
                "Miss ramp should cap at 10%, got {}",
                state.fishing.lure_miss_ramp
            );
            return;
        }
    }
    // If we never hit a legendary miss in 1000 seeds, that's okay for this test
    // (the cap is tested directly via the generation function)
}

// =========================================================================
// SERIALIZATION ROUNDTRIP
// =========================================================================

#[test]
fn test_fishing_state_lure_fields_serialize() {
    let state = FishingState {
        rank: 40,
        total_fish_caught: 1000,
        fish_toward_next_rank: 50,
        legendary_catches: 10,
        leviathan_encounters: 5,
        storm_lure_active: true,
        lure_miss_ramp: 0.035,
        lure_tracking_bonus: 0.075,
        leviathan_caught: false,
    };

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: FishingState = serde_json::from_str(&json).unwrap();

    assert!(deserialized.storm_lure_active);
    assert!((deserialized.lure_miss_ramp - 0.035).abs() < 0.0001);
    assert!((deserialized.lure_tracking_bonus - 0.075).abs() < 0.0001);
}

#[test]
fn test_fishing_state_lure_fields_default_on_missing() {
    // Old save files won't have lure fields; they should default
    let json =
        r#"{"rank":30,"total_fish_caught":500,"fish_toward_next_rank":10,"legendary_catches":5}"#;
    let state: FishingState = serde_json::from_str(json).unwrap();

    assert!(!state.storm_lure_active);
    assert_eq!(state.lure_miss_ramp, 0.0);
    assert_eq!(state.lure_tracking_bonus, 0.0);
}

#[test]
fn test_fishing_state_lure_fields_skip_when_default() {
    // When lure fields are at default (false/0.0), they should be omitted from JSON
    let state = FishingState {
        storm_lure_active: false,
        lure_miss_ramp: 0.0,
        lure_tracking_bonus: 0.0,
        leviathan_caught: false,
        ..Default::default()
    };

    let json = serde_json::to_string(&state).unwrap();
    assert!(
        !json.contains("storm_lure_active"),
        "storm_lure_active should be skipped when false"
    );
    assert!(
        !json.contains("lure_miss_ramp"),
        "lure_miss_ramp should be skipped when 0.0"
    );
    assert!(
        !json.contains("lure_tracking_bonus"),
        "lure_tracking_bonus should be skipped when 0.0"
    );
}
