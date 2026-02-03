# Chess Minigame Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a chess minigame where players can earn prestige ranks by defeating AI opponents of varying difficulty.

**Architecture:** Generic challenge menu system that chess plugs into, with chess-engine crate for move generation and AI. Follows existing patterns: types in dedicated module, logic in separate module, UI scene in src/ui/.

**Tech Stack:** Rust, chess-engine crate, Ratatui for terminal UI

**Design Doc:** `docs/plans/2026-02-03-chess-minigame-design.md`

---

## Task 1: Add chess-engine dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add the chess-engine crate**

Add to `Cargo.toml` dependencies section:

```toml
chess-engine = "0.4"
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds with chess-engine downloaded

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add chess-engine dependency"
```

---

## Task 2: Create challenge menu types

**Files:**
- Create: `src/challenge_menu.rs`
- Modify: `src/main.rs` (add mod declaration)

**Step 1: Write the test for ChallengeMenu basics**

Add to `src/challenge_menu.rs`:

```rust
//! Generic challenge menu system for player-controlled minigames.
//!
//! The challenge menu holds pending challenges that players can accept or decline.
//! Chess is the first producer, but future minigames can add their own challenge types.

use serde::{Deserialize, Serialize};

/// A single pending challenge in the menu
#[derive(Debug, Clone)]
pub struct PendingChallenge {
    pub challenge_type: ChallengeType,
    pub title: String,
    pub icon: &'static str,
    pub description: String,
}

/// Extensible enum for different minigame challenges
#[derive(Debug, Clone)]
pub enum ChallengeType {
    Chess,
}

/// Menu state for navigation
#[derive(Debug, Clone, Default)]
pub struct ChallengeMenu {
    pub challenges: Vec<PendingChallenge>,
    pub is_open: bool,
    pub selected_index: usize,
    pub viewing_detail: bool,
    pub selected_difficulty: usize,
}

impl ChallengeMenu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a challenge of the given type already exists
    pub fn has_chess_challenge(&self) -> bool {
        self.challenges
            .iter()
            .any(|c| matches!(c.challenge_type, ChallengeType::Chess))
    }

    /// Add a new challenge to the menu
    pub fn add_challenge(&mut self, challenge: PendingChallenge) {
        self.challenges.push(challenge);
    }

    /// Remove and return the currently selected challenge
    pub fn take_selected(&mut self) -> Option<PendingChallenge> {
        if self.challenges.is_empty() {
            return None;
        }
        let challenge = self.challenges.remove(self.selected_index);
        self.selected_index = self.selected_index.min(self.challenges.len().saturating_sub(1));
        Some(challenge)
    }

    /// Navigate up in the list or difficulty selector
    pub fn navigate_up(&mut self) {
        if self.viewing_detail {
            if self.selected_difficulty > 0 {
                self.selected_difficulty -= 1;
            }
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Navigate down in the list or difficulty selector
    pub fn navigate_down(&mut self, max_difficulties: usize) {
        if self.viewing_detail {
            if self.selected_difficulty + 1 < max_difficulties {
                self.selected_difficulty += 1;
            }
        } else if self.selected_index + 1 < self.challenges.len() {
            self.selected_index += 1;
        }
    }

    /// Open detail view for selected challenge
    pub fn open_detail(&mut self) {
        if !self.challenges.is_empty() {
            self.viewing_detail = true;
            self.selected_difficulty = 0;
        }
    }

    /// Close detail view, return to list
    pub fn close_detail(&mut self) {
        self.viewing_detail = false;
        self.selected_difficulty = 0;
    }

    /// Open the menu
    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_index = 0;
        self.viewing_detail = false;
        self.selected_difficulty = 0;
    }

    /// Close the menu
    pub fn close(&mut self) {
        self.is_open = false;
        self.viewing_detail = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chess_challenge() -> PendingChallenge {
        PendingChallenge {
            challenge_type: ChallengeType::Chess,
            title: "Chess Challenge".to_string(),
            icon: "♟",
            description: "A mysterious figure challenges you to chess.".to_string(),
        }
    }

    #[test]
    fn test_new_menu_is_empty() {
        let menu = ChallengeMenu::new();
        assert!(menu.challenges.is_empty());
        assert!(!menu.is_open);
        assert!(!menu.viewing_detail);
    }

    #[test]
    fn test_add_and_check_challenge() {
        let mut menu = ChallengeMenu::new();
        assert!(!menu.has_chess_challenge());

        menu.add_challenge(make_chess_challenge());
        assert!(menu.has_chess_challenge());
        assert_eq!(menu.challenges.len(), 1);
    }

    #[test]
    fn test_take_selected() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.add_challenge(make_chess_challenge());

        let taken = menu.take_selected();
        assert!(taken.is_some());
        assert_eq!(menu.challenges.len(), 1);
    }

    #[test]
    fn test_navigation() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.add_challenge(make_chess_challenge());
        menu.add_challenge(make_chess_challenge());

        assert_eq!(menu.selected_index, 0);
        menu.navigate_down(4);
        assert_eq!(menu.selected_index, 1);
        menu.navigate_down(4);
        assert_eq!(menu.selected_index, 2);
        menu.navigate_down(4); // Can't go past end
        assert_eq!(menu.selected_index, 2);
        menu.navigate_up();
        assert_eq!(menu.selected_index, 1);
    }

    #[test]
    fn test_detail_view_navigation() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.open_detail();

        assert!(menu.viewing_detail);
        assert_eq!(menu.selected_difficulty, 0);

        menu.navigate_down(4); // 4 difficulties
        assert_eq!(menu.selected_difficulty, 1);
        menu.navigate_down(4);
        assert_eq!(menu.selected_difficulty, 2);
        menu.navigate_down(4);
        assert_eq!(menu.selected_difficulty, 3);
        menu.navigate_down(4); // Can't go past 3
        assert_eq!(menu.selected_difficulty, 3);
    }

    #[test]
    fn test_open_close() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());

        menu.open();
        assert!(menu.is_open);

        menu.open_detail();
        assert!(menu.viewing_detail);

        menu.close();
        assert!(!menu.is_open);
        assert!(!menu.viewing_detail);
    }
}
```

