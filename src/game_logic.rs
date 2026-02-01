use crate::constants::*;
use crate::game_state::Stat;

/// Calculates the XP required to reach the next level
///
/// # Arguments
/// * `level` - The current level
///
/// # Returns
/// The amount of XP needed to reach level + 1
pub fn xp_for_next_level(level: u32) -> u64 {
    (XP_CURVE_BASE * f64::powi(level as f64, XP_CURVE_EXPONENT as i32)) as u64
}

/// Calculates the XP gained per tick based on prestige rank
///
/// # Arguments
/// * `prestige_rank` - The player's current prestige rank
///
/// # Returns
/// The XP gain per tick multiplied by prestige bonus
pub fn xp_gain_per_tick(prestige_rank: u32) -> f64 {
    BASE_XP_PER_TICK * prestige_multiplier(prestige_rank)
}

/// Calculates the prestige multiplier for XP gains
///
/// # Arguments
/// * `rank` - The prestige rank
///
/// # Returns
/// The multiplier (1.0 for rank 0, 1.5^rank otherwise)
pub fn prestige_multiplier(rank: u32) -> f64 {
    if rank == 0 {
        1.0
    } else {
        1.5_f64.powi(rank as i32)
    }
}

/// Applies XP gain to a stat and processes any level-ups
///
/// # Arguments
/// * `stat` - The stat to add XP to
/// * `xp_gain` - The amount of XP to add
///
/// # Returns
/// The number of level-ups that occurred
pub fn apply_tick_xp(stat: &mut Stat, xp_gain: f64) -> u32 {
    stat.current_xp += xp_gain as u64;

    let mut levelups = 0;

    loop {
        let xp_needed = xp_for_next_level(stat.level);

        if stat.current_xp >= xp_needed {
            stat.current_xp -= xp_needed;
            stat.level += 1;
            levelups += 1;
        } else {
            break;
        }
    }

    levelups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_for_next_level() {
        // Level 1: 100 * 1^1.5 = 100
        assert_eq!(xp_for_next_level(1), 100);

        // Level 2: 100 * 2^1.5 = 100 * 2.828... ≈ 282
        assert_eq!(xp_for_next_level(2), 282);

        // Level 10: 100 * 10^1.5 = 100 * 31.622... ≈ 3162
        assert_eq!(xp_for_next_level(10), 3162);
    }

    #[test]
    fn test_prestige_multiplier() {
        // Rank 0: 1.0
        assert_eq!(prestige_multiplier(0), 1.0);

        // Rank 1: 1.5^1 = 1.5
        assert_eq!(prestige_multiplier(1), 1.5);

        // Rank 2: 1.5^2 = 2.25
        assert_eq!(prestige_multiplier(2), 2.25);

        // Rank 3: 1.5^3 = 3.375
        assert_eq!(prestige_multiplier(3), 3.375);
    }

    #[test]
    fn test_xp_gain_per_tick() {
        // Rank 0: 1.0 * 1.0 = 1.0
        assert_eq!(xp_gain_per_tick(0), 1.0);

        // Rank 1: 1.0 * 1.5 = 1.5
        assert_eq!(xp_gain_per_tick(1), 1.5);
    }

    #[test]
    fn test_apply_tick_xp_no_levelup() {
        let mut stat = Stat::new();

        // Add 50 XP, which is less than the 100 needed for level 2
        let levelups = apply_tick_xp(&mut stat, 50.0);

        assert_eq!(levelups, 0);
        assert_eq!(stat.level, 1);
        assert_eq!(stat.current_xp, 50);
    }

    #[test]
    fn test_apply_tick_xp_single_levelup() {
        let mut stat = Stat::new();

        // Add exactly 100 XP, which is enough for exactly 1 level up
        let levelups = apply_tick_xp(&mut stat, 100.0);

        assert_eq!(levelups, 1);
        assert_eq!(stat.level, 2);
        assert_eq!(stat.current_xp, 0);
    }

    #[test]
    fn test_apply_tick_xp_multiple_levelups() {
        let mut stat = Stat::new();

        // Add 400 XP, which should cause multiple level-ups
        // Level 1->2: needs 100, leaves 300
        // Level 2->3: needs 282, leaves 18
        let levelups = apply_tick_xp(&mut stat, 400.0);

        assert_eq!(levelups, 2);
        assert_eq!(stat.level, 3);
        assert_eq!(stat.current_xp, 18);
    }
}
