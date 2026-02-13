# Chess Puzzle Architecture

Technical architecture for the Chess Puzzle challenge minigame. Chess puzzles present pre-arranged board positions where the player must find the correct move or checkmate sequence.

**Key constraint**: The `chess-engine` crate does NOT support FEN strings. All board positions must be reached by playing a sequence of moves from the standard starting position.

---

## 1. Module Structure

```
src/challenges/chess_puzzle/
├── mod.rs       # Public exports
├── types.rs     # ChessPuzzleDifficulty, ChessPuzzleGame, PuzzleDef, PuzzleSolution, etc.
├── logic.rs     # Input processing, move validation, puzzle advancement, apply_game_result
└── puzzles.rs   # Static puzzle definitions organized by difficulty
```

---

## 2. Type Definitions (`types.rs`)

### Difficulty Enum

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChessPuzzleDifficulty {
    Novice,      // Mate-in-1 puzzles
    Apprentice,  // Simple tactics (forks, pins, skewers)
    Journeyman,  // Mate-in-2 puzzles
    Master,      // Complex tactics (sacrifices, back rank mates)
}

difficulty_enum_impl!(ChessPuzzleDifficulty);
```

Uses the existing `difficulty_enum_impl!` macro from `challenges/mod.rs` for `ALL`, `from_index()`, and `name()`.

### Puzzle Definition

```rust
/// A single chess puzzle definition (static data).
pub struct PuzzleDef {
    /// Short display title (e.g., "Back Rank Mate", "Knight Fork")
    pub title: &'static str,
    /// Hint text shown in the info panel
    pub hint: &'static str,
    /// Moves from standard starting position to reach the puzzle position.
    /// Each tuple is (from_rank, from_file, to_rank, to_file) for Move::Piece.
    /// Uses chess-engine Position::new(rank, file) coordinates:
    ///   rank 0-7 = ranks 1-8, file 0-7 = files a-h
    pub setup_moves: &'static [(i32, i32, i32, i32)],
    /// Which color the player plays as
    pub player_is_white: bool,
    /// The expected solution
    pub solution: PuzzleSolution,
}
```

### Puzzle Solution

```rust
/// How the puzzle is validated.
#[derive(Debug, Clone)]
pub enum PuzzleSolution {
    /// Player must deliver checkmate in one move.
    /// No specific move is checked — any move that results in checkmate is correct.
    MateInOne,

    /// Player must find the specific best move (for tactics: forks, pins, skewers).
    /// Tuple is (from_rank, from_file, to_rank, to_file).
    BestMove(i32, i32, i32, i32),

    /// Player must deliver checkmate in two moves.
    /// move1: player's first move (from_rank, from_file, to_rank, to_file)
    /// After move1, the engine plays the best response automatically.
    /// move2: player's second move that must result in checkmate.
    MateInTwo {
        move1: (i32, i32, i32, i32),
        move2: (i32, i32, i32, i32),
    },
}
```

**Design rationale for MateInOne**: Rather than storing a specific solution move, we check `board.is_checkmate()` after the player's move. This means any valid checkmate is accepted, which is more forgiving and avoids rejecting alternate checkmates that also work.

**Design rationale for BestMove**: For tactics puzzles (forks, pins, skewers), the player must find the specific move. This is necessary because the "best" move creates a positional advantage that can't be detected by simple checkmate checking.

**Design rationale for MateInTwo**: Stores both player moves explicitly. After the player's first move, the AI plays its best response (using `board.get_best_next_move(3)`), then the player makes their second move. We verify the second move results in checkmate. The AI response is not constrained to a specific move — whatever the engine picks, the puzzle's move2 must still produce checkmate. This means puzzles must be designed so that move2 delivers checkmate regardless of the AI's defense.

### Puzzle State

```rust
/// Current state within a single puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleState {
    /// Player is choosing their move
    Solving,
    /// Player found the correct move/sequence — brief "Correct!" feedback
    Correct,
    /// Player made the wrong move — brief "Wrong" feedback
    Wrong,
    /// Waiting for AI response in mate-in-2 (after player's first move)
    WaitingForAI,
}
```

### Persistent Stats

```rust
/// Persistent chess puzzle stats (saved to disk)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChessPuzzleStats {
    pub sessions_played: u32,
    pub sessions_won: u32,
    pub sessions_lost: u32,
    pub puzzles_solved: u32,
    pub puzzles_attempted: u32,
}
```

### Game Result

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessPuzzleResult {
    Win,
    Loss,
}
```

No Draw variant — puzzles are won or lost, never drawn.

### Main Game State

