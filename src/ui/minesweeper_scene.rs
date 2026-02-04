//! Minesweeper game UI rendering.

use crate::minesweeper::{Cell, MinesweeperGame, MinesweeperResult};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the minesweeper game scene.
pub fn render_minesweeper(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    frame.render_widget(Clear, area);

    // Split: Grid on left, info panel on right (24 chars wide)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),    // Grid area
            Constraint::Length(24), // Info panel
        ])
        .split(area);

    render_grid(frame, chunks[0], game);
    render_info_panel(frame, chunks[1], game);

    // Game over overlay (centered on grid area, not full area)
    if game.game_result.is_some() {
        render_game_over_overlay(frame, chunks[0], game);
    }
}

/// Render the minefield grid.
fn render_grid(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let block = Block::default()
        .title(" Trap Detection ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate grid dimensions (each cell is 2 chars wide, 1 char tall)
    let grid_width = (game.width * 2) as u16;
    let grid_height = game.height as u16;

    // Center the grid in available space
    let x_offset = inner.x + (inner.width.saturating_sub(grid_width)) / 2;
    let y_offset = inner.y + (inner.height.saturating_sub(grid_height)) / 2;

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

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        // Title
        Line::from(Span::styled(
            "Trap Detection",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        // Difficulty
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        // Grid size
        Line::from(vec![
            Span::styled("Grid: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}x{}", game.width, game.height),
                Style::default().fg(Color::White),
            ),
        ]),
        // Traps count
        Line::from(vec![
            Span::styled("Traps: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.total_mines),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    // Remaining (mines - flags)
    let remaining = game.mines_remaining();
    let remaining_color = if remaining < 0 {
        Color::Red
    } else {
        Color::White
    };
    lines.push(Line::from(vec![
        Span::styled("Remaining: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", remaining),
            Style::default().fg(remaining_color),
        ),
    ]));

    lines.push(Line::from(""));

    // Status
    let status = if game.game_result.is_some() {
        Span::styled("", Style::default())
    } else if game.forfeit_pending {
        Span::styled("Forfeit game?", Style::default().fg(Color::LightRed))
    } else if !game.first_click_done {
        Span::styled("Click to begin", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("Detecting...", Style::default().fg(Color::Green))
    };
    lines.push(Line::from(status));
    lines.push(Line::from(""));

    // Controls
    if game.game_result.is_none() {
        if game.forfeit_pending {
            lines.push(Line::from(Span::styled(
                "[Esc] Confirm forfeit",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[Any] Cancel",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "[Arrows] Move",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[Enter] Reveal",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[F] Flag",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[Esc] Forfeit",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

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
