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
}
