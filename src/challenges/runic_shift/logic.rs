//! Runic Shift game logic.

use super::types::*;
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;

/// UI-agnostic input actions for Runic Shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicShiftInput {
    Up,
    Down,
    Left,
    Right,
    Swap,       // Space
    ManualRise, // R
    Forfeit,    // Esc
    Other,
}

/// Start a new Runic Shift game at the given difficulty.
pub fn start_runic_shift_game(difficulty: RunicShiftDifficulty) -> ActiveMinigame {
    ActiveMinigame::RunicShift(RunicShiftGame::new(difficulty))
}

/// Process player input.
pub fn process_input(game: &mut RunicShiftGame, input: RunicShiftInput) {
    if game.game_result.is_some() {
        return;
    }

    if game.waiting_to_start {
        if matches!(input, RunicShiftInput::Swap) {
            game.waiting_to_start = false;
        }
        return;
    }

    match input {
        RunicShiftInput::Up => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.cursor_row > 0 {
                game.cursor_row -= 1;
            }
        }
        RunicShiftInput::Down => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.cursor_row + 1 < GRID_ROWS {
                game.cursor_row += 1;
            }
        }
        RunicShiftInput::Left => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.cursor_col > 0 {
                game.cursor_col -= 1;
            }
        }
        RunicShiftInput::Right => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.cursor_col + 1 < GRID_COLS - 1 {
                game.cursor_col += 1;
            }
        }
        RunicShiftInput::Swap => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.has_active_animation() {
                // Ignore swaps while animations are playing.
            } else {
                attempt_swap(game);
            }
        }
        RunicShiftInput::ManualRise => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.clear_timer_ms == 0 && !game.has_active_animation() {
                game.rise_accumulated_ms = 0;
                trigger_rise(game);
            }
        }
        RunicShiftInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(RunicShiftResult::Loss);
            } else {
                game.forfeit_pending = true;
            }
        }
        RunicShiftInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            }
        }
    }
}

/// Advance Runic Shift simulation.
///
/// `dt_ms` is elapsed time in milliseconds since the previous call.
pub fn tick_runic_shift(game: &mut RunicShiftGame, dt_ms: u64) -> bool {
    if game.game_result.is_some() {
        return false;
    }
    if game.waiting_to_start || game.forfeit_pending {
        return false;
    }

    let mut remaining = dt_ms.min(500);
    if remaining == 0 {
        return false;
    }

    game.tick_count += 1;
    let mut changed = false;

    while remaining > 0 && game.game_result.is_none() {
        if game.has_active_animation() {
            let step = remaining.min(animation_remaining_ms(game));
            advance_animation(game, step);
            game.accumulated_time_ms += step;
            remaining -= step;
            changed = true;
            continue;
        }

        if game.clear_timer_ms > 0 {
            let step = remaining.min(game.clear_timer_ms);
            game.clear_timer_ms -= step;
            game.accumulated_time_ms += step;
            remaining -= step;
            changed = true;

            if game.clear_timer_ms == 0 {
                resolve_clearing(game);
            }
            continue;
        }

        let until_rise = game
            .rise_interval_ms
            .saturating_sub(game.rise_accumulated_ms)
            .max(1);
        let step = remaining.min(until_rise);
        game.rise_accumulated_ms += step;
        game.accumulated_time_ms += step;
        remaining -= step;

        if game.rise_accumulated_ms >= game.rise_interval_ms {
            game.rise_accumulated_ms = 0;
            trigger_rise(game);
            changed = true;
        }
    }

    changed
}

fn animation_remaining_ms(game: &RunicShiftGame) -> u64 {
    match game.board_animation.as_ref() {
        Some(BoardAnimation::Swap(anim)) => anim.remaining_ms,
        Some(BoardAnimation::Fall(anim)) => anim.remaining_ms,
        None => 0,
    }
}

fn advance_animation(game: &mut RunicShiftGame, step_ms: u64) {
    let mut completed = false;

    if let Some(animation) = game.board_animation.as_mut() {
        match animation {
            BoardAnimation::Swap(anim) => {
                anim.remaining_ms = anim.remaining_ms.saturating_sub(step_ms);
                completed = anim.remaining_ms == 0;
            }
            BoardAnimation::Fall(anim) => {
                anim.remaining_ms = anim.remaining_ms.saturating_sub(step_ms);
                completed = anim.remaining_ms == 0;
            }
        }
    }

    if completed {
        game.board_animation = None;
        on_animation_complete(game);
    }
}

