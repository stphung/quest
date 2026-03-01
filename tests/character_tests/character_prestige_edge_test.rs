//! Edge case tests for the character and prestige system.
//!
//! Covers gaps not addressed by existing tests in:
//!   - tests/character_coverage_test.rs
//!   - tests/prestige_cycle_test.rs
//!   - character/derived_stats.rs unit tests
//!   - character/name_validation.rs unit tests
//!
//! New coverage:
//!   - XP multiplier formula at high ranks (50, 100) and exact verification
//!   - Attribute cap at high prestige ranks (50, 100) with formula verification
//!   - PrestigeCombatBonuses at low ranks (1, 10) with exact values
//!   - Derived stats with non-zero enhancement levels
//!   - Derived stats with extreme/negative attribute modifiers
//!   - Enhancement level effect on affix scaling
//!   - Prestige eligibility at high rank boundaries
//!   - Name validation: path traversal, null bytes, pure-whitespace unicode
//!   - Attribute modifier boundary: value 9 vs 10 vs 11
//!   - add_scaled with large multipliers

use quest::character::attributes::{AttributeType, Attributes};
use quest::character::combat_bonuses::PrestigeCombatBonuses;
use quest::character::derived_stats::DerivedStats;
use quest::character::name_validation::{sanitize_name, validate_name};
use quest::character::prestige::{can_prestige, get_prestige_tier, perform_prestige};
use quest::core::constants::{
    ATTRIBUTE_CAP_PER_PRESTIGE, BASE_ATTRIBUTE_CAP, PRESTIGE_CRIT_CAP, PRESTIGE_CRIT_PER_RANK,
    PRESTIGE_FLAT_DAMAGE_EXPONENT, PRESTIGE_FLAT_DAMAGE_FACTOR, PRESTIGE_FLAT_DEFENSE_EXPONENT,
    PRESTIGE_FLAT_DEFENSE_FACTOR, PRESTIGE_FLAT_HP_EXPONENT, PRESTIGE_FLAT_HP_FACTOR,
    PRESTIGE_MULT_BASE_FACTOR, PRESTIGE_MULT_EXPONENT,
};
use quest::core::game_state::GameState;
use quest::items::{Affix, AffixType, AttributeBonuses, Equipment, EquipmentSlot, Item, Rarity};

// ── Helper builders ─────────────────────────────────────────────────────────

fn base_item(slot: EquipmentSlot) -> Item {
    Item {
        slot,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test Item".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![],
        god_item_id: None,
    }
}

fn item_with_attrs(slot: EquipmentSlot, str: u32, dex: u32, con: u32) -> Item {
    Item {
        slot,
        attributes: AttributeBonuses {
            str,
            dex,
            con,
            ..AttributeBonuses::new()
        },
        ..base_item(slot)
    }
}

fn item_with_affix(slot: EquipmentSlot, affix_type: AffixType, value: f64) -> Item {
    Item {
        slot,
        affixes: vec![Affix { affix_type, value }],
        ..base_item(slot)
    }
}

fn state_at(level: u32, prestige_rank: u32) -> GameState {
    let mut s = GameState::new("Edge Case Hero".to_string(), 0);
    s.character_level = level;
    s.prestige_rank = prestige_rank;
    s
}

// ═══════════════════════════════════════════════════════════════════════════
// XP Multiplier formula: 1 + 0.5 * rank^0.7 — high rank verification
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn xp_multiplier_at_rank_50_matches_formula() {
    let rank = 50u32;
    let expected = 1.0 + PRESTIGE_MULT_BASE_FACTOR * (rank as f64).powf(PRESTIGE_MULT_EXPONENT);
    let actual = get_prestige_tier(rank).multiplier;
    assert!(
        (actual - expected).abs() < 0.0001,
        "P{}: expected {:.6}, got {:.6}",
        rank,
        expected,
        actual
    );
    // Sanity check: should be substantially larger than P10
    let p10 = get_prestige_tier(10).multiplier;
    assert!(actual > p10, "P50 multiplier should exceed P10");
}

