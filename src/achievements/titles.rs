//! Title definitions — maps curated achievements to display text.

use super::types::AchievementId;

/// A title that can be earned and displayed after the character name.
pub struct TitleDef {
    pub achievement_id: AchievementId,
    pub title_text: &'static str,
}

/// All available titles, in display order.
pub const ALL_TITLES: &[TitleDef] = &[
    // Level & Prestige
    TitleDef {
        achievement_id: AchievementId::Level100,
        title_text: "Centurion",
    },
    TitleDef {
        achievement_id: AchievementId::Level250,
        title_text: "Legendary",
    },
    TitleDef {
        achievement_id: AchievementId::Level750,
        title_text: "Demigod",
    },
    TitleDef {
        achievement_id: AchievementId::Level1500,
        title_text: "Transcendent",
    },
    TitleDef {
        achievement_id: AchievementId::Level2000,
        title_text: "Ascendant",
    },
    TitleDef {
        achievement_id: AchievementId::Level5000,
        title_text: "Worldshaper",
    },
    TitleDef {
        achievement_id: AchievementId::Level10000,
        title_text: "Godmarch",
    },
    TitleDef {
        achievement_id: AchievementId::Level20000,
        title_text: "Infinite",
    },
    TitleDef {
        achievement_id: AchievementId::Level100000,
        title_text: "Unfathomable",
    },
    TitleDef {
        achievement_id: AchievementId::PrestigeX,
        title_text: "Silver-Forged",
    },
    TitleDef {
        achievement_id: AchievementId::PrestigeXXV,
        title_text: "Diamond-Forged",
    },
    TitleDef {
        achievement_id: AchievementId::PrestigeL,
        title_text: "Emerald-Forged",
    },
    TitleDef {
        achievement_id: AchievementId::Eternal,
        title_text: "Everlasting",
    },
    TitleDef {
        achievement_id: AchievementId::Prestige200,
        title_text: "Aeonforged",
    },
    TitleDef {
        achievement_id: AchievementId::Prestige500,
        title_text: "Voidforged",
    },
    TitleDef {
        achievement_id: AchievementId::Prestige1000,
        title_text: "Boundless",
    },
    TitleDef {
        achievement_id: AchievementId::Prestige10000,
        title_text: "Omegaforged",
    },
    TitleDef {
        achievement_id: AchievementId::StormsEnd,
        title_text: "Stormborn",
    },
    TitleDef {
        achievement_id: AchievementId::BeyondInfinity,
        title_text: "Voidwalker",
    },
    // Ascension
    TitleDef {
        achievement_id: AchievementId::AscensionI,
        title_text: "Ascended I",
    },
    TitleDef {
        achievement_id: AchievementId::AscensionIII,
        title_text: "Ascended III",
    },
    TitleDef {
        achievement_id: AchievementId::AscensionVI,
        title_text: "Ascended VI",
    },
    // Combat
    TitleDef {
        achievement_id: AchievementId::SlayerV,
        title_text: "Battleborn",
    },
    TitleDef {
        achievement_id: AchievementId::SlayerX,
        title_text: "Destroyer",
    },
    TitleDef {
        achievement_id: AchievementId::SlayerXIII,
        title_text: "Harbinger",
    },
    TitleDef {
        achievement_id: AchievementId::SlayerXIV,
        title_text: "Reaper",
    },
    TitleDef {
        achievement_id: AchievementId::SlayerXV,
        title_text: "Annihilator",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterV,
        title_text: "Throneseeker",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterX,
        title_text: "Dreadbane",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterXIII,
        title_text: "Titan Breaker",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterXIV,
        title_text: "Worldender",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterXV,
        title_text: "Godslayer",
    },
    // Challenges
    TitleDef {
        achievement_id: AchievementId::GrandChampion,
        title_text: "Grand Champion",
    },
    TitleDef {
        achievement_id: AchievementId::ChessMaster,
        title_text: "Grandmaster",
    },
    TitleDef {
        achievement_id: AchievementId::GoMaster,
        title_text: "Sovereign",
    },
    TitleDef {
        achievement_id: AchievementId::MorrisMaster,
        title_text: "Millwright",
    },
    TitleDef {
        achievement_id: AchievementId::GomokuMaster,
        title_text: "Five-Stone Sage",
    },
    TitleDef {
        achievement_id: AchievementId::MinesweeperMaster,
        title_text: "Trapbreaker",
    },
    TitleDef {
        achievement_id: AchievementId::RuneMaster,
        title_text: "Runeweaver",
    },
    TitleDef {
        achievement_id: AchievementId::FlappyMaster,
        title_text: "Skypiercer",
    },
    TitleDef {
        achievement_id: AchievementId::SnakeMaster,
        title_text: "Serpent Lord",
    },
    TitleDef {
        achievement_id: AchievementId::ContainmentBreachMaster,
        title_text: "Warden",
    },
    TitleDef {
        achievement_id: AchievementId::SigilSurgeMaster,
        title_text: "Sigil Savant",
    },
    // Exploration
    TitleDef {
        achievement_id: AchievementId::FishCatcherVIII,
        title_text: "Leviathan's Rival",
    },
    TitleDef {
        achievement_id: AchievementId::FishCatcherIX,
        title_text: "Poseidon's Hand",
    },
    TitleDef {
        achievement_id: AchievementId::FishCatcherX,
        title_text: "Lord of the Deep",
    },
    TitleDef {
        achievement_id: AchievementId::DungeonMasterVIII,
        title_text: "Labyrinth Lord",
    },
    TitleDef {
        achievement_id: AchievementId::DungeonMasterIX,
        title_text: "Abyss Walker",
    },
    TitleDef {
        achievement_id: AchievementId::DungeonMasterX,
        title_text: "Undying Delver",
    },
    TitleDef {
        achievement_id: AchievementId::StormLeviathan,
        title_text: "Leviathan Slayer",
    },
    TitleDef {
        achievement_id: AchievementId::FishermanIV,
        title_text: "Master Angler",
    },
    TitleDef {
        achievement_id: AchievementId::HavenArchitect,
        title_text: "Architect",
    },
    TitleDef {
        achievement_id: AchievementId::FullyTempered,
        title_text: "Tempered Soul",
    },
    TitleDef {
        achievement_id: AchievementId::SoulConvergence,
        title_text: "Radiant Soul",
    },
    TitleDef {
        achievement_id: AchievementId::SoulforgeMaster,
        title_text: "Emberforged",
    },
    TitleDef {
        achievement_id: AchievementId::SoulforgeGrandmaster,
        title_text: "Flameforged",
    },
    TitleDef {
        achievement_id: AchievementId::SoulforgeAscendant,
        title_text: "Soulforged",
    },
    // Fracture Zones
    TitleDef {
        achievement_id: AchievementId::FractureZone30,
        title_text: "Scar Mender",
    },
    // The Deep
    TitleDef {
        achievement_id: AchievementId::GuildRank5,
        title_text: "Overlord",
    },
    TitleDef {
        achievement_id: AchievementId::VoidExplorer,
        title_text: "Voidtouched",
    },
    TitleDef {
        achievement_id: AchievementId::DeepMissionsC,
        title_text: "Oathsworn",
    },
    TitleDef {
        achievement_id: AchievementId::GatewayOpened,
        title_text: "Unbinder",
    },
];

