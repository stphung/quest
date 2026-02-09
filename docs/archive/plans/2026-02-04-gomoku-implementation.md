# Gomoku Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Gomoku (Five in a Row) as a third challenge minigame with minimax AI.

**Architecture:** Follow Morris pattern - `gomoku.rs` for data structures, `gomoku_logic.rs` for AI/rules, `ui/gomoku_scene.rs` for rendering. Integrate via challenge menu system.

**Tech Stack:** Rust, Ratatui, Serde, existing challenge infrastructure.

---

### Task 1: Create Gomoku Data Structures

**Files:**
- Create: `src/gomoku.rs`

**Step 1: Create the gomoku.rs file with core types**

```rust
//! Gomoku (Five in a Row) minigame data structures.
//!
//! 15x15 board, first to get 5+ in a row wins.

use serde::{Deserialize, Serialize};

/// Board size (15x15 standard)
pub const BOARD_SIZE: usize = 15;

/// Player in Gomoku
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Human,
    Ai,
}

impl Player {
    pub fn opponent(&self) -> Self {
        match self {
            Player::Human => Player::Ai,
            Player::Ai => Player::Human,
        }
    }
}

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GomokuDifficulty {
    Novice,     // depth 2
    Apprentice, // depth 3
    Journeyman, // depth 4
    Master,     // depth 5
}

impl GomokuDifficulty {
    pub const ALL: [GomokuDifficulty; 4] = [
        GomokuDifficulty::Novice,
        GomokuDifficulty::Apprentice,
        GomokuDifficulty::Journeyman,
        GomokuDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(GomokuDifficulty::Novice)
    }

    pub fn search_depth(&self) -> i32 {
        match self {
            Self::Novice => 2,
            Self::Apprentice => 3,
            Self::Journeyman => 4,
            Self::Master => 5,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }

    pub fn reward_prestige(&self) -> u32 {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 2,
            Self::Journeyman => 3,
            Self::Master => 5,
        }
    }
}

/// Game result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GomokuResult {
    Win,
    Loss,
    Draw,
}

/// Main game state
#[derive(Debug, Clone)]
pub struct GomokuGame {
    /// 15x15 board, None = empty
    pub board: [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    /// Current cursor position (row, col)
    pub cursor: (usize, usize),
    /// Whose turn it is
    pub current_player: Player,
    /// Difficulty level
    pub difficulty: GomokuDifficulty,
    /// Game result (None if game in progress)
    pub game_result: Option<GomokuResult>,
    /// Is AI currently thinking?
    pub ai_thinking: bool,
    /// Ticks spent thinking (for delayed AI move)
    pub ai_think_ticks: u32,
    /// Move history for display
    pub move_history: Vec<(usize, usize, Player)>,
    /// Last move position for highlighting
    pub last_move: Option<(usize, usize)>,
    /// Forfeit confirmation pending
    pub forfeit_pending: bool,
}

impl GomokuGame {
    pub fn new(difficulty: GomokuDifficulty) -> Self {
        Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            cursor: (BOARD_SIZE / 2, BOARD_SIZE / 2), // Center
            current_player: Player::Human, // Human plays first
            difficulty,
            game_result: None,
            ai_thinking: false,
            ai_think_ticks: 0,
            move_history: Vec::new(),
            last_move: None,
            forfeit_pending: false,
        }
    }

    /// Check if a position is valid and empty
    pub fn is_valid_move(&self, row: usize, col: usize) -> bool {
        row < BOARD_SIZE && col < BOARD_SIZE && self.board[row][col].is_none()
    }

    /// Place a stone at the given position
    pub fn place_stone(&mut self, row: usize, col: usize) -> bool {
        if !self.is_valid_move(row, col) || self.game_result.is_some() {
            return false;
        }
        self.board[row][col] = Some(self.current_player);
        self.move_history.push((row, col, self.current_player));
        self.last_move = Some((row, col));
        true
    }

    /// Switch to the other player's turn
    pub fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }

    /// Count stones on board
    pub fn stone_count(&self) -> (u32, u32) {
        let mut human = 0;
        let mut ai = 0;
        for row in &self.board {
            for cell in row {
                match cell {
                    Some(Player::Human) => human += 1,
                    Some(Player::Ai) => ai += 1,
                    None => {}
                }
            }
        }
        (human, ai)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        assert_eq!(game.cursor, (7, 7));
        assert_eq!(game.current_player, Player::Human);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_place_stone() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        assert!(game.place_stone(7, 7));
        assert_eq!(game.board[7][7], Some(Player::Human));
        assert!(!game.place_stone(7, 7)); // Can't place on occupied
    }

    #[test]
    fn test_difficulty_depths() {
        assert_eq!(GomokuDifficulty::Novice.search_depth(), 2);
        assert_eq!(GomokuDifficulty::Apprentice.search_depth(), 3);
        assert_eq!(GomokuDifficulty::Journeyman.search_depth(), 4);
        assert_eq!(GomokuDifficulty::Master.search_depth(), 5);
    }

    #[test]
    fn test_difficulty_rewards() {
        assert_eq!(GomokuDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(GomokuDifficulty::Apprentice.reward_prestige(), 2);
        assert_eq!(GomokuDifficulty::Journeyman.reward_prestige(), 3);
        assert_eq!(GomokuDifficulty::Master.reward_prestige(), 5);
    }

    #[test]
    fn test_move_cursor() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.move_cursor(-1, 0); // Up
        assert_eq!(game.cursor, (6, 7));
        game.cursor = (0, 0);
        game.move_cursor(-1, -1); // Should clamp
        assert_eq!(game.cursor, (0, 0));
    }

    #[test]
    fn test_player_opponent() {
        assert_eq!(Player::Human.opponent(), Player::Ai);
        assert_eq!(Player::Ai.opponent(), Player::Human);
    }
}
```

