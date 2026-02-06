//! Input handling for the Game screen.
//!
//! Extracts the input dispatch logic from main.rs into a clean priority chain.

use crate::challenges::chess::logic::{
    apply_game_result as apply_chess_result, process_input as process_chess_input, ChessInput,
};
use crate::challenges::go::{apply_go_result, process_input as process_go_input, GoInput};
use crate::challenges::gomoku::logic::{
    apply_game_result as apply_gomoku_result, process_input as process_gomoku_input, GomokuInput,
};
use crate::challenges::menu::{process_input as process_menu_input, MenuInput};
use crate::challenges::minesweeper::logic::{
    apply_game_result as apply_minesweeper_result, process_input as process_minesweeper_input,
    MinesweeperInput,
};
use crate::challenges::morris::logic::{
    apply_game_result as apply_morris_result, process_input as process_morris_input, MorrisInput,
};
use crate::challenges::rune::logic::{
    apply_game_result as apply_rune_result, process_input as process_rune_input, RuneInput,
};
use crate::challenges::ActiveMinigame;
use crate::character::prestige::{can_prestige, get_prestige_tier, perform_prestige};
use crate::core::game_state::GameState;
use crate::haven;
use crate::haven::Haven;
use crate::items;
use crate::utils::debug_menu::DebugMenu;
use crossterm::event::{KeyCode, KeyEvent};

/// Haven overlay state, shared between CharacterSelect and Game screens.
pub struct HavenUiState {
    pub showing: bool,
    pub selected_room: usize,
    pub confirming_build: bool,
}

impl HavenUiState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_room: 0,
            confirming_build: false,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_room = 0;
        self.confirming_build = false;
    }

    pub fn close(&mut self) {
        self.showing = false;
        self.confirming_build = false;
    }
}

/// Game-screen overlay state. At most one is active at a time.
pub enum GameOverlay {
    None,
    HavenDiscovery,
    PrestigeConfirm,
    VaultSelection {
        selected_index: usize,
        selected_slots: Vec<items::EquipmentSlot>,
    },
    OfflineWelcome {
        elapsed_seconds: i64,
        xp_gained: u64,
        level_before: u32,
        level_after: u32,
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
    /// Haven was modified along with state — save both.
    NeedsSaveAll,
}

/// Main dispatcher for Game screen input. Handles the priority chain.
pub fn handle_game_input(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
    debug_mode: bool,
) -> InputResult {
    // 0. Offline welcome overlay (any key dismisses)
    if matches!(overlay, GameOverlay::OfflineWelcome { .. }) {
        *overlay = GameOverlay::None;
        return InputResult::Continue;
    }

    // 1. Haven discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::HavenDiscovery) {
        return handle_haven_discovery(key, overlay);
    }

    // 2. Haven screen (blocks other input when open)
    if haven_ui.showing {
        return handle_haven(key, state, haven, haven_ui);
    }

    // 3. Vault item selection
    if matches!(overlay, GameOverlay::VaultSelection { .. }) {
        return handle_vault_selection(key, state, haven, overlay);
    }

    // 4. Prestige confirmation
    if matches!(overlay, GameOverlay::PrestigeConfirm) {
        return handle_prestige_confirm(key, state, haven, overlay);
    }

    // 5. Debug menu
    if debug_mode {
        if key.code == KeyCode::Char('`') {
            debug_menu.toggle();
            return InputResult::Continue;
        }
        if debug_menu.is_open {
            return handle_debug_menu(key, state, haven, overlay, debug_menu);
        }
    }

    // 6. Active minigame
    if state.active_minigame.is_some() {
        return handle_minigame(key, state);
    }

    // 7. Challenge menu
    if state.challenge_menu.is_open {
        return handle_challenge_menu(key, state);
    }

    // 8. Tab to open challenge menu
    if key.code == KeyCode::Tab && !state.challenge_menu.challenges.is_empty() {
        state.challenge_menu.open();
        return InputResult::Continue;
    }

    // 9. Base game input
    handle_base_game(key, state, haven, haven_ui, overlay)
}

