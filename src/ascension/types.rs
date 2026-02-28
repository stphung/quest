//! Ascension system constants and helper types.

/// PR cost lookup for Ascension levels 1-6.
const ASCENSION_COSTS: [u32; 6] = [10, 15, 25, 35, 50, 65];

/// Deep layer gate lookup for Ascension levels 1-6.
const ASCENSION_DEEP_GATES: [u32; 6] = [3, 7, 12, 18, 25, 30];

/// Prestige rank cost to Ascend to the given level.
pub fn ascension_cost(level: u32) -> u32 {
    if (1..=6).contains(&level) {
        ASCENSION_COSTS[(level - 1) as usize]
    } else if level > 6 {
        65 + 15 * (level - 6)
    } else {
        0
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
        assert_eq!(ascension_cost(1), 10);
        assert_eq!(ascension_cost(2), 15);
        assert_eq!(ascension_cost(3), 25);
        assert_eq!(ascension_cost(4), 35);
        assert_eq!(ascension_cost(5), 50);
        assert_eq!(ascension_cost(6), 65);
    }

    #[test]
    fn test_ascension_cost_level_7_plus() {
        assert_eq!(ascension_cost(7), 80); // 65 + 15*(7-6) = 80
        assert_eq!(ascension_cost(8), 95); // 65 + 15*(8-6) = 95
        assert_eq!(ascension_cost(10), 125); // 65 + 15*(10-6) = 125
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
        assert_eq!(total, 200);
    }
}
