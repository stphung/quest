//! Flappy Bird ("Skyward Gauntlet") data structures.
//!
//! A real-time action minigame where the player guides a bird through pipe gaps.

use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

/// Difficulty levels for Flappy Bird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlappyBirdDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(FlappyBirdDifficulty);

/// Game area dimensions.
pub const GAME_WIDTH: u16 = 50;
pub const GAME_HEIGHT: u16 = 18;

/// Bird fixed horizontal column position.
pub const BIRD_COL: u16 = 8;

/// Pipe width in characters.
pub const PIPE_WIDTH: u16 = 3;

/// Flap animation duration in physics ticks (6 ticks × 16ms ≈ 96ms).
pub const FLAP_ANIM_TICKS: u32 = 6;

impl FlappyBirdDifficulty {
    /// Gravity (velocity change per 16ms tick).
    ///
    /// Tuned for our 18-row terminal game world. Gravity-to-jump ratio ≈ 18×
    /// following Flappy Bird reference, but scaled down for a gentler feel
    /// in the smaller playing field. ~0.5s rise to apex per flap.
    pub fn gravity(&self) -> f64 {
        match self {
            Self::Novice => 0.005,
            Self::Apprentice => 0.006,
            Self::Journeyman => 0.007,
            Self::Master => 0.008,
        }
    }

    /// Flap impulse — velocity override (negative = upward) per 16ms tick.
    ///
    /// Sets velocity directly (not additive), matching original Flappy Bird
    /// behavior. Ratio of ~18× to gravity for authentic feel.
    pub fn flap_impulse(&self) -> f64 {
        match self {
            Self::Novice => -0.18,
            Self::Apprentice => -0.19,
            Self::Journeyman => -0.20,
            Self::Master => -0.22,
        }
    }

    /// Terminal velocity (max downward velocity per 16ms tick).
    ///
    /// Generous cap — only prevents extreme speeds after extended free-fall.
    pub fn terminal_velocity(&self) -> f64 {
        match self {
            Self::Novice => 0.35,
            Self::Apprentice => 0.35,
            Self::Journeyman => 0.35,
            Self::Master => 0.35,
        }
    }

    /// Pipe gap size in rows.
    pub fn pipe_gap(&self) -> u16 {
        match self {
            Self::Novice => 7,
            Self::Apprentice => 6,
            Self::Journeyman => 5,
            Self::Master => 4,
        }
    }

    /// Pipe speed in cols/tick (16ms). Preserves same cols/sec as original tuning.
    pub fn pipe_speed(&self) -> f64 {
        match self {
            Self::Novice => 0.073,
            Self::Apprentice => 0.097,
            Self::Journeyman => 0.121,
            Self::Master => 0.145,
        }
    }

    /// Horizontal spacing between consecutive pipes in cols.
    pub fn pipe_spacing(&self) -> f64 {
        match self {
            Self::Novice => 20.0,
            Self::Apprentice => 17.0,
            Self::Journeyman => 15.0,
            Self::Master => 13.0,
        }
    }

    /// Number of pipes to pass to win.
    pub fn target_score(&self) -> u32 {
        match self {
            Self::Novice => 10,
            Self::Apprentice => 15,
            Self::Journeyman => 20,
            Self::Master => 30,
        }
    }
}

/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyBirdResult {
    Win,
    Loss,
}

/// A single pipe obstacle (top + bottom pair with a gap).
#[derive(Debug, Clone)]
pub struct Pipe {
    /// X position (float for smooth scrolling).
    pub x: f64,
    /// Row index of the gap center.
    pub gap_center: u16,
    /// Whether the bird has passed this pipe (for scoring).
    pub passed: bool,
}

/// Main game state.
#[derive(Debug, Clone)]
pub struct FlappyBirdGame {
    pub difficulty: FlappyBirdDifficulty,
    pub game_result: Option<FlappyBirdResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space to begin. Physics paused while waiting.
    pub waiting_to_start: bool,

    // Bird state
    /// Vertical position in rows (float for smooth physics). Row 0 = ceiling, row 17 = ground.
    pub bird_y: f64,
    /// Current vertical velocity in rows/tick (positive = downward).
    pub bird_velocity: f64,
    /// Ticks remaining to show flap animation.
    pub flap_timer: u32,

    // Pipe state
    /// Active pipes on screen.
    pub pipes: Vec<Pipe>,
    /// X position where the next pipe will spawn.
    pub next_pipe_x: f64,

    // Scoring
    /// Pipes successfully passed.
    pub score: u32,
    /// Pipes needed to win.
    pub target_score: u32,

    // Timing
    /// Sub-tick time accumulator (milliseconds).
    pub accumulated_time_ms: u64,
    /// Total physics ticks elapsed.
    pub tick_count: u64,

