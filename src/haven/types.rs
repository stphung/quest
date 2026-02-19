//! Haven data structures — account-level state and room management.

pub use super::bonus::*;
pub use super::room_defs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Account-level Haven state, saved to ~/.quest/haven.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Haven {
    pub discovered: bool,
    pub rooms: HashMap<HavenRoomId, u8>,
}

impl Default for Haven {
    fn default() -> Self {
        let mut rooms = HashMap::new();
        for room in HavenRoomId::ALL {
            rooms.insert(room, 0);
        }
        Haven {
            discovered: false,
            rooms,
        }
    }
}

impl Haven {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the tier of a room (0 = unbuilt, 1-3)
    pub fn room_tier(&self, room: HavenRoomId) -> u8 {
        *self.rooms.get(&room).unwrap_or(&0)
    }

    /// Check if a room is unlocked (all parents at T1+)
    pub fn is_room_unlocked(&self, room: HavenRoomId) -> bool {
        room.parents().iter().all(|p| self.room_tier(*p) >= 1)
    }

    /// Check if a room can be built or upgraded
    pub fn can_build(&self, room: HavenRoomId) -> bool {
        let tier = self.room_tier(room);
        tier < room.max_tier() && self.is_room_unlocked(room)
    }

    /// Get the next tier for a room (current + 1), or None if maxed
    pub fn next_tier(&self, room: HavenRoomId) -> Option<u8> {
        let tier = self.room_tier(room);
        if tier < room.max_tier() {
            Some(tier + 1)
        } else {
            None
        }
    }

    /// Get the fishing rank bonus from FishingDock T4 (0 if not at T4)
    pub fn fishing_rank_bonus(&self) -> u32 {
        if self.room_tier(HavenRoomId::FishingDock) >= 4 {
            10
        } else {
            0
        }
    }

    /// Check if StormForge is built
    pub fn has_storm_forge(&self) -> bool {
        self.room_tier(HavenRoomId::StormForge) >= 1
    }

    /// Build or upgrade a room. Returns the new tier, or None if not possible.
    pub fn build_room(&mut self, room: HavenRoomId) -> Option<u8> {
        if !self.can_build(room) {
            return None;
        }
        let new_tier = self.room_tier(room) + 1;
        self.rooms.insert(room, new_tier);
        Some(new_tier)
    }

    /// Count of rooms built (tier >= 1)
    pub fn rooms_built(&self) -> usize {
        self.rooms.values().filter(|&&t| t >= 1).count()
    }

    /// Total rooms in the tree
    pub fn total_rooms(&self) -> usize {
        HavenRoomId::ALL.len()
    }

    /// Get the vault tier (0 if not built)
    pub fn vault_tier(&self) -> u8 {
        self.room_tier(HavenRoomId::Vault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_haven_all_rooms_unbuilt() {
        let haven = Haven::new();
        assert!(!haven.discovered);
        assert_eq!(haven.rooms_built(), 0);
        assert_eq!(haven.total_rooms(), 14); // 13 original + StormForge
        for room in HavenRoomId::ALL {
            assert_eq!(haven.room_tier(room), 0);
        }
    }

    #[test]
    fn test_hearthstone_is_root() {
        assert!(HavenRoomId::Hearthstone.parents().is_empty());
        assert_eq!(
            HavenRoomId::Hearthstone.children(),
            &[HavenRoomId::Armory, HavenRoomId::Bedroom]
        );
    }

    #[test]
    fn test_capstone_requires_two_parents() {
        assert!(HavenRoomId::WarRoom.is_capstone());
        assert_eq!(
            HavenRoomId::WarRoom.parents(),
            &[HavenRoomId::Watchtower, HavenRoomId::AlchemyLab]
        );
        assert!(HavenRoomId::Vault.is_capstone());
        assert_eq!(
            HavenRoomId::Vault.parents(),
            &[HavenRoomId::FishingDock, HavenRoomId::Workshop]
        );
    }

    #[test]
    fn test_hearthstone_unlocked_by_default() {
        let haven = Haven::new();
        assert!(haven.is_room_unlocked(HavenRoomId::Hearthstone));
        assert!(!haven.is_room_unlocked(HavenRoomId::Armory));
        assert!(!haven.is_room_unlocked(HavenRoomId::Bedroom));
    }

    #[test]
    fn test_building_hearthstone_unlocks_children() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);
        assert!(haven.is_room_unlocked(HavenRoomId::Armory));
        assert!(haven.is_room_unlocked(HavenRoomId::Bedroom));
        assert!(!haven.is_room_unlocked(HavenRoomId::TrainingYard));
    }

