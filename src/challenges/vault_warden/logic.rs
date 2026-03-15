//! Vault Warden game logic.

use super::levels::{
    VaultWardenLevel, APPRENTICE_LEVELS, JOURNEYMAN_LEVELS, MASTER_LEVELS, NOVICE_LEVELS,
};
use super::types::*;
use crate::challenges::ActiveMinigame;
use rand::{Rng, RngExt};

/// Parse a Sokoban level string into game state.
pub fn parse_level(level: &VaultWardenLevel, difficulty: VaultWardenDifficulty) -> VaultWardenGame {
    let mut grid = Vec::new();
    let mut crate_positions = Vec::new();
    let mut goal_positions = Vec::new();
    let mut player_pos = (0, 0);

    let lines: Vec<&str> = level.data.lines().collect();
    let height = lines.len();
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    for (row, line) in lines.iter().enumerate() {
        let mut grid_row = vec![Cell::Wall; width];
        for (col, ch) in line.chars().enumerate() {
            match ch {
                '#' => grid_row[col] = Cell::Wall,
                ' ' | '-' => grid_row[col] = Cell::Floor,
                '$' => {
                    grid_row[col] = Cell::Floor;
                    crate_positions.push((row, col));
                }
                '.' => {
                    grid_row[col] = Cell::Floor;
                    goal_positions.push((row, col));
                }
                '*' => {
                    grid_row[col] = Cell::Floor;
                    crate_positions.push((row, col));
                    goal_positions.push((row, col));
                }
                '@' => {
                    grid_row[col] = Cell::Floor;
                    player_pos = (row, col);
                }
                '+' => {
                    grid_row[col] = Cell::Floor;
                    player_pos = (row, col);
                    goal_positions.push((row, col));
                }
                _ => grid_row[col] = Cell::Floor,
            }
        }
        grid.push(grid_row);
    }

    let attempts_max = difficulty.max_attempts();

    VaultWardenGame {
        difficulty,
        game_result: None,
        forfeit_pending: false,
        grid,
        width,
        height,
        player_pos,
        crate_positions: crate_positions.clone(),
        goal_positions,
        moves: 0,
        attempts_remaining: attempts_max,
        attempts_max,
        initial_player_pos: player_pos,
        initial_crate_positions: crate_positions,
    }
}

/// Start a new Vault Warden game with a random level for the given difficulty.
pub fn start_vault_warden_game<R: Rng>(
    difficulty: VaultWardenDifficulty,
    rng: &mut R,
) -> ActiveMinigame {
    let levels = match difficulty {
        VaultWardenDifficulty::Novice => NOVICE_LEVELS,
        VaultWardenDifficulty::Apprentice => APPRENTICE_LEVELS,
        VaultWardenDifficulty::Journeyman => JOURNEYMAN_LEVELS,
        VaultWardenDifficulty::Master => MASTER_LEVELS,
    };
    let idx = rng.random_range(0..levels.len());
    let game = parse_level(&levels[idx], difficulty);
    ActiveMinigame::VaultWarden(game)
}

/// Process player input.
pub fn process_input(game: &mut VaultWardenGame, input: VaultWardenInput) {
    if game.game_result.is_some() {
        return;
    }

    // Handle forfeit double-Esc pattern
    if game.forfeit_pending {
        match input {
            VaultWardenInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    VaultWardenResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return;
    }

    match input {
        VaultWardenInput::Up => try_move(game, -1, 0),
        VaultWardenInput::Down => try_move(game, 1, 0),
        VaultWardenInput::Left => try_move(game, 0, -1),
        VaultWardenInput::Right => try_move(game, 0, 1),
        VaultWardenInput::Restart => restart_level(game),
        VaultWardenInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                VaultWardenResult::Loss,
            );
        }
        VaultWardenInput::Other => {}
    }
}

