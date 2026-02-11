//! Go game logic: placement, capture, ko, scoring.

use super::types::{GoGame, GoMove, Stone, BOARD_SIZE};
use crate::challenges::{ChallengeDifficulty, ChallengeResult, MinigameInput};
use std::collections::HashSet;

/// Get all stones in the same group as the stone at (row, col).
/// Returns empty set if position is empty.
pub fn get_group(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
) -> HashSet<(usize, usize)> {
    let mut group = HashSet::new();
    let Some(stone) = board[row][col] else {
        return group;
    };

    let mut stack = vec![(row, col)];
    while let Some((r, c)) = stack.pop() {
        if group.contains(&(r, c)) {
            continue;
        }
        if board[r][c] == Some(stone) {
            group.insert((r, c));
            // Add adjacent positions
            for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    stack.push((nr as usize, nc as usize));
                }
            }
        }
    }
    group
}

/// Count liberties (empty adjacent points) of a group.
pub fn count_liberties(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    group: &HashSet<(usize, usize)>,
) -> usize {
    let mut liberties = HashSet::new();
    for &(row, col) in group {
        for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                let nr = nr as usize;
                let nc = nc as usize;
                if board[nr][nc].is_none() {
                    liberties.insert((nr, nc));
                }
            }
        }
    }
    liberties.len()
}

/// Get liberties count for the group containing the stone at (row, col).
#[allow(dead_code)]
pub fn get_liberties_at(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
) -> usize {
    let group = get_group(board, row, col);
    count_liberties(board, &group)
}

/// Remove a group from the board and return the number of stones captured.
fn remove_group(
    board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    group: &HashSet<(usize, usize)>,
) -> u32 {
    let count = group.len() as u32;
    for &(row, col) in group {
        board[row][col] = None;
    }
    count
}

/// Check and remove any opponent groups with zero liberties adjacent to (row, col).
/// Returns the total number of stones captured.
pub fn capture_dead_groups(
    board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    capturing_player: Stone,
) -> u32 {
    let opponent = capturing_player.opponent();
    let mut captured = 0;
    let mut checked = HashSet::new();

    // Check all adjacent positions for opponent groups
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;

            if checked.contains(&(nr, nc)) {
                continue;
            }

            if board[nr][nc] == Some(opponent) {
                let group = get_group(board, nr, nc);
                for &pos in &group {
                    checked.insert(pos);
                }
                if count_liberties(board, &group) == 0 {
                    captured += remove_group(board, &group);
                }
            }
        }
    }
    captured
}

/// Check if a move is legal (ignoring ko for now).
/// A move is illegal if:
/// 1. Position is occupied
/// 2. Position is the ko point
/// 3. Move would be suicide (no liberties after placement, and no captures)
pub fn is_legal_move(game: &GoGame, row: usize, col: usize) -> bool {
    // Position must be empty
    if !game.is_empty(row, col) {
        return false;
    }

    // Cannot play at ko point
    if game.ko_point == Some((row, col)) {
        return false;
    }

    // Check for suicide
    // Temporarily place the stone
    let mut test_board = game.board;
    test_board[row][col] = Some(game.current_player);

    // First check if this move captures anything
    let opponent = game.current_player.opponent();
    let mut would_capture = false;
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;
            if test_board[nr][nc] == Some(opponent) {
                let group = get_group(&test_board, nr, nc);
                if count_liberties(&test_board, &group) == 0 {
                    would_capture = true;
                    break;
                }
            }
        }
    }

    // If we capture, move is legal (not suicide)
    if would_capture {
        return true;
    }

    // Check if our group would have liberties
    let our_group = get_group(&test_board, row, col);
    count_liberties(&test_board, &our_group) > 0
}

/// Get all legal moves for the current player.
pub fn get_legal_moves(game: &GoGame) -> Vec<GoMove> {
    let mut moves = Vec::new();
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if is_legal_move(game, row, col) {
                moves.push(GoMove::Place(row, col));
            }
        }
    }
    moves.push(GoMove::Pass); // Pass is always legal
    moves
}

