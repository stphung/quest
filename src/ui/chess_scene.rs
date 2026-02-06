//! Chess board UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_banner,
    render_info_panel_frame, render_status_bar, render_thinking_status_bar, GameResultType,
};
use crate::challenges::chess::{ChessGame, ChessResult};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the chess game scene
pub fn render_chess_scene(frame: &mut Frame, area: Rect, game: &ChessGame) {
    // Check for game over - show board with banner
    if game.game_result.is_some() {
        render_chess_game_over(frame, area, game);
        return;
    }

    // Use shared layout (content needs 19 lines: 1 for move history + 18 for board)
    let layout = create_game_layout(frame, area, " Chess ", Color::Cyan, 19, 22);

    // Split content area: move history on top, board below
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Move history (single line)
            Constraint::Min(18),   // Board
        ])
        .split(layout.content);

    render_move_history(frame, content_chunks[0], game);
    render_board(frame, content_chunks[1], game);
    render_status(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &ChessGame) {
    // Clean grid style: 5 chars per square + borders + rank label
    let cell_width: u16 = 5;
    let board_width: u16 = 3 + (cell_width * 8) + 1; // rank label + 8 cells + right border
    let board_height: u16 = 18; // 8 rows + 9 horizontal lines + file labels

    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    let border_color = Color::Rgb(80, 80, 80); // Dark gray border
    let from_move_color = Color::Rgb(180, 140, 80); // Dim orange for source square
    let to_move_color = Color::Rgb(255, 255, 100); // Bright yellow for destination square

    // Helper to get the highlight color for a square based on last move
    let get_highlight_color = |file: u8, rank: u8| -> Option<Color> {
        let (from, to) = game.last_move?;
        if (file, rank) == from {
            Some(from_move_color)
        } else if (file, rank) == to {
            Some(to_move_color)
        } else {
            None
        }
    };

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

    // Render each rank (8 down to 1) with horizontal separators
    for rank in (0..8).rev() {
        let row_index = 7 - rank;
        let y = y_offset + 1 + (row_index as u16 * 2); // Each rank takes 2 rows (content + separator)
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
            let highlight_color = get_highlight_color(file, rank);
            let is_from_square = highlight_color == Some(from_move_color);
            let is_last_move = highlight_color.is_some();

            let piece_char = get_piece_at(&game.board, file, rank);

            // Build square content
            let (content, fg_color) = if is_cursor {
                // Cursor: show piece or brackets, preserve last-move color
                let color = if is_last_move {
                    if is_from_square {
                        from_move_color
                    } else {
                        to_move_color
                    }
                } else {
                    piece_char
                        .map(piece_color)
                        .unwrap_or(Color::Rgb(100, 100, 100))
                };
                match piece_char {
                    Some(c) => (format!("[{}]", c), color),
                    None if is_legal_destination => (" ◆  ".to_string(), Color::Rgb(200, 100, 200)),
                    None => (" □  ".to_string(), color),
                }
            } else if is_selected {
                // Selected piece: highlight green
                match piece_char {
                    Some(c) => (format!("<{}>", c), Color::Rgb(100, 200, 100)),
                    None => ("    ".to_string(), Color::Reset),
                }
            } else if is_legal_destination {
                // Legal move indicator: pink dot or piece
                match piece_char {
                    Some(c) => (format!(" {}  ", c), piece_color(c)),
                    None => (" ·  ".to_string(), Color::Rgb(200, 100, 200)),
                }
            } else if is_last_move {
                // Last move square: highlight with from/to colors
                let move_color = if is_from_square {
                    from_move_color
                } else {
                    to_move_color
                };
                match piece_char {
                    Some(c) => (format!(" {}  ", c), move_color),
                    None if is_from_square => (" ·  ".to_string(), from_move_color),
                    None => ("    ".to_string(), Color::Reset),
                }
            } else {
                // Normal square
                match piece_char {
                    Some(c) => (format!(" {}  ", c), piece_color(c)),
                    None => ("    ".to_string(), Color::Reset),
                }
            };

            let style = Style::default().fg(fg_color);
            let square = Paragraph::new(content).style(style);
            frame.render_widget(square, Rect::new(x, y, 4, 1));

            // Cell separator (vertical)
            let sep = Paragraph::new("│").style(Style::default().fg(border_color));
            frame.render_widget(sep, Rect::new(x + 4, y, 1, 1));
        }

        // Horizontal separator after each row (except last)
        if rank > 0 {
            let mut sep_line = String::from("  ├");
            for file in 0..8 {
                sep_line.push_str("────");
                if file < 7 {
                    sep_line.push('┼');
                }
            }
            sep_line.push('┤');
            let sep = Paragraph::new(sep_line).style(Style::default().fg(border_color));
            frame.render_widget(sep, Rect::new(x_offset, y + 1, board_width, 1));
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
    frame.render_widget(bottom, Rect::new(x_offset, y_offset + 16, board_width, 1));

    // File labels (A-H)
    let files = "   A    B    C    D    E    F    G    H";
    let file_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        file_labels,
        Rect::new(x_offset, y_offset + 17, board_width, 1),
    );
}

