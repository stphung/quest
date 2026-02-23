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
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    state.combat_state.enemy_attack_timer = 0.0;

    if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
        let total_defense = derived.defense + bonuses.flat_defense;
        let base_damage = enemy.damage.saturating_sub(total_defense).max(1);
        // Apply damage reduction (e.g. Divine Bulwark)
        let enemy_damage = if bonuses.damage_reduction_percent > 0.0 {
            (((base_damage as f64) * (1.0 - bonuses.damage_reduction_percent / 100.0)) as u32)
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
                (enemy_damage as f64 * derived.damage_reflection_percent / 100.0) as u32;
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
                // Check if we died to a boss
                if state.zone_progression.fighting_boss {
                    state.zone_progression.fighting_boss = false;
                    state.combat_state.current_enemy = None;
                    state.combat_state.boss_fight_timer = 0.0;

                    // Zone boss death: reset to subzone 1 as if just arriving
                    // Subzone boss death: 5 kills to retry in same subzone
                    let is_zone_boss =
                        crate::zones::get_zone(state.zone_progression.current_zone_id)
                            .and_then(|zone| {
                                zone.subzones
                                    .iter()
                                    .find(|s| s.id == state.zone_progression.current_subzone_id)
                            })
                            .map(|sub| sub.boss.is_zone_boss)
                            .unwrap_or(false);

                    if is_zone_boss {
                        state.zone_progression.current_subzone_id = 1;
                        state.zone_progression.kills_in_subzone = 0;
                    } else {
                        state.zone_progression.kills_in_subzone =
                            KILLS_FOR_BOSS.saturating_sub(KILLS_FOR_BOSS_RETRY);
                    }
                } else if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                    // Track consecutive deaths for death loop detection
                    state.consecutive_deaths += 1;

                    // Check death loop threshold — trigger retreat
                    if state.consecutive_deaths >= DEATH_LOOP_THRESHOLD {
                        return super::orchestration::resolve_combat_retreat(state);
                    }

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
