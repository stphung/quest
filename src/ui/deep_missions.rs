//! The Deep — Hub and New Mission sub-view rendering.

use crate::deep::{
    base_marks_earned, event_trigger_points, familiarity_gain, AvailableMission, DeepState,
    DeepUiState, LayerTier, MercArchetype, MercStatus, Mission, MissionStatus, MissionType,
};
use chrono::Utc;
use ratatui::style::Color;

use super::deep_shared::{draw_deep_card, truncate_text};
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{current_millis, put_cell, put_text, put_text_centered, SceneCell};

// ── Constants ────────────────────────────────────────────────────────────────

/// Section label color used throughout The Deep UI.
const SECTION_LABEL_COLOR: Color = Color::Rgb(80, 160, 220);

/// Amber color for Warband Marks currency displays.
pub(super) const MARKS_COLOR: Color = Color::Rgb(220, 180, 60);

/// Render a labeled section rule: `── LABEL ───────── (count)`
fn render_section_rule(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    width: usize,
    label: &str,
    count: Option<usize>,
) {
    let count_str = count.map(|c| format!(" ({})", c)).unwrap_or_default();
    let prefix = format!("\u{2500}\u{2500} {} ", label);
    let suffix_len = count_str.len();
    let rule_len = width.saturating_sub(prefix.len() + suffix_len + 2);
    let rule: String = "\u{2500}".repeat(rule_len);
    put_text(buffer, row, 1, &prefix, SECTION_LABEL_COLOR);
    put_text(
        buffer,
        row,
        1 + prefix.len() as i32,
        &rule,
        Color::Rgb(40, 60, 80),
    );
    if !count_str.is_empty() {
        put_text(
            buffer,
            row,
            (width as i32 - suffix_len as i32 - 1).max(0),
            &count_str,
            Color::DarkGray,
        );
    }
}

/// Get the lead merc's name (first squad member) for display.
fn lead_merc_name(deep: &DeepState, squad: &[u64]) -> String {
    squad
        .first()
        .and_then(|id| deep.prestige.find_merc(*id))
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

// ── Color helpers ─────────────────────────────────────────────────────────────

pub(super) fn mission_type_color(t: MissionType) -> Color {
    match t {
        MissionType::SupplyRun => Color::Green,
        MissionType::Recon => Color::Cyan,
        MissionType::Expedition => Color::Yellow,
        MissionType::Breakthrough => Color::LightRed,
        MissionType::GatewayExpedition => Color::Rgb(255, 215, 0),
        MissionType::Construction(_) => Color::Blue,
    }
}

pub(super) fn archetype_color(a: MercArchetype) -> Color {
    match a {
        MercArchetype::Vanguard => Color::LightRed,
        MercArchetype::Scout => Color::Cyan,
        MercArchetype::Arcanist => Color::Magenta,
        MercArchetype::Medic => Color::Green,
        MercArchetype::Saboteur => Color::Yellow,
    }
}

fn risk_label(tier: u8) -> &'static str {
    match tier {
        0 => "Safe",
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "?",
    }
}

/// Display label for a mission type, showing infrastructure type for Construction missions.
fn mission_type_label(mt: MissionType) -> String {
    match mt {
        MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
        other => other.display_name().to_string(),
    }
}

/// Short code used for compact mission identity badges.
fn mission_type_code(mt: MissionType) -> &'static str {
    match mt {
        MissionType::SupplyRun => "SUP",
        MissionType::Recon => "RCN",
        MissionType::Expedition => "EXP",
        MissionType::Breakthrough => "BRK",
        MissionType::GatewayExpedition => "GAT",
        MissionType::Construction(_) => "BLD",
    }
}

/// Stable callsign for active/completed mission cards.
fn mission_callsign(mission: &Mission) -> String {
    format!(
        "{}-L{:02}-{:03}",
        mission_type_code(mission.mission_type),
        mission.layer.min(99),
        mission.id % 1000
    )
}

/// Human-readable urgency badge for active mission cards.
fn mission_urgency_badge(mission: &Mission, remaining_secs: u64) -> (&'static str, Color) {
    if mission.has_pending_event() {
        ("EVENT NOW", Color::Yellow)
    } else if remaining_secs <= 15 * 60 {
        ("CRITICAL ETA", Color::LightRed)
    } else if remaining_secs <= 60 * 60 {
        ("HIGH ETA", Color::Rgb(255, 170, 80))
    } else {
        ("STABLE", Color::Rgb(80, 130, 190))
    }
}

fn risk_color(tier: u8) -> Color {
    match tier {
        0 => Color::Green,
        1 => Color::Cyan,
        2 => Color::Yellow,
        3 => Color::LightRed,
        _ => Color::DarkGray,
    }
}

fn mission_layer_header(layer: u32) -> String {
    format!(
        "-- Layer {} ({}) --",
        layer,
        LayerTier::from_layer(layer).display_name()
    )
}

fn mission_layer_header_color(layer: u32) -> Color {
    match LayerTier::from_layer(layer) {
        LayerTier::Shallows => Color::Rgb(70, 170, 90),
        LayerTier::Warrens => Color::Rgb(90, 170, 230),
        LayerTier::Hollows => Color::Rgb(120, 165, 235),
        LayerTier::SunkenReach => Color::Rgb(150, 155, 240),
        LayerTier::Abyss => Color::Rgb(215, 140, 100),
        LayerTier::Void => Color::Rgb(230, 120, 80),
    }
}

/// Risk-tier icon for at-a-glance difficulty in mission lists.
fn risk_icon(tier: u8) -> &'static str {
    match tier {
        0 => "\u{2690}", // ⚐ safe (flag)
        1 => "\u{25b3}", // △ low
        2 => "\u{26a0}", // ⚠ medium (caution)
        3 => "\u{2620}", // ☠ high (skull)
        _ => " ",
    }
}

/// Format seconds as "Xh Ym" compactly.
fn format_hours(secs: u64) -> String {
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

/// Render one available-mission row using fixed column positions for alignment.
///
/// Columns (relative to `left`):
///   cursor(1)  icon(1+sp)  name(variable)  eta  risk  cost
fn render_mission_row(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    left: i32,
    row_width: usize,
    mission: &AvailableMission,
    is_selected: bool,
) {
    let tc = mission_type_color(mission.mission_type);
    let rt = mission.mission_type.risk_tier();

    // Fixed column positions (computed from right edge for alignment)
    let col_cursor = left;
    let col_icon = left + 2;
    let col_name = left + 4;
    // Cost column: "Free" (4) or "NNN Warband Marks" (max ~18). Allocate 18 chars from right edge.
    let col_cost = (left + row_width as i32 - 19).max(col_name + 12);
    // Risk column: "Safe"/"Low"/"Med"/"High" — 4 chars, just left of cost
    let col_risk = (col_cost - 6).max(col_name + 8);
    // ETA column: "2h"/"10h"/"2h 5m" — up to 6 chars, just left of risk
    let col_eta = (col_risk - 7).max(col_name + 4);
    let max_name_chars = (col_eta - col_name - 1).max(4) as usize;

    // Cursor
    if is_selected {
        put_text(buffer, row, col_cursor, ">", Color::Cyan);
    }
    // Risk icon (colored independently)
    put_text(buffer, row, col_icon, risk_icon(rt), risk_color(rt));
    // Type name
    let type_name = mission_type_label(mission.mission_type);
    let name_display = truncate_text(&type_name, max_name_chars);
    put_text(buffer, row, col_name, &name_display, tc);
    // ETA (right-aligned in column)
    let eta = format_hours(mission.duration_secs);
    let eta_col = col_eta + (6 - eta.len() as i32).max(0);
    put_text(buffer, row, eta_col, &eta, Color::DarkGray);
    // Risk label
    let risk = risk_label(rt);
    put_text(buffer, row, col_risk, risk, risk_color(rt));
    // Cost
    let cost_str = if mission.marks_cost > 0 {
        format!("{} Warband Marks", mission.marks_cost)
    } else {
        "Free".to_string()
    };
    let cost_col = col_cost + (18 - cost_str.len() as i32).max(0);
    put_text(buffer, row, cost_col, &cost_str, MARKS_COLOR);
}

/// Render a block-character progress bar.
/// `ratio` is 0.0..=1.0. Bar fits within `width` chars.
fn render_progress_bar(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    width: usize,
    ratio: f64,
    filled_color: Color,
) {
    if width == 0 {
        return;
    }
    let filled = ((ratio * width as f64).round() as usize).min(width);
    let empty = width - filled;
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2592}".repeat(empty);
    let chars: Vec<char> = bar.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let color = if i < filled {
            filled_color
        } else {
            Color::Rgb(30, 40, 60)
        };
        super::scene_fx::put_cell(buffer, row, col + i as i32, *ch, color);
    }
}

