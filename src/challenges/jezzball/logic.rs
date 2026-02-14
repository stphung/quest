//! JezzBall logic: real-time physics, wall construction, and area capture.

use super::types::*;
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;
use std::collections::VecDeque;

/// Physics tick interval in milliseconds (~60 FPS).
const PHYSICS_TICK_MS: u64 = 16;
/// Ball collision radius (in cell units).
const BALL_RADIUS: f64 = 0.33;

/// UI-agnostic input actions for JezzBall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JezzballInput {
    Up,
    Down,
    Left,
    Right,
    Select,            // Space/Enter: start game or build wall
    ToggleOrientation, // X
    Forfeit,           // Esc
    Other,
}

/// Start a new JezzBall game.
pub fn start_jezzball_game(difficulty: JezzballDifficulty) -> ActiveMinigame {
    let mut rng = rand::rng();
    ActiveMinigame::Jezzball(JezzballGame::new(difficulty, &mut rng))
}

/// Process player input.
pub fn process_input(game: &mut JezzballGame, input: JezzballInput) {
    if game.game_result.is_some() {
        return;
    }

    if game.waiting_to_start {
        if matches!(input, JezzballInput::Select) {
            game.waiting_to_start = false;
        }
        return;
    }

    match input {
        JezzballInput::Up => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.active_wall.is_none() {
                move_cursor(game, 0, -1);
            }
        }
        JezzballInput::Down => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.active_wall.is_none() {
                move_cursor(game, 0, 1);
            }
        }
        JezzballInput::Left => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.active_wall.is_none() {
                move_cursor(game, -1, 0);
            }
        }
        JezzballInput::Right => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.active_wall.is_none() {
                move_cursor(game, 1, 0);
            }
        }
        JezzballInput::ToggleOrientation => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.active_wall.is_none() {
                game.orientation = game.orientation.toggle();
            }
        }
        JezzballInput::Select => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            } else if game.active_wall.is_none() {
                begin_wall(game);
            }
        }
        JezzballInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(JezzballResult::Loss);
            } else {
                game.forfeit_pending = true;
            }
        }
        JezzballInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            }
        }
    }
}

/// Advance JezzBall simulation.
///
/// `dt_ms` is milliseconds since last update.
pub fn tick_jezzball(game: &mut JezzballGame, dt_ms: u64) -> bool {
    if game.game_result.is_some() {
        return false;
    }

    if game.waiting_to_start || game.forfeit_pending {
        return false;
    }

    let dt_ms = dt_ms.min(100);
    game.accumulated_time_ms += dt_ms;

    let mut changed = false;

    while game.accumulated_time_ms >= PHYSICS_TICK_MS {
        game.accumulated_time_ms -= PHYSICS_TICK_MS;
        changed = true;

        if step_physics(game, PHYSICS_TICK_MS) {
            changed = true;
        }

        if game.game_result.is_some() {
            break;
        }
    }

    changed
}

fn step_physics(game: &mut JezzballGame, tick_ms: u64) -> bool {
    let mut changed = false;

    let dt_sec = tick_ms as f64 / 1000.0;

    for i in 0..game.balls.len() {
        step_ball(game, i, dt_sec);
        changed = true;
    }

    if ball_hits_active_wall(game) {
        game.game_result = Some(JezzballResult::Loss);
        return true;
    }

    if game.active_wall.is_some() {
        game.wall_accumulated_ms += tick_ms;

        while game.wall_accumulated_ms >= game.wall_step_ms {
            game.wall_accumulated_ms -= game.wall_step_ms;
            let completed = expand_active_wall(game);
            changed = true;

            if ball_hits_active_wall(game) {
                game.game_result = Some(JezzballResult::Loss);
                return true;
            }

            if completed {
                finalize_active_wall(game);
                changed = true;
                break;
            }
        }
    }

    game.tick_count += 1;
    changed
}

