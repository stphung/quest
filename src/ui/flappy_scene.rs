//! UI rendering for the Flappy Bird challenge minigame.

use crate::challenges::flappy::types::{FlappyGame, FlappyResult};
use crate::challenges::menu::DifficultyInfo;
use crate::ui::game_common::{
    render_forfeit_status_bar, render_game_over_overlay, render_info_panel_frame,
    render_status_bar, GameResultType,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the Flappy Bird game scene.
pub fn render_flappy(frame: &mut Frame, area: Rect, game: &FlappyGame) {
    // Game over overlay takes priority
    if game.game_result.is_some() {
        render_flappy_game_over(frame, area, game);
        return;
    }

    frame.render_widget(Clear, area);

    // Outer border
    let block = Block::default()
        .title(" Flappy Bird ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Horizontal split: game area (left) | info panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(22)])
        .split(inner);

    // Left side: play area (top) + status bar (bottom 2 lines)
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(2)])
        .split(h_chunks[0]);

    render_play_area(frame, v_chunks[0], game);
    render_status_bar_content(frame, v_chunks[1], game);
    render_info_panel(frame, h_chunks[1], game);
}

/// Render the main play area with bird and pipes.
fn render_play_area(frame: &mut Frame, area: Rect, game: &FlappyGame) {
    let width = area.width as usize;
    let height = area.height as usize;

    if width == 0 || height == 0 {
        return;
    }

    // Build the play area line by line
    let mut lines = Vec::with_capacity(height);

    let bird_row = game.bird_y.round() as usize;

    // Scale game coordinates to display area
    let x_scale = if game.area_width > 0 {
        width as f64 / game.area_width as f64
    } else {
        1.0
    };
    let y_scale = if game.area_height > 0 {
        height as f64 / game.area_height as f64
    } else {
        1.0
    };

    for display_row in 0..height {
        let mut spans = Vec::new();
        let game_row = (display_row as f64 / y_scale).round() as usize;

        for display_col in 0..width {
            let game_col = (display_col as f64 / x_scale).round() as i32;

            // Check if this is the bird position
            let bird_display_row = (bird_row as f64 * y_scale).round() as usize;
            let bird_display_col = (game.bird_x as f64 * x_scale).round() as usize;

            if display_row == bird_display_row && display_col == bird_display_col {
                // Bird character
                let bird_char = if game.bird_vel < -0.5 {
                    "▲" // Flapping up
                } else if game.bird_vel > 1.0 {
                    "▼" // Falling fast
                } else {
                    "►" // Neutral
                };
                spans.push(Span::styled(
                    bird_char,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                continue;
            }

            // Check if this is a pipe
            let mut is_pipe = false;
            let mut is_gap_edge = false;
            for pipe in &game.pipes {
                let pipe_display_col = (pipe.x as f64 * x_scale).round() as i32;
                // Pipe is 2 display columns wide
                if game_col >= pipe.x && game_col <= pipe.x + 1 {
                    let gap_top = pipe.gap_top as usize;
                    let gap_bottom = gap_top + game.difficulty.gap_size() as usize;

                    if game_row < gap_top || game_row >= gap_bottom {
                        is_pipe = true;
                    } else if game_row == gap_top || game_row == gap_bottom.saturating_sub(1) {
                        is_gap_edge = true;
                    }
                    let _ = pipe_display_col; // Used for coordinate mapping
                    break;
                }
            }

            if is_pipe {
                spans.push(Span::styled("█", Style::default().fg(Color::Green)));
            } else if is_gap_edge {
                spans.push(Span::styled("░", Style::default().fg(Color::DarkGray)));
            } else {
                spans.push(Span::styled(" ", Style::default()));
            }
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Render the status bar at the bottom.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &FlappyGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    if !game.started {
        render_status_bar(
            frame,
            area,
            "Press Space to start!",
            Color::Yellow,
            &[("[Space/Up/Enter]", "Flap"), ("[Esc]", "Forfeit")],
        );
    } else {
        render_status_bar(
            frame,
            area,
            &format!("Score: {} / {}", game.score, game.difficulty.target_score()),
            Color::Green,
            &[("[Space/Up/Enter]", "Flap"), ("[Esc]", "Forfeit")],
        );
    }
}

/// Render the info panel on the right.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &FlappyGame) {
    let inner = render_info_panel_frame(frame, area);

    if inner.height < 2 || inner.width < 4 {
        return;
    }

    let mut lines = vec![
        Line::from(Span::styled(
            format!(" {} ", game.difficulty.name()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.score),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Target: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.difficulty.target_score()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Gap: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.difficulty.gap_size()),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
    ];

    // Progress bar
    let progress = if game.difficulty.target_score() > 0 {
        (game.score as f64 / game.difficulty.target_score() as f64).min(1.0)
    } else {
        0.0
    };
    let bar_width = (inner.width as usize).saturating_sub(4);
    let filled = (progress * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    lines.push(Line::from(Span::styled(
        " Progress:",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("█".repeat(filled), Style::default().fg(Color::Green)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
    ]));

    // Reward info
    lines.push(Line::from(""));
    let reward = game.difficulty.reward();
    lines.push(Line::from(Span::styled(
        format!(" {}", reward.description()),
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Render the game-over overlay.
fn render_flappy_game_over(frame: &mut Frame, area: Rect, game: &FlappyGame) {
    let (result_type, title, message) = match game.game_result {
        Some(FlappyResult::Win) => (
            GameResultType::Win,
            "VICTORY!",
            format!(
                "You passed {} pipes! The gauntlet is conquered!",
                game.score
            ),
        ),
        Some(FlappyResult::Loss) => (
            GameResultType::Loss,
            "CRASH!",
            format!("You passed {} pipes before crashing.", game.score),
        ),
        Some(FlappyResult::Forfeit) => (
            GameResultType::Forfeit,
            "FORFEIT",
            format!("You gave up after {} pipes.", game.score),
        ),
        None => return,
    };

    let reward = if game.game_result == Some(FlappyResult::Win) {
        game.difficulty.reward().description()
    } else {
        String::new()
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward);
}
