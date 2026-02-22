//! Input handling for The Deep overlay.
//!
//! Dispatches keyboard events to the appropriate handler based on the current
//! UI state: tab navigation, list scrolling, mission launch flow, and event
//! resolution flow.

// The functions in this module are not yet wired into the main game loop
// (pending task: wire Deep overlay into main loop). Suppress dead-code warnings
// until the integration is complete.
#![allow(dead_code)]

use crate::deep::events::{can_use_archetype_resolution, resolve_event};
use crate::deep::guild::upgrade_guild;
use crate::deep::mercenaries::{generate_recruitment_pool, recruit_mercenary};
use crate::deep::missions::{available_missions, start_mission};
use crate::deep::types::{EventResolution, MercStatus, TheDeepState};
use crate::deep::ui_state::{
    DeepTab, DeepUiState, EventResolveState, LaunchStep, MissionLaunchState,
};
use ratatui::crossterm::event::KeyCode;

/// Handle input while The Deep overlay is open.
///
/// Returns `true` if the input was consumed (overlay stays open), or `false`
/// if Esc was pressed at the top level and the overlay should close.
pub fn handle_deep_input(
    key: KeyCode,
    deep: &mut TheDeepState,
    ui: &mut DeepUiState,
    now_unix: i64,
    rng: &mut impl rand::Rng,
) -> bool {
    // Tick down any active status message.
    ui.tick_message();

    // --- Event resolve flow takes priority ---
    if ui.event_resolve.is_some() {
        return handle_event_resolve(key, deep, ui);
    }

    // --- Mission launch flow ---
    if ui.mission_launch.is_some() {
        return handle_mission_launch(key, deep, ui, now_unix, rng);
    }

    // --- Normal tab/list navigation ---
    match key {
        KeyCode::Left | KeyCode::BackTab => {
            ui.active_tab = ui.active_tab.prev();
            ui.reset_for_tab();
            true
        }
        KeyCode::Right | KeyCode::Tab => {
            ui.active_tab = ui.active_tab.next();
            ui.reset_for_tab();
            true
        }
        KeyCode::Up => {
            if ui.selected_index > 0 {
                ui.selected_index -= 1;
            }
            true
        }
        KeyCode::Down => {
            let max = list_len(deep, &ui.active_tab).saturating_sub(1);
            if ui.selected_index < max {
                ui.selected_index += 1;
            }
            true
        }
        KeyCode::Enter => {
            handle_enter(key, deep, ui, now_unix, rng);
            true
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            collect_completed_missions(deep, ui);
            true
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            attempt_guild_upgrade(deep, ui);
            true
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            refresh_recruitment_pool(deep, ui, rng);
            true
        }
        KeyCode::Esc => {
            // Close the overlay
            false
        }
        _ => true,
    }
}

// =========================================================================
// Enter key dispatch
// =========================================================================

fn handle_enter(
    _key: KeyCode,
    deep: &mut TheDeepState,
    ui: &mut DeepUiState,
    now_unix: i64,
    rng: &mut impl rand::Rng,
) {
    match &ui.active_tab.clone() {
        DeepTab::Dashboard => {
            // No action on Dashboard
        }
        DeepTab::Roster => {
            // Toggle detail mode for the selected merc
            if ui.selected_index < deep.run.mercenaries.len() {
                ui.detail_mode = !ui.detail_mode;
            }
        }
        DeepTab::Missions => {
            handle_missions_enter(deep, ui, now_unix, rng);
        }
        DeepTab::Recruitment => {
            handle_recruitment_enter(deep, ui);
        }
    }
}

fn handle_missions_enter(
    deep: &mut TheDeepState,
    ui: &mut DeepUiState,
    now_unix: i64,
    rng: &mut impl rand::Rng,
) {
    let active_count = deep.run.active_missions.len();

    if ui.selected_index < active_count {
        // Selected an active mission: check for pending events
        let mission_idx = ui.selected_index;
        let pending = crate::deep::clock::pending_events(&deep.run.active_missions[mission_idx], now_unix);
        if let Some(&event_idx) = pending.first() {
            let mission = &deep.run.active_missions[mission_idx];
            let all_mercs: Vec<&crate::deep::types::Mercenary> =
                deep.run.mercenaries.iter().collect();
            let archetype_available =
                can_use_archetype_resolution(mission, event_idx, &all_mercs);

            ui.event_resolve = Some(EventResolveState {
                mission_index: mission_idx,
                event_index: event_idx,
                selected_resolution: 0,
                archetype_available,
            });
        }
    } else {
        // Selected from the available missions area: start launch flow
        let available_pool = available_missions(deep, now_unix, rng);
        let pool_index = ui.selected_index.saturating_sub(active_count);
        if pool_index < available_pool.len() {
            ui.mission_launch = Some(MissionLaunchState {
                step: LaunchStep::SelectMission,
                mission_index: pool_index,
                selected_mercs: Vec::new(),
                merc_cursor: 0,
                available_pool,
            });
        }
    }
}

