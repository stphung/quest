//! Flappy Bird ("Skyward Gauntlet") game UI rendering.
//!
//! Uses half-block characters (▀▄) for sub-cell vertical resolution,
//! effectively doubling the smoothness of bird and pipe movement.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::flappy::types::{
    FlappyBirdGame, FlappyBirdResult, BIRD_COL, GAME_HEIGHT, GAME_WIDTH, MAX_LIVES, PIPE_WIDTH,
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

fn lerp_channel(start: u8, end: u8, t: f64) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (start as f64 + (end as f64 - start as f64) * t).round() as u8
}

fn lerp_rgb(start: (u8, u8, u8), end: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        lerp_channel(start.0, end.0, t),
        lerp_channel(start.1, end.1, t),
        lerp_channel(start.2, end.2, t),
    )
}

fn sky_color(row: usize, ground_row: usize, dusk: f64) -> Color {
    let height_t = if ground_row == 0 {
        0.0
    } else {
        row as f64 / ground_row as f64
    };

    let top_day = (120, 196, 255);
    let mid_day = (155, 214, 255);
    let low_day = (208, 236, 255);

    let top_dusk = (30, 48, 92);
    let mid_dusk = (62, 76, 128);
    let low_dusk = (122, 106, 140);

    let top = lerp_rgb(top_day, top_dusk, dusk);
    let mid = lerp_rgb(mid_day, mid_dusk, dusk);
    let low = lerp_rgb(low_day, low_dusk, dusk);

    let rgb = if height_t < 0.45 {
        lerp_rgb(top, mid, height_t / 0.45)
    } else {
        lerp_rgb(mid, low, (height_t - 0.45) / 0.55)
    };
    Color::Rgb(rgb.0, rgb.1, rgb.2)
}

