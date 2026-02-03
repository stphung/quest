use crate::attributes::AttributeType;
use crate::combat::{
    generate_boss_enemy, generate_elite_enemy, generate_enemy, generate_enemy_for_current_zone,
};
use crate::constants::*;
use crate::derived_stats::DerivedStats;
use crate::dungeon::RoomType;
use crate::game_state::GameState;
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

            let enemy = generate_enemy_for_current_zone(
                state.zone_progression.current_zone_id,
                state.zone_progression.current_subzone_id,
                derived.max_hp,
                total_damage,
            );
            state.combat_state.current_enemy = Some(enemy);
            state.combat_state.attack_timer = 0.0;
        }
    }
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

/// Flat chance to discover a dungeon after killing an enemy (5%)
const DUNGEON_DISCOVERY_CHANCE: f64 = 0.05;

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
        // Rank 0, CHA 10 (+0): 1.0 + 0 = 1.0
        assert_eq!(prestige_multiplier(0, 0), 1.0);

        // Rank 1, CHA 10 (+0): 1.2 + 0 = 1.2 (using 1.2^rank formula)
        assert_eq!(prestige_multiplier(1, 0), 1.2);

        // Rank 1, CHA 16 (+3): 1.2 + 0.3 = 1.5
        assert_eq!(prestige_multiplier(1, 3), 1.5);
    }

    #[test]
    fn test_xp_gain_per_tick() {
        // Rank 0, WIS 10 (+0), CHA 10 (+0): 1.0 * 1.0 * 1.0 = 1.0
        assert_eq!(xp_gain_per_tick(0, 0, 0), 1.0);

        // Rank 1, WIS 20 (+5), CHA 16 (+3): 1.5 * 1.25 = 1.875
        assert_eq!(xp_gain_per_tick(1, 5, 3), 1.875);
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
}
