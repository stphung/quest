//! Haven data structures and room definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Room identifiers in the Haven skill tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HavenRoomId {
    // Root
    Hearthstone,
    // Combat branch
    Armory,
    TrainingYard,
    TrophyHall,
    Watchtower,
    AlchemyLab,
    WarRoom,
    // QoL branch
    Bedroom,
    Garden,
    Library,
    FishingDock,
    Workshop,
    Vault,
}

impl HavenRoomId {
    /// All room IDs in tree order
    pub const ALL: [HavenRoomId; 13] = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::WarRoom,
        HavenRoomId::Bedroom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
        HavenRoomId::Vault,
    ];

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "Hearthstone",
            HavenRoomId::Armory => "Armory",
            HavenRoomId::TrainingYard => "Training Yard",
            HavenRoomId::TrophyHall => "Trophy Hall",
            HavenRoomId::Watchtower => "Watchtower",
            HavenRoomId::AlchemyLab => "Alchemy Lab",
            HavenRoomId::WarRoom => "War Room",
            HavenRoomId::Bedroom => "Bedroom",
            HavenRoomId::Garden => "Garden",
            HavenRoomId::Library => "Library",
            HavenRoomId::FishingDock => "Fishing Dock",
            HavenRoomId::Workshop => "Workshop",
            HavenRoomId::Vault => "Vault",
        }
    }

    /// Flavor description for detail panel
    pub fn description(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "The warm center of your Haven.",
            HavenRoomId::Armory => "Your weapon collection strengthens all who visit.",
            HavenRoomId::TrainingYard => "Practice dummies and sparring targets.",
            HavenRoomId::TrophyHall => "Trophies from past victories attract fortune.",
            HavenRoomId::Watchtower => "Sharpens your eye for weak points.",
            HavenRoomId::AlchemyLab => "Brews and tonics always simmering.",
            HavenRoomId::WarRoom => "Tactical planning speeds your strikes.",
            HavenRoomId::Bedroom => "Rest well, fight well.",
            HavenRoomId::Garden => "Patience cultivated here carries over.",
            HavenRoomId::Library => "Ancient tomes reveal hidden challenges.",
            HavenRoomId::FishingDock => "A private spot to cast.",
            HavenRoomId::Workshop => "Better tools yield better finds.",
            HavenRoomId::Vault => "Preserves treasured equipment through prestige resets.",
        }
    }

    /// Parent room(s) that must be T1+ to unlock this room.
    /// Returns empty slice for Hearthstone (root).
    /// Capstones require both parents.
    pub fn parents(&self) -> &'static [HavenRoomId] {
        match self {
            HavenRoomId::Hearthstone => &[],
            // Combat branch
            HavenRoomId::Armory => &[HavenRoomId::Hearthstone],
            HavenRoomId::TrainingYard => &[HavenRoomId::Armory],
            HavenRoomId::TrophyHall => &[HavenRoomId::Armory],
            HavenRoomId::Watchtower => &[HavenRoomId::TrainingYard],
            HavenRoomId::AlchemyLab => &[HavenRoomId::TrophyHall],
            HavenRoomId::WarRoom => &[HavenRoomId::Watchtower, HavenRoomId::AlchemyLab],
            // QoL branch
            HavenRoomId::Bedroom => &[HavenRoomId::Hearthstone],
            HavenRoomId::Garden => &[HavenRoomId::Bedroom],
            HavenRoomId::Library => &[HavenRoomId::Bedroom],
            HavenRoomId::FishingDock => &[HavenRoomId::Garden],
            HavenRoomId::Workshop => &[HavenRoomId::Library],
            HavenRoomId::Vault => &[HavenRoomId::FishingDock, HavenRoomId::Workshop],
        }
    }

    /// Child rooms that this room unlocks when built to T1+.
    #[allow(dead_code)] // Will be used for UI graph rendering
    pub fn children(&self) -> &'static [HavenRoomId] {
        match self {
            HavenRoomId::Hearthstone => &[HavenRoomId::Armory, HavenRoomId::Bedroom],
            HavenRoomId::Armory => &[HavenRoomId::TrainingYard, HavenRoomId::TrophyHall],
            HavenRoomId::TrainingYard => &[HavenRoomId::Watchtower],
            HavenRoomId::TrophyHall => &[HavenRoomId::AlchemyLab],
            HavenRoomId::Watchtower => &[HavenRoomId::WarRoom],
            HavenRoomId::AlchemyLab => &[HavenRoomId::WarRoom],
            HavenRoomId::WarRoom => &[],
            HavenRoomId::Bedroom => &[HavenRoomId::Garden, HavenRoomId::Library],
            HavenRoomId::Garden => &[HavenRoomId::FishingDock],
            HavenRoomId::Library => &[HavenRoomId::Workshop],
            HavenRoomId::FishingDock => &[HavenRoomId::Vault],
            HavenRoomId::Workshop => &[HavenRoomId::Vault],
            HavenRoomId::Vault => &[],
        }
    }

    /// Whether this room is a capstone (requires two parents)
    #[allow(dead_code)] // Will be used for UI styling
    pub fn is_capstone(&self) -> bool {
        matches!(self, HavenRoomId::WarRoom | HavenRoomId::Vault)
    }

    /// Get the depth of this room in the tree (0 = root, 4 = capstones)
    pub fn depth(&self) -> u8 {
        match self {
            HavenRoomId::Hearthstone => 0,
            HavenRoomId::Armory | HavenRoomId::Bedroom => 1,
            HavenRoomId::TrainingYard
            | HavenRoomId::TrophyHall
            | HavenRoomId::Garden
            | HavenRoomId::Library => 2,
            HavenRoomId::Watchtower
            | HavenRoomId::AlchemyLab
            | HavenRoomId::FishingDock
            | HavenRoomId::Workshop => 3,
            HavenRoomId::WarRoom | HavenRoomId::Vault => 4,
        }
    }
}

