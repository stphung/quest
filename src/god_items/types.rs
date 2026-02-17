use serde::{Deserialize, Serialize};

use crate::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

/// Prestige rank cost to forge a god item at the Soulforge.
pub const GOD_ITEM_FORGE_COST: u32 = 50;

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

/// State machine for a god item quest.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GodItemState {
    #[default]
    Undiscovered,
    Discovered,
    ReadyToForge,
    Forged,
}

/// Progress tracking for Asprika's milestone.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AsprikaMilestones {
    /// Whether the Beyond Infinity I achievement has been completed.
    #[serde(default)]
    pub expanse_cycle_complete: bool,
    // Legacy fields for save compat.
    #[serde(default)]
    pub master_challenge_types_won: Vec<String>,
    #[serde(default)]
    pub slots_at_plus_7: u8,
    #[serde(default)]
    pub temple_trial_return_complete: bool,
}

impl AsprikaMilestones {
    pub fn all_met(&self) -> bool {
        self.expanse_cycle_complete
    }
}

/// Progress tracking for Sleipnir's milestones.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SleipnirMilestones {
    /// Distinct Master challenge types won (need >= 3).
    pub master_challenge_types_won: Vec<String>,
    /// Highest enhancement level reached across all slots (need >= 7).
    pub highest_enhancement_level: u8,
}

impl SleipnirMilestones {
    pub fn master_challenges_met(&self) -> bool {
        self.master_challenge_types_won.len() >= 3
    }

    pub fn enhancement_met(&self) -> bool {
        self.highest_enhancement_level >= 7
    }

    pub fn all_met(&self) -> bool {
        self.master_challenges_met() && self.enhancement_met()
    }
}

/// Progress tracking for Megingjord's milestone.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MegingjordMilestones {
    /// Highest enhancement level reached across all slots (need >= 9).
    pub highest_enhancement_level: u8,
}

impl MegingjordMilestones {
    pub fn all_met(&self) -> bool {
        self.highest_enhancement_level >= 9
    }
}

/// Account-wide god item progress.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GodItemProgress {
    pub asprika_state: GodItemState,
    pub asprika_milestones: AsprikaMilestones,
    #[serde(default)]
    pub sleipnir_state: GodItemState,
    #[serde(default)]
    pub sleipnir_milestones: SleipnirMilestones,
    #[serde(default)]
    pub megingjord_state: GodItemState,
    #[serde(default)]
    pub megingjord_milestones: MegingjordMilestones,
}

impl GodItemProgress {
    /// Sync enhancement milestones for Sleipnir and Megingjord.
    pub fn sync_enhancement_milestones(&mut self, enhancement_levels: &[u8; 7]) {
        let highest = enhancement_levels.iter().copied().max().unwrap_or(0);
        self.sleipnir_milestones.highest_enhancement_level = highest;
        self.megingjord_milestones.highest_enhancement_level = highest;
    }

    /// Record a Master difficulty challenge win for Sleipnir's milestone.
    /// Returns true if this was a new type (milestone progressed).
    pub fn record_master_challenge_win(&mut self, challenge_type: &str) -> bool {
        if self.sleipnir_state == GodItemState::Undiscovered {
            return false;
        }
        let type_str = challenge_type.to_string();
        if !self
            .sleipnir_milestones
            .master_challenge_types_won
            .contains(&type_str)
        {
            self.sleipnir_milestones
                .master_challenge_types_won
                .push(type_str);
            true
        } else {
            false
        }
    }

    /// Sync Asprika milestone from achievements (Beyond Infinity I).
    pub fn sync_asprika_milestone(&mut self, has_expanse_cycle_i: bool) {
        self.asprika_milestones.expanse_cycle_complete = has_expanse_cycle_i;
    }
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

    // Milestone tests

    #[test]
    fn test_asprika_milestones_default_not_met() {
        let m = AsprikaMilestones::default();
        assert!(!m.all_met());
    }

    #[test]
    fn test_asprika_milestones_met_with_expanse_cycle() {
        let m = AsprikaMilestones {
            expanse_cycle_complete: true,
            ..Default::default()
        };
        assert!(m.all_met());
    }

    #[test]
    fn test_sleipnir_milestones_default_not_met() {
        let m = SleipnirMilestones::default();
        assert!(!m.master_challenges_met());
        assert!(!m.enhancement_met());
        assert!(!m.all_met());
    }

    #[test]
    fn test_sleipnir_milestones_all_met() {
        let m = SleipnirMilestones {
            master_challenge_types_won: vec![
                "Chess".to_string(),
                "Go".to_string(),
                "Snake".to_string(),
            ],
            highest_enhancement_level: 7,
        };
        assert!(m.master_challenges_met());
        assert!(m.enhancement_met());
        assert!(m.all_met());
    }

    #[test]
    fn test_sleipnir_milestones_partial() {
        let m = SleipnirMilestones {
            master_challenge_types_won: vec!["Chess".to_string()],
            highest_enhancement_level: 7,
        };
        assert!(!m.master_challenges_met());
        assert!(m.enhancement_met());
        assert!(!m.all_met());
    }

    #[test]
    fn test_megingjord_milestones_default_not_met() {
        let m = MegingjordMilestones::default();
        assert!(!m.all_met());
    }

    #[test]
    fn test_megingjord_milestones_met() {
        let m = MegingjordMilestones {
            highest_enhancement_level: 9,
        };
        assert!(m.all_met());
    }

    #[test]
    fn test_megingjord_milestones_not_met_at_8() {
        let m = MegingjordMilestones {
            highest_enhancement_level: 8,
        };
        assert!(!m.all_met());
    }

    #[test]
    fn test_god_item_forge_cost() {
        assert_eq!(GOD_ITEM_FORGE_COST, 50);
    }

    #[test]
    fn test_god_item_state_default_undiscovered() {
        let p = GodItemProgress::default();
        assert_eq!(p.asprika_state, GodItemState::Undiscovered);
        assert_eq!(p.sleipnir_state, GodItemState::Undiscovered);
        assert_eq!(p.megingjord_state, GodItemState::Undiscovered);
    }

    #[test]
    fn test_sync_enhancement_milestones() {
        let mut progress = GodItemProgress::default();
        let levels = [0, 7, 3, 9, 7, 0, 0];
        progress.sync_enhancement_milestones(&levels);
        assert_eq!(progress.sleipnir_milestones.highest_enhancement_level, 9);
        assert_eq!(progress.megingjord_milestones.highest_enhancement_level, 9);
    }

    #[test]
    fn test_record_master_challenge_win() {
        let mut progress = GodItemProgress {
            sleipnir_state: GodItemState::Discovered,
            ..Default::default()
        };
        assert!(progress.record_master_challenge_win("Chess"));
        assert!(progress.record_master_challenge_win("Go"));
        assert!(!progress.record_master_challenge_win("Chess")); // duplicate
        assert_eq!(
            progress
                .sleipnir_milestones
                .master_challenge_types_won
                .len(),
            2
        );
    }

    #[test]
    fn test_record_master_challenge_win_ignored_when_undiscovered() {
        let mut progress = GodItemProgress::default();
        assert!(!progress.record_master_challenge_win("Chess"));
        assert!(progress
            .sleipnir_milestones
            .master_challenge_types_won
            .is_empty());
    }

    #[test]
    fn test_sync_asprika_milestone() {
        let mut progress = GodItemProgress::default();
        assert!(!progress.asprika_milestones.all_met());
        progress.sync_asprika_milestone(true);
        assert!(progress.asprika_milestones.all_met());
    }
}
