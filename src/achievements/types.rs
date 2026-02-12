//! Achievement system types and data structures.

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
    Level100,
    Level150,
    Level200,
    Level250,
    Level500,
    Level750,
    Level1000,
    Level1500,

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
    // Challenge achievements - Flappy Bird
    FlappyNovice,
    FlappyApprentice,
    FlappyJourneyman,
    FlappyMaster,
    // Challenge achievements - Meta
    GrandChampion,

    // Fishing achievements - rank milestones
    GoneFishing,
    FishermanI,
    FishermanII,
    FishermanIII,
    FishermanIV,
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

    /// Achievements unlocked but not yet viewed (not persisted) - for UI indicator
    #[serde(skip)]
    pub pending_notifications: Vec<AchievementId>,

    /// Achievements unlocked this tick that need to be logged (not persisted)
    #[serde(skip)]
    pub newly_unlocked: Vec<AchievementId>,

    /// Achievements waiting to be shown in modal (accumulation window)
    #[serde(skip)]
    pub modal_queue: Vec<AchievementId>,

    /// When the accumulation window started (first achievement unlocked)
    #[serde(skip)]
    pub accumulation_start: Option<std::time::Instant>,
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
        self.pending_notifications.push(id);
        self.newly_unlocked.push(id);

        // Add to modal queue and start accumulation timer if not already started
        self.modal_queue.push(id);
        if self.accumulation_start.is_none() {
            self.accumulation_start = Some(std::time::Instant::now());
        }

        true
    }

    /// Get the count of pending achievement notifications.
    pub fn pending_count(&self) -> usize {
        self.pending_notifications.len()
    }

    /// Clear pending notifications (call when user views achievements).
    pub fn clear_pending_notifications(&mut self) {
        self.pending_notifications.clear();
    }

    /// Take newly unlocked achievements for logging (clears the list).
    pub fn take_newly_unlocked(&mut self) -> Vec<AchievementId> {
        std::mem::take(&mut self.newly_unlocked)
    }

    /// Check if the achievement modal is ready to show.
    /// Returns true if there are queued achievements and 500ms has elapsed.
    pub fn is_modal_ready(&self) -> bool {
        if self.modal_queue.is_empty() {
            return false;
        }
        if let Some(start) = self.accumulation_start {
            start.elapsed() >= std::time::Duration::from_millis(500)
        } else {
            false
        }
    }

    /// Take the modal queue for display (clears queue and resets timer).
    pub fn take_modal_queue(&mut self) -> Vec<AchievementId> {
        self.accumulation_start = None;
        std::mem::take(&mut self.modal_queue)
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

    /// Convenience wrapper: unlock with an `Option<&str>` name (avoids repeated `.map(|s| s.to_string())`).
    fn unlock_with_name(&mut self, id: AchievementId, character_name: Option<&str>) -> bool {
        self.unlock(id, character_name.map(|s| s.to_string()))
    }

    /// Helper to check and unlock milestones. Checks all milestones in order.
    fn check_milestones(
        &mut self,
        current: u64,
        milestones: &[(u64, AchievementId)],
        character_name: Option<&str>,
    ) {
        for &(threshold, achievement_id) in milestones {
            if current >= threshold {
                self.unlock_with_name(achievement_id, character_name);
            }
        }
    }

    /// Get count of unlocked/total by category.
    pub fn count_by_category(&self, category: AchievementCategory) -> (usize, usize) {
        use super::data::ALL_ACHIEVEMENTS;
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| a.category == category)
            .fold((0, 0), |(unlocked, total), a| {
                (unlocked + self.is_unlocked(a.id) as usize, total + 1)
            })
    }

    // =========================================================================
    // Event Handlers (called from game logic)
    // =========================================================================

    /// Called when the Storm Leviathan is caught.
    /// Unlocks the StormLeviathan achievement (required for Stormbreaker).
    pub fn on_storm_leviathan_caught(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::StormLeviathan, character_name);
    }

    /// Called when fishing rank changes.
    /// Unlocks FishermanI/II/III achievements at milestones.
    pub fn on_fishing_rank_up(&mut self, new_rank: u32, character_name: Option<&str>) {
        // Update highest rank
        if new_rank > self.highest_fishing_rank {
            self.highest_fishing_rank = new_rank;
        }

        self.check_milestones(
            new_rank as u64,
            &[
                (10, AchievementId::FishermanI),
                (20, AchievementId::FishermanII),
                (30, AchievementId::FishermanIII),
                (40, AchievementId::FishermanIV),
            ],
            character_name,
        );
    }

    /// Called when a fish is caught.
    /// Unlocks fish catching milestone achievements.
    pub fn on_fish_caught(&mut self, character_name: Option<&str>) {
        self.total_fish_caught += 1;

        self.check_milestones(
            self.total_fish_caught,
            &[
                (1, AchievementId::GoneFishing),
                (100, AchievementId::FishCatcherI),
                (1000, AchievementId::FishCatcherII),
                (10000, AchievementId::FishCatcherIII),
                (100000, AchievementId::FishCatcherIV),
            ],
            character_name,
        );
    }

    // =========================================================================
    // Combat Event Handlers
    // =========================================================================

    /// Called when an enemy is killed.
    /// Unlocks kill and boss milestone achievements.
    pub fn on_enemy_killed(&mut self, is_boss: bool, character_name: Option<&str>) {
        self.total_kills += 1;

        self.check_milestones(
            self.total_kills,
            &[
                (100, AchievementId::SlayerI),
                (500, AchievementId::SlayerII),
                (1000, AchievementId::SlayerIII),
                (5000, AchievementId::SlayerIV),
                (10000, AchievementId::SlayerV),
                (50000, AchievementId::SlayerVI),
                (100000, AchievementId::SlayerVII),
                (500000, AchievementId::SlayerVIII),
                (1000000, AchievementId::SlayerIX),
            ],
            character_name,
        );

        if is_boss {
            self.total_bosses_defeated += 1;

            self.check_milestones(
                self.total_bosses_defeated,
                &[
                    (1, AchievementId::BossHunterI),
                    (10, AchievementId::BossHunterII),
                    (50, AchievementId::BossHunterIII),
                    (100, AchievementId::BossHunterIV),
                    (500, AchievementId::BossHunterV),
                    (1000, AchievementId::BossHunterVI),
                    (5000, AchievementId::BossHunterVII),
                    (10000, AchievementId::BossHunterVIII),
                ],
                character_name,
            );
        }
    }

    // =========================================================================
    // Progression Event Handlers
    // =========================================================================

    /// Called when the character levels up.
    /// Unlocks level milestone achievements.
    pub fn on_level_up(&mut self, new_level: u32, character_name: Option<&str>) {
        if new_level > self.highest_level {
            self.highest_level = new_level;
        }

        self.check_milestones(
            new_level as u64,
            &[
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
            ],
            character_name,
        );
    }

    /// Called when the character prestiges.
    /// Unlocks prestige milestone achievements.
    pub fn on_prestige(&mut self, new_rank: u32, character_name: Option<&str>) {
        if new_rank > self.highest_prestige_rank {
            self.highest_prestige_rank = new_rank;
        }

        self.check_milestones(
            new_rank as u64,
            &[
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
            ],
            character_name,
        );
    }

    /// Called when a zone is fully cleared (all subzones completed).
    pub fn on_zone_fully_cleared(&mut self, zone_id: u32, character_name: Option<&str>) {
        self.zones_fully_cleared += 1;

        // Zone 11 (The Expanse) has cycle-based achievements
        if zone_id == 11 {
            self.expanse_cycles_completed += 1;
            self.check_milestones(
                self.expanse_cycles_completed,
                &[
                    (1, AchievementId::ExpanseCycleI),
                    (100, AchievementId::ExpanseCycleII),
                    (1000, AchievementId::ExpanseCycleIII),
                    (10000, AchievementId::ExpanseCycleIV),
                ],
                character_name,
            );
            return;
        }

        // Individual zone completion achievements (zones 1-10)
        let achievement = match zone_id {
            1 => Some(AchievementId::Zone1Complete),
            2 => Some(AchievementId::Zone2Complete),
            3 => Some(AchievementId::Zone3Complete),
            4 => Some(AchievementId::Zone4Complete),
            5 => Some(AchievementId::Zone5Complete),
            6 => Some(AchievementId::Zone6Complete),
            7 => Some(AchievementId::Zone7Complete),
            8 => Some(AchievementId::Zone8Complete),
            9 => Some(AchievementId::Zone9Complete),
            10 => Some(AchievementId::Zone10Complete),
            _ => None,
        };

        if let Some(id) = achievement {
            self.unlock_with_name(id, character_name);
        }
    }

    /// Called when the game is completed (Zone 10 boss defeated with Stormbreaker).
    pub fn on_storms_end(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::StormsEnd, character_name);
    }

    // =========================================================================
    // Dungeon Event Handlers
    // =========================================================================

    /// Called when a dungeon is completed.
    /// Unlocks dungeon completion milestone achievements.
    pub fn on_dungeon_completed(&mut self, character_name: Option<&str>) {
        self.total_dungeons_completed += 1;

        self.check_milestones(
            self.total_dungeons_completed,
            &[
                (1, AchievementId::DungeonDiver),
                (10, AchievementId::DungeonMasterI),
                (50, AchievementId::DungeonMasterII),
                (100, AchievementId::DungeonMasterIII),
                (1000, AchievementId::DungeonMasterIV),
                (5000, AchievementId::DungeonMasterV),
                (10000, AchievementId::DungeonMasterVI),
            ],
            character_name,
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
        self.total_minigame_wins += 1;

        // Game-specific achievements based on difficulty
        let achievement = match (game_type, difficulty) {
            ("chess", "novice") => Some(AchievementId::ChessNovice),
            ("chess", "apprentice") => Some(AchievementId::ChessApprentice),
            ("chess", "journeyman") => Some(AchievementId::ChessJourneyman),
            ("chess", "master") => Some(AchievementId::ChessMaster),
            ("morris", "novice") => Some(AchievementId::MorrisNovice),
            ("morris", "apprentice") => Some(AchievementId::MorrisApprentice),
            ("morris", "journeyman") => Some(AchievementId::MorrisJourneyman),
            ("morris", "master") => Some(AchievementId::MorrisMaster),
            ("gomoku", "novice") => Some(AchievementId::GomokuNovice),
            ("gomoku", "apprentice") => Some(AchievementId::GomokuApprentice),
            ("gomoku", "journeyman") => Some(AchievementId::GomokuJourneyman),
            ("gomoku", "master") => Some(AchievementId::GomokuMaster),
            ("minesweeper", "novice") => Some(AchievementId::MinesweeperNovice),
            ("minesweeper", "apprentice") => Some(AchievementId::MinesweeperApprentice),
            ("minesweeper", "journeyman") => Some(AchievementId::MinesweeperJourneyman),
            ("minesweeper", "master") => Some(AchievementId::MinesweeperMaster),
            ("rune", "novice") => Some(AchievementId::RuneNovice),
            ("rune", "apprentice") => Some(AchievementId::RuneApprentice),
            ("rune", "journeyman") => Some(AchievementId::RuneJourneyman),
            ("rune", "master") => Some(AchievementId::RuneMaster),
            ("go", "novice") => Some(AchievementId::GoNovice),
            ("go", "apprentice") => Some(AchievementId::GoApprentice),
            ("go", "journeyman") => Some(AchievementId::GoJourneyman),
            ("go", "master") => Some(AchievementId::GoMaster),
            ("flappy_bird", "novice") => Some(AchievementId::FlappyNovice),
            ("flappy_bird", "apprentice") => Some(AchievementId::FlappyApprentice),
            ("flappy_bird", "journeyman") => Some(AchievementId::FlappyJourneyman),
            ("flappy_bird", "master") => Some(AchievementId::FlappyMaster),
            _ => None,
        };

        if let Some(id) = achievement {
            self.unlock_with_name(id, character_name);
        }

        // Grand Champion - 100 total wins
        if self.total_minigame_wins >= 100 {
            self.unlock_with_name(AchievementId::GrandChampion, character_name);
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
        self.unlock_with_name(AchievementId::HavenDiscovered, character_name);
    }

    /// Called when all Haven rooms reach Tier 1.
    pub fn on_haven_all_t1(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::HavenBuilderI, character_name);
    }

    /// Called when all Haven rooms reach Tier 2.
    pub fn on_haven_all_t2(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::HavenBuilderII, character_name);
    }

    /// Called when all Haven rooms reach Tier 3.
    pub fn on_haven_architect(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::HavenArchitect, character_name);
    }

    // =========================================================================
    // State Synchronization (retroactive achievement unlocking)
    // =========================================================================

    /// Syncs achievements with current game state.
    /// Call this when loading a character to retroactively unlock achievements
    /// for milestones already reached.
    ///
    /// This handles the case where a player loads an existing character
    /// (e.g., level 120, prestige 17) and should have achievements for
    /// milestones they've already passed.
    ///
    /// Note: Some achievements cannot be synced because their counters
    /// aren't persisted (e.g., total kills, total bosses, total dungeons).
    /// Those counters start from where they are in the achievements file.
    pub fn sync_from_game_state(
        &mut self,
        level: u32,
        prestige_rank: u32,
        fishing_rank: u32,
        total_fish_caught: u32,
        defeated_bosses: &[(u32, u32)],
        character_name: Option<&str>,
    ) {
        // Sync level achievements
        self.on_level_up(level, character_name);

        // Sync prestige achievements
        if prestige_rank >= 1 {
            self.on_prestige(prestige_rank, character_name);
        }

        // Sync fishing rank achievements
        if fishing_rank >= 1 {
            self.on_fishing_rank_up(fishing_rank, character_name);
        }

        // Sync fish catch count
        // Use the max of save file count vs existing achievement counter
        let effective_fish = (total_fish_caught as u64).max(self.total_fish_caught);
        if effective_fish > 0 {
            // Set the counter to one less and then trigger a catch to unlock achievements
            self.total_fish_caught = effective_fish.saturating_sub(1);
            self.on_fish_caught(character_name);
        }

        // Sync zone completions based on defeated bosses
        self.sync_zone_completions(defeated_bosses, character_name);
    }

    /// Syncs zone completion achievements based on defeated bosses.
    fn sync_zone_completions(
        &mut self,
        defeated_bosses: &[(u32, u32)],
        character_name: Option<&str>,
    ) {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();

        for zone in zones.iter() {
            // Check if all subzones in this zone have been completed
            let total_subzones = zone.subzones.len() as u32;
            let completed_subzones = (1..=total_subzones)
                .filter(|&subzone_id| defeated_bosses.contains(&(zone.id, subzone_id)))
                .count() as u32;

            // If all subzones are complete, unlock the zone achievement
            if completed_subzones == total_subzones {
                self.on_zone_fully_cleared(zone.id, character_name);
            }
        }
    }

    /// Syncs Haven-related achievements based on Haven state.
    /// Call this when loading Haven data.
    pub fn sync_from_haven(
        &mut self,
        discovered: bool,
        room_tiers: &std::collections::HashMap<crate::haven::types::HavenRoomId, u8>,
        character_name: Option<&str>,
    ) {
        use crate::haven::types::HavenRoomId;

        if discovered {
            self.on_haven_discovered(character_name);
        }

        // Count rooms at each tier level
        // Note: StormForge max tier is 1, FishingDock max tier is 4, others are 3
        let buildable_rooms: Vec<_> = HavenRoomId::ALL
            .iter()
            .filter(|r| **r != HavenRoomId::StormForge) // StormForge only has T1
            .collect();

        let all_at_t1 = buildable_rooms
            .iter()
            .all(|room| room_tiers.get(room).copied().unwrap_or(0) >= 1);
        let all_at_t2 = buildable_rooms
            .iter()
            .all(|room| room_tiers.get(room).copied().unwrap_or(0) >= 2);
        let all_at_t3 = buildable_rooms.iter().all(|room| {
            let tier = room_tiers.get(room).copied().unwrap_or(0);
            let max_tier = room.max_tier();
            // For rooms with max tier < 3, being at max tier counts as "T3"
            tier >= 3 || tier >= max_tier
        });

        if all_at_t1 {
            self.on_haven_all_t1(character_name);
        }
        if all_at_t2 {
            self.on_haven_all_t2(character_name);
        }
        if all_at_t3 {
            self.on_haven_architect(character_name);
        }
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
            (100, AchievementId::Level100),
            (150, AchievementId::Level150),
            (200, AchievementId::Level200),
            (250, AchievementId::Level250),
            (500, AchievementId::Level500),
            (750, AchievementId::Level750),
            (1000, AchievementId::Level1000),
            (1500, AchievementId::Level1500),
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

    // =========================================================================
    // State Synchronization Tests
    // =========================================================================

    #[test]
    fn test_sync_from_game_state_level_achievements() {
        let mut achievements = Achievements::default();

        // Sync with level 120 character
        achievements.sync_from_game_state(120, 0, 1, 0, &[], Some("Hero"));

        // Should have all level achievements up to 100
        assert!(achievements.is_unlocked(AchievementId::Level10));
        assert!(achievements.is_unlocked(AchievementId::Level25));
        assert!(achievements.is_unlocked(AchievementId::Level50));
        assert!(achievements.is_unlocked(AchievementId::Level100));
        // But not 150+
        assert!(!achievements.is_unlocked(AchievementId::Level150));
    }

    #[test]
    fn test_sync_from_game_state_prestige_achievements() {
        let mut achievements = Achievements::default();

        // Sync with prestige 17 character
        achievements.sync_from_game_state(1, 17, 1, 0, &[], Some("Hero"));

        // Should have prestige achievements up to P15
        assert!(achievements.is_unlocked(AchievementId::FirstPrestige));
        assert!(achievements.is_unlocked(AchievementId::PrestigeV));
        assert!(achievements.is_unlocked(AchievementId::PrestigeX));
        assert!(achievements.is_unlocked(AchievementId::PrestigeXV));
        // But not P20+
        assert!(!achievements.is_unlocked(AchievementId::PrestigeXX));
    }

    #[test]
    fn test_sync_from_game_state_fishing_achievements() {
        let mut achievements = Achievements::default();

        // Sync with fishing rank 15
        achievements.sync_from_game_state(1, 0, 15, 500, &[], Some("Hero"));

        // Should have FishermanI (rank 10)
        assert!(achievements.is_unlocked(AchievementId::FishermanI));
        // But not FishermanII (rank 20)
        assert!(!achievements.is_unlocked(AchievementId::FishermanII));

        // Should have fish catch achievements
        assert!(achievements.is_unlocked(AchievementId::GoneFishing));
        assert!(achievements.is_unlocked(AchievementId::FishCatcherI)); // 100 fish
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherII)); // 1000 fish
    }

    #[test]
    fn test_sync_from_game_state_zone_completions() {
        let mut achievements = Achievements::default();

        // Zone 1 has 3 subzones, Zone 2 has 3 subzones
        let defeated_bosses = vec![
            (1, 1),
            (1, 2),
            (1, 3), // Zone 1 complete
            (2, 1),
            (2, 2), // Zone 2 incomplete (missing subzone 3)
        ];

        achievements.sync_from_game_state(1, 0, 1, 0, &defeated_bosses, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::Zone1Complete));
        assert!(!achievements.is_unlocked(AchievementId::Zone2Complete));
    }

    #[test]
    fn test_sync_from_game_state_full_progression() {
        let mut achievements = Achievements::default();

        // Simulate a well-progressed character
        let defeated_bosses = vec![
            // Zone 1-4 complete
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 1),
            (2, 2),
            (2, 3),
            (3, 1),
            (3, 2),
            (3, 3),
            (4, 1),
            (4, 2),
            (4, 3),
        ];

        achievements.sync_from_game_state(
            150,  // level
            25,   // prestige
            20,   // fishing rank
            5000, // fish caught
            &defeated_bosses,
            Some("Veteran"),
        );

        // Level achievements
        assert!(achievements.is_unlocked(AchievementId::Level100));
        assert!(achievements.is_unlocked(AchievementId::Level150));
        assert!(!achievements.is_unlocked(AchievementId::Level200));

        // Prestige achievements
        assert!(achievements.is_unlocked(AchievementId::PrestigeXX));
        assert!(achievements.is_unlocked(AchievementId::PrestigeXXV));
        assert!(!achievements.is_unlocked(AchievementId::PrestigeXXX));

        // Fishing achievements
        assert!(achievements.is_unlocked(AchievementId::FishermanI));
        assert!(achievements.is_unlocked(AchievementId::FishermanII));
        assert!(!achievements.is_unlocked(AchievementId::FishermanIII)); // needs rank 30
        assert!(!achievements.is_unlocked(AchievementId::FishermanIV)); // needs rank 40

        // Fish catch achievements (5000 fish)
        assert!(achievements.is_unlocked(AchievementId::FishCatcherI)); // 100
        assert!(achievements.is_unlocked(AchievementId::FishCatcherII)); // 1000
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherIII)); // 10000 - not reached

        // Zone completions
        assert!(achievements.is_unlocked(AchievementId::Zone1Complete));
        assert!(achievements.is_unlocked(AchievementId::Zone2Complete));
        assert!(achievements.is_unlocked(AchievementId::Zone3Complete));
        assert!(achievements.is_unlocked(AchievementId::Zone4Complete));
        assert!(!achievements.is_unlocked(AchievementId::Zone5Complete));
    }

    #[test]
    fn test_sync_does_not_overwrite_higher_counters() {
        // Pre-set a higher fish count in achievements
        let mut achievements = Achievements {
            total_fish_caught: 50000,
            ..Default::default()
        };

        // Sync with a lower fish count from save
        achievements.sync_from_game_state(1, 0, 1, 1000, &[], Some("Hero"));

        // Should NOT have decreased the counter
        assert_eq!(achievements.total_fish_caught, 50000);
        // Should still have the high-count achievements
        assert!(achievements.is_unlocked(AchievementId::FishCatcherIII)); // 10000
    }

    // =========================================================================
    // Storm Leviathan Achievement Tests
    // =========================================================================

    #[test]
    fn test_storm_leviathan_unlocking() {
        let mut achievements = Achievements::default();

        // Storm Leviathan should not be unlocked initially
        assert!(!achievements.is_unlocked(AchievementId::StormLeviathan));

        // Call the event handler for catching Storm Leviathan
        achievements.on_storm_leviathan_caught(Some("Hero"));

        // Should now be unlocked
        assert!(achievements.is_unlocked(AchievementId::StormLeviathan));
    }

    #[test]
    fn test_storm_leviathan_only_unlocks_once() {
        let mut achievements = Achievements::default();

        // First catch unlocks the achievement
        assert!(achievements.unlock(AchievementId::StormLeviathan, Some("Hero".to_string())));

        // Second catch should not unlock again
        assert!(!achievements.unlock(AchievementId::StormLeviathan, None));
    }

    // =========================================================================
    // TheStormbreaker Achievement Tests
    // =========================================================================

    #[test]
    fn test_stormbreaker_can_be_unlocked() {
        let mut achievements = Achievements::default();

        // TheStormbreaker should not be unlocked initially
        assert!(!achievements.is_unlocked(AchievementId::TheStormbreaker));

        // Unlock TheStormbreaker (simulating forge)
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));

        // Should now be unlocked
        assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
    }

    #[test]
    fn test_stormbreaker_unlocks_independently_of_leviathan() {
        let mut achievements = Achievements::default();

        // Can unlock TheStormbreaker without Storm Leviathan (test only - game logic prevents this)
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));

        assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
        assert!(!achievements.is_unlocked(AchievementId::StormLeviathan));
    }

    // =========================================================================
    // Haven Sync Achievement Tests
    // =========================================================================

    /// Build a HashMap of Haven room tiers for testing.
    /// Sets all buildable rooms (excluding StormForge) to the given tier.
    fn build_haven_tiers(tier: u8) -> HashMap<crate::haven::types::HavenRoomId, u8> {
        use crate::haven::types::HavenRoomId;
        HavenRoomId::ALL
            .iter()
            .filter(|r| **r != HavenRoomId::StormForge)
            .map(|r| (*r, tier))
            .collect()
    }

    #[test]
    fn test_haven_sync_discovered() {
        let mut achievements = Achievements::default();
        let room_tiers = HashMap::new();

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));
    }

    #[test]
    fn test_haven_sync_builder_i() {
        let mut achievements = Achievements::default();
        let room_tiers = build_haven_tiers(1);

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
        assert!(!achievements.is_unlocked(AchievementId::HavenBuilderII));
    }

    #[test]
    fn test_haven_sync_builder_ii() {
        let mut achievements = Achievements::default();
        let room_tiers = build_haven_tiers(2);

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));
        assert!(!achievements.is_unlocked(AchievementId::HavenArchitect));
    }

    #[test]
    fn test_haven_sync_architect() {
        use crate::haven::types::HavenRoomId;
        let mut achievements = Achievements::default();
        let room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
            .iter()
            .map(|r| (*r, r.max_tier()))
            .collect();

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));
        assert!(achievements.is_unlocked(AchievementId::HavenArchitect));
    }

    // =========================================================================
    // Fishing Rank Achievement Tests
    // =========================================================================

    #[test]
    fn test_fishing_rank_milestones() {
        let mut achievements = Achievements::default();

        // Rank 9 — no achievements yet
        achievements.on_fishing_rank_up(9, Some("Hero"));
        assert!(!achievements.is_unlocked(AchievementId::FishermanI));

        // Rank 10 unlocks FishermanI
        achievements.on_fishing_rank_up(10, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanI));
        assert!(!achievements.is_unlocked(AchievementId::FishermanII));

        // Rank 20 unlocks FishermanII
        achievements.on_fishing_rank_up(20, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanII));
        assert!(!achievements.is_unlocked(AchievementId::FishermanIII));

        // Rank 30 unlocks FishermanIII
        achievements.on_fishing_rank_up(30, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanIII));
        assert!(!achievements.is_unlocked(AchievementId::FishermanIV));

        // Rank 40 unlocks FishermanIV
        achievements.on_fishing_rank_up(40, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanIV));
    }

    #[test]
    fn test_fishing_rank_tracks_highest() {
        let mut achievements = Achievements::default();

        achievements.on_fishing_rank_up(15, Some("Hero"));
        assert_eq!(achievements.highest_fishing_rank, 15);

        // Lower rank should not decrease the highest
        achievements.on_fishing_rank_up(10, Some("Hero"));
        assert_eq!(achievements.highest_fishing_rank, 15);

        // Higher rank should update
        achievements.on_fishing_rank_up(25, Some("Hero"));
        assert_eq!(achievements.highest_fishing_rank, 25);
    }

    // =========================================================================
    // Count by Category Tests
    // =========================================================================

    #[test]
    fn test_count_by_category_empty() {
        let achievements = Achievements::default();
        let (unlocked, total) = achievements.count_by_category(AchievementCategory::Combat);
        assert_eq!(unlocked, 0);
        assert!(total > 0);
    }

    #[test]
    fn test_count_by_category_partial_unlock() {
        let mut achievements = Achievements {
            total_kills: 99,
            ..Default::default()
        };

        // Unlock some combat achievements
        achievements.on_enemy_killed(false, Some("Hero")); // 100 kills → SlayerI
        achievements.on_enemy_killed(true, Some("Hero")); // 1 boss → BossHunterI

        let (unlocked, total) = achievements.count_by_category(AchievementCategory::Combat);
        assert_eq!(unlocked, 2); // SlayerI + BossHunterI
        assert!(total > 2);

        // Other categories unaffected
        let (level_unlocked, _) = achievements.count_by_category(AchievementCategory::Level);
        assert_eq!(level_unlocked, 0);
    }
}