/// Tint a rectangular panel background (inclusive bounds).
fn tint_panel_background(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    bg: Color,
) {
    for row in top..=bottom {
        if row < 0 || row as usize >= buffer.len() {
            continue;
        }
        for col in left..=right {
            if col < 0 || col as usize >= buffer[row as usize].len() {
                continue;
            }
            buffer[row as usize][col as usize].bg = bg;
        }
    }
}

/// Draw a boxed outline around a panel (inclusive bounds).
fn draw_panel_outline(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    color: Color,
) {
    if left >= right || top >= bottom {
        return;
    }
    for col in (left + 1)..right {
        put_cell(buffer, top, col, '\u{2500}', color);
        put_cell(buffer, bottom, col, '\u{2500}', color);
    }
    for row in (top + 1)..bottom {
        put_cell(buffer, row, left, '\u{2502}', color);
        put_cell(buffer, row, right, '\u{2502}', color);
    }
    put_cell(buffer, top, left, '\u{250c}', color);
    put_cell(buffer, top, right, '\u{2510}', color);
    put_cell(buffer, bottom, left, '\u{2514}', color);
    put_cell(buffer, bottom, right, '\u{2518}', color);
}

/// Render the roster content.
fn render_hub_roster(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    _ctx: &LayoutContext,
    content_top: i32,
) {
    // ── Prestige cycle hint (first visit only) ──
    let mut header_row = content_top;
    if ui.hub_visit_count <= 1 {
        put_text(
            buffer,
            header_row,
            1,
            "Tip: Deep mercs, missions, marks, and infrastructure persist on prestige.",
            Color::Rgb(50, 80, 110),
        );
        header_row += 1;
    }

    // ── Roster list ──
    let roster_bottom = height as i32 - 2; // leave 2 rows for flash + footer
    let mut row = header_row;

    let roster = &deep.prestige.roster;

    if roster.is_empty() {
        put_text_centered(
            buffer,
            row + 2,
            width,
            "No mercenaries yet. Go to Recruit tab to hire.",
            Color::Rgb(60, 80, 110),
        );
    } else {
        // Spacious column layout — full names, full class names
        let col_cursor = 1i32;
        let col_glyph = 4i32;
        let col_name = 6i32;
        let col_class = 28i32;
        let col_lv = 39i32;
        let col_pwr = 43i32;
        let col_res = 48i32;
        let col_status = 53i32;
        let max_name_chars = (col_class - col_name - 1) as usize;

        put_text(buffer, row, col_name, "Name", Color::DarkGray);
        put_text(buffer, row, col_class, "Class", Color::DarkGray);
        put_text(buffer, row, col_lv, "Lv", Color::DarkGray);
        put_text(buffer, row, col_pwr, "Pwr", Color::DarkGray);
        put_text(buffer, row, col_res, "Res", Color::DarkGray);
        put_text(buffer, row, col_status, "Status", Color::DarkGray);
        row += 1;

        for (i, merc) in roster.values().enumerate() {
            if row >= roster_bottom - 6 {
                break;
            }
            if matches!(merc.status, MercStatus::Lost) {
                continue;
            }
            let is_selected = ui.selected_index == i;
            let (q_glyph, q_color) = super::deep_roster::quality_glyph(merc);
            let class_name = merc.archetype.display_name();
            let (status_label, status_color) = match &merc.status {
                MercStatus::Available => ("Ready", Color::Green),
                MercStatus::OnMission(_) => ("On Mission", Color::Cyan),
                MercStatus::Injured {
                    missions_remaining, ..
                } => super::deep_roster::injury_severity_display(*missions_remaining),
                MercStatus::Lost => ("Lost", Color::Red),
            };

            if is_selected {
                for c in 1..width.saturating_sub(1) {
                    if (row as usize) < buffer.len() && c < buffer[row as usize].len() {
                        buffer[row as usize][c].bg = Color::Rgb(16, 28, 46);
                    }
                }
            }

            if is_selected {
                put_text(buffer, row, col_cursor, ">", Color::Cyan);
            }
            put_cell(buffer, row, col_glyph, q_glyph, q_color);
            let name_display: String = merc.name.chars().take(max_name_chars).collect();
            put_text(buffer, row, col_name, &name_display, Color::White);
            put_text(
                buffer,
                row,
                col_class,
                class_name,
                archetype_color(merc.archetype),
            );
            put_text(
                buffer,
                row,
                col_lv,
                &format!("{:2}", merc.level),
                Color::White,
            );
            put_text(
                buffer,
                row,
                col_pwr,
                &format!("{:3}", merc.effective_power()),
                Color::White,
            );
            put_text(
                buffer,
                row,
                col_res,
                &format!("{:3}", merc.effective_resilience()),
                Color::White,
            );
            put_text(buffer, row, col_status, status_label, status_color);
            row += 1;
        }
    }

    // ── Warband log (below roster) ──
    let log = &deep.prestige.warband_log;
    if !log.is_empty() && row + 3 < roster_bottom {
        render_section_rule(buffer, row, width, "WARBAND LOG", None);
        row += 1;
        for entry in log.iter().rev().take(5) {
            if row >= roster_bottom {
                break;
            }
            let (icon, color) = match entry.outcome {
                crate::deep::MissionOutcome::Success => ("\u{2713}", Color::Green),
                crate::deep::MissionOutcome::PartialSuccess => ("\u{25cb}", Color::Yellow),
                crate::deep::MissionOutcome::Failure => ("\u{2717}", Color::LightRed),
            };
            let line = format!(
                "{} Layer {} \u{2014} {} \u{2014} {} Warband Marks",
                icon, entry.layer, entry.mission_name, entry.marks_earned
            );
            put_text(buffer, row, 1, &line, color);
            row += 1;
        }
    }

    // Atmospheric text (if roster and log are both empty)
    if roster.is_empty() && log.is_empty() {
        let millis = super::scene_fx::current_millis();
        let atmosphere_messages = if deep.persistent.gateway_opened {
            &[
                "The Gateway stands open. The Wellspring waits.",
                "The Wellspring has seen this before. It is patient.",
            ][..]
        } else {
            tier_atmosphere_messages(deep.persistent.frontier_layer())
        };
        let msg_idx = (millis / 8000) as usize % atmosphere_messages.len();
        let atmo_row = row + 2;
        if atmo_row < roster_bottom {
            let atmo_color = if deep.persistent.gateway_opened {
                Color::Rgb(255, 215, 0)
            } else {
                Color::Rgb(40, 60, 90)
            };
            put_text_centered(
                buffer,
                atmo_row,
                width,
                atmosphere_messages[msg_idx],
                atmo_color,
            );
        }
    }
}

// ── Standalone tab views ─────────────────────────────────────────────────────

/// Render the Active missions tab (standalone, no sub-tab toggle).
pub(super) fn render_active(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    _ctx: &LayoutContext,
) {
    if height < 4 || width < 20 {
        return;
    }

    // Footer
    if let Some(msg) = &ui.flash_message {
        put_text(buffer, height as i32 - 2, 1, msg, Color::LightRed);
    }
    let footer =
        "[\u{2190}/\u{2192}] Switch  [\u{2191}/\u{2193}] Navigate  [Enter] Select  [Esc] Close";
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);

    let content_top = 0i32;
    let content_bottom = height as i32 - 2;

    render_active_missions_content(buffer, width, content_top, content_bottom, deep, ui);
}

