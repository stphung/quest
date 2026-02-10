#![allow(dead_code)]
use super::names::generate_display_name;
use super::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use rand::Rng;

/// Generate an item with the given slot, rarity, and item level.
/// ilvl determines stat scaling: ilvl 10 (zone 1) to ilvl 100 (zone 10).
pub fn generate_item(slot: EquipmentSlot, rarity: Rarity, ilvl: u32) -> Item {
    let mut rng = rand::thread_rng();

    // Generate attribute bonuses based on rarity and ilvl
    let attributes = generate_attributes(rarity, ilvl, &mut rng);

    // Generate affixes based on rarity and ilvl
    let affixes = generate_affixes(rarity, ilvl, &mut rng);

    let mut item = Item {
        slot,
        rarity,
        ilvl,
        base_name: String::new(),
        display_name: String::new(),
        attributes,
        affixes,
    };

    item.display_name = generate_display_name(&item);
    item.base_name = item.display_name.clone();

    item
}

/// Calculate the ilvl multiplier for scaling stats.
/// ilvl 10: 1.0x, ilvl 50: 2.33x, ilvl 100: 4.0x
fn ilvl_multiplier(ilvl: u32) -> f64 {
    1.0 + (ilvl.max(10) as f64 - 10.0) / 30.0
}

fn generate_attributes(rarity: Rarity, ilvl: u32, rng: &mut impl Rng) -> AttributeBonuses {
    // Base ranges at ilvl 10 (reduced from original)
    let (base_min, base_max) = match rarity {
        Rarity::Common => (1, 1),
        Rarity::Magic => (1, 2),
        Rarity::Rare => (2, 3),
        Rarity::Epic => (3, 4),
        Rarity::Legendary => (4, 6),
    };

    let multiplier = ilvl_multiplier(ilvl);

    // Pick a random number of attributes to boost (1-3)
    let num_attrs = rng.gen_range(1..=3);
    let mut attrs = AttributeBonuses::new();

    for _ in 0..num_attrs {
        let base_value = rng.gen_range(base_min..=base_max) as f64;
        let scaled_value = (base_value * multiplier).round() as u32;
        let value = scaled_value.max(1); // Minimum 1

        match rng.gen_range(0..6) {
            0 => attrs.str += value,
            1 => attrs.dex += value,
            2 => attrs.con += value,
            3 => attrs.int += value,
            4 => attrs.wis += value,
            5 => attrs.cha += value,
            _ => unreachable!(),
        }
    }

    attrs
}

fn generate_affixes(rarity: Rarity, ilvl: u32, rng: &mut impl Rng) -> Vec<Affix> {
    let count = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => rng.gen_range(2..=3),
        Rarity::Epic => rng.gen_range(3..=4),
        Rarity::Legendary => rng.gen_range(4..=5),
    };

    let mut affixes = Vec::new();
    let all_affix_types = [
        AffixType::DamagePercent,
        AffixType::CritChance,
        AffixType::CritMultiplier,
        AffixType::AttackSpeed,
        AffixType::HPBonus,
        AffixType::DamageReduction,
        AffixType::HPRegen,
        AffixType::DamageReflection,
        AffixType::XPGain,
    ];

    for _ in 0..count {
        let affix_type = all_affix_types[rng.gen_range(0..all_affix_types.len())];
        let value = generate_affix_value(affix_type, rarity, ilvl, rng);
        affixes.push(Affix { affix_type, value });
    }

    affixes
}

