//! Type definitions for the input handling module.

use crate::core::game_logic::OfflineReport;
use crate::items;

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
}

impl HavenUiState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_room: 0,
            confirmation: HavenConfirmation::None,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_room = 0;
        self.confirmation = HavenConfirmation::None;
    }

    pub fn close(&mut self) {
        self.showing = false;
        self.confirmation = HavenConfirmation::None;
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
    /// Quit confirmation when pending challenges exist
    QuitConfirm,
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
}
