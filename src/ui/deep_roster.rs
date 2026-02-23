//! The Deep — Roster sub-view rendering.

use crate::deep::{DeepState, DeepUiState, MercStatus};
use ratatui::style::Color;

use super::deep_missions::archetype_color;
use super::deep_scene::DEEP_BORDER_COLOR;
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{put_text, put_text_centered, SceneCell};

fn merc_status_label(status: &MercStatus) -> (&'static str, Color) {
    match status {
        MercStatus::Available => ("Ready", Color::Green),
        MercStatus::OnMission(_) => ("On mission", Color::Cyan),
        MercStatus::Injured { missions_remaining } => {
            // We cannot format here since we need &'static str, so use a fixed label.
            let _ = missions_remaining;
            ("Injured", Color::Yellow)
        }
        MercStatus::Lost => ("Lost", Color::Red),
    }
}

fn merc_status_detail(status: &MercStatus) -> String {
    match status {
        MercStatus::Available => "Ready for assignment".to_string(),
        MercStatus::OnMission(id) => format!("On mission #{}", id),
        MercStatus::Injured { missions_remaining } => {
            format!("Injured ({} missions)", missions_remaining)
        }
        MercStatus::Lost => "Permanently lost".to_string(),
    }
}

pub(super) fn render_roster(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    ctx: &LayoutContext,
) {
    if height < 4 || width < 20 {
        return;
    }

    let rank = deep.persistent.guild_rank;
    let roster = &deep.prestige.roster;

    // ── Title ──
    put_text(buffer, 0, 1, "ROSTER", DEEP_BORDER_COLOR);
    put_text(
        buffer,
        0,
        9,
        &format!(
            "Mercs: {}/{}    Guild Rank: {} ({})",
            roster.len(),
            rank.max_roster(),
            rank.0,
            rank.display_name()
        ),
        Color::DarkGray,
    );

    // ── Footer ──
    put_text(
        buffer,
        height as i32 - 1,
        1,
        "[\u{2191}/\u{2193}] Navigate  [Esc] Back",
        Color::DarkGray,
    );

    let content_top = 1i32;
    let content_bottom = height as i32 - 1;

    if roster.is_empty() {
        put_text_centered(
            buffer,
            content_top + (content_bottom - content_top) / 2,
            width,
            "No mercenaries hired yet.",
            Color::DarkGray,
        );
        put_text_centered(
            buffer,
            content_top + (content_bottom - content_top) / 2 + 1,
            width,
            "Start missions to recruit new members.",
            Color::Rgb(50, 70, 100),
        );
        return;
    }

    let is_compact = ctx.tier <= SizeTier::S || width < 60;

    if is_compact {
        render_roster_compact(buffer, width, height, deep, ui, content_top, content_bottom);
    } else {
        render_roster_split(buffer, width, height, deep, ui, content_top, content_bottom);
    }
}

fn render_roster_compact(
    buffer: &mut [Vec<SceneCell>],
    _width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
) {
    let roster = &deep.prestige.roster;
    let mut row = content_top;

    // Column header
    put_text(buffer, row, 1, "  Name           Arch      Lv  Pwr  Status", Color::DarkGray);
    row += 1;

    for (i, merc) in roster.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let (status_label, status_color) = merc_status_label(&merc.status);
        let arch_str = &merc.archetype.display_name()[..merc.archetype.display_name().len().min(8)];

        let line = format!(
            "{}{:14} {:8} {:2}  {:3}  {}",
            cursor,
            &merc.name[..merc.name.len().min(14)],
            arch_str,
            merc.level,
            merc.effective_power(),
            status_label,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
        // Archetype colored
        let arch_col = 17i32;
        put_text(buffer, row, arch_col, arch_str, archetype_color(merc.archetype));
        // Status colored
        let status_col = line.rfind(status_label).map(|p| p as i32 + 1).unwrap_or(42);
        put_text(buffer, row, status_col, status_label, status_color);
        row += 1;
    }

    // Recruit slot if roster not full
    if row < content_bottom && roster.len() < deep.persistent.guild_rank.max_roster() as usize {
        put_text(
            buffer,
            row,
            1,
            "  [Recruit slot available]",
            Color::DarkGray,
        );
    }
}

fn render_roster_split(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
) {
    let roster = &deep.prestige.roster;
    let list_width = (width * 60 / 100).max(24).min(width.saturating_sub(20));
    let detail_left = list_width as i32;
    let detail_inner_left = detail_left + 1;

    // Inner divider
    let glyphs = super::panel_border_chars();
    for r in content_top..content_bottom {
        super::scene_fx::put_cell(buffer, r, detail_left, glyphs.v, Color::Rgb(40, 60, 80));
    }

    // Left: merc list with column header
    let header_row = content_top;
    put_text(buffer, header_row, 1, "  Name           Archetype  Lv  Pwr  Res  Status", Color::DarkGray);

    for (i, merc) in roster.iter().enumerate() {
        let row = header_row + 1 + i as i32;
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let (status_label, status_color) = merc_status_label(&merc.status);

        let line = format!(
            "{}{:14} {:8}   {:2}  {:3}  {:3}  {}",
            cursor,
            &merc.name[..merc.name.len().min(14)],
            &merc.archetype.display_name()[..merc.archetype.display_name().len().min(8)],
            merc.level,
            merc.effective_power(),
            merc.effective_resilience(),
            status_label,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
        put_text(buffer, row, 17, &merc.archetype.display_name()[..merc.archetype.display_name().len().min(8)], archetype_color(merc.archetype));
        // Status at end
        let stat_offset = 1 + 2 + 14 + 1 + 8 + 3 + 2 + 1 + 3 + 2 + 3 + 2; // approximate
        put_text(buffer, row, stat_offset, status_label, status_color);
    }

    // Recruit slot
    let recruit_row = header_row + 1 + roster.len() as i32;
    if recruit_row < content_bottom && roster.len() < deep.persistent.guild_rank.max_roster() as usize {
        put_text(buffer, recruit_row, 1, "  [Recruit slot available]", Color::DarkGray);
    }

    // Right: detail panel for selected merc
    let Some(merc) = roster.get(ui.selected_index) else {
        return;
    };

    let mut row = content_top;
    let (status_label, status_color) = merc_status_label(&merc.status);

    put_text(buffer, row, detail_inner_left, &merc.name, Color::White);
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Archetype: {}", merc.archetype.display_name()),
        archetype_color(merc.archetype),
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Level: {}    Missions: {}", merc.level, merc.missions_completed),
        Color::DarkGray,
    );
    row += 1;

    row += 1;
    put_text(buffer, row, detail_inner_left, "Stats:", Color::Cyan);
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  Power:      {}", merc.effective_power()),
        Color::White,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  Resilience: {}", merc.effective_resilience()),
        Color::White,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  Expertise:  {}", merc.expertise),
        Color::White,
    );
    row += 1;

    row += 1;
    put_text(buffer, row, detail_inner_left, "Status:", Color::Cyan);
    row += 1;
    put_text(buffer, row, detail_inner_left, &format!("  {}", status_label), status_color);
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  {}", merc_status_detail(&merc.status)),
        Color::DarkGray,
    );
}
