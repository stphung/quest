//! Snake ("Serpent's Path") game UI rendering.
//!
//! Uses half-block pixel rendering for smooth visuals. Each game cell maps to
//! a colored pixel; pairs of vertical pixels are packed into one terminal row
//! using the `▀` (upper half block) character with fg=top, bg=bottom colors.
//! Each cell is 2 terminal columns wide to correct for character aspect ratio.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::snake::types::{SnakeGame, SnakeResult};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

// ── Border characters ────────────────────────────────────────────────
const BORDER_H: char = '\u{2500}'; // ─
const BORDER_V: char = '\u{2502}'; // │
const BORDER_TL: char = '\u{250C}'; // ┌
const BORDER_TR: char = '\u{2510}'; // ┐
const BORDER_BL: char = '\u{2514}'; // └
const BORDER_BR: char = '\u{2518}'; // ┘
const HALF_TOP: char = '\u{2580}'; // ▀ — fg fills top half, bg fills bottom half
const FULL_BLOCK: char = '\u{2588}'; // █

// ── Snake gradient colors ────────────────────────────────────────────
const HEAD_COLOR: Color = Color::Rgb(100, 255, 100);
const BODY_BRIGHT: (f64, f64, f64) = (50.0, 220.0, 50.0);
const BODY_DIM: (f64, f64, f64) = (20.0, 80.0, 20.0);
const EMPTY_BG: Color = Color::Rgb(12, 12, 18);

/// Render the Snake game scene.
pub fn render_snake_scene(frame: &mut Frame, area: Rect, game: &SnakeGame) {
    if game.game_result.is_some() {
        render_snake_game_over(frame, area, game);
        return;
    }

    let layout = create_game_layout(frame, area, " Serpent's Path ", Color::LightGreen, 20, 16);

    render_play_field(frame, layout.content, game);

    if game.waiting_to_start {
        render_start_prompt(frame, layout.content);
    }

    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Calculate interpolated RGB color for a snake body segment.
fn body_color(index: usize, snake_len: usize) -> Color {
    let t = index as f64 / (snake_len - 1).max(1) as f64;
    let r = (BODY_BRIGHT.0 * (1.0 - t) + BODY_DIM.0 * t) as u8;
    let g = (BODY_BRIGHT.1 * (1.0 - t) + BODY_DIM.1 * t) as u8;
    let b = (BODY_BRIGHT.2 * (1.0 - t) + BODY_DIM.2 * t) as u8;
    Color::Rgb(r, g, b)
}

/// Render the play field using half-block pixel rendering.
///
/// Each game cell becomes a colored "pixel". Two vertical pixels are packed
/// into one terminal row via `▀` (fg = top pixel, bg = bottom pixel).
/// Each cell is 2 terminal columns wide to correct for character aspect ratio.
fn render_play_field(frame: &mut Frame, area: Rect, game: &SnakeGame) {
    if area.height < 3 || area.width < 5 {
        return;
    }

    let grid_w = game.grid_width as usize;
    let grid_h = game.grid_height as usize;
    let border_color = Color::Rgb(80, 80, 80);

    // ── Build color grid (game coordinates) ─────────────────────
    let mut pixels: Vec<Vec<Option<Color>>> = vec![vec![None; grid_w]; grid_h];

    // Food (pulsing orange-red)
    let fx = game.food.x as usize;
    let fy = game.food.y as usize;
    if fx < grid_w && fy < grid_h {
        let pulse = ((game.tick_count % 20) as f64 / 20.0 * std::f64::consts::PI * 2.0).sin();
        let food_g = (80.0 + pulse * 30.0) as u8;
        let food_b = (40.0 + pulse * 20.0) as u8;
        pixels[fy][fx] = Some(Color::Rgb(255, food_g, food_b));
    }

    // Snake (head bright, body gradient)
    let snake_len = game.snake.len();
    for (i, seg) in game.snake.iter().enumerate() {
        let sx = seg.x as usize;
        let sy = seg.y as usize;
        if sx < grid_w && sy < grid_h {
            pixels[sy][sx] = Some(if i == 0 {
                HEAD_COLOR
            } else {
                body_color(i, snake_len)
            });
        }
    }

    // ── Layout dimensions ───────────────────────────────────────
    let content_rows = grid_h.div_ceil(2); // 2 game rows per terminal row
    // 1 terminal column per game cell (half-blocks correct the vertical aspect)
    let render_w = ((grid_w + 2) as u16).min(area.width);
    let inner_w = render_w as usize - 2; // chars between left/right border

    let x_off = area.x + (area.width.saturating_sub(render_w)) / 2;
    let y_off = area.y;

    // ── Row 0: Top border with score ────────────────────────────
    {
        let score_val = format!("{}/{}", game.score, game.target_score);
        let label = "Score: ";
        let score_full_len = label.len() + score_val.len();
        let pad_before = inner_w.saturating_sub(score_full_len + 1);
        let pad_after = inner_w.saturating_sub(pad_before + score_full_len);

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            BORDER_TL.to_string(),
            Style::default().fg(border_color),
        ));
        if pad_before > 0 {
            let s: String = std::iter::repeat_n(BORDER_H, pad_before).collect();
            spans.push(Span::styled(s, Style::default().fg(border_color)));
        }
        spans.push(Span::styled(label, Style::default().fg(border_color)));
        spans.push(Span::styled(score_val, Style::default().fg(Color::White)));
        if pad_after > 0 {
            let s: String = std::iter::repeat_n(BORDER_H, pad_after).collect();
            spans.push(Span::styled(s, Style::default().fg(border_color)));
        }
        spans.push(Span::styled(
            BORDER_TR.to_string(),
            Style::default().fg(border_color),
        ));

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(line, Rect::new(x_off, y_off, render_w, 1));
    }

    // ── Game content rows (half-block pixel rendering) ──────────
    let empty_row: Vec<Option<Color>> = vec![None; grid_w];
    for term_row in 0..content_rows {
        let top_gy = term_row * 2;
        let bot_gy = term_row * 2 + 1;
        let top_row = if top_gy < grid_h { &pixels[top_gy] } else { &empty_row };
        let bot_row = if bot_gy < grid_h { &pixels[bot_gy] } else { &empty_row };

        let mut spans: Vec<Span> = Vec::new();

        // Left border
        spans.push(Span::styled(
            BORDER_V.to_string(),
            Style::default().fg(border_color),
        ));

        // Game cells — batch consecutive cells with the same style
        let mut cur_fg = Color::Reset;
        let mut cur_bg = Color::Reset;
        let mut cur_text = String::new();

        for (&top_c, &bot_c) in top_row.iter().zip(bot_row.iter()) {
            // ▀ uses fg for top half, bg for bottom half
            let fg = top_c.unwrap_or(EMPTY_BG);
            let bg = bot_c.unwrap_or(EMPTY_BG);

            if fg != cur_fg || bg != cur_bg {
                if !cur_text.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut cur_text),
                        Style::default().fg(cur_fg).bg(cur_bg),
                    ));
                }
                cur_fg = fg;
                cur_bg = bg;
            }
            // 1 char per game cell (half-blocks correct vertical aspect ratio)
            cur_text.push(HALF_TOP);
        }
        if !cur_text.is_empty() {
            spans.push(Span::styled(
                cur_text,
                Style::default().fg(cur_fg).bg(cur_bg),
            ));
        }

        // Right border
        spans.push(Span::styled(
            BORDER_V.to_string(),
            Style::default().fg(border_color),
        ));

        let row_y = y_off + 1 + term_row as u16;
        if row_y < area.y + area.height {
            let line = Paragraph::new(Line::from(spans));
            frame.render_widget(line, Rect::new(x_off, row_y, render_w, 1));
        }
    }

    // ── Bottom border ───────────────────────────────────────────
    {
        let bot_y = y_off + 1 + content_rows as u16;
        if bot_y < area.y + area.height {
            let mut s = String::new();
            s.push(BORDER_BL);
            for _ in 0..inner_w {
                s.push(BORDER_H);
            }
            s.push(BORDER_BR);
            let line = Paragraph::new(Line::from(Span::styled(
                s,
                Style::default().fg(border_color),
            )));
            frame.render_widget(line, Rect::new(x_off, bot_y, render_w, 1));
        }
    }
}

