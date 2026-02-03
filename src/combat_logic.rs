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
        // HP regen multiplier: higher = faster regen
        let regen_derived =
            DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let effective_regen_duration =
            HP_REGEN_DURATION_SECONDS / regen_derived.hp_regen_multiplier;

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
            let mut damage = derived.total_damage();
            let mut was_crit = false;

            // Roll for crit
            let crit_roll = rand::thread_rng().gen_range(0..100);
            if crit_roll < derived.crit_chance_percent {
                damage = (damage as f64 * derived.crit_multiplier) as u32;
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
    fn test_crit_doubles_damage() {
        // Verify that when a crit occurs, damage is exactly 2x base total_damage
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set high DEX for 100% crit chance (need crit_chance_percent >= 100)
        // crit_chance_percent = 5 + DEX_mod; DEX 210 gives mod 100 => 105%
        state
            .attributes
            .set(crate::attributes::AttributeType::Dexterity, 210);

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert!(derived.crit_chance_percent >= 100);
        let expected_crit_damage = derived.total_damage() * 2;

        // Give enemy enough HP to survive
        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1);

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
            .set(crate::attributes::AttributeType::Dexterity, 0);

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert_eq!(derived.crit_chance_percent, 0);

        // Run many attacks to confirm no crits
        let mut crit_count = 0;
        for _ in 0..100 {
            let mut s = GameState::new("Test Hero".to_string(), 0);
            s.attributes
                .set(crate::attributes::AttributeType::Dexterity, 0);
            s.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 100000, 0));
            s.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
            let events = update_combat(&mut s, 0.1);

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
            .set(crate::attributes::AttributeType::Dexterity, 0); // 0% crit
        state
            .attributes
            .set(crate::attributes::AttributeType::Strength, 20); // +5 mod => phys 15
        state
            .attributes
            .set(crate::attributes::AttributeType::Intelligence, 16); // +3 mod => magic 11

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let expected_damage = derived.total_damage(); // 15 + 11 = 26

        state.combat_state.current_enemy = Some(Enemy::new("Dummy".to_string(), 10000, 0));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1);

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
            .set(crate::attributes::AttributeType::Dexterity, 16); // mod +3 => defense 3

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert_eq!(derived.defense, 3);

        let enemy_base_damage = 20;
        state.combat_state.current_enemy =
            Some(Enemy::new("Attacker".to_string(), 10000, enemy_base_damage));
        let initial_hp = state.combat_state.player_current_hp;

        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1);

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
            .set(crate::attributes::AttributeType::Strength, 30);
        state
            .attributes
            .set(crate::attributes::AttributeType::Dexterity, 0);
        state
            .attributes
            .set(crate::attributes::AttributeType::Constitution, 30);

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
            let events = update_combat(&mut state, 0.1);
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
                update_combat(&mut state, HP_REGEN_DURATION_SECONDS);
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
            let base = crate::combat::generate_enemy(player_hp, player_dmg);
            let elite = crate::combat::generate_elite_enemy(player_hp, player_dmg);
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
            let base = crate::combat::generate_enemy(player_hp, player_dmg);
            let boss = crate::combat::generate_boss_enemy(player_hp, player_dmg);
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
            let e = crate::combat::generate_zone_enemy(
                zone1,
                &zone1.subzones[0],
                player_hp,
                player_dmg,
            );
            z1_hp += e.max_hp as f64;
        }

        // Zone 10 average HP
        let zone10 = &zones[9];
        let mut z10_hp: f64 = 0.0;
        for _ in 0..samples {
            let e = crate::combat::generate_zone_enemy(
                zone10,
                &zone10.subzones[0],
                player_hp,
                player_dmg,
            );
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
        let xp_per_tick = crate::game_logic::xp_gain_per_tick(0, 0, 0);
        let min_expected = xp_per_tick * COMBAT_XP_MIN_TICKS as f64;
        let max_expected = xp_per_tick * COMBAT_XP_MAX_TICKS as f64;

        for _ in 0..100 {
            let xp = crate::game_logic::combat_kill_xp(xp_per_tick);
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
        let events = update_combat(&mut state, 0.1);

        let xp_event = events.iter().find_map(|e| match e {
            CombatEvent::EnemyDied { xp_gained } => Some(*xp_gained),
            _ => None,
        });

        assert!(xp_event.is_some(), "Should emit EnemyDied with XP");
        let xp_gained = xp_event.unwrap();
        assert!(xp_gained > 0, "XP gained should be positive");

        // XP should be in the combat kill range
        let xp_per_tick = crate::game_logic::xp_gain_per_tick(0, 0, 0);
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
        let mut combat = crate::combat::CombatState::new(100);
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
        let mut combat = crate::combat::CombatState::new(100);

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
            .set(crate::attributes::AttributeType::Dexterity, 40); // mod 15 => defense 15

        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        assert!(derived.defense >= 15);

        let initial_hp = state.combat_state.player_current_hp;
        // Enemy with damage less than defense
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 10000, 5));
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

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
        let events = update_combat(&mut state, 0.1);

        // Verify player died
        let died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
        assert!(died);

        // Prestige rank should NOT be changed
        assert_eq!(state.prestige_rank, original_rank);
    }
}
