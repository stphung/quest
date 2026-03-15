//! Runic Lights minigame UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::runic_lights::{RunicLightsGame, RunicLightsResult};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the Runic Lights game scene.
pub fn render_runic_lights(
    frame: &mut Frame,
    area: Rect,
    game: &RunicLightsGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
) {
    // Game over overlay
    if game.game_result.is_some() {
        render_game_over(frame, area, game, show_dismiss_hint);
        return;
    }

    // Minimum terminal size for the game
    let min_width = (game.size as u16 * 4) + 6;
    let min_height = (game.size as u16 * 2) + 6;
    if area.width < min_width || area.height < min_height {
        render_minigame_too_small(frame, area, "Runic Lights", min_width, min_height);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Runic Lights ",
        Color::Cyan,
        (game.size as u16) + 2,
        22,
        ctx,
    );

    render_grid(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the grid of runes.
fn render_grid(frame: &mut Frame, area: Rect, game: &RunicLightsGame) {
    let size = game.size;
    // Each cell: 2 chars wide ("● " or "○ "), 1 char tall
    let cell_width = 2u16;
    let grid_width = size as u16 * cell_width;
    let grid_height = size as u16;

    // Center the grid
    let x_offset = area.x + (area.width.saturating_sub(grid_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(grid_height)) / 2;

    for row in 0..size {
        let mut spans = Vec::new();

        for col in 0..size {
            let lit = game.board[row][col];
            let is_cursor = game.cursor == (row, col);

            let symbol = if lit { "\u{25CF} " } else { "\u{25CB} " };

            let base_color = if lit {
                Color::Rgb(100, 200, 255) // Bright cyan for lit
            } else {
                Color::Rgb(90, 90, 105) // Visible gray for dark
            };

            let mut style = Style::default().fg(base_color);

            if is_cursor {
                style = style
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            spans.push(Span::styled(symbol, style));
        }

        let line = Line::from(spans);
        let y = y_offset + row as u16;
        if y < area.y + area.height {
            frame.render_widget(
                Paragraph::new(vec![line]),
                Rect::new(x_offset, y, grid_width, 1),
            );
        }
    }
}

/// Render the status bar.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &RunicLightsGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Extinguishing...",
        Color::Cyan,
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Toggle"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

/// Render the info panel.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &RunicLightsGame) {
    let inner = render_info_panel_frame(frame, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Grid   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}×{}", game.size, game.size),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Moves  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.moves, game.move_limit),
                Style::default().fg(if game.moves > game.par {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Par    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", game.par), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Lit    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.lit_count(), game.total_cells()),
                Style::default().fg(if game.lit_count() == 0 {
                    Color::Green
                } else {
                    Color::Rgb(100, 200, 255)
                }),
            ),
        ]),
    ];

    // Truncate to fit
    lines.truncate(inner.height as usize);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render game over overlay.
fn render_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &RunicLightsGame,
    show_dismiss_hint: bool,
) {
    use crate::challenges::menu::DifficultyInfo;

    let (result_type, title, message, reward) = match game.game_result {
        Some(RunicLightsResult::Win) => {
            let r = game.difficulty.reward();
            let reward_text = if r.prestige_ranks > 0 {
                format!(
                    "+{} Prestige Ranks, +{} Stormglass",
                    r.prestige_ranks, r.stormglass
                )
            } else {
                format!("+{} Stormglass", r.stormglass)
            };
            let msg = if game.moves <= game.par {
                format!("Solved in {} moves (par {}) \u{2605}", game.moves, game.par)
            } else {
                format!("Solved in {} moves (par {})", game.moves, game.par)
            };
            (
                GameResultType::Win,
                "ALL RUNES EXTINGUISHED!".to_string(),
                msg,
                reward_text,
            )
        }
        _ => (
            GameResultType::Loss,
            "RUNES STILL ABLAZE!".to_string(),
            format!("Exceeded move limit ({}/{})", game.moves, game.move_limit),
            "No penalty incurred.".to_string(),
        ),
    };
    render_game_over_overlay(
        frame,
        area,
        result_type,
        &title,
        &message,
        &reward,
        show_dismiss_hint,
    );
}
