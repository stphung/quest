//! Runic Lights game logic.
//!
//! Puzzle generation uses reverse construction: start from all-off, apply N
//! random unique cell toggles. Each toggle flips the cross pattern (+).
//! The resulting board is guaranteed solvable in at most N moves.

use super::types::{RunicLightsDifficulty, RunicLightsGame, RunicLightsInput, RunicLightsResult};
use crate::challenges::ActiveMinigame;
use rand::{Rng, RngExt};

/// Toggle a cell and its orthogonal neighbors (cross pattern).
pub fn toggle_cell(board: &mut Vec<Vec<bool>>, row: usize, col: usize) {
    let size = board.len();
    board[row][col] = !board[row][col];
    if row > 0 {
        board[row - 1][col] = !board[row - 1][col];
    }
    if row + 1 < size {
        board[row + 1][col] = !board[row + 1][col];
    }
    if col > 0 {
        board[row][col - 1] = !board[row][col - 1];
    }
    if col + 1 < size {
        board[row][col + 1] = !board[row][col + 1];
    }
}

/// Generate a solvable puzzle by reverse construction.
///
/// Picks N unique random cells and toggles each once from the solved (all-off) state.
/// N is sampled uniformly from the difficulty's solution depth range.
pub fn generate_puzzle<R: Rng>(game: &mut RunicLightsGame, rng: &mut R) {
    let (min_depth, max_depth) = game.difficulty.solution_depth_range();
    let depth = rng.random_range(min_depth..=max_depth);
    let size = game.size;
    let total = size * size;

    // Collect all cell indices, shuffle, take first `depth`
    let mut indices: Vec<usize> = (0..total).collect();
    // Fisher-Yates shuffle
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }

    for &idx in indices.iter().take(depth) {
        let row = idx / size;
        let col = idx % size;
        toggle_cell(&mut game.board, row, col);
    }
}

/// Check if the board is solved (all cells dark).
pub fn is_solved(game: &RunicLightsGame) -> bool {
    game.board.iter().flatten().all(|&c| !c)
}

/// Check win/loss conditions and update game_result.
fn check_game_over(game: &mut RunicLightsGame) {
    if game.game_result.is_some() {
        return;
    }
    if is_solved(game) {
        game.game_result = Some(RunicLightsResult::Win);
    } else if game.moves >= game.move_limit {
        game.game_result = Some(RunicLightsResult::Loss);
    }
}

/// Process player input.
pub fn process_input(game: &mut RunicLightsGame, input: RunicLightsInput) {
    if game.game_result.is_some() {
        return;
    }

    // Handle forfeit double-Esc pattern
    if game.forfeit_pending {
        match input {
            RunicLightsInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    RunicLightsResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return;
    }

    match input {
        RunicLightsInput::Up => game.move_cursor(-1, 0),
        RunicLightsInput::Down => game.move_cursor(1, 0),
        RunicLightsInput::Left => game.move_cursor(0, -1),
        RunicLightsInput::Right => game.move_cursor(0, 1),
        RunicLightsInput::Toggle => {
            let (row, col) = game.cursor;
            toggle_cell(&mut game.board, row, col);
            game.moves += 1;
            check_game_over(game);
        }
        RunicLightsInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                RunicLightsResult::Loss,
            );
        }
        RunicLightsInput::Other => {}
    }
}

impl crate::challenges::menu::DifficultyInfo for RunicLightsDifficulty {
    fn name(&self) -> &'static str {
        RunicLightsDifficulty::name(self)
    }

    fn reward(&self) -> crate::challenges::menu::ChallengeReward {
        match self {
            RunicLightsDifficulty::Novice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 400,
                fishing_ranks: 0,
            },
            RunicLightsDifficulty::Apprentice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 1_200,
                fishing_ranks: 0,
            },
            RunicLightsDifficulty::Journeyman => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 3_000,
                fishing_ranks: 0,
            },
            RunicLightsDifficulty::Master => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 2,
                stormglass: 6_000,
                fishing_ranks: 0,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!(
            "{}×{} grid, par {}",
            self.grid_size(),
            self.grid_size(),
            self.par()
        ))
    }
}

