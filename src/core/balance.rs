//! Shared balance constants used by both game and simulator.
//!
//! All core balance numbers should be defined here.
//! Change once, test everywhere.

// Allow dead code - these constants are defined for future use and simulator integration
#![allow(dead_code)]

// =============================================================================
// DERIVED STATS - How attributes convert to combat stats
// =============================================================================

/// Base HP before Constitution bonus.
pub const BASE_HP: i32 = 50;

/// HP gained per point of Constitution modifier.
pub const HP_PER_CON_MOD: i32 = 10;

/// Base physical damage before Strength bonus.
pub const BASE_PHYSICAL_DAMAGE: i32 = 5;

/// Damage gained per point of Strength modifier.
pub const DAMAGE_PER_STR_MOD: i32 = 2;

/// Base magic damage before Intelligence bonus.
pub const BASE_MAGIC_DAMAGE: i32 = 5;

/// Damage gained per point of Intelligence modifier.
pub const DAMAGE_PER_INT_MOD: i32 = 2;

/// Base crit chance percent (before Dexterity).
pub const BASE_CRIT_CHANCE: i32 = 5;

/// Crit chance gained per point of Dexterity modifier.
pub const CRIT_CHANCE_PER_DEX_MOD: i32 = 1;

/// Defense gained per point of Dexterity modifier.
pub const DEFENSE_PER_DEX_MOD: i32 = 1;

/// Base crit damage multiplier (2.0 = double damage).
pub const BASE_CRIT_MULTIPLIER: f64 = 2.0;

/// XP multiplier bonus per point of Wisdom modifier.
pub const XP_MULT_PER_WIS_MOD: f64 = 0.05;

/// Prestige multiplier bonus per point of Charisma modifier.
pub const PRESTIGE_MULT_PER_CHA_MOD: f64 = 0.1;

// =============================================================================
// LEVELING & PROGRESSION
// =============================================================================

/// Base XP required for leveling.
pub const XP_CURVE_BASE: f64 = 100.0;

/// XP curve exponent (polynomial scaling).
/// XP for level N = XP_CURVE_BASE * N^XP_CURVE_EXPONENT
pub const XP_CURVE_EXPONENT: f64 = 1.5;

/// Kills required in a subzone before boss spawns.
pub const KILLS_PER_BOSS: u32 = 10;

// =============================================================================
// COMBAT XP REWARDS
// =============================================================================

/// Base XP per kill (multiplied by zone factors).
pub const COMBAT_XP_BASE: u32 = 10;

/// Additional XP per zone level.
pub const COMBAT_XP_PER_ZONE: u32 = 5;

/// Additional XP per subzone level.
pub const COMBAT_XP_PER_SUBZONE: u32 = 2;

/// Boss XP multiplier (boss gives 10x normal mob XP).
pub const BOSS_XP_MULTIPLIER: u32 = 10;

// =============================================================================
// ENEMY SCALING
// =============================================================================

/// Enemy HP as a fraction of player HP (base).
pub const ENEMY_HP_RATIO_MIN: f64 = 0.5;
pub const ENEMY_HP_RATIO_MAX: f64 = 0.8;

/// Enemy damage as a fraction of player damage.
pub const ENEMY_DAMAGE_RATIO_MIN: f64 = 0.3;
pub const ENEMY_DAMAGE_RATIO_MAX: f64 = 0.5;

/// Zone scaling: enemy stats multiply by (1 + (zone - 1) * ZONE_SCALING).
pub const ZONE_SCALING_PER_LEVEL: f64 = 0.1;

/// Elite enemy stat multiplier (vs normal enemies).
pub const ELITE_STAT_MULTIPLIER: f64 = 1.5;

/// Boss enemy stat multiplier (vs normal enemies).
pub const BOSS_STAT_MULTIPLIER: f64 = 2.0;

// =============================================================================
// ITEM / LOOT
// =============================================================================

/// Base item level per zone (ilvl = zone * ILVL_PER_ZONE).
pub const ILVL_PER_ZONE: u32 = 10;

/// Base item drop chance from normal mobs.
pub const ITEM_DROP_CHANCE_BASE: f64 = 0.15;

/// Legendary drop chance from zone bosses.
pub const BOSS_LEGENDARY_CHANCE: f64 = 0.05;

/// Legendary drop chance from zone 10 final boss.
pub const ZONE10_BOSS_LEGENDARY_CHANCE: f64 = 0.10;

// =============================================================================
// ATTRIBUTE POINT ALLOCATION
// =============================================================================

/// Points gained per level up.
pub const POINTS_PER_LEVEL: u32 = 3;

/// Base attribute cap (before prestige bonuses).
pub const BASE_ATTRIBUTE_CAP: u32 = 20;

/// Additional cap per prestige rank.
pub const ATTRIBUTE_CAP_PER_PRESTIGE: u32 = 5;

// =============================================================================
// Helpers
// =============================================================================

use crate::character::attributes::{AttributeType, Attributes};
use rand::Rng;

/// Calculate XP required for a given level.
pub fn xp_required_for_level(level: u32) -> u64 {
    (XP_CURVE_BASE * f64::powf(level as f64, XP_CURVE_EXPONENT)) as u64
}

/// Calculate attribute cap for a given prestige rank.
pub fn attribute_cap(prestige_rank: u32) -> u32 {
    BASE_ATTRIBUTE_CAP + (prestige_rank * ATTRIBUTE_CAP_PER_PRESTIGE)
}

/// Distribute level-up points randomly among attributes.
/// Returns which attributes were increased.
pub fn distribute_level_up_points(
    attrs: &mut Attributes,
    prestige_rank: u32,
    rng: &mut impl Rng,
) -> Vec<AttributeType> {
    let cap = attribute_cap(prestige_rank);
    let mut increased = Vec::new();

    let mut points = POINTS_PER_LEVEL;
    let mut attempts = 0;
    let max_attempts = 100;

    while points > 0 && attempts < max_attempts {
        let attr_index = rng.gen_range(0..6);
        let attr = AttributeType::all()[attr_index];

        if attrs.get(attr) < cap {
            attrs.increment(attr);
            increased.push(attr);
            points -= 1;
        }
        attempts += 1;
    }

    increased
}

/// Create attributes for a character at a given level (simulating level-ups from 1).
pub fn attributes_at_level(level: u32, prestige_rank: u32, rng: &mut impl Rng) -> Attributes {
    let mut attrs = Attributes::default();

    // Simulate leveling from 1 to target level
    for _ in 1..level {
        distribute_level_up_points(&mut attrs, prestige_rank, rng);
    }

    attrs
}

/// Calculate zone multiplier for enemy stats.
pub fn zone_stat_multiplier(zone_id: u32) -> f64 {
    1.0 + (zone_id.saturating_sub(1) as f64 * ZONE_SCALING_PER_LEVEL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_curve() {
        // XP for level 1 = 100 * 1^1.5 = 100
        assert_eq!(xp_required_for_level(1), 100);
        // XP scales with level
        assert!(xp_required_for_level(50) > xp_required_for_level(1));
        // Verify formula: 100 * 10^1.5 ≈ 3162
        assert!((xp_required_for_level(10) as f64 - 3162.0).abs() < 10.0);
    }

    #[test]
    fn test_zone_multiplier() {
        assert!((zone_stat_multiplier(1) - 1.0).abs() < 0.001);
        assert!((zone_stat_multiplier(10) - 1.9).abs() < 0.001);
    }
}
