//! Challenge menu UI rendering.

use crate::challenge_menu::{ChallengeMenu, ChallengeType, DifficultyInfo};
use crate::chess::ChessDifficulty;
use crate::gomoku::GomokuDifficulty;
use crate::morris::MorrisDifficulty;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the challenge menu (list view or detail view)
pub fn render_challenge_menu(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    frame.render_widget(Clear, area);

    if menu.viewing_detail && !menu.challenges.is_empty() {
        render_detail_view(frame, area, menu);
    } else {
        render_list_view(frame, area, menu);
    }
}

fn render_list_view(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    let block = Block::default()
        .title(" Pending Challenges ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if menu.challenges.is_empty() {
        let text =
            Paragraph::new("No pending challenges.").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, inner);
        return;
    }

    let items: Vec<ListItem> = menu
        .challenges
        .iter()
        .enumerate()
        .map(|(i, challenge)| {
            let prefix = if i == menu.selected_index { "> " } else { "  " };
            let style = if i == menu.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{} {}", prefix, challenge.icon, challenge.title)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Help text at bottom
    if inner.height > 3 {
        let help_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        let help = Paragraph::new("[↑/↓] Navigate  [Enter] View  [Tab/Esc] Close")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }
}

fn render_detail_view(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    let challenge = &menu.challenges[menu.selected_index];

    let block = Block::default()
        .title(format!(" {} ", challenge.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Description (longer flavor text)
            Constraint::Length(1), // Spacer
            Constraint::Length(5), // Difficulty selector
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Outcomes (now single line)
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Description (with text wrapping)
    let desc = Paragraph::new(challenge.description.clone())
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(desc, chunks[0]);

    // Difficulty selector
    match challenge.challenge_type {
        ChallengeType::Chess => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &ChessDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Morris => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &MorrisDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Gomoku => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &GomokuDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
    }

    // Outcomes
    let outcomes = Paragraph::new(vec![Line::from(vec![
        Span::styled("✓ ", Style::default().fg(Color::Green)),
        Span::styled("No penalty for losing", Style::default().fg(Color::Gray)),
        Span::styled("    ✓ ", Style::default().fg(Color::Green)),
        Span::styled("Draw grants bonus XP", Style::default().fg(Color::Gray)),
    ])]);
    frame.render_widget(outcomes, chunks[4]);

    // Help text
    let help = Paragraph::new("[↑/↓] Difficulty  [Enter] Play  [D] Walk away  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[6]);
}

/// Generic difficulty selector that works with any type implementing DifficultyInfo
fn render_difficulty_selector<D: DifficultyInfo>(
    frame: &mut Frame,
    area: Rect,
    options: &[D],
    selected: usize,
) {
    let title = Paragraph::new("Select difficulty:").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, Rect { height: 1, ..area });

    let options_area = Rect {
        y: area.y + 1,
        height: area.height.saturating_sub(1),
        ..area
    };

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, diff)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "> " } else { "  " };

            let prefix_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let reward_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            let mut spans = vec![
                Span::styled(prefix, prefix_style),
                Span::styled(format!("{:<12}", diff.name()), name_style),
            ];

            // Add extra info if present (e.g., ELO for chess)
            if let Some(extra) = diff.extra_info() {
                spans.push(Span::styled(
                    format!("{:<14}", extra),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            spans.push(Span::styled(diff.reward().description(), reward_style));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, options_area);
}
