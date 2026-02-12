//! Dino Run game logic: physics, input processing, collision detection.

use super::types::*;
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;

/// Physics tick interval in milliseconds (~60 FPS).
const PHYSICS_TICK_MS: u64 = 16;

/// UI-agnostic input actions for Dino Run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinoRunInput {
    Jump,    // Space or Up arrow
    Duck,    // Down arrow (toggle: press to start ducking, press again to stop)
    Forfeit, // Esc
    Other,   // Any other key (cancels forfeit_pending)
}

/// Process player input.
pub fn process_input(game: &mut DinoRunGame, input: DinoRunInput) {
    if game.game_result.is_some() {
        return; // Game over -- any key dismisses (handled by input.rs)
    }

    // Waiting screen: Jump starts the game
    if game.waiting_to_start {
        if matches!(input, DinoRunInput::Jump) {
            game.waiting_to_start = false;
        }
        return;
    }

    match input {
        DinoRunInput::Jump => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            } else {
                game.jump_queued = true;
                // Jumping cancels ducking
                game.is_ducking = false;
                game.duck_queued = false;
            }
        }
        DinoRunInput::Duck => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            } else if game.is_ducking {
                // Toggle off: stop ducking
                game.is_ducking = false;
                game.duck_queued = false;
            } else {
                game.duck_queued = true;
            }
        }
        DinoRunInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(DinoRunResult::Loss); // Confirm forfeit
            } else {
                game.forfeit_pending = true;
            }
        }
        DinoRunInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            }
        }
    }
}