/// Get the prestige rank cost for a specific tier (1, 2, or 3) and room.
/// Costs scale with depth: root is cheapest, capstones are most expensive.
pub fn tier_cost(room: HavenRoomId, tier: u8) -> u32 {
    let depth = room.depth();
    match (depth, tier) {
        // Depth 0 (Hearthstone): 1/2/3
        (0, 1) => 1,
        (0, 2) => 2,
        (0, 3) => 3,
        // Depth 1 (Armory, Bedroom): 1/3/5
        (1, 1) => 1,
        (1, 2) => 3,
        (1, 3) => 5,
        // Depth 2-3 (mid-tree): 2/4/6
        (2..=3, 1) => 2,
        (2..=3, 2) => 4,
        (2..=3, 3) => 6,
        // Depth 4 (capstones): 3/5/7
        (4, 1) => 3,
        (4, 2) => 5,
        (4, 3) => 7,
        _ => 0,
    }
}

/// Bonus type that a room provides
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HavenBonusType {
    DamagePercent,
    XpGainPercent,
    DropRatePercent,
    CritChancePercent,
    HpRegenPercent,
    DoubleStrikeChance,
    OfflineXpPercent,
    ChallengeDiscoveryPercent,
    FishingTimerReduction,
    DoubleFishChance,
    ItemRarityPercent,
    HpRegenDelayReduction,
    VaultSlots,
}

/// A specific bonus value for a room at a given tier
#[derive(Debug, Clone, Copy)]
pub struct HavenBonus {
    pub bonus_type: HavenBonusType,
    pub values: [f64; 3], // T1, T2, T3
}

impl HavenRoomId {
    /// Get the bonus definition for this room
    pub fn bonus(&self) -> HavenBonus {
        match self {
            HavenRoomId::Hearthstone => HavenBonus {
                bonus_type: HavenBonusType::OfflineXpPercent,
                values: [25.0, 50.0, 100.0],
            },
            HavenRoomId::Armory => HavenBonus {
                bonus_type: HavenBonusType::DamagePercent,
                values: [5.0, 10.0, 25.0],
            },
            HavenRoomId::TrainingYard => HavenBonus {
                bonus_type: HavenBonusType::XpGainPercent,
                values: [5.0, 10.0, 30.0],
            },
            HavenRoomId::TrophyHall => HavenBonus {
                bonus_type: HavenBonusType::DropRatePercent,
                values: [5.0, 10.0, 15.0],
            },
            HavenRoomId::Watchtower => HavenBonus {
                bonus_type: HavenBonusType::CritChancePercent,
                values: [5.0, 10.0, 20.0],
            },
            HavenRoomId::AlchemyLab => HavenBonus {
                bonus_type: HavenBonusType::HpRegenPercent,
                values: [25.0, 50.0, 100.0],
            },
            HavenRoomId::WarRoom => HavenBonus {
                bonus_type: HavenBonusType::DoubleStrikeChance,
                values: [10.0, 20.0, 35.0],
            },
            HavenRoomId::Bedroom => HavenBonus {
                bonus_type: HavenBonusType::HpRegenDelayReduction,
                values: [15.0, 30.0, 50.0],
            },
            HavenRoomId::Garden => HavenBonus {
                bonus_type: HavenBonusType::FishingTimerReduction,
                values: [10.0, 20.0, 40.0],
            },
            HavenRoomId::Library => HavenBonus {
                bonus_type: HavenBonusType::ChallengeDiscoveryPercent,
                values: [20.0, 30.0, 50.0],
            },
            HavenRoomId::FishingDock => HavenBonus {
                bonus_type: HavenBonusType::DoubleFishChance,
                values: [25.0, 50.0, 100.0],
            },
            HavenRoomId::Workshop => HavenBonus {
                bonus_type: HavenBonusType::ItemRarityPercent,
                values: [10.0, 15.0, 25.0],
            },
            HavenRoomId::Vault => HavenBonus {
                bonus_type: HavenBonusType::VaultSlots,
                values: [1.0, 3.0, 5.0],
            },
        }
    }

