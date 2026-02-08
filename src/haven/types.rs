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
    // Special buildings
    StormForge,
}

impl HavenRoomId {
    /// All room IDs in tree order
    pub const ALL: [HavenRoomId; 14] = [
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
        HavenRoomId::StormForge,
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
            HavenRoomId::StormForge => "Storm Forge",
        }
    }

    /// Flavor description for detail panel
    pub fn description(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "A crackling fire burns at the heart of your Haven, its embers never quite dying out. Even when you're away, its warmth keeps your skills sharp.",
            HavenRoomId::Armory => "Whetstones and weapon oil fill the air with a sharp, metallic tang. Every blade here has been honed to a razor's edge, and their fury flows into whoever wields them.",
            HavenRoomId::TrainingYard => "The clang of steel on wood echoes through the yard at all hours. Sweat-stained targets and chalk-drawn footwork patterns mark the path to mastery.",
            HavenRoomId::TrophyHall => "Glass cases display the spoils of a hundred battles — a dragon's scale, a bandit lord's signet ring, a shard of cursed obsidian. Their presence draws more treasure your way.",
            HavenRoomId::Watchtower => "A spiral staircase leads to a narrow platform where hawks nest and cold wind bites. Hours spent scanning the horizon have taught you to spot a weakness before your enemy even knows it's there.",
            HavenRoomId::AlchemyLab => "Bubbling flasks and copper coils crowd every surface, filling the room with a warm, herbal haze. The potions brewed here mend wounds faster than any battlefield medic could dream.",
            HavenRoomId::WarRoom => "Faded footwork circles are carved into the stone floor, each one paired with strike marks on the opposing wall — one high, one low, in rapid succession. The room teaches your muscles what your mind already knows: one strike is never enough.",
            HavenRoomId::Bedroom => "Heavy curtains block out every sliver of light, and the bed is piled high with furs. In this perfect darkness, your body recovers with an almost unnatural speed.",
            HavenRoomId::Garden => "Water trickles from a carved stone fountain into a shallow basin where lily pads drift. Tending this garden teaches a stillness that makes even the longest fishing wait feel brief.",
            HavenRoomId::Library => "A reading nook tucked beneath a stained-glass window, surrounded by towers of scrolls and ink-stained notes. The more you read, the more the world reveals its hidden trials to you.",
            HavenRoomId::FishingDock => "Morning mist clings to the water as your line breaks the stillness. The fish here bite in pairs, and those who cast long enough swear they've felt something vast stir in the deep — something most anglers will never be ready for.",
            HavenRoomId::Workshop => "Sawdust and iron filings crunch underfoot as you pass workbenches cluttered with half-finished tools and polishing rigs. Gear crafted here always seems to turn out a cut above the rest.",
            HavenRoomId::Vault => "Preserves treasured equipment through prestige resets.",
            HavenRoomId::StormForge => "The legendary forge where Stormbreaker can be crafted.",
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
            // StormForge requires both capstones
            HavenRoomId::StormForge => &[HavenRoomId::WarRoom, HavenRoomId::Vault],
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
            HavenRoomId::WarRoom => &[HavenRoomId::StormForge],
            HavenRoomId::Bedroom => &[HavenRoomId::Garden, HavenRoomId::Library],
            HavenRoomId::Garden => &[HavenRoomId::FishingDock],
            HavenRoomId::Library => &[HavenRoomId::Workshop],
            HavenRoomId::FishingDock => &[HavenRoomId::Vault],
            HavenRoomId::Workshop => &[HavenRoomId::Vault],
            HavenRoomId::Vault => &[HavenRoomId::StormForge],
            HavenRoomId::StormForge => &[],
        }
    }

    /// Whether this room is a capstone (requires two parents)
    #[allow(dead_code)] // Will be used for UI styling
    pub fn is_capstone(&self) -> bool {
        matches!(
            self,
            HavenRoomId::WarRoom | HavenRoomId::Vault | HavenRoomId::StormForge
        )
    }

    /// Get the depth of this room in the tree (0 = root, 4 = capstones, 5 = StormForge)
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
            HavenRoomId::StormForge => 5,
        }
    }

    /// Maximum tier for this room (most rooms are 3, StormForge and FishingDock have special max)
    pub fn max_tier(&self) -> u8 {
        match self {
            HavenRoomId::StormForge => 1,  // Single tier only
            HavenRoomId::FishingDock => 4, // Has tier 4 for max fishing rank
            _ => 3,
        }
    }
}

