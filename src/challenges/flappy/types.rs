//! Data structures for the Flappy Bird challenge minigame.

/// Difficulty levels for Flappy Bird.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(FlappyDifficulty);

impl FlappyDifficulty {
    /// Gap size in rows between top and bottom pipes.
    pub fn gap_size(&self) -> u16 {
        match self {
            FlappyDifficulty::Novice => 8,
            FlappyDifficulty::Apprentice => 7,
            FlappyDifficulty::Journeyman => 6,
            FlappyDifficulty::Master => 5,
        }
    }

    /// Number of ticks between pipe scroll steps.
    /// Lower = faster pipes.
    pub fn pipe_speed_ticks(&self) -> u32 {
        match self {
            FlappyDifficulty::Novice => 3,
            FlappyDifficulty::Apprentice => 2,
            FlappyDifficulty::Journeyman => 2,
            FlappyDifficulty::Master => 1,
        }
    }

    /// Number of pipes to pass to win.
    pub fn target_score(&self) -> u32 {
        match self {
            FlappyDifficulty::Novice => 10,
            FlappyDifficulty::Apprentice => 15,
            FlappyDifficulty::Journeyman => 20,
            FlappyDifficulty::Master => 30,
        }
    }

    /// Horizontal spacing between pipes (in columns).
    pub fn pipe_spacing(&self) -> u16 {
        match self {
            FlappyDifficulty::Novice => 20,
            FlappyDifficulty::Apprentice => 18,
            FlappyDifficulty::Journeyman => 16,
            FlappyDifficulty::Master => 14,
        }
    }
}

/// Result of a flappy bird game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyResult {
    Win,
    Loss,
    Forfeit,
}

/// A single pipe obstacle with a gap.
#[derive(Debug, Clone)]
pub struct Pipe {
    /// X position (column) of the pipe. Decreases each scroll step.
    pub x: i32,
    /// Top of the gap (row index, 0-based from top).
    pub gap_top: u16,
    /// Whether the player has already passed this pipe (for scoring).
    pub scored: bool,
}

/// The Flappy Bird game state.
#[derive(Debug, Clone)]
pub struct FlappyGame {
    /// Difficulty level.
    pub difficulty: FlappyDifficulty,
    /// Bird Y position (fractional, in rows from top). 0.0 = ceiling.
    pub bird_y: f64,
    /// Bird Y velocity (positive = downward).
    pub bird_vel: f64,
    /// Bird X position (fixed column in the play area).
    pub bird_x: u16,
    /// Active pipes scrolling across the screen.
    pub pipes: Vec<Pipe>,
    /// Current score (pipes passed).
    pub score: u32,
    /// Play area height in rows.
    pub area_height: u16,
    /// Play area width in columns.
    pub area_width: u16,
    /// Tick counter for pipe scrolling timing.
    pub tick_count: u32,
    /// Next pipe spawn counter (columns until next pipe).
    pub next_pipe_in: u16,
    /// Game result (None while playing).
    pub game_result: Option<FlappyResult>,
    /// Whether the game has started (first flap).
    pub started: bool,
    /// Forfeit pending (first Esc pressed).
    pub forfeit_pending: bool,
}

impl FlappyGame {
    /// Default play area dimensions.
    pub const DEFAULT_WIDTH: u16 = 50;
    pub const DEFAULT_HEIGHT: u16 = 20;

    /// Gravity applied per tick (downward acceleration).
    pub const GRAVITY: f64 = 0.35;
    /// Velocity applied on flap (upward impulse).
    pub const FLAP_VELOCITY: f64 = -2.5;
    /// Maximum downward velocity (terminal velocity).
    pub const MAX_VELOCITY: f64 = 3.0;

    /// Create a new Flappy Bird game.
    pub fn new(difficulty: FlappyDifficulty) -> Self {
        let area_height = Self::DEFAULT_HEIGHT;
        let area_width = Self::DEFAULT_WIDTH;
        let bird_y = area_height as f64 / 2.0;

        Self {
            difficulty,
            bird_y,
            bird_vel: 0.0,
            bird_x: 8,
            pipes: Vec::new(),
            score: 0,
            area_height,
            area_width,
            tick_count: 0,
            next_pipe_in: 15, // First pipe appears after some space
            game_result: None,
            started: false,
            forfeit_pending: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_gap_sizes_decrease() {
        assert!(FlappyDifficulty::Novice.gap_size() > FlappyDifficulty::Master.gap_size());
    }

    #[test]
    fn test_difficulty_target_scores_increase() {
        assert!(FlappyDifficulty::Novice.target_score() < FlappyDifficulty::Master.target_score());
    }

    #[test]
    fn test_new_game_initial_state() {
        let game = FlappyGame::new(FlappyDifficulty::Novice);
        assert_eq!(game.score, 0);
        assert!(!game.started);
        assert!(game.game_result.is_none());
        assert!(game.pipes.is_empty());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_difficulty_enum_all() {
        assert_eq!(FlappyDifficulty::ALL.len(), 4);
        assert_eq!(FlappyDifficulty::from_index(0), FlappyDifficulty::Novice);
        assert_eq!(FlappyDifficulty::from_index(3), FlappyDifficulty::Master);
    }

    #[test]
    fn test_bird_starts_centered() {
        let game = FlappyGame::new(FlappyDifficulty::Novice);
        let center = FlappyGame::DEFAULT_HEIGHT as f64 / 2.0;
        assert!((game.bird_y - center).abs() < 0.01);
    }
}
