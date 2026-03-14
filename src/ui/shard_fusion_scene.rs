//! Shard Fusion (2048-style) game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::shard_fusion::{
    ShardFusionAnimState, ShardFusionGame, ShardFusionResult, SLIDE_TICKS,
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Painter, Shape},
        Paragraph,
    },
    Frame,
};

/// Tile size in halfblock pixels (width in columns = height in halfblock rows → visually square).
const TILE_PX: usize = 6;
/// Gap between tiles in pixels.
const GAP_PX: usize = 2;
/// Pixel stride per tile slot (tile + gap).
const STRIDE: usize = TILE_PX + GAP_PX;

/// Total board size in pixels (4 tiles + 3 gaps).
const BOARD_PX: usize = 4 * TILE_PX + 3 * GAP_PX;
/// Board width in terminal columns (1 pixel = 1 column).
const BOARD_TERM_W: u16 = BOARD_PX as u16;
/// Board height in terminal rows (2 halfblock pixels = 1 terminal row).
const BOARD_TERM_H: u16 = (BOARD_PX / 2) as u16;

/// Center a fixed-size rect inside a larger area.
fn center_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

/// Map a tile value to its text overlay color.
fn tile_color(value: u32) -> Color {
    match value {
        0 => Color::DarkGray,
        2 | 4 => Color::Gray,
        8 | 16 => Color::Yellow,
        32 | 64 => Color::LightGreen,
        128 | 256 => Color::LightCyan,
        512 | 1024 => Color::LightMagenta,
        _ => Color::LightRed, // 2048+
    }
}

/// Map a tile value to its canvas background color.
fn tile_bg_color(value: u32) -> Color {
    match value {
        0 => Color::Rgb(30, 30, 45),
        2 => Color::Rgb(55, 80, 120),
        4 => Color::Rgb(45, 100, 145),
        8 => Color::Rgb(35, 140, 110),
        16 => Color::Rgb(45, 160, 80),
        32 => Color::Rgb(140, 160, 35),
        64 => Color::Rgb(180, 130, 25),
        128 => Color::Rgb(185, 80, 30),
        256 => Color::Rgb(185, 50, 50),
        512 => Color::Rgb(155, 30, 110),
        1024 => Color::Rgb(115, 30, 165),
        2048 => Color::Rgb(65, 30, 165),
        _ => Color::Rgb(40, 20, 120), // 4096+
    }
}

/// Paint a single tile's pixel rectangle at a fractional grid position.
/// `row_f` / `col_f` are grid coordinates (0.0–3.0); values outside `[0, BOARD_PX)` are clipped.
fn paint_tile_at(painter: &mut Painter, row_f: f64, col_f: f64, value: u32, flash: bool) {
    if value == 0 {
        return;
    }
    let color = if flash {
        Color::Rgb(255, 230, 100)
    } else {
        tile_bg_color(value)
    };
    let px_x = (col_f * STRIDE as f64).round() as i64;
    let px_y = (row_f * STRIDE as f64).round() as i64;
    for dy in 0..TILE_PX as i64 {
        for dx in 0..TILE_PX as i64 {
            let x = px_x + dx;
            let y = px_y + dy;
            if x >= 0 && (x as usize) < BOARD_PX && y >= 0 && (y as usize) < BOARD_PX {
                painter.paint(x as usize, y as usize, color);
            }
        }
    }
}

/// Canvas shape that renders the 4×4 board, with optional slide interpolation.
struct BoardShape<'a> {
    game: &'a ShardFusionGame,
    /// Animation progress: 0.0 = start of slide, 1.0 = fully at destination.
    progress: f64,
}