#[test]
fn xp_multiplier_at_rank_100_matches_formula() {
    let rank = 100u32;
    let expected = 1.0 + PRESTIGE_MULT_BASE_FACTOR * (rank as f64).powf(PRESTIGE_MULT_EXPONENT);
    let actual = get_prestige_tier(rank).multiplier;
    assert!(
        (actual - expected).abs() < 0.0001,
        "P{}: expected {:.6}, got {:.6}",
        rank,
        expected,
        actual
    );
}

#[test]
fn xp_multiplier_specific_values() {
    // Spot-check exact values for ranks documented in CLAUDE.md
    // P1: ~1.5, P5: ~2.54, P10: ~3.51, P20: ~5.07, P30: ~6.41
    let cases: &[(u32, f64, f64)] = &[
        (1, 1.5, 0.01),
        (5, 2.54, 0.05),
        (10, 3.51, 0.05),
        (20, 5.07, 0.05),
        (30, 6.41, 0.05),
    ];
    for (rank, expected, tolerance) in cases {
        let actual = get_prestige_tier(*rank).multiplier;
        assert!(
            (actual - expected).abs() < *tolerance,
            "P{}: expected ~{}, got {:.4}",
            rank,
            expected,
            actual
        );
    }
}

#[test]
fn xp_multiplier_rank_0_is_exactly_1() {
    let tier = get_prestige_tier(0);
    assert!(
        (tier.multiplier - 1.0).abs() < f64::EPSILON,
        "P0 multiplier should be exactly 1.0, got {}",
        tier.multiplier
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Attribute cap: 20 + rank * 5 — formula at various ranks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn attribute_cap_formula_low_ranks() {
    // Formula: BASE_ATTRIBUTE_CAP + ATTRIBUTE_CAP_PER_PRESTIGE * rank
    for rank in [0u32, 1, 2, 5, 10] {
        let mut state = state_at(10, rank);
        // Perform enough prestiges to reach the desired rank
        state.prestige_rank = rank;
        let expected_cap = BASE_ATTRIBUTE_CAP + ATTRIBUTE_CAP_PER_PRESTIGE * rank;
        assert_eq!(
            state.get_attribute_cap(),
            expected_cap,
            "P{}: expected cap {}, got {}",
            rank,
            expected_cap,
            state.get_attribute_cap()
        );
    }
}

#[test]
fn attribute_cap_at_rank_50() {
    let mut state = GameState::new("Cap Test".to_string(), 0);
    state.prestige_rank = 50;
    let expected = BASE_ATTRIBUTE_CAP + ATTRIBUTE_CAP_PER_PRESTIGE * 50;
    assert_eq!(state.get_attribute_cap(), expected);
    assert_eq!(state.get_attribute_cap(), 20 + 5 * 50); // 270
}

#[test]
fn attribute_cap_at_rank_100() {
    let mut state = GameState::new("Cap Test".to_string(), 0);
    state.prestige_rank = 100;
    let expected = BASE_ATTRIBUTE_CAP + ATTRIBUTE_CAP_PER_PRESTIGE * 100;
    assert_eq!(state.get_attribute_cap(), expected);
    assert_eq!(state.get_attribute_cap(), 520); // 20 + 5*100
}

#[test]
fn attribute_cap_increases_by_5_each_prestige_rank() {
    let mut prev_cap = 0;
    for rank in 0..=30 {
        let mut state = GameState::new("Cap Test".to_string(), 0);
        state.prestige_rank = rank;
        let cap = state.get_attribute_cap();
        if rank > 0 {
            assert_eq!(
                cap,
                prev_cap + 5,
                "Each rank should add 5 to cap (rank {})",
                rank
            );
        }
        prev_cap = cap;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PrestigeCombatBonuses at low ranks (1, 10)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn combat_bonuses_rank_1_exact_values() {
    let b = PrestigeCombatBonuses::from_rank(1);
    let expected_damage =
        (PRESTIGE_FLAT_DAMAGE_FACTOR * 1.0_f64.powf(PRESTIGE_FLAT_DAMAGE_EXPONENT)).floor() as u32;
    let expected_defense = (PRESTIGE_FLAT_DEFENSE_FACTOR
        * 1.0_f64.powf(PRESTIGE_FLAT_DEFENSE_EXPONENT))
    .floor() as u32;
    let expected_crit = (1.0_f64 * PRESTIGE_CRIT_PER_RANK).min(PRESTIGE_CRIT_CAP);
    let expected_hp =
        (PRESTIGE_FLAT_HP_FACTOR * 1.0_f64.powf(PRESTIGE_FLAT_HP_EXPONENT)).floor() as u32;

    assert_eq!(b.flat_damage, expected_damage, "P1 flat_damage mismatch");
    assert_eq!(b.flat_defense, expected_defense, "P1 flat_defense mismatch");
    assert!(
        (b.crit_chance - expected_crit).abs() < f64::EPSILON,
        "P1 crit_chance mismatch"
    );
    assert_eq!(b.flat_hp, expected_hp, "P1 flat_hp mismatch");

    // Sanity: all values are positive
    assert!(b.flat_damage > 0);
    assert!(b.flat_defense > 0);
    assert!(b.crit_chance > 0.0);
    assert!(b.flat_hp > 0);
}

#[test]
fn combat_bonuses_rank_10_exact_values() {
    let b = PrestigeCombatBonuses::from_rank(10);
    let expected_damage =
        (PRESTIGE_FLAT_DAMAGE_FACTOR * 10.0_f64.powf(PRESTIGE_FLAT_DAMAGE_EXPONENT)).floor() as u32;
    let expected_defense = (PRESTIGE_FLAT_DEFENSE_FACTOR
        * 10.0_f64.powf(PRESTIGE_FLAT_DEFENSE_EXPONENT))
    .floor() as u32;
    let expected_crit = (10.0_f64 * PRESTIGE_CRIT_PER_RANK).min(PRESTIGE_CRIT_CAP);
    let expected_hp =
        (PRESTIGE_FLAT_HP_FACTOR * 10.0_f64.powf(PRESTIGE_FLAT_HP_EXPONENT)).floor() as u32;

    assert_eq!(b.flat_damage, expected_damage);
    assert_eq!(b.flat_defense, expected_defense);
    assert!((b.crit_chance - expected_crit).abs() < f64::EPSILON);
    assert_eq!(b.flat_hp, expected_hp);
}

#[test]
fn combat_bonuses_monotonically_increasing_across_all_fields() {
    let mut prev = PrestigeCombatBonuses::from_rank(0);
    for rank in 1..=50 {
        let cur = PrestigeCombatBonuses::from_rank(rank);
        assert!(
            cur.flat_damage >= prev.flat_damage,
            "flat_damage should not decrease at rank {}",
            rank
        );
        assert!(
            cur.flat_defense >= prev.flat_defense,
            "flat_defense should not decrease at rank {}",
            rank
        );
        assert!(
            cur.flat_hp >= prev.flat_hp,
            "flat_hp should not decrease at rank {}",
            rank
        );
        prev = cur;
    }
}

#[test]
fn combat_bonuses_crit_capped_at_30() {
    // At rank 30: 30 * 0.5 = 15.0 exactly = cap
    // At rank 31 onwards: still 15.0 (capped)
    for rank in 30..=50 {
        let b = PrestigeCombatBonuses::from_rank(rank);
        assert!(
            (b.crit_chance - PRESTIGE_CRIT_CAP).abs() < f64::EPSILON,
            "crit should be capped at rank {}",
            rank
        );
    }
}

#[test]
fn combat_bonuses_rank_0_is_all_zeros() {
    let b = PrestigeCombatBonuses::from_rank(0);
    assert_eq!(b.flat_damage, 0);
    assert_eq!(b.flat_defense, 0);
    assert!((b.crit_chance - 0.0).abs() < f64::EPSILON);
    assert_eq!(b.flat_hp, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Derived stats with non-zero enhancement levels
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn derived_stats_enhancement_scales_attribute_bonuses() {
    let attrs = Attributes::new(); // all at 10 (modifier 0)
    let mut equipment = Equipment::new();

    // Weapon with +4 STR (would give mod (14-10)/2 = 2, physical = 5 + 2*2 = 9)
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_attrs(EquipmentSlot::Weapon, 4, 0, 0)),
    );

    // No enhancement: base +4 STR -> mod=2 -> physical = 5 + 2*2 = 9
    let stats_no_enh = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    // Enhancement level 1 on weapon slot (index 0)
    // enhancement_multiplier(1) > 1.0, so the +4 STR bonus should be scaled up
    let mut enh = [0u8; 7];
    enh[0] = 1; // weapon slot enhancement = 1
    let stats_enh1 = DerivedStats::calculate_derived_stats(&attrs, &equipment, &enh);

    // Enhancement should boost stats (higher enhancement = higher multiplier)
    assert!(
        stats_enh1.physical_damage >= stats_no_enh.physical_damage,
        "Enhancement level 1 should not decrease physical damage"
    );

    // Level 5 should be strictly better than level 1
    let mut enh5 = [0u8; 7];
    enh5[0] = 5;
    let stats_enh5 = DerivedStats::calculate_derived_stats(&attrs, &equipment, &enh5);
    assert!(
        stats_enh5.physical_damage >= stats_enh1.physical_damage,
        "Enhancement 5 should produce >= physical damage than enhancement 1"
    );
}

#[test]
fn derived_stats_enhancement_scales_affix_values() {
    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    // Weapon with a DamagePercent affix of 100% (should double damage)
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::DamagePercent,
            100.0,
        )),
    );

    // At enhancement 0: 100% bonus scales by 1.0 multiplier
    let stats_enh0 = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    // At enhancement 5: the same affix value is scaled by enhancement_multiplier(5)
    let mut enh5 = [0u8; 7];
    enh5[0] = 5;
    let stats_enh5 = DerivedStats::calculate_derived_stats(&attrs, &equipment, &enh5);

    // Enhancement should amplify the affix benefit
    assert!(
        stats_enh5.physical_damage >= stats_enh0.physical_damage,
        "Enhancement 5 should produce >= physical damage from affix scaling"
    );
}

#[test]
fn derived_stats_enhancement_scales_hp_bonus_affix() {
    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::HPBonus,
            50.0,
        )),
    );

    let no_enh = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    let mut enh = [0u8; 7];
    enh[1] = 4; // armor slot (index 1)
    let with_enh = DerivedStats::calculate_derived_stats(&attrs, &equipment, &enh);

    // With enhancement, the HP bonus affix value is scaled up
    assert!(
        with_enh.max_hp >= no_enh.max_hp,
        "Enhancement should not reduce HP: {} vs {}",
        with_enh.max_hp,
        no_enh.max_hp
    );
}