fn handle_haven_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_haven(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
) -> InputResult {
    if haven_ui.confirming_build {
        match key.code {
            KeyCode::Enter => {
                let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                if let Some((_tier, p_spent)) =
                    haven::try_build_room(room, haven, &mut state.prestige_rank)
                {
                    // Haven saved via NeedsSaveAll (skipped in debug mode)
                    state.combat_state.add_log_entry(
                        format!(
                            "🏠 Built {} (spent {} Prestige Ranks)",
                            room.name(),
                            p_spent
                        ),
                        false,
                        true,
                    );
                    haven_ui.confirming_build = false;
                    return InputResult::NeedsSaveAll;
                }
                haven_ui.confirming_build = false;
            }
            KeyCode::Esc => {
                haven_ui.confirming_build = false;
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Up => {
                haven_ui.selected_room = haven_ui.selected_room.saturating_sub(1);
            }
            KeyCode::Down => {
                if haven_ui.selected_room + 1 < haven::HavenRoomId::ALL.len() {
                    haven_ui.selected_room += 1;
                }
            }
            KeyCode::Enter => {
                let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                if haven.can_build(room) && haven::can_afford(room, haven, state.prestige_rank) {
                    haven_ui.confirming_build = true;
                }
            }
            KeyCode::Esc => {
                haven_ui.close();
            }
            _ => {}
        }
    }
    InputResult::Continue
}

fn handle_vault_selection(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    overlay: &mut GameOverlay,
) -> InputResult {
    if let GameOverlay::VaultSelection {
        ref mut selected_index,
        ref mut selected_slots,
    } = overlay
    {
        match key.code {
            KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if *selected_index < 6 {
                    *selected_index += 1;
                }
            }
            KeyCode::Enter => {
                let slots = [
                    items::EquipmentSlot::Weapon,
                    items::EquipmentSlot::Armor,
                    items::EquipmentSlot::Helmet,
                    items::EquipmentSlot::Gloves,
                    items::EquipmentSlot::Boots,
                    items::EquipmentSlot::Amulet,
                    items::EquipmentSlot::Ring,
                ];
                let slot = slots[*selected_index];
                if state.equipment.get(slot).is_some() {
                    if let Some(pos) = selected_slots.iter().position(|s| *s == slot) {
                        selected_slots.remove(pos);
                    } else if selected_slots.len() < haven.vault_tier() as usize {
                        selected_slots.push(slot);
                    }
                }
            }
            KeyCode::Char(' ') => {
                crate::character::prestige::perform_prestige_with_vault(state, selected_slots);
                *overlay = GameOverlay::None;
                state.combat_state.add_log_entry(
                    format!(
                        "Prestiged to {}! (Vault preserved items)",
                        get_prestige_tier(state.prestige_rank).name
                    ),
                    false,
                    true,
                );
                return InputResult::NeedsSave;
            }
            KeyCode::Esc => {
                *overlay = GameOverlay::None;
            }
            _ => {}
        }
    }
    InputResult::Continue
}

fn handle_prestige_confirm(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    overlay: &mut GameOverlay,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if haven.vault_tier() > 0 {
                *overlay = GameOverlay::VaultSelection {
                    selected_index: 0,
                    selected_slots: Vec::new(),
                };
            } else {
                perform_prestige(state);
                *overlay = GameOverlay::None;
                state.combat_state.add_log_entry(
                    format!(
                        "Prestiged to {}!",
                        get_prestige_tier(state.prestige_rank).name
                    ),
                    false,
                    true,
                );
                return InputResult::NeedsSave;
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            *overlay = GameOverlay::None;
        }
        _ => {}
    }
    InputResult::Continue
}

fn handle_debug_menu(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
) -> InputResult {
    match key.code {
        KeyCode::Up => debug_menu.navigate_up(),
        KeyCode::Down => debug_menu.navigate_down(),
        KeyCode::Enter => {
            let msg = debug_menu.trigger_selected(state, haven);
            state
                .combat_state
                .add_log_entry(format!("[DEBUG] {}", msg), false, true);
            // Show Haven discovery modal if just discovered (no save in debug mode)
            if msg == "Haven discovered!" {
                *overlay = GameOverlay::HavenDiscovery;
            }
        }
        KeyCode::Esc => debug_menu.close(),
        _ => {}
    }
    InputResult::Continue
}

