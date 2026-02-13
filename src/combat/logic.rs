use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::core::constants::*;
use crate::core::game_state::GameState;
use crate::dungeon::types::RoomType;
use crate::zones::get_all_zones;
use rand::Rng;

use crate::zones::BossDefeatResult;

/// Haven bonuses that affect combat
#[derive(Debug, Clone, Default)]
pub struct HavenCombatBonuses {
    /// Alchemy Lab: +% HP regen speed
    pub hp_regen_percent: f64,
    /// Bedroom: -% HP regen delay (reduces wait time before regen starts)
    pub hp_regen_delay_reduction: f64,
    /// Armory: +% damage
    pub damage_percent: f64,
    /// Watchtower: +% crit chance
    pub crit_chance_percent: f64,
    /// War Room: +% chance to strike twice
    pub double_strike_chance: f64,
    /// Training Yard: +% XP from kills
    pub xp_gain_percent: f64,
}

pub enum CombatEvent {
    PlayerAttack {
        damage: u32,
        was_crit: bool,
    },
    /// Player's attack was blocked because boss requires a weapon
    PlayerAttackBlocked {
        weapon_needed: String,
    },
    EnemyAttack {
        damage: u32,
    },
    PlayerDied,
    /// Player died while in a dungeon (no prestige loss)
    PlayerDiedInDungeon,
    EnemyDied {
        xp_gained: u64,
    },
    /// Elite enemy defeated in dungeon (player gets key)
    EliteDefeated {
        xp_gained: u64,
    },
    /// Boss enemy defeated in dungeon (dungeon complete)
    BossDefeated {
        xp_gained: u64,
    },
    /// Subzone boss defeated (zone progression)
    SubzoneBossDefeated {
        xp_gained: u64,
        result: BossDefeatResult,
    },
}

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
        // Check if this is a zone boss (last subzone of the zone)
        let zones = get_all_zones();
        let is_zone_boss = zones
            .iter()
            .find(|z| z.id == state.zone_progression.current_zone_id)
            .is_some_and(|zone| {
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

/// Updates combat state, returns events that occurred
/// `haven` contains all Haven bonuses that affect combat
/// `prestige_bonuses` contains flat combat bonuses from prestige rank
/// `achievements` is used to check for Stormbreaker achievement (Zone 10 boss)
pub fn update_combat(
    state: &mut GameState,
    delta_time: f64,
    haven: &HavenCombatBonuses,
    prestige_bonuses: &PrestigeCombatBonuses,
    achievements: &mut crate::achievements::Achievements,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // Handle regeneration after enemy death
    if state.combat_state.is_regenerating {
        // HP regen multiplier: higher = faster regen (equipment + haven bonus)
        let regen_derived =
            DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let total_regen_multiplier =
            regen_derived.hp_regen_multiplier * (1.0 + haven.hp_regen_percent / 100.0);

        // Apply Bedroom bonus: reduce base regen duration
        let base_regen_duration =
            HP_REGEN_DURATION_SECONDS * (1.0 - haven.hp_regen_delay_reduction / 100.0);
        let effective_regen_duration = base_regen_duration / total_regen_multiplier;

        state.combat_state.regen_timer += delta_time;

        if state.combat_state.regen_timer >= effective_regen_duration {
            state.combat_state.player_current_hp = state.combat_state.player_max_hp;
            state.combat_state.is_regenerating = false;
            state.combat_state.regen_timer = 0.0;
        } else {
            // Gradual regen
            let regen_progress = state.combat_state.regen_timer / effective_regen_duration;
            let start_hp = state.combat_state.player_current_hp;
            let target_hp = state.combat_state.player_max_hp;
            state.combat_state.player_current_hp =
                start_hp + ((target_hp - start_hp) as f64 * regen_progress) as u32;
        }
        return events;
    }

    // No combat if no enemy
    if state.combat_state.current_enemy.is_none() {
        return events;
    }

    // --- Phase 1: Accumulate both timers ---
    state.combat_state.player_attack_timer += delta_time;
    state.combat_state.enemy_attack_timer += delta_time;

    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);

    // Attack speed multiplier: higher = faster attacks
    let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
    let enemy_interval = effective_enemy_attack_interval(state);

    // --- Phase 2: Determine who attacks this tick ---
    let player_attacks = state.combat_state.player_attack_timer >= player_interval;
    let enemy_attacks = state.combat_state.enemy_attack_timer >= enemy_interval;

    // --- Phase 3: Player attack (if ready) ---
    if player_attacks {
        state.combat_state.player_attack_timer = 0.0;

        // Check if boss requires a weapon we don't have
        if let Some(weapon_name) = state.zone_progression.boss_weapon_blocked(achievements) {
            // Attack is blocked - no damage dealt
            events.push(CombatEvent::PlayerAttackBlocked {
                weapon_needed: weapon_name.to_string(),
            });
        } else {
            // Player attacks normally
            // 1. Base damage from DerivedStats (STR/INT + equipment)
            let base_damage = derived.total_damage();
            // 2. Apply Haven Armory multiplier: +% damage
            let haven_damage = (base_damage as f64 * (1.0 + haven.damage_percent / 100.0)) as u32;
            // 3. Apply prestige flat damage (added after Haven %, before crit)
            let pre_crit_damage = haven_damage + prestige_bonuses.flat_damage;
            // 4. Apply enemy defense: min damage floor of 1
            let enemy_def = state
                .combat_state
                .current_enemy
                .as_ref()
                .map_or(0, |e| e.defense);
            let mut damage = pre_crit_damage.saturating_sub(enemy_def).max(1);
            let mut was_crit = false;

            // Roll for crit (base + Haven Watchtower + prestige crit)
            let total_crit_chance = derived.crit_chance_percent
                + haven.crit_chance_percent as u32
                + prestige_bonuses.crit_chance as u32;
            let crit_roll = rand::thread_rng().gen_range(0..100);
            if crit_roll < total_crit_chance {
                damage = (damage as f64 * derived.crit_multiplier) as u32;
                was_crit = true;
            }

            // Roll for double strike (War Room bonus)
            let double_strike_roll = rand::thread_rng().gen::<f64>() * 100.0;
            let num_strikes = if double_strike_roll < haven.double_strike_chance {
                2
            } else {
                1
            };

            if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                // Apply damage (potentially multiple times with double strike)
                for strike in 0..num_strikes {
                    if !enemy.is_alive() {
                        break; // Enemy already dead
                    }
                    enemy.take_damage(damage);
                    // Only first strike uses original crit flag, subsequent strikes are bonus hits
                    let strike_crit = if strike == 0 { was_crit } else { false };
                    events.push(CombatEvent::PlayerAttack {
                        damage,
                        was_crit: strike_crit,
                    });
                }

                // Check if enemy died
                if !enemy.is_alive() {
                    let wis_mod = state
                        .attributes
                        .modifier(crate::character::attributes::AttributeType::Wisdom);
                    let cha_mod = state
                        .attributes
                        .modifier(crate::character::attributes::AttributeType::Charisma);
                    let xp_gained = crate::core::game_logic::combat_kill_xp(
                        crate::core::game_logic::xp_gain_per_tick(
                            state.prestige_rank,
                            wis_mod,
                            cha_mod,
                        ),
                        haven.xp_gain_percent,
                    );

                    // Check if we're in a dungeon and what type of room
                    let dungeon_room_type = state
                        .active_dungeon
                        .as_ref()
                        .and_then(|d| d.current_room())
                        .map(|r| r.room_type);

                    // Track if this was a boss-level kill for achievements
                    let is_boss_kill = matches!(
                        dungeon_room_type,
                        Some(RoomType::Elite) | Some(RoomType::Boss)
                    ) || (state.active_dungeon.is_none()
                        && state.zone_progression.fighting_boss);

                    match dungeon_room_type {
                        Some(RoomType::Elite) => {
                            events.push(CombatEvent::EliteDefeated { xp_gained });
                        }
                        Some(RoomType::Boss) => {
                            events.push(CombatEvent::BossDefeated { xp_gained });
                        }
                        _ => {
                            if state.active_dungeon.is_some() {
                                // Dungeon Combat room kill — don't affect zone progression
                                events.push(CombatEvent::EnemyDied { xp_gained });
                            } else if state.zone_progression.fighting_boss {
                                // Overworld boss defeated
                                let result = state
                                    .zone_progression
                                    .on_boss_defeated(state.prestige_rank, achievements);
                                events.push(CombatEvent::SubzoneBossDefeated { xp_gained, result });
                            } else {
                                // Record the kill for boss spawn tracking (boss flag set if threshold reached)
                                state.zone_progression.record_kill();
                                events.push(CombatEvent::EnemyDied { xp_gained });
                            }
                        }
                    }

                    // Track kill for achievements
                    achievements.on_enemy_killed(is_boss_kill, Some(&state.character_name));

                    // Remove enemy and start regeneration
                    state.combat_state.current_enemy = None;
                    state.combat_state.enemy_attack_timer = 0.0;
                    state.combat_state.is_regenerating = true;
                    state.combat_state.regen_timer = 0.0;

                    return events;
                }
            }
        }
    }

    // --- Phase 4: Enemy attack (if ready) ---
    if enemy_attacks {
        state.combat_state.enemy_attack_timer = 0.0;

        if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
            let total_defense = derived.defense + prestige_bonuses.flat_defense;
            let enemy_damage = enemy.damage.saturating_sub(total_defense).max(1);
            state.combat_state.player_current_hp = state
                .combat_state
                .player_current_hp
                .saturating_sub(enemy_damage);

            events.push(CombatEvent::EnemyAttack {
                damage: enemy_damage,
            });

            // Damage reflection: reflect percentage of damage taken back to attacker
            if derived.damage_reflection_percent > 0.0 && enemy_damage > 0 {
                let reflected =
                    (enemy_damage as f64 * derived.damage_reflection_percent / 100.0) as u32;
                if reflected > 0 {
                    enemy.take_damage(reflected);
                }
            }

            // Check if reflection killed the enemy
            if !enemy.is_alive() {
                let wis_mod = state
                    .attributes
                    .modifier(crate::character::attributes::AttributeType::Wisdom);
                let cha_mod = state
                    .attributes
                    .modifier(crate::character::attributes::AttributeType::Charisma);
                let xp_gained = crate::core::game_logic::combat_kill_xp(
                    crate::core::game_logic::xp_gain_per_tick(
                        state.prestige_rank,
                        wis_mod,
                        cha_mod,
                    ),
                    haven.xp_gain_percent,
                );

                let dungeon_room_type = state
                    .active_dungeon
                    .as_ref()
                    .and_then(|d| d.current_room())
                    .map(|r| r.room_type);

                let is_boss_kill = matches!(
                    dungeon_room_type,
                    Some(RoomType::Elite) | Some(RoomType::Boss)
                ) || (state.active_dungeon.is_none()
                    && state.zone_progression.fighting_boss);

                match dungeon_room_type {
                    Some(RoomType::Elite) => {
                        events.push(CombatEvent::EliteDefeated { xp_gained });
                    }
                    Some(RoomType::Boss) => {
                        events.push(CombatEvent::BossDefeated { xp_gained });
                    }
                    _ => {
                        if state.active_dungeon.is_some() {
                            events.push(CombatEvent::EnemyDied { xp_gained });
                        } else if state.zone_progression.fighting_boss {
                            let result = state
                                .zone_progression
                                .on_boss_defeated(state.prestige_rank, achievements);
                            events.push(CombatEvent::SubzoneBossDefeated { xp_gained, result });
                        } else {
                            state.zone_progression.record_kill();
                            events.push(CombatEvent::EnemyDied { xp_gained });
                        }
                    }
                }

                achievements.on_enemy_killed(is_boss_kill, Some(&state.character_name));

                state.combat_state.current_enemy = None;
                state.combat_state.is_regenerating = true;
                state.combat_state.regen_timer = 0.0;

                return events;
            }

            // Check if player died
            if !state.combat_state.is_player_alive() {
                // Check if we're in a dungeon
                let in_dungeon = state.active_dungeon.is_some();

                if in_dungeon {
                    events.push(CombatEvent::PlayerDiedInDungeon);

                    // Exit dungeon - no prestige loss
                    state.active_dungeon = None;
                } else {
                    events.push(CombatEvent::PlayerDied);
                }

                // Reset player HP (in dungeon or not)
                state.combat_state.player_current_hp = state.combat_state.player_max_hp;

                // Reset both timers on player death
                state.combat_state.player_attack_timer = 0.0;
                state.combat_state.enemy_attack_timer = 0.0;

                // Reset enemy HP if we're not in dungeon (normal combat continues)
                if !in_dungeon {
                    // Check if we died to a boss
                    if state.zone_progression.fighting_boss {
                        // Reset boss encounter but preserve kill counter
                        // Boss respawns after KILLS_FOR_BOSS_RETRY kills (reduced penalty)
                        state.zone_progression.fighting_boss = false;
                        state.zone_progression.kills_in_subzone =
                            KILLS_FOR_BOSS.saturating_sub(KILLS_FOR_BOSS_RETRY);
                        state.combat_state.current_enemy = None;
                    } else if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                        enemy.reset_hp();
                    }
                } else {
                    // In dungeon, clear the enemy since we're exiting
                    state.combat_state.current_enemy = None;
                }
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::super::types::{
        generate_dungeon_boss, generate_dungeon_elite, generate_dungeon_enemy, generate_zone_enemy,
        CombatState, Enemy,
    };
    use super::*;
    use crate::achievements::Achievements;

    // =========================================================================
    // Test Helpers
    // =========================================================================

    fn default_prestige() -> PrestigeCombatBonuses {
        PrestigeCombatBonuses::default()
    }

    /// Forces a player attack by setting the player timer, suppressing enemy attack.
    fn force_player_attack(
        state: &mut GameState,
        haven: &HavenCombatBonuses,
        achievements: &mut Achievements,
    ) -> Vec<CombatEvent> {
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        state.combat_state.enemy_attack_timer = 0.0;
        update_combat(state, 0.1, haven, &default_prestige(), achievements)
    }

    /// Forces an enemy attack by setting the enemy timer, suppressing player attack.
    fn force_enemy_attack(
        state: &mut GameState,
        haven: &HavenCombatBonuses,
        achievements: &mut Achievements,
    ) -> Vec<CombatEvent> {
        state.combat_state.player_attack_timer = 0.0;
        state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
        update_combat(state, 0.1, haven, &default_prestige(), achievements)
    }

    /// Forces both player and enemy to attack in the same tick.
    fn force_both_attacks(
        state: &mut GameState,
        haven: &HavenCombatBonuses,
        achievements: &mut Achievements,
    ) -> Vec<CombatEvent> {
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
        update_combat(state, 0.1, haven, &default_prestige(), achievements)
    }

    /// Asserts that at least one event matching the predicate exists.
    fn assert_has_event(events: &[CombatEvent], name: &str, pred: impl Fn(&CombatEvent) -> bool) {
        assert!(events.iter().any(pred), "Expected {name} event");
    }

    /// Asserts that no event matching the predicate exists.
    fn assert_no_event(events: &[CombatEvent], name: &str, pred: impl Fn(&CombatEvent) -> bool) {
        assert!(!events.iter().any(pred), "Unexpected {name} event");
    }

    /// Asserts XP is within the expected combat kill range for the given state.
    fn assert_xp_in_combat_range(state: &GameState, xp: u64, label: &str) {
        let xp_per_tick = crate::core::game_logic::xp_gain_per_tick(
            state.prestige_rank,
            state
                .attributes
                .modifier(crate::character::attributes::AttributeType::Wisdom),
            state
                .attributes
                .modifier(crate::character::attributes::AttributeType::Charisma),
        );
        let min_xp = (xp_per_tick * COMBAT_XP_MIN_TICKS as f64) as u64;
        let max_xp = (xp_per_tick * COMBAT_XP_MAX_TICKS as f64) as u64;
        assert!(
            xp >= min_xp && xp <= max_xp,
            "{label} XP {xp} should be in range [{min_xp}, {max_xp}]",
        );
    }

    #[test]
    fn test_update_combat_no_enemy() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_update_combat_attack_interval() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 5));

        // Not enough time passed
        let events = update_combat(
            &mut state,
            0.5,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );
        assert_eq!(events.len(), 0);

        // Enough time for player attack (1.5s total), but not enemy (needs 2.0s)
        let events = update_combat(
            &mut state,
            1.0,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );
        assert!(!events.is_empty()); // Player attack (enemy not yet at 2.0s)
    }

    #[test]
    fn test_player_died_resets() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Force both attacks (need enemy to attack to kill player)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have player died event
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Player should be at full HP
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );

        // Enemy should be reset
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(enemy.current_hp, enemy.max_hp);
    }

    #[test]
    fn test_regeneration_after_kill() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.player_current_hp = 10;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 1, 5));

        // Force player attack to kill enemy
        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have enemy died event
        let died = events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
        assert!(died);

        // Should be regenerating
        assert!(state.combat_state.is_regenerating);
        assert!(state.combat_state.current_enemy.is_none());

        // Update to complete regen
        update_combat(
            &mut state,
            HP_REGEN_DURATION_SECONDS,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_player_died_in_dungeon() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Put player in a dungeon
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0, 1));
        assert!(state.active_dungeon.is_some());

        // Force both attacks (need enemy to attack to kill player)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have PlayerDiedInDungeon event (not PlayerDied)
        let died_in_dungeon = events
            .iter()
            .any(|e| matches!(e, CombatEvent::PlayerDiedInDungeon));
        let died_normal = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died_in_dungeon);
        assert!(!died_normal);

        // Dungeon should be cleared
        assert!(state.active_dungeon.is_none());

        // Player HP should be reset
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );

        // Enemy should be cleared (not reset like in overworld)
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_weapon_blocked_boss_no_damage() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default(); // No TheStormbreaker achievement

        // Set up Zone 10 boss fight without Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4; // Zone 10 has 4 subzones, this is the zone boss
        state.zone_progression.fighting_boss = true;

        let enemy_hp = 100;
        state.combat_state.current_enemy =
            Some(Enemy::new("Eternal Storm".to_string(), enemy_hp, 10));

        // Force player attack (blocked, but we only check player side)
        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have PlayerAttackBlocked event
        let blocked = events
            .iter()
            .any(|e| matches!(e, CombatEvent::PlayerAttackBlocked { .. }));
        assert!(blocked);

        // Enemy should NOT have taken damage
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(enemy.current_hp, enemy_hp);
    }

    #[test]
    fn test_weapon_blocked_boss_still_attacks_back() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default(); // No TheStormbreaker achievement

        // Set up Zone 10 boss fight without Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4;
        state.zone_progression.fighting_boss = true;

        let player_hp = state.combat_state.player_current_hp;
        state.combat_state.current_enemy = Some(Enemy::new("Eternal Storm".to_string(), 100, 10));

        // Force both attacks (enemy attacks independently now)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have EnemyAttack event
        let enemy_attacked = events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyAttack { .. }));
        assert!(enemy_attacked);

        // Player should have taken damage
        assert!(state.combat_state.player_current_hp < player_hp);
    }

    #[test]
    fn test_death_to_weapon_blocked_boss_resets_encounter() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default(); // No TheStormbreaker achievement

        // Set up Zone 10 boss fight without Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Low HP so player dies
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Eternal Storm".to_string(), 100, 50));

        // Force both attacks (enemy kills player)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have PlayerDied event
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Boss encounter should be reset with retry mechanic
        assert!(!state.zone_progression.fighting_boss);
        assert_eq!(
            state.zone_progression.kills_in_subzone,
            KILLS_FOR_BOSS.saturating_sub(KILLS_FOR_BOSS_RETRY)
        );

        // Enemy should be cleared (not reset)
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_defense_reduces_enemy_damage() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Increase DEX for more defense (defense = DEX modifier)
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 20); // +5 modifier = 5 defense

        let initial_hp = state.combat_state.player_current_hp;
        let enemy_base_damage = 15;
        state.combat_state.current_enemy =
            Some(Enemy::new("Test".to_string(), 100, enemy_base_damage));

        // Force both attacks (need enemy to attack to test defense)
        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Calculate expected damage reduction
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let expected_damage = enemy_base_damage.saturating_sub(derived.defense);
        let actual_damage = initial_hp - state.combat_state.player_current_hp;

        assert_eq!(actual_damage, expected_damage);
    }

    #[test]
    fn test_defense_reduces_damage_to_minimum_floor() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // High DEX for high defense (defense = DEX modifier)
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 30); // +10 modifier = 10 defense

        let initial_hp = state.combat_state.player_current_hp;
        // Enemy damage lower than defense (5 < 10)
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 100, 5));

        // Force both attacks (need enemy to attack to test defense)
        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Player should take minimum 1 damage (min damage floor)
        assert_eq!(state.combat_state.player_current_hp, initial_hp - 1);
    }

    #[test]
    fn test_subzone_boss_defeat() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set up a subzone boss fight (not Zone 10)
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 1;
        state.zone_progression.fighting_boss = true;

        // Weak enemy that will die in one hit
        state.combat_state.current_enemy = Some(Enemy::new("Boss".to_string(), 1, 5));

        // Force player attack to kill boss
        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have SubzoneBossDefeated event
        let boss_defeated = events
            .iter()
            .any(|e| matches!(e, CombatEvent::SubzoneBossDefeated { .. }));
        assert!(boss_defeated);

        // Boss fight should be over
        assert!(!state.zone_progression.fighting_boss);
    }

    #[test]
    fn test_regular_kill_records_progress() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        let initial_kills = state.zone_progression.kills_in_subzone;

        // Weak enemy
        state.combat_state.current_enemy = Some(Enemy::new("Mob".to_string(), 1, 5));

        // Force player attack to kill
        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have EnemyDied event
        let enemy_died = events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
        assert!(enemy_died);

        // Kill should be recorded
        assert!(state.zone_progression.kills_in_subzone > initial_kills);
    }

    #[test]
    fn test_regeneration_skips_combat() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Even with both timers ready, should not attack while regenerating
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        // No combat events during regen
        assert!(events.is_empty());

        // Enemy should not have taken damage
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(enemy.current_hp, 100);
    }

    #[test]
    fn test_gradual_regeneration() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // Partial regen (half duration)
        update_combat(
            &mut state,
            HP_REGEN_DURATION_SECONDS / 2.0,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        // HP should be partially restored (roughly halfway)
        assert!(state.combat_state.player_current_hp > 10);
        assert!(state.combat_state.player_current_hp < 100);
        assert!(state.combat_state.is_regenerating);
    }

    #[test]
    fn test_death_to_any_boss_resets_encounter() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set up a normal boss fight (not weapon-blocked)
        state.zone_progression.current_zone_id = 5;
        state.zone_progression.current_subzone_id = 2;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Low HP so player dies
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Regular Boss".to_string(), 100, 50));

        // Force both attacks (enemy kills player)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Should have PlayerDied event
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Boss encounter should be reset with retry mechanic
        assert!(!state.zone_progression.fighting_boss);
        assert_eq!(
            state.zone_progression.kills_in_subzone,
            KILLS_FOR_BOSS.saturating_sub(KILLS_FOR_BOSS_RETRY)
        );

        // Enemy should be cleared
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_crit_doubles_damage() {
        // Verify that when a crit occurs, damage is exactly 2x base total_damage
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set high DEX for 100% crit chance (need crit_chance_percent >= 100)
        // crit_chance_percent = 5 + DEX_mod; DEX 210 gives mod 100 => 105%
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 210);

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert!(derived.crit_chance_percent >= 100);
        let expected_crit_damage = derived.total_damage() * 2;

        // Give enemy enough HP to survive
        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        // Find the PlayerAttack event
        let attack_event = events
            .iter()
            .find(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
        assert!(attack_event.is_some());

        if let Some(CombatEvent::PlayerAttack { damage, was_crit }) = attack_event {
            assert!(was_crit, "Should always crit with 100%+ crit chance");
            assert_eq!(*damage, expected_crit_damage);
        }
    }

    #[test]
    fn test_zero_crit_chance_never_crits() {
        // With base attributes (DEX 10, mod 0), crit_chance = 5%.
        // Set DEX very low so crit_chance_percent = 0.
        // crit_chance_percent = (5 + dex_mod).max(0); need dex_mod <= -5 => DEX <= 0
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 0);

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert_eq!(derived.crit_chance_percent, 0);

        // Run many attacks to confirm no crits
        let mut crit_count = 0;
        for _ in 0..100 {
            let mut s = GameState::new("Test Hero".to_string(), 0);
            let mut achievements = Achievements::default();
            s.attributes
                .set(crate::character::attributes::AttributeType::Dexterity, 0);
            s.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 100000, 0));
            s.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
            let events = update_combat(
                &mut s,
                0.1,
                &HavenCombatBonuses::default(),
                &default_prestige(),
                &mut achievements,
            );

            for e in &events {
                if let CombatEvent::PlayerAttack { was_crit, .. } = e {
                    if *was_crit {
                        crit_count += 1;
                    }
                }
            }
        }
        assert_eq!(crit_count, 0, "Should never crit with 0% crit chance");
    }

    #[test]
    fn test_player_total_damage_matches_derived_stats() {
        // With no crit (low DEX), verify damage equals derived total_damage
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 0); // 0% crit
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Strength, 20); // +5 mod => phys 15
        state.attributes.set(
            crate::character::attributes::AttributeType::Intelligence,
            16,
        ); // +3 mod => magic 11

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let expected_damage = derived.total_damage(); // 15 + 11 = 26

        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        let attack_event = events
            .iter()
            .find(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
        if let Some(CombatEvent::PlayerAttack { damage, was_crit }) = attack_event {
            assert!(!was_crit);
            assert_eq!(*damage, expected_damage);
        } else {
            panic!("Expected PlayerAttack event");
        }
    }

    #[test]
    fn test_enemy_damage_exactly_reduced_by_defense() {
        // Verify enemy_damage = enemy.damage.saturating_sub(defense) precisely
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 16); // mod +3 => defense 3

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert_eq!(derived.defense, 3);

        let enemy_base_damage = 20;
        state.combat_state.current_enemy =
            Some(Enemy::new("Attacker".to_string(), 10000, enemy_base_damage));
        let initial_hp = state.combat_state.player_current_hp;

        // Force enemy attack to test damage reduction
        force_enemy_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        let hp_lost = initial_hp - state.combat_state.player_current_hp;
        assert_eq!(hp_lost, enemy_base_damage - derived.defense);
    }

    #[test]
    fn test_multi_turn_combat_kills_enemy() {
        // Run combat over multiple turns until the enemy dies
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        // High STR for high damage, low DEX so no crits
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Strength, 30);
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 0);
        state.attributes.set(
            crate::character::attributes::AttributeType::Constitution,
            30,
        );

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        state.combat_state.player_max_hp = derived.max_hp;
        state.combat_state.player_current_hp = derived.max_hp;

        // Enemy with 50 HP and low damage
        state.combat_state.current_enemy = Some(Enemy::new("Weakling".to_string(), 50, 1));

        let mut enemy_died = false;
        let mut total_player_damage = 0u32;
        let mut turns = 0;

        // Simulate up to 20 attack cycles
        for _ in 0..20 {
            let events = force_both_attacks(
                &mut state,
                &HavenCombatBonuses::default(),
                &mut achievements,
            );
            turns += 1;

            for e in &events {
                match e {
                    CombatEvent::PlayerAttack { damage, .. } => {
                        total_player_damage += damage;
                    }
                    CombatEvent::EnemyDied { .. } => {
                        enemy_died = true;
                    }
                    _ => {}
                }
            }

            if enemy_died {
                break;
            }

            // If regenerating, complete regen before next turn
            if state.combat_state.is_regenerating {
                update_combat(
                    &mut state,
                    HP_REGEN_DURATION_SECONDS,
                    &HavenCombatBonuses::default(),
                    &default_prestige(),
                    &mut achievements,
                );
            }
        }

        assert!(enemy_died, "Enemy should have died within 20 turns");
        assert!(
            total_player_damage >= 50,
            "Total damage must be at least enemy HP"
        );
        assert!(turns <= 20, "Should finish within turn limit");
        assert!(state.combat_state.current_enemy.is_none());
        assert!(state.combat_state.is_regenerating);
    }

    #[test]
    fn test_dungeon_elite_has_higher_stats_than_regular() {
        // Use sampling to handle random variance in stat generation
        let zone_id = 5;
        let samples = 50;
        let mut elite_hp_sum = 0u64;
        let mut regular_hp_sum = 0u64;
        let mut elite_dmg_sum = 0u64;
        let mut regular_dmg_sum = 0u64;

        for _ in 0..samples {
            let regular = generate_dungeon_enemy(zone_id);
            let elite = generate_dungeon_elite(zone_id);
            elite_hp_sum += elite.max_hp as u64;
            regular_hp_sum += regular.max_hp as u64;
            elite_dmg_sum += elite.damage as u64;
            regular_dmg_sum += regular.damage as u64;
        }

        assert!(
            elite_hp_sum > regular_hp_sum,
            "Average elite HP should exceed average regular HP"
        );
        assert!(
            elite_dmg_sum > regular_dmg_sum,
            "Average elite damage should exceed average regular damage"
        );
    }

    #[test]
    fn test_dungeon_boss_has_higher_stats_than_elite() {
        // Use sampling to handle random variance in stat generation
        let zone_id = 5;
        let samples = 50;
        let mut boss_hp_sum = 0u64;
        let mut elite_hp_sum = 0u64;
        let mut boss_dmg_sum = 0u64;
        let mut elite_dmg_sum = 0u64;

        for _ in 0..samples {
            let elite = generate_dungeon_elite(zone_id);
            let boss = generate_dungeon_boss(zone_id);
            boss_hp_sum += boss.max_hp as u64;
            elite_hp_sum += elite.max_hp as u64;
            boss_dmg_sum += boss.damage as u64;
            elite_dmg_sum += elite.damage as u64;
        }

        assert!(
            boss_hp_sum > elite_hp_sum,
            "Average boss HP should exceed average elite HP"
        );
        assert!(
            boss_dmg_sum > elite_dmg_sum,
            "Average boss damage should exceed average elite damage"
        );
    }

    #[test]
    fn test_zone_scaling_increases_enemy_stats() {
        use crate::zones::get_all_zones;
        let zones = get_all_zones();

        // Zone 1 HP (static, no sampling needed)
        let zone1 = &zones[0];
        let e1 = generate_zone_enemy(zone1, &zone1.subzones[0]);
        let z1_hp = e1.max_hp;

        // Zone 10 HP
        let zone10 = &zones[9];
        let e10 = generate_zone_enemy(zone10, &zone10.subzones[0]);
        let z10_hp = e10.max_hp;

        assert!(
            z10_hp > z1_hp * 10,
            "Zone 10 enemies should be significantly stronger than zone 1 (z1={}, z10={})",
            z1_hp,
            z10_hp
        );
    }

    #[test]
    fn test_combat_kill_xp_within_expected_range() {
        // combat_kill_xp returns xp_per_tick * random(200..400)
        let xp_per_tick = crate::core::game_logic::xp_gain_per_tick(0, 0, 0);
        let min_expected = xp_per_tick * COMBAT_XP_MIN_TICKS as f64;
        let max_expected = xp_per_tick * COMBAT_XP_MAX_TICKS as f64;

        for _ in 0..100 {
            let xp = crate::core::game_logic::combat_kill_xp(xp_per_tick, 0.0);
            assert!(
                xp >= min_expected as u64 && xp <= max_expected as u64,
                "XP {} should be in range [{}, {}]",
                xp,
                min_expected as u64,
                max_expected as u64
            );
        }
    }

    #[test]
    fn test_xp_gained_on_enemy_death() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let initial_xp = state.character_xp;

        // Weak enemy that dies in one hit
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 1, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        let xp_event = events.iter().find_map(|e| match e {
            CombatEvent::EnemyDied { xp_gained } => Some(*xp_gained),
            _ => None,
        });

        assert!(xp_event.is_some(), "Should emit EnemyDied with XP");
        let xp_gained = xp_event.unwrap();
        assert!(xp_gained > 0, "XP gained should be positive");

        // XP should be in the combat kill range
        let xp_per_tick = crate::core::game_logic::xp_gain_per_tick(0, 0, 0);
        let min_xp = (xp_per_tick * COMBAT_XP_MIN_TICKS as f64) as u64;
        let max_xp = (xp_per_tick * COMBAT_XP_MAX_TICKS as f64) as u64;
        assert!(
            xp_gained >= min_xp && xp_gained <= max_xp,
            "XP {} not in expected range [{}, {}]",
            xp_gained,
            min_xp,
            max_xp
        );

        // Note: XP is not applied to state in combat_logic, it's just reported
        assert_eq!(
            state.character_xp, initial_xp,
            "combat_logic should not apply XP directly"
        );
    }

    #[test]
    fn test_combat_log_add_entry() {
        let mut combat = CombatState::new(100);
        assert!(combat.combat_log.is_empty());

        combat.add_log_entry("Hit for 10".to_string(), false, true);
        assert_eq!(combat.combat_log.len(), 1);
        assert_eq!(combat.combat_log[0].message, "Hit for 10");
        assert!(!combat.combat_log[0].is_crit);
        assert!(combat.combat_log[0].is_player_action);

        combat.add_log_entry("Critical hit for 20".to_string(), true, true);
        assert_eq!(combat.combat_log.len(), 2);
        assert!(combat.combat_log[1].is_crit);
    }

    #[test]
    fn test_combat_log_caps_at_10_entries() {
        let mut combat = CombatState::new(100);

        for i in 0..15 {
            combat.add_log_entry(format!("Entry {}", i), false, true);
        }

        assert_eq!(combat.combat_log.len(), 10);
        // First entry should be "Entry 5" (entries 0-4 were evicted)
        assert_eq!(combat.combat_log[0].message, "Entry 5");
        assert_eq!(combat.combat_log[9].message, "Entry 14");
    }

    #[test]
    fn test_enemy_min_damage_with_high_defense() {
        // When defense >= enemy damage, player takes minimum 1 damage (min floor)
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 40); // mod 15 => defense 15

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert!(derived.defense >= 15);

        let initial_hp = state.combat_state.player_current_hp;
        // Enemy with damage less than defense
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 10000, 5));
        // Force both attacks so enemy actually attacks
        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Player should take minimum 1 damage (min damage floor)
        assert_eq!(state.combat_state.player_current_hp, initial_hp - 1);
    }

    #[test]
    fn test_death_to_regular_enemy_resets_enemy_hp() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.zone_progression.fighting_boss = false;
        state.combat_state.player_current_hp = 1;

        let enemy_max_hp = 100;
        let mut enemy = Enemy::new("Regular".to_string(), enemy_max_hp, 50);
        enemy.take_damage(30); // Reduce to 70 HP
        state.combat_state.current_enemy = Some(enemy);

        // Force both attacks (enemy kills player)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Regular enemy should have HP reset (not removed)
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(enemy.current_hp, enemy.max_hp);
    }

    #[test]
    fn test_prestige_rank_preserved_on_death() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set a prestige rank (3 = Gold)
        state.prestige_rank = 3;
        let original_rank = state.prestige_rank;

        // Set up boss fight
        state.zone_progression.fighting_boss = true;

        // Low HP so player dies
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Boss".to_string(), 100, 50));

        // Force both attacks (enemy kills player)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Verify player died
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Prestige rank should NOT be changed
        assert_eq!(state.prestige_rank, original_rank);
    }

    #[test]
    fn test_crit_multiplier_from_equipment() {
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set 100% crit chance
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 210);

        // Add weapon with +100% crit multiplier (2.0 -> 3.0x)
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            ilvl: 10,
            base_name: "Sword".to_string(),
            display_name: "Sword".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::CritMultiplier,
                value: 100.0,
            }],
        };
        state.equipment.set(EquipmentSlot::Weapon, Some(weapon));

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let base_damage = derived.total_damage();
        let expected_crit_damage = (base_damage as f64 * 3.0) as u32; // 3x with +100%

        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
        let mut achievements = Achievements::default();
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        let attack = events
            .iter()
            .find_map(|e| match e {
                CombatEvent::PlayerAttack { damage, was_crit } => Some((*damage, *was_crit)),
                _ => None,
            })
            .expect("Should have attack event");

        assert!(attack.1, "Should always crit with 100%+ crit chance");
        assert_eq!(attack.0, expected_crit_damage);
    }

    #[test]
    fn test_attack_speed_reduces_interval() {
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Add gloves with +50% attack speed
        let gloves = Item {
            slot: EquipmentSlot::Gloves,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Gloves".to_string(),
            display_name: "Gloves".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::AttackSpeed,
                value: 50.0,
            }],
        };
        state.equipment.set(EquipmentSlot::Gloves, Some(gloves));

        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));

        // With 50% attack speed, effective interval is 1.5 / 1.5 = 1.0 seconds
        // So attack should trigger at 1.0 seconds instead of 1.5
        state.combat_state.player_attack_timer = 1.0;
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        let attacked = events
            .iter()
            .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
        assert!(attacked, "Should attack with reduced interval");
    }

    #[test]
    fn test_attack_speed_normal_interval_without_affix() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));

        // Without attack speed bonus, 1.0 seconds is not enough (need 1.5)
        state.combat_state.player_attack_timer = 1.0;
        let events = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        let attacked = events
            .iter()
            .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
        assert!(!attacked, "Should NOT attack before full interval");
    }

    #[test]
    fn test_hp_regen_speed_with_affix() {
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Add armor with +100% HP regen (2x speed = half duration)
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Armor".to_string(),
            display_name: "Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::HPRegen,
                value: 100.0,
            }],
        };
        state.equipment.set(EquipmentSlot::Armor, Some(armor));

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // With +100% regen (2x multiplier), duration is 2.5 / 2 = 1.25 seconds
        // After 1.25 seconds, should be fully healed
        update_combat(
            &mut state,
            1.25,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        assert_eq!(state.combat_state.player_current_hp, 100);
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_hp_regen_normal_duration_without_affix() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // Without regen bonus, 1.25 seconds is not enough (need 2.5)
        update_combat(
            &mut state,
            1.25,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        // Should still be regenerating, not fully healed
        assert!(state.combat_state.is_regenerating);
        assert!(state.combat_state.player_current_hp < 100);
    }

    #[test]
    fn test_haven_hp_regen_bonus() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // With +100% Haven regen bonus (2x multiplier), duration is 2.5 / 2 = 1.25 seconds
        // After 1.25 seconds, should be fully healed
        let haven = HavenCombatBonuses {
            hp_regen_percent: 100.0,
            ..Default::default()
        };
        update_combat(
            &mut state,
            1.25,
            &haven,
            &default_prestige(),
            &mut achievements,
        );

        assert_eq!(state.combat_state.player_current_hp, 100);
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_haven_hp_regen_stacks_with_equipment() {
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Add armor with +100% HP regen (2x speed)
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Armor".to_string(),
            display_name: "Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::HPRegen,
                value: 100.0,
            }],
        };
        state.equipment.set(EquipmentSlot::Armor, Some(armor));

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // Equipment: 2x multiplier, Haven +50%: 1.5x multiplier
        // Combined: 2.0 * 1.5 = 3x multiplier
        // Duration: 2.5 / 3 = 0.833 seconds
        let haven = HavenCombatBonuses {
            hp_regen_percent: 50.0,
            ..Default::default()
        };
        let mut achievements = Achievements::default();
        update_combat(
            &mut state,
            0.84,
            &haven,
            &default_prestige(),
            &mut achievements,
        );

        assert_eq!(state.combat_state.player_current_hp, 100);
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_damage_reflection_hurts_attacker() {
        use crate::character::attributes::AttributeType;
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set DEX to 0 to eliminate crit chance (base 5% + dex_mod, with DEX=0 gives 0%)
        state.attributes.set(AttributeType::Dexterity, 0);

        // Add armor with 50% damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Armor".to_string(),
            display_name: "Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 50.0,
            }],
        };
        state.equipment.set(EquipmentSlot::Armor, Some(armor));

        let enemy_damage = 20;
        let enemy_max_hp = 100;
        state.combat_state.current_enemy = Some(Enemy::new(
            "Attacker".to_string(),
            enemy_max_hp,
            enemy_damage,
        ));
        // Force both attacks (enemy attacks -> reflection triggers)
        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Enemy should have taken reflected damage: 20 * 50% = 10
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let player_damage = derived.total_damage();
        let reflected_damage = (enemy_damage as f64 * 0.5) as u32; // 50% reflection

        // Enemy HP = max - player_attack - reflected
        let expected_hp = enemy_max_hp - player_damage - reflected_damage;
        assert_eq!(
            enemy.current_hp, expected_hp,
            "Enemy should take player damage ({}) plus reflection ({})",
            player_damage, reflected_damage
        );
    }

    #[test]
    fn test_damage_reflection_can_kill_enemy() {
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);

        // High damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Legendary,
            ilvl: 10,
            base_name: "Armor".to_string(),
            display_name: "Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 1000.0, // 1000% reflection
            }],
        };
        state.equipment.set(EquipmentSlot::Armor, Some(armor));

        // Weak enemy with high damage (will kill itself via reflection)
        state.combat_state.current_enemy = Some(Enemy::new("Suicidal".to_string(), 5, 100));
        state.combat_state.player_current_hp = 1000; // Survive the hit
        state.combat_state.player_max_hp = 1000;

        let mut achievements = Achievements::default();
        // Force both attacks - player kills enemy outright (5 HP < player damage)
        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Enemy should have died from combined player attack + reflection
        let enemy_died = events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
        assert!(enemy_died, "Enemy should die from reflection damage");
    }

    #[test]
    fn test_damage_reflection_zero_when_no_damage_taken() {
        use crate::items::types::{
            Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
        };

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set DEX to 0 to guarantee 0% crit chance
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 0);

        // Add damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Armor".to_string(),
            display_name: "Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 100.0,
            }],
        };
        state.equipment.set(EquipmentSlot::Armor, Some(armor));

        let enemy_max_hp = 1000;
        // Enemy deals 0 base damage, but min floor means 1 damage dealt
        state.combat_state.current_enemy =
            Some(Enemy::new("Pacifist".to_string(), enemy_max_hp, 0));

        let initial_player_hp = state.combat_state.player_current_hp;
        // Force both attacks (enemy attacks with min floor 1 damage, reflection triggers)
        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Player took 1 damage (min floor)
        assert_eq!(state.combat_state.player_current_hp, initial_player_hp - 1);

        // Enemy takes player attack damage + 1 reflected (100% of 1 damage)
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let reflected = 1u32; // 100% of 1 min-floor damage
        let expected_hp = enemy_max_hp - derived.total_damage() - reflected;
        assert_eq!(enemy.current_hp, expected_hp);
    }

    // =========================================================================
    // Haven Combat Bonus Tests
    // =========================================================================

    #[test]
    fn test_haven_damage_bonus() {
        use crate::character::attributes::AttributeType;

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        // Set DEX to 0 to eliminate crits
        state.attributes.set(AttributeType::Dexterity, 0);

        // Create an enemy with lots of HP
        state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

        // First, attack with no Haven bonus
        let events_no_bonus = update_combat(
            &mut state,
            0.1,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );
        let damage_no_bonus = events_no_bonus
            .iter()
            .find_map(|e| {
                if let CombatEvent::PlayerAttack { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Reset enemy and attack with +50% damage bonus
        state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

        let haven = HavenCombatBonuses {
            damage_percent: 50.0,
            ..Default::default()
        };
        let events_with_bonus = update_combat(
            &mut state,
            0.1,
            &haven,
            &default_prestige(),
            &mut achievements,
        );
        let damage_with_bonus = events_with_bonus
            .iter()
            .find_map(|e| {
                if let CombatEvent::PlayerAttack { damage, .. } = e {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Damage should be 50% higher
        let expected = (damage_no_bonus as f64 * 1.5) as u32;
        assert_eq!(
            damage_with_bonus, expected,
            "Haven +50% damage should increase {} to {}",
            damage_no_bonus, expected
        );
    }

    #[test]
    fn test_haven_crit_chance_bonus() {
        use crate::character::attributes::AttributeType;

        // Run many trials to verify crit rate increase
        let mut crits_no_bonus = 0;
        let mut crits_with_bonus = 0;
        let trials = 10000;

        for _ in 0..trials {
            let mut state = GameState::new("Test Hero".to_string(), 0);
            let mut achievements = Achievements::default();
            state.attributes.set(AttributeType::Dexterity, 0); // Base 0% crit
            state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
            state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

            let events = update_combat(
                &mut state,
                0.1,
                &HavenCombatBonuses::default(),
                &default_prestige(),
                &mut achievements,
            );
            if events
                .iter()
                .any(|e| matches!(e, CombatEvent::PlayerAttack { was_crit: true, .. }))
            {
                crits_no_bonus += 1;
            }
        }

        for _ in 0..trials {
            let mut state = GameState::new("Test Hero".to_string(), 0);
            let mut achievements = Achievements::default();
            state.attributes.set(AttributeType::Dexterity, 0);
            state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
            state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

            let haven = HavenCombatBonuses {
                crit_chance_percent: 20.0, // +20% crit
                ..Default::default()
            };
            let events = update_combat(
                &mut state,
                0.1,
                &haven,
                &default_prestige(),
                &mut achievements,
            );
            if events
                .iter()
                .any(|e| matches!(e, CombatEvent::PlayerAttack { was_crit: true, .. }))
            {
                crits_with_bonus += 1;
            }
        }

        // With +20% crit bonus, should see roughly 20% crits
        // Allow wide tolerance for randomness
        assert!(
            crits_with_bonus > crits_no_bonus + 500,
            "Haven +20% crit should significantly increase crit rate: no_bonus={}, with_bonus={}",
            crits_no_bonus,
            crits_with_bonus
        );
    }

    #[test]
    fn test_haven_double_strike() {
        use crate::character::attributes::AttributeType;

        // Run many trials to verify double strike rate
        let mut double_strikes = 0;
        let trials = 10000;

        for _ in 0..trials {
            let mut state = GameState::new("Test Hero".to_string(), 0);
            let mut achievements = Achievements::default();
            state.attributes.set(AttributeType::Dexterity, 0);
            state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
            state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

            let haven = HavenCombatBonuses {
                double_strike_chance: 35.0, // +35% double strike (T3 War Room)
                ..Default::default()
            };
            let events = update_combat(
                &mut state,
                0.1,
                &haven,
                &default_prestige(),
                &mut achievements,
            );

            // Count PlayerAttack events (should be 2 if double strike procs)
            let attack_count = events
                .iter()
                .filter(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
                .count();
            if attack_count == 2 {
                double_strikes += 1;
            }
        }

        // With 35% double strike chance, expect ~3500 double strikes in 10000 trials
        // Allow 10% tolerance
        assert!(
            (3000..=4000).contains(&double_strikes),
            "Expected ~3500 double strikes (35%), got {}",
            double_strikes
        );
    }

    #[test]
    fn test_haven_regen_delay_reduction() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // With -50% regen delay, base duration is 2.5 * 0.5 = 1.25 seconds
        let haven = HavenCombatBonuses {
            hp_regen_delay_reduction: 50.0,
            ..Default::default()
        };
        update_combat(
            &mut state,
            1.25,
            &haven,
            &default_prestige(),
            &mut achievements,
        );

        assert_eq!(state.combat_state.player_current_hp, 100);
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_haven_combined_combat_bonuses() {
        use crate::character::attributes::AttributeType;

        // Test multiple bonuses together
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.attributes.set(AttributeType::Dexterity, 0);
        state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
        state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

        let haven = HavenCombatBonuses {
            damage_percent: 25.0,
            crit_chance_percent: 10.0,
            double_strike_chance: 10.0,
            hp_regen_percent: 50.0,
            hp_regen_delay_reduction: 30.0,
            xp_gain_percent: 20.0,
        };

        let events = update_combat(
            &mut state,
            0.1,
            &haven,
            &default_prestige(),
            &mut achievements,
        );

        // Should have at least one attack
        assert!(
            events
                .iter()
                .any(|e| matches!(e, CombatEvent::PlayerAttack { .. })),
            "Should have at least one attack event"
        );
    }

    #[test]
    fn test_dungeon_combat_kills_do_not_affect_zone_progression() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Goblin".to_string(), 1, 1));
        state.combat_state.player_current_hp = 1000;
        let initial_kills = state.zone_progression.kills_in_subzone;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        assert_eq!(
            state.zone_progression.kills_in_subzone, initial_kills,
            "Dungeon kills should not increment zone kill counter"
        );
        assert!(
            !state.zone_progression.fighting_boss,
            "Dungeon kills should not trigger zone boss"
        );
    }

    // =========================================================================
    // Dungeon Combat Event Emission Tests
    // =========================================================================

    /// Helper: creates a GameState with a minimal deterministic dungeon
    /// where the player is in a room of the specified type.
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
    fn test_dungeon_combat_room_kill_emits_enemy_died() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Goblin".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        assert_no_event(&events, "SubzoneBossDefeated", |e| {
            matches!(e, CombatEvent::SubzoneBossDefeated { .. })
        });
        assert_no_event(&events, "EliteDefeated", |e| {
            matches!(e, CombatEvent::EliteDefeated { .. })
        });
        assert_no_event(&events, "BossDefeated", |e| {
            matches!(e, CombatEvent::BossDefeated { .. })
        });
    }

    #[test]
    fn test_dungeon_elite_room_kill_emits_elite_defeated() {
        let mut state = setup_dungeon_with_room_type(RoomType::Elite);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Elite Guard".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "EliteDefeated", |e| {
            matches!(e, CombatEvent::EliteDefeated { .. })
        });
        assert_no_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        assert_no_event(&events, "SubzoneBossDefeated", |e| {
            matches!(e, CombatEvent::SubzoneBossDefeated { .. })
        });
        assert_no_event(&events, "BossDefeated", |e| {
            matches!(e, CombatEvent::BossDefeated { .. })
        });
    }

    #[test]
    fn test_dungeon_boss_room_kill_emits_boss_defeated() {
        let mut state = setup_dungeon_with_room_type(RoomType::Boss);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Dungeon Boss".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "BossDefeated", |e| {
            matches!(e, CombatEvent::BossDefeated { .. })
        });
        assert_no_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        assert_no_event(&events, "SubzoneBossDefeated", |e| {
            matches!(e, CombatEvent::SubzoneBossDefeated { .. })
        });
        assert_no_event(&events, "EliteDefeated", |e| {
            matches!(e, CombatEvent::EliteDefeated { .. })
        });
    }

    #[test]
    fn test_dungeon_combat_room_kill_xp_in_valid_range() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Goblin".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        let xp = events
            .iter()
            .find_map(|e| match e {
                CombatEvent::EnemyDied { xp_gained } => Some(*xp_gained),
                _ => None,
            })
            .expect("Should have EnemyDied event with XP");
        assert!(xp > 0, "XP gained should be positive");
        assert_xp_in_combat_range(&state, xp, "Dungeon combat");
    }

    #[test]
    fn test_dungeon_elite_room_kill_xp_in_valid_range() {
        let mut state = setup_dungeon_with_room_type(RoomType::Elite);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Elite Guard".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        let xp = events
            .iter()
            .find_map(|e| match e {
                CombatEvent::EliteDefeated { xp_gained } => Some(*xp_gained),
                _ => None,
            })
            .expect("Should have EliteDefeated event with XP");
        assert!(xp > 0, "Elite XP gained should be positive");
        assert_xp_in_combat_range(&state, xp, "Dungeon elite");
    }

    #[test]
    fn test_dungeon_boss_room_kill_xp_in_valid_range() {
        let mut state = setup_dungeon_with_room_type(RoomType::Boss);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Dungeon Boss".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        let xp = events
            .iter()
            .find_map(|e| match e {
                CombatEvent::BossDefeated { xp_gained } => Some(*xp_gained),
                _ => None,
            })
            .expect("Should have BossDefeated event with XP");
        assert!(xp > 0, "Boss XP gained should be positive");
        assert_xp_in_combat_range(&state, xp, "Dungeon boss");
    }

    #[test]
    fn test_player_died_in_dungeon_emits_correct_event_and_exits() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        let mut achievements = Achievements::default();
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Deadly Mob".to_string(), 100, 50));

        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "PlayerDiedInDungeon", |e| {
            matches!(e, CombatEvent::PlayerDiedInDungeon)
        });
        assert_no_event(&events, "PlayerDied", |e| {
            matches!(e, CombatEvent::PlayerDied)
        });
        assert!(state.active_dungeon.is_none());
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_dungeon_elite_kill_does_not_affect_zone_progression() {
        let mut state = setup_dungeon_with_room_type(RoomType::Elite);
        let mut achievements = Achievements::default();
        let initial_kills = state.zone_progression.kills_in_subzone;
        state.combat_state.current_enemy = Some(Enemy::new("Elite Guard".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_eq!(state.zone_progression.kills_in_subzone, initial_kills);
        assert!(!state.zone_progression.fighting_boss);
    }

    #[test]
    fn test_dungeon_boss_kill_does_not_affect_zone_progression() {
        let mut state = setup_dungeon_with_room_type(RoomType::Boss);
        let mut achievements = Achievements::default();
        let initial_kills = state.zone_progression.kills_in_subzone;
        state.combat_state.current_enemy = Some(Enemy::new("Dungeon Boss".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_eq!(state.zone_progression.kills_in_subzone, initial_kills);
        assert!(!state.zone_progression.fighting_boss);
    }

    #[test]
    fn test_dungeon_kill_with_overworld_boss_flag_still_emits_dungeon_events() {
        // Edge case: if zone_progression.fighting_boss is true but player is
        // in a dungeon, dungeon event logic should take priority.
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        let mut achievements = Achievements::default();
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;
        state.combat_state.current_enemy = Some(Enemy::new("Goblin".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        assert_no_event(&events, "SubzoneBossDefeated", |e| {
            matches!(e, CombatEvent::SubzoneBossDefeated { .. })
        });
        assert!(state.zone_progression.fighting_boss);
        assert_eq!(state.zone_progression.kills_in_subzone, 10);
    }

    #[test]
    fn test_dungeon_death_preserves_prestige_rank() {
        let mut state = setup_dungeon_with_room_type(RoomType::Combat);
        let mut achievements = Achievements::default();
        state.prestige_rank = 7;
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Deadly Mob".to_string(), 100, 50));

        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_eq!(state.prestige_rank, 7);
    }

    #[test]
    fn test_dungeon_entrance_room_kill_emits_enemy_died() {
        // Entrance rooms normally have no combat, but if an enemy is somehow
        // present, the fallback path should emit EnemyDied (not a special event)
        let mut state = setup_dungeon_with_room_type(RoomType::Entrance);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Straggler".to_string(), 1, 0));
        state.combat_state.player_current_hp = 1000;
        state.combat_state.player_max_hp = 1000;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        assert_no_event(&events, "EliteDefeated", |e| {
            matches!(e, CombatEvent::EliteDefeated { .. })
        });
        assert_no_event(&events, "BossDefeated", |e| {
            matches!(e, CombatEvent::BossDefeated { .. })
        });
        assert_no_event(&events, "SubzoneBossDefeated", |e| {
            matches!(e, CombatEvent::SubzoneBossDefeated { .. })
        });
    }

    // =========================================================================
    // Decoupled Attack Timer Tests
    // =========================================================================

    #[test]
    fn test_enemy_attacks_independently_of_player() {
        // Only enemy timer fires; player timer stays below threshold.
        // Should see EnemyAttack but NOT PlayerAttack.
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Mob".to_string(), 10000, 10));
        let initial_hp = state.combat_state.player_current_hp;

        let events = force_enemy_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "EnemyAttack", |e| {
            matches!(e, CombatEvent::EnemyAttack { .. })
        });
        assert_no_event(&events, "PlayerAttack", |e| {
            matches!(e, CombatEvent::PlayerAttack { .. })
        });
        // Player should have taken damage
        assert!(state.combat_state.player_current_hp < initial_hp);
    }

    #[test]
    fn test_player_attacks_independently_of_enemy() {
        // Only player timer fires; enemy timer stays below threshold.
        // Should see PlayerAttack but NOT EnemyAttack.
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Mob".to_string(), 10000, 10));
        let initial_hp = state.combat_state.player_current_hp;

        let events = force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "PlayerAttack", |e| {
            matches!(e, CombatEvent::PlayerAttack { .. })
        });
        assert_no_event(&events, "EnemyAttack", |e| {
            matches!(e, CombatEvent::EnemyAttack { .. })
        });
        // Player should NOT have taken damage (enemy didn't attack)
        assert_eq!(state.combat_state.player_current_hp, initial_hp);
    }

    #[test]
    fn test_both_timers_fire_player_goes_first() {
        // Both timers fire on the same tick. Player's attack kills the enemy.
        // Enemy should NOT get to attack (player advantage).
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        // Weak enemy that dies in one hit
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 1, 50));
        let initial_hp = state.combat_state.player_current_hp;

        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        // Player should have attacked
        assert_has_event(&events, "PlayerAttack", |e| {
            matches!(e, CombatEvent::PlayerAttack { .. })
        });
        // Enemy died
        assert_has_event(&events, "EnemyDied", |e| {
            matches!(e, CombatEvent::EnemyDied { .. })
        });
        // Enemy should NOT have attacked (died before getting a turn)
        assert_no_event(&events, "EnemyAttack", |e| {
            matches!(e, CombatEvent::EnemyAttack { .. })
        });
        // Player HP should be unchanged
        assert_eq!(state.combat_state.player_current_hp, initial_hp);
    }

    #[test]
    fn test_both_timers_fire_enemy_survives_attacks_back() {
        // Both timers fire on the same tick. Enemy survives player's attack.
        // Should see PlayerAttack THEN EnemyAttack, in that order.
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        // Tough enemy that survives
        state.combat_state.current_enemy = Some(Enemy::new("Tough".to_string(), 10000, 10));
        let initial_hp = state.combat_state.player_current_hp;

        let events = force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_has_event(&events, "PlayerAttack", |e| {
            matches!(e, CombatEvent::PlayerAttack { .. })
        });
        assert_has_event(&events, "EnemyAttack", |e| {
            matches!(e, CombatEvent::EnemyAttack { .. })
        });

        // Verify ordering: PlayerAttack appears before EnemyAttack
        let player_idx = events
            .iter()
            .position(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
            .unwrap();
        let enemy_idx = events
            .iter()
            .position(|e| matches!(e, CombatEvent::EnemyAttack { .. }))
            .unwrap();
        assert!(
            player_idx < enemy_idx,
            "PlayerAttack (idx {}) should come before EnemyAttack (idx {})",
            player_idx,
            enemy_idx
        );

        // Player should have taken damage from enemy attack
        assert!(state.combat_state.player_current_hp < initial_hp);
    }

    #[test]
    fn test_enemy_attack_interval_boss_faster() {
        // Subzone boss should use ENEMY_BOSS_ATTACK_INTERVAL_SECONDS (1.8s)
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 1; // Not the last subzone
        state.zone_progression.fighting_boss = true;

        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_BOSS_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Subzone boss interval should be {}, got {}",
            ENEMY_BOSS_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }

    #[test]
    fn test_enemy_attack_interval_zone_boss() {
        // Zone boss (last subzone of a zone) uses ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS (1.5s)
        let mut state = GameState::new("Test Hero".to_string(), 0);
        // Zone 1 has 3 subzones, so subzone_id 3 is the zone boss
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 3;
        state.zone_progression.fighting_boss = true;

        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Zone boss interval should be {}, got {}",
            ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }

    #[test]
    fn test_enemy_attack_interval_dungeon_elite() {
        // Dungeon elite uses ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS (1.6s)
        let state = setup_dungeon_with_room_type(RoomType::Elite);

        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Dungeon elite interval should be {}, got {}",
            ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }

    #[test]
    fn test_enemy_attack_interval_dungeon_boss() {
        // Dungeon boss uses ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS (1.4s)
        let state = setup_dungeon_with_room_type(RoomType::Boss);

        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Dungeon boss interval should be {}, got {}",
            ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }

    #[test]
    fn test_enemy_attack_interval_normal_mob() {
        // Normal mob uses ENEMY_ATTACK_INTERVAL_SECONDS (2.0s)
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.fighting_boss = false;

        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Normal mob interval should be {}, got {}",
            ENEMY_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }

    #[test]
    fn test_enemy_timer_resets_on_new_enemy_spawn() {
        // After killing an enemy, entering regen, and spawning a new enemy,
        // enemy_attack_timer should be 0.0
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 1, 0));

        // Kill the enemy
        force_player_attack(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );
        assert!(state.combat_state.is_regenerating);
        assert_eq!(state.combat_state.enemy_attack_timer, 0.0);

        // Complete regen
        update_combat(
            &mut state,
            HP_REGEN_DURATION_SECONDS,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );
        assert!(!state.combat_state.is_regenerating);

        // Simulate spawning a new enemy (as game_logic does)
        state.combat_state.current_enemy = Some(Enemy::new("New Mob".to_string(), 100, 5));
        state.combat_state.player_attack_timer = 0.0;
        state.combat_state.enemy_attack_timer = 0.0;

        assert_eq!(state.combat_state.player_attack_timer, 0.0);
        assert_eq!(state.combat_state.enemy_attack_timer, 0.0);
    }

    #[test]
    fn test_both_timers_reset_on_player_death() {
        // After death, both timers should be 0.0
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Killer".to_string(), 10000, 50));

        force_both_attacks(
            &mut state,
            &HavenCombatBonuses::default(),
            &mut achievements,
        );

        assert_eq!(
            state.combat_state.player_attack_timer, 0.0,
            "Player timer should reset to 0.0 after death"
        );
        assert_eq!(
            state.combat_state.enemy_attack_timer, 0.0,
            "Enemy timer should reset to 0.0 after death"
        );
    }

    #[test]
    fn test_regen_blocks_both_timers() {
        // During regen, neither timer should advance
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;
        state.combat_state.current_enemy = Some(Enemy::new("Mob".to_string(), 100, 10));

        // Set both timers to known values
        state.combat_state.player_attack_timer = 0.5;
        state.combat_state.enemy_attack_timer = 0.3;

        // Tick during regen
        update_combat(
            &mut state,
            0.5,
            &HavenCombatBonuses::default(),
            &default_prestige(),
            &mut achievements,
        );

        // Timers should NOT have advanced (regen blocks combat)
        assert!(
            (state.combat_state.player_attack_timer - 0.5).abs() < f64::EPSILON,
            "Player timer should not advance during regen, got {}",
            state.combat_state.player_attack_timer
        );
        assert!(
            (state.combat_state.enemy_attack_timer - 0.3).abs() < f64::EPSILON,
            "Enemy timer should not advance during regen, got {}",
            state.combat_state.enemy_attack_timer
        );
    }

    #[test]
    fn test_old_save_migration_attack_timer() {
        // JSON with old "attack_timer" key should load into player_attack_timer
        // and enemy_attack_timer should default to 0.0
        let json = serde_json::json!({
            "current_enemy": null,
            "player_current_hp": 50,
            "player_max_hp": 50,
            "attack_timer": 1.2,
            "regen_timer": 0.0,
            "is_regenerating": false
        });

        let loaded: CombatState = serde_json::from_value(json).unwrap();
        assert!(
            (loaded.player_attack_timer - 1.2).abs() < f64::EPSILON,
            "Old attack_timer should map to player_attack_timer, got {}",
            loaded.player_attack_timer
        );
        assert!(
            (loaded.enemy_attack_timer - 0.0).abs() < f64::EPSILON,
            "Missing enemy_attack_timer should default to 0.0, got {}",
            loaded.enemy_attack_timer
        );
    }
}
