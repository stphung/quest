//! Nine Men's Morris UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_banner,
    render_info_panel_frame, render_minigame_too_small, render_status_bar,
    render_thinking_status_bar, GameResultType,
};
use crate::challenges::morris::{
    MorrisGame, MorrisMove, MorrisPhase, MorrisResult, Player, ADJACENCIES,
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the Nine Men's Morris game scene
pub fn render_morris_scene(
    frame: &mut Frame,
    area: Rect,
    game: &MorrisGame,
    character_level: u32,
    ctx: &super::responsive::LayoutContext,
) {
    // Check for game over - show board with banner
    if game.game_result.is_some() {
        let xp_for_level = crate::core::game_logic::xp_for_next_level(character_level.max(1));
        let xp_reward =
            (xp_for_level as f64 * game.difficulty.reward_xp_percent() as f64 / 100.0) as u64;
        let xp_reward = xp_reward.max(100);
        let is_master = game.difficulty == crate::challenges::morris::MorrisDifficulty::Master;
        render_morris_game_over(frame, area, game, xp_reward, is_master, ctx);
        return;
    }

    const MIN_WIDTH: u16 = 27;
    const MIN_HEIGHT: u16 = 16;
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Nine Men's Morris", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    // Use shared layout
    let layout = create_game_layout(frame, area, " Nine Men's Morris ", Color::Cyan, 13, 24, ctx);

    render_board(frame, layout.content, game, false);
    render_status(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Position coordinates for rendering the 24 board positions
/// Maps position index (0-23) to (x, y) coordinates in the visual representation
const POSITION_COORDS: [(u16, u16); 24] = [
    // Row 0 (y=0): outer ring top
    (0, 0),  // 0
    (12, 0), // 1
    (24, 0), // 2
    // Row 1 (y=2): middle ring top
    (4, 2),  // 3
    (12, 2), // 4
    (20, 2), // 5
    // Row 2 (y=4): inner ring top
    (8, 4),  // 6
    (12, 4), // 7
    (16, 4), // 8
    // Row 3 (y=6): middle horizontal - left side
    (0, 6), // 9
    (4, 6), // 10
    (8, 6), // 11
    // Row 3 (y=6): middle horizontal - right side
    (16, 6), // 12
    (20, 6), // 13
    (24, 6), // 14
    // Row 4 (y=8): inner ring bottom
    (8, 8),  // 15
    (12, 8), // 16
    (16, 8), // 17
    // Row 5 (y=10): middle ring bottom
    (4, 10),  // 18
    (12, 10), // 19
    (20, 10), // 20
    // Row 6 (y=12): outer ring bottom
    (0, 12),  // 21
    (12, 12), // 22
    (24, 12), // 23
];

fn render_board(frame: &mut Frame, area: Rect, game: &MorrisGame, show_last_move: bool) {
    // Board dimensions: 25 chars wide x 13 lines tall
    let board_width: u16 = 25;
    let board_height: u16 = 13;

    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;

    // Colors
    let line_color = Color::Rgb(80, 80, 80); // Dark gray for lines
    let player_color = Color::White;
    let ai_color = Color::LightRed;
    let cursor_color = Color::Yellow;
    let selected_color = Color::Rgb(100, 200, 100); // Green for selected
    let legal_dest_color = Color::Rgb(200, 100, 200); // Pink/magenta for legal destinations
    let capturable_color = Color::Red;
    let last_move_color = Color::Magenta;

    // Get last move position for highlighting
    let last_move_pos: Option<usize> = if show_last_move {
        game.last_move.map(|mv| match mv {
            MorrisMove::Place(pos) => pos,
            MorrisMove::Move { to, .. } => to,
            MorrisMove::Capture(pos) => pos,
        })
    } else {
        None
    };

    // Compute legal destinations for highlighting (only during active game)
    let legal_destinations = if show_last_move {
        Vec::new()
    } else {
        get_legal_destinations(game)
    };
    let capturable_positions = if show_last_move {
        Vec::new()
    } else {
        get_capturable_positions(game)
    };

    // Draw the board lines
    let board_lines = [
        // Outer square
        "\u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{252C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}", // 0
        "\u{2502}           \u{2502}           \u{2502}", // 1
        "\u{2502}   \u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}   \u{2502}", // 2 (middle ring top)
        "\u{2502}   \u{2502}       \u{2502}       \u{2502}   \u{2502}", // 3
        "\u{2502}   \u{2502}   \u{250C}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{2510}   \u{2502}   \u{2502}", // 4 (inner ring top)
        "\u{2502}   \u{2502}   \u{2502}       \u{2502}   \u{2502}   \u{2502}", // 5
        "\u{251C}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{253C}       \u{253C}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{2524}", // 6 (middle horizontal)
        "\u{2502}   \u{2502}   \u{2502}       \u{2502}   \u{2502}   \u{2502}", // 7
        "\u{2502}   \u{2502}   \u{2514}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{2518}   \u{2502}   \u{2502}", // 8 (inner ring bottom)
        "\u{2502}   \u{2502}       \u{2502}       \u{2502}   \u{2502}", // 9
        "\u{2502}   \u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}   \u{2502}", // 10 (middle ring bottom)
        "\u{2502}           \u{2502}           \u{2502}", // 11
        "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2534}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}", // 12 (outer square bottom)
    ];

    // Draw board lines
    for (i, line) in board_lines.iter().enumerate() {
        let text = Paragraph::new(*line).style(Style::default().fg(line_color));
        frame.render_widget(
            text,
            Rect::new(x_offset, y_offset + i as u16, board_width, 1),
        );
    }

    // Draw pieces and position markers
    for (pos, &(px, py)) in POSITION_COORDS.iter().enumerate() {
        let x = x_offset + px;
        let y = y_offset + py;

        let is_cursor = game.cursor == pos && !show_last_move;
        let is_selected = game.selected_position == Some(pos) && !show_last_move;
        let is_legal_dest = legal_destinations.contains(&pos);
        let is_capturable = capturable_positions.contains(&pos);
        let is_last_move = last_move_pos == Some(pos);

        let (symbol, style) = if is_last_move {
            // Highlight last move position
            match game.board[pos] {
                Some(Player::Human) => (
                    "[\u{25CF}]", // [●]
                    Style::default()
                        .fg(last_move_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(Player::Ai) => (
                    "[\u{25CF}]", // [●]
                    Style::default()
                        .fg(last_move_color)
                        .add_modifier(Modifier::BOLD),
                ),
                None => (
                    "[X]", // Captured position
                    Style::default()
                        .fg(last_move_color)
                        .add_modifier(Modifier::BOLD),
                ),
            }
        } else if is_cursor {
            // Cursor position
            match game.board[pos] {
                Some(Player::Human) => (
                    "[\u{25CF}]", // [●]
                    Style::default()
                        .fg(player_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(Player::Ai) => (
                    "[\u{25CF}]", // [●]
                    Style::default().fg(if is_capturable {
                        capturable_color
                    } else {
                        ai_color
                    }),
                ),
                None if is_legal_dest => (
                    "[\u{25C6}]", // [◆]
                    Style::default().fg(legal_dest_color),
                ),
                None => (
                    "[\u{00B7}]", // [·]
                    Style::default().fg(cursor_color),
                ),
            }
        } else if is_selected {
            // Selected piece
            match game.board[pos] {
                Some(Player::Human) => (
                    "<\u{25CF}>", // <●>
                    Style::default()
                        .fg(selected_color)
                        .add_modifier(Modifier::BOLD),
                ),
                _ => (
                    " \u{00B7} ", // Should not happen, but fallback
                    Style::default().fg(line_color),
                ),
            }
        } else if is_legal_dest {
            // Legal move destination (but not cursor)
            match game.board[pos] {
                None => (
                    " \u{25C6} ", // ◆
                    Style::default().fg(legal_dest_color),
                ),
                Some(Player::Human) => (
                    " \u{25CF} ", // ●
                    Style::default()
                        .fg(player_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(Player::Ai) => (
                    " \u{25CF} ", // ●
                    Style::default().fg(ai_color),
                ),
            }
        } else if is_capturable {
            // Capturable AI piece (highlighted in red)
            (
                " \u{25CF} ", // ●
                Style::default().fg(capturable_color),
            )
        } else {
            // Normal position
            match game.board[pos] {
                Some(Player::Human) => (
                    " \u{25CF} ", // ●
                    Style::default()
                        .fg(player_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(Player::Ai) => (
                    " \u{25CF} ", // ●
                    Style::default().fg(ai_color),
                ),
                None => (
                    " \u{00B7} ", // ·
                    Style::default().fg(line_color),
                ),
            }
        };

        let text = Paragraph::new(symbol).style(style);
        // Center the 3-char symbol on the position (position coord is center)
        frame.render_widget(text, Rect::new(x.saturating_sub(1), y, 3, 1));
    }
}

/// Get legal destination positions for the current player's selected piece or placing phase
fn get_legal_destinations(game: &MorrisGame) -> Vec<usize> {
    if game.ai_thinking || game.game_result.is_some() || game.forfeit_pending {
        return Vec::new();
    }

    // In must_capture mode, don't show movement destinations
    if game.must_capture {
        return Vec::new();
    }

    match game.phase {
        MorrisPhase::Placing => {
            // Show all empty positions as legal destinations
            game.board
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.is_none())
                .map(|(pos, _)| pos)
                .collect()
        }
        MorrisPhase::Moving | MorrisPhase::Flying => {
            // Only show destinations if a piece is selected
            if let Some(from) = game.selected_position {
                let can_fly = game.can_fly(Player::Human);
                if can_fly {
                    // Flying: any empty position
                    game.board
                        .iter()
                        .enumerate()
                        .filter(|(_, cell)| cell.is_none())
                        .map(|(pos, _)| pos)
                        .collect()
                } else {
                    // Normal movement: adjacent empty positions
                    ADJACENCIES[from]
                        .iter()
                        .filter(|&&pos| game.board[pos].is_none())
                        .copied()
                        .collect()
                }
            } else {
                Vec::new()
            }
        }
    }
}

/// Get capturable AI positions (when must_capture is true)
fn get_capturable_positions(game: &MorrisGame) -> Vec<usize> {
    if !game.must_capture || game.current_player != Player::Human {
        return Vec::new();
    }

    // Check if all AI pieces are in mills
    let all_in_mills = game
        .board
        .iter()
        .enumerate()
        .filter(|(_, cell)| **cell == Some(Player::Ai))
        .all(|(pos, _)| game.is_in_mill(pos, Player::Ai));

    game.board
        .iter()
        .enumerate()
        .filter(|(pos, cell)| {
            **cell == Some(Player::Ai) && (all_in_mills || !game.is_in_mill(*pos, Player::Ai))
        })
        .map(|(pos, _)| pos)
        .collect()
}

fn render_status(frame: &mut Frame, area: Rect, game: &MorrisGame) {
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent is thinking...");
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let (status_text, status_color) = if game.must_capture {
        ("MILL! Select a piece to capture", Color::Green)
    } else if game.selected_position.is_some() {
        ("Select destination", Color::Cyan)
    } else {
        match game.phase {
            MorrisPhase::Placing => ("Place a piece", Color::White),
            MorrisPhase::Moving => ("Select piece to move", Color::White),
            MorrisPhase::Flying => ("Select piece (flying!)", Color::Magenta),
        }
    };

    let controls: &[(&str, &str)] = if game.must_capture {
        &[("[Arrows]", "Move"), ("[Enter]", "Capture")]
    } else if game.selected_position.is_some() {
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Confirm"),
            ("[Esc]", "Cancel"),
        ]
    } else if game.phase == MorrisPhase::Placing {
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Place"),
            ("[Esc]", "Forfeit"),
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

fn render_info_panel(frame: &mut Frame, area: Rect, game: &MorrisGame) {
    let inner = render_info_panel_frame(frame, area);

    // Piece counts
    let human_on_board = game.pieces_on_board.0;
    let ai_on_board = game.pieces_on_board.1;

    // Rules summary
    let mut lines: Vec<Line> = vec![
        // Mills rule (always applies)
        Line::from(Span::styled(
            "MILLS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "3 in a row = capture 1",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "foe piece not in mill.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "Can break/remake mills.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        // Phase 1: Placing
        Line::from(Span::styled(
            "PHASE 1: PLACING",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Place on empty points.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        // Phase 2: Sliding
        Line::from(Span::styled(
            "PHASE 2: SLIDING",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Move to adjacent point.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        // Phase 3: Flying
        Line::from(Span::styled(
            "PHASE 3: FLYING",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "At 3 pieces, move to",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "ANY empty point.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        // Win condition
        Line::from(Span::styled(
            "WIN: \u{2264}2 or no moves",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        // Piece counts
        Line::from(vec![
            Span::styled("You: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("\u{25CF} x {}", human_on_board),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Foe: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("\u{25CF} x {}", ai_on_board),
                Style::default().fg(Color::LightRed),
            ),
        ]),
    ];

    // Pieces to place (if in placing phase)
    if game.phase == MorrisPhase::Placing {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "To place:",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(vec![
            Span::styled(" You: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", game.pieces_to_place.0),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" Foe: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", game.pieces_to_place.1),
                Style::default().fg(Color::LightRed),
            ),
        ]));
    }

    // Flying indicator
    if game.can_fly(Player::Human) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "You can fly!",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Difficulty
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
        Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
    ]));

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

fn render_morris_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &MorrisGame,
    xp_reward: u64,
    is_master: bool,
    ctx: &super::responsive::LayoutContext,
) {
    use ratatui::widgets::Clear;

    // First render the board with last move highlighted
    frame.render_widget(Clear, area);

    // Create layout matching normal game
    let layout = create_game_layout(frame, area, " Nine Men's Morris ", Color::Cyan, 13, 24, ctx);

    // Render board with last move highlighted
    render_board(frame, layout.content, game, true);
    render_info_panel(frame, layout.info_panel, game);

    let result = game.game_result.unwrap();
    let (result_type, title, message, reward) = match result {
        MorrisResult::Win => {
            let reward_text = if is_master {
                format!("+{} XP, +1 Fishing Rank", xp_reward)
            } else {
                format!("+{} XP", xp_reward)
            };
            (
                GameResultType::Win,
                "VICTORY!",
                "You outwitted the sage!",
                reward_text,
            )
        }
        MorrisResult::Loss => {
            if game.forfeit_pending {
                (
                    GameResultType::Forfeit,
                    "FORFEIT",
                    "You conceded the game",
                    String::new(),
                )
            } else {
                // Determine loss reason from game state
                let msg = if game.pieces_on_board.0 < 3 {
                    "Reduced to fewer than 3 pieces"
                } else {
                    "No legal moves remaining"
                };
                (GameResultType::Loss, "DEFEAT", msg, String::new())
            }
        }
    };

    // Render banner at bottom of content area
    render_game_over_banner(frame, layout.content, result_type, title, message, &reward);
}
