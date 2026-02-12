//! Dino Run ("Gauntlet Run") data structures.
//!
//! A real-time action minigame where the player controls a runner dodging
//! dungeon traps in a Chrome dinosaur-style endless runner.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Difficulty levels for Dino Run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DinoRunDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(DinoRunDifficulty);

/// Game area dimensions.
pub const GAME_WIDTH: u16 = 60;
pub const GAME_HEIGHT: u16 = 18;

/// Ground row (bottom of play area, 0-indexed). Runner stands here.
pub const GROUND_ROW: u16 = 15;

/// Runner fixed horizontal column position (left edge of runner).
pub const RUNNER_COL: u16 = 6;

/// Runner dimensions.
pub const RUNNER_WIDTH: u16 = 2;
pub const RUNNER_STANDING_HEIGHT: u16 = 2; // standing: 2 rows tall (rows 14-15)
pub const RUNNER_DUCKING_HEIGHT: u16 = 1; // ducking: 1 row tall (row 15 only)

/// Flying obstacle row (at standing runner's head height).
/// Standing runner occupies rows 14-15, so flying obstacles at row 14
/// collide with the head. Ducking shrinks the runner to row 15 only,
/// avoiding the collision.
pub const FLYING_ROW: u16 = 14;

/// Run animation frame count (alternates between 2 frames).
pub const RUN_ANIM_FRAMES: u32 = 2;

impl DinoRunDifficulty {
    /// Gravity (velocity change per 16ms tick, positive = downward).
    pub fn gravity(&self) -> f64 {
        match self {
            Self::Novice => 0.012,
            Self::Apprentice => 0.014,
            Self::Journeyman => 0.016,
            Self::Master => 0.018,
        }
    }

    /// Jump impulse (negative = upward, sets velocity directly).
    pub fn jump_impulse(&self) -> f64 {
        match self {
            Self::Novice => -0.28,
            Self::Apprentice => -0.27,
            Self::Journeyman => -0.26,
            Self::Master => -0.25,
        }
    }

    /// Terminal velocity (max downward speed per 16ms tick).
    pub fn terminal_velocity(&self) -> f64 {
        match self {
            Self::Novice => 0.40,
            Self::Apprentice => 0.40,
            Self::Journeyman => 0.40,
            Self::Master => 0.40,
        }
    }

    /// Initial scroll speed in cols/tick.
    pub fn initial_speed(&self) -> f64 {
        match self {
            Self::Novice => 0.10,
            Self::Apprentice => 0.13,
            Self::Journeyman => 0.16,
            Self::Master => 0.19,
        }
    }

    /// Maximum scroll speed in cols/tick.
    pub fn max_speed(&self) -> f64 {
        match self {
            Self::Novice => 0.18,
            Self::Apprentice => 0.22,
            Self::Journeyman => 0.28,
            Self::Master => 0.35,
        }
    }

    /// Speed increase per unit of distance traveled (cols/tick increment).
    pub fn speed_increase_rate(&self) -> f64 {
        match self {
            Self::Novice => 0.0003,
            Self::Apprentice => 0.0004,
            Self::Journeyman => 0.0005,
            Self::Master => 0.0006,
        }
    }

    /// Minimum distance between obstacles (cols).
    pub fn obstacle_frequency_min(&self) -> f64 {
        match self {
            Self::Novice => 25.0,
            Self::Apprentice => 22.0,
            Self::Journeyman => 18.0,
            Self::Master => 15.0,
        }
    }

    /// Maximum distance between obstacles (cols).
    pub fn obstacle_frequency_max(&self) -> f64 {
        match self {
            Self::Novice => 40.0,
            Self::Apprentice => 35.0,
            Self::Journeyman => 30.0,
            Self::Master => 25.0,
        }
    }

    /// Number of obstacles to pass to win.
    pub fn target_score(&self) -> u32 {
        match self {
            Self::Novice => 15,
            Self::Apprentice => 25,
            Self::Journeyman => 40,
            Self::Master => 60,
        }
    }
}

/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinoRunResult {
    Win,
    Loss,
}

/// Types of obstacles the runner must avoid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleType {
    // Ground obstacles -- jump over these
    SmallRock,    // 1 row tall, 2 cols wide
    LargeRock,    // 2 rows tall, 2 cols wide
    Cactus,       // 2 rows tall, 1 col wide
    DoubleCactus, // 2 rows tall, 3 cols wide
    // Flying obstacles -- duck under these
    Bat,        // 1 row tall, 2 cols wide, at head height
    Stalactite, // 1 row tall, 3 cols wide, at head height
}

impl ObstacleType {
    /// Width in columns.
    pub fn width(&self) -> u16 {
        match self {
            Self::SmallRock => 2,
            Self::LargeRock => 2,
            Self::Cactus => 1,
            Self::DoubleCactus => 3,
            Self::Bat => 2,
            Self::Stalactite => 3,
        }
    }

