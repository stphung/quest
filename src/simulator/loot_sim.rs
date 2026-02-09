//! Loot drop simulation.

use rand::Rng;

/// Simulated item rarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimRarity {
    Common,
    Magic,
    Rare,
    Epic,
    Legendary,
}

impl SimRarity {
    /// Get the damage/stat multiplier for this rarity.
    pub fn multiplier(&self) -> f64 {
        match self {
            SimRarity::Common => 1.0,
            SimRarity::Magic => 1.2,
            SimRarity::Rare => 1.5,
            SimRarity::Epic => 2.0,
            SimRarity::Legendary => 3.0,
        }
    }

    /// Get the name for display.
    pub fn name(&self) -> &'static str {
        match self {
            SimRarity::Common => "Common",
            SimRarity::Magic => "Magic",
            SimRarity::Rare => "Rare",
            SimRarity::Epic => "Epic",
            SimRarity::Legendary => "Legendary",
        }
    }
}

/// Simulated equipment slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimSlot {
    Weapon,
    Armor,
    Helmet,
    Gloves,
    Boots,
    Amulet,
    Ring,
}

impl SimSlot {
    pub const ALL: [SimSlot; 7] = [
        SimSlot::Weapon,
        SimSlot::Armor,
        SimSlot::Helmet,
        SimSlot::Gloves,
        SimSlot::Boots,
        SimSlot::Amulet,
        SimSlot::Ring,
    ];
}

/// Simulated item.
#[derive(Debug, Clone)]
pub struct SimItem {
    pub slot: SimSlot,
    pub rarity: SimRarity,
    pub ilvl: u32,
    pub power_score: f64, // Combined power value for comparison
}

impl SimItem {
    /// Generate a random item for the given zone.
    pub fn generate(zone_id: usize, rarity: SimRarity, rng: &mut impl Rng) -> Self {
        let slot = SimSlot::ALL[rng.gen_range(0..7)];
        let ilvl = (zone_id as u32) * 10;

        // Power score = ilvl * rarity multiplier
        let power_score = (ilvl as f64) * rarity.multiplier();

        Self {
            slot,
            rarity,
            ilvl,
            power_score,
        }
    }
}

/// Simulated equipment set.
#[derive(Debug, Clone, Default)]
pub struct SimEquipment {
    pub weapon: Option<SimItem>,
    pub armor: Option<SimItem>,
    pub helmet: Option<SimItem>,
    pub gloves: Option<SimItem>,
    pub boots: Option<SimItem>,
    pub amulet: Option<SimItem>,
    pub ring: Option<SimItem>,
}

impl SimEquipment {
    /// Get the item in a slot.
    pub fn get(&self, slot: SimSlot) -> &Option<SimItem> {
        match slot {
            SimSlot::Weapon => &self.weapon,
            SimSlot::Armor => &self.armor,
            SimSlot::Helmet => &self.helmet,
            SimSlot::Gloves => &self.gloves,
            SimSlot::Boots => &self.boots,
            SimSlot::Amulet => &self.amulet,
            SimSlot::Ring => &self.ring,
        }
    }

    /// Equip an item if it's an upgrade.
    pub fn equip_if_upgrade(&mut self, item: SimItem) -> bool {
        let current = match item.slot {
            SimSlot::Weapon => &mut self.weapon,
            SimSlot::Armor => &mut self.armor,
            SimSlot::Helmet => &mut self.helmet,
            SimSlot::Gloves => &mut self.gloves,
            SimSlot::Boots => &mut self.boots,
            SimSlot::Amulet => &mut self.amulet,
            SimSlot::Ring => &mut self.ring,
        };

        let should_equip = match current {
            None => true,
            Some(existing) => item.power_score > existing.power_score,
        };

        if should_equip {
            *current = Some(item);
            true
        } else {
            false
        }
    }

    /// Calculate total power multiplier from gear.
    pub fn total_damage_mult(&self) -> f64 {
        let weapon_mult = self
            .weapon
            .as_ref()
            .map(|i| 1.0 + (i.ilvl as f64 / 50.0) * i.rarity.multiplier())
            .unwrap_or(1.0);

        // Other slots add smaller bonuses
        let other_bonus: f64 = [
            &self.armor,
            &self.helmet,
            &self.gloves,
            &self.boots,
            &self.amulet,
            &self.ring,
        ]
        .iter()
        .filter_map(|slot| slot.as_ref())
        .map(|item| 0.05 * item.rarity.multiplier())
        .sum();

        weapon_mult * (1.0 + other_bonus)
    }

    /// Calculate total HP multiplier from gear.
    pub fn total_hp_mult(&self) -> f64 {
        let armor_mult = self
            .armor
            .as_ref()
            .map(|i| 1.0 + (i.ilvl as f64 / 100.0) * i.rarity.multiplier())
            .unwrap_or(1.0);

        armor_mult
    }