**Step 2: Add module declaration to main.rs**

Add after line 25 (after `mod zones;`):

```rust
mod challenge_menu;
```

**Step 3: Run tests to verify**

Run: `cargo test challenge_menu`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/challenge_menu.rs src/main.rs
git commit -m "feat: add challenge menu types and navigation"
```

---

## Task 3: Create chess types module

**Files:**
- Create: `src/chess.rs`
- Modify: `src/main.rs` (add mod declaration)

**Step 1: Create chess types**

Create `src/chess.rs`:

```rust
//! Chess minigame data structures and state management.
//!
//! The chess system allows players to earn prestige ranks by defeating
//! AI opponents of varying difficulty.

use serde::{Deserialize, Serialize};

/// AI difficulty levels with different search depths and random move chances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChessDifficulty {
    Novice,     // 50% random moves, ~500 ELO
    Apprentice, // 1-ply search, ~800 ELO
    Journeyman, // 2-ply search, ~1100 ELO
    Master,     // 3-ply search, ~1350 ELO
}

impl ChessDifficulty {
    pub const ALL: [ChessDifficulty; 4] = [
        ChessDifficulty::Novice,
        ChessDifficulty::Apprentice,
        ChessDifficulty::Journeyman,
        ChessDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(ChessDifficulty::Novice)
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

    pub fn estimated_elo(&self) -> u32 {
        match self {
            Self::Novice => 500,
            Self::Apprentice => 800,
            Self::Journeyman => 1100,
            Self::Master => 1350,
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

/// Persistent chess stats that survive prestige (saved to disk)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChessStats {
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub games_drawn: u32,
    pub prestige_earned: u32,
}

/// Result of a completed chess game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessResult {
    Win,
    Loss,
    Draw,
    Forfeit,
}

/// Active chess game session (transient, not saved)
#[derive(Debug, Clone)]
pub struct ChessGame {
    pub board: chess_engine::Board,
    pub difficulty: ChessDifficulty,
    pub cursor: (u8, u8),
    pub selected_square: Option<(u8, u8)>,
    pub legal_moves: Vec<(u8, u8, chess_engine::Board)>, // (from_file, from_rank, resulting_board)
    pub game_result: Option<ChessResult>,
    pub forfeit_pending: bool,
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub ai_think_target: u32,
    pub ai_pending_board: Option<chess_engine::Board>,
    pub player_is_white: bool,
    pub last_move: Option<((u8, u8), (u8, u8))>, // (from, to)
}

impl ChessGame {
    pub fn new(difficulty: ChessDifficulty) -> Self {
        Self {
            board: chess_engine::Board::default(),
            difficulty,
            cursor: (4, 1), // Start on e2 (king's pawn)
            selected_square: None,
            legal_moves: Vec::new(),
            game_result: None,
            forfeit_pending: false,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_board: None,
            player_is_white: true,
            last_move: None,
        }
    }

    /// Move cursor by delta, clamping to board bounds
    pub fn move_cursor(&mut self, dx: i8, dy: i8) {
        let new_x = (self.cursor.0 as i8 + dx).clamp(0, 7) as u8;
        let new_y = (self.cursor.1 as i8 + dy).clamp(0, 7) as u8;
        self.cursor = (new_x, new_y);
    }

    /// Check if the cursor is on a legal move destination
    pub fn is_legal_destination(&self, file: u8, rank: u8) -> bool {
        self.legal_moves.iter().any(|(_, _, board)| {
            // Check if a piece moved to this square
            self.piece_moved_to(board, file, rank)
        })
    }

    /// Helper to check if a piece moved to a specific square in the resulting board
    fn piece_moved_to(&self, _result_board: &chess_engine::Board, _file: u8, _rank: u8) -> bool {
        // Will implement when we handle the chess-engine Board API
        // For now, return false
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(ChessDifficulty::from_index(0), ChessDifficulty::Novice);
        assert_eq!(ChessDifficulty::from_index(1), ChessDifficulty::Apprentice);
        assert_eq!(ChessDifficulty::from_index(2), ChessDifficulty::Journeyman);
        assert_eq!(ChessDifficulty::from_index(3), ChessDifficulty::Master);
        assert_eq!(ChessDifficulty::from_index(99), ChessDifficulty::Novice); // Out of bounds
    }

    #[test]
    fn test_difficulty_properties() {
        assert_eq!(ChessDifficulty::Novice.random_move_chance(), 0.5);
        assert_eq!(ChessDifficulty::Apprentice.random_move_chance(), 0.0);

        assert_eq!(ChessDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(ChessDifficulty::Master.reward_prestige(), 5);

        assert_eq!(ChessDifficulty::Novice.estimated_elo(), 500);
        assert_eq!(ChessDifficulty::Master.estimated_elo(), 1350);
    }

    #[test]
    fn test_chess_game_new() {
        let game = ChessGame::new(ChessDifficulty::Journeyman);
        assert_eq!(game.difficulty, ChessDifficulty::Journeyman);
        assert_eq!(game.cursor, (4, 1)); // e2
        assert!(game.selected_square.is_none());
        assert!(game.game_result.is_none());
        assert!(!game.ai_thinking);
    }

    #[test]
    fn test_cursor_movement() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.cursor = (3, 3); // d4

        game.move_cursor(1, 0); // Right
        assert_eq!(game.cursor, (4, 3)); // e4

        game.move_cursor(0, 1); // Up
        assert_eq!(game.cursor, (4, 4)); // e5

        game.move_cursor(-1, -1); // Left and down
        assert_eq!(game.cursor, (3, 3)); // d4
    }

    #[test]
    fn test_cursor_bounds() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        game.cursor = (0, 0);
        game.move_cursor(-1, -1); // Try to go negative
        assert_eq!(game.cursor, (0, 0)); // Clamped

        game.cursor = (7, 7);
        game.move_cursor(1, 1); // Try to exceed bounds
        assert_eq!(game.cursor, (7, 7)); // Clamped
    }

    #[test]
    fn test_chess_stats_default() {
        let stats = ChessStats::default();
        assert_eq!(stats.games_played, 0);
        assert_eq!(stats.games_won, 0);
        assert_eq!(stats.prestige_earned, 0);
    }
}
```

**Step 2: Add module declaration to main.rs**

Add after `mod challenge_menu;`:

```rust
mod chess;
```

**Step 3: Run tests**

Run: `cargo test chess::`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/chess.rs src/main.rs
git commit -m "feat: add chess types, difficulty levels, and game state"
```

---

## Task 4: Integrate challenge menu and chess into GameState

**Files:**
- Modify: `src/game_state.rs`
- Modify: `src/lib.rs` (export new modules)

**Step 1: Add fields to GameState**

In `src/game_state.rs`, add imports at top:

```rust
use crate::challenge_menu::ChallengeMenu;
use crate::chess::{ChessGame, ChessStats};
```

Add fields to `GameState` struct (after `zone_progression`):

```rust
    /// Generic challenge menu (transient, not saved)
    #[serde(skip)]
    pub challenge_menu: ChallengeMenu,
    /// Persistent chess stats (survives prestige, saved to disk)
    #[serde(default)]
    pub chess_stats: ChessStats,
    /// Active chess game (transient, not saved)
    #[serde(skip)]
    pub active_chess: Option<ChessGame>,
```

Update the `new()` function to initialize these fields:

```rust
            challenge_menu: ChallengeMenu::new(),
            chess_stats: ChessStats::default(),
            active_chess: None,
```

**Step 2: Update lib.rs exports**

Add to `src/lib.rs`:

```rust
pub mod challenge_menu;
pub mod chess;
```

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/game_state.rs src/lib.rs
git commit -m "feat: integrate challenge menu and chess into GameState"
```

---

## Task 5: Create chess logic module

**Files:**
- Create: `src/chess_logic.rs`
- Modify: `src/main.rs` (add mod declaration)

**Step 1: Create chess logic with discovery and AI**

Create `src/chess_logic.rs`:

```rust
//! Chess game logic: discovery, AI moves, and game resolution.

use crate::challenge_menu::{ChallengeType, PendingChallenge};
use crate::chess::{ChessDifficulty, ChessGame, ChessResult};
use crate::game_state::GameState;
use rand::Rng;

/// Chance per tick to discover a chess challenge (0.5% = ~30-60 min)
pub const CHESS_DISCOVERY_CHANCE: f64 = 0.005;

/// Create a chess challenge for the challenge menu
pub fn create_chess_challenge() -> PendingChallenge {
    PendingChallenge {
        challenge_type: ChallengeType::Chess,
        title: "Chess Challenge".to_string(),
        icon: "♟",
        description: "A mysterious figure challenges you to a game of chess.".to_string(),
    }
}

/// Check if chess discovery conditions are met and roll for discovery
pub fn try_discover_chess<R: Rng>(state: &mut GameState, rng: &mut R) -> bool {
    // Requirements: P1+, not in dungeon, not fishing, not in chess, no pending chess challenge
    if state.prestige_rank < 1 {
        return false;
    }
    if state.active_dungeon.is_some() {
        return false;
    }
    if state.active_fishing.is_some() {
        return false;
    }
    if state.active_chess.is_some() {
        return false;
    }
    if state.challenge_menu.has_chess_challenge() {
        return false;
    }

    // Roll for discovery
    if rng.gen::<f64>() < CHESS_DISCOVERY_CHANCE {
        state.challenge_menu.add_challenge(create_chess_challenge());
        true
    } else {
        false
    }
}

/// Start a chess game with the selected difficulty
pub fn start_chess_game(state: &mut GameState, difficulty: ChessDifficulty) {
    state.active_chess = Some(ChessGame::new(difficulty));
    state.challenge_menu.close();
}

/// Calculate variable AI thinking time in ticks (1.5-6s range)
pub fn calculate_think_ticks<R: Rng>(board: &chess_engine::Board, rng: &mut R) -> u32 {
    let base_ticks = rng.gen_range(15..40); // 1.5-4s base
    let legal_moves = board.get_legal_moves();
    let complexity_bonus = (legal_moves.len() / 5) as u32; // More moves = longer think
    base_ticks + complexity_bonus
}

/// Get AI move with difficulty-based weakening
pub fn get_ai_move<R: Rng>(
    board: &chess_engine::Board,
    difficulty: ChessDifficulty,
    rng: &mut R,
) -> chess_engine::Board {
    let legal_moves = board.get_legal_moves();

    if legal_moves.is_empty() {
        return board.clone();
    }

    // Check for random move (Novice difficulty)
    if rng.gen::<f64>() < difficulty.random_move_chance() {
        let idx = rng.gen_range(0..legal_moves.len());
        return legal_moves[idx].clone();
    }

    // Use search at configured depth
    board.get_best_next_move(difficulty.search_depth())
}

/// Process AI thinking tick, returns true if AI made a move
pub fn process_ai_thinking<R: Rng>(game: &mut ChessGame, rng: &mut R) -> bool {
    if !game.ai_thinking {
        return false;
    }

    game.ai_think_ticks += 1;

    // Compute AI move on first tick
    if game.ai_pending_board.is_none() {
        game.ai_pending_board = Some(get_ai_move(&game.board, game.difficulty, rng));
        game.ai_think_target = calculate_think_ticks(&game.board, rng);
    }

    // Apply move after delay
    if game.ai_think_ticks >= game.ai_think_target {
        if let Some(new_board) = game.ai_pending_board.take() {
            game.board = new_board;
        }
        game.ai_thinking = false;
        game.ai_think_ticks = 0;

        // Check for game over
        check_game_over(game);
        return true;
    }

    false
}

/// Check if the game is over (checkmate or stalemate)
pub fn check_game_over(game: &mut ChessGame) {
    let legal_moves = game.board.get_legal_moves();

    if legal_moves.is_empty() {
        // No legal moves - either checkmate or stalemate
        // The chess-engine crate's Board doesn't have a direct "is_check" method,
        // but we can infer: if no legal moves and king would be captured, it's checkmate
        // For simplicity, we'll treat no legal moves as a loss for the side to move
        // (This is a simplification - proper implementation would check for check)
        game.game_result = Some(if game.player_is_white {
            ChessResult::Loss
        } else {
            ChessResult::Win
        });
    }
}

/// Apply game result: update stats and grant prestige on win
pub fn apply_game_result(state: &mut GameState) -> Option<(ChessResult, u32)> {
    let game = state.active_chess.as_ref()?;
    let result = game.game_result?;
    let difficulty = game.difficulty;

    // Update stats
    state.chess_stats.games_played += 1;

    let prestige_gained = match result {
        ChessResult::Win => {
            state.chess_stats.games_won += 1;
            let reward = difficulty.reward_prestige();
            state.prestige_rank += reward;
            state.chess_stats.prestige_earned += reward;
            reward
        }
        ChessResult::Loss => {
            state.chess_stats.games_lost += 1;
            0
        }
        ChessResult::Draw => {
            state.chess_stats.games_drawn += 1;
            0
        }
        ChessResult::Forfeit => {
            state.chess_stats.games_lost += 1;
            0
        }
    };

    // Clear active game
    state.active_chess = None;

    Some((result, prestige_gained))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_challenge() {
        let challenge = create_chess_challenge();
        assert_eq!(challenge.title, "Chess Challenge");
        assert_eq!(challenge.icon, "♟");
        assert!(matches!(challenge.challenge_type, ChallengeType::Chess));
    }

    #[test]
    fn test_discovery_requirements() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut rng = rand::thread_rng();

        // P0 can't discover
        assert!(!try_discover_chess(&mut state, &mut rng));

        // P1+ can discover (but still random)
        state.prestige_rank = 1;
        // We can't guarantee discovery due to RNG, but at least it won't panic
    }

