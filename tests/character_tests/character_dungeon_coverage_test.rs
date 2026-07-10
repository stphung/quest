//! Integration tests covering gaps in:
//! - character/calculation.rs — calculate_derived_stats()
//! - dungeon/pathfinding.rs — find_path_to, find_next_room

use super::helpers::*;
use quest::character::attributes::{AttributeType, Attributes};
use quest::character::DerivedStats;
use quest::dungeon::types::{
    Dungeon, DungeonSize, Room, RoomState, RoomType, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP,
};
use quest::dungeon::{find_next_room, find_path_to};
use quest::enhancement::enhancement_multiplier;
use quest::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use quest::items::Equipment;

// ============================================================================
// Helpers
// ============================================================================

fn make_attrs_all_ten() -> Attributes {
    Attributes::new()
}

fn make_equipment() -> Equipment {
    Equipment::new()
}

fn no_enhancement() -> [u8; 7] {
    [0u8; 7]
}

/// Creates a Small dungeon grid (5x5) with no rooms placed.
fn empty_small_dungeon() -> Dungeon {
    Dungeon::new(DungeonSize::Small)
}

/// Connects two adjacent rooms: room at (ax, ay) to (bx, by) in the given direction.
/// Both rooms must already be placed in the grid.
fn connect_rooms(dungeon: &mut Dungeon, ax: usize, ay: usize, dir_from_a: usize) {
    // DIR_OFFSETS: UP=(0,-1), RIGHT=(1,0), DOWN=(0,1), LEFT=(-1,0)
    let opposite = match dir_from_a {
        DIR_UP => DIR_DOWN,
        DIR_RIGHT => DIR_LEFT,
        DIR_DOWN => DIR_UP,
        DIR_LEFT => DIR_RIGHT,
        _ => panic!("Invalid direction index"),
    };
    let (dx, dy) = quest::dungeon::types::DIR_OFFSETS[dir_from_a];
    let bx = (ax as i32 + dx) as usize;
    let by = (ay as i32 + dy) as usize;

    if let Some(room) = dungeon.get_room_mut(ax, ay) {
        room.connections[dir_from_a] = true;
    }
    if let Some(room) = dungeon.get_room_mut(bx, by) {
        room.connections[opposite] = true;
    }
}

/// Places a room in the grid and sets its state.
fn place_room(dungeon: &mut Dungeon, x: usize, y: usize, room_type: RoomType, state: RoomState) {
    let mut room = Room::new(room_type, (x, y));
    room.state = state;
    dungeon.grid[y][x] = Some(room);
}

// ============================================================================
// Part 1: character/calculation.rs — calculate_derived_stats()
// ============================================================================

// --- Base stats ---

#[test]
fn test_calc_base_stats_all_attributes_ten() {
    // All attributes at 10 → modifier 0 for all
    let attrs = make_attrs_all_ten();
    let equipment = make_equipment();
    let levels = no_enhancement();

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels);

    // max_hp = BASE_HP + (0 * HP_PER_CON_MODIFIER) = 50
    assert_eq!(stats.max_hp, 50);
    // physical_damage = BASE_PHYSICAL_DAMAGE + (0 * DAMAGE_PER_STR_MODIFIER) = 5
    assert_eq!(stats.physical_damage, 5);
    // magic_damage = BASE_MAGIC_DAMAGE + (0 * DAMAGE_PER_INT_MODIFIER) = 5
    assert_eq!(stats.magic_damage, 5);
    // defense = 0 + (0 * 1) = 0
    assert_eq!(stats.defense, 0);
    // crit_chance = BASE_CRIT_CHANCE_PERCENT + (0 * 1) = 5
    assert_eq!(stats.crit_chance_percent, 5);
    // crit_multiplier = BASE_CRIT_MULTIPLIER = 2.0
    assert!((stats.crit_multiplier - 2.0).abs() < f64::EPSILON);
    // attack_speed_multiplier base = 1.0
    assert!((stats.attack_speed_multiplier - 1.0).abs() < f64::EPSILON);
    // hp_regen_multiplier base = 1.0
    assert!((stats.hp_regen_multiplier - 1.0).abs() < f64::EPSILON);
    // damage_reflection_percent base = 0.0
    assert!((stats.damage_reflection_percent - 0.0).abs() < f64::EPSILON);
    // xp_multiplier = 1.0 + (0 * 0.05) = 1.0
    assert!((stats.xp_multiplier - 1.0).abs() < f64::EPSILON);
}

