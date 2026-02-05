//! Rune Deciphering game UI rendering.

use crate::rune::{FeedbackMark, RuneGame, RuneResult, RUNE_SYMBOLS};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the rune deciphering game scene.
pub fn render_rune(frame: &mut Frame, area: Rect, game: &RuneGame) {
    frame.render_widget(Clear, area);

    // Horizontal: game area (left) + info panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(22)])
        .split(area);

    // Left side: grid (top) + status bar (bottom 2 lines)
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(2)])
        .split(h_chunks[0]);

    render_grid(frame, v_chunks[0], game);
    render_status_bar(frame, v_chunks[1], game);
    render_info_panel(frame, h_chunks[1], game);

    if game.game_result.is_some() {
        render_game_over_overlay(frame, h_chunks[0], game);
    }
}

/// Render guess history and current input.
fn render_grid(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let block = Block::default()
        .title(" Rune Deciphering ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut y = inner.y;

    // Render submitted guesses
    for (i, guess) in game.guesses.iter().enumerate() {
        if y >= inner.y + inner.height {
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
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
        );
        y += 1;
    }

    // Blank line separator
    if !game.guesses.is_empty() && game.game_result.is_none() {
        y += 1;
    }

    // Render current guess input (only if game not over)
    if game.game_result.is_none() && y < inner.y + inner.height {
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
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
        );
        y += 2;
    }

    // Available runes
    if game.game_result.is_none() && y < inner.y + inner.height {
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
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
        );
    }
}

/// Render the status bar below the grid (status + controls).
fn render_status_bar(frame: &mut Frame, area: Rect, game: &RuneGame) {
    if area.height < 2 {
        return;
    }

    // Line 1: Status message
    let status = if game.game_result.is_some() {
        Span::styled("", Style::default())
    } else if let Some(ref msg) = game.reject_message {
        Span::styled(msg.clone(), Style::default().fg(Color::LightRed))
    } else if game.forfeit_pending {
        Span::styled(
            "Forfeit game? Press Esc again to confirm",
            Style::default().fg(Color::LightRed),
        )
    } else if game.guesses.is_empty() {
        Span::styled("Begin deciphering", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("Deciphering...", Style::default().fg(Color::Green))
    };
    let status_line = Paragraph::new(Line::from(vec![Span::raw(" "), status]))
        .alignment(Alignment::Left);
    frame.render_widget(
        status_line,
        Rect::new(area.x, area.y, area.width, 1),
    );

    // Line 2: Controls
    if game.game_result.is_none() {
        let controls = if game.forfeit_pending {
            vec![
                Span::styled(" [Esc]", Style::default().fg(Color::White)),
                Span::styled(" Confirm  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Any]", Style::default().fg(Color::White)),
                Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
            ]
        } else {
            vec![
                Span::styled(" [\u{2190}\u{2192}]", Style::default().fg(Color::White)),
                Span::styled(" Move  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[\u{2191}\u{2193}]", Style::default().fg(Color::White)),
                Span::styled(" Cycle  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Enter]", Style::default().fg(Color::White)),
                Span::styled(" Go  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[F]", Style::default().fg(Color::White)),
                Span::styled(" Clear  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Esc]", Style::default().fg(Color::White)),
                Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
            ]
        };
        let controls_line = Paragraph::new(Line::from(controls));
        frame.render_widget(
            controls_line,
            Rect::new(area.x, area.y + 1, area.width, 1),
        );
    }
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

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

/// Render the game over overlay.
fn render_game_over_overlay(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let result = game.game_result.as_ref().unwrap();

    let (title, color) = match result {
        RuneResult::Win => ("Runes Deciphered!", Color::Green),
        RuneResult::Loss => ("Runes Remain Hidden", Color::Red),
    };

    let mut overlay_lines = vec![
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if *result == RuneResult::Loss {
        let mut code_spans = vec![Span::styled("Code: ", Style::default().fg(Color::DarkGray))];
        for &idx in &game.secret_code {
            code_spans.push(Span::styled(
                format!("{} ", RUNE_SYMBOLS[idx]),
                Style::default().fg(Color::White),
            ));
        }
        overlay_lines.push(Line::from(code_spans));
    }

    use crate::challenge_menu::DifficultyInfo;
    let reward_text = if *result == RuneResult::Win {
        game.difficulty.reward().description()
    } else {
        "No reward".to_string()
    };
    overlay_lines.push(Line::from(Span::styled(
        reward_text,
        Style::default().fg(Color::White),
    )));

    overlay_lines.push(Line::from(Span::styled(
        "[Any key to continue]",
        Style::default().fg(Color::DarkGray),
    )));

    let height = overlay_lines.len() as u16 + 2;
    let width = 30;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let text = Paragraph::new(overlay_lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
