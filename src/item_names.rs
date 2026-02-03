#![allow(dead_code)]
use crate::items::{AffixType, EquipmentSlot, Item, Rarity};
use rand::Rng;

pub fn get_base_name(slot: EquipmentSlot) -> Vec<&'static str> {
    match slot {
        EquipmentSlot::Weapon => vec!["Sword", "Axe", "Mace", "Dagger", "Greatsword", "Spear"],
        EquipmentSlot::Armor => vec!["Leather Armor", "Chain Mail", "Plate Mail", "Scale Mail"],
        EquipmentSlot::Helmet => vec!["Cap", "Helm", "Crown", "Coif"],
        EquipmentSlot::Gloves => vec!["Gloves", "Gauntlets", "Mitts", "Handwraps"],
        EquipmentSlot::Boots => vec!["Boots", "Greaves", "Shoes", "Sabatons"],
        EquipmentSlot::Amulet => vec!["Amulet", "Pendant", "Necklace", "Talisman"],
        EquipmentSlot::Ring => vec!["Ring", "Band", "Circle", "Loop"],
    }
}

pub fn get_quality_prefix(rarity: Rarity) -> &'static str {
    match rarity {
        Rarity::Common => "",
        Rarity::Magic => "Fine",
        _ => "", // Rare+ uses procedural names
    }
}

pub fn get_affix_prefix(affix_type: AffixType) -> &'static str {
    match affix_type {
        AffixType::DamagePercent => "Cruel",
        AffixType::CritChance => "Deadly",
        AffixType::CritMultiplier => "Vicious",
        AffixType::AttackSpeed => "Swift",
        AffixType::HPBonus => "Sturdy",
        AffixType::DamageReduction => "Armored",
        AffixType::HPRegen => "Regenerating",
        AffixType::DamageReflection => "Thorned",
        AffixType::XPGain => "Wise",
        AffixType::DropRate => "Lucky",
        AffixType::PrestigeBonus => "Prestigious",
        AffixType::OfflineRate => "Timeless",
    }
}

pub fn get_affix_suffix(affix_type: AffixType) -> &'static str {
    match affix_type {
        AffixType::DamagePercent => "of Power",
        AffixType::CritChance => "of Precision",
        AffixType::CritMultiplier => "of Carnage",
        AffixType::AttackSpeed => "of Haste",
        AffixType::HPBonus => "of Vitality",
        AffixType::DamageReduction => "of Protection",
        AffixType::HPRegen => "of Renewal",
        AffixType::DamageReflection => "of Thorns",
        AffixType::XPGain => "of Learning",
        AffixType::DropRate => "of Fortune",
        AffixType::PrestigeBonus => "of Glory",
        AffixType::OfflineRate => "of Eternity",
    }
}

