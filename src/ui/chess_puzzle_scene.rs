//! Chess Puzzle board UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_banner,
    render_info_panel_frame, render_status_bar, render_thinking_status_bar, GameResultType,
};
use crate::challenges::chess_puzzle::puzzles::get_puzzles;
use crate::challenges::chess_puzzle::{ChessPuzzleGame, ChessPuzzleResult, PuzzleState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Render the chess puzzle scene.
pub fn render_chess_puzzle_scene(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    if game.game_result.is_some() {
        render_game_over(frame, area, game);
        return;
    }

    // Content: 1 for progress + 1 for title + 18 for board = 20
    let layout = create_game_layout(frame, area, " Chess Puzzles ", Color::LightGreen, 20, 22);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Progress line
            Constraint::Length(1), // Puzzle title
            Constraint::Min(18),   // Board
        ])
        .split(layout.content);

    render_progress(frame, content_chunks[0], game);
    render_puzzle_title(frame, content_chunks[1], game);
    render_board(frame, content_chunks[2], game);
    render_status(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_progress(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    let spans = vec![
        Span::styled(
            format!(
                "Puzzle {}/{} ",
                game.current_puzzle_index + 1,
                game.total_puzzles
            ),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("Solved: {}/{}", game.puzzles_solved, game.target_score),
            Style::default().fg(Color::LightGreen),
        ),
    ];
    let text = Paragraph::new(Line::from(spans)).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, area);
}

fn render_puzzle_title(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    if game.current_puzzle_index >= game.puzzle_order.len() {
        return;
    }
    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    let text = Paragraph::new(Line::from(Span::styled(
        format!("\"{}\"", puzzle.title),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
    )))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, area);
}

fn render_board(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    let cell_width: u16 = 5;
    let board_width: u16 = 3 + (cell_width * 8) + 1;
    let board_height: u16 = 18;

    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    let border_color = Color::Rgb(80, 80, 80);
    let from_move_color = Color::Rgb(180, 140, 80);
    let to_move_color = Color::Rgb(255, 255, 100);

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

    // Top border
    let mut top_border = String::from("  \u{250C}");
    for i in 0..8 {
        top_border.push_str("\u{2500}\u{2500}\u{2500}\u{2500}");
        if i < 7 {
            top_border.push('\u{252C}');
        }
    }
    top_border.push('\u{2510}');
    let top = Paragraph::new(top_border).style(Style::default().fg(border_color));
    frame.render_widget(top, Rect::new(x_offset, y_offset, board_width, 1));

    for rank in (0..8).rev() {
        let row_index = 7 - rank;
        let y = y_offset + 1 + (row_index as u16 * 2);
        let rank_label = format!("{} ", rank + 1);

        let label = Paragraph::new(rank_label).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(label, Rect::new(x_offset, y, 2, 1));

        let left_border = Paragraph::new("\u{2502}").style(Style::default().fg(border_color));
        frame.render_widget(left_border, Rect::new(x_offset + 2, y, 1, 1));

        for file in 0..8u8 {
            let x = x_offset + 3 + (file as u16 * cell_width);

            let is_cursor = game.cursor == (file, rank);
            let is_selected = game.selected_square == Some((file, rank));
            let is_legal_destination = game.legal_move_destinations.contains(&(file, rank));
            let highlight_color = get_highlight_color(file, rank);
            let is_from_square = highlight_color == Some(from_move_color);
            let is_last_move = highlight_color.is_some();

            let piece_char = get_piece_at(&game.board, file, rank);

            let (content, fg_color) = if is_cursor {
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
                    None if is_legal_destination => {
                        (" \u{25C6}  ".to_string(), Color::Rgb(200, 100, 200))
                    }
                    None => (" \u{25A1}  ".to_string(), color),
                }
            } else if is_selected {
                match piece_char {
                    Some(c) => (format!("<{}>", c), Color::Rgb(100, 200, 100)),
                    None => ("    ".to_string(), Color::Reset),
                }
            } else if is_legal_destination {
                match piece_char {
                    Some(c) => (format!(" {}  ", c), piece_color(c)),
                    None => (" \u{00B7}  ".to_string(), Color::Rgb(200, 100, 200)),
                }
            } else if is_last_move {
                let move_color = if is_from_square {
                    from_move_color
                } else {
                    to_move_color
                };
                match piece_char {
                    Some(c) => (format!(" {}  ", c), move_color),
                    None if is_from_square => (" \u{00B7}  ".to_string(), from_move_color),
                    None => ("    ".to_string(), Color::Reset),
                }
            } else {
                match piece_char {
                    Some(c) => (format!(" {}  ", c), piece_color(c)),
                    None => ("    ".to_string(), Color::Reset),
                }
            };

            let style = Style::default().fg(fg_color);
            let square = Paragraph::new(content).style(style);
            frame.render_widget(square, Rect::new(x, y, 4, 1));

            let sep = Paragraph::new("\u{2502}").style(Style::default().fg(border_color));
            frame.render_widget(sep, Rect::new(x + 4, y, 1, 1));
        }

        if rank > 0 {
            let mut sep_line = String::from("  \u{251C}");
            for file in 0..8 {
                sep_line.push_str("\u{2500}\u{2500}\u{2500}\u{2500}");
                if file < 7 {
                    sep_line.push('\u{253C}');
                }
            }
            sep_line.push('\u{2524}');
            let sep = Paragraph::new(sep_line).style(Style::default().fg(border_color));
            frame.render_widget(sep, Rect::new(x_offset, y + 1, board_width, 1));
        }
    }

    // Bottom border
    let mut bottom_border = String::from("  \u{2514}");
    for i in 0..8 {
        bottom_border.push_str("\u{2500}\u{2500}\u{2500}\u{2500}");
        if i < 7 {
            bottom_border.push('\u{2534}');
        }
    }
    bottom_border.push('\u{2518}');
    let bottom = Paragraph::new(bottom_border).style(Style::default().fg(border_color));
    frame.render_widget(bottom, Rect::new(x_offset, y_offset + 16, board_width, 1));

    // File labels
    let files = "   A    B    C    D    E    F    G    H";
    let file_labels = Paragraph::new(files).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        file_labels,
        Rect::new(x_offset, y_offset + 17, board_width, 1),
    );
}

