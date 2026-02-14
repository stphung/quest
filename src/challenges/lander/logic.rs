//! Lunar Lander game logic: physics, input processing, collision detection,
//! win/loss conditions.

use super::types::*;
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;

/// UI-agnostic input actions for Lunar Lander.
///
/// Terminal environments only produce key-press events (no key-release), so
/// the input model uses impulse-based "On" actions. The tick function clears
/// the flags after each frame; terminal key-repeat provides the continuous
/// "hold" behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanderInput {
    ThrustOn,      // Space/Up pressed
    RotateLeftOn,  // Left arrow pressed
    RotateRightOn, // Right arrow pressed
    Forfeit,       // Esc
    Other,         // Any other key (cancels forfeit_pending)
}

/// Start a new lander game at the given difficulty.
pub fn start_lander_game(difficulty: LanderDifficulty) -> ActiveMinigame {
    let mut rng = rand::thread_rng();
    ActiveMinigame::Lander(Box::new(LanderGame::new(difficulty, &mut rng)))
}

/// Process player input.
pub fn process_input(game: &mut LanderGame, input: LanderInput) {
    if game.game_result.is_some() {
        return; // Game over -- any key dismisses (handled by input.rs)
    }

    // Waiting screen: thrust starts the game
    if game.waiting_to_start {
        if matches!(input, LanderInput::ThrustOn) {
            game.waiting_to_start = false;
        }
        return;
    }

    match input {
        LanderInput::ThrustOn => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else {
                game.thrusting = true;
                game.thrust_hold_ticks = INPUT_HOLD_TICKS;
            }
        }
        LanderInput::RotateLeftOn => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else {
                game.rotating_left = true;
                game.rotate_left_hold_ticks = INPUT_HOLD_TICKS;
            }
        }
        LanderInput::RotateRightOn => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else {
                game.rotating_right = true;
                game.rotate_right_hold_ticks = INPUT_HOLD_TICKS;
            }
        }
        LanderInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(LanderResult::Loss); // Confirm forfeit
            } else {
                game.forfeit_pending = true;
            }
        }
        LanderInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            }
        }
    }
}