```rust
/// Active chess puzzle session (transient, not saved).
#[derive(Debug, Clone)]
pub struct ChessPuzzleGame {
    // ── Puzzle set ──
    pub difficulty: ChessPuzzleDifficulty,
    /// Ordered list of puzzle indices into the difficulty's puzzle array.
    /// Shuffled at session start for variety.
    pub puzzle_order: Vec<usize>,
    /// Index into puzzle_order (which puzzle we're on)
    pub current_puzzle_index: usize,

    // ── Scoring ──
    pub puzzles_solved: u32,
    pub puzzles_attempted: u32,
    /// How many puzzles the player must solve to win
    pub target_score: u32,
    /// Total puzzles in this session
    pub total_puzzles: u32,

    // ── Board state ──
    /// The chess board at the current puzzle's starting position
    pub board: chess_engine::Board,
    /// Whether the player plays white in the current puzzle
    pub player_is_white: bool,

    // ── Cursor/selection (reuse chess patterns) ──
    pub cursor: (u8, u8),
    pub selected_square: Option<(u8, u8)>,
    pub legal_move_destinations: Vec<(u8, u8)>,

    // ── Puzzle flow ──
    pub puzzle_state: PuzzleState,
    /// Ticks remaining for Correct/Wrong feedback display (10 ticks = 1s)
    pub feedback_ticks: u32,
    /// For mate-in-2: tracks which move the player is on (0 = first, 1 = second)
    pub move_number_in_puzzle: u8,
    /// AI thinking state for mate-in-2 intermediate response
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub ai_think_target: u32,
    pub ai_pending_board: Option<chess_engine::Board>,

    // ── Game result ──
    pub game_result: Option<ChessPuzzleResult>,
    pub forfeit_pending: bool,

    // ── Display ──
    /// Last move highlight (from, to)
    pub last_move: Option<((u8, u8), (u8, u8))>,
}
```

### Constructor

```rust
impl ChessPuzzleGame {
    pub fn new(difficulty: ChessPuzzleDifficulty) -> Self {
        let puzzles = get_puzzles(difficulty);
        let total = puzzles.len();
        let target = difficulty.target_score();

        // Shuffle puzzle order for variety
        let mut puzzle_order: Vec<usize> = (0..total).collect();
        // Shuffle using rand (done in logic.rs start function)

        let mut game = Self {
            difficulty,
            puzzle_order,
            current_puzzle_index: 0,
            puzzles_solved: 0,
            puzzles_attempted: 0,
            target_score: target,
            total_puzzles: total as u32,
            board: chess_engine::Board::default(), // placeholder
            player_is_white: true,
            cursor: (4, 3), // center of board
            selected_square: None,
            legal_move_destinations: Vec::new(),
            puzzle_state: PuzzleState::Solving,
            feedback_ticks: 0,
            move_number_in_puzzle: 0,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_board: None,
            game_result: None,
            forfeit_pending: false,
            last_move: None,
        };

        // Set up the first puzzle's board position
        // (done via setup_current_puzzle in logic.rs)
        game
    }
}
```

### Difficulty Parameters

```rust
impl ChessPuzzleDifficulty {
    pub fn target_score(&self) -> u32 {
        match self {
            Self::Novice => 3,      // Solve 3 out of ~6 mate-in-1
            Self::Apprentice => 4,  // Solve 4 out of ~6 tactics
            Self::Journeyman => 3,  // Solve 3 out of ~5 mate-in-2
            Self::Master => 3,      // Solve 3 out of ~5 complex tactics
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
```

**Reward rationale**: Identical to regular Chess rewards (prestige-only). Chess puzzles test tactical knowledge rather than endurance, so they earn the same prestige as a full chess game at the equivalent difficulty.

---

## 3. Puzzle Definitions (`puzzles.rs`)