// --- Attribute effect tests ---

#[test]
fn test_calc_high_str_increases_physical_damage() {
    // STR 20 → modifier = (20-10)/2 = 5
    // physical_damage = 5 + (5 * 2) = 15
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Strength, 20);

    let stats = DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    assert_eq!(stats.physical_damage, 15);
    // Magic and other stats unchanged
    assert_eq!(stats.magic_damage, 5);
    assert_eq!(stats.max_hp, 50);
}

#[test]
fn test_calc_high_con_increases_max_hp() {
    // CON 20 → modifier = (20-10)/2 = 5
    // max_hp = 50 + (5 * 10) = 100
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Constitution, 20);

    let stats = DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    assert_eq!(stats.max_hp, 100);
    assert_eq!(stats.physical_damage, 5); // Unchanged
}

#[test]
fn test_calc_high_dex_increases_defense_and_crit() {
    // DEX 16 → modifier = (16-10)/2 = 3
    // defense = 3
    // crit_chance = 5 + 3 = 8
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Dexterity, 16);

    let stats = DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    assert_eq!(stats.defense, 3);
    assert_eq!(stats.crit_chance_percent, 8);
}

#[test]
fn test_calc_high_int_increases_magic_damage() {
    // INT 14 → modifier = (14-10)/2 = 2
    // magic_damage = 5 + (2 * 2) = 9
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Intelligence, 14);

    let stats = DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    assert_eq!(stats.magic_damage, 9);
    assert_eq!(stats.physical_damage, 5); // Unchanged
}

#[test]
fn test_calc_high_wis_increases_xp_multiplier() {
    // WIS 20 → modifier = (20-10)/2 = 5
    // xp_multiplier = 1.0 + (5 * 0.05) = 1.25
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Wisdom, 20);

    let stats = DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    assert!((stats.xp_multiplier - 1.25).abs() < 1e-9);
}

#[test]
fn test_calc_multiple_high_attributes() {
    // STR 16 (+3 mod) → physical_damage = 5 + 6 = 11
    // DEX 18 (+4 mod) → defense = 4, crit = 5+4 = 9
    // CON 14 (+2 mod) → max_hp = 50 + 20 = 70
    // INT 12 (+1 mod) → magic_damage = 5 + 2 = 7
    // WIS 20 (+5 mod) → xp_multiplier = 1.0 + 0.25 = 1.25
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Strength, 16);
    attrs.set(AttributeType::Dexterity, 18);
    attrs.set(AttributeType::Constitution, 14);
    attrs.set(AttributeType::Intelligence, 12);
    attrs.set(AttributeType::Wisdom, 20);

    let stats = DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    assert_eq!(stats.max_hp, 70);
    assert_eq!(stats.physical_damage, 11);
    assert_eq!(stats.magic_damage, 7);
    assert_eq!(stats.defense, 4);
    assert_eq!(stats.crit_chance_percent, 9);
    assert!((stats.xp_multiplier - 1.25).abs() < 1e-9);
}

// --- Individual affix type tests ---

#[test]
fn test_calc_affix_damage_percent_multiplies_damage() {
    // DamagePercent 50.0 → damage_mult = 1 + 50/100 = 1.5
    // physical_damage = 5 * 1.5 = 7 (floor), magic_damage = 5 * 1.5 = 7 (floor)
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::DamagePercent,
            50.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    // 5 * 1.5 = 7.5 → truncated to 7
    assert_eq!(stats.physical_damage, 7);
    assert_eq!(stats.magic_damage, 7);
}

