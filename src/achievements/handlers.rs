//! Event handlers and state synchronization for the achievement system.
//!
//! This module contains all `on_*` event handler methods and `sync_*` state
//! synchronization methods for [`Achievements`]. These are separated from
//! `types.rs` to keep that file focused on data structures and core unlock
//! machinery.
//!
//! ## Handler naming convention
//!
//! - `on_*` — called by game systems when a discrete event occurs (enemy kill,
//!   level up, dungeon completed, etc.). Increments counters and checks milestones.
//! - `sync_*` — called once on character load to retroactively unlock achievements
//!   for milestones already passed in a pre-existing save.

use super::milestones::{
    MinigameDifficulty, MinigameType, BOSS_HUNTER_MILESTONES, DEEP_GUILD_RANK_MILESTONES,
    DEEP_LAYER_MILESTONES, DEEP_MISSION_MILESTONES, DUNGEON_MILESTONES, FISHERMAN_MILESTONES,
    FISH_CATCHER_MILESTONES, GRAND_CHAMPION_MILESTONES, LEVEL_MILESTONES, POWER_CORE_MILESTONES,
    PRESTIGE_MILESTONES, SLAYER_MILESTONES,
};
use super::types::{AchievementId, Achievements};

impl Achievements {
    // =========================================================================
    // Fishing Event Handlers
    // =========================================================================

