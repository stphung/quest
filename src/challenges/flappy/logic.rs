//! Flappy Bird game logic: physics, input processing, collision detection.

use super::types::*;
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;
use rand::RngExt;

/// Physics tick interval in milliseconds (~60 FPS).
const PHYSICS_TICK_MS: u64 = 16;

/// UI-agnostic input actions for Flappy Bird.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyBirdInput {
    Flap,    // Space or Up arrow
    Forfeit, // Esc
    Other,   // Any other key (cancels forfeit_pending)
}

/// Process player input.
pub fn process_input(game: &mut FlappyBirdGame, input: FlappyBirdInput) {
    if game.game_result.is_some() {
        return; // Game over — any key dismisses (handled by input.rs)
    }

    // Waiting screen: Space starts the game
    if game.waiting_to_start {
        if matches!(input, FlappyBirdInput::Flap) {
            game.waiting_to_start = false;
            game.flap_queued = true; // First flap to get the bird moving
        }
        return;
    }

    match input {
        FlappyBirdInput::Flap => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            } else {
                game.flap_queued = true;
            }
        }
        FlappyBirdInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(FlappyBirdResult::Loss); // Confirm forfeit
            } else {
                game.forfeit_pending = true;
            }
        }
        FlappyBirdInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            }
        }
    }
}

/// Advance Flappy Bird physics. Called from the main game loop.
///
/// `dt_ms` is milliseconds since last call. Internally steps physics in
/// 16ms increments (~60 FPS). Returns true if the game state changed.
pub fn tick_flappy_bird(game: &mut FlappyBirdGame, dt_ms: u64) -> bool {
    if game.game_result.is_some() {
        return false;
    }

    // Pause physics while waiting to start or during forfeit
    if game.waiting_to_start || game.forfeit_pending {
        return false;
    }

    // Clamp dt to 100ms max to prevent physics explosion after pause/lag
    let dt_ms = dt_ms.min(100);

    game.accumulated_time_ms += dt_ms;
    let mut changed = false;

    // Step physics in fixed 33ms increments
    while game.accumulated_time_ms >= PHYSICS_TICK_MS {
        game.accumulated_time_ms -= PHYSICS_TICK_MS;
        step_physics(game);
        changed = true;

        if game.game_result.is_some() {
            break;
        }
    }

    changed
}

/// Single physics step (16ms tick, values pre-calibrated in types.rs).
fn step_physics(game: &mut FlappyBirdGame) {
    game.tick_count += 1;

    // Consume buffered flap input
    if game.flap_queued {
        game.bird_velocity = game.flap_impulse;
        game.flap_timer = FLAP_ANIM_TICKS;
        game.flap_queued = false;
    }

    // Decrement flap animation timer
    if game.flap_timer > 0 {
        game.flap_timer -= 1;
    }

    // Apply gravity
    game.bird_velocity += game.gravity;

    // Cap terminal velocity
    if game.bird_velocity > game.terminal_velocity {
        game.bird_velocity = game.terminal_velocity;
    }

    // Update bird position
    game.bird_y += game.bird_velocity;

    // Move pipes left
    for pipe in &mut game.pipes {
        pipe.x -= game.pipe_speed;
    }

    // Score: check if any pipe's right edge passed the bird's column
    let bird_x = BIRD_COL as f64;
    let pipe_right_offset = (PIPE_WIDTH as f64) / 2.0;
    for pipe in &mut game.pipes {
        if !pipe.passed && (pipe.x + pipe_right_offset) < bird_x {
            pipe.passed = true;
            game.score += 1;
        }
    }

    // Spawn new pipes when needed
    // The next_pipe_x tracks when a new pipe should enter from the right.
    // We need to move next_pipe_x left along with pipes, then spawn when it enters the screen.
    game.next_pipe_x -= game.pipe_speed;
    if game.next_pipe_x <= GAME_WIDTH as f64 {
        let mut rng = rand::rng();
        game.spawn_pipe(&mut rng);
    }

    // Remove off-screen pipes (well past the left edge)
    game.pipes.retain(|p| p.x > -(PIPE_WIDTH as f64));

    // Collision detection: floor/ceiling
    // Row 0 = ceiling, Row 17 = ground
    let bird_row = game.bird_y.round() as i32;
    if bird_row <= 0 || bird_row >= GAME_HEIGHT as i32 - 1 {
        handle_collision(game);
        return;
    }

    // Collision detection: pipes
    // Bird occupies columns BIRD_COL and BIRD_COL+1 (2 chars wide), row = bird_row
    let bird_left = BIRD_COL as f64;
    let bird_right = (BIRD_COL + 2) as f64;
    let bird_row_f = game.bird_y.round();

    for pipe in &game.pipes {
        let pipe_left = pipe.x - (PIPE_WIDTH as f64) / 2.0;
        let pipe_right = pipe.x + (PIPE_WIDTH as f64) / 2.0;

        // Check horizontal overlap
        if bird_right > pipe_left && bird_left < pipe_right {
            let half_gap = game.pipe_gap as f64 / 2.0;
            let gap_top = (pipe.gap_center as f64 - half_gap).floor();
            let gap_bottom = (pipe.gap_center as f64 + half_gap).ceil();

            // Bird outside the gap?
            if bird_row_f < gap_top || bird_row_f > gap_bottom {
                handle_collision(game);
                return;
            }
        }
    }

    // Win condition
    if game.score >= game.target_score {
        game.game_result = Some(FlappyBirdResult::Win);
    }
}

