//! Haven room detail panel rendering.
#![allow(dead_code)]

use crate::haven::{can_afford, tier_cost, Haven, HavenRoomId};
use ratatui::style::Color;

use super::haven_tree::word_wrap;
use super::scene_fx::{put_cell, put_text, SceneCell};

/// Render the room detail panel into a scene buffer.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_room_detail(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    panel_width: usize,
    panel_height: usize,
    haven: &Haven,
    selected_room: usize,
    prestige_rank: u32,
    achievements: &crate::achievements::Achievements,
) {
    let room = HavenRoomId::ALL[selected_room];
    let tier = haven.room_tier(room);
    let unlocked = haven.is_room_unlocked(room);

    let border_fg = if unlocked {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let right = left + panel_width as i32 - 1;
    let bottom = top + panel_height as i32 - 1;
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
    let title = format!(" {} ", room.name());
    put_text(buffer, top, left + 1, &title, border_fg);

    let inner_left = left + 1;
    let inner_width = (panel_width - 2).max(1);
    let mut row = top + 1;

    // Description (word-wrapped)
    let desc = room.description();
    let wrapped = word_wrap(desc, inner_width);
    for line in &wrapped {
        if row >= bottom {
            break;
        }
        put_text(buffer, row, inner_left, line, Color::White);
        row += 1;
    }
    row += 1;

    // Bonuses
    if row < bottom {
        put_text(buffer, row, inner_left, "Bonuses:", Color::White);
        row += 1;
    }
    let max_tier = room.max_tier();
    for t in 1..=max_tier {
        if row >= bottom {
            break;
        }
        let is_built = t <= tier;
        let is_next = t == tier + 1 && tier < max_tier;
        let style_fg = if is_built {
            Color::Green
        } else if is_next {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let marker = if is_next { "\u{25b6} " } else { "  " };
        let tier_label = format!("T{}: ", t);
        let bonus_text = room.format_bonus(t);
        let mut col = inner_left;
        put_text(buffer, row, col, marker, style_fg);
        col += marker.chars().count() as i32;
        put_text(buffer, row, col, &tier_label, Color::DarkGray);
        col += tier_label.chars().count() as i32;
        put_text(buffer, row, col, &bonus_text, style_fg);
        row += 1;
    }
    row += 1;

    // Requirements
    let parents = room.parents();
    if !parents.is_empty() && row < bottom {
        put_text(buffer, row, inner_left, "Requires:", Color::White);
        row += 1;
        for parent in parents {
            if row >= bottom {
                break;
            }
            let parent_tier = haven.room_tier(*parent);
            let is_built = parent_tier > 0;
            let (marker, style_fg) = if is_built {
                ("\u{2713}", Color::Green)
            } else {
                ("\u{2717}", Color::Red)
            };
            let tier_info = if parent_tier > 0 {
                format!(" (T{})", parent_tier)
            } else {
                String::new()
            };
            let mut col = inner_left;
            put_text(buffer, row, col, &format!("  {} ", marker), style_fg);
            col += 4;
            put_text(buffer, row, col, parent.name(), style_fg);
            col += parent.name().chars().count() as i32;
            put_text(buffer, row, col, &tier_info, Color::DarkGray);
            row += 1;
        }
        row += 1;
    }

    // Cost info
    if row >= bottom {
        return;
    }
    if !unlocked {
        put_text(buffer, row, inner_left, "\u{2716} Locked", Color::Red);
        row += 1;
        if row < bottom {
            put_text(
                buffer,
                row,
                inner_left,
                "Build all required rooms first",
                Color::DarkGray,
            );
        }
    } else if tier < room.max_tier() {
        let next_tier = tier + 1;
        let cost = tier_cost(room, next_tier);
        let can_afford_it = can_afford(room, haven, prestige_rank);
        let cost_fg = if can_afford_it {
            Color::Green
        } else {
            Color::Red
        };
        let cost_text = format!("{} Prestige Ranks", cost);
        put_text(buffer, row, inner_left, "Cost: ", Color::DarkGray);
        put_text(buffer, row, inner_left + 6, &cost_text, cost_fg);
        row += 1;
        if row < bottom {
            let have_text = format!("{} Prestige Ranks", prestige_rank);
            put_text(buffer, row, inner_left, "You have: ", Color::DarkGray);
            put_text(buffer, row, inner_left + 10, &have_text, Color::White);
        }
    } else if room == HavenRoomId::StormForge {
        use crate::achievements::AchievementId;
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);
        if has_stormbreaker {
            put_text(
                buffer,
                row,
                inner_left,
                "\u{2726} Stormbreaker forged!",
                Color::Yellow,
            );
            row += 1;
            if row < bottom {
                put_text(
                    buffer,
                    row,
                    inner_left,
                    "Zone 10 boss accessible",
                    Color::Green,
                );
            }
        } else {
            put_text(
                buffer,
                row,
                inner_left,
                "Press [Enter] to forge",
                Color::Yellow,
            );
            row += 1;
            if row < bottom {
                put_text(
                    buffer,
                    row,
                    inner_left,
                    "Requires: Storm Leviathan + 25 PR",
                    Color::DarkGray,
                );
            }
        }
    } else {
        put_text(
            buffer,
            row,
            inner_left,
            "\u{2713} Max tier reached",
            Color::Green,
        );
    }
}