/// Get the prestige rank cost for a specific tier and room.
/// Costs scale with depth: root is cheapest, capstones are most expensive.
/// Special rooms have unique costs.
pub fn tier_cost(room: HavenRoomId, tier: u8) -> u32 {
    // Special room costs
    match room {
        HavenRoomId::StormForge => {
            // Single tier, costs 25 PR
            if tier == 1 {
                25
            } else {
                0
            }
        }
        HavenRoomId::FishingDock => {
            // T1-3 follow normal depth 3 costs, T4 is special
            match tier {
                1 => 2,
                2 => 4,
                3 => 6,
                4 => 10, // Special T4 cost
                _ => 0,
            }
        }
        _ => {
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
    }
}

/// Bonus type that a room provides
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // MaxFishingRank will be used in fishing logic
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
    MaxFishingRank,   // FishingDock T4 bonus
    StormForgeAccess, // StormForge enables forging
}

/// A specific bonus value for a room at a given tier
#[derive(Debug, Clone, Copy)]
pub struct HavenBonus {
    pub bonus_type: HavenBonusType,
    pub values: [f64; 4], // T1, T2, T3, T4 (T4 only used by FishingDock)
}

impl HavenRoomId {
    /// Get the bonus definition for this room
    pub fn bonus(&self) -> HavenBonus {
        match self {
            HavenRoomId::Hearthstone => HavenBonus {
                bonus_type: HavenBonusType::OfflineXpPercent,
                values: [25.0, 50.0, 100.0, 0.0],
            },
            HavenRoomId::Armory => HavenBonus {
                bonus_type: HavenBonusType::DamagePercent,
                values: [5.0, 10.0, 25.0, 0.0],
            },
            HavenRoomId::TrainingYard => HavenBonus {
                bonus_type: HavenBonusType::XpGainPercent,
                values: [5.0, 10.0, 30.0, 0.0],
            },
            HavenRoomId::TrophyHall => HavenBonus {
                bonus_type: HavenBonusType::DropRatePercent,
                values: [5.0, 10.0, 15.0, 0.0],
            },
            HavenRoomId::Watchtower => HavenBonus {
                bonus_type: HavenBonusType::CritChancePercent,
                values: [5.0, 10.0, 20.0, 0.0],
            },
            HavenRoomId::AlchemyLab => HavenBonus {
                bonus_type: HavenBonusType::HpRegenPercent,
                values: [25.0, 50.0, 100.0, 0.0],
            },
            HavenRoomId::WarRoom => HavenBonus {
                bonus_type: HavenBonusType::DoubleStrikeChance,
                values: [10.0, 20.0, 35.0, 0.0],
            },
            HavenRoomId::Bedroom => HavenBonus {
                bonus_type: HavenBonusType::HpRegenDelayReduction,
                values: [15.0, 30.0, 50.0, 0.0],
            },
            HavenRoomId::Garden => HavenBonus {
                bonus_type: HavenBonusType::FishingTimerReduction,
                values: [10.0, 20.0, 40.0, 0.0],
            },
            HavenRoomId::Library => HavenBonus {
                bonus_type: HavenBonusType::ChallengeDiscoveryPercent,
                values: [20.0, 30.0, 50.0, 0.0],
            },
            HavenRoomId::FishingDock => HavenBonus {
                bonus_type: HavenBonusType::DoubleFishChance,
                // T1-3: Double fish chance, T4: +10 max fishing rank (handled separately)
                values: [25.0, 50.0, 100.0, 100.0],
            },
            HavenRoomId::Workshop => HavenBonus {
                bonus_type: HavenBonusType::ItemRarityPercent,
                values: [10.0, 15.0, 25.0, 0.0],
            },
            HavenRoomId::Vault => HavenBonus {
                bonus_type: HavenBonusType::VaultSlots,
                values: [1.0, 3.0, 5.0, 0.0],
            },
            HavenRoomId::StormForge => HavenBonus {
                bonus_type: HavenBonusType::StormForgeAccess,
                values: [1.0, 0.0, 0.0, 0.0], // Single tier, value 1.0 = enabled
            },
        }
    }

    /// Get the bonus value for a specific tier (0 = unbuilt)
    pub fn bonus_value(&self, tier: u8) -> f64 {
        let max_tier = self.max_tier();
        if tier == 0 || tier > max_tier {
            return 0.0;
        }
        self.bonus().values[(tier - 1) as usize]
    }

