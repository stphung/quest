//! Static achievement definitions.

#![allow(dead_code)] // Will be used when integrated with UI

use super::types::{AchievementCategory, AchievementDef, AchievementId};

/// All achievement definitions in display order.
pub const ALL_ACHIEVEMENTS: &[AchievementDef] = &[
    // ═══════════════════════════════════════════════════════════════
    // COMBAT ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::FirstBlood,
        name: "First Blood",
        description: "Defeat your first enemy",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::SlayerI,
        name: "Slayer I",
        description: "Defeat 100 enemies",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::SlayerII,
        name: "Slayer II",
        description: "Defeat 1,000 enemies",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::SlayerIII,
        name: "Slayer III",
        description: "Defeat 10,000 enemies",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::BossHunterI,
        name: "Boss Hunter I",
        description: "Defeat your first boss",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "👹",
    },
    AchievementDef {
        id: AchievementId::BossHunterII,
        name: "Boss Hunter II",
        description: "Defeat 10 bosses",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "👹",
    },
    AchievementDef {
        id: AchievementId::BossHunterIII,
        name: "Boss Hunter III",
        description: "Defeat 50 bosses",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "👹",
    },
    AchievementDef {
        id: AchievementId::ZoneClearer,
        name: "Zone Clearer",
        description: "Clear all subzones in any zone",
        category: AchievementCategory::Combat,
        secret: false,
        icon: "🗺️",
    },
    // ═══════════════════════════════════════════════════════════════
    // PROGRESSION ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::Level10,
        name: "Getting Started",
        description: "Reach level 10",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "📈",
    },
    AchievementDef {
        id: AchievementId::Level50,
        name: "Seasoned Adventurer",
        description: "Reach level 50",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "📈",
    },
    AchievementDef {
        id: AchievementId::Level100,
        name: "Centurion",
        description: "Reach level 100",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "📈",
    },
    AchievementDef {
        id: AchievementId::FirstPrestige,
        name: "Rebirth",
        description: "Prestige for the first time",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "🔄",
    },
    AchievementDef {
        id: AchievementId::PrestigeV,
        name: "Silver Rank",
        description: "Reach Prestige Rank 5",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "🥈",
    },
    AchievementDef {
        id: AchievementId::PrestigeX,
        name: "Gold Rank",
        description: "Reach Prestige Rank 10",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "🥇",
    },
    AchievementDef {
        id: AchievementId::PrestigeXV,
        name: "Platinum Rank",
        description: "Reach Prestige Rank 15",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "💎",
    },
    AchievementDef {
        id: AchievementId::PrestigeXX,
        name: "Diamond Rank",
        description: "Reach Prestige Rank 20",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "💠",
    },
    AchievementDef {
        id: AchievementId::Eternal,
        name: "Eternal",
        description: "Reach the Eternal prestige tier",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "♾️",
    },
    AchievementDef {
        id: AchievementId::ZoneMasterI,
        name: "Forest Walker",
        description: "Clear Zone 2: Dark Forest",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "🌲",
    },
    AchievementDef {
        id: AchievementId::ZoneMasterII,
        name: "Volcano Conqueror",
        description: "Clear Zone 5: Volcanic Wastes",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "🌋",
    },
    AchievementDef {
        id: AchievementId::ZoneMasterIII,
        name: "Sky Lord",
        description: "Clear Zone 9: Floating Isles",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "☁️",
    },
    AchievementDef {
        id: AchievementId::TheStormbreaker,
        name: "The Stormbreaker",
        description: "Forge the legendary Stormbreaker at the Haven forge",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "⚡",
    },
    AchievementDef {
        id: AchievementId::GameComplete,
        name: "Storm's End",
        description: "Defeat the final boss of Zone 10: Storm Citadel",
        category: AchievementCategory::Progression,
        secret: false,
        icon: "🏆",
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::ChessNovice,
        name: "Chess Novice",
        description: "Win a game of chess",
        category: AchievementCategory::Challenges,
        secret: false,
        icon: "♟️",
    },
    AchievementDef {
        id: AchievementId::ChessMaster,
        name: "Chess Master",
        description: "Win a game of chess on Master difficulty",
        category: AchievementCategory::Challenges,
        secret: false,
        icon: "♛",
    },
    AchievementDef {
        id: AchievementId::MorrisWinner,
        name: "Mill Master",
        description: "Win a game of Nine Men's Morris",
        category: AchievementCategory::Challenges,
        secret: false,
        icon: "⚪",
    },
    AchievementDef {
        id: AchievementId::AllRounder,
        name: "All-Rounder",
        description: "Win each type of minigame at least once",
        category: AchievementCategory::Challenges,
        secret: false,
        icon: "🎮",
    },
    AchievementDef {
        id: AchievementId::GrandChampion,
        name: "Grand Champion",
        description: "Win 100 minigames total",
        category: AchievementCategory::Challenges,
        secret: false,
        icon: "🏅",
    },
    // ═══════════════════════════════════════════════════════════════
    // EXPLORATION ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::GoneFishing,
        name: "Gone Fishing",
        description: "Catch your first fish",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🎣",
    },
    AchievementDef {
        id: AchievementId::FishermanI,
        name: "Fisherman I",
        description: "Reach fishing rank 10",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🐟",
    },
    AchievementDef {
        id: AchievementId::FishermanII,
        name: "Fisherman II",
        description: "Reach fishing rank 20",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🐠",
    },
    AchievementDef {
        id: AchievementId::FishermanIII,
        name: "Fisherman III",
        description: "Reach fishing rank 30 (requires Fishing Dock T4)",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🐡",
    },
    AchievementDef {
        id: AchievementId::StormLeviathan,
        name: "Storm Leviathan",
        description: "Catch the legendary Storm Leviathan",
        category: AchievementCategory::Exploration,
        secret: true, // Hidden until unlocked
        icon: "🐋",
    },
    AchievementDef {
        id: AchievementId::DungeonDiver,
        name: "Dungeon Diver",
        description: "Complete your first dungeon",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🏰",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterI,
        name: "Dungeon Master I",
        description: "Complete 10 dungeons",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🗝️",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterII,
        name: "Dungeon Master II",
        description: "Complete 50 dungeons",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🗝️",
    },
    AchievementDef {
        id: AchievementId::HavenDiscovered,
        name: "Haven Found",
        description: "Discover the Haven",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🏠",
    },
    AchievementDef {
        id: AchievementId::HavenArchitect,
        name: "Haven Architect",
        description: "Upgrade all Haven rooms to Tier 3",
        category: AchievementCategory::Exploration,
        secret: false,
        icon: "🏛️",
    },
];