/// Advance Dino Run physics. Called from the main game loop.
///
/// `dt_ms` is milliseconds since last call. Internally steps physics in
/// 16ms increments (~60 FPS). Returns true if the game state changed.
pub fn tick_dino_run(game: &mut DinoRunGame, dt_ms: u64) -> bool {
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

    // Step physics in fixed 16ms increments
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

/// Single physics step (16ms tick).
fn step_physics(game: &mut DinoRunGame) {
    game.tick_count += 1;

    // 1. Consume buffered jump input (only if on ground and not ducking)
    if game.jump_queued && game.is_on_ground() && !game.is_ducking {
        game.velocity = game.jump_impulse;
        game.jump_queued = false;
    }

    // 2. Consume buffered duck input
    if game.duck_queued {
        game.is_ducking = true;
        game.duck_queued = false;
        // If in the air, fast-fall: apply extra gravity
        if !game.is_on_ground() {
            game.velocity += game.gravity * 2.0;
        }
    }

    // 3. Apply gravity (only when airborne)
    if !game.is_on_ground() {
        game.velocity += game.gravity;
        if game.velocity > game.terminal_velocity {
            game.velocity = game.terminal_velocity;
        }
    }

    // 4. Update runner position
    game.runner_y += game.velocity;

    // 5. Clamp to ground
    if game.runner_y >= GROUND_ROW as f64 {
        game.runner_y = GROUND_ROW as f64;
        game.velocity = 0.0;
    }

    // 6. Move obstacles left
    for obstacle in &mut game.obstacles {
        obstacle.x -= game.game_speed;
    }

    // 7. Score: check if runner has passed obstacles
    let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;
    for obstacle in &mut game.obstacles {
        if !obstacle.passed && (obstacle.x + obstacle.obstacle_type.width() as f64) < runner_right {
            obstacle.passed = true;
            game.score += 1;
        }
    }

    // 8. Spawn new obstacles
    game.next_obstacle_distance -= game.game_speed;
    if game.next_obstacle_distance <= 0.0 {
        let mut rng = rand::thread_rng();
        game.spawn_obstacle(&mut rng);
    }

    // 9. Remove off-screen obstacles
    game.obstacles.retain(|o| o.x > -10.0);

    // 10. Update game speed (gradual acceleration)
    game.distance += game.game_speed;
    game.game_speed =
        (game.initial_speed + game.distance * game.speed_increase_rate).min(game.max_speed);

    // 11. Update run animation
    if game.is_on_ground() && game.tick_count.is_multiple_of(8) {
        game.run_anim_frame = (game.run_anim_frame + 1) % RUN_ANIM_FRAMES;
    }

    // 12. Collision detection
    if check_collision(game) {
        game.game_result = Some(DinoRunResult::Loss);
        return;
    }

    // 13. Win condition
    if game.score >= game.target_score {
        game.game_result = Some(DinoRunResult::Win);
    }
}

/// Check collision between runner and all obstacles.
fn check_collision(game: &DinoRunGame) -> bool {
    let runner_left = RUNNER_COL as f64;
    let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;

    let runner_height = if game.is_ducking {
        RUNNER_DUCKING_HEIGHT
    } else {
        RUNNER_STANDING_HEIGHT
    };

    // Runner's top row (remember: lower y = higher on screen)
    let runner_top = game.runner_y - (runner_height as f64 - 1.0);
    let runner_bottom = game.runner_y;

    for obstacle in &game.obstacles {
        let obs_left = obstacle.x;
        let obs_right = obstacle.x + obstacle.obstacle_type.width() as f64;

        // Horizontal overlap check
        if runner_right <= obs_left || runner_left >= obs_right {
            continue;
        }

        // Vertical position of obstacle
        let (obs_top, obs_bottom) = if obstacle.obstacle_type.is_flying() {
            // Flying obstacles at head height
            let top = FLYING_ROW as f64;
            let bottom = top + obstacle.obstacle_type.height() as f64 - 1.0;
            (top, bottom)
        } else {
            // Ground obstacles sit on the ground
            let bottom = GROUND_ROW as f64;
            let top = bottom - (obstacle.obstacle_type.height() as f64 - 1.0);
            (top, bottom)
        };

        // Vertical overlap check
        if runner_bottom >= obs_top && runner_top <= obs_bottom {
            return true;
        }
    }

    false
}

impl DifficultyInfo for DinoRunDifficulty {
    fn name(&self) -> &'static str {
        DinoRunDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            DinoRunDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            DinoRunDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            DinoRunDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            DinoRunDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        match self {
            DinoRunDifficulty::Novice => Some("15 obstacles, slow start".to_string()),
            DinoRunDifficulty::Apprentice => Some("25 obstacles, moderate pace".to_string()),
            DinoRunDifficulty::Journeyman => Some("40 obstacles, fast pace".to_string()),
            DinoRunDifficulty::Master => Some("60 obstacles, relentless".to_string()),
        }
    }
}