/// Render the Deploy (available missions) tab (standalone, no sub-tab toggle).
pub(super) fn render_deploy(
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

    let is_compact = ctx.tier <= SizeTier::S || width < 60;

    // Footer
    if let Some(msg) = &ui.flash_message {
        put_text(buffer, height as i32 - 2, 1, msg, Color::LightRed);
    }
    let footer = if ui.staging_mission_index.is_some() {
        "[\u{2191}/\u{2193}] Navigate  [Space] Toggle Merc  [Enter] Launch  [Esc] Cancel"
    } else {
        "[\u{2190}/\u{2192}] Switch  [\u{2191}/\u{2193}] Select  [Enter] Assign Squad  [Esc] Close"
    };
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);

    // Right-aligned marks
    let marks_display = format!("\u{25c6} {} Warband Marks", deep.prestige.warband_marks);
    let marks_col = (width as i32 - marks_display.len() as i32 - 2).max(1);
    let min_marks_col = footer.chars().count() as i32 + 3;
    if marks_col > min_marks_col {
        put_text(
            buffer,
            height as i32 - 1,
            marks_col,
            &marks_display,
            MARKS_COLOR,
        );
    }

    let content_top = 0i32;
    let content_bottom = height as i32 - 2;
    let content_height = (content_bottom - content_top).max(0) as usize;
    let available = &deep.prestige.available_missions;

    if available.is_empty() {
        let mid = content_top + content_height as i32 / 2;
        put_text_centered(
            buffer,
            mid - 1,
            width,
            "No missions available.",
            Color::DarkGray,
        );
        let active_count = deep.prestige.active_mission_count();
        if active_count == 0 && deep.prestige.roster.is_empty() {
            put_text_centered(
                buffer,
                mid,
                width,
                "Recruit mercs first \u{2014} go to Recruit tab.",
                Color::Rgb(50, 70, 100),
            );
        } else if active_count > 0 {
            put_text_centered(
                buffer,
                mid,
                width,
                "Mission pool refreshes over time.",
                Color::Rgb(50, 70, 100),
            );
        } else {
            put_text_centered(
                buffer,
                mid,
                width,
                "Mission pool refreshes periodically.",
                Color::Rgb(50, 70, 100),
            );
        }
        return;
    }

    if is_compact {
        render_new_mission_compact(
            buffer,
            width,
            height,
            deep,
            ui,
            content_top,
            content_bottom,
            available,
        );
    } else {
        render_new_mission_split(
            buffer,
            width,
            height,
            deep,
            ui,
            content_top,
            content_bottom,
            available,
        );
    }
}

/// Render the Roster tab (standalone, no sub-tab toggle).
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

    // Footer
    if let Some(msg) = &ui.flash_message {
        put_text(buffer, height as i32 - 2, 1, msg, Color::LightRed);
    }
    if ui.promotion_pending {
        if let Some(merc) = deep
            .prestige
            .roster
            .values()
            .filter(|m| !matches!(m.status, crate::deep::MercStatus::Lost))
            .nth(ui.selected_index)
        {
            if let Some(target) = merc.quality.next() {
                let target_name = match target {
                    crate::deep::MercQuality::Common => "Common",
                    crate::deep::MercQuality::Uncommon => "Uncommon",
                    crate::deep::MercQuality::Rare => "Rare",
                    crate::deep::MercQuality::Elite => "Elite",
                };
                let cost = crate::deep::promotion_cost(merc.id, target);
                let confirm_msg = format!(
                    "Promote {} to {}? ({} Marks) [Enter] Confirm  [Any] Cancel",
                    merc.name, target_name, cost,
                );
                put_text(buffer, height as i32 - 2, 1, &confirm_msg, Color::Yellow);
            }
        }
    }
    let footer =
        "[\u{2190}/\u{2192}] Switch  [\u{2191}/\u{2193}] Navigate  [P] Promote  [G] Guild Rank  [Esc] Close";
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);
    let help_hint = "[?] Help";
    let help_col = (width as i32 - help_hint.len() as i32 - 1).max(footer.len() as i32 + 2);
    put_text(
        buffer,
        height as i32 - 1,
        help_col,
        help_hint,
        Color::Rgb(50, 70, 100),
    );

    render_hub_roster(buffer, width, height, deep, ui, ctx, 0);
}

/// Render active missions content.
fn render_active_missions_content(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    content_top: i32,
    content_bottom: i32,
    deep: &DeepState,
    ui: &DeepUiState,
) {
    let now = Utc::now();
    let millis = current_millis();
    let active = &deep.prestige.active_missions;
    let completed = &deep.prestige.pending_results;
    let mut row = content_top;

    if active.is_empty() && completed.is_empty() {
        let mid = content_top + (content_bottom - content_top) / 2;
        put_text_centered(
            buffer,
            mid,
            width,
            "No active missions. Go to Deploy tab to start one.",
            Color::DarkGray,
        );

        // Warband log
        let log = &deep.prestige.warband_log;
        if !log.is_empty() && mid + 3 < content_bottom {
            render_section_rule(buffer, mid + 2, width, "WARBAND LOG", None);
            let mut lr = mid + 3;
            for entry in log.iter().rev().take(5) {
                if lr >= content_bottom {
                    break;
                }
                let (icon, color) = match entry.outcome {
                    crate::deep::MissionOutcome::Success => ("\u{2713}", Color::Green),
                    crate::deep::MissionOutcome::PartialSuccess => ("\u{25cb}", Color::Yellow),
                    crate::deep::MissionOutcome::Failure => ("\u{2717}", Color::LightRed),
                };
                let line = format!(
                    "{} Layer {} \u{2014} {} \u{2014} {} Warband Marks",
                    icon, entry.layer, entry.mission_name, entry.marks_earned
                );
                put_text(buffer, lr, 1, &line, color);
                lr += 1;
            }
        }
        return;
    }

    // ── Completed missions section ──
    if !completed.is_empty() {
        render_section_rule(buffer, row, width, "COMPLETED", Some(completed.len()));
        row += 1;

        for (completed_idx, mission) in completed.iter().enumerate() {
            if row + 1 >= content_bottom {
                break;
            }
            let tc = mission_type_color(mission.mission_type);
            let type_name = mission_type_label(mission.mission_type);
            let callsign = mission_callsign(mission);
            let tier_name = crate::deep::LayerTier::from_layer(mission.layer).display_name();
            let is_selected = ui.selected_index == completed_idx;
            let reward_marks = mission
                .result
                .as_ref()
                .map(|r| r.marks_earned)
                .unwrap_or_default();
            let card_left = 1i32;
            let card_right = (width as i32 - 2).max(card_left + 4);
            let card_top = row;
            let card_bottom = row + 1;
            tint_panel_background(
                buffer,
                card_left,
                card_top,
                card_right,
                card_bottom,
                if is_selected {
                    Color::Rgb(16, 34, 24)
                } else {
                    Color::Rgb(8, 18, 14)
                },
            );
            draw_panel_outline(
                buffer,
                card_left,
                card_top,
                card_right,
                card_bottom,
                if is_selected {
                    Color::Rgb(95, 175, 235)
                } else {
                    Color::Rgb(56, 98, 74)
                },
            );
            let line_col = card_left + 2;

            let glyph = "[\u{2713}] ";
            put_text(buffer, row, line_col, glyph, Color::Green);
            let info = format!(
                "{}  {} \u{00b7} L{} {}",
                callsign, type_name, mission.layer, tier_name
            );
            let info_w = (card_right - line_col - glyph.len() as i32 - 18).max(10) as usize;
            let info_display = truncate_text(&info, info_w);
            put_text(
                buffer,
                row,
                line_col + glyph.len() as i32,
                &info_display,
                tc,
            );
            let collect_hint = "COLLECT \u{2192} [Enter]";
            let hint_col = (card_right - collect_hint.len() as i32 - 1)
                .max(line_col + glyph.len() as i32 + info_display.len() as i32 + 2);
            put_text(buffer, row, hint_col, collect_hint, Color::Green);
            put_text(
                buffer,
                row + 1,
                line_col,
                &format!(
                    "Debrief ready \u{00b7} +{} Warband Marks confirmed \u{00b7} press [Enter]",
                    reward_marks
                ),
                Color::Rgb(78, 108, 92),
            );
            row += 2;
        }
    }

    // ── Active missions section ──
    if !active.is_empty() {
        render_section_rule(
            buffer,
            row,
            width,
            "ACTIVE \u{00b7} URGENCY ORDER",
            Some(active.len()),
        );
        row += 1;

        let mut display_order: Vec<(usize, bool, u64)> = active
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let has_event = m.has_pending_event();
                let remaining = (m.ends_at - now).num_seconds().max(0) as u64;
                (i, has_event, remaining)
            })
            .collect();
        display_order.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));

        for (display_idx, &(orig_idx, _has_event, _)) in display_order.iter().enumerate() {
            let card_rows = 4i32;
            if row + card_rows > content_bottom {
                break;
            }
            let mission = &active[orig_idx];
            let is_selected = ui.selected_index == completed.len() + display_idx;
            let tc = mission_type_color(mission.mission_type);
            let type_name = mission_type_label(mission.mission_type);
            let callsign = mission_callsign(mission);
            let progress = mission.progress(now);
            let total_secs = (mission.ends_at - mission.started_at).num_seconds().max(1) as u64;
            let elapsed_secs = (now - mission.started_at).num_seconds().max(0) as u64;
            let remaining_secs = total_secs.saturating_sub(elapsed_secs);
            let leader = lead_merc_name(deep, &mission.squad);
            let risk_tier = mission.mission_type.risk_tier();
            let squad_size = mission.squad.len();
            let (urgency_label, urgency_color) = mission_urgency_badge(mission, remaining_secs);
            let card_left = 1i32;
            let card_right = (width as i32 - 2).max(card_left + 8);

            let card_top = row;
            let card_bottom = row + card_rows - 1;
            let fill_bg = if is_selected {
                if mission.has_pending_event() {
                    Color::Rgb(30, 22, 18)
                } else {
                    Color::Rgb(16, 28, 46)
                }
            } else if mission.has_pending_event() {
                Color::Rgb(18, 14, 12)
            } else {
                Color::Rgb(7, 13, 24)
            };
            let border_color = if is_selected {
                Color::Rgb(95, 175, 235)
            } else if mission.has_pending_event() {
                Color::Rgb(130, 98, 56)
            } else {
                Color::Rgb(42, 68, 99)
            };
            draw_deep_card(
                buffer,
                card_left,
                card_top,
                card_right,
                card_bottom,
                border_color,
                fill_bg,
                Some(&callsign),
            );

            let urgency_badge = format!(" {} ", urgency_label);
            let badge_col = (card_right - urgency_badge.len() as i32 - 2).max(card_left + 4);
            put_text(buffer, card_top, badge_col, &urgency_badge, urgency_color);

            let line_col = card_left + 2;

            let content_row1 = card_top + 1;
            let (glyph, glyph_color) = if mission.has_pending_event() {
                ("\u{26a1} ", Color::Yellow)
            } else {
                ("\u{2694} ", Color::Cyan)
            };
            put_text(buffer, content_row1, line_col, glyph, glyph_color);
            let crew_str = if squad_size > 1 {
                format!("{} ({}) +{}", leader, type_name, squad_size - 1)
            } else {
                format!("{} ({})", leader, type_name)
            };
            let layer_risk = format!(
                "L{} {} {}",
                mission.layer,
                risk_icon(risk_tier),
                risk_label(risk_tier)
            );
            let avail_w = (card_right - line_col - 2) as usize;
            let crew_w = avail_w.saturating_sub(layer_risk.len() + 3);
            let crew_display = truncate_text(&crew_str, crew_w);
            put_text(
                buffer,
                content_row1,
                line_col + glyph.chars().count() as i32,
                &crew_display,
                Color::White,
            );
            let risk_col = (card_right - layer_risk.len() as i32 - 1)
                .max(line_col + glyph.chars().count() as i32 + crew_display.len() as i32 + 2);
            put_text(
                buffer,
                content_row1,
                risk_col,
                &layer_risk,
                risk_color(risk_tier),
            );

            let content_row2 = card_top + 2;
            let tier = LayerTier::from_layer(mission.layer);
            let triggers = event_trigger_points(mission.mission_type, tier);
            let event_hint = if mission.has_pending_event() {
                "\u{26a1} Event now!".to_string()
            } else if triggers.is_empty() {
                String::new()
            } else if let Some(&next_trigger) = triggers.iter().find(|&&t| t > progress) {
                let secs_to_event = ((next_trigger - progress) * total_secs as f64).round() as u64;
                format!("Evt ~{}", format_hours(secs_to_event))
            } else {
                String::new()
            };

            let time_str =
                if progress >= 1.0 && !matches!(mission.status, MissionStatus::EventPending) {
                    "Resolving...".to_string()
                } else if remaining_secs > 0 {
                    let pct = (progress * 100.0) as u32;
                    format!("{}% ~{}", pct, format_hours(remaining_secs))
                } else {
                    let pct = (progress * 100.0) as u32;
                    format!("{}% done", pct)
                };
            let suffix = if event_hint.is_empty() {
                format!("  {}", time_str)
            } else {
                format!("  {}  \u{00b7}  {}", time_str, event_hint)
            };
            let bar_width = (card_right - line_col - suffix.len() as i32 - 2).max(8) as usize;
            let bar_color = if progress > 0.95 {
                let pulse = (millis / 500).is_multiple_of(2);
                if pulse {
                    Color::Rgb(120, 220, 160)
                } else {
                    tc
                }
            } else {
                tc
            };
            render_progress_bar(
                buffer,
                content_row2,
                line_col,
                bar_width,
                progress,
                bar_color,
            );
            put_text(
                buffer,
                content_row2,
                line_col + bar_width as i32,
                &suffix,
                Color::DarkGray,
            );
            if mission.has_pending_event() && !event_hint.is_empty() {
                if let Some(pos) = suffix.find('\u{26a1}') {
                    put_text(
                        buffer,
                        content_row2,
                        line_col + bar_width as i32 + pos as i32,
                        &event_hint,
                        Color::Yellow,
                    );
                }
            }

            row += card_rows;
        }
    }
}