    /// Format the bonus for display (e.g., "+5% DMG", "-10% Attack Interval")
    pub fn format_bonus(&self, tier: u8) -> String {
        if tier == 0 {
            return String::new();
        }
        // Special case: FishingDock T4 has a different bonus type
        if *self == HavenRoomId::FishingDock && tier == 4 {
            return "+10 Max Fishing Rank".to_string();
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
            HavenBonusType::MaxFishingRank => format!("+{:.0} Max Fishing Rank", value),
            HavenBonusType::StormForgeAccess => "Stormbreaker forging enabled".to_string(),
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
    pub max_fishing_rank_bonus: u32,
    pub has_storm_forge: bool,
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
            max_fishing_rank_bonus: self.fishing_rank_bonus(),
            has_storm_forge: self.has_storm_forge(),
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
    // Comprehensive Bonus Value Tests (all 13 rooms × 3 tiers)
    // =========================================================================

    #[test]
    fn test_all_room_bonus_values_tier_1() {
        // Verify T1 bonus values match design doc
        assert_eq!(HavenRoomId::Hearthstone.bonus_value(1), 25.0); // Offline XP
        assert_eq!(HavenRoomId::Armory.bonus_value(1), 5.0); // Damage
        assert_eq!(HavenRoomId::Bedroom.bonus_value(1), 15.0); // Regen Delay Reduction
        assert_eq!(HavenRoomId::TrainingYard.bonus_value(1), 5.0); // XP Gain
        assert_eq!(HavenRoomId::Garden.bonus_value(1), 10.0); // Fishing Timer
        assert_eq!(HavenRoomId::TrophyHall.bonus_value(1), 5.0); // Drop Rate
        assert_eq!(HavenRoomId::Library.bonus_value(1), 20.0); // Challenge Discovery
        assert_eq!(HavenRoomId::Watchtower.bonus_value(1), 5.0); // Crit Chance
        assert_eq!(HavenRoomId::FishingDock.bonus_value(1), 25.0); // Double Fish
        assert_eq!(HavenRoomId::AlchemyLab.bonus_value(1), 25.0); // HP Regen
        assert_eq!(HavenRoomId::Workshop.bonus_value(1), 10.0); // Item Rarity
        assert_eq!(HavenRoomId::WarRoom.bonus_value(1), 10.0); // Double Strike
        assert_eq!(HavenRoomId::Vault.bonus_value(1), 1.0); // Vault Slots
    }

    #[test]
    fn test_all_room_bonus_values_tier_2() {
        // Verify T2 bonus values match design doc
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
        // Verify T3 bonus values match design doc
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
            // Tier above max should be 0
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
        // Build full fishing/utility branch to Vault capstone
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

        // Build entire tree in dependency order
        // Root
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_some());

        // Depth 1
        assert!(haven.build_room(HavenRoomId::Armory).is_some());
        assert!(haven.build_room(HavenRoomId::Bedroom).is_some());

        // Depth 2
        assert!(haven.build_room(HavenRoomId::TrainingYard).is_some());
        assert!(haven.build_room(HavenRoomId::Garden).is_some());

        // Depth 3
        assert!(haven.build_room(HavenRoomId::TrophyHall).is_some());
        assert!(haven.build_room(HavenRoomId::Library).is_some());
        assert!(haven.build_room(HavenRoomId::Watchtower).is_some());
        assert!(haven.build_room(HavenRoomId::FishingDock).is_some());
        assert!(haven.build_room(HavenRoomId::AlchemyLab).is_some());
        assert!(haven.build_room(HavenRoomId::Workshop).is_some());

        // Depth 4 (Capstones)
        assert!(haven.build_room(HavenRoomId::WarRoom).is_some());
        assert!(haven.build_room(HavenRoomId::Vault).is_some());

        // Depth 5 (StormForge - requires both capstones)
        assert!(haven.build_room(HavenRoomId::StormForge).is_some());

        assert_eq!(haven.rooms_built(), 14);
        assert_eq!(haven.total_rooms(), 14);
    }

    #[test]
    fn test_max_all_rooms_to_max_tier() {
        let mut haven = Haven::new();

        // Build all rooms to their max tier
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
        // FishingDock has 4 tiers
        for _ in 0..4 {
            haven.build_room(HavenRoomId::FishingDock);
        }
        for _ in 0..3 {
            haven.build_room(HavenRoomId::WarRoom);
            haven.build_room(HavenRoomId::Vault);
        }
        // StormForge has 1 tier
        haven.build_room(HavenRoomId::StormForge);

        // Verify all rooms at their max tier
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
        // T1 + T2 + T3 costs for Hearthstone (depth 0): 1 + 2 + 3 = 6
        let total = tier_cost(HavenRoomId::Hearthstone, 1)
            + tier_cost(HavenRoomId::Hearthstone, 2)
            + tier_cost(HavenRoomId::Hearthstone, 3);
        assert_eq!(total, 6);

        // T1 + T2 + T3 costs for Armory (depth 1): 1 + 3 + 5 = 9
        let total = tier_cost(HavenRoomId::Armory, 1)
            + tier_cost(HavenRoomId::Armory, 2)
            + tier_cost(HavenRoomId::Armory, 3);
        assert_eq!(total, 9);

        // T1 + T2 + T3 costs for capstone (depth 4): 3 + 5 + 7 = 15
        let total = tier_cost(HavenRoomId::WarRoom, 1)
            + tier_cost(HavenRoomId::WarRoom, 2)
            + tier_cost(HavenRoomId::WarRoom, 3);
        assert_eq!(total, 15);
    }

    #[test]
    fn test_total_tokens_to_max_entire_haven() {
        // Calculate total tokens needed to max all 13 rooms
        let mut total = 0u32;
        for room in HavenRoomId::ALL {
            for tier in 1..=3 {
                total += tier_cost(room, tier);
            }
        }
        // This is a lot of tokens - verifies the economy is intentionally grindy
        assert!(total > 100, "Total tokens to max Haven: {}", total);
    }

    #[test]
    fn test_partial_token_spending_preserves_progress() {
        let mut haven = Haven::new();

        // Build Hearthstone T1 (cost 1)
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);

        // Build Armory T1 (cost 1)
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.room_tier(HavenRoomId::Armory), 1);

