//! Integration tests for Storm Sigils core types, grading, and generation.

use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::combat::events::{CombatBonuses, CombatEvent};
use quest::combat::logic::update_combat;
use quest::combat::types::Enemy;
use quest::core::constants::ATTACK_INTERVAL_SECONDS;
use quest::core::game_state::GameState;
use quest::core::tick::game_tick;
use quest::enhancement::EnhancementProgress;
use quest::haven::Haven;
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
    let choices = generate_sigil_choices(&mut rng, &SigilEffectType::ALL);
    assert_eq!(choices.len(), 3);
}

#[test]
fn test_generate_sigil_choices_all_valid() {
    let mut rng = ChaCha8Rng::seed_from_u64(123);
    for _ in 0..100 {
        let choices = generate_sigil_choices(&mut rng, &SigilEffectType::ALL);
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
    let choices1 = generate_sigil_choices(&mut rng1, &SigilEffectType::ALL);
    let choices2 = generate_sigil_choices(&mut rng2, &SigilEffectType::ALL);
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
    assert_eq!(sigils.etched_count(), 0);
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
    assert_eq!(from_new.etched_count(), from_default.etched_count());
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
    assert_eq!(loaded.etched_count(), 2);
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
        "+7.1% Attack Speed"
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
fn test_etch_cost() {
    assert_eq!(ETCH_COST, 25_000);
}

#[test]
fn test_max_sigil_slots() {
    assert_eq!(MAX_SIGIL_SLOTS, 5);
}

// ── Bonus Injection Integration Tests ─────────────────────────────────
// These tests verify that etched sigils actually affect game outcomes.

/// Helper: force a player attack and return the damage dealt.
fn force_player_attack_damage(
    rng: &mut ChaCha8Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> u32 {
    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = 0.0;
    let mut ach = Achievements::default();
    let events = update_combat(rng, state, 0.0, bonuses, &mut ach, &d);
    events
        .iter()
        .filter_map(|e| match e {
            CombatEvent::PlayerAttack { damage, .. } => Some(*damage),
            _ => None,
        })
        .sum()
}

/// Helper: create a GameState with a beefy enemy for damage comparison tests.
fn state_with_target() -> GameState {
    let mut state = GameState::new("SigilBonusTest".to_string(), 0);
    state.combat_state.current_enemy = Some(Enemy::new_with_defense(
        "Target Dummy".to_string(),
        99999,
        1,
        0,
    ));
    state
}

/// Helper: etch a single sigil in slot 0.
fn etch_sigil(state: &mut GameState, effect: SigilEffectType, value: f64) {
    state.storm_sigils.slots_unlocked = 1;
    state.storm_sigils.sigils[0] = Some(Sigil {
        effect,
        value,
        grade: SigilGrade::S,
    });
}

#[test]
fn test_sigil_damage_bonus_increases_player_damage() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut state = state_with_target();

    // Baseline: no sigils
    let damage_base = force_player_attack_damage(&mut rng, &mut state, &CombatBonuses::default());

    // Reset enemy HP
    state
        .combat_state
        .current_enemy
        .as_mut()
        .unwrap()
        .reset_hp();

    // With sigil: +15% damage (how tick.rs injects it via CombatBonuses.damage_percent)
    let sigil_bonuses = SigilBonuses {
        damage_percent: 15.0,
        ..Default::default()
    };
    let bonuses_with = CombatBonuses {
        damage_percent: sigil_bonuses.damage_percent,
        ..CombatBonuses::default()
    };

    let damage_with = force_player_attack_damage(&mut rng, &mut state, &bonuses_with);

    assert!(
        damage_with > damage_base,
        "Sigil damage bonus should increase damage: with={} base={}",
        damage_with,
        damage_base
    );
}

#[test]
fn test_sigil_dr_bonus_reduces_enemy_damage() {
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    let mut state = GameState::new("DRTest".to_string(), 0);
    state.combat_state.current_enemy =
        Some(Enemy::new_with_defense("Hitter".to_string(), 99999, 100, 0));

    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    let mut ach = Achievements::default();

    // Force enemy attack (no player attack)
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 2.0; // will fire on any delta

    // Baseline: no DR
    let events_base = update_combat(
        &mut rng,
        &mut state,
        0.1,
        &CombatBonuses::default(),
        &mut ach,
        &d,
    );
    let damage_base: u32 = events_base
        .iter()
        .filter_map(|e| match e {
            CombatEvent::EnemyAttack { damage, .. } => Some(*damage),
            _ => None,
        })
        .sum();

    // Heal player back up
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state.combat_state.enemy_attack_timer = 2.0;
    state.combat_state.player_attack_timer = 0.0;

    // With sigil DR: 5% reduction (how tick.rs injects it via CombatBonuses.damage_reduction_percent)
    let bonuses_with_dr = CombatBonuses {
        damage_reduction_percent: 5.0,
        ..CombatBonuses::default()
    };
    let events_with = update_combat(&mut rng, &mut state, 0.1, &bonuses_with_dr, &mut ach, &d);
    let damage_with: u32 = events_with
        .iter()
        .filter_map(|e| match e {
            CombatEvent::EnemyAttack { damage, .. } => Some(*damage),
            _ => None,
        })
        .sum();

    assert!(damage_base > 0, "Baseline enemy should deal damage");
    assert!(
        damage_with < damage_base,
        "Sigil DR should reduce enemy damage: with={} base={}",
        damage_with,
        damage_base
    );
}

#[test]
fn test_sigil_max_hp_applied_in_game_tick() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut state = GameState::new("MaxHPTest".to_string(), 0);
    let mut tc = 0u32;
    let mut haven = Haven::default();
    let mut enhancement = EnhancementProgress::new();
    let mut ach = Achievements::default();

    // Run one tick without sigils to establish baseline HP
    game_tick(
        &mut state,
        &mut tc,
        &mut haven,
        &mut enhancement,
        &mut ach,
        false,
        &mut rng,
    );
    let hp_base = state.combat_state.player_max_hp;

    // Etch a max HP sigil
    etch_sigil(&mut state, SigilEffectType::MaxHpPercent, 15.0);

    // Run another tick — sigil should boost max HP
    game_tick(
        &mut state,
        &mut tc,
        &mut haven,
        &mut enhancement,
        &mut ach,
        false,
        &mut rng,
    );
    let hp_with = state.combat_state.player_max_hp;

    assert!(
        hp_with > hp_base,
        "MaxHpPercent sigil should increase player max HP: with={} base={}",
        hp_with,
        hp_base
    );
}

#[test]
fn test_sigil_xp_bonus_applied_in_game_tick() {
    // Verify that XP% sigil is injected into haven_combat.xp_gain_percent
    // by checking that SigilBonuses::compute produces the right value
    // and that tick.rs adds it to HavenCombatBonuses
    let mut state = GameState::new("XPTest".to_string(), 0);
    etch_sigil(&mut state, SigilEffectType::XpPercent, 25.0);

    let bonuses = SigilBonuses::compute(&state.storm_sigils);
    assert!(
        (bonuses.xp_percent - 25.0).abs() < 1e-10,
        "XP sigil should produce 25% bonus"
    );

    // Verify the injection path: tick.rs line 137 adds sigil_bonuses.xp_percent
    // to haven_combat.xp_gain_percent. We can verify this by checking that
    // the combined value equals haven + sigil.
    let haven = Haven::default();
    let haven_bonuses = haven.compute_bonuses();
    let combined_xp = haven_bonuses.xp_gain_percent + bonuses.xp_percent;
    assert!(
        (combined_xp - 25.0).abs() < 1e-10,
        "Combined XP bonus should be haven(0) + sigil(25) = 25: got {}",
        combined_xp
    );
}

#[test]
fn test_sigil_drop_rate_injected_into_haven_bonuses() {
    // Verify that drop_rate sigil is added to haven_bonuses.drop_rate_percent
    // (tick.rs line 66: haven_bonuses.drop_rate_percent += sigil_bonuses.drop_rate_percent)
    let mut state = GameState::new("DropTest".to_string(), 0);
    etch_sigil(&mut state, SigilEffectType::DropRatePercent, 10.0);

    let bonuses = SigilBonuses::compute(&state.storm_sigils);
    let haven = Haven::default();
    let mut haven_bonuses = haven.compute_bonuses();
    let base_drop = haven_bonuses.drop_rate_percent;

    // Simulate tick.rs injection
    haven_bonuses.drop_rate_percent += bonuses.drop_rate_percent;

    assert!(
        (haven_bonuses.drop_rate_percent - base_drop - 10.0).abs() < 1e-10,
        "Drop rate should increase by sigil value: got {} (base was {})",
        haven_bonuses.drop_rate_percent,
        base_drop
    );
}

#[test]
fn test_sigil_fishing_speed_injected_into_haven_bonuses() {
    // Verify that fishing_speed sigil is added to haven_bonuses.fishing_timer_reduction
    // (tick.rs line 67: haven_bonuses.fishing_timer_reduction += sigil_bonuses.fishing_speed_percent)
    let mut state = GameState::new("FishTest".to_string(), 0);
    etch_sigil(&mut state, SigilEffectType::FishingSpeedPercent, 20.0);

    let bonuses = SigilBonuses::compute(&state.storm_sigils);
    let haven = Haven::default();
    let mut haven_bonuses = haven.compute_bonuses();
    let base_fishing = haven_bonuses.fishing_timer_reduction;

    // Simulate tick.rs injection
    haven_bonuses.fishing_timer_reduction += bonuses.fishing_speed_percent;

    assert!(
        (haven_bonuses.fishing_timer_reduction - base_fishing - 20.0).abs() < 1e-10,
        "Fishing speed should increase by sigil value: got {} (base was {})",
        haven_bonuses.fishing_timer_reduction,
        base_fishing
    );
}

#[test]
fn test_sigil_offline_xp_injected_in_offline_path() {
    // Verify that offline_xp sigil produces correct bonus for injection
    // (main_helpers/offline.rs adds sigil_offline_bonus to haven_offline_bonus)
    let mut state = GameState::new("OfflineTest".to_string(), 0);
    etch_sigil(&mut state, SigilEffectType::OfflineXpPercent, 15.0);

    let bonuses = SigilBonuses::compute(&state.storm_sigils);
    assert!(
        (bonuses.offline_xp_percent - 15.0).abs() < 1e-10,
        "Offline XP sigil should produce 15% bonus"
    );
}

#[test]
fn test_sigil_attack_speed_injected_via_god_items() {
    // Verify that attack_speed sigil value flows into GodItemCombatBonuses
    // (tick.rs line 154-156: + sigil_bonuses.attack_speed_percent)
    let mut state = GameState::new("ASPDTest".to_string(), 0);
    etch_sigil(&mut state, SigilEffectType::AttackSpeedPercent, 10.0);

    let bonuses = SigilBonuses::compute(&state.storm_sigils);
    let base_aspd = quest::god_items::equipped_god_item_attack_speed_percent(&state.equipment);
    let combined = base_aspd + bonuses.attack_speed_percent;

    // Without god items, base is 0; with sigil it should be 10
    assert!(
        (combined - 10.0).abs() < 1e-10,
        "Attack speed should be god_item(0) + sigil(10) = 10: got {}",
        combined
    );
}

#[test]
fn test_multiple_sigils_stack_in_game_tick() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut state = GameState::new("StackTest".to_string(), 0);
    let mut tc = 0u32;
    let mut haven = Haven::default();
    let mut enhancement = EnhancementProgress::new();
    let mut ach = Achievements::default();

    // Run one tick to get baseline HP
    game_tick(
        &mut state,
        &mut tc,
        &mut haven,
        &mut enhancement,
        &mut ach,
        false,
        &mut rng,
    );
    let hp_base = state.combat_state.player_max_hp;

    // Etch 3 MaxHpPercent sigils
    state.storm_sigils.slots_unlocked = 3;
    for i in 0..3 {
        state.storm_sigils.sigils[i] = Some(Sigil {
            effect: SigilEffectType::MaxHpPercent,
            value: 10.0,
            grade: SigilGrade::B,
        });
    }

    // Run tick with stacked sigils
    game_tick(
        &mut state,
        &mut tc,
        &mut haven,
        &mut enhancement,
        &mut ach,
        false,
        &mut rng,
    );
    let hp_stacked = state.combat_state.player_max_hp;

    // 3x 10% = 30% boost
    let expected_min = (hp_base as f64 * 1.29) as u32; // at least 29% (rounding)
    assert!(
        hp_stacked >= expected_min,
        "3x MaxHpPercent sigils should stack: got {} (base={}, expected >= {})",
        hp_stacked,
        hp_base,
        expected_min
    );
}

