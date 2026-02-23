//! The Deep — Hub and New Mission sub-view rendering.

use crate::deep::{
    AvailableMission, DeepState, DeepUiState, MercArchetype, MercStatus, MissionStatus,
    MissionType,
};
use chrono::Utc;
use ratatui::style::Color;

use super::deep_scene::DEEP_BORDER_COLOR;
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{put_text, put_text_centered, SceneCell};

// ── Color helpers ─────────────────────────────────────────────────────────────

pub(super) fn mission_type_color(t: MissionType) -> Color {
    match t {
        MissionType::SupplyRun => Color::Green,
        MissionType::Recon => Color::Cyan,
        MissionType::Expedition => Color::Yellow,
        MissionType::Breakthrough => Color::LightRed,
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

fn risk_color(tier: u8) -> Color {
    match tier {
        0 => Color::Green,
        1 => Color::Cyan,
        2 => Color::Yellow,
        3 => Color::LightRed,
        _ => Color::DarkGray,
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
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);
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

// ── Hub view ──────────────────────────────────────────────────────────────────

/// Render the main Hub view.
pub(super) fn render_hub(
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

    let now = Utc::now();
    let rank = deep.persistent.guild_rank;
    let marks = deep.prestige.warband_marks;
    let roster_count = deep.prestige.roster.len();
    let max_roster = rank.max_roster() as usize;
    let frontier = deep.persistent.frontier_layer();

    // ── Header (rows 0–1) ──
    let header = format!(
        "Guild: {} (Rank {})    Marks: {}",
        rank.display_name(),
        rank.0,
        marks
    );
    put_text(buffer, 0, 1, &header, Color::White);

    let sub_header = format!(
        "Frontier: Layer {} ({})    Mercs: {}/{}",
        frontier,
        crate::deep::LayerTier::from_layer(frontier).display_name(),
        roster_count,
        max_roster,
    );
    put_text(buffer, 1, 1, &sub_header, Color::DarkGray);

    // ── Separator ──
    let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
    put_text(buffer, 2, 1, &sep, Color::Rgb(40, 60, 80));

    // ── Active missions list (rows 3 .. height-2) ──
    let missions_top = 3i32;
    let missions_bottom = height as i32 - 2;
    let content_height = (missions_bottom - missions_top).max(0) as usize;

    let active = &deep.prestige.active_missions;
    let completed = &deep.prestige.pending_results;

    if active.is_empty() && completed.is_empty() {
        let msg = "No active missions.";
        put_text_centered(buffer, missions_top + content_height as i32 / 2, width, msg, Color::DarkGray);
        let hint = "[N] New Mission to deploy a squad.";
        put_text_centered(buffer, missions_top + content_height as i32 / 2 + 1, width, hint, Color::Rgb(50, 70, 100));
    } else {
        // Section label
        put_text(buffer, missions_top, 1, "ACTIVE MISSIONS", Color::Rgb(80, 160, 220));

        let mut row = missions_top + 1;
        let is_compact = ctx.tier <= SizeTier::S;
        let rows_per_mission = if is_compact { 2 } else { 3 };

        // Show completed missions first (they need action)
        for mission in completed.iter() {
            if row + 1 >= missions_bottom {
                break;
            }
            let is_selected = false; // completed missions shown without cursor
            let prefix = if is_selected { "\u{25b6} " } else { "  " };
            let type_name = mission.mission_type.display_name();
            let tc = mission_type_color(mission.mission_type);
            let line = format!("{}[{}]  Layer {}  Done!", prefix, type_name, mission.layer);
            put_text(buffer, row, 1, &line, tc);
            put_text(buffer, row, 1, prefix, Color::Cyan);
            let collect_hint = "  \u{2713} Complete \u{2014} [Enter] to collect rewards.";
            put_text(buffer, row + 1, 3, collect_hint, Color::Green);
            row += rows_per_mission;
        }

        // Show active missions
        for (i, mission) in active.iter().enumerate() {
            if row + 1 >= missions_bottom {
                break;
            }
            let is_selected = i == ui.selected_index;
            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            let tc = mission_type_color(mission.mission_type);
            let type_name = mission.mission_type.display_name();
            let progress = mission.progress(now);

            // Build squad name string
            let squad_names: Vec<String> = mission
                .squad
                .iter()
                .filter_map(|id| deep.prestige.find_merc(*id))
                .map(|m| m.name.clone())
                .collect();
            let squad_str = if squad_names.is_empty() {
                "No squad".to_string()
            } else {
                squad_names.join(", ")
            };

            // Elapsed / total
            let total_secs = (mission.ends_at - mission.started_at).num_seconds().max(1) as u64;
            let elapsed_secs = (now - mission.started_at).num_seconds().max(0) as u64;

            let status_suffix = match &mission.status {
                MissionStatus::EventPending => {
                    format!("  \u{26a1} Event pending!")
                }
                MissionStatus::Completed => "  Done!".to_string(),
                _ => format!("  {}%", (progress * 100.0) as u32),
            };

            let line1 = format!(
                "{}[{}]  Layer {}  {}/{}{}",
                cursor,
                type_name,
                mission.layer,
                format_hours(elapsed_secs),
                format_hours(total_secs),
                status_suffix,
            );
            put_text(buffer, row, 1, &line1, tc);
            // Recolor cursor
            put_text(buffer, row, 1, cursor, if is_selected { Color::Cyan } else { Color::DarkGray });
            // Event pending indicator color
            if matches!(mission.status, MissionStatus::EventPending) {
                // Overwrite status suffix color
                let offset = line1.find('\u{26a1}').unwrap_or(0) as i32;
                put_text(buffer, row, 1 + offset, "\u{26a1} Event pending!", Color::Yellow);
            }

            if !is_compact {
                // Progress bar row
                let bar_width = width.saturating_sub(6).min(36);
                render_progress_bar(buffer, row + 1, 3, bar_width, progress, tc);
                let pct = format!(" {}%", (progress * 100.0) as u32);
                put_text(buffer, row + 1, 3 + bar_width as i32, &pct, Color::DarkGray);

                // Squad row
                let squad_label = format!("  Squad: {}", squad_str);
                put_text(buffer, row + 2, 1, &squad_label, Color::DarkGray);
                row += 3;
            } else {
                // Compact: squad on same row appended, progress as short bar
                let bar_width = 12usize;
                render_progress_bar(buffer, row + 1, 3, bar_width, progress, tc);
                let squad_label = format!("  {}", squad_str);
                put_text(buffer, row + 1, 3 + bar_width as i32 + 1, &squad_label, Color::DarkGray);
                row += 2;
            }
        }
    }

    // ── Footer ──
    let footer = match ctx.tier {
        SizeTier::S => "[N]New  [R]Roster  [L]Layers  [Esc]Close",
        _ => "[N] New Mission  [R] Roster  [L] Layers  [Esc] Close",
    };
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);
}

// ── New Mission view ──────────────────────────────────────────────────────────

/// Render the New Mission sub-view.
pub(super) fn render_new_mission(
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

    // ── Title row ──
    put_text(buffer, 0, 1, "NEW MISSION", DEEP_BORDER_COLOR);

    // ── Footer ──
    let footer = if is_compact {
        "[\u{2191}/\u{2193}] Select  [Tab] Panel  [Space] Toggle  [Enter] Launch  [Esc] Back"
    } else {
        "[\u{2191}/\u{2193}] Select Mission  [Tab] Switch Panel  [Space] Toggle Merc  [Enter] Launch  [Esc] Back"
    };
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);

    let content_top = 1i32;
    let content_bottom = height as i32 - 1;
    let content_height = (content_bottom - content_top).max(0) as usize;

    let available = &deep.prestige.available_missions;

    if available.is_empty() {
        put_text_centered(
            buffer,
            content_top + content_height as i32 / 2,
            width,
            "No missions available.",
            Color::DarkGray,
        );
        put_text_centered(
            buffer,
            content_top + content_height as i32 / 2 + 1,
            width,
            "Complete active missions to refresh the pool.",
            Color::Rgb(50, 70, 100),
        );
        return;
    }

    if is_compact {
        render_new_mission_compact(buffer, width, height, deep, ui, content_top, content_bottom, available);
    } else {
        render_new_mission_split(buffer, width, height, deep, ui, content_top, content_bottom, available);
    }
}

/// Compact (S-tier) single-panel new mission view.
fn render_new_mission_compact(
    buffer: &mut [Vec<SceneCell>],
    _width: usize,
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
        // Mission list
        let mut row = content_top;
        for (i, m) in available.iter().enumerate() {
            if row >= content_bottom {
                break;
            }
            let is_sel = i == ui.selected_index;
            let cursor = if is_sel { "\u{25b6} " } else { "  " };
            let tc = mission_type_color(m.mission_type);
            let line = format!(
                "{}[{}]  L{}  {}  {}",
                cursor,
                m.mission_type.display_name(),
                m.layer,
                format_hours(m.duration_secs),
                risk_label(m.mission_type.risk_tier()),
            );
            put_text(buffer, row, 1, &line, tc);
            put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
            row += 1;
        }
    } else {
        // Squad picker for selected mission
        if let Some(mi) = ui.staging_mission_index {
            if let Some(m) = available.get(mi) {
                put_text(buffer, content_top, 1, &format!("[{}]  Layer {}", m.mission_type.display_name(), m.layer), mission_type_color(m.mission_type));
                let mut row = content_top + 1;
                for (ri, merc) in deep.prestige.roster.iter().enumerate() {
                    if row >= content_bottom {
                        break;
                    }
                    let is_sel = ri == ui.selected_index;
                    let is_assigned = ui.staged_squad.contains(&merc.id);
                    let cursor = if is_sel { "\u{25b6} " } else { "  " };
                    let check = if is_assigned { "[\u{2713}] " } else { "[ ] " };
                    let avail_marker = match &merc.status {
                        MercStatus::Available => "",
                        MercStatus::OnMission(_) => " (on mission)",
                        MercStatus::Injured { .. } => " (injured)",
                        MercStatus::Lost => " (lost)",
                    };
                    let line = format!("{}{}{} L{}{}",
                        cursor, check, merc.name, merc.level, avail_marker);
                    let color = if merc.is_available() { Color::White } else { Color::DarkGray };
                    put_text(buffer, row, 1, &line, color);
                    put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
                    put_text(buffer, row, 3, check, if is_assigned { Color::Green } else { Color::DarkGray });
                    row += 1;
                }
                // Power summary
                let squad_power: u32 = ui.staged_squad.iter()
                    .filter_map(|id| deep.prestige.find_merc(*id))
                    .map(|m| m.effective_power())
                    .sum();
                let req_met = squad_power >= m.min_squad_power;
                let power_color = if req_met { Color::Green } else { Color::LightRed };
                put_text(buffer, content_bottom - 1, 1, &format!("Power: {}  Min: {}", squad_power, m.min_squad_power), power_color);
            }
        }
    }
}

