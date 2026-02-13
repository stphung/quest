//! Snake game logic: movement, input processing, collision detection.

use super::types::*;
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;

/// UI-agnostic input actions for Snake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeInput {
    Up,
    Down,
    Left,
    Right,
    Select,  // Space — starts game
    Forfeit, // Esc
    Other,   // Any other key (cancels forfeit_pending)
}

/// Start a new snake game at the given difficulty.
pub fn start_snake_game(difficulty: SnakeDifficulty) -> ActiveMinigame {
    let mut rng = rand::thread_rng();
    ActiveMinigame::Snake(SnakeGame::new(difficulty, &mut rng))
}

/// Process player input.
pub fn process_input(game: &mut SnakeGame, input: SnakeInput) {
    if game.game_result.is_some() {
        return; // Game over — any key dismisses (handled by input.rs)
    }

    // Waiting screen: Space starts the game
    if game.waiting_to_start {
        if matches!(input, SnakeInput::Select) {
            game.waiting_to_start = false;
        }
        return;
    }

    match input {
        SnakeInput::Up => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.direction != Direction::Down {
                game.next_direction = Direction::Up;
            }
        }
        SnakeInput::Down => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.direction != Direction::Up {
                game.next_direction = Direction::Down;
            }
        }
        SnakeInput::Left => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.direction != Direction::Right {
                game.next_direction = Direction::Left;
            }
        }
        SnakeInput::Right => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.direction != Direction::Left {
                game.next_direction = Direction::Right;
            }
        }
        SnakeInput::Select => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            }
        }
        SnakeInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(SnakeResult::Loss); // Confirm forfeit
            } else {
                game.forfeit_pending = true;
            }
        }
        SnakeInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            }
        }
    }
}

/// Advance Snake game. Called from the main game loop.
///
/// `dt_ms` is milliseconds since last call. Internally steps movement in
/// `move_interval_ms` increments. Returns true if the game state changed.
pub fn tick_snake(game: &mut SnakeGame, dt_ms: u64) -> bool {
    if game.game_result.is_some() {
        return false;
    }

    // Pause while waiting to start or during forfeit
    if game.waiting_to_start || game.forfeit_pending {
        return false;
    }

    // Clamp dt to 500ms max to prevent physics explosion after pause/lag
    let dt_ms = dt_ms.min(500);

    game.accumulated_time_ms += dt_ms;
    let mut changed = false;

    // Step movement at fixed intervals
    while game.accumulated_time_ms >= game.move_interval_ms {
        game.accumulated_time_ms -= game.move_interval_ms;
        step_snake(game);
        changed = true;

        if game.game_result.is_some() {
            break;
        }
    }

    changed
}

/// Single movement step.
fn step_snake(game: &mut SnakeGame) {
    game.tick_count += 1;

    // Apply buffered direction
    game.direction = game.next_direction;

    // Calculate new head position
    let (dx, dy) = game.direction.delta();
    let head = game.snake[0];
    let new_head = Position {
        x: head.x + dx,
        y: head.y + dy,
    };

    // Wall collision
    if new_head.x < 0
        || new_head.x >= game.grid_width
        || new_head.y < 0
        || new_head.y >= game.grid_height
    {
        game.game_result = Some(SnakeResult::Loss);
        return;
    }

    // Self collision: check range depends on whether we're eating food.
    // If eating, the tail stays so check full body.
    // If not eating, the tail will be removed, so exclude it (allows tail-chasing).
    let eating = new_head == game.food;
    let collision_range = if eating {
        game.snake.len()
    } else {
        game.snake.len() - 1
    };
    if game
        .snake
        .iter()
        .take(collision_range)
        .any(|&seg| seg == new_head)
    {
        game.game_result = Some(SnakeResult::Loss);
        return;
    }

    // Move: add new head
    game.snake.push_front(new_head);

    // Check food
    if eating {
        // Grow (don't remove tail) and score
        game.score += 1;

        // Win condition
        if game.score >= game.target_score {
            game.game_result = Some(SnakeResult::Win);
            return;
        }

        // Spawn new food
        let mut rng = rand::thread_rng();
        game.food = spawn_food(game, &mut rng);
    } else {
        // Normal move: remove tail
        game.snake.pop_back();
    }
}

