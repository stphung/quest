use super::constants::*;
use super::game_state::GameState;
use crate::character::attributes::AttributeType;
use crate::character::derived_stats::DerivedStats;
use crate::combat::types::{
    generate_boss_for_current_zone, generate_dungeon_boss, generate_dungeon_elite,
    generate_dungeon_enemy, generate_enemy_for_current_zone,
};
use crate::dungeon::types::RoomType;
use rand::Rng;

// Re-export offline progression types for backwards compatibility
pub use super::offline::{calculate_offline_xp, process_offline_progression, OfflineReport};

/// Calculates the XP required to reach the next level
pub fn xp_for_next_level(level: u32) -> u64 {
    (XP_CURVE_BASE * f64::powf(level as f64, XP_CURVE_EXPONENT)) as u64
}

/// Calculates the prestige multiplier for XP gains including CHA bonus
pub fn prestige_multiplier(rank: u32, cha_modifier: i32) -> f64 {
    let base = crate::character::prestige::get_prestige_tier(rank).multiplier;
    base + (cha_modifier as f64 * PRESTIGE_MULT_PER_CHA_MODIFIER)
}

/// Calculates the XP gained per tick based on prestige rank and WIS
pub fn xp_gain_per_tick(prestige_rank: u32, wis_modifier: i32, cha_modifier: i32) -> f64 {
    let prestige_mult = prestige_multiplier(prestige_rank, cha_modifier);
    let wis_mult = 1.0 + (wis_modifier as f64 * XP_MULT_PER_WIS_MODIFIER);
    BASE_XP_PER_TICK * prestige_mult * wis_mult
}

/// Distributes 3 attribute points randomly among non-capped attributes
pub fn distribute_level_up_points(state: &mut GameState) -> Vec<AttributeType> {
    let mut rng = rand::thread_rng();
    let cap = state.get_attribute_cap();
    let mut increased = Vec::new();

    let mut points = LEVEL_UP_ATTRIBUTE_POINTS;
    let mut attempts = 0;
    let max_attempts = LEVEL_UP_MAX_DISTRIBUTION_ATTEMPTS;

    while points > 0 && attempts < max_attempts {
        let attr_index = rng.gen_range(0..NUM_ATTRIBUTES);
        let attr = AttributeType::all()[attr_index];

        if state.attributes.get(attr) < cap {
            state.attributes.increment(attr);
            increased.push(attr);
            points -= 1;
        }

        attempts += 1;
    }

    increased
}

/// Applies XP to the character and processes any level-ups
/// Returns (number of level-ups, attributes increased)
pub fn apply_tick_xp(state: &mut GameState, xp_gain: f64) -> (u32, Vec<AttributeType>) {
    state.character_xp += xp_gain as u64;

    let mut levelups = 0;
    let mut all_increased = Vec::new();

    loop {
        let xp_needed = xp_for_next_level(state.character_level);

        if state.character_xp >= xp_needed {
            state.character_xp -= xp_needed;
            state.character_level += 1;
            levelups += 1;

            let increased = distribute_level_up_points(state);
            all_increased.extend(increased);

            // Update combat state max HP after level up
            let derived =
                DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
            state.combat_state.update_max_hp(derived.max_hp);
        } else {
            break;
        }
    }

    (levelups, all_increased)
}

/// Calculates XP bonus from killing an enemy
/// `haven_xp_gain_percent` is the Training Yard bonus (0.0 if not built)
pub fn combat_kill_xp(passive_xp_rate: f64, haven_xp_gain_percent: f64) -> u64 {
    let ticks = rand::thread_rng().gen_range(COMBAT_XP_MIN_TICKS..=COMBAT_XP_MAX_TICKS);
    let base_xp = passive_xp_rate * ticks as f64;
    // Apply Haven Training Yard bonus
    (base_xp * (1.0 + haven_xp_gain_percent / 100.0)) as u64
}

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

// DUNGEON_DISCOVERY_CHANCE is imported from constants via `use super::constants::*`

