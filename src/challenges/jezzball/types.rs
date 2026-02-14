//! JezzBall data structures.
//!
//! A real-time action minigame where players split the arena with growing walls
//! while avoiding bouncing hazard orbs.

use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

/// Difficulty levels for JezzBall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JezzballDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(JezzballDifficulty);

impl JezzballDifficulty {
    /// Grid width in cells.
    pub fn grid_width(&self) -> i16 {
        34
    }

    /// Grid height in cells.
    pub fn grid_height(&self) -> i16 {
        22
    }

    /// Number of hazard orbs.
    pub fn ball_count(&self) -> usize {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 2,
            Self::Journeyman => 3,
            Self::Master => 4,
        }
    }

    /// Target captured percentage required to win.
    pub fn target_percent(&self) -> u32 {
        match self {
            Self::Novice => 60,
            Self::Apprentice => 70,
            Self::Journeyman => 78,
            Self::Master => 84,
        }
    }

    /// Hazard orb speed in cells/second.
    pub fn ball_speed(&self) -> f64 {
        match self {
            Self::Novice => 6.0,
            Self::Apprentice => 6.8,
            Self::Journeyman => 7.6,
            Self::Master => 8.4,
        }
    }

    /// Wall growth interval in milliseconds.
    pub fn wall_step_ms(&self) -> u64 {
        match self {
            Self::Novice => 70,
            Self::Apprentice => 60,
            Self::Journeyman => 50,
            Self::Master => 40,
        }
    }
}

/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JezzballResult {
    Win,
    Loss,
}

/// Cursor/grid position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

/// Hazard orb with floating-point position and velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ball {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

/// Current wall axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallOrientation {
    Horizontal,
    Vertical,
}

impl WallOrientation {
    /// Toggle horizontal <-> vertical.
    pub fn toggle(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }

    /// Human-readable axis name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "Horizontal",
            Self::Vertical => "Vertical",
        }
    }
}

/// A wall currently being constructed from a pivot cell.
///
/// The wall expands in both directions along its orientation each build step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveWall {
    pub orientation: WallOrientation,
    pub pivot: Position,
    /// Cells grown in the negative direction from pivot.
    pub neg_extent: i16,
    /// Cells grown in the positive direction from pivot.
    pub pos_extent: i16,
    pub neg_done: bool,
    pub pos_done: bool,
}

/// Main JezzBall game state.
#[derive(Debug, Clone)]
pub struct JezzballGame {
    pub difficulty: JezzballDifficulty,
    pub game_result: Option<JezzballResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space/Enter to begin.
    pub waiting_to_start: bool,

    // Grid
    pub grid_width: i16,
    pub grid_height: i16,
    /// True = blocked/captured cell.
    pub blocked: Vec<Vec<bool>>,

    // Player state
    pub cursor: Position,
    pub orientation: WallOrientation,
    pub active_wall: Option<ActiveWall>,

    // Hazard state
    pub balls: Vec<Ball>,

    // Progress
    pub captured_percent: f64,
    pub target_percent: u32,

    // Timing
    pub wall_step_ms: u64,
    pub accumulated_time_ms: u64,
    pub wall_accumulated_ms: u64,
    pub tick_count: u64,
}

impl JezzballGame {
    /// Create a new game with the given difficulty.
    pub fn new<R: Rng>(difficulty: JezzballDifficulty, rng: &mut R) -> Self {
        let grid_width = difficulty.grid_width();
        let grid_height = difficulty.grid_height();

        let mut game = Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            grid_width,
            grid_height,
            blocked: vec![vec![false; grid_width as usize]; grid_height as usize],

            cursor: Position {
                x: grid_width / 2,
                y: grid_height / 2,
            },
            orientation: WallOrientation::Vertical,
            active_wall: None,

            balls: Vec::new(),

            captured_percent: 0.0,
            target_percent: difficulty.target_percent(),

            wall_step_ms: difficulty.wall_step_ms(),
            accumulated_time_ms: 0,
            wall_accumulated_ms: 0,
            tick_count: 0,
        };

        for _ in 0..difficulty.ball_count() {
            game.balls.push(spawn_ball(&game, rng));
        }

        game
    }

    /// Total playable cells.
    pub fn total_cells(&self) -> u32 {
        (self.grid_width as u32) * (self.grid_height as u32)
    }
}