fn render_status(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent responds...");
        return;
    }

    match game.puzzle_state {
        PuzzleState::Correct => {
            render_status_bar(frame, area, "Correct!", Color::Green, &[]);
            return;
        }
        PuzzleState::Wrong => {
            render_status_bar(frame, area, "Wrong move", Color::Red, &[]);
            return;
        }
        _ => {}
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let (status_text, status_color) = if game.selected_square.is_some() {
        ("Select destination", Color::Cyan)
    } else {
        ("Find the best move", Color::White)
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

fn render_info_panel(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    let inner = render_info_panel_frame(frame, area);

    if game.current_puzzle_index >= game.puzzle_order.len() {
        return;
    }
    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    let lines = vec![
        Line::from(Span::styled(
            "PUZZLE",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(puzzle.hint, Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.difficulty.name(),
                Style::default().fg(Color::LightGreen),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Solved: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.puzzles_solved, game.target_score),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Puzzle: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.current_puzzle_index + 1, game.total_puzzles),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let text = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(text, inner);
}

fn render_game_over(frame: &mut Frame, area: Rect, game: &ChessPuzzleGame) {
    use ratatui::widgets::Clear;

    frame.render_widget(Clear, area);

    let layout = create_game_layout(frame, area, " Chess Puzzles ", Color::LightGreen, 20, 22);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(18),
        ])
        .split(layout.content);

    render_progress(frame, content_chunks[0], game);
    render_puzzle_title(frame, content_chunks[1], game);
    render_board(frame, content_chunks[2], game);
    render_info_panel(frame, layout.info_panel, game);

    let result = game.game_result.unwrap();
    let prestige = game.difficulty.reward_prestige();

    let (result_type, title, message, reward) = match result {
        ChessPuzzleResult::Win => (
            GameResultType::Win,
            "PUZZLES COMPLETE!",
            format!("Solved {}/{}", game.puzzles_solved, game.total_puzzles),
            format!("+{} Prestige Ranks", prestige),
        ),
        ChessPuzzleResult::Loss => {
            if game.forfeit_pending {
                (
                    GameResultType::Forfeit,
                    "FORFEIT",
                    "You gave up".to_string(),
                    String::new(),
                )
            } else {
                (
                    GameResultType::Loss,
                    "FAILED",
                    format!("Solved {}/{}", game.puzzles_solved, game.target_score),
                    String::new(),
                )
            }
        }
    };

    render_game_over_banner(
        frame,
        content_chunks[2],
        result_type,
        title,
        &message,
        &reward,
    );
}

/// Get color for a piece character.
fn piece_color(c: char) -> Color {
    if matches!(c, '\u{265A}'..='\u{265F}') {
        Color::White
    } else {
        Color::Rgb(140, 140, 140)
    }
}

/// Get the piece character at a specific square.
fn get_piece_at(board: &chess_engine::Board, file: u8, rank: u8) -> Option<char> {
    use chess_engine::{Color as ChessColor, Piece, Position};

    let position = Position::new(rank as i32, file as i32);

    board.get_piece(position).map(|piece| match piece {
        Piece::King(ChessColor::White, _) => '\u{265A}',
        Piece::Queen(ChessColor::White, _) => '\u{265B}',
        Piece::Rook(ChessColor::White, _) => '\u{265C}',
        Piece::Bishop(ChessColor::White, _) => '\u{265D}',
        Piece::Knight(ChessColor::White, _) => '\u{265E}',
        Piece::Pawn(ChessColor::White, _) => '\u{265F}',
        Piece::King(ChessColor::Black, _) => '\u{2654}',
        Piece::Queen(ChessColor::Black, _) => '\u{2655}',
        Piece::Rook(ChessColor::Black, _) => '\u{2656}',
        Piece::Bishop(ChessColor::Black, _) => '\u{2657}',
        Piece::Knight(ChessColor::Black, _) => '\u{2658}',
        Piece::Pawn(ChessColor::Black, _) => '\u{2659}',
    })
}
