//! Haven bonus types, definitions, and computed bonus aggregation.
//!
//! Contains `HavenBonusType`, `HavenBonus`, `HavenBonuses`, the per-room
//! `bonus()`/`bonus_value()`/`format_bonus()` methods on `HavenRoomId`,
//! `Haven::compute_bonuses()`, and `haven_discovery_chance()`.

use super::room_defs::HavenRoomId;
use super::types::Haven;

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

/// Pre-computed Haven bonuses for efficient access during gameplay
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)] // Some fields used outside tick.rs (main.rs, input.rs)
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
    /// Get the bonus value for a specific bonus type
    pub fn get_bonus(&self, bonus_type: HavenBonusType) -> f64 {
        HavenRoomId::ALL
            .iter()
            .filter(|r| r.bonus().bonus_type == bonus_type)
            .map(|r| r.bonus_value(self.room_tier(*r)))
            .sum()
    }

    /// Compute all bonuses from the current Haven state.
    ///
    /// Uses a single pass over all rooms instead of per-bonus-type passes,
    /// reducing iterations from ~196 (14 rooms x 14 bonus types) to 14.
    pub fn compute_bonuses(&self) -> HavenBonuses {
        let mut bonuses = HavenBonuses::default();
        for room in HavenRoomId::ALL.iter() {
            let tier = self.room_tier(*room);
            if tier == 0 {
                continue;
            }
            let bonus = room.bonus();
            let value = bonus.values[(tier - 1) as usize];
            match bonus.bonus_type {
                HavenBonusType::DamagePercent => bonuses.damage_percent += value,
                HavenBonusType::XpGainPercent => bonuses.xp_gain_percent += value,
                HavenBonusType::DropRatePercent => bonuses.drop_rate_percent += value,
                HavenBonusType::CritChancePercent => bonuses.crit_chance_percent += value,
                HavenBonusType::HpRegenPercent => bonuses.hp_regen_percent += value,
                HavenBonusType::DoubleStrikeChance => bonuses.double_strike_chance += value,
                HavenBonusType::OfflineXpPercent => bonuses.offline_xp_percent += value,
                HavenBonusType::ChallengeDiscoveryPercent => {
                    bonuses.challenge_discovery_percent += value;
                }
                HavenBonusType::FishingTimerReduction => bonuses.fishing_timer_reduction += value,
                HavenBonusType::DoubleFishChance => bonuses.double_fish_chance += value,
                HavenBonusType::ItemRarityPercent => bonuses.item_rarity_percent += value,
                HavenBonusType::HpRegenDelayReduction => {
                    bonuses.hp_regen_delay_reduction += value;
                }
                HavenBonusType::VaultSlots => bonuses.vault_slots = value as u8,
                HavenBonusType::MaxFishingRank | HavenBonusType::StormForgeAccess => {
                    // Handled below via dedicated methods that encode special logic
                }
            }
        }
        // FishingDock T4 grants +10 max fishing rank (different bonus type than T1-3)
        bonuses.max_fishing_rank_bonus = self.fishing_rank_bonus();
        // StormForge is a boolean presence check (single tier)
        bonuses.has_storm_forge = self.has_storm_forge();
        bonuses
    }
}