    /// Get the bonus value for a specific tier (0 = unbuilt)
    pub fn bonus_value(&self, tier: u8) -> f64 {
        if tier == 0 || tier > 3 {
            return 0.0;
        }
        self.bonus().values[(tier - 1) as usize]
    }

    /// Format the bonus for display (e.g., "+5% DMG", "-10% Attack Interval")
    pub fn format_bonus(&self, tier: u8) -> String {
        if tier == 0 {
            return String::new();
        }
        let value = self.bonus_value(tier);
        match self.bonus().bonus_type {
            HavenBonusType::DamagePercent => format!("+{:.0}% DMG", value),
            HavenBonusType::XpGainPercent => format!("+{:.0}% XP", value),
            HavenBonusType::DropRatePercent => format!("+{:.0}% Drops", value),
            HavenBonusType::CritChancePercent => format!("+{:.0}% Crit", value),
            HavenBonusType::HpRegenPercent => format!("+{:.0}% HP Regen", value),
            HavenBonusType::DoubleStrikeChance => format!("+{:.0}% Double Strike", value),
            HavenBonusType::OfflineXpPercent => format!("+{:.0}% Offline XP", value),
            HavenBonusType::ChallengeDiscoveryPercent => format!("+{:.0}% Discovery", value),
            HavenBonusType::FishingTimerReduction => format!("-{:.0}% Fishing Timers", value),
            HavenBonusType::DoubleFishChance => format!("+{:.0}% Double Fish", value),
            HavenBonusType::ItemRarityPercent => format!("+{:.0}% Item Rarity", value),
            HavenBonusType::HpRegenDelayReduction => format!("-{:.0}% Regen Delay", value),
            HavenBonusType::VaultSlots => format!(
                "{:.0} item{} preserved",
                value,
                if value > 1.0 { "s" } else { "" }
            ),
        }
    }
}

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
        tier < 3 && self.is_room_unlocked(room)
    }

    /// Get the next tier for a room (current + 1), or None if maxed
    pub fn next_tier(&self, room: HavenRoomId) -> Option<u8> {
        let tier = self.room_tier(room);
        if tier < 3 {
            Some(tier + 1)
        } else {
            None
        }
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

    /// Get the bonus value for a specific bonus type
    pub fn get_bonus(&self, bonus_type: HavenBonusType) -> f64 {
        HavenRoomId::ALL
            .iter()
            .filter(|r| r.bonus().bonus_type == bonus_type)
            .map(|r| r.bonus_value(self.room_tier(*r)))
            .sum()
    }
}

/// Discovery chance per tick. Scales with prestige rank.
/// Base: 0.000014 (~2hr at P10), +0.000007 per rank above 10.
pub fn haven_discovery_chance(prestige_rank: u32) -> f64 {
    if prestige_rank < 10 {
        return 0.0;
    }
    0.000014 + (prestige_rank - 10) as f64 * 0.000007
}

/// Pre-computed Haven bonuses for efficient access during gameplay
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Will be used for bonus application in follow-up PR
pub struct HavenBonuses {
    pub damage_percent: f64,
    pub xp_gain_percent: f64,
    pub drop_rate_percent: f64,
    pub crit_chance_percent: f64,
    pub hp_regen_percent: f64,
    pub double_strike_chance: f64,
    pub offline_xp_percent: f64,
    pub challenge_discovery_percent: f64,
    pub fishing_timer_reduction: f64,
    pub double_fish_chance: f64,
    pub item_rarity_percent: f64,
    pub hp_regen_delay_reduction: f64,
    pub vault_slots: u8,
}

impl Haven {
    /// Compute all bonuses from the current Haven state
    #[allow(dead_code)] // Will be used for bonus application in follow-up PR
    pub fn compute_bonuses(&self) -> HavenBonuses {
        HavenBonuses {
            damage_percent: self.get_bonus(HavenBonusType::DamagePercent),
            xp_gain_percent: self.get_bonus(HavenBonusType::XpGainPercent),
            drop_rate_percent: self.get_bonus(HavenBonusType::DropRatePercent),
            crit_chance_percent: self.get_bonus(HavenBonusType::CritChancePercent),
            hp_regen_percent: self.get_bonus(HavenBonusType::HpRegenPercent),
            double_strike_chance: self.get_bonus(HavenBonusType::DoubleStrikeChance),
            offline_xp_percent: self.get_bonus(HavenBonusType::OfflineXpPercent),
            challenge_discovery_percent: self.get_bonus(HavenBonusType::ChallengeDiscoveryPercent),
            fishing_timer_reduction: self.get_bonus(HavenBonusType::FishingTimerReduction),
            double_fish_chance: self.get_bonus(HavenBonusType::DoubleFishChance),
            item_rarity_percent: self.get_bonus(HavenBonusType::ItemRarityPercent),
            hp_regen_delay_reduction: self.get_bonus(HavenBonusType::HpRegenDelayReduction),
            vault_slots: self.vault_tier(),
        }
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
        assert_eq!(haven.total_rooms(), 13);
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
}