    /// Called when the Storm Leviathan is caught.
    /// Unlocks the StormLeviathan achievement (required for Stormbreaker).
    pub fn on_storm_leviathan_caught(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::StormLeviathan, character_name);
    }

    /// Called when fishing rank changes.
    /// Unlocks FishermanI/II/III/IV achievements at milestones.
    pub fn on_fishing_rank_up(&mut self, new_rank: u32, character_name: Option<&str>) {
        // Update highest rank
        if new_rank > self.highest_fishing_rank {
            self.highest_fishing_rank = new_rank;
        }

        self.check_milestones(new_rank as u64, FISHERMAN_MILESTONES, character_name);
    }

    /// Called when a fish is caught.
    /// Unlocks fish catching milestone achievements.
    pub fn on_fish_caught(&mut self, character_name: Option<&str>) {
        self.total_fish_caught += 1;

        self.check_milestones(
            self.total_fish_caught,
            FISH_CATCHER_MILESTONES,
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

        self.check_milestones(self.total_kills, SLAYER_MILESTONES, character_name);

        if is_boss {
            self.total_bosses_defeated += 1;

            self.check_milestones(
                self.total_bosses_defeated,
                BOSS_HUNTER_MILESTONES,
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

        self.check_milestones(new_level as u64, LEVEL_MILESTONES, character_name);
    }

    /// Called when the character prestiges.
    /// Unlocks prestige milestone achievements.
    pub fn on_prestige(&mut self, new_rank: u32, character_name: Option<&str>) {
        if new_rank > self.highest_prestige_rank {
            self.highest_prestige_rank = new_rank;
        }

        self.check_milestones(new_rank as u64, PRESTIGE_MILESTONES, character_name);
    }

    /// Called when a zone is fully cleared (all subzones completed).
    pub fn on_zone_fully_cleared(&mut self, zone_id: u32, character_name: Option<&str>) {
        self.zones_fully_cleared += 1;

        // Zone 11 (The Expanse) — single achievement for entering
        if zone_id == 11 {
            self.expanse_cycles_completed += 1;
            self.unlock_with_name(AchievementId::BeyondInfinity, character_name);
            return;
        }

        // Individual zone completion achievements (zones 1-10, 12-20)
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
            // Fracture zone completion achievements (zones 12-20)
            12 => Some(AchievementId::FractureZone12),
            13 => Some(AchievementId::FractureZone13),
            14 => Some(AchievementId::FractureZone14),
            15 => Some(AchievementId::FractureZone15),
            16 => Some(AchievementId::FractureZone16),
            17 => Some(AchievementId::FractureZone17),
            18 => Some(AchievementId::FractureZone18),
            19 => Some(AchievementId::FractureZone19),
            20 => Some(AchievementId::FractureZone20),
            21 => Some(AchievementId::FractureZone21),
            22 => Some(AchievementId::FractureZone22),
            23 => Some(AchievementId::FractureZone23),
            24 => Some(AchievementId::FractureZone24),
            25 => Some(AchievementId::FractureZone25),
            26 => Some(AchievementId::FractureZone26),
            27 => Some(AchievementId::FractureZone27),
            28 => Some(AchievementId::FractureZone28),
            29 => Some(AchievementId::FractureZone29),
            30 => Some(AchievementId::FractureZone30),
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

    /// Called when the character Ascends to a new level.
    pub fn on_ascended(&mut self, new_level: u32, character_name: Option<&str>) {
        let id = match new_level {
            1 => Some(AchievementId::AscensionI),
            2 => Some(AchievementId::AscensionII),
            3 => Some(AchievementId::AscensionIII),
            4 => Some(AchievementId::AscensionIV),
            5 => Some(AchievementId::AscensionV),
            6 => Some(AchievementId::AscensionVI),
            7 => Some(AchievementId::AscensionVII),
            8 => Some(AchievementId::AscensionVIII),
            9 => Some(AchievementId::AscensionIX),
            10 => Some(AchievementId::AscensionX),
            _ => None,
        };
        if let Some(id) = id {
            self.unlock_with_name(id, character_name);
        }
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
            DUNGEON_MILESTONES,
            character_name,
        );
    }

    // =========================================================================
    // Challenge/Minigame Event Handlers
    // =========================================================================

    /// Called when a minigame is won.
    ///
    /// Uses type-safe [`MinigameType`] and [`MinigameDifficulty`] enums
    /// instead of raw strings, providing exhaustive compile-time coverage.
    pub fn on_minigame_won(
        &mut self,
        game_type: MinigameType,
        difficulty: MinigameDifficulty,
        character_name: Option<&str>,
    ) {
        self.total_minigame_wins += 1;

        // Game-specific achievements based on difficulty
        let achievement = match (game_type, difficulty) {
            (MinigameType::Chess, MinigameDifficulty::Novice) => Some(AchievementId::ChessNovice),
            (MinigameType::Chess, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::ChessApprentice)
            }
            (MinigameType::Chess, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::ChessJourneyman)
            }
            (MinigameType::Chess, MinigameDifficulty::Master) => Some(AchievementId::ChessMaster),
            (MinigameType::Morris, MinigameDifficulty::Novice) => Some(AchievementId::MorrisNovice),
            (MinigameType::Morris, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::MorrisApprentice)
            }
            (MinigameType::Morris, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::MorrisJourneyman)
            }
            (MinigameType::Morris, MinigameDifficulty::Master) => Some(AchievementId::MorrisMaster),
            (MinigameType::Gomoku, MinigameDifficulty::Novice) => Some(AchievementId::GomokuNovice),
            (MinigameType::Gomoku, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::GomokuApprentice)
            }
            (MinigameType::Gomoku, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::GomokuJourneyman)
            }
            (MinigameType::Gomoku, MinigameDifficulty::Master) => Some(AchievementId::GomokuMaster),
            (MinigameType::Minesweeper, MinigameDifficulty::Novice) => {
                Some(AchievementId::MinesweeperNovice)
            }
            (MinigameType::Minesweeper, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::MinesweeperApprentice)
            }
            (MinigameType::Minesweeper, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::MinesweeperJourneyman)
            }
            (MinigameType::Minesweeper, MinigameDifficulty::Master) => {
                Some(AchievementId::MinesweeperMaster)
            }
            (MinigameType::Rune, MinigameDifficulty::Novice) => Some(AchievementId::RuneNovice),
            (MinigameType::Rune, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::RuneApprentice)
            }
            (MinigameType::Rune, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::RuneJourneyman)
            }
            (MinigameType::Rune, MinigameDifficulty::Master) => Some(AchievementId::RuneMaster),
            (MinigameType::Go, MinigameDifficulty::Novice) => Some(AchievementId::GoNovice),
            (MinigameType::Go, MinigameDifficulty::Apprentice) => Some(AchievementId::GoApprentice),
            (MinigameType::Go, MinigameDifficulty::Journeyman) => Some(AchievementId::GoJourneyman),
            (MinigameType::Go, MinigameDifficulty::Master) => Some(AchievementId::GoMaster),
            (MinigameType::FlappyBird, MinigameDifficulty::Novice) => {
                Some(AchievementId::FlappyNovice)
            }
            (MinigameType::FlappyBird, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::FlappyApprentice)
            }
            (MinigameType::FlappyBird, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::FlappyJourneyman)
            }
            (MinigameType::FlappyBird, MinigameDifficulty::Master) => {
                Some(AchievementId::FlappyMaster)
            }
            (MinigameType::Snake, MinigameDifficulty::Novice) => Some(AchievementId::SnakeNovice),
            (MinigameType::Snake, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::SnakeApprentice)
            }
            (MinigameType::Snake, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::SnakeJourneyman)
            }
            (MinigameType::Snake, MinigameDifficulty::Master) => Some(AchievementId::SnakeMaster),
            (MinigameType::Jezzball, MinigameDifficulty::Novice) => {
                Some(AchievementId::ContainmentBreachNovice)
            }
            (MinigameType::Jezzball, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::ContainmentBreachApprentice)
            }
            (MinigameType::Jezzball, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::ContainmentBreachJourneyman)
            }
            (MinigameType::Jezzball, MinigameDifficulty::Master) => {
                Some(AchievementId::ContainmentBreachMaster)
            }
            (MinigameType::RunicShift, MinigameDifficulty::Novice) => {
                Some(AchievementId::SigilSurgeNovice)
            }
            (MinigameType::RunicShift, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::SigilSurgeApprentice)
            }
            (MinigameType::RunicShift, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::SigilSurgeJourneyman)
            }
            (MinigameType::RunicShift, MinigameDifficulty::Master) => {
                Some(AchievementId::SigilSurgeMaster)
            }
            (MinigameType::SigilMatrix, MinigameDifficulty::Novice) => {
                Some(AchievementId::SigilMatrixNovice)
            }
            (MinigameType::SigilMatrix, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::SigilMatrixApprentice)
            }
            (MinigameType::SigilMatrix, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::SigilMatrixJourneyman)
            }
            (MinigameType::SigilMatrix, MinigameDifficulty::Master) => {
                Some(AchievementId::SigilMatrixMaster)
            }
            (MinigameType::ShardFusion, MinigameDifficulty::Novice) => {
                Some(AchievementId::ShardFusionNovice)
            }
            (MinigameType::ShardFusion, MinigameDifficulty::Apprentice) => {
                Some(AchievementId::ShardFusionApprentice)
            }
            (MinigameType::ShardFusion, MinigameDifficulty::Journeyman) => {
                Some(AchievementId::ShardFusionJourneyman)
            }
            (MinigameType::ShardFusion, MinigameDifficulty::Master) => {
                Some(AchievementId::ShardFusionMaster)
            }
        };

        if let Some(id) = achievement {
            self.unlock_with_name(id, character_name);
        }

        // Grand Champion - 100 total wins
        self.check_milestones(
            self.total_minigame_wins,
            GRAND_CHAMPION_MILESTONES,
            character_name,
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
    // The Deep Event Handlers
    // =========================================================================

    /// Called when The Deep is first discovered.
    pub fn on_deep_discovered(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::TheDeepDiscovered, character_name);
    }

    /// Called when a Deep mission is completed.
    /// Unlocks mission completion milestone achievements.
    pub fn on_deep_mission_complete(&mut self, character_name: Option<&str>) {
        self.total_deep_missions_completed += 1;

        self.check_milestones(
            self.total_deep_missions_completed,
            DEEP_MISSION_MILESTONES,
            character_name,
        );
    }

    /// Called when a breakthrough mission is completed in The Deep.
    /// Unlocks FirstBreakthrough and checks layer milestones.
    pub fn on_deep_breakthrough(&mut self, new_layer: u32, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::FirstBreakthrough, character_name);

        if new_layer > self.highest_deep_layer {
            self.highest_deep_layer = new_layer;
        }

        self.check_milestones(new_layer as u64, DEEP_LAYER_MILESTONES, character_name);
        self.check_milestones(new_layer as u64, POWER_CORE_MILESTONES, character_name);
    }

    /// Called when the guild rank increases in The Deep.
    pub fn on_deep_guild_rank_up(&mut self, new_rank: u32, character_name: Option<&str>) {
        if new_rank > self.highest_guild_rank {
            self.highest_guild_rank = new_rank;
        }

        self.check_milestones(new_rank as u64, DEEP_GUILD_RANK_MILESTONES, character_name);
    }

    /// Called when a mercenary is permanently lost in The Deep.
    pub fn on_deep_merc_lost(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::FirstMercLost, character_name);
    }

    /// Called when the Gateway Expedition succeeds, opening the sealed gateway.
    pub fn on_deep_gateway_opened(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::GatewayOpened, character_name);
    }

    // =========================================================================
    // Enhancement/Soulforge Event Handlers
    // =========================================================================

    /// Called when the Soulforge is first discovered.
    pub fn on_soulforge_discovered(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::SoulforgeDiscovered, character_name);
    }

    /// Called after an enhancement attempt completes (success or failure).
    /// Checks all enhancement-related achievement milestones.
    pub fn on_enhancement_upgraded(
        &mut self,
        new_level: u8,
        all_levels: &[u8; 7],
        total_attempts: u32,
        character_name: Option<&str>,
    ) {
        if new_level >= 1 {
            self.unlock_with_name(AchievementId::ApprenticeSmith, character_name);
        }
        if new_level >= 5 {
            self.unlock_with_name(AchievementId::JourneymanSmith, character_name);
        }
        if new_level >= 6 {
            self.unlock_with_name(AchievementId::SoulforgeAdept, character_name);
        }
        if new_level >= 7 {
            self.unlock_with_name(AchievementId::SoulforgeSavant, character_name);
        }
        if new_level >= 8 {
            self.unlock_with_name(AchievementId::SoulforgeMaster, character_name);
        }
        if new_level >= 9 {
            self.unlock_with_name(AchievementId::SoulforgeGrandmaster, character_name);
        }
        if new_level >= 10 {
            self.unlock_with_name(AchievementId::SoulforgeAscendant, character_name);
        }
        if all_levels.iter().all(|&l| l >= 4) {
            self.unlock_with_name(AchievementId::FullyTempered, character_name);
        }
        if all_levels.iter().all(|&l| l >= 7) {
            self.unlock_with_name(AchievementId::SoulConvergence, character_name);
        }
        if total_attempts >= 100 {
            self.unlock_with_name(AchievementId::PersistentHammering, character_name);
        }
    }

    // =========================================================================
    // Loom
    // =========================================================================

    /// Called when the Loom of Worlds is discovered.
    pub fn on_loom_discovered(&mut self, character_name: Option<&str>) {
        self.unlock_with_name(AchievementId::LoomDiscovered, character_name);
    }

    /// Called when a Woven Pattern is completed.
    /// `completed_count` is the total number of completed patterns.
    pub fn on_loom_pattern_completed(
        &mut self,
        completed_count: usize,
        character_name: Option<&str>,
    ) {
        const PATTERN_MILESTONES: &[(usize, AchievementId)] = &[
            (1, AchievementId::LoomPattern1),
            (4, AchievementId::LoomPattern4),
            (8, AchievementId::LoomPattern8),
            (16, AchievementId::LoomPattern16),
            (22, AchievementId::LoomPattern22),
            (28, AchievementId::LoomPattern28),
        ];
        for &(threshold, id) in PATTERN_MILESTONES {
            if completed_count >= threshold {
                self.unlock_with_name(id, character_name);
            }
        }
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
        defeated_bosses: &std::collections::BTreeSet<(u32, u32)>,
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

        // Refresh progress bars from current counters
        self.refresh_progress();
    }

    /// Syncs ascension achievements based on current ascension level.
    /// Call this when loading a character to retroactively unlock achievements
    /// for ascension levels already reached.
    pub fn sync_from_ascension(&mut self, ascension_level: u32, character_name: Option<&str>) {
        for level in 1..=ascension_level {
            self.on_ascended(level, character_name);
        }
    }

    /// Syncs zone completion achievements based on defeated bosses.
    fn sync_zone_completions(
        &mut self,
        defeated_bosses: &std::collections::BTreeSet<(u32, u32)>,
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

    /// Refresh progress entries from existing aggregate counters.
    /// Called on load so progress bars display even for counters accumulated before this feature.
    pub fn refresh_progress(&mut self) {
        let series: &[(u64, &[(u64, AchievementId)])] = &[
            (self.total_kills, SLAYER_MILESTONES),
            (self.total_bosses_defeated, BOSS_HUNTER_MILESTONES),
            (self.total_dungeons_completed, DUNGEON_MILESTONES),
            (self.total_fish_caught, FISH_CATCHER_MILESTONES),
            (self.total_minigame_wins, GRAND_CHAMPION_MILESTONES),
            (self.total_deep_missions_completed, DEEP_MISSION_MILESTONES),
            (self.highest_deep_layer as u64, DEEP_LAYER_MILESTONES),
            (self.highest_guild_rank as u64, DEEP_GUILD_RANK_MILESTONES),
        ];

        for (current, milestones) in series {
            for &(threshold, achievement_id) in *milestones {
                self.update_progress(achievement_id, *current, threshold);
            }
        }
    }

    /// Syncs The Deep achievements based on Deep persistent state.
    /// Call this when loading a character to retroactively unlock achievements
    /// for Deep milestones already reached (guild rank, layers).
    ///
    /// Note: Mission count is already tracked in the achievements struct via
    /// `on_deep_mission_complete()` and does not need syncing from Deep state.
    pub fn sync_from_deep(
        &mut self,
        discovered: bool,
        guild_rank: u32,
        deepest_layer: u32,
        character_name: Option<&str>,
    ) {
        if discovered {
            self.on_deep_discovered(character_name);
        }

        // Sync guild rank milestones
        if guild_rank > self.highest_guild_rank {
            self.highest_guild_rank = guild_rank;
        }
        if guild_rank >= 2 {
            self.on_deep_guild_rank_up(guild_rank, character_name);
        }

        // Sync layer milestones
        if deepest_layer > self.highest_deep_layer {
            self.highest_deep_layer = deepest_layer;
        }
        if deepest_layer >= 1 {
            self.on_deep_breakthrough(deepest_layer, character_name);
        }

        // Refresh progress bars from current counters
        self.refresh_progress();
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

    /// Syncs Loom-related achievements based on Loom state.
    /// Call this when loading Loom data.
    pub fn sync_from_loom(
        &mut self,
        discovered: bool,
        completed_patterns: usize,
        character_name: Option<&str>,
    ) {
        if discovered {
            self.on_loom_discovered(character_name);
        }
        if completed_patterns > 0 {
            self.on_loom_pattern_completed(completed_patterns, character_name);
        }
    }
}
