//! Haven skill tree panel rendering.
#![allow(dead_code)]

use crate::haven::{Haven, HavenRoomId};
use ratatui::style::Color;

use super::scene_fx::{put_cell, put_text, SceneCell};

/// Render the skill tree panel into a scene buffer.
pub(super) fn render_skill_tree(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    panel_width: usize,
    panel_height: usize,
    haven: &Haven,
    selected_room: usize,
) {
    let border_fg = Color::DarkGray;
    let right = left + panel_width as i32 - 1;
    let bottom = top + panel_height as i32 - 1;

    // Draw border
    put_cell(buffer, top, left, '\u{250c}', border_fg);
    put_cell(buffer, top, right, '\u{2510}', border_fg);
    put_cell(buffer, bottom, left, '\u{2514}', border_fg);
    put_cell(buffer, bottom, right, '\u{2518}', border_fg);
    for c in (left + 1)..right {
        put_cell(buffer, top, c, '\u{2500}', border_fg);
        put_cell(buffer, bottom, c, '\u{2500}', border_fg);
    }
    for r in (top + 1)..bottom {
        put_cell(buffer, r, left, '\u{2502}', border_fg);
        put_cell(buffer, r, right, '\u{2502}', border_fg);
    }
    put_text(buffer, top, left + 1, " Buildings ", border_fg);

    let inner_left = left + 1;
    let content_top = top + 1;

    for (i, room) in HavenRoomId::ALL.iter().enumerate() {
        let row = content_top + i as i32;
        if row >= bottom {
            break;
        }

        let tier = haven.room_tier(*room);
        let unlocked = haven.is_room_unlocked(*room);
        let is_selected = i == selected_room;

        let max_t = room.max_tier();
        let tier_str: String = (1..=max_t)
            .map(|t| if tier >= t { "\u{2605}" } else { "\u{00b7}" })
            .collect::<Vec<_>>()
            .join("");

        let prefix = if is_selected { "\u{25b6} " } else { "  " };

        let indent = match room {
            HavenRoomId::Hearthstone => "",
            HavenRoomId::Armory | HavenRoomId::Bedroom => "  ",
            HavenRoomId::TrainingYard
            | HavenRoomId::TrophyHall
            | HavenRoomId::Garden
            | HavenRoomId::Library => "    ",
            HavenRoomId::Watchtower
            | HavenRoomId::AlchemyLab
            | HavenRoomId::FishingDock
            | HavenRoomId::Workshop => "      ",
            HavenRoomId::WarRoom | HavenRoomId::Vault => "        ",
            HavenRoomId::StormForge => "          ",
        };

        let style_fg = if !unlocked {
            Color::DarkGray
        } else if is_selected {
            Color::Cyan
        } else if tier > 0 {
            Color::Green
        } else {
            Color::White
        };

        let tier_fg = if tier > 0 {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let lock_indicator = if !unlocked { "\u{2716} " } else { "" };

        let mut col = inner_left;
        put_text(buffer, row, col, prefix, style_fg);
        col += prefix.chars().count() as i32;
        put_text(buffer, row, col, &tier_str, tier_fg);
        col += tier_str.chars().count() as i32;
        put_text(buffer, row, col, " ", style_fg);
        col += 1;
        put_text(buffer, row, col, indent, style_fg);
        col += indent.chars().count() as i32;
        put_text(buffer, row, col, lock_indicator, Color::DarkGray);
        col += lock_indicator.chars().count() as i32;
        put_text(buffer, row, col, room.name(), style_fg);

        // Highlight selected row background
        if is_selected {
            let highlight_bg = Color::Rgb(30, 22, 12);
            let row_usize = row as usize;
            if row_usize < buffer.len() {
                for c in (inner_left as usize)..((right) as usize) {
                    if c < buffer[row_usize].len() {
                        buffer[row_usize][c].bg = highlight_bg;
                    }
                }
            }
        }
    }
}

/// Simple word-wrap: break text into lines that fit within `max_width` characters.
pub(super) fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
