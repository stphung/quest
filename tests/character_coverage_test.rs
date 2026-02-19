#![allow(clippy::field_reassign_with_default)]

//! Coverage tests for character submodules:
//!   - character/combat_bonuses.rs
//!   - character/multipliers.rs
//!   - character/prestige_actions.rs
//!   - character/tiers.rs
//!
//! Avoids duplicating tests already in:
//!   - tests/item_combat_coverage_test.rs (PrestigeCombatBonuses formula/scaling)
//!   - tests/prestige_cycle_test.rs (full prestige cycle, fishing preservation)

use quest::character::attributes::{AttributeType, Attributes};
use quest::character::combat_bonuses::PrestigeCombatBonuses;
use quest::character::multipliers::{prestige_multiplier, prestige_multiplier_with_equipment};
use quest::character::prestige::{
    can_prestige, get_adventurer_rank, get_next_prestige_tier, get_prestige_tier, perform_prestige,
    perform_prestige_with_vault,
};
use quest::core::game_state::GameState;
use quest::items::types::{AttributeBonuses, EquipmentSlot, Item, Rarity};
use quest::items::Equipment;

// ── Helper ─────────────────────────────────────────────────────────

fn make_item(slot: EquipmentSlot, cha_bonus: u32) -> Item {
    Item {
        slot,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test Item".to_string(),
        attributes: AttributeBonuses {
            cha: cha_bonus,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    }
}

fn make_state_at_level(level: u32, prestige_rank: u32) -> GameState {
    let mut state = GameState::new("Test Hero".to_string(), 0);
    state.character_level = level;
    state.prestige_rank = prestige_rank;
    state
}

// ═══════════════════════════════════════════════════════════════════
// character/combat_bonuses — exact values at specific ranks
// (formulas already tested in item_combat_coverage_test;
//  these verify computed values at higher ranks and edge cases)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn combat_bonuses_exact_values_at_rank_5() {
    let b = PrestigeCombatBonuses::from_rank(5);
    let expected_damage = (5.0_f64 * 5.0_f64.powf(0.7)).floor() as u32;
    let expected_defense = (3.0_f64 * 5.0_f64.powf(0.6)).floor() as u32;
    let expected_crit = 5.0 * 0.5;
    let expected_hp = (15.0_f64 * 5.0_f64.powf(0.6)).floor() as u32;

    assert_eq!(b.flat_damage, expected_damage);
    assert_eq!(b.flat_defense, expected_defense);
    assert!((b.crit_chance - expected_crit).abs() < f64::EPSILON);
    assert_eq!(b.flat_hp, expected_hp);
}

#[test]
fn combat_bonuses_exact_values_at_rank_20() {
    let b = PrestigeCombatBonuses::from_rank(20);
    let expected_damage = (5.0_f64 * 20.0_f64.powf(0.7)).floor() as u32;
    let expected_defense = (3.0_f64 * 20.0_f64.powf(0.6)).floor() as u32;
    let expected_crit = (20.0_f64 * 0.5).min(15.0);
    let expected_hp = (15.0_f64 * 20.0_f64.powf(0.6)).floor() as u32;

    assert_eq!(b.flat_damage, expected_damage);
    assert_eq!(b.flat_defense, expected_defense);
    assert!((b.crit_chance - expected_crit).abs() < f64::EPSILON);
    assert_eq!(b.flat_hp, expected_hp);
}

#[test]
fn combat_bonuses_very_high_rank_100() {
    let b = PrestigeCombatBonuses::from_rank(100);
    // All values should be positive and substantial
    assert!(b.flat_damage > 0);
    assert!(b.flat_defense > 0);
    assert!((b.crit_chance - 15.0).abs() < f64::EPSILON); // Capped
    assert!(b.flat_hp > 0);

    // Should be much higher than rank 20
    let b20 = PrestigeCombatBonuses::from_rank(20);
    assert!(b.flat_damage > b20.flat_damage);
    assert!(b.flat_defense > b20.flat_defense);
    assert!(b.flat_hp > b20.flat_hp);
}

#[test]
fn combat_bonuses_crit_reaches_cap_at_rank_30() {
    let b30 = PrestigeCombatBonuses::from_rank(30);
    // 30 * 0.5 = 15.0, exactly at cap
    assert!((b30.crit_chance - 15.0).abs() < f64::EPSILON);
}