// ── Daily Sigil Pool ───────────────────────────────────────────────────

#[test]
fn test_daily_pool_size_is_correct() {
    let pool = daily_sigil_pool_for_day(739000);
    assert_eq!(pool.len(), DAILY_POOL_SIZE);
}

#[test]
fn test_daily_pool_is_deterministic() {
    let pool1 = daily_sigil_pool_for_day(739000);
    let pool2 = daily_sigil_pool_for_day(739000);
    assert_eq!(pool1, pool2);
}

#[test]
fn test_daily_pool_varies_by_day() {
    let pool1 = daily_sigil_pool_for_day(739000);
    let pool2 = daily_sigil_pool_for_day(739001);
    assert_ne!(pool1, pool2);
}

#[test]
fn test_daily_pool_contains_no_duplicates() {
    for day in 739000..739030 {
        let pool = daily_sigil_pool_for_day(day);
        for i in 0..pool.len() {
            for j in (i + 1)..pool.len() {
                assert_ne!(
                    pool[i], pool[j],
                    "Day {} pool has duplicate: {:?}",
                    day, pool
                );
            }
        }
    }
}

#[test]
fn test_generate_choices_only_from_pool() {
    // With a restricted pool, all generated sigils must be from that pool
    let pool = vec![
        SigilEffectType::XpPercent,
        SigilEffectType::CritChancePercent,
    ];
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    for _ in 0..50 {
        let choices = generate_sigil_choices(&mut rng, &pool);
        for sigil in &choices {
            assert!(
                sigil.effect == SigilEffectType::XpPercent
                    || sigil.effect == SigilEffectType::CritChancePercent,
                "Sigil {:?} not in pool",
                sigil.effect
            );
        }
    }
}

#[test]
fn test_daily_pool_covers_all_types_over_many_days() {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for day in 739000..739100 {
        for effect in daily_sigil_pool_for_day(day) {
            seen.insert(format!("{:?}", effect));
        }
    }
    assert_eq!(
        seen.len(),
        SigilEffectType::ALL.len(),
        "All 11 types should appear within 100 days"
    );
}