impl<'a> Shape for BoardShape<'a> {
    fn draw(&self, painter: &mut Painter) {
        let is_flashing = matches!(self.game.anim_state, ShardFusionAnimState::Flashing(_));
        let is_sliding = self.progress < 1.0 && !self.game.slide_moves.is_empty();

        if is_sliding {
            // Collect source cells — moving tiles are rendered at lerped positions instead.
            let source_cells: std::collections::HashSet<(usize, usize)> =
                self.game.slide_moves.iter().map(|m| m.from).collect();

            // Stationary tiles from the pre-slide snapshot.
            for r in 0..4usize {
                for c in 0..4usize {
                    if source_cells.contains(&(r, c)) {
                        continue;
                    }
                    let value = self.game.pre_slide_board[r][c];
                    paint_tile_at(painter, r as f64, c as f64, value, false);
                }
            }

            // Moving tiles at interpolated positions.
            for tm in &self.game.slide_moves {
                let (from_r, from_c) = tm.from;
                let (to_r, to_c) = tm.to;
                let src_val = self.game.pre_slide_board[from_r][from_c];
                let row_f = from_r as f64 + (to_r as f64 - from_r as f64) * self.progress;
                let col_f = from_c as f64 + (to_c as f64 - from_c as f64) * self.progress;
                paint_tile_at(painter, row_f, col_f, src_val, false);
            }
        } else {
            // Normal rendering (Idle or Flashing): draw from the final board.
            for r in 0..4usize {
                for c in 0..4usize {
                    let value = self.game.board[r][c];
                    let flash = is_flashing && self.game.merged_cells.contains(&(r, c));
                    paint_tile_at(painter, r as f64, c as f64, value, flash);
                }
            }
        }
    }
}

/// Render the Shard Fusion scene.
pub fn render_shard_fusion_scene(
    frame: &mut Frame,
    area: Rect,
    game: &ShardFusionGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    if game.game_result.is_some() {
        render_shard_fusion_game_over(frame, area, game, show_dismiss_hint, stormglass_discovered);
        return;
    }

    let min_height = BOARD_TERM_H + 4;
    let min_width = BOARD_TERM_W + 26;
    if area.width < min_width || area.height < min_height {
        render_minigame_too_small(frame, area, "Shard Fusion", min_width, min_height);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Shard Fusion ",
        Color::Yellow,
        BOARD_TERM_H,
        24,
        ctx,
    );

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the 4×4 game board using Canvas HalfBlock pixel art with animated tile sliding.
fn render_board(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    let canvas_area = center_rect(BOARD_TERM_W, BOARD_TERM_H, area);

    // 0.0 = tiles at start positions, 1.0 = tiles at destination positions.
    let progress = match game.anim_state {
        ShardFusionAnimState::Sliding(t) => (SLIDE_TICKS - t) as f64 / SLIDE_TICKS as f64,
        _ => 1.0,
    };

    let canvas = Canvas::default()
        .x_bounds([0.0, BOARD_PX as f64])
        .y_bounds([0.0, (BOARD_TERM_H * 2) as f64])
        .marker(Marker::HalfBlock)
        .background_color(Color::Rgb(15, 15, 25))
        .paint(|ctx| {
            ctx.draw(&BoardShape { game, progress });
        });
    frame.render_widget(canvas, canvas_area);

    overlay_tile_numbers(frame, canvas_area, game, progress);
}

/// Overlay tile numbers as Paragraph widgets on top of the canvas.
///
/// During sliding, stationary tiles are drawn at their grid positions and moving
/// tiles are drawn at their interpolated positions using the pre-slide board values.
fn overlay_tile_numbers(
    frame: &mut Frame,
    canvas_area: Rect,
    game: &ShardFusionGame,
    progress: f64,
) {
    let is_flashing = matches!(game.anim_state, ShardFusionAnimState::Flashing(_));
    let is_sliding = progress < 1.0 && !game.slide_moves.is_empty();

    let source_cells: std::collections::HashSet<(usize, usize)> = if is_sliding {
        game.slide_moves.iter().map(|m| m.from).collect()
    } else {
        std::collections::HashSet::new()
    };

    // Board to use for grid-position tiles.
    let board_src = if is_sliding {
        &game.pre_slide_board
    } else {
        &game.board
    };

    // Stationary tiles.
    for (r, row) in board_src.iter().enumerate() {
        for (c, &value) in row.iter().enumerate() {
            if is_sliding && source_cells.contains(&(r, c)) {
                continue;
            }
            if value == 0 {
                continue;
            }
            let fg = if !is_sliding && is_flashing && game.merged_cells.contains(&(r, c)) {
                Color::White
            } else {
                tile_color(value)
            };
            render_number_at(frame, canvas_area, r as f64, c as f64, value, fg);
        }
    }

    // Moving tiles at interpolated positions.
    if is_sliding {
        for tm in &game.slide_moves {
            let (from_r, from_c) = tm.from;
            let (to_r, to_c) = tm.to;
            let src_val = game.pre_slide_board[from_r][from_c];
            if src_val == 0 {
                continue;
            }
            let row_f = from_r as f64 + (to_r as f64 - from_r as f64) * progress;
            let col_f = from_c as f64 + (to_c as f64 - from_c as f64) * progress;
            render_number_at(
                frame,
                canvas_area,
                row_f,
                col_f,
                src_val,
                tile_color(src_val),
            );
        }
    }
}

/// Render a tile's number at a fractional grid position, snapped to terminal rows.
fn render_number_at(
    frame: &mut Frame,
    canvas_area: Rect,
    row_f: f64,
    col_f: f64,
    value: u32,
    fg: Color,
) {
    // Terminal rows per tile and vertical centre row (derived from TILE_PX constant).
    const TILE_TERM_H: u16 = (TILE_PX / 2) as u16;
    const TEXT_ROW: usize = TILE_TERM_H as usize / 2;

    let px_x = (col_f * STRIDE as f64).round() as u16;
    let px_y = (row_f * STRIDE as f64).round() as u16;
    let term_x = canvas_area.x + px_x;
    let term_y = canvas_area.y + px_y / 2;

    if term_x >= canvas_area.x + canvas_area.width || term_y >= canvas_area.y + canvas_area.height {
        return;
    }

    let avail_w = (canvas_area.x + canvas_area.width).saturating_sub(term_x);
    let text_area = Rect {
        x: term_x,
        y: term_y,
        width: (TILE_PX as u16).min(avail_w),
        height: TILE_TERM_H,
    };

    let mut lines: Vec<Line> = (0..TILE_TERM_H as usize).map(|_| Line::from("")).collect();
    lines[TEXT_ROW] = Line::from(Span::styled(value.to_string(), Style::default().fg(fg)));

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        text_area,
    );
}

