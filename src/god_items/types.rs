use serde::{Deserialize, Serialize};

use crate::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

/// Unique identifier for each god item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GodItemId {
    Asprika,
    Sleipnir,
    Megingjord,
}

/// Passive ability unique to a god item.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GodItemPassive {
    /// Reduces all incoming damage by the given percentage (applied after defense).
    DivineBulwark { damage_reduction_percent: f64 },
    /// Increases attack speed by the given percentage.
    Windborne { attack_speed_percent: f64 },
    /// Increases all damage by the given percentage.
    GiantsMight { damage_percent: f64 },
}

/// Non-combat bonus unique to a god item.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GodItemBonus {
    /// Reduces regen delay between encounters (multiplicative with Haven).
    Swiftstrider { regen_reduction_percent: f64 },
    /// Reduces dungeon room movement timers (multiplicative).
    Swiftfoot { dungeon_speed_percent: f64 },
    /// Reduces fishing phase timers (multiplicative, stacks with Haven).
    NimbleHands { fishing_reduction_percent: f64 },
}

/// Static definition of a god item.
pub struct GodItemDefinition {
    pub id: GodItemId,
    pub name: &'static str,
    pub title: &'static str,
    pub slot: EquipmentSlot,
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
    pub passive: GodItemPassive,
    pub bonuses: Vec<GodItemBonus>,
}

impl GodItemDefinition {
    /// Creates the Item struct for this god item definition.
    pub fn to_item(&self) -> Item {
        Item {
            slot: self.slot,
            rarity: Rarity::Mythic,
            ilvl: 100,
            tier: 9,
            base_name: self.name.to_string(),
            display_name: self.name.to_string(),
            attributes: self.attributes.clone(),
            affixes: self.affixes.clone(),
            god_item_id: Some(self.id),
        }
    }
}

/// Returns the definition for Asprika.
pub fn asprika_definition() -> GodItemDefinition {
    GodItemDefinition {
        id: GodItemId::Asprika,
        name: "Asprika",
        title: "Armor of the \u{00C6}sir",
        slot: EquipmentSlot::Armor,
        attributes: AttributeBonuses {
            str: 0,
            dex: 0,
            con: 40,
            int: 0,
            wis: 20,
            cha: 0,
        },
        affixes: vec![Affix {
            affix_type: AffixType::XPGain,
            value: 40.0,
        }],
        passive: GodItemPassive::DivineBulwark {
            damage_reduction_percent: 30.0,
        },
        bonuses: vec![],
    }
}

/// Returns the definition for Sleipnir.
pub fn sleipnir_definition() -> GodItemDefinition {
    GodItemDefinition {
        id: GodItemId::Sleipnir,
        name: "Sleipnir",
        title: "Boots of the Eight-Legged",
        slot: EquipmentSlot::Boots,
        attributes: AttributeBonuses {
            str: 0,
            dex: 40,
            con: 0,
            int: 0,
            wis: 20,
            cha: 0,
        },
        affixes: vec![Affix {
            affix_type: AffixType::XPGain,
            value: 40.0,
        }],
        passive: GodItemPassive::Windborne {
            attack_speed_percent: 100.0,
        },
        bonuses: vec![
            GodItemBonus::Swiftstrider {
                regen_reduction_percent: 50.0,
            },
            GodItemBonus::Swiftfoot {
                dungeon_speed_percent: 50.0,
            },
            GodItemBonus::NimbleHands {
                fishing_reduction_percent: 50.0,
            },
        ],
    }
}

/// Returns the definition for Megingjord.
pub fn megingjord_definition() -> GodItemDefinition {
    GodItemDefinition {
        id: GodItemId::Megingjord,
        name: "Megingjord",
        title: "Belt of Giant Strength",
        slot: EquipmentSlot::Ring,
        attributes: AttributeBonuses {
            str: 40,
            dex: 0,
            con: 20,
            int: 0,
            wis: 0,
            cha: 0,
        },
        affixes: vec![Affix {
            affix_type: AffixType::XPGain,
            value: 40.0,
        }],
        passive: GodItemPassive::GiantsMight {
            damage_percent: 150.0,
        },
        bonuses: vec![],
    }
}

