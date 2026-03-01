//! Edge case tests for Stormglass currency and Storm Sigils systems.
//!
//! Covers gaps not addressed in stormglass_test.rs, storm_sigils_test.rs,
//! storm_lure_test.rs, storm_lure_integration_test.rs, and chrono_overcharge_test.rs.

use quest::challenges::menu::ChallengeType;
use quest::core::game_state::GameState;
use quest::dungeon::types::DungeonSize;
use quest::items::types::Rarity;
use quest::stormglass::earning;
use quest::stormglass::sigils::{
    daily_sigil_pool_for_day, generate_sigil_choices, roll_sigil, Sigil, SigilBonuses,
    SigilEffectType, SigilGrade, StormSigils, DAILY_POOL_SIZE, ETCH_COST, MAX_SIGIL_SLOTS,
    SLOT_UNLOCK_COSTS,
};
use quest::stormglass::spending;
use quest::stormglass::types::{ChronoSurgeState, ExchangePhase, ExchangeUiState, STORM_LURE_COST};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ── Earning: all rarity salvage values are ordered correctly ─────────────

#[test]
fn test_salvage_value_ordering() {
    // Common <= Magic <= Rare <= Epic <= Legendary <= Mythic
    assert!(earning::salvage_value(Rarity::Common) <= earning::salvage_value(Rarity::Magic));
    assert!(earning::salvage_value(Rarity::Magic) <= earning::salvage_value(Rarity::Rare));
    assert!(earning::salvage_value(Rarity::Rare) <= earning::salvage_value(Rarity::Epic));
    assert!(earning::salvage_value(Rarity::Epic) <= earning::salvage_value(Rarity::Legendary));
    assert!(earning::salvage_value(Rarity::Legendary) <= earning::salvage_value(Rarity::Mythic));
}

#[test]
fn test_salvage_value_rare_is_greater_than_magic() {
    // Rare should be strictly greater than Magic
    assert!(earning::salvage_value(Rarity::Rare) > earning::salvage_value(Rarity::Magic));
}

#[test]
fn test_salvage_value_legendary_is_greater_than_epic() {
    assert!(earning::salvage_value(Rarity::Legendary) > earning::salvage_value(Rarity::Epic));
}

// ── Earning: dungeon cache ordering ─────────────────────────────────────

#[test]
fn test_dungeon_cache_ordering() {
    let small = earning::dungeon_cache(DungeonSize::Small);
    let medium = earning::dungeon_cache(DungeonSize::Medium);
    let large = earning::dungeon_cache(DungeonSize::Large);
    let epic = earning::dungeon_cache(DungeonSize::Epic);
    let legendary = earning::dungeon_cache(DungeonSize::Legendary);

    assert!(small < medium, "Small < Medium");
    assert!(medium < large, "Medium < Large");
    assert!(large < epic, "Large < Epic");
    assert!(epic < legendary, "Epic < Legendary");
}

#[test]
fn test_dungeon_cache_all_nonzero() {
    for size in [
        DungeonSize::Small,
        DungeonSize::Medium,
        DungeonSize::Large,
        DungeonSize::Epic,
        DungeonSize::Legendary,
    ] {
        assert!(
            earning::dungeon_cache(size) > 0,
            "{:?} cache should be > 0",
            size
        );
    }
}

// ── Earning: soulforge consolation detailed table ────────────────────────

#[test]
fn test_soulforge_consolation_levels_1_through_4_are_zero() {
    for level in 1u8..=4 {
        assert_eq!(
            earning::soulforge_consolation(level),
            0,
            "Level {} (100% success) should give 0 consolation",
            level
        );
    }
}

#[test]
fn test_soulforge_consolation_level_5_is_positive() {
    assert!(
        earning::soulforge_consolation(5) > 0,
        "Level 5 should give positive consolation"
    );
}

#[test]
fn test_soulforge_consolation_increases_with_level() {
    // Consolation should be non-decreasing from level 5 onward
    let mut prev = earning::soulforge_consolation(5);
    for level in 6u8..=10 {
        let curr = earning::soulforge_consolation(level);
        assert!(
            curr >= prev,
            "Consolation at level {} ({}) should be >= level {} ({})",
            level,
            curr,
            level - 1,
            prev
        );
        prev = curr;
    }
}

#[test]
fn test_soulforge_consolation_level_10_is_highest() {
    // Level 10 should have the highest consolation
    let level_10 = earning::soulforge_consolation(10);
    for level in 5u8..=9 {
        assert!(
            level_10 >= earning::soulforge_consolation(level),
            "Level 10 consolation should be >= level {} consolation",
            level
        );
    }
}

