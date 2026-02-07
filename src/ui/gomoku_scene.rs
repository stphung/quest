//! Gomoku game UI rendering.

use super::board_styles::{calculate_board_centering, symbols, BoardColors};
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_banner,
    render_info_panel_frame, render_status_bar, render_thinking_status_bar, GameResultType,
};
use crate::challenges::gomoku::{GomokuGame, Player, BOARD_SIZE};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Shared colors for Gomoku board rendering
const COLORS: BoardColors = BoardColors {
    human: Color::White,
    ai: Color::LightRed,
    cursor: Color::Yellow,
    last_move: Color::Green,
    empty: Color::DarkGray,
    winning: Color::Magenta,
};

/// Render the Gomoku game scene.
pub fn render_gomoku_scene(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    // Game over overlay
    if game.game_result.is_some() {
        render_gomoku_game_over(frame, area, game);
        return;
    }

    // Use shared layout
    let layout = create_game_layout(frame, area, " Gomoku ", Color::Cyan, 15, 22);

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    render_board_with_highlight(frame, area, game, false);
}

fn render_board_with_highlight(
    frame: &mut Frame,
    area: Rect,
    game: &GomokuGame,
    show_winning_line: bool,
) {
    // Calculate centering using shared utility
    let board_height = BOARD_SIZE as u16;
    let board_width = (BOARD_SIZE * 2 - 1) as u16; // "● " format
    let (x_offset, y_offset) = calculate_board_centering(
        area.x,
        area.y,
        area.width,
        area.height,
        board_width,
        board_height,
    );

    // Check if position is in winning line
    let is_winning_pos = |row: usize, col: usize| -> bool {
        show_winning_line
            && game
                .winning_line
                .as_ref()
                .is_some_and(|line| line.contains(&(row, col)))
    };

    // Draw board
    for row in 0..BOARD_SIZE {
        let mut spans = Vec::new();
        for col in 0..BOARD_SIZE {
            let is_cursor = game.cursor == (row, col) && !show_winning_line;
            let is_last_move = game.last_move == Some((row, col)) && !show_winning_line;
            let is_winning = is_winning_pos(row, col);

            let (symbol, style) = match game.board[row][col] {
                Some(Player::Human) => {
                    let style = COLORS.piece_style(true, is_cursor, is_last_move, is_winning);
                    (symbols::FILLED_CIRCLE, style)
                }
                Some(Player::Ai) => {
                    let style = COLORS.piece_style(false, is_cursor, is_last_move, is_winning);
                    (symbols::FILLED_CIRCLE, style)
                }
                None => {
                    if is_cursor {
                        (symbols::CURSOR_SQUARE, COLORS.cursor_style())
                    } else {
                        (symbols::EMPTY_DOT, COLORS.empty_style(false))
                    }
                }
            };

            spans.push(Span::styled(symbol, style));
            if col < BOARD_SIZE - 1 {
                spans.push(Span::raw(" "));
            }
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(x_offset, y_offset + row as u16, board_width, 1),
        );
    }
}

/// Render the status bar below the board.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent is thinking...");
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Your turn",
        Color::White,
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Place"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    let inner = render_info_panel_frame(frame, area);

    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "RULES",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Place stones. First",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "to get 5 in a row",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled("wins.", Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("You: ", Style::default().fg(Color::White)),
            Span::styled(
                "●",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  AI: ", Style::default().fg(Color::Gray)),
            Span::styled("●", Style::default().fg(Color::LightRed)),
        ]),
    ];

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

fn render_gomoku_game_over(frame: &mut Frame, area: Rect, game: &GomokuGame) {
    use crate::challenges::menu::DifficultyInfo;
    use ratatui::widgets::Clear;

    // First render the board with winning line highlighted
    frame.render_widget(Clear, area);

    // Create layout matching normal game (but without status bar interaction)
    let layout = create_game_layout(frame, area, " Gomoku ", Color::Cyan, 15, 22);

    // Render board with winning line highlighted
    render_board_with_highlight(frame, layout.content, game, true);
    render_info_panel(frame, layout.info_panel, game);

    let result = game.game_result.as_ref().unwrap();
    let (result_type, title, message) = match result {
        crate::challenges::gomoku::GomokuResult::Win => {
            (GameResultType::Win, "VICTORY!", "Five in a row!")
        }
        crate::challenges::gomoku::GomokuResult::Loss => {
            (GameResultType::Loss, "DEFEAT", "Opponent got five in a row")
        }
        crate::challenges::gomoku::GomokuResult::Draw => {
            (GameResultType::Draw, "DRAW", "Board full, no winner")
        }
    };

    let reward = match result {
        crate::challenges::gomoku::GomokuResult::Win => {
            game.difficulty.reward().description().replace("Win: ", "")
        }
        _ => String::new(),
    };

    // Render banner at bottom of content area (not status bar, which is too small)
    render_game_over_banner(frame, layout.content, result_type, title, message, &reward);
}
