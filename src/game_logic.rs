use crate::constants::*;
use crate::game_state::{GameState, Stat};
use chrono::Utc;

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

/// Report of offline progression results
#[derive(Debug, Default)]
pub struct OfflineReport {
    pub elapsed_seconds: i64,
    pub total_level_ups: u32,
    pub xp_gained: u64,
}

/// Calculates the XP gained during offline time
///
/// # Arguments
/// * `elapsed_seconds` - Time elapsed since last save
/// * `prestige_rank` - The player's current prestige rank
///
/// # Returns
/// The total XP gained during offline time (with 50% multiplier)
pub fn calculate_offline_xp(elapsed_seconds: i64, prestige_rank: u32) -> f64 {
    // Cap elapsed time at MAX_OFFLINE_SECONDS (7 days)
    let capped_seconds = elapsed_seconds.min(MAX_OFFLINE_SECONDS);

    // Calculate number of ticks that would have occurred
    let ticks = (capped_seconds as f64) * (1000.0 / TICK_INTERVAL_MS as f64);

    // Calculate XP per tick with prestige bonus
    let xp_per_tick = xp_gain_per_tick(prestige_rank);

    // Apply offline multiplier (50% of online rate)
    ticks * xp_per_tick * OFFLINE_MULTIPLIER
}

/// Processes offline progression and updates game state
///
/// # Arguments
/// * `state` - The game state to update
///
/// # Returns
/// An OfflineReport with details about the progression
pub fn process_offline_progression(state: &mut GameState) -> OfflineReport {
    let current_time = Utc::now().timestamp();
    let elapsed_seconds = current_time - state.last_save_time;

    // If no time has elapsed or time went backwards, return empty report
    if elapsed_seconds <= 0 {
        return OfflineReport::default();
    }

    // Calculate offline XP
    let offline_xp = calculate_offline_xp(elapsed_seconds, state.prestige_rank);

    // Apply XP to all stats and count level-ups
    let mut total_level_ups = 0;
    for stat in &mut state.stats {
        total_level_ups += apply_tick_xp(stat, offline_xp);
    }

    // Update last save time
    state.last_save_time = current_time;

    OfflineReport {
        elapsed_seconds,
        total_level_ups,
        xp_gained: offline_xp as u64,
    }
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

    #[test]
    fn test_calculate_offline_xp() {
        // 1 hour = 3600 seconds
        // Ticks per second = 1000 / 100 = 10
        // Total ticks = 3600 * 10 = 36000
        // XP per tick = 1.0 (rank 0)
        // Offline XP = 36000 * 1.0 * 0.5 = 18000
        let xp = calculate_offline_xp(3600, 0);
        assert_eq!(xp as u64, 18000);
    }

    #[test]
    fn test_calculate_offline_xp_with_prestige() {
        // 1 hour with prestige rank 1
        // Ticks = 36000
        // XP per tick = 1.0 * 1.5 = 1.5
        // Offline XP = 36000 * 1.5 * 0.5 = 27000
        let xp = calculate_offline_xp(3600, 1);
        assert_eq!(xp as u64, 27000);
    }

    #[test]
    fn test_calculate_offline_xp_capped() {
        // 10 days should be capped at 7 days
        let ten_days = 10 * 24 * 60 * 60;
        let seven_days = 7 * 24 * 60 * 60;

        let xp_10_days = calculate_offline_xp(ten_days, 0);
        let xp_7_days = calculate_offline_xp(seven_days, 0);

        // Both should be equal since 10 days is capped at 7
        assert_eq!(xp_10_days, xp_7_days);
    }

    #[test]
    fn test_process_offline_progression() {
        // Create state with last_save_time 1 hour ago
        let current_time = Utc::now().timestamp();
        let one_hour_ago = current_time - 3600;
        let mut state = GameState::new(one_hour_ago);

        // Process offline progression
        let report = process_offline_progression(&mut state);

        // Verify report
        assert_eq!(report.elapsed_seconds, 3600);
        assert_eq!(report.xp_gained, 18000);

        // Verify state was updated
        assert_eq!(state.last_save_time, current_time);

        // Each stat should have gained 18000 XP
        for stat in &state.stats {
            // At level 1, we need 100 XP to reach level 2
            // With 18000 XP, we should level up many times
            assert!(stat.level > 1);
        }
    }
}