#[test]
fn test_calc_affix_crit_chance_adds_to_crit() {
    // CritChance 15.0 → crit_chance = 5 + 15 = 20
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Ring,
        Some(item_with_affix(
            EquipmentSlot::Ring,
            AffixType::CritChance,
            15.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.crit_chance_percent, 20);
}

#[test]
fn test_calc_affix_crit_multiplier_adds_to_crit_mult() {
    // CritMultiplier 50.0 → crit_multiplier = 2.0 + 50/100 = 2.5
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::CritMultiplier,
            50.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.crit_multiplier - 2.5).abs() < 1e-9);
}

#[test]
fn test_calc_affix_attack_speed_increases_multiplier() {
    // AttackSpeed 25.0 → attack_speed_multiplier = 1.0 + 25/100 = 1.25
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Gloves,
        Some(item_with_affix(
            EquipmentSlot::Gloves,
            AffixType::AttackSpeed,
            25.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.attack_speed_multiplier - 1.25).abs() < 1e-9);
}

#[test]
fn test_calc_affix_hp_bonus_adds_flat_hp() {
    // HPBonus 30.0 → max_hp = 50 + 30 = 80
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::HPBonus,
            30.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.max_hp, 80);
}

#[test]
fn test_calc_affix_damage_reduction_multiplies_defense_base_zero() {
    // Base defense is 0 (DEX modifier is 0). DamageReduction 20.0 → defense = 0 * 1.2 = 0
    // Defense only matters when DEX provides a non-zero base.
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::DamageReduction,
            20.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.defense, 0);
}

#[test]
fn test_calc_affix_damage_reduction_with_dex_defense() {
    // DEX 20 → modifier 5 → base defense = 5
    // DamageReduction 100.0 → defense_mult = 1 + 100/100 = 2.0 → defense = 5 * 2 = 10
    let mut attrs = make_attrs_all_ten();
    attrs.set(AttributeType::Dexterity, 20);
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::DamageReduction,
            100.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.defense, 10);
}

#[test]
fn test_calc_affix_hp_regen_increases_regen_multiplier() {
    // HPRegen 50.0 → hp_regen_multiplier = 1.0 + 50/100 = 1.5
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Boots,
        Some(item_with_affix(
            EquipmentSlot::Boots,
            AffixType::HPRegen,
            50.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.hp_regen_multiplier - 1.5).abs() < 1e-9);
}

#[test]
fn test_calc_affix_damage_reflection_sets_percent() {
    // DamageReflection 25.0 → damage_reflection_percent = 25.0
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::DamageReflection,
            25.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.damage_reflection_percent - 25.0).abs() < 1e-9);
}

#[test]
fn test_calc_affix_xp_gain_multiplies_xp_multiplier() {
    // XPGain 100.0 → xp_mult = 1 + 100/100 = 2.0 → xp_multiplier = 1.0 * 2.0 = 2.0
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Amulet,
        Some(item_with_affix(
            EquipmentSlot::Amulet,
            AffixType::XPGain,
            100.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.xp_multiplier - 2.0).abs() < 1e-9);
}

#[test]
fn test_calc_affix_unknown_has_no_effect() {
    // Unknown affixes are ignored — stats should match the baseline exactly.
    let attrs = make_attrs_all_ten();
    let baseline =
        DerivedStats::calculate_derived_stats(&attrs, &make_equipment(), &no_enhancement());

    let mut equipment = make_equipment();
    let item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Rare,
        ilvl: 50,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test Item".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::Unknown,
            value: 999.0,
        }],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Ring, Some(item));

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.max_hp, baseline.max_hp);
    assert_eq!(stats.physical_damage, baseline.physical_damage);
    assert_eq!(stats.magic_damage, baseline.magic_damage);
    assert_eq!(stats.defense, baseline.defense);
    assert_eq!(stats.crit_chance_percent, baseline.crit_chance_percent);
    assert!((stats.crit_multiplier - baseline.crit_multiplier).abs() < 1e-9);
    assert!((stats.attack_speed_multiplier - baseline.attack_speed_multiplier).abs() < 1e-9);
    assert!((stats.hp_regen_multiplier - baseline.hp_regen_multiplier).abs() < 1e-9);
    assert!((stats.damage_reflection_percent - baseline.damage_reflection_percent).abs() < 1e-9);
    assert!((stats.xp_multiplier - baseline.xp_multiplier).abs() < 1e-9);
}