/// Spawn a hazard orb away from borders and other balls.
fn spawn_ball<R: Rng>(game: &JezzballGame, rng: &mut R) -> Ball {
    let speed = game.difficulty.ball_speed();

    loop {
        let x = rng.random_range(1.5..(game.grid_width as f64 - 1.5));
        let y = rng.random_range(1.5..(game.grid_height as f64 - 1.5));

        if game
            .balls
            .iter()
            .any(|ball| ((ball.x - x).powi(2) + (ball.y - y).powi(2)).sqrt() < 2.0)
        {
            continue;
        }

        let angle = rng.random_range(0.0..std::f64::consts::TAU);
        let vx = speed * angle.cos();
        let vy = speed * angle.sin();

        // Keep trajectories from being too axis-aligned.
        if vx.abs() < speed * 0.35 || vy.abs() < speed * 0.35 {
            continue;
        }

        return Ball { x, y, vx, vy };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_defaults() {
        let mut rng = rand::rng();
        let game = JezzballGame::new(JezzballDifficulty::Novice, &mut rng);

        assert_eq!(game.difficulty, JezzballDifficulty::Novice);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert!(game.waiting_to_start);
        assert_eq!(game.grid_width, 34);
        assert_eq!(game.grid_height, 22);
        assert_eq!(game.cursor, Position { x: 17, y: 11 });
        assert_eq!(game.orientation, WallOrientation::Vertical);
        assert_eq!(game.balls.len(), 1);
        assert_eq!(game.target_percent, 60);
        assert_eq!(game.wall_step_ms, 70);
        assert_eq!(game.total_cells(), 748);
    }

    #[test]
    fn test_difficulty_parameters() {
        assert_eq!(JezzballDifficulty::Novice.ball_count(), 1);
        assert_eq!(JezzballDifficulty::Apprentice.ball_count(), 2);
        assert_eq!(JezzballDifficulty::Journeyman.ball_count(), 3);
        assert_eq!(JezzballDifficulty::Master.ball_count(), 4);

        assert_eq!(JezzballDifficulty::Novice.target_percent(), 60);
        assert_eq!(JezzballDifficulty::Master.target_percent(), 84);

        assert_eq!(JezzballDifficulty::Novice.wall_step_ms(), 70);
        assert_eq!(JezzballDifficulty::Master.wall_step_ms(), 40);

        assert!(JezzballDifficulty::Master.ball_speed() > JezzballDifficulty::Novice.ball_speed());
    }

    #[test]
    fn test_difficulty_helpers() {
        assert_eq!(
            JezzballDifficulty::from_index(0),
            JezzballDifficulty::Novice
        );
        assert_eq!(
            JezzballDifficulty::from_index(1),
            JezzballDifficulty::Apprentice
        );
        assert_eq!(
            JezzballDifficulty::from_index(2),
            JezzballDifficulty::Journeyman
        );
        assert_eq!(
            JezzballDifficulty::from_index(3),
            JezzballDifficulty::Master
        );
        assert_eq!(
            JezzballDifficulty::from_index(99),
            JezzballDifficulty::Novice
        );

        assert_eq!(JezzballDifficulty::Novice.name(), "Novice");
        assert_eq!(JezzballDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(JezzballDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(JezzballDifficulty::Master.name(), "Master");
        assert_eq!(JezzballDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_orientation_toggle() {
        assert_eq!(
            WallOrientation::Horizontal.toggle(),
            WallOrientation::Vertical
        );
        assert_eq!(
            WallOrientation::Vertical.toggle(),
            WallOrientation::Horizontal
        );
        assert_eq!(WallOrientation::Horizontal.name(), "Horizontal");
        assert_eq!(WallOrientation::Vertical.name(), "Vertical");
    }

    #[test]
    fn test_spawned_balls_within_bounds() {
        let mut rng = rand::rng();
        let game = JezzballGame::new(JezzballDifficulty::Master, &mut rng);

        for ball in &game.balls {
            assert!(ball.x >= 1.0 && ball.x <= game.grid_width as f64 - 1.0);
            assert!(ball.y >= 1.0 && ball.y <= game.grid_height as f64 - 1.0);
            assert!(ball.vx.abs() > 0.1);
            assert!(ball.vy.abs() > 0.1);
        }
    }
}
