//! Shard Fusion game logic.
//!
//! Implements the 2048-style slide-and-merge mechanics, animation ticking,
//! and input routing.

use super::types::{
    ShardFusionAnimState, ShardFusionDifficulty, ShardFusionGame, ShardFusionResult, TileMove,
    FLASH_TICKS, SLIDE_TICKS,
};
use crate::challenges::ActiveMinigame;
use rand::{Rng, RngExt};

/// Cardinal directions for board sliding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Input actions for the Shard Fusion game (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionInput {
    Left,
    Right,
    Up,
    Down,
    Forfeit,
    Other,
}

/// Result of sliding a single row left.
#[derive(Debug, Clone)]
pub struct SlideRowResult {
    /// The row after compaction and merging.
    pub row: [u32; 4],
    /// Total score gained from merges in this row.
    pub score_gain: u32,
    /// Destination columns that received a merged tile.
    pub merged_cols: Vec<usize>,
}

/// Compact non-zero tiles left and merge adjacent equal pairs (no double-merge).
///
/// Examples:
/// - `[2, 2, 0, 0]` → `[4, 0, 0, 0]`, score_gain = 4
/// - `[2, 2, 2, 2]` → `[4, 4, 0, 0]`, score_gain = 8
/// - `[0, 0, 2, 4]` → `[2, 4, 0, 0]`, score_gain = 0
pub fn slide_row_left(row: [u32; 4]) -> SlideRowResult {
    // Step 1: compact non-zeros left
    let mut compacted = [0u32; 4];
    let mut ci = 0;
    for &v in &row {
        if v != 0 {
            compacted[ci] = v;
            ci += 1;
        }
    }

    // Step 2: merge adjacent equal pairs (no double-merge)
    let mut merged = [0u32; 4];
    let mut score_gain = 0u32;
    let mut merged_cols = Vec::new();
    let mut src = 0;
    let mut dst = 0;
    while src < 4 {
        if compacted[src] == 0 {
            src += 1;
            continue;
        }
        if src + 1 < 4 && compacted[src] == compacted[src + 1] && compacted[src + 1] != 0 {
            let new_val = compacted[src] * 2;
            merged[dst] = new_val;
            score_gain += new_val;
            merged_cols.push(dst);
            src += 2;
        } else {
            merged[dst] = compacted[src];
            src += 1;
        }
        dst += 1;
    }

    SlideRowResult {
        row: merged,
        score_gain,
        merged_cols,
    }
}

/// Returns true if the board has at least one valid move (empty cell or adjacent merge).
pub fn has_valid_moves(board: &[[u32; 4]; 4]) -> bool {
    for r in 0..4 {
        for c in 0..4 {
            // Empty cell counts as a valid move
            if board[r][c] == 0 {
                return true;
            }
            // Adjacent horizontal merge
            if c + 1 < 4 && board[r][c] == board[r][c + 1] {
                return true;
            }
            // Adjacent vertical merge
            if r + 1 < 4 && board[r][c] == board[r + 1][c] {
                return true;
            }
        }
    }
    false
}

/// Rotate the board 90 degrees clockwise.
/// Used to reduce all slide directions to "slide left".
fn rotate_cw(board: [[u32; 4]; 4]) -> [[u32; 4]; 4] {
    let mut out = [[0u32; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            out[c][3 - r] = board[r][c];
        }
    }
    out
}

/// Un-rotate a (row, col) coordinate by one CW rotation (i.e., apply one CCW rotation).
/// This maps a coordinate in the rotated space back to the original space.
fn unrotate_coord(r: usize, c: usize) -> (usize, usize) {
    (3 - c, r)
}