/// Compact (S-tier) single-panel new mission view.
#[allow(clippy::too_many_arguments)]
fn render_new_mission_compact(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    available: &[AvailableMission],
) {
    // Determine focus: false = mission list, true = squad picker
    let squad_panel = ui.staging_mission_index.is_some();

    if !squad_panel {
        // Phase 1: mission list with detail for selected
        let mut row = content_top;
        let mut last_layer: Option<u32> = None;
        for (i, m) in available.iter().enumerate() {
            if row >= content_bottom {
                break;
            }
            if last_layer != Some(m.layer) {
                let layer_line = mission_layer_header(m.layer);
                put_text(
                    buffer,
                    row,
                    1,
                    &layer_line,
                    mission_layer_header_color(m.layer),
                );
                row += 1;
                last_layer = Some(m.layer);
                if row >= content_bottom {
                    break;
                }
            }
            let is_sel = i == ui.selected_index;
            render_mission_row(buffer, row, 1, width.saturating_sub(2), m, is_sel);
            row += 1;
        }
        // Compact Phase 1 detail for selected mission
        if let Some(m) = available.get(ui.selected_index) {
            row += 1;
            if row < content_bottom && !m.description.is_empty() {
                let desc_trunc: String = m.description.chars().take((width - 4).max(10)).collect();
                put_text(buffer, row, 1, &desc_trunc, Color::DarkGray);
                row += 1;
            }
            if row < content_bottom {
                let hint = truncate_text(
                    mission_type_hint(m.mission_type),
                    (width.saturating_sub(4)).max(10),
                );
                put_text(buffer, row, 1, &hint, Color::Rgb(42, 68, 96));
                row += 1;
            }
            if row < content_bottom {
                let mut detail = format!("Pwr: {}  ", m.min_squad_power);
                if let Some(req) = m.required_archetype {
                    let in_roster = deep.prestige.roster.values().any(|r| r.archetype == req);
                    if in_roster {
                        detail.push_str(&format!("\u{2713} {} req  ", req.display_name()));
                    } else {
                        detail.push_str(&format!("\u{26a0} {} missing!  ", req.display_name()));
                    }
                }
                if m.marks_cost > 0 {
                    let marks = deep.prestige.warband_marks;
                    let afford = if marks >= m.marks_cost { "" } else { " LOW" };
                    detail.push_str(&format!(
                        "{} Warband Marks (have {}{})",
                        m.marks_cost, marks, afford
                    ));
                }
                put_text(buffer, row, 1, &detail, Color::DarkGray);
                // Color archetype warning
                if let Some(req) = m.required_archetype {
                    let in_roster = deep.prestige.roster.values().any(|r| r.archetype == req);
                    if !in_roster {
                        if let Some(pos) = detail.find('\u{26a0}') {
                            put_text(
                                buffer,
                                row,
                                1 + pos as i32,
                                &format!("\u{26a0} {} missing!", req.display_name()),
                                Color::Yellow,
                            );
                        }
                    }
                }
                row += 1;
            }
            if row < content_bottom {
                put_text(
                    buffer,
                    row,
                    1,
                    "[Enter] Assign squad",
                    Color::Rgb(50, 120, 60),
                );
            }
        }
    } else {
        // Phase 2: Squad picker for selected mission
        if let Some(mi) = ui.staging_mission_index {
            if let Some(m) = available.get(mi) {
                let tc = mission_type_color(m.mission_type);
                let cost_str = if m.marks_cost > 0 {
                    format!("  Cost: {} Warband Marks", m.marks_cost)
                } else {
                    "  Cost: Free".to_string()
                };
                put_text(
                    buffer,
                    content_top,
                    1,
                    &format!(
                        "[{}]  Layer {}{}",
                        mission_type_label(m.mission_type),
                        m.layer,
                        cost_str
                    ),
                    tc,
                );
                let mut row = content_top + 1;

                // Available mercs (selectable)
                let mut avail_idx = 0usize;
                for merc in deep.prestige.roster.values() {
                    if !merc.is_available() {
                        continue;
                    }
                    if row >= content_bottom - 2 {
                        break;
                    }
                    let is_sel = avail_idx == ui.selected_index;
                    let is_assigned = ui.staged_squad.contains(&merc.id);
                    let cursor = if is_sel { "\u{25b6} " } else { "  " };
                    let check = if is_assigned { "[\u{2713}] " } else { "[ ] " };
                    let line = format!("{}{}{} L{}", cursor, check, merc.name, merc.level);
                    put_text(buffer, row, 1, &line, Color::White);
                    put_text(
                        buffer,
                        row,
                        1,
                        cursor,
                        if is_sel { Color::Cyan } else { Color::DarkGray },
                    );
                    put_text(
                        buffer,
                        row,
                        3,
                        check,
                        if is_assigned {
                            Color::Green
                        } else {
                            Color::DarkGray
                        },
                    );
                    row += 1;
                    avail_idx += 1;
                }

                // Unavailable mercs (dimmed, not selectable)
                let has_unavailable = deep.prestige.roster.values().any(|m| !m.is_available());
                if has_unavailable && row < content_bottom - 2 {
                    let sep_str: String = "\u{2500} ".repeat((width / 2).max(4));
                    let sep_display = truncate_text(&sep_str, width.saturating_sub(2));
                    put_text(buffer, row, 1, &sep_display, Color::Rgb(40, 60, 80));
                    row += 1;
                    for merc in deep.prestige.roster.values() {
                        if merc.is_available() {
                            continue;
                        }
                        if row >= content_bottom - 2 {
                            break;
                        }
                        let avail_str = match &merc.status {
                            MercStatus::OnMission(_) => "(on mission)",
                            MercStatus::Injured { .. } => "(injured)",
                            MercStatus::Lost => "(lost)",
                            _ => "",
                        };
                        put_text(
                            buffer,
                            row,
                            3,
                            &format!("    {} {}", merc.name, avail_str),
                            Color::Rgb(50, 60, 70),
                        );
                        row += 1;
                    }
                }

                // Power summary + archetype check at bottom
                let squad_power: u32 = ui
                    .staged_squad
                    .iter()
                    .filter_map(|id| deep.prestige.find_merc(*id))
                    .map(|m| m.effective_power())
                    .sum();
                let min = m.min_squad_power;
                let is_safe = matches!(
                    m.mission_type,
                    MissionType::SupplyRun | MissionType::Construction(_)
                );
                let ratio = if min == 0 {
                    1.0
                } else {
                    squad_power as f64 / min as f64
                };
                let (forecast, fc) = if ui.staged_squad.is_empty() {
                    ("Select mercs", Color::DarkGray)
                } else if is_safe {
                    ("Always succeeds", Color::Green)
                } else if ratio >= 1.5 {
                    ("95% + faster", Color::Rgb(80, 220, 120))
                } else if ratio >= 1.0 {
                    ("60-90%", Color::Green)
                } else if ratio >= 0.75 {
                    ("Risky ~30%", Color::Yellow)
                } else {
                    ("Likely fail", Color::LightRed)
                };
                let power_color = if is_safe || squad_power >= min {
                    Color::Green
                } else {
                    Color::LightRed
                };
                let ratio_pct = if min == 0 {
                    999
                } else {
                    squad_power * 100 / min
                };

                // Archetype check line
                let mut arch_info = String::new();
                if let Some(req) = m.required_archetype {
                    let squad_archs: Vec<MercArchetype> = ui
                        .staged_squad
                        .iter()
                        .filter_map(|id| deep.prestige.find_merc(*id))
                        .map(|merc| merc.archetype)
                        .collect();
                    if squad_archs.contains(&req) {
                        arch_info = format!("  \u{2713}{}", req.display_name());
                    } else {
                        arch_info = format!("  (!) {} missing", req.display_name());
                    }
                }

                let summary_row = content_bottom - 2;
                put_text(
                    buffer,
                    summary_row,
                    1,
                    &format!(
                        "Pwr: {}/{} ({}%)  {}{}",
                        squad_power, min, ratio_pct, forecast, arch_info
                    ),
                    power_color,
                );
                let forecast_col =
                    format!("Pwr: {}/{} ({}%)  ", squad_power, min, ratio_pct).len() as i32 + 1;
                put_text(buffer, summary_row, forecast_col, forecast, fc);
                // Color the archetype info
                if !arch_info.is_empty() {
                    let arch_col = forecast_col + forecast.len() as i32;
                    let arch_color = if arch_info.contains("(!)") {
                        Color::Yellow
                    } else {
                        Color::Green
                    };
                    put_text(buffer, summary_row, arch_col, &arch_info, arch_color);
                }
            }
        }
    }
}