/// Discovery chance per tick. Scales with prestige rank.
/// Base: HAVEN_DISCOVERY_BASE_CHANCE (~2hr at P10), +HAVEN_DISCOVERY_RANK_BONUS per rank above min.
pub fn haven_discovery_chance(prestige_rank: u32) -> f64 {
    use crate::core::constants::{
        HAVEN_DISCOVERY_BASE_CHANCE, HAVEN_DISCOVERY_RANK_BONUS, HAVEN_MIN_PRESTIGE_RANK,
    };
    if prestige_rank < HAVEN_MIN_PRESTIGE_RANK {
        return 0.0;
    }
    HAVEN_DISCOVERY_BASE_CHANCE
        + (prestige_rank - HAVEN_MIN_PRESTIGE_RANK) as f64 * HAVEN_DISCOVERY_RANK_BONUS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::HAVEN_MIN_PRESTIGE_RANK;

    #[test]
    fn test_bonus_value_zero_at_tier_zero() {
        assert_eq!(HavenRoomId::Armory.bonus_value(0), 0.0);
    }

    #[test]
    fn test_bonus_value_zero_above_max_tier() {
        // Armory maxes at T3, so T4 is out of range.
        assert_eq!(HavenRoomId::Armory.bonus_value(4), 0.0);
        // StormForge maxes at T1.
        assert_eq!(HavenRoomId::StormForge.bonus_value(2), 0.0);
    }

    #[test]
    fn test_bonus_value_matches_table_for_each_tier() {
        assert_eq!(HavenRoomId::Armory.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(3), 25.0);
    }

    #[test]
    fn test_bonus_types_for_every_room() {
        assert_eq!(
            HavenRoomId::Hearthstone.bonus().bonus_type,
            HavenBonusType::OfflineXpPercent
        );
        assert_eq!(
            HavenRoomId::Armory.bonus().bonus_type,
            HavenBonusType::DamagePercent
        );
        assert_eq!(
            HavenRoomId::TrainingYard.bonus().bonus_type,
            HavenBonusType::XpGainPercent
        );
        assert_eq!(
            HavenRoomId::TrophyHall.bonus().bonus_type,
            HavenBonusType::DropRatePercent
        );
        assert_eq!(
            HavenRoomId::Watchtower.bonus().bonus_type,
            HavenBonusType::CritChancePercent
        );
        assert_eq!(
            HavenRoomId::AlchemyLab.bonus().bonus_type,
            HavenBonusType::HpRegenPercent
        );
        assert_eq!(
            HavenRoomId::WarRoom.bonus().bonus_type,
            HavenBonusType::DoubleStrikeChance
        );
        assert_eq!(
            HavenRoomId::Bedroom.bonus().bonus_type,
            HavenBonusType::HpRegenDelayReduction
        );
        assert_eq!(
            HavenRoomId::Garden.bonus().bonus_type,
            HavenBonusType::FishingTimerReduction
        );
        assert_eq!(
            HavenRoomId::Library.bonus().bonus_type,
            HavenBonusType::ChallengeDiscoveryPercent
        );
        assert_eq!(
            HavenRoomId::FishingDock.bonus().bonus_type,
            HavenBonusType::DoubleFishChance
        );
        assert_eq!(
            HavenRoomId::Workshop.bonus().bonus_type,
            HavenBonusType::ItemRarityPercent
        );
        assert_eq!(
            HavenRoomId::Vault.bonus().bonus_type,
            HavenBonusType::VaultSlots
        );
        assert_eq!(
            HavenRoomId::StormForge.bonus().bonus_type,
            HavenBonusType::StormForgeAccess
        );
    }

    #[test]
    fn test_format_bonus_empty_at_tier_zero() {
        assert_eq!(HavenRoomId::Armory.format_bonus(0), "");
    }

    #[test]
    fn test_format_bonus_fishing_dock_t4_special_case() {
        assert_eq!(
            HavenRoomId::FishingDock.format_bonus(4),
            "+10 Max Fishing Rank"
        );
    }

    #[test]
    fn test_format_bonus_covers_every_bonus_type() {
        assert_eq!(HavenRoomId::Armory.format_bonus(1), "+5% DMG");
        assert_eq!(HavenRoomId::TrainingYard.format_bonus(1), "+5% XP");
        assert_eq!(HavenRoomId::TrophyHall.format_bonus(1), "+5% Drops");
        assert_eq!(HavenRoomId::Watchtower.format_bonus(1), "+5% Crit");
        assert_eq!(HavenRoomId::AlchemyLab.format_bonus(1), "+25% HP Regen");
        assert_eq!(HavenRoomId::WarRoom.format_bonus(1), "+10% Double Strike");
        assert_eq!(HavenRoomId::Hearthstone.format_bonus(1), "+25% Offline XP");
        assert_eq!(HavenRoomId::Library.format_bonus(1), "+20% Discovery");
        assert_eq!(HavenRoomId::Garden.format_bonus(1), "-10% Fishing Timers");
        assert_eq!(HavenRoomId::FishingDock.format_bonus(1), "+25% Double Fish");
        assert_eq!(HavenRoomId::Workshop.format_bonus(1), "+10% Item Rarity");
        assert_eq!(HavenRoomId::Bedroom.format_bonus(1), "-15% Regen Delay");
        assert_eq!(
            HavenRoomId::StormForge.format_bonus(1),
            "Stormbreaker forging enabled"
        );
    }

    #[test]
    fn test_format_bonus_vault_slots_pluralization() {
        // T1 = 1 item preserved (singular)
        assert_eq!(HavenRoomId::Vault.format_bonus(1), "1 item preserved");
        // T2 = 3 items preserved (plural)
        assert_eq!(HavenRoomId::Vault.format_bonus(2), "3 items preserved");
        assert_eq!(HavenRoomId::Vault.format_bonus(3), "5 items preserved");
    }

    #[test]
    fn test_get_bonus_zero_when_unbuilt() {
        let haven = Haven::new();
        assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 0.0);
    }

    #[test]
    fn test_get_bonus_sums_matching_rooms() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 5.0);
    }

    #[test]
    fn test_compute_bonuses_matches_get_bonus_for_every_type() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);

        let computed = haven.compute_bonuses();
        assert_eq!(
            computed.damage_percent,
            haven.get_bonus(HavenBonusType::DamagePercent)
        );
        assert_eq!(
            computed.xp_gain_percent,
            haven.get_bonus(HavenBonusType::XpGainPercent)
        );
        assert_eq!(
            computed.drop_rate_percent,
            haven.get_bonus(HavenBonusType::DropRatePercent)
        );
        assert_eq!(
            computed.crit_chance_percent,
            haven.get_bonus(HavenBonusType::CritChancePercent)
        );
    }

    #[test]
    fn test_compute_bonuses_defaults_are_zero() {
        let haven = Haven::new();
        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.damage_percent, 0.0);
        assert_eq!(bonuses.xp_gain_percent, 0.0);
        assert_eq!(bonuses.drop_rate_percent, 0.0);
        assert_eq!(bonuses.crit_chance_percent, 0.0);
        assert_eq!(bonuses.hp_regen_percent, 0.0);
        assert_eq!(bonuses.double_strike_chance, 0.0);
        assert_eq!(bonuses.offline_xp_percent, 0.0);
        assert_eq!(bonuses.challenge_discovery_percent, 0.0);
        assert_eq!(bonuses.fishing_timer_reduction, 0.0);
        assert_eq!(bonuses.double_fish_chance, 0.0);
        assert_eq!(bonuses.item_rarity_percent, 0.0);
        assert_eq!(bonuses.hp_regen_delay_reduction, 0.0);
        assert_eq!(bonuses.vault_slots, 0);
        assert_eq!(bonuses.max_fishing_rank_bonus, 0);
        assert!(!bonuses.has_storm_forge);
    }

    #[test]
    fn test_compute_bonuses_fishing_dock_t4_grants_max_rank_bonus() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::FishingDock); // T1
        haven.build_room(HavenRoomId::FishingDock); // T2
        haven.build_room(HavenRoomId::FishingDock); // T3
        haven.build_room(HavenRoomId::FishingDock); // T4

        let bonuses = haven.compute_bonuses();
        assert_eq!(bonuses.max_fishing_rank_bonus, 10);
        // T1-3 double fish chance should be at its max value (100%).
        assert_eq!(bonuses.double_fish_chance, 100.0);
    }

    #[test]
    fn test_compute_bonuses_storm_forge_presence() {
        let mut haven = Haven::new();
        assert!(!haven.compute_bonuses().has_storm_forge);

        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        haven.build_room(HavenRoomId::AlchemyLab);
        haven.build_room(HavenRoomId::WarRoom);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::Workshop);
        haven.build_room(HavenRoomId::Vault);
        haven.build_room(HavenRoomId::StormForge);

        assert!(haven.compute_bonuses().has_storm_forge);
    }

    #[test]
    fn test_haven_discovery_chance_zero_below_min_rank() {
        assert_eq!(haven_discovery_chance(0), 0.0);
        assert_eq!(haven_discovery_chance(HAVEN_MIN_PRESTIGE_RANK - 1), 0.0);
    }

    #[test]
    fn test_haven_discovery_chance_positive_at_min_rank() {
        use crate::core::constants::HAVEN_DISCOVERY_BASE_CHANCE;
        let chance = haven_discovery_chance(HAVEN_MIN_PRESTIGE_RANK);
        assert!((chance - HAVEN_DISCOVERY_BASE_CHANCE).abs() < f64::EPSILON);
    }

    #[test]
    fn test_haven_discovery_chance_increases_with_rank() {
        let low = haven_discovery_chance(HAVEN_MIN_PRESTIGE_RANK);
        let high = haven_discovery_chance(HAVEN_MIN_PRESTIGE_RANK + 10);
        assert!(high > low);
    }
}