impl DifficultyInfo for SnakeDifficulty {
    fn name(&self) -> &'static str {
        SnakeDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            SnakeDifficulty::Novice => ChallengeReward {
                xp_percent: 25,
                ..Default::default()
            },
            SnakeDifficulty::Apprentice => ChallengeReward {
                xp_percent: 75,
                ..Default::default()
            },
            SnakeDifficulty::Journeyman => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            SnakeDifficulty::Master => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 100,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        match self {
            SnakeDifficulty::Novice => Some("10 food, slow (200ms)".to_string()),
            SnakeDifficulty::Apprentice => Some("15 food, moderate (150ms)".to_string()),
            SnakeDifficulty::Journeyman => Some("20 food, fast (120ms)".to_string()),
            SnakeDifficulty::Master => Some("25 food, very fast (90ms)".to_string()),
        }
    }
}

/// Apply game result using the shared challenge reward system.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, score, target) = {
        if let Some(ActiveMinigame::Snake(ref game)) = state.active_minigame {
            (
                game.game_result,
                game.difficulty,
                game.score,
                game.target_score,
            )
        } else {
            return None;
        }
    };

    let result = result?;
    let won = matches!(result, SnakeResult::Win);
    let reward = difficulty.reward();

    // Log score-specific message before the shared reward system logs its messages
    if won {
        state.combat_state.add_log_entry(
            format!(
                "~ You conquered the Serpent's Path! ({}/{} food)",
                score, target
            ),
            false,
            true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!("~ The serpent falls after {} food.", score),
            false,
            true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "snake",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "~",
            win_message: "Serpent's Path conquered!",
            loss_message: "The serpent has fallen.",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Novice move interval (200ms) for tests.
    const NOVICE_INTERVAL: u64 = 200;

    /// Create a game that has already been started (skips the "Press Space" screen).
    fn started_game(difficulty: SnakeDifficulty) -> SnakeGame {
        let mut rng = rand::thread_rng();
        let mut game = SnakeGame::new(difficulty, &mut rng);
        game.waiting_to_start = false;
        game
    }

    #[test]
    fn test_waiting_to_start_blocks_input() {
        let mut rng = rand::thread_rng();
        let mut game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        assert!(game.waiting_to_start);

        // Non-select input is ignored
        process_input(&mut game, SnakeInput::Other);
        assert!(game.waiting_to_start);

        // Direction input is ignored
        process_input(&mut game, SnakeInput::Up);
        assert!(game.waiting_to_start);

        // Select starts the game
        process_input(&mut game, SnakeInput::Select);
        assert!(!game.waiting_to_start);
    }

    #[test]
    fn test_waiting_to_start_blocks_physics() {
        let mut rng = rand::thread_rng();
        let mut game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        let head_before = game.snake[0];

        let changed = tick_snake(&mut game, 1000);

        assert!(!changed);
        assert_eq!(game.snake[0], head_before);
    }

    #[test]
    fn test_direction_change() {
        let mut game = started_game(SnakeDifficulty::Novice);
        assert_eq!(game.direction, Direction::Right);

        process_input(&mut game, SnakeInput::Up);
        assert_eq!(game.next_direction, Direction::Up);
    }

    #[test]
    fn test_direction_prevents_180_reversal() {
        let mut game = started_game(SnakeDifficulty::Novice);
        assert_eq!(game.direction, Direction::Right);

        // Trying to go left (opposite of right) should be ignored
        process_input(&mut game, SnakeInput::Left);
        assert_eq!(game.next_direction, Direction::Right);
    }

    #[test]
    fn test_direction_prevents_180_reversal_all_directions() {
        let mut game = started_game(SnakeDifficulty::Novice);

        // Moving right, can't go left
        game.direction = Direction::Right;
        game.next_direction = Direction::Right;
        process_input(&mut game, SnakeInput::Left);
        assert_eq!(game.next_direction, Direction::Right);

        // Moving left, can't go right
        game.direction = Direction::Left;
        game.next_direction = Direction::Left;
        process_input(&mut game, SnakeInput::Right);
        assert_eq!(game.next_direction, Direction::Left);

        // Moving up, can't go down
        game.direction = Direction::Up;
        game.next_direction = Direction::Up;
        process_input(&mut game, SnakeInput::Down);
        assert_eq!(game.next_direction, Direction::Up);

        // Moving down, can't go up
        game.direction = Direction::Down;
        game.next_direction = Direction::Down;
        process_input(&mut game, SnakeInput::Up);
        assert_eq!(game.next_direction, Direction::Down);
    }

    #[test]
    fn test_forfeit_flow() {
        let mut game = started_game(SnakeDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, SnakeInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Second Esc confirms
        process_input(&mut game, SnakeInput::Forfeit);
        assert_eq!(game.game_result, Some(SnakeResult::Loss));
    }

    #[test]
    fn test_forfeit_cancelled_by_other() {
        let mut game = started_game(SnakeDifficulty::Novice);

        process_input(&mut game, SnakeInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, SnakeInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_forfeit_cancelled_by_direction() {
        let mut game = started_game(SnakeDifficulty::Novice);

        process_input(&mut game, SnakeInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, SnakeInput::Up);
        assert!(!game.forfeit_pending);
        // Direction should NOT change when cancelling forfeit
        assert_eq!(game.next_direction, Direction::Right);
    }

    #[test]
    fn test_input_ignored_when_game_over() {
        let mut game = started_game(SnakeDifficulty::Novice);
        game.game_result = Some(SnakeResult::Win);

        process_input(&mut game, SnakeInput::Up);
        assert_eq!(game.next_direction, Direction::Right);
    }

    #[test]
    fn test_snake_moves_right() {
        let mut game = started_game(SnakeDifficulty::Novice);
        let head_before = game.snake[0];

        // Tick enough to trigger one movement step
        tick_snake(&mut game, NOVICE_INTERVAL);

        let head_after = game.snake[0];
        assert_eq!(head_after.x, head_before.x + 1);
        assert_eq!(head_after.y, head_before.y);
    }

    #[test]
    fn test_snake_moves_up() {
        let mut game = started_game(SnakeDifficulty::Novice);
        game.next_direction = Direction::Up;
        let head_before = game.snake[0];

        tick_snake(&mut game, NOVICE_INTERVAL);

        let head_after = game.snake[0];
        assert_eq!(head_after.x, head_before.x);
        assert_eq!(head_after.y, head_before.y - 1);
    }

    #[test]
    fn test_snake_length_preserved_without_food() {
        let mut game = started_game(SnakeDifficulty::Novice);
        // Move food far away
        game.food = Position { x: 0, y: 0 };
        let len_before = game.snake.len();

        tick_snake(&mut game, NOVICE_INTERVAL);

        assert_eq!(game.snake.len(), len_before);
    }

    #[test]
    fn test_eating_food_grows_snake() {
        let mut game = started_game(SnakeDifficulty::Novice);
        let head = game.snake[0];
        // Place food directly ahead
        game.food = Position {
            x: head.x + 1,
            y: head.y,
        };
        let len_before = game.snake.len();

        tick_snake(&mut game, NOVICE_INTERVAL);

        assert_eq!(game.snake.len(), len_before + 1);
        assert_eq!(game.score, 1);
    }

    #[test]
    fn test_wall_collision_right() {
        let mut game = started_game(SnakeDifficulty::Novice);
        // Place head near right wall
        game.snake[0] = Position {
            x: game.grid_width - 1,
            y: 7,
        };
        game.direction = Direction::Right;
        game.next_direction = Direction::Right;

        tick_snake(&mut game, NOVICE_INTERVAL);

        assert_eq!(game.game_result, Some(SnakeResult::Loss));
    }

    #[test]
    fn test_wall_collision_top() {
        let mut game = started_game(SnakeDifficulty::Novice);
        game.snake[0] = Position { x: 10, y: 0 };
        game.direction = Direction::Up;
        game.next_direction = Direction::Up;

        tick_snake(&mut game, NOVICE_INTERVAL);

        assert_eq!(game.game_result, Some(SnakeResult::Loss));
    }

    #[test]
    fn test_self_collision() {
        let mut game = started_game(SnakeDifficulty::Novice);
        // Create a snake that will collide with itself:
        // Shape like a U-turn: head moving left into its own body
        game.snake.clear();
        game.snake.push_back(Position { x: 5, y: 5 }); // head
        game.snake.push_back(Position { x: 5, y: 4 });
        game.snake.push_back(Position { x: 6, y: 4 });
        game.snake.push_back(Position { x: 6, y: 5 });
        game.snake.push_back(Position { x: 6, y: 6 });
        game.direction = Direction::Right;
        game.next_direction = Direction::Right;
        // Moving right from (5,5) goes to (6,5) which is occupied
        game.food = Position { x: 0, y: 0 };

        tick_snake(&mut game, NOVICE_INTERVAL);

        assert_eq!(game.game_result, Some(SnakeResult::Loss));
    }

    #[test]
    fn test_win_condition() {
        let mut game = started_game(SnakeDifficulty::Novice);
        game.score = 9; // One away from winning (target = 10)
        let head = game.snake[0];
        game.food = Position {
            x: head.x + 1,
            y: head.y,
        };

        tick_snake(&mut game, NOVICE_INTERVAL);

        assert_eq!(game.score, 10);
        assert_eq!(game.game_result, Some(SnakeResult::Win));
    }

    #[test]
    fn test_tick_returns_false_when_game_over() {
        let mut game = started_game(SnakeDifficulty::Novice);
        game.game_result = Some(SnakeResult::Win);

        let changed = tick_snake(&mut game, NOVICE_INTERVAL);
        assert!(!changed, "Tick should return false when game is over");
    }

    #[test]
    fn test_physics_paused_during_forfeit() {
        let mut game = started_game(SnakeDifficulty::Novice);
        game.forfeit_pending = true;
        let head_before = game.snake[0];

        let changed = tick_snake(&mut game, 1000);

        assert!(!changed);
        assert_eq!(game.snake[0], head_before);
    }

    #[test]
    fn test_dt_clamped() {
        let mut game = started_game(SnakeDifficulty::Novice);
        // Huge dt should be clamped to 500ms max
        tick_snake(&mut game, 50000);

        // At 200ms per step, 500ms clamped = at most 2 steps
        assert!(game.tick_count <= 3);
    }

    #[test]
    fn test_reward_structure() {
        assert_eq!(
            SnakeDifficulty::Novice.reward(),
            ChallengeReward {
                xp_percent: 25,
                ..Default::default()
            }
        );
        assert_eq!(
            SnakeDifficulty::Apprentice.reward(),
            ChallengeReward {
                xp_percent: 75,
                ..Default::default()
            }
        );
        assert_eq!(
            SnakeDifficulty::Journeyman.reward(),
            ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            }
        );
        assert_eq!(
            SnakeDifficulty::Master.reward(),
            ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 100,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_extra_info() {
        assert_eq!(
            SnakeDifficulty::Novice.extra_info().unwrap(),
            "10 food, slow (200ms)"
        );
        assert_eq!(
            SnakeDifficulty::Master.extra_info().unwrap(),
            "25 food, very fast (90ms)"
        );
    }

    #[test]
    fn test_difficulty_str_values() {
        assert_eq!(SnakeDifficulty::Novice.difficulty_str(), "novice");
        assert_eq!(SnakeDifficulty::Apprentice.difficulty_str(), "apprentice");
        assert_eq!(SnakeDifficulty::Journeyman.difficulty_str(), "journeyman");
        assert_eq!(SnakeDifficulty::Master.difficulty_str(), "master");
    }

    #[test]
    fn test_apply_game_result_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let initial_xp = state.character_xp;

        let mut rng = rand::thread_rng();
        let mut game = SnakeGame::new(SnakeDifficulty::Apprentice, &mut rng);
        game.game_result = Some(SnakeResult::Win);
        game.score = 15;
        state.active_minigame = Some(ActiveMinigame::Snake(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.game_type, "snake");
        assert_eq!(info.difficulty, "apprentice");
        assert!(state.character_xp > initial_xp);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_game_result_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;

        let mut rng = rand::thread_rng();
        let mut game = SnakeGame::new(SnakeDifficulty::Novice, &mut rng);
        game.game_result = Some(SnakeResult::Loss);
        state.active_minigame = Some(ActiveMinigame::Snake(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_none());
        assert_eq!(state.character_xp, initial_xp);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_game_result_no_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_minigame = None;

        let result = apply_game_result(&mut state);
        assert!(result.is_none());
    }

    #[test]
    fn test_tail_chasing_allowed() {
        let mut game = started_game(SnakeDifficulty::Novice);
        // Set up a snake where the head moves into the tail's current position.
        // The tail will vacate that position this step, so it should be allowed.
        // Snake: head at (5,5), body at (4,5), tail at (4,6)
        // Moving down from (5,5) would go to (5,6) -- not tail chasing.
        // Instead, set up: head at (5,5), body at (6,5), tail at (6,6), and move down.
        // Head goes to (5,6), which is NOT occupied. Not a good test.
        // Better: snake going in a tight circle. Head at (5,5), body at (5,6), tail at (6,6).
        // Direction: Right. New head = (6,5). Not occupied. Still not tail chasing.
        // Real test: Head at (5,5), body at (6,5), body at (6,6), tail at (5,6).
        // Direction: Down. New head = (5,6) which is the tail position.
        game.snake.clear();
        game.snake.push_back(Position { x: 5, y: 5 }); // head
        game.snake.push_back(Position { x: 6, y: 5 });
        game.snake.push_back(Position { x: 6, y: 6 });
        game.snake.push_back(Position { x: 5, y: 6 }); // tail
        game.direction = Direction::Down;
        game.next_direction = Direction::Down;
        // Food is NOT at (5,6), so tail will be removed
        game.food = Position { x: 0, y: 0 };

        tick_snake(&mut game, NOVICE_INTERVAL);

        // Should NOT die -- the tail at (5,6) vacates before head arrives
        assert!(
            game.game_result.is_none(),
            "Snake should be able to chase its own tail"
        );
        assert_eq!(game.snake[0], Position { x: 5, y: 6 });
    }

    #[test]
    fn test_food_respawn_after_eating() {
        let mut game = started_game(SnakeDifficulty::Novice);
        let head = game.snake[0];
        // Place food directly ahead
        game.food = Position {
            x: head.x + 1,
            y: head.y,
        };

        tick_snake(&mut game, NOVICE_INTERVAL);

        // New food should be within bounds and not on the snake
        assert!(game.food.x >= 0 && game.food.x < game.grid_width);
        assert!(game.food.y >= 0 && game.food.y < game.grid_height);
        assert!(
            !game.snake.contains(&game.food),
            "New food must not overlap with the snake body"
        );
    }

    #[test]
    fn test_all_difficulties_have_valid_parameters() {
        for diff in &SnakeDifficulty::ALL {
            assert!(
                diff.grid_width() > 0,
                "{:?} grid_width must be positive",
                diff
            );
            assert!(
                diff.grid_height() > 0,
                "{:?} grid_height must be positive",
                diff
            );
            assert!(
                diff.move_interval_ms() > 0,
                "{:?} move_interval_ms must be positive",
                diff
            );
            assert!(
                diff.target_score() > 0,
                "{:?} target_score must be positive",
                diff
            );
        }
    }
}