/// Full split-panel (M/L/XL) new mission view.
#[allow(clippy::too_many_arguments)]
fn render_new_mission_split(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    available: &[AvailableMission],
) {
    let preferred_left = (width * 52 / 100).max(24);
    let max_left = width.saturating_sub(28).max(18);
    let list_width = preferred_left.min(max_left);
    let detail_left = list_width as i32;
    let detail_width = width.saturating_sub(list_width);
    let staging = ui.staging_mission_index.is_some();

    // Panel chrome: backgrounds, outlines, divider, and headings.
    render_split_panel_chrome(
        buffer,
        width,
        detail_left,
        content_top,
        content_bottom,
        staging,
    );

    let list_inner_top = content_top + 1;

    if !staging {
        render_mission_list_left(
            buffer,
            available,
            ui.selected_index,
            list_width,
            list_inner_top,
            content_bottom,
        );
    } else {
        render_squad_assembly_left(buffer, deep, ui, list_width, list_inner_top, content_bottom);
    }

    // Right panel
    let detail_inner_left = detail_left + 1;
    let detail_inner_w = detail_width.saturating_sub(2) as i32;

    if detail_inner_w <= 0 {
        return;
    }

    let detail_idx = ui.staging_mission_index.unwrap_or(ui.selected_index);
    let Some(m) = available.get(detail_idx) else {
        return;
    };

    if !staging {
        render_mission_detail_phase1(
            buffer,
            deep,
            m,
            detail_inner_left,
            detail_inner_w,
            content_top,
            content_bottom,
            ui.mission_visit_count,
        );
    } else {
        // +2 to clear the panel outline border on the left
        render_squad_summary_panel(
            buffer,
            deep,
            ui,
            m,
            detail_inner_left + 1,
            detail_inner_w - 1,
            content_top + 1,
            content_bottom,
        );
    }
}

/// Render split-panel chrome: staging backgrounds/outlines, vertical divider, and headings.
fn render_split_panel_chrome(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    detail_left: i32,
    content_top: i32,
    content_bottom: i32,
    staging: bool,
) {
    // In squad-staging mode, visibly emphasize the active left panel.
    if staging {
        let panel_top = content_top;
        let panel_bottom = (content_bottom - 1).max(content_top);
        let left_left = 0;
        let left_right = detail_left - 1;
        let right_left = detail_left + 1;
        let right_right = width as i32 - 1;

        tint_panel_background(
            buffer,
            left_left + 1,
            panel_top + 1,
            left_right - 1,
            panel_bottom - 1,
            Color::Rgb(9, 18, 34),
        );
        tint_panel_background(
            buffer,
            right_left + 1,
            panel_top + 1,
            right_right - 1,
            panel_bottom - 1,
            Color::Rgb(5, 10, 20),
        );
        draw_panel_outline(
            buffer,
            left_left,
            panel_top,
            left_right,
            panel_bottom,
            Color::Rgb(95, 175, 235),
        );
        draw_panel_outline(
            buffer,
            right_left,
            panel_top,
            right_right,
            panel_bottom,
            Color::Rgb(40, 60, 90),
        );
    }

    // Draw inner divider
    let glyphs = super::panel_border_chars();
    for r in content_top..content_bottom {
        super::scene_fx::put_cell(buffer, r, detail_left, glyphs.v, Color::Rgb(40, 60, 80));
    }

    // Left panel heading
    let left_heading = if staging {
        "ASSIGN SQUAD"
    } else {
        "MISSIONS BY LAYER"
    };
    let left_heading_color = if staging {
        Color::Rgb(95, 175, 235)
    } else {
        SECTION_LABEL_COLOR
    };
    put_text(buffer, content_top, 1, left_heading, left_heading_color);
    // Right panel heading intentionally blank in staging mode;
    // mission info is rendered by render_squad_summary_panel.
}

