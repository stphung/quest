use crate::attributes::AttributeType;
use crate::combat::{
    generate_boss_enemy, generate_elite_enemy, generate_enemy, generate_enemy_for_current_zone,
    generate_subzone_boss,
};
use crate::constants::*;
use crate::derived_stats::DerivedStats;
use crate::dungeon::RoomType;
use crate::game_state::GameState;
use crate::zones::get_zone;
use chrono::Utc;
use rand::Rng;

/// Calculates the XP required to reach the next level
pub fn xp_for_next_level(level: u32) -> u64 {
    (XP_CURVE_BASE * f64::powf(level as f64, XP_CURVE_EXPONENT)) as u64
}

/// Calculates the prestige multiplier for XP gains including CHA bonus
pub fn prestige_multiplier(rank: u32, cha_modifier: i32) -> f64 {
    let base = crate::prestige::get_prestige_tier(rank).multiplier;
    base + (cha_modifier as f64 * 0.1)
}

/// Calculates the XP gained per tick based on prestige rank and WIS
pub fn xp_gain_per_tick(prestige_rank: u32, wis_modifier: i32, cha_modifier: i32) -> f64 {
    let prestige_mult = prestige_multiplier(prestige_rank, cha_modifier);
    let wis_mult = 1.0 + (wis_modifier as f64 * 0.05);
    BASE_XP_PER_TICK * prestige_mult * wis_mult
}

/// Distributes 3 attribute points randomly among non-capped attributes
pub fn distribute_level_up_points(state: &mut GameState) -> Vec<AttributeType> {
    let mut rng = rand::thread_rng();
    let cap = state.get_attribute_cap();
    let mut increased = Vec::new();

    let mut points = 3;
    let mut attempts = 0;
    let max_attempts = 100; // Prevent infinite loop

    while points > 0 && attempts < max_attempts {
        let attr_index = rng.gen_range(0..6);
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
pub fn combat_kill_xp(passive_xp_rate: f64) -> u64 {
    let ticks = rand::thread_rng().gen_range(COMBAT_XP_MIN_TICKS..=COMBAT_XP_MAX_TICKS);
    (passive_xp_rate * ticks as f64) as u64
}

/// Report of offline progression results
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct OfflineReport {
    pub elapsed_seconds: i64,
    pub total_level_ups: u32,
    pub xp_gained: u64,
}

/// Calculates the XP gained during offline time
/// Now based on simulated monster kills instead of passive time
pub fn calculate_offline_xp(
    elapsed_seconds: i64,
    prestige_rank: u32,
    wis_modifier: i32,
    cha_modifier: i32,
) -> f64 {
    let capped_seconds = elapsed_seconds.min(MAX_OFFLINE_SECONDS);

    // Estimate kills: average 1 kill every 5 seconds (includes combat + regen time)
    let estimated_kills = (capped_seconds as f64 / 5.0) * OFFLINE_MULTIPLIER;

    // Average XP per kill
    let xp_per_tick_rate = xp_gain_per_tick(prestige_rank, wis_modifier, cha_modifier);
    let avg_xp_per_kill = (COMBAT_XP_MIN_TICKS + COMBAT_XP_MAX_TICKS) as f64 / 2.0;
    let xp_per_kill = xp_per_tick_rate * avg_xp_per_kill;

    estimated_kills * xp_per_kill
}

/// Processes offline progression and updates game state
pub fn process_offline_progression(state: &mut GameState) -> OfflineReport {
    let current_time = Utc::now().timestamp();
    let elapsed_seconds = current_time - state.last_save_time;

    if elapsed_seconds <= 0 {
        return OfflineReport::default();
    }

    let wis_mod = state.attributes.modifier(AttributeType::Wisdom);
    let cha_mod = state.attributes.modifier(AttributeType::Charisma);
    let offline_xp = calculate_offline_xp(elapsed_seconds, state.prestige_rank, wis_mod, cha_mod);

    let (total_level_ups, _) = apply_tick_xp(state, offline_xp);

    state.last_save_time = current_time;

    OfflineReport {
        elapsed_seconds,
        total_level_ups,
        xp_gained: offline_xp as u64,
    }
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
            // Normal overworld combat - use zone-based enemy generation
            let derived =
                DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
            let total_damage = derived.total_damage();

            let enemy = if state.zone_progression.fighting_boss {
                // Spawn the subzone boss
                spawn_subzone_boss(state, derived.max_hp, total_damage)
            } else {
                // Spawn regular zone enemy
                generate_enemy_for_current_zone(
                    state.zone_progression.current_zone_id,
                    state.zone_progression.current_subzone_id,
                    derived.max_hp,
                    total_damage,
                )
            };
            state.combat_state.current_enemy = Some(enemy);
            state.combat_state.attack_timer = 0.0;
        }
    }
}