    #[test]
    fn test_start_chess_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.open();

        start_chess_game(&mut state, ChessDifficulty::Journeyman);

        assert!(state.active_chess.is_some());
        assert!(!state.challenge_menu.is_open);
        let game = state.active_chess.as_ref().unwrap();
        assert_eq!(game.difficulty, ChessDifficulty::Journeyman);
    }

    #[test]
    fn test_apply_win_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = ChessGame::new(ChessDifficulty::Master);
        game.game_result = Some(ChessResult::Win);
        state.active_chess = Some(game);

        let result = apply_game_result(&mut state);

        assert!(result.is_some());
        let (chess_result, prestige) = result.unwrap();
        assert_eq!(chess_result, ChessResult::Win);
        assert_eq!(prestige, 5); // Master reward
        assert_eq!(state.prestige_rank, 10); // 5 + 5
        assert_eq!(state.chess_stats.games_won, 1);
        assert!(state.active_chess.is_none());
    }

    #[test]
    fn test_apply_loss_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.game_result = Some(ChessResult::Loss);
        state.active_chess = Some(game);

        let result = apply_game_result(&mut state);

        assert!(result.is_some());
        let (chess_result, prestige) = result.unwrap();
        assert_eq!(chess_result, ChessResult::Loss);
        assert_eq!(prestige, 0);
        assert_eq!(state.prestige_rank, 5); // Unchanged
        assert_eq!(state.chess_stats.games_lost, 1);
    }
}
```

**Step 2: Add module declaration to main.rs**

Add after `mod chess;`:

```rust
mod chess_logic;
```

**Step 3: Run tests**

Run: `cargo test chess_logic`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/chess_logic.rs src/main.rs
git commit -m "feat: add chess logic - discovery, AI, game resolution"
```

