//! Character persistence operations (save, load, list, delete, rename).
//!
//! Contains the [`CharacterManager`] CRUD implementation methods, extracted
//! from `manager.rs` which retains the struct and type definitions.

use std::fs;
use std::io;

use chrono::Utc;

use super::manager::{CharacterInfo, CharacterManager, CharacterSaveData, ACCOUNT_FILES};
use super::name_validation::{sanitize_name, validate_name};
use crate::core::constants::SAVE_FILE_VERSION;

impl CharacterManager {
    pub fn save_character(&self, state: &crate::core::game_state::GameState) -> io::Result<()> {
        // Use current time as last_save_time to prevent offline XP exploits.
        // Previously this used state.last_save_time which was only updated on load,
        // allowing players to accumulate offline XP during active play sessions.
        let save_data = CharacterSaveData {
            version: SAVE_FILE_VERSION,
            character_id: state.character_id.clone(),
            character_name: state.character_name.clone(),
            character_level: state.character_level,
            character_xp: state.character_xp,
            attributes: state.attributes,
            prestige_rank: state.prestige_rank,
            total_prestige_count: state.total_prestige_count,
            last_save_time: Utc::now().timestamp(),
            play_time_seconds: state.play_time_seconds,
            combat_state: state.combat_state.clone(),
            equipment: state.equipment.clone(),
            active_dungeon: state.active_dungeon.clone(),
            fishing: state.fishing.clone(),
            zone_progression: state.zone_progression.clone(),
            stormglass: state.stormglass,
            stormglass_discovered: state.stormglass_discovered,
            storm_sigils: state.storm_sigils.clone(),
        };

        let json = serde_json::to_string_pretty(&save_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let filename = format!("{}.json", sanitize_name(&state.character_name));
        let filepath = self.quest_dir.join(filename);
        fs::write(filepath, json)?;

        Ok(())
    }

    pub fn load_character(&self, filename: &str) -> io::Result<crate::core::game_state::GameState> {
        let filepath = self.quest_dir.join(filename);
        let json_content = fs::read_to_string(filepath)?;

        let save_data: CharacterSaveData = serde_json::from_str(&json_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let player = crate::core::player_identity::PlayerIdentity {
            character_id: save_data.character_id.clone(),
            character_name: save_data.character_name.clone(),
            character_level: save_data.character_level,
            character_xp: save_data.character_xp,
            attributes: save_data.attributes,
            prestige_rank: save_data.prestige_rank,
            total_prestige_count: save_data.total_prestige_count,
        };

        let combat_ctx = crate::core::combat_context::CombatContext {
            combat_state: save_data.combat_state.clone(),
            equipment: save_data.equipment.clone(),
            zone_progression: save_data.zone_progression.clone(),
            active_dungeon: None,
            session_kills: 0,
            consecutive_deaths: 0,
        };

        Ok(crate::core::game_state::GameState {
            character_id: save_data.character_id,
            character_name: save_data.character_name,
            character_level: save_data.character_level,
            character_xp: save_data.character_xp,
            attributes: save_data.attributes,
            prestige_rank: save_data.prestige_rank,
            total_prestige_count: save_data.total_prestige_count,
            last_save_time: save_data.last_save_time,
            play_time_seconds: save_data.play_time_seconds,
            combat_state: save_data.combat_state,
            equipment: save_data.equipment,
            active_dungeon: save_data.active_dungeon,
            fishing: save_data.fishing.clone(),
            active_fishing: None,
            zone_progression: save_data.zone_progression,
            challenge_menu: crate::challenges::menu::ChallengeMenu::new(),
            chess_stats: Default::default(),
            active_minigame: None,
            session_kills: 0,
            consecutive_deaths: 0,
            recent_drops: std::collections::VecDeque::new(),
            ticker: crate::core::game_state::Ticker::new(),
            last_minigame_win: None,
            cached_derived_stats: Default::default(),
            cached_prestige_bonuses: Default::default(),
            derived_stats_dirty: true,
            xp_rate_samples: std::collections::VecDeque::new(),
            xp_this_second: 0,
            combat_seconds_this_tick: false,
            game_over_shown_at: None,
            stormglass: save_data.stormglass,
            stormglass_discovered: save_data.stormglass_discovered,
            storm_sigils: save_data.storm_sigils.clone(),
            chrono_surge_active: false,
            debug_force_overcharge: false,
            player,
            combat_ctx,
            prog: crate::core::progression_state::ProgressionState {
                fishing: save_data.fishing,
                active_fishing: None,
                stormglass: save_data.stormglass,
                stormglass_discovered: save_data.stormglass_discovered,
                storm_sigils: save_data.storm_sigils,
                challenge_menu: crate::challenges::menu::ChallengeMenu::new(),
                chess_stats: crate::challenges::chess::ChessStats::default(),
                active_minigame: None,
                last_minigame_win: None,
            },
            sess: crate::core::session_state::SessionState {
                last_save_time: save_data.last_save_time,
                play_time_seconds: save_data.play_time_seconds,
                chrono_surge_active: false,
                debug_force_overcharge: false,
                recent_drops: std::collections::VecDeque::with_capacity(5),
                xp_rate_samples: std::collections::VecDeque::new(),
                xp_this_second: 0,
                ticker: crate::core::ticker::Ticker::new(),
                cached_derived_stats: crate::character::derived_stats::DerivedStats::default(),
                cached_prestige_bonuses: crate::character::prestige::PrestigeCombatBonuses::default(
                ),
                derived_stats_dirty: true,
                combat_seconds_this_tick: false,
                game_over_shown_at: None,
            },
        })
    }

    pub fn list_characters(&self) -> io::Result<Vec<CharacterInfo>> {
        let mut characters = Vec::new();

        // Read directory entries
        let entries = fs::read_dir(&self.quest_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Skip account-level JSON files (not character saves)
            if ACCOUNT_FILES.contains(&filename.as_str()) {
                continue;
            }

            // Try to load character
            match self.load_character(&filename) {
                Ok(state) => {
                    characters.push(CharacterInfo {
                        character_id: state.character_id,
                        character_name: state.character_name,
                        filename,
                        character_level: state.character_level,
                        prestige_rank: state.prestige_rank,
                        play_time_seconds: state.play_time_seconds,
                        last_save_time: state.last_save_time,
                        attributes: state.attributes,
                        equipment: state.equipment,
                        is_corrupted: false,
                    });
                }
                Err(_) => {
                    // Mark as corrupted but include in list
                    characters.push(CharacterInfo {
                        character_id: String::new(),
                        character_name: "[CORRUPTED]".to_string(),
                        filename,
                        character_level: 0,
                        prestige_rank: 0,
                        play_time_seconds: 0,
                        last_save_time: 0,
                        attributes: super::attributes::Attributes::new(),
                        equipment: crate::items::Equipment::new(),
                        is_corrupted: true,
                    });
                }
            }
        }

        // Sort by last_save_time (most recent first)
        characters.sort_by(|a, b| b.last_save_time.cmp(&a.last_save_time));

        Ok(characters)
    }

    pub fn delete_character(&self, filename: &str) -> io::Result<()> {
        let filepath = self.quest_dir.join(filename);
        fs::remove_file(filepath)?;
        Ok(())
    }

    pub fn rename_character(&self, old_filename: &str, new_name: String) -> io::Result<()> {
        // Validate new name
        validate_name(&new_name).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // Load existing character
        let mut state = self.load_character(old_filename)?;

        // Update character name
        state.character_name = new_name.clone();

        // Save with new name
        self.save_character(&state)?;

        // Delete old file
        let old_filepath = self.quest_dir.join(old_filename);
        fs::remove_file(old_filepath)?;

        Ok(())
    }
}
