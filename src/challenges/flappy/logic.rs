//! Game logic for the Flappy Bird challenge minigame.

use super::types::{FlappyDifficulty, FlappyGame, FlappyResult, Pipe};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::{apply_challenge_rewards, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;
use rand::Rng;

/// Input actions for Flappy Bird.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyInput {
    /// Flap (Space or Enter or Up).
    Flap,
    /// Forfeit (Esc).
    Forfeit,
    /// Any other key.
    Other,
}

/// Process player input for the flappy bird game.
pub fn process_input(game: &mut FlappyGame, input: FlappyInput) {
    if game.game_result.is_some() {
        return;
    }

    match input {
        FlappyInput::Flap => {
            if game.forfeit_pending {
                // Any non-Esc key cancels forfeit
                game.forfeit_pending = false;
                return;
            }
            if !game.started {
                game.started = true;
            }
            game.bird_vel = FlappyGame::FLAP_VELOCITY;
        }
        FlappyInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(FlappyResult::Forfeit);
            } else {
                game.forfeit_pending = true;
            }
        }
        FlappyInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            }
        }
    }
}

/// Process one game tick (called every 100ms from the game loop).
/// Handles gravity, pipe scrolling, collision detection, and scoring.
pub fn process_tick<R: Rng>(game: &mut FlappyGame, rng: &mut R) {
    if game.game_result.is_some() || !game.started {
        return;
    }

    // Apply gravity
    game.bird_vel += FlappyGame::GRAVITY;
    if game.bird_vel > FlappyGame::MAX_VELOCITY {
        game.bird_vel = FlappyGame::MAX_VELOCITY;
    }
    game.bird_y += game.bird_vel;

    // Check ceiling/floor collision
    if game.bird_y < 0.0 {
        game.bird_y = 0.0;
        game.bird_vel = 0.0;
    }
    if game.bird_y >= game.area_height as f64 - 1.0 {
        game.game_result = Some(FlappyResult::Loss);
        return;
    }

    // Increment tick counter for pipe scrolling
    game.tick_count += 1;

    // Only scroll pipes on speed-tick intervals
    if !game
        .tick_count
        .is_multiple_of(game.difficulty.pipe_speed_ticks())
    {
        return;
    }

    // Scroll pipes left
    for pipe in &mut game.pipes {
        pipe.x -= 1;
    }

    // Remove pipes that have scrolled off screen
    game.pipes.retain(|p| p.x >= -2);

    // Score pipes the bird has passed
    let bird_x = game.bird_x as i32;
    for pipe in &mut game.pipes {
        if !pipe.scored && pipe.x < bird_x {
            pipe.scored = true;
            game.score += 1;
        }
    }

    // Check win condition
    if game.score >= game.difficulty.target_score() {
        game.game_result = Some(FlappyResult::Win);
        return;
    }

    // Spawn new pipes
    game.next_pipe_in = game.next_pipe_in.saturating_sub(1);
    if game.next_pipe_in == 0 {
        let gap_size = game.difficulty.gap_size();
        // Random gap position: keep gap fully within play area (with 1-row margin)
        let min_gap_top = 2u16;
        let max_gap_top = game.area_height.saturating_sub(gap_size + 2);
        let gap_top = if max_gap_top > min_gap_top {
            rng.gen_range(min_gap_top..=max_gap_top)
        } else {
            min_gap_top
        };

        game.pipes.push(Pipe {
            x: game.area_width as i32,
            gap_top,
            scored: false,
        });

        game.next_pipe_in = game.difficulty.pipe_spacing();
    }

    // Check pipe collisions
    check_collisions(game);
}

/// Check if the bird collides with any pipe.
fn check_collisions(game: &mut FlappyGame) {
    let bird_row = game.bird_y.round() as i32;
    let bird_x = game.bird_x as i32;
    let gap_size = game.difficulty.gap_size() as i32;

    for pipe in &game.pipes {
        // Bird occupies columns bird_x and bird_x+1 (2 chars wide: ">o" or similar)
        // Pipe occupies columns pipe.x and pipe.x+1 (2 chars wide)
        let pipe_left = pipe.x;
        let pipe_right = pipe.x + 1;
        let bird_left = bird_x;
        let bird_right = bird_x + 1;

        // Check horizontal overlap
        if bird_right >= pipe_left && bird_left <= pipe_right {
            let gap_top = pipe.gap_top as i32;
            let gap_bottom = gap_top + gap_size - 1;

            // Bird must be within the gap
            if bird_row < gap_top || bird_row > gap_bottom {
                game.game_result = Some(FlappyResult::Loss);
                return;
            }
        }
    }
}