        // Upgrade Hearthstone T2 (cost 2)
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 2);

        // Previous buildings still intact
        assert_eq!(haven.room_tier(HavenRoomId::Armory), 1);
    }

    // =========================================================================
    // Edge Cases and Error Handling
    // =========================================================================

    #[test]
    fn test_cannot_build_child_before_parent() {
        let haven = Haven::new();
        // TrainingYard requires Armory, which requires Hearthstone
        assert!(!haven.is_room_unlocked(HavenRoomId::TrainingYard));
        assert!(!haven.can_build(HavenRoomId::TrainingYard));
    }

    #[test]
    fn test_capstone_not_unlocked_with_only_one_parent() {
        let mut haven = Haven::new();

        // Build path to Watchtower only
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);

        // WarRoom needs both Watchtower AND AlchemyLab
        assert!(!haven.is_room_unlocked(HavenRoomId::WarRoom));
        assert!(!haven.can_build(HavenRoomId::WarRoom));
    }

    #[test]
    fn test_building_returns_new_tier() {
        let mut haven = Haven::new();
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), Some(1));
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), Some(2));
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), Some(3));
        assert_eq!(haven.build_room(HavenRoomId::Hearthstone), None); // Already maxed
    }

    #[test]
    fn test_tree_structure_integrity() {
        // Verify every non-root room has at least one parent
        for room in HavenRoomId::ALL {
            if room != HavenRoomId::Hearthstone {
                assert!(!room.parents().is_empty(), "{:?} should have parents", room);
            }
        }

        // Verify capstones have exactly 2 parents
        assert_eq!(HavenRoomId::WarRoom.parents().len(), 2);
        assert_eq!(HavenRoomId::Vault.parents().len(), 2);
        assert_eq!(HavenRoomId::StormForge.parents().len(), 2);

        // Verify StormForge (ultimate capstone) has no children
        assert!(HavenRoomId::StormForge.children().is_empty());
    }

    #[test]
    fn test_all_bonus_types_mapped_to_rooms() {
        // Ensure every bonus type is provided by exactly one room
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

        // Build one room of each bonus type to T1
        haven.build_room(HavenRoomId::Hearthstone); // Offline XP
        haven.build_room(HavenRoomId::Armory); // Damage
        haven.build_room(HavenRoomId::Bedroom); // Regen Delay
        haven.build_room(HavenRoomId::TrainingYard); // XP Gain
        haven.build_room(HavenRoomId::Garden); // Fishing Timer
        haven.build_room(HavenRoomId::TrophyHall); // Drop Rate
        haven.build_room(HavenRoomId::Library); // Challenge Discovery
        haven.build_room(HavenRoomId::Watchtower); // Crit
        haven.build_room(HavenRoomId::FishingDock); // Double Fish
        haven.build_room(HavenRoomId::AlchemyLab); // HP Regen
        haven.build_room(HavenRoomId::Workshop); // Item Rarity
        haven.build_room(HavenRoomId::WarRoom); // Double Strike
        haven.build_room(HavenRoomId::Vault); // Vault Slots

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

        // Build path to Vault
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::Workshop);

        // Build Vault tiers
        haven.build_room(HavenRoomId::Vault);
        assert_eq!(haven.vault_tier(), 1);

        haven.build_room(HavenRoomId::Vault);
        assert_eq!(haven.vault_tier(), 2);

        haven.build_room(HavenRoomId::Vault);
        assert_eq!(haven.vault_tier(), 3);
    }
}
