//! Custom serde implementation for GameState that maintains flat JSON format
//! after the struct is decomposed into sub-structs.
//!
//! This module is used during the migration period where GameState fields
//! are being moved into sub-structs. It ensures saves remain compatible.

use serde::{Deserialize, Serialize};

/// Flat JSON representation matching the original GameState format.
/// Used as an intermediate for serialization/deserialization.
#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct FlatGameState {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: crate::character::attributes::Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub combat_state: crate::combat::types::CombatState,
    pub equipment: crate::items::equipment::Equipment,
    #[serde(default)]
    pub active_dungeon: Option<crate::dungeon::types::Dungeon>,
    #[serde(default)]
    pub fishing: crate::fishing::types::FishingState,
    #[serde(default)]
    pub zone_progression: crate::zones::ZoneProgression,
    #[serde(default)]
    pub stormglass: u64,
    #[serde(default)]
    pub stormglass_discovered: bool,
    #[serde(default)]
    pub storm_sigils: crate::stormglass::sigils::StormSigils,
}

impl From<&crate::core::game_state::GameState> for FlatGameState {
    fn from(state: &crate::core::game_state::GameState) -> Self {
        Self {
            character_id: state.character_id.clone(),
            character_name: state.character_name.clone(),
            character_level: state.character_level,
            character_xp: state.character_xp,
            attributes: state.attributes,
            prestige_rank: state.prestige_rank,
            total_prestige_count: state.total_prestige_count,
            last_save_time: state.last_save_time,
            play_time_seconds: state.play_time_seconds,
            combat_state: state.combat_state.clone(),
            equipment: state.equipment.clone(),
            active_dungeon: state.active_dungeon.clone(),
            fishing: state.fishing.clone(),
            zone_progression: state.zone_progression.clone(),
            stormglass: state.stormglass,
            stormglass_discovered: state.stormglass_discovered,
            storm_sigils: state.storm_sigils.clone(),
        }
    }
}

