//! Gomoku game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_banner,
    render_info_panel_frame, render_status_bar, render_thinking_status_bar, GameResultType,
};
use crate::challenges::gomoku::{GomokuGame, Player, BOARD_SIZE};
use crate::challenges::ChallengeResult;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
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
    // Calculate centering offset (no border - outer block provides it)
    let board_height = BOARD_SIZE as u16;
    let board_width = (BOARD_SIZE * 2 - 1) as u16; // "● " format
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;
    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;

    // Colors
    let human_color = Color::White;
    let ai_color = Color::LightRed;
    let cursor_color = Color::Yellow;
    let last_move_color = Color::Green;
    let empty_color = Color::DarkGray;
    let winning_line_color = Color::Magenta;

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
                    let base_style = Style::default()
                        .fg(human_color)
                        .add_modifier(Modifier::BOLD);
                    if is_winning {
                        (
                            "●",
                            Style::default()
                                .fg(winning_line_color)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else if is_cursor {
                        ("●", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("●", base_style.fg(last_move_color))
                    } else {
                        ("●", base_style)
                    }
                }
                Some(Player::Ai) => {
                    let base_style = Style::default().fg(ai_color).add_modifier(Modifier::BOLD);
                    if is_winning {
                        (
                            "●",
                            Style::default()
                                .fg(winning_line_color)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else if is_cursor {
                        ("●", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("●", base_style.fg(last_move_color))
                    } else {
                        ("●", base_style)
                    }
                }
                None => {
                    if is_cursor {
                        (
                            "□",
                            Style::default()
                                .fg(cursor_color)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        ("·", Style::default().fg(empty_color))
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
        ChallengeResult::Win => (GameResultType::Win, "VICTORY!", "Five in a row!"),
        ChallengeResult::Loss | ChallengeResult::Forfeit => {
            (GameResultType::Loss, "DEFEAT", "Opponent got five in a row")
        }
        ChallengeResult::Draw => (GameResultType::Draw, "DRAW", "Board full, no winner"),
    };

    let reward = match result {
        ChallengeResult::Win => crate::challenges::menu::ChallengeType::Gomoku
            .reward(game.difficulty)
            .description()
            .replace("Win: ", ""),
        _ => String::new(),
    };

    // Render banner at bottom of content area (not status bar, which is too small)
    render_game_over_banner(frame, layout.content, result_type, title, message, &reward);
}
