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

impl EquipmentSlot {
    pub fn name(&self) -> &'static str {
        match self {
            EquipmentSlot::Weapon => "Weapon",
            EquipmentSlot::Armor => "Armor",
            EquipmentSlot::Helmet => "Helmet",
            EquipmentSlot::Gloves => "Gloves",
            EquipmentSlot::Boots => "Boots",
            EquipmentSlot::Amulet => "Amulet",
            EquipmentSlot::Ring => "Ring",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EquipmentSlot::Weapon => "\u{2694}",
            EquipmentSlot::Armor => "\u{1f6e1}",
            EquipmentSlot::Helmet => "\u{1fa96}",
            EquipmentSlot::Gloves => "\u{1f9e4}",
            EquipmentSlot::Boots => "\u{1f462}",
            EquipmentSlot::Amulet => "\u{1f4ff}",
            EquipmentSlot::Ring => "\u{1f48d}",
        }
    }

    /// Terminal cell width of the icon (text symbols = 1, emoji = 2).
    pub fn icon_width(&self) -> u8 {
        match self {
            EquipmentSlot::Weapon | EquipmentSlot::Armor => 1,
            _ => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    Common = 0,
    Magic = 1,
    Rare = 2,
    Epic = 3,
    Legendary = 4,
    Mythic = 5,
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
            Rarity::Mythic => "God",
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

    #[allow(dead_code)] // Used by integration tests
    pub fn total(&self) -> u32 {
        self.str + self.dex + self.con + self.int + self.wis + self.cha
    }

    /// Returns all six attribute values as an array in order: STR, DEX, CON, INT, WIS, CHA.
    pub fn as_array(&self) -> [u32; 6] {
        [self.str, self.dex, self.con, self.int, self.wis, self.cha]
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
    /// Catch-all for removed variants (DropRate, PrestigeBonus, OfflineRate).
    /// Stripped on load — never appears at runtime.
    #[serde(other)]
    Unknown,
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
    #[serde(default = "default_ilvl")]
    pub ilvl: u32,
    #[serde(default = "default_tier")]
    pub tier: u8,
    pub base_name: String,
    pub display_name: String,
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
    #[serde(default)]
    pub god_item_id: Option<crate::god_items::GodItemId>,
}

fn default_ilvl() -> u32 {
    10
}

fn default_tier() -> u8 {
    1
}

impl Item {
    /// Returns a short stat summary string like "+8 STR +3 DEX +Crit"
    /// Uses a single String allocation with in-place writes instead of Vec<String> + join.
    pub fn stat_summary(&self) -> String {
        use std::fmt::Write;
        const ATTR_LABELS: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
        let mut result = String::with_capacity(64);

        for (value, label) in self.attributes.as_array().iter().zip(ATTR_LABELS.iter()) {
            if *value > 0 {
                if !result.is_empty() {
                    result.push(' ');
                }
                let _ = write!(result, "+{} {}", value, label);
            }
        }

        for affix in &self.affixes {
            if affix.affix_type == AffixType::Unknown {
                continue;
            }
            if !result.is_empty() {
                result.push(' ');
            }
            let _ = match affix.affix_type {
                AffixType::DamagePercent => write!(result, "+{:.0}% Dmg", affix.value),
                AffixType::CritChance => write!(result, "+{:.0}% Crit", affix.value),
                AffixType::CritMultiplier => write!(result, "+{:.1}x CritDmg", affix.value),
                AffixType::AttackSpeed => write!(result, "+{:.0}% AtkSpd", affix.value),
                AffixType::HPBonus => write!(result, "+{:.0} HP", affix.value),
                AffixType::DamageReduction => write!(result, "+{:.0}% DR", affix.value),
                AffixType::HPRegen => write!(result, "+{:.0} Regen", affix.value),
                AffixType::DamageReflection => write!(result, "+{:.0}% Reflect", affix.value),
                AffixType::XPGain => write!(result, "+{:.0}% XP", affix.value),
                AffixType::Unknown => Ok(()),
            };
        }

        result
    }

    /// Returns the slot name as a string
    pub fn slot_name(&self) -> &'static str {
        self.slot.name()
    }

    /// Returns the intrinsic power score for this item.
    ///
    /// Computed from raw attribute totals (equal weight) plus weighted affix values.
    /// Independent of character build — the same item always returns the same power.
    pub fn power(&self) -> u32 {
        use crate::items::scoring::affix_power_weight;

        let attr_total: f64 = self.attributes.as_array().iter().map(|&v| v as f64).sum();
        let affix_total: f64 = self
            .affixes
            .iter()
            .map(|a| a.value * affix_power_weight(a.affix_type))
            .sum();
        (attr_total + affix_total).round() as u32
    }
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
    fn test_mythic_rarity_above_legendary() {
        assert!(Rarity::Legendary < Rarity::Mythic);
        assert!(Rarity::Mythic > Rarity::Epic);
    }

    #[test]
    fn test_mythic_rarity_name() {
        assert_eq!(Rarity::Mythic.name(), "God");
    }

    #[test]
    fn test_item_creation() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Fine Sword".to_string(),
            attributes: AttributeBonuses {
                str: 2,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
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
            ilvl: 10,
            tier: 5,
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
            god_item_id: None,
        };
        assert_eq!(item.affixes.len(), 3);
        assert_eq!(item.affixes[0].affix_type, AffixType::DamagePercent);
        assert!((item.affixes[0].value - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipment_slot_names() {
        assert_eq!(EquipmentSlot::Weapon.name(), "Weapon");
        assert_eq!(EquipmentSlot::Armor.name(), "Armor");
        assert_eq!(EquipmentSlot::Helmet.name(), "Helmet");
        assert_eq!(EquipmentSlot::Gloves.name(), "Gloves");
        assert_eq!(EquipmentSlot::Boots.name(), "Boots");
        assert_eq!(EquipmentSlot::Amulet.name(), "Amulet");
        assert_eq!(EquipmentSlot::Ring.name(), "Ring");
    }

    #[test]
    fn test_equipment_slot_icons() {
        assert_eq!(EquipmentSlot::Weapon.icon(), "\u{2694}");
        assert_eq!(EquipmentSlot::Armor.icon(), "\u{1f6e1}");
        assert_eq!(EquipmentSlot::Helmet.icon(), "\u{1fa96}");
        assert_eq!(EquipmentSlot::Gloves.icon(), "\u{1f9e4}");
        assert_eq!(EquipmentSlot::Boots.icon(), "\u{1f462}");
        assert_eq!(EquipmentSlot::Amulet.icon(), "\u{1f4ff}");
        assert_eq!(EquipmentSlot::Ring.icon(), "\u{1f48d}");
    }

    #[test]
    fn test_equipment_slot_icon_width() {
        // Weapon and Armor render as single-width text glyphs.
        assert_eq!(EquipmentSlot::Weapon.icon_width(), 1);
        assert_eq!(EquipmentSlot::Armor.icon_width(), 1);
        // All other slots use double-width emoji.
        assert_eq!(EquipmentSlot::Helmet.icon_width(), 2);
        assert_eq!(EquipmentSlot::Gloves.icon_width(), 2);
        assert_eq!(EquipmentSlot::Boots.icon_width(), 2);
        assert_eq!(EquipmentSlot::Amulet.icon_width(), 2);
        assert_eq!(EquipmentSlot::Ring.icon_width(), 2);
    }

    fn empty_item(slot: EquipmentSlot) -> Item {
        Item {
            slot,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 1,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
            god_item_id: None,
        }
    }

    #[test]
    fn test_slot_name_delegates_to_equipment_slot() {
        let item = empty_item(EquipmentSlot::Helmet);
        assert_eq!(item.slot_name(), "Helmet");
    }

    #[test]
    fn test_stat_summary_empty_item_is_empty_string() {
        let item = empty_item(EquipmentSlot::Weapon);
        assert_eq!(item.stat_summary(), "");
    }

    #[test]
    fn test_stat_summary_attributes_only() {
        let mut item = empty_item(EquipmentSlot::Weapon);
        item.attributes = AttributeBonuses {
            str: 8,
            dex: 3,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        };
        let summary = item.stat_summary();
        assert_eq!(summary, "+8 STR +3 DEX");
    }

    #[test]
    fn test_stat_summary_all_attributes_present() {
        let mut item = empty_item(EquipmentSlot::Weapon);
        item.attributes = AttributeBonuses {
            str: 1,
            dex: 2,
            con: 3,
            int: 4,
            wis: 5,
            cha: 6,
        };
        assert_eq!(
            item.stat_summary(),
            "+1 STR +2 DEX +3 CON +4 INT +5 WIS +6 CHA"
        );
    }

    #[test]
    fn test_stat_summary_covers_every_affix_type() {
        let mut item = empty_item(EquipmentSlot::Ring);
        item.affixes = vec![
            Affix {
                affix_type: AffixType::DamagePercent,
                value: 10.0,
            },
            Affix {
                affix_type: AffixType::CritChance,
                value: 5.0,
            },
            Affix {
                affix_type: AffixType::CritMultiplier,
                value: 1.5,
            },
            Affix {
                affix_type: AffixType::AttackSpeed,
                value: 12.0,
            },
            Affix {
                affix_type: AffixType::HPBonus,
                value: 20.0,
            },
            Affix {
                affix_type: AffixType::DamageReduction,
                value: 7.0,
            },
            Affix {
                affix_type: AffixType::HPRegen,
                value: 3.0,
            },
            Affix {
                affix_type: AffixType::DamageReflection,
                value: 9.0,
            },
            Affix {
                affix_type: AffixType::XPGain,
                value: 40.0,
            },
        ];
        let summary = item.stat_summary();
        assert!(summary.contains("+10% Dmg"));
        assert!(summary.contains("+5% Crit"));
        assert!(summary.contains("+1.5x CritDmg"));
        assert!(summary.contains("+12% AtkSpd"));
        assert!(summary.contains("+20 HP"));
        assert!(summary.contains("+7% DR"));
        assert!(summary.contains("+3 Regen"));
        assert!(summary.contains("+9% Reflect"));
        assert!(summary.contains("+40% XP"));
    }

    #[test]
    fn test_stat_summary_skips_unknown_affix() {
        let mut item = empty_item(EquipmentSlot::Ring);
        item.affixes = vec![
            Affix {
                affix_type: AffixType::Unknown,
                value: 5.0,
            },
            Affix {
                affix_type: AffixType::DamagePercent,
                value: 10.0,
            },
        ];
        // Unknown affix should be skipped entirely, leaving only the DamagePercent entry.
        assert_eq!(item.stat_summary(), "+10% Dmg");
    }

    #[test]
    fn test_power_score_attributes_only() {
        let mut item = empty_item(EquipmentSlot::Weapon);
        item.attributes = AttributeBonuses {
            str: 5,
            dex: 5,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        };
        assert_eq!(item.power(), 10);
    }

    #[test]
    fn test_power_score_weights_affixes() {
        let mut item = empty_item(EquipmentSlot::Weapon);
        // DamagePercent has weight 2.0, so 10.0 value contributes 20.0 power.
        item.affixes = vec![Affix {
            affix_type: AffixType::DamagePercent,
            value: 10.0,
        }];
        assert_eq!(item.power(), 20);
    }

    #[test]
    fn test_power_score_combines_attributes_and_affixes() {
        let mut item = empty_item(EquipmentSlot::Weapon);
        item.attributes = AttributeBonuses {
            str: 10,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        };
        item.affixes = vec![Affix {
            affix_type: AffixType::HPBonus, // weight 0.5
            value: 20.0,
        }];
        // 10 (attrs) + 20*0.5 (affix) = 20
        assert_eq!(item.power(), 20);
    }

    #[test]
    fn test_power_score_rounds_to_nearest_u32() {
        let mut item = empty_item(EquipmentSlot::Weapon);
        // CritChance weight 1.5 * 3.0 = 4.5, rounds to 5 (round-half-away-from-zero... actually round-half-to-even isn't used by f64::round)
        item.affixes = vec![Affix {
            affix_type: AffixType::CritChance,
            value: 3.0,
        }];
        assert_eq!(item.power(), 5);
    }

    #[test]
    fn test_deserialize_removed_affix_types() {
        // Old saves may contain DropRate, PrestigeBonus, or OfflineRate affixes.
        // These should deserialize as Unknown instead of failing.
        let json = r#"{"affix_type":"DropRate","value":5.0}"#;
        let affix: Affix = serde_json::from_str(json).unwrap();
        assert_eq!(affix.affix_type, AffixType::Unknown);

        let json = r#"{"affix_type":"PrestigeBonus","value":10.0}"#;
        let affix: Affix = serde_json::from_str(json).unwrap();
        assert_eq!(affix.affix_type, AffixType::Unknown);

        let json = r#"{"affix_type":"OfflineRate","value":3.0}"#;
        let affix: Affix = serde_json::from_str(json).unwrap();
        assert_eq!(affix.affix_type, AffixType::Unknown);
    }
}