/// Advance Lunar Lander physics. Called from the main game loop.
///
/// `dt_ms` is milliseconds since last call. Internally steps physics in
/// 16ms increments (~60 FPS). Returns true if the game state changed.
///
/// Input flags (`thrusting`, `rotating_left`, `rotating_right`) persist for
/// `INPUT_HOLD_TICKS` physics steps after each key press. This bridges the
/// gap between terminal key-repeat events (~500ms initial delay) so holding
/// a key feels continuous.
pub fn tick_lander(game: &mut LanderGame, dt_ms: u64) -> bool {
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

    // Step physics in fixed PHYSICS_TICK_MS increments
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
fn step_physics(game: &mut LanderGame) {
    game.tick_count += 1;

    // Handle rotation
    if game.rotating_left {
        game.angle -= ROTATION_SPEED;
    }
    if game.rotating_right {
        game.angle += ROTATION_SPEED;
    }
    // Clamp angle to prevent full rotation (~60 degrees each way)
    game.angle = game.angle.clamp(-1.05, 1.05);

    // Handle thrust
    if game.thrusting && game.fuel > 0.0 {
        // Thrust direction: angle=0 means upward thrust (countering gravity)
        // Thrust vector: (-sin(angle), -cos(angle)) where negative y is upward
        let thrust_x = -game.angle.sin() * THRUST_POWER;
        let thrust_y = -game.angle.cos() * THRUST_POWER;

        game.vx += thrust_x;
        game.vy += thrust_y;

        // Consume fuel
        game.fuel -= FUEL_BURN_RATE;
        if game.fuel < 0.0 {
            game.fuel = 0.0;
        }

        // Flame animation
        game.flame_timer = FLAME_ANIM_TICKS;
    }

    // Decrement flame animation timer
    if game.flame_timer > 0 {
        game.flame_timer -= 1;
    }

    // Decay hold timers -- clear input flags when timer expires
    if game.thrust_hold_ticks > 0 {
        game.thrust_hold_ticks -= 1;
        if game.thrust_hold_ticks == 0 {
            game.thrusting = false;
        }
    }
    if game.rotate_left_hold_ticks > 0 {
        game.rotate_left_hold_ticks -= 1;
        if game.rotate_left_hold_ticks == 0 {
            game.rotating_left = false;
        }
    }
    if game.rotate_right_hold_ticks > 0 {
        game.rotate_right_hold_ticks -= 1;
        if game.rotate_right_hold_ticks == 0 {
            game.rotating_right = false;
        }
    }

    // Apply gravity (positive = downward)
    game.vy += game.gravity;

    // Cap terminal velocity
    if game.vy > game.terminal_velocity {
        game.vy = game.terminal_velocity;
    }

    // Update position
    game.x += game.vx;
    game.y += game.vy;

    // Horizontal boundary wrapping (optional: or clamp)
    if game.x < 0.0 {
        game.x = 0.0;
        game.vx = 0.0;
    } else if game.x > GAME_WIDTH as f64 {
        game.x = GAME_WIDTH as f64;
        game.vx = 0.0;
    }

    // Check ceiling collision
    if game.y < 0.0 {
        game.y = 0.0;
        game.vy = 0.0;
    }

    // Check terrain collision
    check_collision(game);
}

/// Check if the lander has contacted the terrain and determine win/loss.
fn check_collision(game: &mut LanderGame) {
    let x_idx = (game.x.round() as usize).min(GAME_WIDTH as usize);
    let terrain_height = game.terrain.heights[x_idx];
    let terrain_y = GAME_HEIGHT as f64 - terrain_height;

    // No collision yet
    if game.y < terrain_y {
        return;
    }

    // Lander has touched or passed through the terrain
    // Check if on the landing pad
    let on_pad = x_idx >= game.terrain.pad_left && x_idx <= game.terrain.pad_right;

    if on_pad {
        // Check landing conditions
        let vy_ok = game.vy <= MAX_LANDING_VY;
        let vx_ok = game.vx.abs() <= MAX_LANDING_VX;
        let angle_ok = game.angle.abs() <= MAX_LANDING_ANGLE;

        if vy_ok && vx_ok && angle_ok {
            game.game_result = Some(LanderResult::Win);
        } else {
            game.game_result = Some(LanderResult::Loss);
        }
    } else {
        // Hit terrain outside pad = crash
        game.game_result = Some(LanderResult::Loss);
    }

    // Snap to terrain surface
    game.y = terrain_y;
    game.vx = 0.0;
    game.vy = 0.0;
}

impl DifficultyInfo for LanderDifficulty {
    fn name(&self) -> &'static str {
        LanderDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            LanderDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            LanderDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            LanderDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            LanderDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        match self {
            LanderDifficulty::Novice => Some("100% fuel, wide pad, low gravity".to_string()),
            LanderDifficulty::Apprentice => Some("80% fuel, medium pad".to_string()),
            LanderDifficulty::Journeyman => Some("60% fuel, small pad, jagged".to_string()),
            LanderDifficulty::Master => Some("40% fuel, tiny pad, high gravity".to_string()),
        }
    }
}

