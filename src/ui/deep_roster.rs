//! The Deep — Roster and Recruit sub-view rendering.

use crate::deep::{DeepState, DeepUiState, MercArchetype, MercStatus, Mercenary};
use chrono::Utc;
use ratatui::style::Color;

use super::deep_missions::archetype_color;
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{put_cell, put_text, put_text_centered, SceneCell};

/// Infer quality tier glyph + color from base stats vs archetype baseline.
fn quality_glyph(merc: &Mercenary) -> (char, Color) {
    let (bp, br, be) = merc.archetype.base_stats();
    let base_total = bp + br + be;
    let actual_total = merc.power + merc.resilience + merc.expertise;
    if actual_total >= base_total + 20 {
        ('\u{2726}', Color::Rgb(255, 215, 0)) // ✦ Elite (gold)
    } else if actual_total >= base_total + 10 {
        ('\u{2605}', Color::Yellow) // ★ Rare
    } else if actual_total >= base_total + 5 {
        ('\u{25c9}', Color::Green) // ◉ Uncommon
    } else {
        ('\u{25cf}', Color::White) // ● Common
    }
}

/// Quality tier label text.
fn quality_label(merc: &Mercenary) -> (&'static str, Color) {
    let (bp, br, be) = merc.archetype.base_stats();
    let base_total = bp + br + be;
    let actual_total = merc.power + merc.resilience + merc.expertise;
    if actual_total >= base_total + 20 {
        ("Elite", Color::Rgb(255, 215, 0))
    } else if actual_total >= base_total + 10 {
        ("Rare", Color::Yellow)
    } else if actual_total >= base_total + 5 {
        ("Uncommon", Color::Green)
    } else {
        ("Common", Color::White)
    }
}

/// 3-letter archetype abbreviation.
fn archetype_abbrev(archetype: MercArchetype) -> &'static str {
    match archetype {
        MercArchetype::Vanguard => "VAN",
        MercArchetype::Scout => "SCT",
        MercArchetype::Arcanist => "ARC",
        MercArchetype::Medic => "MED",
        MercArchetype::Saboteur => "SAB",
    }
}

/// Archetype role tag and description.
fn archetype_role_desc(archetype: MercArchetype) -> (&'static str, &'static str) {
    match archetype {
        MercArchetype::Vanguard => (
            "Frontline tank",
            "High Power + Resilience. Reduces casualties.",
        ),
        MercArchetype::Scout => (
            "Recon specialist",
            "High Expertise. Better auto-resolve, faster missions.",
        ),
        MercArchetype::Arcanist => (
            "Elemental expert",
            "Highest Expertise. Counters hazards. Fragile.",
        ),
        MercArchetype::Medic => (
            "Squad healer",
            "Highest Resilience. Prevents permanent loss.",
        ),
        MercArchetype::Saboteur => (
            "Trap specialist",
            "High Expertise. Speeds missions, alternate routes.",
        ),
    }
}

/// Injury severity label and color from missions_remaining.
fn injury_severity_display(missions_remaining: u32) -> (&'static str, Color) {
    match missions_remaining {
        1 => ("Light", Color::Yellow),
        2 => ("Moderate", Color::LightRed),
        _ => ("Severe", Color::Red),
    }
}

/// Archetype-flavored injury description.
fn injury_flavor(archetype: MercArchetype, missions_remaining: u32) -> &'static str {
    use MercArchetype::*;
    match (archetype, missions_remaining) {
        // Light (1 mission)
        (Vanguard, 1) => "bruised but standing",
        (Scout, 1) => "twisted ankle in the dark",
        (Arcanist, 1) => "overchanneled, stabilizing",
        (Medic, 1) => "healed others first",
        (Saboteur, 1) => "trap misfired nearby",
        // Moderate (2 missions)
        (Vanguard, 2) => "shield arm broken",
        (Scout, 2) => "fell into a fissure",
        (Arcanist, 2) => "ward collapsed inward",
        (Medic, 2) => "no one left to tend her wounds",
        (Saboteur, 2) => "caught in his own device",
        // Severe (3+ missions)
        (Vanguard, _) => "took the hit for the squad",
        (Scout, _) => "barely made it back",
        (Arcanist, _) => "mind touched something old",
        (Medic, _) => "spent everything keeping them alive",
        (Saboteur, _) => "the mechanism wasn't done yet",
    }
}