/// Apply a slide in the given direction. Returns `true` if the board changed.
///
/// On a successful slide:
/// - Updates `game.board`
/// - Adds `game.score`
/// - Populates `game.slide_moves` and `game.merged_cells`
/// - Sets `game.anim_state` to `Sliding(SLIDE_TICKS)`
/// - Checks for win condition immediately
pub fn apply_slide(game: &mut ShardFusionGame, direction: Direction) -> bool {
    // Number of CW rotations to bring the desired slide direction to "slide left":
    //   Left  = 0 rotations
    //   Down  = 1 rotation
    //   Right = 2 rotations
    //   Up    = 3 rotations
    let rotations = match direction {
        Direction::Left => 0,
        Direction::Down => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    };

    // Rotate the board into the "slide left" orientation
    let mut rotated = game.board;
    for _ in 0..rotations {
        rotated = rotate_cw(rotated);
    }

    // Slide each row left and collect results
    let mut new_rotated = [[0u32; 4]; 4];
    let mut total_score = 0u32;
    let mut slide_moves: Vec<TileMove> = Vec::new();
    let mut merged_in_rotated: Vec<(usize, usize)> = Vec::new();

    for row_idx in 0..4 {
        let row = rotated[row_idx];
        let result = slide_row_left(row);
        new_rotated[row_idx] = result.row;
        total_score += result.score_gain;

        for &mc in &result.merged_cols {
            merged_in_rotated.push((row_idx, mc));
        }

        // Build TileMoves: trace source columns for each destination tile
        // We need to figure out where each destination tile came from in the *original* board.
        //
        // Algorithm: simulate the compaction + merge step tracking source column indices.
        let compacted_src: Vec<(usize, u32)> = row
            .iter()
            .enumerate()
            .filter(|(_, &v)| v != 0)
            .map(|(c, &v)| (c, v))
            .collect();

        // Replay merge to determine which source columns end up at each destination
        let mut src = 0;
        let mut dst = 0;
        while src < compacted_src.len() {
            let (sc, sv) = compacted_src[src];
            if src + 1 < compacted_src.len() && sv == compacted_src[src + 1].1 {
                // Merge: both source tiles go to dst
                let (sc2, _) = compacted_src[src + 1];
                let merged_val = sv * 2;
                // Record both source tiles moving to dst
                // Un-rotate both source and destination coordinates
                let (orig_from_r1, orig_from_c1) = {
                    let mut coord = (row_idx, sc);
                    // Un-rotate `rotations` times
                    for _ in 0..rotations {
                        coord = unrotate_coord(coord.0, coord.1);
                    }
                    coord
                };
                let (orig_from_r2, orig_from_c2) = {
                    let mut coord = (row_idx, sc2);
                    for _ in 0..rotations {
                        coord = unrotate_coord(coord.0, coord.1);
                    }
                    coord
                };
                let (orig_to_r, orig_to_c) = {
                    let mut coord = (row_idx, dst);
                    for _ in 0..rotations {
                        coord = unrotate_coord(coord.0, coord.1);
                    }
                    coord
                };
                slide_moves.push(TileMove {
                    from: (orig_from_r1, orig_from_c1),
                    to: (orig_to_r, orig_to_c),
                    value: merged_val,
                });
                slide_moves.push(TileMove {
                    from: (orig_from_r2, orig_from_c2),
                    to: (orig_to_r, orig_to_c),
                    value: merged_val,
                });
                src += 2;
            } else {
                // Simple move: tile slides from sc to dst
                let (orig_from_r, orig_from_c) = {
                    let mut coord = (row_idx, sc);
                    for _ in 0..rotations {
                        coord = unrotate_coord(coord.0, coord.1);
                    }
                    coord
                };
                let (orig_to_r, orig_to_c) = {
                    let mut coord = (row_idx, dst);
                    for _ in 0..rotations {
                        coord = unrotate_coord(coord.0, coord.1);
                    }
                    coord
                };
                // Only record if the tile actually moved
                if (orig_from_r, orig_from_c) != (orig_to_r, orig_to_c) {
                    slide_moves.push(TileMove {
                        from: (orig_from_r, orig_from_c),
                        to: (orig_to_r, orig_to_c),
                        value: sv,
                    });
                }
                src += 1;
            }
            dst += 1;
        }
    }

    // Check if anything changed
    if new_rotated == rotated {
        return false;
    }

    // Rotate the new board back to original orientation
    // Rotating CW `rotations` times → rotate CW (4 - rotations) times to reverse
    let mut new_board = new_rotated;
    let reverse_rotations = (4 - rotations) % 4;
    for _ in 0..reverse_rotations {
        new_board = rotate_cw(new_board);
    }

    // Un-rotate merged cell coordinates back to original space
    let merged_cells: Vec<(usize, usize)> = merged_in_rotated
        .into_iter()
        .map(|(mr, mc)| {
            let mut coord = (mr, mc);
            for _ in 0..rotations {
                coord = unrotate_coord(coord.0, coord.1);
            }
            coord
        })
        .collect();

    // Apply the changes
    game.board = new_board;
    game.score += total_score;
    game.slide_moves = slide_moves;
    game.merged_cells = merged_cells;
    game.anim_state = ShardFusionAnimState::Sliding(SLIDE_TICKS);

    // Check for win immediately after slide
    check_game_over(game);

    true
}

