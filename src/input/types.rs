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
        title_browser: crate::ui::title_browser_scene::TitleBrowserState,
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
    /// Time Vault (save history browser)
    TimeVault {
        browser: Box<crate::ui::time_vault_scene::TimeVaultState>,
    },
}

/// Result of handling a game input event.
pub enum InputResult {
    /// Continue the game loop normally.
    Continue,
    /// Player quit to character select. State should be saved first.
    QuitToSelect,
    /// State was modified (haven build, etc.) and should be saved.
    NeedsSave,
    /// State was modified and should be saved with a git history commit.
    NeedsSaveWithEvent(crate::history::SaveEvent),
    /// Haven was modified along with state -- save both.
    NeedsSaveAll,
    /// Haven was modified and should be saved with a git history commit.
    NeedsSaveAllWithEvent(crate::history::SaveEvent),
    /// Toggle the update details expanded state.
    ToggleUpdateDetails,
    /// Start a Chrono Surge with the given number of ticks.
    StartChronoSurge { ticks: u64 },
    /// Open the Time Vault (main.rs populates state from HistoryRepo).
    OpenTimeVault,
    /// Restore game state to a specific commit in save history.
    RestoreSave { commit_id: String },
    /// Reload commits for a different branch in the save history browser.
    RefreshSaveHistoryCommits { branch_name: String },
    /// Fork a new save branch at a specific commit.
    ForkSave {
        commit_id: String,
        branch_name: String,
    },
    /// Switch the active save branch.
    SwitchSaveBranch { branch_name: String },
    /// Delete a save branch.
    DeleteSaveBranch { branch_name: String },
    /// Validate a PAT and fetch repos for selection.
    ValidateToken { token: String },
    /// Change the linked cloud repo (re-use existing PAT from config).
    ChangeRepo,
    /// Link a GitHub account for cloud sync.
    LinkCloud {
        token: String,
        repo_name: String,
        private: bool,
    },
    /// Push all save branches to the cloud.
    PushCloud,
    /// Pull save branches from the cloud.
    PullCloud,
    /// Unlink the GitHub cloud account.
    UnlinkCloud,
    /// Resolve divergence: keep local saves, force-push to cloud.
    ResolveKeepLocal,
    /// Resolve divergence: use cloud saves, discard local.
    ResolveUseCloud,
    /// Resolve divergence: keep both (backup local, reset to cloud).
    ResolveKeepBoth,
    /// Update the stored PAT with a new token.
    UpdateToken { token: String },
}
