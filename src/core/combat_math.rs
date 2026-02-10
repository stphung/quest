//! Shared combat math functions for game and simulator.
//!
//! These pure functions calculate combat outcomes without side effects.
//! Both the real game and simulator use these for consistent combat math.

// Allow dead code - these functions are being integrated incrementally
#![allow(dead_code)]

use crate::character::derived_stats::DerivedStats;
use rand::Rng;

/// Result of a player attack calculation.
#[derive(Debug, Clone, Copy)]
pub struct AttackResult {
    /// Damage dealt (after crit multiplier if applicable).
    pub damage: u32,
    /// Whether this attack was a critical hit.
    pub is_crit: bool,
}

/// Calculate player attack damage with crit roll.
///
/// # Arguments
/// * `stats` - Player's derived stats (damage, crit chance, crit multiplier)
/// * `bonus_crit_chance` - Additional crit chance (e.g., from Haven)
/// * `damage_multiplier` - Damage multiplier (e.g., 1.0 + haven bonus)
/// * `rng` - Random number generator
///
/// # Returns
/// AttackResult with final damage and crit flag
pub fn calculate_player_attack(
    stats: &DerivedStats,
    bonus_crit_chance: u32,
    damage_multiplier: f64,
    rng: &mut impl Rng,
) -> AttackResult {
    let base_damage = stats.total_damage();
    let mut damage = (base_damage as f64 * damage_multiplier) as u32;

    // Roll for crit
    let total_crit_chance = stats.crit_chance_percent + bonus_crit_chance;
    let is_crit = roll_crit(total_crit_chance, rng);

    if is_crit {
        damage = (damage as f64 * stats.crit_multiplier) as u32;
    }

    AttackResult { damage, is_crit }
}

/// Simplified attack calculation (no bonuses).
/// Convenience wrapper for simulator and tests.
pub fn calculate_attack_simple(stats: &DerivedStats, rng: &mut impl Rng) -> AttackResult {
    calculate_player_attack(stats, 0, 1.0, rng)
}

/// Roll for critical hit.
///
/// # Arguments
/// * `crit_chance_percent` - Chance to crit (0-100+)
/// * `rng` - Random number generator
///
/// # Returns
/// true if crit, false otherwise
pub fn roll_crit(crit_chance_percent: u32, rng: &mut impl Rng) -> bool {
    let roll = rng.gen_range(0..100);
    roll < crit_chance_percent
}

/// Calculate actual damage taken after defense.
///
/// # Arguments
/// * `raw_damage` - Incoming damage before defense
/// * `defense` - Defense stat
///
/// # Returns
/// Actual damage taken (minimum 0)
pub fn calculate_damage_taken(raw_damage: u32, defense: u32) -> u32 {
    raw_damage.saturating_sub(defense)
}

/// Apply damage to HP, returning remaining HP.
///
/// # Arguments
/// * `current_hp` - Current HP before damage
/// * `damage` - Damage to apply
///
/// # Returns
/// HP remaining after damage (minimum 0)
pub fn apply_damage(current_hp: u32, damage: u32) -> u32 {
    current_hp.saturating_sub(damage)
}

/// Check if entity is still alive.
pub fn is_alive(current_hp: u32) -> bool {
    current_hp > 0
}

/// Calculate damage reflection.
///
/// # Arguments
/// * `damage_taken` - Damage taken by the reflector
/// * `reflection_percent` - Percentage of damage to reflect (0-100)
///
/// # Returns
/// Damage to reflect back to attacker
pub fn calculate_damage_reflection(damage_taken: u32, reflection_percent: f64) -> u32 {
    if reflection_percent > 0.0 && damage_taken > 0 {
        (damage_taken as f64 * reflection_percent / 100.0) as u32
    } else {
        0
    }
}

/// Simulate one round of combat (player attacks, enemy attacks back).
///
/// # Arguments
/// * `player_stats` - Player's derived stats
/// * `player_hp` - Player's current HP
/// * `enemy_hp` - Enemy's current HP
/// * `enemy_damage` - Enemy's damage per attack
/// * `rng` - Random number generator
///
/// # Returns
/// (player_hp_after, enemy_hp_after, attack_result)
pub fn simulate_combat_round(
    player_stats: &DerivedStats,
    player_hp: u32,
    enemy_hp: u32,
    enemy_damage: u32,
    rng: &mut impl Rng,
) -> (u32, u32, AttackResult) {
    // Player attacks
    let attack = calculate_attack_simple(player_stats, rng);
    let enemy_hp_after = apply_damage(enemy_hp, attack.damage);

    // Enemy attacks back (if still alive)
    let player_hp_after = if is_alive(enemy_hp_after) {
        let damage_taken = calculate_damage_taken(enemy_damage, player_stats.defense);
        apply_damage(player_hp, damage_taken)
    } else {
        player_hp
    };

    (player_hp_after, enemy_hp_after, attack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::attributes::Attributes;
    use crate::items::Equipment;

    fn make_test_stats() -> DerivedStats {
        let attrs = Attributes::default();
        let equip = Equipment::new();
        DerivedStats::calculate_derived_stats(&attrs, &equip)
    }

    #[test]
    fn test_calculate_damage_taken() {
        assert_eq!(calculate_damage_taken(20, 5), 15);
        assert_eq!(calculate_damage_taken(5, 10), 0); // Can't go negative
        assert_eq!(calculate_damage_taken(10, 0), 10);
    }

    #[test]
    fn test_apply_damage() {
        assert_eq!(apply_damage(100, 30), 70);
        assert_eq!(apply_damage(30, 100), 0); // Can't go negative
        assert_eq!(apply_damage(50, 0), 50);
    }

    #[test]
    fn test_is_alive() {
        assert!(is_alive(1));
        assert!(is_alive(100));
        assert!(!is_alive(0));
    }

    #[test]
    fn test_roll_crit_always() {
        let mut rng = rand::thread_rng();
        // 100% crit chance should always crit
        for _ in 0..10 {
            assert!(roll_crit(100, &mut rng));
        }
    }

    #[test]
    fn test_roll_crit_never() {
        let mut rng = rand::thread_rng();
        // 0% crit chance should never crit
        for _ in 0..10 {
            assert!(!roll_crit(0, &mut rng));
        }
    }

    #[test]
    fn test_calculate_attack_simple() {
        let stats = make_test_stats();
        let mut rng = rand::thread_rng();

        let result = calculate_attack_simple(&stats, &mut rng);
        assert!(result.damage > 0);
        // Base stats have 10 total damage (5 phys + 5 magic)
        // Crit would make it 20
        assert!(result.damage >= 10);
    }

    #[test]
    fn test_damage_reflection() {
        assert_eq!(calculate_damage_reflection(100, 30.0), 30);
        assert_eq!(calculate_damage_reflection(100, 0.0), 0);
        assert_eq!(calculate_damage_reflection(0, 30.0), 0);
    }

    #[test]
    fn test_simulate_combat_round() {
        let stats = make_test_stats();
        let mut rng = rand::thread_rng();

        let (player_hp, enemy_hp, attack) = simulate_combat_round(&stats, 100, 50, 10, &mut rng);

        // Player dealt damage
        assert!(enemy_hp < 50 || attack.damage == 0);

        // Enemy attacked back (if alive)
        if enemy_hp > 0 {
            assert!(player_hp < 100);
        }
    }
}