/// Look up a god item definition by ID.
pub fn get_god_item_definition(id: GodItemId) -> GodItemDefinition {
    match id {
        GodItemId::Asprika => asprika_definition(),
        GodItemId::Sleipnir => sleipnir_definition(),
        GodItemId::Megingjord => megingjord_definition(),
    }
}

/// Cached god item bonuses, computed once when equipment changes.
/// Avoids 4+ per-tick linear scans through equipment slots.
#[derive(Debug, Clone, Copy, Default)]
pub struct CachedGodItemBonuses {
    /// Divine Bulwark damage reduction %
    pub damage_reduction_percent: f64,
    /// Windborne attack speed %
    pub attack_speed_percent: f64,
    /// Giant's Might damage %
    pub damage_percent: f64,
    /// Swiftstrider regen reduction %
    pub regen_reduction_percent: f64,
    /// Swiftfoot dungeon speed %
    pub dungeon_speed_percent: f64,
    /// NimbleHands fishing timer reduction %
    pub fishing_reduction_percent: f64,
}

impl CachedGodItemBonuses {
    /// Compute all god item bonuses in a single pass over equipped items.
    pub fn compute(equipment: &crate::items::Equipment) -> Self {
        let mut bonuses = Self::default();
        for item in equipment.iter_equipped() {
            if let Some(id) = item.god_item_id {
                let def = get_god_item_definition(id);
                match def.passive {
                    GodItemPassive::DivineBulwark {
                        damage_reduction_percent,
                    } => bonuses.damage_reduction_percent = damage_reduction_percent,
                    GodItemPassive::Windborne {
                        attack_speed_percent,
                    } => bonuses.attack_speed_percent = attack_speed_percent,
                    GodItemPassive::GiantsMight { damage_percent } => {
                        bonuses.damage_percent = damage_percent
                    }
                }
                for bonus in &def.bonuses {
                    match bonus {
                        GodItemBonus::Swiftstrider {
                            regen_reduction_percent,
                        } => bonuses.regen_reduction_percent = *regen_reduction_percent,
                        GodItemBonus::Swiftfoot {
                            dungeon_speed_percent,
                        } => bonuses.dungeon_speed_percent = *dungeon_speed_percent,
                        GodItemBonus::NimbleHands {
                            fishing_reduction_percent,
                        } => bonuses.fishing_reduction_percent = *fishing_reduction_percent,
                    }
                }
            }
        }
        bonuses
    }
}

/// Returns the god item damage reduction percent from equipped items, if any.
pub fn equipped_god_item_dr(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            if let GodItemPassive::DivineBulwark {
                damage_reduction_percent,
            } = def.passive
            {
                return damage_reduction_percent;
            }
        }
    }
    0.0
}

/// Returns the god item attack speed bonus percent from equipped items, if any.
pub fn equipped_god_item_attack_speed_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            if let GodItemPassive::Windborne {
                attack_speed_percent,
            } = def.passive
            {
                return attack_speed_percent;
            }
        }
    }
    0.0
}

/// Returns the god item damage bonus percent from equipped items, if any.
pub fn equipped_god_item_damage_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            if let GodItemPassive::GiantsMight { damage_percent } = def.passive {
                return damage_percent;
            }
        }
    }
    0.0
}

/// Returns the god item regen reduction percent from equipped items, if any.
pub fn equipped_god_item_regen_reduction_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            for bonus in &def.bonuses {
                if let GodItemBonus::Swiftstrider {
                    regen_reduction_percent,
                } = bonus
                {
                    return *regen_reduction_percent;
                }
            }
        }
    }
    0.0
}

/// Returns the god item dungeon movement speed bonus percent (0.0 if none).
pub fn equipped_god_item_dungeon_speed_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            for bonus in &def.bonuses {
                if let GodItemBonus::Swiftfoot {
                    dungeon_speed_percent,
                } = bonus
                {
                    return *dungeon_speed_percent;
                }
            }
        }
    }
    0.0
}