/// Phase 1 left panel: available missions grouped by layer.
fn render_mission_list_left(
    buffer: &mut [Vec<SceneCell>],
    available: &[AvailableMission],
    selected_index: usize,
    list_width: usize,
    list_inner_top: i32,
    content_bottom: i32,
) {
    let mut row = list_inner_top;
    let mut last_layer: Option<u32> = None;
    for (i, m) in available.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        if last_layer != Some(m.layer) {
            let role_line =
                truncate_text(&mission_layer_header(m.layer), list_width.saturating_sub(2));
            put_text(
                buffer,
                row,
                1,
                &role_line,
                mission_layer_header_color(m.layer),
            );
            row += 1;
            last_layer = Some(m.layer);
            if row >= content_bottom {
                break;
            }
        }
        let is_sel = i == selected_index;
        render_mission_row(buffer, row, 1, list_width.saturating_sub(2), m, is_sel);
        row += 1;
    }
}

/// Phase 2 left panel: squad assembly with available/unavailable merc groups.
#[allow(clippy::too_many_arguments)]
fn render_squad_assembly_left(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    ui: &DeepUiState,
    list_width: usize,
    list_inner_top: i32,
    content_bottom: i32,
) {
    let mut row = list_inner_top;

    // Available mercs (selectable)
    let mut avail_idx = 0usize;
    for merc in deep.prestige.roster.values() {
        if !merc.is_available() {
            continue;
        }
        if row >= content_bottom - 1 {
            break;
        }
        let is_sel = avail_idx == ui.selected_index;
        let is_assigned = ui.staged_squad.contains(&merc.id);
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let check = if is_assigned { "[\u{2713}] " } else { "[ ] " };
        let arch_str = format!("  {} L{}", merc.archetype.display_name(), merc.level);
        let merc_color = Color::White;
        let arch_color = archetype_color(merc.archetype);
        let name_line = format!("{}{}{}", cursor, check, merc.name);
        put_text(buffer, row, 1, &name_line, merc_color);
        put_text(
            buffer,
            row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        put_text(
            buffer,
            row,
            3,
            check,
            if is_assigned {
                Color::Green
            } else {
                Color::DarkGray
            },
        );
        put_text(
            buffer,
            row,
            3 + check.len() as i32 + merc.name.len() as i32,
            &arch_str,
            arch_color,
        );
        row += 1;
        avail_idx += 1;
    }

    // Separator and unavailable mercs
    let has_unavailable = deep.prestige.roster.values().any(|m| !m.is_available());
    if has_unavailable && row < content_bottom - 1 {
        let sep_str: String = "\u{2500} ".repeat((list_width / 2).max(4));
        let sep_display = truncate_text(&sep_str, list_width);
        put_text(buffer, row, 1, &sep_display, Color::Rgb(40, 60, 80));
        row += 1;

        for merc in deep.prestige.roster.values() {
            if merc.is_available() {
                continue;
            }
            if row >= content_bottom {
                break;
            }
            let avail_str = match &merc.status {
                MercStatus::OnMission(_) => "(on mission)".to_string(),
                MercStatus::Injured { missions_remaining } => {
                    format!("(injured: {})", missions_remaining)
                }
                MercStatus::Lost => "(lost)".to_string(),
                _ => String::new(),
            };
            put_text(
                buffer,
                row,
                3,
                &format!(
                    "    {:14} {:8} {}",
                    truncate_text(&merc.name, 14),
                    merc.archetype.display_name(),
                    avail_str
                ),
                Color::Rgb(50, 60, 70),
            );
            row += 1;
        }
    }
}

/// One-line mission type description shown during first visits.
fn mission_type_hint(mt: MissionType) -> &'static str {
    match mt {
        MissionType::SupplyRun => "Baseline run: best Warband Marks per hour on cleared ground",
        MissionType::Recon => "Quick scout: raises Familiarity efficiently to cut future times",
        MissionType::Expedition => {
            "Deep push: best Familiarity per mission, with more risk and events"
        }
        MissionType::Breakthrough => "Progression run: clears the frontier to open next layer",
        MissionType::GatewayExpedition => "The final expedition \u{2014} breach the sealed gateway",
        MissionType::Construction(_) => {
            "Builds permanent infrastructure \u{2014} survives prestige"
        }
    }
}

/// Phase 1 right panel: mission detail with description, affordability, archetype warnings.
#[allow(clippy::too_many_arguments)]
fn render_mission_detail_phase1(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    mission: &AvailableMission,
    detail_inner_left: i32,
    detail_inner_w: i32,
    content_top: i32,
    content_bottom: i32,
    _mission_visit_count: u8,
) {
    let mut row = content_top;

    // Layer + tier heading
    let tier_name = crate::deep::LayerTier::from_layer(mission.layer).display_name();
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Layer {} \u{2014} {}", mission.layer, tier_name),
        Color::White,
    );
    row += 1;

    // Layer narrative (word-wrapped, quoted)
    let narrative = crate::deep::narratives::layer_narrative(mission.layer, &mission.mission_type);
    if !narrative.is_empty() && row + 1 < content_bottom {
        let max_w = (detail_inner_w - 2).max(10) as usize; // -2 for quote marks
        let quoted = format!("\u{201c}{}\u{201d}", narrative);
        let words: Vec<&str> = quoted.split_whitespace().collect();
        let mut line_buf = String::new();
        let mut lines_rendered = 0;
        for word in &words {
            if lines_rendered >= 3 {
                break;
            }
            if line_buf.len() + word.len() + 1 > max_w && !line_buf.is_empty() {
                let prefix = if lines_rendered == 0 { "" } else { " " };
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("{}{}", prefix, line_buf),
                    Color::Rgb(80, 100, 130),
                );
                row += 1;
                lines_rendered += 1;
                line_buf.clear();
            }
            if !line_buf.is_empty() {
                line_buf.push(' ');
            }
            line_buf.push_str(word);
        }
        if !line_buf.is_empty() && lines_rendered < 3 {
            let prefix = if lines_rendered == 0 { "" } else { " " };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("{}{}", prefix, line_buf),
                Color::Rgb(80, 100, 130),
            );
            row += 1;
        }
        row += 1; // blank line after narrative
    }

    // ── Operations Section ──
    if row < content_bottom {
        put_text(buffer, row, detail_inner_left, "Operations", Color::Cyan);
        row += 1;
    }

    // Duration
    if row < content_bottom {
        put_text(
            buffer,
            row,
            detail_inner_left,
            &format!(
                "  \u{23f1} Duration   {}",
                format_hours(mission.duration_secs)
            ),
            Color::DarkGray,
        );
        row += 1;
    }

    // Risk
    if row < content_bottom {
        let risk_tier = mission.mission_type.risk_tier();
        put_text(
            buffer,
            row,
            detail_inner_left,
            &format!("  Risk         {}", risk_label(risk_tier)),
            risk_color(risk_tier),
        );
        row += 1;
    }

    // Cost
    if row < content_bottom {
        let marks = deep.prestige.warband_marks;
        if mission.marks_cost > 0 {
            let cost_str = format!("  Cost         {} Marks", mission.marks_cost);
            put_text(buffer, row, detail_inner_left, &cost_str, MARKS_COLOR);
            let (afford_str, afford_color) = if marks >= mission.marks_cost {
                (format!(" (have {})", marks), Color::Rgb(60, 180, 80))
            } else {
                (
                    format!(" (need {})", mission.marks_cost - marks),
                    Color::LightRed,
                )
            };
            put_text(
                buffer,
                row,
                detail_inner_left + cost_str.len() as i32,
                &afford_str,
                afford_color,
            );
        } else {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "  Cost         Free",
                Color::Rgb(60, 180, 80),
            );
        }
        row += 1;
    }
    row += 1; // blank line after Operations

    // ── Requires Section ──
    if row < content_bottom {
        put_text(buffer, row, detail_inner_left, "Requires", Color::Cyan);
        row += 1;
    }

    if row < content_bottom {
        put_text(
            buffer,
            row,
            detail_inner_left,
            &format!("  Min Power  {}", mission.min_squad_power),
            Color::White,
        );
        row += 1;
    }

    // Required archetype
    if let Some(req_arch) = mission.required_archetype {
        if row < content_bottom {
            let in_roster = deep
                .prestige
                .roster
                .values()
                .any(|m| m.archetype == req_arch);
            let (prefix, label_color) = if in_roster {
                ("\u{2713} ", Color::Green)
            } else {
                ("\u{26a0} ", Color::Yellow)
            };
            let suffix = if !in_roster {
                " (not in roster!)"
            } else {
                " (required)"
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  {}{}{}", prefix, req_arch.display_name(), suffix),
                label_color,
            );
            row += 1;
        }
    }

    // Recommended archetype
    if let Some(rec_arch) = mission.recommended_archetype {
        if row < content_bottom {
            let in_roster = deep
                .prestige
                .roster
                .values()
                .any(|m| m.archetype == rec_arch);
            let (prefix, color) = if in_roster {
                ("\u{2605} ", Color::Cyan)
            } else {
                ("  ", Color::DarkGray)
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  {}{} (recommended)", prefix, rec_arch.display_name()),
                color,
            );
            row += 1;
        }
    }
    row += 1; // blank line after Requires

    // ── Rewards Section ──
    if row < content_bottom {
        if matches!(mission.mission_type, MissionType::Construction(_)) {
            put_text(buffer, row, detail_inner_left, "Rewards", Color::Cyan);
            row += 1;
            if row < content_bottom {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    "  \u{25c6} Infrastructure (permanent)",
                    Color::DarkGray,
                );
                row += 1;
            }
        } else {
            put_text(buffer, row, detail_inner_left, "Rewards", Color::Cyan);
            row += 1;

            if row < content_bottom {
                let base = base_marks_earned(mission.mission_type, mission.layer);
                let min_marks = (base as f64 * 0.85).round() as u32;
                let max_marks = (base as f64 * 1.15).round() as u32;
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!(
                        "  \u{25c6} {}\u{2013}{} Warband Marks",
                        min_marks, max_marks
                    ),
                    MARKS_COLOR,
                );
                row += 1;
            }

            if row < content_bottom {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    "  \u{2605} Merc progression: +1",
                    Color::DarkGray,
                );
                row += 1;
            }

            if row < content_bottom
                && matches!(
                    mission.mission_type,
                    MissionType::Recon | MissionType::Expedition
                )
            {
                let fam = familiarity_gain(mission.mission_type);
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("  \u{25c8} Familiarity: +{}", fam),
                    Color::Rgb(80, 150, 200),
                );
                row += 1;
            }
        }
    }

    // ── Tip + Action at bottom ──
    // Reserve 3 rows from the bottom: tip (1-2 lines) + action hint
    let tip_text = format!(
        "Tip: {} \u{2014} {}",
        mission_type_label(mission.mission_type),
        mission_type_hint(mission.mission_type)
    );
    let action_row = content_bottom - 1;
    let max_tip_w = (detail_inner_w - 1).max(10) as usize;

    // Word-wrap the tip into up to 2 lines
    let tip_words: Vec<&str> = tip_text.split_whitespace().collect();
    let mut tip_lines: Vec<String> = Vec::new();
    let mut tip_buf = String::new();
    for word in &tip_words {
        if tip_buf.len() + word.len() + 1 > max_tip_w && !tip_buf.is_empty() {
            tip_lines.push(tip_buf.clone());
            tip_buf.clear();
            if tip_lines.len() >= 2 {
                break;
            }
        }
        if !tip_buf.is_empty() {
            tip_buf.push(' ');
        }
        tip_buf.push_str(word);
    }
    if !tip_buf.is_empty() && tip_lines.len() < 2 {
        tip_lines.push(tip_buf);
    }

    let tip_start = action_row - tip_lines.len() as i32 - 1;
    let tip_start = tip_start.max(row + 1);
    for (i, line) in tip_lines.iter().enumerate() {
        let r = tip_start + i as i32;
        if r < action_row {
            let indent = if i > 0 { "  " } else { "" };
            put_text(
                buffer,
                r,
                detail_inner_left,
                &format!("{}{}", indent, line),
                Color::DarkGray,
            );
        }
    }

    if action_row > tip_start {
        put_text(
            buffer,
            action_row,
            detail_inner_left,
            "[Enter] Assign squad \u{2192}",
            Color::Rgb(50, 120, 60),
        );
    }
}

