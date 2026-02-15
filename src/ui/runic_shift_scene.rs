//! Runic Shift scene rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::runic_shift::types::{
    Block, BlockState, BoardAnimation, RuneColor, RunicShiftGame, RunicShiftResult, SwapAnimation,
    GRID_COLS, GRID_ROWS, MAX_LIVES,
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
    let swap_anim = match &game.board_animation {
        Some(BoardAnimation::Swap(anim)) => Some(*anim),
        _ => None,
    };
    let (hidden_cells, overlay_cells) = animation_layers(game);

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
        let row_is_target_line = row == game.target_line_row;

        let danger_prefix = if row == 1 && game.has_danger() && danger_flash_on {
            Span::styled("!! ", Style::default().fg(Color::LightRed))
        } else if row_is_target_line {
            Span::styled(
                "▼  ",
                Style::default()
                    .fg(Color::LightMagenta)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("   ", Style::default().fg(Color::DarkGray))
        };
        spans.push(danger_prefix);
        let row_border_style = if row_has_cursor {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if row_is_target_line {
            Style::default().fg(Color::LightMagenta)
        } else {
            border_style
        };
        spans.push(Span::styled("│", row_border_style));

        for col in 0..GRID_COLS {
            let is_swap_cell = swap_anim.is_some_and(|anim| {
                row == anim.row && (col == anim.left_col || col == anim.right_col)
            });
            let (glyph, style) = if let Some(anim) = swap_anim {
                if let Some((glyph, style)) = swap_cell_visual(anim, row, col, clear_flash_on) {
                    (glyph, style)
                } else {
                    let base_block = if hidden_cells[row][col] {
                        None
                    } else {
                        game.grid[row][col]
                    };
                    cell_visual_with_overlay(base_block, overlay_cells[row][col], clear_flash_on)
                }
            } else {
                let base_block = if hidden_cells[row][col] {
                    None
                } else {
                    game.grid[row][col]
                };
                cell_visual_with_overlay(base_block, overlay_cells[row][col], clear_flash_on)
            };
            let highlighted = !is_swap_cell
                && row == game.cursor_row
                && (col == game.cursor_col || col == game.cursor_col + 1);
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
                let swap_gap_motion = swap_anim.is_some_and(|anim| {
                    row == anim.row && col == anim.left_col && swap_phase(anim) == 1
                });
                if swap_gap_motion {
                    spans.push(Span::styled(
                        "↔",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else if row_has_cursor && col == game.cursor_col {
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

fn cell_visual_with_overlay(
    base_block: Option<Block>,
    overlay_color: Option<RuneColor>,
    clear_flash_on: bool,
) -> (String, Style) {
    if let Some(color) = overlay_color {
        return (
            color.glyph().to_string(),
            rune_style(color).add_modifier(Modifier::BOLD),
        );
    }
    cell_visual(base_block, clear_flash_on)
}

fn swap_phase(anim: SwapAnimation) -> u8 {
    let duration = anim.duration_ms.max(1);
    let progress = anim.duration_ms.saturating_sub(anim.remaining_ms);
    let scaled = progress.saturating_mul(10) / duration;
    if scaled < 3 {
        0
    } else if scaled < 7 {
        1
    } else {
        2
    }
}

fn swap_cell_visual(
    anim: SwapAnimation,
    row: usize,
    col: usize,
    clear_flash_on: bool,
) -> Option<(String, Style)> {
    if row != anim.row || (col != anim.left_col && col != anim.right_col) {
        return None;
    }

    let phase = swap_phase(anim);
    let (from, to, mid_glyph) = if col == anim.left_col {
        (anim.left_block, anim.right_block, "▐ ")
    } else {
        (anim.right_block, anim.left_block, " ▌")
    };

    let visual = match phase {
        0 => cell_visual(from, clear_flash_on),
        1 => {
            let Some(block) = from else {
                return Some(("  ".to_string(), Style::default()));
            };
            (
                mid_glyph.to_string(),
                rune_style(block.color).add_modifier(Modifier::BOLD),
            )
        }
        _ => cell_visual(to, clear_flash_on),
    };

    Some(visual)
}

fn animation_layers(
    game: &RunicShiftGame,
) -> (
    [[bool; GRID_COLS]; GRID_ROWS],
    [[Option<RuneColor>; GRID_COLS]; GRID_ROWS],
) {
    let mut hidden = [[false; GRID_COLS]; GRID_ROWS];
    let mut overlay = [[None; GRID_COLS]; GRID_ROWS];

    let Some(animation) = &game.board_animation else {
        return (hidden, overlay);
    };

    match animation {
        BoardAnimation::Fall(anim) => {
            let duration = anim.duration_ms.max(1) as f32;
            let elapsed = (anim.duration_ms.saturating_sub(anim.remaining_ms)) as f32;
            let t = (elapsed / duration).clamp(0.0, 1.0);

            for motion in &anim.blocks {
                hidden[motion.to_row][motion.col] = true;
                let row = lerp_index(motion.from_row, motion.to_row, t, GRID_ROWS - 1);
                overlay[row][motion.col] = Some(motion.color);
            }
        }
        BoardAnimation::Swap(_) => {}
    }

    (hidden, overlay)
}

fn lerp_index(from: usize, to: usize, t: f32, max: usize) -> usize {
    let from = from as f32;
    let to = to as f32;
    let value = from + (to - from) * t;
    value.round().clamp(0.0, max as f32) as usize
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

    if let Some(animation) = &game.board_animation {
        let (text, color) = match animation {
            crate::challenges::runic_shift::types::BoardAnimation::Swap(_) => {
                ("Panels swapping...", Color::Yellow)
            }
            crate::challenges::runic_shift::types::BoardAnimation::Fall(_) => {
                ("Panels falling...", Color::Yellow)
            }
        };
        render_status_bar(frame, area, text, color, &[("[Esc]", "Forfeit")]);
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
    let below_count = game.runes_below_target_line();
    let rows_to_clear = GRID_ROWS.saturating_sub(game.target_line_row);

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
            Span::styled("Goal: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("Clear {} bottom rows", rows_to_clear),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Below line: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                below_count.to_string(),
                Style::default()
                    .fg(if below_count == 0 {
                        Color::Green
                    } else {
                        Color::White
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Clears: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.clears.to_string(), Style::default().fg(Color::White)),
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
                "Target line cleared - best chain x{} ({} clears)",
                game.best_chain.max(1),
                game.clears
            ),
        ),
        RunicShiftResult::Loss if game.forfeit_pending => (
            GameResultType::Forfeit,
            "CHALLENGE ABANDONED",
            format!(
                "You stepped away with {} runes below the line.",
                game.runes_below_target_line()
            ),
        ),
        RunicShiftResult::Loss => (
            GameResultType::Loss,
            "RUNES OVERFLOW",
            format!(
                "{} runes remained below the line.",
                game.runes_below_target_line()
            ),
        ),
    };

    let reward_text = if result == RunicShiftResult::Win {
        game.difficulty.reward().description()
    } else {
        "No reward".to_string()
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward_text);
}
