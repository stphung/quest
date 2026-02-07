//! Achievement system types and data structures.

#![allow(dead_code)] // Will be used when integrated with UI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Achievement categories for organization in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementCategory {
    Combat,
    Progression,
    Challenges,
    Exploration,
}

impl AchievementCategory {
    /// All categories in display order.
    pub const ALL: [AchievementCategory; 4] = [
        AchievementCategory::Combat,
        AchievementCategory::Progression,
        AchievementCategory::Challenges,
        AchievementCategory::Exploration,
    ];

    /// Display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            AchievementCategory::Combat => "Combat",
            AchievementCategory::Progression => "Progression",
            AchievementCategory::Challenges => "Challenges",
            AchievementCategory::Exploration => "Exploration",
        }
    }
}

/// Unique identifier for each achievement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // Combat achievements
    FirstBlood,
    SlayerI,
    SlayerII,
    SlayerIII,
    BossHunterI,
    BossHunterII,
    BossHunterIII,
    ZoneClearer,

    // Progression achievements
    Level10,
    Level50,
    Level100,
    FirstPrestige,
    PrestigeV,
    PrestigeX,
    PrestigeXV,
    PrestigeXX,
    Eternal,
    // Zone completion achievements (one per zone)
    Zone1Complete,  // Meadow
    Zone2Complete,  // Dark Forest
    Zone3Complete,  // Mountain Pass
    Zone4Complete,  // Ancient Ruins
    Zone5Complete,  // Volcanic Wastes
    Zone6Complete,  // Frozen Tundra
    Zone7Complete,  // Crystal Caverns
    Zone8Complete,  // Sunken Kingdom
    Zone9Complete,  // Floating Isles
    Zone10Complete, // Storm Citadel
    Zone11Complete, // The Expanse (first cycle)
    TheStormbreaker,
    GameComplete,

    // Challenge achievements - Chess
    ChessNovice,
    ChessApprentice,
    ChessJourneyman,
    ChessMaster,
    // Challenge achievements - Morris
    MorrisNovice,
    MorrisApprentice,
    MorrisJourneyman,
    MorrisMaster,
    // Challenge achievements - Gomoku
    GomokuNovice,
    GomokuApprentice,
    GomokuJourneyman,
    GomokuMaster,
    // Challenge achievements - Minesweeper
    MinesweeperNovice,
    MinesweeperApprentice,
    MinesweeperJourneyman,
    MinesweeperMaster,
    // Challenge achievements - Rune
    RuneNovice,
    RuneApprentice,
    RuneJourneyman,
    RuneMaster,
    // Challenge achievements - Go
    GoNovice,
    GoApprentice,
    GoJourneyman,
    GoMaster,
    // Challenge achievements - Meta
    AllRounder,
    GrandChampion,

    // Exploration achievements
    GoneFishing,
    FishermanI,
    FishermanII,
    FishermanIII,
    StormLeviathan,
    DungeonDiver,
    DungeonMasterI,
    DungeonMasterII,
    HavenDiscovered,
    HavenArchitect,
}

/// Static definition of an achievement.
#[derive(Debug, Clone)]
pub struct AchievementDef {
    pub id: AchievementId,
    pub name: &'static str,
    pub description: &'static str,
    pub category: AchievementCategory,
    pub secret: bool,
    pub icon: &'static str,
}

/// Progress on a single achievement (for multi-stage achievements).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AchievementProgress {
    pub current: u64,
    pub target: u64,
}

/// Record of an unlocked achievement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockedAchievement {
    pub unlocked_at: i64,
    pub character_name: Option<String>,
}

/// Global achievement state (saved to disk).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Achievements {
    /// Map of unlocked achievements.
    pub unlocked: HashMap<AchievementId, UnlockedAchievement>,
    /// Progress tracking for multi-stage achievements.
    pub progress: HashMap<AchievementId, AchievementProgress>,

    // Aggregate tracking across all characters
    pub total_kills: u64,
    pub total_bosses_defeated: u64,
    pub total_fish_caught: u64,
    pub total_dungeons_completed: u64,
    pub total_minigame_wins: u64,
    pub highest_prestige_rank: u32,
    pub highest_level: u32,
    pub highest_fishing_rank: u32,
    pub zones_fully_cleared: u32,
}