---

## Task 6: Create challenge menu UI scene

**Files:**
- Create: `src/ui/challenge_menu_scene.rs`
- Modify: `src/ui/mod.rs` (add module and export)

**Step 1: Create challenge menu scene**

Create `src/ui/challenge_menu_scene.rs`:

```rust
//! Challenge menu UI rendering.

use crate::challenge_menu::{ChallengeMenu, ChallengeType};
use crate::chess::ChessDifficulty;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the challenge menu (list view or detail view)
pub fn render_challenge_menu(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    // Clear the area first
    frame.render_widget(Clear, area);

    if menu.viewing_detail && !menu.challenges.is_empty() {
        render_detail_view(frame, area, menu);
    } else {
        render_list_view(frame, area, menu);
    }
}

fn render_list_view(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    let block = Block::default()
        .title(" Pending Challenges ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if menu.challenges.is_empty() {
        let text = Paragraph::new("No pending challenges.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, inner);
        return;
    }

    let items: Vec<ListItem> = menu
        .challenges
        .iter()
        .enumerate()
        .map(|(i, challenge)| {
            let prefix = if i == menu.selected_index { "> " } else { "  " };
            let style = if i == menu.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{} {}", prefix, challenge.icon, challenge.title)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Draw help text at bottom
    let help_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(2),
        width: inner.width,
        height: 2,
    };
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "[↑/↓] Navigate  [Enter] View  [Tab/Esc] Close",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(help, help_area);
}

fn render_detail_view(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    let challenge = &menu.challenges[menu.selected_index];

    let block = Block::default()
        .title(format!(" {} ", challenge.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into description, difficulty selector, and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Description
            Constraint::Length(8),  // Difficulty selector
            Constraint::Length(3),  // Outcomes
            Constraint::Min(0),     // Spacer
            Constraint::Length(3),  // Help
        ])
        .split(inner);

    // Description
    let desc = Paragraph::new(challenge.description.clone())
        .style(Style::default().fg(Color::White));
    frame.render_widget(desc, chunks[0]);

    // Difficulty selector (chess-specific)
    if matches!(challenge.challenge_type, ChallengeType::Chess) {
        render_difficulty_selector(frame, chunks[1], menu.selected_difficulty);
    }

    // Outcomes
    let outcomes = Paragraph::new(vec![
        Line::from(Span::styled("Lose: No penalty", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("Draw: Bonus XP", Style::default().fg(Color::Gray))),
    ]);
    frame.render_widget(outcomes, chunks[2]);

    // Help text
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "[↑/↓] Difficulty  [Enter] Accept  [D] Decline  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(help, chunks[4]);
}

fn render_difficulty_selector(frame: &mut Frame, area: Rect, selected: usize) {
    let title = Paragraph::new("Select difficulty:")
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    let title_area = Rect {
        height: 1,
        ..area
    };
    frame.render_widget(title, title_area);

    let options_area = Rect {
        y: area.y + 1,
        height: area.height.saturating_sub(1),
        ..area
    };

    let items: Vec<ListItem> = ChessDifficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, diff)| {
            let prefix = if i == selected { "> " } else { "  " };
            let style = if i == selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let text = format!(
                "{}{:<12} ~{:<5} +{}P",
                prefix,
                diff.name(),
                diff.estimated_elo(),
                diff.reward_prestige()
            );
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, options_area);
}
```

