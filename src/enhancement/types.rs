use serde::{Deserialize, Serialize};

/// Account-wide enhancement progress, persisted to ~/.quest/enhancement.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementProgress {
    pub discovered: bool,
    pub levels: [u8; 7], // Per-slot, 0-10, indexed by EquipmentSlot order
    pub total_attempts: u32,
    pub total_successes: u32,
    pub total_failures: u32,
    pub highest_level_reached: u8,
}

impl Default for EnhancementProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancementProgress {
    pub fn new() -> Self {
        Self {
            discovered: false,
            levels: [0; 7],
            total_attempts: 0,
            total_successes: 0,
            total_failures: 0,
            highest_level_reached: 0,
        }
    }

    /// Get enhancement level for a slot (by index 0-6)
    pub fn level(&self, slot_index: usize) -> u8 {
        self.levels.get(slot_index).copied().unwrap_or(0)
    }

    /// Set enhancement level for a slot
    pub fn set_level(&mut self, slot_index: usize, level: u8) {
        if let Some(l) = self.levels.get_mut(slot_index) {
            *l = level.min(MAX_ENHANCEMENT_LEVEL);
        }
        self.highest_level_reached = self
            .highest_level_reached
            .max(level.min(MAX_ENHANCEMENT_LEVEL));
    }
}

pub const MAX_ENHANCEMENT_LEVEL: u8 = 10;
pub const BLACKSMITH_MIN_PRESTIGE_RANK: u32 = 15;
pub const BLACKSMITH_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
pub const BLACKSMITH_DISCOVERY_RANK_BONUS: f64 = 0.000007;

pub const ENHANCEMENT_SUCCESS_RATES: [f64; 10] = [
    1.00, 1.00, 1.00, 1.00, // +1-4: 100%
    0.70, 0.60, 0.50, // +5-7: 70%, 60%, 50%
    0.30, 0.15, 0.05, // +8-10: 30%, 15%, 5%
];

pub const ENHANCEMENT_COSTS: [u32; 10] = [
    1, 1, 1, 1, // +1-4: 1 PR each
    3, 3, 3, // +5-7: 3 PR each
    5, 5,  // +8-9: 5 PR each
    10, // +10: 10 PR
];

pub const ENHANCEMENT_FAIL_PENALTY: [u8; 10] = [
    0, 0, 0, 0, // +1-4: safe
    1, 1, 1, // +5-7: -1
    2, 2, 2, // +8-10: -2
];

pub const ENHANCEMENT_CUMULATIVE_BONUS: [f64; 11] =
    [0.0, 1.0, 2.0, 4.0, 6.0, 9.0, 13.0, 18.0, 25.0, 35.0, 50.0];

pub fn success_rate(target_level: u8) -> f64 {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return 0.0;
    }
    ENHANCEMENT_SUCCESS_RATES[(target_level - 1) as usize]
}

pub fn enhancement_cost(target_level: u8) -> u32 {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return 0;
    }
    ENHANCEMENT_COSTS[(target_level - 1) as usize]
}

pub fn fail_penalty(target_level: u8) -> u8 {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return 0;
    }
    ENHANCEMENT_FAIL_PENALTY[(target_level - 1) as usize]
}

pub fn enhancement_multiplier(level: u8) -> f64 {
    let idx = (level as usize).min(MAX_ENHANCEMENT_LEVEL as usize);
    1.0 + ENHANCEMENT_CUMULATIVE_BONUS[idx] / 100.0
}

/// Format an enhancement prefix for display (e.g., "+5 " or "" for +0)
pub fn enhancement_prefix(level: u8) -> String {
    if level == 0 {
        String::new()
    } else {
        format!("+{} ", level)
    }
}

/// Color tier: 0=none, 1=white(+1-4), 2=yellow(+5-7), 3=magenta(+8-9), 4=gold(+10)
pub fn enhancement_color_tier(level: u8) -> u8 {
    match level {
        0 => 0,
        1..=4 => 1,
        5..=7 => 2,
        8..=9 => 3,
        _ => 4,
    }
}