```rust
use super::types::{ChessPuzzleDifficulty, PuzzleDef, PuzzleSolution};

/// Get the puzzle set for a given difficulty.
pub fn get_puzzles(difficulty: ChessPuzzleDifficulty) -> &'static [PuzzleDef] {
    match difficulty {
        ChessPuzzleDifficulty::Novice => NOVICE_PUZZLES,
        ChessPuzzleDifficulty::Apprentice => APPRENTICE_PUZZLES,
        ChessPuzzleDifficulty::Journeyman => JOURNEYMAN_PUZZLES,
        ChessPuzzleDifficulty::Master => MASTER_PUZZLES,
    }
}

/// Novice: Mate-in-1 puzzles (5-8 puzzles)
static NOVICE_PUZZLES: &[PuzzleDef] = &[
    PuzzleDef {
        title: "Scholar's Mate",
        hint: "The queen delivers checkmate",
        setup_moves: &[
            // 1. e4 e5  2. Bc4 Nc6  3. Qh5 Nf6??
            (1, 4, 3, 4), // e2-e4
            (6, 4, 4, 4), // e7-e5
            (0, 5, 3, 2), // Bf1-c4  (actually Bc4: rank0,file5 -> rank3,file2)
            (7, 1, 5, 2), // Nb8-c6
            (0, 3, 4, 7), // Qd1-h5
            (7, 6, 5, 5), // Ng8-f6??
        ],
        player_is_white: true,
        solution: PuzzleSolution::MateInOne,
        // Solution: Qh5xf7# (Qxf7 is checkmate)
    },
    // ... more puzzles (provided by chess grandmaster in Task #1)
];

static APPRENTICE_PUZZLES: &[PuzzleDef] = &[
    // Simple tactics: forks, pins, skewers
    // Solution is BestMove(from_rank, from_file, to_rank, to_file)
];

static JOURNEYMAN_PUZZLES: &[PuzzleDef] = &[
    // Mate-in-2: MateInTwo { move1, move2 }
];

static MASTER_PUZZLES: &[PuzzleDef] = &[
    // Complex tactics: sacrifices, discovered attacks
    // Mix of BestMove and MateInTwo
];
```

**Puzzle data comes from Task #1** (chess grandmaster). The architecture supports any number of puzzles per difficulty. The `setup_moves` field uses chess-engine's `Position::new(rank, file)` coordinate system where rank 0 = rank 1 and file 0 = file a.

---

## 4. Logic (`logic.rs`)