/// Attempts to discover a dungeon after killing an enemy
/// Returns true if a dungeon was discovered and entered
pub fn try_discover_dungeon(state: &mut GameState) -> bool {
    // Don't discover if already in a dungeon
    if state.active_dungeon.is_some() {
        return false;
    }

    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() >= DUNGEON_DISCOVERY_CHANCE {
        return false;
    }

    // Discover dungeon!
    // Prestige affects dungeon quality (size, rewards), not discovery rate
    let zone_id = state.zone_progression.current_zone_id;
    let dungeon = crate::dungeon::generation::generate_dungeon(
        state.character_level,
        state.prestige_rank,
        zone_id,
    );
    state.active_dungeon = Some(dungeon);

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_for_next_level() {
        assert_eq!(xp_for_next_level(1), 100);
        assert_eq!(xp_for_next_level(2), 282);
        assert_eq!(xp_for_next_level(10), 3162);
    }

    #[test]
    fn test_prestige_multiplier() {
        // New formula: base = 1 + 0.5 * rank^0.7, then add CHA bonus

        // Rank 0, CHA 10 (+0): 1.0 + 0 = 1.0
        assert_eq!(prestige_multiplier(0, 0), 1.0);

        // Rank 1, CHA 10 (+0): 1.5 + 0 = 1.5 (using 1 + 0.5*rank^0.7 formula)
        assert_eq!(prestige_multiplier(1, 0), 1.5);

        // Rank 1, CHA 16 (+3): 1.5 + 0.3 = 1.8
        assert_eq!(prestige_multiplier(1, 3), 1.8);
    }

    #[test]
    fn test_xp_gain_per_tick() {
        // Rank 0, WIS 10 (+0), CHA 10 (+0): 1.0 * 1.0 * 1.0 = 1.0
        assert_eq!(xp_gain_per_tick(0, 0, 0), 1.0);

        // Rank 1, WIS 20 (+5), CHA 16 (+3): 1.8 * 1.25 = 2.25
        assert_eq!(xp_gain_per_tick(1, 5, 3), 2.25);
    }

    #[test]
    fn test_distribute_level_up_points() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let increased = distribute_level_up_points(&mut state);

        // Should distribute 3 points
        assert_eq!(increased.len(), 3);

        // Total attribute sum should be 60 + 3 = 63
        let mut sum = 0;
        for attr in AttributeType::all() {
            sum += state.attributes.get(attr);
        }
        assert_eq!(sum, 63);
    }

    #[test]
    fn test_distribute_respects_caps() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set all attributes to cap - 1 (prestige 0 = cap 20)
        for attr in AttributeType::all() {
            state.attributes.set(attr, 19);
        }

        let increased = distribute_level_up_points(&mut state);
        assert_eq!(increased.len(), 3);

        // All should be at cap now (20)
        for attr in increased {
            assert!(state.attributes.get(attr) <= 20);
        }
    }

    #[test]
    fn test_apply_tick_xp_no_levelup() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let (levelups, increased) = apply_tick_xp(&mut state, 50.0);

        assert_eq!(levelups, 0);
        assert_eq!(increased.len(), 0);
        assert_eq!(state.character_level, 1);
        assert_eq!(state.character_xp, 50);
    }

    #[test]
    fn test_apply_tick_xp_single_levelup() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let (levelups, increased) = apply_tick_xp(&mut state, 100.0);

        assert_eq!(levelups, 1);
        assert_eq!(increased.len(), 3);
        assert_eq!(state.character_level, 2);
        assert_eq!(state.character_xp, 0);
    }

    #[test]
    fn test_combat_kill_xp() {
        let xp = combat_kill_xp(1.0, 0.0);
        assert!((200..=400).contains(&xp));
    }

    #[test]
    fn test_combat_kill_xp_with_haven_bonus() {
        // Run many trials to verify average XP is higher with bonus
        let mut total_no_bonus = 0u64;
        let mut total_with_bonus = 0u64;
        let trials = 1000;

        for _ in 0..trials {
            total_no_bonus += combat_kill_xp(1.0, 0.0);
            total_with_bonus += combat_kill_xp(1.0, 30.0); // +30% XP from Training Yard
        }

        let avg_no_bonus = total_no_bonus as f64 / trials as f64;
        let avg_with_bonus = total_with_bonus as f64 / trials as f64;
        let ratio = avg_with_bonus / avg_no_bonus;

        // Should be approximately 30% higher
        assert!(
            (1.25..=1.35).contains(&ratio),
            "Haven +30% XP should increase average XP by ~30%, got {:.2}x",
            ratio
        );
    }

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

        // Spawn first enemy
        spawn_enemy_if_needed(&mut state);
        let first_enemy_hp = state.combat_state.current_enemy.as_ref().unwrap().max_hp;

        // Try to spawn again - should keep the same enemy
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

        // Should not spawn while regenerating
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_spawn_enemy_spawns_boss_when_fighting_boss() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.fighting_boss = true;

        spawn_enemy_if_needed(&mut state);

        // Should have spawned a boss enemy
        assert!(state.combat_state.current_enemy.is_some());
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        // Boss enemies have higher stats - just verify it exists
        assert!(enemy.max_hp > 0);
    }

    #[test]
    fn test_try_discover_dungeon_skips_when_in_dungeon() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Already in a dungeon
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0, 1));

        // Should never discover a new dungeon while in one
        for _ in 0..100 {
            assert!(!try_discover_dungeon(&mut state));
        }
    }

    #[test]
    fn test_try_discover_dungeon_probability() {
        // Test that dungeon discovery happens with expected probability (2%)
        // Run many trials and check it's in reasonable range
        let mut discoveries = 0;
        let trials = 10000;

        for _ in 0..trials {
            let mut state = GameState::new("Test Hero".to_string(), 0);
            if try_discover_dungeon(&mut state) {
                discoveries += 1;
            }
        }

        // 2% rate = 200 expected discoveries in 10000 trials
        // Allow reasonable variance (1% to 4% = 100 to 400)
        assert!(
            (100..=400).contains(&discoveries),
            "Expected ~200 discoveries (2%), got {}",
            discoveries
        );
    }

    #[test]
    fn test_try_discover_dungeon_creates_valid_dungeon() {
        // Keep trying until we discover a dungeon
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.character_level = 10;
        state.prestige_rank = 1;

        // Force a discovery by trying many times
        let mut discovered = false;
        for _ in 0..1000 {
            if try_discover_dungeon(&mut state) {
                discovered = true;
                break;
            }
            state.active_dungeon = None; // Reset for next try
        }

        if discovered {
            let dungeon = state.active_dungeon.as_ref().unwrap();
            // Verify dungeon has a valid grid
            assert!(!dungeon.grid.is_empty());
            // Player position should be at entrance
            assert_eq!(dungeon.player_position, dungeon.entrance_position);
        }
    }

    #[test]
    fn test_apply_tick_xp_multiple_levelups() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Give enough XP for multiple level ups
        // Level 1->2: 100, Level 2->3: 282, Total: 382
        let (levelups, increased) = apply_tick_xp(&mut state, 400.0);

        assert_eq!(levelups, 2);
        assert_eq!(increased.len(), 6); // 3 points per level * 2 levels
        assert_eq!(state.character_level, 3);
    }

    #[test]
    fn test_xp_for_next_level_scaling() {
        // Verify XP curve increases with level
        let xp_1 = xp_for_next_level(1);
        let xp_5 = xp_for_next_level(5);
        let xp_10 = xp_for_next_level(10);
        let xp_50 = xp_for_next_level(50);

        assert!(xp_1 < xp_5);
        assert!(xp_5 < xp_10);
        assert!(xp_10 < xp_50);
    }

    #[test]
    fn test_prestige_multiplier_negative_charisma() {
        // CHA below 10 gives negative modifier
        let mult = prestige_multiplier(1, -2); // CHA 6 = -2 modifier
                                               // 1.5 + (-0.2) = 1.3
        assert_eq!(mult, 1.3);
    }

    #[test]
    fn test_distribute_when_all_at_cap() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set all attributes to cap
        for attr in AttributeType::all() {
            state.attributes.set(attr, 20);
        }

        let increased = distribute_level_up_points(&mut state);

        // Should return empty since no points could be distributed
        assert!(increased.len() < 3); // May distribute some if loop hasn't hit max attempts
    }

    // =========================================================================
    // DUNGEON ROOM TYPE SPAWNING TESTS
    // =========================================================================

    /// Helper: create a minimal dungeon with the player in a room of the given type.
    /// Returns a GameState with an active dungeon where current_room_cleared = false
    /// so spawn_enemy_if_needed will attempt to spawn.
    fn setup_dungeon_with_room_type(room_type: RoomType) -> GameState {
        use crate::dungeon::types::{Dungeon, DungeonSize, Room, RoomState};

        let mut state = GameState::new("Dungeon Tester".to_string(), 0);
        state.character_level = 10;

        // Build a minimal 5x5 dungeon with one room at center
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

        // Regular dungeon enemies do NOT have "Elite" or "Boss" prefix
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
        // When current_room_cleared is true, no enemy should be spawned
        // even for combat room types
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);

        // Mark room as already cleared
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
        // Use sampling to handle random variance in stat generation
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
        // Use sampling to handle random variance in stat generation
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
        // Enemies in higher zones should have more HP
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
        // Enemy damage in higher zones should be higher
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
        // Verify that dungeon enemy spawning uses zone-based scaling
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);

        // Set dungeon zone_id to a known value
        state.active_dungeon.as_mut().unwrap().zone_id = 5;

        spawn_enemy_if_needed(&mut state);

        let enemy = state
            .combat_state
            .current_enemy
            .as_ref()
            .expect("Should have spawned enemy");

        // Enemy stats should be near zone 5 base stats (with ±10% random variance)
        // zone_base_stats uses zone_id - 1 as index
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
        // If an enemy already exists, spawn_enemy_if_needed should not replace it
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);

        // Manually place an enemy
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
        // During HP regen phase, no enemy should be spawned
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
        // When a new dungeon enemy is spawned, both attack timers should be reset to 0
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        state.combat_state.player_attack_timer = 5.0; // Non-zero
        state.combat_state.enemy_attack_timer = 3.0; // Non-zero

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
}
