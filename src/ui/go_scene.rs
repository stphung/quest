//! Go game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_banner,
    render_info_panel_frame, render_minigame_too_small, render_status_bar,
    render_thinking_status_bar, GameResultType,
};
use crate::challenges::go::{GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use crate::challenges::menu::DifficultyInfo;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the Go game scene.
pub fn render_go_scene(
    frame: &mut Frame,
    area: Rect,
    game: &GoGame,
    ctx: &super::responsive::LayoutContext,
) {
    // Game over overlay
    if game.game_result.is_some() {
        render_go_game_over(frame, area, game, ctx);
        return;
    }

    const MIN_WIDTH: u16 = 27;
    const MIN_HEIGHT: u16 = 14;
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Go", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    // Use shared layout - Go board needs width for box drawing chars
    let layout = create_game_layout(frame, area, " Go ", Color::Green, 11, 24, ctx);

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &GoGame) {
    let board_height = BOARD_SIZE as u16;
    let board_width = (BOARD_SIZE * 3 - 2) as u16; // "●──" format
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;
    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;

    let human_color = Color::White;
    let ai_color = Color::LightRed;
    let cursor_color = Color::Yellow;
    let last_move_color = Color::Green;
    let grid_color = Color::DarkGray;

    for row in 0..BOARD_SIZE {
        let mut spans = Vec::new();
        for col in 0..BOARD_SIZE {
            let is_cursor = game.cursor == (row, col);
            let is_last_move = game.last_move == Some(GoMove::Place(row, col));
            let is_ko = game.ko_point == Some((row, col));

            // Determine the intersection character
            let (symbol, style) = match game.board[row][col] {
                Some(Stone::Black) => {
                    let base_style = Style::default()
                        .fg(human_color)
                        .add_modifier(Modifier::BOLD);
                    if is_cursor {
                        ("●", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("●", base_style.fg(last_move_color))
                    } else {
                        ("●", base_style)
                    }
                }
                Some(Stone::White) => {
                    let base_style = Style::default().fg(ai_color).add_modifier(Modifier::BOLD);
                    if is_cursor {
                        ("○", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("○", base_style.fg(last_move_color))
                    } else {
                        ("○", base_style)
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
                    } else if is_ko {
                        ("×", Style::default().fg(Color::Red))
                    } else {
                        // Grid intersection
                        let ch = get_intersection_char(row, col);
                        (ch, Style::default().fg(grid_color))
                    }
                }
            };

            spans.push(Span::styled(symbol, style));

            // Add horizontal line between intersections
            if col < BOARD_SIZE - 1 {
                spans.push(Span::styled("──", Style::default().fg(grid_color)));
            }
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(x_offset, y_offset + row as u16, board_width, 1),
        );
    }
}

/// Get the appropriate intersection character based on position.
fn get_intersection_char(row: usize, col: usize) -> &'static str {
    let is_top = row == 0;
    let is_bottom = row == BOARD_SIZE - 1;
    let is_left = col == 0;
    let is_right = col == BOARD_SIZE - 1;

    match (is_top, is_bottom, is_left, is_right) {
        (true, _, true, _) => "┌",
        (true, _, _, true) => "┐",
        (_, true, true, _) => "└",
        (_, true, _, true) => "┘",
        (true, _, _, _) => "┬",
        (_, true, _, _) => "┴",
        (_, _, true, _) => "├",
        (_, _, _, true) => "┤",
        _ => "┼",
    }
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &GoGame) {
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent is thinking...");
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    // Show pass feedback when opponent passed
    let (status_text, status_color) = if game.last_move == Some(GoMove::Pass) {
        ("Opponent passed - Your turn", Color::Yellow)
    } else {
        ("Your turn", Color::White)
    };

    render_status_bar(
        frame,
        area,
        status_text,
        status_color,
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Place"),
            ("[P]", "Pass"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &GoGame) {
    let inner = render_info_panel_frame(frame, area);

    let mut lines: Vec<Line> = vec![
        // Minimal rules (2-rule style)
        Line::from(Span::styled(
            "Surround to capture",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "Most territory wins",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        // Players
        Line::from(vec![
            Span::styled(
                "● ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("You", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                "○ ",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Opponent", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        // Captures (split for clarity)
        Line::from(vec![Span::styled(
            "Captured:",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![
            Span::styled(" You: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.captured_by_black),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Foe: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.captured_by_white),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        // Difficulty
        Line::from(vec![Span::styled(
            game.difficulty.name(),
            Style::default().fg(Color::Cyan),
        )]),
    ];

    // Show pass indicator when opponent passed
    if game.last_move == Some(GoMove::Pass) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            ">> Foe passed! <<",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_go_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &GoGame,
    ctx: &super::responsive::LayoutContext,
) {
    use crate::challenges::go::logic::calculate_score;
    use ratatui::widgets::Clear;

    // First render the board showing final territory
    frame.render_widget(Clear, area);

    // Create layout matching normal game
    let layout = create_game_layout(frame, area, " Go ", Color::Green, 11, 24, ctx);

    // Render board and info panel (territory will be visible)
    render_board(frame, layout.content, game);
    render_info_panel(frame, layout.info_panel, game);

    let result = game.game_result.as_ref().unwrap();

    // Get final scores for message (human=Black, AI=White)
    let (black_score, white_score) = calculate_score(&game.board);
    let score_msg = format!("You: {} vs AI: {}", black_score, white_score);

    let (result_type, title, message) = match result {
        GoResult::Win => (GameResultType::Win, "VICTORY!", score_msg),
        GoResult::Loss => (GameResultType::Loss, "DEFEAT", score_msg),
        GoResult::Draw => (GameResultType::Draw, "DRAW", score_msg),
    };

    let reward = match result {
        GoResult::Win => game.difficulty.reward().description().replace("Win: ", ""),
        _ => String::new(),
    };

    // Render banner at bottom of content area
    render_game_over_banner(frame, layout.content, result_type, title, &message, &reward);
}