impl Achievements {
    /// Check if an achievement is unlocked.
    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.unlocked.contains_key(&id)
    }

    /// Unlock an achievement. Returns true if newly unlocked.
    pub fn unlock(&mut self, id: AchievementId, character_name: Option<String>) -> bool {
        if self.is_unlocked(id) {
            return false;
        }
        self.unlocked.insert(
            id,
            UnlockedAchievement {
                unlocked_at: chrono::Utc::now().timestamp(),
                character_name,
            },
        );
        true
    }

    /// Update progress on a tracked achievement.
    pub fn update_progress(&mut self, id: AchievementId, current: u64, target: u64) {
        self.progress
            .insert(id, AchievementProgress { current, target });
    }

    /// Get the progress for an achievement, if any.
    pub fn get_progress(&self, id: AchievementId) -> Option<&AchievementProgress> {
        self.progress.get(&id)
    }

    /// Get the total number of achievements.
    pub fn total_count(&self) -> usize {
        use super::data::ALL_ACHIEVEMENTS;
        ALL_ACHIEVEMENTS.len()
    }

    /// Get the number of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Get unlock percentage (0.0 - 100.0).
    pub fn unlock_percentage(&self) -> f32 {
        let total = self.total_count();
        if total == 0 {
            return 0.0;
        }
        (self.unlocked_count() as f32 / total as f32) * 100.0
    }

    /// Get count of unlocked/total by category.
    pub fn count_by_category(&self, category: AchievementCategory) -> (usize, usize) {
        use super::data::ALL_ACHIEVEMENTS;

        let category_achievements: Vec<_> = ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| a.category == category)
            .collect();

        let unlocked = category_achievements
            .iter()
            .filter(|a| self.is_unlocked(a.id))
            .count();

        (unlocked, category_achievements.len())
    }

    // =========================================================================
    // Event Handlers (called from game logic)
    // =========================================================================

    /// Called when the Storm Leviathan is caught.
    /// Unlocks the StormLeviathan achievement (secret, required for Stormbreaker).
    pub fn on_storm_leviathan_caught(&mut self, character_name: Option<&str>) {
        self.unlock(
            AchievementId::StormLeviathan,
            character_name.map(|s| s.to_string()),
        );
    }

    /// Called when fishing rank changes.
    /// Unlocks FishermanI/II/III achievements at milestones.
    pub fn on_fishing_rank_up(&mut self, new_rank: u32, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        // Update highest rank
        if new_rank > self.highest_fishing_rank {
            self.highest_fishing_rank = new_rank;
        }

        // Check for milestone unlocks
        if new_rank >= 10 {
            self.unlock(AchievementId::FishermanI, char_name.clone());
        }
        if new_rank >= 20 {
            self.unlock(AchievementId::FishermanII, char_name.clone());
        }
        if new_rank >= 30 {
            self.unlock(AchievementId::FishermanIII, char_name);
        }
    }

    /// Called when a fish is caught.
    /// Unlocks GoneFishing on first catch.
    pub fn on_fish_caught(&mut self, character_name: Option<&str>) {
        self.total_fish_caught += 1;

        // First fish unlocks GoneFishing
        if self.total_fish_caught == 1 {
            self.unlock(
                AchievementId::GoneFishing,
                character_name.map(|s| s.to_string()),
            );
        }
    }

    // =========================================================================
    // Combat Event Handlers
    // =========================================================================

    /// Called when an enemy is killed.
    /// Unlocks FirstBlood, SlayerI/II/III at milestones.
    pub fn on_enemy_killed(&mut self, is_boss: bool, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        self.total_kills += 1;

        // First kill unlocks FirstBlood
        if self.total_kills == 1 {
            self.unlock(AchievementId::FirstBlood, char_name.clone());
        }

        // Track boss kills
        if is_boss {
            self.total_bosses_defeated += 1;

            // Boss hunter milestones
            if self.total_bosses_defeated >= 1 {
                self.unlock(AchievementId::BossHunterI, char_name.clone());
            }
            if self.total_bosses_defeated >= 10 {
                self.unlock(AchievementId::BossHunterII, char_name.clone());
            }
            if self.total_bosses_defeated >= 50 {
                self.unlock(AchievementId::BossHunterIII, char_name.clone());
            }
        }

        // Slayer milestones
        if self.total_kills >= 100 {
            self.unlock(AchievementId::SlayerI, char_name.clone());
        }
        if self.total_kills >= 1000 {
            self.unlock(AchievementId::SlayerII, char_name.clone());
        }
        if self.total_kills >= 10000 {
            self.unlock(AchievementId::SlayerIII, char_name);
        }

        // Update progress tracking for UI
        self.update_progress(AchievementId::SlayerI, self.total_kills.min(100), 100);
        self.update_progress(AchievementId::SlayerII, self.total_kills.min(1000), 1000);
        self.update_progress(AchievementId::SlayerIII, self.total_kills.min(10000), 10000);
        self.update_progress(
            AchievementId::BossHunterI,
            self.total_bosses_defeated.min(1),
            1,
        );
        self.update_progress(
            AchievementId::BossHunterII,
            self.total_bosses_defeated.min(10),
            10,
        );
        self.update_progress(
            AchievementId::BossHunterIII,
            self.total_bosses_defeated.min(50),
            50,
        );
    }

    // =========================================================================
    // Progression Event Handlers
    // =========================================================================

    /// Called when the character levels up.
    /// Unlocks Level10/50/100 achievements.
    pub fn on_level_up(&mut self, new_level: u32, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        // Update highest level
        if new_level > self.highest_level {
            self.highest_level = new_level;
        }

        // Level milestones
        if new_level >= 10 {
            self.unlock(AchievementId::Level10, char_name.clone());
        }
        if new_level >= 50 {
            self.unlock(AchievementId::Level50, char_name.clone());
        }
        if new_level >= 100 {
            self.unlock(AchievementId::Level100, char_name);
        }

        // Update progress tracking
        self.update_progress(AchievementId::Level10, new_level.min(10) as u64, 10);
        self.update_progress(AchievementId::Level50, new_level.min(50) as u64, 50);
        self.update_progress(AchievementId::Level100, new_level.min(100) as u64, 100);
    }

    /// Called when the character prestiges.
    /// Unlocks prestige milestone achievements.
    pub fn on_prestige(&mut self, new_rank: u32, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        // Update highest prestige
        if new_rank > self.highest_prestige_rank {
            self.highest_prestige_rank = new_rank;
        }

        // First prestige
        if new_rank >= 1 {
            self.unlock(AchievementId::FirstPrestige, char_name.clone());
        }

        // Prestige milestones
        if new_rank >= 5 {
            self.unlock(AchievementId::PrestigeV, char_name.clone());
        }
        if new_rank >= 10 {
            self.unlock(AchievementId::PrestigeX, char_name.clone());
        }
        if new_rank >= 15 {
            self.unlock(AchievementId::PrestigeXV, char_name.clone());
        }
        if new_rank >= 20 {
            self.unlock(AchievementId::PrestigeXX, char_name.clone());
        }

        // Eternal tier (P100+)
        if new_rank >= 100 {
            self.unlock(AchievementId::Eternal, char_name);
        }
    }

    /// Called when a zone is fully cleared (all subzones completed).
    pub fn on_zone_fully_cleared(&mut self, zone_id: u32, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        self.zones_fully_cleared += 1;

        // ZoneClearer - clear any zone
        self.unlock(AchievementId::ZoneClearer, char_name.clone());

        // Individual zone completion achievements
        let achievement = match zone_id {
            1 => Some(AchievementId::Zone1Complete),   // Meadow
            2 => Some(AchievementId::Zone2Complete),   // Dark Forest
            3 => Some(AchievementId::Zone3Complete),   // Mountain Pass
            4 => Some(AchievementId::Zone4Complete),   // Ancient Ruins
            5 => Some(AchievementId::Zone5Complete),   // Volcanic Wastes
            6 => Some(AchievementId::Zone6Complete),   // Frozen Tundra
            7 => Some(AchievementId::Zone7Complete),   // Crystal Caverns
            8 => Some(AchievementId::Zone8Complete),   // Sunken Kingdom
            9 => Some(AchievementId::Zone9Complete),   // Floating Isles
            10 => Some(AchievementId::Zone10Complete), // Storm Citadel
            11 => Some(AchievementId::Zone11Complete), // The Expanse (first cycle)
            _ => None,
        };

        if let Some(id) = achievement {
            self.unlock(id, char_name);
        }
    }

    // =========================================================================
    // Dungeon Event Handlers
    // =========================================================================

    /// Called when a dungeon is completed.
    /// Unlocks DungeonDiver, DungeonMasterI/II achievements.
    pub fn on_dungeon_completed(&mut self, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        self.total_dungeons_completed += 1;

        // First dungeon
        if self.total_dungeons_completed == 1 {
            self.unlock(AchievementId::DungeonDiver, char_name.clone());
        }

        // Dungeon master milestones
        if self.total_dungeons_completed >= 10 {
            self.unlock(AchievementId::DungeonMasterI, char_name.clone());
        }
        if self.total_dungeons_completed >= 50 {
            self.unlock(AchievementId::DungeonMasterII, char_name);
        }

        // Update progress tracking
        self.update_progress(
            AchievementId::DungeonMasterI,
            self.total_dungeons_completed.min(10),
            10,
        );
        self.update_progress(
            AchievementId::DungeonMasterII,
            self.total_dungeons_completed.min(50),
            50,
        );
    }

    // =========================================================================
    // Challenge/Minigame Event Handlers
    // =========================================================================

    /// Called when a minigame is won.
    /// game_type: "chess", "morris", "gomoku", "minesweeper", "rune", "go"
    /// difficulty: "novice", "apprentice", "journeyman", "master"
    pub fn on_minigame_won(
        &mut self,
        game_type: &str,
        difficulty: &str,
        character_name: Option<&str>,
    ) {
        let char_name = character_name.map(|s| s.to_string());

        self.total_minigame_wins += 1;

        // Game-specific achievements based on difficulty
        let achievement = match (game_type, difficulty) {
            // Chess
            ("chess", "novice") => Some(AchievementId::ChessNovice),
            ("chess", "apprentice") => Some(AchievementId::ChessApprentice),
            ("chess", "journeyman") => Some(AchievementId::ChessJourneyman),
            ("chess", "master") => Some(AchievementId::ChessMaster),
            // Morris
            ("morris", "novice") => Some(AchievementId::MorrisNovice),
            ("morris", "apprentice") => Some(AchievementId::MorrisApprentice),
            ("morris", "journeyman") => Some(AchievementId::MorrisJourneyman),
            ("morris", "master") => Some(AchievementId::MorrisMaster),
            // Gomoku
            ("gomoku", "novice") => Some(AchievementId::GomokuNovice),
            ("gomoku", "apprentice") => Some(AchievementId::GomokuApprentice),
            ("gomoku", "journeyman") => Some(AchievementId::GomokuJourneyman),
            ("gomoku", "master") => Some(AchievementId::GomokuMaster),
            // Minesweeper
            ("minesweeper", "novice") => Some(AchievementId::MinesweeperNovice),
            ("minesweeper", "apprentice") => Some(AchievementId::MinesweeperApprentice),
            ("minesweeper", "journeyman") => Some(AchievementId::MinesweeperJourneyman),
            ("minesweeper", "master") => Some(AchievementId::MinesweeperMaster),
            // Rune
            ("rune", "novice") => Some(AchievementId::RuneNovice),
            ("rune", "apprentice") => Some(AchievementId::RuneApprentice),
            ("rune", "journeyman") => Some(AchievementId::RuneJourneyman),
            ("rune", "master") => Some(AchievementId::RuneMaster),
            // Go
            ("go", "novice") => Some(AchievementId::GoNovice),
            ("go", "apprentice") => Some(AchievementId::GoApprentice),
            ("go", "journeyman") => Some(AchievementId::GoJourneyman),
            ("go", "master") => Some(AchievementId::GoMaster),
            _ => None,
        };

        if let Some(id) = achievement {
            self.unlock(id, char_name.clone());
        }

        // AllRounder - check if won at least one game of each type (any difficulty)
        let has_chess = self.is_unlocked(AchievementId::ChessNovice)
            || self.is_unlocked(AchievementId::ChessApprentice)
            || self.is_unlocked(AchievementId::ChessJourneyman)
            || self.is_unlocked(AchievementId::ChessMaster);
        let has_morris = self.is_unlocked(AchievementId::MorrisNovice)
            || self.is_unlocked(AchievementId::MorrisApprentice)
            || self.is_unlocked(AchievementId::MorrisJourneyman)
            || self.is_unlocked(AchievementId::MorrisMaster);
        let has_gomoku = self.is_unlocked(AchievementId::GomokuNovice)
            || self.is_unlocked(AchievementId::GomokuApprentice)
            || self.is_unlocked(AchievementId::GomokuJourneyman)
            || self.is_unlocked(AchievementId::GomokuMaster);
        let has_minesweeper = self.is_unlocked(AchievementId::MinesweeperNovice)
            || self.is_unlocked(AchievementId::MinesweeperApprentice)
            || self.is_unlocked(AchievementId::MinesweeperJourneyman)
            || self.is_unlocked(AchievementId::MinesweeperMaster);
        let has_rune = self.is_unlocked(AchievementId::RuneNovice)
            || self.is_unlocked(AchievementId::RuneApprentice)
            || self.is_unlocked(AchievementId::RuneJourneyman)
            || self.is_unlocked(AchievementId::RuneMaster);
        let has_go = self.is_unlocked(AchievementId::GoNovice)
            || self.is_unlocked(AchievementId::GoApprentice)
            || self.is_unlocked(AchievementId::GoJourneyman)
            || self.is_unlocked(AchievementId::GoMaster);

        if has_chess && has_morris && has_gomoku && has_minesweeper && has_rune && has_go {
            self.unlock(AchievementId::AllRounder, char_name.clone());
        }

        // Grand Champion - 100 total wins
        if self.total_minigame_wins >= 100 {
            self.unlock(AchievementId::GrandChampion, char_name);
        }

        // Update progress tracking
        self.update_progress(
            AchievementId::GrandChampion,
            self.total_minigame_wins.min(100),
            100,
        );
    }

    // =========================================================================
    // Haven Event Handlers
    // =========================================================================

    /// Called when Haven is first discovered.
    pub fn on_haven_discovered(&mut self, character_name: Option<&str>) {
        self.unlock(
            AchievementId::HavenDiscovered,
            character_name.map(|s| s.to_string()),
        );
    }

    /// Called when all Haven rooms are maxed.
    pub fn on_haven_architect(&mut self, character_name: Option<&str>) {
        self.unlock(
            AchievementId::HavenArchitect,
            character_name.map(|s| s.to_string()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_unlock() {
        let mut achievements = Achievements::default();

        assert!(!achievements.is_unlocked(AchievementId::FirstBlood));
        assert!(achievements.unlock(AchievementId::FirstBlood, Some("Hero".to_string())));
        assert!(achievements.is_unlocked(AchievementId::FirstBlood));

        // Second unlock should return false
        assert!(!achievements.unlock(AchievementId::FirstBlood, None));
    }

    #[test]
    fn test_achievement_progress() {
        let mut achievements = Achievements::default();

        achievements.update_progress(AchievementId::SlayerI, 50, 100);

        let progress = achievements.get_progress(AchievementId::SlayerI).unwrap();
        assert_eq!(progress.current, 50);
        assert_eq!(progress.target, 100);
    }

    #[test]
    fn test_category_names() {
        assert_eq!(AchievementCategory::Combat.name(), "Combat");
        assert_eq!(AchievementCategory::Progression.name(), "Progression");
        assert_eq!(AchievementCategory::Challenges.name(), "Challenges");
        assert_eq!(AchievementCategory::Exploration.name(), "Exploration");
    }
}
