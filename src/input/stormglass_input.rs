//! Input handling for the Stormglass Exchange overlay.

use crate::challenges::menu::create_challenge;
use crate::core::game_state::GameState;
use crate::input::InputResult;
use crate::stormglass::sigils::ETCH_COST;
use crate::stormglass::spending::{chrono_surge_cost, generate_trial_options};
use crate::stormglass::types::{
    ExchangePhase, ExchangeUiState, CHRONO_SURGE_OPTIONS, EXCHANGE_MENU_ITEMS, INVOKE_TRIAL_COST,
    STORM_LURE_COST,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current time in milliseconds since UNIX epoch.
fn current_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Handle input while the Stormglass Exchange overlay is open.
pub fn handle_stormglass_exchange(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match exchange_ui.phase {
        ExchangePhase::Menu => handle_menu(key, exchange_ui, state),
        ExchangePhase::InvokeTrialConfirm => handle_invoke_trial_confirm(key, exchange_ui, state),
        ExchangePhase::InvokeTrialRolling => handle_invoke_trial_rolling(key, exchange_ui),
        ExchangePhase::InvokeTrial => handle_invoke_trial(key, exchange_ui, state),
        ExchangePhase::InvokeTrialForfeitConfirm => {
            handle_invoke_trial_forfeit_confirm(key, exchange_ui)
        }
        ExchangePhase::ChronoSurge => handle_chrono_surge_select(key, exchange_ui, state),
        ExchangePhase::SigilsList => handle_sigils_list(key, exchange_ui, state),
        ExchangePhase::SigilUnlockConfirm => handle_sigil_unlock_confirm(key, exchange_ui, state),
        ExchangePhase::SigilEtchConfirm => handle_sigil_etch_confirm(key, exchange_ui, state),
        ExchangePhase::SigilRerollConfirm => handle_sigil_reroll_confirm(key, exchange_ui, state),
        ExchangePhase::SigilRolling => handle_sigil_rolling(key, exchange_ui),
        ExchangePhase::SigilPick => handle_sigil_pick(key, exchange_ui, state),
        ExchangePhase::SigilForfeitConfirm => handle_sigil_forfeit_confirm(key, exchange_ui),
        ExchangePhase::SigilResult => handle_sigil_result(key, exchange_ui),
        ExchangePhase::StormLureConfirm => handle_storm_lure_confirm(key, exchange_ui, state),
    }
}

fn handle_menu(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            if exchange_ui.selected_item > 0 {
                exchange_ui.selected_item -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if exchange_ui.selected_item + 1 < EXCHANGE_MENU_ITEMS {
                exchange_ui.selected_item += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            match exchange_ui.selected_item {
                0 => {
                    // Invoke Challenge — show confirmation first
                    if state.stormglass >= INVOKE_TRIAL_COST {
                        // Guard: if all 10 challenge types are already pending, do nothing
                        if state.challenge_menu.challenges.len() >= 10 {
                            return InputResult::Continue;
                        }
                        exchange_ui.set_phase(ExchangePhase::InvokeTrialConfirm);
                    }
                }
                1 => {
                    // Chrono Surge — enter duration selection
                    exchange_ui.surge_selected = 0;
                    exchange_ui.set_phase(ExchangePhase::ChronoSurge);
                }
                2 => {
                    // Storm Sigils — enter sigils list
                    exchange_ui.sigil_selected_slot = 0;
                    exchange_ui.set_phase(ExchangePhase::SigilsList);
                }
                3 => {
                    // Storm Lure — show confirmation
                    if state.stormglass >= STORM_LURE_COST
                        && !state.fishing.storm_lure_active
                        && !state.fishing.leviathan_caught
                        && state.fishing.rank >= 40
                    {
                        exchange_ui.set_phase(ExchangePhase::StormLureConfirm);
                    }
                }
                _ => {}
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            exchange_ui.close();
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_chrono_surge_select(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            if exchange_ui.surge_selected > 0 {
                exchange_ui.surge_selected -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if exchange_ui.surge_selected + 1 < CHRONO_SURGE_OPTIONS.len() {
                exchange_ui.surge_selected += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            if let Some((ticks, cost, _label)) = chrono_surge_cost(exchange_ui.surge_selected) {
                if state.stormglass >= cost {
                    state.stormglass -= cost;
                    exchange_ui.close();
                    return InputResult::StartChronoSurge { ticks };
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::Menu);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_invoke_trial_confirm(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Deduct SG and generate trial options
            state.stormglass -= INVOKE_TRIAL_COST;
            let mut rng = rand::rng();
            let pending: Vec<_> = state
                .challenge_menu
                .challenges
                .iter()
                .map(|c| c.challenge_type.clone())
                .collect();
            exchange_ui.trial_options = generate_trial_options(&mut rng, &pending);
            exchange_ui.trial_selected = 0;
            exchange_ui.invoke_animation_start_ms = Some(current_millis());
            exchange_ui.set_phase(ExchangePhase::InvokeTrialRolling);
            InputResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::Menu);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_invoke_trial_forfeit_confirm(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
) -> InputResult {
    match key.code {
        KeyCode::Enter => {
            // Confirm forfeit — SG already gone
            exchange_ui.close();
            InputResult::Continue
        }
        _ => {
            // Esc or any other key — return to trial selection
            exchange_ui.set_phase(ExchangePhase::InvokeTrial);
            InputResult::Continue
        }
    }
}

fn handle_invoke_trial(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    let option_count = exchange_ui.trial_options.len();
    match key.code {
        KeyCode::Up => {
            if exchange_ui.trial_selected > 0 {
                exchange_ui.trial_selected -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if exchange_ui.trial_selected + 1 < option_count {
                exchange_ui.trial_selected += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            if let Some(trial) = exchange_ui.trial_options.get(exchange_ui.trial_selected) {
                // Create the challenge and start it
                let challenge = create_challenge(&trial.challenge_type);
                state.challenge_menu.add_challenge(challenge);
                // Start the challenge immediately by opening the menu and selecting it
                state.challenge_menu.open();
            }
            exchange_ui.close();
            InputResult::Continue
        }
        KeyCode::Esc => {
            // Forfeit — show confirmation before closing
            exchange_ui.set_phase(ExchangePhase::InvokeTrialForfeitConfirm);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// Minimum display time before invoke animation skip is accepted.
const INVOKE_ANIMATION_SKIP_FLOOR_MS: u128 = 200;

/// Total invoke animation duration before auto-transitioning to trial select.
pub(crate) const INVOKE_ANIMATION_DURATION_MS: u128 = 4200;

fn handle_invoke_trial_rolling(key: KeyEvent, exchange_ui: &mut ExchangeUiState) -> InputResult {
    let _ = key; // any key triggers skip (after floor check)
    let elapsed = exchange_ui
        .invoke_animation_start_ms
        .map(|start| current_millis().saturating_sub(start))
        .unwrap_or(0);

    if elapsed >= INVOKE_ANIMATION_SKIP_FLOOR_MS {
        exchange_ui.invoke_animation_start_ms = None;
        exchange_ui.set_phase(ExchangePhase::InvokeTrial);
    }
    InputResult::Continue
}

fn handle_sigils_list(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    let sigils = &state.storm_sigils;
    // Max selectable index: up to 1 beyond unlocked slots (for unlock action),
    // but capped at MAX_SIGIL_SLOTS - 1
    let max_slot =
        (sigils.slots_unlocked as usize).min(crate::stormglass::sigils::MAX_SIGIL_SLOTS - 1);

    match key.code {
        KeyCode::Up => {
            if exchange_ui.sigil_selected_slot > 0 {
                exchange_ui.sigil_selected_slot -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if exchange_ui.sigil_selected_slot < max_slot {
                exchange_ui.sigil_selected_slot += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            let slot = exchange_ui.sigil_selected_slot;
            if slot >= sigils.slots_unlocked as usize {
                // Locked slot (next unlockable) — go to unlock confirm (requires SG)
                if let Some(cost) = sigils.next_unlock_cost() {
                    if state.stormglass >= cost {
                        exchange_ui.set_phase(ExchangePhase::SigilUnlockConfirm);
                    }
                }
            } else if sigils.sigils[slot].is_some() {
                // Etched slot — reroll (requires SG)
                if state.stormglass >= ETCH_COST {
                    exchange_ui.sigil_target_slot = slot;
                    exchange_ui.set_phase(ExchangePhase::SigilRerollConfirm);
                }
            } else {
                // Empty slot — etch (requires SG)
                if state.stormglass >= ETCH_COST {
                    exchange_ui.sigil_target_slot = slot;
                    exchange_ui.set_phase(ExchangePhase::SigilEtchConfirm);
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::Menu);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_sigil_unlock_confirm(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(cost) = state.storm_sigils.next_unlock_cost() {
                if state.stormglass >= cost {
                    state.stormglass -= cost;
                    state.storm_sigils.slots_unlocked += 1;
                }
            }
            exchange_ui.set_phase(ExchangePhase::SigilsList);
            InputResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::SigilsList);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_sigil_etch_confirm(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if state.stormglass >= ETCH_COST {
                state.stormglass -= ETCH_COST;
                let mut rng = rand::rng();
                let pool = crate::stormglass::sigils::daily_sigil_pool();
                let choices = crate::stormglass::sigils::generate_sigil_choices(&mut rng, &pool);
                exchange_ui.sigil_choices = choices.map(Some);
                exchange_ui.sigil_pick_selected = 0;
                exchange_ui.sigil_animation_start_ms = Some(current_millis());
                exchange_ui.sigil_animation_skipped = false;
                exchange_ui.set_phase(ExchangePhase::SigilRolling);
            }
            InputResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::SigilsList);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_sigil_reroll_confirm(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if state.stormglass >= ETCH_COST {
                state.stormglass -= ETCH_COST;
                // Destroy current sigil
                let slot = exchange_ui.sigil_target_slot;
                state.storm_sigils.sigils[slot] = None;
                state.invalidate_bonuses();
                let mut rng = rand::rng();
                let pool = crate::stormglass::sigils::daily_sigil_pool();
                let choices = crate::stormglass::sigils::generate_sigil_choices(&mut rng, &pool);
                exchange_ui.sigil_choices = choices.map(Some);
                exchange_ui.sigil_pick_selected = 0;
                exchange_ui.sigil_animation_start_ms = Some(current_millis());
                exchange_ui.sigil_animation_skipped = false;
                exchange_ui.set_phase(ExchangePhase::SigilRolling);
            }
            InputResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::SigilsList);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_sigil_pick(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Up => {
            if exchange_ui.sigil_pick_selected > 0 {
                exchange_ui.sigil_pick_selected -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if exchange_ui.sigil_pick_selected < 2 {
                exchange_ui.sigil_pick_selected += 1;
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            let idx = exchange_ui.sigil_pick_selected;
            if let Some(sigil) = exchange_ui.sigil_choices[idx].take() {
                let slot = exchange_ui.sigil_target_slot;
                exchange_ui.sigil_result = Some(sigil.clone());
                state.storm_sigils.sigils[slot] = Some(sigil);
                state.invalidate_bonuses();
                exchange_ui.set_phase(ExchangePhase::SigilResult);
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::SigilForfeitConfirm);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_sigil_forfeit_confirm(key: KeyEvent, exchange_ui: &mut ExchangeUiState) -> InputResult {
    match key.code {
        KeyCode::Enter => {
            // Confirm forfeit — SG already gone, slot stays empty
            exchange_ui.sigil_choices = [None, None, None];
            exchange_ui.set_phase(ExchangePhase::SigilsList);
            InputResult::Continue
        }
        _ => {
            // Esc or any other key — return to pick screen
            exchange_ui.set_phase(ExchangePhase::SigilPick);
            InputResult::Continue
        }
    }
}

fn handle_storm_lure_confirm(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if state.stormglass >= STORM_LURE_COST
                && !state.fishing.storm_lure_active
                && !state.fishing.leviathan_caught
            {
                state.stormglass -= STORM_LURE_COST;
                state.fishing.storm_lure_active = true;
                state.fishing.lure_miss_ramp = 0.0;
                exchange_ui.close();
            }
            InputResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            exchange_ui.set_phase(ExchangePhase::Menu);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_sigil_result(key: KeyEvent, exchange_ui: &mut ExchangeUiState) -> InputResult {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            exchange_ui.sigil_result = None;
            exchange_ui.sigil_choices = [None, None, None];
            exchange_ui.set_phase(ExchangePhase::SigilsList);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// Minimum display time before skip is accepted (prevents key-repeat instant-skip).
const SIGIL_ANIMATION_SKIP_FLOOR_MS: u128 = 200;

/// Total animation duration before auto-transitioning to pick.
pub(crate) const SIGIL_ANIMATION_DURATION_MS: u128 = 4000;

fn handle_sigil_rolling(key: KeyEvent, exchange_ui: &mut ExchangeUiState) -> InputResult {
    let _ = key; // any key triggers skip (after floor check)
    let elapsed = exchange_ui
        .sigil_animation_start_ms
        .map(|start| current_millis().saturating_sub(start))
        .unwrap_or(0);

    if elapsed >= SIGIL_ANIMATION_SKIP_FLOOR_MS {
        exchange_ui.sigil_animation_start_ms = None;
        exchange_ui.sigil_animation_skipped = false;
        exchange_ui.set_phase(ExchangePhase::SigilPick);
    }
    InputResult::Continue
}

/// Check if sigil animation has completed and should auto-transition to pick.
/// Called from the main game loop each tick.
pub fn check_sigil_animation_timeout(exchange_ui: &mut ExchangeUiState) {
    if exchange_ui.phase == ExchangePhase::InvokeTrialRolling {
        let elapsed = exchange_ui
            .invoke_animation_start_ms
            .map(|start| current_millis().saturating_sub(start))
            .unwrap_or(0);

        if elapsed >= INVOKE_ANIMATION_DURATION_MS {
            exchange_ui.invoke_animation_start_ms = None;
            exchange_ui.set_phase(ExchangePhase::InvokeTrial);
        }
        return;
    }

    if exchange_ui.phase != ExchangePhase::SigilRolling {
        return;
    }
    let elapsed = exchange_ui
        .sigil_animation_start_ms
        .map(|start| current_millis().saturating_sub(start))
        .unwrap_or(0);

    if elapsed >= SIGIL_ANIMATION_DURATION_MS {
        exchange_ui.sigil_animation_start_ms = None;
        exchange_ui.sigil_animation_skipped = false;
        exchange_ui.set_phase(ExchangePhase::SigilPick);
    }
}
