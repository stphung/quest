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

pub const ENHANCEMENT_CUMULATIVE_BONUS: [f64; 11] = [
    0.0, 5.0, 10.0, 15.0, 20.0, 30.0, 40.0, 55.0, 75.0, 100.0, 150.0,
];

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

/// Returns the RGB color for an enhancement level.
/// 0=none(128,128,128), 1-4=white(255,255,255), 5-7=yellow(255,255,0),
/// 8-9=magenta(255,0,255), 10=gold(255,215,0)
pub fn enhancement_color_rgb(level: u8) -> (u8, u8, u8) {
    match enhancement_color_tier(level) {
        1 => (255, 255, 255), // White
        2 => (255, 255, 0),   // Yellow
        3 => (255, 0, 255),   // Magenta
        4 => (255, 215, 0),   // Gold
        _ => (128, 128, 128), // DarkGray
    }
}

// --- Blacksmith UI state types ---
// These live here (not in input.rs) so the UI module can access them from both
// the binary and library crates.

/// Blacksmith enhancement phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlacksmithPhase {
    Menu,
    Confirming,
    Hammering,
    ResultSuccess,
    ResultFailure,
}

/// Result of an enhancement attempt (for display)
pub struct EnhancementResult {
    pub slot_index: usize,
    pub success: bool,
    pub old_level: u8,
    pub new_level: u8,
}

/// Blacksmith overlay state
pub struct BlacksmithUiState {
    pub open: bool,
    pub selected_slot: usize,
    pub phase: BlacksmithPhase,
    pub animation_tick: u8,
    pub last_result: Option<EnhancementResult>,
}

impl Default for BlacksmithUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl BlacksmithUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_slot: 0,
            phase: BlacksmithPhase::Menu,
            animation_tick: 0,
            last_result: None,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.selected_slot = 0;
        self.phase = BlacksmithPhase::Menu;
        self.animation_tick = 0;
        self.last_result = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.phase = BlacksmithPhase::Menu;
        self.last_result = None;
    }
}