// ── Spending: trial options with all types excluded ──────────────────────

#[test]
fn test_generate_trial_options_excludes_many_types() {
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    // Exclude 7 of 10 types, leaving only 3 — exactly enough for options
    let exclude = vec![
        ChallengeType::Chess,
        ChallengeType::Morris,
        ChallengeType::Gomoku,
        ChallengeType::Minesweeper,
        ChallengeType::Rune,
        ChallengeType::Go,
        ChallengeType::FlappyBird,
    ];
    let options = spending::generate_trial_options(&mut rng, &exclude);
    assert_eq!(options.len(), 3, "Should still return 3 options");
    for opt in &options {
        assert!(
            !exclude.contains(&opt.challenge_type),
            "Excluded type {:?} should not appear",
            opt.challenge_type
        );
    }
}

#[test]
fn test_generate_trial_options_seeded_deterministic() {
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let opts1 = spending::generate_trial_options(&mut rng1, &[]);
    let opts2 = spending::generate_trial_options(&mut rng2, &[]);
    assert_eq!(opts1.len(), opts2.len());
    for (a, b) in opts1.iter().zip(opts2.iter()) {
        assert_eq!(a.challenge_type, b.challenge_type);
        assert_eq!(a.display_name, b.display_name);
    }
}

#[test]
fn test_generate_trial_options_display_names_nonempty() {
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let options = spending::generate_trial_options(&mut rng, &[]);
    for opt in &options {
        assert!(
            !opt.display_name.is_empty(),
            "display_name for {:?} should not be empty",
            opt.challenge_type
        );
        // Display names should be meaningful (>5 chars)
        assert!(
            opt.display_name.len() > 5,
            "display_name '{}' too short",
            opt.display_name
        );
    }
}

// ── Spending: Storm Lure purchase edge cases ─────────────────────────────

#[test]
fn test_can_purchase_storm_lure_rank_exactly_40() {
    assert!(
        spending::can_purchase_storm_lure(STORM_LURE_COST, false, 40),
        "Rank 40 exactly should allow purchase"
    );
}

#[test]
fn test_can_purchase_storm_lure_rank_41() {
    // Rank above 40 should also work
    assert!(
        spending::can_purchase_storm_lure(STORM_LURE_COST, false, 41),
        "Rank 41 should allow purchase"
    );
}

#[test]
fn test_cannot_purchase_storm_lure_rank_39() {
    assert!(
        !spending::can_purchase_storm_lure(STORM_LURE_COST, false, 39),
        "Rank 39 should not allow purchase"
    );
}

#[test]
fn test_cannot_purchase_storm_lure_one_sg_short() {
    assert!(
        !spending::can_purchase_storm_lure(STORM_LURE_COST - 1, false, 40),
        "One SG short should not allow purchase"
    );
}

#[test]
fn test_cannot_purchase_storm_lure_zero_balance() {
    assert!(
        !spending::can_purchase_storm_lure(0, false, 40),
        "Zero balance should not allow purchase"
    );
}

#[test]
fn test_cannot_purchase_storm_lure_all_three_conditions_failing() {
    // All three conditions wrong: insufficient SG, already active, rank too low
    assert!(!spending::can_purchase_storm_lure(0, true, 1));
}

#[test]
fn test_can_purchase_storm_lure_well_above_cost() {
    assert!(spending::can_purchase_storm_lure(1_000_000, false, 40));
}

// ── Spending: chrono surge cost edge cases ───────────────────────────────

#[test]
fn test_chrono_surge_cost_first_option_is_cheapest() {
    let (ticks_0, cost_0, _) = spending::chrono_surge_cost(0).unwrap();
    let (ticks_3, cost_3, _) = spending::chrono_surge_cost(3).unwrap();
    assert!(cost_0 < cost_3, "First option cheapest");
    assert!(ticks_0 < ticks_3, "First option fewest ticks");
}

#[test]
fn test_chrono_surge_cost_labels_nonempty() {
    for i in 0..4 {
        let (_, _, label) = spending::chrono_surge_cost(i).unwrap();
        assert!(!label.is_empty(), "Option {} label should not be empty", i);
    }
}

// ── ExchangeUiState: phase transitions ──────────────────────────────────

#[test]
fn test_exchange_ui_set_phase_transition() {
    let mut ui = ExchangeUiState::new();
    assert_eq!(ui.phase, ExchangePhase::Menu);
    assert!(ui.previous_phase().is_none());

    ui.set_phase(ExchangePhase::InvokeTrialConfirm);
    assert_eq!(ui.phase, ExchangePhase::InvokeTrialConfirm);
    assert_eq!(ui.previous_phase(), Some(ExchangePhase::Menu));
}

