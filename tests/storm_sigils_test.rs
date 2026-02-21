//! Integration tests for Storm Sigils core types, grading, and generation.

use quest::stormglass::sigils::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ── Effect Type Ranges ──────────────────────────────────────────────────

#[test]
fn test_all_effect_type_ranges_valid() {
    for effect in SigilEffectType::ALL {
        let (min, max) = effect.range();
        assert!(
            min < max,
            "{:?} range invalid: min={} >= max={}",
            effect,
            min,
            max
        );
        assert!(min > 0.0, "{:?} min must be positive", effect);
    }
}

#[test]
fn test_effect_type_all_has_11_variants() {
    assert_eq!(SigilEffectType::ALL.len(), 11);
}

// ── Grade Boundary Percentiles ──────────────────────────────────────────

#[test]
fn test_grade_from_percentile_all_21_boundaries() {
    // Test exact boundary values for each of the 21 grades
    assert_eq!(SigilGrade::from_percentile(0.00), SigilGrade::FMinus);
    assert_eq!(SigilGrade::from_percentile(0.029), SigilGrade::FMinus);
    assert_eq!(SigilGrade::from_percentile(0.03), SigilGrade::F);
    assert_eq!(SigilGrade::from_percentile(0.069), SigilGrade::F);
    assert_eq!(SigilGrade::from_percentile(0.07), SigilGrade::FPlus);
    assert_eq!(SigilGrade::from_percentile(0.099), SigilGrade::FPlus);
    assert_eq!(SigilGrade::from_percentile(0.10), SigilGrade::EMinus);
    assert_eq!(SigilGrade::from_percentile(0.129), SigilGrade::EMinus);
    assert_eq!(SigilGrade::from_percentile(0.13), SigilGrade::E);
    assert_eq!(SigilGrade::from_percentile(0.169), SigilGrade::E);
    assert_eq!(SigilGrade::from_percentile(0.17), SigilGrade::EPlus);
    assert_eq!(SigilGrade::from_percentile(0.199), SigilGrade::EPlus);
    assert_eq!(SigilGrade::from_percentile(0.20), SigilGrade::DMinus);
    assert_eq!(SigilGrade::from_percentile(0.269), SigilGrade::DMinus);
    assert_eq!(SigilGrade::from_percentile(0.27), SigilGrade::D);
    assert_eq!(SigilGrade::from_percentile(0.349), SigilGrade::D);
    assert_eq!(SigilGrade::from_percentile(0.35), SigilGrade::DPlus);
    assert_eq!(SigilGrade::from_percentile(0.399), SigilGrade::DPlus);
    assert_eq!(SigilGrade::from_percentile(0.40), SigilGrade::CMinus);
    assert_eq!(SigilGrade::from_percentile(0.469), SigilGrade::CMinus);
    assert_eq!(SigilGrade::from_percentile(0.47), SigilGrade::C);
    assert_eq!(SigilGrade::from_percentile(0.549), SigilGrade::C);
    assert_eq!(SigilGrade::from_percentile(0.55), SigilGrade::CPlus);
    assert_eq!(SigilGrade::from_percentile(0.599), SigilGrade::CPlus);
    assert_eq!(SigilGrade::from_percentile(0.60), SigilGrade::BMinus);
    assert_eq!(SigilGrade::from_percentile(0.669), SigilGrade::BMinus);
    assert_eq!(SigilGrade::from_percentile(0.67), SigilGrade::B);
    assert_eq!(SigilGrade::from_percentile(0.749), SigilGrade::B);
    assert_eq!(SigilGrade::from_percentile(0.75), SigilGrade::BPlus);
    assert_eq!(SigilGrade::from_percentile(0.799), SigilGrade::BPlus);
    assert_eq!(SigilGrade::from_percentile(0.80), SigilGrade::AMinus);
    assert_eq!(SigilGrade::from_percentile(0.849), SigilGrade::AMinus);
    assert_eq!(SigilGrade::from_percentile(0.85), SigilGrade::A);
    assert_eq!(SigilGrade::from_percentile(0.919), SigilGrade::A);
    assert_eq!(SigilGrade::from_percentile(0.92), SigilGrade::APlus);
    assert_eq!(SigilGrade::from_percentile(0.949), SigilGrade::APlus);
    assert_eq!(SigilGrade::from_percentile(0.95), SigilGrade::SMinus);
    assert_eq!(SigilGrade::from_percentile(0.959), SigilGrade::SMinus);
    assert_eq!(SigilGrade::from_percentile(0.96), SigilGrade::S);
    assert_eq!(SigilGrade::from_percentile(0.984), SigilGrade::S);
    assert_eq!(SigilGrade::from_percentile(0.985), SigilGrade::SPlus);
    assert_eq!(SigilGrade::from_percentile(1.0), SigilGrade::SPlus);
}