/// Get the definition for a specific achievement.
pub fn get_achievement_def(id: AchievementId) -> Option<&'static AchievementDef> {
    ALL_ACHIEVEMENTS.iter().find(|a| a.id == id)
}

/// Get achievements filtered by category.
pub fn get_achievements_by_category(category: AchievementCategory) -> Vec<&'static AchievementDef> {
    ALL_ACHIEVEMENTS
        .iter()
        .filter(|a| a.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_achievements_have_unique_ids() {
        use std::collections::HashSet;
        let mut ids = HashSet::new();
        for achievement in ALL_ACHIEVEMENTS {
            assert!(
                ids.insert(achievement.id),
                "Duplicate achievement ID: {:?}",
                achievement.id
            );
        }
    }

    #[test]
    fn test_get_achievement_def() {
        let def = get_achievement_def(AchievementId::FirstBlood).unwrap();
        assert_eq!(def.name, "First Blood");
        assert_eq!(def.category, AchievementCategory::Combat);
    }

    #[test]
    fn test_get_achievements_by_category() {
        let combat = get_achievements_by_category(AchievementCategory::Combat);
        assert!(!combat.is_empty());
        for a in combat {
            assert_eq!(a.category, AchievementCategory::Combat);
        }
    }

    #[test]
    fn test_secret_achievements() {
        // StormLeviathan should be secret
        let def = get_achievement_def(AchievementId::StormLeviathan).unwrap();
        assert!(def.secret);

        // FirstBlood should not be secret
        let def = get_achievement_def(AchievementId::FirstBlood).unwrap();
        assert!(!def.secret);
    }
}
