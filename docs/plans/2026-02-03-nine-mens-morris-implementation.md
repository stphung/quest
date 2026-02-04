# Nine Men's Morris Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Nine Men's Morris as a playable challenge type, following the same patterns as chess.

**Architecture:** Three new files (morris.rs, morris_logic.rs, ui/morris_scene.rs) plus integrations into challenge_menu.rs, game_state.rs, main.rs, and ui/mod.rs.

**Tech Stack:** Pure Rust, Ratatui for UI, minimax AI with alpha-beta pruning.

---

## Task 1: Create Morris Data Structures

**Files:**
- Create: `src/morris.rs`

**Step 1: Create the morris.rs file with all data structures**

```rust
//! Nine Men's Morris minigame data structures and state management.

use serde::{Deserialize, Serialize};

/// The 24 board positions indexed 0-23
/// ```text
/// 0-----------1-----------2
/// |           |           |
/// |   3-------4-------5   |
/// |   |       |       |   |
/// |   |   6---7---8   |   |
/// |   |   |       |   |   |
/// 9---10--11      12--13--14
/// |   |   |       |   |   |
/// |   |   15--16--17  |   |
/// |   |       |       |   |
/// |   18------19------20  |
/// |           |           |
/// 21----------22----------23
/// ```

/// Which player owns a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Human,
    Ai,
}

/// AI difficulty levels (same as chess)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorrisDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

impl MorrisDifficulty {
    pub const ALL: [MorrisDifficulty; 4] = [
        MorrisDifficulty::Novice,
        MorrisDifficulty::Apprentice,
        MorrisDifficulty::Journeyman,
        MorrisDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(MorrisDifficulty::Novice)
    }

    pub fn search_depth(&self) -> i32 {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 1,
            Self::Journeyman => 2,
            Self::Master => 3,
        }
    }

    pub fn random_move_chance(&self) -> f64 {
        match self {
            Self::Novice => 0.5,
            _ => 0.0,
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

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }
}

/// Result of a completed Morris game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorrisResult {
    Win,
    Loss,
    Forfeit,
}

/// Current phase of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorrisPhase {
    /// Players are placing their 9 pieces
    Placing,
    /// Normal movement along lines
    Moving,
    /// Player with 3 pieces can move anywhere (flying)
    Flying,
}

/// A move in Morris
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorrisMove {
    /// Place a piece at position (during Placing phase)
    Place(usize),
    /// Move a piece from one position to another
    Move { from: usize, to: usize },
    /// Capture an opponent's piece at position
    Capture(usize),
}

/// The 16 possible mills (three-in-a-row lines)
pub const MILLS: [[usize; 3]; 16] = [
    // Outer square
    [0, 1, 2],
    [2, 14, 23],
    [21, 22, 23],
    [0, 9, 21],
    // Middle square
    [3, 4, 5],
    [5, 13, 20],
    [18, 19, 20],
    [3, 10, 18],
    // Inner square
    [6, 7, 8],
    [8, 12, 17],
    [15, 16, 17],
    [6, 11, 15],
    // Connecting lines (spokes)
    [1, 4, 7],
    [12, 13, 14],
    [16, 19, 22],
    [9, 10, 11],
];

/// Adjacency list for each position
pub const ADJACENCIES: [&[usize]; 24] = [
    &[1, 9],           // 0
    &[0, 2, 4],        // 1
    &[1, 14],          // 2
    &[4, 10],          // 3
    &[1, 3, 5, 7],     // 4
    &[4, 13],          // 5
    &[7, 11],          // 6
    &[4, 6, 8],        // 7
    &[7, 12],          // 8
    &[0, 10, 21],      // 9
    &[3, 9, 11, 18],   // 10
    &[6, 10, 15],      // 11
    &[8, 13, 17],      // 12
    &[5, 12, 14, 20],  // 13
    &[2, 13, 23],      // 14
    &[11, 16],         // 15
    &[15, 17, 19],     // 16
    &[12, 16],         // 17
    &[10, 19],         // 18
    &[16, 18, 20, 22], // 19
    &[13, 19],         // 20
    &[9, 22],          // 21
    &[19, 21, 23],     // 22
    &[14, 22],         // 23
];

/// Active Morris game session
#[derive(Debug, Clone)]
pub struct MorrisGame {
    /// Board state: 24 positions, each None or Some(Player)
    pub board: [Option<Player>; 24],
    /// Current game phase
    pub phase: MorrisPhase,
    /// Pieces remaining to place (human, ai)
    pub pieces_to_place: (u8, u8),
    /// Pieces on board (human, ai)
    pub pieces_on_board: (u8, u8),
    /// Selected difficulty
    pub difficulty: MorrisDifficulty,
    /// Cursor position (0-23)
    pub cursor: usize,
    /// Selected piece position for moving
    pub selected_position: Option<usize>,
    /// True when player must capture after forming a mill
    pub must_capture: bool,
    /// Whose turn it is
    pub current_player: Player,
    /// Game result when finished
    pub game_result: Option<MorrisResult>,
    /// Forfeit confirmation pending
    pub forfeit_pending: bool,
    /// AI is thinking
    pub ai_thinking: bool,
    /// AI thinking tick counter
    pub ai_think_ticks: u32,
    /// Target ticks before AI moves
    pub ai_think_target: u32,
    /// Pending AI move
    pub ai_pending_move: Option<MorrisMove>,
}

impl MorrisGame {
    pub fn new(difficulty: MorrisDifficulty) -> Self {
        Self {
            board: [None; 24],
            phase: MorrisPhase::Placing,
            pieces_to_place: (9, 9),
            pieces_on_board: (0, 0),
            difficulty,
            cursor: 0,
            selected_position: None,
            must_capture: false,
            current_player: Player::Human,
            game_result: None,
            forfeit_pending: false,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_move: None,
        }
    }

    /// Check if a position is part of a completed mill for the given player
    pub fn is_in_mill(&self, pos: usize, player: Player) -> bool {
        MILLS.iter().any(|mill| {
            mill.contains(&pos)
                && mill
                    .iter()
                    .all(|&p| self.board[p] == Some(player))
        })
    }