/// Render the status bar below the board.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Your move",
        Color::White,
        &[("[Arrows]", "Slide"), ("[Esc]", "Forfeit")],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let inner = render_info_panel_frame(frame, area);

    let target = game.difficulty.target_value();
    let highest = game.highest_tile();
    let highest_color = tile_color(highest);

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Score:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.score.to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Target:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(target.to_string(), Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(vec![
            Span::styled("Highest: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if highest == 0 {
                    "-".to_string()
                } else {
                    highest.to_string()
                },
                Style::default().fg(highest_color),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Diff:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::White)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the game over overlay.
fn render_shard_fusion_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &ShardFusionGame,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    let Some(result) = game.game_result else {
        return;
    };

    let (result_type, title, message) = match result {
        ShardFusionResult::Win => (
            GameResultType::Win,
            "SHARDS FUSED!",
            format!(
                "You merged the crystal shards to {}! Score: {}",
                game.difficulty.target_value(),
                game.score
            ),
        ),
        ShardFusionResult::Loss if game.forfeit_pending => (
            GameResultType::Forfeit,
            "FUSION ABANDONED",
            format!(
                "You stepped away with a highest tile of {}.",
                game.highest_tile()
            ),
        ),
        ShardFusionResult::Loss => (
            GameResultType::Loss,
            "FUSION FAILED",
            format!(
                "No more moves remain. Highest tile: {}.",
                game.highest_tile()
            ),
        ),
    };

    let reward_text = if result == ShardFusionResult::Win {
        game.difficulty.reward().description(stormglass_discovered)
    } else {
        "No reward".to_string()
    };

    render_game_over_overlay(
        frame,
        area,
        result_type,
        title,
        &message,
        &reward_text,
        show_dismiss_hint,
    );
}
