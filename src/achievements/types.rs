//! Achievement system types and data structures.

#![allow(dead_code)] // Will be used when integrated with UI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Achievement categories for organization in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementCategory {
    Combat,
    Level,
    Progression,
    Challenges,
    Exploration,
}

impl AchievementCategory {
    /// All categories in display order.
    pub const ALL: [AchievementCategory; 5] = [
        AchievementCategory::Combat,
        AchievementCategory::Level,
        AchievementCategory::Progression,
        AchievementCategory::Challenges,
        AchievementCategory::Exploration,
    ];

    /// Display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            AchievementCategory::Combat => "Combat",
            AchievementCategory::Level => "Level",
            AchievementCategory::Progression => "Progression",
            AchievementCategory::Challenges => "Challenges",
            AchievementCategory::Exploration => "Exploration",
        }
    }
}

/// Unique identifier for each achievement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // Combat achievements - enemy kills
    SlayerI,    // 100 kills
    SlayerII,   // 500 kills
    SlayerIII,  // 1,000 kills
    SlayerIV,   // 5,000 kills
    SlayerV,    // 10,000 kills
    SlayerVI,   // 50,000 kills
    SlayerVII,  // 100,000 kills
    SlayerVIII, // 500,000 kills
    SlayerIX,   // 1,000,000 kills
    // Combat achievements - boss kills
    BossHunterI,    // 1 boss
    BossHunterII,   // 10 bosses
    BossHunterIII,  // 50 bosses
    BossHunterIV,   // 100 bosses
    BossHunterV,    // 500 bosses
    BossHunterVI,   // 1,000 bosses
    BossHunterVII,  // 5,000 bosses
    BossHunterVIII, // 10,000 bosses

    // Level achievements
    Level10,
    Level25,
    Level50,
    Level75,
    Level100,
    Level150,
    Level200,
    Level250,
    Level300,

    // Prestige achievements
    FirstPrestige,
    PrestigeV,
    PrestigeX,
    PrestigeXV,
    PrestigeXX,
    PrestigeXXV,
    PrestigeXXX,
    PrestigeXL,
    PrestigeL,
    PrestigeLXX,
    PrestigeXC,
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
    TheStormbreaker,
    StormsEnd,
    // The Expanse cycle achievements
    ExpanseCycleI,   // 1 cycle
    ExpanseCycleII,  // 100 cycles
    ExpanseCycleIII, // 1,000 cycles
    ExpanseCycleIV,  // 10,000 cycles

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

    // Fishing achievements - rank milestones
    GoneFishing,
    FishermanI,
    FishermanII,
    FishermanIII,
    StormLeviathan,
    // Fishing achievements - catch counts
    FishCatcherI,   // 100 fish
    FishCatcherII,  // 1,000 fish
    FishCatcherIII, // 10,000 fish
    FishCatcherIV,  // 100,000 fish

    // Dungeon achievements
    DungeonDiver,
    DungeonMasterI,
    DungeonMasterII,
    DungeonMasterIII, // 100 dungeons
    DungeonMasterIV,  // 1,000 dungeons
    DungeonMasterV,   // 5,000 dungeons
    DungeonMasterVI,  // 10,000 dungeons

    // Haven achievements
    HavenDiscovered,
    HavenBuilderI,  // All rooms at T1
    HavenBuilderII, // All rooms at T2
    HavenArchitect, // All rooms at T3
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
    pub expanse_cycles_completed: u64,
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
    /// Unlocks fish catching milestone achievements.
    pub fn on_fish_caught(&mut self, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        self.total_fish_caught += 1;

        // Fish catching milestones
        if self.total_fish_caught >= 1 {
            self.unlock(AchievementId::GoneFishing, char_name.clone());
        }
        if self.total_fish_caught >= 100 {
            self.unlock(AchievementId::FishCatcherI, char_name.clone());
        }
        if self.total_fish_caught >= 1000 {
            self.unlock(AchievementId::FishCatcherII, char_name.clone());
        }
        if self.total_fish_caught >= 10000 {
            self.unlock(AchievementId::FishCatcherIII, char_name.clone());
        }
        if self.total_fish_caught >= 100000 {
            self.unlock(AchievementId::FishCatcherIV, char_name);
        }
    }

    // =========================================================================
    // Combat Event Handlers
    // =========================================================================

    /// Called when an enemy is killed.
    /// Unlocks kill and boss milestone achievements.
    pub fn on_enemy_killed(&mut self, is_boss: bool, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        self.total_kills += 1;

        // Slayer milestones: 100, 500, 1K, 5K, 10K, 50K, 100K, 500K, 1M
        if self.total_kills >= 100 {
            self.unlock(AchievementId::SlayerI, char_name.clone());
        }
        if self.total_kills >= 500 {
            self.unlock(AchievementId::SlayerII, char_name.clone());
        }
        if self.total_kills >= 1000 {
            self.unlock(AchievementId::SlayerIII, char_name.clone());
        }
        if self.total_kills >= 5000 {
            self.unlock(AchievementId::SlayerIV, char_name.clone());
        }
        if self.total_kills >= 10000 {
            self.unlock(AchievementId::SlayerV, char_name.clone());
        }
        if self.total_kills >= 50000 {
            self.unlock(AchievementId::SlayerVI, char_name.clone());
        }
        if self.total_kills >= 100000 {
            self.unlock(AchievementId::SlayerVII, char_name.clone());
        }
        if self.total_kills >= 500000 {
            self.unlock(AchievementId::SlayerVIII, char_name.clone());
        }
        if self.total_kills >= 1000000 {
            self.unlock(AchievementId::SlayerIX, char_name.clone());
        }

        // Track boss kills
        if is_boss {
            self.total_bosses_defeated += 1;

            // Boss hunter milestones: 1, 10, 50, 100, 500, 1K, 5K, 10K
            if self.total_bosses_defeated >= 1 {
                self.unlock(AchievementId::BossHunterI, char_name.clone());
            }
            if self.total_bosses_defeated >= 10 {
                self.unlock(AchievementId::BossHunterII, char_name.clone());
            }
            if self.total_bosses_defeated >= 50 {
                self.unlock(AchievementId::BossHunterIII, char_name.clone());
            }
            if self.total_bosses_defeated >= 100 {
                self.unlock(AchievementId::BossHunterIV, char_name.clone());
            }
            if self.total_bosses_defeated >= 500 {
                self.unlock(AchievementId::BossHunterV, char_name.clone());
            }
            if self.total_bosses_defeated >= 1000 {
                self.unlock(AchievementId::BossHunterVI, char_name.clone());
            }
            if self.total_bosses_defeated >= 5000 {
                self.unlock(AchievementId::BossHunterVII, char_name.clone());
            }
            if self.total_bosses_defeated >= 10000 {
                self.unlock(AchievementId::BossHunterVIII, char_name);
            }
        }
    }

    // =========================================================================
    // Progression Event Handlers
    // =========================================================================

    /// Called when the character levels up.
    /// Unlocks level milestone achievements.
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
        if new_level >= 25 {
            self.unlock(AchievementId::Level25, char_name.clone());
        }
        if new_level >= 50 {
            self.unlock(AchievementId::Level50, char_name.clone());
        }
        if new_level >= 75 {
            self.unlock(AchievementId::Level75, char_name.clone());
        }
        if new_level >= 100 {
            self.unlock(AchievementId::Level100, char_name.clone());
        }
        if new_level >= 150 {
            self.unlock(AchievementId::Level150, char_name.clone());
        }
        if new_level >= 200 {
            self.unlock(AchievementId::Level200, char_name.clone());
        }
        if new_level >= 250 {
            self.unlock(AchievementId::Level250, char_name.clone());
        }
        if new_level >= 300 {
            self.unlock(AchievementId::Level300, char_name);
        }
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
        if new_rank >= 25 {
            self.unlock(AchievementId::PrestigeXXV, char_name.clone());
        }
        if new_rank >= 30 {
            self.unlock(AchievementId::PrestigeXXX, char_name.clone());
        }
        if new_rank >= 40 {
            self.unlock(AchievementId::PrestigeXL, char_name.clone());
        }
        if new_rank >= 50 {
            self.unlock(AchievementId::PrestigeL, char_name.clone());
        }
        if new_rank >= 70 {
            self.unlock(AchievementId::PrestigeLXX, char_name.clone());
        }
        if new_rank >= 90 {
            self.unlock(AchievementId::PrestigeXC, char_name.clone());
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

        // Zone 11 (The Expanse) has cycle-based achievements
        if zone_id == 11 {
            self.expanse_cycles_completed += 1;

            // Expanse cycle milestones: 1, 100, 1K, 10K
            if self.expanse_cycles_completed >= 1 {
                self.unlock(AchievementId::ExpanseCycleI, char_name.clone());
            }
            if self.expanse_cycles_completed >= 100 {
                self.unlock(AchievementId::ExpanseCycleII, char_name.clone());
            }
            if self.expanse_cycles_completed >= 1000 {
                self.unlock(AchievementId::ExpanseCycleIII, char_name.clone());
            }
            if self.expanse_cycles_completed >= 10000 {
                self.unlock(AchievementId::ExpanseCycleIV, char_name);
            }
            return;
        }

        // Individual zone completion achievements (zones 1-10)
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
            _ => None,
        };

        if let Some(id) = achievement {
            self.unlock(id, char_name);
        }
    }

    /// Called when the game is completed (Zone 10 boss defeated with Stormbreaker).
    pub fn on_storms_end(&mut self, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());
        self.unlock(AchievementId::StormsEnd, char_name);
    }

    // =========================================================================
    // Dungeon Event Handlers
    // =========================================================================

    /// Called when a dungeon is completed.
    /// Unlocks dungeon completion milestone achievements.
    pub fn on_dungeon_completed(&mut self, character_name: Option<&str>) {
        let char_name = character_name.map(|s| s.to_string());

        self.total_dungeons_completed += 1;

        // Dungeon completion milestones
        if self.total_dungeons_completed >= 1 {
            self.unlock(AchievementId::DungeonDiver, char_name.clone());
        }
        if self.total_dungeons_completed >= 10 {
            self.unlock(AchievementId::DungeonMasterI, char_name.clone());
        }
        if self.total_dungeons_completed >= 50 {
            self.unlock(AchievementId::DungeonMasterII, char_name.clone());
        }
        if self.total_dungeons_completed >= 100 {
            self.unlock(AchievementId::DungeonMasterIII, char_name.clone());
        }
        if self.total_dungeons_completed >= 1000 {
            self.unlock(AchievementId::DungeonMasterIV, char_name.clone());
        }
        if self.total_dungeons_completed >= 5000 {
            self.unlock(AchievementId::DungeonMasterV, char_name.clone());
        }
        if self.total_dungeons_completed >= 10000 {
            self.unlock(AchievementId::DungeonMasterVI, char_name);
        }
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

    /// Called when all Haven rooms reach Tier 1.
    pub fn on_haven_all_t1(&mut self, character_name: Option<&str>) {
        self.unlock(
            AchievementId::HavenBuilderI,
            character_name.map(|s| s.to_string()),
        );
    }

    /// Called when all Haven rooms reach Tier 2.
    pub fn on_haven_all_t2(&mut self, character_name: Option<&str>) {
        self.unlock(
            AchievementId::HavenBuilderII,
            character_name.map(|s| s.to_string()),
        );
    }

    /// Called when all Haven rooms reach Tier 3.
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

        assert!(!achievements.is_unlocked(AchievementId::SlayerI));
        assert!(achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string())));
        assert!(achievements.is_unlocked(AchievementId::SlayerI));

        // Second unlock should return false
        assert!(!achievements.unlock(AchievementId::SlayerI, None));
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
        assert_eq!(AchievementCategory::Level.name(), "Level");
        assert_eq!(AchievementCategory::Progression.name(), "Progression");
        assert_eq!(AchievementCategory::Challenges.name(), "Challenges");
        assert_eq!(AchievementCategory::Exploration.name(), "Exploration");
    }

    // =========================================================================
    // Slayer Achievement Tests
    // =========================================================================

    #[test]
    fn test_slayer_achievements_milestones() {
        let mut achievements = Achievements::default();

        // Kill 99 enemies - no slayer achievement yet
        for _ in 0..99 {
            achievements.on_enemy_killed(false, Some("Hero"));
        }
        assert!(!achievements.is_unlocked(AchievementId::SlayerI));

        // 100th kill unlocks SlayerI
        achievements.on_enemy_killed(false, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::SlayerI));
        assert!(!achievements.is_unlocked(AchievementId::SlayerII));

        // Reach 500 kills for SlayerII
        for _ in 0..400 {
            achievements.on_enemy_killed(false, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::SlayerII));
        assert!(!achievements.is_unlocked(AchievementId::SlayerIII));

        // Reach 1000 kills for SlayerIII
        for _ in 0..500 {
            achievements.on_enemy_killed(false, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::SlayerIII));
    }

    #[test]
    fn test_slayer_all_milestones() {
        let mut achievements = Achievements::default();

        // Set total_kills directly to test all milestones
        let milestones = [
            (100, AchievementId::SlayerI),
            (500, AchievementId::SlayerII),
            (1000, AchievementId::SlayerIII),
            (5000, AchievementId::SlayerIV),
            (10000, AchievementId::SlayerV),
            (50000, AchievementId::SlayerVI),
            (100000, AchievementId::SlayerVII),
            (500000, AchievementId::SlayerVIII),
            (1000000, AchievementId::SlayerIX),
        ];

        for (kills, achievement_id) in milestones {
            achievements.total_kills = kills - 1;
            achievements.on_enemy_killed(false, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} kills",
                achievement_id,
                kills
            );
        }
    }

    // =========================================================================
    // Boss Hunter Achievement Tests
    // =========================================================================

    #[test]
    fn test_boss_hunter_achievements_milestones() {
        let mut achievements = Achievements::default();

        // First boss unlocks BossHunterI
        assert!(!achievements.is_unlocked(AchievementId::BossHunterI));
        achievements.on_enemy_killed(true, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::BossHunterI));
        assert!(!achievements.is_unlocked(AchievementId::BossHunterII));

        // 9 more bosses (10 total) unlocks BossHunterII
        for _ in 0..9 {
            achievements.on_enemy_killed(true, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::BossHunterII));
        assert!(!achievements.is_unlocked(AchievementId::BossHunterIII));

        // 40 more bosses (50 total) unlocks BossHunterIII
        for _ in 0..40 {
            achievements.on_enemy_killed(true, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::BossHunterIII));
    }

    #[test]
    fn test_boss_hunter_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::BossHunterI),
            (10, AchievementId::BossHunterII),
            (50, AchievementId::BossHunterIII),
            (100, AchievementId::BossHunterIV),
            (500, AchievementId::BossHunterV),
            (1000, AchievementId::BossHunterVI),
            (5000, AchievementId::BossHunterVII),
            (10000, AchievementId::BossHunterVIII),
        ];

        for (bosses, achievement_id) in milestones {
            achievements.total_bosses_defeated = bosses - 1;
            achievements.on_enemy_killed(true, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} bosses",
                achievement_id,
                bosses
            );
        }
    }

    // =========================================================================
    // Expanse Cycle Achievement Tests
    // =========================================================================

    #[test]
    fn test_expanse_cycle_first_completion() {
        let mut achievements = Achievements::default();

        assert!(!achievements.is_unlocked(AchievementId::ExpanseCycleI));
        assert_eq!(achievements.expanse_cycles_completed, 0);

        // Complete first cycle of The Expanse (zone 11)
        achievements.on_zone_fully_cleared(11, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::ExpanseCycleI));
        assert_eq!(achievements.expanse_cycles_completed, 1);
        assert!(!achievements.is_unlocked(AchievementId::ExpanseCycleII));
    }

    #[test]
    fn test_expanse_cycle_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::ExpanseCycleI),
            (100, AchievementId::ExpanseCycleII),
            (1000, AchievementId::ExpanseCycleIII),
            (10000, AchievementId::ExpanseCycleIV),
        ];

        for (cycles, achievement_id) in milestones {
            achievements.expanse_cycles_completed = cycles - 1;
            achievements.on_zone_fully_cleared(11, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} cycles",
                achievement_id,
                cycles
            );
        }
    }

    #[test]
    fn test_expanse_does_not_affect_other_zones() {
        let mut achievements = Achievements::default();

        // Completing zone 11 should not unlock zone completion achievements for zones 1-10
        achievements.on_zone_fully_cleared(11, Some("Hero"));

        assert!(!achievements.is_unlocked(AchievementId::Zone1Complete));
        assert!(!achievements.is_unlocked(AchievementId::Zone10Complete));
        assert!(achievements.is_unlocked(AchievementId::ExpanseCycleI));
    }

    // =========================================================================
    // Zone Completion Achievement Tests
    // =========================================================================

    #[test]
    fn test_zone_completion_achievements() {
        let mut achievements = Achievements::default();

        let zones = [
            (1, AchievementId::Zone1Complete),
            (2, AchievementId::Zone2Complete),
            (3, AchievementId::Zone3Complete),
            (4, AchievementId::Zone4Complete),
            (5, AchievementId::Zone5Complete),
            (6, AchievementId::Zone6Complete),
            (7, AchievementId::Zone7Complete),
            (8, AchievementId::Zone8Complete),
            (9, AchievementId::Zone9Complete),
            (10, AchievementId::Zone10Complete),
        ];

        for (zone_id, achievement_id) in zones {
            assert!(
                !achievements.is_unlocked(achievement_id),
                "Zone {} should not be unlocked initially",
                zone_id
            );
            achievements.on_zone_fully_cleared(zone_id, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Zone {} should be unlocked after clearing",
                zone_id
            );
        }
    }

    // =========================================================================
    // Fish Catcher Achievement Tests
    // =========================================================================

    #[test]
    fn test_fish_catcher_achievements() {
        let mut achievements = Achievements::default();

        // First fish unlocks GoneFishing
        achievements.on_fish_caught(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::GoneFishing));
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherI));

        // 99 more fish (100 total) unlocks FishCatcherI
        for _ in 0..99 {
            achievements.on_fish_caught(Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::FishCatcherI));
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherII));
    }

    #[test]
    fn test_fish_catcher_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::GoneFishing),
            (100, AchievementId::FishCatcherI),
            (1000, AchievementId::FishCatcherII),
            (10000, AchievementId::FishCatcherIII),
            (100000, AchievementId::FishCatcherIV),
        ];

        for (fish, achievement_id) in milestones {
            achievements.total_fish_caught = fish - 1;
            achievements.on_fish_caught(Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} fish",
                achievement_id,
                fish
            );
        }
    }

    // =========================================================================
    // Dungeon Master Achievement Tests
    // =========================================================================

    #[test]
    fn test_dungeon_master_achievements() {
        let mut achievements = Achievements::default();

        // First dungeon unlocks DungeonDiver
        achievements.on_dungeon_completed(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::DungeonDiver));
        assert!(!achievements.is_unlocked(AchievementId::DungeonMasterI));

        // 9 more dungeons (10 total) unlocks DungeonMasterI
        for _ in 0..9 {
            achievements.on_dungeon_completed(Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::DungeonMasterI));
        assert!(!achievements.is_unlocked(AchievementId::DungeonMasterII));
    }

    #[test]
    fn test_dungeon_master_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::DungeonDiver),
            (10, AchievementId::DungeonMasterI),
            (50, AchievementId::DungeonMasterII),
            (100, AchievementId::DungeonMasterIII),
            (1000, AchievementId::DungeonMasterIV),
            (5000, AchievementId::DungeonMasterV),
            (10000, AchievementId::DungeonMasterVI),
        ];

        for (dungeons, achievement_id) in milestones {
            achievements.total_dungeons_completed = dungeons - 1;
            achievements.on_dungeon_completed(Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} dungeons",
                achievement_id,
                dungeons
            );
        }
    }

    // =========================================================================
    // Haven Achievement Tests
    // =========================================================================

    #[test]
    fn test_haven_achievements() {
        let mut achievements = Achievements::default();

        // Haven discovered
        assert!(!achievements.is_unlocked(AchievementId::HavenDiscovered));
        achievements.on_haven_discovered(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));

        // Haven builder tiers
        assert!(!achievements.is_unlocked(AchievementId::HavenBuilderI));
        achievements.on_haven_all_t1(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));

        assert!(!achievements.is_unlocked(AchievementId::HavenBuilderII));
        achievements.on_haven_all_t2(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));

        assert!(!achievements.is_unlocked(AchievementId::HavenArchitect));
        achievements.on_haven_architect(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenArchitect));
    }

    // =========================================================================
    // Level Achievement Tests
    // =========================================================================

    #[test]
    fn test_level_achievements() {
        let mut achievements = Achievements::default();

        let milestones = [
            (10, AchievementId::Level10),
            (25, AchievementId::Level25),
            (50, AchievementId::Level50),
            (75, AchievementId::Level75),
            (100, AchievementId::Level100),
            (150, AchievementId::Level150),
            (200, AchievementId::Level200),
            (250, AchievementId::Level250),
            (300, AchievementId::Level300),
        ];

        for (level, achievement_id) in milestones {
            achievements.on_level_up(level, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at level {}",
                achievement_id,
                level
            );
        }
    }

    // =========================================================================
    // Prestige Achievement Tests
    // =========================================================================

    #[test]
    fn test_prestige_achievements() {
        let mut achievements = Achievements::default();

        let milestones = [
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

        for (rank, achievement_id) in milestones {
            achievements.on_prestige(rank, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at prestige rank {}",
                achievement_id,
                rank
            );
        }
    }

    // =========================================================================
    // Storms End Achievement Test
    // =========================================================================

    #[test]
    fn test_storms_end_achievement() {
        let mut achievements = Achievements::default();

        assert!(!achievements.is_unlocked(AchievementId::StormsEnd));
        achievements.on_storms_end(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::StormsEnd));
    }
}