fn on_animation_complete(game: &mut RunicShiftGame) {
    let pending = game.pending_settle.take();
    match pending {
        Some(PendingSettle::AfterSwap) => {
            let fall_moves = apply_gravity_collect(game);
            if start_fall_animation(game, fall_moves, false) {
                return;
            }
            start_clear_if_matches(game, false);
        }
        Some(PendingSettle::AfterFall { chain_continuation }) => {
            if start_clear_if_matches(game, chain_continuation) {
                return;
            }
            if chain_continuation {
                game.current_chain = 0;
                game.chain_depth = 0;
            }
        }
        None => {}
    }
}

fn attempt_swap(game: &mut RunicShiftGame) -> bool {
    if game.clear_timer_ms > 0 || game.has_active_animation() {
        return false;
    }

    let row = game.cursor_row;
    let left_col = game.cursor_col;
    let right_col = left_col + 1;

    let left = game.grid[row][left_col];
    let right = game.grid[row][right_col];

    if left.is_none() && right.is_none() {
        return false;
    }

    if left.is_some_and(|b| b.state != BlockState::Normal)
        || right.is_some_and(|b| b.state != BlockState::Normal)
    {
        return false;
    }

    game.grid[row].swap(left_col, right_col);
    game.board_animation = Some(BoardAnimation::Swap(SwapAnimation {
        row,
        left_col,
        right_col,
        left_block: left,
        right_block: right,
        duration_ms: SWAP_ANIM_MS,
        remaining_ms: SWAP_ANIM_MS,
    }));
    game.pending_settle = Some(PendingSettle::AfterSwap);
    true
}

fn trigger_rise(game: &mut RunicShiftGame) -> bool {
    if game.clear_timer_ms > 0 {
        return false;
    }

    // Shift all rows upward by one.
    for row in 0..(GRID_ROWS - 1) {
        game.grid[row] = game.grid[row + 1];
    }

    // Insert preview row at bottom.
    let mut bottom = [None; GRID_COLS];
    for (col, slot) in bottom.iter_mut().enumerate() {
        *slot = Some(Block::normal(game.next_row[col]));
    }
    game.grid[GRID_ROWS - 1] = bottom;

    let mut rng = rand::rng();
    game.next_row = game.roll_next_row(&mut rng);

    if game.has_top_occupancy() {
        lose_life(game);
        return true;
    }

    start_clear_if_matches(game, false)
}

fn lose_life(game: &mut RunicShiftGame) {
    if game.lives > 1 {
        game.lives -= 1;
        game.reset_for_retry();
    } else {
        game.lives = 0;
        game.game_result = Some(RunicShiftResult::Loss);
    }
}

fn resolve_clearing(game: &mut RunicShiftGame) {
    let removed = remove_clearing_blocks(game);
    if removed > 0 {
        game.clears = game.clears.saturating_add(removed);
        if game.clears >= game.target_clears {
            game.game_result = Some(RunicShiftResult::Win);
            return;
        }
    }

    let fall_moves = apply_gravity_collect(game);
    if start_fall_animation(game, fall_moves, true) {
        return;
    }

    if start_clear_if_matches(game, true) {
        return;
    }
    game.current_chain = 0;
    game.chain_depth = 0;
}

fn start_clear_if_matches(game: &mut RunicShiftGame, chain_continuation: bool) -> bool {
    let matches = find_matches(&game.grid);
    if matches.is_empty() {
        return false;
    }

    if chain_continuation {
        game.chain_depth = game.chain_depth.saturating_add(1).max(2);
        game.current_chain = game.chain_depth;

        let bonus = game.chain_depth.saturating_sub(1);
        game.clears = game.clears.saturating_add(bonus);
        if game.clears >= game.target_clears {
            game.game_result = Some(RunicShiftResult::Win);
            return true;
        }
    } else {
        game.chain_depth = 1;
        game.current_chain = 1;
    }

    game.best_chain = game.best_chain.max(game.current_chain);

    for (row, col) in matches {
        if let Some(block) = game.grid[row][col].as_mut() {
            block.state = BlockState::Clearing;
        }
    }

    game.clear_timer_ms = game.difficulty.clear_duration_ms();
    true
}

fn remove_clearing_blocks(game: &mut RunicShiftGame) -> u32 {
    let mut removed = 0;

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            if let Some(block) = game.grid[row][col] {
                if block.state == BlockState::Clearing {
                    game.grid[row][col] = None;
                    removed += 1;
                }
            }
        }
    }

    removed
}

fn apply_gravity_collect(game: &mut RunicShiftGame) -> Vec<FallingBlockAnim> {
    let mut motions = Vec::new();

    for col in 0..GRID_COLS {
        let mut write_row = GRID_ROWS;

        for read_row in (0..GRID_ROWS).rev() {
            if let Some(mut block) = game.grid[read_row][col].take() {
                // Clearing blocks should have been removed already.
                block.state = BlockState::Normal;
                write_row -= 1;
                if write_row != read_row {
                    motions.push(FallingBlockAnim {
                        color: block.color,
                        col,
                        from_row: read_row,
                        to_row: write_row,
                    });
                }
                game.grid[write_row][col] = Some(block);
            }
        }

        for row in 0..write_row {
            game.grid[row][col] = None;
        }
    }

    motions
}

