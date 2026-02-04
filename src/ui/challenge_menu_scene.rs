//! Challenge menu UI rendering.

use crate::challenge_menu::{ChallengeMenu, ChallengeType};
use crate::chess::ChessDifficulty;
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

    // Description
    let desc =
        Paragraph::new(challenge.description.clone()).style(Style::default().fg(Color::White));
    frame.render_widget(desc, chunks[0]);

    // Difficulty selector
    match challenge.challenge_type {
        ChallengeType::Chess => {
            render_chess_difficulty_selector(frame, chunks[2], menu.selected_difficulty);
        }
        ChallengeType::Morris => {
            render_morris_difficulty_selector(frame, chunks[2], menu.selected_difficulty);
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

fn render_chess_difficulty_selector(frame: &mut Frame, area: Rect, selected: usize) {
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

    let items: Vec<ListItem> = ChessDifficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, diff)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "> " } else { "  " };

            let reward = diff.reward_prestige();
            let reward_text = if reward == 1 {
                "Win: +1 Prestige Rank".to_string()
            } else {
                format!("Win: +{} Prestige Ranks", reward)
            };

            // Selected items get cyan/yellow highlighting
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

            let spans = vec![
                Span::styled(prefix, prefix_style),
                Span::styled(format!("{:<12}", diff.name()), name_style),
                Span::styled(
                    format!("~{:<5} ELO   ", diff.estimated_elo()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(reward_text, reward_style),
            ];

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, options_area);
}

fn render_morris_difficulty_selector(frame: &mut Frame, area: Rect, selected: usize) {
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

    let items: Vec<ListItem> = MorrisDifficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, diff)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "> " } else { "  " };

            let pct = diff.reward_xp_percent();
            let reward_text = if *diff == MorrisDifficulty::Master {
                format!("Win: +{}% level XP, +1 Fish Rank", pct)
            } else {
                format!("Win: +{}% level XP", pct)
            };

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

            let spans = vec![
                Span::styled(prefix, prefix_style),
                Span::styled(format!("{:<12}", diff.name()), name_style),
                Span::styled(reward_text, reward_style),
            ];

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, options_area);
}