    // Input buffer
    /// Flap input waiting to be consumed next physics tick.
    pub flap_queued: bool,

    // Cached difficulty parameters
    pub gravity: f64,
    pub flap_impulse: f64,
    pub terminal_velocity: f64,
    pub pipe_gap: u16,
    pub pipe_speed: f64,
    pub pipe_spacing: f64,
}

impl FlappyBirdGame {
    /// Create a new game with the given difficulty.
    pub fn new(difficulty: FlappyBirdDifficulty) -> Self {
        let pipe_spacing = difficulty.pipe_spacing();
        Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            // Bird starts roughly in the middle of the playable area
            bird_y: 8.0,
            bird_velocity: 0.0,
            flap_timer: 0,

            pipes: Vec::new(),
            // First pipe spawns off the right edge
            next_pipe_x: GAME_WIDTH as f64 + 5.0,

            score: 0,
            target_score: difficulty.target_score(),

            accumulated_time_ms: 0,
            tick_count: 0,

            flap_queued: false,

            gravity: difficulty.gravity(),
            flap_impulse: difficulty.flap_impulse(),
            terminal_velocity: difficulty.terminal_velocity(),
            pipe_gap: difficulty.pipe_gap(),
            pipe_speed: difficulty.pipe_speed(),
            pipe_spacing,
        }
    }

    /// Spawn a new pipe with a random gap position.
    pub fn spawn_pipe<R: Rng>(&mut self, rng: &mut R) {
        let half_gap = self.pipe_gap / 2;
        // Gap center constrained between rows 3 and 14 so gap never clips ceiling/ground.
        let min_center = 3 + half_gap;
        let max_center = 14u16.saturating_sub(half_gap).max(min_center);
        let gap_center = rng.random_range(min_center..=max_center);

        self.pipes.push(Pipe {
            x: self.next_pipe_x,
            gap_center,
            passed: false,
        });
        self.next_pipe_x += self.pipe_spacing;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_defaults() {
        let game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        assert_eq!(game.difficulty, FlappyBirdDifficulty::Novice);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert_eq!(game.score, 0);
        assert_eq!(game.target_score, 10);
        assert!(game.pipes.is_empty());
        assert!(!game.flap_queued);
    }

    #[test]
    fn test_difficulty_parameters() {
        // Novice
        let d = FlappyBirdDifficulty::Novice;
        assert!((d.gravity() - 0.005).abs() < f64::EPSILON);
        assert!((d.flap_impulse() - (-0.18)).abs() < f64::EPSILON);
        assert_eq!(d.pipe_gap(), 7);
        assert_eq!(d.target_score(), 10);

        // Master
        let d = FlappyBirdDifficulty::Master;
        assert!((d.gravity() - 0.008).abs() < f64::EPSILON);
        assert!((d.flap_impulse() - (-0.22)).abs() < f64::EPSILON);
        assert_eq!(d.pipe_gap(), 4);
        assert_eq!(d.target_score(), 30);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(
            FlappyBirdDifficulty::from_index(0),
            FlappyBirdDifficulty::Novice
        );
        assert_eq!(
            FlappyBirdDifficulty::from_index(1),
            FlappyBirdDifficulty::Apprentice
        );
        assert_eq!(
            FlappyBirdDifficulty::from_index(2),
            FlappyBirdDifficulty::Journeyman
        );
        assert_eq!(
            FlappyBirdDifficulty::from_index(3),
            FlappyBirdDifficulty::Master
        );
        assert_eq!(
            FlappyBirdDifficulty::from_index(99),
            FlappyBirdDifficulty::Novice
        );
    }

    #[test]
    fn test_difficulty_names() {
        assert_eq!(FlappyBirdDifficulty::Novice.name(), "Novice");
        assert_eq!(FlappyBirdDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(FlappyBirdDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(FlappyBirdDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_all_difficulties() {
        assert_eq!(FlappyBirdDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_spawn_pipe() {
        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        let mut rng = rand::rng();
        let initial_next = game.next_pipe_x;

        game.spawn_pipe(&mut rng);

        assert_eq!(game.pipes.len(), 1);
        let pipe = &game.pipes[0];
        assert!((pipe.x - initial_next).abs() < f64::EPSILON);
        assert!(!pipe.passed);
        // Gap center should be within valid range
        let half_gap = game.pipe_gap / 2;
        assert!(pipe.gap_center >= 3 + half_gap);
        assert!(pipe.gap_center <= 14 - half_gap);
        // next_pipe_x should have advanced
        assert!((game.next_pipe_x - (initial_next + game.pipe_spacing)).abs() < f64::EPSILON);
    }
}
