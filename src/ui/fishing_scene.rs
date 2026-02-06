//! Fishing scene UI rendering.
//!
//! Displays the active fishing session with animated water, catch progress,
//! caught fish list, and fishing rank progression.

#![allow(dead_code)]

use crate::fishing::types::{FishingSession, FishingState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Renders the fishing scene UI.
///
/// # Layout
/// ```text
/// +---------------------------------------+
/// |  FISHING - [Spot Name]                |
/// +---------------------------------------+
/// |     ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~         |
/// |       ~~~~~~ O ~~~~~~                 |
/// |     ~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~           |
/// |              |                        |
/// +---------------------------------------+
/// |  Caught: X/Y fish                     |
/// +---------------------------------------+
/// |  [Uncommon] Trout - 180 XP            |
/// |  [Common] Carp - 65 XP                |
/// |  [Rare] Salmon - 520 XP  [Item]       |
/// +---------------------------------------+
/// |  Rank: [Rank Name] (N)                |
/// |  Progress: ████████░░ X/Y             |
/// +---------------------------------------+
/// ```
pub fn render_fishing_scene(
    frame: &mut Frame,
    area: Rect,
    session: &FishingSession,
    fishing_state: &FishingState,
) {
    // Main vertical layout (recent catches now shown in the Loot panel)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with spot name
            Constraint::Min(6),    // Water animation area
            Constraint::Length(4), // Catch progress + phase status
            Constraint::Length(5), // Rank info and progress bar
        ])
        .split(area);

    // Draw header with spot name
    draw_header(frame, chunks[0], session);

    // Draw water animation with bobber
    draw_water_scene(frame, chunks[1], session);

    // Draw catch progress
    draw_catch_progress(frame, chunks[2], session);

    // Draw rank info and progress
    draw_rank_info(frame, chunks[3], fishing_state);
}

/// Draws the header with fishing spot name.
fn draw_header(frame: &mut Frame, area: Rect, session: &FishingSession) {
    let title = format!(" FISHING - {} ", session.spot_name);

    let header_text = vec![Line::from(vec![Span::styled(
        format!("Fishing at {}", session.spot_name),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .alignment(Alignment::Center);

    frame.render_widget(header, area);
}

/// Draws the ASCII water scene with bobber.
fn draw_water_scene(frame: &mut Frame, area: Rect, session: &FishingSession) {
    use crate::fishing::types::FishingPhase;

    // Calculate bobber animation based on phase
    let bobber_depth = if session.phase == FishingPhase::Reeling {
        // Fish is biting - bobber dips
        2
    } else {
        // Normal floating (Casting or Waiting)
        1
    };

    let water_lines = if bobber_depth == 2 {
        // Fish biting - more disturbance
        vec![
            Line::from(Span::styled(
                "    ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~",
                Style::default().fg(Color::Blue),
            )),
            Line::from(vec![
                Span::styled("      ~~~", Style::default().fg(Color::Blue)),
                Span::styled("~", Style::default().fg(Color::LightBlue)),
                Span::styled("~", Style::default().fg(Color::Blue)),
                Span::styled(
                    " O ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("~", Style::default().fg(Color::Blue)),
                Span::styled("~", Style::default().fg(Color::LightBlue)),
                Span::styled("~~~", Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("    ~ ~ ~ ~", Style::default().fg(Color::Blue)),
                Span::styled(" |", Style::default().fg(Color::DarkGray)),
                Span::styled(" ~ ~ ~ ~ ~ ~", Style::default().fg(Color::Blue)),
            ]),
            Line::from(Span::styled(
                "             |",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        // Normal floating
        vec![
            Line::from(Span::styled(
                "    ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~",
                Style::default().fg(Color::Blue),
            )),
            Line::from(vec![
                Span::styled("      ~~~~~~", Style::default().fg(Color::Blue)),
                Span::styled(
                    " O ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("~~~~~~", Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("    ~ ~ ~ ~ ~", Style::default().fg(Color::Blue)),
                Span::styled("|", Style::default().fg(Color::DarkGray)),
                Span::styled("~ ~ ~ ~ ~ ~ ~", Style::default().fg(Color::Blue)),
            ]),
            Line::from(Span::styled(
                "             |",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let water_block = Block::default().borders(Borders::LEFT | Borders::RIGHT);

    let water_paragraph = Paragraph::new(water_lines)
        .block(water_block)
        .alignment(Alignment::Center);

    frame.render_widget(water_paragraph, area);
}

/// Draws the catch progress indicator with current phase.
fn draw_catch_progress(frame: &mut Frame, area: Rect, session: &FishingSession) {
    use crate::fishing::types::FishingPhase;

    use super::throbber::spinner_char;

    let spinner = spinner_char();

    let caught = session.fish_caught.len() as u32;
    let total = session.total_fish;

    // Get phase text and color
    let (phase_text, phase_color) = match session.phase {
        FishingPhase::Casting => (format!("{} Casting line...", spinner), Color::White),
        FishingPhase::Waiting => (format!("{} Waiting for bite...", spinner), Color::Cyan),
        FishingPhase::Reeling => ("🐟 FISH ON! Reeling in!".to_string(), Color::Yellow),
    };

    let progress_text = vec![
        Line::from(vec![
            Span::styled("Caught: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}/{}", caught, total),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" fish"),
        ]),
        Line::from(vec![Span::styled(
            phase_text,
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let progress_block = Block::default().borders(Borders::ALL).title(" Status ");

    let progress_paragraph = Paragraph::new(progress_text)
        .block(progress_block)
        .alignment(Alignment::Center);

    frame.render_widget(progress_paragraph, area);
}

/// Draws the fishing rank info and progress bar.
fn draw_rank_info(frame: &mut Frame, area: Rect, fishing_state: &FishingState) {
    let rank_block = Block::default()
        .borders(Borders::ALL)
        .title(" Fishing Rank ");

    let inner = rank_block.inner(area);
    frame.render_widget(rank_block, area);

    // Split inner area for rank name and progress bar
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Rank name
            Constraint::Length(1), // Progress bar
        ])
        .split(inner);

    // Draw rank name
    let rank_name = fishing_state.rank_name();
    let rank_text = vec![Line::from(vec![
        Span::styled("Rank: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            rank_name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("({})", fishing_state.rank),
            Style::default().fg(Color::Yellow),
        ),
    ])];

    let rank_paragraph = Paragraph::new(rank_text);
    frame.render_widget(rank_paragraph, inner_chunks[0]);

    // Draw progress bar
    let required = FishingState::fish_required_for_rank(fishing_state.rank);
    let progress = fishing_state.fish_toward_next_rank;
    let ratio = if required > 0 {
        (progress as f64 / required as f64).min(1.0)
    } else {
        0.0
    };

    let progress_label = format!("{}/{}", progress, required);

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .label(progress_label)
        .ratio(ratio);

    frame.render_widget(gauge, inner_chunks[1]);
}

