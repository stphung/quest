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
        achievement_id: AchievementId::Level250,
        title_text: "Legendary",
    },
    TitleDef {
        achievement_id: AchievementId::Level500,
        title_text: "Mythic",
    },
    TitleDef {
        achievement_id: AchievementId::Level1000,
        title_text: "Immortal",
    },
    TitleDef {
        achievement_id: AchievementId::Level1500,
        title_text: "Transcendent",
    },
    TitleDef {
        achievement_id: AchievementId::PrestigeXXV,
        title_text: "Diamond",
    },
    TitleDef {
        achievement_id: AchievementId::PrestigeL,
        title_text: "Emerald",
    },
    TitleDef {
        achievement_id: AchievementId::PrestigeLXX,
        title_text: "Obsidian",
    },
    TitleDef {
        achievement_id: AchievementId::Eternal,
        title_text: "Eternal",
    },
    // Combat
    TitleDef {
        achievement_id: AchievementId::SlayerV,
        title_text: "Slayer",
    },
    TitleDef {
        achievement_id: AchievementId::SlayerX,
        title_text: "Destroyer",
    },
    TitleDef {
        achievement_id: AchievementId::SlayerXV,
        title_text: "Annihilator",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterV,
        title_text: "Boss Hunter",
    },
    TitleDef {
        achievement_id: AchievementId::BossHunterX,
        title_text: "Bane of Bosses",
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
        achievement_id: AchievementId::MasterSmith,
        title_text: "Soulforged",
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
        assert_eq!(get_title_text(AchievementId::Eternal), Some("Eternal"));
        assert_eq!(get_title_text(AchievementId::SlayerV), Some("Slayer"));
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
        assert_eq!(titles[0].title_text, "Slayer");
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
