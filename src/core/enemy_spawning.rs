use super::constants::*;
use super::game_state::GameState;
use crate::combat::types::{
    generate_boss_for_current_zone, generate_dungeon_boss, generate_dungeon_elite,
    generate_dungeon_enemy, generate_enemy_for_current_zone,
};
use crate::dungeon::types::RoomType;

/// Spawns a new enemy if none exists
pub fn spawn_enemy_if_needed(state: &mut GameState) {
    if state.combat_state.current_enemy.is_none() && !state.combat_state.is_regenerating {
        // Check if we're in a dungeon
        if let Some(dungeon) = &state.active_dungeon {
            // Don't spawn if room combat is already complete
            if dungeon.current_room_cleared {
                return;
            }

            if let Some(room) = dungeon.current_room() {
                // Only spawn in combat rooms
                match room.room_type {
                    RoomType::Combat | RoomType::Elite | RoomType::Boss => {
                        spawn_dungeon_enemy(state);
                    }
                    _ => {} // No enemies in entrance/treasure rooms
                }
            }
        } else {
            // Normal overworld combat - use zone-based static enemy generation
            let zone_id = state.zone_progression.current_zone_id;
            let subzone_id = state.zone_progression.current_subzone_id;
            let enemy = if state.zone_progression.fighting_boss {
                generate_boss_for_current_zone(zone_id, subzone_id)
            } else {
                generate_enemy_for_current_zone(zone_id, subzone_id)
            };
            state.combat_state.current_enemy = Some(enemy);
            state.combat_state.player_attack_timer = 0.0;
            state.combat_state.enemy_attack_timer = 0.0;
        }
    }
}