// ── Exponential Curve ───────────────────────────────────────────────────

#[test]
fn test_exponential_value_at_zero_is_zero() {
    let val = exponential_value(0.0);
    assert!(val.abs() < 1e-10, "f(0) should be 0, got {}", val);
}

#[test]
fn test_exponential_value_at_one_is_one() {
    let val = exponential_value(1.0);
    assert!((val - 1.0).abs() < 1e-10, "f(1) should be 1, got {}", val);
}

#[test]
fn test_exponential_value_monotonically_increasing() {
    let mut prev = exponential_value(0.0);
    for i in 1..=1000 {
        let p = i as f64 / 1000.0;
        let val = exponential_value(p);
        assert!(
            val >= prev,
            "exponential_value not monotonic at p={}: {} < {}",
            p,
            val,
            prev
        );
        prev = val;
    }
}

#[test]
fn test_exponential_value_compresses_low_end() {
    // At p=0.5, the curved value should be less than 0.5 (compressed)
    let val = exponential_value(0.5);
    assert!(
        val < 0.5,
        "exponential curve should compress low end: f(0.5)={} should be < 0.5",
        val
    );
}

// ── Roll Values Within Range ────────────────────────────────────────────

#[test]
fn test_roll_sigil_values_within_range_for_all_effects() {
    for effect in SigilEffectType::ALL {
        let (min, max) = effect.range();
        // Test at various percentile points
        for &roll in &[0.0, 0.001, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99, 0.999] {
            let sigil = roll_sigil(effect, roll);
            assert!(
                sigil.value >= min && sigil.value <= max,
                "{:?} at roll={}: value {} not in [{}, {}]",
                effect,
                roll,
                sigil.value,
                min,
                max
            );
            assert_eq!(sigil.effect, effect);
        }
    }
}

#[test]
fn test_roll_sigil_value_rounded_to_one_decimal() {
    for effect in SigilEffectType::ALL {
        let sigil = roll_sigil(effect, 0.3456);
        let rounded = (sigil.value * 10.0).round() / 10.0;
        assert!(
            (sigil.value - rounded).abs() < 1e-10,
            "{:?}: value {} not rounded to 1 decimal",
            effect,
            sigil.value
        );
    }
}

// ── Generate Sigil Choices ──────────────────────────────────────────────

#[test]
fn test_generate_sigil_choices_produces_three() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let choices = generate_sigil_choices(&mut rng);
    assert_eq!(choices.len(), 3);
}

#[test]
fn test_generate_sigil_choices_all_valid() {
    let mut rng = ChaCha8Rng::seed_from_u64(123);
    for _ in 0..100 {
        let choices = generate_sigil_choices(&mut rng);
        for sigil in &choices {
            let (min, max) = sigil.effect.range();
            assert!(
                sigil.value >= min && sigil.value <= max,
                "Generated sigil {:?} value {} out of range [{}, {}]",
                sigil.effect,
                sigil.value,
                min,
                max
            );
        }
    }
}

#[test]
fn test_generate_sigil_choices_deterministic_with_seed() {
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let choices1 = generate_sigil_choices(&mut rng1);
    let choices2 = generate_sigil_choices(&mut rng2);
    for (a, b) in choices1.iter().zip(choices2.iter()) {
        assert_eq!(a.effect, b.effect);
        assert!((a.value - b.value).abs() < 1e-10);
        assert_eq!(a.grade, b.grade);
    }
}

// ── SigilBonuses Aggregation ────────────────────────────────────────────

#[test]
fn test_sigil_bonuses_sums_same_effect() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 5;
    for i in 0..5 {
        sigils.sigils[i] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 10.0,
            grade: SigilGrade::B,
        });
    }
    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.xp_percent - 50.0).abs() < 1e-10);
    // All other effects should be zero
    assert!((bonuses.damage_percent).abs() < 1e-10);
    assert!((bonuses.crit_chance_percent).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_different_effects() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 3;
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::DamagePercent,
        value: 8.5,
        grade: SigilGrade::C,
    });
    sigils.sigils[1] = Some(Sigil {
        effect: SigilEffectType::CritChancePercent,
        value: 4.0,
        grade: SigilGrade::B,
    });
    sigils.sigils[2] = Some(Sigil {
        effect: SigilEffectType::RegenDelayPercent,
        value: 7.0,
        grade: SigilGrade::A,
    });

    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.damage_percent - 8.5).abs() < 1e-10);
    assert!((bonuses.crit_chance_percent - 4.0).abs() < 1e-10);
    assert!((bonuses.regen_delay_percent - 7.0).abs() < 1e-10);
}