#[test]
fn test_exchange_ui_set_same_phase_no_update() {
    let mut ui = ExchangeUiState::new();
    ui.set_phase(ExchangePhase::ChronoSurge);
    let prev = ui.previous_phase();

    // Setting the same phase again should not change previous_phase
    ui.set_phase(ExchangePhase::ChronoSurge);
    assert_eq!(ui.previous_phase(), prev);
    assert_eq!(ui.phase, ExchangePhase::ChronoSurge);
}

#[test]
fn test_exchange_ui_phase_sequence() {
    let mut ui = ExchangeUiState::new();

    // Simulate a typical sigil workflow
    ui.set_phase(ExchangePhase::SigilsList);
    assert_eq!(ui.previous_phase(), Some(ExchangePhase::Menu));

    ui.set_phase(ExchangePhase::SigilEtchConfirm);
    assert_eq!(ui.previous_phase(), Some(ExchangePhase::SigilsList));

    ui.set_phase(ExchangePhase::SigilRolling);
    assert_eq!(ui.previous_phase(), Some(ExchangePhase::SigilEtchConfirm));

    ui.set_phase(ExchangePhase::SigilPick);
    assert_eq!(ui.previous_phase(), Some(ExchangePhase::SigilRolling));

    ui.set_phase(ExchangePhase::SigilResult);
    assert_eq!(ui.previous_phase(), Some(ExchangePhase::SigilPick));
}

#[test]
fn test_exchange_ui_phase_elapsed_ms_is_some_after_open() {
    let mut ui = ExchangeUiState::new();
    ui.open();
    // After open(), phase_entered_at_ms should be set
    assert!(
        ui.phase_elapsed_ms().is_some(),
        "phase_elapsed_ms should be Some after open()"
    );
}

#[test]
fn test_exchange_ui_phase_elapsed_ms_is_none_before_open() {
    // Before open() or set_phase(), phase_entered_at_ms is not initialized,
    // so phase_elapsed_ms() returns None.
    let ui = ExchangeUiState::new();
    // new() doesn't call open() or set_phase(), so no timing data is set
    assert!(
        ui.phase_elapsed_ms().is_none(),
        "phase_elapsed_ms should be None before open() is called"
    );
}

#[test]
fn test_exchange_ui_open_resets_all_sigil_fields() {
    let mut ui = ExchangeUiState::new();
    // Set various sigil fields
    ui.sigil_selected_slot = 3;
    ui.sigil_pick_selected = 2;
    ui.sigil_target_slot = 4;
    ui.sigil_result = Some(Sigil {
        effect: SigilEffectType::XpPercent,
        value: 12.0,
        grade: SigilGrade::A,
    });
    ui.sigil_animation_start_ms = Some(99999);
    ui.sigil_animation_skipped = true;

    ui.open();

    assert_eq!(ui.sigil_selected_slot, 0);
    assert_eq!(ui.sigil_pick_selected, 0);
    assert_eq!(ui.sigil_target_slot, 0);
    assert!(ui.sigil_result.is_none());
    assert!(ui.sigil_animation_start_ms.is_none());
    assert!(!ui.sigil_animation_skipped);
    for choice in &ui.sigil_choices {
        assert!(choice.is_none());
    }
}

#[test]
fn test_exchange_ui_close_resets_all_sigil_fields() {
    let mut ui = ExchangeUiState::new();
    ui.sigil_selected_slot = 2;
    ui.sigil_pick_selected = 1;
    ui.sigil_target_slot = 3;
    ui.sigil_result = Some(Sigil {
        effect: SigilEffectType::DamagePercent,
        value: 10.0,
        grade: SigilGrade::B,
    });
    ui.sigil_animation_start_ms = Some(12345);
    ui.sigil_animation_skipped = true;

    ui.close();

    assert_eq!(ui.sigil_selected_slot, 0);
    assert_eq!(ui.sigil_pick_selected, 0);
    assert_eq!(ui.sigil_target_slot, 0);
    assert!(ui.sigil_result.is_none());
    assert!(ui.sigil_animation_start_ms.is_none());
    assert!(!ui.sigil_animation_skipped);
    for choice in &ui.sigil_choices {
        assert!(choice.is_none());
    }
}

// ── ChronoSurgeState: overcharge and acceleration ────────────────────────

#[test]
fn test_chrono_surge_state_overcharged_flag() {
    let normal = ChronoSurgeState::new(9_000, false);
    let overcharged = ChronoSurgeState::new(9_000, true);
    assert!(!normal.overcharged);
    assert!(overcharged.overcharged);
}

