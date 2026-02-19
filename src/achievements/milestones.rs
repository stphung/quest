//! Milestone threshold constants and minigame type enums.
//!
//! Extracted from `types.rs` to keep data structures separate from
//! achievement threshold tables. Each constant maps a counter value
//! to the [`AchievementId`] that should unlock when reached.

use super::types::AchievementId;

// =========================================================================
// Minigame type and difficulty enums
// =========================================================================

/// Type-safe identifier for each minigame variant.
///
/// Used by [`Achievements::on_minigame_won`] to replace the previous
/// `&'static str` game-type parameter and eliminate the 40-arm string match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinigameType {
    Chess,
    Morris,
    Gomoku,
    Minesweeper,
    Rune,
    Go,
    FlappyBird,
    Snake,
    Jezzball,
    RunicShift,
}

/// Type-safe difficulty level shared by all minigames.
///
/// Mirrors the per-game `*Difficulty` enums but is game-agnostic,
/// allowing a single enum to describe difficulty across all games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinigameDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

// =========================================================================
// Milestone threshold constants
// =========================================================================

/// Kill count milestones for the Slayer achievement series.
pub const SLAYER_MILESTONES: &[(u64, AchievementId)] = &[
    (100, AchievementId::SlayerI),
    (500, AchievementId::SlayerII),
    (1000, AchievementId::SlayerIII),
    (5000, AchievementId::SlayerIV),
    (10000, AchievementId::SlayerV),
    (50000, AchievementId::SlayerVI),
    (100000, AchievementId::SlayerVII),
    (500000, AchievementId::SlayerVIII),
    (1000000, AchievementId::SlayerIX),
    (2500000, AchievementId::SlayerX),
    (10000000, AchievementId::SlayerXI),
    (50000000, AchievementId::SlayerXII),
    (100000000, AchievementId::SlayerXIII),
    (500000000, AchievementId::SlayerXIV),
    (1000000000, AchievementId::SlayerXV),
];

/// Boss kill milestones for the Boss Hunter achievement series.
pub const BOSS_HUNTER_MILESTONES: &[(u64, AchievementId)] = &[
    (1, AchievementId::BossHunterI),
    (10, AchievementId::BossHunterII),
    (50, AchievementId::BossHunterIII),
    (100, AchievementId::BossHunterIV),
    (500, AchievementId::BossHunterV),
    (1000, AchievementId::BossHunterVI),
    (5000, AchievementId::BossHunterVII),
    (10000, AchievementId::BossHunterVIII),
    (25000, AchievementId::BossHunterIX),
    (75000, AchievementId::BossHunterX),
    (250000, AchievementId::BossHunterXI),
    (750000, AchievementId::BossHunterXII),
    (2500000, AchievementId::BossHunterXIII),
    (5000000, AchievementId::BossHunterXIV),
    (10000000, AchievementId::BossHunterXV),
];

/// Level milestones for level-up achievements.
pub const LEVEL_MILESTONES: &[(u64, AchievementId)] = &[
    (10, AchievementId::Level10),
    (25, AchievementId::Level25),
    (50, AchievementId::Level50),
    (100, AchievementId::Level100),
    (150, AchievementId::Level150),
    (200, AchievementId::Level200),
    (250, AchievementId::Level250),
    (500, AchievementId::Level500),
    (750, AchievementId::Level750),
    (1000, AchievementId::Level1000),
    (1500, AchievementId::Level1500),
];

/// Prestige rank milestones for prestige achievements.
pub const PRESTIGE_MILESTONES: &[(u64, AchievementId)] = &[
    (1, AchievementId::FirstPrestige),
    (5, AchievementId::PrestigeV),
    (10, AchievementId::PrestigeX),
    (15, AchievementId::PrestigeXV),
    (20, AchievementId::PrestigeXX),
    (25, AchievementId::PrestigeXXV),
    (30, AchievementId::PrestigeXXX),
    (40, AchievementId::PrestigeXL),
    (50, AchievementId::PrestigeL),
    (70, AchievementId::PrestigeLXX),
    (90, AchievementId::PrestigeXC),
    (100, AchievementId::Eternal),
];

/// Fish catch count milestones for fish catcher achievements.
pub const FISH_CATCHER_MILESTONES: &[(u64, AchievementId)] = &[
    (1, AchievementId::GoneFishing),
    (100, AchievementId::FishCatcherI),
    (1000, AchievementId::FishCatcherII),
    (10000, AchievementId::FishCatcherIII),
    (100000, AchievementId::FishCatcherIV),
    (500000, AchievementId::FishCatcherV),
    (1000000, AchievementId::FishCatcherVI),
    (5000000, AchievementId::FishCatcherVII),
    (10000000, AchievementId::FishCatcherVIII),
    (50000000, AchievementId::FishCatcherIX),
    (100000000, AchievementId::FishCatcherX),
];

/// Fishing rank milestones for fisherman achievements.
pub const FISHERMAN_MILESTONES: &[(u64, AchievementId)] = &[
    (10, AchievementId::FishermanI),
    (20, AchievementId::FishermanII),
    (30, AchievementId::FishermanIII),
    (40, AchievementId::FishermanIV),
];

/// Dungeon completion milestones for dungeon master achievements.
pub const DUNGEON_MILESTONES: &[(u64, AchievementId)] = &[
    (1, AchievementId::DungeonDiver),
    (10, AchievementId::DungeonMasterI),
    (50, AchievementId::DungeonMasterII),
    (100, AchievementId::DungeonMasterIII),
    (1000, AchievementId::DungeonMasterIV),
    (5000, AchievementId::DungeonMasterV),
    (10000, AchievementId::DungeonMasterVI),
    (25000, AchievementId::DungeonMasterVII),
    (100000, AchievementId::DungeonMasterVIII),
    (500000, AchievementId::DungeonMasterIX),
    (1000000, AchievementId::DungeonMasterX),
];

/// Minigame win count milestone for the Grand Champion achievement.
pub const GRAND_CHAMPION_MILESTONES: &[(u64, AchievementId)] =
    &[(100, AchievementId::GrandChampion)];
