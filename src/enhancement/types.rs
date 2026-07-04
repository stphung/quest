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
pub use crate::core::constants::{
    SOULFORGE_DISCOVERY_BASE_CHANCE, SOULFORGE_DISCOVERY_RANK_BONUS, SOULFORGE_MIN_PRESTIGE_RANK,
};

pub const ENHANCEMENT_SUCCESS_RATES: [f64; 10] = [
    1.00, 1.00, 1.00, 1.00, // +1-4: 100%
    0.70, 0.55, 0.40, // +5-7: 70%, 55%, 40%
    0.30, 0.20, 0.10, // +8-10: 30%, 20%, 10%
];

pub const ENHANCEMENT_COSTS: [u32; 10] = [
    1, 1, 1, 1, // +1-4: 1 PR each
    2, 3, 3, // +5: 2 PR, +6-7: 3 PR each
    4, 4, // +8-9: 4 PR each
    5, // +10: 5 PR
];

// Soul Tithe prices for +8-+10 are ~0.75x the expected PR cost of gambling
// that step (attempt fees plus re-buying levels lost to failures via the
// tithes below), the same discount ratio the +5-+7 prices imply.
pub const ENHANCEMENT_SOUL_TITHE_COSTS: [Option<u32>; 10] = [
    None,
    None,
    None,
    None, // +1-4: no soul tithe (already 100%)
    Some(4),
    Some(6),
    Some(8), // +5-7: soul tithe for guaranteed 100%
    Some(25),
    Some(85),
    Some(750), // +8-10: guaranteed, priced against the failure cascade
];

pub const ENHANCEMENT_FAIL_PENALTY: [u8; 10] = [
    0, 0, 0, 0, // +1-4: safe
    1, 1, 1, // +5-7: -1
    1, 1, // +8-9: -1
    2, // +10: -2
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

pub fn soul_tithe_cost(target_level: u8) -> Option<u32> {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return None;
    }
    ENHANCEMENT_SOUL_TITHE_COSTS[(target_level - 1) as usize]
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

// --- Soulforge UI state types ---
// These live here (not in input.rs) so the UI module can access them from both
// the binary and library crates.

/// Soulforge enhancement phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoulforgePhase {
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
    pub cost: u32,
}

/// Soulforge overlay state
pub struct SoulforgeUiState {
    pub open: bool,
    pub selected_slot: usize,
    pub phase: SoulforgePhase,
    pub animation_tick: u8,
    pub last_result: Option<EnhancementResult>,
    pub soul_tithe: bool,
}

