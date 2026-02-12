//! Dino Run ("Gauntlet Run") game UI rendering.
//!
//! Uses a cell buffer approach (like flappy_scene.rs) for per-character
//! color control. The runner, obstacles, and ground are drawn into a
//! 2D grid and then stamped row-by-row as Paragraph widgets.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, GameResultType,
};
use crate::challenges::dino::types::{
    DinoRunGame, DinoRunResult, ObstacleType, FLYING_ROW, GAME_HEIGHT, GAME_WIDTH, GROUND_ROW,
    RUNNER_COL, RUNNER_DUCKING_HEIGHT, RUNNER_STANDING_HEIGHT, RUNNER_WIDTH,
};
use crate::challenges::menu::DifficultyInfo;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

// ── Ground rendering characters ─────────────────────────────────────
const GROUND_CHAR: char = '▓';
const GROUND_SUB: char = '░';

/// Render the Dino Run game scene.
pub fn render_dino_scene(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    // Game over overlay takes priority
    if game.game_result.is_some() {
        render_dino_game_over(frame, area, game);
        return;
    }

    // Create standardized layout
    let layout = create_game_layout(frame, area, " Gauntlet Run ", Color::LightYellow, 15, 18);

    // Render the play field
    render_play_field(frame, layout.content, game);

    // Overlay "Press Space to Start" centered on the play field
    if game.waiting_to_start {
        render_start_prompt(frame, layout.content);
    }

    // Render status bar
    render_status_bar_content(frame, layout.status_bar, game);

    // Render info panel
    render_info_panel(frame, layout.info_panel, game);
}

/// Cell in the render buffer with foreground and background colors.
#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: Color,
    bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}