    /// Calculate crit bonus from gear.
    pub fn total_crit_bonus(&self) -> f64 {
        self.gloves
            .as_ref()
            .map(|i| 0.05 * i.rarity.multiplier())
            .unwrap_or(0.0)
    }

    /// Count equipped items.
    pub fn equipped_count(&self) -> usize {
        [
            &self.weapon,
            &self.armor,
            &self.helmet,
            &self.gloves,
            &self.boots,
            &self.amulet,
            &self.ring,
        ]
        .iter()
        .filter(|s| s.is_some())
        .count()
    }

    /// Get average ilvl of equipped items.
    pub fn average_ilvl(&self) -> f64 {
        let items: Vec<_> = [
            &self.weapon,
            &self.armor,
            &self.helmet,
            &self.gloves,
            &self.boots,
            &self.amulet,
            &self.ring,
        ]
        .iter()
        .filter_map(|s| s.as_ref())
        .collect();

        if items.is_empty() {
            0.0
        } else {
            items.iter().map(|i| i.ilvl as f64).sum::<f64>() / items.len() as f64
        }
    }
}

/// Roll for item drop from a mob (not boss).
pub fn roll_mob_drop(zone_id: usize, rng: &mut impl Rng) -> Option<SimItem> {
    // 15% base drop chance
    if rng.gen::<f64>() > 0.15 {
        return None;
    }

    // Rarity distribution (no legendaries from mobs)
    let rarity = {
        let roll = rng.gen::<f64>();
        if roll < 0.60 {
            SimRarity::Common
        } else if roll < 0.88 {
            SimRarity::Magic
        } else if roll < 0.98 {
            SimRarity::Rare
        } else {
            SimRarity::Epic
        }
    };

    Some(SimItem::generate(zone_id, rarity, rng))
}

/// Roll for item drop from a boss (guaranteed drop, can be legendary).
pub fn roll_boss_drop(zone_id: usize, is_final_zone: bool, rng: &mut impl Rng) -> SimItem {
    let rarity = if is_final_zone {
        // Zone 10 boss: better rates
        let roll = rng.gen::<f64>();
        if roll < 0.20 {
            SimRarity::Magic
        } else if roll < 0.60 {
            SimRarity::Rare
        } else if roll < 0.90 {
            SimRarity::Epic
        } else {
            SimRarity::Legendary
        }
    } else {
        // Normal boss
        let roll = rng.gen::<f64>();
        if roll < 0.40 {
            SimRarity::Magic
        } else if roll < 0.75 {
            SimRarity::Rare
        } else if roll < 0.95 {
            SimRarity::Epic
        } else {
            SimRarity::Legendary
        }
    };

    SimItem::generate(zone_id, rarity, rng)
}

/// Statistics about loot drops.
#[derive(Debug, Clone, Default)]
pub struct LootStats {
    pub total_drops: u32,
    pub upgrades_equipped: u32,
    pub common_drops: u32,
    pub magic_drops: u32,
    pub rare_drops: u32,
    pub epic_drops: u32,
    pub legendary_drops: u32,
}

impl LootStats {
    pub fn record_drop(&mut self, item: &SimItem, was_upgrade: bool) {
        self.total_drops += 1;
        if was_upgrade {
            self.upgrades_equipped += 1;
        }

        match item.rarity {
            SimRarity::Common => self.common_drops += 1,
            SimRarity::Magic => self.magic_drops += 1,
            SimRarity::Rare => self.rare_drops += 1,
            SimRarity::Epic => self.epic_drops += 1,
            SimRarity::Legendary => self.legendary_drops += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_generation() {
        let mut rng = rand::thread_rng();
        let item = SimItem::generate(5, SimRarity::Rare, &mut rng);
        assert_eq!(item.ilvl, 50);
        assert_eq!(item.rarity, SimRarity::Rare);
    }

    #[test]
    fn test_equipment_upgrade() {
        let mut equip = SimEquipment::default();
        let mut rng = rand::thread_rng();

        let item1 = SimItem::generate(1, SimRarity::Common, &mut rng);
        let slot = item1.slot;
        assert!(equip.equip_if_upgrade(item1));

        // Same rarity, higher ilvl should upgrade
        let mut item2 = SimItem::generate(5, SimRarity::Common, &mut rng);
        item2.slot = slot;
        item2.power_score = 100.0; // Force higher score
        assert!(equip.equip_if_upgrade(item2));
    }

    #[test]
    fn test_boss_can_drop_legendary() {
        let mut rng = rand::thread_rng();
        let mut found_legendary = false;

        for _ in 0..500 {
            let item = roll_boss_drop(10, true, &mut rng);
            if item.rarity == SimRarity::Legendary {
                found_legendary = true;
                break;
            }
        }

        assert!(found_legendary, "Boss should be able to drop legendary");
    }
}
