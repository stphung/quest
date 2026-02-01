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
    pub fn all() -> [AttributeType; 6] {
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
    values: [u32; 6],
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}

impl Attributes {
    pub fn new() -> Self {
        Self { values: [10; 6] }
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
        (value - 10) / 2
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
}