/// Render the main play field: runner, obstacles, ground, score, progress.
fn render_play_field(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    if area.height < 2 || area.width < 10 {
        return;
    }

    let render_height = area.height.min(GAME_HEIGHT);
    let render_width = area.width.min(GAME_WIDTH);

    // Build cell buffer
    let mut buffer: Vec<Vec<Cell>> =
        vec![vec![Cell::default(); render_width as usize]; render_height as usize];

    let y_scale = render_height as f64 / GAME_HEIGHT as f64;
    let x_scale = render_width as f64 / GAME_WIDTH as f64;

    // ── Background: sparse dungeon atmosphere ─────────────────────────
    let drift_offset = (game.tick_count as f64 * 0.02) % render_width as f64;
    for &(base_x, y, pattern) in &[
        (10.0_f64, 2u16, ".."),
        (30.0, 1, "..."),
        (50.0, 3, "."),
        (22.0, 4, ".."),
    ] {
        let cx = ((base_x - drift_offset).rem_euclid(render_width as f64)) as usize;
        let ry = (y as f64 * y_scale).round() as usize;
        if ry < render_height as usize - 1 {
            for (i, ch) in pattern.chars().enumerate() {
                let col = (cx + i) % render_width as usize;
                if buffer[ry][col].ch == ' ' {
                    buffer[ry][col] = Cell {
                        ch,
                        fg: Color::Rgb(50, 45, 40),
                        bg: Color::Reset,
                    };
                }
            }
        }
    }

    // ── Ground (last two rows for depth) ──────────────────────────────
    let ground_row = (render_height - 1) as usize;
    for cell in buffer[ground_row].iter_mut().take(render_width as usize) {
        *cell = Cell {
            ch: GROUND_CHAR,
            fg: Color::Rgb(90, 70, 50),
            bg: Color::Rgb(50, 40, 30),
        };
    }
    if ground_row > 0 {
        for (i, cell) in buffer[ground_row - 1]
            .iter_mut()
            .enumerate()
            .take(render_width as usize)
        {
            if cell.ch == ' ' && i % 5 == 0 {
                *cell = Cell {
                    ch: GROUND_SUB,
                    fg: Color::Rgb(70, 55, 40),
                    bg: Color::Reset,
                };
            }
        }
    }

    // ── Obstacles ─────────────────────────────────────────────────────
    for obstacle in &game.obstacles {
        let obs_x = (obstacle.x * x_scale).round() as i32;
        let obs_w = (obstacle.obstacle_type.width() as f64 * x_scale)
            .ceil()
            .max(1.0) as i32;
        let obs_h = obstacle.obstacle_type.height();

        let is_flying = obstacle.obstacle_type.is_flying();

        let (ch, fg) = match obstacle.obstacle_type {
            ObstacleType::SmallRock => ('#', Color::Rgb(120, 100, 80)),
            ObstacleType::LargeRock => ('#', Color::Rgb(140, 110, 80)),
            ObstacleType::Cactus => ('|', Color::Rgb(60, 140, 60)),
            ObstacleType::DoubleCactus => ('|', Color::Rgb(50, 130, 50)),
            ObstacleType::Bat => ('V', Color::Rgb(160, 80, 160)),
            ObstacleType::Stalactite => ('V', Color::Rgb(130, 130, 150)),
        };

        for dx in 0..obs_w {
            let col = obs_x + dx;
            if col < 0 || col >= render_width as i32 {
                continue;
            }
            let col = col as usize;

            for dy in 0..obs_h {
                let row = if is_flying {
                    let base = (FLYING_ROW as f64 * y_scale).round() as i32;
                    base + dy as i32
                } else {
                    let base = (GROUND_ROW as f64 * y_scale).round() as i32;
                    base - (obs_h as i32 - 1 - dy as i32)
                };

                if row >= 0 && row < ground_row as i32 {
                    buffer[row as usize][col] = Cell {
                        ch,
                        fg,
                        bg: Color::Reset,
                    };
                }
            }
        }
    }

    // ── Runner ────────────────────────────────────────────────────────
    let runner_col = (RUNNER_COL as f64 * x_scale).round() as i32;
    let runner_foot_row = (game.runner_y * y_scale).round() as i32;
    let runner_h = if game.is_ducking {
        RUNNER_DUCKING_HEIGHT
    } else {
        RUNNER_STANDING_HEIGHT
    };
    let runner_w = (RUNNER_WIDTH as f64 * x_scale).ceil().max(1.0) as i32;

    let runner_color = Color::LightYellow;

    // Draw runner body
    for dy in 0..runner_h as i32 {
        let row = runner_foot_row - dy;
        if row < 0 || row >= ground_row as i32 {
            continue;
        }
        for dx in 0..runner_w {
            let col = runner_col + dx;
            if col >= 0 && col < render_width as i32 {
                let ch = if game.is_ducking {
                    // Ducking: flat runner
                    if dx == 0 {
                        '\u{2590}'
                    } else {
                        '\u{2588}'
                    } // ▐ █
                } else if dy == 0 {
                    // Bottom row (feet): alternating run animation
                    if game.run_anim_frame == 0 {
                        if dx == 0 {
                            '/'
                        } else {
                            ' '
                        }
                    } else if dx == 0 {
                        ' '
                    } else {
                        '\\'
                    }
                } else {
                    // Top row (head/body)
                    '\u{2588}' // █
                };

                if ch != ' ' {
                    buffer[row as usize][col as usize] = Cell {
                        ch,
                        fg: runner_color,
                        bg: Color::Reset,
                    };
                }
            }
        }
    }

    // ── Score display (top-right) ─────────────────────────────────────
    let score_text = format!("{}/{}", game.score, game.target_score);
    let label = "Score: ";
    let total_len = label.len() + score_text.len();
    let score_start = (render_width as usize).saturating_sub(total_len + 1);

    for (i, ch) in label.chars().enumerate() {
        let col = score_start + i;
        if col < render_width as usize {
            buffer[0][col] = Cell {
                ch,
                fg: Color::DarkGray,
                bg: Color::Reset,
            };
        }
    }
    for (i, ch) in score_text.chars().enumerate() {
        let col = score_start + label.len() + i;
        if col < render_width as usize {
            buffer[0][col] = Cell {
                ch,
                fg: Color::White,
                bg: Color::Reset,
            };
        }
    }

    // ── Progress bar (row 1, right-aligned) ───────────────────────────
    let bar_width = 12usize;
    let bar_start = (render_width as usize).saturating_sub(bar_width + 2);
    let filled = if game.target_score > 0 {
        ((game.score as f64 / game.target_score as f64) * bar_width as f64).round() as usize
    } else {
        0
    };

    if render_height > 1 {
        if bar_start > 0 {
            buffer[1][bar_start - 1] = Cell {
                ch: '[',
                fg: Color::DarkGray,
                bg: Color::Reset,
            };
        }
        for i in 0..bar_width {
            let col = bar_start + i;
            if col < render_width as usize {
                if i < filled {
                    buffer[1][col] = Cell {
                        ch: '\u{2588}',
                        fg: Color::LightYellow,
                        bg: Color::Reset,
                    };
                } else {
                    buffer[1][col] = Cell {
                        ch: '\u{2591}',
                        fg: Color::Rgb(50, 50, 40),
                        bg: Color::Reset,
                    };
                }
            }
        }
        let bracket_col = bar_start + bar_width;
        if bracket_col < render_width as usize {
            buffer[1][bracket_col] = Cell {
                ch: ']',
                fg: Color::DarkGray,
                bg: Color::Reset,
            };
        }
    }

    // ── Render buffer to terminal ─────────────────────────────────────
    let x_offset = area.x;
    let y_offset = area.y;

    for (row_idx, row_data) in buffer.iter().enumerate().take(render_height as usize) {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;
        let mut current_text = String::new();

        for &cell in row_data.iter() {
            if (cell.fg != current_fg || cell.bg != current_bg) && !current_text.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut current_text),
                    Style::default().fg(current_fg).bg(current_bg),
                ));
            }
            current_fg = cell.fg;
            current_bg = cell.bg;
            current_text.push(cell.ch);
        }
        if !current_text.is_empty() {
            spans.push(Span::styled(
                current_text,
                Style::default().fg(current_fg).bg(current_bg),
            ));
        }

        let line = Paragraph::new(Line::from(spans));
        let row_area = Rect::new(x_offset, y_offset + row_idx as u16, render_width, 1);
        if row_area.y < area.y + area.height {
            frame.render_widget(line, row_area);
        }
    }
}