### Input Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessPuzzleInput {
    Up,
    Down,
    Left,
    Right,
    Select,  // Enter — select piece or confirm move
    Forfeit, // Esc — clear selection or forfeit
    Other,
}
```

Same key mapping as regular Chess. Reuses the same arrow/Enter/Esc pattern.

### Setup Puzzle

```rust
/// Set up the board for the current puzzle by replaying setup_moves from the starting position.
pub fn setup_current_puzzle(game: &mut ChessPuzzleGame) {
    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    // Start from standard position
    let mut board = chess_engine::Board::default();

    // Replay setup moves
    for &(from_rank, from_file, to_rank, to_file) in puzzle.setup_moves {
        let from = chess_engine::Position::new(from_rank, from_file);
        let to = chess_engine::Position::new(to_rank, to_file);
        let m = chess_engine::Move::Piece(from, to);
        match board.play_move(m) {
            chess_engine::GameResult::Continuing(new_board) => {
                board = new_board;
            }
            _ => {
                // Setup move resulted in game end — puzzle definition is broken.
                // In release builds, skip this puzzle.
                // In debug builds, panic.
                debug_assert!(false, "Puzzle setup move ended game: {:?}", puzzle.title);
                return;
            }
        }
    }

    game.board = board;
    game.player_is_white = puzzle.player_is_white;
    game.puzzle_state = PuzzleState::Solving;
    game.move_number_in_puzzle = 0;
    game.selected_square = None;
    game.legal_move_destinations.clear();
    game.last_move = None;
    game.forfeit_pending = false;

    // Place cursor near center of board
    game.cursor = if puzzle.player_is_white { (4, 3) } else { (4, 4) };
}
```

### Process Input

```rust
/// Process player input during the puzzle.
pub fn process_input(game: &mut ChessPuzzleGame, input: ChessPuzzleInput) -> bool {
    // Block input during AI thinking or feedback display
    if game.ai_thinking || game.puzzle_state == PuzzleState::Correct
        || game.puzzle_state == PuzzleState::Wrong
    {
        return false;
    }

    match input {
        ChessPuzzleInput::Up => game.move_cursor(0, 1),
        ChessPuzzleInput::Down => game.move_cursor(0, -1),
        ChessPuzzleInput::Left => game.move_cursor(-1, 0),
        ChessPuzzleInput::Right => game.move_cursor(1, 0),
        ChessPuzzleInput::Select => process_select(game),
        ChessPuzzleInput::Forfeit => process_cancel(game),
        ChessPuzzleInput::Other => {
            game.forfeit_pending = false;
        }
    }
    true
}
```

`move_cursor` reuses the same clamping logic from `ChessGame::move_cursor`.

### Move Validation

When the player makes a move, we validate it against the puzzle solution:

```rust
fn validate_player_move(game: &mut ChessPuzzleGame, from: (u8, u8), to: (u8, u8)) {
    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    // Apply the move to a temporary board to check result
    let from_pos = chess_engine::Position::new(from.1 as i32, from.0 as i32);
    let to_pos = chess_engine::Position::new(to.1 as i32, to.0 as i32);
    let player_move = chess_engine::Move::Piece(from_pos, to_pos);

    match &puzzle.solution {
        PuzzleSolution::MateInOne => {
            match game.board.play_move(player_move) {
                chess_engine::GameResult::Victory(_) => {
                    // Checkmate! Correct.
                    game.board = chess_engine::Board::default(); // doesn't matter, we show feedback
                    mark_correct(game);
                }
                chess_engine::GameResult::Continuing(new_board) => {
                    // Check if checkmate (Victory catches this, but also check is_checkmate)
                    if new_board.is_checkmate() {
                        mark_correct(game);
                    } else {
                        mark_wrong(game);
                    }
                    game.board = new_board;
                }
                _ => {
                    mark_wrong(game);
                }
            }
        }

        PuzzleSolution::BestMove(exp_fr, exp_ff, exp_tr, exp_tf) => {
            // Check if the move matches the expected best move
            if from.1 as i32 == *exp_fr && from.0 as i32 == *exp_ff
                && to.1 as i32 == *exp_tr && to.0 as i32 == *exp_tf
            {
                // Apply the move to update the board visually
                if let chess_engine::GameResult::Continuing(new_board) =
                    game.board.play_move(player_move)
                {
                    game.board = new_board;
                }
                mark_correct(game);
            } else {
                // Wrong move
                if let chess_engine::GameResult::Continuing(new_board) =
                    game.board.play_move(player_move)
                {
                    game.board = new_board;
                }
                mark_wrong(game);
            }
        }

        PuzzleSolution::MateInTwo { move1, move2 } => {
            if game.move_number_in_puzzle == 0 {
                // First move: check if it matches move1
                let (exp_fr, exp_ff, exp_tr, exp_tf) = move1;
                if from.1 as i32 == *exp_fr && from.0 as i32 == *exp_ff
                    && to.1 as i32 == *exp_tr && to.0 as i32 == *exp_tf
                {
                    // Correct first move — apply and trigger AI response
                    if let chess_engine::GameResult::Continuing(new_board) =
                        game.board.play_move(player_move)
                    {
                        game.board = new_board;
                        game.move_number_in_puzzle = 1;
                        game.last_move = Some((from, to));
                        game.selected_square = None;
                        game.legal_move_destinations.clear();
                        // Start AI thinking for the forced response
                        game.ai_thinking = true;
                        game.ai_think_ticks = 0;
                        game.ai_think_target = 8; // ~0.8 seconds
                    }
                } else {
                    mark_wrong(game);
                }
            } else {
                // Second move: check if it results in checkmate
                match game.board.play_move(player_move) {
                    chess_engine::GameResult::Victory(_) => {
                        mark_correct(game);
                    }
                    chess_engine::GameResult::Continuing(new_board) => {
                        if new_board.is_checkmate() {
                            mark_correct(game);
                        } else {
                            game.board = new_board;
                            mark_wrong(game);
                        }
                    }
                    _ => {
                        mark_wrong(game);
                    }
                }
            }
        }
    }
}

fn mark_correct(game: &mut ChessPuzzleGame) {
    game.puzzle_state = PuzzleState::Correct;
    game.puzzles_solved += 1;
    game.puzzles_attempted += 1;
    game.feedback_ticks = 10; // 1 second at 100ms tick
    game.selected_square = None;
    game.legal_move_destinations.clear();
}