#[test]
fn test_chrono_surge_progress_at_quarter() {
    let mut surge = ChronoSurgeState::new(1000, false);
    surge.ticks_remaining = 750; // 25% consumed
    let p = surge.progress();
    assert!(
        (p - 0.25).abs() < 0.01,
        "25% consumed → progress=0.25, got {}",
        p
    );
}

#[test]
fn test_chrono_surge_progress_at_three_quarters() {
    let mut surge = ChronoSurgeState::new(1000, false);
    surge.ticks_remaining = 250; // 75% consumed
    let p = surge.progress();
    assert!(
        (p - 0.75).abs() < 0.01,
        "75% consumed → progress=0.75, got {}",
        p
    );
}

#[test]
fn test_chrono_surge_progress_zero_ticks_total() {
    // Edge: progress() with ticks_total=0 should return 1.0 (no divide by zero)
    let mut surge = ChronoSurgeState::new(1, false);
    surge.ticks_total = 0;
    surge.ticks_remaining = 0;
    assert!((surge.progress() - 1.0).abs() < 0.001);
}

#[test]
fn test_chrono_surge_speed_multiplier_increases_with_progress() {
    let mut surge = ChronoSurgeState::new(10000, false);
    let speed_start = surge.speed_multiplier();
    surge.ticks_remaining = 5000; // 50% done
    let speed_mid = surge.speed_multiplier();
    surge.ticks_remaining = 0; // 100% done
    let speed_end = surge.speed_multiplier();

    assert!(
        speed_start <= speed_mid,
        "Speed should increase: start={} mid={}",
        speed_start,
        speed_mid
    );
    assert!(
        speed_mid <= speed_end,
        "Speed should increase: mid={} end={}",
        speed_mid,
        speed_end
    );
}

#[test]
fn test_chrono_surge_current_batch_size_is_positive() {
    let surge = ChronoSurgeState::new(9_000, false);
    assert!(
        surge.current_batch_size() > 0,
        "Batch size should always be > 0"
    );
}

#[test]
fn test_chrono_surge_8hour_has_larger_batch_than_15min() {
    let min15 = ChronoSurgeState::new(9_000, false);
    let hr8 = ChronoSurgeState::new(288_000, false);
    assert!(
        hr8.batch_size > min15.batch_size,
        "8hr batch ({}) > 15min batch ({})",
        hr8.batch_size,
        min15.batch_size
    );
}

// ── StormSigils: slot unlock and etching mechanics ───────────────────────

#[test]
fn test_storm_sigils_slots_unlocked_starts_at_zero() {
    let sigils = StormSigils::new();
    assert_eq!(sigils.slots_unlocked, 0);
}

#[test]
fn test_storm_sigils_etched_count_with_locked_slots() {
    // Etching into slots beyond slots_unlocked (invalid game state, but etched_count
    // counts all Some values regardless of lock state)
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 1; // only slot 0 is unlocked
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::XpPercent,
        value: 10.0,
        grade: SigilGrade::C,
    });
    // Slots 1-4 are locked — normal game state won't etch here
    assert_eq!(sigils.etched_count(), 1);
}

#[test]
fn test_storm_sigils_etched_count_all_five_slots() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;
    for i in 0..5 {
        sigils.sigils[i] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 8.0,
            grade: SigilGrade::B,
        });
    }
    assert_eq!(sigils.etched_count(), 5);
}

#[test]
fn test_storm_sigils_next_unlock_cost_at_each_step() {
    let mut sigils = StormSigils::new();
    let expected_costs: [u64; 5] = [25_000, 50_000, 100_000, 200_000, 400_000];

    for (i, &expected) in expected_costs.iter().enumerate() {
        sigils.slots_unlocked = i as u8;
        assert_eq!(
            sigils.next_unlock_cost(),
            Some(expected),
            "At {} slots unlocked, next cost should be {}",
            i,
            expected
        );
    }
    // After all 5 are unlocked, no cost
    sigils.slots_unlocked = 5;
    assert_eq!(sigils.next_unlock_cost(), None);
}

#[test]
fn test_slot_unlock_costs_are_exponential_2x() {
    // Each cost should be exactly 2x the previous
    for i in 1..MAX_SIGIL_SLOTS {
        assert_eq!(
            SLOT_UNLOCK_COSTS[i],
            SLOT_UNLOCK_COSTS[i - 1] * 2,
            "SLOT_UNLOCK_COSTS[{}] should be 2x SLOT_UNLOCK_COSTS[{}]",
            i,
            i - 1
        );
    }
}

// ── SigilBonuses: all 12 effect types ────────────────────────────────────