    /// Check if placing/moving to a position completes a mill
    pub fn forms_mill(&self, pos: usize, player: Player) -> bool {
        MILLS.iter().any(|mill| {
            mill.contains(&pos)
                && mill.iter().all(|&p| {
                    if p == pos {
                        true // The position we're checking
                    } else {
                        self.board[p] == Some(player)
                    }
                })
        })
    }

    /// Count pieces for a player
    pub fn count_pieces(&self, player: Player) -> u8 {
        match player {
            Player::Human => self.pieces_on_board.0,
            Player::Ai => self.pieces_on_board.1,
        }
    }

    /// Get pieces remaining to place for a player
    pub fn pieces_to_place_for(&self, player: Player) -> u8 {
        match player {
            Player::Human => self.pieces_to_place.0,
            Player::Ai => self.pieces_to_place.1,
        }
    }

    /// Check if a player can fly (has exactly 3 pieces and placing is done)
    pub fn can_fly(&self, player: Player) -> bool {
        self.phase != MorrisPhase::Placing && self.count_pieces(player) == 3
    }

    /// Move cursor to adjacent position in given direction
    pub fn move_cursor(&mut self, direction: CursorDirection) {
        // Find best adjacent position in the given direction
        let current = self.cursor;
        let positions = get_position_coords();
        let (cx, cy) = positions[current];

        let mut best_pos = current;
        let mut best_dist = i32::MAX;

        for &adj in ADJACENCIES[current] {
            let (ax, ay) = positions[adj];
            let matches_direction = match direction {
                CursorDirection::Up => ay < cy,
                CursorDirection::Down => ay > cy,
                CursorDirection::Left => ax < cx,
                CursorDirection::Right => ax > cx,
            };

            if matches_direction {
                let dist = (ax - cx).abs() + (ay - cy).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_pos = adj;
                }
            }
        }

        // If no adjacent in that direction, try non-adjacent positions
        if best_pos == current {
            for (pos, &(px, py)) in positions.iter().enumerate() {
                if pos == current {
                    continue;
                }
                let matches_direction = match direction {
                    CursorDirection::Up => py < cy,
                    CursorDirection::Down => py > cy,
                    CursorDirection::Left => px < cx,
                    CursorDirection::Right => px > cx,
                };

                if matches_direction {
                    let dist = (px - cx).abs() + (py - cy).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_pos = pos;
                    }
                }
            }
        }

        self.cursor = best_pos;
    }

    /// Clear piece selection
    pub fn clear_selection(&mut self) {
        self.selected_position = None;
    }
}