**Step 2: Add module to ui/mod.rs**

Add after `pub mod fishing_scene;`:

```rust
pub mod challenge_menu_scene;
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/ui/challenge_menu_scene.rs src/ui/mod.rs
git commit -m "feat: add challenge menu UI scene"
```

---

## Task 7: Create chess board UI scene

**Files:**
- Create: `src/ui/chess_scene.rs`
- Modify: `src/ui/mod.rs` (add module and export)

**Step 1: Create chess scene**

Create `src/ui/chess_scene.rs`:

```rust
//! Chess board UI rendering.

use crate::chess::{ChessGame, ChessResult};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Unicode chess pieces
const WHITE_PIECES: [char; 6] = ['♔', '♕', '♖', '♗', '♘', '♙'];
const BLACK_PIECES: [char; 6] = ['♚', '♛', '♜', '♝', '♞', '♟'];

/// Render the chess game scene
pub fn render_chess_scene(frame: &mut Frame, area: Rect, game: &ChessGame) {
    frame.render_widget(Clear, area);

    // Check for game over overlay
    if let Some(result) = game.game_result {
        render_game_over_overlay(frame, area, result, game.difficulty.reward_prestige());
        return;
    }

    let block = Block::default()
        .title(" Chess ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into board and status areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Board
            Constraint::Length(3), // Status
        ])
        .split(inner);

    render_board(frame, chunks[0], game);
    render_status(frame, chunks[1], game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &ChessGame) {
    // We need at least 10 rows (8 ranks + 2 for file labels) and ~26 cols
    let board_width = 26;
    let board_height = 10;

    // Center the board
    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    // File labels (a-h)
    let files = "  a  b  c  d  e  f  g  h";
    let top_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        top_labels,
        Rect::new(x_offset, y_offset, board_width, 1),
    );

    // Render each rank (8 down to 1)
    for rank in (0..8).rev() {
        let y = y_offset + 1 + (7 - rank) as u16;
        let rank_label = format!("{}", rank + 1);

        // Left rank label
        let label = Paragraph::new(rank_label.clone()).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(label, Rect::new(x_offset, y, 1, 1));

        // Squares
        for file in 0..8u8 {
            let x = x_offset + 2 + (file as u16 * 3);

            // Determine square color
            let is_light = (file + rank) % 2 == 1;
            let is_cursor = game.cursor == (file, rank);
            let is_selected = game.selected_square == Some((file, rank));

            let bg_color = if is_cursor {
                Color::Yellow
            } else if is_selected {
                Color::Green
            } else if is_light {
                Color::Rgb(200, 200, 180)
            } else {
                Color::Rgb(120, 80, 50)
            };

            // Get piece at this square
            let piece_char = get_piece_at(&game.board, file, rank);
            let piece_str = piece_char.map(|c| c.to_string()).unwrap_or_else(|| " ".to_string());

            let fg_color = if piece_char.map(|c| WHITE_PIECES.contains(&c)).unwrap_or(false) {
                Color::White
            } else {
                Color::Black
            };

            let style = Style::default().fg(fg_color).bg(bg_color);
            let square = Paragraph::new(format!(" {} ", piece_str)).style(style);
            frame.render_widget(square, Rect::new(x, y, 3, 1));
        }

        // Right rank label
        let label_r = Paragraph::new(rank_label).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(label_r, Rect::new(x_offset + 26, y, 1, 1));
    }

    // Bottom file labels
    let bottom_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        bottom_labels,
        Rect::new(x_offset, y_offset + 9, board_width, 1),
    );
}

fn render_status(frame: &mut Frame, area: Rect, game: &ChessGame) {
    let status_text = if game.ai_thinking {
        "Opponent is thinking..."
    } else if game.forfeit_pending {
        "Press Esc again to forfeit"
    } else if game.selected_square.is_some() {
        "Select destination (Enter to confirm, Esc to cancel)"
    } else {
        "Your move (select a piece)"
    };

    let style = if game.ai_thinking {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let status = Paragraph::new(status_text).style(style);
    frame.render_widget(status, area);
}

fn render_game_over_overlay(frame: &mut Frame, area: Rect, result: ChessResult, prestige: u32) {
    frame.render_widget(Clear, area);

    let (title, message, reward) = match result {
        ChessResult::Win => (
            "♔ VICTORY! ♔",
            "You checkmated the mysterious figure!",
            format!("+{} Prestige Ranks", prestige),
        ),
        ChessResult::Loss => (
            "DEFEAT",
            "The mysterious figure has checkmated you.",
            "No penalty incurred.".to_string(),
        ),
        ChessResult::Draw => (
            "DRAW",
            "The game ends in stalemate.",
            "+5000 XP".to_string(),
        ),
        ChessResult::Forfeit => (
            "FORFEIT",
            "You conceded the game.",
            "No penalty incurred.".to_string(),
        ),
    };

    let title_color = match result {
        ChessResult::Win => Color::Green,
        ChessResult::Loss => Color::Red,
        ChessResult::Draw => Color::Yellow,
        ChessResult::Forfeit => Color::Gray,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Center the content
    let content_height = 7;
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

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(
        text,
        Rect::new(inner.x, y_offset, inner.width, content_height),
    );
}

/// Get the piece character at a specific square
fn get_piece_at(board: &chess_engine::Board, file: u8, rank: u8) -> Option<char> {
    // The chess-engine crate represents the board internally
    // We need to query it for the piece at each square
    // This is a simplified version - actual implementation depends on crate API

    // For now, return starting position pieces
    // TODO: Implement proper board state reading from chess_engine::Board
    match (file, rank) {
        // White pieces (rank 0-1)
        (0, 0) | (7, 0) => Some('♖'),
        (1, 0) | (6, 0) => Some('♘'),
        (2, 0) | (5, 0) => Some('♗'),
        (3, 0) => Some('♕'),
        (4, 0) => Some('♔'),
        (_, 1) => Some('♙'),
        // Black pieces (rank 6-7)
        (0, 7) | (7, 7) => Some('♜'),
        (1, 7) | (6, 7) => Some('♞'),
        (2, 7) | (5, 7) => Some('♝'),
        (3, 7) => Some('♛'),
        (4, 7) => Some('♚'),
        (_, 6) => Some('♟'),
        _ => None,
    }
}
```