fn mark_wrong(game: &mut ChessPuzzleGame) {
    game.puzzle_state = PuzzleState::Wrong;
    game.puzzles_attempted += 1;
    game.feedback_ticks = 10; // 1 second
    game.selected_square = None;
    game.legal_move_destinations.clear();
}
```

### AI Thinking (Mate-in-2 intermediate response)

```rust
/// Process AI thinking tick for mate-in-2 intermediate response.
/// Called from game_tick() at 100ms intervals.
pub fn process_ai_thinking(game: &mut ChessPuzzleGame) {
    if !game.ai_thinking {
        return;
    }

    game.ai_think_ticks += 1;

    // Compute AI response on first tick
    if game.ai_pending_board.is_none() {
        // Use 3-ply search for the AI's best defense
        let (best_move, _, _) = game.board.get_best_next_move(3);
        if let chess_engine::GameResult::Continuing(new_board) =
            game.board.play_move(best_move)
        {
            game.ai_pending_board = Some(new_board);
        }
    }

    // Apply after delay
    if game.ai_think_ticks >= game.ai_think_target {
        if let Some(new_board) = game.ai_pending_board.take() {
            game.board = new_board;
        }
        game.ai_thinking = false;
        game.ai_think_ticks = 0;
        // Player now needs to make their second move
        game.puzzle_state = PuzzleState::Solving;
    }
}
```

**No RNG parameter needed**: Unlike regular Chess AI which uses `Rng` for variable think time and random moves, the puzzle AI uses a fixed think delay and always plays the best response. This keeps the function signature simple.

### Feedback Tick and Puzzle Advancement

```rust
/// Process feedback countdown and advance to next puzzle.
/// Called from game_tick() at 100ms intervals.
pub fn tick_feedback(game: &mut ChessPuzzleGame) {
    if game.puzzle_state != PuzzleState::Correct && game.puzzle_state != PuzzleState::Wrong {
        return;
    }

    if game.feedback_ticks > 0 {
        game.feedback_ticks -= 1;
        return;
    }

    // Feedback period over — advance to next puzzle or end session
    game.current_puzzle_index += 1;

    // Check if player has already won (reached target)
    if game.puzzles_solved >= game.target_score {
        game.game_result = Some(ChessPuzzleResult::Win);
        return;
    }

    // Check if winning is still mathematically possible
    let remaining = game.total_puzzles - game.current_puzzle_index as u32;
    if game.puzzles_solved + remaining < game.target_score {
        // Can't win anymore
        game.game_result = Some(ChessPuzzleResult::Loss);
        return;
    }

    // Check if all puzzles exhausted
    if game.current_puzzle_index >= game.puzzle_order.len() {
        // Ran out of puzzles without reaching target
        game.game_result = Some(ChessPuzzleResult::Loss);
        return;
    }

    // Set up next puzzle
    setup_current_puzzle(game);
}
```

### Apply Game Result

```rust
/// Apply game result: update stats, grant rewards, and add combat log entries.
pub fn apply_game_result(state: &mut GameState) -> Option<crate::challenges::MinigameWinInfo> {
    use crate::challenges::menu::DifficultyInfo;
    use crate::challenges::{apply_challenge_rewards, GameResultInfo};

    let game = match state.active_minigame.as_ref() {
        Some(ActiveMinigame::ChessPuzzle(g)) => g,
        _ => return None,
    };
    let result = game.game_result?;
    let difficulty = game.difficulty;
    let reward = difficulty.reward();

    // Stats tracking
    state.chess_puzzle_stats.sessions_played += 1;
    state.chess_puzzle_stats.puzzles_solved += game.puzzles_solved;
    state.chess_puzzle_stats.puzzles_attempted += game.puzzles_attempted;

    let (won, loss_message) = match result {
        ChessPuzzleResult::Win => {
            state.chess_puzzle_stats.sessions_won += 1;
            (true, "")
        }
        ChessPuzzleResult::Loss => {
            state.chess_puzzle_stats.sessions_lost += 1;
            (
                false,
                "The puzzle master shakes their head slowly and fades away.",
            )
        }
    };

    apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "chess_puzzle",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "\u{265E}", // Knight symbol ♞
            win_message: &format!(
                "Puzzle mastery! Solved {}/{} puzzles.",
                game.puzzles_solved, game.total_puzzles
            ),
            loss_message,
        },
    )
}
```

**Note on `win_message`**: This needs a `format!()` to include the solve count. Since `GameResultInfo` expects `&'static str`, we'll need to either: (a) use a pre-formatted String stored on the game struct and pass a reference, or (b) handle the combat log entry directly before calling `apply_challenge_rewards`. Option (b) is cleaner — emit the puzzle-specific log line first, then call the shared helper with a generic message. We'll finalize this during implementation.

### Select/Cancel Helpers

These follow the exact same patterns as regular chess (`chess/logic.rs`):

```rust
fn process_select(game: &mut ChessPuzzleGame) {
    if game.selected_square.is_some() {
        if game.legal_move_destinations.contains(&game.cursor) {
            // Try to make the move
            let from = game.selected_square.unwrap();
            let to = game.cursor;
            game.last_move = Some((from, to));
            validate_player_move(game, from, to);
        } else if cursor_on_player_piece(game) {
            select_piece_at_cursor(game);
        } else {
            game.selected_square = None;
            game.legal_move_destinations.clear();
        }
    } else {
        select_piece_at_cursor(game);
    }
}

fn process_cancel(game: &mut ChessPuzzleGame) {
    if game.forfeit_pending {
        game.game_result = Some(ChessPuzzleResult::Loss);
    } else if game.selected_square.is_some() {
        game.selected_square = None;
        game.legal_move_destinations.clear();
        game.forfeit_pending = false;
    } else {
        game.forfeit_pending = true;
    }
}
```

