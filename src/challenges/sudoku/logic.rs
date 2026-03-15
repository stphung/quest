use crate::challenges::menu::{ChallengeReward, DifficultyInfo};

use super::types::{SudokuDifficulty, SudokuGame, SudokuResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SudokuInput {
    Up,
    Down,
    Left,
    Right,
    Place(u8),
    Clear,
    Forfeit,
    Other,
}

pub fn process_sudoku_input(game: &mut SudokuGame, input: SudokuInput) -> bool {
    // Handle forfeit confirmation first
    if game.forfeit_pending {
        match input {
            SudokuInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    SudokuResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return true;
    }

    match input {
        SudokuInput::Up => {
            game.cursor_row = if game.cursor_row == 0 {
                8
            } else {
                game.cursor_row - 1
            };
        }
        SudokuInput::Down => {
            game.cursor_row = (game.cursor_row + 1) % 9;
        }
        SudokuInput::Left => {
            game.cursor_col = if game.cursor_col == 0 {
                8
            } else {
                game.cursor_col - 1
            };
        }
        SudokuInput::Right => {
            game.cursor_col = (game.cursor_col + 1) % 9;
        }
        SudokuInput::Place(digit) => {
            if !game.given[game.cursor_row][game.cursor_col] {
                game.board[game.cursor_row][game.cursor_col] = digit;
                update_conflicts(game);
                check_win(game);
            }
        }
        SudokuInput::Clear => {
            if !game.given[game.cursor_row][game.cursor_col] {
                game.board[game.cursor_row][game.cursor_col] = 0;
                update_conflicts(game);
            }
        }
        SudokuInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                SudokuResult::Loss,
            );
        }
        SudokuInput::Other => {}
    }
    true
}

/// Recalculate all conflict flags for the entire board.
fn update_conflicts(game: &mut SudokuGame) {
    game.conflicts = [[false; 9]; 9];

    for r in 0..9 {
        for c in 0..9 {
            let val = game.board[r][c];
            if val == 0 {
                continue;
            }

            // Check row for duplicates
            for c2 in 0..9 {
                if c2 != c && game.board[r][c2] == val {
                    game.conflicts[r][c] = true;
                    game.conflicts[r][c2] = true;
                }
            }

            // Check column for duplicates
            for r2 in 0..9 {
                if r2 != r && game.board[r2][c] == val {
                    game.conflicts[r][c] = true;
                    game.conflicts[r2][c] = true;
                }
            }

            // Check 3x3 box for duplicates
            let box_row = (r / 3) * 3;
            let box_col = (c / 3) * 3;
            for r2 in box_row..box_row + 3 {
                for c2 in box_col..box_col + 3 {
                    if (r2, c2) != (r, c) && game.board[r2][c2] == val {
                        game.conflicts[r][c] = true;
                        game.conflicts[r2][c2] = true;
                    }
                }
            }
        }
    }
}

/// Check if the board is complete and matches the solution.
fn check_win(game: &mut SudokuGame) {
    if game.board == game.solution {
        game.game_result = Some(SudokuResult::Win);
    }
}

/// Count how many cells are currently filled (non-zero).
pub fn filled_count(game: &SudokuGame) -> usize {
    game.board.iter().flatten().filter(|&&v| v != 0).count()
}

/// Count how many cells were given (pre-filled).
pub fn given_count(game: &SudokuGame) -> usize {
    game.given.iter().flatten().filter(|&&v| v).count()
}

impl DifficultyInfo for SudokuDifficulty {
    fn name(&self) -> &'static str {
        SudokuDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            SudokuDifficulty::Novice => ChallengeReward {
                stormglass: 400,
                ..Default::default()
            },
            SudokuDifficulty::Apprentice => ChallengeReward {
                stormglass: 1_200,
                ..Default::default()
            },
            SudokuDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                stormglass: 3_000,
                ..Default::default()
            },
            SudokuDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                stormglass: 6_000,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        let (min, max) = self.cells_to_remove_range();
        let given_max = 81 - min;
        let given_min = 81 - max;
        Some(format!("{}-{} sigils given", given_min, given_max))
    }
}

