use crate::items::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use crate::item_names::generate_display_name;
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
        AffixType::DropRate,
        AffixType::PrestigeBonus,
        AffixType::OfflineRate,
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
}
