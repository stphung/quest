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
    pub fn to_attributes(&self) -> crate::attributes::Attributes {
        crate::attributes::Attributes::from_bonuses(
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
    DropRate,
    PrestigeBonus,
    OfflineRate,
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
}