/// Phase 2 right panel: squad summary with power meter, archetype checks, smart hints.
#[allow(clippy::too_many_arguments)]
fn render_squad_summary_panel(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    ui: &DeepUiState,
    mission: &AvailableMission,
    detail_inner_left: i32,
    detail_inner_w: i32,
    content_top: i32,
    content_bottom: i32,
) {
    let squad_power: u32 = ui
        .staged_squad
        .iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.effective_power())
        .sum();
    let min = mission.min_squad_power;
    let marks = deep.prestige.warband_marks;
    let is_safe = matches!(
        mission.mission_type,
        MissionType::SupplyRun | MissionType::Construction(_)
    );
    let can_afford = marks >= mission.marks_cost;

    let mut row = content_top;

    // ── Header: Layer + Tier ──
    let tier_name = LayerTier::from_layer(mission.layer).display_name();
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Layer {} \u{2014} {}", mission.layer, tier_name),
        Color::White,
    );
    row += 1;

    // Narrative text (word-wrapped, quoted)
    let narrative = crate::deep::narratives::layer_narrative(mission.layer, &mission.mission_type);
    if !narrative.is_empty() && row + 1 < content_bottom {
        let max_w = (detail_inner_w - 2).max(10) as usize;
        let quoted = format!("\u{201c}{}\u{201d}", narrative);
        let words: Vec<&str> = quoted.split_whitespace().collect();
        let mut line_buf = String::new();
        let mut lines_rendered = 0;
        for word in &words {
            if lines_rendered >= 3 {
                break;
            }
            if line_buf.len() + word.len() + 1 > max_w && !line_buf.is_empty() {
                let prefix = if lines_rendered == 0 { "" } else { " " };
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("{}{}", prefix, line_buf),
                    Color::Rgb(80, 100, 130),
                );
                row += 1;
                lines_rendered += 1;
                line_buf.clear();
            }
            if !line_buf.is_empty() {
                line_buf.push(' ');
            }
            line_buf.push_str(word);
        }
        if !line_buf.is_empty() && lines_rendered < 3 {
            let prefix = if lines_rendered == 0 { "" } else { " " };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("{}{}", prefix, line_buf),
                Color::Rgb(80, 100, 130),
            );
            row += 1;
        }
    }

    // Compact operations line: ⏱ duration  Risk: X  Cost: X
    // Render each segment separately to avoid wide-char column misalignment.
    if row < content_bottom {
        let risk_tier = mission.mission_type.risk_tier();
        let mut col = detail_inner_left;

        let dur_seg = format!("\u{23f1} {}  ", format_hours(mission.duration_secs));
        put_text(buffer, row, col, &dur_seg, Color::DarkGray);
        col += super::scene_fx::display_width(&dur_seg) as i32;

        let risk_seg = format!("Risk: {}  ", risk_label(risk_tier));
        put_text(buffer, row, col, &risk_seg, risk_color(risk_tier));
        col += super::scene_fx::display_width(&risk_seg) as i32;

        if mission.marks_cost > 0 {
            let cost_color = if can_afford {
                MARKS_COLOR
            } else {
                Color::LightRed
            };
            put_text(
                buffer,
                row,
                col,
                &format!("Cost: {}", mission.marks_cost),
                cost_color,
            );
        } else {
            put_text(buffer, row, col, "Cost: Free", Color::Rgb(60, 180, 80));
        }
        row += 1;
    }
    row += 1; // blank line

    // ── Squad Power Section ──

    let ratio = if min == 0 {
        1.0
    } else {
        squad_power as f64 / min as f64
    };
    let ratio_pct = if min == 0 {
        999u32
    } else {
        squad_power * 100 / min
    };

    let (bar_color, forecast_label, forecast_color) = if ui.staged_squad.is_empty() {
        (
            Color::DarkGray,
            "Select mercs with [Space]",
            Color::DarkGray,
        )
    } else if is_safe {
        (Color::Green, "Always succeeds", Color::Green)
    } else if ratio >= 1.5 {
        (
            Color::Rgb(80, 220, 120),
            "Overpowered \u{2014} 95% + faster!",
            Color::Rgb(80, 220, 120),
        )
    } else if ratio >= 1.0 {
        (Color::Green, "Good \u{2014} 60-90% success", Color::Green)
    } else if ratio >= 0.75 {
        (Color::Yellow, "Risky \u{2014} ~30% success", Color::Yellow)
    } else {
        (Color::LightRed, "Likely to fail", Color::LightRed)
    };

    let power_str = if is_safe || min == 0 {
        format!("Squad Power  {}", squad_power)
    } else {
        format!("Squad Power  {} / {}  ({}%)", squad_power, min, ratio_pct)
    };
    put_text(buffer, row, detail_inner_left, &power_str, Color::White);
    // Recolor percentage based on success band
    if !is_safe && min > 0 {
        let pct_str = format!("({}%)", ratio_pct);
        if let Some(pos) = power_str.find('(') {
            let pct_color = if ratio_pct >= 150 {
                Color::Rgb(80, 220, 120)
            } else if ratio_pct >= 100 {
                Color::Green
            } else if ratio_pct >= 75 {
                Color::Yellow
            } else {
                Color::LightRed
            };
            put_text(
                buffer,
                row,
                detail_inner_left + pos as i32,
                &pct_str,
                pct_color,
            );
        }
    }
    row += 1;

    let bar_w = (detail_inner_w as usize).saturating_sub(2).min(24);
    render_progress_bar(
        buffer,
        row,
        detail_inner_left,
        bar_w,
        ratio.min(1.0),
        bar_color,
    );
    row += 1;

    put_text(
        buffer,
        row,
        detail_inner_left,
        forecast_label,
        forecast_color,
    );
    row += 1;

    // Required archetype check
    if let Some(req_arch) = mission.required_archetype {
        let squad_archetypes: Vec<crate::deep::MercArchetype> = ui
            .staged_squad
            .iter()
            .filter_map(|id| deep.prestige.find_merc(*id))
            .map(|m| m.archetype)
            .collect();
        let req_present = squad_archetypes.contains(&req_arch);
        if row < content_bottom - 3 {
            let (prefix, color, suffix) = if req_present {
                ("\u{2713} ", Color::Green, " (required)")
            } else {
                ("(!) ", Color::Yellow, " (required \u{2014} missing!)")
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  {}{}{}", prefix, req_arch.display_name(), suffix),
                color,
            );
            row += 1;
        }
    }

    // Recommended archetype check
    if let Some(rec_arch) = mission.recommended_archetype {
        let squad_archetypes: Vec<crate::deep::MercArchetype> = ui
            .staged_squad
            .iter()
            .filter_map(|id| deep.prestige.find_merc(*id))
            .map(|m| m.archetype)
            .collect();
        let rec_present = squad_archetypes.contains(&rec_arch);
        if row < content_bottom - 3 {
            let (prefix, color, suffix) = if rec_present {
                ("\u{2605} ", Color::Cyan, " (recommended)")
            } else {
                ("  ", Color::DarkGray, " (recommended)")
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  {}{}{}", prefix, rec_arch.display_name(), suffix),
                color,
            );
            row += 1;
        }
    }
    row += 1; // blank line after squad section

    // ── Rewards Section ──
    if row < content_bottom - 4 {
        if matches!(mission.mission_type, MissionType::Construction(_)) {
            put_text(buffer, row, detail_inner_left, "Rewards", Color::Cyan);
            row += 1;
            if row < content_bottom - 3 {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    "  \u{25c6} Infrastructure (permanent)",
                    Color::DarkGray,
                );
                row += 1;
            }
        } else {
            put_text(buffer, row, detail_inner_left, "Rewards", Color::Cyan);
            row += 1;

            if row < content_bottom - 3 {
                let base = base_marks_earned(mission.mission_type, mission.layer);
                let min_marks = (base as f64 * 0.85).round() as u32;
                let max_marks = (base as f64 * 1.15).round() as u32;
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!(
                        "  \u{25c6} {}\u{2013}{} Warband Marks",
                        min_marks, max_marks
                    ),
                    MARKS_COLOR,
                );
                row += 1;
            }

            if row < content_bottom - 3 {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    "  \u{2605} Merc progression: +1",
                    Color::DarkGray,
                );
                row += 1;
            }

            if row < content_bottom - 3
                && matches!(
                    mission.mission_type,
                    MissionType::Recon | MissionType::Expedition
                )
            {
                let fam = familiarity_gain(mission.mission_type);
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("  \u{25c8} Familiarity: +{}", fam),
                    Color::Rgb(80, 150, 200),
                );
                row += 1;
            }
        }
    }

    // ── Tip + Launch at bottom ──
    let tip_text = format!(
        "Tip: {} \u{2014} {}",
        mission_type_label(mission.mission_type),
        mission_type_hint(mission.mission_type)
    );
    let launch_row = content_bottom - 1;
    let max_tip_w = (detail_inner_w - 1).max(10) as usize;

    // Word-wrap the tip into up to 2 lines
    let tip_words: Vec<&str> = tip_text.split_whitespace().collect();
    let mut tip_lines: Vec<String> = Vec::new();
    let mut tip_buf = String::new();
    for word in &tip_words {
        if tip_buf.len() + word.len() + 1 > max_tip_w && !tip_buf.is_empty() {
            tip_lines.push(tip_buf.clone());
            tip_buf.clear();
            if tip_lines.len() >= 2 {
                break;
            }
        }
        if !tip_buf.is_empty() {
            tip_buf.push(' ');
        }
        tip_buf.push_str(word);
    }
    if !tip_buf.is_empty() && tip_lines.len() < 2 {
        tip_lines.push(tip_buf);
    }

    let tip_start = launch_row - tip_lines.len() as i32 - 1;
    let tip_start = tip_start.max(row + 1);
    for (i, line) in tip_lines.iter().enumerate() {
        let r = tip_start + i as i32;
        if r < launch_row {
            let indent = if i > 0 { "  " } else { "" };
            put_text(
                buffer,
                r,
                detail_inner_left,
                &format!("{}{}", indent, line),
                Color::DarkGray,
            );
        }
    }

    // Launch action
    if launch_row > tip_start {
        let launch_color = if ui.staged_squad.is_empty() {
            Color::DarkGray
        } else {
            Color::Rgb(60, 180, 80)
        };
        put_text(
            buffer,
            launch_row,
            detail_inner_left,
            "[Enter] Launch Mission",
            launch_color,
        );
    }
}

