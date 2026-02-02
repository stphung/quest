use crate::attributes::{AttributeType, Attributes};
use crate::equipment::Equipment;

#[derive(Debug, Clone, Copy)]
pub struct DerivedStats {
    pub max_hp: u32,
    pub physical_damage: u32,
    pub magic_damage: u32,
    pub defense: u32,
    pub crit_chance_percent: u32,
    pub xp_multiplier: f64,
}

impl DerivedStats {
    /// Creates derived stats from attributes with no equipment bonuses.
    /// Primarily used for backward compatibility and tests.
    ///
    /// Note: Prestige multiplier is calculated separately using the
    /// `prestige_multiplier()` static method, not in this constructor.
    #[allow(dead_code)]
    pub fn from_attributes(attrs: &Attributes) -> Self {
        Self::calculate_derived_stats(attrs, &Equipment::new())
    }

    /// Calculates derived stats from attributes and equipment bonuses.
    ///
    /// Equipment bonuses are added to base attributes before calculating modifiers.
    pub fn calculate_derived_stats(attrs: &Attributes, equipment: &Equipment) -> Self {
        // Sum equipment attribute bonuses
        let mut total_attrs = *attrs;
        for item in equipment.iter_equipped() {
            total_attrs.set(
                AttributeType::Strength,
                total_attrs.get(AttributeType::Strength) + item.attributes.str,
            );
            total_attrs.set(
                AttributeType::Dexterity,
                total_attrs.get(AttributeType::Dexterity) + item.attributes.dex,
            );
            total_attrs.set(
                AttributeType::Constitution,
                total_attrs.get(AttributeType::Constitution) + item.attributes.con,
            );
            total_attrs.set(
                AttributeType::Intelligence,
                total_attrs.get(AttributeType::Intelligence) + item.attributes.int,
            );
            total_attrs.set(
                AttributeType::Wisdom,
                total_attrs.get(AttributeType::Wisdom) + item.attributes.wis,
            );
            total_attrs.set(
                AttributeType::Charisma,
                total_attrs.get(AttributeType::Charisma) + item.attributes.cha,
            );
        }

        let str_mod = total_attrs.modifier(AttributeType::Strength);
        let dex_mod = total_attrs.modifier(AttributeType::Dexterity);
        let con_mod = total_attrs.modifier(AttributeType::Constitution);
        let int_mod = total_attrs.modifier(AttributeType::Intelligence);
        let wis_mod = total_attrs.modifier(AttributeType::Wisdom);

        // Max HP = 50 + (CON_mod × 10)
        let max_hp = (50 + con_mod * 10).max(1) as u32;

        // Physical Damage = 5 + (STR_mod × 2)
        let physical_damage = (5 + str_mod * 2).max(1) as u32;

        // Magic Damage = 5 + (INT_mod × 2)
        let magic_damage = (5 + int_mod * 2).max(1) as u32;

        // Defense = 0 + (DEX_mod × 1)
        let defense = dex_mod.max(0) as u32;

        // Crit Chance = 5% + (DEX_mod × 1%)
        let crit_chance_percent = (5 + dex_mod).max(0) as u32;

        // XP Multiplier = 1.0 + (WIS_mod × 0.05)
        let xp_multiplier = 1.0 + (wis_mod as f64 * 0.05);

        Self {
            max_hp,
            physical_damage,
            magic_damage,
            defense,
            crit_chance_percent,
            xp_multiplier,
        }
    }

    pub fn total_damage(&self) -> u32 {
        self.physical_damage + self.magic_damage
    }

    /// Calculates prestige multiplier with equipment bonuses included.
    #[allow(dead_code)]
    pub fn prestige_multiplier_with_equipment(
        base_multiplier: f64,
        attrs: &Attributes,
        equipment: &Equipment,
    ) -> f64 {
        // Sum equipment charisma bonuses
        let mut total_cha = attrs.get(AttributeType::Charisma);
        for item in equipment.iter_equipped() {
            total_cha += item.attributes.cha;
        }

        let mut temp_attrs = *attrs;
        temp_attrs.set(AttributeType::Charisma, total_cha);
        let cha_mod = temp_attrs.modifier(AttributeType::Charisma);
        base_multiplier + (cha_mod as f64 * 0.1)
    }

    pub fn prestige_multiplier(base_multiplier: f64, attrs: &Attributes) -> f64 {
        let cha_mod = attrs.modifier(AttributeType::Charisma);
        base_multiplier + (cha_mod as f64 * 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};

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
        };

        equipment.set(EquipmentSlot::Weapon, Some(weapon));

        let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment);

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
        };

        equipment.set(EquipmentSlot::Amulet, Some(amulet));

        let prestige = DerivedStats::prestige_multiplier_with_equipment(2.0, &attrs, &equipment);
        // Base: CHA 10 (+0 mod) -> 2.0 + 0 = 2.0
        // With equipment: CHA 13 (+1 mod) -> 2.0 + 0.1 = 2.1
        assert_eq!(prestige, 2.1);
    }
}
