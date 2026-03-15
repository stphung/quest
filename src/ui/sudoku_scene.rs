//! Sigil Matrix (Sudoku) game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::sudoku::{filled_count, given_count, SudokuGame, SudokuResult};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const SUDOKU_CONTROLS: &[(&str, &str)] = &[
    ("[Arrows]", "Move"),
    ("[1-9]", "Place"),
    ("[BS]", "Clear"),
    ("[Esc]", "Forfeit"),
];
const MIN_WIDTH: u16 = 30;
const MIN_HEIGHT: u16 = 18;

/// Render the Sigil Matrix (Sudoku) game scene.
pub fn render_sudoku(
    frame: &mut Frame,
    area: Rect,
    game: &SudokuGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    if game.game_result.is_some() {
        render_sudoku_game_over(frame, area, game, show_dismiss_hint, stormglass_discovered);
        return;
    }

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Sigil Matrix", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    let layout = create_game_layout(frame, area, " Sigil Matrix ", Color::Magenta, 15, 26, ctx);

    render_grid(frame, layout.content, game);
    render_sudoku_status_bar(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the 9x9 Sudoku grid with box-drawing characters.
fn render_grid(frame: &mut Frame, area: Rect, game: &SudokuGame) {
    let mut y = area.y;
    let x = area.x + 1;
    let grid_width = area.width.saturating_sub(2);

    // Top border
    if y < area.y + area.height {
        let line = Paragraph::new(Line::from(Span::styled(
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{252c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{252c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(line, Rect::new(x, y, grid_width, 1));
        y += 1;
    }

    for row in 0..9 {
        if y >= area.y + area.height {
            break;
        }

        // Mid separator after rows 3 and 6
        if row == 3 || row == 6 {
            let sep = Paragraph::new(Line::from(Span::styled(
                "\u{251c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2524}",
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(sep, Rect::new(x, y, grid_width, 1));
            y += 1;
            if y >= area.y + area.height {
                break;
            }
        }

        // Row content: "│ X X X │ X X X │ X X X │" = 25 chars
        let mut spans = Vec::new();
        for col in 0..9 {
            // Left border or box separator
            if col == 0 || col == 3 || col == 6 {
                spans.push(Span::styled(
                    "\u{2502} ",
                    Style::default().fg(Color::DarkGray),
                ));
            } else {
                spans.push(Span::raw(" "));
            }

            let val = game.board[row][col];
            let is_cursor = row == game.cursor_row && col == game.cursor_col;
            let is_given = game.given[row][col];
            let is_conflict = game.conflicts[row][col];

            let (text, mut style) = if val == 0 {
                (
                    "\u{00b7}".to_string(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )
            } else if is_conflict {
                (
                    val.to_string(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else if is_given {
                (
                    val.to_string(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (val.to_string(), Style::default().fg(Color::Cyan))
            };

            if is_cursor {
                style = style.bg(Color::DarkGray);
            }

            spans.push(Span::styled(text, style));

            // Trailing space after last digit in each 3-cell box
            if col == 2 || col == 5 || col == 8 {
                spans.push(Span::raw(" "));
            }
        }
        // Right border
        spans.push(Span::styled(
            "\u{2502}",
            Style::default().fg(Color::DarkGray),
        ));

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(line, Rect::new(x, y, grid_width, 1));
        y += 1;
    }

    // Bottom border
    if y < area.y + area.height {
        let line = Paragraph::new(Line::from(Span::styled(
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2534}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2534}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(line, Rect::new(x, y, grid_width, 1));
    }
}

/// Render the status bar with sigil count and controls.
fn render_sudoku_status_bar(frame: &mut Frame, area: Rect, game: &SudokuGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let filled = filled_count(game);
    let status_text = format!("{}/81 sigils placed", filled);
    let status_color = if filled == 81 {
        Color::Green
    } else {
        Color::Yellow
    };

    render_status_bar(frame, area, &status_text, status_color, SUDOKU_CONTROLS);
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &SudokuGame) {
    let inner = render_info_panel_frame(frame, area);

    let given = given_count(game);
    let filled = filled_count(game);
    let remaining = 81 - filled;
    let conflict_count: usize = game.conflicts.iter().flatten().filter(|&&c| c).count();

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Given: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", given), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Filled: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}/81", filled), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Remaining: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", remaining),
                Style::default().fg(if remaining == 0 {
                    Color::Green
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Cursor: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("R{}C{}", game.cursor_row + 1, game.cursor_col + 1),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    if conflict_count > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("{} conflicts!", conflict_count),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

/// Render the game over overlay.
fn render_sudoku_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &SudokuGame,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    let result = game.game_result.as_ref().unwrap();

    let (result_type, title, message, reward) = match result {
        SudokuResult::Win => (
            GameResultType::Win,
            ":: MATRIX COMPLETE! ::",
            "The sigil matrix hums with power!".to_string(),
            game.difficulty.reward().description(stormglass_discovered),
        ),
        SudokuResult::Loss => (
            GameResultType::Loss,
            "MATRIX FRACTURED",
            "The sigil pattern is lost.".to_string(),
            "No penalty incurred.".to_string(),
        ),
    };

    render_game_over_overlay(
        frame,
        area,
        result_type,
        title,
        &message,
        &reward,
        show_dismiss_hint,
    );
}
