//! Runic Shift data structures.
//!
//! A real-time action-puzzle challenge inspired by panel-matching games.

use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

/// Grid width in cells.
pub const GRID_COLS: usize = 6;
/// Grid height in cells.
pub const GRID_ROWS: usize = 12;
/// Number of attempts per challenge.
pub const MAX_LIVES: u32 = 3;
/// Swap animation duration.
pub const SWAP_ANIM_MS: u64 = 120;
/// Minimum falling animation duration.
pub const FALL_ANIM_MIN_MS: u64 = 80;
/// Extra falling animation duration per dropped row.
pub const FALL_ANIM_PER_ROW_MS: u64 = 40;
/// Cap falling animation duration.
pub const FALL_ANIM_MAX_MS: u64 = 260;

/// Rune block colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuneColor {
    Fire,
    Water,
    Earth,
    Light,
    Frost,
    Arcane,
}

impl RuneColor {
    /// Display glyph for this rune.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Fire => "🔥",
            Self::Water => "💧",
            Self::Earth => "🍀",
            Self::Light => "⭐",
            Self::Frost => "❄️",
            Self::Arcane => "🔮",
        }
    }

    /// Palette order used by random generation.
    pub const ALL: [RuneColor; 6] = [
        RuneColor::Fire,
        RuneColor::Water,
        RuneColor::Earth,
        RuneColor::Light,
        RuneColor::Frost,
        RuneColor::Arcane,
    ];
}

/// Block lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    Normal,
    Clearing,
}

/// A single rune block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub color: RuneColor,
    pub state: BlockState,
}

impl Block {
    pub fn normal(color: RuneColor) -> Self {
        Self {
            color,
            state: BlockState::Normal,
        }
    }
}

/// A single block movement for falling animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallingBlockAnim {
    pub color: RuneColor,
    pub col: usize,
    pub from_row: usize,
    pub to_row: usize,
}

/// Swap animation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapAnimation {
    pub row: usize,
    pub left_col: usize,
    pub right_col: usize,
    pub left_block: Option<Block>,
    pub right_block: Option<Block>,
    pub duration_ms: u64,
    pub remaining_ms: u64,
}

/// Falling animation state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallAnimation {
    pub blocks: Vec<FallingBlockAnim>,
    pub duration_ms: u64,
    pub remaining_ms: u64,
}

/// Active animation for the board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardAnimation {
    Swap(SwapAnimation),
    Fall(FallAnimation),
}

/// Post-animation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingSettle {
    AfterSwap,
    AfterFall { chain_continuation: bool },
}

/// Difficulty levels for Runic Shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunicShiftDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(RunicShiftDifficulty);

impl RunicShiftDifficulty {
    /// Number of active rune colors.
    pub fn num_colors(&self) -> usize {
        match self {
            Self::Novice => 4,
            Self::Apprentice | Self::Journeyman => 5,
            Self::Master => 6,
        }
    }

    /// Automatic rise interval in milliseconds.
    pub fn rise_interval_ms(&self) -> u64 {
        match self {
            Self::Novice => 4000,
            Self::Apprentice => 3000,
            Self::Journeyman => 2200,
            Self::Master => 1500,
        }
    }

    /// Target clear count for victory.
    pub fn target_clears(&self) -> u32 {
        match self {
            Self::Novice => 30,
            Self::Apprentice => 50,
            Self::Journeyman => 75,
            Self::Master => 100,
        }
    }

    /// Starting filled rows.
    pub fn starting_rows(&self) -> usize {
        match self {
            Self::Novice => 5,
            Self::Apprentice => 6,
            Self::Journeyman => 7,
            Self::Master => 8,
        }
    }

    /// Clearing animation duration.
    pub fn clear_duration_ms(&self) -> u64 {
        match self {
            Self::Novice => 400,
            Self::Apprentice => 350,
            Self::Journeyman => 300,
            Self::Master => 250,
        }
    }
}

/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicShiftResult {
    Win,
    Loss,
}

/// Main Runic Shift game state.
#[derive(Debug, Clone)]
pub struct RunicShiftGame {
    pub difficulty: RunicShiftDifficulty,
    pub game_result: Option<RunicShiftResult>,
    pub forfeit_pending: bool,
    /// True until Space is pressed.
    pub waiting_to_start: bool,