fn start_fall_animation(
    game: &mut RunicShiftGame,
    motions: Vec<FallingBlockAnim>,
    chain_continuation: bool,
) -> bool {
    if motions.is_empty() {
        return false;
    }

    let max_drop = motions
        .iter()
        .map(|m| m.to_row.abs_diff(m.from_row) as u64)
        .max()
        .unwrap_or(1);
    let duration = (FALL_ANIM_MIN_MS + max_drop * FALL_ANIM_PER_ROW_MS).min(FALL_ANIM_MAX_MS);

    game.board_animation = Some(BoardAnimation::Fall(FallAnimation {
        blocks: motions,
        duration_ms: duration,
        remaining_ms: duration,
    }));
    game.pending_settle = Some(PendingSettle::AfterFall { chain_continuation });
    true
}

fn find_matches(grid: &[[Option<Block>; GRID_COLS]; GRID_ROWS]) -> Vec<(usize, usize)> {
    let mut marked = [[false; GRID_COLS]; GRID_ROWS];

    // Horizontal runs.
    for row in 0..GRID_ROWS {
        let mut col = 0;
        while col < GRID_COLS {
            let Some(block) = grid[row][col] else {
                col += 1;
                continue;
            };
            if block.state != BlockState::Normal {
                col += 1;
                continue;
            }

            let color = block.color;
            let start = col;
            col += 1;
            while col < GRID_COLS {
                let Some(next) = grid[row][col] else {
                    break;
                };
                if next.state != BlockState::Normal || next.color != color {
                    break;
                }
                col += 1;
            }

            let len = col - start;
            if len >= 3 {
                for cell in marked[row].iter_mut().take(col).skip(start) {
                    *cell = true;
                }
            }
        }
    }

    // Vertical runs.
    for col in 0..GRID_COLS {
        let mut row = 0;
        while row < GRID_ROWS {
            let Some(block) = grid[row][col] else {
                row += 1;
                continue;
            };
            if block.state != BlockState::Normal {
                row += 1;
                continue;
            }

            let color = block.color;
            let start = row;
            row += 1;
            while row < GRID_ROWS {
                let Some(next) = grid[row][col] else {
                    break;
                };
                if next.state != BlockState::Normal || next.color != color {
                    break;
                }
                row += 1;
            }

            let len = row - start;
            if len >= 3 {
                for row_marks in marked.iter_mut().take(row).skip(start) {
                    row_marks[col] = true;
                }
            }
        }
    }

    let mut out = Vec::new();
    for (row, row_marks) in marked.iter().enumerate() {
        for (col, marked) in row_marks.iter().enumerate() {
            if *marked {
                out.push((row, col));
            }
        }
    }
    out
}

impl DifficultyInfo for RunicShiftDifficulty {
    fn name(&self) -> &'static str {
        RunicShiftDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            RunicShiftDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            RunicShiftDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            RunicShiftDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            RunicShiftDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!(
            "{} clears, {} colors, {}ms rise",
            self.target_clears(),
            self.num_colors(),
            self.rise_interval_ms()
        ))
    }
}