impl Default for SoulforgeUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl SoulforgeUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_slot: 0,
            phase: SoulforgePhase::Menu,
            animation_tick: 0,
            last_result: None,
            soul_tithe: false,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.selected_slot = 0;
        self.phase = SoulforgePhase::Menu;
        self.animation_tick = 0;
        self.last_result = None;
        self.soul_tithe = false;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.phase = SoulforgePhase::Menu;
        self.last_result = None;
        self.soul_tithe = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhancement_progress_clamps_levels_and_tracks_highest() {
        let mut progress = EnhancementProgress::new();

        progress.set_level(2, 7);
        progress.set_level(4, MAX_ENHANCEMENT_LEVEL + 5);
        progress.set_level(99, 3);

        assert_eq!(progress.level(2), 7);
        assert_eq!(progress.level(4), MAX_ENHANCEMENT_LEVEL);
        assert_eq!(progress.level(99), 0);
        assert_eq!(progress.highest_level_reached, MAX_ENHANCEMENT_LEVEL);
    }

    #[test]
    fn lookup_helpers_return_zero_or_none_out_of_range() {
        assert_eq!(success_rate(0), 0.0);
        assert_eq!(success_rate(11), 0.0);
        assert_eq!(success_rate(5), 0.70);

        assert_eq!(enhancement_cost(0), 0);
        assert_eq!(enhancement_cost(11), 0);
        assert_eq!(enhancement_cost(10), 5);

        assert_eq!(soul_tithe_cost(0), None);
        assert_eq!(soul_tithe_cost(8), Some(25));
        assert_eq!(soul_tithe_cost(6), Some(6));

        assert_eq!(fail_penalty(0), 0);
        assert_eq!(fail_penalty(11), 0);
        assert_eq!(fail_penalty(10), 2);
    }

    #[test]
    fn display_helpers_cover_all_color_tiers() {
        assert_eq!(enhancement_multiplier(0), 1.0);
        assert_eq!(enhancement_multiplier(MAX_ENHANCEMENT_LEVEL), 2.5);

        assert_eq!(enhancement_prefix(0), "");
        assert_eq!(enhancement_prefix(8), "+8 ");

        assert_eq!(enhancement_color_tier(0), 0);
        assert_eq!(enhancement_color_tier(4), 1);
        assert_eq!(enhancement_color_tier(7), 2);
        assert_eq!(enhancement_color_tier(9), 3);
        assert_eq!(enhancement_color_tier(10), 4);
        assert_eq!(enhancement_color_rgb(10), (255, 215, 0));
    }

    #[test]
    fn enhancement_color_rgb_covers_every_tier() {
        // Each tier maps to a distinct RGB match arm in enhancement_color_rgb;
        // the test above only exercises tier 4 (Gold). Cover the rest here.
        assert_eq!(enhancement_color_rgb(0), (128, 128, 128)); // tier 0: DarkGray
        assert_eq!(enhancement_color_rgb(2), (255, 255, 255)); // tier 1: White
        assert_eq!(enhancement_color_rgb(6), (255, 255, 0)); // tier 2: Yellow
        assert_eq!(enhancement_color_rgb(9), (255, 0, 255)); // tier 3: Magenta
    }

    #[test]
    fn enhancement_progress_default_matches_new() {
        let default_progress = EnhancementProgress::default();
        let new_progress = EnhancementProgress::new();
        assert_eq!(default_progress.discovered, new_progress.discovered);
        assert_eq!(default_progress.levels, new_progress.levels);
        assert_eq!(default_progress.highest_level_reached, 0);
    }

    #[test]
    fn soulforge_ui_state_default_matches_new() {
        let default_ui = SoulforgeUiState::default();
        assert!(!default_ui.open);
        assert_eq!(default_ui.selected_slot, 0);
        assert_eq!(default_ui.phase, SoulforgePhase::Menu);
        assert_eq!(default_ui.animation_tick, 0);
        assert!(default_ui.last_result.is_none());
        assert!(!default_ui.soul_tithe);
    }

    #[test]
    fn soulforge_ui_open_and_close_reset_transient_state() {
        let mut ui = SoulforgeUiState::new();
        ui.selected_slot = 4;
        ui.phase = SoulforgePhase::ResultFailure;
        ui.animation_tick = 9;
        ui.last_result = Some(EnhancementResult {
            slot_index: 4,
            success: false,
            old_level: 8,
            new_level: 6,
            cost: 5,
        });
        ui.soul_tithe = true;

        ui.open();
        assert!(ui.open);
        assert_eq!(ui.selected_slot, 0);
        assert_eq!(ui.phase, SoulforgePhase::Menu);
        assert_eq!(ui.animation_tick, 0);
        assert!(ui.last_result.is_none());
        assert!(!ui.soul_tithe);

        ui.selected_slot = 3;
        ui.phase = SoulforgePhase::Hammering;
        ui.last_result = Some(EnhancementResult {
            slot_index: 3,
            success: true,
            old_level: 4,
            new_level: 5,
            cost: 2,
        });
        ui.soul_tithe = true;

        ui.close();
        assert!(!ui.open);
        assert_eq!(ui.selected_slot, 3);
        assert_eq!(ui.phase, SoulforgePhase::Menu);
        assert!(ui.last_result.is_none());
        assert!(!ui.soul_tithe);
    }
}