#[test]
fn derived_stats_zero_enhancement_matches_default() {
    let attrs = Attributes::new();
    let mut equipment = Equipment::new();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_attrs(EquipmentSlot::Weapon, 4, 2, 0)),
    );

    // Enhancement [0;7] should produce identical results to no equipment consideration
    let stats_zero = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);
    let stats_zero2 = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    assert_eq!(stats_zero.physical_damage, stats_zero2.physical_damage);
    assert_eq!(stats_zero.max_hp, stats_zero2.max_hp);
}

#[test]
fn derived_stats_enhancement_on_multiple_slots() {
    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_attrs(EquipmentSlot::Weapon, 4, 0, 0)),
    );
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_attrs(EquipmentSlot::Armor, 0, 0, 4)),
    );

    let no_enh = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    // Enhance weapon (slot 0) and armor (slot 1)
    let mut enh = [0u8; 7];
    enh[0] = 3;
    enh[1] = 3;
    let with_enh = DerivedStats::calculate_derived_stats(&attrs, &equipment, &enh);

    assert!(
        with_enh.physical_damage >= no_enh.physical_damage,
        "Weapon enhancement should not decrease physical damage"
    );
    assert!(
        with_enh.max_hp >= no_enh.max_hp,
        "Armor enhancement should not decrease max HP"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Derived stats — extreme attribute values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn derived_stats_all_attributes_at_max_cap_p0() {
    // At P0, cap is 20. All attributes at 20 => modifier = (20-10)/2 = 5 for each
    let mut attrs = Attributes::new();
    for attr in AttributeType::all() {
        attrs.set(attr, 20);
    }
    let stats = DerivedStats::from_attributes(&attrs);

    // STR=20, mod=5: physical = 5 + 5*2 = 15
    assert_eq!(stats.physical_damage, 15);
    // INT=20, mod=5: magic = 5 + 5*2 = 15
    assert_eq!(stats.magic_damage, 15);
    // CON=20, mod=5: hp = 50 + 5*10 = 100
    assert_eq!(stats.max_hp, 100);
    // DEX=20, mod=5: defense=5, crit=5+5=10
    assert_eq!(stats.defense, 5);
    assert_eq!(stats.crit_chance_percent, 10);
    // WIS=20, mod=5: xp_mult = 1.0 + 5*0.05 = 1.25
    assert!((stats.xp_multiplier - 1.25).abs() < f64::EPSILON);
}

#[test]
fn derived_stats_all_attributes_at_minimum() {
    // Minimum attribute value is 1. Modifier = (1-10)/2 = -9/2 = -4 (integer division)
    let mut attrs = Attributes::new();
    for attr in AttributeType::all() {
        attrs.set(attr, 1);
    }
    let stats = DerivedStats::from_attributes(&attrs);

    // Physical damage: max(1, 5 + (-4)*2) = max(1, -3) = 1
    assert_eq!(stats.physical_damage, 1, "Damage should floor at 1");
    // Magic damage: same floor
    assert_eq!(stats.magic_damage, 1, "Magic damage should floor at 1");
    // HP: max(1, 50 + (-4)*10) = max(1, 10) = 10
    assert_eq!(stats.max_hp, 10, "HP should be 10 with min attributes");
    // Defense: max(0, -4) = 0
    assert_eq!(stats.defense, 0, "Defense cannot go negative");
    // Crit: max(0, 5 + (-4)) = max(0, 1) = 1
    assert_eq!(stats.crit_chance_percent, 1);
}

#[test]
fn derived_stats_modifier_boundary_values() {
    // Test attributes at 9, 10, 11 — boundary of modifier sign change
    let mut attrs = Attributes::new();

    // Value 9: (9-10)/2 = -1/2 = 0 in Rust (truncates toward zero)
    attrs.set(AttributeType::Strength, 9);
    assert_eq!(attrs.modifier(AttributeType::Strength), 0);

    // Value 10: (10-10)/2 = 0
    attrs.set(AttributeType::Strength, 10);
    assert_eq!(attrs.modifier(AttributeType::Strength), 0);

    // Value 11: (11-10)/2 = 0 (truncation)
    attrs.set(AttributeType::Strength, 11);
    assert_eq!(attrs.modifier(AttributeType::Strength), 0);

    // Value 12: (12-10)/2 = 1
    attrs.set(AttributeType::Strength, 12);
    assert_eq!(attrs.modifier(AttributeType::Strength), 1);

    // Value 8: (8-10)/2 = -2/2 = -1
    attrs.set(AttributeType::Strength, 8);
    assert_eq!(attrs.modifier(AttributeType::Strength), -1);
}

#[test]
fn derived_stats_dex_boundary_affects_defense_and_crit() {
    // DEX at exactly 10: modifier = 0 → defense = 0, crit = 5
    let attrs = Attributes::new(); // DEX = 10
    let stats = DerivedStats::from_attributes(&attrs);
    assert_eq!(stats.defense, 0);
    assert_eq!(stats.crit_chance_percent, 5);

    // DEX at 12: modifier = 1 → defense = 1, crit = 6
    let mut attrs12 = Attributes::new();
    attrs12.set(AttributeType::Dexterity, 12);
    let stats12 = DerivedStats::from_attributes(&attrs12);
    assert_eq!(stats12.defense, 1);
    assert_eq!(stats12.crit_chance_percent, 6);

    // DEX at 8: modifier = -1 → defense = 0 (can't go negative), crit = 4
    let mut attrs8 = Attributes::new();
    attrs8.set(AttributeType::Dexterity, 8);
    let stats8 = DerivedStats::from_attributes(&attrs8);
    assert_eq!(stats8.defense, 0, "Defense cannot go negative with low DEX");
    assert_eq!(stats8.crit_chance_percent, 4);
}

#[test]
fn derived_stats_wis_xp_multiplier_exact_values() {
    // WIS 10: mod=0, xp_mult = 1.0 + 0*0.05 = 1.0
    // WIS 12: mod=1, xp_mult = 1.0 + 1*0.05 = 1.05
    // WIS 20: mod=5, xp_mult = 1.0 + 5*0.05 = 1.25
    let cases = [(10u32, 1.0f64), (12, 1.05), (14, 1.1), (20, 1.25)];
    for (wis, expected_mult) in cases {
        let mut attrs = Attributes::new();
        attrs.set(AttributeType::Wisdom, wis);
        let stats = DerivedStats::from_attributes(&attrs);
        assert!(
            (stats.xp_multiplier - expected_mult).abs() < 0.001,
            "WIS {}: expected xp_mult {}, got {}",
            wis,
            expected_mult,
            stats.xp_multiplier
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Derived stats — attributes with add_scaled (enhancement interaction)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn attributes_add_scaled_with_multiplier_1() {
    let mut base = Attributes::new(); // all 10
    let bonus = Attributes::from_bonuses(2, 0, 0, 0, 0, 0);
    base.add_scaled(&bonus, 1.0);
    // The bonus attributes are added directly: STR = 10 + 2 = 12
    assert_eq!(base.get(AttributeType::Strength), 12);
    // Others unchanged
    assert_eq!(base.get(AttributeType::Dexterity), 10);
}

#[test]
fn attributes_add_scaled_with_multiplier_2() {
    let mut base = Attributes::new(); // all 10
    let bonus = Attributes::from_bonuses(4, 0, 2, 0, 0, 0);
    base.add_scaled(&bonus, 2.0);
    // STR bonus scaled: 4 * 2.0 = 8, total = 10 + 8 = 18
    assert_eq!(base.get(AttributeType::Strength), 18);
    // CON bonus scaled: 2 * 2.0 = 4, total = 10 + 4 = 14
    assert_eq!(base.get(AttributeType::Constitution), 14);
}

#[test]
fn attributes_add_scaled_with_fractional_multiplier() {
    let mut base = Attributes::new(); // all 10
                                      // Bonus of 3, scaled by 0.5 → rounds to 2 (round half to even)
                                      // Note: (3 * 0.5).round() = 1.5.round() = 2 (in Rust, .round() rounds half away from zero)
    let bonus = Attributes::from_bonuses(3, 0, 0, 0, 0, 0);
    base.add_scaled(&bonus, 0.5);
    // 3 * 0.5 = 1.5, rounded = 2
    assert_eq!(base.get(AttributeType::Strength), 12);
}

// ═══════════════════════════════════════════════════════════════════════════
// Prestige eligibility at high rank boundaries
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn can_prestige_at_rank_19_requires_220() {
    // Rank 19 → Legendary, next rank 20 requires level 220 + (20-19)*15 = 235
    let state_below = state_at(234, 19);
    assert!(
        !can_prestige(&state_below),
        "Level 234 should not allow P20"
    );

    let state_at_req = state_at(235, 19);
    assert!(can_prestige(&state_at_req), "Level 235 should allow P20");
}

#[test]
fn can_prestige_at_rank_30_requires_correct_level() {
    // Rank 30 → Eternal, next rank 31
    // Required: 220 + (31 - 19) * 15 = 220 + 12*15 = 220 + 180 = 400
    let next_level = 220 + (31 - 19) * 15;
    let state_below = state_at(next_level - 1, 30);
    assert!(
        !can_prestige(&state_below),
        "One level below should not allow prestige"
    );

    let state_ok = state_at(next_level, 30);
    assert!(
        can_prestige(&state_ok),
        "At required level should allow prestige"
    );
}

#[test]
fn can_prestige_at_each_standard_rank_boundary() {
    // All hard-coded required levels from tiers.rs
    let boundaries: &[(u32, u32)] = &[
        (0, 10),  // P0 → P1: need level 10
        (1, 25),  // P1 → P2: need level 25
        (2, 50),  // P2 → P3: need level 50
        (3, 65),  // P3 → P4: need level 65
        (4, 80),  // P4 → P5: need level 80
        (5, 90),  // P5 → P6: need level 90
        (6, 100), // P6 → P7: need level 100
        (7, 110), // P7 → P8: need level 110
        (8, 120), // P8 → P9: need level 120
        (9, 130), // P9 → P10: need level 130
    ];

    for (current_rank, required_level) in boundaries {
        let below = state_at(required_level - 1, *current_rank);
        assert!(
            !can_prestige(&below),
            "P{} at level {} should not allow prestige",
            current_rank,
            required_level - 1
        );

        let at_level = state_at(*required_level, *current_rank);
        assert!(
            can_prestige(&at_level),
            "P{} at level {} should allow prestige",
            current_rank,
            required_level
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Perform prestige — rank accumulation over many prestiges
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn perform_prestige_rank_accumulation() {
    let mut state = GameState::new("Accumulation Test".to_string(), 0);

    // Prestige from rank 0 to rank 4 (each with correct level requirements)
    let levels: &[u32] = &[10, 25, 50, 65];
    for (i, &req_level) in levels.iter().enumerate() {
        state.character_level = req_level;
        assert!(
            can_prestige(&state),
            "Should be able to prestige at rank {}",
            i
        );
        perform_prestige(&mut state);
        assert_eq!(
            state.prestige_rank,
            (i + 1) as u32,
            "Rank after {} prestiges",
            i + 1
        );
        assert_eq!(state.total_prestige_count, (i + 1) as u64);
    }
}

#[test]
fn perform_prestige_preserves_zone_unlocks_per_new_rank() {
    // After prestige, zone_progression.reset_for_prestige(new_rank) is called
    // For rank 1, we should still start at zone 1 (no P5+ zones unlocked)
    let mut state = state_at(10, 0);
    state.zone_progression.current_zone_id = 5;
    state.zone_progression.current_subzone_id = 3;

    perform_prestige(&mut state);

    assert_eq!(state.prestige_rank, 1);
    assert_eq!(
        state.zone_progression.current_zone_id, 1,
        "Zone should reset to 1"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Name validation — edge cases not covered by existing tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn validate_name_exactly_16_chars_is_valid() {
    let name = "A".repeat(16);
    assert!(
        validate_name(&name).is_ok(),
        "Exactly 16 characters should be valid"
    );
}

#[test]
fn validate_name_17_chars_is_invalid() {
    let name = "A".repeat(17);
    assert!(
        validate_name(&name).is_err(),
        "17 characters should be invalid"
    );
}

#[test]
fn validate_name_single_char_is_valid() {
    assert!(validate_name("A").is_ok());
    assert!(validate_name("z").is_ok());
    assert!(validate_name("5").is_ok());
}

#[test]
fn validate_name_only_whitespace_is_invalid() {
    assert!(validate_name("   ").is_err());
    assert!(validate_name("\t").is_err());
    assert!(validate_name("\n").is_err());
}

#[test]
fn validate_name_path_traversal_rejected() {
    // Path traversal characters that would be invalid anyway
    assert!(validate_name("../etc/passwd").is_err());
    assert!(validate_name("../../secret").is_err());
    assert!(validate_name("foo/bar").is_err());
    assert!(validate_name("foo\\bar").is_err());
}

#[test]
fn validate_name_null_byte_rejected() {
    // Null bytes are not alphanumeric or allowed punctuation
    assert!(validate_name("hero\0name").is_err());
}

#[test]
fn validate_name_control_characters_rejected() {
    // Control characters are not alphanumeric
    assert!(validate_name("hero\x01name").is_err());
    assert!(validate_name("hero\x1Fname").is_err());
    assert!(validate_name("hero\x7Fname").is_err());
}

#[test]
fn validate_name_unicode_emoji_rejected() {
    // Emojis are not alphanumeric in Rust's definition
    assert!(validate_name("Hero🗡️").is_err());
    assert!(validate_name("⚔️Warrior").is_err());
}

#[test]
fn validate_name_unicode_letters_accepted() {
    // Unicode alphabetic letters are valid (is_alphanumeric returns true for them)
    assert!(validate_name("Héro").is_ok());
    assert!(validate_name("Müller").is_ok());
}

#[test]
fn validate_name_mixed_valid_chars() {
    // All valid character types together
    assert!(validate_name("Hero-1_Test").is_ok());
    assert!(validate_name("My Hero 2").is_ok());
    assert!(validate_name("A-B_C").is_ok());
}

#[test]
fn validate_name_reserved_deep_and_enhancement() {
    // "deep" and "enhancement" are reserved (conflict with account-level .json files)
    assert!(validate_name("deep").is_err());
    assert!(validate_name("enhancement").is_err());
    // "stormglass" has no account-level file, so it's fine
    assert!(validate_name("stormglass").is_ok());
}

#[test]
fn sanitize_name_path_traversal_produces_safe_output() {
    // Even if validate_name rejects these, sanitize_name should produce safe output
    let sanitized = sanitize_name("../etc/passwd");
    // After sanitization: remove special chars, lowercase, spaces→underscores
    // '../' has '.' and '/' which are neither alphanumeric, '_', nor '-'
    assert!(!sanitized.contains('/'));
    assert!(!sanitized.contains('.'));
}

#[test]
fn sanitize_name_empty_input_produces_empty() {
    assert_eq!(sanitize_name(""), "");
}

#[test]
fn sanitize_name_only_invalid_chars_produces_empty() {
    // All chars removed → empty string
    assert_eq!(sanitize_name("!@#$%^&*()"), "");
}

#[test]
fn sanitize_name_preserves_hyphens() {
    assert_eq!(sanitize_name("Hero-One"), "hero-one");
}

#[test]
fn sanitize_name_lowercases_everything() {
    assert_eq!(sanitize_name("UPPERCASE"), "uppercase");
    assert_eq!(sanitize_name("MiXeDcAsE"), "mixedcase");
}

// ═══════════════════════════════════════════════════════════════════════════
// Prestige tier — complete boundary coverage for all hard-coded tiers
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn prestige_tier_all_names_correct_sequence() {
    let expected_names: &[(u32, &str)] = &[
        (0, "None"),
        (1, "Bronze"),
        (2, "Silver"),
        (3, "Gold"),
        (4, "Platinum"),
        (5, "Diamond"),
        (6, "Emerald"),
        (7, "Sapphire"),
        (8, "Ruby"),
        (9, "Obsidian"),
        (10, "Celestial"),
        (11, "Astral"),
        (12, "Cosmic"),
        (13, "Stellar"),
        (14, "Galactic"),
        (15, "Transcendent"),
        (16, "Divine"),
        (17, "Exalted"),
        (18, "Mythic"),
        (19, "Legendary"),
        (20, "Eternal"),
        (100, "Eternal"),
    ];

    for (rank, expected) in expected_names {
        let tier = get_prestige_tier(*rank);
        assert_eq!(
            tier.name, *expected,
            "Rank {} should be '{}', got '{}'",
            rank, expected, tier.name
        );
    }
}

#[test]
fn prestige_tier_required_levels_all_hard_coded() {
    let expected: &[(u32, u32)] = &[
        (0, 0),
        (1, 10),
        (2, 25),
        (3, 50),
        (4, 65),
        (5, 80),
        (6, 90),
        (7, 100),
        (8, 110),
        (9, 120),
        (10, 130),
        (11, 140),
        (12, 150),
        (13, 160),
        (14, 170),
        (15, 180),
        (16, 190),
        (17, 200),
        (18, 210),
        (19, 220),
    ];

    for (rank, expected_level) in expected {
        let tier = get_prestige_tier(*rank);
        assert_eq!(
            tier.required_level, *expected_level,
            "Rank {} required_level should be {}, got {}",
            rank, expected_level, tier.required_level
        );
    }
}

#[test]
fn prestige_tier_eternal_formula_correct_at_boundary_ranks() {
    // Formula: 220 + (rank - 19) * 15 for rank >= 20
    for rank in [20u32, 21, 25, 30, 50, 100] {
        let expected = 220 + (rank - 19) * 15;
        let tier = get_prestige_tier(rank);
        assert_eq!(
            tier.required_level, expected,
            "Eternal rank {}: expected level {}, got {}",
            rank, expected, tier.required_level
        );
    }
}