// --- Enhancement level scaling ---

#[test]
fn test_calc_enhancement_level_zero_is_one_times() {
    // enhancement_multiplier(0) == 1.0 — no boost
    assert!((enhancement_multiplier(0) - 1.0).abs() < 1e-9);
}

#[test]
fn test_calc_enhancement_level_increases_multiplier() {
    // Each level from 1-10 should increase the multiplier above 1.0
    for level in 1u8..=10 {
        let mult = enhancement_multiplier(level);
        assert!(
            mult > 1.0,
            "enhancement_multiplier({level}) = {mult} should be > 1.0"
        );
    }
}

#[test]
fn test_calc_enhancement_scales_affix_value() {
    // Weapon slot (index 0) at enhancement level 5
    // enhancement_multiplier(5) = 1.0 + 30/100 = 1.30
    // DamagePercent affix value=100.0 → scaled_value = 100.0 * 1.30 = 130.0
    // damage_mult = 1 + 130/100 = 2.30
    // physical_damage = 5 * 2.30 = 11 (floor)
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::DamagePercent,
            100.0,
        )),
    );

    // No enhancement
    let levels_zero = no_enhancement();
    let stats_zero = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels_zero);

    // Level 5 on weapon slot (index 0)
    let mut levels_five = no_enhancement();
    levels_five[0] = 5;
    let stats_five = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels_five);

    // Enhanced version must yield strictly higher damage
    assert!(
        stats_five.physical_damage > stats_zero.physical_damage,
        "Enhancement level 5 should increase damage: {} vs {}",
        stats_five.physical_damage,
        stats_zero.physical_damage
    );
    assert!(
        stats_five.magic_damage > stats_zero.magic_damage,
        "Enhancement level 5 should increase magic damage: {} vs {}",
        stats_five.magic_damage,
        stats_zero.magic_damage
    );
}

#[test]
fn test_calc_enhancement_scales_equipment_attribute_bonuses() {
    // An item with +4 STR on weapon slot (index 0), enhancement level 4
    // enhancement_multiplier(4) = 1.0 + 20/100 = 1.20
    // STR bonus from item = round(4 * 1.20) = round(4.8) = 5
    // total STR = 10 + 5 = 15, modifier = (15-10)/2 = 2
    // physical_damage = 5 + (2 * 2) = 9
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Rare,
        ilvl: 50,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Sword".to_string(),
        attributes: AttributeBonuses {
            str: 4,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));

    // No enhancement: STR=10+4=14, mod=2, pdmg=5+4=9
    let stats_zero = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());
    assert_eq!(stats_zero.physical_damage, 9);

    // Enhancement 4: bonus = round(4 * 1.20) = 5, STR=10+5=15, mod=2, pdmg=5+4=9
    // Actually mod(15) = (15-10)/2 = 2, same as without enhancement at this scale.
    // Let's verify enhancement 10 definitely makes a difference (multiplier=2.5, bonus=round(10)=10, STR=20, mod=5, pdmg=5+10=15)
    let mut levels_ten = no_enhancement();
    levels_ten[0] = 10;
    let stats_ten = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels_ten);

    // With level 10: STR bonus = round(4 * 2.5) = 10, total STR = 20, mod = 5
    // physical_damage = 5 + (5 * 2) = 15
    assert_eq!(stats_ten.physical_damage, 15);
    assert!(
        stats_ten.physical_damage > stats_zero.physical_damage,
        "Enhancement 10 should yield higher physical_damage"
    );
}

