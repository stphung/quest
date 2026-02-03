use crate::constants::*;
use crate::derived_stats::DerivedStats;
use crate::dungeon::RoomType;
use crate::game_state::GameState;
use rand::Rng;

use crate::zones::BossDefeatResult;

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
pub fn update_combat(state: &mut GameState, delta_time: f64) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // Handle regeneration after enemy death
    if state.combat_state.is_regenerating {
        state.combat_state.regen_timer += delta_time;

        if state.combat_state.regen_timer >= HP_REGEN_DURATION_SECONDS {
            state.combat_state.player_current_hp = state.combat_state.player_max_hp;
            state.combat_state.is_regenerating = false;
            state.combat_state.regen_timer = 0.0;
        } else {
            // Gradual regen
            let regen_progress = state.combat_state.regen_timer / HP_REGEN_DURATION_SECONDS;
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

    if state.combat_state.attack_timer >= ATTACK_INTERVAL_SECONDS {
        state.combat_state.attack_timer = 0.0;

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);

        // Check if boss requires a weapon we don't have
        if let Some(weapon_name) = state.zone_progression.boss_weapon_blocked() {
            // Attack is blocked - no damage dealt
            events.push(CombatEvent::PlayerAttackBlocked {
                weapon_needed: weapon_name.to_string(),
            });

            // Enemy still attacks back (see below)
        } else {
            // Player attacks normally
            let mut damage = derived.total_damage();
            let mut was_crit = false;

            // Roll for crit
            let crit_roll = rand::thread_rng().gen_range(0..100);
            if crit_roll < derived.crit_chance_percent {
                damage *= 2;
                was_crit = true;
            }

            if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                enemy.take_damage(damage);
                events.push(CombatEvent::PlayerAttack { damage, was_crit });

                // Check if enemy died
                if !enemy.is_alive() {
                    let wis_mod = state
                        .attributes
                        .modifier(crate::attributes::AttributeType::Wisdom);
                    let cha_mod = state
                        .attributes
                        .modifier(crate::attributes::AttributeType::Charisma);
                    let xp_gained = crate::game_logic::combat_kill_xp(
                        crate::game_logic::xp_gain_per_tick(state.prestige_rank, wis_mod, cha_mod),
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
        if let Some(enemy) = state.combat_state.current_enemy.as_ref() {
            let enemy_damage = enemy.damage.saturating_sub(derived.defense);
            state.combat_state.player_current_hp = state
                .combat_state
                .player_current_hp
                .saturating_sub(enemy_damage);

            events.push(CombatEvent::EnemyAttack {
                damage: enemy_damage,
            });

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
    use super::*;
    use crate::combat::Enemy;

    #[test]
    fn test_update_combat_no_enemy() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let events = update_combat(&mut state, 0.1);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_update_combat_attack_interval() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 5));

        // Not enough time passed
        let events = update_combat(&mut state, 0.5);
        assert_eq!(events.len(), 0);

        // Enough time for attack
        let events = update_combat(&mut state, 1.0);
        assert!(events.len() >= 2); // Player attack + enemy attack
    }

    #[test]
    fn test_player_died_resets() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 50));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

        // Should have enemy died event
        let died = events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
        assert!(died);

        // Should be regenerating
        assert!(state.combat_state.is_regenerating);
        assert!(state.combat_state.current_enemy.is_none());

        // Update to complete regen
        update_combat(&mut state, HP_REGEN_DURATION_SECONDS);
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
        state.active_dungeon = Some(crate::dungeon_generation::generate_dungeon(1, 0));
        assert!(state.active_dungeon.is_some());

        // Force an attack that kills player
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
            .set(crate::attributes::AttributeType::Dexterity, 20); // +5 modifier = 5 defense

        let initial_hp = state.combat_state.player_current_hp;
        let enemy_base_damage = 15;
        state.combat_state.current_enemy =
            Some(Enemy::new("Test".to_string(), 100, enemy_base_damage));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1);

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
            .set(crate::attributes::AttributeType::Dexterity, 30); // +10 modifier = 10 defense

        let initial_hp = state.combat_state.player_current_hp;
        // Enemy damage lower than defense (5 < 10)
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 100, 5));

        // Force an attack
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
        update_combat(&mut state, HP_REGEN_DURATION_SECONDS / 2.0);

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
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

        // Verify player died
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Prestige rank should NOT be changed
        assert_eq!(state.prestige_rank, original_rank);
    }
}