impl FlatGameState {
    #[allow(dead_code)]
    pub(crate) fn into_game_state(self) -> crate::core::game_state::GameState {
        use crate::challenges::chess::ChessStats;
        use crate::challenges::menu::ChallengeMenu;
        use crate::character::derived_stats::DerivedStats;
        use crate::character::prestige::PrestigeCombatBonuses;
        use crate::core::combat_context::CombatContext;
        use crate::core::game_state::GameState;
        use crate::core::player_identity::PlayerIdentity;
        use crate::core::progression_state::ProgressionState;
        use crate::core::session_state::SessionState;
        use crate::core::ticker::Ticker;
        use std::collections::VecDeque;

        let player = PlayerIdentity {
            character_id: self.character_id.clone(),
            character_name: self.character_name.clone(),
            character_level: self.character_level,
            character_xp: self.character_xp,
            attributes: self.attributes,
            prestige_rank: self.prestige_rank,
            total_prestige_count: self.total_prestige_count,
        };

        let combat_ctx = CombatContext {
            combat_state: self.combat_state.clone(),
            equipment: self.equipment.clone(),
            zone_progression: self.zone_progression.clone(),
            active_dungeon: self.active_dungeon.clone(),
            session_kills: 0,
            consecutive_deaths: 0,
        };

        let prog = ProgressionState {
            fishing: self.fishing.clone(),
            active_fishing: None,
            stormglass: self.stormglass,
            stormglass_discovered: self.stormglass_discovered,
            storm_sigils: self.storm_sigils.clone(),
            challenge_menu: ChallengeMenu::new(),
            chess_stats: ChessStats::default(),
            active_minigame: None,
            last_minigame_win: None,
        };

        let sess = SessionState {
            last_save_time: self.last_save_time,
            play_time_seconds: self.play_time_seconds,
            chrono_surge_active: false,
            debug_force_overcharge: false,
            recent_drops: VecDeque::with_capacity(5),
            xp_rate_samples: VecDeque::new(),
            xp_this_second: 0,
            ticker: Ticker::new(),
            cached_derived_stats: DerivedStats::default(),
            cached_prestige_bonuses: PrestigeCombatBonuses::default(),
            derived_stats_dirty: true,
            combat_seconds_this_tick: false,
            game_over_shown_at: None,
        };

        let mut state = GameState {
            // Persisted fields
            character_id: self.character_id,
            character_name: self.character_name,
            character_level: self.character_level,
            character_xp: self.character_xp,
            attributes: self.attributes,
            prestige_rank: self.prestige_rank,
            total_prestige_count: self.total_prestige_count,
            last_save_time: self.last_save_time,
            play_time_seconds: self.play_time_seconds,
            combat_state: self.combat_state,
            equipment: self.equipment,
            active_dungeon: self.active_dungeon,
            fishing: self.fishing,
            zone_progression: self.zone_progression,
            stormglass: self.stormglass,
            stormglass_discovered: self.stormglass_discovered,
            storm_sigils: self.storm_sigils,
            // Transient fields — defaults
            active_fishing: None,
            challenge_menu: ChallengeMenu::new(),
            chess_stats: ChessStats::default(),
            active_minigame: None,
            session_kills: 0,
            consecutive_deaths: 0,
            chrono_surge_active: false,
            debug_force_overcharge: false,
            recent_drops: VecDeque::with_capacity(5),
            ticker: Ticker::new(),
            last_minigame_win: None,
            cached_derived_stats: DerivedStats::default(),
            cached_prestige_bonuses: PrestigeCombatBonuses::default(),
            derived_stats_dirty: true,
            xp_rate_samples: VecDeque::new(),
            xp_this_second: 0,
            combat_seconds_this_tick: false,
            game_over_shown_at: None,
            // Sub-structs
            player,
            combat_ctx,
            prog,
            sess,
        };
        state.sync_sub_structs();
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_game_state_round_trip() {
        // Minimal JSON that exercises all required fields.
        // Attributes serializes as {"values": [...]}.
        // CombatState persistent fields: current_enemy, player_current_hp,
        // player_max_hp, player_attack_timer, enemy_attack_timer,
        // regen_timer, is_regenerating.
        let json = r#"{"character_id":"test","character_name":"Hero","character_level":5,"character_xp":1000,"attributes":{"values":[12,10,14,10,10,10]},"prestige_rank":2,"total_prestige_count":3,"last_save_time":1000000,"play_time_seconds":3600,"combat_state":{"current_enemy":null,"player_current_hp":100,"player_max_hp":100,"player_attack_timer":0.0,"enemy_attack_timer":0.0,"regen_timer":0.0,"is_regenerating":false},"equipment":{"weapon":null,"armor":null,"helmet":null,"gloves":null,"boots":null,"amulet":null,"ring":null}}"#;
        let flat: FlatGameState = serde_json::from_str(json).unwrap();
        assert_eq!(flat.character_name, "Hero");
        assert_eq!(flat.character_level, 5);
        assert_eq!(flat.prestige_rank, 2);
        let re_json = serde_json::to_string(&flat).unwrap();
        let flat2: FlatGameState = serde_json::from_str(&re_json).unwrap();
        assert_eq!(flat2.character_name, flat.character_name);
        assert_eq!(flat2.character_level, flat.character_level);
    }

    #[test]
    fn test_from_game_state_to_flat() {
        use crate::core::game_state::GameState;
        let state = GameState::new("ConvTest".to_string(), 12345);
        let flat = FlatGameState::from(&state);
        assert_eq!(flat.character_name, "ConvTest");
        assert_eq!(flat.character_level, 1);
        assert_eq!(flat.last_save_time, 12345);
    }

    #[test]
    fn test_flat_into_game_state() {
        use crate::core::game_state::GameState;
        let original = GameState::new("RoundTrip".to_string(), 99999);
        let flat = FlatGameState::from(&original);
        let restored = flat.into_game_state();
        assert_eq!(restored.character_name, "RoundTrip");
        assert_eq!(restored.character_level, 1);
        assert_eq!(restored.last_save_time, 99999);
        // Sub-structs should also be populated via sync_sub_structs
        assert_eq!(restored.player.character_name, "RoundTrip");
        assert_eq!(restored.player.character_level, 1);
        assert_eq!(restored.combat_ctx.zone_progression.current_zone_id, 1);
        assert_eq!(restored.sess.last_save_time, 99999);
        assert_eq!(restored.prog.stormglass, 0);
    }

    #[test]
    fn test_flat_round_trip_preserves_all_persisted_fields() {
        use crate::core::game_state::GameState;
        let mut original = GameState::new("FullTrip".to_string(), 55555);
        original.character_level = 20;
        original.character_xp = 50000;
        original.prestige_rank = 5;
        original.total_prestige_count = 10;
        original.play_time_seconds = 7200;
        original.stormglass = 42000;
        original.stormglass_discovered = true;

        let flat = FlatGameState::from(&original);
        let restored = flat.into_game_state();

        assert_eq!(restored.character_level, 20);
        assert_eq!(restored.character_xp, 50000);
        assert_eq!(restored.prestige_rank, 5);
        assert_eq!(restored.total_prestige_count, 10);
        assert_eq!(restored.play_time_seconds, 7200);
        assert_eq!(restored.stormglass, 42000);
        assert!(restored.stormglass_discovered);
        // Transient fields should be at defaults
        assert_eq!(restored.session_kills, 0);
        assert_eq!(restored.consecutive_deaths, 0);
        assert!(restored.recent_drops.is_empty());
        assert!(restored.derived_stats_dirty);
    }
}