**Step 2: Add module to ui/mod.rs**

Add after `pub mod challenge_menu_scene;`:

```rust
pub mod chess_scene;
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/ui/chess_scene.rs src/ui/mod.rs
git commit -m "feat: add chess board UI scene with game over overlays"
```

---

## Task 8: Integrate UI rendering dispatch

**Files:**
- Modify: `src/ui/mod.rs`

**Step 1: Update draw_ui_with_update to include chess and challenge menu**

In `src/ui/mod.rs`, update the `draw_ui_with_update` function. Replace the right panel rendering section:

```rust
    // Draw right panel based on current activity
    // Priority: chess > challenge menu > fishing > dungeon > combat
    if let Some(ref game) = game_state.active_chess {
        chess_scene::render_chess_scene(frame, chunks[1], game);
    } else if game_state.challenge_menu.is_open {
        challenge_menu_scene::render_challenge_menu(frame, chunks[1], &game_state.challenge_menu);
    } else if let Some(ref session) = game_state.active_fishing {
        fishing_scene::render_fishing_scene(frame, chunks[1], session, &game_state.fishing);
    } else if let Some(dungeon) = &game_state.active_dungeon {
        draw_dungeon_view(frame, chunks[1], game_state, dungeon);
    } else {
        combat_scene::draw_combat_scene(frame, chunks[1], game_state);
    }
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: integrate chess and challenge menu into UI dispatch"
```