#[test]
fn test_sigil_bonuses_all_12_effect_types() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;

    // We can only have 5 slots, so we test a subset, then re-test with others
    let first_five = [
        (SigilEffectType::XpPercent, 10.0),
        (SigilEffectType::DamagePercent, 8.0),
        (SigilEffectType::DamageReductionPercent, 3.0),
        (SigilEffectType::CritChancePercent, 5.0),
        (SigilEffectType::DropRatePercent, 7.0),
    ];

    for (i, (effect, value)) in first_five.iter().enumerate() {
        sigils.sigils[i] = Some(Sigil {
            effect: *effect,
            value: *value,
            grade: SigilGrade::C,
        });
    }

    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.xp_percent - 10.0).abs() < 1e-10);
    assert!((bonuses.damage_percent - 8.0).abs() < 1e-10);
    assert!((bonuses.damage_reduction_percent - 3.0).abs() < 1e-10);
    assert!((bonuses.crit_chance_percent - 5.0).abs() < 1e-10);
    assert!((bonuses.drop_rate_percent - 7.0).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_second_five_effect_types() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;

    let effects = [
        (SigilEffectType::MaxHpPercent, 12.0),
        (SigilEffectType::FishingSpeedPercent, 20.0),
        (SigilEffectType::OfflineXpPercent, 15.0),
        (SigilEffectType::AttackSpeedPercent, 9.0),
        (SigilEffectType::DoubleStrikePercent, 4.0),
    ];

    for (i, (effect, value)) in effects.iter().enumerate() {
        sigils.sigils[i] = Some(Sigil {
            effect: *effect,
            value: *value,
            grade: SigilGrade::B,
        });
    }

    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.max_hp_percent - 12.0).abs() < 1e-10);
    assert!((bonuses.fishing_speed_percent - 20.0).abs() < 1e-10);
    assert!((bonuses.offline_xp_percent - 15.0).abs() < 1e-10);
    assert!((bonuses.attack_speed_percent - 9.0).abs() < 1e-10);
    assert!((bonuses.double_strike_percent - 4.0).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_regen_delay_and_overcharge() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 2;
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::RegenDelayPercent,
        value: 8.0,
        grade: SigilGrade::A,
    });
    sigils.sigils[1] = Some(Sigil {
        effect: SigilEffectType::ChronoOverchargePercent,
        value: 18.0,
        grade: SigilGrade::S,
    });

    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.regen_delay_percent - 8.0).abs() < 1e-10);
    assert!((bonuses.chrono_overcharge_percent - 18.0).abs() < 1e-10);
    // All others should be zero
    assert!((bonuses.xp_percent).abs() < 1e-10);
    assert!((bonuses.damage_percent).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_multiple_same_effect_type_stacks() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;
    // All 5 slots with the same effect type
    for i in 0..5 {
        sigils.sigils[i] = Some(Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 4.0,
            grade: SigilGrade::C,
        });
    }

    let bonuses = SigilBonuses::compute(&sigils);
    assert!(
        (bonuses.crit_chance_percent - 20.0).abs() < 1e-10,
        "5x4% crit should total 20%, got {}",
        bonuses.crit_chance_percent
    );
    // All other effects should be zero
    assert!((bonuses.damage_percent).abs() < 1e-10);
    assert!((bonuses.xp_percent).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_no_slots_unlocked_all_none() {
    // Even with 0 slots unlocked, sigils vec is full of None — compute should return zeros
    let sigils = StormSigils::new();
    assert_eq!(sigils.slots_unlocked, 0);
    assert_eq!(sigils.sigils.len(), MAX_SIGIL_SLOTS);

    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.xp_percent).abs() < 1e-10);
    assert!((bonuses.damage_percent).abs() < 1e-10);
    assert!((bonuses.crit_chance_percent).abs() < 1e-10);
    assert!((bonuses.regen_delay_percent).abs() < 1e-10);
    assert!((bonuses.chrono_overcharge_percent).abs() < 1e-10);
}

// ── Sigil rolling: grade consistency with value ───────────────────────────

#[test]
fn test_roll_sigil_f_minus_grade_at_low_percentile() {
    // Percentile 0.01 should give F-
    let sigil = roll_sigil(SigilEffectType::XpPercent, 0.01);
    assert_eq!(sigil.grade, SigilGrade::FMinus);
}

#[test]
fn test_roll_sigil_s_plus_grade_at_high_percentile() {
    let sigil = roll_sigil(SigilEffectType::DamagePercent, 0.99);
    assert_eq!(sigil.grade, SigilGrade::SPlus);
}

#[test]
fn test_roll_sigil_c_grade_at_middle_percentile() {
    // 0.47 is the lower boundary of C grade
    let sigil = roll_sigil(SigilEffectType::CritChancePercent, 0.47);
    assert_eq!(sigil.grade, SigilGrade::C);
}