**Step 2: Add module to lib.rs**

In `src/lib.rs`, add after `mod game_state;`:
```rust
pub mod gomoku;
```

**Step 3: Run tests**

Run: `cargo test gomoku --lib`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/gomoku.rs src/lib.rs
git commit -m "feat(gomoku): add core data structures"
```

---

### Task 2: Create Win Detection Logic

**Files:**
- Create: `src/gomoku_logic.rs`

**Step 1: Create gomoku_logic.rs with win detection**

```rust
//! Gomoku game logic and AI.

use crate::gomoku::{GomokuGame, GomokuResult, Player, BOARD_SIZE};

/// Directions to check for lines: (row_delta, col_delta)
const DIRECTIONS: [(i32, i32); 4] = [
    (0, 1),  // Horizontal
    (1, 0),  // Vertical
    (1, 1),  // Diagonal down-right
    (1, -1), // Diagonal down-left
];

/// Check if placing at (row, col) creates 5+ in a row for the given player.
/// Assumes the stone is already placed.
pub fn check_win(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE], row: usize, col: usize, player: Player) -> bool {
    for (dr, dc) in DIRECTIONS {
        let count = count_line(board, row, col, dr, dc, player);
        if count >= 5 {
            return true;
        }
    }
    false
}

/// Count consecutive stones in both directions from (row, col).
fn count_line(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> u32 {
    let mut count = 1; // Count the center stone

    // Count in positive direction
    count += count_direction(board, row, col, dr, dc, player);
    // Count in negative direction
    count += count_direction(board, row, col, -dr, -dc, player);

    count
}

/// Count consecutive stones in one direction from (row, col), excluding center.
fn count_direction(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> u32 {
    let mut count = 0;
    let mut r = row as i32 + dr;
    let mut c = col as i32 + dc;

    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        if board[r as usize][c as usize] == Some(player) {
            count += 1;
            r += dr;
            c += dc;
        } else {
            break;
        }
    }
    count
}

/// Check if the board is full (draw condition).
pub fn is_board_full(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE]) -> bool {
    for row in board {
        for cell in row {
            if cell.is_none() {
                return false;
            }
        }
    }
    true
}

/// Process a human move at the cursor position.
pub fn process_human_move(game: &mut GomokuGame) -> bool {
    if game.game_result.is_some() || game.current_player != Player::Human {
        return false;
    }

    let (row, col) = game.cursor;
    if !game.place_stone(row, col) {
        return false;
    }

    // Check for win
    if check_win(&game.board, row, col, Player::Human) {
        game.game_result = Some(GomokuResult::Win);
        return true;
    }

    // Check for draw
    if is_board_full(&game.board) {
        game.game_result = Some(GomokuResult::Draw);
        return true;
    }

    // Switch to AI turn
    game.switch_player();
    game.ai_thinking = true;
    game.ai_think_ticks = 0;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(game: &mut GomokuGame, row: usize, col: usize, player: Player) {
        game.board[row][col] = Some(player);
    }

    #[test]
    fn test_horizontal_win() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for c in 0..5 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(check_win(&game.board, 7, 2, Player::Human));
    }

    #[test]
    fn test_vertical_win() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for r in 0..5 {
            place(&mut game, r, 7, Player::Human);
        }
        assert!(check_win(&game.board, 2, 7, Player::Human));
    }

    #[test]
    fn test_diagonal_win() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for i in 0..5 {
            place(&mut game, i, i, Player::Human);
        }
        assert!(check_win(&game.board, 2, 2, Player::Human));
    }

    #[test]
    fn test_no_win_with_four() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for c in 0..4 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(!check_win(&game.board, 7, 2, Player::Human));
    }

    #[test]
    fn test_six_in_row_wins() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for c in 0..6 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(check_win(&game.board, 7, 3, Player::Human));
    }

    #[test]
    fn test_board_not_full() {
        let game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        assert!(!is_board_full(&game.board));
    }
}
```

**Step 2: Add module to lib.rs**

In `src/lib.rs`, add after `pub mod gomoku;`:
```rust
pub mod gomoku_logic;
```

**Step 3: Run tests**

Run: `cargo test gomoku_logic --lib`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/gomoku_logic.rs src/lib.rs
git commit -m "feat(gomoku): add win detection logic"
```

