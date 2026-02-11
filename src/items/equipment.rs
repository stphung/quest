use super::types::{EquipmentSlot, Item};
use serde::{Deserialize, Serialize};

/// Player equipment slots.
///
/// IMPORTANT: When adding new slots, use `#[serde(default)]` to maintain
/// backward compatibility with old save files. See test_minimal_v2_save_still_loads.
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
    use super::super::types::{AttributeBonuses, Rarity};
    use super::*;

    fn create_test_item(slot: EquipmentSlot) -> Item {
        Item {
            slot,
            rarity: Rarity::Common,
            ilvl: 10,
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

    #[test]
    fn test_equip_all_seven_slots() {
        let mut eq = Equipment::new();
        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ];

        for slot in slots {
            eq.set(slot, Some(create_test_item(slot)));
        }

        assert_eq!(eq.iter_equipped().count(), 7);
        for slot in slots {
            assert!(eq.get(slot).is_some(), "Slot {:?} should be equipped", slot);
        }
    }

    #[test]
    fn test_equipment_replacement() {
        let mut eq = Equipment::new();

        let item1 = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            base_name: "Old Sword".to_string(),
            display_name: "Old Sword".to_string(),
            attributes: AttributeBonuses {
                str: 1,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        let item2 = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "New Sword".to_string(),
            display_name: "New Sword".to_string(),
            attributes: AttributeBonuses {
                str: 10,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };

        eq.set(EquipmentSlot::Weapon, Some(item1));
        assert_eq!(
            eq.get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            1
        );

        eq.set(EquipmentSlot::Weapon, Some(item2));
        assert_eq!(
            eq.get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            10
        );
        assert_eq!(eq.iter_equipped().count(), 1);
    }

    #[test]
    fn test_unequip_slot() {
        let mut eq = Equipment::new();
        eq.set(
            EquipmentSlot::Weapon,
            Some(create_test_item(EquipmentSlot::Weapon)),
        );
        assert!(eq.get(EquipmentSlot::Weapon).is_some());

        eq.set(EquipmentSlot::Weapon, None);
        assert!(eq.get(EquipmentSlot::Weapon).is_none());
        assert_eq!(eq.iter_equipped().count(), 0);
    }

    #[test]
    fn test_iter_equipped_returns_correct_items() {
        let mut eq = Equipment::new();

        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            base_name: "Sword".to_string(),
            display_name: "Sword".to_string(),
            attributes: AttributeBonuses {
                str: 5,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Plate".to_string(),
            display_name: "Plate".to_string(),
            attributes: AttributeBonuses {
                con: 8,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };

        eq.set(EquipmentSlot::Weapon, Some(weapon));
        eq.set(EquipmentSlot::Armor, Some(armor));

        let items: Vec<&Item> = eq.iter_equipped().collect();
        assert_eq!(items.len(), 2);

        let total_str: u32 = items.iter().map(|i| i.attributes.str).sum();
        let total_con: u32 = items.iter().map(|i| i.attributes.con).sum();
        assert_eq!(total_str, 5);
        assert_eq!(total_con, 8);
    }
}