/// Render the status bar below the play field.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &SnakeGame) {
    if game.waiting_to_start {
        render_status_bar(
            frame,
            area,
            "Ready",
            Color::LightGreen,
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
        "Slither!",
        Color::Green,
        &[("[Arrows]", "Move"), ("[Esc]", "Forfeit")],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &SnakeGame) {
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
            Span::styled("Grid: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}x{}", game.grid_width, game.grid_height),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Speed: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}ms", game.move_interval_ms),
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
            Span::styled(
                format!(" {FULL_BLOCK} "),
                Style::default().fg(HEAD_COLOR),
            ),
            Span::styled("Head", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {FULL_BLOCK} "),
                Style::default().fg(Color::Rgb(
                    BODY_BRIGHT.0 as u8,
                    BODY_BRIGHT.1 as u8,
                    BODY_BRIGHT.2 as u8,
                )),
            ),
            Span::styled("Body", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {FULL_BLOCK} "),
                Style::default().fg(Color::Rgb(255, 80, 40)),
            ),
            Span::styled("Food", Style::default().fg(Color::DarkGray)),
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
fn render_snake_game_over(frame: &mut Frame, area: Rect, game: &SnakeGame) {
    let result = game.game_result.as_ref().unwrap();

    let (result_type, title, message, reward) = match result {
        SnakeResult::Win => {
            let reward_text = game.difficulty.reward().description();
            (
                GameResultType::Win,
                ":: SERPENT'S PATH CONQUERED! ::",
                format!(
                    "You guided the serpent to victory! {}/{} food consumed.",
                    game.score, game.target_score
                ),
                reward_text,
            )
        }
        SnakeResult::Loss => {
            let message = if game.forfeit_pending || game.score == 0 {
                "You abandoned the Serpent's Path.".to_string()
            } else {
                format!(
                    "The serpent falls after {} food. The path claims another.",
                    game.score
                )
            };
            (
                GameResultType::Loss,
                "SERPENT FALLS",
                message,
                "No penalty incurred.".to_string(),
            )
        }
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward);
}
