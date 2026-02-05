//! Fishing scene UI rendering.
//!
//! Displays the active fishing session with animated water, catch progress,
//! caught fish list, and fishing rank progression.

#![allow(dead_code)]

use crate::fishing::types::{FishRarity, FishingSession, FishingState};
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
    // Main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with spot name
            Constraint::Length(6), // Water animation area
            Constraint::Length(4), // Catch progress + phase status
            Constraint::Length(9), // Caught fish list (last 5 + borders)
            Constraint::Length(5), // Rank info and progress bar
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Draw header with spot name
    draw_header(frame, chunks[0], session);

    // Draw water animation with bobber
    draw_water_scene(frame, chunks[1], session);

    // Draw catch progress
    draw_catch_progress(frame, chunks[2], session);

    // Draw caught fish list
    draw_fish_list(frame, chunks[3], session);

    // Draw rank info and progress
    draw_rank_info(frame, chunks[4], fishing_state);
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

/// Draws the list of caught fish (last 5).
fn draw_fish_list(frame: &mut Frame, area: Rect, session: &FishingSession) {
    let fish_block = Block::default()
        .borders(Borders::ALL)
        .title(" Recent Catches ");

    let inner = fish_block.inner(area);
    frame.render_widget(fish_block, area);

    // Get the last 5 caught fish (most recent first)
    let recent_fish: Vec<_> = session.fish_caught.iter().rev().take(5).collect();

    if recent_fish.is_empty() {
        let empty_text = vec![Line::from(Span::styled(
            "No fish caught yet...",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))];

        let empty_paragraph = Paragraph::new(empty_text).alignment(Alignment::Center);
        frame.render_widget(empty_paragraph, inner);
        return;
    }

    // Track which fish indices dropped items
    let fish_with_items: std::collections::HashSet<usize> = session
        .items_found
        .iter()
        .enumerate()
        .map(|(i, _)| i)
        .collect();

    let mut lines = Vec::new();

    for (display_idx, fish) in recent_fish.iter().enumerate() {
        // Calculate the actual index in fish_caught (for item drop tracking)
        let actual_idx = session.fish_caught.len() - 1 - display_idx;

        // Get rarity color and name
        let (rarity_color, rarity_name) = get_rarity_style(fish.rarity);

        // Check if this fish dropped an item (simplified: check if we have items and index matches)
        let has_item = fish_with_items.len() > display_idx
            && actual_idx < session.fish_caught.len()
            && session.items_found.len() > display_idx;

        let mut spans = vec![
            Span::styled(
                format!("[{}]", rarity_name),
                Style::default().fg(rarity_color),
            ),
            Span::raw(" "),
            Span::styled(&fish.name, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled(
                format!("{} XP", fish.xp_reward),
                Style::default().fg(Color::Yellow),
            ),
        ];

        // Add item indicator if this fish dropped an item
        if has_item {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                "[Item]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        lines.push(Line::from(spans));
    }

    let fish_paragraph = Paragraph::new(lines);
    frame.render_widget(fish_paragraph, inner);
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

/// Returns the color and display name for a fish rarity.
fn get_rarity_style(rarity: FishRarity) -> (Color, &'static str) {
    match rarity {
        FishRarity::Common => (Color::Gray, "Common"),
        FishRarity::Uncommon => (Color::Green, "Uncommon"),
        FishRarity::Rare => (Color::Blue, "Rare"),
        FishRarity::Epic => (Color::Magenta, "Epic"),
        FishRarity::Legendary => (Color::Yellow, "Legendary"),
    }
}