fn render_status(frame: &mut Frame, area: Rect, game: &ChessGame) {
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent is thinking...");
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let (status_text, status_color) = if game.selected_square.is_some() {
        ("Select destination", Color::Cyan)
    } else {
        ("Your move", Color::White)
    };

    let controls: &[(&str, &str)] = if game.selected_square.is_some() {
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Confirm"),
            ("[Esc]", "Cancel"),
        ]
    } else {
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Select"),
            ("[Esc]", "Forfeit"),
        ]
    };

    render_status_bar(frame, area, status_text, status_color, controls);
}

fn render_move_history(frame: &mut Frame, area: Rect, game: &ChessGame) {
    if game.move_history.is_empty() {
        let text = Paragraph::new("Moves: -")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
        return;
    }

    // Build horizontal move list: "Moves: 4.Ba4 Nf6  3.Bb5 a6  2.Nf3 Nc6  1.e4 e5"
    let moves = &game.move_history;
    let last_move_idx = moves.len() - 1;

    // Style for highlighting the most recent move (matches board highlight)
    let highlight_style = Style::default()
        .fg(Color::Rgb(255, 255, 100))
        .add_modifier(Modifier::BOLD);

    let mut spans: Vec<Span> = vec![Span::styled(
        "Moves: ",
        Style::default().fg(Color::DarkGray),
    )];

    // Build move pairs in reverse order (most recent first)
    let num_pairs = moves.len().div_ceil(2);
    for i in (0..num_pairs).rev() {
        let white_idx = i * 2;
        let black_idx = i * 2 + 1;

        // Move number
        spans.push(Span::styled(
            format!("{}.", i + 1),
            Style::default().fg(Color::DarkGray),
        ));

        // White's move
        let white_style = if white_idx == last_move_idx {
            highlight_style
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(moves[white_idx].clone(), white_style));

        // Black's move (if exists)
        if let Some(black_move) = moves.get(black_idx) {
            spans.push(Span::styled(" ", Style::default()));
            let black_style = if black_idx == last_move_idx {
                highlight_style
            } else {
                Style::default().fg(Color::Gray)
            };
            spans.push(Span::styled(black_move.clone(), black_style));
        }

        // Separator between move pairs
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }
    }

    let text = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(text, area);
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &ChessGame) {
    let inner = render_info_panel_frame(frame, area);

    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "RULES",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Checkmate the enemy",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "king to win.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("You: ", Style::default().fg(Color::White)),
            Span::styled(
                "♚♛♜♝♞♟",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Foe: ", Style::default().fg(Color::Gray)),
            Span::styled("♔♕♖♗♘♙", Style::default().fg(Color::Rgb(140, 140, 140))),
        ]),
    ];

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

fn render_chess_game_over(frame: &mut Frame, area: Rect, game: &ChessGame) {
    use ratatui::widgets::Clear;

    // First render the board showing checkmate position
    frame.render_widget(Clear, area);

    // Create layout matching normal game
    let layout = create_game_layout(frame, area, " Chess ", Color::Cyan, 19, 22);

    // Split content area: move history on top, board below
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Move history
            Constraint::Min(18),   // Board
        ])
        .split(layout.content);

    render_move_history(frame, content_chunks[0], game);
    render_board(frame, content_chunks[1], game);
    render_info_panel(frame, layout.info_panel, game);

    let result = game.game_result.unwrap();
    let prestige = game.difficulty.reward_prestige();
    let (result_type, title, message, reward) = match result {
        ChessResult::Win => (
            GameResultType::Win,
            "VICTORY!",
            "Checkmate!",
            format!("+{} Prestige Ranks", prestige),
        ),
        ChessResult::Loss => (GameResultType::Loss, "DEFEAT", "Checkmate", String::new()),
        ChessResult::Draw => (
            GameResultType::Draw,
            "DRAW",
            "Stalemate",
            "+5000 XP".to_string(),
        ),
        ChessResult::Forfeit => (
            GameResultType::Forfeit,
            "FORFEIT",
            "You conceded",
            String::new(),
        ),
    };

    // Render banner at bottom of board area
    render_game_over_banner(
        frame,
        content_chunks[1],
        result_type,
        title,
        message,
        &reward,
    );
}

/// Get color for a piece character (white pieces are bright, black pieces are dim)
fn piece_color(c: char) -> Color {
    // chess-engine swaps Unicode symbols: Black symbols (hollow) are used for White pieces
    if matches!(c, '\u{265A}'..='\u{265F}') {
        Color::White
    } else {
        Color::Rgb(140, 140, 140)
    }
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