`select_piece_at_cursor()` and `cursor_on_player_piece()` are implemented as free functions in `logic.rs` (not methods on ChessPuzzleGame) to keep the type definition in `types.rs` lean. They replicate the same logic from `ChessGame::select_piece_at_cursor()` and `ChessGame::cursor_on_player_piece()`.

---

## 5. Tick Integration (`src/core/tick.rs`)

Chess puzzles need two tick-driven processes:

1. **AI thinking** for mate-in-2 intermediate responses
2. **Feedback countdown** for Correct/Wrong display before advancing

Both are called from `game_tick()` Section 1 (challenge AI thinking):

```rust
// In game_tick(), Section 1:
match &mut state.active_minigame {
    Some(ActiveMinigame::Chess(game)) => {
        crate::challenges::chess::logic::process_ai_thinking(game, rng);
    }
    // ... existing arms ...
    Some(ActiveMinigame::ChessPuzzle(game)) => {
        crate::challenges::chess_puzzle::logic::process_ai_thinking(game);
        crate::challenges::chess_puzzle::logic::tick_feedback(game);
    }
    _ => {}
}
```

**No RNG needed**: The puzzle AI always plays the best response (no randomization), and think time is fixed, so the tick functions don't need an `&mut R: Rng` parameter.

---

## 6. UI Approach (`src/ui/chess_puzzle_scene.rs`)

### Layout

