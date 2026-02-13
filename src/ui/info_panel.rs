use super::responsive::{LayoutContext, SizeTier};
use crate::core::game_state::GameState;
use crate::items::types::Rarity;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Safely truncate a string to fit within `max_width` bytes, respecting char boundaries.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        return s.to_string();
    }
    let limit = max_width.saturating_sub(3); // room for "…" (3 bytes UTF-8)
    let boundary = s.floor_char_boundary(limit);
    format!("{}…", &s[..boundary])
}

/// Draws the full-width bottom section: loot (left) and combat log (right) side by side
pub fn draw_info_panel(frame: &mut Frame, area: Rect, game_state: &GameState, ctx: &LayoutContext) {
    match ctx.tier {
        SizeTier::XL | SizeTier::L => {
            // Full side-by-side with borders
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            draw_recent_gains(frame, chunks[0], game_state);
            draw_combat_log(frame, chunks[1], game_state);
        }
        SizeTier::M => {
            // Compact: side-by-side, no borders, less padding
            draw_loot_combat_compact(frame, area, game_state);
        }
        SizeTier::S => {
            // Merged chronological feed
            draw_merged_feed(frame, area, game_state);
        }
        SizeTier::TooSmall => {}
    }
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
        let msg = truncate_to_width(&entry.message, max_width);

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
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
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

/// Compact side-by-side loot + combat log for M tier (no borders, minimal padding).
fn draw_loot_combat_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Loot (no border)
    {
        let loot_area = chunks[0];
        let mut lines: Vec<Line> = Vec::new();
        let max_lines = loot_area.height as usize;

        if game_state.recent_drops.is_empty() {
            lines.push(Line::from(Span::styled(
                "No loot yet",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for drop in game_state.recent_drops.iter() {
                if lines.len() >= max_lines {
                    break;
                }
                let color = rarity_color(drop.rarity);
                let equipped_tag = if drop.equipped { " ++" } else { "" };
                let max_name_len = (loot_area.width as usize).saturating_sub(12);
                let name = if drop.name.len() > max_name_len {
                    format!("{}...", &drop.name[..max_name_len.saturating_sub(3)])
                } else {
                    drop.name.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", drop.rarity.name().chars().next().unwrap_or('?')),
                        Style::default().fg(color),
                    ),
                    Span::styled(name, Style::default().fg(color)),
                    Span::styled(equipped_tag, Style::default().fg(Color::Green)),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, loot_area);
    }

    // Combat log (no border)
    {
        let combat_area = chunks[1];
        let mut lines: Vec<Line> = Vec::new();
        let max_entries = combat_area.height as usize;
        let max_width = combat_area.width as usize;

        for entry in game_state
            .combat_state
            .combat_log
            .iter()
            .rev()
            .take(max_entries)
        {
            let color = if entry.is_player_action {
                if entry.is_crit {
                    Color::Yellow
                } else {
                    Color::Green
                }
            } else {
                Color::Red
            };

            let msg = truncate_to_width(&entry.message, max_width);

            lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, combat_area);
    }
}

/// Merged feed for S tier: interleaved loot + combat entries in a single list.
pub(super) fn draw_merged_feed(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let max_lines = area.height as usize;
    let max_width = area.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    // Interleave: take from combat log (newest first) and loot drops alternately
    let mut combat_iter = game_state.combat_state.combat_log.iter().rev();
    let mut loot_iter = game_state.recent_drops.iter();

    // Alternate between combat and loot entries for a mixed feel
    let mut next_is_combat = true;
    while lines.len() < max_lines {
        if next_is_combat {
            if let Some(entry) = combat_iter.next() {
                let color = if entry.is_player_action {
                    if entry.is_crit {
                        Color::Yellow
                    } else {
                        Color::Green
                    }
                } else {
                    Color::Red
                };
                let msg = truncate_to_width(&entry.message, max_width);
                lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
                next_is_combat = false;
                continue;
            }
        }
        // Try loot
        if let Some(drop) = loot_iter.next() {
            let color = rarity_color(drop.rarity);
            let equipped_tag = if drop.equipped { " ++" } else { "" };
            let name_max = max_width.saturating_sub(8);
            let name = if drop.name.len() > name_max {
                format!("{}...", &drop.name[..name_max.saturating_sub(3)])
            } else {
                drop.name.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", drop.rarity.name().chars().next().unwrap_or('?')),
                    Style::default().fg(color),
                ),
                Span::styled(name, Style::default().fg(color)),
                Span::styled(equipped_tag, Style::default().fg(Color::Green)),
            ]));
            next_is_combat = true;
            continue;
        }
        // Try remaining combat entries
        if let Some(entry) = combat_iter.next() {
            let color = if entry.is_player_action {
                if entry.is_crit {
                    Color::Yellow
                } else {
                    Color::Green
                }
            } else {
                Color::Red
            };
            let msg = truncate_to_width(&entry.message, max_width);
            lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
            continue;
        }
        break;
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Awaiting adventure...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
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
