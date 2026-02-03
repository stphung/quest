//! Chess board UI rendering.

use crate::chess::{ChessGame, ChessResult};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the chess game scene
pub fn render_chess_scene(frame: &mut Frame, area: Rect, game: &ChessGame) {
    frame.render_widget(Clear, area);

    // Check for game over overlay
    if let Some(result) = game.game_result {
        render_game_over_overlay(frame, area, result, game.difficulty.reward_prestige());
        return;
    }

    let block = Block::default()
        .title(" Chess ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Board
            Constraint::Length(2), // Status
        ])
        .split(inner);

    render_board(frame, chunks[0], game);
    render_status(frame, chunks[1], game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &ChessGame) {
    // Clean grid style: 5 chars per square + borders + rank label
    let cell_width: u16 = 5;
    let board_width: u16 = 3 + (cell_width * 8) + 1; // rank label + 8 cells + right border
    let board_height: u16 = 11; // 8 rows + top border + bottom border + file labels

    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    let border_color = Color::Rgb(80, 80, 80); // Dark gray border

    // Top border: ┌────┬────┬...┐
    let mut top_border = String::from("  ┌");
    for i in 0..8 {
        top_border.push_str("────");
        if i < 7 {
            top_border.push('┬');
        }
    }
    top_border.push('┐');
    let top = Paragraph::new(top_border).style(Style::default().fg(border_color));
    frame.render_widget(top, Rect::new(x_offset, y_offset, board_width, 1));

    // Render each rank (8 down to 1)
    for rank in (0..8).rev() {
        let y = y_offset + 1 + (7 - rank) as u16;
        let rank_label = format!("{} ", rank + 1);

        // Rank label
        let label = Paragraph::new(rank_label).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(label, Rect::new(x_offset, y, 2, 1));

        // Left border
        let left_border = Paragraph::new("│").style(Style::default().fg(border_color));
        frame.render_widget(left_border, Rect::new(x_offset + 2, y, 1, 1));

        // Squares
        for file in 0..8u8 {
            let x = x_offset + 3 + (file as u16 * cell_width);

            let is_cursor = game.cursor == (file, rank);
            let is_selected = game.selected_square == Some((file, rank));
            let is_legal_destination = game.legal_move_destinations.contains(&(file, rank));

            let piece_char = get_piece_at(&game.board, file, rank);

            // Build square content
            let (content, fg_color) = if is_cursor {
                // Cursor: show piece or brackets
                match piece_char {
                    Some(c) => {
                        let color = if is_white_piece(c) {
                            Color::White
                        } else {
                            Color::Rgb(140, 140, 140)
                        };
                        (format!("[{}]", c), color)
                    }
                    None => {
                        if is_legal_destination {
                            (" ◆  ".to_string(), Color::Rgb(200, 100, 200)) // Pink diamond for legal move under cursor
                        } else {
                            (" □  ".to_string(), Color::Rgb(100, 100, 100)) // Empty cursor
                        }
                    }
                }
            } else if is_selected {
                // Selected piece: highlight
                match piece_char {
                    Some(c) => (format!("<{}>", c), Color::Rgb(100, 200, 100)),
                    None => ("    ".to_string(), Color::Reset),
                }
            } else if is_legal_destination {
                // Legal move indicator: pink dot
                match piece_char {
                    Some(c) => {
                        let color = if is_white_piece(c) {
                            Color::White
                        } else {
                            Color::Rgb(140, 140, 140)
                        };
                        (format!(" {}  ", c), color)
                    }
                    None => (" ·  ".to_string(), Color::Rgb(200, 100, 200)), // Pink dot
                }
            } else {
                // Normal square
                match piece_char {
                    Some(c) => {
                        let color = if is_white_piece(c) {
                            Color::White
                        } else {
                            Color::Rgb(140, 140, 140)
                        };
                        (format!(" {}  ", c), color)
                    }
                    None => ("    ".to_string(), Color::Reset),
                }
            };

            let style = Style::default().fg(fg_color);
            let square = Paragraph::new(content).style(style);
            frame.render_widget(square, Rect::new(x, y, 4, 1));

            // Cell separator
            let sep = Paragraph::new("│").style(Style::default().fg(border_color));
            frame.render_widget(sep, Rect::new(x + 4, y, 1, 1));
        }
    }

    // Bottom border: └────┴────┴...┘
    let mut bottom_border = String::from("  └");
    for i in 0..8 {
        bottom_border.push_str("────");
        if i < 7 {
            bottom_border.push('┴');
        }
    }
    bottom_border.push('┘');
    let bottom = Paragraph::new(bottom_border).style(Style::default().fg(border_color));
    frame.render_widget(bottom, Rect::new(x_offset, y_offset + 9, board_width, 1));

    // File labels (A-H)
    let files = "   A    B    C    D    E    F    G    H";
    let file_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(file_labels, Rect::new(x_offset, y_offset + 10, board_width, 1));
}

