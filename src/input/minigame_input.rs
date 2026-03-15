//! Minigame input handling for all 10 challenge types.

use super::InputResult;
use crate::challenges::chess::logic::{
    apply_game_result as apply_chess_result, process_input as process_chess_input, ChessInput,
};
use crate::challenges::flappy::logic::{
    apply_game_result as apply_flappy_result, process_input as process_flappy_input,
    FlappyBirdInput,
};
use crate::challenges::go::{apply_go_result, process_input as process_go_input, GoInput};
use crate::challenges::gomoku::logic::{
    apply_game_result as apply_gomoku_result, process_input as process_gomoku_input, GomokuInput,
};
use crate::challenges::jezzball::logic::{
    apply_game_result as apply_jezzball_result, process_input as process_jezzball_input,
    JezzballInput,
};
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
use crate::challenges::runic_shift::logic::{
    apply_game_result as apply_runic_shift_result, process_input as process_runic_shift_input,
    RunicShiftInput,
};
use crate::challenges::snake::logic::{
    apply_game_result as apply_snake_result, process_input as process_snake_input, SnakeInput,
};
use crate::challenges::sudoku::{
    apply_game_result as apply_sudoku_result, process_sudoku_input, SudokuInput,
};
use crate::challenges::{ActiveMinigame, MinigameWinInfo};
use crate::core::game_state::GameState;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Duration to ignore input after game-over screen appears.
pub(super) const GAME_OVER_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(2);

/// Convert a minigame win into a save-with-event result, or Continue for losses.
fn result_for_challenge(win_info: &Option<MinigameWinInfo>) -> InputResult {
    match win_info {
        Some(info) => InputResult::NeedsSaveWithEvent(crate::history::SaveEvent::ChallengeWon(
            format!("{:?}", info.game_type),
            format!("{:?}", info.difficulty),
        )),
        None => InputResult::Continue,
    }
}