#[test]
fn test_calc_enhancement_only_affects_slot_with_nonzero_level() {
    // Two items: weapon slot (index 0) at level 5, armor slot (index 1) at level 0
    // Weapon has DamagePercent 100.0, Armor has HPBonus 50.0
    // Only the weapon affix should be scaled up.
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::DamagePercent,
            100.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::HPBonus,
            50.0,
        )),
    );

    // All at zero
    let stats_zero = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    // Level 5 on weapon only
    let mut levels = no_enhancement();
    levels[0] = 5;
    let stats_enhanced = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels);

    // Weapon damage should increase due to enhanced DamagePercent
    assert!(stats_enhanced.physical_damage > stats_zero.physical_damage);
    // HP bonus from armor is still unenhanced (same as zero level)
    // For the armor slot at level 0: same hp_bonus value should apply.
    // Verify by checking that hp is still 50+50=100 (no armor enhancement)
    assert_eq!(stats_zero.max_hp, 100); // 50 base + 50 bonus
    assert_eq!(stats_enhanced.max_hp, 100); // HP unchanged since armor slot is level 0
}

// --- Multiple affixes stacking ---

#[test]
fn test_calc_multiple_damage_percent_affixes_multiply() {
    // Two items with DamagePercent 50.0 each → damage_mult = 1.5 * 1.5 = 2.25
    // physical_damage = floor(5 * 2.25) = 11
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::DamagePercent,
            50.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Ring,
        Some(item_with_affix(
            EquipmentSlot::Ring,
            AffixType::DamagePercent,
            50.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.physical_damage, 11);
    assert_eq!(stats.magic_damage, 11);
}

#[test]
fn test_calc_multiple_crit_chance_affixes_add() {
    // CritChance from weapon (10.0) + ring (5.0) → crit = 5 + 10 + 5 = 20
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::CritChance,
            10.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Ring,
        Some(item_with_affix(
            EquipmentSlot::Ring,
            AffixType::CritChance,
            5.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert_eq!(stats.crit_chance_percent, 20);
}

#[test]
fn test_calc_multiple_damage_reflection_affixes_add() {
    // Armor 20% + Helmet 15% = 35% reflection
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::DamageReflection,
            20.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Helmet,
        Some(item_with_affix(
            EquipmentSlot::Helmet,
            AffixType::DamageReflection,
            15.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.damage_reflection_percent - 35.0).abs() < 1e-9);
}

#[test]
fn test_calc_multiple_xp_gain_affixes_multiply() {
    // XPGain 100.0 from two slots → xp_mult = 2.0 * 2.0 = 4.0
    // xp_multiplier = 1.0 * 4.0 = 4.0
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();
    equipment.set(
        EquipmentSlot::Amulet,
        Some(item_with_affix(
            EquipmentSlot::Amulet,
            AffixType::XPGain,
            100.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Ring,
        Some(item_with_affix(
            EquipmentSlot::Ring,
            AffixType::XPGain,
            100.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    assert!((stats.xp_multiplier - 4.0).abs() < 1e-9);
}

#[test]
fn test_calc_all_affix_types_simultaneously() {
    // One item per relevant slot, each with a different affix type.
    let attrs = make_attrs_all_ten();
    let mut equipment = make_equipment();

    equipment.set(
        EquipmentSlot::Weapon,
        Some(item_with_affix(
            EquipmentSlot::Weapon,
            AffixType::DamagePercent,
            100.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Armor,
        Some(item_with_affix(
            EquipmentSlot::Armor,
            AffixType::HPBonus,
            50.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Helmet,
        Some(item_with_affix(
            EquipmentSlot::Helmet,
            AffixType::CritChance,
            10.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Gloves,
        Some(item_with_affix(
            EquipmentSlot::Gloves,
            AffixType::AttackSpeed,
            50.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Boots,
        Some(item_with_affix(
            EquipmentSlot::Boots,
            AffixType::HPRegen,
            100.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Amulet,
        Some(item_with_affix(
            EquipmentSlot::Amulet,
            AffixType::XPGain,
            50.0,
        )),
    );
    equipment.set(
        EquipmentSlot::Ring,
        Some(item_with_affix(
            EquipmentSlot::Ring,
            AffixType::CritMultiplier,
            100.0,
        )),
    );

    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &no_enhancement());

    // physical_damage = floor(5 * 2.0) = 10 (DamagePercent 100%)
    assert_eq!(stats.physical_damage, 10);
    assert_eq!(stats.magic_damage, 10);
    // max_hp = 50 + 50 = 100 (HPBonus 50)
    assert_eq!(stats.max_hp, 100);
    // crit_chance = 5 + 10 = 15 (CritChance 10)
    assert_eq!(stats.crit_chance_percent, 15);
    // attack_speed = 1.0 + 50/100 = 1.5 (AttackSpeed 50)
    assert!((stats.attack_speed_multiplier - 1.5).abs() < 1e-9);
    // hp_regen = 1.0 + 100/100 = 2.0 (HPRegen 100)
    assert!((stats.hp_regen_multiplier - 2.0).abs() < 1e-9);
    // xp_multiplier = 1.0 * (1+50/100) = 1.5 (XPGain 50)
    assert!((stats.xp_multiplier - 1.5).abs() < 1e-9);
    // crit_multiplier = 2.0 + 100/100 = 3.0 (CritMultiplier 100)
    assert!((stats.crit_multiplier - 3.0).abs() < 1e-9);
}

// ============================================================================
// Part 2: dungeon/pathfinding.rs — find_path_to, find_next_room
// ============================================================================

// ============================================================================
// find_path_to tests
// ============================================================================

#[test]
fn test_find_path_same_position_returns_single_element() {
    // A dungeon with a single room. Path from (1,1) to (1,1) = [(1,1)]
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    dungeon.player_position = (1, 1);

    let path = find_path_to(&dungeon, (1, 1), (1, 1));

    assert!(path.is_some(), "Path from a room to itself should exist");
    let path = path.unwrap();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0], (1, 1));
}

#[test]
fn test_find_path_adjacent_connected_rooms() {
    // Two rooms at (1,1) and (2,1) connected right/left.
    // Path from (1,1) to (2,1) should have length 2.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);

    let path = find_path_to(&dungeon, (1, 1), (2, 1));

    assert!(
        path.is_some(),
        "Adjacent connected rooms should have a path"
    );
    let path = path.unwrap();
    assert_eq!(path.len(), 2);
    assert_eq!(path[0], (1, 1));
    assert_eq!(path[1], (2, 1));
}

#[test]
fn test_find_path_no_connection_returns_none() {
    // Two rooms with no connections between them — no path.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 3, 1, RoomType::Combat, RoomState::Revealed);
    // No connections set — rooms are isolated.

    let path = find_path_to(&dungeon, (1, 1), (3, 1));

    assert!(path.is_none(), "Disconnected rooms should have no path");
}

#[test]
fn test_find_path_multi_hop_through_cleared_rooms() {
    // Chain: (1,1) — (2,1) — (3,1)
    // (1,1) is Current, (2,1) is Cleared, (3,1) is Revealed
    // Path from (1,1) to (3,1) should be length 3.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Cleared);
    place_room(&mut dungeon, 3, 1, RoomType::Combat, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 2, 1, DIR_RIGHT);
    dungeon.player_position = (1, 1);

    let path = find_path_to(&dungeon, (1, 1), (3, 1));

    assert!(path.is_some(), "3-room chain should have a path");
    let path = path.unwrap();
    assert_eq!(path.len(), 3);
    assert_eq!(path[0], (1, 1));
    assert_eq!(path[1], (2, 1));
    assert_eq!(path[2], (3, 1));
}

#[test]
fn test_find_path_cannot_traverse_hidden_rooms() {
    // Chain: (1,1) — (2,1) — (3,1), but (2,1) is Hidden.
    // BFS should not pass through Hidden rooms, so no path exists.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Hidden);
    place_room(&mut dungeon, 3, 1, RoomType::Combat, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 2, 1, DIR_RIGHT);

    let path = find_path_to(&dungeon, (1, 1), (3, 1));

    assert!(
        path.is_none(),
        "Hidden rooms should block pathfinding traversal"
    );
}

#[test]
fn test_find_path_can_traverse_revealed_rooms_as_intermediate() {
    // Chain: (1,1) Current — (2,1) Revealed — (3,1) Revealed
    // A Revealed intermediate room is traversable.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Revealed);
    place_room(&mut dungeon, 3, 1, RoomType::Treasure, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 2, 1, DIR_RIGHT);

    let path = find_path_to(&dungeon, (1, 1), (3, 1));

    assert!(
        path.is_some(),
        "Revealed intermediate rooms are traversable"
    );
    assert_eq!(path.unwrap().len(), 3);
}

#[test]
fn test_find_path_two_hops_vertical() {
    // Vertical chain: (2,1) — (2,2) — (2,3)
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 2, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 2, RoomType::Combat, RoomState::Cleared);
    place_room(&mut dungeon, 2, 3, RoomType::Treasure, RoomState::Revealed);
    connect_rooms(&mut dungeon, 2, 1, DIR_DOWN);
    connect_rooms(&mut dungeon, 2, 2, DIR_DOWN);
    dungeon.player_position = (2, 1);

    let path = find_path_to(&dungeon, (2, 1), (2, 3));

    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 3);
    assert_eq!(path[0], (2, 1));
    assert_eq!(path[1], (2, 2));
    assert_eq!(path[2], (2, 3));
}

#[test]
fn test_find_path_l_shaped_route() {
    // L-shape: (1,1) → right → (2,1) → down → (2,2)
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Cleared);
    place_room(&mut dungeon, 2, 2, RoomType::Boss, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 2, 1, DIR_DOWN);
    dungeon.player_position = (1, 1);
    dungeon.boss_position = (2, 2);

    let path = find_path_to(&dungeon, (1, 1), (2, 2));

    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), 3);
}

// ============================================================================
// find_next_room tests
// ============================================================================

#[test]
fn test_find_next_room_returns_none_when_no_revealed_rooms() {
    // Only a single Current room with no revealed neighbors.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 2, 2, RoomType::Entrance, RoomState::Current);
    dungeon.player_position = (2, 2);
    dungeon.entrance_position = (2, 2);
    dungeon.boss_position = (2, 2); // No real boss in this test

    let next = find_next_room(&dungeon);

    assert!(next.is_none(), "No revealed rooms means no next room");
}

#[test]
fn test_find_next_room_goes_to_adjacent_revealed_room() {
    // Current room at (2,2), revealed neighbor at (3,2).
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 2, 2, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 3, 2, RoomType::Combat, RoomState::Revealed);
    connect_rooms(&mut dungeon, 2, 2, DIR_RIGHT);
    dungeon.player_position = (2, 2);
    dungeon.entrance_position = (2, 2);
    dungeon.boss_position = (4, 4); // Far away boss, not reachable

    let next = find_next_room(&dungeon);

    assert_eq!(next, Some((3, 2)), "Should navigate to the revealed room");
}

#[test]
fn test_find_next_room_skips_boss_without_key() {
    // Current room at (1,1). Boss room at (2,1) is Revealed but player has no key.
    // No other revealed rooms. Should return None.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Boss, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (2, 1);
    dungeon.has_key = false;

    let next = find_next_room(&dungeon);

    assert!(
        next.is_none(),
        "Without key, boss room should be skipped and no other room exists"
    );
}

#[test]
fn test_find_next_room_goes_to_boss_with_key() {
    // Current room at (1,1). Boss room at (2,1) is Revealed and player has key.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Boss, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (2, 1);
    dungeon.has_key = true;

    let next = find_next_room(&dungeon);

    assert_eq!(next, Some((2, 1)), "With key, should navigate toward boss");
}

#[test]
fn test_find_next_room_skips_cleared_boss_even_with_key() {
    // Boss room is already Cleared — should not re-enter it.
    // Another revealed room exists and should be the target instead.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Boss, RoomState::Cleared);
    place_room(&mut dungeon, 1, 2, RoomType::Treasure, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 1, 1, DIR_DOWN);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (2, 1);
    dungeon.has_key = true;

    let next = find_next_room(&dungeon);

    assert_eq!(
        next,
        Some((1, 2)),
        "Cleared boss room should not be re-entered; should go to Treasure room"
    );
}

#[test]
fn test_find_next_room_prioritizes_nearest_revealed_room() {
    // Current at (1,1). Two revealed rooms: (2,1) and (3,1).
    // (2,1) is directly adjacent; (3,1) is two hops away.
    // Should pick (2,1) as the next step toward the closest unexplored room.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Revealed);
    place_room(&mut dungeon, 3, 1, RoomType::Treasure, RoomState::Revealed);
    // (2,1) connects to both (1,1) and (3,1)
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 2, 1, DIR_RIGHT);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (0, 4); // Far away boss

    let next = find_next_room(&dungeon);

    // The nearest revealed room is (2,1) — one hop away.
    // find_next_room returns the first step along the shortest path.
    assert_eq!(
        next,
        Some((2, 1)),
        "Should step toward the nearest revealed room"
    );
}

#[test]
fn test_find_next_room_with_key_prioritizes_boss_over_other_revealed() {
    // Has key, boss at (2,1) Revealed, another room at (1,2) Revealed.
    // Boss should take priority.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Boss, RoomState::Revealed);
    place_room(&mut dungeon, 1, 2, RoomType::Combat, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 1, 1, DIR_DOWN);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (2, 1);
    dungeon.has_key = true;

    let next = find_next_room(&dungeon);

    assert_eq!(
        next,
        Some((2, 1)),
        "With key, boss room should take priority over other revealed rooms"
    );
}

#[test]
fn test_find_next_room_returns_first_step_not_destination() {
    // Chain: (1,1) Current — (2,1) Cleared — (3,1) Revealed
    // Boss is far away with no key. The only revealed target is (3,1).
    // The first step should be (2,1), not (3,1).
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Cleared);
    place_room(&mut dungeon, 3, 1, RoomType::Treasure, RoomState::Revealed);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    connect_rooms(&mut dungeon, 2, 1, DIR_RIGHT);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (0, 4);
    dungeon.has_key = false;

    let next = find_next_room(&dungeon);

    assert_eq!(
        next,
        Some((2, 1)),
        "find_next_room returns the first step toward the target, not the target itself"
    );
}

#[test]
fn test_find_next_room_all_cleared_no_targets() {
    // All reachable rooms are Cleared except the Current one.
    // No Revealed rooms → nothing to explore.
    let mut dungeon = empty_small_dungeon();
    place_room(&mut dungeon, 1, 1, RoomType::Entrance, RoomState::Current);
    place_room(&mut dungeon, 2, 1, RoomType::Combat, RoomState::Cleared);
    connect_rooms(&mut dungeon, 1, 1, DIR_RIGHT);
    dungeon.player_position = (1, 1);
    dungeon.entrance_position = (1, 1);
    dungeon.boss_position = (4, 4);
    dungeon.has_key = false;

    let next = find_next_room(&dungeon);

    assert!(
        next.is_none(),
        "No revealed rooms to explore should return None"
    );
}