/// Attempt to move the player in direction (dr, dc).
fn try_move(game: &mut VaultWardenGame, dr: i32, dc: i32) {
    let (pr, pc) = game.player_pos;
    let new_r = pr as i32 + dr;
    let new_c = pc as i32 + dc;

    if new_r < 0 || new_c < 0 {
        return;
    }
    let new_pos = (new_r as usize, new_c as usize);

    if new_pos.0 >= game.height || new_pos.1 >= game.width {
        return;
    }
    if game.grid[new_pos.0][new_pos.1] == Cell::Wall {
        return;
    }

    // Check if pushing a crate
    if game.has_crate_at(new_pos) {
        let crate_new_r = new_pos.0 as i32 + dr;
        let crate_new_c = new_pos.1 as i32 + dc;

        if crate_new_r < 0 || crate_new_c < 0 {
            return;
        }
        let crate_new_pos = (crate_new_r as usize, crate_new_c as usize);

        if crate_new_pos.0 >= game.height || crate_new_pos.1 >= game.width {
            return;
        }
        if game.grid[crate_new_pos.0][crate_new_pos.1] == Cell::Wall {
            return;
        }
        if game.has_crate_at(crate_new_pos) {
            return;
        }

        // Move the crate
        if let Some(idx) = game.crate_positions.iter().position(|&c| c == new_pos) {
            game.crate_positions[idx] = crate_new_pos;
        }
    }

    // Move player
    game.player_pos = new_pos;
    game.moves += 1;

    // Check win
    if game.crates_on_goals() == game.total_crates() {
        game.game_result = Some(VaultWardenResult::Win);
    }
}

/// Restart the level from scratch, consuming one attempt.
fn restart_level(game: &mut VaultWardenGame) {
    if game.attempts_remaining == 0 {
        game.game_result = Some(VaultWardenResult::Loss);
        return;
    }
    game.attempts_remaining -= 1;
    game.player_pos = game.initial_player_pos;
    game.crate_positions = game.initial_crate_positions.clone();
    game.moves = 0;
}

/// Check if a crate is deadlocked (in a non-goal corner or against a dead wall).
pub fn is_crate_deadlocked(game: &VaultWardenGame, crate_pos: (usize, usize)) -> bool {
    if game.is_goal(crate_pos) {
        return false;
    }

    let (r, c) = crate_pos;

    let up_wall = r == 0 || game.grid[r - 1][c] == Cell::Wall;
    let down_wall = r + 1 >= game.height || game.grid[r + 1][c] == Cell::Wall;
    let left_wall = c == 0 || game.grid[r][c - 1] == Cell::Wall;
    let right_wall = c + 1 >= game.width || game.grid[r][c + 1] == Cell::Wall;

    // Corner deadlock: wall on two perpendicular sides
    if (up_wall || down_wall) && (left_wall || right_wall) {
        return true;
    }

    // Wall-edge deadlock: crate against a straight wall with no goal along the segment
    if up_wall || down_wall {
        let no_goal_left = !has_goal_along_horizontal(game, r, c, -1, up_wall, down_wall);
        let no_goal_right = !has_goal_along_horizontal(game, r, c, 1, up_wall, down_wall);
        if no_goal_left && no_goal_right {
            return true;
        }
    }

    if left_wall || right_wall {
        let no_goal_up = !has_goal_along_vertical(game, r, c, -1, left_wall, right_wall);
        let no_goal_down = !has_goal_along_vertical(game, r, c, 1, left_wall, right_wall);
        if no_goal_up && no_goal_down {
            return true;
        }
    }

    false
}

/// Walk horizontally along a wall checking for goals.
fn has_goal_along_horizontal(
    game: &VaultWardenGame,
    row: usize,
    start_col: usize,
    dc: i32,
    wall_above: bool,
    wall_below: bool,
) -> bool {
    if game.is_goal((row, start_col)) {
        return true;
    }
    let mut c = start_col as i32 + dc;
    while c >= 0 && (c as usize) < game.width {
        let col = c as usize;
        if game.grid[row][col] == Cell::Wall {
            break;
        }
        // Check wall continuity
        let still_walled = if wall_above {
            row == 0 || game.grid[row - 1][col] == Cell::Wall
        } else if wall_below {
            row + 1 >= game.height || game.grid[row + 1][col] == Cell::Wall
        } else {
            false
        };
        if !still_walled {
            return true; // Wall ended — crate could escape
        }
        if game.is_goal((row, col)) {
            return true;
        }
        c += dc;
    }
    false
}

