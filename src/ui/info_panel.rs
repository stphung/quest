use super::responsive::{LayoutContext, SizeTier};
use crate::core::game_state::GameState;
use crate::items::types::Rarity;
use ratatui::{
    layout::Rect,
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
            // Full-width combat log (ticker handles loot display)
            draw_combat_log(frame, area, game_state);
        }
        SizeTier::M => {
            // Compact combat log (ticker handles loot display)
            draw_combat_log_compact(frame, area, game_state);
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

/// Compact combat log for M tier (no borders, no loot side).
fn draw_combat_log_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut lines: Vec<Line> = Vec::new();
    let max_entries = area.height as usize;
    let max_width = area.width as usize;

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
    frame.render_widget(paragraph, area);
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