/// Direction for cursor movement
#[derive(Debug, Clone, Copy)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Get logical coordinates for each position (for cursor navigation)
/// Returns (x, y) where x is 0-6 and y is 0-6
fn get_position_coords() -> [(i32, i32); 24] {
    [
        (0, 0), (3, 0), (6, 0),  // 0, 1, 2
        (1, 1), (3, 1), (5, 1),  // 3, 4, 5
        (2, 2), (3, 2), (4, 2),  // 6, 7, 8
        (0, 3), (1, 3), (2, 3),  // 9, 10, 11
        (4, 3), (5, 3), (6, 3),  // 12, 13, 14
        (2, 4), (3, 4), (4, 4),  // 15, 16, 17
        (1, 5), (3, 5), (5, 5),  // 18, 19, 20
        (0, 6), (3, 6), (6, 6),  // 21, 22, 23
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(MorrisDifficulty::from_index(0), MorrisDifficulty::Novice);
        assert_eq!(MorrisDifficulty::from_index(1), MorrisDifficulty::Apprentice);
        assert_eq!(MorrisDifficulty::from_index(2), MorrisDifficulty::Journeyman);
        assert_eq!(MorrisDifficulty::from_index(3), MorrisDifficulty::Master);
        assert_eq!(MorrisDifficulty::from_index(99), MorrisDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_properties() {
        assert_eq!(MorrisDifficulty::Novice.random_move_chance(), 0.5);
        assert_eq!(MorrisDifficulty::Apprentice.random_move_chance(), 0.0);
        assert_eq!(MorrisDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(MorrisDifficulty::Master.reward_prestige(), 5);
    }

    #[test]
    fn test_new_game() {
        let game = MorrisGame::new(MorrisDifficulty::Journeyman);
        assert_eq!(game.difficulty, MorrisDifficulty::Journeyman);
        assert_eq!(game.pieces_to_place, (9, 9));
        assert_eq!(game.pieces_on_board, (0, 0));
        assert_eq!(game.phase, MorrisPhase::Placing);
        assert!(game.game_result.is_none());
        assert!(!game.ai_thinking);
    }

    #[test]
    fn test_mills_count() {
        assert_eq!(MILLS.len(), 16);
    }

    #[test]
    fn test_adjacencies() {
        // Position 4 (center of outer-middle connection) has 4 neighbors
        assert_eq!(ADJACENCIES[4].len(), 4);
        assert!(ADJACENCIES[4].contains(&1));
        assert!(ADJACENCIES[4].contains(&3));
        assert!(ADJACENCIES[4].contains(&5));
        assert!(ADJACENCIES[4].contains(&7));

        // Corner positions have 2 neighbors
        assert_eq!(ADJACENCIES[0].len(), 2);
        assert_eq!(ADJACENCIES[2].len(), 2);
        assert_eq!(ADJACENCIES[21].len(), 2);
        assert_eq!(ADJACENCIES[23].len(), 2);
    }

    #[test]
    fn test_is_in_mill() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        // Place a mill for human: positions 0, 1, 2
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.board[2] = Some(Player::Human);

        assert!(game.is_in_mill(0, Player::Human));
        assert!(game.is_in_mill(1, Player::Human));
        assert!(game.is_in_mill(2, Player::Human));
        assert!(!game.is_in_mill(3, Player::Human));
    }

    #[test]
    fn test_forms_mill() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        // Place two pieces of a mill
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);

        // Placing at 2 would form a mill
        assert!(game.forms_mill(2, Player::Human));
        // Placing at 3 would not form a mill
        assert!(!game.forms_mill(3, Player::Human));
    }

    #[test]
    fn test_can_fly() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_on_board = (3, 5);

        assert!(game.can_fly(Player::Human));
        assert!(!game.can_fly(Player::Ai));
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | head -20`
Expected: Errors about morris module not being declared in main.rs (that's OK for now)

**Step 3: Commit**

```bash
git add src/morris.rs
git commit -m "feat(morris): add data structures for Nine Men's Morris

- MorrisDifficulty with same tiers as chess
- MorrisGame with board state, phases, and cursor
- Board constants: MILLS (16 lines) and ADJACENCIES
- Mill detection and piece counting utilities"
```

---

## Task 2: Create Morris Game Logic

**Files:**
- Create: `src/morris_logic.rs`

**Step 1: Create morris_logic.rs with game logic and AI**

```rust
//! Nine Men's Morris game logic: legal moves, AI, and game processing.

use crate::challenge_menu::{ChallengeType, PendingChallenge};
use crate::game_state::GameState;
use crate::morris::{
    MorrisDifficulty, MorrisGame, MorrisMove, MorrisPhase, MorrisResult, Player, ADJACENCIES, MILLS,
};
use rand::Rng;

/// Chance per tick to discover a morris challenge (~2 hour average)
/// Same as chess: at 10 ticks/sec, 0.000014 chance/tick ≈ 2 hours average
pub const MORRIS_DISCOVERY_CHANCE: f64 = 0.000014;

/// Create a Morris challenge for the challenge menu
pub fn create_morris_challenge() -> PendingChallenge {
    PendingChallenge {
        challenge_type: ChallengeType::Morris,
        title: "Morris Challenge".to_string(),
        icon: "⚆",
        description: "A weathered board sits between you and a cloaked stranger. \
            \"Do you know the Miller's Game?\" they ask, gesturing at the carved lines. \
            Get three adjacent to capture pieces. Reduce your opponent to two to win."
            .to_string(),
    }
}

/// Check if morris discovery conditions are met and roll for discovery
pub fn try_discover_morris<R: Rng>(state: &mut GameState, rng: &mut R) -> bool {
    // Requirements: P1+, not in dungeon, not fishing, not in chess, not in morris, no pending morris
    if state.prestige_rank < 1
        || state.active_dungeon.is_some()
        || state.active_fishing.is_some()
        || state.active_chess.is_some()
        || state.active_morris.is_some()
        || state.challenge_menu.has_morris_challenge()
    {
        return false;
    }

    if rng.gen::<f64>() < MORRIS_DISCOVERY_CHANCE {
        state.challenge_menu.add_challenge(create_morris_challenge());
        true
    } else {
        false
    }
}

/// Start a Morris game with the selected difficulty
pub fn start_morris_game(state: &mut GameState, difficulty: MorrisDifficulty) {
    state.active_morris = Some(MorrisGame::new(difficulty));
    state.challenge_menu.close();
}

/// Get all legal moves for the current player
pub fn get_legal_moves(game: &MorrisGame) -> Vec<MorrisMove> {
    let player = game.current_player;

    if game.must_capture {
        return get_capture_moves(game, player);
    }

    match game.phase {
        MorrisPhase::Placing => get_placing_moves(game),
        MorrisPhase::Moving | MorrisPhase::Flying => get_movement_moves(game, player),
    }
}

/// Get legal placement moves (any empty position)
fn get_placing_moves(game: &MorrisGame) -> Vec<MorrisMove> {
    (0..24)
        .filter(|&pos| game.board[pos].is_none())
        .map(MorrisMove::Place)
        .collect()
}

/// Get legal movement moves for a player
fn get_movement_moves(game: &MorrisGame, player: Player) -> Vec<MorrisMove> {
    let can_fly = game.can_fly(player);
    let mut moves = Vec::new();

    for from in 0..24 {
        if game.board[from] != Some(player) {
            continue;
        }

        if can_fly {
            // Can move to any empty position
            for to in 0..24 {
                if game.board[to].is_none() {
                    moves.push(MorrisMove::Move { from, to });
                }
            }
        } else {
            // Can only move to adjacent empty positions
            for &to in ADJACENCIES[from] {
                if game.board[to].is_none() {
                    moves.push(MorrisMove::Move { from, to });
                }
            }
        }
    }

    moves
}

/// Get legal capture moves (opponent pieces not in mills, unless all are)
fn get_capture_moves(game: &MorrisGame, player: Player) -> Vec<MorrisMove> {
    let opponent = match player {
        Player::Human => Player::Ai,
        Player::Ai => Player::Human,
    };

    let opponent_positions: Vec<usize> = (0..24)
        .filter(|&pos| game.board[pos] == Some(opponent))
        .collect();

    // First, try pieces not in mills
    let not_in_mill: Vec<MorrisMove> = opponent_positions
        .iter()
        .filter(|&&pos| !game.is_in_mill(pos, opponent))
        .map(|&pos| MorrisMove::Capture(pos))
        .collect();

    if not_in_mill.is_empty() {
        // All opponent pieces are in mills - can capture any
        opponent_positions
            .into_iter()
            .map(MorrisMove::Capture)
            .collect()
    } else {
        not_in_mill
    }
}

/// Apply a move to the game state
pub fn apply_move(game: &mut MorrisGame, mv: MorrisMove) {
    let player = game.current_player;

    match mv {
        MorrisMove::Place(pos) => {
            game.board[pos] = Some(player);
            match player {
                Player::Human => {
                    game.pieces_to_place.0 -= 1;
                    game.pieces_on_board.0 += 1;
                }
                Player::Ai => {
                    game.pieces_to_place.1 -= 1;
                    game.pieces_on_board.1 += 1;
                }
            }

            // Check if mill was formed
            if game.forms_mill(pos, player) {
                game.must_capture = true;
            } else {
                end_turn(game);
            }
        }
        MorrisMove::Move { from, to } => {
            game.board[from] = None;
            game.board[to] = Some(player);

            // Check if mill was formed
            if game.forms_mill(to, player) {
                game.must_capture = true;
            } else {
                end_turn(game);
            }
        }
        MorrisMove::Capture(pos) => {
            game.board[pos] = None;
            match game.current_player {
                Player::Human => game.pieces_on_board.1 -= 1,
                Player::Ai => game.pieces_on_board.0 -= 1,
            }
            game.must_capture = false;
            end_turn(game);
        }
    }

    game.selected_position = None;
}

/// End the current turn and switch players
fn end_turn(game: &mut MorrisGame) {
    // Check for phase transition
    if game.phase == MorrisPhase::Placing
        && game.pieces_to_place.0 == 0
        && game.pieces_to_place.1 == 0
    {
        game.phase = MorrisPhase::Moving;
    }

    // Update flying status
    if game.phase != MorrisPhase::Placing {
        if game.can_fly(Player::Human) || game.can_fly(Player::Ai) {
            game.phase = MorrisPhase::Flying;
        }
    }

    // Check win conditions before switching
    check_win_condition(game);

    if game.game_result.is_none() {
        // Switch player
        game.current_player = match game.current_player {
            Player::Human => Player::Ai,
            Player::Ai => Player::Human,
        };

        // Start AI thinking if it's AI's turn
        if game.current_player == Player::Ai {
            game.ai_thinking = true;
            game.ai_think_ticks = 0;
            game.ai_pending_move = None;
        }
    }
}

/// Check if the game is over
fn check_win_condition(game: &mut MorrisGame) {
    if game.game_result.is_some() {
        return;
    }

    // Only check after placing phase
    if game.phase == MorrisPhase::Placing {
        return;
    }

    // Win condition 1: Opponent has fewer than 3 pieces
    if game.pieces_on_board.1 < 3 {
        game.game_result = Some(MorrisResult::Win);
        return;
    }
    if game.pieces_on_board.0 < 3 {
        game.game_result = Some(MorrisResult::Loss);
        return;
    }

    // Win condition 2: Opponent cannot move
    let next_player = match game.current_player {
        Player::Human => Player::Ai,
        Player::Ai => Player::Human,
    };

    // Temporarily switch to check opponent's moves
    let mut test_game = game.clone();
    test_game.current_player = next_player;
    test_game.must_capture = false;

    if get_legal_moves(&test_game).is_empty() {
        // Current player wins because opponent can't move
        game.game_result = Some(match game.current_player {
            Player::Human => MorrisResult::Win,
            Player::Ai => MorrisResult::Loss,
        });
    }
}

/// Calculate AI thinking time in ticks
pub fn calculate_think_ticks<R: Rng>(rng: &mut R) -> u32 {
    rng.gen_range(10..30) // 1-3 seconds
}

/// Evaluate board position for AI (positive = good for AI)
fn evaluate_board(game: &MorrisGame) -> i32 {
    let ai_pieces = game.pieces_on_board.1 as i32 + game.pieces_to_place.1 as i32;
    let human_pieces = game.pieces_on_board.0 as i32 + game.pieces_to_place.0 as i32;

    // Piece difference (heavily weighted)
    let piece_score = (ai_pieces - human_pieces) * 100;

    // Mill count
    let ai_mills = count_mills(game, Player::Ai);
    let human_mills = count_mills(game, Player::Human);
    let mill_score = (ai_mills - human_mills) * 50;

    // Potential mills (2 pieces with empty third)
    let ai_potential = count_potential_mills(game, Player::Ai);
    let human_potential = count_potential_mills(game, Player::Human);
    let potential_score = (ai_potential - human_potential) * 20;

    // Mobility (number of legal moves)
    let ai_mobility = count_mobility(game, Player::Ai);
    let human_mobility = count_mobility(game, Player::Human);
    let mobility_score = (ai_mobility - human_mobility) * 10;

    piece_score + mill_score + potential_score + mobility_score
}

/// Count completed mills for a player
fn count_mills(game: &MorrisGame, player: Player) -> i32 {
    MILLS
        .iter()
        .filter(|mill| mill.iter().all(|&pos| game.board[pos] == Some(player)))
        .count() as i32
}

/// Count potential mills (2 pieces + empty) for a player
fn count_potential_mills(game: &MorrisGame, player: Player) -> i32 {
    MILLS
        .iter()
        .filter(|mill| {
            let player_count = mill.iter().filter(|&&pos| game.board[pos] == Some(player)).count();
            let empty_count = mill.iter().filter(|&&pos| game.board[pos].is_none()).count();
            player_count == 2 && empty_count == 1
        })
        .count() as i32
}

/// Count legal moves for a player
fn count_mobility(game: &MorrisGame, player: Player) -> i32 {
    let mut test_game = game.clone();
    test_game.current_player = player;
    test_game.must_capture = false;
    get_legal_moves(&test_game).len() as i32
}

/// Minimax with alpha-beta pruning
fn minimax(
    game: &MorrisGame,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
) -> i32 {
    if depth == 0 || game.game_result.is_some() {
        return evaluate_board(game);
    }

    let moves = get_legal_moves(game);
    if moves.is_empty() {
        return evaluate_board(game);
    }

    if maximizing {
        let mut max_eval = i32::MIN;
        for mv in moves {
            let mut new_game = game.clone();
            apply_move(&mut new_game, mv);
            let eval = minimax(&new_game, depth - 1, alpha, beta, !maximizing);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for mv in moves {
            let mut new_game = game.clone();
            apply_move(&mut new_game, mv);
            let eval = minimax(&new_game, depth - 1, alpha, beta, !maximizing);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

/// Get the best AI move
pub fn get_ai_move<R: Rng>(game: &MorrisGame, rng: &mut R) -> Option<MorrisMove> {
    let moves = get_legal_moves(game);
    if moves.is_empty() {
        return None;
    }

    // Random move for Novice difficulty
    if rng.gen::<f64>() < game.difficulty.random_move_chance() {
        let idx = rng.gen_range(0..moves.len());
        return Some(moves[idx]);
    }

    let depth = game.difficulty.search_depth();
    let mut best_move = moves[0];
    let mut best_score = i32::MIN;

    for mv in &moves {
        let mut new_game = game.clone();
        apply_move(&mut new_game, *mv);
        let score = minimax(&new_game, depth - 1, i32::MIN, i32::MAX, false);
        if score > best_score {
            best_score = score;
            best_move = *mv;
        }
    }

    Some(best_move)
}

/// Process AI thinking tick
pub fn process_ai_thinking<R: Rng>(game: &mut MorrisGame, rng: &mut R) -> bool {
    if !game.ai_thinking {
        return false;
    }

    game.ai_think_ticks += 1;

    // Compute AI move on first tick
    if game.ai_pending_move.is_none() {
        game.ai_pending_move = get_ai_move(game, rng);
        game.ai_think_target = calculate_think_ticks(rng);
    }

    // Apply move after delay
    if game.ai_think_ticks >= game.ai_think_target {
        if let Some(mv) = game.ai_pending_move.take() {
            apply_move(game, mv);
        }
        game.ai_thinking = false;
        game.ai_think_ticks = 0;
        return true;
    }

    false
}

/// Apply game result: grant prestige on win
pub fn apply_game_result(state: &mut GameState) -> Option<(MorrisResult, u32)> {
    let game = state.active_morris.as_ref()?;
    let result = game.game_result?;
    let difficulty = game.difficulty;

    let prestige_gained = match result {
        MorrisResult::Win => {
            let reward = difficulty.reward_prestige();
            state.prestige_rank += reward;
            reward
        }
        MorrisResult::Loss | MorrisResult::Forfeit => 0,
    };

    state.active_morris = None;
    Some((result, prestige_gained))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_challenge() {
        let challenge = create_morris_challenge();
        assert_eq!(challenge.title, "Morris Challenge");
        assert_eq!(challenge.icon, "⚆");
        assert!(matches!(challenge.challenge_type, ChallengeType::Morris));
    }

    #[test]
    fn test_placing_moves() {
        let game = MorrisGame::new(MorrisDifficulty::Novice);
        let moves = get_legal_moves(&game);
        // All 24 positions should be available
        assert_eq!(moves.len(), 24);
        assert!(moves.iter().all(|m| matches!(m, MorrisMove::Place(_))));
    }

    #[test]
    fn test_apply_place_move() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        apply_move(&mut game, MorrisMove::Place(0));

        assert_eq!(game.board[0], Some(Player::Human));
        assert_eq!(game.pieces_to_place.0, 8);
        assert_eq!(game.pieces_on_board.0, 1);
        // Turn should switch to AI (starts thinking)
        assert_eq!(game.current_player, Player::Ai);
        assert!(game.ai_thinking);
    }

    #[test]
    fn test_mill_triggers_capture() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        // Place two pieces for human
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.pieces_on_board.0 = 2;
        game.pieces_to_place.0 = 7;

        // Place one piece for AI so there's something to capture
        game.board[3] = Some(Player::Ai);
        game.pieces_on_board.1 = 1;
        game.pieces_to_place.1 = 8;

        // Placing at 2 should form a mill and require capture
        apply_move(&mut game, MorrisMove::Place(2));

        assert!(game.must_capture);
        assert_eq!(game.current_player, Player::Human); // Turn doesn't switch until capture
    }

    #[test]
    fn test_capture_move() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.board[3] = Some(Player::Ai);
        game.pieces_on_board.1 = 1;
        game.must_capture = true;

        apply_move(&mut game, MorrisMove::Capture(3));

        assert!(game.board[3].is_none());
        assert_eq!(game.pieces_on_board.1, 0);
        assert!(!game.must_capture);
    }

    #[test]
    fn test_movement_moves() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.board[0] = Some(Player::Human);
        game.pieces_on_board.0 = 4; // Not flying

        let moves = get_legal_moves(&game);

        // Position 0 is adjacent to 1 and 9
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&MorrisMove::Move { from: 0, to: 1 }));
        assert!(moves.contains(&MorrisMove::Move { from: 0, to: 9 }));
    }

    #[test]
    fn test_flying_moves() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.board[0] = Some(Player::Human);
        game.pieces_on_board.0 = 3; // Flying!

        let moves = get_legal_moves(&game);

        // Can move to any of the 23 empty positions
        assert_eq!(moves.len(), 23);
    }

    #[test]
    fn test_ai_returns_legal_move() {
        let game = MorrisGame::new(MorrisDifficulty::Novice);
        let mut rng = rand::thread_rng();

        let ai_move = get_ai_move(&game, &mut rng);
        assert!(ai_move.is_some());

        let legal_moves = get_legal_moves(&game);
        assert!(legal_moves.contains(&ai_move.unwrap()));
    }

    #[test]
    fn test_win_by_piece_count() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_on_board = (5, 2); // AI has only 2 pieces

        check_win_condition(&mut game);

        assert_eq!(game.game_result, Some(MorrisResult::Win));
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | head -30`
Expected: Errors about module not declared and missing ChallengeType::Morris (expected)

**Step 3: Commit**

```bash
git add src/morris_logic.rs
git commit -m "feat(morris): add game logic and AI