fn move_cursor(game: &mut JezzballGame, dx: i16, dy: i16) {
    game.cursor.x = (game.cursor.x + dx).clamp(0, game.grid_width - 1);
    game.cursor.y = (game.cursor.y + dy).clamp(0, game.grid_height - 1);
}

fn begin_wall(game: &mut JezzballGame) {
    if cell_blocked(game, game.cursor) {
        return;
    }

    game.active_wall = Some(ActiveWall {
        orientation: game.orientation,
        pivot: game.cursor,
        neg_extent: 0,
        pos_extent: 0,
        neg_done: false,
        pos_done: false,
    });

    game.wall_accumulated_ms = 0;

    if ball_hits_active_wall(game) {
        game.game_result = Some(JezzballResult::Loss);
    }
}

fn step_ball(game: &mut JezzballGame, index: usize, dt_sec: f64) {
    let mut ball = game.balls[index];

    let next_x = ball.x + ball.vx * dt_sec;
    if position_is_open(game, next_x, ball.y) {
        ball.x = next_x;
    } else {
        ball.vx = -ball.vx;
        let rebound_x = ball.x + ball.vx * dt_sec;
        if position_is_open(game, rebound_x, ball.y) {
            ball.x = rebound_x;
        }
    }

    let next_y = ball.y + ball.vy * dt_sec;
    if position_is_open(game, ball.x, next_y) {
        ball.y = next_y;
    } else {
        ball.vy = -ball.vy;
        let rebound_y = ball.y + ball.vy * dt_sec;
        if position_is_open(game, ball.x, rebound_y) {
            ball.y = rebound_y;
        }
    }

    game.balls[index] = ball;
}

fn position_is_open(game: &JezzballGame, x: f64, y: f64) -> bool {
    let min_x = (x - BALL_RADIUS).floor() as i16;
    let max_x = (x + BALL_RADIUS).floor() as i16;
    let min_y = (y - BALL_RADIUS).floor() as i16;
    let max_y = (y + BALL_RADIUS).floor() as i16;

    for cy in min_y..=max_y {
        for cx in min_x..=max_x {
            if cell_blocked(game, Position { x: cx, y: cy }) {
                return false;
            }
        }
    }

    true
}

fn cell_blocked(game: &JezzballGame, pos: Position) -> bool {
    if !in_bounds(game, pos) {
        return true;
    }
    game.blocked[pos.y as usize][pos.x as usize]
}

fn in_bounds(game: &JezzballGame, pos: Position) -> bool {
    pos.x >= 0 && pos.x < game.grid_width && pos.y >= 0 && pos.y < game.grid_height
}

fn ball_hits_active_wall(game: &JezzballGame) -> bool {
    let Some(wall) = game.active_wall else {
        return false;
    };

    for ball in &game.balls {
        let min_x = (ball.x - BALL_RADIUS).floor() as i16;
        let max_x = (ball.x + BALL_RADIUS).floor() as i16;
        let min_y = (ball.y - BALL_RADIUS).floor() as i16;
        let max_y = (ball.y + BALL_RADIUS).floor() as i16;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if wall_contains_cell(wall, Position { x, y }) {
                    return true;
                }
            }
        }
    }

    false
}

fn wall_contains_cell(wall: ActiveWall, pos: Position) -> bool {
    match wall.orientation {
        WallOrientation::Horizontal => {
            pos.y == wall.pivot.y
                && pos.x >= wall.pivot.x - wall.neg_extent
                && pos.x <= wall.pivot.x + wall.pos_extent
        }
        WallOrientation::Vertical => {
            pos.x == wall.pivot.x
                && pos.y >= wall.pivot.y - wall.neg_extent
                && pos.y <= wall.pivot.y + wall.pos_extent
        }
    }
}

