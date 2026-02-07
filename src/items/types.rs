#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Helmet,
    Gloves,
    Boots,
    Amulet,
    Ring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    Common = 0,
    Magic = 1,
    Rare = 2,
    Epic = 3,
    Legendary = 4,
}

impl Rarity {
    /// Returns the display name for this rarity tier.
    pub fn name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Magic => "Magic",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeBonuses {
    pub str: u32,
    pub dex: u32,
    pub con: u32,
    pub int: u32,
    pub wis: u32,
    pub cha: u32,
}

impl AttributeBonuses {
    pub fn new() -> Self {
        Self {
            str: 0,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.str + self.dex + self.con + self.int + self.wis + self.cha
    }

    /// Converts to an Attributes struct with base 0 values plus these bonuses.
    pub fn to_attributes(&self) -> crate::character::attributes::Attributes {
        crate::character::attributes::Attributes::from_bonuses(
            self.str, self.dex, self.con, self.int, self.wis, self.cha,
        )
    }
}

impl Default for AttributeBonuses {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AffixType {
    // Damage-focused
    DamagePercent,
    CritChance,
    CritMultiplier,
    AttackSpeed,
    // Survivability
    HPBonus,
    DamageReduction,
    HPRegen,
    DamageReflection,
    // Progression
    XPGain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Affix {
    pub affix_type: AffixType,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub base_name: String,
    pub display_name: String,
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_bonuses_total() {
        let attrs = AttributeBonuses {
            str: 5,
            dex: 3,
            con: 2,
            int: 1,
            wis: 0,
            cha: 4,
        };
        assert_eq!(attrs.total(), 15);
    }

    #[test]
    fn test_rarity_ordering() {
        assert!(Rarity::Common < Rarity::Magic);
        assert!(Rarity::Magic < Rarity::Rare);
        assert!(Rarity::Rare < Rarity::Epic);
        assert!(Rarity::Epic < Rarity::Legendary);
    }

    #[test]
    fn test_item_creation() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            base_name: "Sword".to_string(),
            display_name: "Fine Sword".to_string(),
            attributes: AttributeBonuses {
                str: 2,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        assert_eq!(item.slot, EquipmentSlot::Weapon);
        assert_eq!(item.rarity, Rarity::Common);
        assert_eq!(item.attributes.str, 2);
    }

    #[test]
    fn test_attribute_bonuses_default_is_zero() {
        let attrs = AttributeBonuses::new();
        assert_eq!(attrs.total(), 0);
        assert_eq!(attrs.str, 0);
        assert_eq!(attrs.dex, 0);
        assert_eq!(attrs.con, 0);
        assert_eq!(attrs.int, 0);
        assert_eq!(attrs.wis, 0);
        assert_eq!(attrs.cha, 0);
    }

    #[test]
    fn test_attribute_bonuses_default_trait() {
        let attrs = AttributeBonuses::default();
        assert_eq!(attrs.total(), 0);
    }

    #[test]
    fn test_rarity_name() {
        assert_eq!(Rarity::Common.name(), "Common");
        assert_eq!(Rarity::Magic.name(), "Magic");
        assert_eq!(Rarity::Rare.name(), "Rare");
        assert_eq!(Rarity::Epic.name(), "Epic");
        assert_eq!(Rarity::Legendary.name(), "Legendary");
    }

    #[test]
    fn test_attribute_bonuses_to_attributes() {
        let bonuses = AttributeBonuses {
            str: 5,
            dex: 3,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        };
        let attrs = bonuses.to_attributes();
        assert_eq!(
            attrs.get(crate::character::attributes::AttributeType::Strength),
            5
        );
        assert_eq!(
            attrs.get(crate::character::attributes::AttributeType::Dexterity),
            3
        );
        assert_eq!(
            attrs.get(crate::character::attributes::AttributeType::Constitution),
            0
        );
    }

    #[test]
    fn test_item_with_multiple_affixes() {
        let item = Item {
            slot: EquipmentSlot::Ring,
            rarity: Rarity::Epic,
            base_name: "Ring".to_string(),
            display_name: "Ring of Power".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![
                Affix {
                    affix_type: AffixType::DamagePercent,
                    value: 15.0,
                },
                Affix {
                    affix_type: AffixType::CritChance,
                    value: 10.0,
                },
                Affix {
                    affix_type: AffixType::HPBonus,
                    value: 50.0,
                },
            ],
        };
        assert_eq!(item.affixes.len(), 3);
        assert_eq!(item.affixes[0].affix_type, AffixType::DamagePercent);
        assert!((item.affixes[0].value - 15.0).abs() < f64::EPSILON);
    }
}