fn handle_minigame(key: KeyEvent, state: &mut GameState) -> InputResult {
    if let Some(ref mut minigame) = state.active_minigame {
        match minigame {
            ActiveMinigame::Rune(rune_game) => {
                if rune_game.game_result.is_some() {
                    apply_rune_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Left => RuneInput::Left,
                    KeyCode::Right => RuneInput::Right,
                    KeyCode::Up => RuneInput::Up,
                    KeyCode::Down => RuneInput::Down,
                    KeyCode::Enter => RuneInput::Submit,
                    KeyCode::Char('f') | KeyCode::Char('F') => RuneInput::ClearGuess,
                    KeyCode::Esc => RuneInput::Forfeit,
                    _ => RuneInput::Other,
                };
                let mut rng = rand::thread_rng();
                process_rune_input(rune_game, input, &mut rng);
            }
            ActiveMinigame::Minesweeper(minesweeper_game) => {
                if minesweeper_game.game_result.is_some() {
                    apply_minesweeper_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => MinesweeperInput::Up,
                    KeyCode::Down => MinesweeperInput::Down,
                    KeyCode::Left => MinesweeperInput::Left,
                    KeyCode::Right => MinesweeperInput::Right,
                    KeyCode::Enter => MinesweeperInput::Reveal,
                    KeyCode::Char('f') | KeyCode::Char('F') => MinesweeperInput::ToggleFlag,
                    KeyCode::Esc => MinesweeperInput::Forfeit,
                    _ => MinesweeperInput::Other,
                };
                let mut rng = rand::thread_rng();
                process_minesweeper_input(minesweeper_game, input, &mut rng);
            }
            ActiveMinigame::Gomoku(gomoku_game) => {
                if gomoku_game.game_result.is_some() {
                    apply_gomoku_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => GomokuInput::Up,
                    KeyCode::Down => GomokuInput::Down,
                    KeyCode::Left => GomokuInput::Left,
                    KeyCode::Right => GomokuInput::Right,
                    KeyCode::Enter => GomokuInput::PlaceStone,
                    KeyCode::Esc => GomokuInput::Forfeit,
                    _ => GomokuInput::Other,
                };
                process_gomoku_input(gomoku_game, input);
            }
            ActiveMinigame::Chess(chess_game) => {
                if chess_game.game_result.is_some() {
                    apply_chess_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => ChessInput::Up,
                    KeyCode::Down => ChessInput::Down,
                    KeyCode::Left => ChessInput::Left,
                    KeyCode::Right => ChessInput::Right,
                    KeyCode::Enter => ChessInput::Select,
                    KeyCode::Esc => ChessInput::Cancel,
                    _ => ChessInput::Other,
                };
                process_chess_input(chess_game, input);
            }
            ActiveMinigame::Morris(morris_game) => {
                if morris_game.game_result.is_some() {
                    apply_morris_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => MorrisInput::Up,
                    KeyCode::Down => MorrisInput::Down,
                    KeyCode::Left => MorrisInput::Left,
                    KeyCode::Right => MorrisInput::Right,
                    KeyCode::Enter => MorrisInput::Select,
                    KeyCode::Esc => MorrisInput::Cancel,
                    _ => MorrisInput::Other,
                };
                process_morris_input(morris_game, input);
            }
            ActiveMinigame::Go(go_game) => {
                if go_game.game_result.is_some() {
                    apply_go_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => GoInput::Up,
                    KeyCode::Down => GoInput::Down,
                    KeyCode::Left => GoInput::Left,
                    KeyCode::Right => GoInput::Right,
                    KeyCode::Enter => GoInput::PlaceStone,
                    KeyCode::Char('p') | KeyCode::Char('P') => GoInput::Pass,
                    KeyCode::Esc => GoInput::Forfeit,
                    _ => GoInput::Other,
                };
                process_go_input(go_game, input);
            }
        }
    }
    InputResult::Continue
}

fn handle_challenge_menu(key: KeyEvent, state: &mut GameState) -> InputResult {
    let input = match key.code {
        KeyCode::Up => MenuInput::Up,
        KeyCode::Down => MenuInput::Down,
        KeyCode::Enter => MenuInput::Select,
        KeyCode::Char('d') | KeyCode::Char('D') => MenuInput::Decline,
        KeyCode::Esc | KeyCode::Tab => MenuInput::Cancel,
        _ => MenuInput::Other,
    };
    process_menu_input(state, input);
    InputResult::Continue
}

fn handle_base_game(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    haven_ui: &mut HavenUiState,
    overlay: &mut GameOverlay,
) -> InputResult {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => InputResult::QuitToSelect,
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if can_prestige(state) {
                *overlay = GameOverlay::PrestigeConfirm;
            }
            InputResult::Continue
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            if haven.discovered {
                haven_ui.open();
            }
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