/// Execute a move on the game state.
/// Returns true if the move was successful.
pub fn make_move(game: &mut GoGame, mv: GoMove) -> bool {
    match mv {
        GoMove::Pass => {
            game.consecutive_passes += 1;
            game.ko_point = None;
            game.last_move = Some(GoMove::Pass);
            game.switch_player();

            // Check for game end (two consecutive passes)
            if game.consecutive_passes >= 2 {
                end_game_by_scoring(game);
            }
            true
        }
        GoMove::Place(row, col) => {
            if !is_legal_move(game, row, col) {
                return false;
            }

            // Place the stone
            game.board[row][col] = Some(game.current_player);
            game.consecutive_passes = 0;

            // Capture any dead opponent groups
            let captured = capture_dead_groups(&mut game.board, row, col, game.current_player);

            // Update capture counts
            match game.current_player {
                Stone::Black => game.captured_by_black += captured,
                Stone::White => game.captured_by_white += captured,
            }

            // Set ko point if exactly one stone was captured
            game.ko_point = if captured == 1 {
                // Find where the captured stone was (it's now empty and adjacent)
                let mut ko = None;
                for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        if game.board[nr][nc].is_none() {
                            // Verify this was where the capture happened by checking
                            // if replaying there would recapture our stone
                            let mut test_board = game.board;
                            test_board[nr][nc] = Some(game.current_player.opponent());
                            let test_group = get_group(&test_board, row, col);
                            if count_liberties(&test_board, &test_group) == 0 {
                                ko = Some((nr, nc));
                                break;
                            }
                        }
                    }
                }
                ko
            } else {
                None
            };

            game.last_move = Some(GoMove::Place(row, col));
            game.switch_player();
            true
        }
    }
}

/// End the game and calculate scores using Chinese rules.
fn end_game_by_scoring(game: &mut GoGame) {
    let (black_score, white_score) = calculate_score(&game.board);

    // Determine winner (Black plays as human)
    game.game_result = Some(if black_score > white_score {
        ChallengeResult::Win
    } else if white_score > black_score {
        ChallengeResult::Loss
    } else {
        ChallengeResult::Draw
    });
}

/// Calculate scores using Chinese rules (stones + territory).
pub fn calculate_score(board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE]) -> (i32, i32) {
    let mut black_score = 0i32;
    let mut white_score = 0i32;
    let mut counted = [[false; BOARD_SIZE]; BOARD_SIZE];

    // Count stones
    for row in board {
        for cell in row {
            match cell {
                Some(Stone::Black) => black_score += 1,
                Some(Stone::White) => white_score += 1,
                None => {}
            }
        }
    }

    // Count territory (empty regions completely surrounded by one color)
    for (row, board_row) in board.iter().enumerate() {
        for (col, cell) in board_row.iter().enumerate() {
            if cell.is_none() && !counted[row][col] {
                let (region, owner) = get_empty_region(board, row, col);
                for &(r, c) in &region {
                    counted[r][c] = true;
                }
                match owner {
                    Some(Stone::Black) => black_score += region.len() as i32,
                    Some(Stone::White) => white_score += region.len() as i32,
                    None => {} // Contested - no points
                }
            }
        }
    }

    // Apply komi (6.5 points to White for going second)
    // We use integer math, so 6 points (simplified)
    white_score += 6;

    (black_score, white_score)
}