    #[test]
    fn test_cannot_build_locked_room() {
        let mut haven = Haven::new();
        assert!(!haven.can_build(HavenRoomId::Armory));
        assert!(haven.build_room(HavenRoomId::Armory).is_none());
    }

    #[test]
    fn test_cannot_build_past_tier_3() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone); // T1
        haven.build_room(HavenRoomId::Hearthstone); // T2
        haven.build_room(HavenRoomId::Hearthstone); // T3
        assert!(!haven.can_build(HavenRoomId::Hearthstone));
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_none());
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 3);
    }

    #[test]
    fn test_capstone_requires_both_parents() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        // Only one parent built — should NOT unlock War Room
        assert!(!haven.is_room_unlocked(HavenRoomId::WarRoom));
        // Build second parent
        haven.build_room(HavenRoomId::AlchemyLab);
        assert!(haven.is_room_unlocked(HavenRoomId::WarRoom));
    }

    #[test]
    fn test_tier_costs() {
        // Depth 0 (Hearthstone): 1/2/3
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 1), 1);
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 2), 2);
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 3), 3);
        // Depth 1 (Armory): 1/3/5
        assert_eq!(tier_cost(HavenRoomId::Armory, 1), 1);
        assert_eq!(tier_cost(HavenRoomId::Armory, 2), 3);
        assert_eq!(tier_cost(HavenRoomId::Armory, 3), 5);
        // Depth 2-3 (mid-tree): 2/4/6
        assert_eq!(tier_cost(HavenRoomId::TrainingYard, 1), 2);
        assert_eq!(tier_cost(HavenRoomId::Watchtower, 3), 6);
        // Depth 4 (capstones): 3/5/7
        assert_eq!(tier_cost(HavenRoomId::WarRoom, 1), 3);
        assert_eq!(tier_cost(HavenRoomId::Vault, 3), 7);
        // Invalid tier
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 0), 0);
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 4), 0);
    }

    #[test]
    fn test_bonus_values() {
        assert_eq!(HavenRoomId::Armory.bonus_value(0), 0.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(3), 25.0);
    }

    #[test]
    fn test_format_bonus() {
        assert_eq!(HavenRoomId::Armory.format_bonus(1), "+5% DMG");
        assert_eq!(HavenRoomId::WarRoom.format_bonus(3), "+35% Double Strike");
        assert_eq!(HavenRoomId::Vault.format_bonus(1), "1 item preserved");
        assert_eq!(HavenRoomId::Vault.format_bonus(3), "5 items preserved");
    }

    #[test]
    fn test_rooms_built_count() {
        let mut haven = Haven::new();
        assert_eq!(haven.rooms_built(), 0);
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.rooms_built(), 1);
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.rooms_built(), 2);
        // Upgrading doesn't change count
        haven.build_room(HavenRoomId::Hearthstone); // T2
        assert_eq!(haven.rooms_built(), 2);
    }

    #[test]
    fn test_discovery_chance_below_p10() {
        assert_eq!(haven_discovery_chance(0), 0.0);
        assert_eq!(haven_discovery_chance(9), 0.0);
    }

    #[test]
    fn test_discovery_chance_scales_with_prestige() {
        let p10 = haven_discovery_chance(10);
        let p12 = haven_discovery_chance(12);
        let p20 = haven_discovery_chance(20);
        assert!(p10 > 0.0);
        assert!(p12 > p10);
        assert!(p20 > p12);
        assert!((p10 - 0.000014).abs() < 0.0000001);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut haven = Haven::new();
        haven.discovered = true;
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone); // T2
        haven.build_room(HavenRoomId::Armory);

        let json = serde_json::to_string(&haven).unwrap();
        let loaded: Haven = serde_json::from_str(&json).unwrap();

        assert!(loaded.discovered);
        assert_eq!(loaded.room_tier(HavenRoomId::Hearthstone), 2);
        assert_eq!(loaded.room_tier(HavenRoomId::Armory), 1);
        assert_eq!(loaded.room_tier(HavenRoomId::Bedroom), 0);
    }

    #[test]
    fn test_get_bonus_from_haven() {
        let mut haven = Haven::new();
        assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 0.0);
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 5.0);
    }

    #[test]
    fn test_full_combat_branch_buildable() {
        let mut haven = Haven::new();
        // Build full combat branch
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_some());
        assert!(haven.build_room(HavenRoomId::Armory).is_some());
        assert!(haven.build_room(HavenRoomId::TrainingYard).is_some());
        assert!(haven.build_room(HavenRoomId::TrophyHall).is_some());
        assert!(haven.build_room(HavenRoomId::Watchtower).is_some());
        assert!(haven.build_room(HavenRoomId::AlchemyLab).is_some());
        assert!(haven.build_room(HavenRoomId::WarRoom).is_some());
        assert_eq!(haven.rooms_built(), 7);
    }

    #[test]
    fn test_compute_bonuses() {
        let mut haven = Haven::new();
        let bonuses = haven.compute_bonuses();

        // Empty haven has no bonuses
        assert_eq!(bonuses.damage_percent, 0.0);
        assert_eq!(bonuses.xp_gain_percent, 0.0);
        assert_eq!(bonuses.vault_slots, 0);

        // Build some rooms
        haven.build_room(HavenRoomId::Hearthstone); // +25% Offline XP
        haven.build_room(HavenRoomId::Armory); // +5% DMG

        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.damage_percent, 5.0);
        assert_eq!(bonuses.offline_xp_percent, 25.0);

        // Upgrade Armory to T2
        haven.build_room(HavenRoomId::Armory); // +10% DMG now
        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.damage_percent, 10.0);
    }

    // =========================================================================
    // Comprehensive Bonus Value Tests (all 13 rooms x 3 tiers)
    // =========================================================================

    #[test]
    fn test_all_room_bonus_values_tier_1() {
        assert_eq!(HavenRoomId::Hearthstone.bonus_value(1), 25.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::Bedroom.bonus_value(1), 15.0);
        assert_eq!(HavenRoomId::TrainingYard.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::Garden.bonus_value(1), 10.0);
        assert_eq!(HavenRoomId::TrophyHall.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::Library.bonus_value(1), 20.0);
        assert_eq!(HavenRoomId::Watchtower.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::FishingDock.bonus_value(1), 25.0);
        assert_eq!(HavenRoomId::AlchemyLab.bonus_value(1), 25.0);
        assert_eq!(HavenRoomId::Workshop.bonus_value(1), 10.0);
        assert_eq!(HavenRoomId::WarRoom.bonus_value(1), 10.0);
        assert_eq!(HavenRoomId::Vault.bonus_value(1), 1.0);
    }

    #[test]
    fn test_all_room_bonus_values_tier_2() {
        assert_eq!(HavenRoomId::Hearthstone.bonus_value(2), 50.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::Bedroom.bonus_value(2), 30.0);
        assert_eq!(HavenRoomId::TrainingYard.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::Garden.bonus_value(2), 20.0);
        assert_eq!(HavenRoomId::TrophyHall.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::Library.bonus_value(2), 30.0);
        assert_eq!(HavenRoomId::Watchtower.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::FishingDock.bonus_value(2), 50.0);
        assert_eq!(HavenRoomId::AlchemyLab.bonus_value(2), 50.0);
        assert_eq!(HavenRoomId::Workshop.bonus_value(2), 15.0);
        assert_eq!(HavenRoomId::WarRoom.bonus_value(2), 20.0);
        assert_eq!(HavenRoomId::Vault.bonus_value(2), 3.0);
    }

    #[test]
    fn test_all_room_bonus_values_tier_3() {
        assert_eq!(HavenRoomId::Hearthstone.bonus_value(3), 100.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(3), 25.0);
        assert_eq!(HavenRoomId::Bedroom.bonus_value(3), 50.0);
        assert_eq!(HavenRoomId::TrainingYard.bonus_value(3), 30.0);
        assert_eq!(HavenRoomId::Garden.bonus_value(3), 40.0);
        assert_eq!(HavenRoomId::TrophyHall.bonus_value(3), 15.0);
        assert_eq!(HavenRoomId::Library.bonus_value(3), 50.0);
        assert_eq!(HavenRoomId::Watchtower.bonus_value(3), 20.0);
        assert_eq!(HavenRoomId::FishingDock.bonus_value(3), 100.0);
        assert_eq!(HavenRoomId::AlchemyLab.bonus_value(3), 100.0);
        assert_eq!(HavenRoomId::Workshop.bonus_value(3), 25.0);
        assert_eq!(HavenRoomId::WarRoom.bonus_value(3), 35.0);
        assert_eq!(HavenRoomId::Vault.bonus_value(3), 5.0);
    }

    #[test]
    fn test_bonus_value_returns_zero_for_unbuilt_and_invalid() {
        for room in HavenRoomId::ALL {
            assert_eq!(room.bonus_value(0), 0.0, "{:?} tier 0 should be 0", room);
            let above_max = room.max_tier() + 1;
            assert_eq!(
                room.bonus_value(above_max),
                0.0,
                "{:?} tier {} (above max) should be 0",
                room,
                above_max
            );
            assert_eq!(
                room.bonus_value(255),
                0.0,
                "{:?} tier 255 should be 0",
                room
            );
        }
    }

    // =========================================================================
    // Full Tree Unlock Path Tests
    // =========================================================================

    #[test]
    fn test_full_fishing_branch_buildable() {
        let mut haven = Haven::new();
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_some());
        assert!(haven.build_room(HavenRoomId::Bedroom).is_some());
        assert!(haven.build_room(HavenRoomId::Garden).is_some());
        assert!(haven.build_room(HavenRoomId::Library).is_some());
        assert!(haven.build_room(HavenRoomId::FishingDock).is_some());
        assert!(haven.build_room(HavenRoomId::Workshop).is_some());
        assert!(haven.build_room(HavenRoomId::Vault).is_some());
        assert_eq!(haven.rooms_built(), 7);
    }

    #[test]
    fn test_complete_haven_all_rooms_buildable() {
        let mut haven = Haven::new();
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_some());
        assert!(haven.build_room(HavenRoomId::Armory).is_some());
        assert!(haven.build_room(HavenRoomId::Bedroom).is_some());
        assert!(haven.build_room(HavenRoomId::TrainingYard).is_some());
        assert!(haven.build_room(HavenRoomId::Garden).is_some());
        assert!(haven.build_room(HavenRoomId::TrophyHall).is_some());
        assert!(haven.build_room(HavenRoomId::Library).is_some());
        assert!(haven.build_room(HavenRoomId::Watchtower).is_some());
        assert!(haven.build_room(HavenRoomId::FishingDock).is_some());
        assert!(haven.build_room(HavenRoomId::AlchemyLab).is_some());
        assert!(haven.build_room(HavenRoomId::Workshop).is_some());
        assert!(haven.build_room(HavenRoomId::WarRoom).is_some());
        assert!(haven.build_room(HavenRoomId::Vault).is_some());
        assert!(haven.build_room(HavenRoomId::StormForge).is_some());
        assert_eq!(haven.rooms_built(), 14);
        assert_eq!(haven.total_rooms(), 14);
    }

    #[test]
    fn test_max_all_rooms_to_max_tier() {
        let mut haven = Haven::new();
        for _ in 0..3 {
            haven.build_room(HavenRoomId::Hearthstone);
        }
        for _ in 0..3 {
            haven.build_room(HavenRoomId::Armory);
            haven.build_room(HavenRoomId::Bedroom);
        }
        for _ in 0..3 {
            haven.build_room(HavenRoomId::TrainingYard);
            haven.build_room(HavenRoomId::Garden);
        }
        for _ in 0..3 {
            haven.build_room(HavenRoomId::TrophyHall);
            haven.build_room(HavenRoomId::Library);
            haven.build_room(HavenRoomId::Watchtower);
            haven.build_room(HavenRoomId::AlchemyLab);
            haven.build_room(HavenRoomId::Workshop);
        }
        for _ in 0..4 {
            haven.build_room(HavenRoomId::FishingDock);
        }
        for _ in 0..3 {
            haven.build_room(HavenRoomId::WarRoom);
            haven.build_room(HavenRoomId::Vault);
        }
        haven.build_room(HavenRoomId::StormForge);

        for room in HavenRoomId::ALL {
            assert_eq!(
                haven.room_tier(room),
                room.max_tier(),
                "{:?} should be at max tier {}",
                room,
                room.max_tier()
            );
        }
    }

    // =========================================================================
    // Prestige Token Economy Tests
    // =========================================================================

    #[test]
    fn test_total_tokens_to_max_single_room() {
        let total = tier_cost(HavenRoomId::Hearthstone, 1)
            + tier_cost(HavenRoomId::Hearthstone, 2)
            + tier_cost(HavenRoomId::Hearthstone, 3);
        assert_eq!(total, 6);

        let total = tier_cost(HavenRoomId::Armory, 1)
            + tier_cost(HavenRoomId::Armory, 2)
            + tier_cost(HavenRoomId::Armory, 3);
        assert_eq!(total, 9);

        let total = tier_cost(HavenRoomId::WarRoom, 1)
            + tier_cost(HavenRoomId::WarRoom, 2)
            + tier_cost(HavenRoomId::WarRoom, 3);
        assert_eq!(total, 15);
    }

    #[test]
    fn test_total_tokens_to_max_entire_haven() {
        let mut total = 0u32;
        for room in HavenRoomId::ALL {
            for tier in 1..=3 {
                total += tier_cost(room, tier);
            }
        }
        assert!(total > 100, "Total tokens to max Haven: {}", total);
    }

    #[test]
    fn test_partial_token_spending_preserves_progress() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.room_tier(HavenRoomId::Armory), 1);
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 2);
        assert_eq!(haven.room_tier(HavenRoomId::Armory), 1);
    }

    // =========================================================================
    // Edge Cases and Error Handling
    // =========================================================================

    #[test]
    fn test_cannot_build_child_before_parent() {
        let haven = Haven::new();
        assert!(!haven.is_room_unlocked(HavenRoomId::TrainingYard));
        assert!(!haven.can_build(HavenRoomId::TrainingYard));
    }

    #[test]
    fn test_capstone_not_unlocked_with_only_one_parent() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        assert!(!haven.is_room_unlocked(HavenRoomId::WarRoom));
        assert!(!haven.can_build(HavenRoomId::WarRoom));
    }

    #[test]
    fn test_building_returns_new_tier() {
        let mut haven = Haven::new();
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), Some(1));
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), Some(2));
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), Some(3));
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), None);
    }

    #[test]
    fn test_tree_structure_integrity() {
        for room in HavenRoomId::ALL {
            if room != HavenRoomId::Hearthstone {
                assert!(!room.parents().is_empty(), "{:?} should have parents", room);
            }
        }
        assert_eq!(HavenRoomId::WarRoom.parents().len(), 2);
        assert_eq!(HavenRoomId::Vault.parents().len(), 2);
        assert_eq!(HavenRoomId::StormForge.parents().len(), 2);
        assert!(HavenRoomId::StormForge.children().is_empty());
    }

    #[test]
    fn test_all_bonus_types_mapped_to_rooms() {
        let bonus_types = [
            HavenBonusType::DamagePercent,
            HavenBonusType::XpGainPercent,
            HavenBonusType::DropRatePercent,
            HavenBonusType::CritChancePercent,
            HavenBonusType::HpRegenPercent,
            HavenBonusType::DoubleStrikeChance,
            HavenBonusType::OfflineXpPercent,
            HavenBonusType::ChallengeDiscoveryPercent,
            HavenBonusType::FishingTimerReduction,
            HavenBonusType::DoubleFishChance,
            HavenBonusType::ItemRarityPercent,
            HavenBonusType::HpRegenDelayReduction,
            HavenBonusType::VaultSlots,
        ];

        for bonus_type in bonus_types {
            let providing_rooms: Vec<_> = HavenRoomId::ALL
                .iter()
                .filter(|r| r.bonus().bonus_type == bonus_type)
                .collect();
            assert_eq!(
                providing_rooms.len(),
                1,
                "{:?} should be provided by exactly one room, found {:?}",
                bonus_type,
                providing_rooms
            );
        }
    }

    #[test]
    fn test_compute_bonuses_all_fields() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::Watchtower);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::AlchemyLab);
        haven.build_room(HavenRoomId::Workshop);
        haven.build_room(HavenRoomId::WarRoom);
        haven.build_room(HavenRoomId::Vault);

        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.offline_xp_percent, 25.0);
        assert_eq!(bonuses.damage_percent, 5.0);
        assert_eq!(bonuses.hp_regen_delay_reduction, 15.0);
        assert_eq!(bonuses.xp_gain_percent, 5.0);
        assert_eq!(bonuses.fishing_timer_reduction, 10.0);
        assert_eq!(bonuses.drop_rate_percent, 5.0);
        assert_eq!(bonuses.challenge_discovery_percent, 20.0);
        assert_eq!(bonuses.crit_chance_percent, 5.0);
        assert_eq!(bonuses.double_fish_chance, 25.0);
        assert_eq!(bonuses.hp_regen_percent, 25.0);
        assert_eq!(bonuses.item_rarity_percent, 10.0);
        assert_eq!(bonuses.double_strike_chance, 10.0);
        assert_eq!(bonuses.vault_slots, 1);
    }

    #[test]
    fn test_vault_tier_convenience_method() {
        let mut haven = Haven::new();
        assert_eq!(haven.vault_tier(), 0);
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::Workshop);
        haven.build_room(HavenRoomId::Vault);
        assert_eq!(haven.vault_tier(), 1);
        haven.build_room(HavenRoomId::Vault);
        assert_eq!(haven.vault_tier(), 2);
        haven.build_room(HavenRoomId::Vault);
        assert_eq!(haven.vault_tier(), 3);
    }

    // =========================================================================
    // Vault Slot Count Bug Regression Tests
    // =========================================================================

    fn haven_with_vault_at_tier(tier: u8) -> Haven {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::Workshop);
        for _ in 0..tier {
            haven.build_room(HavenRoomId::Vault);
        }
        assert_eq!(haven.vault_tier(), tier);
        haven
    }

    #[test]
    fn test_vault_bonus_value_matches_slot_count_t1() {
        assert_eq!(HavenRoomId::Vault.bonus_value(1), 1.0);
    }

    #[test]
    fn test_vault_bonus_value_matches_slot_count_t2() {
        assert_eq!(HavenRoomId::Vault.bonus_value(2), 3.0);
    }

    #[test]
    fn test_vault_bonus_value_matches_slot_count_t3() {
        assert_eq!(HavenRoomId::Vault.bonus_value(3), 5.0);
    }

    #[test]
    fn test_compute_bonuses_vault_slots_t1() {
        let haven = haven_with_vault_at_tier(1);
        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.vault_slots, 1);
    }

    #[test]
    fn test_compute_bonuses_vault_slots_t2() {
        let haven = haven_with_vault_at_tier(2);
        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.vault_slots, 3);
    }

    #[test]
    fn test_compute_bonuses_vault_slots_t3() {
        let haven = haven_with_vault_at_tier(3);
        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.vault_slots, 5);
    }

    #[test]
    fn test_compute_bonuses_vault_slots_unbuilt() {
        let haven = Haven::new();
        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.vault_slots, 0);
    }

    #[test]
    fn test_get_bonus_returns_vault_slot_count_not_tier() {
        let haven = haven_with_vault_at_tier(2);
        let vault_bonus = haven.get_bonus(HavenBonusType::VaultSlots);
        assert_eq!(vault_bonus, 3.0);

        let haven = haven_with_vault_at_tier(3);
        let vault_bonus = haven.get_bonus(HavenBonusType::VaultSlots);
        assert_eq!(vault_bonus, 5.0);
    }
}
