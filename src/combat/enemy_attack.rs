use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::core::constants::*;
use crate::core::game_state::GameState;

use super::events::{CombatEvent, GodItemCombatBonuses, HavenCombatBonuses};

/// Resolves a single enemy attack against the player.
///
/// Handles the full enemy damage pipeline:
/// 1. Total defense (derived + prestige flat defense)
/// 2. Base damage after defense subtraction (min 1)
/// 3. Divine Bulwark damage reduction % (god item, min 1)
/// 4. Apply damage to player HP
/// 5. Damage reflection back to attacker
/// 6. Check if reflection killed the enemy
/// 7. Check if player died (dungeon exit or boss retry reset)
///
/// Returns the events generated during this attack resolution.
/// The caller should append these events and, if the function returns
/// a non-empty vec, should return them (early return on enemy death
/// or player death).
pub(crate) fn resolve_enemy_attack(
    state: &mut GameState,
    haven: &HavenCombatBonuses,
    prestige_bonuses: &PrestigeCombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
    god_items: &GodItemCombatBonuses,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    state.combat_state.enemy_attack_timer = 0.0;

    if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
        let total_defense = derived.defense + prestige_bonuses.flat_defense;
        let base_damage = enemy.damage.saturating_sub(total_defense).max(1);
        // Apply Divine Bulwark (god item damage reduction)
        let enemy_damage = if god_items.damage_reduction_percent > 0.0 {
            (((base_damage as f64) * (1.0 - god_items.damage_reduction_percent / 100.0)) as u32)
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
            let (death_events, _) =
                super::damage::handle_enemy_death(state, achievements, haven.xp_gain_percent);
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

    events
}