    /// Grid storage, row 0 = top.
    pub grid: [[Option<Block>; GRID_COLS]; GRID_ROWS],
    /// Bottom-row preview.
    pub next_row: [RuneColor; GRID_COLS],

    /// Cursor column (0..=4, spans two columns).
    pub cursor_col: usize,
    /// Cursor row (0..=11).
    pub cursor_row: usize,

    /// Auto-rise cadence and accumulation.
    pub rise_interval_ms: u64,
    pub rise_accumulated_ms: u64,

    /// Clearing timer (remaining ms). When >0, marked cells are flashing.
    pub clear_timer_ms: u64,
    /// Flash cadence (ms) for clearing animation.
    pub clear_flash_ms: u64,

    /// Total clears (includes chain bonuses).
    pub clears: u32,
    pub target_clears: u32,
    /// Current chain depth (1 during first clear, 2+ for true chain steps).
    pub current_chain: u32,
    /// Highest chain depth seen this run.
    pub best_chain: u32,

    pub lives: u32,

    /// Session timing / diagnostics.
    pub accumulated_time_ms: u64,
    pub tick_count: u64,

    /// Cached palette size from difficulty.
    pub num_colors: usize,

    /// Internal chain depth tracker across a clear cascade.
    pub chain_depth: u32,

    /// Active board animation.
    pub board_animation: Option<BoardAnimation>,
    /// Settle/match resolution to run when current animation completes.
    pub pending_settle: Option<PendingSettle>,
}

impl RunicShiftGame {
    /// Create a new Runic Shift game.
    pub fn new(difficulty: RunicShiftDifficulty) -> Self {
        let mut game = Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            grid: [[None; GRID_COLS]; GRID_ROWS],
            next_row: [RuneColor::Fire; GRID_COLS],

            cursor_col: 2,
            cursor_row: GRID_ROWS - 2,

            rise_interval_ms: difficulty.rise_interval_ms(),
            rise_accumulated_ms: 0,

            clear_timer_ms: 0,
            clear_flash_ms: 100,

            clears: 0,
            target_clears: difficulty.target_clears(),
            current_chain: 0,
            best_chain: 0,

            lives: MAX_LIVES,

            accumulated_time_ms: 0,
            tick_count: 0,

            num_colors: difficulty.num_colors(),
            chain_depth: 0,
            board_animation: None,
            pending_settle: None,
        };