- Legal move generation for all phases
- Mill detection and capture logic
- Minimax AI with alpha-beta pruning
- Discovery system matching chess (~2hr average)
- Win condition checking"
```

---

## Task 3: Add Morris to Challenge Menu

**Files:**
- Modify: `src/challenge_menu.rs`

**Step 1: Add Morris variant to ChallengeType**

In `src/challenge_menu.rs`, update the `ChallengeType` enum:

```rust
/// Extensible enum for different minigame challenges
#[derive(Debug, Clone)]
pub enum ChallengeType {
    Chess,
    Morris,
}
```

**Step 2: Add has_morris_challenge method**

Add this method to `ChallengeMenu` impl:

```rust
    pub fn has_morris_challenge(&self) -> bool {
        self.challenges
            .iter()
            .any(|c| matches!(c.challenge_type, ChallengeType::Morris))
    }
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -20`

**Step 4: Commit**

```bash
git add src/challenge_menu.rs
git commit -m "feat(morris): add Morris variant to challenge menu

- Add ChallengeType::Morris enum variant
- Add has_morris_challenge() check method"
```

---

## Task 4: Add Morris to Game State

**Files:**
- Modify: `src/game_state.rs`

**Step 1: Add import and field**

Add to imports at top:
```rust
use crate::morris::MorrisGame;
```

Add field to `GameState` struct after `active_chess`:
```rust
    /// Active morris game (transient, not saved)
    #[serde(skip)]
    pub active_morris: Option<MorrisGame>,