/// Get the title text for an achievement, if it grants a title.
pub fn get_title_text(id: AchievementId) -> Option<&'static str> {
    ALL_TITLES
        .iter()
        .find(|t| t.achievement_id == id)
        .map(|t| t.title_text)
}

/// Get all titles the player has unlocked, in display order.
pub fn get_unlocked_titles(achievements: &super::types::Achievements) -> Vec<&'static TitleDef> {
    ALL_TITLES
        .iter()
        .filter(|t| achievements.is_unlocked(t.achievement_id))
        .collect()
}

/// Validate the selected title — clear it if the achievement isn't unlocked.
pub fn validate_selected_title(achievements: &mut super::types::Achievements) {
    if let Some(id) = achievements.selected_title {
        if !achievements.is_unlocked(id) || get_title_text(id).is_none() {
            achievements.selected_title = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::types::Achievements;

    #[test]
    fn test_get_title_text_exists() {
        assert_eq!(get_title_text(AchievementId::Eternal), Some("Everlasting"));
        assert_eq!(
            get_title_text(AchievementId::Level100000),
            Some("Unfathomable")
        );
        assert_eq!(
            get_title_text(AchievementId::Prestige10000),
            Some("Omegaforged")
        );
        assert_eq!(get_title_text(AchievementId::SlayerV), Some("Battleborn"));
    }

    #[test]
    fn test_get_title_text_none_for_non_title() {
        assert_eq!(get_title_text(AchievementId::SlayerI), None);
        assert_eq!(get_title_text(AchievementId::Level10), None);
    }

    #[test]
    fn test_get_unlocked_titles_empty() {
        let achievements = Achievements::default();
        assert!(get_unlocked_titles(&achievements).is_empty());
    }

    #[test]
    fn test_get_unlocked_titles_filters() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerV, Some("Hero".to_string()));
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string())); // no title
        let titles = get_unlocked_titles(&achievements);
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].title_text, "Battleborn");
    }

    #[test]
    fn test_validate_clears_invalid() {
        let mut achievements = Achievements {
            selected_title: Some(AchievementId::Eternal),
            ..Default::default()
        };
        validate_selected_title(&mut achievements);
        assert_eq!(achievements.selected_title, None);
    }

    #[test]
    fn test_validate_keeps_valid() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerV, Some("Hero".to_string()));
        achievements.selected_title = Some(AchievementId::SlayerV);
        validate_selected_title(&mut achievements);
        assert_eq!(achievements.selected_title, Some(AchievementId::SlayerV));
    }

    #[test]
    fn test_validate_clears_non_title_achievement() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
        achievements.selected_title = Some(AchievementId::SlayerI);
        validate_selected_title(&mut achievements);
        assert_eq!(achievements.selected_title, None);
    }
}