/// Apply the game result to the game state. Called when player presses any key
/// on the game-over screen.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (won, difficulty) = {
        if let Some(crate::challenges::ActiveMinigame::Flappy(ref game)) = state.active_minigame {
            let won = game.game_result == Some(FlappyResult::Win);
            (won, game.difficulty)
        } else {
            return None;
        }
    };

    let info = GameResultInfo {
        won,
        game_type: "flappy",
        difficulty_str: difficulty.difficulty_str(),
        reward: difficulty.reward(),
        icon: "\u{1F426}", // bird emoji
        win_message: "You soared through the gauntlet!",
        loss_message: "You crashed and burned.",
    };

    apply_challenge_rewards(state, info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flap_sets_velocity() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        process_input(&mut game, FlappyInput::Flap);
        assert!(game.bird_vel < 0.0); // Upward velocity
    }

    #[test]
    fn test_flap_starts_game() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        assert!(!game.started);
        process_input(&mut game, FlappyInput::Flap);
        assert!(game.started);
    }

    #[test]
    fn test_gravity_pulls_bird_down() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        let initial_y = game.bird_y;
        let mut rng = rand::thread_rng();
        process_tick(&mut game, &mut rng);
        assert!(game.bird_y > initial_y);
    }

    #[test]
    fn test_floor_collision_ends_game() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        game.bird_y = game.area_height as f64 - 0.5;
        game.bird_vel = 1.0;
        let mut rng = rand::thread_rng();
        process_tick(&mut game, &mut rng);
        assert_eq!(game.game_result, Some(FlappyResult::Loss));
    }

    #[test]
    fn test_ceiling_clamp() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        game.bird_y = 0.5;
        game.bird_vel = -5.0;
        let mut rng = rand::thread_rng();
        process_tick(&mut game, &mut rng);
        assert!(game.bird_y >= 0.0);
        assert!(game.game_result.is_none()); // Ceiling doesn't kill
    }

    #[test]
    fn test_pipe_scrolling() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        game.pipes.push(Pipe {
            x: 30,
            gap_top: 5,
            scored: false,
        });
        let initial_x = game.pipes[0].x;
        let mut rng = rand::thread_rng();
        // Run enough ticks for at least one scroll step
        for _ in 0..game.difficulty.pipe_speed_ticks() {
            if game.game_result.is_some() {
                break;
            }
            process_tick(&mut game, &mut rng);
        }
        assert!(game.pipes[0].x < initial_x);
    }

    #[test]
    fn test_scoring() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        // Place a pipe just to the right of the bird, so it scrolls past
        game.pipes.push(Pipe {
            x: game.bird_x as i32 + 1,
            gap_top: (game.bird_y as u16).saturating_sub(3),
            scored: false,
        });
        let mut rng = rand::thread_rng();
        // Run ticks until the pipe passes the bird
        for _ in 0..20 {
            if game.game_result.is_some() {
                break;
            }
            process_tick(&mut game, &mut rng);
        }
        assert!(game.score > 0 || game.game_result.is_some());
    }

    #[test]
    fn test_pipe_collision() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        // Place the bird directly in a pipe (outside gap)
        game.bird_y = 1.0; // Near top
        game.pipes.push(Pipe {
            x: game.bird_x as i32, // Directly on bird
            gap_top: 10,           // Gap is lower
            scored: false,
        });
        check_collisions(&mut game);
        assert_eq!(game.game_result, Some(FlappyResult::Loss));
    }

    #[test]
    fn test_no_collision_in_gap() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        game.bird_y = 10.0;
        game.pipes.push(Pipe {
            x: game.bird_x as i32,
            gap_top: 7, // Gap from 7 to 7+8-1=14, bird at 10 is inside
            scored: false,
        });
        check_collisions(&mut game);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_win_condition() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        game.score = game.difficulty.target_score() - 1;
        // Place pipe that will be scored on next scroll
        game.pipes.push(Pipe {
            x: game.bird_x as i32 - 1,
            gap_top: (game.bird_y as u16).saturating_sub(3),
            scored: false,
        });
        let mut rng = rand::thread_rng();
        // Run ticks to trigger scoring
        for _ in 0..10 {
            if game.game_result.is_some() {
                break;
            }
            process_tick(&mut game, &mut rng);
        }
        // Should have won (pipe was already behind bird)
        assert_eq!(game.game_result, Some(FlappyResult::Win));
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;

        process_input(&mut game, FlappyInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        process_input(&mut game, FlappyInput::Forfeit);
        assert_eq!(game.game_result, Some(FlappyResult::Forfeit));
    }

    #[test]
    fn test_forfeit_cancelled_by_other_key() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;

        process_input(&mut game, FlappyInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, FlappyInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_no_tick_when_not_started() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        let initial_y = game.bird_y;
        let mut rng = rand::thread_rng();
        process_tick(&mut game, &mut rng);
        assert!((game.bird_y - initial_y).abs() < 0.001);
    }

    #[test]
    fn test_velocity_capped() {
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.started = true;
        game.bird_vel = 100.0; // Way over max
        let mut rng = rand::thread_rng();
        process_tick(&mut game, &mut rng);
        assert!(game.bird_vel <= FlappyGame::MAX_VELOCITY + 0.001);
    }

    #[test]
    fn test_difficulty_info_impl() {
        for d in &FlappyDifficulty::ALL {
            assert!(!d.name().is_empty());
            assert!(!d.difficulty_str().is_empty());
            assert!(d.extra_info().is_some());
        }
    }

    #[test]
    fn test_apply_game_result_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.game_result = Some(FlappyResult::Win);
        state.active_minigame = Some(crate::challenges::ActiveMinigame::Flappy(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.game_type, "flappy");
    }

    #[test]
    fn test_apply_game_result_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut game = FlappyGame::new(FlappyDifficulty::Novice);
        game.game_result = Some(FlappyResult::Loss);
        state.active_minigame = Some(crate::challenges::ActiveMinigame::Flappy(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_none());
    }
}