```

**Step 2: Initialize in GameState::new()**

Add to the `Self { ... }` block:
```rust
            active_morris: None,
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -20`

**Step 4: Commit**

```bash
git add src/game_state.rs
git commit -m "feat(morris): add active_morris field to GameState"
```

---

## Task 5: Create Morris UI Scene

**Files:**
- Create: `src/ui/morris_scene.rs`

**Step 1: Create the UI rendering file**

```rust
//! Nine Men's Morris board UI rendering.

use crate::morris::{MorrisDifficulty, MorrisGame, MorrisPhase, MorrisResult, Player, ADJACENCIES};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the Morris game scene
pub fn render_morris_scene(frame: &mut Frame, area: Rect, game: &MorrisGame) {
    frame.render_widget(Clear, area);

    // Check for game over overlay
    if let Some(result) = game.game_result {
        render_game_over_overlay(frame, area, result, game.difficulty.reward_prestige());
        return;
    }

    let block = Block::default()
        .title(" Nine Men's Morris ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Horizontal layout: board on left, help panel on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),    // Board area
            Constraint::Length(24), // Help panel
        ])
        .split(inner);

    // Vertical layout for board area: board + status
    let board_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(15),   // Board
            Constraint::Length(2), // Status
        ])
        .split(chunks[0]);

    render_board(frame, board_chunks[0], game);
    render_status(frame, board_chunks[1], game);
    render_help_panel(frame, chunks[1], game);
}