#[test]
fn combat_bonuses_crit_below_cap_at_rank_29() {
    let b29 = PrestigeCombatBonuses::from_rank(29);
    // 29 * 0.5 = 14.5, below cap
    assert!((b29.crit_chance - 14.5).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// character/multipliers — prestige_multiplier, prestige_multiplier_with_equipment
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multiplier_base_cha_10_gives_zero_bonus() {
    // CHA 10 => modifier = (10-10)/2 = 0, so no CHA bonus
    let attrs = Attributes::new(); // all 10
    let base_mult = 1.5; // e.g. P1
    let result = prestige_multiplier(base_mult, &attrs);
    assert!((result - 1.5).abs() < f64::EPSILON);
}

#[test]
fn multiplier_cha_20_adds_bonus() {
    // CHA 20 => modifier = (20-10)/2 = 5, bonus = 5 * 0.1 = 0.5
    let mut attrs = Attributes::new();
    attrs.set(AttributeType::Charisma, 20);
    let base_mult = 1.5;
    let result = prestige_multiplier(base_mult, &attrs);
    assert!((result - 2.0).abs() < f64::EPSILON);
}

#[test]
fn multiplier_cha_12_adds_small_bonus() {
    // CHA 12 => modifier = (12-10)/2 = 1, bonus = 1 * 0.1 = 0.1
    let mut attrs = Attributes::new();
    attrs.set(AttributeType::Charisma, 12);
    let base_mult = 1.0;
    let result = prestige_multiplier(base_mult, &attrs);
    assert!((result - 1.1).abs() < f64::EPSILON);
}

#[test]
fn multiplier_low_cha_gives_negative_bonus() {
    // CHA 8 => modifier = (8-10)/2 = -1
    // bonus = -1 * 0.1 = -0.1
    let mut attrs = Attributes::new();
    attrs.set(AttributeType::Charisma, 8);
    let base_mult = 2.0;
    let result = prestige_multiplier(base_mult, &attrs);
    assert!((result - 1.9).abs() < f64::EPSILON);
}

#[test]
fn multiplier_cha_11_has_zero_modifier() {
    // CHA 11 => modifier = (11-10)/2 = 1/2 = 0 (integer division)
    let mut attrs = Attributes::new();
    attrs.set(AttributeType::Charisma, 11);
    let base_mult = 3.0;
    let result = prestige_multiplier(base_mult, &attrs);
    assert!((result - 3.0).abs() < f64::EPSILON);
}

#[test]
fn multiplier_with_equipment_no_cha_items() {
    let attrs = Attributes::new();
    let equipment = Equipment::new();
    let base_mult = 1.5;
    let result = prestige_multiplier_with_equipment(base_mult, &attrs, &equipment);
    // No equipment CHA => same as base
    assert!((result - 1.5).abs() < f64::EPSILON);
}

#[test]
fn multiplier_with_equipment_cha_item_adds_bonus() {
    let attrs = Attributes::new(); // CHA = 10
    let mut equipment = Equipment::new();
    // Equip item with +10 CHA
    equipment.set(
        EquipmentSlot::Ring,
        Some(make_item(EquipmentSlot::Ring, 10)),
    );

    let base_mult = 1.5;
    let result = prestige_multiplier_with_equipment(base_mult, &attrs, &equipment);
    // Total CHA = 10 (base) + 10 (item) = 20, modifier = (20-10)/2 = 5
    // bonus = 5 * 0.1 = 0.5, total = 1.5 + 0.5 = 2.0
    assert!((result - 2.0).abs() < f64::EPSILON);
}

#[test]
fn multiplier_with_equipment_multiple_cha_items() {
    let attrs = Attributes::new(); // CHA = 10
    let mut equipment = Equipment::new();
    equipment.set(EquipmentSlot::Ring, Some(make_item(EquipmentSlot::Ring, 4)));
    equipment.set(
        EquipmentSlot::Amulet,
        Some(make_item(EquipmentSlot::Amulet, 6)),
    );

    let base_mult = 1.0;
    let result = prestige_multiplier_with_equipment(base_mult, &attrs, &equipment);
    // Total CHA = 10 + 4 + 6 = 20, modifier = 5, bonus = 0.5
    assert!((result - 1.5).abs() < f64::EPSILON);
}

#[test]
fn multiplier_with_equipment_stacks_with_base_cha() {
    let mut attrs = Attributes::new();
    attrs.set(AttributeType::Charisma, 14); // modifier = (14-10)/2 = 2
    let mut equipment = Equipment::new();
    equipment.set(
        EquipmentSlot::Amulet,
        Some(make_item(EquipmentSlot::Amulet, 6)),
    );

    let base_mult = 2.0;
    let result = prestige_multiplier_with_equipment(base_mult, &attrs, &equipment);
    // Total CHA = 14 + 6 = 20, modifier = 5, bonus = 0.5
    assert!((result - 2.5).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// character/prestige_actions — can_prestige, perform_prestige, perform_prestige_with_vault
// (Full cycle tests in prestige_cycle_test.rs; these test specific edge cases)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn can_prestige_below_required_level() {
    let state = make_state_at_level(9, 0); // Need level 10
    assert!(!can_prestige(&state));
}

#[test]
fn can_prestige_at_required_level() {
    let state = make_state_at_level(10, 0);
    assert!(can_prestige(&state));
}

#[test]
fn can_prestige_above_required_level() {
    let state = make_state_at_level(50, 0);
    assert!(can_prestige(&state));
}

#[test]
fn can_prestige_second_prestige_needs_25() {
    let state = make_state_at_level(24, 1);
    assert!(!can_prestige(&state));

    let state = make_state_at_level(25, 1);
    assert!(can_prestige(&state));
}

#[test]
fn can_prestige_high_rank_needs_high_level() {
    // At rank 19, next tier is rank 20 which requires level 235 (220 + (20-19)*15)
    let state = make_state_at_level(234, 19);
    assert!(!can_prestige(&state));

    let state = make_state_at_level(235, 19);
    assert!(can_prestige(&state));
}

#[test]
fn perform_prestige_resets_level_and_xp() {
    let mut state = make_state_at_level(10, 0);
    state.character_xp = 5000;
    perform_prestige(&mut state);

    assert_eq!(state.character_level, 1);
    assert_eq!(state.character_xp, 0);
    assert_eq!(state.prestige_rank, 1);
}

#[test]
fn perform_prestige_resets_all_attributes_to_base() {
    let mut state = make_state_at_level(10, 0);
    state.attributes.set(AttributeType::Strength, 20);
    state.attributes.set(AttributeType::Dexterity, 18);
    state.attributes.set(AttributeType::Constitution, 16);

    perform_prestige(&mut state);

    for attr in AttributeType::all() {
        assert_eq!(
            state.attributes.get(attr),
            10,
            "{:?} should reset to 10",
            attr
        );
    }
}

#[test]
fn perform_prestige_clears_all_equipment() {
    let mut state = make_state_at_level(10, 0);
    state.equipment.set(
        EquipmentSlot::Weapon,
        Some(make_item(EquipmentSlot::Weapon, 0)),
    );
    state.equipment.set(
        EquipmentSlot::Armor,
        Some(make_item(EquipmentSlot::Armor, 0)),
    );
    state
        .equipment
        .set(EquipmentSlot::Ring, Some(make_item(EquipmentSlot::Ring, 0)));

    perform_prestige(&mut state);

    assert_eq!(state.equipment.iter_equipped().count(), 0);
}

#[test]
fn perform_prestige_clears_dungeon() {
    use quest::dungeon::{Dungeon, DungeonSize};
    let mut state = make_state_at_level(10, 0);
    state.active_dungeon = Some(Dungeon::new(DungeonSize::Small));

    perform_prestige(&mut state);

    assert!(state.active_dungeon.is_none());
}

#[test]
fn perform_prestige_clears_active_fishing() {
    use quest::fishing::{CaughtFish, FishRarity, FishingPhase, FishingSession};
    let mut state = make_state_at_level(10, 0);
    state.active_fishing = Some(FishingSession {
        spot_name: "Lake".to_string(),
        total_fish: 5,
        fish_caught: vec![CaughtFish {
            name: "Trout".to_string(),
            rarity: FishRarity::Common,
            xp_reward: 10,
        }],
        items_found: vec![],
        ticks_remaining: 10,
        phase: FishingPhase::Waiting,
    });

    perform_prestige(&mut state);

    assert!(state.active_fishing.is_none());
}

#[test]
fn perform_prestige_clears_active_minigame() {
    let mut state = make_state_at_level(10, 0);
    // active_minigame is Option<ActiveMinigame>, set it to None is default
    // Just verify it stays None after prestige
    state.active_minigame = None;
    perform_prestige(&mut state);
    assert!(state.active_minigame.is_none());
}

#[test]
fn perform_prestige_resets_combat_state() {
    let mut state = make_state_at_level(10, 0);
    state.combat_state.player_current_hp = 5;
    state.combat_state.is_regenerating = true;

    perform_prestige(&mut state);

    assert_eq!(state.combat_state.player_current_hp, 50); // BASE_HP
    assert_eq!(state.combat_state.player_max_hp, 50);
    assert!(!state.combat_state.is_regenerating);
    assert!(state.combat_state.current_enemy.is_none());
}

#[test]
fn perform_prestige_increments_total_prestige_count() {
    let mut state = make_state_at_level(10, 0);
    perform_prestige(&mut state);
    assert_eq!(state.total_prestige_count, 1);

    state.character_level = 25; // Need 25 for second prestige
    perform_prestige(&mut state);
    assert_eq!(state.total_prestige_count, 2);
}

#[test]
fn perform_prestige_clears_xp_rate_tracking() {
    let mut state = make_state_at_level(10, 0);
    state.xp_rate_samples.push_back(1000);
    state.xp_rate_samples.push_back(2000);
    state.xp_this_second = 500;

    perform_prestige(&mut state);

    assert!(state.xp_rate_samples.is_empty());
    assert_eq!(state.xp_this_second, 0);
}

#[test]
fn perform_prestige_does_nothing_when_ineligible() {
    let mut state = make_state_at_level(5, 0);
    let old_rank = state.prestige_rank;
    let old_level = state.character_level;

    perform_prestige(&mut state);

    assert_eq!(state.prestige_rank, old_rank);
    assert_eq!(state.character_level, old_level);
}

#[test]
fn perform_prestige_resets_zone_progression() {
    let mut state = make_state_at_level(10, 0);
    state.zone_progression.current_zone_id = 3;
    state.zone_progression.current_subzone_id = 2;

    perform_prestige(&mut state);

    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
}

// ── perform_prestige_with_vault ────────────────────────────────────

#[test]
fn vault_prestige_preserves_single_slot() {
    let mut state = make_state_at_level(10, 0);
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        ilvl: 50,
        tier: 8,
        base_name: "Blade".to_string(),
        display_name: "Legendary Blade".to_string(),
        attributes: AttributeBonuses {
            str: 20,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    state
        .equipment
        .set(EquipmentSlot::Weapon, Some(weapon.clone()));
    state.equipment.set(
        EquipmentSlot::Armor,
        Some(make_item(EquipmentSlot::Armor, 0)),
    );

    perform_prestige_with_vault(&mut state, &[EquipmentSlot::Weapon]);

    assert_eq!(state.prestige_rank, 1);
    assert_eq!(state.character_level, 1);
    // Weapon preserved
    let w = state.equipment.get(EquipmentSlot::Weapon);
    assert!(w.is_some());
    assert_eq!(w.as_ref().unwrap().display_name, "Legendary Blade");
    // Armor cleared
    assert!(state.equipment.get(EquipmentSlot::Armor).is_none());
}

#[test]
fn vault_prestige_preserves_multiple_slots() {
    let mut state = make_state_at_level(10, 0);
    state.equipment.set(
        EquipmentSlot::Weapon,
        Some(make_item(EquipmentSlot::Weapon, 0)),
    );
    state.equipment.set(
        EquipmentSlot::Armor,
        Some(make_item(EquipmentSlot::Armor, 0)),
    );
    state
        .equipment
        .set(EquipmentSlot::Ring, Some(make_item(EquipmentSlot::Ring, 5)));

    perform_prestige_with_vault(&mut state, &[EquipmentSlot::Weapon, EquipmentSlot::Ring]);

    assert!(state.equipment.get(EquipmentSlot::Weapon).is_some());
    assert!(state.equipment.get(EquipmentSlot::Ring).is_some());
    assert!(state.equipment.get(EquipmentSlot::Armor).is_none());
}

#[test]
fn vault_prestige_preserves_empty_slot_gracefully() {
    let mut state = make_state_at_level(10, 0);
    // No weapon equipped, but requesting to preserve weapon slot
    perform_prestige_with_vault(&mut state, &[EquipmentSlot::Weapon]);

    assert_eq!(state.prestige_rank, 1);
    assert!(state.equipment.get(EquipmentSlot::Weapon).is_none());
}

#[test]
fn vault_prestige_no_preserved_slots_clears_all() {
    let mut state = make_state_at_level(10, 0);
    state.equipment.set(
        EquipmentSlot::Weapon,
        Some(make_item(EquipmentSlot::Weapon, 0)),
    );

    perform_prestige_with_vault(&mut state, &[]);

    assert_eq!(state.prestige_rank, 1);
    assert_eq!(state.equipment.iter_equipped().count(), 0);
}

#[test]
fn vault_prestige_does_nothing_when_ineligible() {
    let mut state = make_state_at_level(5, 0);
    state.equipment.set(
        EquipmentSlot::Weapon,
        Some(make_item(EquipmentSlot::Weapon, 0)),
    );

    perform_prestige_with_vault(&mut state, &[EquipmentSlot::Weapon]);

    // Should not have prestiged
    assert_eq!(state.prestige_rank, 0);
    assert_eq!(state.character_level, 5);
    // Equipment untouched
    assert!(state.equipment.get(EquipmentSlot::Weapon).is_some());
}

// ═══════════════════════════════════════════════════════════════════
// character/tiers — get_prestige_tier, get_next_prestige_tier, get_adventurer_rank
// (Basic tier names tested in prestige.rs unit tests;
//  these cover high-rank scaling and boundary conditions)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn tier_rank_0_is_none_with_multiplier_1() {
    let tier = get_prestige_tier(0);
    assert_eq!(tier.name, "None");
    assert_eq!(tier.required_level, 0);
    assert!((tier.multiplier - 1.0).abs() < f64::EPSILON);
}

#[test]
fn tier_rank_1_bronze_level_10() {
    let tier = get_prestige_tier(1);
    assert_eq!(tier.name, "Bronze");
    assert_eq!(tier.required_level, 10);
    assert!((tier.multiplier - 1.5).abs() < 0.01);
}

#[test]
fn tier_metals_2_through_4() {
    let t2 = get_prestige_tier(2);
    assert_eq!(t2.name, "Silver");
    assert_eq!(t2.required_level, 25);

    let t3 = get_prestige_tier(3);
    assert_eq!(t3.name, "Gold");
    assert_eq!(t3.required_level, 50);

    let t4 = get_prestige_tier(4);
    assert_eq!(t4.name, "Platinum");
    assert_eq!(t4.required_level, 65);
}

#[test]
fn tier_gems_5_through_9() {
    assert_eq!(get_prestige_tier(5).name, "Diamond");
    assert_eq!(get_prestige_tier(6).name, "Emerald");
    assert_eq!(get_prestige_tier(7).name, "Sapphire");
    assert_eq!(get_prestige_tier(8).name, "Ruby");
    assert_eq!(get_prestige_tier(9).name, "Obsidian");
}

#[test]
fn tier_cosmic_10_through_14() {
    assert_eq!(get_prestige_tier(10).name, "Celestial");
    assert_eq!(get_prestige_tier(11).name, "Astral");
    assert_eq!(get_prestige_tier(12).name, "Cosmic");
    assert_eq!(get_prestige_tier(13).name, "Stellar");
    assert_eq!(get_prestige_tier(14).name, "Galactic");
}

#[test]
fn tier_divine_15_through_19() {
    assert_eq!(get_prestige_tier(15).name, "Transcendent");
    assert_eq!(get_prestige_tier(16).name, "Divine");
    assert_eq!(get_prestige_tier(17).name, "Exalted");
    assert_eq!(get_prestige_tier(18).name, "Mythic");
    assert_eq!(get_prestige_tier(19).name, "Legendary");
}

#[test]
fn tier_eternal_20_and_beyond() {
    assert_eq!(get_prestige_tier(20).name, "Eternal");
    assert_eq!(get_prestige_tier(50).name, "Eternal");
    assert_eq!(get_prestige_tier(100).name, "Eternal");
}

#[test]
fn tier_high_rank_level_scaling() {
    // Rank 19 is the last hard-coded: level 220
    let t19 = get_prestige_tier(19);
    assert_eq!(t19.required_level, 220);

    // Rank 20+: 220 + (rank - 19) * 15
    let t20 = get_prestige_tier(20);
    assert_eq!(t20.required_level, 235);

    let t25 = get_prestige_tier(25);
    assert_eq!(t25.required_level, 220 + (25 - 19) * 15); // 310

    let t30 = get_prestige_tier(30);
    assert_eq!(t30.required_level, 220 + (30 - 19) * 15); // 385
}

#[test]
fn tier_multiplier_increases_monotonically() {
    let mut prev = get_prestige_tier(0).multiplier;
    for rank in 1..=50 {
        let current = get_prestige_tier(rank).multiplier;
        assert!(
            current > prev,
            "Multiplier should increase: P{} ({}) > P{} ({})",
            rank,
            current,
            rank - 1,
            prev
        );
        prev = current;
    }
}

#[test]
fn tier_multiplier_formula_verification() {
    // Formula: 1 + 0.5 * rank^0.7
    for rank in [1u32, 5, 10, 15, 20, 30, 50] {
        let expected = 1.0 + 0.5 * (rank as f64).powf(0.7);
        let actual = get_prestige_tier(rank).multiplier;
        assert!(
            (actual - expected).abs() < 0.001,
            "P{}: expected {:.4}, got {:.4}",
            rank,
            expected,
            actual
        );
    }
}

#[test]
fn tier_required_levels_increase_monotonically() {
    let mut prev_level = 0;
    for rank in 1..=30 {
        let level = get_prestige_tier(rank).required_level;
        assert!(
            level > prev_level,
            "Required level should increase: P{} ({}) > P{} ({})",
            rank,
            level,
            rank - 1,
            prev_level
        );
        prev_level = level;
    }
}

// ── get_next_prestige_tier ─────────────────────────────────────────

#[test]
fn next_tier_from_rank_0_is_bronze() {
    let next = get_next_prestige_tier(0);
    assert_eq!(next.rank, 1);
    assert_eq!(next.name, "Bronze");
    assert_eq!(next.required_level, 10);
}

#[test]
fn next_tier_from_rank_19_is_eternal() {
    let next = get_next_prestige_tier(19);
    assert_eq!(next.rank, 20);
    assert_eq!(next.name, "Eternal");
}

#[test]
fn next_tier_is_always_current_plus_one() {
    for rank in 0..=30 {
        let next = get_next_prestige_tier(rank);
        let direct = get_prestige_tier(rank + 1);
        assert_eq!(next.rank, direct.rank);
        assert_eq!(next.name, direct.name);
        assert_eq!(next.required_level, direct.required_level);
        assert!((next.multiplier - direct.multiplier).abs() < f64::EPSILON);
    }
}

// ── get_adventurer_rank ────────────────────────────────────────────

#[test]
fn adventurer_rank_novice_boundaries() {
    assert_eq!(get_adventurer_rank(0), "Novice");
    assert_eq!(get_adventurer_rank(5), "Novice");
    assert_eq!(get_adventurer_rank(9), "Novice");
}

#[test]
fn adventurer_rank_adept_boundaries() {
    assert_eq!(get_adventurer_rank(10), "Adept");
    assert_eq!(get_adventurer_rank(15), "Adept");
    assert_eq!(get_adventurer_rank(24), "Adept");
}

#[test]
fn adventurer_rank_master_boundaries() {
    assert_eq!(get_adventurer_rank(25), "Master");
    assert_eq!(get_adventurer_rank(37), "Master");
    assert_eq!(get_adventurer_rank(49), "Master");
}

#[test]
fn adventurer_rank_grand_master_boundaries() {
    assert_eq!(get_adventurer_rank(50), "Grand Master");
    assert_eq!(get_adventurer_rank(62), "Grand Master");
    assert_eq!(get_adventurer_rank(74), "Grand Master");
}

#[test]
fn adventurer_rank_legend_boundaries() {
    assert_eq!(get_adventurer_rank(75), "Legend");
    assert_eq!(get_adventurer_rank(88), "Legend");
    assert_eq!(get_adventurer_rank(99), "Legend");
}

#[test]
fn adventurer_rank_mythic_boundaries() {
    assert_eq!(get_adventurer_rank(100), "Mythic");
    assert_eq!(get_adventurer_rank(150), "Mythic");
    assert_eq!(get_adventurer_rank(1000), "Mythic");
}

#[test]
fn adventurer_rank_all_six_values_distinct() {
    let ranks: Vec<&str> = [0u32, 10, 25, 50, 75, 100]
        .iter()
        .map(|&lvl| get_adventurer_rank(lvl))
        .collect();
    // All should be unique
    for i in 0..ranks.len() {
        for j in (i + 1)..ranks.len() {
            assert_ne!(
                ranks[i],
                ranks[j],
                "Ranks at levels {} and {} should differ",
                [0, 10, 25, 50, 75, 100][i],
                [0, 10, 25, 50, 75, 100][j]
            );
        }
    }
}
