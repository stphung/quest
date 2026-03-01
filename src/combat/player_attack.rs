use crate::character::derived_stats::DerivedStats;
use crate::core::game_state::GameState;
use rand::{Rng, RngExt};

use super::events::{CombatBonuses, CombatEvent};

/// Resolves a single player attack against the current enemy.
///
/// Handles the full player damage pipeline:
/// 1. Weapon gate check (boss may require a specific weapon)
/// 2. Base damage from DerivedStats
/// 3. early_damage_percent % (e.g. Giant's Might bonus)
/// 4. damage_percent % (e.g. Haven Armory bonus)
/// 5. flat_damage added after % bonuses, before enemy defense
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
    bonuses: &CombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
    fracture_zone_cap: u32,
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
    // 1b. Apply early damage percent (e.g. Giant's Might): +% base damage
    let early_boosted_damage =
        (base_damage as f64 * (1.0 + bonuses.early_damage_percent / 100.0)) as u32;
    // 2. Apply damage percent multiplier (e.g. Haven Armory): +% damage
    let boosted_damage =
        (early_boosted_damage as f64 * (1.0 + bonuses.damage_percent / 100.0)) as u32;
    // 3. Apply flat damage bonus (e.g. prestige), added after % bonuses, before defense
    let pre_defense_damage = boosted_damage + bonuses.flat_damage;
    // 4. Apply Ascension multiplier to damage
    let pre_crit_damage = (pre_defense_damage as f64 * bonuses.ascension_multiplier) as u32;
    // 5. Apply enemy defense: min damage floor of 1
    let enemy_def = state
        .combat_state
        .current_enemy
        .as_ref()
        .map_or(0, |e| e.defense);
    let mut damage = pre_crit_damage.saturating_sub(enemy_def).max(1);
    let mut was_crit = false;

    // Roll for crit (base from derived + crit_chance_percent from bonuses, already includes prestige)
    let total_crit_chance = derived.crit_chance_percent as f64 + bonuses.crit_chance_percent;
    let crit_roll = rng.random_range(0..100);
    if (crit_roll as f64) < total_crit_chance {
        damage = (damage as f64 * derived.crit_multiplier) as u32;
        was_crit = true;
    }

    // Roll for double strike
    let double_strike_roll = rng.random::<f64>() * 100.0;
    let num_strikes = if double_strike_roll < bonuses.double_strike_chance {
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
            let (death_events, _) = super::damage::handle_enemy_death(
                rng,
                state,
                achievements,
                bonuses.xp_gain_percent,
                fracture_zone_cap,
            );
            events.extend(death_events);
            return (events, true);
        }
    }

    (events, false)
}