/// Walk vertically along a wall checking for goals.
fn has_goal_along_vertical(
    game: &VaultWardenGame,
    start_row: usize,
    col: usize,
    dr: i32,
    wall_left: bool,
    wall_right: bool,
) -> bool {
    if game.is_goal((start_row, col)) {
        return true;
    }
    let mut r = start_row as i32 + dr;
    while r >= 0 && (r as usize) < game.height {
        let row = r as usize;
        if game.grid[row][col] == Cell::Wall {
            break;
        }
        let still_walled = if wall_left {
            col == 0 || game.grid[row][col - 1] == Cell::Wall
        } else if wall_right {
            col + 1 >= game.width || game.grid[row][col + 1] == Cell::Wall
        } else {
            false
        };
        if !still_walled {
            return true;
        }
        if game.is_goal((row, col)) {
            return true;
        }
        r += dr;
    }
    false
}

// --- DifficultyInfo impl ---

impl crate::challenges::menu::DifficultyInfo for VaultWardenDifficulty {
    fn name(&self) -> &'static str {
        VaultWardenDifficulty::name(self)
    }

    fn reward(&self) -> crate::challenges::menu::ChallengeReward {
        match self {
            VaultWardenDifficulty::Novice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 400,
                fishing_ranks: 0,
            },
            VaultWardenDifficulty::Apprentice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 1_200,
                fishing_ranks: 0,
            },
            VaultWardenDifficulty::Journeyman => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 3_000,
                fishing_ranks: 0,
            },
            VaultWardenDifficulty::Master => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 2,
                stormglass: 6_000,
                fishing_ranks: 0,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("{} restart attempts", self.max_attempts()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_level() -> VaultWardenLevel {
        VaultWardenLevel {
            data: "\
#####
#   #
# $.#
#   #
#@###",
        }
    }

    #[test]
    fn test_parse_level() {
        let level = make_test_level();
        let game = parse_level(&level, VaultWardenDifficulty::Novice);
        assert_eq!(game.height, 5);
        assert_eq!(game.width, 5);
        assert_eq!(game.player_pos, (4, 1));
        assert_eq!(game.crate_positions, vec![(2, 2)]);
        assert_eq!(game.goal_positions, vec![(2, 3)]);
        assert_eq!(game.attempts_remaining, 5);
    }

    #[test]
    fn test_push_crate_to_win() {
        // Simple level: player left of crate, goal right of crate
        let level = VaultWardenLevel {
            data: "\
#####
#@$.#
#####",
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        process_input(&mut game, VaultWardenInput::Right);
        assert_eq!(game.player_pos, (1, 2));
        assert_eq!(game.crate_positions, vec![(1, 3)]);
        assert_eq!(game.moves, 1);
        assert_eq!(game.game_result, Some(VaultWardenResult::Win));
    }

    #[test]
    fn test_cant_push_into_wall() {
        let level = VaultWardenLevel {
            data: "\
#####
#@$##
#####",
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        let orig_crate = game.crate_positions[0];

        process_input(&mut game, VaultWardenInput::Right);
        assert_eq!(game.crate_positions[0], orig_crate);
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn test_cant_push_into_crate() {
        let level = VaultWardenLevel {
            data: "\
######
#@$$.#
######",
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        process_input(&mut game, VaultWardenInput::Right);
        // Can't push because crate at (1,2) would push into crate at (1,3)
        assert_eq!(game.player_pos, (1, 1));
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn test_move_without_push() {
        let level = VaultWardenLevel {
            data: "\
######
#@  .#
# $  #
######",
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        process_input(&mut game, VaultWardenInput::Right);
        assert_eq!(game.player_pos, (1, 2));
        assert_eq!(game.moves, 1);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_restart() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        let orig_player = game.player_pos;
        let orig_crates = game.crate_positions.clone();
        let orig_attempts = game.attempts_remaining;

        process_input(&mut game, VaultWardenInput::Up);
        assert_ne!(game.player_pos, orig_player);
        assert_eq!(game.moves, 1);

        process_input(&mut game, VaultWardenInput::Restart);
        assert_eq!(game.player_pos, orig_player);
        assert_eq!(game.crate_positions, orig_crates);
        assert_eq!(game.moves, 0);
        assert_eq!(game.attempts_remaining, orig_attempts - 1);
    }

    #[test]
    fn test_forfeit() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        process_input(&mut game, VaultWardenInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Any other key cancels
        process_input(&mut game, VaultWardenInput::Up);
        assert!(!game.forfeit_pending);

        // Double Esc confirms
        process_input(&mut game, VaultWardenInput::Forfeit);
        process_input(&mut game, VaultWardenInput::Forfeit);
        assert_eq!(game.game_result, Some(VaultWardenResult::Loss));
    }

    #[test]
    fn test_attempts_exhaustion_loss() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        assert_eq!(game.attempts_remaining, 5);

        for _ in 0..5 {
            process_input(&mut game, VaultWardenInput::Restart);
            assert!(game.game_result.is_none());
        }
        assert_eq!(game.attempts_remaining, 0);

        // Next restart triggers loss
        process_input(&mut game, VaultWardenInput::Restart);
        assert_eq!(game.game_result, Some(VaultWardenResult::Loss));
    }

    #[test]
    fn test_corner_deadlock() {
        let level = VaultWardenLevel {
            data: "\
#####
#  .#
# $ #
#@  #
#####",
        };
        let game = parse_level(&level, VaultWardenDifficulty::Novice);

        // Crate at (2,2) is not in a corner
        assert!(!is_crate_deadlocked(&game, (2, 2)));

        // Simulate crate in top-left corner (1,1) — no goal there
        let mut game2 = game.clone();
        game2.crate_positions = vec![(1, 1)];
        assert!(is_crate_deadlocked(&game2, (1, 1)));

        // Crate on goal is never deadlocked
        let mut game3 = game.clone();
        game3.crate_positions = vec![(1, 3)];
        assert!(!is_crate_deadlocked(&game3, (1, 3)));
    }

    #[test]
    fn test_parse_crate_on_goal() {
        let level = VaultWardenLevel {
            data: "\
####
#@ #
# *#
####",
        };
        let game = parse_level(&level, VaultWardenDifficulty::Novice);
        assert!(game.crate_positions.contains(&(2, 2)));
        assert!(game.goal_positions.contains(&(2, 2)));
    }

    #[test]
    fn test_parse_player_on_goal() {
        let level = VaultWardenLevel {
            data: "\
####
# $#
#+.#
####",
        };
        let game = parse_level(&level, VaultWardenDifficulty::Novice);
        assert_eq!(game.player_pos, (2, 1));
        assert!(game.goal_positions.contains(&(2, 1)));
    }

    #[test]
    fn test_all_level_pools_nonempty() {
        assert!(!NOVICE_LEVELS.is_empty());
        assert!(!APPRENTICE_LEVELS.is_empty());
        assert!(!JOURNEYMAN_LEVELS.is_empty());
        assert!(!MASTER_LEVELS.is_empty());
    }

    #[test]
    fn test_all_levels_parseable() {
        for (pool, diff) in [
            (NOVICE_LEVELS, VaultWardenDifficulty::Novice),
            (APPRENTICE_LEVELS, VaultWardenDifficulty::Apprentice),
            (JOURNEYMAN_LEVELS, VaultWardenDifficulty::Journeyman),
            (MASTER_LEVELS, VaultWardenDifficulty::Master),
        ] {
            for (i, level) in pool.iter().enumerate() {
                let game = parse_level(level, diff);
                assert!(
                    !game.crate_positions.is_empty(),
                    "{:?} level {} has no crates",
                    diff,
                    i
                );
                assert_eq!(
                    game.crate_positions.len(),
                    game.goal_positions.len(),
                    "{:?} level {} has mismatched crate/goal count",
                    diff,
                    i
                );
                assert!(
                    game.player_pos.0 < game.height && game.player_pos.1 < game.width,
                    "{:?} level {} has out-of-bounds player",
                    diff,
                    i
                );
            }
        }
    }

    #[test]
    fn test_ignore_input_after_game_over() {
        let level = VaultWardenLevel {
            data: "\
#####
#@$.#
#####",
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        process_input(&mut game, VaultWardenInput::Right); // Win
        assert_eq!(game.game_result, Some(VaultWardenResult::Win));

        let pos = game.player_pos;
        process_input(&mut game, VaultWardenInput::Left); // Should be ignored
        assert_eq!(game.player_pos, pos);
    }

    #[test]
    fn test_difficulty_info_rewards() {
        use crate::challenges::menu::DifficultyInfo;

        let novice = VaultWardenDifficulty::Novice.reward();
        assert_eq!(novice.stormglass, 400);
        assert_eq!(novice.prestige_ranks, 0);

        let master = VaultWardenDifficulty::Master.reward();
        assert_eq!(master.stormglass, 6_000);
        assert_eq!(master.prestige_ranks, 2);
    }
}
