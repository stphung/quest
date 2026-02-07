#![allow(dead_code)]
use super::names::generate_display_name;
use super::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use rand::Rng;

pub fn generate_item(slot: EquipmentSlot, rarity: Rarity, _player_level: u32) -> Item {
    let mut rng = rand::thread_rng();

    // Generate attribute bonuses based on rarity
    let attributes = generate_attributes(rarity, &mut rng);

    // Generate affixes based on rarity
    let affixes = generate_affixes(rarity, &mut rng);

    let mut item = Item {
        slot,
        rarity,
        base_name: String::new(), // Will be set by display name
        display_name: String::new(),
        attributes,
        affixes,
    };

    item.display_name = generate_display_name(&item);
    item.base_name = item.display_name.clone();

    item
}

fn generate_attributes(rarity: Rarity, rng: &mut impl Rng) -> AttributeBonuses {
    let (min, max) = match rarity {
        Rarity::Common => (1, 2),
        Rarity::Magic => (2, 4),
        Rarity::Rare => (3, 6),
        Rarity::Epic => (5, 10),
        Rarity::Legendary => (8, 15),
    };

    // Pick a random number of attributes to boost (1-3)
    let num_attrs = rng.gen_range(1..=3);
    let mut attrs = AttributeBonuses::new();

    for _ in 0..num_attrs {
        let value = rng.gen_range(min..=max);
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

fn generate_affixes(rarity: Rarity, rng: &mut impl Rng) -> Vec<Affix> {
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
        let value = generate_affix_value(affix_type, rarity, rng);
        affixes.push(Affix { affix_type, value });
    }

    affixes
}

fn generate_affix_value(affix_type: AffixType, rarity: Rarity, rng: &mut impl Rng) -> f64 {
    let (min, max) = match rarity {
        Rarity::Common => (0.0, 0.0),
        Rarity::Magic => (5.0, 10.0),
        Rarity::Rare => (10.0, 20.0),
        Rarity::Epic => (15.0, 30.0),
        Rarity::Legendary => (25.0, 50.0),
    };

    // Some affixes use different ranges
    match affix_type {
        AffixType::HPBonus => {
            // Flat HP bonus
            let (hp_min, hp_max) = match rarity {
                Rarity::Magic => (10.0, 30.0),
                Rarity::Rare => (30.0, 60.0),
                Rarity::Epic => (50.0, 100.0),
                Rarity::Legendary => (80.0, 150.0),
                _ => (0.0, 0.0),
            };
            rng.gen_range(hp_min..=hp_max)
        }
        _ => rng.gen_range(min..=max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_common_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        assert_eq!(item.rarity, Rarity::Common);
        assert_eq!(item.slot, EquipmentSlot::Weapon);
        assert_eq!(item.affixes.len(), 0);
        assert!(item.attributes.total() > 0);
    }

    #[test]
    fn test_generate_magic_item_has_affix() {
        let item = generate_item(EquipmentSlot::Armor, Rarity::Magic, 5);
        assert_eq!(item.rarity, Rarity::Magic);
        assert_eq!(item.affixes.len(), 1);
    }

    #[test]
    fn test_generate_rare_item_has_multiple_affixes() {
        let item = generate_item(EquipmentSlot::Helmet, Rarity::Rare, 10);
        assert_eq!(item.rarity, Rarity::Rare);
        assert!(item.affixes.len() >= 2 && item.affixes.len() <= 3);
    }

    #[test]
    fn test_generate_legendary_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
        assert_eq!(item.rarity, Rarity::Legendary);
        assert!(item.affixes.len() >= 4 && item.affixes.len() <= 5);
        assert!(item.attributes.total() >= 8);
    }

    #[test]
    fn test_item_has_display_name() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Magic, 5);
        assert!(!item.display_name.is_empty());
    }

    #[test]
    fn test_generate_epic_item_affixes() {
        let item = generate_item(EquipmentSlot::Boots, Rarity::Epic, 15);
        assert_eq!(item.rarity, Rarity::Epic);
        assert!(item.affixes.len() >= 3 && item.affixes.len() <= 4);
        assert!(item.attributes.total() >= 5);
    }

    #[test]
    fn test_common_attribute_bounds() {
        // Common items: 1-2 per attribute, 1-3 attributes boosted
        // So total should be between 1 and 6 (3 attrs * 2 max)
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 1);
            assert!(item.attributes.total() >= 1, "Common total too low");
            assert!(
                item.attributes.total() <= 6,
                "Common total too high: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_legendary_attribute_bounds() {
        // Legendary items: 8-15 per attribute, 1-3 attributes boosted
        // So total should be between 8 and 45 (3 attrs * 15 max)
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
            assert!(
                item.attributes.total() >= 8,
                "Legendary total too low: {}",
                item.attributes.total()
            );
            assert!(
                item.attributes.total() <= 45,
                "Legendary total too high: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_magic_attribute_bounds() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Armor, Rarity::Magic, 5);
            assert!(item.attributes.total() >= 2, "Magic total too low");
            assert!(
                item.attributes.total() <= 12,
                "Magic total too high: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_rare_attribute_bounds() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Helmet, Rarity::Rare, 10);
            assert!(item.attributes.total() >= 3, "Rare total too low");
            assert!(
                item.attributes.total() <= 18,
                "Rare total too high: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_epic_attribute_bounds() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Ring, Rarity::Epic, 15);
            assert!(item.attributes.total() >= 5, "Epic total too low");
            assert!(
                item.attributes.total() <= 30,
                "Epic total too high: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_affix_values_within_magic_range() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Magic, 5);
            for affix in &item.affixes {
                match affix.affix_type {
                    AffixType::HPBonus => {
                        assert!(
                            affix.value >= 10.0 && affix.value <= 30.0,
                            "Magic HPBonus out of range: {}",
                            affix.value
                        );
                    }
                    _ => {
                        assert!(
                            affix.value >= 5.0 && affix.value <= 10.0,
                            "Magic affix {:?} out of range: {}",
                            affix.affix_type,
                            affix.value
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_affix_values_within_legendary_range() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
            for affix in &item.affixes {
                match affix.affix_type {
                    AffixType::HPBonus => {
                        assert!(
                            affix.value >= 80.0 && affix.value <= 150.0,
                            "Legendary HPBonus out of range: {}",
                            affix.value
                        );
                    }
                    _ => {
                        assert!(
                            affix.value >= 25.0 && affix.value <= 50.0,
                            "Legendary affix {:?} out of range: {}",
                            affix.affix_type,
                            affix.value
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_common_items_never_have_affixes() {
        for _ in 0..100 {
            let item = generate_item(EquipmentSlot::Gloves, Rarity::Common, 1);
            assert_eq!(
                item.affixes.len(),
                0,
                "Common items should never have affixes"
            );
        }
    }

    #[test]
    fn test_generate_item_all_slots() {
        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ];
        for slot in slots {
            let item = generate_item(slot, Rarity::Rare, 10);
            assert_eq!(item.slot, slot);
            assert!(item.attributes.total() > 0);
            assert!(!item.display_name.is_empty());
        }
    }

    #[test]
    fn test_rarity_ordering_implies_stronger_attributes() {
        // Over many samples, higher rarity should produce higher average totals
        let sample = |rarity: Rarity| -> f64 {
            let sum: u32 = (0..100)
                .map(|_| {
                    generate_item(EquipmentSlot::Weapon, rarity, 10)
                        .attributes
                        .total()
                })
                .sum();
            sum as f64 / 100.0
        };

        let common_avg = sample(Rarity::Common);
        let magic_avg = sample(Rarity::Magic);
        let rare_avg = sample(Rarity::Rare);
        let epic_avg = sample(Rarity::Epic);
        let legendary_avg = sample(Rarity::Legendary);

        assert!(
            common_avg < magic_avg,
            "Common ({common_avg}) should be < Magic ({magic_avg})"
        );
        assert!(
            magic_avg < rare_avg,
            "Magic ({magic_avg}) should be < Rare ({rare_avg})"
        );
        assert!(
            rare_avg < epic_avg,
            "Rare ({rare_avg}) should be < Epic ({epic_avg})"
        );
        assert!(
            epic_avg < legendary_avg,
            "Epic ({epic_avg}) should be < Legendary ({legendary_avg})"
        );
    }
}