/// Spawn a new tile in a random empty cell. 90% chance of value 2, 10% chance of 4.
/// No-op if the board is full.
pub fn spawn_tile<R: Rng>(board: &mut [[u32; 4]; 4], rng: &mut R) {
    let empty_cells: Vec<(usize, usize)> = (0..4)
        .flat_map(|r| (0..4).map(move |c| (r, c)))
        .filter(|&(r, c)| board[r][c] == 0)
        .collect();

    if empty_cells.is_empty() {
        return;
    }

    let idx = rng.random_range(0..empty_cells.len());
    let (r, c) = empty_cells[idx];
    board[r][c] = if rng.random_range(0u32..10) < 9 { 2 } else { 4 };
}

/// Check win/loss conditions and update `game.game_result`.
///
/// Win: highest tile >= difficulty target.
/// Loss: no valid moves remain.
/// Already-decided games are not re-evaluated.
pub fn check_game_over(game: &mut ShardFusionGame) {
    if game.game_result.is_some() {
        return;
    }
    if game.highest_tile() >= game.difficulty.target_value() {
        game.game_result = Some(ShardFusionResult::Win);
    } else if !has_valid_moves(&game.board) {
        game.game_result = Some(ShardFusionResult::Loss);
    }
}

/// Advance the animation state by one tick (100ms).
///
/// State transitions:
/// - `Sliding(1)` → spawn tile, check game over, then transition to `Flashing(FLASH_TICKS)`
///   if there were merged cells, or `Idle` if not.
/// - `Sliding(t > 1)` → `Sliding(t - 1)`
/// - `Flashing(1)` → clear `merged_cells`, transition to `Idle`
/// - `Flashing(t > 1)` → `Flashing(t - 1)`
/// - `Idle` → no-op
pub fn tick_shard_fusion<R: Rng>(game: &mut ShardFusionGame, rng: &mut R) {
    match game.anim_state {
        ShardFusionAnimState::Sliding(1) => {
            spawn_tile(&mut game.board, rng);
            check_game_over(game);
            game.slide_moves.clear();
            if game.merged_cells.is_empty() {
                game.anim_state = ShardFusionAnimState::Idle;
            } else {
                game.anim_state = ShardFusionAnimState::Flashing(FLASH_TICKS);
            }
        }
        ShardFusionAnimState::Sliding(t) => {
            game.anim_state = ShardFusionAnimState::Sliding(t - 1);
        }
        ShardFusionAnimState::Flashing(1) => {
            game.merged_cells.clear();
            game.anim_state = ShardFusionAnimState::Idle;
        }
        ShardFusionAnimState::Flashing(t) => {
            game.anim_state = ShardFusionAnimState::Flashing(t - 1);
        }
        ShardFusionAnimState::Idle => {}
    }
}

/// Route a player input action. Blocks direction inputs during animation.
///
/// - If `game_result` is already set, ignores all input.
/// - Handles the double-Esc forfeit pattern.
/// - Blocks slide inputs while `anim_state != Idle`.
pub fn process_input<R: Rng>(game: &mut ShardFusionGame, input: ShardFusionInput, rng: &mut R) {
    if game.game_result.is_some() {
        return;
    }

    // Handle forfeit double-Esc pattern
    if game.forfeit_pending {
        match input {
            ShardFusionInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    ShardFusionResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return;
    }

    match input {
        ShardFusionInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                ShardFusionResult::Loss,
            );
        }
        ShardFusionInput::Left
        | ShardFusionInput::Right
        | ShardFusionInput::Up
        | ShardFusionInput::Down => {
            // Block input during animation
            if game.anim_state != ShardFusionAnimState::Idle {
                return;
            }
            let dir = match input {
                ShardFusionInput::Left => Direction::Left,
                ShardFusionInput::Right => Direction::Right,
                ShardFusionInput::Up => Direction::Up,
                ShardFusionInput::Down => Direction::Down,
                _ => unreachable!(),
            };
            apply_slide(game, dir);
            let _ = rng; // rng not used here; spawn happens in tick
        }
        ShardFusionInput::Other => {}
    }
}

impl crate::challenges::menu::DifficultyInfo for ShardFusionDifficulty {
    fn name(&self) -> &'static str {
        ShardFusionDifficulty::name(self)
    }

    fn reward(&self) -> crate::challenges::menu::ChallengeReward {
        match self {
            ShardFusionDifficulty::Novice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 1_000,
                fishing_ranks: 0,
            },
            ShardFusionDifficulty::Apprentice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 2_500,
                fishing_ranks: 0,
            },
            ShardFusionDifficulty::Journeyman => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 5_000,
                fishing_ranks: 0,
            },
            ShardFusionDifficulty::Master => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 2,
                stormglass: 10_000,
                fishing_ranks: 0,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("Reach {} to win", self.target_value()))
    }
}