#[test]
fn test_roll_sigil_value_is_rounded_to_one_decimal_all_effects() {
    for effect in SigilEffectType::ALL {
        for &roll in &[0.1111, 0.3333, 0.5555, 0.7777, 0.9999] {
            let sigil = roll_sigil(effect, roll);
            let rounded = (sigil.value * 10.0).round() / 10.0;
            assert!(
                (sigil.value - rounded).abs() < 1e-9,
                "{:?} at roll={}: value {} should be rounded to 1 decimal",
                effect,
                roll,
                sigil.value
            );
        }
    }
}

#[test]
fn test_roll_sigil_grade_matches_percentile_at_boundary() {
    // Test exact grade boundaries
    let boundaries = [
        (0.0, SigilGrade::FMinus),
        (0.03, SigilGrade::F),
        (0.07, SigilGrade::FPlus),
        (0.10, SigilGrade::EMinus),
        (0.60, SigilGrade::BMinus),
        (0.85, SigilGrade::A),
        (0.985, SigilGrade::SPlus),
    ];
    for (percentile, expected_grade) in boundaries {
        let sigil = roll_sigil(SigilEffectType::XpPercent, percentile);
        assert_eq!(
            sigil.grade, expected_grade,
            "At percentile {} expected {:?}, got {:?}",
            percentile, expected_grade, sigil.grade
        );
    }
}

// ── Daily sigil pool: specific day values ───────────────────────────────

#[test]
fn test_daily_sigil_pool_at_day_zero() {
    // Day 0 should produce a valid pool
    let pool = daily_sigil_pool_for_day(0);
    assert_eq!(pool.len(), DAILY_POOL_SIZE);
    // No duplicates
    for i in 0..pool.len() {
        for j in (i + 1)..pool.len() {
            assert_ne!(pool[i], pool[j], "Day 0 has duplicate types");
        }
    }
}

#[test]
fn test_daily_sigil_pool_at_negative_day() {
    // Negative day (past days from epoch) should not panic
    let pool = daily_sigil_pool_for_day(-1);
    assert_eq!(pool.len(), DAILY_POOL_SIZE);
}

#[test]
fn test_daily_sigil_pool_at_large_day() {
    let pool = daily_sigil_pool_for_day(i32::MAX / 2);
    assert_eq!(pool.len(), DAILY_POOL_SIZE);
    // No duplicates
    for i in 0..pool.len() {
        for j in (i + 1)..pool.len() {
            assert_ne!(pool[i], pool[j], "Large day has duplicate types");
        }
    }
}

#[test]
fn test_daily_sigil_pool_contains_only_valid_effect_types() {
    for day in [0, 1, 100, 1000, 739000, 739001] {
        let pool = daily_sigil_pool_for_day(day);
        for effect in &pool {
            assert!(
                SigilEffectType::ALL.contains(effect),
                "Day {} pool contains invalid effect type {:?}",
                day,
                effect
            );
        }
    }
}

// ── generate_sigil_choices: pool with single type ────────────────────────

#[test]
fn test_generate_sigil_choices_single_effect_pool() {
    let pool = vec![SigilEffectType::DamagePercent];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let choices = generate_sigil_choices(&mut rng, &pool);
    assert_eq!(choices.len(), 3);
    for sigil in &choices {
        assert_eq!(
            sigil.effect,
            SigilEffectType::DamagePercent,
            "All choices should be DamagePercent from single-type pool"
        );
    }
}

#[test]
fn test_generate_sigil_choices_values_in_range() {
    let mut rng = ChaCha8Rng::seed_from_u64(77);
    for _ in 0..50 {
        let choices = generate_sigil_choices(&mut rng, &SigilEffectType::ALL);
        for sigil in &choices {
            let (min, max) = sigil.effect.range();
            assert!(
                sigil.value >= min && sigil.value <= max,
                "{:?} value {} out of range [{}, {}]",
                sigil.effect,
                sigil.value,
                min,
                max
            );
        }
    }
}

// ── SigilEffectType: display formatting edge cases ──────────────────────

#[test]
fn test_sigil_effect_format_value_regen_delay_shows_minus() {
    let formatted = SigilEffectType::RegenDelayPercent.format_value(5.0);
    assert!(
        formatted.starts_with('-'),
        "RegenDelay should show minus sign, got '{}'",
        formatted
    );
    assert!(
        formatted.contains("Regen Delay"),
        "Should contain 'Regen Delay', got '{}'",
        formatted
    );
}