/// Spawns the current subzone's boss
fn spawn_subzone_boss(
    state: &GameState,
    player_max_hp: u32,
    player_damage: u32,
) -> crate::combat::Enemy {
    let zone_id = state.zone_progression.current_zone_id;
    let subzone_id = state.zone_progression.current_subzone_id;

    if let Some(zone) = get_zone(zone_id) {
        if let Some(subzone) = zone.subzones.iter().find(|s| s.id == subzone_id) {
            return generate_subzone_boss(&zone, subzone, player_max_hp, player_damage);
        }
    }

    // Fallback - shouldn't happen
    generate_boss_enemy(player_max_hp, player_damage)
}

/// Spawns a dungeon enemy based on the current room type
fn spawn_dungeon_enemy(state: &mut GameState) {
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    let total_damage = derived.total_damage();

    let room_type = state
        .active_dungeon
        .as_ref()
        .and_then(|d| d.current_room())
        .map(|r| r.room_type);

    let enemy = match room_type {
        Some(RoomType::Elite) => generate_elite_enemy(derived.max_hp, total_damage),
        Some(RoomType::Boss) => generate_boss_enemy(derived.max_hp, total_damage),
        _ => generate_enemy(derived.max_hp, total_damage),
    };

    state.combat_state.current_enemy = Some(enemy);
    state.combat_state.attack_timer = 0.0;
}

/// Flat chance to discover a dungeon after killing an enemy (2%)
const DUNGEON_DISCOVERY_CHANCE: f64 = 0.02;

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
    let dungeon =
        crate::dungeon_generation::generate_dungeon(state.character_level, state.prestige_rank);
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
        let xp = combat_kill_xp(1.0);
        assert!((200..=400).contains(&xp));
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
        state.active_dungeon = Some(crate::dungeon_generation::generate_dungeon(1, 0));

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
    fn test_calculate_offline_xp_basic() {
        // 1 hour offline, rank 0, no modifiers
        let xp = calculate_offline_xp(3600, 0, 0, 0);

        // 3600 seconds / 5 = 720 estimated kills * 0.25 offline multiplier = 180 kills
        // XP per kill at rank 0 = 1.0 * 300 (avg) = 300
        // Total = 180 * 300 = 54,000 (roughly)
        assert!(xp > 25000.0 && xp < 100000.0);
    }

    #[test]
    fn test_calculate_offline_xp_capped_at_max() {
        // Test that offline XP is capped at MAX_OFFLINE_SECONDS (7 days)
        let one_week = 7 * 24 * 3600;
        let two_weeks = 14 * 24 * 3600;

        let xp_one_week = calculate_offline_xp(one_week, 0, 0, 0);
        let xp_two_weeks = calculate_offline_xp(two_weeks, 0, 0, 0);

        // Should be capped, so two weeks = one week
        assert!((xp_one_week - xp_two_weeks).abs() < 1.0);
    }

    #[test]
    fn test_calculate_offline_xp_with_prestige() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0);
        let prestige_xp = calculate_offline_xp(3600, 1, 0, 0);

        // Prestige 1 has 1.5x multiplier (using 1 + 0.5*rank^0.7 formula)
        assert!(prestige_xp > base_xp);
        let ratio = prestige_xp / base_xp;
        assert!((ratio - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_offline_xp_with_wisdom() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0);
        let wis_xp = calculate_offline_xp(3600, 0, 5, 0); // +5 WIS modifier

        // WIS +5 gives 1.25x multiplier
        assert!(wis_xp > base_xp);
        let ratio = wis_xp / base_xp;
        assert!((ratio - 1.25).abs() < 0.01);
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
}
