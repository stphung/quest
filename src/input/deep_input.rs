//! The Deep overlay input handling.
//!
//! Handles keyboard input for all sub-views of The Deep overlay:
//! Hub, NewMission (with squad staging), Roster, Layers (infrastructure),
//! EventResponse, and Recruit.

use super::types::InputResult;
use crate::core::game_state::GameState;
use crate::deep::types::{DeepState, DeepUiState, DeepView, MissionStatus};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Top-level dispatcher for The Deep overlay input.
///
/// Routes input to the appropriate sub-view handler based on `deep_ui.view`.
/// Tab/BackTab cycle between navigable tabs from any view (except during squad staging).
pub(super) fn handle_deep(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    game_state: &mut GameState,
    achievements: &mut crate::achievements::Achievements,
) -> InputResult {
    // Farewell modal: dismiss with Enter
    if !deep_ui.farewell_mercs.is_empty() {
        if matches!(key.code, KeyCode::Enter) {
            deep_ui.farewell_mercs.clear();
        }
        return InputResult::Continue;
    }

    // Clear any transient error message on the next key press.
    deep_ui.flash_message = None;

    // [?] toggles help panel from any view.
    if key.code == KeyCode::Char('?') {
        deep_ui.show_help = !deep_ui.show_help;
        return InputResult::Continue;
    }

    // Tab cycling works from any view except EventResponse and squad staging.
    if deep_ui.view != DeepView::EventResponse && deep_ui.staging_mission_index.is_none() {
        match key.code {
            KeyCode::Tab | KeyCode::Right => {
                let new_view = deep_ui.view.next_tab();
                switch_view(deep_ui, new_view);
                return InputResult::Continue;
            }
            KeyCode::BackTab | KeyCode::Left => {
                let new_view = deep_ui.view.prev_tab();
                switch_view(deep_ui, new_view);
                return InputResult::Continue;
            }
            _ => {}
        }
    }

    match deep_ui.view {
        DeepView::Hub => handle_hub(key, deep_state, deep_ui, game_state, achievements),
        DeepView::NewMission => handle_new_mission(key, deep_state, deep_ui),
        DeepView::Roster => handle_roster(key, deep_state, deep_ui),
        DeepView::Infrastructure => handle_infrastructure(key, deep_state, deep_ui),
        DeepView::EventResponse => handle_event_response(key, deep_state, deep_ui),
        DeepView::Recruit => handle_recruit(key, deep_state, deep_ui),
    }
}

// ── Hub View ────────────────────────────────────────────────────────────────────