/// Full split-panel (M/L/XL) new mission view.
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
    let list_width = (width * 40 / 100).max(18).min(width.saturating_sub(20));
    let detail_left = list_width as i32;
    let detail_width = width.saturating_sub(list_width);

    // Draw inner divider
    let glyphs = super::panel_border_chars();
    for r in content_top..content_bottom {
        super::scene_fx::put_cell(buffer, r, detail_left, glyphs.v, Color::Rgb(40, 60, 80));
    }

    // Left: mission list
    put_text(buffer, content_top, 1, "AVAILABLE", Color::Rgb(80, 160, 220));
    let list_inner_top = content_top + 1;

    for (i, m) in available.iter().enumerate() {
        let row = list_inner_top + i as i32;
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index && ui.staging_mission_index.is_none();
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let tc = mission_type_color(m.mission_type);
        let max_name_w = (list_width as i32 - 4).max(8) as usize;
        let type_name = m.mission_type.display_name();
        let line = format!(
            "{}[{:width$}]  L{}  {}",
            cursor,
            &type_name[..type_name.len().min(max_name_w)],
            m.layer,
            format_hours(m.duration_secs),
            width = 0,
        );
        put_text(buffer, row, 1, &line, tc);
        put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
    }

    // Right: mission detail + squad picker
    let detail_inner_left = detail_left + 1;
    let detail_inner_w = detail_width.saturating_sub(2) as i32;

    if detail_inner_w <= 0 {
        return;
    }

    // Find which mission to show detail for
    let detail_idx = ui.staging_mission_index.unwrap_or(ui.selected_index);
    let Some(m) = available.get(detail_idx) else {
        return;
    };

    let mut row = content_top;

    // Mission name + layer tier
    let tier_name = crate::deep::LayerTier::from_layer(m.layer).display_name();
    put_text(buffer, row, detail_inner_left, &format!("Layer {} \u{2014} {}", m.layer, tier_name), Color::White);
    row += 1;

    // Duration + risk
    put_text(buffer, row, detail_inner_left, &format!("Duration: {}", format_hours(m.duration_secs)), Color::DarkGray);
    row += 1;
    let risk_str = format!("Risk:     {}", risk_label(m.mission_type.risk_tier()));
    put_text(buffer, row, detail_inner_left, &risk_str, risk_color(m.mission_type.risk_tier()));
    row += 1;
    put_text(buffer, row, detail_inner_left, "Reward:   Marks + items", Color::DarkGray);
    row += 1;

    // Requirements
    row += 1;
    put_text(buffer, row, detail_inner_left, "Requires:", Color::Cyan);
    row += 1;
    put_text(buffer, row, detail_inner_left, &format!("  Min Power {}", m.min_squad_power), Color::White);
    row += 1;
    if let Some(req_arch) = m.required_archetype {
        put_text(buffer, row, detail_inner_left, &format!("  {} required", req_arch.display_name()), archetype_color(req_arch));
        row += 1;
    }
    if let Some(rec_arch) = m.recommended_archetype {
        put_text(buffer, row, detail_inner_left, &format!("  {} recommended", rec_arch.display_name()), Color::DarkGray);
        row += 1;
    }

    // Squad section divider
    row += 1;
    let sep: String = "\u{2500}".repeat(detail_inner_w.max(0) as usize);
    put_text(buffer, row, detail_inner_left, &sep, Color::Rgb(40, 60, 80));
    row += 1;
    put_text(buffer, row, detail_inner_left, "Assign Squad:", Color::Cyan);
    row += 1;

    // Merc list for squad assignment
    let squad_panel_focused = ui.staging_mission_index.is_some();
    for (ri, merc) in deep.prestige.roster.iter().enumerate() {
        if row >= content_bottom - 1 {
            break;
        }
        let is_sel = ri == ui.selected_index && squad_panel_focused;
        let is_assigned = ui.staged_squad.contains(&merc.id);
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let check = if is_assigned { "[\u{2713}] " } else { "[ ] " };
        let avail_str = match &merc.status {
            MercStatus::Available => format!("  {} L{}", merc.archetype.display_name(), merc.level),
            MercStatus::OnMission(_) => "  (on mission)".to_string(),
            MercStatus::Injured { missions_remaining } => format!("  (injured {})", missions_remaining),
            MercStatus::Lost => "  (lost)".to_string(),
        };
        let merc_color = if merc.is_available() { Color::White } else { Color::DarkGray };
        let arch_color = if merc.is_available() { archetype_color(merc.archetype) } else { Color::DarkGray };
        let name_line = format!("{}{}{}", cursor, check, merc.name);
        put_text(buffer, row, detail_inner_left, &name_line, merc_color);
        put_text(buffer, row, detail_inner_left, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
        put_text(buffer, row, detail_inner_left + 2, check, if is_assigned { Color::Green } else { Color::DarkGray });
        put_text(buffer, row, detail_inner_left + 2 + check.len() as i32 + merc.name.len() as i32, &avail_str, arch_color);
        row += 1;
    }

    // Power summary at bottom of detail panel
    let squad_power: u32 = ui.staged_squad.iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.effective_power())
        .sum();
    let req_met = squad_power >= m.min_squad_power;
    let power_color = if req_met { Color::Green } else { Color::LightRed };
    let req_icon = if req_met { "\u{2713}" } else { "\u{2717}" };
    let power_line = format!("Power: {}  {} Requirements", squad_power, req_icon);
    put_text(buffer, content_bottom - 1, detail_inner_left, &power_line, power_color);
}