fn star_hash(row: usize, col: usize) -> u32 {
    let seed = (row as u32)
        .wrapping_mul(1664525)
        .wrapping_add((col as u32).wrapping_mul(1013904223));
    seed ^ (seed >> 13)
}

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
    let ground_row = (render_height - 1) as usize;
    let dusk = ((game.tick_count as f64 * 0.004).sin() * 0.5 + 0.5).powf(1.15);

    // ── Sky gradient + celestial body ───────────────────────────────
    for (row, row_cells) in buffer.iter_mut().enumerate().take(ground_row) {
        let bg = sky_color(row, ground_row, dusk);
        for cell in row_cells.iter_mut().take(render_width as usize) {
            cell.bg = bg;
        }
    }

    let orb_col = ((render_width as f64 * 0.74) + (game.tick_count as f64 * 0.025).sin() * 4.0)
        .round() as i32;
    let orb_row =
        ((ground_row as f64 * 0.20) + (game.tick_count as f64 * 0.02).sin()).round() as i32;
    let (orb_char, orb_color) = if dusk < 0.56 {
        ('●', Color::Rgb(255, 228, 148))
    } else {
        ('◑', Color::Rgb(232, 237, 255))
    };
    for (dx, dy, ch, fg) in [
        (0, 0, orb_char, orb_color),
        (-1, 0, '·', Color::Rgb(240, 226, 176)),
        (1, 0, '·', Color::Rgb(240, 226, 176)),
        (0, -1, '·', Color::Rgb(234, 220, 172)),
        (0, 1, '·', Color::Rgb(234, 220, 172)),
    ] {
        let col = orb_col + dx;
        let row = orb_row + dy;
        if row >= 0 && (row as usize) < ground_row && col >= 0 && col < render_width as i32 {
            buffer[row as usize][col as usize] = Cell {
                ch,
                fg,
                bg: buffer[row as usize][col as usize].bg,
            };
        }
    }

    // ── Stars + layered clouds ──────────────────────────────────────
    if dusk > 0.25 {
        let twinkle_tick = (game.tick_count / 6) as usize;
        for (row, row_cells) in buffer
            .iter_mut()
            .enumerate()
            .take(ground_row.saturating_sub(2))
        {
            for (col, cell) in row_cells.iter_mut().enumerate().take(render_width as usize) {
                if star_hash(row, col).is_multiple_of(97) && cell.ch == ' ' {
                    let bright =
                        star_hash(row + twinkle_tick, col + twinkle_tick).is_multiple_of(3);
                    *cell = Cell {
                        ch: if bright { '*' } else { '.' },
                        fg: if bright {
                            Color::Rgb(245, 245, 255)
                        } else {
                            Color::Rgb(185, 190, 230)
                        },
                        bg: cell.bg,
                    };
                }
            }
        }
    }

    for &(base_x, y, speed, pattern, shade) in &[
        (4.0_f64, 2.0_f64, 0.018_f64, "~~", 140u8),
        (20.0, 3.0, 0.022, "~~~", 135),
        (36.0, 1.0, 0.028, "~~~~", 128),
        (14.0, 5.0, 0.012, "~ ~", 122),
        (43.0, 4.0, 0.017, "~~", 126),
    ] {
        let drift = (game.tick_count as f64 * speed) % render_width as f64;
        let cx = ((base_x - drift).rem_euclid(render_width as f64)) as usize;
        let ry = (y * y_scale).round() as usize;
        if ry < ground_row {
            for (i, ch) in pattern.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = (cx + i) % render_width as usize;
                if buffer[ry][col].ch == ' ' {
                    let tint = lerp_channel(shade, 205, 1.0 - dusk);
                    buffer[ry][col] = Cell {
                        ch,
                        fg: Color::Rgb(tint, tint, tint.saturating_add(6)),
                        bg: buffer[ry][col].bg,
                    };
                }
            }
        }
    }

    // ── Distant cliffs for depth ────────────────────────────────────
    let horizon = ground_row.saturating_sub(1);
    let mut col = 0usize;
    while col < render_width as usize {
        let far_h = (1.0 + ((col as f64 * 0.27 + game.tick_count as f64 * 0.006).sin() + 1.0) * 1.4)
            .round() as i32;
        let near_h = (1.0
            + ((col as f64 * 0.16 + game.tick_count as f64 * 0.011 + 1.7).sin() + 1.0) * 1.8)
            .round() as i32;

        for (height, ch, color) in [
            (
                far_h,
                '░',
                Color::Rgb(
                    lerp_channel(86, 62, dusk),
                    lerp_channel(112, 88, dusk),
                    lerp_channel(130, 118, dusk),
                ),
            ),
            (
                near_h,
                '▒',
                Color::Rgb(
                    lerp_channel(72, 54, dusk),
                    lerp_channel(96, 72, dusk),
                    lerp_channel(110, 98, dusk),
                ),
            ),
        ] {
            let top = horizon as i32 - height;
            for row in top.max(0)..=horizon as i32 {
                let row = row as usize;
                if row < ground_row && buffer[row][col].ch == ' ' {
                    buffer[row][col] = Cell {
                        ch,
                        fg: color,
                        bg: buffer[row][col].bg,
                    };
                }
            }
        }
        col += 1;
    }

    // ── Ground layers ───────────────────────────────────────────────
    for (i, cell) in buffer[ground_row]
        .iter_mut()
        .enumerate()
        .take(render_width as usize)
    {
        *cell = Cell {
            ch: if (i + (game.tick_count as usize / 2)).is_multiple_of(4) {
                GROUND_SUB
            } else {
                GROUND_CHAR
            },
            fg: Color::Rgb(98, 74, 52),
            bg: Color::Rgb(52, 38, 28),
        };
    }
    if ground_row > 0 {
        for (i, cell) in buffer[ground_row - 1]
            .iter_mut()
            .enumerate()
            .take(render_width as usize)
        {
            let bg = cell.bg;
            *cell = Cell {
                ch: if (i + game.tick_count as usize).is_multiple_of(6) {
                    '┬'
                } else {
                    GROUND_SUB
                },
                fg: Color::Rgb(86, 120, 64),
                bg,
            };
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

                let backdrop = buffer_row[col].bg;
                let (ch, fg, bg) = if row + 1 == gap_top_row && gap_top_row > 0 {
                    // Cap at bottom of top pipe
                    if is_edge {
                        (
                            PIPE_CAP_TOP,
                            Color::Rgb(54, 126, 54),
                            Color::Rgb(36, 92, 38),
                        )
                    } else {
                        (
                            PIPE_CAP_TOP,
                            Color::Rgb(82, 188, 82),
                            Color::Rgb(46, 120, 48),
                        )
                    }
                } else if row == gap_bottom_row && gap_bottom_row < ground_row {
                    // Cap at top of bottom pipe
                    if is_edge {
                        (
                            PIPE_CAP_BOT,
                            Color::Rgb(54, 126, 54),
                            Color::Rgb(36, 92, 38),
                        )
                    } else {
                        (
                            PIPE_CAP_BOT,
                            Color::Rgb(82, 188, 82),
                            Color::Rgb(46, 120, 48),
                        )
                    }
                } else if is_edge {
                    // Pipe edges (slightly darker)
                    (
                        if dx < 0 { PIPE_EDGE_L } else { PIPE_EDGE_R },
                        Color::Rgb(48, 130, 50),
                        Color::Rgb(34, 88, 36),
                    )
                } else {
                    // Pipe body
                    let texture = (row + col + (game.tick_count as usize / 3)).is_multiple_of(6);
                    let highlight_band = dx.abs() <= (pipe_half_w / 2).max(1);
                    (
                        if texture { '▓' } else { PIPE_BODY },
                        if highlight_band {
                            Color::Rgb(92, 206, 90)
                        } else {
                            Color::Rgb(68, 168, 68)
                        },
                        if highlight_band {
                            Color::Rgb(54, 132, 56)
                        } else {
                            Color::Rgb(44, 112, 46)
                        },
                    )
                };

                buffer_row[col] = Cell {
                    ch,
                    fg,
                    bg: if matches!(bg, Color::Reset) {
                        backdrop
                    } else {
                        bg
                    },
                };
            }
        }
    }

    // ── Bird ───────────────────────────────────────────────────────
    let bird_y_scaled = game.bird_y * y_scale;
    let bird_row = bird_y_scaled.round() as i32;
    let bird_col = (BIRD_COL as f64 * x_scale).round() as i32;

    // Show flap visuals immediately when queued (don't wait for physics tick)
    let is_flapping = game.flap_timer > 0 || game.flap_queued;
    let rising_fast = game.bird_velocity < -0.35;
    let falling_fast = game.bird_velocity > 0.5;

    let (wing, body, beak) = if is_flapping {
        ('╱', '◉', '>')
    } else if rising_fast {
        ('╲', '◉', '>')
    } else if falling_fast {
        ('╱', '●', '>')
    } else {
        ('─', '●', '>')
    };

    let bird_color = if is_flapping {
        Color::Rgb(255, 242, 124)
    } else if rising_fast {
        Color::Rgb(255, 224, 102)
    } else {
        Color::Rgb(248, 204, 72)
    };

    if bird_row >= 0 && bird_row < (render_height as i32 - 1) {
        let row = bird_row as usize;
        let shadow_row = (bird_row + 1).min(render_height as i32 - 1) as usize;

        for (idx, trail_col) in [bird_col - 2, bird_col - 3].iter().enumerate() {
            if *trail_col >= 0 && *trail_col < render_width as i32 {
                let col = *trail_col as usize;
                if buffer[row][col].ch == ' ' {
                    buffer[row][col] = Cell {
                        ch: if idx == 0 { '·' } else { '.' },
                        fg: Color::Rgb(220, 196, 98),
                        bg: buffer[row][col].bg,
                    };
                }
            }
        }

        if bird_col >= 0 && bird_col < render_width as i32 {
            let col = bird_col as usize;
            if buffer[shadow_row][col].ch == ' ' {
                buffer[shadow_row][col] = Cell {
                    ch: '.',
                    fg: Color::Rgb(70, 74, 82),
                    bg: buffer[shadow_row][col].bg,
                };
            }
        }

        // Wing/tail
        if bird_col >= 1 && (bird_col - 1) < render_width as i32 {
            let col = (bird_col - 1) as usize;
            buffer[row][col] = Cell {
                ch: wing,
                fg: bird_color,
                bg: buffer[row][col].bg,
            };
        }
        // Body
        if bird_col >= 0 && bird_col < render_width as i32 {
            buffer[row][bird_col as usize] = Cell {
                ch: body,
                fg: bird_color,
                bg: buffer[row][bird_col as usize].bg,
            };
        }
        // Beak
        let beak_col = bird_col + 1;
        if beak_col >= 0 && beak_col < render_width as i32 {
            buffer[row][beak_col as usize] = Cell {
                ch: beak,
                fg: Color::Rgb(255, 170, 68),
                bg: buffer[row][beak_col as usize].bg,
            };
        }
    }

    // ── Lives display (top-left) ────────────────────────────────────
    {
        let lives_str: String = (0..MAX_LIVES)
            .map(|i| {
                if i < game.lives {
                    '\u{2665}'
                } else {
                    '\u{2661}'
                }
            })
            .collect();
        for (i, ch) in lives_str.chars().enumerate() {
            let col = 1 + i;
            if col < render_width as usize {
                buffer[0][col] = Cell {
                    ch,
                    fg: if ch == '\u{2665}' {
                        Color::Rgb(255, 90, 90)
                    } else {
                        Color::Rgb(100, 60, 60)
                    },
                    bg: Color::Rgb(18, 26, 44),
                };
            }
        }
        // Background for lives plate
        if render_width > 0 {
            buffer[0][0] = Cell {
                ch: ' ',
                fg: Color::Reset,
                bg: Color::Rgb(18, 26, 44),
            };
        }
        let end = (1 + MAX_LIVES as usize + 1).min(render_width as usize);
        if end < render_width as usize {
            buffer[0][end - 1] = Cell {
                ch: ' ',
                fg: Color::Reset,
                bg: Color::Rgb(18, 26, 44),
            };
        }
    }

    // ── Score display (top-right) ───────────────────────────────────
    let score_text = format!("{}/{}", game.score, game.target_score);
    let label = "Score: ";
    let total_len = label.len() + score_text.len();
    let score_start = (render_width as usize).saturating_sub(total_len + 1);
    let plate_start = score_start.saturating_sub(1);
    let plate_end = (score_start + total_len + 1).min(render_width as usize);

    for row_cells in buffer.iter_mut().take(render_height.min(2) as usize) {
        for cell in row_cells.iter_mut().take(plate_end).skip(plate_start) {
            cell.bg = Color::Rgb(18, 26, 44);
        }
    }

    for (i, ch) in label.chars().enumerate() {
        let col = score_start + i;
        if col < render_width as usize {
            buffer[0][col] = Cell {
                ch,
                fg: Color::Rgb(150, 170, 192),
                bg: buffer[0][col].bg,
            };
        }
    }
    for (i, ch) in score_text.chars().enumerate() {
        let col = score_start + label.len() + i;
        if col < render_width as usize {
            buffer[0][col] = Cell {
                ch,
                fg: Color::White,
                bg: buffer[0][col].bg,
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
                fg: Color::Rgb(150, 170, 192),
                bg: buffer[1][bar_start - 1].bg,
            };
        }
        for i in 0..bar_width {
            let col = bar_start + i;
            if col < render_width as usize {
                if i < filled {
                    let progress_t = if bar_width <= 1 {
                        0.0
                    } else {
                        i as f64 / (bar_width - 1) as f64
                    };
                    let fill = lerp_rgb((90, 218, 255), (124, 255, 170), progress_t);
                    buffer[1][col] = Cell {
                        ch: '█',
                        fg: Color::Rgb(fill.0, fill.1, fill.2),
                        bg: buffer[1][col].bg,
                    };
                } else {
                    buffer[1][col] = Cell {
                        ch: '░',
                        fg: Color::Rgb(82, 96, 116),
                        bg: buffer[1][col].bg,
                    };
                }
            }
        }
        let bracket_col = bar_start + bar_width;
        if bracket_col < render_width as usize {
            buffer[1][bracket_col] = Cell {
                ch: ']',
                fg: Color::Rgb(150, 170, 192),
                bg: buffer[1][bracket_col].bg,
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
            Span::styled("Lives: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.lives, MAX_LIVES),
                Style::default()
                    .fg(if game.lives > 0 {
                        Color::Rgb(255, 90, 90)
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
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
            Span::styled(" ◉> ", Style::default().fg(Color::Yellow)),
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