---

## Task 9: Add challenge notification to stats panel

**Files:**
- Modify: `src/ui/stats_panel.rs`

**Step 1: Add challenge pending notification**

In `src/ui/stats_panel.rs`, find where status info is rendered and add a challenge notification section. Add this helper function and call it from the main render:

```rust
/// Render challenge pending notification if any
fn render_challenge_notification(frame: &mut Frame, area: Rect, challenge_count: usize) {
    if challenge_count == 0 {
        return;
    }

    let text = format!(
        "{} challenge{} pending - [Tab] to view",
        challenge_count,
        if challenge_count == 1 { "" } else { "s" }
    );

    let notification = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    frame.render_widget(notification, area);
}
```

Then call this in the appropriate location within `draw_stats_panel_with_update`, passing `game_state.challenge_menu.challenges.len()`.

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat: add challenge pending notification to stats panel"
```

---

## Task 10: Wire up input handling in main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Add chess and challenge menu input handling**

In `main.rs`, find the game loop input handling section. Add handlers for:

1. Active chess game (highest priority)
2. Challenge menu open
3. Tab to open challenge menu
4. Chess discovery during combat tick

This requires significant changes - add imports and input handlers. The key patterns to follow are similar to how fishing and dungeon inputs are handled.

Add these imports at the top:

```rust
use challenge_menu::ChallengeType;
use chess::ChessDifficulty;
use chess_logic::{try_discover_chess, start_chess_game, process_ai_thinking, apply_game_result};
```

In the input handling section, add before existing handlers:

```rust
// PRIORITY 1: Active chess game
if let Some(ref mut chess) = game_state.active_chess {
    if chess.game_result.is_some() {
        // Any key dismisses result
        let _ = apply_game_result(&mut game_state);
    } else if !chess.ai_thinking {
        match key.code {
            KeyCode::Up => chess.move_cursor(0, 1),
            KeyCode::Down => chess.move_cursor(0, -1),
            KeyCode::Left => chess.move_cursor(-1, 0),
            KeyCode::Right => chess.move_cursor(1, 0),
            KeyCode::Enter => {
                // TODO: Implement piece selection and move confirmation
            }
            KeyCode::Esc => {
                if chess.forfeit_pending {
                    chess.game_result = Some(chess::ChessResult::Forfeit);
                } else if chess.selected_square.is_some() {
                    chess.selected_square = None;
                } else {
                    chess.forfeit_pending = true;
                }
            }
            _ => {
                chess.forfeit_pending = false;
            }
        }
    }
    continue; // Don't process other inputs
}