fn handle_hub(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    game_state: &mut GameState,
    _achievements: &mut crate::achievements::Achievements,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            InputResult::Continue
        }
        KeyCode::Down => {
            let mission_count = deep_state.prestige.active_missions.len()
                + deep_state.prestige.pending_results.len();
            if mission_count > 0 && deep_ui.selected_index + 1 < mission_count {
                deep_ui.selected_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            // If a completed mission is selected, collect its results.
            let pending_count = deep_state.prestige.pending_results.len();
            if pending_count > 0 {
                let active_count = deep_state.prestige.active_missions.len();
                if deep_ui.selected_index >= active_count {
                    let pending_idx = deep_ui.selected_index - active_count;
                    if pending_idx < pending_count {
                        // Apply character rewards before removing the result.
                        let mission = &deep_state.prestige.pending_results[pending_idx];
                        if let Some(ref result) = mission.result {
                            if result.xp_earned > 0 {
                                game_state.character_xp += result.xp_earned as u64;
                            }
                            if result.stormglass_earned > 0 {
                                game_state.stormglass += result.stormglass_earned;
                            }
                        }

                        deep_state.prestige.pending_results.remove(pending_idx);
                        // Clamp selected index after removal.
                        let total = active_count + deep_state.prestige.pending_results.len();
                        if deep_ui.selected_index >= total && total > 0 {
                            deep_ui.selected_index = total - 1;
                        }
                        return InputResult::NeedsSave;
                    }
                }
            }
            // If an active mission with a pending event is selected, jump to event response.
            if let Some(mission) = deep_state
                .prestige
                .active_missions
                .get(deep_ui.selected_index)
            {
                if mission.has_pending_event() {
                    deep_ui.event_mission_id = Some(mission.id);
                    deep_ui.event_choice_index = 0;
                    deep_ui.view = DeepView::EventResponse;
                    deep_ui.event_visit_count = deep_ui.event_visit_count.saturating_add(1);
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
            deep_ui.close();
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// ── New Mission View ────────────────────────────────────────────────────────────

fn handle_new_mission(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
) -> InputResult {
    // If we're in squad staging mode, delegate to squad assignment handler.
    if deep_ui.staging_mission_index.is_some() {
        return handle_squad_assignment(key, deep_state, deep_ui);
    }

    match key.code {
        KeyCode::Up => {
            deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            InputResult::Continue
        }
        KeyCode::Down => {
            let count = deep_state.prestige.available_missions.len();
            if count > 0 && deep_ui.selected_index + 1 < count {
                deep_ui.selected_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            let count = deep_state.prestige.available_missions.len();
            if deep_ui.selected_index < count {
                // Check concurrent mission limit.
                let active = deep_state.prestige.active_mission_count() as u32;
                let max = deep_state.persistent.guild_rank.concurrent_missions();
                if active < max {
                    deep_ui.staging_mission_index = Some(deep_ui.selected_index);
                    deep_ui.staged_squad.clear();
                    deep_ui.selected_index = 0; // Reset cursor for roster browsing.
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            deep_ui.view = DeepView::Hub;
            deep_ui.selected_index = 0;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// ── Squad Assignment (sub-mode of New Mission) ──────────────────────────────────

fn handle_squad_assignment(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
) -> InputResult {
    let available_mercs: Vec<u64> = deep_state
        .prestige
        .roster
        .iter()
        .filter(|m| m.is_available())
        .map(|m| m.id)
        .collect();

    match key.code {
        KeyCode::Up => {
            deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            InputResult::Continue
        }
        KeyCode::Down => {
            if !available_mercs.is_empty() && deep_ui.selected_index + 1 < available_mercs.len() {
                deep_ui.selected_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Char(' ') => {
            // Toggle merc selection.
            if let Some(&merc_id) = available_mercs.get(deep_ui.selected_index) {
                if let Some(pos) = deep_ui.staged_squad.iter().position(|&id| id == merc_id) {
                    deep_ui.staged_squad.remove(pos);
                } else {
                    // Cap squad size at 5.
                    if deep_ui.staged_squad.len() < 5 {
                        deep_ui.staged_squad.push(merc_id);
                    }
                }
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            // Confirm and launch the mission if squad is non-empty.
            if deep_ui.staged_squad.is_empty() {
                deep_ui.flash_message = Some("Select at least one merc with [Space]".to_string());
                return InputResult::Continue;
            }

            if let Some(mission_idx) = deep_ui.staging_mission_index {
                if mission_idx < deep_state.prestige.available_missions.len() {
                    let available = deep_state.prestige.available_missions.remove(mission_idx);

                    // Check and deduct cost.
                    if available.marks_cost > 0
                        && !deep_state.prestige.spend_marks(available.marks_cost)
                    {
                        // Can't afford — put mission back.
                        deep_ui.flash_message = Some(format!(
                            "Not enough Marks! Need {} but have {}",
                            available.marks_cost, deep_state.prestige.warband_marks
                        ));
                        deep_state
                            .prestige
                            .available_missions
                            .insert(mission_idx, available);
                        return InputResult::Continue;
                    }

                    // Build mission with wall-clock timing.
                    let now = chrono::Utc::now();
                    let duration_reduction = deep_state
                        .persistent
                        .layer_record(available.layer)
                        .map(|l| l.total_duration_reduction())
                        .unwrap_or(0.0);
                    let effective_duration_secs =
                        (available.duration_secs as f64 * (1.0 - duration_reduction)) as i64;

                    let mission_id = deep_state.persistent.next_mission_id();

                    // Mark squad mercs as on-mission.
                    for &merc_id in &deep_ui.staged_squad {
                        if let Some(merc) = deep_state.prestige.find_merc_mut(merc_id) {
                            merc.status = crate::deep::MercStatus::OnMission(mission_id);
                        }
                    }

                    let mission = crate::deep::Mission {
                        id: mission_id,
                        mission_type: available.mission_type,
                        layer: available.layer,
                        squad: deep_ui.staged_squad.clone(),
                        started_at: now,
                        ends_at: now + chrono::Duration::seconds(effective_duration_secs),
                        events: Vec::new(), // Events will be generated by logic.rs
                        pending_event_index: 0,
                        status: MissionStatus::Active,
                        result: None,
                    };

                    deep_state.prestige.active_missions.push(mission);

                    // Clean up staging state and return to hub.
                    deep_ui.staging_mission_index = None;
                    deep_ui.staged_squad.clear();
                    deep_ui.view = DeepView::Hub;
                    deep_ui.selected_index = 0;

                    return InputResult::NeedsSave;
                }
            }

            // Staging index was invalid — clean up.
            deep_ui.staging_mission_index = None;
            deep_ui.staged_squad.clear();
            InputResult::Continue
        }
        KeyCode::Esc => {
            // Cancel squad assignment, return to mission list.
            deep_ui.staging_mission_index = None;
            deep_ui.staged_squad.clear();
            deep_ui.selected_index = 0;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// ── Event Response View ─────────────────────────────────────────────────────────

fn handle_event_response(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
) -> InputResult {
    let mission_id = match deep_ui.event_mission_id {
        Some(id) => id,
        None => {
            deep_ui.view = DeepView::Hub;
            return InputResult::Continue;
        }
    };

    // Find the mission and its pending event.
    let mission = match deep_state.prestige.find_mission_mut(mission_id) {
        Some(m) => m,
        None => {
            deep_ui.view = DeepView::Hub;
            deep_ui.event_mission_id = None;
            return InputResult::Continue;
        }
    };

    // Find the first unresolved event.
    let event_idx = mission.events.iter().position(|e| !e.is_resolved());

    let event_idx = match event_idx {
        Some(idx) => idx,
        None => {
            // No pending events — return to hub.
            mission.status = MissionStatus::Active;
            deep_ui.view = DeepView::Hub;
            deep_ui.event_mission_id = None;
            return InputResult::Continue;
        }
    };

    let choice_count = mission.events[event_idx].choices.len();

    match key.code {
        KeyCode::Char('1') if choice_count > 0 => {
            mission.events[event_idx].resolved_choice = Some(0);
            transition_after_event_resolve(mission, deep_ui);
            InputResult::NeedsSave
        }
        KeyCode::Char('2') if choice_count > 1 => {
            mission.events[event_idx].resolved_choice = Some(1);
            transition_after_event_resolve(mission, deep_ui);
            InputResult::NeedsSave
        }
        KeyCode::Char('3') if choice_count > 2 => {
            mission.events[event_idx].resolved_choice = Some(2);
            transition_after_event_resolve(mission, deep_ui);
            InputResult::NeedsSave
        }
        KeyCode::Up => {
            if deep_ui.event_choice_index > 0 {
                deep_ui.event_choice_index -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if choice_count > 0 && deep_ui.event_choice_index + 1 < choice_count {
                deep_ui.event_choice_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            // Confirm currently highlighted choice.
            if deep_ui.event_choice_index < choice_count {
                mission.events[event_idx].resolved_choice = Some(deep_ui.event_choice_index);
                transition_after_event_resolve(mission, deep_ui);
                return InputResult::NeedsSave;
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            // Leave the event unresolved (auto-resolve will handle it later).
            deep_ui.view = DeepView::Hub;
            deep_ui.event_mission_id = None;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// After resolving an event, transition the mission back to Active and return to hub.
fn transition_after_event_resolve(mission: &mut crate::deep::Mission, deep_ui: &mut DeepUiState) {
    // If no more unresolved events, set mission back to Active.
    if mission.unresolved_event_count() == 0 {
        mission.status = MissionStatus::Active;
    }
    deep_ui.view = DeepView::Hub;
    deep_ui.event_mission_id = None;
    deep_ui.event_choice_index = 0;
}

// ── Roster View ─────────────────────────────────────────────────────────────────

fn handle_roster(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            InputResult::Continue
        }
        KeyCode::Down => {
            let count = deep_state.prestige.roster.len();
            if count > 0 && deep_ui.selected_index + 1 < count {
                deep_ui.selected_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            deep_ui.view = DeepView::Hub;
            deep_ui.selected_index = 0;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// ── Infrastructure (Layers) View ────────────────────────────────────────────────

fn handle_infrastructure(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            InputResult::Continue
        }
        KeyCode::Down => {
            let count = deep_state.persistent.layers.len();
            if count > 0 && deep_ui.selected_index + 1 < count {
                deep_ui.selected_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            deep_ui.view = DeepView::Hub;
            deep_ui.selected_index = 0;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// ── Recruit View ────────────────────────────────────────────────────────────────

fn handle_recruit(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            InputResult::Continue
        }
        KeyCode::Down => {
            let count = deep_state.prestige.recruit_pool.candidates.len();
            if count > 0 && deep_ui.selected_index + 1 < count {
                deep_ui.selected_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            // Recruit the selected candidate.
            let idx = deep_ui.selected_index;
            let pool = &deep_state.prestige.recruit_pool;
            if idx < pool.candidates.len() {
                // Check roster cap.
                let max_roster = deep_state.persistent.guild_rank.max_roster() as usize;
                if deep_state.prestige.roster.len() >= max_roster {
                    return InputResult::Continue;
                }

                // Check cost.
                let cost = pool.recruit_costs.get(idx).copied().unwrap_or(0);
                if cost > 0 && !deep_state.prestige.spend_marks(cost) {
                    return InputResult::Continue;
                }

                // Move candidate from pool to roster.
                let mut merc = deep_state.prestige.recruit_pool.candidates.remove(idx);
                deep_state.prestige.recruit_pool.recruit_costs.remove(idx);
                merc.id = deep_state.persistent.next_merc_id();
                deep_state.prestige.roster.push(merc);

                // Clamp selection after removal.
                let remaining = deep_state.prestige.recruit_pool.candidates.len();
                if deep_ui.selected_index >= remaining && remaining > 0 {
                    deep_ui.selected_index = remaining - 1;
                }

                return InputResult::NeedsSave;
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            deep_ui.view = DeepView::Hub;
            deep_ui.selected_index = 0;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// ── View switching with visit counter ──────────────────────────────────────────

/// Switch to a new view, resetting selection and incrementing the visit counter.
fn switch_view(ui: &mut DeepUiState, target: DeepView) {
    ui.view = target;
    ui.selected_index = 0;
    ui.staged_squad.clear();
    ui.show_help = false;
    match target {
        DeepView::Hub => ui.hub_visit_count = ui.hub_visit_count.saturating_add(1),
        DeepView::NewMission => ui.mission_visit_count = ui.mission_visit_count.saturating_add(1),
        DeepView::Roster => ui.roster_visit_count = ui.roster_visit_count.saturating_add(1),
        DeepView::Infrastructure => ui.layer_visit_count = ui.layer_visit_count.saturating_add(1),
        DeepView::EventResponse => ui.event_visit_count = ui.event_visit_count.saturating_add(1),
        DeepView::Recruit => ui.recruit_visit_count = ui.recruit_visit_count.saturating_add(1),
    }
}