fn handle_recruitment_enter(deep: &mut TheDeepState, ui: &mut DeepUiState) {
    if ui.selected_index < deep.run.recruitment_pool.len() {
        match recruit_mercenary(deep, ui.selected_index) {
            Ok(()) => {
                ui.set_message("Mercenary recruited!");
                // Clamp selected_index after removal
                let pool_len = deep.run.recruitment_pool.len();
                if ui.selected_index >= pool_len && pool_len > 0 {
                    ui.selected_index = pool_len - 1;
                }
            }
            Err(e) => {
                ui.set_message(format!("Cannot recruit: {}", e));
            }
        }
    }
}

// =========================================================================
// Mission launch flow
// =========================================================================

fn handle_mission_launch(
    key: KeyCode,
    deep: &mut TheDeepState,
    ui: &mut DeepUiState,
    _now_unix: i64,
    _rng: &mut impl rand::Rng,
) -> bool {
    let launch = match ui.mission_launch.as_mut() {
        Some(l) => l,
        None => return true,
    };

    match launch.step.clone() {
        LaunchStep::SelectMission => match key {
            KeyCode::Up => {
                if launch.mission_index > 0 {
                    launch.mission_index -= 1;
                }
                true
            }
            KeyCode::Down => {
                let max = launch.available_pool.len().saturating_sub(1);
                if launch.mission_index < max {
                    launch.mission_index += 1;
                }
                true
            }
            KeyCode::Enter => {
                if launch.mission_index < launch.available_pool.len() {
                    launch.step = LaunchStep::SelectSquad;
                    launch.merc_cursor = 0;
                    launch.selected_mercs.clear();
                }
                true
            }
            KeyCode::Esc => {
                ui.mission_launch = None;
                true
            }
            _ => true,
        },

        LaunchStep::SelectSquad => match key {
            KeyCode::Up => {
                if let Some(l) = ui.mission_launch.as_mut() {
                    if l.merc_cursor > 0 {
                        l.merc_cursor -= 1;
                    }
                }
                true
            }
            KeyCode::Down => {
                let ready_count = deep
                    .run
                    .mercenaries
                    .iter()
                    .filter(|m| m.status == MercStatus::Ready)
                    .count();
                if let Some(l) = ui.mission_launch.as_mut() {
                    if l.merc_cursor + 1 < ready_count {
                        l.merc_cursor += 1;
                    }
                }
                true
            }
            KeyCode::Char(' ') => {
                // Toggle selection of the merc under the cursor
                let ready_mercs: Vec<u32> = deep
                    .run
                    .mercenaries
                    .iter()
                    .filter(|m| m.status == MercStatus::Ready)
                    .map(|m| m.id)
                    .collect();

                if let Some(l) = ui.mission_launch.as_mut() {
                    if let Some(&merc_id) = ready_mercs.get(l.merc_cursor) {
                        if l.selected_mercs.contains(&merc_id) {
                            l.selected_mercs.retain(|&id| id != merc_id);
                        } else {
                            l.selected_mercs.push(merc_id);
                        }
                    }
                }
                true
            }
            KeyCode::Enter => {
                if let Some(l) = ui.mission_launch.as_mut() {
                    if !l.selected_mercs.is_empty() {
                        l.step = LaunchStep::Confirm;
                    } else {
                        // Need at least one merc to proceed
                    }
                }
                true
            }
            KeyCode::Esc => {
                if let Some(l) = ui.mission_launch.as_mut() {
                    l.step = LaunchStep::SelectMission;
                    l.selected_mercs.clear();
                }
                true
            }
            _ => true,
        },

        LaunchStep::Confirm => match key {
            KeyCode::Enter => {
                // Extract the data we need before borrowing deep mutably
                let (mission_template, squad_ids) = if let Some(l) = &ui.mission_launch {
                    (
                        l.available_pool
                            .get(l.mission_index)
                            .cloned(),
                        l.selected_mercs.clone(),
                    )
                } else {
                    (None, Vec::new())
                };

                if let Some(mission) = mission_template {
                    // Apply Saboteur duration reduction if one is in the squad
                    let has_saboteur = deep.run.mercenaries.iter().any(|m| {
                        squad_ids.contains(&m.id)
                            && m.archetype == crate::deep::types::MercArchetype::Saboteur
                            && m.status == MercStatus::Ready
                    });

                    let mut mission = mission;
                    if has_saboteur {
                        mission.duration_secs = mission.duration_secs * 85 / 100;
                    }

                    match start_mission(deep, mission, squad_ids) {
                        Ok(()) => {
                            ui.set_message("Mission launched!");
                            ui.mission_launch = None;
                        }
                        Err(e) => {
                            ui.set_message(format!("Cannot launch: {}", e));
                            ui.mission_launch = None;
                        }
                    }
                } else {
                    ui.mission_launch = None;
                }
                true
            }
            KeyCode::Esc => {
                ui.mission_launch = None;
                true
            }
            _ => true,
        },
    }
}