// PRIORITY 2: Challenge menu open
if game_state.challenge_menu.is_open {
    let menu = &mut game_state.challenge_menu;
    if menu.viewing_detail {
        match key.code {
            KeyCode::Up => menu.navigate_up(),
            KeyCode::Down => menu.navigate_down(4), // 4 difficulties
            KeyCode::Enter => {
                if let Some(challenge) = menu.take_selected() {
                    if matches!(challenge.challenge_type, ChallengeType::Chess) {
                        let difficulty = ChessDifficulty::from_index(menu.selected_difficulty);
                        start_chess_game(&mut game_state, difficulty);
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                menu.take_selected(); // Decline - just remove
                menu.close_detail();
                if menu.challenges.is_empty() {
                    menu.close();
                }
            }
            KeyCode::Esc => menu.close_detail(),
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Up => menu.navigate_up(),
            KeyCode::Down => menu.navigate_down(4),
            KeyCode::Enter => menu.open_detail(),
            KeyCode::Tab | KeyCode::Esc => menu.close(),
            _ => {}
        }
    }
    continue;
}

// PRIORITY 3: Tab to open challenge menu
if key.code == KeyCode::Tab && !game_state.challenge_menu.challenges.is_empty() {
    game_state.challenge_menu.open();
    continue;
}
```

In the tick processing section, add chess AI processing and discovery:

```rust
// Process chess AI thinking
if let Some(ref mut chess) = game_state.active_chess {
    process_ai_thinking(chess, &mut rng);
}

// Try to discover chess challenge during normal combat
if game_state.active_chess.is_none()
    && game_state.active_dungeon.is_none()
    && game_state.active_fishing.is_none()
{
    try_discover_chess(&mut game_state, &mut rng);
}
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds (may have warnings about unused variables)

**Step 3: Run the game and test**

Run: `cargo run`
Expected: Game runs, Tab opens challenge menu if challenges exist

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up chess and challenge menu input handling"
```

---

## Task 11: Implement chess-engine board state reading

**Files:**
- Modify: `src/ui/chess_scene.rs`
- Modify: `src/chess.rs`

**Step 1: Implement proper board state reading**

The chess-engine crate's Board type needs to be queried for piece positions. Update the `get_piece_at` function in `chess_scene.rs` to properly read from the board.

Research the chess-engine crate API and implement proper piece position querying. The crate uses a `Board` struct that can be queried for pieces.

**Step 2: Implement move selection in ChessGame**

Update `src/chess.rs` to properly handle:
- Getting legal moves for a selected piece
- Filtering legal moves to show valid destinations
- Applying a move when confirmed

**Step 3: Test with actual gameplay**

Run: `cargo run`
Expected: Can move pieces on the board

**Step 4: Commit**

```bash
git add src/ui/chess_scene.rs src/chess.rs
git commit -m "feat: implement board state reading and move selection"
```

---

## Task 12: Add combat log messages

**Files:**
- Modify: `src/chess_logic.rs`
- Modify: `src/main.rs`

**Step 1: Add combat log entries for chess events**

When chess is discovered, add log entry:
```
"♟ A mysterious figure steps from the shadows..."
"♟ Press [Tab] to view pending challenges"
```

When game ends, add appropriate log entry based on result.

**Step 2: Verify log messages appear**

Run: `cargo run`
Expected: Combat log shows chess-related messages

**Step 3: Commit**

```bash
git add src/chess_logic.rs src/main.rs
git commit -m "feat: add combat log messages for chess events"
```

---

## Task 13: Final integration testing

**Files:**
- Create: `tests/chess_integration_test.rs`

**Step 1: Create integration test for chess flow**

```rust
//! Integration test: Chess minigame flow

use quest::chess::{ChessDifficulty, ChessGame, ChessResult};
use quest::chess_logic::{apply_game_result, start_chess_game};
use quest::game_state::GameState;

#[test]
fn test_complete_chess_win_flow() {
    let mut state = GameState::new("Chess Master".to_string(), 0);
    state.prestige_rank = 5;

    // Start a chess game
    start_chess_game(&mut state, ChessDifficulty::Master);
    assert!(state.active_chess.is_some());

    // Simulate a win
    state.active_chess.as_mut().unwrap().game_result = Some(ChessResult::Win);

    // Apply result
    let result = apply_game_result(&mut state);
    assert!(result.is_some());

    let (chess_result, prestige) = result.unwrap();
    assert_eq!(chess_result, ChessResult::Win);
    assert_eq!(prestige, 5); // Master reward
    assert_eq!(state.prestige_rank, 10); // 5 + 5
    assert!(state.active_chess.is_none());
}

#[test]
fn test_chess_loss_no_penalty() {
    let mut state = GameState::new("Chess Learner".to_string(), 0);
    state.prestige_rank = 3;

    start_chess_game(&mut state, ChessDifficulty::Novice);
    state.active_chess.as_mut().unwrap().game_result = Some(ChessResult::Loss);

    let result = apply_game_result(&mut state);
    let (_, prestige) = result.unwrap();

    assert_eq!(prestige, 0);
    assert_eq!(state.prestige_rank, 3); // Unchanged
}

#[test]
fn test_difficulty_rewards() {
    assert_eq!(ChessDifficulty::Novice.reward_prestige(), 1);
    assert_eq!(ChessDifficulty::Apprentice.reward_prestige(), 2);
    assert_eq!(ChessDifficulty::Journeyman.reward_prestige(), 3);
    assert_eq!(ChessDifficulty::Master.reward_prestige(), 5);
}
```

**Step 2: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 3: Run full CI checks**

Run: `make check`
Expected: All checks pass

**Step 4: Commit**

```bash
git add tests/chess_integration_test.rs
git commit -m "test: add chess integration tests"
```

---

## Summary

This implementation plan covers:

1. **Tasks 1-4**: Foundation - dependency, types, GameState integration
2. **Tasks 5**: Core logic - discovery, AI, game resolution
3. **Tasks 6-9**: UI - challenge menu, chess board, stats panel notification
4. **Tasks 10-12**: Integration - input handling, board state, combat log
5. **Task 13**: Testing - integration tests

Total: ~13 tasks, each with clear steps and verification.

**Key files created:**
- `src/challenge_menu.rs` - Generic challenge menu system
- `src/chess.rs` - Chess types and game state
- `src/chess_logic.rs` - Chess game logic
- `src/ui/challenge_menu_scene.rs` - Challenge menu UI
- `src/ui/chess_scene.rs` - Chess board UI
- `tests/chess_integration_test.rs` - Integration tests

**Key files modified:**
- `Cargo.toml` - chess-engine dependency
- `src/main.rs` - Module declarations, input handling, tick processing
- `src/game_state.rs` - New fields for chess
- `src/lib.rs` - Module exports
- `src/ui/mod.rs` - UI dispatch
- `src/ui/stats_panel.rs` - Challenge notification
