//! The Deep overlay input handling.
//!
//! Handles keyboard input for all sub-views of The Deep overlay:
//! Hub (warband operations), NewMission (with squad staging), Roster,
//! Layers (infrastructure), EventResponse, and Recruit.

use super::types::InputResult;
use crate::core::game_state::GameState;
use crate::deep::types::{
    DeepState, DeepUiState, DeepView, MercStatus, MissionStatus, MissionType,
};
use chrono::Utc;
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
    debug_mode: bool,
) -> InputResult {
    // Farewell modal: dismiss with Enter
    if !deep_ui.farewell_mercs.is_empty() {
        if matches!(key.code, KeyCode::Enter) {
            deep_ui.farewell_mercs.clear();
        }
        return InputResult::Continue;
    }

    // Mission-complete modal: capture Enter and block background input.
    if !deep_state.prestige.pending_results.is_empty() {
        if matches!(key.code, KeyCode::Enter)
            && collect_pending_result(deep_state, deep_ui, game_state, 0)
        {
            return InputResult::NeedsSave;
        }
        return InputResult::Continue;
    }

    // Event modal: intercept input before tab cycling and view dispatch.
    if deep_ui.event_modal_open {
        return handle_event_response(key, deep_state, deep_ui);
    }

    // Clear any transient error message on the next key press.
    deep_ui.flash_message = None;

    // [?] toggles help panel from any view.
    if key.code == KeyCode::Char('?') {
        deep_ui.show_help = !deep_ui.show_help;
        return InputResult::Continue;
    }

    // Left/Right arrow cycling works from any view except during squad staging.
    if deep_ui.staging_mission_index.is_none() {
        match key.code {
            KeyCode::Right => {
                let new_view = deep_ui.view.next_tab();
                switch_view(deep_ui, new_view);
                return InputResult::Continue;
            }
            KeyCode::Left => {
                let new_view = deep_ui.view.prev_tab();
                switch_view(deep_ui, new_view);
                return InputResult::Continue;
            }
            _ => {}
        }
    }

    match deep_ui.view {
        DeepView::Hub => handle_hub(key, deep_state, deep_ui, game_state, achievements),
        DeepView::NewMission => {
            handle_new_mission(key, deep_state, deep_ui, game_state, debug_mode)
        }
        DeepView::Infrastructure => handle_infrastructure(key, deep_state, deep_ui),
        DeepView::Roster | DeepView::EventResponse | DeepView::Recruit => {
            // Roster and Recruit are absorbed into Status tab; EventResponse is a modal.
            // All arms are unreachable but kept for exhaustive matching.
            InputResult::Continue
        }
    }
}

// ── Hub View ────────────────────────────────────────────────────────────────────

enum HubSelection {
    Completed(usize),
    Active(usize),
}

/// Active mission order used by the Hub UI: pending events first, then earliest completion.
fn hub_active_display_order(deep_state: &DeepState) -> Vec<usize> {
    let now = Utc::now();
    let mut order: Vec<(usize, bool, u64)> = deep_state
        .prestige
        .active_missions
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let has_event = m.has_pending_event();
            let remaining = (m.ends_at - now).num_seconds().max(0) as u64;
            (i, has_event, remaining)
        })
        .collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    order.into_iter().map(|(idx, _, _)| idx).collect()
}

/// Map hub cursor index to the UI-visible mission ordering.
/// Hub render order is: completed results first, then sorted active missions.
fn map_hub_selection(deep_state: &DeepState, selected_index: usize) -> Option<HubSelection> {
    let pending_count = deep_state.prestige.pending_results.len();
    if selected_index < pending_count {
        return Some(HubSelection::Completed(selected_index));
    }

    let active_display_idx = selected_index - pending_count;
    let active_order = hub_active_display_order(deep_state);
    active_order
        .get(active_display_idx)
        .copied()
        .map(HubSelection::Active)
}

