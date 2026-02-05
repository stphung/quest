//! Minesweeper game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, GameResultType,
};
use crate::challenges::minesweeper::{Cell, MinesweeperGame, MinesweeperResult};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the minesweeper game scene.
pub fn render_minesweeper(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    // Game over overlay
    if game.game_result.is_some() {
        render_minesweeper_game_over(frame, area, game);
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

            let (text, color) = get_cell_display(cell);

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
fn get_cell_display(cell: &Cell) -> (&'static str, Color) {
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
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let (status_text, status_color) = if !game.first_click_done {
        ("Click to begin", Color::Yellow)
    } else {
        ("Detecting...", Color::Green)
    };

    render_status_bar(
        frame,
        area,
        status_text,
        status_color,
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Reveal"),
            ("[F]", "Flag"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let inner = render_info_panel_frame(frame, area);

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

fn render_minesweeper_game_over(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let result = game.game_result.as_ref().unwrap();

    let (result_type, title, message, reward) = match result {
        MinesweeperResult::Win => {
            let prestige = match game.difficulty {
                crate::challenges::minesweeper::MinesweeperDifficulty::Novice => 1,
                crate::challenges::minesweeper::MinesweeperDifficulty::Apprentice => 2,
                crate::challenges::minesweeper::MinesweeperDifficulty::Journeyman => 3,
                crate::challenges::minesweeper::MinesweeperDifficulty::Master => 5,
            };
            (
                GameResultType::Win,
                ":: AREA SECURED! ::",
                "You detected all the traps!",
                format!("+{} Prestige Ranks", prestige),
            )
        }
        MinesweeperResult::Loss => (
            GameResultType::Loss,
            "TRAP TRIGGERED!",
            "You stepped on a hidden trap.",
            "No penalty incurred.".to_string(),
        ),
    };

    render_game_over_overlay(frame, area, result_type, title, message, &reward);
}
