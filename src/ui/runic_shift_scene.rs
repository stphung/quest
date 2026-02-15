//! Runic Shift scene rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::runic_shift::types::{
    Block, BlockState, RuneColor, RunicShiftGame, RunicShiftResult, GRID_COLS, GRID_ROWS, MAX_LIVES,
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const WELL_INNER_WIDTH: u16 = 17;
const ROW_PREFIX_WIDTH: u16 = 3;
const CURSOR_BG: Color = Color::Rgb(255, 196, 64);

/// Render the Runic Shift scene.
pub fn render_runic_shift_scene(
    frame: &mut Frame,
    area: Rect,
    game: &RunicShiftGame,
    ctx: &super::responsive::LayoutContext,
) {
    if game.game_result.is_some() {
        render_runic_shift_game_over(frame, area, game);
        return;
    }

    const MIN_WIDTH: u16 = 36;
    const MIN_HEIGHT: u16 = 18;
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Runic Shift", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Runic Shift ",
        Color::LightMagenta,
        16,
        18,
        ctx,
    );

    render_well(frame, layout.content, game);

    if game.waiting_to_start {
        render_start_prompt(frame, layout.content);
    }

    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_well(frame: &mut Frame, area: Rect, game: &RunicShiftGame) {
    if area.height < GRID_ROWS as u16 + 3 || area.width < 24 {
        return;
    }

    let danger_flash_on = (game.accumulated_time_ms / 250).is_multiple_of(2);
    let flash_div = game.clear_flash_ms.max(1);
    let clear_flash_on =
        game.clear_timer_ms > 0 && (game.clear_timer_ms / flash_div).is_multiple_of(2);

    let render_width = ROW_PREFIX_WIDTH + WELL_INNER_WIDTH + 4;
    let render_height = GRID_ROWS as u16 + 3;

    let x_off = area.x + area.width.saturating_sub(render_width) / 2;
    let y_off = area.y + area.height.saturating_sub(render_height) / 2;

    let border_style = Style::default().fg(Color::DarkGray);

    // Top border.
    let top = format!(
        "{}╭─────────────────╮",
        " ".repeat(ROW_PREFIX_WIDTH as usize)
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(top, border_style))),
        Rect::new(x_off, y_off, render_width, 1),
    );

    // Grid rows.
    for row in 0..GRID_ROWS {
        let mut spans = Vec::new();
        let row_has_cursor = row == game.cursor_row;

        let danger_prefix = if row == 1 && game.has_danger() && danger_flash_on {
            Span::styled("!! ", Style::default().fg(Color::LightRed))
        } else {
            Span::styled("   ", Style::default().fg(Color::DarkGray))
        };
        spans.push(danger_prefix);
        let row_border_style = if row_has_cursor {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            border_style
        };
        spans.push(Span::styled("│", row_border_style));

        for col in 0..GRID_COLS {
            let (glyph, style) = cell_visual(game.grid[row][col], clear_flash_on);
            let highlighted =
                row == game.cursor_row && (col == game.cursor_col || col == game.cursor_col + 1);
            let style = if highlighted {
                style
                    .fg(Color::Black)
                    .bg(CURSOR_BG)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                style
            };
            spans.push(Span::styled(glyph, style));

            if col + 1 < GRID_COLS {
                if row_has_cursor && col == game.cursor_col {
                    spans.push(Span::styled(" ", Style::default().bg(CURSOR_BG)));
                } else {
                    spans.push(Span::raw(" "));
                }
            }
        }

        spans.push(Span::styled("│", row_border_style));

        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(x_off, y_off + 1 + row as u16, render_width, 1),
        );
    }

    // Bottom border.
    let bottom = format!(
        "{}╰─────────────────╯",
        " ".repeat(ROW_PREFIX_WIDTH as usize)
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(bottom, border_style))),
        Rect::new(x_off, y_off + 1 + GRID_ROWS as u16, render_width, 1),
    );

    // Next row preview.
    let mut preview_spans = vec![Span::styled(
        "   next: ",
        Style::default().fg(Color::DarkGray),
    )];
    for (i, color) in game.next_row.iter().enumerate() {
        preview_spans.push(Span::styled(color.glyph(), preview_style(*color)));
        if i + 1 < GRID_COLS {
            preview_spans.push(Span::styled(" ", Style::default().fg(Color::DarkGray)));
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(preview_spans)),
        Rect::new(x_off, y_off + 2 + GRID_ROWS as u16, render_width, 1),
    );
}

