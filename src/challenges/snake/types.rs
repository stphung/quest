//! Snake game data structures.
//!
//! A real-time action minigame where the player guides a snake to eat food and grow.

use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Difficulty levels for Snake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnakeDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(SnakeDifficulty);

impl SnakeDifficulty {
    /// Grid width (same for all difficulties — speed is the differentiator).
    pub fn grid_width(&self) -> i16 {
        26
    }

    /// Grid height (same for all difficulties — speed is the differentiator).
    pub fn grid_height(&self) -> i16 {
        26
    }

    /// Movement interval in milliseconds (lower = faster).
    pub fn move_interval_ms(&self) -> u64 {
        match self {
            Self::Novice => 200,
            Self::Apprentice => 150,
            Self::Journeyman => 120,
            Self::Master => 90,
        }
    }

    /// Number of food items to eat to win.
    pub fn target_score(&self) -> u32 {
        match self {
            Self::Novice => 10,
            Self::Apprentice => 15,
            Self::Journeyman => 20,
            Self::Master => 25,
        }
    }
}

/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeResult {
    Win,
    Loss,
}

/// Cardinal direction for snake movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns the opposite direction.
    #[allow(dead_code)]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Returns the (dx, dy) delta for this direction.
    pub fn delta(&self) -> (i16, i16) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }
}

/// A position on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

/// Main game state.
#[derive(Debug, Clone)]
pub struct SnakeGame {
    pub difficulty: SnakeDifficulty,
    pub game_result: Option<SnakeResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space to begin. Physics paused while waiting.
    pub waiting_to_start: bool,

    // Grid dimensions (cached from difficulty)
    pub grid_width: i16,
    pub grid_height: i16,

    // Snake state
    /// Snake body segments. Head is at the front (index 0).
    pub snake: VecDeque<Position>,
    /// Current movement direction.
    pub direction: Direction,
    /// Buffered next direction (prevents 180-degree reversal within a single step).
    pub next_direction: Direction,

    // Food
    pub food: Position,

    // Scoring
    pub score: u32,
    pub target_score: u32,

    // Timing
    /// Movement interval in milliseconds (cached from difficulty).
    pub move_interval_ms: u64,
    /// Sub-step time accumulator (milliseconds).
    pub accumulated_time_ms: u64,
    /// Total movement steps elapsed.
    pub tick_count: u64,
}

impl SnakeGame {
    /// Create a new game with the given difficulty.
    pub fn new<R: Rng>(difficulty: SnakeDifficulty, rng: &mut R) -> Self {
        let grid_width = difficulty.grid_width();
        let grid_height = difficulty.grid_height();

        // Snake starts in the center, 3 segments long, moving right
        let center_x = grid_width / 2;
        let center_y = grid_height / 2;
        let mut snake = VecDeque::new();
        snake.push_back(Position {
            x: center_x,
            y: center_y,
        }); // head
        snake.push_back(Position {
            x: center_x - 1,
            y: center_y,
        });
        snake.push_back(Position {
            x: center_x - 2,
            y: center_y,
        });

        let mut game = Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            grid_width,
            grid_height,

            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,

            // Temporary food position; will be overwritten by spawn_food
            food: Position { x: 0, y: 0 },

            score: 0,
            target_score: difficulty.target_score(),

            move_interval_ms: difficulty.move_interval_ms(),
            accumulated_time_ms: 0,
            tick_count: 0,
        };

        game.food = spawn_food(&game, rng);
        game
    }
}

/// Find a random empty cell for food (not occupied by the snake).
pub fn spawn_food<R: Rng>(game: &SnakeGame, rng: &mut R) -> Position {
    loop {
        let x = rng.random_range(0..game.grid_width);
        let y = rng.random_range(0..game.grid_height);
        let pos = Position { x, y };
        if !game.snake.contains(&pos) {
            return pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_defaults() {
        let mut rng = rand::rng();
        let game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        assert_eq!(game.difficulty, SnakeDifficulty::Novice);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert!(game.waiting_to_start);
        assert_eq!(game.score, 0);
        assert_eq!(game.target_score, 10);
        assert_eq!(game.snake.len(), 3);
        assert_eq!(game.direction, Direction::Right);
        assert_eq!(game.next_direction, Direction::Right);
        // All difficulties use the same 26x26 square grid
        assert_eq!(game.grid_width, 26);
        assert_eq!(game.grid_height, 26);
    }

    #[test]
    fn test_snake_initial_position() {
        let mut rng = rand::rng();
        let game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        let head = game.snake[0];
        assert_eq!(head.x, 13); // grid_width/2
        assert_eq!(head.y, 13); // grid_height/2
                                // Body extends left from head
        assert_eq!(game.snake[1].x, 12);
        assert_eq!(game.snake[2].x, 11);
    }

    #[test]
    fn test_food_not_on_snake() {
        let mut rng = rand::rng();
        let game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        assert!(!game.snake.contains(&game.food));
    }

    #[test]
    fn test_difficulty_parameters() {
        // All difficulties share the same 26x26 square grid
        for d in &SnakeDifficulty::ALL {
            assert_eq!(d.grid_width(), 26);
            assert_eq!(d.grid_height(), 26);
        }

        // Speed and target vary by difficulty
        assert_eq!(SnakeDifficulty::Novice.move_interval_ms(), 200);
        assert_eq!(SnakeDifficulty::Novice.target_score(), 10);

        assert_eq!(SnakeDifficulty::Master.move_interval_ms(), 90);
        assert_eq!(SnakeDifficulty::Master.target_score(), 25);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(SnakeDifficulty::from_index(0), SnakeDifficulty::Novice);
        assert_eq!(SnakeDifficulty::from_index(1), SnakeDifficulty::Apprentice);
        assert_eq!(SnakeDifficulty::from_index(2), SnakeDifficulty::Journeyman);
        assert_eq!(SnakeDifficulty::from_index(3), SnakeDifficulty::Master);
        assert_eq!(SnakeDifficulty::from_index(99), SnakeDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_names() {
        assert_eq!(SnakeDifficulty::Novice.name(), "Novice");
        assert_eq!(SnakeDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(SnakeDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(SnakeDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_all_difficulties() {
        assert_eq!(SnakeDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn test_spawn_food_avoids_snake() {
        let mut rng = rand::rng();
        let game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        for _ in 0..100 {
            let food = spawn_food(&game, &mut rng);
            assert!(!game.snake.contains(&food));
            assert!(food.x >= 0 && food.x < game.grid_width);
            assert!(food.y >= 0 && food.y < game.grid_height);
        }
    }
}