Uses `create_game_layout()` with:
- Title: `" Chess Puzzles "`
- Border color: `Color::LightGreen` (distinct from regular Chess's `Color::Cyan`)
- Content height: 20 (1 for puzzle progress + 1 for puzzle title + 18 for board)
- Info panel width: 22

```
┌─ Chess Puzzles ──────────────────┬─ Info ──────────┐
│ Puzzle 3/6 — Solved: 2           │ PUZZLE           │
│ "Back Rank Mate"                 │                  │
│                                  │ Hint:            │
│   ┌────┬────┬────┬────┬...       │ Use the rook to  │
│   │    │    │    │    │          │ deliver mate.    │
│   ├────┼────┼────┼────┼...       │                  │
│   │    │ ♜  │    │    │          │ Difficulty:      │
│   │    │    │    │    │          │ Novice           │
│   ...                            │                  │
│   └────┴────┴────┴────┴...       │ Target: 3/6      │
│ [status bar]                     │                  │
└──────────────────────────────────┴──────────────────┘
```

### Board Rendering

**Reuse `get_piece_at()` and `piece_color()` from `chess_scene.rs`**: These are currently private functions in `chess_scene.rs`. To share them, either:
- (a) Move them to a shared module (e.g., `ui/chess_common.rs`) — preferred
- (b) Duplicate them in `chess_puzzle_scene.rs` — simpler, acceptable for 2 small functions

Decision: **(a) Extract to `ui/chess_common.rs`** since both scenes need identical board rendering. This module would contain `get_piece_at()`, `piece_color()`, and the board grid rendering function.

The board rendering is identical to `chess_scene.rs::render_board()` with one exception: no AI thinking state (puzzles don't show "Opponent is thinking..." on the board itself — that's shown in the status bar during mate-in-2 AI response).

### Status Bar States

```rust
fn render_status(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    // AI thinking (mate-in-2 intermediate response)
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent responds...");
        return;
    }

    // Feedback display
    match game.puzzle_state {
        PuzzleState::Correct => {
            render_status_bar(frame, area, "Correct!", Color::Green, &[]);
            return;
        }
        PuzzleState::Wrong => {
            render_status_bar(frame, area, "Wrong move", Color::Red, &[]);
            return;
        }
        _ => {}
    }

    // Forfeit confirmation
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    // Normal controls
    let (status_text, status_color) = if game.selected_square.is_some() {
        ("Select destination", Color::Cyan)
    } else {
        ("Find the best move", Color::White)
    };

    let controls: &[(&str, &str)] = if game.selected_square.is_some() {
        &[("[Arrows]", "Move"), ("[Enter]", "Confirm"), ("[Esc]", "Cancel")]
    } else {
        &[("[Arrows]", "Move"), ("[Enter]", "Select"), ("[Esc]", "Forfeit")]
    };

    render_status_bar(frame, area, status_text, status_color, controls);
}
```

### Info Panel

```rust
fn render_info_panel(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    let inner = render_info_panel_frame(frame, area);

    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    let lines = vec![
        Line::from(Span::styled("PUZZLE", Style::default().fg(Color::Yellow).bold())),
        Line::from(""),
        Line::from(Span::styled(puzzle.hint, Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::LightGreen)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Solved: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.puzzles_solved, game.target_score),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Puzzle: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.current_puzzle_index + 1, game.total_puzzles),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}
```

### Game Over

Reuses `render_game_over_banner()` from `game_common.rs`:

```rust
fn render_game_over(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    let result = game.game_result.unwrap();
    let prestige = game.difficulty.reward_prestige();

    let (result_type, title, message, reward) = match result {
        ChessPuzzleResult::Win => (
            GameResultType::Win,
            "PUZZLES COMPLETE!",
            &format!("Solved {}/{}", game.puzzles_solved, game.total_puzzles),
            format!("+{} Prestige Ranks", prestige),
        ),
        ChessPuzzleResult::Loss => {
            if game.forfeit_pending {
                (GameResultType::Forfeit, "FORFEIT", "You gave up", String::new())
            } else {
                (
                    GameResultType::Loss,
                    "FAILED",
                    &format!("Solved {}/{}", game.puzzles_solved, game.target_score),
                    String::new(),
                )
            }
        }
    };

    render_game_over_banner(frame, area, result_type, title, message, &reward);
}
```

---

## 7. Integration Checklist

Every file that needs modification, organized by type:

### NEW Files

| File | Contents |
|------|----------|
| `src/challenges/chess_puzzle/mod.rs` | Module declarations, re-exports |
| `src/challenges/chess_puzzle/types.rs` | ChessPuzzleDifficulty, ChessPuzzleGame, PuzzleDef, PuzzleSolution, PuzzleState, ChessPuzzleResult, ChessPuzzleStats |
| `src/challenges/chess_puzzle/logic.rs` | ChessPuzzleInput, process_input, setup_current_puzzle, validate_player_move, process_ai_thinking, tick_feedback, apply_game_result, tests |
| `src/challenges/chess_puzzle/puzzles.rs` | Static puzzle definitions by difficulty (from Task #1) |
| `src/ui/chess_puzzle_scene.rs` | render_chess_puzzle_scene, board rendering, status bar, info panel, game over |

### MODIFY Files

| File | Changes |
|------|---------|
| `src/challenges/mod.rs` | Add `pub mod chess_puzzle;`, add `ChessPuzzle(Box<ChessPuzzleGame>)` to `ActiveMinigame`, add re-exports |
| `src/challenges/menu.rs` | Add `ChessPuzzle` to `ChallengeType` enum, icon(), discovery_flavor(), create_challenge(). Add `ChessPuzzleDifficulty` import. Implement `DifficultyInfo` for `ChessPuzzleDifficulty`. Add `ChessPuzzle` arm to `accept_selected_challenge()`. Add entry to `CHALLENGE_TABLE` (weight: ~10). |
| `src/core/game_state.rs` | Add `chess_puzzle_stats: ChessPuzzleStats` field (with `#[serde(default)]`). Import `ChessPuzzleStats`. Initialize in `GameState::new()`. |
| `src/core/tick.rs` | Add `ActiveMinigame::ChessPuzzle(game)` arm in Section 1 to call `process_ai_thinking` and `tick_feedback`. |
| `src/input.rs` | Add `ActiveMinigame::ChessPuzzle(game)` arm in `handle_minigame()`. Import `ChessPuzzleInput`, `process_input`, `apply_game_result`. |
| `src/ui/mod.rs` | Add `pub mod chess_puzzle_scene;`. Add `ActiveMinigame::ChessPuzzle(game)` arm in render dispatch. |
| `src/ui/chess_scene.rs` | Extract `get_piece_at()` and `piece_color()` to `chess_common.rs` (or keep duplicated). |
| `src/utils/debug_menu.rs` | Add `"Trigger Chess Puzzle Challenge"` to `DEBUG_OPTIONS`. Add `trigger_chess_puzzle_challenge()`. Update indices in `trigger_selected()`. |
| `src/lib.rs` | Add `ChessPuzzleDifficulty`, `ChessPuzzleGame`, `ChessPuzzleResult` to re-exports. |

### Achievement Integration

| File | Changes |
|------|---------|
| `src/achievements/types.rs` | Add `ChessPuzzleNovice`, `ChessPuzzleApprentice`, `ChessPuzzleJourneyman`, `ChessPuzzleMaster` to `AchievementId`. Add `("chess_puzzle", difficulty)` arms in `on_minigame_won()`. |
| `src/achievements/data.rs` | Add 4 achievement definitions: "Chess Puzzle Novice/Apprentice/Journeyman/Master" with descriptions. |

---

## 8. Puzzle Validation Strategy

### Compile-Time Safety

Puzzle definitions are `&'static` data. Setup moves and solutions use coordinate tuples that map directly to `chess_engine::Position::new(rank, file)`. Type safety prevents mixing up coordinates.

### Runtime Validation (Tests)

Every puzzle should have a test that:
1. Replays setup_moves and verifies no moves fail
2. Verifies the expected solution is correct:
   - MateInOne: Makes the (any valid checkmate) move and confirms checkmate
   - BestMove: Makes the expected move and confirms it's legal
   - MateInTwo: Makes move1, gets AI response, makes move2, confirms checkmate
3. Verifies the player is the correct color after setup

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Verify all puzzles are solvable and correctly defined.
    #[test]
    fn test_all_puzzles_valid() {
        for difficulty in ChessPuzzleDifficulty::ALL {
            let puzzles = get_puzzles(difficulty);
            assert!(!puzzles.is_empty(), "{:?} has no puzzles", difficulty);

            for (i, puzzle) in puzzles.iter().enumerate() {
                // Replay setup
                let mut board = chess_engine::Board::default();
                for (j, &(fr, ff, tr, tf)) in puzzle.setup_moves.iter().enumerate() {
                    let m = chess_engine::Move::Piece(
                        chess_engine::Position::new(fr, ff),
                        chess_engine::Position::new(tr, tf),
                    );
                    match board.play_move(m) {
                        chess_engine::GameResult::Continuing(b) => board = b,
                        other => panic!(
                            "{:?} puzzle {} '{}' setup move {} failed: {:?}",
                            difficulty, i, puzzle.title, j, other
                        ),
                    }
                }

                // Verify player color matches whose turn it is
                let expected_color = if puzzle.player_is_white {
                    chess_engine::Color::White
                } else {
                    chess_engine::Color::Black
                };
                assert_eq!(
                    board.get_turn_color(), expected_color,
                    "{:?} puzzle {} '{}' — wrong turn after setup",
                    difficulty, i, puzzle.title
                );

                // Verify solution works (puzzle-type specific)
                // ... (detailed per PuzzleSolution variant)
            }
        }
    }
}
```

This single test validates every puzzle in the game. If a puzzle definition is wrong, the test pinpoints exactly which puzzle and which setup move failed.

---

## 9. Key Design Decisions

### Why a separate ChallengeType (not reusing Chess)?

Chess puzzles and regular Chess are fundamentally different experiences:
- Chess: Full game against AI, 5-30 minute sessions
- Chess Puzzles: Quick tactical exercises, 2-5 minute sessions
- Different game state structure (puzzle tracking vs. AI state)
- Different UI (puzzle progress, hints, Correct/Wrong feedback)
- Separate discovery rolls and achievement tracking

### Why Box<ChessPuzzleGame> in ActiveMinigame?

ChessPuzzleGame contains `chess_engine::Board` (large struct) plus puzzle tracking. Boxing prevents bloating the `ActiveMinigame` enum size. This follows the same pattern as `Chess(Box<ChessGame>)`.

### Why owned puzzle_order instead of borrowed puzzles?

The `puzzle_order: Vec<usize>` stores shuffled indices rather than borrowing puzzle data directly. This avoids lifetime complications — the static puzzle data is accessed via `get_puzzles()` when needed, while the game state owns only lightweight index data.

### Why no `move_history` or algebraic notation?

Unlike regular Chess which tracks full move history for display, puzzle mode only needs to show the current puzzle position and the player's move. No move history panel is needed. This simplifies the game state.

### Why fixed AI think time (no RNG)?

The AI response in mate-in-2 puzzles is a brief visual pause to show the opponent "responding." Since it always plays the best move (no randomization), variable think time adds no value. A fixed 0.8s delay is consistent and predictable.

---

## 10. Summary

| Aspect | Detail |
|--------|--------|
| Module | `src/challenges/chess_puzzle/` (4 files) |
| UI | `src/ui/chess_puzzle_scene.rs` (1 file) |
| Modified files | 10 existing files |
| New types | 7 (ChessPuzzleDifficulty, ChessPuzzleGame, PuzzleDef, PuzzleSolution, PuzzleState, ChessPuzzleResult, ChessPuzzleStats) |
| Tick integration | Section 1 of game_tick() — AI thinking + feedback countdown |
| Input | Same keys as Chess (arrows, Enter, Esc) |
| Achievements | 4 new (one per difficulty) |
| Discovery weight | ~10 (same as Chess/Go — rare) |
| Rewards | Prestige-only: 1/2/3/5 per difficulty (matches Chess) |
| Border color | LightGreen (distinct from Chess's Cyan) |
| Icon | ♞ (Knight symbol — distinct from Chess's ♟) |
