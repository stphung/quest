#![allow(dead_code)]
use crate::items::{EquipmentSlot, Item};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub helmet: Option<Item>,
    pub gloves: Option<Item>,
    pub boots: Option<Item>,
    pub amulet: Option<Item>,
    pub ring: Option<Item>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            weapon: None,
            armor: None,
            helmet: None,
            gloves: None,
            boots: None,
            amulet: None,
            ring: None,
        }
    }

    pub fn get(&self, slot: EquipmentSlot) -> &Option<Item> {
        match slot {
            EquipmentSlot::Weapon => &self.weapon,
            EquipmentSlot::Armor => &self.armor,
            EquipmentSlot::Helmet => &self.helmet,
            EquipmentSlot::Gloves => &self.gloves,
            EquipmentSlot::Boots => &self.boots,
            EquipmentSlot::Amulet => &self.amulet,
            EquipmentSlot::Ring => &self.ring,
        }
    }

    pub fn set(&mut self, slot: EquipmentSlot, item: Option<Item>) {
        match slot {
            EquipmentSlot::Weapon => self.weapon = item,
            EquipmentSlot::Armor => self.armor = item,
            EquipmentSlot::Helmet => self.helmet = item,
            EquipmentSlot::Gloves => self.gloves = item,
            EquipmentSlot::Boots => self.boots = item,
            EquipmentSlot::Amulet => self.amulet = item,
            EquipmentSlot::Ring => self.ring = item,
        }
    }

    pub fn iter_equipped(&self) -> impl Iterator<Item = &Item> {
        [
            &self.weapon,
            &self.armor,
            &self.helmet,
            &self.gloves,
            &self.boots,
            &self.amulet,
            &self.ring,
        ]
        .into_iter()
        .filter_map(|item| item.as_ref())
    }
}

impl Default for Equipment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{AttributeBonuses, Rarity};

    fn create_test_item(slot: EquipmentSlot) -> Item {
        Item {
            slot,
            rarity: Rarity::Common,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        }
    }

    #[test]
    fn test_equipment_starts_empty() {
        let eq = Equipment::new();
        assert!(eq.weapon.is_none());
        assert!(eq.armor.is_none());
        assert_eq!(eq.iter_equipped().count(), 0);
    }

    #[test]
    fn test_equipment_get_set() {
        let mut eq = Equipment::new();
        let weapon = create_test_item(EquipmentSlot::Weapon);

        eq.set(EquipmentSlot::Weapon, Some(weapon.clone()));
        assert_eq!(eq.get(EquipmentSlot::Weapon), &Some(weapon));
    }

    #[test]
    fn test_iter_equipped() {
        let mut eq = Equipment::new();
        eq.set(
            EquipmentSlot::Weapon,
            Some(create_test_item(EquipmentSlot::Weapon)),
        );
        eq.set(
            EquipmentSlot::Armor,
            Some(create_test_item(EquipmentSlot::Armor)),
        );

        assert_eq!(eq.iter_equipped().count(), 2);
    }
}
