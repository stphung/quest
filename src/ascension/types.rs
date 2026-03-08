//! Ascension system constants and helper types.

/// Maximum Ascension level currently available.
pub const MAX_ASCENSION_LEVEL: u32 = 10;

/// PR cost lookup for Ascension levels 1-6.
const ASCENSION_COSTS: [u32; 6] = [35, 65, 120, 200, 325, 500];

/// Deep layer gate lookup for Ascension levels 1-6.
const ASCENSION_DEEP_GATES: [u32; 6] = [3, 7, 12, 18, 25, 30];

/// Loom-gated Ascension costs for levels 7-10.
const LOOM_ASCENSION_COSTS: [u32; 4] = [1500, 4000, 8000, 15000];

/// Loom pattern gates for Ascension levels 7-10.
const LOOM_ASCENSION_PATTERN_GATES: [usize; 4] = [8, 16, 22, 28];

/// Max shuttle level per Ascension tier (7-10).
const LOOM_SHUTTLE_LEVEL_CAPS: [u32; 4] = [3, 5, 7, 10];

/// Prestige rank cost to Ascend to the given level.
pub fn ascension_cost(level: u32) -> u32 {
    if (1..=6).contains(&level) {
        ASCENSION_COSTS[(level - 1) as usize]
    } else if (7..=10).contains(&level) {
        LOOM_ASCENSION_COSTS[(level - 7) as usize]
    } else {
        0
    }
}

/// Loom pattern gate for the given Ascension level. None means no pattern gate.
pub fn ascension_pattern_gate(level: u32) -> Option<usize> {
    if (7..=10).contains(&level) {
        Some(LOOM_ASCENSION_PATTERN_GATES[(level - 7) as usize])
    } else {
        None
    }
}

/// Maximum shuttle level allowed at the given Ascension level.
pub fn max_shuttle_level(ascension_level: u32) -> u32 {
    if (7..=10).contains(&ascension_level) {
        LOOM_SHUTTLE_LEVEL_CAPS[(ascension_level - 7) as usize]
    } else {
        1
    }
}

/// Deep layer gate for the given Ascension level. None means no Deep gate (PR only).
pub fn ascension_deep_gate(level: u32) -> Option<u32> {
    if (1..=6).contains(&level) {
        Some(ASCENSION_DEEP_GATES[(level - 1) as usize])
    } else {
        None
    }
}

/// Combat multiplier at a given Ascension level.
/// Level 0 = 1.0x, Levels 1-6 = 2^level, Levels 7+ = 64 * 1.5^(level-6).
pub fn ascension_combat_multiplier(level: u32) -> f64 {
    if level == 0 {
        1.0
    } else if level <= 6 {
        2.0_f64.powi(level as i32)
    } else {
        64.0 * 1.5_f64.powi((level - 6) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascension_cost_levels_1_through_6() {
        assert_eq!(ascension_cost(1), 35);
        assert_eq!(ascension_cost(2), 65);
        assert_eq!(ascension_cost(3), 120);
        assert_eq!(ascension_cost(4), 200);
        assert_eq!(ascension_cost(5), 325);
        assert_eq!(ascension_cost(6), 500);
    }

    #[test]
    fn test_ascension_cost_levels_7_through_10_loom() {
        assert_eq!(ascension_cost(7), 1500);
        assert_eq!(ascension_cost(8), 4000);
        assert_eq!(ascension_cost(9), 8000);
        assert_eq!(ascension_cost(10), 15000);
    }

    #[test]
    fn test_ascension_cost_level_11_plus_returns_zero() {
        assert_eq!(ascension_cost(11), 0);
        assert_eq!(ascension_cost(100), 0);
    }

    #[test]
    fn test_ascension_pattern_gate() {
        assert_eq!(ascension_pattern_gate(1), None);
        assert_eq!(ascension_pattern_gate(6), None);
        assert_eq!(ascension_pattern_gate(7), Some(8));
        assert_eq!(ascension_pattern_gate(8), Some(16));
        assert_eq!(ascension_pattern_gate(9), Some(22));
        assert_eq!(ascension_pattern_gate(10), Some(28));
    }

    #[test]
    fn test_max_shuttle_level_for_ascension() {
        assert_eq!(max_shuttle_level(0), 1);
        assert_eq!(max_shuttle_level(6), 1);
        assert_eq!(max_shuttle_level(7), 3);
        assert_eq!(max_shuttle_level(8), 5);
        assert_eq!(max_shuttle_level(9), 7);
        assert_eq!(max_shuttle_level(10), 10);
    }

    #[test]
    fn test_ascension_combat_multiplier_levels_7_through_10() {
        assert!((ascension_combat_multiplier(7) - 96.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(8) - 144.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(9) - 216.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(10) - 324.0).abs() < 1e-10);
    }

    #[test]
    fn test_ascension_deep_gate_levels_1_through_6() {
        assert_eq!(ascension_deep_gate(1), Some(3));
        assert_eq!(ascension_deep_gate(2), Some(7));
        assert_eq!(ascension_deep_gate(3), Some(12));
        assert_eq!(ascension_deep_gate(4), Some(18));
        assert_eq!(ascension_deep_gate(5), Some(25));
        assert_eq!(ascension_deep_gate(6), Some(30));
    }

    #[test]
    fn test_ascension_deep_gate_level_7_plus_none() {
        assert_eq!(ascension_deep_gate(7), None);
        assert_eq!(ascension_deep_gate(100), None);
    }

    #[test]
    fn test_ascension_combat_multiplier() {
        assert!((ascension_combat_multiplier(0) - 1.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(1) - 2.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(2) - 4.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(3) - 8.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(4) - 16.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(5) - 32.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(6) - 64.0).abs() < 1e-10);
    }

    #[test]
    fn test_ascension_combat_multiplier_level_7_plus() {
        assert!((ascension_combat_multiplier(7) - 96.0).abs() < 1e-10); // 64 * 1.5
        assert!((ascension_combat_multiplier(8) - 144.0).abs() < 1e-10); // 64 * 1.5^2
    }

    #[test]
    fn test_total_pr_for_levels_1_through_6() {
        let total: u32 = (1..=6).map(ascension_cost).sum();
        assert_eq!(total, 1245);
    }
}