        let mut rng = rand::rng();
        game.fill_starting_rows(&mut rng);
        game.next_row = game.roll_next_row(&mut rng);
        game
    }

    /// Reset for another life while preserving progress.
    pub fn reset_for_retry(&mut self) {
        self.grid = [[None; GRID_COLS]; GRID_ROWS];
        self.cursor_col = 2;
        self.cursor_row = GRID_ROWS - 2;

        self.forfeit_pending = false;
        self.waiting_to_start = true;

        self.rise_accumulated_ms = 0;
        self.clear_timer_ms = 0;
        self.current_chain = 0;
        self.chain_depth = 0;

        self.accumulated_time_ms = 0;
        self.tick_count = 0;
        self.board_animation = None;
        self.pending_settle = None;

        let mut rng = rand::rng();
        self.fill_starting_rows(&mut rng);
        self.next_row = self.roll_next_row(&mut rng);
    }

    /// True when row 1 contains any block.
    pub fn has_danger(&self) -> bool {
        self.grid[1].iter().any(Option::is_some)
    }

    /// True when row 0 contains any block.
    pub fn has_top_occupancy(&self) -> bool {
        self.grid[0].iter().any(Option::is_some)
    }

    /// Generate a preview row using active colors.
    pub fn roll_next_row<R: Rng>(&self, rng: &mut R) -> [RuneColor; GRID_COLS] {
        let mut row = [RuneColor::Fire; GRID_COLS];

        for col in 0..GRID_COLS {
            loop {
                let color = self.random_color(rng);
                if col >= 2 && row[col - 1] == color && row[col - 2] == color {
                    continue;
                }
                row[col] = color;
                break;
            }
        }

        row
    }

    /// Fill the starting board height with randomized blocks.
    pub fn fill_starting_rows<R: Rng>(&mut self, rng: &mut R) {
        self.grid = [[None; GRID_COLS]; GRID_ROWS];

        let start_row = GRID_ROWS.saturating_sub(self.difficulty.starting_rows());
        for row in start_row..GRID_ROWS {
            for col in 0..GRID_COLS {
                loop {
                    let color = self.random_color(rng);

                    // Avoid obvious triples at generation time.
                    if col >= 2 {
                        let left1 = self.grid[row][col - 1].map(|b| b.color);
                        let left2 = self.grid[row][col - 2].map(|b| b.color);
                        if left1 == Some(color) && left2 == Some(color) {
                            continue;
                        }
                    }
                    if row >= 2 {
                        let up1 = self.grid[row - 1][col].map(|b| b.color);
                        let up2 = self.grid[row - 2][col].map(|b| b.color);
                        if up1 == Some(color) && up2 == Some(color) {
                            continue;
                        }
                    }

                    self.grid[row][col] = Some(Block::normal(color));
                    break;
                }
            }
        }
    }

    /// Pick a random color from the active palette.
    pub fn random_color<R: Rng>(&self, rng: &mut R) -> RuneColor {
        RuneColor::ALL[rng.random_range(0..self.num_colors)]
    }

    /// True when any board animation is in progress.
    pub fn has_active_animation(&self) -> bool {
        self.board_animation.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_parameters() {
        assert_eq!(RunicShiftDifficulty::Novice.num_colors(), 4);
        assert_eq!(RunicShiftDifficulty::Novice.rise_interval_ms(), 4000);
        assert_eq!(RunicShiftDifficulty::Novice.target_clears(), 30);
        assert_eq!(RunicShiftDifficulty::Novice.starting_rows(), 5);
        assert_eq!(RunicShiftDifficulty::Novice.clear_duration_ms(), 400);

        assert_eq!(RunicShiftDifficulty::Master.num_colors(), 6);
        assert_eq!(RunicShiftDifficulty::Master.rise_interval_ms(), 1500);
        assert_eq!(RunicShiftDifficulty::Master.target_clears(), 100);
        assert_eq!(RunicShiftDifficulty::Master.starting_rows(), 8);
        assert_eq!(RunicShiftDifficulty::Master.clear_duration_ms(), 250);
    }

    #[test]
    fn test_new_game_defaults() {
        let game = RunicShiftGame::new(RunicShiftDifficulty::Apprentice);
        assert_eq!(game.difficulty, RunicShiftDifficulty::Apprentice);
        assert!(game.game_result.is_none());
        assert!(game.waiting_to_start);
        assert_eq!(game.lives, MAX_LIVES);
        assert_eq!(game.target_clears, 50);
        assert_eq!(game.num_colors, 5);
        assert_eq!(game.cursor_col, 2);
        assert_eq!(game.cursor_row, GRID_ROWS - 2);
    }

    #[test]
    fn test_has_danger_and_top_occupancy() {
        let mut game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        game.grid = [[None; GRID_COLS]; GRID_ROWS];
        assert!(!game.has_danger());
        assert!(!game.has_top_occupancy());

        game.grid[1][3] = Some(Block::normal(RuneColor::Fire));
        assert!(game.has_danger());
        assert!(!game.has_top_occupancy());

        game.grid[0][0] = Some(Block::normal(RuneColor::Water));
        assert!(game.has_top_occupancy());
    }

    #[test]
    fn test_reset_for_retry_preserves_progress() {
        let mut game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        game.clears = 12;
        game.best_chain = 4;
        game.lives = 2;
        game.waiting_to_start = false;
        game.clear_timer_ms = 200;
        game.current_chain = 3;

        game.reset_for_retry();

        assert_eq!(game.clears, 12);
        assert_eq!(game.best_chain, 4);
        assert_eq!(game.lives, 2);
        assert!(game.waiting_to_start);
        assert_eq!(game.clear_timer_ms, 0);
        assert_eq!(game.current_chain, 0);
    }

    #[test]
    fn test_roll_next_row_respects_palette() {
        let game = RunicShiftGame::new(RunicShiftDifficulty::Novice);
        let mut rng = rand::rng();
        let row = game.roll_next_row(&mut rng);
        assert!(row.iter().all(|color| {
            matches!(
                color,
                RuneColor::Fire | RuneColor::Water | RuneColor::Earth | RuneColor::Light
            )
        }));
    }
}
