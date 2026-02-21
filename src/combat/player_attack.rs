use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::core::game_state::GameState;
use rand::{Rng, RngExt};

use super::events::{CombatEvent, GodItemCombatBonuses, HavenCombatBonuses};

/// Resolves a single player attack against the current enemy.
///
/// Handles the full player damage pipeline:
/// 1. Weapon gate check (boss may require a specific weapon)
/// 2. Base damage from DerivedStats
/// 3. Giant's Might % (god item damage bonus)
/// 4. Haven Armory % (Haven damage bonus)
/// 5. Prestige flat damage
/// 6. Enemy defense subtraction (min 1)
/// 7. Crit roll and multiplier
/// 8. Double strike roll (War Room bonus)
/// 9. Apply damage to enemy (potentially multiple strikes)
/// 10. Handle enemy death if killed
///
/// Returns `(events, enemy_died)`. If the enemy died, the caller should
/// return early (no further combat phases this tick).
pub(crate) fn resolve_player_attack<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    haven: &HavenCombatBonuses,
    prestige_bonuses: &PrestigeCombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
    god_items: &GodItemCombatBonuses,
) -> (Vec<CombatEvent>, bool) {
    let mut events = Vec::new();

    state.combat_state.player_attack_timer = 0.0;

    // Check if boss requires a weapon we don't have
    if let Some(weapon_name) = state.zone_progression.boss_weapon_blocked(achievements) {
        // Attack is blocked - no damage dealt
        events.push(CombatEvent::PlayerAttackBlocked {
            weapon_needed: weapon_name.to_string(),
        });
        return (events, false);
    }

    // Player attacks normally
    // 1. Base damage from DerivedStats (STR/INT + equipment)
    let base_damage = derived.total_damage();
    // 1b. Apply Giant's Might: +% base damage
    let god_boosted_damage = (base_damage as f64 * (1.0 + god_items.damage_percent / 100.0)) as u32;
    // 2. Apply Haven Armory multiplier: +% damage
    let haven_damage = (god_boosted_damage as f64 * (1.0 + haven.damage_percent / 100.0)) as u32;
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
    let crit_roll = rng.random_range(0..100);
    if crit_roll < total_crit_chance {
        damage = (damage as f64 * derived.crit_multiplier) as u32;
        was_crit = true;
    }

    // Roll for double strike (War Room bonus)
    let double_strike_roll = rng.random::<f64>() * 100.0;
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
            let (death_events, _) =
                super::damage::handle_enemy_death(rng, state, achievements, haven.xp_gain_percent);
            events.extend(death_events);
            return (events, true);
        }
    }

    (events, false)
}
