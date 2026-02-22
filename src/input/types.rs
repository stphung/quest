//! Type definitions for the input handling module.

use crate::core::game_logic::OfflineReport;
use crate::items;
use std::time::{SystemTime, UNIX_EPOCH};

/// Haven confirmation dialog state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HavenConfirmation {
    None,
    Build,
    Forge,
}

/// Haven overlay state, shared between CharacterSelect and Game screens.
pub struct HavenUiState {
    pub showing: bool,
    pub selected_room: usize,
    pub confirmation: HavenConfirmation,
    opened_at_ms: Option<u128>,
}

impl HavenUiState {
    fn current_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    pub fn new() -> Self {
        Self {
            showing: false,
            selected_room: 0,
            confirmation: HavenConfirmation::None,
            opened_at_ms: None,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_room = 0;
        self.confirmation = HavenConfirmation::None;
        self.opened_at_ms = Some(Self::current_millis());
    }

    pub fn close(&mut self) {
        self.showing = false;
        self.confirmation = HavenConfirmation::None;
        self.opened_at_ms = None;
    }

    pub fn open_elapsed_ms(&self) -> Option<u128> {
        self.opened_at_ms
            .map(|start| Self::current_millis().saturating_sub(start))
    }
}

/// Game-screen overlay state. At most one is active at a time.
pub enum GameOverlay {
    None,
    Help,
    HavenDiscovery,
    SoulforgeDiscovery,
    PrestigeConfirm,
    VaultSelection {
        selected_index: usize,
        selected_slots: Vec<items::EquipmentSlot>,
        confirm_pending: bool,
    },
    OfflineWelcome {
        report: OfflineReport,
    },
    Achievements {
        browser: crate::ui::achievement_browser_scene::AchievementBrowserState,
    },
    /// Achievement unlock celebration modal
    AchievementUnlocked {
        achievements: Vec<crate::achievements::AchievementId>,
    },
    /// Storm Leviathan encounter modal (fishing)
    LeviathanEncounter {
        encounter_number: u8,
    },
    /// Stormglass discovery celebration modal
    StormglassDiscovery,
    /// Quit confirmation when pending challenges exist
    QuitConfirm,
    /// Bug report overlay with game state summary
    BugReport {
        summary: String,
        clipboard_ready: bool,
        error: Option<String>,
    },
}

/// Result of handling a game input event.
pub enum InputResult {
    /// Continue the game loop normally.
    Continue,
    /// Player quit to character select. State should be saved first.
    QuitToSelect,
    /// State was modified (prestige, haven build) and should be saved.
    NeedsSave,
    /// Haven was modified along with state -- save both.
    NeedsSaveAll,
    /// Toggle the update details expanded state.
    ToggleUpdateDetails,
    /// Start a Chrono Surge with the given number of ticks.
    StartChronoSurge { ticks: u64 },
}
