use super::attributes::{AttributeType, Attributes};
use crate::core::constants::*;
use crate::items::Equipment;

#[derive(Debug, Clone, Copy)]
pub struct DerivedStats {
    pub max_hp: u64,
    pub physical_damage: u64,
    pub magic_damage: u64,
    pub defense: u64,
    pub crit_chance_percent: u32,
    pub crit_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub hp_regen_multiplier: f64,
    pub damage_reflection_percent: f64,
    #[allow(dead_code)]
    pub xp_multiplier: f64,
}

impl Default for DerivedStats {
    fn default() -> Self {
        Self::from_attributes(&Attributes::new())
    }
}

impl DerivedStats {
    /// Creates derived stats from attributes with no equipment bonuses.
    /// Primarily used for backward compatibility and tests.
    ///
    /// Note: Prestige multiplier is calculated separately using the
    /// `prestige_multiplier()` static method, not in this constructor.
    #[allow(dead_code)]
    pub fn from_attributes(attrs: &Attributes) -> Self {
        Self::calculate_derived_stats(attrs, &Equipment::new(), &[0; 7])
    }

    /// Calculates derived stats from attributes and equipment bonuses.
    ///
    /// Equipment bonuses are added to base attributes before calculating modifiers.
    /// Affixes are then applied as multipliers/bonuses to the calculated stats.
    /// Enhancement levels scale item attribute and affix contributions per slot.
    pub fn calculate_derived_stats(
        attrs: &Attributes,
        equipment: &Equipment,
        enhancement_levels: &[u8; 7],
    ) -> Self {
        super::calculation::calculate_derived_stats(attrs, equipment, enhancement_levels)
    }

    pub fn total_damage(&self) -> u64 {
        self.physical_damage + self.magic_damage
    }

    /// Calculates prestige multiplier with equipment bonuses included.
    #[allow(dead_code)]
    pub fn prestige_multiplier_with_equipment(
        base_multiplier: f64,
        attrs: &Attributes,
        equipment: &Equipment,
    ) -> f64 {
        super::multipliers::prestige_multiplier_with_equipment(base_multiplier, attrs, equipment)
    }

    pub fn prestige_multiplier(base_multiplier: f64, attrs: &Attributes) -> f64 {
        super::multipliers::prestige_multiplier(base_multiplier, attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{
        Affix, AffixType, AttributeBonuses, Equipment, EquipmentSlot, Item, Rarity,
    };

    #[test]
    fn test_derived_stats_base() {
        let attrs = Attributes::new();
        let stats = DerivedStats::from_attributes(&attrs);

        // All attributes at 10 (modifier = 0)
        assert_eq!(stats.max_hp, 50);
        assert_eq!(stats.physical_damage, 5);
        assert_eq!(stats.magic_damage, 5);
        assert_eq!(stats.defense, 0);
        assert_eq!(stats.crit_chance_percent, 5);
        assert_eq!(stats.xp_multiplier, 1.0);
        assert_eq!(stats.total_damage(), 10);
    }

    #[test]
    fn test_derived_stats_high_attributes() {
        let mut attrs = Attributes::new();
        attrs.set(AttributeType::Strength, 16); // +3 mod
        attrs.set(AttributeType::Dexterity, 18); // +4 mod
        attrs.set(AttributeType::Constitution, 14); // +2 mod
        attrs.set(AttributeType::Intelligence, 12); // +1 mod
        attrs.set(AttributeType::Wisdom, 20); // +5 mod

        let stats = DerivedStats::from_attributes(&attrs);

        assert_eq!(stats.max_hp, 70); // 50 + (2 * 10)
        assert_eq!(stats.physical_damage, 11); // 5 + (3 * 2)
        assert_eq!(stats.magic_damage, 7); // 5 + (1 * 2)
        assert_eq!(stats.defense, 4); // 0 + 4
        assert_eq!(stats.crit_chance_percent, 9); // 5 + 4
        assert_eq!(stats.xp_multiplier, 1.25); // 1.0 + (5 * 0.05)
        assert_eq!(stats.total_damage(), 18);
    }

    #[test]
    fn test_prestige_multiplier_with_charisma() {
        let mut attrs = Attributes::new();
        attrs.set(AttributeType::Charisma, 16); // +3 mod

        let prestige = DerivedStats::prestige_multiplier(2.25, &attrs);
        assert_eq!(prestige, 2.55); // 2.25 + (3 * 0.1)
    }

    #[test]
    fn test_low_attributes() {
        let mut attrs = Attributes::new();
        attrs.set(AttributeType::Strength, 8); // -1 mod
        attrs.set(AttributeType::Constitution, 8); // -1 mod

        let stats = DerivedStats::from_attributes(&attrs);

        // Should never go below 1 for damage/hp
        assert_eq!(stats.max_hp, 40); // 50 + (-1 * 10)
        assert_eq!(stats.physical_damage, 3); // 5 + (-1 * 2)
    }

    #[test]
    fn test_derived_stats_with_equipment() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create a weapon with +2 STR and +1 DEX
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Iron Sword".to_string(),
            attributes: AttributeBonuses {
                str: 2,
                dex: 1,
                con: 0,
                int: 0,
                wis: 0,
                cha: 0,
            },
            affixes: vec![],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Weapon, Some(weapon));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base: STR 10 (+0 mod), DEX 10 (+0 mod)
        // With equipment: STR 12 (+1 mod), DEX 11 (+0 mod)
        assert_eq!(stats.physical_damage, 7); // 5 + (1 * 2)
        assert_eq!(stats.crit_chance_percent, 5); // 5 + 0

        // Verify without equipment gives original stats
        let stats_no_equipment = DerivedStats::from_attributes(&attrs);
        assert_eq!(stats_no_equipment.physical_damage, 5);
    }