pub fn generate_display_name(item: &Item) -> String {
    let mut rng = rand::thread_rng();
    let base_names = get_base_name(item.slot);
    let base = base_names[rng.gen_range(0..base_names.len())];

    match item.rarity {
        Rarity::Common => base.to_string(),
        Rarity::Magic => {
            let prefix = get_quality_prefix(item.rarity);
            format!("{} {}", prefix, base)
        }
        Rarity::Rare | Rarity::Epic | Rarity::Legendary => {
            // Use first affix for naming (if any)
            if let Some(first_affix) = item.affixes.first() {
                let use_prefix = rng.gen_bool(0.5);
                if use_prefix {
                    let prefix = get_affix_prefix(first_affix.affix_type);
                    format!("{} {}", prefix, base)
                } else {
                    let suffix = get_affix_suffix(first_affix.affix_type);
                    format!("{} {}", base, suffix)
                }
            } else {
                base.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Affix, AttributeBonuses};

    #[test]
    fn test_common_item_name() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            base_name: "Sword".to_string(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        };
        let name = generate_display_name(&item);
        // Should be just base name
        assert!(!name.is_empty());
        assert!(!name.contains("Fine"));
    }

    #[test]
    fn test_magic_item_name() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            base_name: "Sword".to_string(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        };
        let name = generate_display_name(&item);
        assert!(name.starts_with("Fine"));
    }

    #[test]
    fn test_rare_item_name_with_affix() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            base_name: "Sword".to_string(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 15.0,
            }],
        };
        let name = generate_display_name(&item);
        // Should contain either "Cruel" or "of Power"
        assert!(name.contains("Cruel") || name.contains("of Power"));
    }

    #[test]
    fn test_base_names_exist_for_all_slots() {
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
            let names = get_base_name(slot);
            assert!(!names.is_empty());
        }
    }

    #[test]
    fn test_all_affix_types_have_prefix() {
        let affix_types = [
            AffixType::DamagePercent,
            AffixType::CritChance,
            AffixType::CritMultiplier,
            AffixType::AttackSpeed,
            AffixType::HPBonus,
            AffixType::DamageReduction,
            AffixType::HPRegen,
            AffixType::DamageReflection,
            AffixType::XPGain,
            AffixType::DropRate,
            AffixType::PrestigeBonus,
            AffixType::OfflineRate,
        ];
        for affix_type in affix_types {
            let prefix = get_affix_prefix(affix_type);
            assert!(
                !prefix.is_empty(),
                "Affix {:?} should have a prefix",
                affix_type
            );
        }
    }

    #[test]
    fn test_all_affix_types_have_suffix() {
        let affix_types = [
            AffixType::DamagePercent,
            AffixType::CritChance,
            AffixType::CritMultiplier,
            AffixType::AttackSpeed,
            AffixType::HPBonus,
            AffixType::DamageReduction,
            AffixType::HPRegen,
            AffixType::DamageReflection,
            AffixType::XPGain,
            AffixType::DropRate,
            AffixType::PrestigeBonus,
            AffixType::OfflineRate,
        ];
        for affix_type in affix_types {
            let suffix = get_affix_suffix(affix_type);
            assert!(
                !suffix.is_empty(),
                "Affix {:?} should have a suffix",
                affix_type
            );
            assert!(
                suffix.starts_with("of "),
                "Suffix for {:?} should start with 'of '",
                affix_type
            );
        }
    }

    #[test]
    fn test_epic_item_name_with_affix() {
        let item = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Epic,
            base_name: String::new(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::HPBonus,
                value: 50.0,
            }],
        };
        let name = generate_display_name(&item);
        // Should use affix prefix "Sturdy" or suffix "of Vitality"
        assert!(
            name.contains("Sturdy") || name.contains("of Vitality"),
            "Epic item name '{}' should contain affix-derived prefix or suffix",
            name
        );
    }

    #[test]
    fn test_legendary_item_name_with_affix() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            base_name: String::new(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::CritChance,
                value: 30.0,
            }],
        };
        let name = generate_display_name(&item);
        assert!(
            name.contains("Deadly") || name.contains("of Precision"),
            "Legendary item name '{}' should reference CritChance affix",
            name
        );
    }

    #[test]
    fn test_rare_item_without_affixes_uses_base_name() {
        let item = Item {
            slot: EquipmentSlot::Ring,
            rarity: Rarity::Rare,
            base_name: String::new(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        };
        let name = generate_display_name(&item);
        // Should be a plain base name since there are no affixes
        let ring_names = get_base_name(EquipmentSlot::Ring);
        assert!(
            ring_names.iter().any(|&n| name == n),
            "Rare item with no affixes should use plain base name, got '{}'",
            name
        );
    }

    #[test]
    fn test_common_name_is_base_name_only() {
        // Run multiple times since base name is randomly chosen
        for _ in 0..20 {
            let item = Item {
                slot: EquipmentSlot::Boots,
                rarity: Rarity::Common,
                base_name: String::new(),
                display_name: String::new(),
                attributes: AttributeBonuses::new(),
                affixes: vec![],
            };
            let name = generate_display_name(&item);
            let base_names = get_base_name(EquipmentSlot::Boots);
            assert!(
                base_names.iter().any(|&n| name == n),
                "Common item name '{}' should be one of the base names",
                name
            );
        }
    }

    #[test]
    fn test_quality_prefix_only_for_magic() {
        assert_eq!(get_quality_prefix(Rarity::Common), "");
        assert_eq!(get_quality_prefix(Rarity::Magic), "Fine");
        assert_eq!(get_quality_prefix(Rarity::Rare), "");
        assert_eq!(get_quality_prefix(Rarity::Epic), "");
        assert_eq!(get_quality_prefix(Rarity::Legendary), "");
    }

    // =========================================================================
    // BULK GENERATION TESTS
    // =========================================================================

    #[test]
    fn test_generate_display_name_never_empty() {
        // Generate 100 items of each rarity and slot, verify names are never empty
        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ];
        let rarities = [
            Rarity::Common,
            Rarity::Magic,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ];

        for slot in &slots {
            for rarity in &rarities {
                for _ in 0..100 {
                    let item = Item {
                        slot: *slot,
                        rarity: *rarity,
                        base_name: String::new(),
                        display_name: String::new(),
                        attributes: AttributeBonuses::new(),
                        affixes: if *rarity >= Rarity::Rare {
                            vec![Affix {
                                affix_type: AffixType::DamagePercent,
                                value: 10.0,
                            }]
                        } else {
                            vec![]
                        },
                    };

                    let name = generate_display_name(&item);
                    assert!(
                        !name.is_empty(),
                        "Generated name should never be empty for {:?} {:?}",
                        rarity,
                        slot
                    );
                    assert!(
                        !name.trim().is_empty(),
                        "Generated name should not be only whitespace for {:?} {:?}",
                        rarity,
                        slot
                    );
                }
            }
        }
    }

    #[test]
    fn test_generate_display_name_no_double_spaces() {
        // Verify names don't have double spaces or leading/trailing spaces
        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
        ];

        for slot in &slots {
            for _ in 0..50 {
                let item = Item {
                    slot: *slot,
                    rarity: Rarity::Magic,
                    base_name: String::new(),
                    display_name: String::new(),
                    attributes: AttributeBonuses::new(),
                    affixes: vec![],
                };

                let name = generate_display_name(&item);
                assert!(
                    !name.contains("  "),
                    "Name '{}' should not have double spaces",
                    name
                );
                assert_eq!(
                    name,
                    name.trim(),
                    "Name '{}' should not have leading/trailing whitespace",
                    name
                );
            }
        }
    }

    #[test]
    fn test_all_slots_produce_different_base_names() {
        // Verify each slot has unique base names (no shared names between slots)
        use std::collections::HashSet;

        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ];

        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                let names_i: HashSet<_> = get_base_name(slots[i]).into_iter().collect();
                let names_j: HashSet<_> = get_base_name(slots[j]).into_iter().collect();

                let overlap: Vec<_> = names_i.intersection(&names_j).collect();
                assert!(
                    overlap.is_empty(),
                    "Slots {:?} and {:?} should not share base names, but share: {:?}",
                    slots[i],
                    slots[j],
                    overlap
                );
            }
        }
    }
}