/// Spawns a dungeon enemy based on the current room type using zone-based stats.
fn spawn_dungeon_enemy(state: &mut GameState) {
    let dungeon_zone_id = state.active_dungeon.as_ref().map_or(1, |d| d.zone_id);

    let room_type = state
        .active_dungeon
        .as_ref()
        .and_then(|d| d.current_room())
        .map(|r| r.room_type);

    let enemy = match room_type {
        Some(RoomType::Elite) => generate_dungeon_elite(dungeon_zone_id),
        Some(RoomType::Boss) => generate_dungeon_boss(dungeon_zone_id),
        _ => generate_dungeon_enemy(dungeon_zone_id),
    };

    state.combat_state.current_enemy = Some(enemy);
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_enemy_if_needed() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        assert!(state.combat_state.current_enemy.is_none());

        spawn_enemy_if_needed(&mut state);
        assert!(state.combat_state.current_enemy.is_some());

        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert!(!enemy.name.is_empty());
        assert!(enemy.max_hp > 0);
    }

    #[test]
    fn test_spawn_enemy_skips_when_enemy_exists() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        spawn_enemy_if_needed(&mut state);
        let first_enemy_hp = state.combat_state.current_enemy.as_ref().unwrap().max_hp;

        spawn_enemy_if_needed(&mut state);
        assert_eq!(
            state.combat_state.current_enemy.as_ref().unwrap().max_hp,
            first_enemy_hp
        );
    }

    #[test]
    fn test_spawn_enemy_skips_when_regenerating() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.is_regenerating = true;

        spawn_enemy_if_needed(&mut state);
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_spawn_enemy_spawns_boss_when_fighting_boss() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.fighting_boss = true;

        spawn_enemy_if_needed(&mut state);

        assert!(state.combat_state.current_enemy.is_some());
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert!(enemy.max_hp > 0);
    }

    // =========================================================================
    // DUNGEON ROOM TYPE SPAWNING TESTS
    // =========================================================================

    fn setup_dungeon_with_room_type(room_type: RoomType) -> GameState {
        use crate::dungeon::types::{Dungeon, DungeonSize, Room, RoomState};

        let mut state = GameState::new("Dungeon Tester".to_string(), 0);
        state.character_level = 10;

        let mut dungeon = Dungeon::new(DungeonSize::Small);
        let pos = (2, 2);

        let mut room = Room::new(room_type, pos);
        room.state = RoomState::Current;
        dungeon.grid[pos.1][pos.0] = Some(room);
        dungeon.player_position = pos;
        dungeon.entrance_position = pos;
        dungeon.boss_position = pos;
        dungeon.current_room_cleared = false;

        state.active_dungeon = Some(dungeon);
        state
    }

    #[test]
    fn test_spawn_dungeon_enemy_combat_room_spawns_regular() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);

        spawn_enemy_if_needed(&mut state);

        let enemy = state
            .combat_state
            .current_enemy
            .as_ref()
            .expect("Combat room should spawn an enemy");

        assert!(
            !enemy.name.starts_with("Elite "),
            "Combat room should spawn regular enemy, got: {}",
            enemy.name
        );
        assert!(
            !enemy.name.starts_with("Boss "),
            "Combat room should spawn regular enemy, got: {}",
            enemy.name
        );
        assert!(enemy.max_hp > 0);
        assert!(enemy.damage > 0);
    }

    #[test]
    fn test_spawn_dungeon_enemy_elite_room_spawns_elite() {
        let mut state = setup_dungeon_with_room_type(RoomType::Elite);

        spawn_enemy_if_needed(&mut state);

        let enemy = state
            .combat_state
            .current_enemy
            .as_ref()
            .expect("Elite room should spawn an enemy");

        assert!(
            enemy.name.starts_with("Elite "),
            "Elite room should spawn elite enemy, got: {}",
            enemy.name
        );
        assert!(enemy.max_hp > 0);
        assert!(enemy.damage > 0);
    }

    #[test]
    fn test_spawn_dungeon_enemy_boss_room_spawns_boss() {
        let mut state = setup_dungeon_with_room_type(RoomType::Boss);

        spawn_enemy_if_needed(&mut state);

        let enemy = state
            .combat_state
            .current_enemy
            .as_ref()
            .expect("Boss room should spawn an enemy");

        assert!(
            enemy.name.starts_with("Boss "),
            "Boss room should spawn boss enemy, got: {}",
            enemy.name
        );
        assert!(enemy.max_hp > 0);
        assert!(enemy.damage > 0);
    }

    #[test]
    fn test_spawn_dungeon_enemy_entrance_does_not_spawn() {
        let mut state = setup_dungeon_with_room_type(RoomType::Entrance);

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_none(),
            "Entrance room should NOT spawn an enemy"
        );
    }

    #[test]
    fn test_spawn_dungeon_enemy_treasure_does_not_spawn() {
        let mut state = setup_dungeon_with_room_type(RoomType::Treasure);

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_none(),
            "Treasure room should NOT spawn an enemy"
        );
    }

    #[test]
    fn test_spawn_enemy_if_needed_respects_current_room_cleared() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        state.active_dungeon.as_mut().unwrap().current_room_cleared = true;

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_none(),
            "Should not spawn enemy when current_room_cleared is true"
        );
    }

    #[test]
    fn test_spawn_enemy_if_needed_cleared_elite_no_spawn() {
        let mut state = setup_dungeon_with_room_type(RoomType::Elite);
        state.active_dungeon.as_mut().unwrap().current_room_cleared = true;

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_none(),
            "Should not spawn elite enemy when room is already cleared"
        );
    }

    #[test]
    fn test_spawn_enemy_if_needed_cleared_boss_no_spawn() {
        let mut state = setup_dungeon_with_room_type(RoomType::Boss);
        state.active_dungeon.as_mut().unwrap().current_room_cleared = true;

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_none(),
            "Should not spawn boss enemy when room is already cleared"
        );
    }

    #[test]
    fn test_dungeon_elite_stats_higher_than_regular() {
        let zone_id = 5;
        let samples = 50;
        let mut elite_hp = 0u64;
        let mut regular_hp = 0u64;

        for _ in 0..samples {
            let regular = generate_dungeon_enemy(zone_id);
            let elite = generate_dungeon_elite(zone_id);
            elite_hp += elite.max_hp as u64;
            regular_hp += regular.max_hp as u64;
        }

        assert!(
            elite_hp > regular_hp,
            "Average elite HP should exceed average regular HP"
        );
    }

    #[test]
    fn test_dungeon_boss_stats_higher_than_elite() {
        let zone_id = 5;
        let samples = 50;
        let mut boss_hp = 0u64;
        let mut elite_hp = 0u64;

        for _ in 0..samples {
            let elite = generate_dungeon_elite(zone_id);
            let boss = generate_dungeon_boss(zone_id);
            boss_hp += boss.max_hp as u64;
            elite_hp += elite.max_hp as u64;
        }

        assert!(
            boss_hp > elite_hp,
            "Average boss HP should exceed average elite HP"
        );
    }

    #[test]
    fn test_dungeon_enemy_stats_scale_with_zone() {
        let low_zone = generate_dungeon_enemy(1);
        let high_zone = generate_dungeon_enemy(10);

        assert!(
            high_zone.max_hp > low_zone.max_hp,
            "Zone 10 enemy HP {} should exceed zone 1 enemy HP {}",
            high_zone.max_hp,
            low_zone.max_hp
        );
    }

    #[test]
    fn test_dungeon_enemy_damage_scales_with_zone() {
        let low_zone = generate_dungeon_enemy(1);
        let high_zone = generate_dungeon_enemy(10);

        assert!(
            high_zone.damage > low_zone.damage,
            "Zone 10 enemy damage {} should exceed zone 1 enemy damage {}",
            high_zone.damage,
            low_zone.damage
        );
    }

    #[test]
    fn test_spawn_dungeon_enemy_uses_zone_scaling() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        state.active_dungeon.as_mut().unwrap().zone_id = 5;

        spawn_enemy_if_needed(&mut state);

        let enemy = state
            .combat_state
            .current_enemy
            .as_ref()
            .expect("Should have spawned enemy");

        let (base_hp, _, base_dmg, _, _, _) = ZONE_ENEMY_STATS[4];
        let hp_lo = (base_hp as f64 * 0.85) as u32;
        let hp_hi = (base_hp as f64 * 1.15) as u32;
        assert!(
            enemy.max_hp >= hp_lo && enemy.max_hp <= hp_hi,
            "Dungeon enemy HP {} should be near zone 5 base HP {} (range {}-{})",
            enemy.max_hp,
            base_hp,
            hp_lo,
            hp_hi
        );
        let dmg_lo = (base_dmg as f64 * 0.85) as u32;
        let dmg_hi = (base_dmg as f64 * 1.15) as u32;
        assert!(
            enemy.damage >= dmg_lo && enemy.damage <= dmg_hi,
            "Dungeon enemy damage {} should be near zone 5 base damage {} (range {}-{})",
            enemy.damage,
            base_dmg,
            dmg_lo,
            dmg_hi
        );
    }

    #[test]
    fn test_spawn_dungeon_enemy_does_not_overwrite_existing() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);

        let sentinel = crate::combat::types::Enemy::new("Sentinel".to_string(), 9999, 1);
        state.combat_state.current_enemy = Some(sentinel);

        spawn_enemy_if_needed(&mut state);

        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(
            enemy.name, "Sentinel",
            "Should not overwrite existing enemy"
        );
        assert_eq!(enemy.max_hp, 9999);
    }

    #[test]
    fn test_spawn_dungeon_enemy_skips_when_regenerating() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        state.combat_state.is_regenerating = true;

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_none(),
            "Should not spawn enemy while regenerating"
        );
    }

    #[test]
    fn test_spawn_dungeon_enemy_resets_attack_timers() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        state.combat_state.player_attack_timer = 5.0;
        state.combat_state.enemy_attack_timer = 3.0;

        spawn_enemy_if_needed(&mut state);

        assert!(
            state.combat_state.current_enemy.is_some(),
            "Should have spawned enemy"
        );
        assert_eq!(
            state.combat_state.player_attack_timer, 0.0,
            "Player attack timer should be reset to 0 on new enemy spawn"
        );
        assert_eq!(
            state.combat_state.enemy_attack_timer, 0.0,
            "Enemy attack timer should be reset to 0 on new enemy spawn"
        );
    }

    // =========================================================================
    // ENEMY SPAWNING WITH ZONE/PRESTIGE SCALING
    // =========================================================================

    #[test]
    fn test_spawn_enemy_at_zone_10() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 1;
        state.zone_progression.unlock_zone(10);

        spawn_enemy_if_needed(&mut state);

        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        let (base_hp, _, _, _, _, _) = ZONE_ENEMY_STATS[9];
        assert!(
            enemy.max_hp >= (base_hp as f64 * 0.85) as u32,
            "Zone 10 enemy HP {} should be near base {}",
            enemy.max_hp,
            base_hp
        );
    }

    #[test]
    fn test_spawn_enemy_at_zone_11() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.current_zone_id = 11;
        state.zone_progression.current_subzone_id = 1;
        state.zone_progression.unlock_zone(11);

        spawn_enemy_if_needed(&mut state);

        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        let (base_hp, _, _, _, _, _) = ZONE_ENEMY_STATS[10];
        assert!(
            enemy.max_hp >= (base_hp as f64 * 0.85) as u32,
            "Zone 11 enemy HP {} should be near base {}",
            enemy.max_hp,
            base_hp
        );
    }
}