#[test]
fn test_sigil_bonuses_empty_returns_zeros() {
    let sigils = StormSigils::new();
    let bonuses = SigilBonuses::compute(&sigils);
    assert!((bonuses.xp_percent).abs() < 1e-10);
    assert!((bonuses.damage_percent).abs() < 1e-10);
    assert!((bonuses.damage_reduction_percent).abs() < 1e-10);
    assert!((bonuses.crit_chance_percent).abs() < 1e-10);
    assert!((bonuses.drop_rate_percent).abs() < 1e-10);
    assert!((bonuses.max_hp_percent).abs() < 1e-10);
    assert!((bonuses.fishing_speed_percent).abs() < 1e-10);
    assert!((bonuses.offline_xp_percent).abs() < 1e-10);
    assert!((bonuses.attack_speed_percent).abs() < 1e-10);
    assert!((bonuses.double_strike_percent).abs() < 1e-10);
    assert!((bonuses.regen_delay_percent).abs() < 1e-10);
}

// ── StormSigils::new() ─────────────────────────────────────────────────

#[test]
fn test_storm_sigils_new_has_zero_slots_unlocked() {
    let sigils = StormSigils::new();
    assert_eq!(sigils.slots_unlocked, 0);
    assert_eq!(sigils.sigils.len(), MAX_SIGIL_SLOTS);
    assert_eq!(sigils.inscribed_count(), 0);
    for slot in &sigils.sigils {
        assert!(slot.is_none());
    }
}

#[test]
fn test_storm_sigils_default_matches_new() {
    let from_new = StormSigils::new();
    let from_default = StormSigils::default();
    assert_eq!(from_new.slots_unlocked, from_default.slots_unlocked);
    assert_eq!(from_new.sigils.len(), from_default.sigils.len());
    assert_eq!(from_new.inscribed_count(), from_default.inscribed_count());
}

// ── Grade Labels ────────────────────────────────────────────────────────

#[test]
fn test_all_21_grade_labels() {
    let expected = [
        (SigilGrade::FMinus, "F-"),
        (SigilGrade::F, "F"),
        (SigilGrade::FPlus, "F+"),
        (SigilGrade::EMinus, "E-"),
        (SigilGrade::E, "E"),
        (SigilGrade::EPlus, "E+"),
        (SigilGrade::DMinus, "D-"),
        (SigilGrade::D, "D"),
        (SigilGrade::DPlus, "D+"),
        (SigilGrade::CMinus, "C-"),
        (SigilGrade::C, "C"),
        (SigilGrade::CPlus, "C+"),
        (SigilGrade::BMinus, "B-"),
        (SigilGrade::B, "B"),
        (SigilGrade::BPlus, "B+"),
        (SigilGrade::AMinus, "A-"),
        (SigilGrade::A, "A"),
        (SigilGrade::APlus, "A+"),
        (SigilGrade::SMinus, "S-"),
        (SigilGrade::S, "S"),
        (SigilGrade::SPlus, "S+"),
    ];
    for (grade, label) in &expected {
        assert_eq!(grade.label(), *label, "{:?} label mismatch", grade);
    }
}

// ── Plus/Minus Detection ────────────────────────────────────────────────

#[test]
fn test_grade_is_plus_for_all_plus_variants() {
    let plus_grades = [
        SigilGrade::FPlus,
        SigilGrade::EPlus,
        SigilGrade::DPlus,
        SigilGrade::CPlus,
        SigilGrade::BPlus,
        SigilGrade::APlus,
        SigilGrade::SPlus,
    ];
    for grade in &plus_grades {
        assert!(grade.is_plus(), "{:?} should be plus", grade);
        assert!(!grade.is_minus(), "{:?} should not be minus", grade);
    }
}

#[test]
fn test_grade_is_minus_for_all_minus_variants() {
    let minus_grades = [
        SigilGrade::FMinus,
        SigilGrade::EMinus,
        SigilGrade::DMinus,
        SigilGrade::CMinus,
        SigilGrade::BMinus,
        SigilGrade::AMinus,
        SigilGrade::SMinus,
    ];
    for grade in &minus_grades {
        assert!(grade.is_minus(), "{:?} should be minus", grade);
        assert!(!grade.is_plus(), "{:?} should not be plus", grade);
    }
}