fn handle_hub(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    _game_state: &mut GameState,
    _achievements: &mut crate::achievements::Achievements,
) -> InputResult {
    // Shared keys for both sub-views.
    match key.code {
        KeyCode::Tab | KeyCode::BackTab => {
            // Toggle between Roster and Recruit sub-views.
            deep_ui.status_show_recruit = !deep_ui.status_show_recruit;
            deep_ui.selected_index = 0;
            return InputResult::Continue;
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            // Attempt guild rank upgrade (available from both sub-views).
            match crate::deep::try_upgrade_guild_rank(
                &mut deep_state.persistent,
                &mut deep_state.prestige,
            ) {
                Ok(new_rank) => {
                    _achievements.on_deep_guild_rank_up(new_rank.0 as u32, None);
                    deep_ui.flash_message = Some(format!(
                        "Guild promoted to Rank {} — {}!",
                        new_rank.0,
                        new_rank.display_name(),
                    ));
                    return InputResult::NeedsSave;
                }
                Err(e) => {
                    let msg = match e {
                        crate::deep::GuildUpgradeError::AlreadyMaxRank => {
                            "Already at maximum guild rank.".to_string()
                        }
                        crate::deep::GuildUpgradeError::LayerRequirementNotMet {
                            required_layer,
                        } => {
                            format!("Need Layer {} breakthrough first.", required_layer)
                        }
                        crate::deep::GuildUpgradeError::InsufficientMarks {
                            required,
                            available,
                        } => {
                            format!("Need {} Marks (have {}).", required, available)
                        }
                    };
                    deep_ui.flash_message = Some(msg);
                    return InputResult::Continue;
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
            deep_ui.close();
            return InputResult::Continue;
        }
        _ => {}
    }

    if deep_ui.status_show_recruit {
        // ── Recruit sub-view ──
        match key.code {
            KeyCode::Up => {
                deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                let count = deep_state.prestige.recruit_pool.candidates.len();
                if count > 0 && deep_ui.selected_index + 1 < count {
                    deep_ui.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                // Recruit the selected candidate.
                let idx = deep_ui.selected_index;
                let pool = &deep_state.prestige.recruit_pool;
                if idx < pool.candidates.len() {
                    let max_roster = deep_state.persistent.guild_rank.max_roster() as usize;
                    let occupied_slots = deep_state
                        .prestige
                        .roster
                        .iter()
                        .filter(|m| !matches!(m.status, MercStatus::Lost))
                        .count();
                    if occupied_slots >= max_roster {
                        return InputResult::Continue;
                    }
                    let cost = pool.recruit_costs.get(idx).copied().unwrap_or(0);
                    if cost > 0 && !deep_state.prestige.spend_marks(cost) {
                        return InputResult::Continue;
                    }
                    let mut merc = deep_state.prestige.recruit_pool.candidates.remove(idx);
                    deep_state.prestige.recruit_pool.recruit_costs.remove(idx);
                    merc.id = deep_state.persistent.next_merc_id();
                    deep_state.prestige.roster.push(merc);

                    let now = Utc::now();
                    let mut rng = rand::rng();
                    let _ = crate::deep::missions::maybe_refresh_mission_pool(
                        &mut deep_state.prestige,
                        &deep_state.persistent,
                        now,
                        &mut rng,
                    );

                    let remaining = deep_state.prestige.recruit_pool.candidates.len();
                    if deep_ui.selected_index >= remaining && remaining > 0 {
                        deep_ui.selected_index = remaining - 1;
                    }
                    return InputResult::NeedsSave;
                }
            }
            _ => {}
        }
        InputResult::Continue
    } else {
        // ── Roster sub-view (default) ──
        match key.code {
            KeyCode::Up => {
                deep_ui.selected_index = deep_ui.selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                let count = deep_state.prestige.roster.len();
                if count > 0 && deep_ui.selected_index + 1 < count {
                    deep_ui.selected_index += 1;
                }
            }
            _ => {}
        }
        InputResult::Continue
    }
}

fn collect_pending_result(
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    game_state: &mut GameState,
    pending_idx: usize,
) -> bool {
    if pending_idx >= deep_state.prestige.pending_results.len() {
        return false;
    }

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

    // Lost mercs remain visible until all result modals are acknowledged.
    if deep_state.prestige.pending_results.is_empty() {
        let _ = crate::deep::purge_lost_mercs(&mut deep_state.prestige.roster);
    }

    // Active missions are now on the Missions tab; clamp selection there.
    if matches!(deep_ui.view, DeepView::NewMission) && deep_ui.missions_show_active {
        let total =
            deep_state.prestige.active_missions.len() + deep_state.prestige.pending_results.len();
        if total == 0 {
            deep_ui.selected_index = 0;
        } else if deep_ui.selected_index >= total {
            deep_ui.selected_index = total - 1;
        }
    }

    true
}

// ── New Mission View ────────────────────────────────────────────────────────────

fn handle_new_mission(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    game_state: &mut GameState,
    debug_mode: bool,
) -> InputResult {
    // If we're in squad staging mode, delegate to squad assignment handler.
    if deep_ui.staging_mission_index.is_some() {
        return handle_squad_assignment(key, deep_state, deep_ui, debug_mode);
    }

    // Tab toggles between active missions and available pool.
    if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
        deep_ui.missions_show_active = !deep_ui.missions_show_active;
        deep_ui.selected_index = 0;
        return InputResult::Continue;
    }

    if deep_ui.missions_show_active {
        // Active missions sub-view (adapted from former handle_hub).
        handle_active_missions(key, deep_state, deep_ui, game_state)
    } else {
        // Available mission pool sub-view (original behavior).
        handle_mission_pool(key, deep_state, deep_ui, debug_mode)
    }
}

fn handle_active_missions(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    game_state: &mut GameState,
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
            match map_hub_selection(deep_state, deep_ui.selected_index) {
                Some(HubSelection::Completed(pending_idx)) => {
                    if collect_pending_result(deep_state, deep_ui, game_state, pending_idx) {
                        return InputResult::NeedsSave;
                    }
                }
                Some(HubSelection::Active(active_idx)) => {
                    if let Some(mission) = deep_state.prestige.active_missions.get(active_idx) {
                        if mission.has_pending_event() {
                            deep_ui.event_mission_id = Some(mission.id);
                            deep_ui.event_choice_index = 0;
                            deep_ui.event_modal_open = true;
                            deep_ui.event_visit_count = deep_ui.event_visit_count.saturating_add(1);
                        } else {
                            deep_ui.flash_message = Some(
                                "No action yet. Wait for a check-in event or mission completion."
                                    .to_string(),
                            );
                        }
                    }
                }
                None => {}
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            deep_ui.close();
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_mission_pool(
    key: KeyEvent,
    deep_state: &mut DeepState,
    deep_ui: &mut DeepUiState,
    _debug_mode: bool,
) -> InputResult {
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
                let max = crate::deep::effective_concurrent_missions(
                    deep_state.persistent.guild_rank,
                    deep_state.persistent.deepest_layer_reached,
                );
                if active < max {
                    deep_ui.staging_mission_index = Some(deep_ui.selected_index);
                    deep_ui.staged_squad.clear();
                    deep_ui.selected_index = 0; // Reset cursor for roster browsing.
                } else {
                    deep_ui.flash_message = Some(format!(
                        "All mission slots are in use ({}/{}). Wait for completion or rank up.",
                        active, max
                    ));
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            deep_ui.close();
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
    debug_mode: bool,
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
                    let mut effective_duration_secs =
                        (available.duration_secs as f64 * (1.0 - duration_reduction)) as i64;
                    effective_duration_secs =
                        effective_duration_secs
                            .max(crate::deep::minimum_mission_duration_secs_for_layer(
                                available.layer,
                            ) as i64);
                    if available.mission_type == MissionType::SupplyRun && available.marks_cost == 0
                    {
                        // Free Supply Runs are intentionally slower fallback missions.
                        effective_duration_secs = effective_duration_secs
                            .max(crate::deep::missions::FREE_SUPPLY_RUN_MIN_DURATION_SECS as i64);
                    }

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
                        ends_at: if debug_mode {
                            now
                        } else {
                            now + chrono::Duration::seconds(effective_duration_secs)
                        },
                        events: Vec::new(), // Events will be generated by logic.rs
                        pending_event_index: 0,
                        status: MissionStatus::Active,
                        result: None,
                        is_first_orders: false,
                    };

                    deep_state.prestige.active_missions.push(mission);
                    // Rolling refill: keep mission roles balanced after each launch.
                    let mut refill_rng = rand::rng();
                    let _ = crate::deep::missions::maybe_refresh_mission_pool(
                        &mut deep_state.prestige,
                        &deep_state.persistent,
                        now,
                        &mut refill_rng,
                    );

                    // Clean up staging state and switch to active missions view.
                    deep_ui.staging_mission_index = None;
                    deep_ui.staged_squad.clear();
                    deep_ui.missions_show_active = true;
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
    // Handle Esc before early returns so it works even when no event is active.
    if matches!(key.code, KeyCode::Esc) {
        deep_ui.event_mission_id = None;
        deep_ui.event_modal_open = false;
        return InputResult::Continue;
    }

    let mission_id = match deep_ui.event_mission_id {
        Some(id) => id,
        None => {
            deep_ui.event_modal_open = false;
            return InputResult::Continue;
        }
    };

    // Find the mission and its pending event.
    let mission = match deep_state.prestige.find_mission_mut(mission_id) {
        Some(m) => m,
        None => {
            deep_ui.event_modal_open = false;
            deep_ui.event_mission_id = None;
            return InputResult::Continue;
        }
    };

    // Find the first unresolved event.
    let event_idx = mission.events.iter().position(|e| !e.is_resolved());

    let event_idx = match event_idx {
        Some(idx) => idx,
        None => {
            // No pending events — close modal.
            mission.status = MissionStatus::Active;
            deep_ui.event_modal_open = false;
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
        _ => InputResult::Continue,
    }
}

/// After resolving an event, transition the mission back to Active and close the modal.
fn transition_after_event_resolve(mission: &mut crate::deep::Mission, deep_ui: &mut DeepUiState) {
    // If no more unresolved events, set mission back to Active.
    if mission.unresolved_event_count() == 0 {
        mission.status = MissionStatus::Active;
    }
    deep_ui.event_modal_open = false;
    deep_ui.event_mission_id = None;
    deep_ui.event_choice_index = 0;
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
        KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
            deep_ui.close();
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
    ui.missions_show_active = true;
    ui.status_show_recruit = false;
    match target {
        DeepView::Hub => ui.hub_visit_count = ui.hub_visit_count.saturating_add(1),
        DeepView::NewMission => ui.mission_visit_count = ui.mission_visit_count.saturating_add(1),
        DeepView::Roster => ui.roster_visit_count = ui.roster_visit_count.saturating_add(1),
        DeepView::Infrastructure => ui.layer_visit_count = ui.layer_visit_count.saturating_add(1),
        DeepView::EventResponse => ui.event_visit_count = ui.event_visit_count.saturating_add(1),
        DeepView::Recruit => ui.recruit_visit_count = ui.recruit_visit_count.saturating_add(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::Achievements;
    use crate::deep::{
        MercArchetype, MercStatus, Mercenary, Mission, MissionOutcome, MissionResult, MissionType,
    };
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn mission_with_result(xp_earned: u32, stormglass_earned: u64) -> Mission {
        let now = chrono::Utc::now();
        Mission {
            id: 1,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: Vec::new(),
            started_at: now - chrono::Duration::hours(1),
            ends_at: now,
            events: Vec::new(),
            pending_event_index: 0,
            status: MissionStatus::Completed,
            result: Some(MissionResult {
                outcome: MissionOutcome::Success,
                marks_earned: 0,
                xp_earned,
                stormglass_earned,
                item_ilvl: None,
                injured_mercs: Vec::new(),
                lost_mercs: Vec::new(),
                merc_level_ups: Vec::new(),
                danger_bonus_xp: false,
            }),
            is_first_orders: false,
        }
    }

    fn make_lost_merc(id: u64) -> Mercenary {
        Mercenary {
            id,
            name: format!("Lost {}", id),
            archetype: MercArchetype::Vanguard,
            power: 10,
            resilience: 10,
            expertise: 10,
            level: 1,
            missions_completed: 0,
            status: MercStatus::Lost,
        }
    }

    #[test]
    fn mission_result_modal_enter_collects_from_any_view() {
        let mut deep_state = DeepState::new();
        deep_state
            .prestige
            .pending_results
            .push(mission_with_result(125, 9));

        let mut deep_ui = DeepUiState::new();
        deep_ui.view = DeepView::NewMission;

        let mut game_state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        let result = handle_deep(
            key(KeyCode::Enter),
            &mut deep_state,
            &mut deep_ui,
            &mut game_state,
            &mut achievements,
            false,
        );

        assert!(matches!(result, InputResult::NeedsSave));
        assert!(deep_state.prestige.pending_results.is_empty());
        assert_eq!(game_state.character_xp, 125);
        assert_eq!(game_state.stormglass, 9);
        assert_eq!(deep_ui.view, DeepView::NewMission);
    }

    #[test]
    fn mission_result_modal_blocks_background_navigation() {
        let mut deep_state = DeepState::new();
        deep_state
            .prestige
            .pending_results
            .push(mission_with_result(0, 0));

        let mut deep_ui = DeepUiState::new();
        deep_ui.view = DeepView::NewMission;

        let mut game_state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        let result = handle_deep(
            key(KeyCode::Tab),
            &mut deep_state,
            &mut deep_ui,
            &mut game_state,
            &mut achievements,
            false,
        );

        assert!(matches!(result, InputResult::Continue));
        assert_eq!(deep_state.prestige.pending_results.len(), 1);
        assert_eq!(deep_ui.view, DeepView::NewMission);
    }

    #[test]
    fn hub_letter_navigation_shortcuts_are_disabled() {
        let mut deep_state = DeepState::new();
        let mut deep_ui = DeepUiState::new();
        deep_ui.view = DeepView::Hub;
        let mut game_state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();

        for key_code in [KeyCode::Char('n'), KeyCode::Char('r'), KeyCode::Char('l')] {
            let result = handle_deep(
                key(key_code),
                &mut deep_state,
                &mut deep_ui,
                &mut game_state,
                &mut achievements,
                false,
            );
            assert!(matches!(result, InputResult::Continue));
            assert_eq!(deep_ui.view, DeepView::Hub);
        }
    }

    #[test]
    fn collect_pending_result_purges_lost_after_last_result() {
        let mut deep_state = DeepState::new();
        deep_state.prestige.roster.push(make_lost_merc(7));
        deep_state
            .prestige
            .pending_results
            .push(mission_with_result(0, 0));
        let mut deep_ui = DeepUiState::new();
        let mut game_state = GameState::new("Hero".to_string(), 0);

        let collected = collect_pending_result(&mut deep_state, &mut deep_ui, &mut game_state, 0);
        assert!(collected);
        assert!(deep_state.prestige.pending_results.is_empty());
        assert!(
            deep_state.prestige.roster.is_empty(),
            "Lost merc should be purged once all results are collected"
        );
    }

    #[test]
    fn collect_pending_result_keeps_lost_while_other_results_remain() {
        let mut deep_state = DeepState::new();
        deep_state.prestige.roster.push(make_lost_merc(7));
        deep_state
            .prestige
            .pending_results
            .push(mission_with_result(0, 0));
        deep_state
            .prestige
            .pending_results
            .push(mission_with_result(0, 0));
        let mut deep_ui = DeepUiState::new();
        let mut game_state = GameState::new("Hero".to_string(), 0);

        let collected = collect_pending_result(&mut deep_state, &mut deep_ui, &mut game_state, 0);
        assert!(collected);
        assert_eq!(deep_state.prestige.pending_results.len(), 1);
        assert_eq!(
            deep_state.prestige.roster.len(),
            1,
            "Lost merc should stay visible until all results are acknowledged"
        );
        assert!(matches!(
            deep_state.prestige.roster[0].status,
            MercStatus::Lost
        ));
    }
}