/// Returns the god item fishing timer reduction percent (0.0 if none).
pub fn equipped_god_item_fishing_reduction_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            for bonus in &def.bonuses {
                if let GodItemBonus::NimbleHands {
                    fishing_reduction_percent,
                } = bonus
                {
                    return *fishing_reduction_percent;
                }
            }
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asprika_definition_is_mythic_armor() {
        let def = asprika_definition();
        assert_eq!(def.id, GodItemId::Asprika);
        assert_eq!(def.slot, EquipmentSlot::Armor);
        assert_eq!(def.name, "Asprika");
    }

    #[test]
    fn test_asprika_has_divine_bulwark_passive() {
        let def = asprika_definition();
        match def.passive {
            GodItemPassive::DivineBulwark {
                damage_reduction_percent,
            } => {
                assert!((damage_reduction_percent - 30.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected DivineBulwark passive"),
        }
    }

    #[test]
    fn test_asprika_has_no_bonuses() {
        let def = asprika_definition();
        assert!(def.bonuses.is_empty());
    }

    #[test]
    fn test_asprika_has_xp_gain_affix_only() {
        let def = asprika_definition();
        assert_eq!(def.affixes.len(), 1);
        assert_eq!(def.affixes[0].affix_type, AffixType::XPGain);
    }

    #[test]
    fn test_asprika_has_con_primary_wis_supporting() {
        let def = asprika_definition();
        assert!(def.attributes.con > 0, "CON should be primary");
        assert!(def.attributes.wis > 0, "WIS should be supporting");
        assert_eq!(def.attributes.str, 0);
        assert_eq!(def.attributes.dex, 0);
        assert_eq!(def.attributes.int, 0);
        assert_eq!(def.attributes.cha, 0);
    }

    #[test]
    fn test_to_item_creates_mythic_item() {
        let def = asprika_definition();
        let item = def.to_item();
        assert_eq!(item.rarity, Rarity::Mythic);
        assert_eq!(item.slot, EquipmentSlot::Armor);
        assert_eq!(item.display_name, "Asprika");
        assert_eq!(item.god_item_id, Some(GodItemId::Asprika));
    }

    #[test]
    fn test_get_god_item_definition() {
        let def = get_god_item_definition(GodItemId::Asprika);
        assert_eq!(def.id, GodItemId::Asprika);
        let def = get_god_item_definition(GodItemId::Sleipnir);
        assert_eq!(def.id, GodItemId::Sleipnir);
        let def = get_god_item_definition(GodItemId::Megingjord);
        assert_eq!(def.id, GodItemId::Megingjord);
    }

    // Sleipnir tests

    #[test]
    fn test_sleipnir_definition_is_mythic_boots() {
        let def = sleipnir_definition();
        assert_eq!(def.id, GodItemId::Sleipnir);
        assert_eq!(def.slot, EquipmentSlot::Boots);
        assert_eq!(def.name, "Sleipnir");
        let item = def.to_item();
        assert_eq!(item.rarity, Rarity::Mythic);
    }

    #[test]
    fn test_sleipnir_has_windborne_passive() {
        let def = sleipnir_definition();
        match def.passive {
            GodItemPassive::Windborne {
                attack_speed_percent,
            } => {
                assert!((attack_speed_percent - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Windborne passive"),
        }
    }

    #[test]
    fn test_sleipnir_has_swiftstrider_bonus() {
        let def = sleipnir_definition();
        let has_bonus = def.bonuses.iter().any(|b| {
            matches!(b, GodItemBonus::Swiftstrider { regen_reduction_percent } if (*regen_reduction_percent - 50.0).abs() < f64::EPSILON)
        });
        assert!(has_bonus, "Expected Swiftstrider bonus with 50%");
    }

    #[test]
    fn test_sleipnir_has_dex_primary_wis_supporting() {
        let def = sleipnir_definition();
        assert_eq!(def.attributes.dex, 40);
        assert_eq!(def.attributes.wis, 20);
        assert_eq!(def.attributes.str, 0);
        assert_eq!(def.attributes.con, 0);
    }

    #[test]
    fn test_sleipnir_has_three_bonuses() {
        let def = sleipnir_definition();
        assert_eq!(def.bonuses.len(), 3);
    }

    #[test]
    fn test_sleipnir_has_swiftfoot_bonus() {
        let def = sleipnir_definition();
        let has_bonus = def.bonuses.iter().any(|b| {
            matches!(b, GodItemBonus::Swiftfoot { dungeon_speed_percent } if (*dungeon_speed_percent - 50.0).abs() < f64::EPSILON)
        });
        assert!(has_bonus, "Expected Swiftfoot bonus with 50%");
    }

    #[test]
    fn test_sleipnir_has_nimble_hands_bonus() {
        let def = sleipnir_definition();
        let has_bonus = def.bonuses.iter().any(|b| {
            matches!(b, GodItemBonus::NimbleHands { fishing_reduction_percent } if (*fishing_reduction_percent - 50.0).abs() < f64::EPSILON)
        });
        assert!(has_bonus, "Expected NimbleHands bonus with 50%");
    }

    // Megingjord tests

    #[test]
    fn test_megingjord_definition_is_mythic_ring() {
        let def = megingjord_definition();
        assert_eq!(def.id, GodItemId::Megingjord);
        assert_eq!(def.slot, EquipmentSlot::Ring);
        assert_eq!(def.name, "Megingjord");
        let item = def.to_item();
        assert_eq!(item.rarity, Rarity::Mythic);
    }

    #[test]
    fn test_megingjord_has_giants_might_passive() {
        let def = megingjord_definition();
        match def.passive {
            GodItemPassive::GiantsMight { damage_percent } => {
                assert!((damage_percent - 150.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected GiantsMight passive"),
        }
    }

    #[test]
    fn test_megingjord_has_no_bonuses() {
        let def = megingjord_definition();
        assert!(def.bonuses.is_empty());
    }

    #[test]
    fn test_megingjord_has_str_primary_con_supporting() {
        let def = megingjord_definition();
        assert_eq!(def.attributes.str, 40);
        assert_eq!(def.attributes.con, 20);
        assert_eq!(def.attributes.dex, 0);
        assert_eq!(def.attributes.wis, 0);
    }

    // Helper function tests

    #[test]
    fn test_equipped_god_item_dr_with_asprika() {
        let mut equipment = crate::items::Equipment::new();
        let asprika = asprika_definition().to_item();
        equipment.set(crate::items::EquipmentSlot::Armor, Some(asprika));
        let dr = equipped_god_item_dr(&equipment);
        assert!((dr - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_dr_without_god_item() {
        let equipment = crate::items::Equipment::new();
        assert!((equipped_god_item_dr(&equipment)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_attack_speed_with_sleipnir() {
        let mut equipment = crate::items::Equipment::new();
        let sleipnir = sleipnir_definition().to_item();
        equipment.set(crate::items::EquipmentSlot::Boots, Some(sleipnir));
        assert!((equipped_god_item_attack_speed_percent(&equipment) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_attack_speed_without_god_item() {
        let equipment = crate::items::Equipment::new();
        assert!((equipped_god_item_attack_speed_percent(&equipment)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_damage_with_megingjord() {
        let mut equipment = crate::items::Equipment::new();
        let megingjord = megingjord_definition().to_item();
        equipment.set(crate::items::EquipmentSlot::Ring, Some(megingjord));
        assert!((equipped_god_item_damage_percent(&equipment) - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_damage_without_god_item() {
        let equipment = crate::items::Equipment::new();
        assert!((equipped_god_item_damage_percent(&equipment)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_regen_reduction_with_sleipnir() {
        let mut equipment = crate::items::Equipment::new();
        let sleipnir = sleipnir_definition().to_item();
        equipment.set(crate::items::EquipmentSlot::Boots, Some(sleipnir));
        assert!(
            (equipped_god_item_regen_reduction_percent(&equipment) - 50.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_equipped_god_item_dungeon_speed_with_sleipnir() {
        let mut equipment = crate::items::Equipment::new();
        let sleipnir = sleipnir_definition().to_item();
        equipment.set(crate::items::EquipmentSlot::Boots, Some(sleipnir));
        assert!((equipped_god_item_dungeon_speed_percent(&equipment) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_dungeon_speed_without_god_item() {
        let equipment = crate::items::Equipment::new();
        assert!((equipped_god_item_dungeon_speed_percent(&equipment)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_fishing_reduction_with_sleipnir() {
        let mut equipment = crate::items::Equipment::new();
        let sleipnir = sleipnir_definition().to_item();
        equipment.set(crate::items::EquipmentSlot::Boots, Some(sleipnir));
        assert!(
            (equipped_god_item_fishing_reduction_percent(&equipment) - 50.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_equipped_god_item_fishing_reduction_without_god_item() {
        let equipment = crate::items::Equipment::new();
        assert!((equipped_god_item_fishing_reduction_percent(&equipment)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equipped_god_item_regen_reduction_without_god_item() {
        let equipment = crate::items::Equipment::new();
        assert!((equipped_god_item_regen_reduction_percent(&equipment)).abs() < f64::EPSILON);
    }

    // CachedGodItemBonuses::compute() tests

    #[test]
    fn test_cached_bonuses_default_is_all_zero() {
        let bonuses = CachedGodItemBonuses::default();
        assert_eq!(bonuses.damage_reduction_percent, 0.0);
        assert_eq!(bonuses.attack_speed_percent, 0.0);
        assert_eq!(bonuses.damage_percent, 0.0);
        assert_eq!(bonuses.regen_reduction_percent, 0.0);
        assert_eq!(bonuses.dungeon_speed_percent, 0.0);
        assert_eq!(bonuses.fishing_reduction_percent, 0.0);
    }

    #[test]
    fn test_cached_bonuses_compute_empty_equipment() {
        let equipment = crate::items::Equipment::new();
        let bonuses = CachedGodItemBonuses::compute(&equipment);
        assert_eq!(bonuses.damage_reduction_percent, 0.0);
        assert_eq!(bonuses.attack_speed_percent, 0.0);
        assert_eq!(bonuses.damage_percent, 0.0);
    }

    #[test]
    fn test_cached_bonuses_compute_with_asprika_sets_dr() {
        let mut equipment = crate::items::Equipment::new();
        equipment.set(
            crate::items::EquipmentSlot::Armor,
            Some(asprika_definition().to_item()),
        );
        let bonuses = CachedGodItemBonuses::compute(&equipment);
        assert!((bonuses.damage_reduction_percent - 30.0).abs() < f64::EPSILON);
        // Asprika has no non-combat bonuses.
        assert_eq!(bonuses.regen_reduction_percent, 0.0);
        assert_eq!(bonuses.dungeon_speed_percent, 0.0);
        assert_eq!(bonuses.fishing_reduction_percent, 0.0);
    }

    #[test]
    fn test_cached_bonuses_compute_with_sleipnir_sets_all_bonuses() {
        let mut equipment = crate::items::Equipment::new();
        equipment.set(
            crate::items::EquipmentSlot::Boots,
            Some(sleipnir_definition().to_item()),
        );
        let bonuses = CachedGodItemBonuses::compute(&equipment);
        assert!((bonuses.attack_speed_percent - 100.0).abs() < f64::EPSILON);
        assert!((bonuses.regen_reduction_percent - 50.0).abs() < f64::EPSILON);
        assert!((bonuses.dungeon_speed_percent - 50.0).abs() < f64::EPSILON);
        assert!((bonuses.fishing_reduction_percent - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cached_bonuses_compute_with_megingjord_sets_damage() {
        let mut equipment = crate::items::Equipment::new();
        equipment.set(
            crate::items::EquipmentSlot::Ring,
            Some(megingjord_definition().to_item()),
        );
        let bonuses = CachedGodItemBonuses::compute(&equipment);
        assert!((bonuses.damage_percent - 150.0).abs() < f64::EPSILON);
        assert_eq!(bonuses.damage_reduction_percent, 0.0);
        assert_eq!(bonuses.attack_speed_percent, 0.0);
    }

    #[test]
    fn test_cached_bonuses_compute_with_all_three_god_items_equipped() {
        let mut equipment = crate::items::Equipment::new();
        equipment.set(
            crate::items::EquipmentSlot::Armor,
            Some(asprika_definition().to_item()),
        );
        equipment.set(
            crate::items::EquipmentSlot::Boots,
            Some(sleipnir_definition().to_item()),
        );
        equipment.set(
            crate::items::EquipmentSlot::Ring,
            Some(megingjord_definition().to_item()),
        );

        let bonuses = CachedGodItemBonuses::compute(&equipment);
        assert!((bonuses.damage_reduction_percent - 30.0).abs() < f64::EPSILON);
        assert!((bonuses.attack_speed_percent - 100.0).abs() < f64::EPSILON);
        assert!((bonuses.damage_percent - 150.0).abs() < f64::EPSILON);
        assert!((bonuses.regen_reduction_percent - 50.0).abs() < f64::EPSILON);
        assert!((bonuses.dungeon_speed_percent - 50.0).abs() < f64::EPSILON);
        assert!((bonuses.fishing_reduction_percent - 50.0).abs() < f64::EPSILON);
    }
}