#[test]
fn test_sigil_effect_format_value_all_non_regen_show_plus() {
    for effect in SigilEffectType::ALL {
        if effect == SigilEffectType::RegenDelayPercent {
            continue;
        }
        let formatted = effect.format_value(5.0);
        assert!(
            formatted.starts_with('+'),
            "{:?} should show + sign, got '{}'",
            effect,
            formatted
        );
    }
}

#[test]
fn test_sigil_effect_all_icons_nonempty() {
    for effect in SigilEffectType::ALL {
        assert!(!effect.icon().is_empty(), "{:?} has empty icon", effect);
    }
}

#[test]
fn test_sigil_effect_all_short_names_nonempty() {
    for effect in SigilEffectType::ALL {
        assert!(
            !effect.short_name().is_empty(),
            "{:?} has empty short name",
            effect
        );
    }
}

#[test]
fn test_sigil_effect_format_range_all_types() {
    for effect in SigilEffectType::ALL {
        let formatted = effect.format_range();
        assert!(!formatted.is_empty(), "{:?} format_range is empty", effect);
        assert!(
            formatted.contains('%'),
            "{:?} format_range missing '%': '{}'",
            effect,
            formatted
        );
    }
}

// ── P15+ gating: exact boundary tests ───────────────────────────────────

#[test]
fn test_stormglass_discovered_false_by_default() {
    let state = GameState::new("Test".to_string(), 0);
    assert!(!state.stormglass_discovered);
    assert_eq!(state.stormglass, 0);
}

#[test]
fn test_stormglass_state_initial_balance_zero() {
    let state = GameState::new("GapTest".to_string(), 0);
    assert_eq!(state.stormglass, 0u64);
}

// ── Sigil grade: complete ordering check ────────────────────────────────

#[test]
fn test_sigil_grade_full_ordering_f_through_s() {
    use quest::stormglass::sigils::SigilGrade::*;
    let ordered = [
        FMinus, F, FPlus, EMinus, E, EPlus, DMinus, D, DPlus, CMinus, C, CPlus, BMinus, B, BPlus,
        AMinus, A, APlus, SMinus, S, SPlus,
    ];
    for i in 1..ordered.len() {
        assert!(
            ordered[i] > ordered[i - 1],
            "{:?} should be greater than {:?}",
            ordered[i],
            ordered[i - 1]
        );
    }
}

// ── ETCH_COST constant verification ─────────────────────────────────────

#[test]
fn test_etch_cost_is_25000() {
    assert_eq!(ETCH_COST, 25_000);
}

#[test]
fn test_etch_cost_less_than_storm_lure_cost() {
    // Etch cost (25k) should be less than Storm Lure (50k)
    // Etch cost (25k) should be less than Storm Lure (50k) — verified at compile time
    const { assert!(ETCH_COST < STORM_LURE_COST) }
}

// ── Serde roundtrip: StormSigils with various configurations ────────────

#[test]
fn test_storm_sigils_serde_all_slots_empty() {
    let sigils = StormSigils::new();
    let json = serde_json::to_string(&sigils).unwrap();
    let loaded: StormSigils = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.slots_unlocked, 0);
    assert_eq!(loaded.etched_count(), 0);
    assert_eq!(loaded.sigils.len(), MAX_SIGIL_SLOTS);
}

#[test]
fn test_storm_sigils_serde_full_all_slots() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;
    let effects = [
        SigilEffectType::XpPercent,
        SigilEffectType::DamagePercent,
        SigilEffectType::CritChancePercent,
        SigilEffectType::RegenDelayPercent,
        SigilEffectType::ChronoOverchargePercent,
    ];
    for (i, effect) in effects.iter().enumerate() {
        sigils.sigils[i] = Some(Sigil {
            effect: *effect,
            value: (i + 1) as f64 * 2.5,
            grade: SigilGrade::B,
        });
    }

    let json = serde_json::to_string(&sigils).unwrap();
    let loaded: StormSigils = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.slots_unlocked, 5);
    assert_eq!(loaded.etched_count(), 5);

    for (i, effect) in effects.iter().enumerate() {
        let sigil = loaded.sigils[i].as_ref().unwrap();
        assert_eq!(sigil.effect, *effect);
        let expected_value = (i + 1) as f64 * 2.5;
        assert!(
            (sigil.value - expected_value).abs() < 1e-10,
            "Slot {} value mismatch: {} != {}",
            i,
            sigil.value,
            expected_value
        );
    }
}

