use crate::core::constants::{BASE_ATTRIBUTE_VALUE, NUM_ATTRIBUTES};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttributeType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AttributeType {
    pub fn all() -> [AttributeType; NUM_ATTRIBUTES] {
        [
            AttributeType::Strength,
            AttributeType::Dexterity,
            AttributeType::Constitution,
            AttributeType::Intelligence,
            AttributeType::Wisdom,
            AttributeType::Charisma,
        ]
    }

    pub fn abbrev(&self) -> &str {
        match self {
            AttributeType::Strength => "STR",
            AttributeType::Dexterity => "DEX",
            AttributeType::Constitution => "CON",
            AttributeType::Intelligence => "INT",
            AttributeType::Wisdom => "WIS",
            AttributeType::Charisma => "CHA",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            AttributeType::Strength => 0,
            AttributeType::Dexterity => 1,
            AttributeType::Constitution => 2,
            AttributeType::Intelligence => 3,
            AttributeType::Wisdom => 4,
            AttributeType::Charisma => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    values: [u32; NUM_ATTRIBUTES],
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            values: [BASE_ATTRIBUTE_VALUE; NUM_ATTRIBUTES],
        }
    }

    pub fn get(&self, attr: AttributeType) -> u32 {
        self.values[attr.index()]
    }

    pub fn set(&mut self, attr: AttributeType, value: u32) {
        self.values[attr.index()] = value;
    }

    pub fn increment(&mut self, attr: AttributeType) {
        self.values[attr.index()] = self.values[attr.index()].saturating_add(1);
    }

    pub fn modifier(&self, attr: AttributeType) -> i32 {
        let value = self.get(attr) as i32;
        (value - BASE_ATTRIBUTE_VALUE as i32) / 2
    }

    /// Adds another Attributes' values to this one (for equipment bonuses).
    pub fn add(&mut self, other: &Attributes) {
        for attr in AttributeType::all() {
            self.values[attr.index()] += other.get(attr);
        }
    }

    /// Creates Attributes from individual attribute bonuses.
    pub fn from_bonuses(str: u32, dex: u32, con: u32, int: u32, wis: u32, cha: u32) -> Self {
        let mut attrs = Self::new();
        attrs.values[0] = str;
        attrs.values[1] = dex;
        attrs.values[2] = con;
        attrs.values[3] = int;
        attrs.values[4] = wis;
        attrs.values[5] = cha;
        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_attributes() {
        let attrs = Attributes::new();
        for attr_type in AttributeType::all() {
            assert_eq!(attrs.get(attr_type), 10);
        }
    }

    #[test]
    fn test_get_set() {
        let mut attrs = Attributes::new();
        attrs.set(AttributeType::Strength, 16);
        assert_eq!(attrs.get(AttributeType::Strength), 16);
        assert_eq!(attrs.get(AttributeType::Dexterity), 10);
    }

    #[test]
    fn test_increment() {
        let mut attrs = Attributes::new();
        attrs.increment(AttributeType::Wisdom);
        assert_eq!(attrs.get(AttributeType::Wisdom), 11);
    }

    #[test]
    fn test_modifier_calculation() {
        let mut attrs = Attributes::new();

        // 10-11 = +0
        attrs.set(AttributeType::Strength, 10);
        assert_eq!(attrs.modifier(AttributeType::Strength), 0);
        attrs.set(AttributeType::Strength, 11);
        assert_eq!(attrs.modifier(AttributeType::Strength), 0);

        // 12-13 = +1
        attrs.set(AttributeType::Strength, 12);
        assert_eq!(attrs.modifier(AttributeType::Strength), 1);
        attrs.set(AttributeType::Strength, 13);
        assert_eq!(attrs.modifier(AttributeType::Strength), 1);

        // 14-15 = +2
        attrs.set(AttributeType::Strength, 14);
        assert_eq!(attrs.modifier(AttributeType::Strength), 2);

        // 20 = +5
        attrs.set(AttributeType::Strength, 20);
        assert_eq!(attrs.modifier(AttributeType::Strength), 5);

        // 8-9 = -1
        attrs.set(AttributeType::Strength, 8);
        assert_eq!(attrs.modifier(AttributeType::Strength), -1);
    }

    #[test]
    fn test_attribute_type_abbrev() {
        assert_eq!(AttributeType::Strength.abbrev(), "STR");
        assert_eq!(AttributeType::Dexterity.abbrev(), "DEX");
        assert_eq!(AttributeType::Constitution.abbrev(), "CON");
        assert_eq!(AttributeType::Intelligence.abbrev(), "INT");
        assert_eq!(AttributeType::Wisdom.abbrev(), "WIS");
        assert_eq!(AttributeType::Charisma.abbrev(), "CHA");
    }

    #[test]
    fn test_all_returns_six_types() {
        let all = AttributeType::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], AttributeType::Strength);
        assert_eq!(all[1], AttributeType::Dexterity);
        assert_eq!(all[2], AttributeType::Constitution);
        assert_eq!(all[3], AttributeType::Intelligence);
        assert_eq!(all[4], AttributeType::Wisdom);
        assert_eq!(all[5], AttributeType::Charisma);
    }

    #[test]
    fn test_index_returns_unique_values() {
        let all = AttributeType::all();
        for (i, attr) in all.iter().enumerate() {
            assert_eq!(attr.index(), i);
        }
    }

    #[test]
    fn test_modifier_below_ten() {
        let mut attrs = Attributes::new();

        // Rust integer division truncates toward zero: -1/2 = 0, -3/2 = -1, etc.
        // value 9: (9-10)/2 = -1/2 = 0
        attrs.set(AttributeType::Strength, 9);
        assert_eq!(attrs.modifier(AttributeType::Strength), 0);

        // value 7: (7-10)/2 = -3/2 = -1
        attrs.set(AttributeType::Strength, 7);
        assert_eq!(attrs.modifier(AttributeType::Strength), -1);

        // value 6: (6-10)/2 = -4/2 = -2
        attrs.set(AttributeType::Strength, 6);
        assert_eq!(attrs.modifier(AttributeType::Strength), -2);

        // value 5: (5-10)/2 = -5/2 = -2
        attrs.set(AttributeType::Strength, 5);
        assert_eq!(attrs.modifier(AttributeType::Strength), -2);

        // value 4: (4-10)/2 = -6/2 = -3
        attrs.set(AttributeType::Strength, 4);
        assert_eq!(attrs.modifier(AttributeType::Strength), -3);

        // value 1: (1-10)/2 = -9/2 = -4
        attrs.set(AttributeType::Strength, 1);
        assert_eq!(attrs.modifier(AttributeType::Strength), -4);

        // value 0: (0-10)/2 = -10/2 = -5
        attrs.set(AttributeType::Strength, 0);
        assert_eq!(attrs.modifier(AttributeType::Strength), -5);
    }

    #[test]
    fn test_increment_saturates_at_max() {
        let mut attrs = Attributes::new();
        attrs.set(AttributeType::Dexterity, u32::MAX);
        attrs.increment(AttributeType::Dexterity);
        assert_eq!(attrs.get(AttributeType::Dexterity), u32::MAX);
    }

    #[test]
    fn test_add_combines_attributes() {
        let mut base = Attributes::new(); // all 10
        let bonuses = Attributes::from_bonuses(2, 3, 0, 1, 0, 5);
        base.add(&bonuses);

        assert_eq!(base.get(AttributeType::Strength), 12);
        assert_eq!(base.get(AttributeType::Dexterity), 13);
        assert_eq!(base.get(AttributeType::Constitution), 10); // 10 + 0
        assert_eq!(base.get(AttributeType::Intelligence), 11);
        assert_eq!(base.get(AttributeType::Wisdom), 10);
        assert_eq!(base.get(AttributeType::Charisma), 15);
    }

    #[test]
    fn test_from_bonuses() {
        let attrs = Attributes::from_bonuses(1, 2, 3, 4, 5, 6);
        assert_eq!(attrs.get(AttributeType::Strength), 1);
        assert_eq!(attrs.get(AttributeType::Dexterity), 2);
        assert_eq!(attrs.get(AttributeType::Constitution), 3);
        assert_eq!(attrs.get(AttributeType::Intelligence), 4);
        assert_eq!(attrs.get(AttributeType::Wisdom), 5);
        assert_eq!(attrs.get(AttributeType::Charisma), 6);
    }

    #[test]
    fn test_default_equals_new() {
        let from_new = Attributes::new();
        let from_default = Attributes::default();
        for attr in AttributeType::all() {
            assert_eq!(from_new.get(attr), from_default.get(attr));
        }
    }
}