/// Cumulative missions needed to reach a given level (from level 1).
fn missions_for_level(level: u32) -> u32 {
    (1..level).map(Mercenary::missions_to_next_level).sum()
}

/// Render a filled/empty progress bar within the scene buffer.
fn render_merc_progress_bar(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    bar_width: usize,
    progress: f64,
) {
    let filled = ((progress * bar_width as f64).round() as usize).min(bar_width);
    for i in 0..filled {
        put_cell(buffer, row, col + i as i32, '\u{2588}', Color::Cyan);
    }
    for i in filled..bar_width {
        put_cell(
            buffer,
            row,
            col + i as i32,
            '\u{2591}',
            Color::Rgb(30, 40, 60),
        );
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

    // ── Summary header ──
    let summary = format!(
        "Mercs: {}/{}    Guild Rank: {} ({})",
        roster.len(),
        rank.max_roster(),
        rank.0,
        rank.display_name()
    );
    put_text(buffer, 0, 1, &summary, Color::DarkGray);

    // ── Footer ──
    put_text(
        buffer,
        height as i32 - 1,
        1,
        "[\u{2191}/\u{2193}] Navigate  [Esc] Back",
        Color::DarkGray,
    );
    let help_hint = "[?] Help";
    let help_col = (width as i32 - help_hint.len() as i32 - 1).max(1);
    put_text(
        buffer,
        height as i32 - 1,
        help_col,
        help_hint,
        Color::Rgb(50, 70, 100),
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

/// Status glyph + label for compact display.
fn merc_status_compact(status: &MercStatus) -> (String, Color) {
    match status {
        MercStatus::Available => ("Ready".to_string(), Color::Green),
        MercStatus::OnMission(_) => ("\u{25cf} Mission".to_string(), Color::Cyan),
        MercStatus::Injured { missions_remaining } => (
            format!("\u{2695} {} miss", missions_remaining),
            Color::Yellow,
        ),
        MercStatus::Lost => ("\u{2716} Lost".to_string(), Color::Red),
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
    //       cursor(2) name(14) sp arch(5) sp lv(2) sp pwr(3) sp res(3) gap status
    put_text(
        buffer,
        row,
        1,
        "  Name           Role  Lv  Pwr Res  Status",
        Color::DarkGray,
    );
    row += 1;

    for (i, merc) in roster.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let abbrev = archetype_abbrev(merc.archetype);
        let (status_str, status_color) = merc_status_compact(&merc.status);

        // Fixed column layout:
        // cursor(2) + name(14) + sp(1) + abbrev(3) + sp(2) + lv(2) + sp(2) + pwr(3) + sp(1) + res(3) + gap(2) = col 35
        let line = format!(
            "{}{:14} {:3} {:2}  {:3} {:3}  {}",
            cursor,
            &merc.name[..merc.name.len().min(14)],
            abbrev,
            merc.level,
            merc.effective_power(),
            merc.effective_resilience(),
            status_str,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(
            buffer,
            row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        // Archetype abbreviation colored
        let arch_col = 18i32; // after cursor(2) + name(14) + sp(1) + offset 1
        put_text(
            buffer,
            row,
            arch_col,
            abbrev,
            archetype_color(merc.archetype),
        );
        // Status colored at fixed column
        // cursor(2) + name(14) + sp(1) + abbrev(3) + sp(2) + lv(2) + sp(2) + pwr(3) + sp(1) + res(3) + gap(2) = 35
        let status_col = 1 + 35;
        put_text(buffer, row, status_col, &status_str, status_color);
        row += 1; // no blank row — glyphs provide visual rhythm
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
    let list_width = (width * 55 / 100).max(24).min(width.saturating_sub(20));
    let detail_left = list_width as i32;
    let detail_inner_left = detail_left + 1;

    // Inner divider
    let glyphs = super::panel_border_chars();
    for r in content_top..content_bottom {
        super::scene_fx::put_cell(buffer, r, detail_left, glyphs.v, Color::Rgb(40, 60, 80));
    }

    // Left: merc list with column header
    let header_row = content_top;
    put_text(
        buffer,
        header_row,
        1,
        "  Name           Archetype  Lv Pwr Res  Status",
        Color::DarkGray,
    );

    let mut list_row = header_row + 1;
    for (i, merc) in roster.iter().enumerate() {
        if list_row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let (status_str, status_color) = merc_status_compact(&merc.status);
        let (qg, qc) = quality_glyph(merc);
        let arch_name =
            &merc.archetype.display_name()[..merc.archetype.display_name().len().min(8)];

        let line = format!(
            "{}{:14} {:8} {} {:2} {:3} {:3}  {}",
            cursor,
            &merc.name[..merc.name.len().min(14)],
            arch_name,
            qg,
            merc.level,
            merc.effective_power(),
            merc.effective_resilience(),
            status_str,
        );
        put_text(buffer, list_row, 1, &line, Color::White);
        put_text(
            buffer,
            list_row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        // Archetype colored
        put_text(
            buffer,
            list_row,
            17,
            arch_name,
            archetype_color(merc.archetype),
        );
        // Quality glyph colored
        let qg_col = 17 + 8 + 1; // after archetype + space
        let qg_str = format!("{}", qg);
        put_text(buffer, list_row, qg_col, &qg_str, qc);
        // Status colored at fixed offset
        // cursor(2) + name(14) + sp(1) + arch(8) + sp(1) + qg(1) + sp(1) + lv(2) + sp(1) + pwr(3) + sp(1) + res(3) + gap(2) = 40
        let status_offset = 1 + 40;
        put_text(buffer, list_row, status_offset, &status_str, status_color);
        list_row += 1;
    }

    // Recruit slot
    let recruit_row = list_row;
    if recruit_row < content_bottom
        && roster.len() < deep.persistent.guild_rank.max_roster() as usize
    {
        put_text(
            buffer,
            recruit_row,
            1,
            "  [Recruit slot available]",
            Color::DarkGray,
        );
    }

    // Right: detail panel for selected merc
    let Some(merc) = roster.get(ui.selected_index) else {
        return;
    };

    let mut row = content_top;
    let (ql, _qlc) = quality_label(merc);
    let (qg, qc) = quality_glyph(merc);

    // Name + quality
    put_text(buffer, row, detail_inner_left, &merc.name, Color::White);
    let quality_str = format!("{} {}", qg, ql);
    let quality_col = detail_inner_left + merc.name.len() as i32 + 2;
    put_text(buffer, row, quality_col, &quality_str, qc);
    row += 1;

    // Archetype + level + missions
    let sub_header = format!(
        "{}  \u{00b7}  Level {}  \u{00b7}  Missions: {}",
        merc.archetype.display_name(),
        merc.level,
        merc.missions_completed
    );
    put_text(buffer, row, detail_inner_left, &sub_header, Color::DarkGray);
    put_text(
        buffer,
        row,
        detail_inner_left,
        merc.archetype.display_name(),
        archetype_color(merc.archetype),
    );
    row += 1;

    // Role description
    row += 1;
    let (role_tag, role_desc) = archetype_role_desc(merc.archetype);
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Role: {}", role_tag),
        Color::Cyan,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  {}", role_desc),
        Color::DarkGray,
    );
    row += 1;

    // Stats with hints
    row += 1;
    put_text(buffer, row, detail_inner_left, "Stats:", Color::Cyan);
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!(
            "  Power:      {:3}  (combat effectiveness)",
            merc.effective_power()
        ),
        Color::White,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!(
            "  Resilience: {:3}  (injury resistance)",
            merc.effective_resilience()
        ),
        Color::White,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  Expertise:  {:3}  (event bonuses)", merc.expertise),
        Color::White,
    );
    row += 1;

    // Progression bar
    row += 1;
    let missions_needed = Mercenary::missions_to_next_level(merc.level);
    let missions_at_level_start = missions_for_level(merc.level);
    let progress_in_level = merc
        .missions_completed
        .saturating_sub(missions_at_level_start);
    let progress_ratio = if missions_needed > 0 {
        (progress_in_level as f64 / missions_needed as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    put_text(buffer, row, detail_inner_left, "Progression:", Color::Cyan);
    row += 1;
    let bar_width = 10usize;
    let bar_col = detail_inner_left + 2;
    render_merc_progress_bar(buffer, row, bar_col, bar_width, progress_ratio);
    let progress_str = format!(
        "  {}/{}  \u{2192} Lv {}",
        progress_in_level,
        missions_needed,
        merc.level + 1
    );
    put_text(
        buffer,
        row,
        bar_col + bar_width as i32,
        &progress_str,
        Color::DarkGray,
    );
    row += 1;

    // Level-up stat preview
    let next_level = merc.level + 1;
    let next_pwr = merc.effective_power() + 3;
    let next_res = merc.effective_resilience() + 2;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  At Lv{}:  Pwr {}  Res {}", next_level, next_pwr, next_res),
        Color::DarkGray,
    );
    row += 1;

    // Status section
    row += 1;
    put_text(buffer, row, detail_inner_left, "Status:", Color::Cyan);
    row += 1;
    match &merc.status {
        MercStatus::Available => {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "  Ready for assignment",
                Color::Green,
            );
        }
        MercStatus::OnMission(mission_id) => {
            // Find the mission and show type + progress
            let now = Utc::now();
            if let Some(mission) = deep
                .prestige
                .active_missions
                .iter()
                .find(|m| m.id == *mission_id)
            {
                let progress = mission.progress(now);
                let remaining_secs = (mission.ends_at - now).num_seconds().max(0);
                let type_name = mission.mission_type.display_name();
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!(
                        "  On Mission \u{2014} Layer {} {}",
                        mission.layer, type_name
                    ),
                    Color::Cyan,
                );
                row += 1;
                // Mission progress bar
                let bar_col = detail_inner_left + 2;
                let bar_width = 12usize;
                render_merc_progress_bar(buffer, row, bar_col, bar_width, progress);
                let eta_str = if remaining_secs > 3600 {
                    format!(
                        "  {}% \u{2014} ~{}h {}m left",
                        (progress * 100.0) as u32,
                        remaining_secs / 3600,
                        (remaining_secs % 3600) / 60,
                    )
                } else {
                    format!(
                        "  {}% \u{2014} ~{}m left",
                        (progress * 100.0) as u32,
                        (remaining_secs / 60).max(1),
                    )
                };
                put_text(
                    buffer,
                    row,
                    bar_col + bar_width as i32,
                    &eta_str,
                    Color::DarkGray,
                );
            } else {
                put_text(buffer, row, detail_inner_left, "  On Mission", Color::Cyan);
            }
        }
        MercStatus::Injured { missions_remaining } => {
            let (severity_label, severity_color) = injury_severity_display(*missions_remaining);
            let flavor = injury_flavor(merc.archetype, *missions_remaining);
            let hours = missions_remaining * 6;
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  Injured \u{2014} {} \u{2014} {}", severity_label, flavor),
                severity_color,
            );
            row += 1;
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!(
                    "  Recovery: ~{} missions remaining (~{}h)",
                    missions_remaining, hours
                ),
                Color::DarkGray,
            );
        }
        MercStatus::Lost => {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "  Permanently lost",
                Color::Red,
            );
        }
    }
}

// ── Recruit view ─────────────────────────────────────────────────────────────

/// Format a duration in seconds as a human-readable countdown (e.g. "5h 23m").
fn format_countdown(secs: i64) -> String {
    if secs <= 0 {
        return "Now".to_string();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h == 0 {
        format!("{}m", m)
    } else if m == 0 {
        format!("{}h", h)
    } else {
        format!("{}h {}m", h, m)
    }
}

pub(super) fn render_recruit(
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
    let roster_count = deep.prestige.roster.len();
    let max_roster = rank.max_roster() as usize;
    let pool = &deep.prestige.recruit_pool;
    let marks = deep.prestige.warband_marks;

    // ── Summary header ──
    let now = Utc::now();
    let refresh_secs = (pool.refreshed_at + chrono::Duration::hours(24) - now)
        .num_seconds()
        .max(0);
    let refresh_str = if refresh_secs <= 0 {
        "Refreshing...".to_string()
    } else {
        format!("Refreshes in {}", format_countdown(refresh_secs))
    };
    let capacity_color = if roster_count < max_roster {
        Color::Green
    } else {
        Color::LightRed
    };
    let summary = format!(
        "Roster: {}/{}    Marks: {}    {}",
        roster_count, max_roster, marks, refresh_str,
    );
    put_text(buffer, 0, 1, &summary, Color::DarkGray);
    // Highlight capacity portion
    let cap_str = format!("{}/{}", roster_count, max_roster);
    put_text(buffer, 0, 9, &cap_str, capacity_color);

    // ── Footer ──
    let footer = if ctx.tier <= SizeTier::S {
        "[\u{2191}/\u{2193}] Select  [Enter] Recruit  [Esc] Back"
    } else {
        "[\u{2191}/\u{2193}] Navigate  [Enter] Recruit  [Esc] Back"
    };
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);
    let recruit_help_hint = "[?] Help";
    let recruit_help_col = (width as i32 - recruit_help_hint.len() as i32 - 1).max(1);
    put_text(
        buffer,
        height as i32 - 1,
        recruit_help_col,
        recruit_help_hint,
        Color::Rgb(50, 70, 100),
    );

    let content_top = 1i32;
    let content_bottom = height as i32 - 1;

    if pool.candidates.is_empty() {
        put_text_centered(
            buffer,
            content_top + (content_bottom - content_top) / 2,
            width,
            "No recruits available.",
            Color::DarkGray,
        );
        put_text_centered(
            buffer,
            content_top + (content_bottom - content_top) / 2 + 1,
            width,
            &format!("Pool refreshes in {}.", format_countdown(refresh_secs)),
            Color::Rgb(50, 70, 100),
        );
        return;
    }

    let is_compact = ctx.tier <= SizeTier::S || width < 60;

    if is_compact {
        render_recruit_compact(
            buffer,
            width,
            height,
            deep,
            ui,
            content_top,
            content_bottom,
            marks,
            roster_count,
            max_roster,
        );
    } else {
        render_recruit_split(
            buffer,
            width,
            height,
            deep,
            ui,
            content_top,
            content_bottom,
            marks,
            roster_count,
            max_roster,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_recruit_compact(
    buffer: &mut [Vec<SceneCell>],
    _width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    marks: u32,
    roster_count: usize,
    max_roster: usize,
) {
    let pool = &deep.prestige.recruit_pool;
    let mut row = content_top;

    // Column header
    put_text(
        buffer,
        row,
        1,
        "  Name           Arch      Pwr  Cost",
        Color::DarkGray,
    );
    row += 1;

    for (i, candidate) in pool.candidates.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let cost = pool.recruit_costs.get(i).copied().unwrap_or(0);
        let can_afford = marks >= cost && roster_count < max_roster;
        let arch_str =
            &candidate.archetype.display_name()[..candidate.archetype.display_name().len().min(8)];

        let line = format!(
            "{}{:14} {:8} {:3}  {}M",
            cursor,
            &candidate.name[..candidate.name.len().min(14)],
            arch_str,
            candidate.effective_power(),
            cost,
        );
        let text_color = if can_afford {
            Color::White
        } else {
            Color::DarkGray
        };
        put_text(buffer, row, 1, &line, text_color);
        put_text(
            buffer,
            row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        // Archetype colored
        let arch_col = 17i32;
        put_text(
            buffer,
            row,
            arch_col,
            arch_str,
            if can_afford {
                archetype_color(candidate.archetype)
            } else {
                Color::DarkGray
            },
        );
        // Cost colored (green if affordable, red if not)
        let cost_str = format!("{}M", cost);
        let cost_col = 1 + 2 + 14 + 1 + 8 + 1 + 3 + 2;
        let cost_color = if marks >= cost {
            Color::Green
        } else {
            Color::LightRed
        };
        put_text(
            buffer,
            row,
            cost_col,
            &cost_str,
            if roster_count < max_roster {
                cost_color
            } else {
                Color::DarkGray
            },
        );
        row += 1;
    }

    // Roster full warning
    if roster_count >= max_roster && row < content_bottom {
        row += 1;
        put_text(
            buffer,
            row,
            1,
            "Roster full. Upgrade Guild Rank for more slots.",
            Color::LightRed,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_recruit_split(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    marks: u32,
    roster_count: usize,
    max_roster: usize,
) {
    let pool = &deep.prestige.recruit_pool;
    let list_width = (width * 50 / 100).max(24).min(width.saturating_sub(20));
    let detail_left = list_width as i32;
    let detail_inner_left = detail_left + 1;

    // Inner divider
    let glyphs = super::panel_border_chars();
    for r in content_top..content_bottom {
        put_cell(buffer, r, detail_left, glyphs.v, Color::Rgb(40, 60, 80));
    }

    // Left: candidate list with column header
    put_text(
        buffer,
        content_top,
        1,
        "  Name           Archetype  Pwr  Cost",
        Color::DarkGray,
    );
    let list_inner_top = content_top + 1;

    for (i, candidate) in pool.candidates.iter().enumerate() {
        let row = list_inner_top + i as i32;
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let cost = pool.recruit_costs.get(i).copied().unwrap_or(0);
        let can_afford = marks >= cost && roster_count < max_roster;
        let arch_str =
            &candidate.archetype.display_name()[..candidate.archetype.display_name().len().min(8)];

        let line = format!(
            "{}{:14} {:8}   {:3}  {}M",
            cursor,
            &candidate.name[..candidate.name.len().min(14)],
            arch_str,
            candidate.effective_power(),
            cost,
        );
        let text_color = if can_afford {
            Color::White
        } else {
            Color::DarkGray
        };
        put_text(buffer, row, 1, &line, text_color);
        put_text(
            buffer,
            row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        // Archetype colored
        put_text(
            buffer,
            row,
            17,
            arch_str,
            if can_afford {
                archetype_color(candidate.archetype)
            } else {
                Color::DarkGray
            },
        );
        // Cost colored
        let cost_str = format!("{}M", cost);
        let cost_col = 1 + 2 + 14 + 1 + 8 + 3 + 3 + 2;
        let cost_color = if marks >= cost {
            Color::Green
        } else {
            Color::LightRed
        };
        put_text(
            buffer,
            row,
            cost_col,
            &cost_str,
            if roster_count < max_roster {
                cost_color
            } else {
                Color::DarkGray
            },
        );
    }

    // Roster full warning at bottom of list
    let warn_row = list_inner_top + pool.candidates.len() as i32 + 1;
    if roster_count >= max_roster && warn_row < content_bottom {
        put_text(buffer, warn_row, 1, "Roster full!", Color::LightRed);
    }

    // Right: detail panel for selected candidate
    let Some(candidate) = pool.candidates.get(ui.selected_index) else {
        return;
    };
    let cost = pool
        .recruit_costs
        .get(ui.selected_index)
        .copied()
        .unwrap_or(0);
    let can_afford = marks >= cost;

    let mut row = content_top;

    // Name + quality
    let (qg, qc) = quality_glyph(candidate);
    let (ql, _qlc) = quality_label(candidate);
    put_text(
        buffer,
        row,
        detail_inner_left,
        &candidate.name,
        Color::White,
    );
    let quality_str = format!("{} {}", qg, ql);
    let quality_col = detail_inner_left + candidate.name.len() as i32 + 2;
    put_text(buffer, row, quality_col, &quality_str, qc);
    row += 1;

    // Archetype + level
    let sub_header = format!(
        "{}  \u{00b7}  Level {}",
        candidate.archetype.display_name(),
        candidate.level,
    );
    put_text(buffer, row, detail_inner_left, &sub_header, Color::DarkGray);
    put_text(
        buffer,
        row,
        detail_inner_left,
        candidate.archetype.display_name(),
        archetype_color(candidate.archetype),
    );
    row += 1;

    // Role description
    row += 1;
    let (role_tag, role_desc) = archetype_role_desc(candidate.archetype);
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Role: {}", role_tag),
        Color::Cyan,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  {}", role_desc),
        Color::DarkGray,
    );
    row += 1;

    // Stats with hints
    row += 1;
    put_text(buffer, row, detail_inner_left, "Stats:", Color::Cyan);
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!(
            "  Power:      {:3}  (combat effectiveness)",
            candidate.effective_power()
        ),
        Color::White,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!(
            "  Resilience: {:3}  (injury resistance)",
            candidate.effective_resilience()
        ),
        Color::White,
    );
    row += 1;
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  Expertise:  {:3}  (event bonuses)", candidate.expertise),
        Color::White,
    );
    row += 1;

    // Cost section
    row += 1;
    put_text(buffer, row, detail_inner_left, "Cost:", Color::Cyan);
    row += 1;
    let cost_color = if can_afford {
        Color::Green
    } else {
        Color::LightRed
    };
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  {} Warband Marks", cost),
        cost_color,
    );
    row += 1;
    if !can_afford {
        put_text(
            buffer,
            row,
            detail_inner_left,
            &format!("  (Need {} more)", cost.saturating_sub(marks)),
            Color::LightRed,
        );
        row += 1;
    }

    // Roster capacity
    row += 1;
    let cap_color = if roster_count < max_roster {
        Color::Green
    } else {
        Color::LightRed
    };
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Roster: {}/{}", roster_count, max_roster),
        cap_color,
    );
    if roster_count >= max_roster {
        row += 1;
        put_text(
            buffer,
            row,
            detail_inner_left,
            "  Upgrade Guild Rank for more slots",
            Color::DarkGray,
        );
    }
}