/// Render the game board
fn render_board(frame: &mut Frame, area: Rect, game: &MorrisGame) {
    // Board dimensions
    let board_height = 13;
    let board_width = 25;

    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    // Position coordinates on the visual board (x, y relative to board top-left)
    let positions: [(u16, u16); 24] = [
        (0, 0),   (12, 0),  (24, 0),   // 0, 1, 2
        (4, 2),   (12, 2),  (20, 2),   // 3, 4, 5
        (8, 4),   (12, 4),  (16, 4),   // 6, 7, 8
        (0, 6),   (4, 6),   (8, 6),    // 9, 10, 11
        (16, 6),  (20, 6),  (24, 6),   // 12, 13, 14
        (8, 8),   (12, 8),  (16, 8),   // 15, 16, 17
        (4, 10),  (12, 10), (20, 10),  // 18, 19, 20
        (0, 12),  (12, 12), (24, 12),  // 21, 22, 23
    ];

    let line_color = Color::Rgb(60, 60, 60);

    // Draw the board lines
    let board_lines = [
        // Outer square
        "●───────────●───────────●",
        "│           │           │",
        "│   ●───────●───────●   │",
        "│   │       │       │   │",
        "│   │   ●───●───●   │   │",
        "│   │   │       │   │   │",
        "●───●───●       ●───●───●",
        "│   │   │       │   │   │",
        "│   │   ●───●───●   │   │",
        "│   │       │       │   │",
        "│   ●───────●───────●   │",
        "│           │           │",
        "●───────────●───────────●",
    ];

    for (i, line) in board_lines.iter().enumerate() {
        // Replace ● with · for empty intersection display
        let display_line = line.replace('●', "·");
        let para = Paragraph::new(display_line).style(Style::default().fg(line_color));
        frame.render_widget(
            para,
            Rect::new(x_offset, y_offset + i as u16, board_width, 1),
        );
    }

    // Get legal move destinations for highlighting
    let legal_destinations: Vec<usize> = if game.selected_position.is_some() && !game.must_capture {
        crate::morris_logic::get_legal_moves(game)
            .iter()
            .filter_map(|m| match m {
                crate::morris::MorrisMove::Move { to, .. } => Some(*to),
                _ => None,
            })
            .collect()
    } else if game.must_capture {
        crate::morris_logic::get_legal_moves(game)
            .iter()
            .filter_map(|m| match m {
                crate::morris::MorrisMove::Capture(pos) => Some(*pos),
                _ => None,
            })
            .collect()
    } else {
        vec![]
    };

    // Draw pieces and cursor
    for (pos, &(px, py)) in positions.iter().enumerate() {
        let screen_x = x_offset + px;
        let screen_y = y_offset + py;

        let is_cursor = game.cursor == pos;
        let is_selected = game.selected_position == Some(pos);
        let is_legal_dest = legal_destinations.contains(&pos);
        let piece = game.board[pos];

        let (char_str, style) = if is_cursor {
            match piece {
                Some(Player::Human) => ("[●]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Some(Player::Ai) => ("[○]", Style::default().fg(Color::Rgb(140, 140, 140))),
                None if is_legal_dest => ("[◆]", Style::default().fg(Color::Rgb(200, 100, 200))),
                None => ("[·]", Style::default().fg(Color::Yellow)),
            }
        } else if is_selected {
            match piece {
                Some(Player::Human) => ("<●>", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                _ => (" · ", Style::default().fg(Color::DarkGray)),
            }
        } else if is_legal_dest {
            match piece {
                Some(Player::Ai) => (" ○ ", Style::default().fg(Color::Red)), // Capturable
                None => (" ◆ ", Style::default().fg(Color::Rgb(200, 100, 200))),
                _ => (" · ", Style::default().fg(Color::DarkGray)),
            }
        } else {
            match piece {
                Some(Player::Human) => (" ● ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Some(Player::Ai) => (" ○ ", Style::default().fg(Color::Rgb(140, 140, 140))),
                None => (" · ", Style::default().fg(line_color)),
            }
        };

        // Render the piece/position (3 chars wide, centered on the position)
        let para = Paragraph::new(char_str).style(style);
        frame.render_widget(
            para,
            Rect::new(screen_x.saturating_sub(1), screen_y, 3, 1),
        );
    }
}

/// Render the status line
fn render_status(frame: &mut Frame, area: Rect, game: &MorrisGame) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (status_text, status_style) = if game.ai_thinking {
        const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let frame_idx = ((millis / 100) % 10) as usize;
        let spinner = SPINNER[frame_idx];
        (
            format!("{} Opponent is thinking...", spinner),
            Style::default().fg(Color::Yellow),
        )
    } else if game.forfeit_pending {
        ("Forfeit game?".to_string(), Style::default().fg(Color::Red))
    } else if game.must_capture {
        (
            "MILL! Select a piece to capture".to_string(),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )
    } else if game.selected_position.is_some() {
        (
            "Select destination".to_string(),
            Style::default().fg(Color::Cyan),
        )
    } else {
        let phase_text = match game.phase {
            MorrisPhase::Placing => "Place a piece",
            MorrisPhase::Moving => "Select piece to move",
            MorrisPhase::Flying => "Select piece to move (flying!)",
        };
        (phase_text.to_string(), Style::default().fg(Color::White))
    };

    let controls_text = if game.ai_thinking {
        ""
    } else if game.forfeit_pending {
        "[Esc] Confirm forfeit  [Any] Cancel"
    } else if game.selected_position.is_some() || game.must_capture {
        "[Arrows] Move  [Enter] Confirm  [Esc] Cancel"
    } else {
        "[Arrows] Move  [Enter] Select  [Esc] Forfeit"
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Center);
    frame.render_widget(status, Rect { height: 1, ..area });

    if !controls_text.is_empty() {
        let controls = Paragraph::new(controls_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(
            controls,
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );
    }
}

/// Render the help panel
fn render_help_panel(frame: &mut Frame, area: Rect, game: &MorrisGame) {
    let block = Block::default()
        .title(" How to Play ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_lines = vec![
        Line::from(Span::styled("1. PLACE: Put 9", Style::default().fg(Color::White))),
        Line::from(Span::styled("   pieces", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("2. MOVE: Slide", Style::default().fg(Color::White))),
        Line::from(Span::styled("   along lines", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("3. Three adjacent", Style::default().fg(Color::White))),
        Line::from(Span::styled("   → capture one", Style::default().fg(Color::Green))),
        Line::from(Span::styled("   of theirs", Style::default().fg(Color::Green))),
        Line::from(""),
        Line::from(Span::styled("4. Win: 2 pieces", Style::default().fg(Color::White))),
        Line::from(Span::styled("   or blocked", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("(3 left = fly)", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            format!("You: ● × {}", game.pieces_on_board.0),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("Foe: ○ × {}", game.pieces_on_board.1),
            Style::default().fg(Color::Rgb(140, 140, 140)),
        )),
    ];

    // Add pieces to place if in placing phase
    let mut lines = help_lines;
    if game.phase == MorrisPhase::Placing {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("To place: {}", game.pieces_to_place.0),
            Style::default().fg(Color::Yellow),
        )));
    }

    let help = Paragraph::new(lines);
    frame.render_widget(help, inner);
}

/// Render the game over overlay
fn render_game_over_overlay(frame: &mut Frame, area: Rect, result: MorrisResult, prestige: u32) {
    frame.render_widget(Clear, area);

    let (title, message, reward) = match result {
        MorrisResult::Win => (
            ":: VICTORY! ::",
            "The stranger nods with respect.\n\"Well played. Until we meet again.\"",
            format!("+{} Prestige Ranks", prestige),
        ),
        MorrisResult::Loss => (
            "DEFEAT",
            "The stranger collects their pieces.\n\"Perhaps another time,\" they say.",
            "No penalty incurred.".to_string(),
        ),
        MorrisResult::Forfeit => (
            "FORFEIT",
            "You concede the game.\nThe stranger silently fades away.",
            "No penalty incurred.".to_string(),
        ),
    };

    let title_color = match result {
        MorrisResult::Win => Color::Green,
        MorrisResult::Loss => Color::Red,
        MorrisResult::Forfeit => Color::Gray,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content_height: u16 = 9;
    let y_offset = inner.y + (inner.height.saturating_sub(content_height)) / 2;

    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(reward, Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from(Span::styled(
            "[Press any key]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(
        text,
        Rect::new(inner.x, y_offset, inner.width, content_height),
    );
}
```

**Step 2: Export in ui/mod.rs**

Add to `src/ui/mod.rs`:
```rust
pub mod morris_scene;
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -30`

**Step 4: Commit**

```bash
git add src/ui/morris_scene.rs src/ui/mod.rs
git commit -m "feat(morris): add UI rendering for Morris game

- Board rendering with Unicode lines and piece symbols
- Cursor navigation and selection highlighting
- Legal move destination markers
- Help panel with rules
- Game over overlay with victory/defeat messages
- AI thinking spinner animation"
```

---

## Task 6: Wire Up Main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Add module declarations**

Add near the top with other mod declarations:
```rust
mod morris;
mod morris_logic;
```

**Step 2: Add imports**

Add to the imports section:
```rust
use morris::MorrisDifficulty;
use morris_logic::{
    apply_game_result as apply_morris_result, process_ai_thinking as process_morris_ai,
    start_morris_game, try_discover_morris,
};
```

**Step 3: Add Morris rendering in draw_ui_with_update**

In `src/ui/mod.rs`, update the rendering priority in `draw_ui_with_update`:

```rust
    // Draw right panel based on current activity
    // Priority: morris > chess > challenge menu > fishing > dungeon > combat
    if let Some(ref game) = game_state.active_morris {
        morris_scene::render_morris_scene(frame, chunks[1], game);
    } else if let Some(ref game) = game_state.active_chess {
```

Also update the challenge banner check:
```rust
    let show_challenge_banner = !game_state.challenge_menu.challenges.is_empty()
        && !game_state.challenge_menu.is_open
        && game_state.active_chess.is_none()
        && game_state.active_morris.is_none();
```

**Step 4: Add Morris input handling in main.rs**

After the chess input handling block (around line 553), add Morris input handling:

```rust
                            // Handle active Morris game input
                            if let Some(ref mut morris_game) = state.active_morris {
                                if morris_game.game_result.is_some() {
                                    // Any key dismisses result
                                    let old_prestige = state.prestige_rank;
                                    if let Some((result, prestige_gained)) =
                                        apply_morris_result(&mut state)
                                    {
                                        use morris::MorrisResult;
                                        match result {
                                            MorrisResult::Win => {
                                                let new_prestige = old_prestige + prestige_gained;
                                                state.combat_state.add_log_entry(
                                                    "⚆ You won the Miller's Game!".to_string(),
                                                    false,
                                                    true,
                                                );
                                                state.combat_state.add_log_entry(
                                                    format!(
                                                        "⚆ +{} Prestige Ranks (P{} → P{})",
                                                        prestige_gained, old_prestige, new_prestige
                                                    ),
                                                    false,
                                                    true,
                                                );
                                            }
                                            MorrisResult::Loss => {
                                                state.combat_state.add_log_entry(
                                                    "⚆ The stranger collects their pieces and leaves."
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                            MorrisResult::Forfeit => {
                                                state.combat_state.add_log_entry(
                                                    "⚆ You concede the game.".to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                        }
                                    }
                                    continue;
                                }
                                if !morris_game.ai_thinking {
                                    use morris::CursorDirection;
                                    match key_event.code {
                                        KeyCode::Up => morris_game.move_cursor(CursorDirection::Up),
                                        KeyCode::Down => morris_game.move_cursor(CursorDirection::Down),
                                        KeyCode::Left => morris_game.move_cursor(CursorDirection::Left),
                                        KeyCode::Right => morris_game.move_cursor(CursorDirection::Right),
                                        KeyCode::Enter => {
                                            handle_morris_enter(&mut state);
                                        }
                                        KeyCode::Esc => {
                                            if morris_game.forfeit_pending {
                                                morris_game.game_result =
                                                    Some(morris::MorrisResult::Forfeit);
                                            } else if morris_game.selected_position.is_some() {
                                                morris_game.clear_selection();
                                                morris_game.forfeit_pending = false;
                                            } else {
                                                morris_game.forfeit_pending = true;
                                            }
                                        }
                                        _ => {
                                            morris_game.forfeit_pending = false;
                                        }
                                    }
                                }
                                continue;
                            }
```

**Step 5: Add Morris enter handler function**

Add this function before the main function:

```rust
/// Handle Enter key press in Morris game
fn handle_morris_enter(state: &mut GameState) {
    let Some(ref mut game) = state.active_morris else {
        return;
    };

    use morris::{MorrisMove, MorrisPhase, Player};
    use morris_logic::{apply_move, get_legal_moves};

    let legal_moves = get_legal_moves(game);
    let cursor = game.cursor;

    if game.must_capture {
        // Must capture an opponent piece
        if let Some(mv) = legal_moves
            .iter()
            .find(|m| matches!(m, MorrisMove::Capture(pos) if *pos == cursor))
        {
            apply_move(game, *mv);
        }
    } else if game.phase == MorrisPhase::Placing {
        // Placing phase: place at cursor if legal
        if let Some(mv) = legal_moves
            .iter()
            .find(|m| matches!(m, MorrisMove::Place(pos) if *pos == cursor))
        {
            apply_move(game, *mv);
        }
    } else {
        // Moving phase
        if let Some(selected) = game.selected_position {
            // Try to move to cursor
            if let Some(mv) = legal_moves
                .iter()
                .find(|m| matches!(m, MorrisMove::Move { from, to } if *from == selected && *to == cursor))
            {
                apply_move(game, *mv);
            } else if game.board[cursor] == Some(Player::Human) {
                // Select different piece
                game.selected_position = Some(cursor);
            } else {
                game.clear_selection();
            }
        } else {
            // Select piece at cursor
            if game.board[cursor] == Some(Player::Human) {
                game.selected_position = Some(cursor);
            }
        }
    }

    game.forfeit_pending = false;
}
```

**Step 6: Add Morris tick processing**

In the `game_tick` function, add Morris processing after chess:

```rust
    // Process Morris AI thinking
    if let Some(ref mut morris_game) = game_state.active_morris {
        let mut rng = rand::thread_rng();
        process_morris_ai(morris_game, &mut rng);
    }

    // Try Morris discovery during normal combat
    if game_state.active_morris.is_none()
        && game_state.active_chess.is_none()
        && game_state.active_dungeon.is_none()
        && game_state.active_fishing.is_none()
    {
        let mut rng = rand::thread_rng();
        if try_discover_morris(game_state, &mut rng) {
            game_state.combat_state.add_log_entry(
                "⚆ A cloaked stranger approaches with a weathered board...".to_string(),
                false,
                true,
            );
            game_state.combat_state.add_log_entry(
                "⚆ Press [Tab] to view pending challenges".to_string(),
                false,
                true,
            );
        }
    }
```

**Step 7: Add Morris challenge acceptance in challenge menu**

In the challenge menu Enter handling, add Morris case:

```rust
                                    ChallengeType::Morris => {
                                        let difficulty =
                                            MorrisDifficulty::from_index(state.challenge_menu.selected_difficulty);
                                        start_morris_game(&mut state, difficulty);
                                    }
```

**Step 8: Verify it compiles and runs**

Run: `cargo build && cargo test`

**Step 9: Commit**

```bash
git add src/main.rs src/ui/mod.rs
git commit -m "feat(morris): wire up Morris game in main loop

- Add module declarations and imports
- Add Morris input handling (cursor, selection, forfeit)
- Add Morris tick processing and AI
- Add Morris discovery during idle gameplay
- Add Morris challenge acceptance from menu
- Update rendering priority"
```

---

## Task 7: Final Testing and Polish

**Step 1: Run full test suite**

```bash
cargo test
```

**Step 2: Run clippy**

```bash
cargo clippy --all-targets -- -D warnings
```

**Step 3: Fix any issues**

Address any clippy warnings or test failures.

**Step 4: Manual testing**

Run the game and test:
1. Wait for Morris challenge to appear (or temporarily boost discovery rate)
2. Accept challenge at each difficulty
3. Test placing phase
4. Test moving phase
5. Test capture mechanics
6. Test forfeit
7. Test win/loss conditions

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat(morris): complete Nine Men's Morris implementation

Nine Men's Morris is now playable as a challenge type:
- Same discovery rate as chess (~2 hour average)
- Same difficulty tiers with prestige rewards
- Full game logic with placing, moving, flying phases
- Mill detection and capture mechanics
- Minimax AI with alpha-beta pruning
- Unicode board rendering with help panel
- Win by reducing opponent to 2 pieces or blocking moves"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Data structures | `src/morris.rs` |
| 2 | Game logic & AI | `src/morris_logic.rs` |
| 3 | Challenge menu integration | `src/challenge_menu.rs` |
| 4 | Game state integration | `src/game_state.rs` |
| 5 | UI rendering | `src/ui/morris_scene.rs`, `src/ui/mod.rs` |
| 6 | Main loop wiring | `src/main.rs`, `src/ui/mod.rs` |
| 7 | Testing & polish | All files |
