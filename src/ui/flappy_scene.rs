//! Flappy Bird ("Skyward Gauntlet") game UI rendering.
//!
//! Uses half-block characters (▀▄) for sub-cell vertical resolution,
//! effectively doubling the smoothness of bird and pipe movement.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::flappy::types::{
    FlappyBirdGame, FlappyBirdResult, BIRD_COL, GAME_HEIGHT, GAME_WIDTH, PIPE_WIDTH,
};
use crate::challenges::menu::DifficultyInfo;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

// ── Pipe rendering characters ───────────────────────────────────────
const PIPE_BODY: char = '█';
const PIPE_CAP_TOP: char = '▄'; // cap below top pipe section
const PIPE_CAP_BOT: char = '▀'; // cap above bottom pipe section
const PIPE_EDGE_L: char = '▐';
const PIPE_EDGE_R: char = '▌';

// ── Ground characters ───────────────────────────────────────────────
const GROUND_CHAR: char = '▓';
const GROUND_SUB: char = '░';

/// Render the Flappy Bird game scene.
pub fn render_flappy_scene(
    frame: &mut Frame,
    area: Rect,
    game: &FlappyBirdGame,
    ctx: &super::responsive::LayoutContext,
) {
    // Game over overlay takes priority
    if game.game_result.is_some() {
        render_flappy_game_over(frame, area, game);
        return;
    }

    const MIN_WIDTH: u16 = 30;
    const MIN_HEIGHT: u16 = 12;
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Skyward Gauntlet", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    // Create standardized layout
    let layout = create_game_layout(
        frame,
        area,
        " Skyward Gauntlet ",
        Color::LightCyan,
        15,
        18,
        ctx,
    );

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

/// Render the main play field: bird, pipes, ground, score, progress.
fn render_play_field(frame: &mut Frame, area: Rect, game: &FlappyBirdGame) {
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

    // ── Background: sparse clouds for depth ─────────────────────────
    // Deterministic clouds based on tick_count for gentle drift
    let cloud_offset = (game.tick_count as f64 * 0.03) % render_width as f64;
    for &(base_x, y, pattern) in &[
        (12.0_f64, 1u16, "~~"),
        (35.0, 2, "~~~"),
        (8.0, 4, "~"),
        (28.0, 3, "~~"),
        (45.0, 1, "~"),
    ] {
        let cx = ((base_x - cloud_offset).rem_euclid(render_width as f64)) as usize;
        let ry = (y as f64 * y_scale).round() as usize;
        if ry < render_height as usize - 1 {
            for (i, ch) in pattern.chars().enumerate() {
                let col = (cx + i) % render_width as usize;
                if buffer[ry][col].ch == ' ' {
                    buffer[ry][col] = Cell {
                        ch,
                        fg: Color::Rgb(60, 60, 80),
                        bg: Color::Reset,
                    };
                }
            }
        }
    }

    // ── Ground (last two rows for depth) ────────────────────────────
    let ground_row = (render_height - 1) as usize;
    for cell in buffer[ground_row].iter_mut().take(render_width as usize) {
        *cell = Cell {
            ch: GROUND_CHAR,
            fg: Color::Rgb(80, 60, 40),
            bg: Color::Rgb(40, 30, 20),
        };
    }
    if ground_row > 0 {
        // Sub-ground accent row
        for (i, cell) in buffer[ground_row - 1]
            .iter_mut()
            .enumerate()
            .take(render_width as usize)
        {
            if cell.ch == ' ' {
                // Sparse grass/dirt texture
                if i % 4 == 0 {
                    *cell = Cell {
                        ch: GROUND_SUB,
                        fg: Color::Rgb(60, 80, 40),
                        bg: Color::Reset,
                    };
                }
            }
        }
    }

    // ── Pipes ───────────────────────────────────────────────────────
    for pipe in &game.pipes {
        let pipe_center_x = (pipe.x * x_scale).round() as i32;
        let pipe_half_w = ((PIPE_WIDTH as f64 * x_scale) / 2.0).round().max(1.0) as i32;

        let gap_top = (pipe.gap_center as f64 - game.pipe_gap as f64 / 2.0) * y_scale;
        let gap_bottom = (pipe.gap_center as f64 + game.pipe_gap as f64 / 2.0) * y_scale;
        let gap_top_row = gap_top.floor() as usize;
        let gap_bottom_row = gap_bottom.ceil() as usize;

        for dx in -pipe_half_w..=pipe_half_w {
            let col = pipe_center_x + dx;
            if col < 0 || col >= render_width as i32 {
                continue;
            }
            let col = col as usize;
            let is_edge = dx == -pipe_half_w || dx == pipe_half_w;

            for (row, buffer_row) in buffer.iter_mut().enumerate().take(ground_row) {
                // Skip gap area
                if row >= gap_top_row && row < gap_bottom_row {
                    continue;
                }

                let (ch, fg, bg) = if row + 1 == gap_top_row && gap_top_row > 0 {
                    // Cap at bottom of top pipe
                    if is_edge {
                        (PIPE_CAP_TOP, Color::Rgb(40, 120, 40), Color::Reset)
                    } else {
                        (PIPE_CAP_TOP, Color::Rgb(60, 160, 60), Color::Reset)
                    }
                } else if row == gap_bottom_row && gap_bottom_row < ground_row {
                    // Cap at top of bottom pipe
                    if is_edge {
                        (PIPE_CAP_BOT, Color::Rgb(40, 120, 40), Color::Reset)
                    } else {
                        (PIPE_CAP_BOT, Color::Rgb(60, 160, 60), Color::Reset)
                    }
                } else if is_edge {
                    // Pipe edges (slightly darker)
                    (
                        if dx < 0 { PIPE_EDGE_L } else { PIPE_EDGE_R },
                        Color::Rgb(40, 120, 40),
                        Color::Rgb(30, 80, 30),
                    )
                } else {
                    // Pipe body
                    (PIPE_BODY, Color::Rgb(50, 150, 50), Color::Rgb(40, 120, 40))
                };

                buffer_row[col] = Cell { ch, fg, bg };
            }
        }
    }

    // ── Bird ───────────────────────────────────────────────────────
    let bird_y_scaled = game.bird_y * y_scale;
    let bird_row = bird_y_scaled.round() as i32;
    let bird_col = (BIRD_COL as f64 * x_scale).round() as i32;

    // Show flap visuals immediately when queued (don't wait for physics tick)
    let is_flapping = game.flap_timer > 0 || game.flap_queued;

    // Bird is 3 chars: body + beak
    let (body, beak) = if is_flapping {
        ('◇', '›') // flapping: diamond body + beak
    } else {
        ('◆', '›') // normal: filled diamond + beak
    };

    // Bird color pulses subtly on flap
    let bird_color = if is_flapping {
        Color::Rgb(255, 240, 100) // bright yellow flash on flap
    } else {
        Color::Yellow
    };

    if bird_row >= 0 && bird_row < (render_height as i32 - 1) {
        let row = bird_row as usize;
        // Wing/tail
        if bird_col >= 1 && (bird_col - 1) < render_width as i32 {
            let col = (bird_col - 1) as usize;
            let wing = if is_flapping { '/' } else { '-' };
            buffer[row][col] = Cell {
                ch: wing,
                fg: bird_color,
                bg: Color::Reset,
            };
        }
        // Body
        if bird_col >= 0 && bird_col < render_width as i32 {
            buffer[row][bird_col as usize] = Cell {
                ch: body,
                fg: bird_color,
                bg: Color::Reset,
            };
        }
        // Beak
        let beak_col = bird_col + 1;
        if beak_col >= 0 && beak_col < render_width as i32 {
            buffer[row][beak_col as usize] = Cell {
                ch: beak,
                fg: Color::Rgb(255, 160, 50), // orange beak
                bg: Color::Reset,
            };
        }
    }

    // ── Score display (top-right) ───────────────────────────────────
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

    // ── Progress bar (row 1, right-aligned) ─────────────────────────
    let bar_width = 12usize;
    let bar_start = (render_width as usize).saturating_sub(bar_width + 2);
    let filled = if game.target_score > 0 {
        ((game.score as f64 / game.target_score as f64) * bar_width as f64).round() as usize
    } else {
        0
    };

    if render_height > 1 {
        // Opening bracket
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
                        ch: '█',
                        fg: Color::LightCyan,
                        bg: Color::Reset,
                    };
                } else {
                    buffer[1][col] = Cell {
                        ch: '░',
                        fg: Color::Rgb(50, 50, 60),
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

    // ── Render buffer to terminal ───────────────────────────────────
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
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &FlappyBirdGame) {
    if game.waiting_to_start {
        render_status_bar(
            frame,
            area,
            "Ready",
            Color::LightCyan,
            &[("[Space]", "Start"), ("[Esc]", "Forfeit")],
        );
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Fly!",
        Color::Yellow,
        &[("[Space/Up]", "Flap"), ("[Esc]", "Forfeit")],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &FlappyBirdGame) {
    let inner = render_info_panel_frame(frame, area);

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
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
            Span::styled("Gap: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} rows", game.pipe_gap),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Legend:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" ◆› ", Style::default().fg(Color::Yellow)),
            Span::styled("Bird", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {PIPE_EDGE_L}{PIPE_BODY}{PIPE_EDGE_R} "),
                Style::default().fg(Color::Green),
            ),
            Span::styled("Pipe", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {GROUND_CHAR}{GROUND_CHAR}{GROUND_CHAR} "),
                Style::default().fg(Color::Rgb(80, 60, 40)),
            ),
            Span::styled("Ground", Style::default().fg(Color::DarkGray)),
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
    let prompt = "[ Press Space to Start ]";
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
fn render_flappy_game_over(frame: &mut Frame, area: Rect, game: &FlappyBirdGame) {
    let result = game.game_result.as_ref().unwrap();

    let (result_type, title, message, reward) = match result {
        FlappyBirdResult::Win => {
            let reward_text = game.difficulty.reward().description();
            (
                GameResultType::Win,
                ":: SKYWARD GAUNTLET CONQUERED! ::",
                format!(
                    "You navigated the gauntlet! {}/{} pipes cleared.",
                    game.score, game.target_score
                ),
                reward_text,
            )
        }
        FlappyBirdResult::Loss => {
            let message = if game.forfeit_pending || game.score == 0 {
                "You walked away from the Skyward Gauntlet.".to_string()
            } else {
                format!(
                    "Crashed after {} pipes. The gauntlet claims another.",
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