/// Get an empty region and determine its owner (if surrounded by one color).
fn get_empty_region(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    start_row: usize,
    start_col: usize,
) -> (HashSet<(usize, usize)>, Option<Stone>) {
    let mut region = HashSet::new();
    let mut stack = vec![(start_row, start_col)];
    let mut borders_black = false;
    let mut borders_white = false;

    while let Some((row, col)) = stack.pop() {
        if region.contains(&(row, col)) {
            continue;
        }

        match board[row][col] {
            None => {
                region.insert((row, col));
                for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        stack.push((nr as usize, nc as usize));
                    }
                }
            }
            Some(Stone::Black) => borders_black = true,
            Some(Stone::White) => borders_white = true,
        }
    }

    let owner = match (borders_black, borders_white) {
        (true, false) => Some(Stone::Black),
        (false, true) => Some(Stone::White),
        _ => None, // Contested or touches both
    };

    (region, owner)
}

use super::mcts::mcts_best_move;

/// Process a key input during active Go game.
/// Returns true if the input was handled.
pub fn process_input(game: &mut GoGame, input: MinigameInput) -> bool {
    // Don't process input while AI is thinking
    if game.ai_thinking {
        return false;
    }

    // Handle forfeit confirmation (double-Esc pattern)
    if game.forfeit_pending {
        match input {
            MinigameInput::Cancel => {
                // Second Esc - confirm forfeit
                game.game_result = Some(ChallengeResult::Forfeit);
                game.forfeit_pending = false;
                return true;
            }
            _ => {
                // Any other key cancels forfeit
                game.forfeit_pending = false;
                return true;
            }
        }
    }

    match input {
        MinigameInput::Up => {
            game.move_cursor(-1, 0);
            true
        }
        MinigameInput::Down => {
            game.move_cursor(1, 0);
            true
        }
        MinigameInput::Left => {
            game.move_cursor(0, -1);
            true
        }
        MinigameInput::Right => {
            game.move_cursor(0, 1);
            true
        }
        MinigameInput::Primary => process_human_move(game),
        MinigameInput::Secondary => process_human_pass(game),
        MinigameInput::Cancel => {
            game.forfeit_pending = true;
            true
        }
        MinigameInput::Other => false,
    }
}

/// Process human move at cursor position.
pub fn process_human_move(game: &mut GoGame) -> bool {
    if game.game_result.is_some() || game.current_player != Stone::Black {
        return false;
    }

    let (row, col) = game.cursor;
    if !is_legal_move(game, row, col) {
        return false;
    }

    make_move(game, GoMove::Place(row, col));

    // Check for game end
    if game.game_result.is_some() {
        return true;
    }

    // Start AI thinking
    game.ai_thinking = true;
    game.ai_think_ticks = 0;
    true
}

/// Process human pass.
pub fn process_human_pass(game: &mut GoGame) -> bool {
    if game.game_result.is_some() || game.current_player != Stone::Black {
        return false;
    }

    make_move(game, GoMove::Pass);

    // Check for game end
    if game.game_result.is_some() {
        return true;
    }

    // Start AI thinking
    game.ai_thinking = true;
    game.ai_think_ticks = 0;
    true
}

