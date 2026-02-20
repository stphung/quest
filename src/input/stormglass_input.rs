//! Input handling for the Stormglass Exchange overlay.

use crate::challenges::menu::create_challenge;
use crate::character::prestige_actions::grant_prestige_rank_no_reset;
use crate::core::game_state::GameState;
use crate::input::InputResult;
use crate::stormglass::spending::{chrono_surge_cost, generate_trial_options, prestige_rank_cost};
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
        ExchangePhase::InvokeTrial => handle_invoke_trial(key, exchange_ui, state),
        ExchangePhase::ChronoSurge => handle_chrono_surge_select(key, exchange_ui, state),
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
                    // Invoke Trial (50 SG)
                    if state.stormglass >= INVOKE_TRIAL_COST {
                        state.stormglass -= INVOKE_TRIAL_COST;
                        let mut rng = rand::rng();
                        exchange_ui.trial_options = generate_trial_options(&mut rng);
                        exchange_ui.trial_selected = 0;
                        exchange_ui.phase = ExchangePhase::InvokeTrial;
                    }
                }
                1 => {
                    // Prestige Rank purchase
                    let cost = prestige_rank_cost(state.prestige_rank);
                    if state.stormglass >= cost {
                        state.stormglass -= cost;
                        grant_prestige_rank_no_reset(state);
                        state.combat_state.add_log_entry(
                            format!(
                                "\u{26A1} Stormglass Exchange: Prestige rank increased to P{}!",
                                state.prestige_rank
                            ),
                            false,
                            true,
                        );
                        exchange_ui.close();
                        return InputResult::NeedsSave;
                    }
                }
                2 => {
                    // Chrono Surge — enter duration selection
                    exchange_ui.surge_selected = 0;
                    exchange_ui.phase = ExchangePhase::ChronoSurge;
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
            // Forfeit — SG already spent, just close
            exchange_ui.close();
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
