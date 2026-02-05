//! Minesweeper game UI rendering.

use super::game_common::{create_game_layout, render_status_bar};
use crate::minesweeper::{Cell, MinesweeperGame, MinesweeperResult};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the minesweeper game scene.
pub fn render_minesweeper(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    // Game over overlay
    if game.game_result.is_some() {
        render_game_over_overlay(frame, area, game);
        return;
    }

    // Use shared layout
    let layout = create_game_layout(frame, area, " Trap Detection ", Color::Yellow, 10, 24);

    render_grid(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the minefield grid.
fn render_grid(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    // Calculate grid dimensions (each cell is 2 chars wide, 1 char tall)
    // No border - outer block provides it
    let grid_width = (game.width * 2) as u16;
    let grid_height = game.height as u16;

    // Center the grid in available space
    let x_offset = area.x + (area.width.saturating_sub(grid_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(grid_height)) / 2;

    let game_over = game.game_result.is_some();

    // Draw each row
    for row in 0..game.height {
        let mut spans = Vec::new();

        for col in 0..game.width {
            let cell = &game.grid[row][col];
            let is_cursor = game.cursor == (row, col);

            let (text, color) = get_cell_display(cell, game_over);

            let mut style = Style::default().fg(color);
            if is_cursor && !game_over {
                style = style.bg(Color::DarkGray);
            }

            spans.push(Span::styled(text, style));
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(x_offset, y_offset + row as u16, grid_width, 1),
        );
    }
}

/// Get the display text and color for a cell.
fn get_cell_display(cell: &Cell, _game_over: bool) -> (&'static str, Color) {
    if cell.flagged && !cell.revealed {
        return ("F ", Color::Red);
    }

    if !cell.revealed {
        return ("# ", Color::Gray);
    }

    // Cell is revealed
    if cell.has_mine {
        return ("* ", Color::Red);
    }

    // Revealed number or empty
    match cell.adjacent_mines {
        0 => (". ", Color::DarkGray),
        1 => ("1 ", Color::Blue),
        2 => ("2 ", Color::Green),
        3 => ("3 ", Color::Red),
        4 => ("4 ", Color::Magenta),
        5 => ("5 ", Color::Yellow),
        6 => ("6 ", Color::Cyan),
        7 => ("7 ", Color::Gray),
        8 => ("8 ", Color::White),
        _ => ("? ", Color::White),
    }
}

/// Render the status bar below the grid.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    if game.game_result.is_some() {
        return;
    }

    let (status_text, status_color) = if game.forfeit_pending {
        ("Forfeit game?", Color::LightRed)
    } else if !game.first_click_done {
        ("Click to begin", Color::Yellow)
    } else {
        ("Detecting...", Color::Green)
    };

    let controls: &[(&str, &str)] = if game.forfeit_pending {
        &[("[Esc]", "Confirm"), ("[Any]", "Cancel")]
    } else {
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Reveal"),
            ("[F]", "Flag"),
            ("[Esc]", "Forfeit"),
        ]
    };

    render_status_bar(frame, area, status_text, status_color, controls);
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Remaining (mines - flags)
    let remaining = game.mines_remaining();
    let remaining_color = if remaining < 0 {
        Color::Red
    } else {
        Color::White
    };

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Grid: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}x{}", game.width, game.height),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Traps: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.total_mines),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Remaining: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", remaining),
                Style::default().fg(remaining_color),
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
            Span::styled(" # ", Style::default().fg(Color::Gray)),
            Span::styled("Hidden", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" F ", Style::default().fg(Color::Red)),
            Span::styled("Flag", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" * ", Style::default().fg(Color::Red)),
            Span::styled("Trap", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" . ", Style::default().fg(Color::DarkGray)),
            Span::styled("Empty", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" 1-8 ", Style::default().fg(Color::Blue)),
            Span::styled("Adjacent", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

/// Render the game over overlay.
fn render_game_over_overlay(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let result = game.game_result.as_ref().unwrap();

    let (title, color) = match result {
        MinesweeperResult::Win => ("Area Secured!", Color::Green),
        MinesweeperResult::Loss => ("Trap Triggered!", Color::Red),
    };

    let reward_text = match result {
        MinesweeperResult::Win => {
            // Calculate reward based on difficulty
            let prestige = match game.difficulty {
                crate::minesweeper::MinesweeperDifficulty::Novice => 1,
                crate::minesweeper::MinesweeperDifficulty::Apprentice => 2,
                crate::minesweeper::MinesweeperDifficulty::Journeyman => 3,
                crate::minesweeper::MinesweeperDifficulty::Master => 5,
            };
            format!("+{} Prestige Ranks", prestige)
        }
        MinesweeperResult::Loss => "No reward".to_string(),
    };

    // Center overlay
    let width = 30;
    let height = 6;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(reward_text, Style::default().fg(Color::White))),
        Line::from(Span::styled(
            "[Any key to continue]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