/// Render the status bar below the play field.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    if game.waiting_to_start {
        render_status_bar(
            frame,
            area,
            "Ready",
            Color::LightYellow,
            &[("[Space/Up]", "Start"), ("[Esc]", "Forfeit")],
        );
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let duck_hint = if game.is_ducking { "Stand" } else { "Duck" };
    render_status_bar(
        frame,
        area,
        "Run!",
        Color::LightYellow,
        &[
            ("[Space/Up]", "Jump"),
            ("[Down]", duck_hint),
            ("[Esc]", "Forfeit"),
        ],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    let inner = render_info_panel_frame(frame, area);

    let speed_pct = if game.max_speed > game.initial_speed {
        ((game.game_speed - game.initial_speed) / (game.max_speed - game.initial_speed) * 100.0)
            .round() as u32
    } else {
        0
    };

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.difficulty.name(),
                Style::default().fg(Color::LightYellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.score, game.target_score),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Speed: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}%", speed_pct), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Legend:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" \u{2588} ", Style::default().fg(Color::LightYellow)),
            Span::styled("Runner", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" # ", Style::default().fg(Color::Rgb(120, 100, 80))),
            Span::styled("Rock", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" | ", Style::default().fg(Color::Rgb(60, 140, 60))),
            Span::styled("Cactus", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" V ", Style::default().fg(Color::Rgb(160, 80, 160))),
            Span::styled("Flying", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

/// Render the "Press Space to Start" prompt centered on the play field.
fn render_start_prompt(frame: &mut Frame, area: Rect) {
    if area.height < 5 || area.width < 20 {
        return;
    }

    let center_y = area.y + area.height / 2;
    let prompt = "[ Press Space/Up to Start ]";
    let x = area.x + area.width.saturating_sub(prompt.len() as u16) / 2;

    let line = Paragraph::new(Line::from(vec![Span::styled(
        prompt,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    let prompt_area = Rect::new(x, center_y, prompt.len() as u16, 1);
    if prompt_area.y < area.y + area.height {
        frame.render_widget(line, prompt_area);
    }
}

/// Render the game over overlay.
fn render_dino_game_over(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    let result = game.game_result.as_ref().unwrap();

    let (result_type, title, message, reward) = match result {
        DinoRunResult::Win => {
            let reward_text = game.difficulty.reward().description();
            (
                GameResultType::Win,
                ":: GAUNTLET RUN CONQUERED! ::",
                format!(
                    "You survived the gauntlet! {}/{} obstacles cleared.",
                    game.score, game.target_score
                ),
                reward_text,
            )
        }
        DinoRunResult::Loss => {
            let message = if game.forfeit_pending || game.score == 0 {
                "You walked away from the Gauntlet Run.".to_string()
            } else {
                format!(
                    "Stumbled after {} obstacles. The gauntlet claims another.",
                    game.score
                )
            };
            (
                GameResultType::Loss,
                "GAUNTLET FAILED",
                message,
                "No penalty incurred.".to_string(),
            )
        }
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward);
}