    #[test]
    fn test_prestige_multiplier_with_equipment_charisma() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create an amulet with +3 CHA
        let amulet = Item {
            slot: EquipmentSlot::Amulet,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Amulet".to_string(),
            display_name: "Blessed Amulet".to_string(),
            attributes: AttributeBonuses {
                str: 0,
                dex: 0,
                con: 0,
                int: 0,
                wis: 0,
                cha: 3,
            },
            affixes: vec![],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Amulet, Some(amulet));

        let prestige = DerivedStats::prestige_multiplier_with_equipment(2.0, &attrs, &equipment);
        // Base: CHA 10 (+0 mod) -> 2.0 + 0 = 2.0
        // With equipment: CHA 13 (+1 mod) -> 2.0 + 0.1 = 2.1
        assert_eq!(prestige, 2.1);
    }

    #[test]
    fn test_derived_stats_with_affixes() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create a weapon with +20% damage affix
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Enchanted Sword".to_string(),
            attributes: AttributeBonuses {
                str: 0,
                dex: 0,
                con: 0,
                int: 0,
                wis: 0,
                cha: 0,
            },
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 20.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Weapon, Some(weapon));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base damage: 5 + (0 * 2) = 5
        // With +20% multiplier: 5 * 1.2 = 6
        assert_eq!(stats.physical_damage, 6);
        assert_eq!(stats.magic_damage, 6);
    }

    #[test]
    fn test_derived_stats_with_multiple_affixes() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create a weapon with damage and crit affixes
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Legendary Sword".to_string(),
            attributes: AttributeBonuses {
                str: 0,
                dex: 0,
                con: 0,
                int: 0,
                wis: 0,
                cha: 0,
            },
            affixes: vec![
                Affix {
                    affix_type: AffixType::DamagePercent,
                    value: 25.0,
                },
                Affix {
                    affix_type: AffixType::CritChance,
                    value: 10.0,
                },
                Affix {
                    affix_type: AffixType::HPBonus,
                    value: 20.0,
                },
            ],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Weapon, Some(weapon));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base damage: 5 + (0 * 2) = 5
        // With +25% multiplier: 5 * 1.25 = 6.25 -> 6
        assert_eq!(stats.physical_damage, 6);
        assert_eq!(stats.magic_damage, 6);

        // Base crit: 5 + 0 = 5
        // With +10 bonus: 5 + 10 = 15
        assert_eq!(stats.crit_chance_percent, 15);

        // Base HP: 50 + (0 * 10) = 50
        // With +20 bonus: 50 + 20 = 70
        assert_eq!(stats.max_hp, 70);
    }

    #[test]
    fn test_derived_stats_with_defense_affix() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create armor with damage reduction affix
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Armor".to_string(),
            display_name: "Reinforced Armor".to_string(),
            attributes: AttributeBonuses {
                str: 0,
                dex: 0,
                con: 0,
                int: 0,
                wis: 0,
                cha: 0,
            },
            affixes: vec![Affix {
                affix_type: AffixType::DamageReduction,
                value: 15.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Armor, Some(armor));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base defense: 0 + 0 = 0
        // With +15% multiplier: 0 * 1.15 = 0
        assert_eq!(stats.defense, 0);
    }

    #[test]
    fn test_derived_stats_with_xp_gain_affix() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create amulet with XP gain affix
        let amulet = Item {
            slot: EquipmentSlot::Amulet,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Amulet".to_string(),
            display_name: "Enchanted Amulet".to_string(),
            attributes: AttributeBonuses {
                str: 0,
                dex: 0,
                con: 0,
                int: 0,
                wis: 0,
                cha: 0,
            },
            affixes: vec![Affix {
                affix_type: AffixType::XPGain,
                value: 50.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Amulet, Some(amulet));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base XP multiplier: 1.0 + (0 * 0.05) = 1.0
        // With +50% multiplier: 1.0 * 1.5 = 1.5
        assert_eq!(stats.xp_multiplier, 1.5);
    }

    #[test]
    fn test_crit_multiplier_affix() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        // Create weapon with +50% crit multiplier affix
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Vicious Sword".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::CritMultiplier,
                value: 50.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Weapon, Some(weapon));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base crit multiplier: 2.0
        // With +50%: 2.0 + 0.5 = 2.5
        assert!((stats.crit_multiplier - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_crit_multiplier_stacks_from_multiple_items() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Sword".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::CritMultiplier,
                value: 25.0,
            }],
            god_item_id: None,
        };

        let ring = Item {
            slot: EquipmentSlot::Ring,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Ring".to_string(),
            display_name: "Ring".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::CritMultiplier,
                value: 25.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Weapon, Some(weapon));
        equipment.set(EquipmentSlot::Ring, Some(ring));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base: 2.0, +25% + 25% = 2.5
        assert!((stats.crit_multiplier - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_attack_speed_affix() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        let gloves = Item {
            slot: EquipmentSlot::Gloves,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Gloves".to_string(),
            display_name: "Swift Gloves".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::AttackSpeed,
                value: 25.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Gloves, Some(gloves));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base: 1.0, +25% = 1.25
        assert!((stats.attack_speed_multiplier - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hp_regen_affix() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Armor".to_string(),
            display_name: "Regenerating Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::HPRegen,
                value: 50.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Armor, Some(armor));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Base: 1.0, +50% = 1.5
        assert!((stats.hp_regen_multiplier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_reflection_affix() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Armor".to_string(),
            display_name: "Thorned Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 30.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Armor, Some(armor));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // Direct percentage: 30%
        assert!((stats.damage_reflection_percent - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_reflection_stacks() {
        let attrs = Attributes::new();
        let mut equipment = Equipment::new();

        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Armor".to_string(),
            display_name: "Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 20.0,
            }],
            god_item_id: None,
        };

        let helmet = Item {
            slot: EquipmentSlot::Helmet,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Helmet".to_string(),
            display_name: "Helmet".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 15.0,
            }],
            god_item_id: None,
        };

        equipment.set(EquipmentSlot::Armor, Some(armor));
        equipment.set(EquipmentSlot::Helmet, Some(helmet));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

        // 20% + 15% = 35%
        assert!((stats.damage_reflection_percent - 35.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_base_values_without_equipment() {
        let attrs = Attributes::new();
        let stats = DerivedStats::from_attributes(&attrs);

        // Verify base values for new stats
        assert!((stats.crit_multiplier - 2.0).abs() < f64::EPSILON);
        assert!((stats.attack_speed_multiplier - 1.0).abs() < f64::EPSILON);
        assert!((stats.hp_regen_multiplier - 1.0).abs() < f64::EPSILON);
        assert!((stats.damage_reflection_percent - 0.0).abs() < f64::EPSILON);
    }
}
