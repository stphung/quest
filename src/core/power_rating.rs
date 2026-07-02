//! Character power rating — a single number capturing overall combat strength.
//!
//! Power = sqrt(effective_DPS × effective_HP), a geometric mean that naturally
//! balances offense and survivability. All bonus sources (equipment, enhancement,
//! prestige, Haven, god items, sigils, ascension) flow through DerivedStats and
//! CombatBonuses, so they're automatically incorporated.

use crate::character::derived_stats::DerivedStats;
use crate::combat::events::CombatBonuses;
use crate::core::constants::ATTACK_INTERVAL_SECONDS;

/// Compute the character's power rating from current stats and bonuses.
///
/// Inputs:
/// - `derived`: Base stats from attributes + equipment + enhancement levels
/// - `bonuses`: Merged combat bonuses (Haven, god items, prestige, sigils, ascension)
/// - `player_max_hp`: Fully-boosted max HP (after flat prestige HP, ascension, sigil HP%)
pub fn compute_power_rating(
    derived: &DerivedStats,
    bonuses: &CombatBonuses,
    player_max_hp: u64,
) -> f64 {
    // ── Effective damage per hit (mirrors player_attack.rs pipeline) ──
    let base_dmg = derived.physical_damage.max(derived.magic_damage) as f64;
    let after_early = base_dmg * (1.0 + bonuses.early_damage_percent / 100.0);
    let after_haven = after_early * (1.0 + bonuses.damage_percent / 100.0);
    let after_flat = after_haven + bonuses.flat_damage as f64;
    let after_asc = after_flat * bonuses.ascension_multiplier;

    // ── Effective crit factor ──
    let total_crit = (derived.crit_chance_percent as f64 + bonuses.crit_chance_percent).min(100.0);
    let crit_factor = 1.0 + (total_crit / 100.0) * (derived.crit_multiplier - 1.0);

    // ── Double strike factor ──
    let double_strike = 1.0 + bonuses.double_strike_chance / 100.0;

    // ── Hits per second (attack speed) ──
    let effective_interval = ATTACK_INTERVAL_SECONDS / (1.0 + bonuses.attack_speed_percent / 100.0);
    let hits_per_second = 1.0 / effective_interval;

    // ── Effective DPS ──
    let effective_dps = after_asc * crit_factor * double_strike * hits_per_second;

    // ── Effective HP (DR makes HP worth more) ──
    let dr = bonuses.damage_reduction_percent.min(99.0);
    let dr_factor = 1.0 / (1.0 - dr / 100.0);
    let effective_hp = player_max_hp as f64 * dr_factor;

    // ── Power = geometric mean of DPS and eHP ──
    (effective_dps * effective_hp).sqrt().round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_character_power() {
        let derived = DerivedStats::default();
        let bonuses = CombatBonuses::default();
        let power = compute_power_rating(&derived, &bonuses, 50);
        // Fresh character: low but positive
        assert!(power > 0.0, "Power should be positive: {}", power);
        assert!(
            power < 100.0,
            "Fresh character power should be modest: {}",
            power
        );
    }

    #[test]
    fn test_power_increases_with_ascension() {
        let derived = DerivedStats {
            physical_damage: 50,
            max_hp: 100,
            crit_chance_percent: 10,
            crit_multiplier: 2.0,
            ..DerivedStats::default()
        };
        let base_bonuses = CombatBonuses::default();
        let base_power = compute_power_rating(&derived, &base_bonuses, 100);

        let asc_bonuses = CombatBonuses {
            ascension_multiplier: 64.0,
            ..CombatBonuses::default()
        };
        // Ascension boosts HP too: 100 * 64 = 6400
        let asc_power = compute_power_rating(&derived, &asc_bonuses, 6400);

        assert!(
            asc_power > base_power * 10.0,
            "Ascension VI should dramatically increase power: base={}, asc={}",
            base_power,
            asc_power
        );
    }

    #[test]
    fn test_power_increases_with_damage() {
        let low = DerivedStats {
            physical_damage: 10,
            max_hp: 50,
            crit_chance_percent: 5,
            crit_multiplier: 2.0,
            ..DerivedStats::default()
        };
        let high = DerivedStats {
            physical_damage: 100,
            max_hp: 50,
            crit_chance_percent: 5,
            crit_multiplier: 2.0,
            ..DerivedStats::default()
        };
        let bonuses = CombatBonuses::default();
        let low_power = compute_power_rating(&low, &bonuses, 50);
        let high_power = compute_power_rating(&high, &bonuses, 50);
        assert!(high_power > low_power);
    }

    #[test]
    fn test_power_increases_with_hp() {
        let derived = DerivedStats {
            physical_damage: 50,
            max_hp: 100,
            crit_chance_percent: 5,
            crit_multiplier: 2.0,
            ..DerivedStats::default()
        };
        let bonuses = CombatBonuses::default();
        let low_hp_power = compute_power_rating(&derived, &bonuses, 100);
        let high_hp_power = compute_power_rating(&derived, &bonuses, 500);
        assert!(high_hp_power > low_hp_power);
    }
}