    /// Height in rows.
    pub fn height(&self) -> u16 {
        match self {
            Self::SmallRock => 1,
            Self::LargeRock => 2,
            Self::Cactus => 2,
            Self::DoubleCactus => 2,
            Self::Bat => 1,
            Self::Stalactite => 1,
        }
    }

    /// True if this obstacle is airborne (duck to avoid).
    pub fn is_flying(&self) -> bool {
        matches!(self, Self::Bat | Self::Stalactite)
    }
}

/// A single obstacle in the game world.
#[derive(Debug, Clone)]
pub struct Obstacle {
    /// X position (float for smooth scrolling, cols from left edge).
    pub x: f64,
    /// The type of obstacle (determines hitbox and rendering).
    pub obstacle_type: ObstacleType,
    /// Whether the runner has cleared this obstacle (for scoring).
    pub passed: bool,
}

/// Main game state.
#[derive(Debug, Clone)]
pub struct DinoRunGame {
    pub difficulty: DinoRunDifficulty,
    pub game_result: Option<DinoRunResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space/Up to begin. Physics paused while waiting.
    pub waiting_to_start: bool,

    // -- Runner state --
    /// Vertical position of runner's feet in rows (float for smooth physics).
    /// GROUND_ROW = on ground, lower values = higher in the air.
    pub runner_y: f64,
    /// Current vertical velocity in rows/tick (negative = upward).
    pub velocity: f64,
    /// Whether the runner is currently ducking.
    pub is_ducking: bool,
    /// Duck input queued for next physics tick.
    pub duck_queued: bool,
    /// Jump input queued for next physics tick.
    pub jump_queued: bool,
    /// Animation frame for running (0 or 1, alternates every N ticks).
    pub run_anim_frame: u32,

    // -- Obstacle state --
    /// Active obstacles on screen.
    pub obstacles: Vec<Obstacle>,
    /// Distance until next obstacle spawns (in cols).
    pub next_obstacle_distance: f64,

    // -- Scoring --
    /// Obstacles successfully passed.
    pub score: u32,
    /// Obstacles needed to win.
    pub target_score: u32,
    /// Current game speed in cols/tick (increases over time).
    pub game_speed: f64,
    /// Total distance traveled (cols), used for speed ramping.
    pub distance: f64,

    // -- Timing --
    /// Sub-tick time accumulator (milliseconds).
    pub accumulated_time_ms: u64,
    /// Total physics ticks elapsed.
    pub tick_count: u64,

    // -- Cached difficulty parameters --
    pub gravity: f64,
    pub jump_impulse: f64,
    pub terminal_velocity: f64,
    pub initial_speed: f64,
    pub max_speed: f64,
    pub speed_increase_rate: f64,
    pub obstacle_frequency_min: f64,
    pub obstacle_frequency_max: f64,
}

