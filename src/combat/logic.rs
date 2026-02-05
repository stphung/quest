use crate::character::derived_stats::DerivedStats;
use crate::core::constants::*;
use crate::core::game_state::GameState;
use crate::dungeon::types::RoomType;
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

#[allow(dead_code)]
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
    None,
}

/// Updates combat state, returns events that occurred
/// `haven` contains all Haven bonuses that affect combat
pub fn update_combat(
    state: &mut GameState,
    delta_time: f64,
    haven: &HavenCombatBonuses,
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
        let base_regen_duration = HP_REGEN_DURATION_SECONDS * (1.0 - haven.hp_regen_delay_reduction / 100.0);
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

    // Update attack timer
    state.combat_state.attack_timer += delta_time;

    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);

    // Attack speed multiplier: higher = faster attacks
    let effective_attack_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;

    if state.combat_state.attack_timer >= effective_attack_interval {
        state.combat_state.attack_timer = 0.0;

        // Check if boss requires a weapon we don't have
        if let Some(weapon_name) = state.zone_progression.boss_weapon_blocked() {
            // Attack is blocked - no damage dealt
            events.push(CombatEvent::PlayerAttackBlocked {
                weapon_needed: weapon_name.to_string(),
            });

            // Enemy still attacks back (see below)
        } else {
            // Player attacks normally
            // Apply Armory bonus: +% damage
            let base_damage = derived.total_damage();
            let mut damage = (base_damage as f64 * (1.0 + haven.damage_percent / 100.0)) as u32;
            let mut was_crit = false;

            // Roll for crit (apply Watchtower bonus: +% crit chance)
            let total_crit_chance = derived.crit_chance_percent + haven.crit_chance_percent as u32;
            let crit_roll = rand::thread_rng().gen_range(0..100);
            if crit_roll < total_crit_chance {
                damage = (damage as f64 * derived.crit_multiplier) as u32;
                was_crit = true;
            }

            // Roll for double strike (War Room bonus)
            let double_strike_roll = rand::thread_rng().gen::<f64>() * 100.0;
            let num_strikes = if double_strike_roll < haven.double_strike_chance { 2 } else { 1 };

            if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                // Apply damage (potentially multiple times with double strike)
                for strike in 0..num_strikes {
                    if !enemy.is_alive() {
                        break; // Enemy already dead
                    }
                    enemy.take_damage(damage);
                    // Only first strike uses original crit flag, subsequent strikes are bonus hits
                    let strike_crit = if strike == 0 { was_crit } else { false };
                    events.push(CombatEvent::PlayerAttack { damage, was_crit: strike_crit });
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

                    match dungeon_room_type {
                        Some(RoomType::Elite) => {
                            events.push(CombatEvent::EliteDefeated { xp_gained });
                        }
                        Some(RoomType::Boss) => {
                            events.push(CombatEvent::BossDefeated { xp_gained });
                        }
                        _ => {
                            // Check if this was a subzone boss (overworld)
                            if state.zone_progression.fighting_boss {
                                let result =
                                    state.zone_progression.on_boss_defeated(state.prestige_rank);
                                events.push(CombatEvent::SubzoneBossDefeated { xp_gained, result });
                            } else {
                                // Record the kill for boss spawn tracking (boss flag set if threshold reached)
                                state.zone_progression.record_kill();
                                events.push(CombatEvent::EnemyDied { xp_gained });
                            }
                        }
                    }

                    // Remove enemy and start regeneration
                    state.combat_state.current_enemy = None;
                    state.combat_state.is_regenerating = true;
                    state.combat_state.regen_timer = 0.0;

                    return events;
                }
            }
        }

        // Enemy attacks back
        if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
            let enemy_damage = enemy.damage.saturating_sub(derived.defense);
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

                // Reset enemy HP if we're not in dungeon (normal combat continues)
                if !in_dungeon {
                    // Check if we died to a boss
                    if state.zone_progression.fighting_boss {
                        // Reset boss encounter - go back to fighting regular enemies
                        state.zone_progression.fighting_boss = false;
                        state.zone_progression.kills_in_subzone = 0;
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
        generate_boss_enemy, generate_elite_enemy, generate_enemy, generate_zone_enemy,
        CombatState, Enemy,
    };
    use super::*;

    #[test]
    fn test_update_combat_no_enemy() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_update_combat_attack_interval() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 5));

        // Not enough time passed
        let events = update_combat(&mut state, 0.5, &HavenCombatBonuses::default());
        assert_eq!(events.len(), 0);

        // Enough time for attack
        let events = update_combat(&mut state, 1.0, &HavenCombatBonuses::default());
        assert!(events.len() >= 2); // Player attack + enemy attack
    }

    #[test]
    fn test_player_died_resets() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
        state.combat_state.player_current_hp = 10;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 1, 5));

        // Force attack to kill enemy
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Should have enemy died event
        let died = events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
        assert!(died);

        // Should be regenerating
        assert!(state.combat_state.is_regenerating);
        assert!(state.combat_state.current_enemy.is_none());

        // Update to complete regen
        update_combat(&mut state, HP_REGEN_DURATION_SECONDS, &HavenCombatBonuses::default());
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_player_died_in_dungeon() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Put player in a dungeon
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0));
        assert!(state.active_dungeon.is_some());

        // Force an attack that kills player
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        // Set up Zone 10 boss fight without Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4; // Zone 10 has 4 subzones, this is the zone boss
        state.zone_progression.fighting_boss = true;
        state.zone_progression.has_stormbreaker = false;

        let enemy_hp = 100;
        state.combat_state.current_enemy =
            Some(Enemy::new("Eternal Storm".to_string(), enemy_hp, 10));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        // Set up Zone 10 boss fight without Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.has_stormbreaker = false;

        let player_hp = state.combat_state.player_current_hp;
        state.combat_state.current_enemy = Some(Enemy::new("Eternal Storm".to_string(), 100, 10));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        // Set up Zone 10 boss fight without Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;
        state.zone_progression.has_stormbreaker = false;

        // Low HP so player dies
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Eternal Storm".to_string(), 100, 50));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Should have PlayerDied event
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Boss encounter should be reset
        assert!(!state.zone_progression.fighting_boss);
        assert_eq!(state.zone_progression.kills_in_subzone, 0);

        // Enemy should be cleared (not reset)
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_defense_reduces_enemy_damage() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Increase DEX for more defense (defense = DEX modifier)
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 20); // +5 modifier = 5 defense

        let initial_hp = state.combat_state.player_current_hp;
        let enemy_base_damage = 15;
        state.combat_state.current_enemy =
            Some(Enemy::new("Test".to_string(), 100, enemy_base_damage));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Calculate expected damage reduction
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let expected_damage = enemy_base_damage.saturating_sub(derived.defense);
        let actual_damage = initial_hp - state.combat_state.player_current_hp;

        assert_eq!(actual_damage, expected_damage);
    }

    #[test]
    fn test_defense_can_reduce_damage_to_zero() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // High DEX for high defense (defense = DEX modifier)
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 30); // +10 modifier = 10 defense

        let initial_hp = state.combat_state.player_current_hp;
        // Enemy damage lower than defense (5 < 10)
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 100, 5));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Player should take no damage (5 - 10 = 0 via saturating_sub)
        assert_eq!(state.combat_state.player_current_hp, initial_hp);
    }

    #[test]
    fn test_subzone_boss_defeat() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set up a subzone boss fight (not Zone 10)
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 1;
        state.zone_progression.fighting_boss = true;

        // Weak enemy that will die in one hit
        state.combat_state.current_enemy = Some(Enemy::new("Boss".to_string(), 1, 5));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        let initial_kills = state.zone_progression.kills_in_subzone;

        // Weak enemy
        state.combat_state.current_enemy = Some(Enemy::new("Mob".to_string(), 1, 5));

        // Force an attack to kill
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Even with attack timer ready, should not attack while regenerating
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // No combat events during regen
        assert!(events.is_empty());

        // Enemy should not have taken damage
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(enemy.current_hp, 100);
    }

    #[test]
    fn test_gradual_regeneration() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // Partial regen (half duration)
        update_combat(&mut state, HP_REGEN_DURATION_SECONDS / 2.0, &HavenCombatBonuses::default());

        // HP should be partially restored (roughly halfway)
        assert!(state.combat_state.player_current_hp > 10);
        assert!(state.combat_state.player_current_hp < 100);
        assert!(state.combat_state.is_regenerating);
    }

    #[test]
    fn test_death_to_any_boss_resets_encounter() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set up a normal boss fight (not weapon-blocked)
        state.zone_progression.current_zone_id = 5;
        state.zone_progression.current_subzone_id = 2;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Low HP so player dies
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Regular Boss".to_string(), 100, 50));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Should have PlayerDied event
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Boss encounter should be reset
        assert!(!state.zone_progression.fighting_boss);
        assert_eq!(state.zone_progression.kills_in_subzone, 0);

        // Enemy should be cleared
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_crit_doubles_damage() {
        // Verify that when a crit occurs, damage is exactly 2x base total_damage
        let mut state = GameState::new("Test Hero".to_string(), 0);

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
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
            s.attributes
                .set(crate::character::attributes::AttributeType::Dexterity, 0);
            s.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 100000, 0));
            s.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
            let events = update_combat(&mut s, 0.1, &HavenCombatBonuses::default());

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
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 16); // mod +3 => defense 3

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert_eq!(derived.defense, 3);

        let enemy_base_damage = 20;
        state.combat_state.current_enemy =
            Some(Enemy::new("Attacker".to_string(), 10000, enemy_base_damage));
        let initial_hp = state.combat_state.player_current_hp;

        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        let hp_lost = initial_hp - state.combat_state.player_current_hp;
        assert_eq!(hp_lost, enemy_base_damage - derived.defense);
    }

    #[test]
    fn test_multi_turn_combat_kills_enemy() {
        // Run combat over multiple turns until the enemy dies
        let mut state = GameState::new("Test Hero".to_string(), 0);
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
            state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
            let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());
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
                update_combat(&mut state, HP_REGEN_DURATION_SECONDS, &HavenCombatBonuses::default());
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
    fn test_elite_enemy_has_150_percent_stats() {
        // Run multiple generations and verify elite stats are roughly 1.5x base
        let player_hp = 200;
        let player_dmg = 50;

        let mut base_hp_sum: f64 = 0.0;
        let mut elite_hp_sum: f64 = 0.0;
        let samples = 500;

        for _ in 0..samples {
            let base = generate_enemy(player_hp, player_dmg);
            let elite = generate_elite_enemy(player_hp, player_dmg);
            base_hp_sum += base.max_hp as f64;
            elite_hp_sum += elite.max_hp as f64;
        }

        let avg_base = base_hp_sum / samples as f64;
        let avg_elite = elite_hp_sum / samples as f64;
        let ratio = avg_elite / avg_base;

        // Should be approximately 1.5x (allow 20% tolerance for random variance)
        assert!(
            (1.2..=1.8).contains(&ratio),
            "Elite HP ratio should be ~1.5x, got {:.2}x",
            ratio
        );
    }

    #[test]
    fn test_boss_enemy_has_200_percent_stats() {
        let player_hp = 200;
        let player_dmg = 50;

        let mut base_hp_sum: f64 = 0.0;
        let mut boss_hp_sum: f64 = 0.0;
        let samples = 500;

        for _ in 0..samples {
            let base = generate_enemy(player_hp, player_dmg);
            let boss = generate_boss_enemy(player_hp, player_dmg);
            base_hp_sum += base.max_hp as f64;
            boss_hp_sum += boss.max_hp as f64;
        }

        let avg_base = base_hp_sum / samples as f64;
        let avg_boss = boss_hp_sum / samples as f64;
        let ratio = avg_boss / avg_base;

        // Should be approximately 2.0x (allow 20% tolerance)
        assert!(
            (1.6..=2.4).contains(&ratio),
            "Boss HP ratio should be ~2.0x, got {:.2}x",
            ratio
        );
    }

    #[test]
    fn test_zone_scaling_increases_enemy_stats() {
        use crate::zones::get_all_zones;
        let zones = get_all_zones();
        let player_hp = 200;
        let player_dmg = 50;
        let samples = 300;

        // Zone 1 average HP
        let zone1 = &zones[0];
        let mut z1_hp: f64 = 0.0;
        for _ in 0..samples {
            let e = generate_zone_enemy(zone1, &zone1.subzones[0], player_hp, player_dmg);
            z1_hp += e.max_hp as f64;
        }

        // Zone 10 average HP
        let zone10 = &zones[9];
        let mut z10_hp: f64 = 0.0;
        for _ in 0..samples {
            let e = generate_zone_enemy(zone10, &zone10.subzones[0], player_hp, player_dmg);
            z10_hp += e.max_hp as f64;
        }

        let avg_z1 = z1_hp / samples as f64;
        let avg_z10 = z10_hp / samples as f64;

        // Zone 10 multiplier: 1.0 + (10-1)*0.1 = 1.9x
        assert!(
            avg_z10 > avg_z1 * 1.4,
            "Zone 10 enemies should be significantly stronger than zone 1 (z1={:.0}, z10={:.0})",
            avg_z1,
            avg_z10
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
        let initial_xp = state.character_xp;

        // Weak enemy that dies in one hit
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 1, 0));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
    fn test_enemy_zero_damage_with_high_defense() {
        // When defense >= enemy damage, player takes 0 damage
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 40); // mod 15 => defense 15

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert!(derived.defense >= 15);

        let initial_hp = state.combat_state.player_current_hp;
        // Enemy with damage less than defense
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 10000, 5));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Player should take zero damage from the enemy
        assert_eq!(state.combat_state.player_current_hp, initial_hp);
    }

    #[test]
    fn test_death_to_regular_enemy_resets_enemy_hp() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.zone_progression.fighting_boss = false;
        state.combat_state.player_current_hp = 1;

        let enemy_max_hp = 100;
        let mut enemy = Enemy::new("Regular".to_string(), enemy_max_hp, 50);
        enemy.take_damage(30); // Reduce to 70 HP
        state.combat_state.current_enemy = Some(enemy);

        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Regular enemy should have HP reset (not removed)
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(enemy.current_hp, enemy.max_hp);
    }

    #[test]
    fn test_prestige_rank_preserved_on_death() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set a prestige rank (3 = Gold)
        state.prestige_rank = 3;
        let original_rank = state.prestige_rank;

        // Set up boss fight
        state.zone_progression.fighting_boss = true;

        // Low HP so player dies
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Boss".to_string(), 100, 50));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        // Add gloves with +50% attack speed
        let gloves = Item {
            slot: EquipmentSlot::Gloves,
            rarity: Rarity::Rare,
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
        state.combat_state.attack_timer = 1.0;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        let attacked = events
            .iter()
            .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
        assert!(attacked, "Should attack with reduced interval");
    }

    #[test]
    fn test_attack_speed_normal_interval_without_affix() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));

        // Without attack speed bonus, 1.0 seconds is not enough (need 1.5)
        state.combat_state.attack_timer = 1.0;
        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        // Add armor with +100% HP regen (2x speed = half duration)
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
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
        update_combat(&mut state, 1.25, &HavenCombatBonuses::default());

        assert_eq!(state.combat_state.player_current_hp, 100);
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_hp_regen_normal_duration_without_affix() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // Without regen bonus, 1.25 seconds is not enough (need 2.5)
        update_combat(&mut state, 1.25, &HavenCombatBonuses::default());

        // Should still be regenerating, not fully healed
        assert!(state.combat_state.is_regenerating);
        assert!(state.combat_state.player_current_hp < 100);
    }

    #[test]
    fn test_haven_hp_regen_bonus() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

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
        update_combat(&mut state, 1.25, &haven);

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
        update_combat(&mut state, 0.84, &haven);

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

        // Set DEX to 0 to eliminate crit chance (base 5% + dex_mod, with DEX=0 gives 0%)
        state.attributes.set(AttributeType::Dexterity, 0);

        // Add armor with 50% damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
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
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

        update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

        let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

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

        // Set DEX to 0 to guarantee 0% crit chance
        state
            .attributes
            .set(crate::character::attributes::AttributeType::Dexterity, 0);

        // Add damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
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
        // Enemy deals 0 damage, so player takes 0 damage, so 0 is reflected
        state.combat_state.current_enemy =
            Some(Enemy::new("Pacifist".to_string(), enemy_max_hp, 0));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

        let initial_player_hp = state.combat_state.player_current_hp;
        update_combat(&mut state, 0.1, &HavenCombatBonuses::default());

        // Player took no damage
        assert_eq!(state.combat_state.player_current_hp, initial_player_hp);

        // Enemy should only take player attack damage, no reflection (0 damage = 0 reflected)
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let expected_hp = enemy_max_hp - derived.total_damage();
        assert_eq!(enemy.current_hp, expected_hp);
    }

    // =========================================================================
    // Haven Combat Bonus Tests
    // =========================================================================

    #[test]
    fn test_haven_damage_bonus() {
        use crate::character::attributes::AttributeType;

        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set DEX to 0 to eliminate crits
        state.attributes.set(AttributeType::Dexterity, 0);

        // Create an enemy with lots of HP
        state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

        // First, attack with no Haven bonus
        let events_no_bonus = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());
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
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

        let haven = HavenCombatBonuses {
            damage_percent: 50.0,
            ..Default::default()
        };
        let events_with_bonus = update_combat(&mut state, 0.1, &haven);
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
            state.attributes.set(AttributeType::Dexterity, 0); // Base 0% crit
            state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
            state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

            let events = update_combat(&mut state, 0.1, &HavenCombatBonuses::default());
            if events.iter().any(|e| matches!(e, CombatEvent::PlayerAttack { was_crit: true, .. })) {
                crits_no_bonus += 1;
            }
        }

        for _ in 0..trials {
            let mut state = GameState::new("Test Hero".to_string(), 0);
            state.attributes.set(AttributeType::Dexterity, 0);
            state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
            state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

            let haven = HavenCombatBonuses {
                crit_chance_percent: 20.0, // +20% crit
                ..Default::default()
            };
            let events = update_combat(&mut state, 0.1, &haven);
            if events.iter().any(|e| matches!(e, CombatEvent::PlayerAttack { was_crit: true, .. })) {
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
            state.attributes.set(AttributeType::Dexterity, 0);
            state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
            state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

            let haven = HavenCombatBonuses {
                double_strike_chance: 35.0, // +35% double strike (T3 War Room)
                ..Default::default()
            };
            let events = update_combat(&mut state, 0.1, &haven);

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

        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
        state.combat_state.player_current_hp = 10;
        state.combat_state.player_max_hp = 100;

        // With -50% regen delay, base duration is 2.5 * 0.5 = 1.25 seconds
        let haven = HavenCombatBonuses {
            hp_regen_delay_reduction: 50.0,
            ..Default::default()
        };
        update_combat(&mut state, 1.25, &haven);

        assert_eq!(state.combat_state.player_current_hp, 100);
        assert!(!state.combat_state.is_regenerating);
    }

    #[test]
    fn test_haven_combined_combat_bonuses() {
        use crate::character::attributes::AttributeType;

        // Test multiple bonuses together
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.attributes.set(AttributeType::Dexterity, 0);
        state.combat_state.current_enemy = Some(Enemy::new("Target".to_string(), 10000, 0));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;

        let haven = HavenCombatBonuses {
            damage_percent: 25.0,
            crit_chance_percent: 10.0,
            double_strike_chance: 10.0,
            hp_regen_percent: 50.0,
            hp_regen_delay_reduction: 30.0,
            xp_gain_percent: 20.0,
        };

        let events = update_combat(&mut state, 0.1, &haven);

        // Should have at least one attack
        assert!(
            events.iter().any(|e| matches!(e, CombatEvent::PlayerAttack { .. })),
            "Should have at least one attack event"
        );
    }
}