/// Apply game result using the shared challenge reward system.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, fuel, max_fuel) = {
        if let Some(ActiveMinigame::Lander(ref game)) = state.active_minigame {
            (game.game_result, game.difficulty, game.fuel, game.max_fuel)
        } else {
            return None;
        }
    };

    let result = result?;
    let won = matches!(result, LanderResult::Win);
    let reward = difficulty.reward();

    // Log score-specific message before the shared reward system logs its messages
    if won {
        let fuel_pct = (fuel / max_fuel * 100.0) as u32;
        state.combat_state.add_log_entry(
            format!("^ Lunar Descent complete! ({fuel_pct}% fuel remaining)",),
            false,
            true,
        );
    } else {
        state.combat_state.add_log_entry(
            "^ The lander crashed into the surface.".to_string(),
            false,
            true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "lander",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "^",
            win_message: "Lunar Descent complete!",
            loss_message: "The lander has been destroyed.",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a game that has already been started (skips the "Press Space" screen).
    fn started_game(difficulty: LanderDifficulty) -> LanderGame {
        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(difficulty, &mut rng);
        game.waiting_to_start = false;
        game
    }

    #[test]
    fn test_waiting_to_start_blocks_input() {
        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        assert!(game.waiting_to_start);

        // Non-thrust input is ignored
        process_input(&mut game, LanderInput::Other);
        assert!(game.waiting_to_start);

        // Rotation is ignored
        process_input(&mut game, LanderInput::RotateLeftOn);
        assert!(game.waiting_to_start);

        // Thrust starts the game
        process_input(&mut game, LanderInput::ThrustOn);
        assert!(!game.waiting_to_start);
    }

    #[test]
    fn test_waiting_to_start_blocks_physics() {
        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        let y_before = game.y;

        let changed = tick_lander(&mut game, 100);

        assert!(!changed);
        assert!((game.y - y_before).abs() < f64::EPSILON);
    }

    #[test]
    fn test_thrust_input() {
        let mut game = started_game(LanderDifficulty::Novice);
        assert!(!game.thrusting);

        process_input(&mut game, LanderInput::ThrustOn);
        assert!(game.thrusting);
        assert_eq!(game.thrust_hold_ticks, INPUT_HOLD_TICKS);

        // Thrust persists across ticks via hold timer
        tick_lander(&mut game, PHYSICS_TICK_MS);
        assert!(game.thrusting);
        assert!(game.thrust_hold_ticks < INPUT_HOLD_TICKS);
    }

    #[test]
    fn test_rotation_input() {
        let mut game = started_game(LanderDifficulty::Novice);
        assert!(!game.rotating_left);
        assert!(!game.rotating_right);

        process_input(&mut game, LanderInput::RotateLeftOn);
        assert!(game.rotating_left);
        assert_eq!(game.rotate_left_hold_ticks, INPUT_HOLD_TICKS);

        // Rotation persists across ticks via hold timer
        tick_lander(&mut game, PHYSICS_TICK_MS);
        assert!(game.rotating_left);

        process_input(&mut game, LanderInput::RotateRightOn);
        assert!(game.rotating_right);
        assert_eq!(game.rotate_right_hold_ticks, INPUT_HOLD_TICKS);
    }

    #[test]
    fn test_forfeit_flow() {
        let mut game = started_game(LanderDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, LanderInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Second Esc confirms
        process_input(&mut game, LanderInput::Forfeit);
        assert_eq!(game.game_result, Some(LanderResult::Loss));
    }

    #[test]
    fn test_forfeit_cancelled_by_other() {
        let mut game = started_game(LanderDifficulty::Novice);

        process_input(&mut game, LanderInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, LanderInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_forfeit_cancelled_by_thrust() {
        let mut game = started_game(LanderDifficulty::Novice);

        process_input(&mut game, LanderInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, LanderInput::ThrustOn);
        assert!(!game.forfeit_pending);
        // Thrust should NOT be active when cancelling forfeit
        assert!(!game.thrusting);
    }

    #[test]
    fn test_forfeit_cancelled_by_rotation() {
        let mut game = started_game(LanderDifficulty::Novice);

        process_input(&mut game, LanderInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, LanderInput::RotateLeftOn);
        assert!(!game.forfeit_pending);
        assert!(!game.rotating_left);
    }

    #[test]
    fn test_input_ignored_when_game_over() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.game_result = Some(LanderResult::Win);

        process_input(&mut game, LanderInput::ThrustOn);
        assert!(!game.thrusting);
    }

    #[test]
    fn test_gravity_pulls_lander_down() {
        let mut game = started_game(LanderDifficulty::Novice);
        let initial_y = game.y;
        let initial_vy = game.vy;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.vy > initial_vy,
            "Gravity should increase downward velocity"
        );
        assert!(game.y > initial_y, "Lander should fall due to gravity");
    }

    #[test]
    fn test_thrust_counters_gravity() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.angle = 0.0; // Upright

        // Activate thrust via input (sets hold timer)
        process_input(&mut game, LanderInput::ThrustOn);

        // Run several ticks -- hold timer keeps thrust active
        for _ in 0..10 {
            tick_lander(&mut game, PHYSICS_TICK_MS);
            if game.game_result.is_some() {
                break;
            }
        }

        // With upright thrust, vy should be negative (thrust overpowers gravity)
        // Novice gravity: 0.002, thrust: 0.02 → net per tick: 0.002 - 0.02 = -0.018
        assert!(
            game.vy < 0.0,
            "Full thrust should overpower gravity, vy={}",
            game.vy
        );
    }

    #[test]
    fn test_thrust_consumes_fuel() {
        let mut game = started_game(LanderDifficulty::Novice);
        let initial_fuel = game.fuel;
        game.thrusting = true;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert!(game.fuel < initial_fuel, "Thrusting should consume fuel");
    }

    #[test]
    fn test_no_thrust_without_fuel() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.fuel = 0.0;
        game.thrusting = true;
        game.thrust_hold_ticks = INPUT_HOLD_TICKS;
        game.vy = 0.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        // Should only have gravity effect, no thrust
        assert!(
            (game.vy - game.gravity).abs() < f64::EPSILON,
            "Without fuel, only gravity should apply, vy={}",
            game.vy
        );
    }

    #[test]
    fn test_rotation() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.rotating_left = true;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert!(game.angle < 0.0, "Left rotation should decrease angle");
    }

    #[test]
    fn test_rotation_clamped() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.angle = 1.0;
        game.rotating_right = true;
        game.rotate_right_hold_ticks = 100; // Enough to cover the loop

        for _ in 0..100 {
            tick_lander(&mut game, PHYSICS_TICK_MS);
            if game.game_result.is_some() {
                break;
            }
        }

        assert!(
            game.angle <= 1.06,
            "Angle should be clamped, got {}",
            game.angle
        );
    }

    #[test]
    fn test_terminal_velocity() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.vy = 100.0; // Absurdly high

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert!(
            game.vy <= game.terminal_velocity,
            "Velocity should be capped at terminal velocity"
        );
    }

    #[test]
    fn test_horizontal_boundary_clamping() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.x = -1.0;
        game.vx = -1.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert!(game.x >= 0.0, "X should be clamped at left boundary");
        assert!(
            (game.vx - 0.0).abs() < f64::EPSILON,
            "VX should be zeroed at boundary"
        );
    }

    #[test]
    fn test_ceiling_collision() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.y = 0.1;
        game.vy = -1.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert!(game.y >= 0.0, "Y should be clamped at ceiling");
        assert!(
            (game.vy - 0.0).abs() < f64::EPSILON,
            "VY should be zeroed at ceiling"
        );
    }

    #[test]
    fn test_crash_on_terrain() {
        let mut game = started_game(LanderDifficulty::Novice);
        // Position lander directly above terrain, far from pad
        let off_pad_x = if game.terrain.pad_left > 10 {
            1
        } else {
            game.terrain.pad_right + 5
        };
        let off_pad_x = off_pad_x.min(GAME_WIDTH as usize);
        game.x = off_pad_x as f64;
        let terrain_y = GAME_HEIGHT as f64 - game.terrain.heights[off_pad_x];
        game.y = terrain_y - 0.01; // Just above terrain
        game.vy = 0.1; // Moving down

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.game_result,
            Some(LanderResult::Loss),
            "Hitting terrain outside pad should crash"
        );
    }

    #[test]
    fn test_safe_landing_on_pad() {
        let mut game = started_game(LanderDifficulty::Novice);
        // Position lander just above pad center
        let pad_center = (game.terrain.pad_left + game.terrain.pad_right) / 2;
        game.x = pad_center as f64;
        let pad_y = GAME_HEIGHT as f64 - game.terrain.pad_height;
        game.y = pad_y - 0.01; // Just above pad
        game.vy = 0.02; // Slow descent (below MAX_LANDING_VY)
        game.vx = 0.0;
        game.angle = 0.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.game_result,
            Some(LanderResult::Win),
            "Safe landing on pad should win"
        );
    }

    #[test]
    fn test_fast_landing_on_pad_crashes() {
        let mut game = started_game(LanderDifficulty::Novice);
        let pad_center = (game.terrain.pad_left + game.terrain.pad_right) / 2;
        game.x = pad_center as f64;
        let pad_y = GAME_HEIGHT as f64 - game.terrain.pad_height;
        game.y = pad_y - 0.01;
        game.vy = 0.2; // Too fast (above MAX_LANDING_VY)
        game.vx = 0.0;
        game.angle = 0.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.game_result,
            Some(LanderResult::Loss),
            "Fast landing on pad should crash"
        );
    }

    #[test]
    fn test_tilted_landing_on_pad_crashes() {
        let mut game = started_game(LanderDifficulty::Novice);
        let pad_center = (game.terrain.pad_left + game.terrain.pad_right) / 2;
        game.x = pad_center as f64;
        let pad_y = GAME_HEIGHT as f64 - game.terrain.pad_height;
        game.y = pad_y - 0.01;
        game.vy = 0.02; // Safe speed
        game.vx = 0.0;
        game.angle = 0.5; // Too tilted (above MAX_LANDING_ANGLE)

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.game_result,
            Some(LanderResult::Loss),
            "Tilted landing on pad should crash"
        );
    }

    #[test]
    fn test_horizontal_drift_landing_crashes() {
        let mut game = started_game(LanderDifficulty::Novice);
        let pad_center = (game.terrain.pad_left + game.terrain.pad_right) / 2;
        game.x = pad_center as f64;
        let pad_y = GAME_HEIGHT as f64 - game.terrain.pad_height;
        game.y = pad_y - 0.01;
        game.vy = 0.02; // Safe vertical speed
        game.vx = 0.1; // Too much horizontal drift (above MAX_LANDING_VX)
        game.angle = 0.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        assert_eq!(
            game.game_result,
            Some(LanderResult::Loss),
            "Landing with too much horizontal drift should crash"
        );
    }

    #[test]
    fn test_physics_paused_during_forfeit() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.forfeit_pending = true;
        let y_before = game.y;

        let changed = tick_lander(&mut game, 100);

        assert!(!changed);
        assert!((game.y - y_before).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_returns_false_when_game_over() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.game_result = Some(LanderResult::Win);

        let changed = tick_lander(&mut game, PHYSICS_TICK_MS);
        assert!(!changed, "Tick should return false when game is over");
    }

    #[test]
    fn test_dt_clamped() {
        let mut game = started_game(LanderDifficulty::Novice);
        let y_before = game.y;

        // Huge dt should be clamped to 100ms max
        tick_lander(&mut game, 5000);

        // Should have only done ~6 physics ticks (100ms / 16ms)
        assert!(game.tick_count <= 7);
        // Lander should have moved, but not exploded
        assert!((game.y - y_before).abs() < 5.0);
    }

    #[test]
    fn test_flame_timer_on_thrust() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.thrusting = true;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        // After thrusting, flame_timer should be set (then decremented once)
        assert_eq!(
            game.flame_timer,
            FLAME_ANIM_TICKS - 1,
            "Flame timer should be set and decremented by one tick"
        );
    }

    #[test]
    fn test_angled_thrust() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.thrusting = true;
        game.angle = 0.5; // Tilted right
        game.vy = 0.0;
        game.vx = 0.0;

        tick_lander(&mut game, PHYSICS_TICK_MS);

        // With rightward tilt, thrust should push left (negative vx) and up (negative vy before gravity)
        // thrust_x = -sin(0.5) * 0.012 ≈ -0.00575 (leftward)
        // thrust_y = -cos(0.5) * 0.012 ≈ -0.01053 (upward)
        // vy after = thrust_y + gravity = -0.01053 + 0.003 = -0.00753
        assert!(
            game.vx < 0.0,
            "Rightward tilt should thrust leftward, vx={}",
            game.vx
        );
    }

    #[test]
    fn test_reward_structure() {
        assert_eq!(
            LanderDifficulty::Novice.reward(),
            ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            }
        );
        assert_eq!(
            LanderDifficulty::Apprentice.reward(),
            ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            }
        );
        assert_eq!(
            LanderDifficulty::Journeyman.reward(),
            ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            }
        );
        assert_eq!(
            LanderDifficulty::Master.reward(),
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
            LanderDifficulty::Novice.extra_info().unwrap(),
            "100% fuel, wide pad, low gravity"
        );
        assert_eq!(
            LanderDifficulty::Master.extra_info().unwrap(),
            "40% fuel, tiny pad, high gravity"
        );
    }

    #[test]
    fn test_difficulty_str_values() {
        assert_eq!(LanderDifficulty::Novice.difficulty_str(), "novice");
        assert_eq!(LanderDifficulty::Apprentice.difficulty_str(), "apprentice");
        assert_eq!(LanderDifficulty::Journeyman.difficulty_str(), "journeyman");
        assert_eq!(LanderDifficulty::Master.difficulty_str(), "master");
    }

    #[test]
    fn test_apply_game_result_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let initial_xp = state.character_xp;

        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Apprentice, &mut rng);
        game.game_result = Some(LanderResult::Win);
        game.fuel = 50.0;
        state.active_minigame = Some(ActiveMinigame::Lander(Box::new(game)));

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.game_type, "lander");
        assert_eq!(info.difficulty, "apprentice");
        assert!(state.character_xp > initial_xp);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_game_result_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;

        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        game.game_result = Some(LanderResult::Loss);
        state.active_minigame = Some(ActiveMinigame::Lander(Box::new(game)));

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
    fn test_fuel_cannot_go_negative() {
        let mut game = started_game(LanderDifficulty::Novice);
        game.fuel = 0.1; // Almost empty
        game.thrusting = true;
        game.thrust_hold_ticks = INPUT_HOLD_TICKS;

        // Run several ticks -- hold timer keeps thrust active
        for _ in 0..10 {
            tick_lander(&mut game, PHYSICS_TICK_MS);
            if game.game_result.is_some() {
                break;
            }
        }

        assert!(game.fuel >= 0.0, "Fuel should never go negative");
    }

    #[test]
    fn test_all_difficulties_have_valid_parameters() {
        for diff in &LanderDifficulty::ALL {
            assert!(diff.gravity() > 0.0, "{:?} gravity must be positive", diff);
            assert!(
                diff.starting_fuel() > 0.0,
                "{:?} starting fuel must be positive",
                diff
            );
            assert!(
                diff.pad_width() > 0,
                "{:?} pad width must be positive",
                diff
            );
            assert!(
                diff.terrain_roughness() > 0.0,
                "{:?} roughness must be positive",
                diff
            );
            assert!(
                diff.terminal_velocity() > 0.0,
                "{:?} terminal velocity must be positive",
                diff
            );
        }
    }
}