fn expand_active_wall(game: &mut JezzballGame) -> bool {
    let Some(mut wall) = game.active_wall.take() else {
        return false;
    };

    match wall.orientation {
        WallOrientation::Horizontal => {
            if !wall.neg_done {
                let next = Position {
                    x: wall.pivot.x - wall.neg_extent - 1,
                    y: wall.pivot.y,
                };
                if cell_blocked(game, next) {
                    wall.neg_done = true;
                } else {
                    wall.neg_extent += 1;
                }
            }

            if !wall.pos_done {
                let next = Position {
                    x: wall.pivot.x + wall.pos_extent + 1,
                    y: wall.pivot.y,
                };
                if cell_blocked(game, next) {
                    wall.pos_done = true;
                } else {
                    wall.pos_extent += 1;
                }
            }
        }
        WallOrientation::Vertical => {
            if !wall.neg_done {
                let next = Position {
                    x: wall.pivot.x,
                    y: wall.pivot.y - wall.neg_extent - 1,
                };
                if cell_blocked(game, next) {
                    wall.neg_done = true;
                } else {
                    wall.neg_extent += 1;
                }
            }

            if !wall.pos_done {
                let next = Position {
                    x: wall.pivot.x,
                    y: wall.pivot.y + wall.pos_extent + 1,
                };
                if cell_blocked(game, next) {
                    wall.pos_done = true;
                } else {
                    wall.pos_extent += 1;
                }
            }
        }
    }

    let complete = wall.neg_done && wall.pos_done;
    game.active_wall = Some(wall);
    complete
}

fn finalize_active_wall(game: &mut JezzballGame) {
    let Some(wall) = game.active_wall.take() else {
        return;
    };

    mark_wall_cells(game, wall);
    capture_regions_without_balls(game);
    recalculate_captured_percent(game);

    if game.captured_percent >= game.target_percent as f64 {
        game.game_result = Some(JezzballResult::Win);
    }
}

fn mark_wall_cells(game: &mut JezzballGame, wall: ActiveWall) {
    match wall.orientation {
        WallOrientation::Horizontal => {
            for x in (wall.pivot.x - wall.neg_extent)..=(wall.pivot.x + wall.pos_extent) {
                let pos = Position { x, y: wall.pivot.y };
                if in_bounds(game, pos) {
                    game.blocked[pos.y as usize][pos.x as usize] = true;
                }
            }
        }
        WallOrientation::Vertical => {
            for y in (wall.pivot.y - wall.neg_extent)..=(wall.pivot.y + wall.pos_extent) {
                let pos = Position { x: wall.pivot.x, y };
                if in_bounds(game, pos) {
                    game.blocked[pos.y as usize][pos.x as usize] = true;
                }
            }
        }
    }
}

fn capture_regions_without_balls(game: &mut JezzballGame) {
    let mut visited = vec![vec![false; game.grid_width as usize]; game.grid_height as usize];
    let mut queue = VecDeque::new();

    for ball in &game.balls {
        let pos = Position {
            x: ball.x.floor() as i16,
            y: ball.y.floor() as i16,
        };

        if in_bounds(game, pos)
            && !game.blocked[pos.y as usize][pos.x as usize]
            && !visited[pos.y as usize][pos.x as usize]
        {
            visited[pos.y as usize][pos.x as usize] = true;
            queue.push_back(pos);
        }
    }

    while let Some(pos) = queue.pop_front() {
        let neighbors = [
            Position {
                x: pos.x + 1,
                y: pos.y,
            },
            Position {
                x: pos.x - 1,
                y: pos.y,
            },
            Position {
                x: pos.x,
                y: pos.y + 1,
            },
            Position {
                x: pos.x,
                y: pos.y - 1,
            },
        ];

        for next in neighbors {
            if !in_bounds(game, next) {
                continue;
            }
            if game.blocked[next.y as usize][next.x as usize] {
                continue;
            }
            if visited[next.y as usize][next.x as usize] {
                continue;
            }
            visited[next.y as usize][next.x as usize] = true;
            queue.push_back(next);
        }
    }

    for y in 0..game.grid_height {
        for x in 0..game.grid_width {
            if !game.blocked[y as usize][x as usize] && !visited[y as usize][x as usize] {
                game.blocked[y as usize][x as usize] = true;
            }
        }
    }
}

