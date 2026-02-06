use crate::core::game_state::GameState;
use crate::items::types::Rarity;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draws the full-width bottom section: loot (left) and combat log (right) side by side
pub fn draw_info_panel(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Loot
            Constraint::Percentage(50), // Combat log
        ])
        .split(area);

    draw_recent_gains(frame, chunks[0], game_state);
    draw_combat_log(frame, chunks[1], game_state);
}

/// Draws the combat log panel
fn draw_combat_log(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Combat ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Show combat log entries (newest first)
    let max_entries = inner.height as usize;
    let entries = game_state
        .combat_state
        .combat_log
        .iter()
        .rev()
        .take(max_entries);

    for entry in entries {
        let color = if entry.is_player_action {
            if entry.is_crit {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::Red
        };

        let modifier = if entry.is_crit {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        // Truncate long messages to fit panel width
        let max_width = inner.width as usize;
        let msg = if entry.message.len() > max_width {
            format!("{}…", &entry.message[..max_width.saturating_sub(1)])
        } else {
            entry.message.clone()
        };

        lines.push(Line::from(vec![Span::styled(
            msg,
            Style::default().fg(color).add_modifier(modifier),
        )]));
    }

    // Pad remaining space
    while lines.len() < inner.height as usize {
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Draws the loot panel (items, fish, etc.) with two-line format for equipment.
fn draw_recent_gains(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Loot ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    let max_lines = inner.height as usize;

    if game_state.recent_drops.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No gains yet",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for drop in game_state.recent_drops.iter() {
            if lines.len() >= max_lines {
                break;
            }

            let color = rarity_color(drop.rarity);
            let rarity_tag = format!("[{}]", drop.rarity.name());
            let equipped_tag = if drop.equipped { " 🔨" } else { "" };

            // Line 1: icon [Rarity] Name 🔨  Slot
            let mut spans = vec![
                Span::styled(format!("{} ", drop.icon), Style::default().fg(color)),
                Span::styled(
                    format!("{} ", rarity_tag),
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&drop.name, Style::default().fg(color)),
                Span::styled(equipped_tag, Style::default().fg(Color::Green)),
            ];

            if !drop.slot.is_empty() {
                spans.push(Span::styled(
                    format!("  {}", drop.slot),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            lines.push(Line::from(spans));

            // Line 2: stat summary (only for equipment with stats)
            if !drop.stats.is_empty() && lines.len() < max_lines {
                lines.push(Line::from(Span::styled(
                    format!("  {}", drop.stats),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    // Pad remaining space
    while lines.len() < max_lines {
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn rarity_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::Gray,
        Rarity::Magic => Color::Blue,
        Rarity::Rare => Color::Yellow,
        Rarity::Epic => Color::Magenta,
        Rarity::Legendary => Color::Rgb(255, 165, 0),
    }
}

