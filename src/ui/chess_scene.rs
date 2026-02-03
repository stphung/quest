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
    let board_width: u16 = 34; // 2 (rank labels) + 8*4 (squares)
    let board_height: u16 = 10;

    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    // File labels (4-char spacing)
    let files = "   a   b   c   d   e   f   g   h";
    let top_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(top_labels, Rect::new(x_offset, y_offset, board_width, 1));

    // Render each rank (8 down to 1)
    for rank in (0..8).rev() {
        let y = y_offset + 1 + (7 - rank) as u16;
        let rank_label = format!("{}", rank + 1);

        // Left rank label
        let label = Paragraph::new(rank_label.clone()).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(label, Rect::new(x_offset, y, 1, 1));

        // Squares (4 chars each)
        for file in 0..8u8 {
            let x = x_offset + 2 + (file as u16 * 4);

            let is_light = (file + rank) % 2 == 1;
            let is_cursor = game.cursor == (file, rank);
            let is_selected = game.selected_square == Some((file, rank));
            let is_legal_destination = game.legal_move_destinations.contains(&(file, rank));

            let bg_color = if is_cursor {
                if is_legal_destination {
                    Color::Rgb(200, 200, 50) // Yellow-green for cursor on legal move
                } else {
                    Color::Yellow
                }
            } else if is_selected {
                Color::Green
            } else if is_legal_destination {
                Color::Rgb(144, 238, 144) // Light green for legal moves
            } else if is_light {
                Color::Rgb(200, 200, 180)
            } else {
                Color::Rgb(120, 80, 50)
            };

            let piece_char = get_piece_at(&game.board, file, rank);
            let piece_str = piece_char
                .map(|c| c.to_string())
                .unwrap_or_else(|| " ".to_string());

            let fg_color = if piece_char.map(is_white_piece).unwrap_or(false) {
                Color::White
            } else {
                Color::Black
            };

            let style = Style::default().fg(fg_color).bg(bg_color);
            let square = Paragraph::new(format!(" {}  ", piece_str)).style(style);
            frame.render_widget(square, Rect::new(x, y, 4, 1));
        }

        // Right rank label
        let label_r = Paragraph::new(rank_label).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(label_r, Rect::new(x_offset + 34, y, 1, 1));
    }

    // Bottom file labels
    let bottom_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        bottom_labels,
        Rect::new(x_offset, y_offset + 9, board_width, 1),
    );
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
