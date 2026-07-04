use crate::character::derived_stats::DerivedStats;
use crate::core::constants::*;
use crate::core::game_state::GameState;
use rand::Rng;

use super::events::{CombatBonuses, CombatEvent};

/// Resolves a single enemy attack against the player.
///
/// Handles the full enemy damage pipeline:
/// 1. Total defense (derived + flat_defense bonus)
/// 2. Base damage after defense subtraction (min 1)
/// 3. damage_reduction_percent % applied after defense subtraction (e.g. Divine Bulwark, min 1)
/// 4. Apply damage to player HP
/// 5. Damage reflection back to attacker
/// 6. Check if reflection killed the enemy
/// 7. Check if player died (dungeon exit or boss retry reset)
///
/// Returns the events generated during this attack resolution.
/// The caller should append these events and, if the function returns
/// a non-empty vec, should return them (early return on enemy death
/// or player death).
pub(crate) fn resolve_enemy_attack<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    bonuses: &CombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
    fracture_zone_cap: u32,
    loom_zone_cap: u32,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    state.combat_state.enemy_attack_timer = 0.0;

    if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
        let base_defense = derived.defense.saturating_add(bonuses.flat_defense);
        let total_defense = (base_defense as f64 * bonuses.ascension_multiplier) as u64;
        let base_damage = enemy.damage.saturating_sub(total_defense).max(1);
        // Apply damage reduction (e.g. Divine Bulwark)
        let enemy_damage = if bonuses.damage_reduction_percent > 0.0 {
            (((base_damage as f64) * (1.0 - bonuses.damage_reduction_percent / 100.0)) as u64)
                .max(1)
        } else {
            base_damage
        };
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
                (enemy_damage as f64 * derived.damage_reflection_percent / 100.0) as u64;
            if reflected > 0 {
                enemy.take_damage(reflected);
                events.push(CombatEvent::DamageReflected { damage: reflected });
            }
        }

        // Check if reflection killed the enemy
        if !enemy.is_alive() {
            let (death_events, _) = super::damage::handle_enemy_death(
                rng,
                state,
                achievements,
                bonuses.xp_gain_percent,
                fracture_zone_cap,
                loom_zone_cap,
            );
            events.extend(death_events);
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
                if state.zone_progression.fighting_boss {
                    // Boss death: retreat immediately
                    return super::orchestration::resolve_combat_retreat(state, true);
                }

                // Mob death: track consecutive deaths, retreat after threshold
                state.consecutive_deaths += 1;
                if state.consecutive_deaths >= DEATH_LOOP_THRESHOLD {
                    return super::orchestration::resolve_combat_retreat(state, true);
                }

                if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                    enemy.reset_hp();
                }
            } else {
                // In dungeon, clear the enemy since we're exiting
                state.combat_state.current_enemy = None;
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::Achievements;
    use crate::combat::types::Enemy;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn default_derived(state: &GameState) -> DerivedStats {
        DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7])
    }

    #[test]
    fn resolve_enemy_attack_without_current_enemy_is_a_noop() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = None;
        state.combat_state.enemy_attack_timer = 5.0;
        let derived = default_derived(&state);

        let events = resolve_enemy_attack(
            &mut rng,
            &mut state,
            &CombatBonuses::default(),
            &mut achievements,
            &derived,
            11,
            30,
        );

        assert!(events.is_empty());
        assert_eq!(
            state.combat_state.enemy_attack_timer, 0.0,
            "Timer resets even when there is no enemy to attack"
        );
    }

    #[test]
    fn resolve_enemy_attack_reflection_rounds_down_to_zero_emits_no_event() {
        // 1% reflection of the 1-damage min floor rounds down to 0, so no
        // DamageReflected event should fire and the enemy should take no
        // reflected damage.
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 1000, 0));
        let mut derived = default_derived(&state);
        derived.damage_reflection_percent = 1.0;

        let events = resolve_enemy_attack(
            &mut rng,
            &mut state,
            &CombatBonuses::default(),
            &mut achievements,
            &derived,
            11,
            30,
        );

        assert!(
            !events
                .iter()
                .any(|e| matches!(e, CombatEvent::DamageReflected { .. })),
            "Rounded-to-zero reflection should not emit an event"
        );
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(
            enemy.current_hp, enemy.max_hp,
            "Enemy should take no reflected damage when reflected amount rounds to 0"
        );
    }

    #[test]
    fn resolve_enemy_attack_mob_death_loop_retreats_after_threshold() {
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.zone_progression.current_zone_id = 6;
        state.zone_progression.current_subzone_id = 2;
        state.consecutive_deaths = DEATH_LOOP_THRESHOLD - 1;
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Killer".to_string(), 100, 999));
        let derived = default_derived(&state);

        let events = resolve_enemy_attack(
            &mut rng,
            &mut state,
            &CombatBonuses::default(),
            &mut achievements,
            &derived,
            11,
            30,
        );

        assert!(
            events
                .iter()
                .any(|e| matches!(e, CombatEvent::CombatRetreat { .. })),
            "Reaching the death-loop threshold should trigger a retreat"
        );
        assert_eq!(
            state.consecutive_deaths, 0,
            "Retreat resets the consecutive death counter"
        );
    }

    #[test]
    fn resolve_enemy_attack_mob_death_below_threshold_resets_enemy_and_continues() {
        let mut rng = ChaCha8Rng::seed_from_u64(4);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Killer".to_string(), 100, 999));
        let derived = default_derived(&state);

        let events = resolve_enemy_attack(
            &mut rng,
            &mut state,
            &CombatBonuses::default(),
            &mut achievements,
            &derived,
            11,
            30,
        );

        assert!(events.iter().any(|e| matches!(e, CombatEvent::PlayerDied)));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, CombatEvent::CombatRetreat { .. })),
            "Should not retreat before hitting the death-loop threshold"
        );
        assert_eq!(state.consecutive_deaths, 1);
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(
            enemy.current_hp, enemy.max_hp,
            "Enemy HP resets so the mob fight continues"
        );
    }

    #[test]
    fn resolve_enemy_attack_boss_death_retreats_immediately() {
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        state.zone_progression.current_zone_id = 5;
        state.zone_progression.current_subzone_id = 2;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;
        state.combat_state.player_current_hp = 1;
        state.combat_state.current_enemy = Some(Enemy::new("Boss".to_string(), 500, 999));
        let derived = default_derived(&state);

        let events = resolve_enemy_attack(
            &mut rng,
            &mut state,
            &CombatBonuses::default(),
            &mut achievements,
            &derived,
            11,
            30,
        );

        assert!(
            events
                .iter()
                .any(|e| matches!(e, CombatEvent::CombatRetreat { .. })),
            "Boss death should retreat immediately regardless of death-loop threshold"
        );
        assert!(!state.zone_progression.fighting_boss);
        assert!(state.combat_state.current_enemy.is_none());
    }
}