// =========================================================================
// Event resolve flow
// =========================================================================

fn handle_event_resolve(
    key: KeyCode,
    deep: &mut TheDeepState,
    ui: &mut DeepUiState,
) -> bool {
    let (mission_index, event_index, archetype_available) = match &ui.event_resolve {
        Some(e) => (e.mission_index, e.event_index, e.archetype_available),
        None => return true,
    };

    // Resolution options: 0 = Safe, 1 = Archetype (if available), 2 = Risky
    // When archetype is not available, option 1 is skipped: 0=Safe, 1=Risky.
    let option_count = if archetype_available { 3 } else { 2 };

    match key {
        KeyCode::Up => {
            if let Some(e) = ui.event_resolve.as_mut() {
                if e.selected_resolution > 0 {
                    e.selected_resolution -= 1;
                }
            }
            true
        }
        KeyCode::Down => {
            if let Some(e) = ui.event_resolve.as_mut() {
                if e.selected_resolution + 1 < option_count {
                    e.selected_resolution += 1;
                }
            }
            true
        }
        KeyCode::Enter => {
            let selected = ui
                .event_resolve
                .as_ref()
                .map(|e| e.selected_resolution)
                .unwrap_or(0);

            let resolution = if archetype_available {
                match selected {
                    0 => EventResolution::Safe,
                    1 => EventResolution::Archetype,
                    _ => EventResolution::Risky,
                }
            } else {
                match selected {
                    0 => EventResolution::Safe,
                    _ => EventResolution::Risky,
                }
            };

            if let Some(mission) = deep.run.active_missions.get_mut(mission_index) {
                match resolve_event(mission, event_index, resolution) {
                    Ok(()) => {
                        ui.set_message("Event resolved!");
                    }
                    Err(e) => {
                        ui.set_message(format!("Error: {}", e));
                    }
                }
            }
            ui.event_resolve = None;
            true
        }
        KeyCode::Esc => {
            ui.event_resolve = None;
            true
        }
        _ => true,
    }
}

// =========================================================================
// Action helpers
// =========================================================================

/// Collect all completed missions: sum marks, mark as collected, show message.
fn collect_completed_missions(deep: &mut TheDeepState, ui: &mut DeepUiState) {
    let uncollected: Vec<usize> = deep
        .run
        .completed_missions
        .iter()
        .enumerate()
        .filter(|(_, m)| !m.collected)
        .map(|(i, _)| i)
        .collect();

    if uncollected.is_empty() {
        ui.set_message("No completed missions to collect.");
        return;
    }

    let total_marks: u32 = uncollected
        .iter()
        .map(|&i| deep.run.completed_missions[i].marks_earned)
        .sum();

    for &i in &uncollected {
        deep.run.completed_missions[i].collected = true;
    }

    ui.set_message(format!(
        "Collected {} missions (+{} Marks).",
        uncollected.len(),
        total_marks
    ));
}

/// Attempt a guild upgrade; report success or failure.
fn attempt_guild_upgrade(deep: &mut TheDeepState, ui: &mut DeepUiState) {
    match upgrade_guild(deep) {
        Ok(new_rank) => {
            ui.set_message(format!("Guild upgraded to {:?}!", new_rank));
        }
        Err(e) => {
            ui.set_message(format!("Cannot upgrade guild: {}", e));
        }
    }
}

/// Refresh the recruitment pool if it is empty or stale.
fn refresh_recruitment_pool(
    deep: &mut TheDeepState,
    ui: &mut DeepUiState,
    rng: &mut impl rand::Rng,
) {
    if deep.run.recruitment_pool.is_empty() {
        deep.run.recruitment_pool = generate_recruitment_pool(deep.account.guild_rank, rng);
        ui.set_message("Recruitment pool refreshed.");
    } else {
        ui.set_message("Pool already available — hire or wait for daily refresh.");
    }
}

// =========================================================================
// Helpers
// =========================================================================

/// Returns the number of items in the current tab's list, used to clamp the
/// selected index on Down presses.
fn list_len(deep: &TheDeepState, tab: &DeepTab) -> usize {
    match tab {
        DeepTab::Dashboard => 0,
        DeepTab::Roster => deep.run.mercenaries.len(),
        DeepTab::Missions => deep.run.active_missions.len() + deep.run.completed_missions.len(),
        DeepTab::Recruitment => deep.run.recruitment_pool.len(),
    }
}
