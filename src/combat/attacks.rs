use crate::core::constants::*;
use crate::core::game_state::GameState;
use crate::dungeon::types::RoomType;
use crate::zones::get_zone;

/// Calculates the effective enemy attack interval for the current encounter.
/// Uses fixed constants per enemy tier (game design doc values).
pub fn effective_enemy_attack_interval(state: &GameState) -> f64 {
    // Check dungeon room type first
    if let Some(dungeon) = &state.active_dungeon {
        if let Some(room) = dungeon.current_room() {
            return match room.room_type {
                RoomType::Boss => ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS,
                RoomType::Elite => ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS,
                _ => ENEMY_ATTACK_INTERVAL_SECONDS,
            };
        }
    }

    // Overworld boss
    if state.zone_progression.fighting_boss {
        // Check if this is a zone boss (last subzone of the zone).
        // Zone ids are contiguous, so use the O(1) index lookup instead of
        // scanning all 50 zones on every combat tick of a boss fight.
        let is_zone_boss = get_zone(state.zone_progression.current_zone_id).is_some_and(|zone| {
            state.zone_progression.current_subzone_id == zone.subzones.len() as u32
        });
        if is_zone_boss {
            return ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS;
        }
        return ENEMY_BOSS_ATTACK_INTERVAL_SECONDS;
    }

    // Normal mob
    ENEMY_ATTACK_INTERVAL_SECONDS
}
