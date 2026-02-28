//! Custom serde implementation for GameState that maintains flat JSON format
//! after the struct is decomposed into sub-structs.
//!
//! This module is used during the migration period where GameState fields
//! are being moved into sub-structs. It ensures saves remain compatible.

use serde::{Deserialize, Serialize};

/// Flat JSON representation matching the original GameState format.
/// Used as an intermediate for serialization/deserialization.
#[derive(Serialize, Deserialize)]
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
}
