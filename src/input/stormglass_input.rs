//! Input handling for the Stormglass Exchange overlay.

use crate::challenges::menu::create_challenge;
use crate::core::game_state::GameState;
use crate::input::InputResult;
use crate::stormglass::sigils::INSCRIBE_COST;
use crate::stormglass::spending::{chrono_surge_cost, generate_trial_options};
use crate::stormglass::types::{
    ExchangePhase, ExchangeUiState, CHRONO_SURGE_OPTIONS, EXCHANGE_MENU_ITEMS, INVOKE_TRIAL_COST,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle input while the Stormglass Exchange overlay is open.
pub fn handle_stormglass_exchange(
    key: KeyEvent,
    exchange_ui: &mut ExchangeUiState,
    state: &mut GameState,
) -> InputResult {
    match exchange_ui.phase {
        ExchangePhase::Menu => handle_menu(key, exchange_ui, state),
        ExchangePhase::InvokeTrialConfirm => handle_invoke_trial_confirm(key, exchange_ui, state),
        ExchangePhase::InvokeTrial => handle_invoke_trial(key, exchange_ui, state),
        ExchangePhase::InvokeTrialForfeitConfirm => {
            handle_invoke_trial_forfeit_confirm(key, exchange_ui)
        }
        ExchangePhase::ChronoSurge => handle_chrono_surge_select(key, exchange_ui, state),
        ExchangePhase::SigilsList => handle_sigils_list(key, exchange_ui, state),
        // Storm Sigils sub-phases — stubs for now, implemented in Tasks 5-6
        ExchangePhase::SigilUnlockConfirm
        | ExchangePhase::SigilInscribeConfirm
        | ExchangePhase::SigilRerollConfirm
        | ExchangePhase::SigilPick
        | ExchangePhase::SigilForfeitConfirm
        | ExchangePhase::SigilResult => {
            if key.code == KeyCode::Esc {
                exchange_ui.phase = ExchangePhase::SigilsList;
            }
            InputResult::Continue
        }
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
                    // Invoke Trial — show confirmation first
                    if state.stormglass >= INVOKE_TRIAL_COST {
                        // Guard: if all 10 challenge types are already pending, do nothing
                        if state.challenge_menu.challenges.len() >= 10 {
                            return InputResult::Continue;
                        }
                        exchange_ui.phase = ExchangePhase::InvokeTrialConfirm;
                    }
                }
                1 => {
                    // Chrono Surge — enter duration selection
                    exchange_ui.surge_selected = 0;
                    exchange_ui.phase = ExchangePhase::ChronoSurge;
                }
                2 => {
                    // Storm Sigils — enter sigils list
                    exchange_ui.sigil_selected_slot = 0;
                    exchange_ui.phase = ExchangePhase::SigilsList;
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
            exchange_ui.phase = ExchangePhase::Menu;
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
            exchange_ui.phase = ExchangePhase::InvokeTrial;
            InputResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            exchange_ui.phase = ExchangePhase::Menu;
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
            exchange_ui.phase = ExchangePhase::InvokeTrial;
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
            exchange_ui.phase = ExchangePhase::InvokeTrialForfeitConfirm;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
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
                // Locked slot (next unlockable) — go to unlock confirm
                if sigils.next_unlock_cost().is_some() {
                    exchange_ui.phase = ExchangePhase::SigilUnlockConfirm;
                }
            } else if sigils.sigils[slot].is_some() {
                // Inscribed slot — reroll (requires SG)
                if state.stormglass >= INSCRIBE_COST {
                    exchange_ui.sigil_target_slot = slot;
                    exchange_ui.phase = ExchangePhase::SigilRerollConfirm;
                }
            } else {
                // Empty slot — inscribe (requires SG)
                if state.stormglass >= INSCRIBE_COST {
                    exchange_ui.sigil_target_slot = slot;
                    exchange_ui.phase = ExchangePhase::SigilInscribeConfirm;
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            exchange_ui.phase = ExchangePhase::Menu;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