/// Process AI turn (called each tick while ai_thinking is true).
pub fn process_go_ai<R: rand::Rng>(game: &mut GoGame, rng: &mut R) {
    if !game.ai_thinking || game.game_result.is_some() {
        return;
    }

    // Simulate thinking delay (5-15 ticks = 0.5-1.5 seconds)
    game.ai_think_ticks += 1;
    let min_ticks = match game.difficulty {
        ChallengeDifficulty::Novice => 5,
        ChallengeDifficulty::Apprentice => 8,
        ChallengeDifficulty::Journeyman => 10,
        ChallengeDifficulty::Master => 15,
    };

    if game.ai_think_ticks < min_ticks {
        return;
    }

    // Get AI move using MCTS
    let ai_move = mcts_best_move(game, rng);
    make_move(game, ai_move);
    game.ai_thinking = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(
        board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
        row: usize,
        col: usize,
        stone: Stone,
    ) {
        board[row][col] = Some(stone);
    }

    #[test]
    fn test_single_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        // Center stone has 4 liberties
        assert_eq!(get_liberties_at(&board, 4, 4), 4);
    }

    #[test]
    fn test_corner_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 0, 0, Stone::Black);
        // Corner stone has 2 liberties
        assert_eq!(get_liberties_at(&board, 0, 0), 2);
    }

    #[test]
    fn test_edge_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 0, 4, Stone::Black);
        // Edge stone has 3 liberties
        assert_eq!(get_liberties_at(&board, 0, 4), 3);
    }

    #[test]
    fn test_group_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Two connected stones share liberties
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 4, 5, Stone::Black);
        // Group has 6 liberties (shared liberty counted once)
        assert_eq!(get_liberties_at(&board, 4, 4), 6);
        assert_eq!(get_liberties_at(&board, 4, 5), 6);
    }

    #[test]
    fn test_surrounded_stone() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 3, 4, Stone::White);
        place(&mut board, 5, 4, Stone::White);
        place(&mut board, 4, 3, Stone::White);
        place(&mut board, 4, 5, Stone::White);
        // Surrounded stone has 0 liberties
        assert_eq!(get_liberties_at(&board, 4, 4), 0);
    }

    #[test]
    fn test_get_group() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 4, 5, Stone::Black);
        place(&mut board, 4, 6, Stone::Black);

        let group = get_group(&board, 4, 4);
        assert_eq!(group.len(), 3);
        assert!(group.contains(&(4, 4)));
        assert!(group.contains(&(4, 5)));
        assert!(group.contains(&(4, 6)));
    }

    #[test]
    fn test_empty_position_group() {
        let board = [[None; BOARD_SIZE]; BOARD_SIZE];
        let group = get_group(&board, 4, 4);
        assert!(group.is_empty());
    }

    #[test]
    fn test_capture_single_stone() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Set up a capture scenario
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // White has 1 liberty at (4, 5)
        assert_eq!(get_liberties_at(&board, 4, 4), 1);

        // Black plays at (4, 5) to capture
        place(&mut board, 4, 5, Stone::Black);
        let captured = capture_dead_groups(&mut board, 4, 5, Stone::Black);

        assert_eq!(captured, 1);
        assert!(board[4][4].is_none()); // White stone removed
    }

    #[test]
    fn test_capture_group() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Two white stones
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 4, 5, Stone::White);
        // Surround with black
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 3, 5, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 5, 5, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // One liberty left at (4, 6)

        place(&mut board, 4, 6, Stone::Black);
        let captured = capture_dead_groups(&mut board, 4, 6, Stone::Black);

        assert_eq!(captured, 2);
        assert!(board[4][4].is_none());
        assert!(board[4][5].is_none());
    }

    #[test]
    fn test_no_capture_with_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // White still has liberty at (4, 5) - not surrounded

        // Black plays elsewhere
        place(&mut board, 0, 0, Stone::Black);
        let captured = capture_dead_groups(&mut board, 0, 0, Stone::Black);

        assert_eq!(captured, 0);
        assert!(board[4][4].is_some()); // White stone still there
    }

    #[test]
    fn test_legal_move_empty_board() {
        let game = GoGame::new(ChallengeDifficulty::Novice);
        assert!(is_legal_move(&game, 4, 4));
        assert!(is_legal_move(&game, 0, 0));
    }

    #[test]
    fn test_illegal_move_occupied() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        game.board[4][4] = Some(Stone::Black);
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_illegal_move_ko() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        game.ko_point = Some((4, 4));
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_suicide_illegal() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        // Create a surrounded empty point
        game.board[3][4] = Some(Stone::White);
        game.board[5][4] = Some(Stone::White);
        game.board[4][3] = Some(Stone::White);
        game.board[4][5] = Some(Stone::White);
        game.current_player = Stone::Black;
        // Black playing at (4,4) would be suicide
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_capture_not_suicide() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        // White stone at center
        game.board[4][4] = Some(Stone::White);
        // Black almost surrounds
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.current_player = Stone::Black;
        // Black playing at (4,5) captures white, so it's legal even though
        // black would have no liberties without the capture
        assert!(is_legal_move(&game, 4, 5));
    }

    #[test]
    fn test_get_legal_moves_includes_pass() {
        let game = GoGame::new(ChallengeDifficulty::Novice);
        let moves = get_legal_moves(&game);
        assert!(moves.contains(&GoMove::Pass));
    }

    #[test]
    fn test_get_legal_moves_empty_board() {
        let game = GoGame::new(ChallengeDifficulty::Novice);
        let moves = get_legal_moves(&game);
        // 81 board positions + 1 pass
        assert_eq!(moves.len(), 82);
    }

    #[test]
    fn test_make_move_place() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        assert!(make_move(&mut game, GoMove::Place(4, 4)));
        assert_eq!(game.board[4][4], Some(Stone::Black));
        assert_eq!(game.current_player, Stone::White);
        assert_eq!(game.consecutive_passes, 0);
    }

    #[test]
    fn test_make_move_pass() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        assert!(make_move(&mut game, GoMove::Pass));
        assert_eq!(game.current_player, Stone::White);
        assert_eq!(game.consecutive_passes, 1);
    }

    #[test]
    fn test_two_passes_end_game() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        make_move(&mut game, GoMove::Pass);
        make_move(&mut game, GoMove::Pass);
        assert!(game.game_result.is_some());
    }

    #[test]
    fn test_capture_updates_count() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        // Set up capture
        game.board[4][4] = Some(Stone::White);
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);

        make_move(&mut game, GoMove::Place(4, 5)); // Black captures
        assert_eq!(game.captured_by_black, 1);
        assert!(game.board[4][4].is_none());
    }

    #[test]
    fn test_ko_point_set() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        // Classic ko shape
        // . B W .
        // B . B W
        // . B W .
        game.board[0][1] = Some(Stone::Black);
        game.board[0][2] = Some(Stone::White);
        game.board[1][0] = Some(Stone::Black);
        game.board[1][2] = Some(Stone::Black);
        game.board[1][3] = Some(Stone::White);
        game.board[2][1] = Some(Stone::Black);
        game.board[2][2] = Some(Stone::White);
        game.current_player = Stone::White;

        // White captures at (1,1)
        make_move(&mut game, GoMove::Place(1, 1));

        // Ko point should be set
        assert!(game.ko_point.is_some());
    }

    #[test]
    fn test_calculate_score_empty_board() {
        let board = [[None; BOARD_SIZE]; BOARD_SIZE];
        let (black, white) = calculate_score(&board);
        // Empty board = 0 + 0 stones, all territory contested, white gets 6 komi
        assert_eq!(black, 0);
        assert_eq!(white, 6);
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, MinigameInput::Cancel);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Second Esc confirms forfeit
        process_input(&mut game, MinigameInput::Cancel);
        assert!(!game.forfeit_pending);
        assert_eq!(game.game_result, Some(ChallengeResult::Forfeit));
    }

    #[test]
    fn test_forfeit_cancelled_by_other_key() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, MinigameInput::Cancel);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, MinigameInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_calculate_score_with_territory() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Create a small enclosed black territory in corner
        // Black wall from (0,2) to (2,2) and (2,0) to (2,2)
        // This encloses a 2x2 region (4 points of territory)
        board[0][2] = Some(Stone::Black);
        board[1][2] = Some(Stone::Black);
        board[2][0] = Some(Stone::Black);
        board[2][1] = Some(Stone::Black);
        board[2][2] = Some(Stone::Black);

        // Add white stones elsewhere to make rest of board contested
        board[4][4] = Some(Stone::White);
        board[5][5] = Some(Stone::White);

        let (black, white) = calculate_score(&board);
        // Black: 5 stones + 4 territory (positions (0,0), (0,1), (1,0), (1,1)) = 9
        // White: 2 stones + 0 territory + 6 komi = 8
        // Rest of board is contested (touches both colors or neither)
        assert_eq!(black, 9);
        assert_eq!(white, 8);
    }
}