fn render_status(frame: &mut Frame, area: Rect, game: &ChessGame) {
    let status_text = if game.ai_thinking {
        "Opponent is thinking..."
    } else if game.forfeit_pending {
        "Press Esc again to forfeit"
    } else if game.selected_square.is_some() {
        "Select destination (Enter to confirm, Esc to cancel)"
    } else {
        "Your move (select a piece with Enter)"
    };

    let style = if game.ai_thinking {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let status = Paragraph::new(status_text)
        .style(style)
        .alignment(Alignment::Center);
    frame.render_widget(status, area);
}

fn render_game_over_overlay(frame: &mut Frame, area: Rect, result: ChessResult, prestige: u32) {
    frame.render_widget(Clear, area);

    let (title, message, reward) = match result {
        ChessResult::Win => (
            ":: VICTORY! ::",
            "You checkmated the mysterious figure!",
            format!("+{} Prestige Ranks", prestige),
        ),
        ChessResult::Loss => (
            "DEFEAT",
            "The mysterious figure has checkmated you.",
            "No penalty incurred.".to_string(),
        ),
        ChessResult::Draw => (
            "DRAW",
            "The game ends in stalemate.",
            "+5000 XP".to_string(),
        ),
        ChessResult::Forfeit => (
            "FORFEIT",
            "You conceded the game.",
            "No penalty incurred.".to_string(),
        ),
    };

    let title_color = match result {
        ChessResult::Win => Color::Green,
        ChessResult::Loss => Color::Red,
        ChessResult::Draw => Color::Yellow,
        ChessResult::Forfeit => Color::Gray,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content_height: u16 = 7;
    let y_offset = inner.y + (inner.height.saturating_sub(content_height)) / 2;

    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(reward, Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from(Span::styled(
            "[Press any key]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(
        text,
        Rect::new(inner.x, y_offset, inner.width, content_height),
    );
}

/// Check if a piece character represents a white piece
fn is_white_piece(c: char) -> bool {
    // In chess-engine crate, white pieces use hollow symbols (confusingly reversed from Unicode standard)
    // Based on the Display impl in piece.rs: White uses filled symbols, Black uses hollow
    // Actually looking at the code: Color::Black uses hollow (filled-looking), Color::White uses filled (outline)
    // The crate swaps them: Black -> hollow symbols, White -> filled symbols
    matches!(c, '\u{265A}'..='\u{265F}') // Black Unicode symbols (hollow) are used for White pieces
}

/// Get the piece character at a specific square from the chess-engine Board
fn get_piece_at(board: &chess_engine::Board, file: u8, rank: u8) -> Option<char> {
    use chess_engine::{Color as ChessColor, Piece, Position};

    let position = Position::new(rank as i32, file as i32);

    board.get_piece(position).map(|piece| {
        // Note: chess-engine swaps Unicode symbols:
        // - Color::White pieces display with filled symbols (normally black in Unicode)
        // - Color::Black pieces display with hollow symbols (normally white in Unicode)
        match piece {
            Piece::King(ChessColor::White, _) => '\u{265A}', // Black king symbol for white
            Piece::Queen(ChessColor::White, _) => '\u{265B}', // Black queen symbol for white
            Piece::Rook(ChessColor::White, _) => '\u{265C}', // Black rook symbol for white
            Piece::Bishop(ChessColor::White, _) => '\u{265D}', // Black bishop symbol for white
            Piece::Knight(ChessColor::White, _) => '\u{265E}', // Black knight symbol for white
            Piece::Pawn(ChessColor::White, _) => '\u{265F}', // Black pawn symbol for white
            Piece::King(ChessColor::Black, _) => '\u{2654}', // White king symbol for black
            Piece::Queen(ChessColor::Black, _) => '\u{2655}', // White queen symbol for black
            Piece::Rook(ChessColor::Black, _) => '\u{2656}', // White rook symbol for black
            Piece::Bishop(ChessColor::Black, _) => '\u{2657}', // White bishop symbol for black
            Piece::Knight(ChessColor::Black, _) => '\u{2658}', // White knight symbol for black
            Piece::Pawn(ChessColor::Black, _) => '\u{2659}', // White pawn symbol for black
        }
    })
}