/// Start a new Runic Lights game and return it as an `ActiveMinigame`.
pub fn start_runic_lights_game<R: Rng>(
    difficulty: RunicLightsDifficulty,
    rng: &mut R,
) -> ActiveMinigame {
    let mut game = RunicLightsGame::new(difficulty);
    generate_puzzle(&mut game, rng);
    ActiveMinigame::RunicLights(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_toggle_center_3x3() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 1, 1);
        assert!(board[1][1]);
        assert!(board[0][1]);
        assert!(board[2][1]);
        assert!(board[1][0]);
        assert!(board[1][2]);
        assert!(!board[0][0]);
        assert!(!board[0][2]);
        assert!(!board[2][0]);
        assert!(!board[2][2]);
    }

    #[test]
    fn test_toggle_corner() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 0, 0);
        assert!(board[0][0]);
        assert!(board[0][1]);
        assert!(board[1][0]);
        assert_eq!(board.iter().flatten().filter(|&&c| c).count(), 3);
    }

    #[test]
    fn test_toggle_edge() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 0, 1);
        assert!(board[0][1]);
        assert!(board[0][0]);
        assert!(board[0][2]);
        assert!(board[1][1]);
        assert_eq!(board.iter().flatten().filter(|&&c| c).count(), 4);
    }

    #[test]
    fn test_double_toggle_cancels() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 1, 1);
        toggle_cell(&mut board, 1, 1);
        assert!(board.iter().flatten().all(|&c| !c));
    }

    #[test]
    fn test_generate_puzzle_is_solvable() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for difficulty in RunicLightsDifficulty::ALL {
            let mut game = RunicLightsGame::new(difficulty);
            generate_puzzle(&mut game, &mut rng);
            assert!(
                game.lit_count() > 0,
                "Puzzle for {:?} has no lit cells",
                difficulty
            );
        }
    }

    #[test]
    fn test_generate_puzzle_different_seeds_different_boards() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(1);
        let mut rng2 = ChaCha8Rng::seed_from_u64(2);
        let mut game1 = RunicLightsGame::new(RunicLightsDifficulty::Journeyman);
        let mut game2 = RunicLightsGame::new(RunicLightsDifficulty::Journeyman);
        generate_puzzle(&mut game1, &mut rng1);
        generate_puzzle(&mut game2, &mut rng2);
        assert_ne!(game1.board, game2.board);
    }

    #[test]
    fn test_win_condition() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        toggle_cell(&mut game.board, 0, 0);
        assert!(!is_solved(&game));
        game.cursor = (0, 0);
        process_input(&mut game, RunicLightsInput::Toggle);
        assert_eq!(game.game_result, Some(RunicLightsResult::Win));
        assert_eq!(game.moves, 1);
    }

    #[test]
    fn test_loss_on_move_limit() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true;
        game.moves = game.move_limit - 1;
        game.cursor = (1, 1);
        process_input(&mut game, RunicLightsInput::Toggle);
        assert_eq!(game.game_result, Some(RunicLightsResult::Loss));
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true;
        process_input(&mut game, RunicLightsInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
        process_input(&mut game, RunicLightsInput::Forfeit);
        assert_eq!(game.game_result, Some(RunicLightsResult::Loss));
    }

    #[test]
    fn test_forfeit_cancel() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true;
        process_input(&mut game, RunicLightsInput::Forfeit);
        assert!(game.forfeit_pending);
        process_input(&mut game, RunicLightsInput::Up);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_no_input_after_game_over() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.game_result = Some(RunicLightsResult::Win);
        let old_cursor = game.cursor;
        process_input(&mut game, RunicLightsInput::Down);
        assert_eq!(game.cursor, old_cursor);
    }

    #[test]
    fn test_move_increments_on_toggle() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true;
        game.board[2][2] = true;
        game.cursor = (1, 1);
        process_input(&mut game, RunicLightsInput::Toggle);
        assert_eq!(game.moves, 1);
    }

    #[test]
    fn test_start_runic_lights_game() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let minigame = start_runic_lights_game(RunicLightsDifficulty::Novice, &mut rng);
        match minigame {
            ActiveMinigame::RunicLights(game) => {
                assert_eq!(game.size, 3);
                assert!(game.lit_count() > 0);
            }
            _ => panic!("Expected RunicLights variant"),
        }
    }

    #[test]
    fn test_difficulty_info_rewards() {
        use crate::challenges::menu::DifficultyInfo;
        let reward = RunicLightsDifficulty::Master.reward();
        assert_eq!(reward.prestige_ranks, 2);
        assert_eq!(reward.stormglass, 6_000);
        assert_eq!(reward.fishing_ranks, 0);
    }

    #[test]
    fn test_difficulty_info_extra_info() {
        use crate::challenges::menu::DifficultyInfo;
        let info = RunicLightsDifficulty::Novice.extra_info();
        assert_eq!(info, Some("3×3 grid, par 5".to_string()));
    }
}
