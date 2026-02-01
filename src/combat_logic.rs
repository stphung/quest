use crate::constants::*;
use crate::derived_stats::DerivedStats;
use crate::game_state::GameState;
use rand::Rng;

pub enum CombatEvent {
    PlayerAttack { damage: u32, was_crit: bool },
    EnemyAttack { damage: u32 },
    PlayerDied,
    EnemyDied { xp_gained: u64 },
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

        let derived = DerivedStats::from_attributes(&state.attributes);

        // Player attacks
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
                let xp_gained = crate::game_logic::combat_kill_xp(
                    crate::game_logic::xp_gain_per_tick(
                        state.prestige_rank,
                        derived.xp_multiplier as i32,
                        state.attributes.modifier(crate::attributes::AttributeType::Charisma),
                    ),
                );

                events.push(CombatEvent::EnemyDied { xp_gained });

                // Remove enemy and start regeneration
                state.combat_state.current_enemy = None;
                state.combat_state.is_regenerating = true;
                state.combat_state.regen_timer = 0.0;

                return events;
            }
        }

        // Enemy attacks back
        if let Some(enemy) = state.combat_state.current_enemy.as_ref() {
            let enemy_damage = enemy.damage.saturating_sub(derived.defense);
            state.combat_state.player_current_hp =
                state.combat_state.player_current_hp.saturating_sub(enemy_damage);

            events.push(CombatEvent::EnemyAttack {
                damage: enemy_damage,
            });

            // Check if player died
            if !state.combat_state.is_player_alive() {
                events.push(CombatEvent::PlayerDied);

                // Reset both to full HP
                state.combat_state.player_current_hp = state.combat_state.player_max_hp;
                if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                    enemy.reset_hp();
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
        let mut state = GameState::new(0);
        let events = update_combat(&mut state, 0.1);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_update_combat_attack_interval() {
        let mut state = GameState::new(0);
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
        let mut state = GameState::new(0);
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
        let mut state = GameState::new(0);
        state.combat_state.player_current_hp = 10;
        state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 1, 5));

        // Force attack to kill enemy
        state.combat_state.attack_timer = ATTACK_INTERVAL_SECONDS;
        let events = update_combat(&mut state, 0.1);

        // Should have enemy died event
        let died = events.iter().any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
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
}