#[test]
fn test_grade_base_is_neither_plus_nor_minus() {
    let base_grades = [
        SigilGrade::F,
        SigilGrade::E,
        SigilGrade::D,
        SigilGrade::C,
        SigilGrade::B,
        SigilGrade::A,
        SigilGrade::S,
    ];
    for grade in &base_grades {
        assert!(!grade.is_plus(), "{:?} base should not be plus", grade);
        assert!(!grade.is_minus(), "{:?} base should not be minus", grade);
    }
}

// ── Slot Unlock Costs ───────────────────────────────────────────────────

#[test]
fn test_slot_unlock_costs_match_design() {
    assert_eq!(
        SLOT_UNLOCK_COSTS,
        [25_000, 50_000, 100_000, 200_000, 400_000]
    );
}

#[test]
fn test_next_unlock_cost_progression() {
    let mut sigils = StormSigils::new();

    // 0 slots unlocked → next costs 25k (index 0)
    assert_eq!(sigils.next_unlock_cost(), Some(25_000));

    sigils.slots_unlocked = 1;
    assert_eq!(sigils.next_unlock_cost(), Some(50_000));

    sigils.slots_unlocked = 2;
    assert_eq!(sigils.next_unlock_cost(), Some(100_000));

    sigils.slots_unlocked = 3;
    assert_eq!(sigils.next_unlock_cost(), Some(200_000));

    sigils.slots_unlocked = 4;
    assert_eq!(sigils.next_unlock_cost(), Some(400_000));

    sigils.slots_unlocked = 5;
    assert_eq!(sigils.next_unlock_cost(), None); // All unlocked
}

// ── Tier Letter ─────────────────────────────────────────────────────────

#[test]
fn test_grade_tier_letters() {
    assert_eq!(SigilGrade::SPlus.tier_letter(), 'S');
    assert_eq!(SigilGrade::S.tier_letter(), 'S');
    assert_eq!(SigilGrade::SMinus.tier_letter(), 'S');
    assert_eq!(SigilGrade::APlus.tier_letter(), 'A');
    assert_eq!(SigilGrade::A.tier_letter(), 'A');
    assert_eq!(SigilGrade::AMinus.tier_letter(), 'A');
    assert_eq!(SigilGrade::BPlus.tier_letter(), 'B');
    assert_eq!(SigilGrade::B.tier_letter(), 'B');
    assert_eq!(SigilGrade::BMinus.tier_letter(), 'B');
    assert_eq!(SigilGrade::CPlus.tier_letter(), 'C');
    assert_eq!(SigilGrade::C.tier_letter(), 'C');
    assert_eq!(SigilGrade::CMinus.tier_letter(), 'C');
    assert_eq!(SigilGrade::DPlus.tier_letter(), 'D');
    assert_eq!(SigilGrade::D.tier_letter(), 'D');
    assert_eq!(SigilGrade::DMinus.tier_letter(), 'D');
    assert_eq!(SigilGrade::EPlus.tier_letter(), 'E');
    assert_eq!(SigilGrade::E.tier_letter(), 'E');
    assert_eq!(SigilGrade::EMinus.tier_letter(), 'E');
    assert_eq!(SigilGrade::FPlus.tier_letter(), 'F');
    assert_eq!(SigilGrade::F.tier_letter(), 'F');
    assert_eq!(SigilGrade::FMinus.tier_letter(), 'F');
}

// ── Serde ───────────────────────────────────────────────────────────────

#[test]
fn test_sigil_serde_round_trip() {
    let sigil = Sigil {
        effect: SigilEffectType::AttackSpeedPercent,
        value: 7.3,
        grade: SigilGrade::APlus,
    };
    let json = serde_json::to_string(&sigil).unwrap();
    let loaded: Sigil = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.effect, SigilEffectType::AttackSpeedPercent);
    assert!((loaded.value - 7.3).abs() < 1e-10);
    assert_eq!(loaded.grade, SigilGrade::APlus);
}

#[test]
fn test_storm_sigils_serde_round_trip() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 3;
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::XpPercent,
        value: 20.0,
        grade: SigilGrade::S,
    });
    sigils.sigils[1] = Some(Sigil {
        effect: SigilEffectType::DoubleStrikePercent,
        value: 3.2,
        grade: SigilGrade::BPlus,
    });

    let json = serde_json::to_string(&sigils).unwrap();
    let loaded: StormSigils = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.slots_unlocked, 3);
    assert_eq!(loaded.inscribed_count(), 2);
    assert!(loaded.sigils[0].is_some());
    assert!(loaded.sigils[1].is_some());
    assert!(loaded.sigils[2].is_none());
}