/// Start a new Shard Fusion game and return it as an `ActiveMinigame`.
/// Spawns 2 initial tiles.
pub fn start_shard_fusion_game<R: Rng>(
    difficulty: ShardFusionDifficulty,
    rng: &mut R,
) -> ActiveMinigame {
    let mut game = ShardFusionGame::new(difficulty);
    spawn_tile(&mut game.board, rng);
    spawn_tile(&mut game.board, rng);
    ActiveMinigame::ShardFusion(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_slide_left_basic_merge() {
        let result = slide_row_left([2, 2, 0, 0]);
        assert_eq!(result.row, [4, 0, 0, 0]);
        assert_eq!(result.score_gain, 4);
        assert_eq!(result.merged_cols, vec![0]);
    }

    #[test]
    fn test_slide_left_no_merge() {
        let result = slide_row_left([2, 4, 8, 16]);
        assert_eq!(result.row, [2, 4, 8, 16]);
        assert_eq!(result.score_gain, 0);
        assert!(result.merged_cols.is_empty());
    }

    #[test]
    fn test_slide_left_compacts() {
        let result = slide_row_left([0, 0, 2, 4]);
        assert_eq!(result.row, [2, 4, 0, 0]);
        assert_eq!(result.score_gain, 0);
    }

    #[test]
    fn test_slide_left_no_double_merge() {
        let result = slide_row_left([2, 2, 2, 2]);
        assert_eq!(result.row, [4, 4, 0, 0]);
        assert_eq!(result.score_gain, 8);
    }

    #[test]
    fn test_has_valid_moves_full_no_adjacent() {
        let b = [[2u32, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        assert!(!has_valid_moves(&b));
    }

    #[test]
    fn test_has_valid_moves_empty_cell() {
        let mut b = [[0u32; 4]; 4];
        b[0][0] = 2;
        assert!(has_valid_moves(&b));
    }

    #[test]
    fn test_apply_slide_left_updates_board_and_score() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        game.board[0][1] = 2;
        let changed = apply_slide(&mut game, Direction::Left);
        assert!(changed);
        assert_eq!(game.board[0][0], 4);
        assert_eq!(game.score, 4);
        assert_eq!(game.anim_state, ShardFusionAnimState::Sliding(SLIDE_TICKS));
    }

    #[test]
    fn test_apply_slide_no_change_returns_false() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        game.board[0][1] = 4;
        game.board[0][2] = 8;
        game.board[0][3] = 16;
        // already compacted left, no merges — left slide changes nothing
        let changed = apply_slide(&mut game, Direction::Left);
        assert!(!changed);
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_tick_advances_sliding_to_flashing() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.merged_cells = vec![(0, 0)];
        game.anim_state = ShardFusionAnimState::Sliding(1);
        // Board must have room for spawn_tile
        game.board[0][0] = 4; // already merged tile
        let mut rng = SmallRng::seed_from_u64(0);
        tick_shard_fusion(&mut game, &mut rng);
        assert_eq!(game.anim_state, ShardFusionAnimState::Flashing(FLASH_TICKS));
    }

    #[test]
    fn test_tick_advances_sliding_to_idle_no_merges() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.anim_state = ShardFusionAnimState::Sliding(1);
        game.board[0][0] = 4;
        let mut rng = SmallRng::seed_from_u64(0);
        tick_shard_fusion(&mut game, &mut rng);
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_tick_advances_flashing_to_idle() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.anim_state = ShardFusionAnimState::Flashing(1);
        let mut rng = SmallRng::seed_from_u64(0);
        tick_shard_fusion(&mut game, &mut rng);
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_tick_counts_down_sliding() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.anim_state = ShardFusionAnimState::Sliding(3);
        let mut rng = SmallRng::seed_from_u64(0);
        tick_shard_fusion(&mut game, &mut rng);
        assert_eq!(game.anim_state, ShardFusionAnimState::Sliding(2));
    }

    #[test]
    fn test_win_detected_after_slide() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice); // target = 512
        game.board[0][0] = 256;
        game.board[0][1] = 256;
        apply_slide(&mut game, Direction::Left);
        assert_eq!(game.game_result, Some(ShardFusionResult::Win));
    }

    #[test]
    fn test_loss_detected_when_no_moves() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        check_game_over(&mut game);
        assert_eq!(game.game_result, Some(ShardFusionResult::Loss));
    }

    // ---- Additional coverage ----

    #[test]
    fn test_slide_left_all_zeros() {
        let result = slide_row_left([0, 0, 0, 0]);
        assert_eq!(result.row, [0, 0, 0, 0]);
        assert_eq!(result.score_gain, 0);
        assert!(result.merged_cols.is_empty());
    }

    #[test]
    fn test_slide_left_single_tile() {
        let result = slide_row_left([0, 0, 0, 4]);
        assert_eq!(result.row, [4, 0, 0, 0]);
        assert_eq!(result.score_gain, 0);
    }

    #[test]
    fn test_slide_right() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][2] = 2;
        game.board[0][3] = 2;
        let changed = apply_slide(&mut game, Direction::Right);
        assert!(changed);
        assert_eq!(game.board[0][3], 4);
        assert_eq!(game.board[0][2], 0);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn test_slide_up() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[2][0] = 2;
        game.board[3][0] = 2;
        let changed = apply_slide(&mut game, Direction::Up);
        assert!(changed);
        assert_eq!(game.board[0][0], 4);
        assert_eq!(game.board[1][0], 0);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn test_slide_down() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        game.board[1][0] = 2;
        let changed = apply_slide(&mut game, Direction::Down);
        assert!(changed);
        assert_eq!(game.board[3][0], 4);
        assert_eq!(game.board[2][0], 0);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn test_spawn_tile_places_on_empty_board() {
        let mut board = [[0u32; 4]; 4];
        let mut rng = SmallRng::seed_from_u64(42);
        spawn_tile(&mut board, &mut rng);
        let non_zero: Vec<u32> = board
            .iter()
            .flatten()
            .filter(|&&v| v != 0)
            .copied()
            .collect();
        assert_eq!(non_zero.len(), 1);
        assert!(non_zero[0] == 2 || non_zero[0] == 4);
    }

    #[test]
    fn test_spawn_tile_noop_on_full_board() {
        let mut board = [[2u32, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        let original = board;
        let mut rng = SmallRng::seed_from_u64(0);
        spawn_tile(&mut board, &mut rng);
        assert_eq!(board, original);
    }

    #[test]
    fn test_process_input_blocks_during_animation() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        game.board[0][1] = 2;
        game.anim_state = ShardFusionAnimState::Sliding(SLIDE_TICKS);
        let board_before = game.board;
        let mut rng = SmallRng::seed_from_u64(0);
        process_input(&mut game, ShardFusionInput::Left, &mut rng);
        assert_eq!(game.board, board_before);
    }

    #[test]
    fn test_process_input_forfeit_single_esc() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        let mut rng = SmallRng::seed_from_u64(0);
        process_input(&mut game, ShardFusionInput::Forfeit, &mut rng);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_double_esc() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        let mut rng = SmallRng::seed_from_u64(0);
        process_input(&mut game, ShardFusionInput::Forfeit, &mut rng);
        process_input(&mut game, ShardFusionInput::Forfeit, &mut rng);
        assert_eq!(game.game_result, Some(ShardFusionResult::Loss));
    }

    #[test]
    fn test_process_input_forfeit_cancelled() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        let mut rng = SmallRng::seed_from_u64(0);
        process_input(&mut game, ShardFusionInput::Forfeit, &mut rng);
        process_input(&mut game, ShardFusionInput::Other, &mut rng);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_ignored_when_game_over() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.game_result = Some(ShardFusionResult::Loss);
        game.board[0][0] = 2;
        game.board[0][1] = 2;
        let board_before = game.board;
        let mut rng = SmallRng::seed_from_u64(0);
        process_input(&mut game, ShardFusionInput::Left, &mut rng);
        assert_eq!(game.board, board_before);
    }

    #[test]
    fn test_rotate_cw_round_trip() {
        let board = [
            [1u32, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16],
        ];
        let mut rotated = board;
        for _ in 0..4 {
            rotated = rotate_cw(rotated);
        }
        assert_eq!(rotated, board);
    }

    #[test]
    fn test_merged_cells_cleared_after_flash() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.merged_cells = vec![(0, 0), (1, 1)];
        game.anim_state = ShardFusionAnimState::Flashing(1);
        let mut rng = SmallRng::seed_from_u64(0);
        tick_shard_fusion(&mut game, &mut rng);
        assert!(game.merged_cells.is_empty());
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_check_game_over_no_result_when_moves_available() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        check_game_over(&mut game);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_check_game_over_does_not_overwrite_result() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.game_result = Some(ShardFusionResult::Win);
        // Full board with no moves — would normally be Loss but Win takes precedence
        game.board = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        check_game_over(&mut game);
        // Should remain Win, not be overwritten
        assert_eq!(game.game_result, Some(ShardFusionResult::Win));
    }
}
