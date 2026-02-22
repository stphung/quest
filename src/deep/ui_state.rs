//! UI state for The Deep overlay.

/// UI state for the Deep overlay, tracking active tab, selections, and multi-step flows.
#[derive(Debug, Clone, Default)]
pub struct DeepUiState {
    /// The currently visible tab.
    pub active_tab: DeepTab,
    /// Selected item index in the current tab's list.
    pub selected_index: usize,
    /// Whether the user is viewing the detail pane for the selected item.
    pub detail_mode: bool,
    /// In-progress mission launch flow state (Some when flow is active).
    pub mission_launch: Option<MissionLaunchState>,
    /// In-progress event resolve flow state (Some when resolving a check-in event).
    pub event_resolve: Option<EventResolveState>,
    /// Transient status message to display in the overlay footer.
    pub message: Option<String>,
    /// Countdown in ticks until `message` is cleared (decremented each tick).
    pub message_timer: u8,
}

impl DeepUiState {
    /// Reset selection and flow state when switching tabs.
    pub fn reset_for_tab(&mut self) {
        self.selected_index = 0;
        self.detail_mode = false;
        self.mission_launch = None;
        self.event_resolve = None;
    }

    /// Set a timed status message (visible for approximately 3 seconds at 10 ticks/sec).
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_timer = 30;
    }

    /// Tick down the message timer; clears the message when the timer expires.
    pub fn tick_message(&mut self) {
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message = None;
            }
        }
    }

    /// Returns true if the overlay is in a sub-flow (mission launch or event resolve).
    pub fn in_flow(&self) -> bool {
        self.mission_launch.is_some() || self.event_resolve.is_some()
    }
}

/// Which tab is active in the Deep overlay.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum DeepTab {
    /// Summary view: guild rank, marks, active mission progress.
    #[default]
    Dashboard,
    /// Mercenary roster: status, archetype, level.
    Roster,
    /// Mission management: active missions, available missions.
    Missions,
    /// Recruitment pool for hiring new mercenaries.
    Recruitment,
}

impl DeepTab {
    /// Short label for display in the tab bar.
    pub fn label(&self) -> &str {
        match self {
            DeepTab::Dashboard => "Dashboard",
            DeepTab::Roster => "Roster",
            DeepTab::Missions => "Missions",
            DeepTab::Recruitment => "Recruit",
        }
    }

    /// All tab variants in display order.
    pub fn all() -> &'static [DeepTab] {
        &[
            DeepTab::Dashboard,
            DeepTab::Roster,
            DeepTab::Missions,
            DeepTab::Recruitment,
        ]
    }

    /// Returns the next tab in the cycle.
    pub fn next(&self) -> DeepTab {
        match self {
            DeepTab::Dashboard => DeepTab::Roster,
            DeepTab::Roster => DeepTab::Missions,
            DeepTab::Missions => DeepTab::Recruitment,
            DeepTab::Recruitment => DeepTab::Dashboard,
        }
    }

    /// Returns the previous tab in the cycle.
    pub fn prev(&self) -> DeepTab {
        match self {
            DeepTab::Dashboard => DeepTab::Recruitment,
            DeepTab::Roster => DeepTab::Dashboard,
            DeepTab::Missions => DeepTab::Roster,
            DeepTab::Recruitment => DeepTab::Missions,
        }
    }
}

/// State for an in-progress mission launch flow.
#[derive(Debug, Clone)]
pub struct MissionLaunchState {
    /// Which step of the launch flow we are on.
    pub step: LaunchStep,
    /// Index into the available missions pool (generated when entering the flow).
    pub mission_index: usize,
    /// Mercenary IDs that have been selected for the squad.
    pub selected_mercs: Vec<u32>,
    /// Cursor position in the roster list during squad selection.
    pub merc_cursor: usize,
    /// Cached available missions pool (generated once when the flow starts).
    pub available_pool: Vec<crate::deep::types::ActiveMission>,
}

/// Steps within the mission launch flow.
#[derive(Debug, Clone, PartialEq)]
pub enum LaunchStep {
    /// Choose which available mission to send the squad on.
    SelectMission,
    /// Choose which ready mercenaries to include in the squad.
    SelectSquad,
    /// Review and confirm the mission launch.
    Confirm,
}

/// State for an in-progress event resolution flow.
#[derive(Debug, Clone)]
pub struct EventResolveState {
    /// Index into `state.run.active_missions` for the mission being resolved.
    pub mission_index: usize,
    /// Index into `mission.events` for the event being resolved.
    pub event_index: usize,
    /// Cursor position for selecting a resolution option (0=Safe, 1=Archetype, 2=Risky).
    pub selected_resolution: usize,
    /// Whether the archetype resolution option is available for the current event/squad.
    pub archetype_available: bool,
}
