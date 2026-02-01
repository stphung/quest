use crate::attributes::{AttributeType, Attributes};

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
    pub fn from_attributes(attrs: &Attributes, base_prestige_mult: f64) -> Self {
        let str_mod = attrs.modifier(AttributeType::Strength);
        let dex_mod = attrs.modifier(AttributeType::Dexterity);
        let con_mod = attrs.modifier(AttributeType::Constitution);
        let int_mod = attrs.modifier(AttributeType::Intelligence);
        let wis_mod = attrs.modifier(AttributeType::Wisdom);
        let cha_mod = attrs.modifier(AttributeType::Charisma);

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

    pub fn prestige_multiplier(base_multiplier: f64, attrs: &Attributes) -> f64 {
        let cha_mod = attrs.modifier(AttributeType::Charisma);
        base_multiplier + (cha_mod as f64 * 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derived_stats_base() {
        let attrs = Attributes::new();
        let stats = DerivedStats::from_attributes(&attrs, 1.0);

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

        let stats = DerivedStats::from_attributes(&attrs, 1.0);

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

        let stats = DerivedStats::from_attributes(&attrs, 1.0);

        // Should never go below 1 for damage/hp
        assert_eq!(stats.max_hp, 40); // 50 + (-1 * 10)
        assert_eq!(stats.physical_damage, 3); // 5 + (-1 * 2)
    }
}