impl_apply_game_result! {
    variant: Sudoku;
    result_body: |result, _state, _reward| {
        match result {
            SudokuResult::Win => (true, ""),
            SudokuResult::Loss => (false, "The sigil matrix fractures. The pattern is lost."),
        }
    }
    game_type: crate::achievements::MinigameType::SigilMatrix;
    icon: "\u{2B21}";
    win_message: "The sigil matrix hums with power! Pattern complete.";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::sudoku::types::SudokuGame;

    fn make_test_game() -> SudokuGame {
        let solution = [
            [5, 3, 4, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ];
        let mut board = solution;
        let mut given = [[true; 9]; 9];
        board[0][2] = 0;
        given[0][2] = false;
        board[1][1] = 0;
        given[1][1] = false;
        board[4][4] = 0;
        given[4][4] = false;
        SudokuGame::new(SudokuDifficulty::Novice, board, solution, given)
    }

    #[test]
    fn test_cursor_wrapping() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        process_sudoku_input(&mut game, SudokuInput::Up);
        assert_eq!(game.cursor_row, 8);
        game.cursor_col = 8;
        process_sudoku_input(&mut game, SudokuInput::Right);
        assert_eq!(game.cursor_col, 0);
    }

    #[test]
    fn test_cannot_modify_given_cell() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 0;
        process_sudoku_input(&mut game, SudokuInput::Place(9));
        assert_eq!(game.board[0][0], 5);
    }

    #[test]
    fn test_place_and_clear() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 2;
        process_sudoku_input(&mut game, SudokuInput::Place(4));
        assert_eq!(game.board[0][2], 4);
        process_sudoku_input(&mut game, SudokuInput::Clear);
        assert_eq!(game.board[0][2], 0);
    }

    #[test]
    fn test_conflict_detection() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 2;
        process_sudoku_input(&mut game, SudokuInput::Place(5));
        assert!(game.conflicts[0][2]);
        assert!(game.conflicts[0][0]);
    }

    #[test]
    fn test_win_condition() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 2;
        process_sudoku_input(&mut game, SudokuInput::Place(4));
        assert!(game.game_result.is_none());
        game.cursor_row = 1;
        game.cursor_col = 1;
        process_sudoku_input(&mut game, SudokuInput::Place(7));
        assert!(game.game_result.is_none());
        game.cursor_row = 4;
        game.cursor_col = 4;
        process_sudoku_input(&mut game, SudokuInput::Place(5));
        assert_eq!(game.game_result, Some(SudokuResult::Win));
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = make_test_game();
        process_sudoku_input(&mut game, SudokuInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
        process_sudoku_input(&mut game, SudokuInput::Up);
        assert!(!game.forfeit_pending);
        process_sudoku_input(&mut game, SudokuInput::Forfeit);
        assert!(game.forfeit_pending);
        process_sudoku_input(&mut game, SudokuInput::Forfeit);
        assert_eq!(game.game_result, Some(SudokuResult::Loss));
    }

    #[test]
    fn test_difficulty_rewards() {
        assert_eq!(SudokuDifficulty::Novice.reward().stormglass, 400);
        assert_eq!(SudokuDifficulty::Novice.reward().prestige_ranks, 0);
        assert_eq!(SudokuDifficulty::Apprentice.reward().stormglass, 1_200);
        assert_eq!(SudokuDifficulty::Journeyman.reward().stormglass, 3_000);
        assert_eq!(SudokuDifficulty::Journeyman.reward().prestige_ranks, 1);
        assert_eq!(SudokuDifficulty::Master.reward().stormglass, 6_000);
        assert_eq!(SudokuDifficulty::Master.reward().prestige_ranks, 2);
    }
}