// ── Tier-Specific Atmosphere Messages ────────────────────────────────────────

/// Narrative atmosphere messages keyed to the frontier layer tier.
/// These rotate in the hub every 8 seconds.
fn tier_atmosphere_messages(frontier_layer: u32) -> &'static [&'static str] {
    match LayerTier::from_layer(frontier_layer) {
        LayerTier::Shallows => &[
            "The walls here were carved with purpose. This was no mine.",
            "Your scouts find a collapsed barracks. Decades of dust.",
            "The captain traces a finger along a carved warning.",
            "Tool marks on the walls change from picks to ritual implements.",
            "The tunnels breathe. Your company awaits orders.",
        ],
        LayerTier::Warrens => &[
            "Gareth found a child's doll in the rubble. Stone, but carefully carved.",
            "The archive tablets mention 'the Wellspring' seventeen times.",
            "The Overseer's body twitches even in death. Its purpose outlasted its makers.",
            "Living quarters line these corridors. Families lived here.",
            "Distant rumbles echo from below. The Deep stirs.",
        ],
        LayerTier::Hollows => &[
            "The walls pulse with a slow rhythm. It matches your heartbeat.",
            "Your Arcanist says the light here isn't bioluminescence. It's memory.",
            "An Echo walks past the camp. It doesn't see you. It never will.",
            "The spore clouds aren't toxic by nature. They're a defense mechanism.",
            "The stone remembers being shaped. It remembers the hands that shaped it.",
        ],
        LayerTier::SunkenReach => &[
            "The seals glow brighter when your Arcanist approaches.",
            "Water pressure should have crushed these chambers millennia ago.",
            "The Drowned King's throne faces downward. Even in death, it watched below.",
            "These chambers were flooded deliberately. Water as a barrier.",
            "The rune patterns on the seals match the god items.",
        ],
        LayerTier::Abyss => &[
            "Mira returned with six days of rations consumed. She was gone four hours.",
            "The Vanguard's battle-axe is two inches shorter. The edge is sharper.",
            "Sound travels wrong here \u{2014} you hear your orders before you give them.",
            "Your Medic's wound records don't match. She was injured on missions she hasn't run.",
            "The Wellspring doesn't call to you. It recognizes you.",
        ],
        LayerTier::Void => &[
            "There is no stone here. Your mercs walk on solidified will.",
            "The Wellspring pulses. It has been waiting longer than your world has existed.",
            "Your Vanguard's wounds close before the Medic reaches them.",
            "The void is not empty. It is aware.",
            "Each step closer. The Gateway waits at the end of everything.",
        ],
    }
}
