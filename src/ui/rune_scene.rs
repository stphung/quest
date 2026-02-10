//! Rune Deciphering game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, GameResultType,
};
use crate::challenges::rune::{FeedbackMark, RuneGame, RUNE_SYMBOLS};
use crate::challenges::ChallengeResult;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the rune deciphering game scene.
pub fn render_rune(frame: &mut Frame, area: Rect, game: &RuneGame) {
    // Game over overlay
    if game.game_result.is_some() {
        render_rune_game_over(frame, area, game);
        return;
    }

    // Use shared layout
    let layout = create_game_layout(frame, area, " Rune Deciphering ", Color::Magenta, 6, 22);

    render_grid(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render guess history and current input.
fn render_grid(frame: &mut Frame, area: Rect, game: &RuneGame) {
    // No border - outer block provides it
    let mut y = area.y;

    // Render submitted guesses
    for (i, guess) in game.guesses.iter().enumerate() {
        if y >= area.y + area.height {
            break;
        }
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{:>2}: ", i + 1),
            Style::default().fg(Color::DarkGray),
        ));

        for &rune_idx in &guess.runes {
            let ch = RUNE_SYMBOLS[rune_idx];
            spans.push(Span::styled(
                format!("{} ", ch),
                Style::default().fg(Color::White),
            ));
        }

        spans.push(Span::raw("  "));

        for mark in &guess.feedback {
            let (sym, color) = match mark {
                FeedbackMark::Exact => ("\u{25CF}", Color::Green),
                FeedbackMark::Misplaced => ("\u{25CB}", Color::Yellow),
                FeedbackMark::Wrong => ("\u{00B7}", Color::DarkGray),
            };
            spans.push(Span::styled(
                format!("{} ", sym),
                Style::default().fg(color),
            ));
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(area.x + 1, y, area.width.saturating_sub(2), 1),
        );
        y += 1;
    }

    // Blank line separator
    if !game.guesses.is_empty() && game.game_result.is_none() {
        y += 1;
    }

    // Render current guess input (only if game not over)
    if game.game_result.is_none() && y < area.y + area.height {
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{:>2}: ", game.guesses.len() + 1),
            Style::default().fg(Color::DarkGray),
        ));

        for (i, slot) in game.current_guess.iter().enumerate() {
            let is_cursor = i == game.cursor_slot;
            let text = match slot {
                Some(idx) => format!("{} ", RUNE_SYMBOLS[*idx]),
                None => "_ ".to_string(),
            };
            let mut style = Style::default().fg(Color::Cyan);
            if is_cursor {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }
            spans.push(Span::styled(text, style));
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(area.x + 1, y, area.width.saturating_sub(2), 1),
        );
        y += 2;
    }

    // Available runes
    if game.game_result.is_none() && y < area.y + area.height {
        let mut spans = vec![Span::styled(
            "Runes: ",
            Style::default().fg(Color::DarkGray),
        )];
        for symbol in RUNE_SYMBOLS.iter().take(game.num_runes) {
            spans.push(Span::styled(
                format!("{} ", symbol),
                Style::default().fg(Color::White),
            ));
        }
        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(area.x + 1, y, area.width.saturating_sub(2), 1),
        );
    }
}

/// Render the status bar below the grid (status + controls).
/// Standard controls for Rune game.
const RUNE_CONTROLS: &[(&str, &str)] = &[
    ("[←→]", "Move"),
    ("[↑↓]", "Cycle"),
    ("[Enter]", "Go"),
    ("[F]", "Clear"),
    ("[Esc]", "Quit"),
];

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &RuneGame) {
    // Handle rejection message specially (shows error inline)
    if let Some(ref msg) = game.reject_message {
        render_status_bar(frame, area, msg, Color::LightRed, RUNE_CONTROLS);
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let (status_text, status_color) = if game.guesses.is_empty() {
        ("Begin deciphering", Color::Yellow)
    } else {
        ("Deciphering...", Color::Green)
    };

    render_status_bar(frame, area, status_text, status_color, RUNE_CONTROLS);
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let inner = render_info_panel_frame(frame, area);

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Runes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.num_runes),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Slots: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.num_slots),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Guesses: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} left", game.guesses_remaining()),
                Style::default().fg(if game.guesses_remaining() <= 2 {
                    Color::Red
                } else {
                    Color::White
                }),
            ),
        ]),
    ];

    if game.allow_duplicates {
        lines.push(Line::from(Span::styled(
            "Duplicates: Yes",
            Style::default().fg(Color::Yellow),
        )));
    }

    lines.push(Line::from(""));

    // Feedback legend
    lines.push(Line::from(Span::styled(
        "Feedback:",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled(" \u{25CF} ", Style::default().fg(Color::Green)),
        Span::styled("Correct pos", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" \u{25CB} ", Style::default().fg(Color::Yellow)),
        Span::styled("Wrong pos", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" \u{00B7} ", Style::default().fg(Color::DarkGray)),
        Span::styled("Not in code", Style::default().fg(Color::DarkGray)),
    ]));

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

fn render_rune_game_over(frame: &mut Frame, area: Rect, game: &RuneGame) {
    use crate::challenges::menu::ChallengeType;

    let result = game.game_result.as_ref().unwrap();

    let (result_type, title, message, reward) = match result {
        ChallengeResult::Win => (
            GameResultType::Win,
            ":: RUNES DECIPHERED! ::",
            "You cracked the ancient code!".to_string(),
            ChallengeType::Rune.reward(game.difficulty).description(),
        ),
        ChallengeResult::Loss | ChallengeResult::Draw | ChallengeResult::Forfeit => {
            // Build the code string to show in message
            let code: String = game
                .secret_code
                .iter()
                .map(|&idx| RUNE_SYMBOLS[idx].to_string())
                .collect::<Vec<_>>()
                .join(" ");
            (
                GameResultType::Loss,
                "RUNES REMAIN HIDDEN",
                format!("The code was: {}", code),
                "No penalty incurred.".to_string(),
            )
        }
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward);
}