#[test]
fn test_sigil_serde_all_grade_variants() {
    use quest::stormglass::sigils::SigilGrade::*;
    let grades = [
        FMinus, F, FPlus, EMinus, E, EPlus, DMinus, D, DPlus, CMinus, C, CPlus, BMinus, B, BPlus,
        AMinus, A, APlus, SMinus, S, SPlus,
    ];
    for grade in grades {
        let sigil = Sigil {
            effect: SigilEffectType::XpPercent,
            value: 10.0,
            grade,
        };
        let json = serde_json::to_string(&sigil).unwrap();
        let loaded: Sigil = serde_json::from_str(&json).unwrap();
        assert_eq!(
            loaded.grade, grade,
            "Grade {:?} failed serde roundtrip",
            grade
        );
    }
}

// ── Multiple sigil interactions: mixed bonuses ───────────────────────────

#[test]
fn test_multiple_different_sigils_independent_bonus_accumulation() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::XpPercent,
        value: 15.0,
        grade: SigilGrade::A,
    });
    sigils.sigils[1] = Some(Sigil {
        effect: SigilEffectType::DamagePercent,
        value: 8.0,
        grade: SigilGrade::B,
    });
    sigils.sigils[2] = Some(Sigil {
        effect: SigilEffectType::MaxHpPercent,
        value: 10.0,
        grade: SigilGrade::C,
    });
    sigils.sigils[3] = Some(Sigil {
        effect: SigilEffectType::FishingSpeedPercent,
        value: 20.0,
        grade: SigilGrade::S,
    });
    sigils.sigils[4] = Some(Sigil {
        effect: SigilEffectType::OfflineXpPercent,
        value: 12.0,
        grade: SigilGrade::B,
    });

    let bonuses = SigilBonuses::compute(&sigils);

    // Each effect type should accumulate independently
    assert!((bonuses.xp_percent - 15.0).abs() < 1e-10);
    assert!((bonuses.damage_percent - 8.0).abs() < 1e-10);
    assert!((bonuses.max_hp_percent - 10.0).abs() < 1e-10);
    assert!((bonuses.fishing_speed_percent - 20.0).abs() < 1e-10);
    assert!((bonuses.offline_xp_percent - 12.0).abs() < 1e-10);

    // Effects with no sigil should remain zero
    assert!((bonuses.crit_chance_percent).abs() < 1e-10);
    assert!((bonuses.damage_reduction_percent).abs() < 1e-10);
    assert!((bonuses.attack_speed_percent).abs() < 1e-10);
    assert!((bonuses.double_strike_percent).abs() < 1e-10);
    assert!((bonuses.regen_delay_percent).abs() < 1e-10);
    assert!((bonuses.chrono_overcharge_percent).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_partial_slots_etched() {
    // 3 slots unlocked, only 2 etched — third should not contribute
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 3;
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::XpPercent,
        value: 20.0,
        grade: SigilGrade::S,
    });
    sigils.sigils[1] = Some(Sigil {
        effect: SigilEffectType::XpPercent,
        value: 5.0,
        grade: SigilGrade::F,
    });
    // Slot 2 is unlocked but not etched (None)
    assert!(sigils.sigils[2].is_none());

    let bonuses = SigilBonuses::compute(&sigils);
    assert!(
        (bonuses.xp_percent - 25.0).abs() < 1e-10,
        "Only etched sigils should contribute: expected 25.0, got {}",
        bonuses.xp_percent
    );
}

// ── GameState: stormglass fields accessible ──────────────────────────────

#[test]
fn test_game_state_stormglass_fields() {
    let mut state = GameState::new("FieldTest".to_string(), 0);
    assert_eq!(state.stormglass, 0);
    assert!(!state.stormglass_discovered);

    // Manually set discovered and balance
    state.stormglass_discovered = true;
    state.stormglass = 5_000;
    assert_eq!(state.stormglass, 5_000);
    assert!(state.stormglass_discovered);
}

#[test]
fn test_game_state_storm_sigils_initial_state() {
    let state = GameState::new("SigilTest".to_string(), 0);
    assert_eq!(state.storm_sigils.slots_unlocked, 0);
    assert_eq!(state.storm_sigils.etched_count(), 0);
    assert_eq!(state.storm_sigils.sigils.len(), MAX_SIGIL_SLOTS);
}

#[test]
fn test_game_state_storm_sigils_can_be_modified() {
    let mut state = GameState::new("ModTest".to_string(), 0);
    state.storm_sigils.slots_unlocked = 2;
    state.storm_sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::DamagePercent,
        value: 10.0,
        grade: SigilGrade::A,
    });

    assert_eq!(state.storm_sigils.slots_unlocked, 2);
    assert_eq!(state.storm_sigils.etched_count(), 1);
    let bonuses = SigilBonuses::compute(&state.storm_sigils);
    assert!((bonuses.damage_percent - 10.0).abs() < 1e-10);
}