fn recalculate_captured_percent(game: &mut JezzballGame) {
    let open_cells = game.blocked.iter().flatten().filter(|&&cell| !cell).count() as f64;

    let total_cells = game.total_cells() as f64;
    game.captured_percent = ((total_cells - open_cells) / total_cells * 100.0).clamp(0.0, 100.0);
}

impl DifficultyInfo for JezzballDifficulty {
    fn name(&self) -> &'static str {
        JezzballDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            JezzballDifficulty::Novice => ChallengeReward {
                xp_percent: 25,
                ..Default::default()
            },
            JezzballDifficulty::Apprentice => ChallengeReward {
                xp_percent: 75,
                ..Default::default()
            },
            JezzballDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 100,
                ..Default::default()
            },
            JezzballDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 100,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!(
            "{} balls, {}% target",
            self.ball_count(),
            self.target_percent()
        ))
    }
}

/// Apply game result via shared challenge reward system.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, captured, target) = {
        if let Some(ActiveMinigame::Jezzball(ref game)) = state.active_minigame {
            (
                game.game_result,
                game.difficulty,
                game.captured_percent,
                game.target_percent,
            )
        } else {
            return None;
        }
    };

    let result = result?;
    let won = matches!(result, JezzballResult::Win);
    let reward = difficulty.reward();

    if won {
        state.combat_state.add_log_entry(
            format!(
                "▣ Arena secured: {:.0}% captured (target {}%).",
                captured, target
            ),
            false,
            true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!("▣ Containment failed at {:.0}% captured.", captured.floor()),
            false,
            true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "jezzball",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "▣",
            win_message: "Containment Breach conquered!",
            loss_message: "The arena remains uncontrolled.",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started_game(difficulty: JezzballDifficulty) -> JezzballGame {
        let mut rng = rand::rng();
        let mut game = JezzballGame::new(difficulty, &mut rng);
        game.waiting_to_start = false;
        game
    }

    #[test]
    fn test_waiting_to_start_blocks_input_and_ticks() {
        let mut rng = rand::rng();
        let mut game = JezzballGame::new(JezzballDifficulty::Novice, &mut rng);
        let cursor_before = game.cursor;
        let ball_before = game.balls[0];

        process_input(&mut game, JezzballInput::Right);
        assert_eq!(game.cursor, cursor_before);

        let changed = tick_jezzball(&mut game, 1000);
        assert!(!changed);
        assert_eq!(game.balls[0], ball_before);
    }

    #[test]
    fn test_select_starts_game() {
        let mut rng = rand::rng();
        let mut game = JezzballGame::new(JezzballDifficulty::Novice, &mut rng);
        assert!(game.waiting_to_start);

        process_input(&mut game, JezzballInput::Select);
        assert!(!game.waiting_to_start);
    }

    #[test]
    fn test_forfeit_flow() {
        let mut game = started_game(JezzballDifficulty::Novice);

        process_input(&mut game, JezzballInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        process_input(&mut game, JezzballInput::Forfeit);
        assert_eq!(game.game_result, Some(JezzballResult::Loss));
    }

    #[test]
    fn test_forfeit_canceled_by_other_input() {
        let mut game = started_game(JezzballDifficulty::Novice);

        process_input(&mut game, JezzballInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, JezzballInput::Other);
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_toggle_orientation() {
        let mut game = started_game(JezzballDifficulty::Novice);
        assert_eq!(game.orientation, WallOrientation::Vertical);

        process_input(&mut game, JezzballInput::ToggleOrientation);
        assert_eq!(game.orientation, WallOrientation::Horizontal);
    }

    #[test]
    fn test_cursor_moves_when_idle() {
        let mut game = started_game(JezzballDifficulty::Novice);
        let start = game.cursor;

        process_input(&mut game, JezzballInput::Right);
        process_input(&mut game, JezzballInput::Down);

        assert_eq!(game.cursor.x, start.x + 1);
        assert_eq!(game.cursor.y, start.y + 1);
    }

    #[test]
    fn test_select_starts_wall() {
        let mut game = started_game(JezzballDifficulty::Novice);
        assert!(game.active_wall.is_none());

        process_input(&mut game, JezzballInput::Select);

        assert!(game.active_wall.is_some());
        let wall = game.active_wall.unwrap();
        assert_eq!(wall.pivot, game.cursor);
        assert_eq!(wall.neg_extent, 0);
        assert_eq!(wall.pos_extent, 0);
    }

    #[test]
    fn test_start_wall_on_ball_cell_loses() {
        let mut game = started_game(JezzballDifficulty::Novice);
        game.cursor = Position {
            x: game.balls[0].x.floor() as i16,
            y: game.balls[0].y.floor() as i16,
        };

        process_input(&mut game, JezzballInput::Select);

        assert_eq!(game.game_result, Some(JezzballResult::Loss));
    }

    #[test]
    fn test_wall_completion_captures_area() {
        let mut game = started_game(JezzballDifficulty::Novice);

        // Make simulation deterministic and avoid accidental wall hits.
        game.balls = vec![Ball {
            x: 2.5,
            y: 2.5,
            vx: 0.0,
            vy: 0.0,
        }];
        game.cursor = Position {
            x: game.grid_width / 2,
            y: game.grid_height / 2,
        };
        game.orientation = WallOrientation::Vertical;

        process_input(&mut game, JezzballInput::Select);
        assert!(game.active_wall.is_some());

        for _ in 0..400 {
            tick_jezzball(&mut game, 100);
            if game.active_wall.is_none() || game.game_result.is_some() {
                break;
            }
        }

        assert!(game.active_wall.is_none());
        assert!(game.captured_percent > 10.0);
    }

    #[test]
    fn test_ball_bounces_off_boundaries() {
        let mut game = started_game(JezzballDifficulty::Novice);
        game.balls = vec![Ball {
            x: 0.4,
            y: 5.0,
            vx: -6.0,
            vy: 0.0,
        }];

        tick_jezzball(&mut game, 100);

        assert!(game.balls[0].vx > 0.0);
    }

    #[test]
    fn test_tick_clamps_large_dt() {
        let mut game = started_game(JezzballDifficulty::Novice);
        let before = game.tick_count;

        tick_jezzball(&mut game, 50_000);

        // Clamped to 100ms => at most 6 fixed ticks.
        assert!(game.tick_count - before <= 6);
    }

    #[test]
    fn test_apply_game_result_win_returns_info() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut game = started_game(JezzballDifficulty::Novice);
        game.game_result = Some(JezzballResult::Win);
        game.captured_percent = 62.0;
        state.active_minigame = Some(ActiveMinigame::Jezzball(game));

        let info = apply_game_result(&mut state);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.game_type, "jezzball");
        assert_eq!(info.difficulty, "novice");
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_game_result_loss_returns_none() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut game = started_game(JezzballDifficulty::Novice);
        game.game_result = Some(JezzballResult::Loss);
        game.captured_percent = 23.0;
        state.active_minigame = Some(ActiveMinigame::Jezzball(game));

        let info = apply_game_result(&mut state);

        assert!(info.is_none());
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_difficulty_rewards() {
        assert_eq!(
            JezzballDifficulty::Novice.reward(),
            ChallengeReward {
                xp_percent: 25,
                ..Default::default()
            }
        );
        assert_eq!(
            JezzballDifficulty::Journeyman.reward(),
            ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 100,
                ..Default::default()
            }
        );
        assert_eq!(
            JezzballDifficulty::Master.reward(),
            ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 100,
                ..Default::default()
            }
        );
    }
}