---

### Task 3: Create Board Evaluation for AI

**Files:**
- Modify: `src/gomoku_logic.rs`

**Step 1: Add pattern scoring constants and evaluation function**

Add to `src/gomoku_logic.rs` after the existing code:

```rust
// === AI Evaluation ===

/// Score values for different patterns
const SCORE_FIVE: i32 = 100_000;
const SCORE_OPEN_FOUR: i32 = 10_000;
const SCORE_CLOSED_FOUR: i32 = 1_000;
const SCORE_OPEN_THREE: i32 = 500;
const SCORE_CLOSED_THREE: i32 = 100;
const SCORE_OPEN_TWO: i32 = 50;
const SCORE_CENTER_BONUS: i32 = 5;

/// Evaluate the board from AI's perspective.
/// Positive = good for AI, negative = good for Human.
pub fn evaluate_board(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE]) -> i32 {
    let mut score = 0;

    // Evaluate all lines on the board
    score += evaluate_all_lines(board, Player::Ai);
    score -= evaluate_all_lines(board, Player::Human);

    // Small bonus for center control
    let center = BOARD_SIZE / 2;
    for r in center.saturating_sub(2)..=(center + 2).min(BOARD_SIZE - 1) {
        for c in center.saturating_sub(2)..=(center + 2).min(BOARD_SIZE - 1) {
            if board[r][c] == Some(Player::Ai) {
                score += SCORE_CENTER_BONUS;
            } else if board[r][c] == Some(Player::Human) {
                score -= SCORE_CENTER_BONUS;
            }
        }
    }

    score
}

/// Evaluate all lines for a player, summing pattern scores.
fn evaluate_all_lines(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE], player: Player) -> i32 {
    let mut score = 0;

    // Check all rows
    for r in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, r, 0, 0, 1, player);
    }

    // Check all columns
    for c in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, 0, c, 1, 0, player);
    }

    // Check diagonals (down-right)
    for start in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, start, 0, 1, 1, player);
        if start > 0 {
            score += evaluate_line_segment(board, 0, start, 1, 1, player);
        }
    }

    // Check diagonals (down-left)
    for start in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, start, BOARD_SIZE - 1, 1, -1, player);
        if start > 0 {
            score += evaluate_line_segment(board, 0, BOARD_SIZE - 1 - start, 1, -1, player);
        }
    }

    score
}

/// Evaluate a line segment starting at (r, c) going in direction (dr, dc).
fn evaluate_line_segment(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    start_r: usize,
    start_c: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> i32 {
    let mut score = 0;
    let mut r = start_r as i32;
    let mut c = start_c as i32;

    // Collect the line
    let mut line = Vec::new();
    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        line.push(board[r as usize][c as usize]);
        r += dr;
        c += dc;
    }

    // Score windows of 5 in this line
    if line.len() >= 5 {
        for window in line.windows(5) {
            score += score_window(window, player);
        }
    }

    score
}

/// Score a window of 5 cells for patterns.
fn score_window(window: &[Option<Player>], player: Player) -> i32 {
    let own = window.iter().filter(|&&c| c == Some(player)).count();
    let empty = window.iter().filter(|&&c| c.is_none()).count();
    let opponent = 5 - own - empty;

    // If opponent has stones in this window, we can't complete it
    if opponent > 0 {
        return 0;
    }

    match own {
        5 => SCORE_FIVE,
        4 if empty == 1 => SCORE_CLOSED_FOUR, // One empty = closed four
        3 if empty == 2 => SCORE_OPEN_THREE,
        2 if empty == 3 => SCORE_OPEN_TWO,
        _ => 0,
    }
}

#[cfg(test)]
mod eval_tests {
    use super::*;
    use crate::gomoku::GomokuDifficulty;

    #[test]
    fn test_evaluate_empty_board() {
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        let score = evaluate_board(&game.board);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_evaluate_ai_advantage() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // AI has 3 in a row with space
        game.board[7][7] = Some(Player::Ai);
        game.board[7][8] = Some(Player::Ai);
        game.board[7][9] = Some(Player::Ai);
        let score = evaluate_board(&game.board);
        assert!(score > 0, "AI should have positive score");
    }

    #[test]
    fn test_evaluate_human_advantage() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Human has 3 in a row with space
        game.board[7][7] = Some(Player::Human);
        game.board[7][8] = Some(Player::Human);
        game.board[7][9] = Some(Player::Human);
        let score = evaluate_board(&game.board);
        assert!(score < 0, "Human advantage should give negative score");
    }
}
```

