//! Character persistence operations (save, load, list, delete, rename).
//!
//! Contains the [`CharacterManager`] CRUD implementation methods, extracted
//! from `manager.rs` which retains the struct and type definitions.

use std::fs;
use std::io;

use chrono::Utc;

use super::manager::{CharacterHeader, CharacterInfo, CharacterManager, CharacterSaveData};
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
            ascension_level: state.ascension_level,
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
            fishing: save_data.fishing,
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
            cached_power_rating: 0.0,
            cached_fracture_zone_cap: 0,
            cached_loom_zone_cap: 0,
            cached_haven_bonuses: Default::default(),
            cached_sigil_bonuses: Default::default(),
            bonuses_dirty: true,
            cached_god_item_bonuses: Default::default(),
            stormglass: save_data.stormglass,
            stormglass_discovered: save_data.stormglass_discovered,
            storm_sigils: save_data.storm_sigils,
            ascension_level: save_data.ascension_level,
            chrono_surge_active: false,
            debug_force_overcharge: false,
        })
    }

    /// Probes a JSON file to determine if it is a character save.
    ///
    /// Three possible outcomes:
    /// - `Ok(Some(header))` — confirmed character file; header fields were parsed successfully.
    /// - `Ok(None)` — confirmed non-character file; valid JSON but no `character_name` field,
    ///   so this is an account-level save (haven.json, deep.json, etc.).  Skip it silently.
    /// - `Err(_)` — could not determine (malformed JSON or I/O error); caller should treat the
    ///   file as a potentially corrupted character save so it shows up in the list.
    pub(super) fn load_character_header(
        &self,
        filename: &str,
    ) -> io::Result<Option<CharacterHeader>> {
        let filepath = self.quest_dir.join(filename);
        let json_content = fs::read_to_string(filepath)?;

        // Parse to a generic value so we can inspect structure without full deserialization.
        let value: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // No `character_name` string field → this is an account-level file, not a character.
        if !value.get("character_name").is_some_and(|v| v.is_string()) {
            return Ok(None);
        }

        // Has `character_name` — deserialize the header fields we care about.
        // If individual header fields are malformed, bubble up as Err so the caller
        // treats it as a corrupted character (rather than silently skipping it).
        let header = serde_json::from_value::<CharacterHeader>(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(header))
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

            // Probe the file to determine how to handle it:
            //   Ok(None)    → account-level file (no character_name), skip silently
            //   Ok(Some(_)) → character file; proceed to full load
            //   Err(_)      → malformed JSON or I/O error; include as corrupted character
            match self.load_character_header(&filename) {
                Ok(None) => {
                    // Confirmed account-level file — skip without adding to list.
                }
                Ok(Some(header)) => {
                    // Valid character header; do full load for equipment/sigils.
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
                                ascension_level: state.ascension_level,
                                storm_sigils: state.storm_sigils,
                                is_corrupted: false,
                            });
                        }
                        Err(_) => {
                            // Header parsed but full load failed — show as corrupted with
                            // whatever we already know from the header.
                            characters.push(CharacterInfo {
                                character_id: header.character_id,
                                character_name: header.character_name,
                                filename,
                                character_level: header.character_level,
                                prestige_rank: header.prestige_rank,
                                play_time_seconds: header.play_time_seconds,
                                last_save_time: header.last_save_time,
                                attributes: super::attributes::Attributes::new(),
                                equipment: crate::items::Equipment::new(),
                                ascension_level: header.ascension_level,
                                storm_sigils: crate::stormglass::sigils::StormSigils::new(),
                                is_corrupted: true,
                            });
                        }
                    }
                }
                Err(_) => {
                    // Malformed JSON or I/O error — we can't tell if this is a character file,
                    // so include it as a corrupted entry so the player can see and delete it.
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
                        ascension_level: 0,
                        storm_sigils: crate::stormglass::sigils::StormSigils::new(),
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