pub(super) fn handle_minigame(key: KeyEvent, state: &mut GameState) -> InputResult {
    if let Some(ref minigame) = state.active_minigame {
        if minigame.has_game_result() {
            let now = std::time::Instant::now();
            match state.game_over_shown_at {
                None => {
                    // First keypress after game-over: start cooldown, swallow input
                    state.game_over_shown_at = Some(now);
                    return InputResult::Continue;
                }
                Some(shown_at) if now.duration_since(shown_at) < GAME_OVER_COOLDOWN => {
                    // Still in cooldown: swallow input
                    return InputResult::Continue;
                }
                _ => {
                    // Cooldown expired: dismiss game-over
                    state.game_over_shown_at = None;
                }
            }
        }
    }

    if let Some(ref mut minigame) = state.active_minigame {
        match minigame {
            ActiveMinigame::Rune(rune_game) => {
                if rune_game.game_result.is_some() {
                    state.last_minigame_win = apply_rune_result(state);
                    return result_for_challenge(&state.last_minigame_win);
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
                let mut rng = rand::rng();
                process_rune_input(rune_game, input, &mut rng);
            }
            ActiveMinigame::Minesweeper(minesweeper_game) => {
                if minesweeper_game.game_result.is_some() {
                    state.last_minigame_win = apply_minesweeper_result(state);
                    return result_for_challenge(&state.last_minigame_win);
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
                let mut rng = rand::rng();
                process_minesweeper_input(minesweeper_game, input, &mut rng);
            }
            ActiveMinigame::Gomoku(gomoku_game) => {
                if gomoku_game.game_result.is_some() {
                    state.last_minigame_win = apply_gomoku_result(state);
                    return result_for_challenge(&state.last_minigame_win);
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
                    state.last_minigame_win = apply_chess_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Up => ChessInput::Up,
                    KeyCode::Down => ChessInput::Down,
                    KeyCode::Left => ChessInput::Left,
                    KeyCode::Right => ChessInput::Right,
                    KeyCode::Enter => ChessInput::Select,
                    KeyCode::Esc => ChessInput::Forfeit,
                    _ => ChessInput::Other,
                };
                process_chess_input(chess_game, input);
            }
            ActiveMinigame::Morris(morris_game) => {
                if morris_game.game_result.is_some() {
                    state.last_minigame_win = apply_morris_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Up => MorrisInput::Up,
                    KeyCode::Down => MorrisInput::Down,
                    KeyCode::Left => MorrisInput::Left,
                    KeyCode::Right => MorrisInput::Right,
                    KeyCode::Enter => MorrisInput::Select,
                    KeyCode::Esc => MorrisInput::Forfeit,
                    _ => MorrisInput::Other,
                };
                process_morris_input(morris_game, input);
            }
            ActiveMinigame::Go(go_game) => {
                if go_game.game_result.is_some() {
                    state.last_minigame_win = apply_go_result(state);
                    return result_for_challenge(&state.last_minigame_win);
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
            ActiveMinigame::FlappyBird(flappy_game) => {
                if flappy_game.game_result.is_some() {
                    state.last_minigame_win = apply_flappy_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Char(' ') | KeyCode::Up => FlappyBirdInput::Flap,
                    KeyCode::Esc => FlappyBirdInput::Forfeit,
                    _ => FlappyBirdInput::Other,
                };
                process_flappy_input(flappy_game, input);
            }
            ActiveMinigame::Jezzball(jezzball_game) => {
                if jezzball_game.game_result.is_some() {
                    state.last_minigame_win = apply_jezzball_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Up => JezzballInput::Up,
                    KeyCode::Down => JezzballInput::Down,
                    KeyCode::Left => JezzballInput::Left,
                    KeyCode::Right => JezzballInput::Right,
                    KeyCode::Enter | KeyCode::Char(' ') => JezzballInput::Select,
                    KeyCode::Char('x') | KeyCode::Char('X') => JezzballInput::ToggleOrientation,
                    KeyCode::Esc => JezzballInput::Forfeit,
                    _ => JezzballInput::Other,
                };
                process_jezzball_input(jezzball_game, input);
            }
            ActiveMinigame::Snake(snake_game) => {
                if snake_game.game_result.is_some() {
                    state.last_minigame_win = apply_snake_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Up => SnakeInput::Up,
                    KeyCode::Down => SnakeInput::Down,
                    KeyCode::Left => SnakeInput::Left,
                    KeyCode::Right => SnakeInput::Right,
                    KeyCode::Char(' ') => SnakeInput::Select,
                    KeyCode::Esc => SnakeInput::Forfeit,
                    _ => SnakeInput::Other,
                };
                process_snake_input(snake_game, input);
            }
            ActiveMinigame::RunicShift(runic_shift_game) => {
                if runic_shift_game.game_result.is_some() {
                    state.last_minigame_win = apply_runic_shift_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Up => RunicShiftInput::Up,
                    KeyCode::Down => RunicShiftInput::Down,
                    KeyCode::Left => RunicShiftInput::Left,
                    KeyCode::Right => RunicShiftInput::Right,
                    KeyCode::Char(' ') => RunicShiftInput::Swap,
                    KeyCode::Char('r') | KeyCode::Char('R') => RunicShiftInput::ManualRise,
                    KeyCode::Esc => RunicShiftInput::Forfeit,
                    _ => RunicShiftInput::Other,
                };
                process_runic_shift_input(runic_shift_game, input);
            }
            ActiveMinigame::Sudoku(sudoku_game) => {
                if sudoku_game.game_result.is_some() {
                    state.last_minigame_win = apply_sudoku_result(state);
                    return result_for_challenge(&state.last_minigame_win);
                }
                let input = match key.code {
                    KeyCode::Up => SudokuInput::Up,
                    KeyCode::Down => SudokuInput::Down,
                    KeyCode::Left => SudokuInput::Left,
                    KeyCode::Right => SudokuInput::Right,
                    KeyCode::Char(c @ '1'..='9') => SudokuInput::Place(c as u8 - b'0'),
                    KeyCode::Backspace | KeyCode::Delete => SudokuInput::Clear,
                    KeyCode::Esc => SudokuInput::Forfeit,
                    _ => SudokuInput::Other,
                };
                process_sudoku_input(sudoku_game, input);
            }
        }
    }
    InputResult::Continue
}