fn cell_visual(block: Option<Block>, clear_flash_on: bool) -> (String, Style) {
    match block {
        Some(block) if block.state == BlockState::Clearing && clear_flash_on => (
            "✦✦".to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Some(block) => (block.color.glyph().to_string(), rune_style(block.color)),
        None => (
            "  ".to_string(),
            Style::default().fg(Color::Rgb(40, 40, 52)),
        ),
    }
}

fn rune_style(color: RuneColor) -> Style {
    Style::default().fg(match color {
        RuneColor::Fire => Color::LightRed,
        RuneColor::Water => Color::LightBlue,
        RuneColor::Earth => Color::LightGreen,
        RuneColor::Light => Color::Yellow,
        RuneColor::Frost => Color::Cyan,
        RuneColor::Arcane => Color::Magenta,
    })
}

fn preview_style(color: RuneColor) -> Style {
    rune_style(color).add_modifier(Modifier::DIM)
}

fn render_start_prompt(frame: &mut Frame, area: Rect) {
    if area.height < 5 || area.width < 24 {
        return;
    }

    let prompt = "[ Press Space to Start ]";
    let x = area.x + area.width.saturating_sub(prompt.len() as u16) / 2;
    let y = area.y + area.height / 2;

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            prompt,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))),
        Rect::new(x, y, prompt.len() as u16, 1),
    );
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &RunicShiftGame) {
    if game.waiting_to_start {
        render_status_bar(
            frame,
            area,
            "Rune well primed",
            Color::LightMagenta,
            &[
                ("[Arrows]", "Move"),
                ("[Space]", "Start"),
                ("[R]", "Rise"),
                ("[Esc]", "Forfeit"),
            ],
        );
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    if game.clear_timer_ms > 0 {
        render_status_bar(
            frame,
            area,
            "Runes clearing...",
            Color::Yellow,
            &[("[Esc]", "Forfeit")],
        );
        return;
    }

    let danger_flash_on = (game.accumulated_time_ms / 250).is_multiple_of(2);
    if game.has_danger() && danger_flash_on {
        render_status_bar(
            frame,
            area,
            "DANGER: Top rows are filling",
            Color::LightRed,
            &[
                ("[Arrows]", "Move"),
                ("[Space]", "Swap"),
                ("[R]", "Rise"),
                ("[Esc]", "Forfeit"),
            ],
        );
        return;
    }

    render_status_bar(
        frame,
        area,
        "Match runes and build chains",
        Color::LightCyan,
        &[
            ("[Arrows]", "Move"),
            ("[Space]", "Swap"),
            ("[R]", "Rise"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &RunicShiftGame) {
    let inner = render_info_panel_frame(frame, area);

    let rise_text = if game.clear_timer_ms > 0 {
        "Paused".to_string()
    } else {
        let remaining_ms = game
            .rise_interval_ms
            .saturating_sub(game.rise_accumulated_ms.min(game.rise_interval_ms));
        format!("{:.1}s", remaining_ms as f64 / 1000.0)
    };

    let lives = (0..MAX_LIVES)
        .map(|i| if i < game.lives { '●' } else { '○' })
        .collect::<String>();

    let chain = if game.current_chain > 1 {
        format!("x{}!", game.current_chain)
    } else if game.current_chain == 1 {
        "x1".to_string()
    } else {
        "-".to_string()
    };
    let best = if game.best_chain > 0 {
        format!("x{}", game.best_chain)
    } else {
        "-".to_string()
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Clears: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.clears.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.target_clears.to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Chain: ", Style::default().fg(Color::DarkGray)),
            Span::styled(chain, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Best:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(best, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Lives: ", Style::default().fg(Color::DarkGray)),
            Span::styled(lives, Style::default().fg(Color::LightRed)),
        ]),
        Line::from(vec![
            Span::styled("Rise:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(rise_text, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Colors: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.num_colors.to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_runic_shift_game_over(frame: &mut Frame, area: Rect, game: &RunicShiftGame) {
    let Some(result) = game.game_result else {
        return;
    };

    let (result_type, title, message) = match result {
        RunicShiftResult::Win => (
            GameResultType::Win,
            "RUNE MASTERED!",
            format!(
                "{} clears reached (target {}) - best chain x{}",
                game.clears,
                game.target_clears,
                game.best_chain.max(1)
            ),
        ),
        RunicShiftResult::Loss if game.forfeit_pending => (
            GameResultType::Forfeit,
            "CHALLENGE ABANDONED",
            format!("You stepped away at {} clears.", game.clears),
        ),
        RunicShiftResult::Loss => (
            GameResultType::Loss,
            "RUNES OVERFLOW",
            format!("Defeated at {} clears.", game.clears),
        ),
    };

    let reward_text = if result == RunicShiftResult::Win {
        game.difficulty.reward().description()
    } else {
        "No reward".to_string()
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward_text);
}
