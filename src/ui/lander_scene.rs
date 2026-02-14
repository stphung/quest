//! Lunar Lander ("Lunar Descent") game UI rendering.
//!
//! Renders terrain, lander sprite with rotation, thrust flame,
//! HUD instruments, and game over overlay using a cell buffer approach.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::lander::types::{
    LanderAngle, LanderGame, LanderResult, GAME_HEIGHT, GAME_WIDTH,
};
use crate::challenges::menu::DifficultyInfo;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the Lunar Lander game scene.
pub fn render_lander_scene(
    frame: &mut Frame,
    area: Rect,
    game: &LanderGame,
    ctx: &super::responsive::LayoutContext,
) {
    // Game over overlay takes priority
    if game.game_result.is_some() {
        render_lander_game_over(frame, area, game);
        return;
    }

    const MIN_WIDTH: u16 = 30;
    const MIN_HEIGHT: u16 = 12;
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Lunar Descent", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    // Create standardized layout
    let layout = create_game_layout(
        frame,
        area,
        " Lunar Descent ",
        Color::LightBlue,
        15,
        20,
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

/// Cell in the render buffer.
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

/// Render the main play field: terrain, lander, flame, HUD.
#[allow(clippy::needless_range_loop)]
fn render_play_field(frame: &mut Frame, area: Rect, game: &LanderGame) {
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

    // -- Stars (sparse background) --
    // Deterministic star positions based on coordinates
    for row in 0..(render_height as usize).saturating_sub(4) {
        for col in 0..render_width as usize {
            let hash = (row * 137 + col * 251 + 97) % 200;
            if hash == 0 {
                buffer[row][col] = Cell {
                    ch: '.',
                    fg: Color::DarkGray,
                    bg: Color::Reset,
                };
            } else if hash == 1 {
                buffer[row][col] = Cell {
                    ch: '*',
                    fg: Color::Rgb(80, 80, 100),
                    bg: Color::Reset,
                };
            }
        }
    }

    // -- Terrain --
    for col in 0..render_width as usize {
        // Map render column to game x coordinate
        let game_x = (col as f64 / x_scale).round() as usize;
        let game_x = game_x.min(GAME_WIDTH as usize);

        let terrain_height = game.terrain.heights[game_x];
        let terrain_screen_y = GAME_HEIGHT as f64 - terrain_height;
        let render_terrain_y = (terrain_screen_y * y_scale).round() as usize;

        let on_pad = game_x >= game.terrain.pad_left && game_x <= game.terrain.pad_right;

        for row in render_terrain_y..render_height as usize {
            if row >= buffer.len() {
                break;
            }
            if row == render_terrain_y {
                // Terrain surface
                if on_pad {
                    buffer[row][col] = Cell {
                        ch: '=',
                        fg: Color::Green,
                        bg: Color::Reset,
                    };
                } else {
                    buffer[row][col] = Cell {
                        ch: '^',
                        fg: Color::Rgb(140, 120, 100),
                        bg: Color::Reset,
                    };
                }
            } else {
                // Terrain body
                let depth_char = if (row + col) % 3 == 0 { '.' } else { ' ' };
                buffer[row][col] = Cell {
                    ch: depth_char,
                    fg: Color::Rgb(60, 50, 40),
                    bg: Color::Rgb(30, 25, 20),
                };
            }
        }
    }

    // -- Pad markers (arrows pointing to pad edges) --
    let pad_left_col = (game.terrain.pad_left as f64 * x_scale).round() as usize;
    let pad_right_col = (game.terrain.pad_right as f64 * x_scale).round() as usize;
    let pad_screen_y = GAME_HEIGHT as f64 - game.terrain.pad_height;
    let pad_render_y = (pad_screen_y * y_scale).round() as usize;

    if pad_render_y > 0 && pad_render_y - 1 < render_height as usize {
        let marker_row = pad_render_y - 1;
        if pad_left_col < render_width as usize {
            buffer[marker_row][pad_left_col] = Cell {
                ch: '[',
                fg: Color::Green,
                bg: Color::Reset,
            };
        }
        if pad_right_col < render_width as usize {
            buffer[marker_row][pad_right_col] = Cell {
                ch: ']',
                fg: Color::Green,
                bg: Color::Reset,
            };
        }
    }

    // -- Thrust flame (rendered behind lander) --
    let lander_render_x = (game.x * x_scale).round() as i32;
    let lander_render_y = (game.y * y_scale).round() as i32;

    if game.flame_timer > 0 || (game.thrusting && game.fuel > 0.0) {
        let angle = game.sprite_angle();
        let (flame_chars, flame_offsets) = flame_sprite(angle, game.flame_timer);

        for (ch, (dx, dy)) in flame_chars.iter().zip(flame_offsets.iter()) {
            let fx = lander_render_x + dx;
            let fy = lander_render_y + dy;
            if fx >= 0 && fx < render_width as i32 && fy >= 0 && fy < render_height as i32 {
                let color = if game.flame_timer.is_multiple_of(2) {
                    Color::Yellow
                } else {
                    Color::LightRed
                };
                buffer[fy as usize][fx as usize] = Cell {
                    ch: *ch,
                    fg: color,
                    bg: Color::Reset,
                };
            }
        }
    }

    // -- Lander sprite --
    let sprite = lander_sprite(game.sprite_angle());
    for (ch, (dx, dy)) in sprite.iter() {
        let sx = lander_render_x + dx;
        let sy = lander_render_y + dy;
        if sx >= 0 && sx < render_width as i32 && sy >= 0 && sy < render_height as i32 {
            buffer[sy as usize][sx as usize] = Cell {
                ch: *ch,
                fg: Color::White,
                bg: Color::Reset,
            };
        }
    }

    // -- HUD: fuel bar (top-left) --
    let fuel_pct = if game.max_fuel > 0.0 {
        (game.fuel / game.max_fuel).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fuel_label = "FUEL ";
    let bar_width = 10usize;
    let filled = (fuel_pct * bar_width as f64).round() as usize;

    if render_height > 1 {
        // Label
        for (i, ch) in fuel_label.chars().enumerate() {
            if i < render_width as usize {
                buffer[0][i] = Cell {
                    ch,
                    fg: Color::DarkGray,
                    bg: Color::Reset,
                };
            }
        }
        // Bar
        let bar_start = fuel_label.len();
        if bar_start < render_width as usize {
            buffer[0][bar_start] = Cell {
                ch: '[',
                fg: Color::DarkGray,
                bg: Color::Reset,
            };
        }
        for i in 0..bar_width {
            let col = bar_start + 1 + i;
            if col < render_width as usize {
                let fuel_color = if fuel_pct < 0.2 {
                    Color::Red
                } else if fuel_pct < 0.5 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                buffer[0][col] = Cell {
                    ch: if i < filled { '|' } else { ' ' },
                    fg: fuel_color,
                    bg: Color::Reset,
                };
            }
        }
        let close_col = bar_start + 1 + bar_width;
        if close_col < render_width as usize {
            buffer[0][close_col] = Cell {
                ch: ']',
                fg: Color::DarkGray,
                bg: Color::Reset,
            };
        }
        // Percentage
        let pct_text = format!(" {}%", (fuel_pct * 100.0).round() as u32);
        let pct_start = close_col + 1;
        for (i, ch) in pct_text.chars().enumerate() {
            let col = pct_start + i;
            if col < render_width as usize {
                buffer[0][col] = Cell {
                    ch,
                    fg: Color::White,
                    bg: Color::Reset,
                };
            }
        }
    }

    // -- HUD: altitude & velocity (top-right) --
    let alt = game.altitude();
    let alt_text = format!("ALT:{:.1}", alt);
    let alt_start = (render_width as usize).saturating_sub(alt_text.len() + 1);
    if render_height > 0 {
        for (i, ch) in alt_text.chars().enumerate() {
            let col = alt_start + i;
            if col < render_width as usize {
                let color = if game.over_pad() {
                    Color::Green
                } else {
                    Color::White
                };
                buffer[0][col] = Cell {
                    ch,
                    fg: color,
                    bg: Color::Reset,
                };
            }
        }
    }

    // Vertical velocity (row 1 right)
    if render_height > 1 {
        let vy_text = format!("VY:{:+.3}", game.vy);
        let vy_start = (render_width as usize).saturating_sub(vy_text.len() + 1);
        let vy_color = if game.vy.abs() <= 0.08 {
            Color::Green
        } else if game.vy.abs() <= 0.15 {
            Color::Yellow
        } else {
            Color::Red
        };
        for (i, ch) in vy_text.chars().enumerate() {
            let col = vy_start + i;
            if col < render_width as usize {
                buffer[1][col] = Cell {
                    ch,
                    fg: vy_color,
                    bg: Color::Reset,
                };
            }
        }
    }

    // Horizontal velocity (row 2 right)
    if render_height > 2 {
        let vx_text = format!("VX:{:+.3}", game.vx);
        let vx_start = (render_width as usize).saturating_sub(vx_text.len() + 1);
        let vx_color = if game.vx.abs() <= 0.04 {
            Color::Green
        } else if game.vx.abs() <= 0.08 {
            Color::Yellow
        } else {
            Color::Red
        };
        for (i, ch) in vx_text.chars().enumerate() {
            let col = vx_start + i;
            if col < render_width as usize {
                buffer[2][col] = Cell {
                    ch,
                    fg: vx_color,
                    bg: Color::Reset,
                };
            }
        }
    }

    // -- Render buffer to terminal --
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

/// Get lander sprite characters and offsets for a given angle.
/// Returns Vec of (char, (dx, dy)) where dx/dy are offsets from lander center.
fn lander_sprite(angle: LanderAngle) -> Vec<(char, (i32, i32))> {
    match angle {
        LanderAngle::Straight => vec![
            ('^', (0, -1)), // top: nose
            ('|', (0, 0)),  // middle: body
            ('/', (-1, 1)), // left leg
            ('\\', (1, 1)), // right leg
        ],
        LanderAngle::Left => vec![
            ('/', (-1, -1)), // nose tilted left
            ('|', (0, 0)),   // body
            ('/', (-1, 1)),  // left leg
            ('_', (1, 1)),   // right leg flat
        ],
        LanderAngle::HardLeft => vec![
            ('/', (-1, -1)), // nose tilted hard left
            ('-', (0, 0)),   // body horizontal
            ('/', (-1, 1)),  // left leg
            ('_', (1, 1)),   // right leg
        ],
        LanderAngle::Right => vec![
            ('\\', (1, -1)), // nose tilted right
            ('|', (0, 0)),   // body
            ('_', (-1, 1)),  // left leg flat
            ('\\', (1, 1)),  // right leg
        ],
        LanderAngle::HardRight => vec![
            ('\\', (1, -1)), // nose tilted hard right
            ('-', (0, 0)),   // body horizontal
            ('_', (-1, 1)),  // left leg
            ('\\', (1, 1)),  // right leg
        ],
    }
}

/// Get flame sprite characters and offsets for a given angle.
/// Returns (chars, offsets) for the thrust flame below the lander.
fn flame_sprite(angle: LanderAngle, timer: u32) -> (Vec<char>, Vec<(i32, i32)>) {
    let long = !timer.is_multiple_of(3); // Vary flame length

    match angle {
        LanderAngle::Straight => {
            let mut chars = vec!['*'];
            let mut offsets = vec![(0, 2)];
            if long {
                chars.push('.');
                offsets.push((0, 3));
            }
            (chars, offsets)
        }
        LanderAngle::Left | LanderAngle::HardLeft => {
            let mut chars = vec!['*'];
            let mut offsets = vec![(1, 2)];
            if long {
                chars.push('.');
                offsets.push((2, 3));
            }
            (chars, offsets)
        }
        LanderAngle::Right | LanderAngle::HardRight => {
            let mut chars = vec!['*'];
            let mut offsets = vec![(-1, 2)];
            if long {
                chars.push('.');
                offsets.push((-2, 3));
            }
            (chars, offsets)
        }
    }
}

/// Render the status bar below the play field.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &LanderGame) {
    if game.waiting_to_start {
        render_status_bar(
            frame,
            area,
            "Ready",
            Color::LightBlue,
            &[("[Space]", "Start"), ("[Esc]", "Forfeit")],
        );
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let status = if game.fuel <= 0.0 {
        "NO FUEL"
    } else if game.over_pad() {
        "Over pad"
    } else {
        "Landing"
    };

    let status_color = if game.fuel <= 0.0 {
        Color::Red
    } else if game.over_pad() {
        Color::Green
    } else {
        Color::LightBlue
    };

    render_status_bar(
        frame,
        area,
        status,
        status_color,
        &[
            ("[L/R]", "Rotate"),
            ("[Space/Up]", "Thrust"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &LanderGame) {
    if area.width < 2 {
        return;
    }

    let inner = render_info_panel_frame(frame, area);

    let fuel_pct = if game.max_fuel > 0.0 {
        (game.fuel / game.max_fuel * 100.0).round() as u32
    } else {
        0
    };

    let fuel_color = if fuel_pct < 20 {
        Color::Red
    } else if fuel_pct < 50 {
        Color::Yellow
    } else {
        Color::Green
    };

    let vy_color = if game.vy.abs() <= 0.08 {
        Color::Green
    } else if game.vy.abs() <= 0.15 {
        Color::Yellow
    } else {
        Color::Red
    };

    let angle_deg = (game.angle.to_degrees()).round() as i32;

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Fuel: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", fuel_pct),
                Style::default().fg(fuel_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Alt:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}", game.altitude()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("VelY: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:+.3}", game.vy), Style::default().fg(vy_color)),
        ]),
        Line::from(vec![
            Span::styled("VelX: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:+.3}", game.vx),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Angle:", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" {}deg", angle_deg),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Safe landing:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" VY  ", Style::default().fg(Color::DarkGray)),
            Span::styled("<0.08", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(" VX  ", Style::default().fg(Color::DarkGray)),
            Span::styled("<0.04", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(" Tilt", Style::default().fg(Color::DarkGray)),
            Span::styled(" <15d", Style::default().fg(Color::Green)),
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
fn render_lander_game_over(frame: &mut Frame, area: Rect, game: &LanderGame) {
    let result = game.game_result.as_ref().unwrap();

    let fuel_pct = if game.max_fuel > 0.0 {
        (game.fuel / game.max_fuel * 100.0).round() as u32
    } else {
        0
    };

    let (result_type, title, message, reward) = match result {
        LanderResult::Win => {
            let reward_text = game.difficulty.reward().description();
            (
                GameResultType::Win,
                ":: LUNAR DESCENT COMPLETE! ::",
                format!("Successful landing with {}% fuel remaining.", fuel_pct),
                reward_text,
            )
        }
        LanderResult::Loss => {
            let message = if game.forfeit_pending {
                "You aborted the landing attempt.".to_string()
            } else if game.fuel <= 0.0 {
                "Out of fuel. The lander crashed into the surface.".to_string()
            } else {
                "The lander crashed into the surface.".to_string()
            };
            let result_type = if game.forfeit_pending {
                GameResultType::Forfeit
            } else {
                GameResultType::Loss
            };
            (
                result_type,
                "LANDING FAILED",
                message,
                "No penalty incurred.".to_string(),
            )
        }
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward);
}