/// Handle a collision: consume a life and reset, or end the game.
fn handle_collision(game: &mut FlappyBirdGame) {
    if game.lives > 0 {
        game.lives -= 1;
        game.reset_for_retry();
    } else {
        game.game_result = Some(FlappyBirdResult::Loss);
    }
}

impl DifficultyInfo for FlappyBirdDifficulty {
    fn name(&self) -> &'static str {
        FlappyBirdDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            FlappyBirdDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            FlappyBirdDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            FlappyBirdDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            FlappyBirdDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        match self {
            FlappyBirdDifficulty::Novice => Some("10 pipes, wide gaps, 3 lives".to_string()),
            FlappyBirdDifficulty::Apprentice => Some("15 pipes, normal gaps, 3 lives".to_string()),
            FlappyBirdDifficulty::Journeyman => Some("20 pipes, narrow gaps, 3 lives".to_string()),
            FlappyBirdDifficulty::Master => Some("30 pipes, razor gaps, 3 lives".to_string()),
        }
    }
}

/// Apply game result using the shared challenge reward system.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, score, target) = {
        if let Some(ActiveMinigame::FlappyBird(ref game)) = state.active_minigame {
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
    let won = matches!(result, FlappyBirdResult::Win);
    let reward = difficulty.reward();

    // Log score-specific message before the shared reward system logs its messages
    if won {
        state.combat_state.add_log_entry(
            format!(
                "> You conquered the Skyward Gauntlet! ({}/{} pipes)",
                score, target
            ),
            false,
            true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!(
                "> Crashed after {} pipes ({} lives used).",
                score, MAX_LIVES
            ),
            false,
            true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "flappy_bird",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: ">",
            win_message: "Skyward Gauntlet conquered!",
            loss_message: "The gauntlet claims another.",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a game that has already been started (skips the "Press Space" screen).
    fn started_game(difficulty: FlappyBirdDifficulty) -> FlappyBirdGame {
        let mut game = FlappyBirdGame::new(difficulty);
        game.waiting_to_start = false;
        game
    }

    #[test]
    fn test_waiting_to_start_blocks_input() {
        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        assert!(game.waiting_to_start);

        // Non-flap input is ignored
        process_input(&mut game, FlappyBirdInput::Other);
        assert!(game.waiting_to_start);

        // Flap starts the game
        process_input(&mut game, FlappyBirdInput::Flap);
        assert!(!game.waiting_to_start);
        assert!(game.flap_queued);
    }

    #[test]
    fn test_waiting_to_start_blocks_physics() {
        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        let y_before = game.bird_y;

        let changed = tick_flappy_bird(&mut game, 100);

        assert!(!changed);
        assert!((game.bird_y - y_before).abs() < f64::EPSILON);
    }

    #[test]
    fn test_process_input_flap_queues() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        assert!(!game.flap_queued);

        process_input(&mut game, FlappyBirdInput::Flap);
        assert!(game.flap_queued);
    }

    #[test]
    fn test_process_input_forfeit_flow() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, FlappyBirdInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Second Esc confirms
        process_input(&mut game, FlappyBirdInput::Forfeit);
        assert_eq!(game.game_result, Some(FlappyBirdResult::Loss));
    }

    #[test]
    fn test_process_input_forfeit_cancelled_by_other() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);

        process_input(&mut game, FlappyBirdInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, FlappyBirdInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_cancelled_by_flap() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);

        process_input(&mut game, FlappyBirdInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, FlappyBirdInput::Flap);
        assert!(!game.forfeit_pending);
        // Flap should NOT be queued when cancelling forfeit
        assert!(!game.flap_queued);
    }

    #[test]
    fn test_process_input_ignored_when_game_over() {
        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        game.game_result = Some(FlappyBirdResult::Win);

        process_input(&mut game, FlappyBirdInput::Flap);
        assert!(!game.flap_queued);
    }

    #[test]
    fn test_physics_gravity_accumulates() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        let initial_y = game.bird_y;

        // Run a few ticks — bird should fall due to gravity
        tick_flappy_bird(&mut game, 100);

        assert!(game.bird_y > initial_y, "Bird should fall due to gravity");
    }

    #[test]
    fn test_physics_flap_moves_bird_up() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 10.0;
        game.bird_velocity = 0.0;

        // Queue a flap
        game.flap_queued = true;

        // Single physics tick
        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        // Velocity should be negative (upward)
        assert!(
            game.bird_velocity < 0.0,
            "After flap, velocity should be negative (upward)"
        );
    }

    #[test]
    fn test_physics_terminal_velocity_cap() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 5.0;
        game.bird_velocity = 100.0; // absurdly high

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.bird_velocity <= game.terminal_velocity,
            "Velocity should be capped at terminal velocity"
        );
    }

    #[test]
    fn test_physics_paused_during_forfeit() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.forfeit_pending = true;
        let y_before = game.bird_y;

        let changed = tick_flappy_bird(&mut game, 100);

        assert!(!changed);
        assert!((game.bird_y - y_before).abs() < f64::EPSILON);
    }

    #[test]
    fn test_collision_ground_consumes_life() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 16.5;
        game.bird_velocity = 1.0;

        // Tick until bird hits ground
        for _ in 0..10 {
            tick_flappy_bird(&mut game, PHYSICS_TICK_MS);
            if game.waiting_to_start {
                break;
            }
        }

        // Should have lost a life and reset, not ended the game
        assert!(game.game_result.is_none());
        assert_eq!(game.lives, MAX_LIVES - 1);
        assert!(game.waiting_to_start);
    }

    #[test]
    fn test_collision_ceiling_consumes_life() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 0.5;
        game.bird_velocity = -2.0;

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert!(game.game_result.is_none());
        assert_eq!(game.lives, MAX_LIVES - 1);
        assert!(game.waiting_to_start);
    }

    #[test]
    fn test_collision_on_last_life_ends_game() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.lives = 0;
        game.bird_y = 0.5;
        game.bird_velocity = -2.0;

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert_eq!(game.game_result, Some(FlappyBirdResult::Loss));
    }

    #[test]
    fn test_scoring_pipe_passed() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 8.0;

        // Place a pipe that the bird has already passed horizontally
        game.pipes.push(Pipe {
            x: (BIRD_COL as f64) - 5.0,
            gap_center: 8,
            passed: false,
        });

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert_eq!(game.score, 1);
        assert!(game.pipes[0].passed);
    }

    #[test]
    fn test_win_condition() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.score = 9;
        game.bird_y = 8.0;

        // Place a pipe that will be scored
        game.pipes.push(Pipe {
            x: (BIRD_COL as f64) - 5.0,
            gap_center: 8,
            passed: false,
        });

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert_eq!(game.score, 10);
        assert_eq!(game.game_result, Some(FlappyBirdResult::Win));
    }

    #[test]
    fn test_pipe_collision_consumes_life() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        // Bird at row 3, pipe gap centered at row 12 (gap: rows 8..16 for Novice gap=7)
        // Bird is above the gap
        game.bird_y = 3.0;
        game.bird_velocity = 0.0;

        // Place a pipe right at the bird's x position
        game.pipes.push(Pipe {
            x: (BIRD_COL as f64) + 1.0,
            gap_center: 12,
            passed: false,
        });

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.game_result.is_none(),
            "Should lose a life, not end game"
        );
        assert_eq!(game.lives, MAX_LIVES - 1);
        assert!(game.waiting_to_start);
    }

    #[test]
    fn test_bird_passes_through_gap() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        // Bird at row 8, pipe gap centered at row 8 (gap: rows 4.5..11.5 for Novice gap=7)
        game.bird_y = 8.0;
        game.bird_velocity = 0.0;

        // Place a pipe right at the bird's x position
        game.pipes.push(Pipe {
            x: (BIRD_COL as f64) + 1.0,
            gap_center: 8,
            passed: false,
        });

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.game_result.is_none(),
            "Bird should pass through the gap"
        );
    }

    #[test]
    fn test_dt_clamped() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        let y_before = game.bird_y;

        // Huge dt should be clamped to 100ms max
        tick_flappy_bird(&mut game, 5000);

        // Should have only done ~6 physics ticks (100ms / 16ms)
        assert!(game.tick_count <= 7);
        // Bird should have moved, but not exploded
        assert!((game.bird_y - y_before).abs() < 5.0);
    }

    #[test]
    fn test_reward_structure() {
        assert_eq!(
            FlappyBirdDifficulty::Novice.reward(),
            ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            FlappyBirdDifficulty::Apprentice.reward(),
            ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            }
        );
        assert_eq!(
            FlappyBirdDifficulty::Journeyman.reward(),
            ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            }
        );
        assert_eq!(
            FlappyBirdDifficulty::Master.reward(),
            ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            }
        );
    }

    #[test]
    fn test_apply_game_result_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let initial_xp = state.character_xp;

        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Apprentice);
        game.game_result = Some(FlappyBirdResult::Win);
        game.score = 15;
        state.active_minigame = Some(ActiveMinigame::FlappyBird(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.game_type, "flappy_bird");
        assert_eq!(info.difficulty, "apprentice");
        assert!(state.character_xp > initial_xp);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_game_result_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;

        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        game.game_result = Some(FlappyBirdResult::Loss);
        state.active_minigame = Some(ActiveMinigame::FlappyBird(game));

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
    fn test_extra_info() {
        assert_eq!(
            FlappyBirdDifficulty::Novice.extra_info().unwrap(),
            "10 pipes, wide gaps, 3 lives"
        );
        assert_eq!(
            FlappyBirdDifficulty::Master.extra_info().unwrap(),
            "30 pipes, razor gaps, 3 lives"
        );
    }

    #[test]
    fn test_pipe_spawning_during_gameplay() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        // Move next_pipe_x to just barely off the right edge so it triggers soon
        game.next_pipe_x = GAME_WIDTH as f64 + 0.1;
        game.bird_y = 8.0;

        // Run enough ticks for the pipe to enter the screen
        for _ in 0..20 {
            tick_flappy_bird(&mut game, PHYSICS_TICK_MS);
            if !game.pipes.is_empty() {
                break;
            }
        }

        assert!(
            !game.pipes.is_empty(),
            "Pipes should spawn when next_pipe_x reaches screen edge"
        );
    }

    #[test]
    fn test_offscreen_pipe_cleanup() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 8.0;

        // Add a pipe that's already way past the left edge
        game.pipes.push(Pipe {
            x: -(PIPE_WIDTH as f64) - 1.0,
            gap_center: 8,
            passed: true,
        });
        assert_eq!(game.pipes.len(), 1);

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert!(game.pipes.is_empty(), "Off-screen pipes should be removed");
    }

    #[test]
    fn test_collision_pipe_bottom_consumes_life() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        // Bird at row 15, pipe gap centered at row 5 (gap: rows 1..9 for Novice gap=7)
        // Bird is below the gap
        game.bird_y = 15.0;
        game.bird_velocity = 0.0;

        game.pipes.push(Pipe {
            x: (BIRD_COL as f64) + 1.0,
            gap_center: 5,
            passed: false,
        });

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.game_result.is_none(),
            "Should lose a life, not end game"
        );
        assert_eq!(game.lives, MAX_LIVES - 1);
        assert!(game.waiting_to_start);
    }

    #[test]
    fn test_difficulty_str_values() {
        assert_eq!(FlappyBirdDifficulty::Novice.difficulty_str(), "novice");
        assert_eq!(
            FlappyBirdDifficulty::Apprentice.difficulty_str(),
            "apprentice"
        );
        assert_eq!(
            FlappyBirdDifficulty::Journeyman.difficulty_str(),
            "journeyman"
        );
        assert_eq!(FlappyBirdDifficulty::Master.difficulty_str(), "master");
    }

    #[test]
    fn test_all_difficulties_have_valid_parameters() {
        for diff in &FlappyBirdDifficulty::ALL {
            assert!(diff.gravity() > 0.0, "{:?} gravity must be positive", diff);
            assert!(
                diff.flap_impulse() < 0.0,
                "{:?} flap impulse must be negative (upward)",
                diff
            );
            assert!(
                diff.terminal_velocity() > 0.0,
                "{:?} terminal velocity must be positive",
                diff
            );
            assert!(diff.pipe_gap() > 0, "{:?} pipe gap must be positive", diff);
            assert!(
                diff.pipe_speed() > 0.0,
                "{:?} pipe speed must be positive",
                diff
            );
            assert!(
                diff.pipe_spacing() > 0.0,
                "{:?} pipe spacing must be positive",
                diff
            );
            assert!(
                diff.target_score() > 0,
                "{:?} target score must be positive",
                diff
            );
        }
    }

    #[test]
    fn test_tick_returns_false_when_game_over() {
        let mut game = FlappyBirdGame::new(FlappyBirdDifficulty::Novice);
        game.game_result = Some(FlappyBirdResult::Win);

        let changed = tick_flappy_bird(&mut game, PHYSICS_TICK_MS);
        assert!(!changed, "Tick should return false when game is over");
    }

    #[test]
    fn test_flap_animation_timer() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 8.0;
        game.flap_queued = true;

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        // After consuming the flap, flap_timer should be set (then decremented once)
        assert_eq!(
            game.flap_timer,
            FLAP_ANIM_TICKS - 1,
            "Flap timer should be set and decremented by one tick"
        );
        assert!(!game.flap_queued, "Flap should be consumed");
    }

    #[test]
    fn test_score_does_not_double_count() {
        let mut game = started_game(FlappyBirdDifficulty::Novice);
        game.bird_y = 8.0;

        // Place a pipe that's already been scored
        game.pipes.push(Pipe {
            x: (BIRD_COL as f64) - 5.0,
            gap_center: 8,
            passed: true, // Already scored
        });

        tick_flappy_bird(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.score, 0,
            "Already-passed pipes should not be scored again"
        );
    }
}