fn generate_affix_value(
    affix_type: AffixType,
    rarity: Rarity,
    ilvl: u32,
    rng: &mut impl Rng,
) -> f64 {
    let multiplier = ilvl_multiplier(ilvl);

    // Base ranges at ilvl 10 (significantly reduced from original)
    let (base_min, base_max) = match rarity {
        Rarity::Common => (0.0, 0.0),
        Rarity::Magic => (1.0, 3.0),
        Rarity::Rare => (2.0, 4.0),
        Rarity::Epic => (4.0, 6.0),
        Rarity::Legendary => (6.0, 10.0),
    };

    match affix_type {
        AffixType::HPBonus => {
            // Flat HP bonus uses different base ranges
            let (hp_min, hp_max) = match rarity {
                Rarity::Common => (0.0, 0.0),
                Rarity::Magic => (10.0, 20.0),
                Rarity::Rare => (20.0, 35.0),
                Rarity::Epic => (30.0, 50.0),
                Rarity::Legendary => (50.0, 80.0),
            };
            let base = rng.gen_range(hp_min..=hp_max);
            (base * multiplier).round()
        }
        AffixType::CritMultiplier => {
            // Crit multiplier bonus as percentage points (e.g., 20.0 = +20% crit mult)
            // Applied as: crit_mult = 2.0 + (value / 100.0)
            let (cm_min, cm_max) = match rarity {
                Rarity::Common => (0.0, 0.0),
                Rarity::Magic => (5.0, 10.0),
                Rarity::Rare => (10.0, 15.0),
                Rarity::Epic => (15.0, 25.0),
                Rarity::Legendary => (20.0, 35.0),
            };
            let base = rng.gen_range(cm_min..=cm_max);
            (base * multiplier).round()
        }
        _ => {
            // Percentage affixes
            let base = rng.gen_range(base_min..=base_max);
            (base * multiplier).round()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ilvl_multiplier() {
        assert!((ilvl_multiplier(10) - 1.0).abs() < 0.01);
        assert!((ilvl_multiplier(40) - 2.0).abs() < 0.01);
        assert!((ilvl_multiplier(70) - 3.0).abs() < 0.01);
        assert!((ilvl_multiplier(100) - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_common_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 10);
        assert_eq!(item.rarity, Rarity::Common);
        assert_eq!(item.slot, EquipmentSlot::Weapon);
        assert_eq!(item.ilvl, 10);
        assert_eq!(item.affixes.len(), 0);
        assert!(item.attributes.total() > 0);
    }

    #[test]
    fn test_generate_magic_item_has_affix() {
        let item = generate_item(EquipmentSlot::Armor, Rarity::Magic, 50);
        assert_eq!(item.rarity, Rarity::Magic);
        assert_eq!(item.ilvl, 50);
        assert_eq!(item.affixes.len(), 1);
    }

    #[test]
    fn test_generate_rare_item_has_multiple_affixes() {
        let item = generate_item(EquipmentSlot::Helmet, Rarity::Rare, 100);
        assert_eq!(item.rarity, Rarity::Rare);
        assert!(item.affixes.len() >= 2 && item.affixes.len() <= 3);
    }

    #[test]
    fn test_generate_legendary_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 100);
        assert_eq!(item.rarity, Rarity::Legendary);
        assert_eq!(item.ilvl, 100);
        assert!(item.affixes.len() >= 4 && item.affixes.len() <= 5);
    }

    #[test]
    fn test_item_has_display_name() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Magic, 50);
        assert!(!item.display_name.is_empty());
    }

    #[test]
    fn test_higher_ilvl_stronger_items() {
        // Over many samples, ilvl 100 should produce higher average totals than ilvl 10
        let sample = |ilvl: u32| -> f64 {
            let sum: u32 = (0..100)
                .map(|_| {
                    generate_item(EquipmentSlot::Weapon, Rarity::Rare, ilvl)
                        .attributes
                        .total()
                })
                .sum();
            sum as f64 / 100.0
        };

        let ilvl_10_avg = sample(10);
        let ilvl_50_avg = sample(50);
        let ilvl_100_avg = sample(100);

        assert!(
            ilvl_10_avg < ilvl_50_avg,
            "ilvl 10 ({ilvl_10_avg}) should be < ilvl 50 ({ilvl_50_avg})"
        );
        assert!(
            ilvl_50_avg < ilvl_100_avg,
            "ilvl 50 ({ilvl_50_avg}) should be < ilvl 100 ({ilvl_100_avg})"
        );
    }

    #[test]
    fn test_ilvl_100_legendary_reasonable_values() {
        // Verify ilvl 100 legendaries are in expected range
        for _ in 0..20 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 100);

            // Attributes: 4-6 base * 4.0 multiplier * 1-3 stats = 16-72 total
            assert!(
                item.attributes.total() >= 16,
                "Legendary attrs too low: {}",
                item.attributes.total()
            );
            assert!(
                item.attributes.total() <= 80,
                "Legendary attrs too high: {}",
                item.attributes.total()
            );

            // Check affix values are reasonable
            for affix in &item.affixes {
                match affix.affix_type {
                    AffixType::HPBonus => {
                        // 50-80 base * 4.0 = 200-320
                        assert!(
                            affix.value >= 150.0 && affix.value <= 350.0,
                            "HP bonus out of range: {}",
                            affix.value
                        );
                    }
                    AffixType::CritMultiplier => {
                        // 20-35 base * 4.0 = 80-140 (percentage points)
                        assert!(
                            affix.value >= 70.0 && affix.value <= 150.0,
                            "Crit mult out of range: {}",
                            affix.value
                        );
                    }
                    _ => {
                        // 6-10 base * 4.0 = 24-40%
                        assert!(
                            affix.value >= 20.0 && affix.value <= 45.0,
                            "Affix {:?} out of range: {}",
                            affix.affix_type,
                            affix.value
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_ilvl_10_items_weak() {
        // Verify ilvl 10 items are appropriately weak
        for _ in 0..20 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 10);

            // Attributes: 4-6 base * 1.0 multiplier * 1-3 stats = 4-18 total
            assert!(
                item.attributes.total() >= 4,
                "Legendary attrs too low: {}",
                item.attributes.total()
            );
            assert!(
                item.attributes.total() <= 20,
                "Legendary attrs too high at ilvl 10: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_common_items_never_have_affixes() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Gloves, Rarity::Common, 100);
            assert_eq!(
                item.affixes.len(),
                0,
                "Common items should never have affixes"
            );
        }
    }
}