/// Apply game result using the shared challenge reward system.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, clears, target, best_chain, forfeit_pending) = {
        if let Some(ActiveMinigame::RunicShift(ref game)) = state.active_minigame {
            (
                game.game_result,
                game.difficulty,
                game.clears,
                game.target_clears,
                game.best_chain,
                game.forfeit_pending,
            )
        } else {
            return None;
        }
    };

    let result = result?;
    let won = matches!(result, RunicShiftResult::Win);
    let reward = difficulty.reward();

    if won {
        state.combat_state.add_log_entry(
            format!(
                "⇄ Runic Shift mastered! {}/{} clears (best chain x{})",
                clears, target, best_chain
            ),
            false,
            true,
        );
    } else if forfeit_pending {
        state.combat_state.add_log_entry(
            "⇄ You abandon the shifting runes.".to_string(),
            false,
            true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!("⇄ The runes overwhelm you at {}/{} clears.", clears, target),
            false,
            true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "runic_shift",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "⇄",
            win_message: "Runic Shift conquered!",
            loss_message: "The runes rise beyond control.",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started_game() -> RunicShiftGame {
        let mut game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        game.waiting_to_start = false;
        game
    }

    fn clear_grid(game: &mut RunicShiftGame) {
        game.grid = [[None; GRID_COLS]; GRID_ROWS];
    }

    #[test]
    fn test_waiting_to_start_only_space_starts() {
        let mut game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        assert!(game.waiting_to_start);

        process_input(&mut game, RunicShiftInput::Left);
        assert!(game.waiting_to_start);

        process_input(&mut game, RunicShiftInput::Swap);
        assert!(!game.waiting_to_start);
    }

    #[test]
    fn test_forfeit_flow() {
        let mut game = started_game();

        process_input(&mut game, RunicShiftInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        process_input(&mut game, RunicShiftInput::Forfeit);
        assert_eq!(game.game_result, Some(RunicShiftResult::Loss));
    }

    #[test]
    fn test_swap_with_empty_and_gravity() {
        let mut game = started_game();
        clear_grid(&mut game);

        // Place a block above the cursor row and swap into empty space.
        game.cursor_row = 5;
        game.cursor_col = 2;
        game.grid[5][2] = Some(Block::normal(RuneColor::Fire));

        process_input(&mut game, RunicShiftInput::Swap);
        assert!(matches!(
            game.board_animation,
            Some(BoardAnimation::Swap(_))
        ));

        // Let swap+fall animations resolve.
        tick_runic_shift(&mut game, 1000);

        // After swap + gravity, block should settle at bottom of right column.
        assert!(game.grid[GRID_ROWS - 1][3].is_some());
        assert!(game.grid[5][2].is_none());
    }

    #[test]
    fn test_swap_starts_animation_and_pending_settle() {
        let mut game = started_game();
        clear_grid(&mut game);
        game.cursor_row = 6;
        game.cursor_col = 2;
        game.grid[6][2] = Some(Block::normal(RuneColor::Fire));
        game.grid[6][3] = Some(Block::normal(RuneColor::Water));

        process_input(&mut game, RunicShiftInput::Swap);

        assert!(matches!(
            game.board_animation,
            Some(BoardAnimation::Swap(_))
        ));
        assert_eq!(game.pending_settle, Some(PendingSettle::AfterSwap));
    }

    #[test]
    fn test_horizontal_match_marks_clearing() {
        let mut game = started_game();
        clear_grid(&mut game);
        game.cursor_row = GRID_ROWS - 1;
        game.cursor_col = 0;

        game.grid[GRID_ROWS - 1][0] = Some(Block::normal(RuneColor::Fire));
        game.grid[GRID_ROWS - 1][1] = Some(Block::normal(RuneColor::Fire));
        game.grid[GRID_ROWS - 1][2] = Some(Block::normal(RuneColor::Fire));

        assert!(start_clear_if_matches(&mut game, false));
        assert!(game.clear_timer_ms > 0);
        assert_eq!(
            game.grid[GRID_ROWS - 1][1].map(|b| b.state),
            Some(BlockState::Clearing)
        );
    }

    #[test]
    fn test_clear_resolution_adds_score_and_resets_chain_when_done() {
        let mut game = started_game();
        clear_grid(&mut game);

        game.grid[GRID_ROWS - 1][0] = Some(Block::normal(RuneColor::Fire));
        game.grid[GRID_ROWS - 1][1] = Some(Block::normal(RuneColor::Fire));
        game.grid[GRID_ROWS - 1][2] = Some(Block::normal(RuneColor::Fire));

        start_clear_if_matches(&mut game, false);
        assert_eq!(game.current_chain, 1);

        // Advance until clear resolves.
        tick_runic_shift(&mut game, 1000);

        assert_eq!(game.clears, 3);
        assert_eq!(game.current_chain, 0);
        assert_eq!(game.chain_depth, 0);
    }

    #[test]
    fn test_rise_can_consume_life_and_reset_board() {
        let mut game = started_game();
        clear_grid(&mut game);
        game.lives = 2;

        // Occupy row 1 so one rise pushes to top row.
        game.grid[1][0] = Some(Block::normal(RuneColor::Fire));

        trigger_rise(&mut game);

        assert_eq!(game.lives, 1);
        assert!(game.waiting_to_start);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_final_life_loss_ends_game() {
        let mut game = started_game();
        clear_grid(&mut game);
        game.lives = 1;
        game.grid[1][2] = Some(Block::normal(RuneColor::Water));

        trigger_rise(&mut game);

        assert_eq!(game.lives, 0);
        assert_eq!(game.game_result, Some(RunicShiftResult::Loss));
    }

    #[test]
    fn test_apply_game_result_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        game.game_result = Some(RunicShiftResult::Win);
        game.clears = game.target_clears;
        state.active_minigame = Some(ActiveMinigame::RunicShift(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let win_info = result.unwrap();
        assert_eq!(win_info.game_type, "runic_shift");
        assert_eq!(win_info.difficulty, "novice");
    }

    #[test]
    fn test_apply_game_result_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        game.game_result = Some(RunicShiftResult::Loss);
        state.active_minigame = Some(ActiveMinigame::RunicShift(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_none());
    }
}