**Step 2: Run tests**

Run: `cargo test gomoku --lib`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/gomoku_logic.rs
git commit -m "feat(gomoku): add board evaluation for AI"
```

---

### Task 4: Implement Minimax AI

**Files:**
- Modify: `src/gomoku_logic.rs`

**Step 1: Add minimax with alpha-beta pruning**

Add to `src/gomoku_logic.rs`:

```rust
// === Minimax AI ===

use rand::seq::SliceRandom;
use rand::Rng;

/// Get candidate moves (positions near existing stones).
fn get_candidate_moves(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE]) -> Vec<(usize, usize)> {
    let mut candidates = std::collections::HashSet::new();
    let mut has_stones = false;

    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if board[r][c].is_some() {
                has_stones = true;
                // Add empty positions within 2 spaces
                for dr in -2i32..=2 {
                    for dc in -2i32..=2 {
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                            let nr = nr as usize;
                            let nc = nc as usize;
                            if board[nr][nc].is_none() {
                                candidates.insert((nr, nc));
                            }
                        }
                    }
                }
            }
        }
    }

    // If no stones on board, return center area
    if !has_stones {
        let center = BOARD_SIZE / 2;
        return vec![(center, center)];
    }

    candidates.into_iter().collect()
}

/// Minimax with alpha-beta pruning.
fn minimax(
    board: &mut [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    last_move: Option<(usize, usize)>,
) -> i32 {
    // Check for terminal state
    if let Some((r, c)) = last_move {
        let last_player = if maximizing { Player::Human } else { Player::Ai };
        if check_win(board, r, c, last_player) {
            return if maximizing { -SCORE_FIVE } else { SCORE_FIVE };
        }
    }

    if depth == 0 {
        return evaluate_board(board);
    }

    let candidates = get_candidate_moves(board);
    if candidates.is_empty() {
        return 0; // Draw
    }

    if maximizing {
        let mut max_eval = i32::MIN;
        for (r, c) in candidates {
            board[r][c] = Some(Player::Ai);
            let eval = minimax(board, depth - 1, alpha, beta, false, Some((r, c)));
            board[r][c] = None;
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for (r, c) in candidates {
            board[r][c] = Some(Player::Human);
            let eval = minimax(board, depth - 1, alpha, beta, true, Some((r, c)));
            board[r][c] = None;
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

/// Find the best move for AI using minimax.
pub fn find_best_move<R: Rng>(game: &GomokuGame, rng: &mut R) -> Option<(usize, usize)> {
    let mut board = game.board;
    let depth = game.difficulty.search_depth();
    let candidates = get_candidate_moves(&board);

    if candidates.is_empty() {
        return None;
    }

    // First check for immediate winning move
    for &(r, c) in &candidates {
        board[r][c] = Some(Player::Ai);
        if check_win(&board, r, c, Player::Ai) {
            return Some((r, c));
        }
        board[r][c] = None;
    }

    // Then check for blocking opponent's winning move
    for &(r, c) in &candidates {
        board[r][c] = Some(Player::Human);
        if check_win(&board, r, c, Player::Human) {
            board[r][c] = None;
            return Some((r, c));
        }
        board[r][c] = None;
    }

    // Use minimax for other moves
    let mut best_moves = Vec::new();
    let mut best_score = i32::MIN;

    for (r, c) in candidates {
        board[r][c] = Some(Player::Ai);
        let score = minimax(&mut board, depth - 1, i32::MIN, i32::MAX, false, Some((r, c)));
        board[r][c] = None;

        if score > best_score {
            best_score = score;
            best_moves.clear();
            best_moves.push((r, c));
        } else if score == best_score {
            best_moves.push((r, c));
        }
    }

    // Randomly pick among equally good moves
    best_moves.choose(rng).copied()
}

/// Process AI thinking (called each tick).
pub fn process_ai_thinking<R: Rng>(game: &mut GomokuGame, rng: &mut R) {
    if !game.ai_thinking || game.game_result.is_some() {
        return;
    }

    // Add small delay for visual feedback (5-15 ticks = 0.5-1.5 seconds)
    game.ai_think_ticks += 1;
    let min_ticks = 5 + game.difficulty.search_depth() as u32 * 2;
    if game.ai_think_ticks < min_ticks {
        return;
    }

    // Find and make move
    if let Some((r, c)) = find_best_move(game, rng) {
        game.board[r][c] = Some(Player::Ai);
        game.move_history.push((r, c, Player::Ai));
        game.last_move = Some((r, c));

        // Check for AI win
        if check_win(&game.board, r, c, Player::Ai) {
            game.game_result = Some(GomokuResult::Loss);
        } else if is_board_full(&game.board) {
            game.game_result = Some(GomokuResult::Draw);
        } else {
            game.switch_player();
        }
    }

    game.ai_thinking = false;
    game.ai_think_ticks = 0;
}

#[cfg(test)]
mod ai_tests {
    use super::*;
    use crate::gomoku::GomokuDifficulty;

    #[test]
    fn test_ai_takes_winning_move() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // AI has 4 in a row, should complete it
        game.board[7][3] = Some(Player::Ai);
        game.board[7][4] = Some(Player::Ai);
        game.board[7][5] = Some(Player::Ai);
        game.board[7][6] = Some(Player::Ai);

        let mut rng = rand::thread_rng();
        let best = find_best_move(&game, &mut rng);
        assert!(best == Some((7, 2)) || best == Some((7, 7)), "AI should complete 5 in a row");
    }

    #[test]
    fn test_ai_blocks_human_win() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Human has 4 in a row
        game.board[7][3] = Some(Player::Human);
        game.board[7][4] = Some(Player::Human);
        game.board[7][5] = Some(Player::Human);
        game.board[7][6] = Some(Player::Human);

        let mut rng = rand::thread_rng();
        let best = find_best_move(&game, &mut rng);
        assert!(best == Some((7, 2)) || best == Some((7, 7)), "AI should block human win");
    }

    #[test]
    fn test_get_candidates_empty_board() {
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        let candidates = get_candidate_moves(&game.board);
        assert_eq!(candidates, vec![(7, 7)], "Empty board should suggest center");
    }

    #[test]
    fn test_get_candidates_near_stones() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.board[7][7] = Some(Player::Human);
        let candidates = get_candidate_moves(&game.board);
        assert!(!candidates.is_empty());
        assert!(!candidates.contains(&(7, 7)), "Occupied position should not be candidate");
    }
}
```

**Step 2: Run tests**

Run: `cargo test gomoku --lib`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/gomoku_logic.rs
git commit -m "feat(gomoku): implement minimax AI with alpha-beta pruning"
```

---

### Task 5: Create UI Scene

**Files:**
- Create: `src/ui/gomoku_scene.rs`

**Step 1: Create the UI rendering**

```rust
//! Gomoku game UI rendering.

use crate::gomoku::{GomokuDifficulty, GomokuGame, Player, BOARD_SIZE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the Gomoku game scene.
pub fn render_gomoku_scene(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    // Split: Board on left, help panel on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(32),    // Board (15*2 + borders)
            Constraint::Length(22), // Help panel
        ])
        .split(area);

    render_board(frame, chunks[0], game);
    render_help_panel(frame, chunks[1], game);

    // Game over overlay
    if game.game_result.is_some() {
        render_game_over_overlay(frame, area, game);
    }
}

fn render_board(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    let block = Block::default()
        .title(" Gomoku ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate centering offset
    let board_height = BOARD_SIZE as u16;
    let board_width = (BOARD_SIZE * 2 - 1) as u16; // "● " format
    let y_offset = inner.y + (inner.height.saturating_sub(board_height)) / 2;
    let x_offset = inner.x + (inner.width.saturating_sub(board_width)) / 2;

    // Colors
    let human_color = Color::White;
    let ai_color = Color::LightRed;
    let cursor_color = Color::Yellow;
    let last_move_color = Color::Green;
    let empty_color = Color::DarkGray;

    // Draw board
    for row in 0..BOARD_SIZE {
        let mut spans = Vec::new();
        for col in 0..BOARD_SIZE {
            let is_cursor = game.cursor == (row, col);
            let is_last_move = game.last_move == Some((row, col));

            let (symbol, style) = match game.board[row][col] {
                Some(Player::Human) => {
                    let base_style = Style::default().fg(human_color).add_modifier(Modifier::BOLD);
                    if is_cursor {
                        ("●", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("●", base_style.fg(last_move_color))
                    } else {
                        ("●", base_style)
                    }
                }
                Some(Player::Ai) => {
                    let base_style = Style::default().fg(ai_color).add_modifier(Modifier::BOLD);
                    if is_cursor {
                        ("●", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("●", base_style.fg(last_move_color))
                    } else {
                        ("●", base_style)
                    }
                }
                None => {
                    if is_cursor {
                        ("□", Style::default().fg(cursor_color).add_modifier(Modifier::BOLD))
                    } else {
                        ("·", Style::default().fg(empty_color))
                    }
                }
            };

            spans.push(Span::styled(symbol, style));
            if col < BOARD_SIZE - 1 {
                spans.push(Span::raw(" "));
            }
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(x_offset, y_offset + row as u16, board_width, 1),
        );
    }
}

fn render_help_panel(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "RULES",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Place stones. First",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "to get 5 in a row",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "wins.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
    ];

    // Difficulty
    lines.push(Line::from(vec![
        Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
        Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(""));

    // Status
    let status = if game.ai_thinking {
        Span::styled("AI thinking...", Style::default().fg(Color::Yellow))
    } else if game.forfeit_pending {
        Span::styled("Forfeit? (Y/N)", Style::default().fg(Color::LightRed))
    } else if game.current_player == Player::Human {
        Span::styled("Your turn", Style::default().fg(Color::Green))
    } else {
        Span::styled("", Style::default())
    };
    lines.push(Line::from(status));
    lines.push(Line::from(""));

    // Controls
    lines.push(Line::from(Span::styled(
        "[Arrows] Move",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "[Enter] Place",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "[Esc] Forfeit",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

fn render_game_over_overlay(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    let result = game.game_result.as_ref().unwrap();
    let (title, color) = match result {
        crate::gomoku::GomokuResult::Win => ("Victory!", Color::Green),
        crate::gomoku::GomokuResult::Loss => ("Defeat", Color::Red),
        crate::gomoku::GomokuResult::Draw => ("Draw", Color::Yellow),
    };

    let reward_text = match result {
        crate::gomoku::GomokuResult::Win => {
            format!("+{} Prestige Ranks", game.difficulty.reward_prestige())
        }
        _ => "No reward".to_string(),
    };

    // Center overlay
    let width = 24;
    let height = 6;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(reward_text, Style::default().fg(Color::White))),
        Line::from(Span::styled(
            "[Any key to continue]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}
```

**Step 2: Add module to ui/mod.rs**

In `src/ui/mod.rs`, add:
```rust
pub mod gomoku_scene;
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/ui/gomoku_scene.rs src/ui/mod.rs
git commit -m "feat(gomoku): add UI scene rendering"
```

---

### Task 6: Integrate with Challenge System

**Files:**
- Modify: `src/challenge_menu.rs`
- Modify: `src/game_state.rs`

**Step 1: Add Gomoku to ChallengeType**

In `src/challenge_menu.rs`, update `ChallengeType` enum:

```rust
/// Extensible enum for different minigame challenges
#[derive(Debug, Clone, PartialEq)]
pub enum ChallengeType {
    Chess,
    Morris,
    Gomoku,
}
```

**Step 2: Add Gomoku to CHALLENGE_TABLE**

In `src/challenge_menu.rs`, update `CHALLENGE_TABLE`:

```rust
const CHALLENGE_TABLE: &[ChallengeWeight] = &[
    ChallengeWeight {
        challenge_type: ChallengeType::Chess,
        weight: 33,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Morris,
        weight: 33,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Gomoku,
        weight: 34,
    },
];
```

**Step 3: Add create_challenge case for Gomoku**

In `src/challenge_menu.rs`, find the `create_challenge` function and add:

```rust
        ChallengeType::Gomoku => PendingChallenge {
            challenge_type: ChallengeType::Gomoku,
            title: "Gomoku".to_string(),
            icon: "◎",
            description: "A wandering strategist places a worn board before you. \
                \"Five stones in a row,\" they explain. \"Simple rules, deep tactics.\""
                .to_string(),
        },
```

**Step 4: Add active_gomoku to GameState**

In `src/game_state.rs`, add the import at the top:

```rust
use crate::gomoku::GomokuGame;
```

Then add the field after `active_morris`:

```rust
    /// Active gomoku game (transient, not saved)
    #[serde(skip)]
    pub active_gomoku: Option<GomokuGame>,
```

And initialize it in `GameState::new()`:

```rust
            active_gomoku: None,
```

**Step 5: Build to verify**

Run: `cargo build`
Expected: Compiles (may have warnings about unused)

**Step 6: Commit**

```bash
git add src/challenge_menu.rs src/game_state.rs
git commit -m "feat(gomoku): integrate with challenge system"
```

---

### Task 7: Add Input Handling and Game Loop Integration

**Files:**
- Modify: `src/main.rs`

**Step 1: Add gomoku imports**

At the top of `src/main.rs`, add:

```rust
use gomoku::{GomokuDifficulty, GomokuResult};
use gomoku_logic::{process_ai_thinking as process_gomoku_ai, process_human_move as process_gomoku_move};
```

**Step 2: Add Gomoku to UI rendering priority**

Find the section in `ui/mod.rs` `draw_ui_with_update` that checks for active games. Add Gomoku check before Morris:

```rust
    if let Some(ref game) = game_state.active_gomoku {
        gomoku_scene::render_gomoku_scene(frame, chunks[1], game);
    } else if let Some(ref game) = game_state.active_morris {
```

**Step 3: Add Gomoku input handling in main.rs**

In the main game loop's input handling section, add Gomoku handling after Morris handling (search for `active_morris` input handling block):

```rust
                            // Handle active Gomoku game input
                            if let Some(ref mut gomoku_game) = state.active_gomoku {
                                if gomoku_game.game_result.is_some() {
                                    // Any key dismisses result
                                    let old_prestige = state.prestige_rank;
                                    if let Some(result) = &gomoku_game.game_result {
                                        match result {
                                            GomokuResult::Win => {
                                                let gained = gomoku_game.difficulty.reward_prestige();
                                                state.prestige_rank += gained;
                                                state.combat_state.add_log_entry(
                                                    format!("◎ Victory! +{} Prestige Ranks (P{} → P{})",
                                                        gained, old_prestige, state.prestige_rank),
                                                    false, true,
                                                );
                                            }
                                            GomokuResult::Loss => {
                                                state.combat_state.add_log_entry(
                                                    "◎ The strategist nods respectfully and departs.".to_string(),
                                                    false, true,
                                                );
                                            }
                                            GomokuResult::Draw => {
                                                state.combat_state.add_log_entry(
                                                    "◎ A rare draw. The strategist seems impressed.".to_string(),
                                                    false, true,
                                                );
                                            }
                                        }
                                    }
                                    state.active_gomoku = None;
                                    continue;
                                }

                                // Handle forfeit confirmation
                                if gomoku_game.forfeit_pending {
                                    match key_event.code {
                                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                                            gomoku_game.game_result = Some(GomokuResult::Loss);
                                        }
                                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                            gomoku_game.forfeit_pending = false;
                                        }
                                        _ => {}
                                    }
                                    continue;
                                }

                                // Normal game input
                                if !gomoku_game.ai_thinking {
                                    match key_event.code {
                                        KeyCode::Up => gomoku_game.move_cursor(-1, 0),
                                        KeyCode::Down => gomoku_game.move_cursor(1, 0),
                                        KeyCode::Left => gomoku_game.move_cursor(0, -1),
                                        KeyCode::Right => gomoku_game.move_cursor(0, 1),
                                        KeyCode::Enter => {
                                            process_gomoku_move(gomoku_game);
                                        }
                                        KeyCode::Esc => {
                                            gomoku_game.forfeit_pending = true;
                                        }
                                        _ => {}
                                    }
                                }
                                continue;
                            }
```

**Step 4: Add Gomoku AI tick processing**

In `game_tick` function, add after Morris AI processing:

```rust
    // Process Gomoku AI thinking
    if let Some(ref mut gomoku_game) = game_state.active_gomoku {
        let mut rng = rand::thread_rng();
        process_gomoku_ai(gomoku_game, &mut rng);
    }
```

**Step 5: Add Gomoku to challenge menu start logic**

Find where challenges are started (search for `start_morris_game` or `start_chess_game`). Add Gomoku case:

```rust
                                        ChallengeType::Gomoku => {
                                            let difficulty = GomokuDifficulty::from_index(
                                                state.challenge_menu.selected_difficulty
                                            );
                                            state.active_gomoku = Some(
                                                crate::gomoku::GomokuGame::new(difficulty)
                                            );
                                            state.challenge_menu.close();
                                        }
```

**Step 6: Build and test**

Run: `cargo build`
Expected: Compiles

**Step 7: Commit**

```bash
git add src/main.rs src/ui/mod.rs
git commit -m "feat(gomoku): add input handling and game loop integration"
```

---

### Task 8: Add to Debug Menu

**Files:**
- Modify: `src/debug_menu.rs`

**Step 1: Add Gomoku option**

Update `DEBUG_OPTIONS`:

```rust
pub const DEBUG_OPTIONS: &[&str] = &[
    "Trigger Dungeon",
    "Trigger Fishing",
    "Trigger Chess Challenge",
    "Trigger Morris Challenge",
    "Trigger Gomoku Challenge",
];
```

**Step 2: Add trigger function**

Add after `trigger_morris_challenge`:

```rust
fn trigger_gomoku_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Gomoku) {
        return "Gomoku challenge already pending!";
    }
    state.challenge_menu.add_challenge(PendingChallenge {
        challenge_type: ChallengeType::Gomoku,
        title: "Gomoku".to_string(),
        icon: "◎",
        description: "A wandering strategist places a worn board before you. \
            \"Five stones in a row,\" they explain. \"Simple rules, deep tactics.\""
            .to_string(),
    });
    "Gomoku challenge added!"
}
```

**Step 3: Update trigger_selected match**

```rust
    pub fn trigger_selected(&mut self, state: &mut GameState) -> &'static str {
        let msg = match self.selected_index {
            0 => trigger_dungeon(state),
            1 => trigger_fishing(state),
            2 => trigger_chess_challenge(state),
            3 => trigger_morris_challenge(state),
            4 => trigger_gomoku_challenge(state),
            _ => "Unknown option",
        };
        self.close();
        msg
    }
```

**Step 4: Build and test**

Run: `cargo build`
Run: `cargo run -- --debug`
Test: Press backtick, select Gomoku, verify it works

**Step 5: Commit**

```bash
git add src/debug_menu.rs
git commit -m "feat(gomoku): add to debug menu"
```

---

### Task 9: Add Gomoku Difficulty Selector to Challenge Menu Scene

**Files:**
- Modify: `src/ui/challenge_menu_scene.rs`

**Step 1: Add Gomoku difficulty renderer**

Find `render_detail_view` function. Add Gomoku case in the difficulty selector match:

```rust
        ChallengeType::Gomoku => {
            render_gomoku_difficulty_selector(frame, chunks[2], menu.selected_difficulty);
        }
```

**Step 2: Add the renderer function**

Add after `render_morris_difficulty_selector`:

```rust
fn render_gomoku_difficulty_selector(frame: &mut Frame, area: Rect, selected: usize) {
    let title = Paragraph::new("Select difficulty:").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, Rect { height: 1, ..area });

    let options_area = Rect {
        y: area.y + 1,
        height: area.height.saturating_sub(1),
        ..area
    };

    let items: Vec<ListItem> = crate::gomoku::GomokuDifficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, diff)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "> " } else { "  " };

            let reward = diff.reward_prestige();
            let reward_text = if reward == 1 {
                "Win: +1 Prestige Rank".to_string()
            } else {
                format!("Win: +{} Prestige Ranks", reward)
            };

            let prefix_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let reward_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            let spans = vec![
                Span::styled(prefix, prefix_style),
                Span::styled(format!("{:<12}", diff.name()), name_style),
                Span::styled(reward_text, reward_style),
            ];

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, options_area);
}
```

**Step 3: Build and verify**

Run: `cargo build`

**Step 4: Commit**

```bash
git add src/ui/challenge_menu_scene.rs
git commit -m "feat(gomoku): add difficulty selector to challenge menu"
```

---

### Task 10: Final Testing and Cleanup

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

**Step 3: Run formatter**

Run: `cargo fmt`

**Step 4: Full CI check**

Run: `make check`
Expected: All checks pass

**Step 5: Manual testing**

Run: `cargo run -- --debug`
- Trigger Gomoku challenge
- Play a game at each difficulty
- Verify win/loss/forfeit work
- Check prestige rewards apply

**Step 6: Final commit**

```bash
git add -A
git commit -m "chore(gomoku): cleanup and verify all tests pass"
```

---

## Summary

This plan implements Gomoku in 10 tasks:
1. Core data structures
2. Win detection
3. Board evaluation
4. Minimax AI
5. UI rendering
6. Challenge system integration
7. Main loop integration
8. Debug menu
9. Difficulty selector
10. Testing and cleanup

Each task is atomic and can be committed independently.