// ── Display Formatting ──────────────────────────────────────────────────

#[test]
fn test_sigil_effect_format_value_all_types() {
    assert_eq!(SigilEffectType::XpPercent.format_value(12.3), "+12.3% XP");
    assert_eq!(
        SigilEffectType::DamagePercent.format_value(8.0),
        "+8.0% Damage"
    );
    assert_eq!(
        SigilEffectType::DamageReductionPercent.format_value(3.4),
        "+3.4% DR"
    );
    assert_eq!(
        SigilEffectType::CritChancePercent.format_value(5.8),
        "+5.8% Crit"
    );
    assert_eq!(
        SigilEffectType::DropRatePercent.format_value(6.1),
        "+6.1% Drop Rate"
    );
    assert_eq!(
        SigilEffectType::MaxHpPercent.format_value(11.3),
        "+11.3% HP"
    );
    assert_eq!(
        SigilEffectType::FishingSpeedPercent.format_value(17.4),
        "+17.4% Fishing Speed"
    );
    assert_eq!(
        SigilEffectType::OfflineXpPercent.format_value(14.7),
        "+14.7% Offline XP"
    );
    assert_eq!(
        SigilEffectType::AttackSpeedPercent.format_value(7.1),
        "+7.1% ASPD"
    );
    assert_eq!(
        SigilEffectType::DoubleStrikePercent.format_value(3.6),
        "+3.6% Double Strike"
    );
    assert_eq!(
        SigilEffectType::RegenDelayPercent.format_value(6.8),
        "-6.8% Regen Delay"
    );
}

#[test]
fn test_sigil_effect_sigil_names() {
    assert_eq!(SigilEffectType::XpPercent.sigil_name(), "Sigil of Wisdom");
    assert_eq!(SigilEffectType::DamagePercent.sigil_name(), "Sigil of Fury");
    assert_eq!(
        SigilEffectType::DamageReductionPercent.sigil_name(),
        "Sigil of the Bulwark"
    );
    assert_eq!(
        SigilEffectType::CritChancePercent.sigil_name(),
        "Sigil of Precision"
    );
    assert_eq!(
        SigilEffectType::DropRatePercent.sigil_name(),
        "Sigil of Fortune"
    );
    assert_eq!(
        SigilEffectType::MaxHpPercent.sigil_name(),
        "Sigil of Vitality"
    );
    assert_eq!(
        SigilEffectType::FishingSpeedPercent.sigil_name(),
        "Sigil of the Tide"
    );
    assert_eq!(
        SigilEffectType::OfflineXpPercent.sigil_name(),
        "Sigil of Echoes"
    );
    assert_eq!(
        SigilEffectType::AttackSpeedPercent.sigil_name(),
        "Sigil of Swiftness"
    );
    assert_eq!(
        SigilEffectType::DoubleStrikePercent.sigil_name(),
        "Sigil of the Twin Strike"
    );
    assert_eq!(
        SigilEffectType::RegenDelayPercent.sigil_name(),
        "Sigil of Renewal"
    );
}

// ── Grade Ordering ──────────────────────────────────────────────────────

#[test]
fn test_grade_ordering_f_through_s() {
    let grades = [
        SigilGrade::FMinus,
        SigilGrade::F,
        SigilGrade::FPlus,
        SigilGrade::EMinus,
        SigilGrade::E,
        SigilGrade::EPlus,
        SigilGrade::DMinus,
        SigilGrade::D,
        SigilGrade::DPlus,
        SigilGrade::CMinus,
        SigilGrade::C,
        SigilGrade::CPlus,
        SigilGrade::BMinus,
        SigilGrade::B,
        SigilGrade::BPlus,
        SigilGrade::AMinus,
        SigilGrade::A,
        SigilGrade::APlus,
        SigilGrade::SMinus,
        SigilGrade::S,
        SigilGrade::SPlus,
    ];
    for i in 1..grades.len() {
        assert!(
            grades[i] > grades[i - 1],
            "{:?} should be > {:?}",
            grades[i],
            grades[i - 1]
        );
    }
}

// ── Constants ───────────────────────────────────────────────────────────

#[test]
fn test_inscribe_cost() {
    assert_eq!(INSCRIBE_COST, 25_000);
}

#[test]
fn test_max_sigil_slots() {
    assert_eq!(MAX_SIGIL_SLOTS, 5);
}