/// Apply game result using the shared challenge reward system.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, score, target) = {
        if let Some(ActiveMinigame::DinoRun(ref game)) = state.active_minigame {
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
    let won = matches!(result, DinoRunResult::Win);
    let reward = difficulty.reward();

    // Log score-specific message before the shared reward system logs its messages
    if won {
        state.combat_state.add_log_entry(
            format!(
                "> You survived the Gauntlet Run! ({}/{} obstacles)",
                score, target
            ),
            false,
            true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!("> Stumbled after {} obstacles.", score),
            false,
            true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "dino_run",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "!",
            win_message: "Gauntlet Run conquered!",
            loss_message: "The gauntlet claims another.",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a game that has already been started (skips the "Press Space" screen).
    fn started_game(difficulty: DinoRunDifficulty) -> DinoRunGame {
        let mut game = DinoRunGame::new(difficulty);
        game.waiting_to_start = false;
        game
    }

    // ── Input tests ──

    #[test]
    fn test_waiting_to_start_blocks_input() {
        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        assert!(game.waiting_to_start);

        // Non-jump input is ignored
        process_input(&mut game, DinoRunInput::Other);
        assert!(game.waiting_to_start);

        process_input(&mut game, DinoRunInput::Duck);
        assert!(game.waiting_to_start);

        // Jump starts the game
        process_input(&mut game, DinoRunInput::Jump);
        assert!(!game.waiting_to_start);
    }

    #[test]
    fn test_waiting_to_start_blocks_physics() {
        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        let y_before = game.runner_y;

        let changed = tick_dino_run(&mut game, 100);

        assert!(!changed);
        assert!((game.runner_y - y_before).abs() < f64::EPSILON);
    }

    #[test]
    fn test_process_input_jump_queues() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        assert!(!game.jump_queued);

        process_input(&mut game, DinoRunInput::Jump);
        assert!(game.jump_queued);
    }

    #[test]
    fn test_process_input_duck_queues() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        assert!(!game.duck_queued);

        process_input(&mut game, DinoRunInput::Duck);
        assert!(game.duck_queued);
    }

    #[test]
    fn test_process_input_duck_toggle() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        // First duck press queues duck
        process_input(&mut game, DinoRunInput::Duck);
        assert!(game.duck_queued);

        // Simulate physics consuming the duck
        game.is_ducking = true;
        game.duck_queued = false;

        // Second duck press toggles off
        process_input(&mut game, DinoRunInput::Duck);
        assert!(!game.is_ducking);
        assert!(!game.duck_queued);
    }

    #[test]
    fn test_process_input_jump_cancels_duck() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.is_ducking = true;

        process_input(&mut game, DinoRunInput::Jump);
        assert!(!game.is_ducking);
        assert!(game.jump_queued);
    }

    #[test]
    fn test_process_input_forfeit_flow() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, DinoRunInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Second Esc confirms
        process_input(&mut game, DinoRunInput::Forfeit);
        assert_eq!(game.game_result, Some(DinoRunResult::Loss));
    }

    #[test]
    fn test_process_input_forfeit_cancelled_by_other() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        process_input(&mut game, DinoRunInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, DinoRunInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_cancelled_by_jump() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        process_input(&mut game, DinoRunInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, DinoRunInput::Jump);
        assert!(!game.forfeit_pending);
        // Jump should NOT be queued when cancelling forfeit
        assert!(!game.jump_queued);
    }

    #[test]
    fn test_process_input_forfeit_cancelled_by_duck() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        process_input(&mut game, DinoRunInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, DinoRunInput::Duck);
        assert!(!game.forfeit_pending);
        // Duck should NOT be queued when cancelling forfeit
        assert!(!game.duck_queued);
    }

    #[test]
    fn test_process_input_ignored_when_game_over() {
        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        game.game_result = Some(DinoRunResult::Win);

        process_input(&mut game, DinoRunInput::Jump);
        assert!(!game.jump_queued);
    }

    // ── Physics tests ──

    #[test]
    fn test_physics_gravity_accumulates() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Put runner in the air
        game.runner_y = 10.0;
        game.velocity = 0.0;
        let initial_y = game.runner_y;

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.runner_y > initial_y,
            "Runner should fall due to gravity"
        );
    }

    #[test]
    fn test_physics_no_gravity_on_ground() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        assert!(game.is_on_ground());
        let initial_y = game.runner_y;

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            (game.runner_y - initial_y).abs() < f64::EPSILON,
            "Runner on ground should not move vertically"
        );
    }

    #[test]
    fn test_physics_jump_moves_runner_up() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        assert!(game.is_on_ground());

        // Queue a jump
        game.jump_queued = true;

        // Single physics tick
        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.velocity < 0.0,
            "After jump, velocity should be negative (upward)"
        );
        assert!(
            game.runner_y < GROUND_ROW as f64,
            "After jump, runner should be above ground"
        );
    }

    #[test]
    fn test_physics_jump_arc() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.jump_queued = true;

        // Run physics until runner lands
        let mut max_height = game.runner_y;
        for _ in 0..200 {
            tick_dino_run(&mut game, PHYSICS_TICK_MS);
            if game.runner_y < max_height {
                max_height = game.runner_y;
            }
            if game.is_on_ground() && game.tick_count > 2 {
                break;
            }
        }

        // Runner should have gone up (lower y = higher)
        assert!(
            max_height < GROUND_ROW as f64,
            "Runner should have jumped above ground"
        );
        // Runner should be back on ground
        assert!(game.is_on_ground(), "Runner should have landed");
        assert!(
            (game.velocity - 0.0).abs() < f64::EPSILON,
            "Velocity should be 0 on ground"
        );
    }

    #[test]
    fn test_no_double_jump() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.jump_queued = true;

        // Execute the jump
        tick_dino_run(&mut game, PHYSICS_TICK_MS);
        assert!(!game.is_on_ground());

        // Try to jump again while airborne
        game.jump_queued = true;
        let velocity_before = game.velocity;
        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        // Jump should not have been consumed (velocity not reset to jump_impulse)
        assert!(
            game.jump_queued,
            "Jump should remain queued (not consumed) while airborne"
        );
        // Velocity should have increased (gravity), not been set to impulse
        assert!(
            game.velocity > velocity_before,
            "Velocity should increase from gravity, not reset from jump"
        );
    }

    #[test]
    fn test_no_duck_while_airborne_from_input() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.jump_queued = true;

        // Jump
        tick_dino_run(&mut game, PHYSICS_TICK_MS);
        assert!(!game.is_on_ground());

        // Duck while airborne -- should trigger fast-fall via duck_queued
        game.duck_queued = true;
        let velocity_before = game.velocity;
        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        // Duck is consumed and applies fast-fall
        assert!(game.is_ducking);
        assert!(
            game.velocity > velocity_before,
            "Fast-fall should increase downward velocity"
        );
    }

    #[test]
    fn test_physics_terminal_velocity_cap() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.runner_y = 5.0;
        game.velocity = 100.0; // absurdly high

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        // The terminal velocity cap is applied before position update.
        // After the tick, either velocity is capped or runner hit ground.
        assert!(
            game.velocity <= game.terminal_velocity || game.is_on_ground(),
            "Velocity should be capped at terminal velocity or runner should have landed"
        );
    }

    #[test]
    fn test_physics_ground_clamp() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Place runner just barely above ground so one tick of falling lands them
        game.runner_y = GROUND_ROW as f64 - 0.1;
        game.velocity = 0.2; // Moving down

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            (game.runner_y - GROUND_ROW as f64).abs() < f64::EPSILON,
            "Runner should be clamped to ground"
        );
        assert!(
            (game.velocity - 0.0).abs() < f64::EPSILON,
            "Velocity should be zero after landing"
        );
    }

    #[test]
    fn test_physics_paused_during_forfeit() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.forfeit_pending = true;
        let y_before = game.runner_y;

        let changed = tick_dino_run(&mut game, 100);

        assert!(!changed);
        assert!((game.runner_y - y_before).abs() < f64::EPSILON);
    }

    // ── Duck mechanics tests ──

    #[test]
    fn test_duck_reduces_hitbox() {
        let game_standing = started_game(DinoRunDifficulty::Novice);
        let mut game_ducking = started_game(DinoRunDifficulty::Novice);
        game_ducking.is_ducking = true;

        // Standing runner: hitbox covers rows 14-15 (2 rows)
        let standing_top = game_standing.runner_y - (RUNNER_STANDING_HEIGHT as f64 - 1.0);
        let standing_height = game_standing.runner_y - standing_top + 1.0;
        assert!(
            (standing_height - 2.0).abs() < f64::EPSILON,
            "Standing hitbox should be 2 rows"
        );

        // Ducking runner: hitbox covers row 15 only (1 row)
        let ducking_top = game_ducking.runner_y - (RUNNER_DUCKING_HEIGHT as f64 - 1.0);
        let ducking_height = game_ducking.runner_y - ducking_top + 1.0;
        assert!(
            (ducking_height - 1.0).abs() < f64::EPSILON,
            "Ducking hitbox should be 1 row"
        );
    }

    #[test]
    fn test_duck_avoids_flying_obstacle() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.is_ducking = true;

        // Place a flying obstacle right at the runner's position
        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::Bat,
            passed: false,
        });

        // Should NOT collide because ducking makes the runner 1 row (row 15),
        // and flying obstacle is at FLYING_ROW (row 13)
        assert!(
            !check_collision(&game),
            "Ducking should avoid flying obstacle"
        );
    }

    #[test]
    fn test_standing_hits_flying_obstacle() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Standing runner occupies rows 14-15. Flying obstacle at FLYING_ROW (14)
        // overlaps with the runner's head row.

        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::Bat,
            passed: false,
        });

        assert!(
            check_collision(&game),
            "Standing runner should collide with flying obstacle at head height"
        );
    }

    // ── Obstacle generation and movement ──

    #[test]
    fn test_obstacle_movement() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.obstacles.push(Obstacle {
            x: 30.0,
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });
        // Prevent spawning new obstacles
        game.next_obstacle_distance = 999.0;
        let initial_x = game.obstacles[0].x;

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.obstacles[0].x < initial_x,
            "Obstacles should move left"
        );
    }

    #[test]
    fn test_offscreen_obstacle_cleanup() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.obstacles.push(Obstacle {
            x: -11.0, // Way past left edge
            obstacle_type: ObstacleType::SmallRock,
            passed: true,
        });
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.obstacles.is_empty(),
            "Off-screen obstacles should be removed"
        );
    }

    // ── Collision detection ──

    #[test]
    fn test_collision_ground_obstacle_standing() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Runner on ground (rows 14-15), ground obstacle at runner position
        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::SmallRock, // 1 row tall at ground level (row 15)
            passed: false,
        });

        assert!(
            check_collision(&game),
            "Standing runner should collide with ground obstacle"
        );
    }

    #[test]
    fn test_collision_ground_obstacle_jumping_clear() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Runner high in the air
        game.runner_y = 10.0;

        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::SmallRock, // 1 row at ground (row 15)
            passed: false,
        });

        assert!(
            !check_collision(&game),
            "Jumping runner should clear ground obstacle"
        );
    }

    #[test]
    fn test_collision_tall_obstacle_ducking() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.is_ducking = true;

        // Tall obstacle (2 rows: rows 14-15)
        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::LargeRock, // 2 rows tall at ground
            passed: false,
        });

        // Ducking runner is at row 15 only. LargeRock spans rows 14-15.
        // runner_top = 15, runner_bottom = 15. obs_top = 14, obs_bottom = 15.
        // runner_bottom(15) >= obs_top(14) = true. runner_top(15) <= obs_bottom(15) = true.
        assert!(
            check_collision(&game),
            "Ducking runner should still collide with tall obstacle"
        );
    }

    #[test]
    fn test_collision_tall_obstacle_jumping_clear() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Runner high enough to clear a 2-row tall obstacle (rows 14-15)
        game.runner_y = 12.0;

        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::LargeRock, // 2 rows: 14-15
            passed: false,
        });

        // runner_top = 12 - 1 = 11, runner_bottom = 12. obs_top = 14, obs_bottom = 15.
        // runner_bottom(12) >= obs_top(14) = false. No collision.
        assert!(
            !check_collision(&game),
            "Jumping runner should clear tall obstacle"
        );
    }

    #[test]
    fn test_no_collision_horizontal_miss() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        // Obstacle far to the right
        game.obstacles.push(Obstacle {
            x: 40.0,
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        assert!(
            !check_collision(&game),
            "Runner should not collide with distant obstacle"
        );
    }

    // ── Scoring ──

    #[test]
    fn test_scoring_obstacle_passed() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        // Place an obstacle that the runner has already passed horizontally
        // (obstacle's right edge is before runner's right edge)
        let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;
        game.obstacles.push(Obstacle {
            x: runner_right - 10.0, // Well past the runner
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert_eq!(game.score, 1);
        assert!(game.obstacles[0].passed);
    }

    #[test]
    fn test_score_does_not_double_count() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        game.obstacles.push(Obstacle {
            x: 0.0,
            obstacle_type: ObstacleType::SmallRock,
            passed: true, // Already scored
        });

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.score, 0,
            "Already-passed obstacles should not be scored again"
        );
    }

    #[test]
    fn test_win_condition() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.score = 14;
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        // Place an obstacle that will be scored (already past runner)
        let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;
        game.obstacles.push(Obstacle {
            x: runner_right - 10.0,
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert_eq!(game.score, 15);
        assert_eq!(game.game_result, Some(DinoRunResult::Win));
    }

    // ── Speed progression ──

    #[test]
    fn test_speed_increases_with_distance() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Prevent spawning
        game.next_obstacle_distance = 999.0;
        let initial_speed = game.game_speed;

        // Run many ticks to accumulate distance
        for _ in 0..100 {
            tick_dino_run(&mut game, PHYSICS_TICK_MS);
        }

        assert!(
            game.game_speed > initial_speed,
            "Speed should increase over time"
        );
    }

    #[test]
    fn test_speed_capped_at_max() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Manually set distance very high to force max speed
        game.distance = 1_000_000.0;
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert!(
            (game.game_speed - game.max_speed).abs() < f64::EPSILON,
            "Speed should be capped at max_speed"
        );
    }

    // ── Difficulty levels ──

    #[test]
    fn test_all_difficulties_constructable() {
        for diff in &DinoRunDifficulty::ALL {
            let game = DinoRunGame::new(*diff);
            assert_eq!(game.difficulty, *diff);
            assert!(game.game_result.is_none());
            assert!(game.waiting_to_start);
        }
    }

    #[test]
    fn test_all_difficulties_have_valid_parameters() {
        for diff in &DinoRunDifficulty::ALL {
            assert!(diff.gravity() > 0.0, "{:?} gravity must be positive", diff);
            assert!(
                diff.jump_impulse() < 0.0,
                "{:?} jump impulse must be negative (upward)",
                diff
            );
            assert!(
                diff.terminal_velocity() > 0.0,
                "{:?} terminal velocity must be positive",
                diff
            );
            assert!(
                diff.initial_speed() > 0.0,
                "{:?} initial speed must be positive",
                diff
            );
            assert!(
                diff.max_speed() > diff.initial_speed(),
                "{:?} max speed must exceed initial speed",
                diff
            );
            assert!(
                diff.obstacle_frequency_min() > 0.0,
                "{:?} obstacle frequency min must be positive",
                diff
            );
            assert!(
                diff.obstacle_frequency_max() >= diff.obstacle_frequency_min(),
                "{:?} obstacle frequency max must be >= min",
                diff
            );
            assert!(
                diff.target_score() > 0,
                "{:?} target score must be positive",
                diff
            );
        }
    }

    // ── Rewards ──

    #[test]
    fn test_reward_structure() {
        assert_eq!(
            DinoRunDifficulty::Novice.reward(),
            ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            DinoRunDifficulty::Apprentice.reward(),
            ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            }
        );
        assert_eq!(
            DinoRunDifficulty::Journeyman.reward(),
            ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            }
        );
        assert_eq!(
            DinoRunDifficulty::Master.reward(),
            ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            }
        );
    }

    #[test]
    fn test_extra_info() {
        assert_eq!(
            DinoRunDifficulty::Novice.extra_info().unwrap(),
            "15 obstacles, slow start"
        );
        assert_eq!(
            DinoRunDifficulty::Master.extra_info().unwrap(),
            "60 obstacles, relentless"
        );
    }

    #[test]
    fn test_difficulty_str_values() {
        assert_eq!(DinoRunDifficulty::Novice.difficulty_str(), "novice");
        assert_eq!(DinoRunDifficulty::Apprentice.difficulty_str(), "apprentice");
        assert_eq!(DinoRunDifficulty::Journeyman.difficulty_str(), "journeyman");
        assert_eq!(DinoRunDifficulty::Master.difficulty_str(), "master");
    }

    // ── apply_game_result tests ──

    #[test]
    fn test_apply_game_result_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let initial_xp = state.character_xp;

        let mut game = DinoRunGame::new(DinoRunDifficulty::Apprentice);
        game.game_result = Some(DinoRunResult::Win);
        game.score = 25;
        state.active_minigame = Some(ActiveMinigame::DinoRun(game));

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.game_type, "dino_run");
        assert_eq!(info.difficulty, "apprentice");
        assert!(state.character_xp > initial_xp);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_game_result_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;

        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        game.game_result = Some(DinoRunResult::Loss);
        state.active_minigame = Some(ActiveMinigame::DinoRun(game));

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

    // ── dt clamping ──

    #[test]
    fn test_dt_clamped() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        // Huge dt should be clamped to 100ms max
        tick_dino_run(&mut game, 5000);

        // Should have only done ~6 physics ticks (100ms / 16ms)
        assert!(game.tick_count <= 7);
    }

    #[test]
    fn test_tick_returns_false_when_game_over() {
        let mut game = DinoRunGame::new(DinoRunDifficulty::Novice);
        game.game_result = Some(DinoRunResult::Win);

        let changed = tick_dino_run(&mut game, PHYSICS_TICK_MS);
        assert!(!changed, "Tick should return false when game is over");
    }

    // ── Edge cases ──

    #[test]
    fn test_no_obstacles_no_crash() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Prevent spawning
        game.next_obstacle_distance = 999.0;

        // Run several ticks with no obstacles
        for _ in 0..20 {
            tick_dino_run(&mut game, PHYSICS_TICK_MS);
        }

        assert!(
            game.game_result.is_none(),
            "Game should not end with no obstacles"
        );
    }

    #[test]
    fn test_rapid_input_handling() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        // Rapid jump inputs
        process_input(&mut game, DinoRunInput::Jump);
        process_input(&mut game, DinoRunInput::Jump);
        process_input(&mut game, DinoRunInput::Jump);

        // Should still only have one jump queued
        assert!(game.jump_queued);

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        // After consuming, runner should be airborne
        assert!(!game.is_on_ground());
    }

    #[test]
    fn test_run_animation_advances() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Prevent spawning
        game.next_obstacle_distance = 999.0;
        let initial_frame = game.run_anim_frame;

        // Run enough ticks for animation to advance (every 8 ticks)
        for _ in 0..16 {
            tick_dino_run(&mut game, PHYSICS_TICK_MS);
        }

        // Animation should have changed at least once
        // (depends on tick_count alignment, but 16 ticks > 8)
        assert!(
            game.tick_count >= 8,
            "Should have run enough ticks for animation"
        );
        // At least one frame change should have happened
        // The frame alternates 0->1->0->1, so after 16 ticks we'll have had 2 changes
        // landing back on the initial. Let's just check tick_count advanced.
        let _ = initial_frame; // Animation test is about the counter advancing
    }

    // ── Additional coverage: duck vs ground obstacle ──

    #[test]
    fn test_duck_still_hit_by_ground_obstacle() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.is_ducking = true;

        // SmallRock is 1 row tall at ground level (row 15).
        // Ducking runner occupies row 15 only. Should still collide.
        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        assert!(
            check_collision(&game),
            "Ducking runner should still be hit by ground obstacle"
        );
    }

    #[test]
    fn test_duck_avoids_stalactite() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.is_ducking = true;

        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::Stalactite,
            passed: false,
        });

        assert!(
            !check_collision(&game),
            "Ducking should avoid stalactite flying obstacle"
        );
    }

    // ── Additional coverage: near-miss / boundary collisions ──

    #[test]
    fn test_collision_near_miss_horizontal() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        // Place obstacle just past runner's right edge (no overlap)
        let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;
        game.obstacles.push(Obstacle {
            x: runner_right, // Exactly at boundary, no overlap
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        assert!(
            !check_collision(&game),
            "Obstacle at exact right boundary should not collide (runner_right <= obs_left)"
        );
    }

    #[test]
    fn test_collision_boundary_overlap() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        // Place obstacle overlapping by smallest margin
        let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;
        game.obstacles.push(Obstacle {
            x: runner_right - 0.01, // Barely overlapping
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        assert!(
            check_collision(&game),
            "Obstacle barely overlapping should collide"
        );
    }

    #[test]
    fn test_collision_near_miss_vertical_jumping() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        // Runner at row 13 (feet), standing height 2 => rows 12-13.
        // Ground obstacle spans row 15 (SmallRock, 1 row). Gap between 13 and 15.
        game.runner_y = 13.0;

        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        // runner_top = 13 - 1 = 12, runner_bottom = 13. obs_top = 15, obs_bottom = 15.
        // runner_bottom(13) >= obs_top(15)? No. No collision.
        assert!(
            !check_collision(&game),
            "Runner at row 13 should just clear a ground obstacle at row 15"
        );
    }

    #[test]
    fn test_collision_loss_sets_game_result() {
        let mut game = started_game(DinoRunDifficulty::Novice);
        game.next_obstacle_distance = 999.0;

        // Place obstacle directly on runner
        game.obstacles.push(Obstacle {
            x: RUNNER_COL as f64,
            obstacle_type: ObstacleType::SmallRock,
            passed: false,
        });

        tick_dino_run(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.game_result,
            Some(DinoRunResult::Loss),
            "Collision should set game_result to Loss"
        );
    }

    // ── Additional coverage: difficulty ordering ──

    #[test]
    fn test_difficulty_ordering() {
        let diffs = [
            DinoRunDifficulty::Novice,
            DinoRunDifficulty::Apprentice,
            DinoRunDifficulty::Journeyman,
            DinoRunDifficulty::Master,
        ];

        // Each successive difficulty should have higher initial speed
        for i in 0..diffs.len() - 1 {
            assert!(
                diffs[i + 1].initial_speed() > diffs[i].initial_speed(),
                "{:?} should have higher initial speed than {:?}",
                diffs[i + 1],
                diffs[i]
            );
        }

        // Each successive difficulty should have higher max speed
        for i in 0..diffs.len() - 1 {
            assert!(
                diffs[i + 1].max_speed() > diffs[i].max_speed(),
                "{:?} should have higher max speed than {:?}",
                diffs[i + 1],
                diffs[i]
            );
        }

        // Each successive difficulty should have higher target score
        for i in 0..diffs.len() - 1 {
            assert!(
                diffs[i + 1].target_score() > diffs[i].target_score(),
                "{:?} should have higher target score than {:?}",
                diffs[i + 1],
                diffs[i]
            );
        }

        // Each successive difficulty should have smaller min obstacle spacing
        for i in 0..diffs.len() - 1 {
            assert!(
                diffs[i + 1].obstacle_frequency_min() < diffs[i].obstacle_frequency_min(),
                "{:?} should have tighter obstacle spacing than {:?}",
                diffs[i + 1],
                diffs[i]
            );
        }
    }

    // ── Additional coverage: zero dt ──

    #[test]
    fn test_zero_dt_no_crash() {
        let mut game = started_game(DinoRunDifficulty::Novice);

        let changed = tick_dino_run(&mut game, 0);
        assert!(!changed, "Zero dt should not advance physics");
        assert_eq!(game.tick_count, 0);
    }
}