impl DinoRunGame {
    /// Create a new game with the given difficulty.
    pub fn new(difficulty: DinoRunDifficulty) -> Self {
        Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            runner_y: GROUND_ROW as f64,
            velocity: 0.0,
            is_ducking: false,
            duck_queued: false,
            jump_queued: false,
            run_anim_frame: 0,

            obstacles: Vec::new(),
            next_obstacle_distance: GAME_WIDTH as f64 + 10.0,

            score: 0,
            target_score: difficulty.target_score(),
            game_speed: difficulty.initial_speed(),
            distance: 0.0,

            accumulated_time_ms: 0,
            tick_count: 0,

            gravity: difficulty.gravity(),
            jump_impulse: difficulty.jump_impulse(),
            terminal_velocity: difficulty.terminal_velocity(),
            initial_speed: difficulty.initial_speed(),
            max_speed: difficulty.max_speed(),
            speed_increase_rate: difficulty.speed_increase_rate(),
            obstacle_frequency_min: difficulty.obstacle_frequency_min(),
            obstacle_frequency_max: difficulty.obstacle_frequency_max(),
        }
    }

    /// Returns true if the runner is on the ground.
    pub fn is_on_ground(&self) -> bool {
        self.runner_y >= GROUND_ROW as f64
    }

    /// Spawn a new obstacle with a random type.
    pub fn spawn_obstacle<R: Rng>(&mut self, rng: &mut R) {
        let obstacle_type = if self.score > 5 && rng.gen::<f64>() < 0.25 {
            // 25% chance of flying obstacle after score > 5
            if rng.gen::<bool>() {
                ObstacleType::Bat
            } else {
                ObstacleType::Stalactite
            }
        } else {
            // Ground obstacles
            match rng.gen_range(0..4) {
                0 => ObstacleType::SmallRock,
                1 => ObstacleType::LargeRock,
                2 => ObstacleType::Cactus,
                _ => ObstacleType::DoubleCactus,
            }
        };

        let x = GAME_WIDTH as f64 + obstacle_type.width() as f64;
        self.obstacles.push(Obstacle {
            x,
            obstacle_type,
            passed: false,
        });

        // Randomize next obstacle distance within frequency range
        self.next_obstacle_distance =
            rng.gen_range(self.obstacle_frequency_min..=self.obstacle_frequency_max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_defaults() {
        let game = DinoRunGame::new(DinoRunDifficulty::Novice);
        assert_eq!(game.difficulty, DinoRunDifficulty::Novice);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert!(game.waiting_to_start);
        assert_eq!(game.score, 0);
        assert_eq!(game.target_score, 15);
        assert!(game.obstacles.is_empty());
        assert!(!game.jump_queued);
        assert!(!game.duck_queued);
        assert!(!game.is_ducking);
        assert!(game.is_on_ground());
    }

    #[test]
    fn test_difficulty_parameters() {
        let d = DinoRunDifficulty::Novice;
        assert!((d.gravity() - 0.012).abs() < f64::EPSILON);
        assert!((d.jump_impulse() - (-0.28)).abs() < f64::EPSILON);
        assert!((d.terminal_velocity() - 0.40).abs() < f64::EPSILON);
        assert!((d.initial_speed() - 0.10).abs() < f64::EPSILON);
        assert!((d.max_speed() - 0.18).abs() < f64::EPSILON);
        assert_eq!(d.target_score(), 15);

        let d = DinoRunDifficulty::Master;
        assert!((d.gravity() - 0.018).abs() < f64::EPSILON);
        assert!((d.jump_impulse() - (-0.25)).abs() < f64::EPSILON);
        assert!((d.initial_speed() - 0.19).abs() < f64::EPSILON);
        assert!((d.max_speed() - 0.35).abs() < f64::EPSILON);
        assert_eq!(d.target_score(), 60);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(DinoRunDifficulty::from_index(0), DinoRunDifficulty::Novice);
        assert_eq!(
            DinoRunDifficulty::from_index(1),
            DinoRunDifficulty::Apprentice
        );
        assert_eq!(
            DinoRunDifficulty::from_index(2),
            DinoRunDifficulty::Journeyman
        );
        assert_eq!(DinoRunDifficulty::from_index(3), DinoRunDifficulty::Master);
        assert_eq!(DinoRunDifficulty::from_index(99), DinoRunDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_names() {
        assert_eq!(DinoRunDifficulty::Novice.name(), "Novice");
        assert_eq!(DinoRunDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(DinoRunDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(DinoRunDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_all_difficulties() {
        assert_eq!(DinoRunDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_obstacle_type_dimensions() {
        assert_eq!(ObstacleType::SmallRock.width(), 2);
        assert_eq!(ObstacleType::SmallRock.height(), 1);
        assert!(!ObstacleType::SmallRock.is_flying());

        assert_eq!(ObstacleType::LargeRock.width(), 2);
        assert_eq!(ObstacleType::LargeRock.height(), 2);
        assert!(!ObstacleType::LargeRock.is_flying());

        assert_eq!(ObstacleType::Cactus.width(), 1);
        assert_eq!(ObstacleType::Cactus.height(), 2);
        assert!(!ObstacleType::Cactus.is_flying());

        assert_eq!(ObstacleType::DoubleCactus.width(), 3);
        assert_eq!(ObstacleType::DoubleCactus.height(), 2);
        assert!(!ObstacleType::DoubleCactus.is_flying());

        assert_eq!(ObstacleType::Bat.width(), 2);
        assert_eq!(ObstacleType::Bat.height(), 1);
        assert!(ObstacleType::Bat.is_flying());

        assert_eq!(ObstacleType::Stalactite.width(), 3);
        assert_eq!(ObstacleType::Stalactite.height(), 1);
        assert!(ObstacleType::Stalactite.is_flying());
    }

    #[test]
    fn test_spawn_obstacle() {
        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        let mut rng = rand::thread_rng();

        game.spawn_obstacle(&mut rng);

        assert_eq!(game.obstacles.len(), 1);
        let obs = &game.obstacles[0];
        assert!(!obs.passed);
        // Obstacle should be spawned off the right edge
        assert!(obs.x >= GAME_WIDTH as f64);
        // next_obstacle_distance should be reset within frequency range
        assert!(game.next_obstacle_distance >= game.obstacle_frequency_min);
        assert!(game.next_obstacle_distance <= game.obstacle_frequency_max);
    }

    #[test]
    fn test_is_on_ground() {
        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        assert!(game.is_on_ground());

        game.runner_y = 10.0;
        assert!(!game.is_on_ground());

        game.runner_y = GROUND_ROW as f64;
        assert!(game.is_on_ground());
    }
}
