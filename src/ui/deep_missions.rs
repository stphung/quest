//! The Deep — Hub and New Mission sub-view rendering.

use crate::deep::{
    apply_duration_modifiers, base_marks_earned, base_mission_duration_secs, event_trigger_points,
    merc_xp_per_mission, AvailableMission, DeepState, DeepUiState, DurationModifiers,
    FamiliarityLevel, Infrastructure, LayerTier, MercArchetype, MercStatus, Mission, MissionStatus,
    MissionType,
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

/// Hub timeline strip: next completion, pending events, recruit refresh.
fn render_hub_timeline_strip(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    row: i32,
    deep: &DeepState,
    now: chrono::DateTime<Utc>,
) -> i32 {
    if width < 52 || row + 3 >= buffer.len() as i32 {
        return row;
    }

    let left = 1i32;
    let right = width as i32 - 2;
    let top = row;
    let bottom = row + 3;
    draw_deep_card(
        buffer,
        left,
        top,
        right,
        bottom,
        Color::Rgb(72, 136, 198),
        Color::Rgb(6, 13, 26),
        Some("COMMAND DECK"),
    );

    let next_completion = deep
        .prestige
        .active_missions
        .iter()
        .map(|m| (m.ends_at - now).num_seconds().max(0) as u64)
        .min();
    let next_str = next_completion
        .map(|secs| {
            if secs == 0 {
                "resolving".to_string()
            } else {
                format!("~{}", format_hours(secs))
            }
        })
        .unwrap_or_else(|| "idle".to_string());

    let pending_events = deep
        .prestige
        .active_missions
        .iter()
        .filter(|m| m.has_pending_event())
        .count();
    let events_str = if pending_events > 0 {
        format!(
            "{} ping{}",
            pending_events,
            if pending_events == 1 { "" } else { "s" }
        )
    } else {
        "clear".to_string()
    };

    let refresh_secs =
        (deep.prestige.recruit_pool.refreshed_at + chrono::Duration::hours(24) - now).num_seconds();
    let recruit_str = if refresh_secs <= 0 {
        "ready".to_string()
    } else {
        format_hours(refresh_secs as u64)
    };

    let active_count = deep.prestige.active_mission_count();
    let max_concurrent = crate::deep::effective_concurrent_missions(
        deep.persistent.guild_rank,
        deep.persistent.deepest_layer_reached,
    ) as usize;
    let ready_mercs = deep
        .prestige
        .roster
        .iter()
        .filter(|m| matches!(m.status, MercStatus::Available))
        .count();
    let injured_mercs = deep
        .prestige
        .roster
        .iter()
        .filter(|m| matches!(m.status, MercStatus::Injured { .. }))
        .count();

    let line1 = format!(
        "[OPS] {}/{} live   [EVENT] {}   [NEXT] {}",
        active_count, max_concurrent, events_str, next_str
    );
    let line2 = format!(
        "[CREW] {} ready / {} injured   [RECRUIT] {}",
        ready_mercs, injured_mercs, recruit_str
    );

    let inner_w = (right - left - 2).max(1) as usize;
    let line1 = truncate_text(&line1, inner_w);
    let line2 = truncate_text(&line2, inner_w);
    put_text(buffer, top + 1, left + 1, &line1, Color::White);
    put_text(buffer, top + 2, left + 1, &line2, Color::Rgb(140, 165, 190));

    if let Some(pos) = line1.find("[OPS]") {
        put_text(
            buffer,
            top + 1,
            left + 1 + pos as i32,
            "[OPS]",
            Color::Rgb(95, 175, 235),
        );
    }
    if let Some(pos) = line1.find("[EVENT]") {
        put_text(
            buffer,
            top + 1,
            left + 1 + pos as i32,
            "[EVENT]",
            if pending_events > 0 {
                Color::Yellow
            } else {
                Color::Green
            },
        );
    }
    if let Some(pos) = line1.find("[NEXT]") {
        put_text(
            buffer,
            top + 1,
            left + 1 + pos as i32,
            "[NEXT]",
            Color::Cyan,
        );
    }
    if let Some(pos) = line2.find("[CREW]") {
        put_text(
            buffer,
            top + 2,
            left + 1 + pos as i32,
            "[CREW]",
            Color::Rgb(100, 200, 130),
        );
    }
    if let Some(pos) = line2.find("[RECRUIT]") {
        put_text(
            buffer,
            top + 2,
            left + 1 + pos as i32,
            "[RECRUIT]",
            Color::Rgb(180, 160, 100),
        );
    }

    top + 4
}

// ── Compact Hub (S-tier) ─────────────────────────────────────────────────────

/// Render a compact hub for S-tier (small) terminals.
/// Shows guild summary, active missions, and navigation keys in minimal space.
fn render_compact_hub(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
) {
    if width == 0 || height < 4 {
        return;
    }
    let mut row = 0i32;

    // Title line with generation counter
    put_text(buffer, row, 1, "THE DEEP", SECTION_LABEL_COLOR);
    let gen_label = format!("Gen.{}", deep.prestige.generation_number.max(1));
    let gen_col = (width as i32) - gen_label.len() as i32 - 1;
    put_text(buffer, row, gen_col.max(12), &gen_label, Color::DarkGray);
    row += 1;

    // Atmospheric quote
    let quotes = [
        "\"The tunnels breathe.\"",
        "\"Stone remembers.\"",
        "\"Deeper. Always deeper.\"",
        "\"The dark welcomes you.\"",
        "\"Echoes of the fallen.\"",
    ];
    let millis = super::scene_fx::current_millis();
    let quote_idx = (millis / 12_000) as usize % quotes.len();
    put_text(buffer, row, 1, quotes[quote_idx], Color::Rgb(60, 80, 120));
    row += 1;

    // Separator
    let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
    put_text(buffer, row, 1, &sep, Color::DarkGray);
    row += 1;

    // Guild info
    let rank = deep.persistent.guild_rank;
    let guild_line = format!(
        "GUILD  {} (Rank {})    L{}",
        rank.display_name(),
        rank.0,
        deep.persistent.deepest_layer_reached.max(1)
    );
    put_text(buffer, row, 1, &guild_line, Color::White);
    row += 1;

    let marks_line = format!("MARKS  {} WM", deep.prestige.warband_marks);
    put_text(buffer, row, 1, &marks_line, MARKS_COLOR);
    row += 1;

    // Separator
    put_text(buffer, row, 1, &sep, Color::DarkGray);
    row += 1;

    // Completed missions first (matches full Hub ordering)
    for mission in &deep.prestige.pending_results {
        if row >= height as i32 - 1 {
            break;
        }
        let type_label = mission_type_label(mission.mission_type);
        let callsign = mission_callsign(mission);
        let line = format!(
            "\u{2713} {} {} L{}  COMPLETE",
            callsign, type_label, mission.layer
        );
        put_text(buffer, row, 1, &line, Color::Green);
        row += 1;
    }

    // Active missions (compact), sorted by urgency then ETA.
    let now = Utc::now();
    let mut display_order: Vec<(usize, bool, u64)> = deep
        .prestige
        .active_missions
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let has_event = m.has_pending_event();
            let remaining = (m.ends_at - now).num_seconds().max(0) as u64;
            (i, has_event, remaining)
        })
        .collect();
    display_order.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));

    for (priority, &(idx, _, _)) in display_order.iter().enumerate() {
        if row >= height as i32 - 1 {
            break;
        }
        let mission = &deep.prestige.active_missions[idx];
        let type_label = match mission.mission_type {
            MissionType::SupplyRun => "Supply",
            MissionType::Recon => "Recon",
            MissionType::Expedition => "Exped",
            MissionType::Breakthrough => "Break",
            MissionType::GatewayExpedition => "Gate",
            MissionType::Construction(_) => "Build",
        };
        let callsign = mission_callsign(mission);
        let remaining = (mission.ends_at - now).num_seconds().max(0) as u64;
        let h = remaining / 3600;
        let m = (remaining % 3600) / 60;
        let (urgency, _) = mission_urgency_badge(mission, remaining);
        let evt = if mission.has_pending_event() {
            " [evt!]"
        } else {
            ""
        };
        let line = format!(
            ">P{} {} {} L{}  {}h {:02}m {}{}",
            priority + 1,
            callsign,
            type_label,
            mission.layer,
            h,
            m,
            urgency,
            evt
        );
        let color = if mission.has_pending_event() {
            Color::Yellow
        } else {
            Color::Cyan
        };
        put_text(buffer, row, 1, &line, color);
        row += 1;
    }

    if deep.prestige.active_missions.is_empty() && deep.prestige.pending_results.is_empty() {
        put_text(buffer, row, 1, "  No active missions.", Color::DarkGray);
        row += 1;

        // Show warband log if available
        let log = &deep.prestige.warband_log;
        if !log.is_empty() && row < height as i32 - 2 {
            for entry in log.iter().rev().take(3) {
                if row >= height as i32 - 1 {
                    break;
                }
                let (icon, color) = match entry.outcome {
                    crate::deep::MissionOutcome::Success => ("\u{2713}", Color::Green),
                    crate::deep::MissionOutcome::PartialSuccess => ("\u{25cb}", Color::Yellow),
                    crate::deep::MissionOutcome::Failure => ("\u{2717}", Color::LightRed),
                };
                let line = format!(
                    "{} L{} {} {}M",
                    icon, entry.layer, entry.mission_name, entry.marks_earned
                );
                put_text(buffer, row, 1, &line, color);
                row += 1;
            }
        }

        // First-visit hint
        if ui.hub_visit_count <= 1 && log.is_empty() && row < height as i32 - 1 {
            put_text(
                buffer,
                row,
                1,
                "Start with a Supply Run (free).",
                Color::Rgb(50, 80, 110),
            );
        }
    }

    // Navigation keys (footer)
    put_text(
        buffer,
        height as i32 - 1,
        1,
        "[N]ew  [R]ecruit  [L]ayers  [?]",
        SECTION_LABEL_COLOR,
    );
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

    if ctx.tier <= SizeTier::S {
        render_compact_hub(buffer, width, height, deep, ui);
        return;
    }

    let now = Utc::now();
    let rank = deep.persistent.guild_rank;
    let marks = deep.prestige.warband_marks;
    let roster_count = deep.prestige.roster.len();
    let max_roster = rank.max_roster() as usize;
    let active_count = deep.prestige.active_mission_count() as u32;
    let max_concurrent =
        crate::deep::effective_concurrent_missions(rank, deep.persistent.deepest_layer_reached);
    let frontier = deep.persistent.frontier_layer();
    let deepest = deep.persistent.deepest_layer_reached;
    let is_compact = ctx.tier <= SizeTier::S;

    // ── Guild Status Block ──
    let mut header_row = 0i32;

    let generation = deep.prestige.generation_number;

    if is_compact {
        // Compact: 2-row guild block
        let marks_str = format!("\u{25c6}{}M", marks);
        let gen_str = if generation > 1 {
            format!("  Gen {}", generation)
        } else {
            String::new()
        };
        let line = format!(
            "Rank {} {}  {}/{}  {}/{}  {}{}",
            rank.0,
            rank.display_name(),
            roster_count,
            max_roster,
            active_count,
            max_concurrent,
            marks_str,
            gen_str,
        );
        put_text(buffer, header_row, 1, &line, Color::White);
        // Recolor marks in amber
        if let Some(pos) = line.find('\u{25c6}') {
            put_text(buffer, header_row, 1 + pos as i32, &marks_str, MARKS_COLOR);
        }
        header_row += 1;

        let frontier_tier = crate::deep::LayerTier::from_layer(frontier);
        put_text(
            buffer,
            header_row,
            1,
            &format!("Frontier: L{} {}", frontier, frontier_tier.display_name()),
            Color::DarkGray,
        );
        header_row += 1;
    } else {
        // Full: 4-row guild block
        let gen_label = if generation > 1 {
            format!("GUILD STATUS  \u{2014}  Generation {}", generation)
        } else {
            "GUILD STATUS".to_string()
        };
        put_text(buffer, header_row, 1, &gen_label, SECTION_LABEL_COLOR);
        header_row += 1;

        // Row 1: Rank + key stats + marks (with goal hint)
        let cheapest_recruit = deep
            .prestige
            .recruit_pool
            .recruit_costs
            .iter()
            .copied()
            .filter(|&c| c > 0)
            .min();
        let marks_str = if let Some(cost) = cheapest_recruit {
            if marks < cost {
                format!("\u{25c6} {}/{} Marks", marks, cost)
            } else {
                format!("\u{25c6} {} Marks", marks)
            }
        } else {
            format!("\u{25c6} {} Marks", marks)
        };
        let rank_line = format!(
            "Rank {} \u{2014} {}    Mercs: {}/{}   Missions: {}/{}   {}",
            rank.0,
            rank.display_name(),
            roster_count,
            max_roster,
            active_count,
            max_concurrent,
            marks_str,
        );
        put_text(buffer, header_row, 1, &rank_line, Color::White);
        // Recolor marks in amber
        if let Some(pos) = rank_line.find('\u{25c6}') {
            put_text(buffer, header_row, 1 + pos as i32, &marks_str, MARKS_COLOR);
        }
        header_row += 1;

        // Row 2: Frontier info
        let frontier_tier = crate::deep::LayerTier::from_layer(frontier);
        put_text(
            buffer,
            header_row,
            1,
            &format!(
                "Frontier: Layer {} ({})   Deepest: Layer {}",
                frontier,
                frontier_tier.display_name(),
                deepest.max(1),
            ),
            Color::DarkGray,
        );
        header_row += 1;

        // Row 2b: Readiness indicator
        let avail_mercs = deep
            .prestige
            .roster
            .iter()
            .filter(|m| matches!(m.status, MercStatus::Available))
            .count();
        let injured_mercs = deep
            .prestige
            .roster
            .iter()
            .filter(|m| matches!(m.status, MercStatus::Injured { .. }))
            .count();
        let deployed_mercs = deep
            .prestige
            .roster
            .iter()
            .filter(|m| matches!(m.status, MercStatus::OnMission(_)))
            .count();
        let mut readiness_col = 1i32;
        let ready_str = format!("Ready: {}", avail_mercs);
        let ready_color = if avail_mercs > 0 {
            Color::Green
        } else {
            Color::LightRed
        };
        put_text(buffer, header_row, readiness_col, &ready_str, ready_color);
        readiness_col += ready_str.len() as i32 + 3;
        if injured_mercs > 0 {
            let inj_str = format!("Injured: {}", injured_mercs);
            put_text(buffer, header_row, readiness_col, &inj_str, Color::Yellow);
            readiness_col += inj_str.len() as i32 + 3;
        }
        if deployed_mercs > 0 {
            let dep_str = format!("Deployed: {}", deployed_mercs);
            put_text(buffer, header_row, readiness_col, &dep_str, Color::Cyan);
        }
        header_row += 1;

        // Row 3: Inheritance message (only if generation > 1)
        if generation > 1 {
            let cleared_count = deep.persistent.layers.iter().filter(|l| l.cleared).count();
            let infra_count: usize = deep
                .persistent
                .layers
                .iter()
                .map(|l| l.infrastructure.len())
                .sum();
            if cleared_count > 0 || infra_count > 0 {
                put_text(
                    buffer,
                    header_row,
                    1,
                    &format!(
                        "Your predecessors cleared {} layers and built {} structures. Their work endures.",
                        cleared_count, infra_count,
                    ),
                    Color::Rgb(50, 80, 110),
                );
                header_row += 1;
            }
        }

        // Row 4: Next rank requirement (only if can advance)
        if rank.can_advance() {
            if let Some(next) = rank.next() {
                if let Some(needed_layer) = next.required_breakthrough_layer() {
                    put_text(
                        buffer,
                        header_row,
                        1,
                        &format!(
                            "Advance to {}: complete Layer {} Breakthrough",
                            next.display_name(),
                            needed_layer
                        ),
                        Color::Rgb(50, 120, 60),
                    );
                    header_row += 1;
                }
            }
        }

        // Timeline strip for scannable operational context.
        if header_row + 4 < height as i32 - 6 {
            header_row = render_hub_timeline_strip(buffer, width, header_row, deep, now);
        }
    }

    // ── Prestige cycle hint (first visit only) ──
    if ui.hub_visit_count <= 1 {
        let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
        put_text(buffer, header_row, 1, &sep, Color::Rgb(40, 60, 80));
        header_row += 1;
        put_text(
            buffer,
            header_row,
            1,
            "Tip: Mercs and Marks reset on prestige. Infrastructure persists.",
            Color::Rgb(50, 80, 110),
        );
        header_row += 1;
    }

    // ── Separator ──
    let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
    put_text(buffer, header_row, 1, &sep, Color::Rgb(40, 60, 80));
    header_row += 1;

    // ── Active missions list ──
    let missions_top = header_row;
    let missions_bottom = height as i32 - 1; // leave 1 row for footer

    let active = &deep.prestige.active_missions;
    let completed = &deep.prestige.pending_results;

    let mut missions_end_row = missions_top;
    if active.is_empty() && completed.is_empty() {
        // No active missions — show warband log or atmospheric text
        let log = &deep.prestige.warband_log;
        if !log.is_empty() {
            // Show last 5 warband log entries
            let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
            put_text(buffer, missions_end_row, 1, &sep, Color::Rgb(40, 60, 80));
            missions_end_row += 1;
            put_text(
                buffer,
                missions_end_row,
                1,
                "WARBAND LOG",
                SECTION_LABEL_COLOR,
            );
            missions_end_row += 1;
            for entry in log.iter().rev().take(5) {
                if missions_end_row >= missions_bottom {
                    break;
                }
                let (icon, color) = match entry.outcome {
                    crate::deep::MissionOutcome::Success => ("\u{2713}", Color::Green),
                    crate::deep::MissionOutcome::PartialSuccess => ("\u{25cb}", Color::Yellow),
                    crate::deep::MissionOutcome::Failure => ("\u{2717}", Color::LightRed),
                };
                let line = format!(
                    "{} Layer {} \u{2014} {} \u{2014} {} Marks",
                    icon, entry.layer, entry.mission_name, entry.marks_earned
                );
                put_text(buffer, missions_end_row, 1, &line, color);
                missions_end_row += 1;
            }
        }

        // Atmospheric text when no missions and no log, or below log if space remains
        let remaining_space = (missions_bottom - missions_end_row).max(0) as usize;
        if remaining_space >= 3 {
            let millis = super::scene_fx::current_millis();
            let atmosphere_messages = if deep.persistent.gateway_opened {
                // Permanent post-gateway message
                &[
                    "The Gateway stands open. The Wellspring waits.",
                    "The Wellspring has seen this before. It is patient.",
                    "What waits below the Wellspring is not a reward. It is an answer.",
                    "Your predecessors went as far as this. You have gone further.",
                ][..]
            } else {
                tier_atmosphere_messages(deep.persistent.frontier_layer())
            };
            let msg_idx = (millis / 8000) as usize % atmosphere_messages.len();
            let atmo_row = if log.is_empty() {
                missions_top + remaining_space as i32 / 2
            } else {
                missions_end_row + 1
            };
            if atmo_row < missions_bottom {
                let atmo_color = if deep.persistent.gateway_opened {
                    Color::Rgb(255, 215, 0) // Gold for gateway
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

        // ── Actionable empty state panel ──
        let remaining_for_actions = (missions_bottom - missions_end_row).max(0) as usize;
        if remaining_for_actions >= 3 {
            let mut action_row = missions_end_row + 1;

            // Check if all mercs are injured and no missions active
            let all_injured = !deep.prestige.roster.is_empty()
                && deep
                    .prestige
                    .roster
                    .iter()
                    .all(|m| matches!(m.status, MercStatus::Injured { .. }))
                && deep.prestige.active_missions.is_empty();

            if all_injured {
                put_text(
                    buffer,
                    action_row,
                    3,
                    "Your mercs are recovering. They'll be ready after the next mission resolves.",
                    Color::Rgb(80, 80, 120),
                );
                action_row += 2;
            }

            // Action shortcuts
            let has_missions = !deep.prestige.available_missions.is_empty();
            if has_missions && action_row < missions_bottom {
                put_text(
                    buffer,
                    action_row,
                    3,
                    "[N] New Mission",
                    SECTION_LABEL_COLOR,
                );
                put_text(
                    buffer,
                    action_row,
                    20,
                    "\u{2014} Send your first squad",
                    Color::DarkGray,
                );
                action_row += 1;
            }
            if action_row < missions_bottom {
                put_text(buffer, action_row, 3, "[R] Recruit", SECTION_LABEL_COLOR);
                put_text(
                    buffer,
                    action_row,
                    20,
                    "\u{2014} Hire mercenaries",
                    Color::DarkGray,
                );
                action_row += 1;
            }
            if action_row < missions_bottom {
                put_text(buffer, action_row, 3, "[L] Layers", SECTION_LABEL_COLOR);
                put_text(
                    buffer,
                    action_row,
                    20,
                    "\u{2014} View explored territory",
                    Color::DarkGray,
                );
                action_row += 1;
            }

            if marks == 0 && action_row < missions_bottom {
                put_text(
                    buffer,
                    action_row,
                    3,
                    "Supply Runs are free \u{2014} start there.",
                    Color::Rgb(40, 80, 50),
                );
            }
        }
    } else {
        let millis = current_millis();
        let mut row = missions_top;

        // ── Completed missions section ──
        if !completed.is_empty() {
            render_section_rule(buffer, row, width, "COMPLETED", Some(completed.len()));
            row += 1;

            for (completed_idx, mission) in completed.iter().enumerate() {
                if row + 1 >= missions_bottom {
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

                // [✓] Callsign + mission identity      COLLECT -> [Enter]
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
                        "Debrief ready \u{00b7} +{} WM confirmed \u{00b7} press [Enter]",
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
            if row < missions_bottom {
                put_text(
                    buffer,
                    row,
                    3,
                    "Priority: pending events first, then shortest ETA.",
                    Color::Rgb(68, 102, 140),
                );
                row += 1;
            }

            // Sort display order: event-pending first, then by time remaining ascending
            let mut display_order: Vec<(usize, bool, u64)> = active
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let has_event = m.has_pending_event();
                    let remaining = (m.ends_at - now).num_seconds().max(0) as u64;
                    (i, has_event, remaining)
                })
                .collect();
            display_order.sort_by(|a, b| {
                b.1.cmp(&a.1) // events first
                    .then(a.2.cmp(&b.2)) // then by remaining time ascending
            });

            for (display_idx, &(orig_idx, _has_event, _)) in display_order.iter().enumerate() {
                let card_height = if is_compact { 2 } else { 3 };
                if row + card_height > missions_bottom {
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
                let card_bottom = row + card_height - 1;
                let base_bg = if mission.has_pending_event() {
                    Color::Rgb(18, 14, 12)
                } else {
                    Color::Rgb(7, 13, 24)
                };
                let selected_bg = if mission.has_pending_event() {
                    Color::Rgb(30, 22, 18)
                } else {
                    Color::Rgb(16, 28, 46)
                };
                tint_panel_background(
                    buffer,
                    card_left,
                    card_top,
                    card_right,
                    card_bottom,
                    if is_selected { selected_bg } else { base_bg },
                );
                let border_color = if is_selected {
                    Color::Rgb(95, 175, 235)
                } else if mission.has_pending_event() {
                    Color::Rgb(130, 98, 56)
                } else {
                    Color::Rgb(42, 68, 99)
                };
                draw_panel_outline(
                    buffer,
                    card_left,
                    card_top,
                    card_right,
                    card_bottom,
                    border_color,
                );
                let line_col = card_left + 2;

                // Status glyph
                let (glyph, glyph_color) = if mission.has_pending_event() {
                    ("[!] ", Color::Yellow)
                } else {
                    ("[\u{25b6}] ", Color::Cyan)
                };

                // Line 1: glyph + priority + callsign identity + urgency badge.
                put_text(buffer, row, line_col, glyph, glyph_color);
                let prio = format!("[P{}] ", display_idx + 1);
                put_text(
                    buffer,
                    row,
                    line_col + glyph.len() as i32,
                    &prio,
                    urgency_color,
                );
                let info = format!(
                    "{}  {} \u{00b7} L{} \u{00b7} {} {}",
                    callsign,
                    type_name,
                    mission.layer,
                    risk_icon(risk_tier),
                    risk_label(risk_tier)
                );
                let urgency_badge = format!(" [{}]", urgency_label);
                let reserved = urgency_badge.len() as i32;
                let info_w =
                    (card_right - line_col - glyph.len() as i32 - prio.len() as i32 - reserved - 2)
                        .max(12) as usize;
                let info_display = truncate_text(&info, info_w);
                put_text(
                    buffer,
                    row,
                    line_col + glyph.len() as i32 + prio.len() as i32,
                    &info_display,
                    tc,
                );
                let badge_col = (card_right - urgency_badge.len() as i32 - 1).max(
                    line_col
                        + glyph.len() as i32
                        + prio.len() as i32
                        + info_display.len() as i32
                        + 1,
                );
                put_text(buffer, row, badge_col, &urgency_badge, urgency_color);
                row += 1;

                if !is_compact {
                    // Line 2: progress bar + % + time
                    let bar_width = (card_right - line_col - 17).max(12) as usize;
                    // Pulse effect at >95% progress
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
                    render_progress_bar(buffer, row, line_col + 2, bar_width, progress, bar_color);
                    let time_str = if progress >= 1.0
                        && !matches!(mission.status, MissionStatus::EventPending)
                    {
                        "  Resolving...".to_string()
                    } else if remaining_secs > 0 {
                        let pct = (progress * 100.0) as u32;
                        format!("  {}%   ~{} left", pct, format_hours(remaining_secs))
                    } else {
                        let pct = (progress * 100.0) as u32;
                        format!("  {}%   done", pct)
                    };
                    put_text(
                        buffer,
                        row,
                        line_col + 2 + bar_width as i32,
                        &time_str,
                        Color::DarkGray,
                    );
                    row += 1;

                    // Event countdown line + squad identity
                    let tier = LayerTier::from_layer(mission.layer);
                    let triggers = event_trigger_points(mission.mission_type, tier);
                    let event_hint = if mission.has_pending_event() {
                        "\u{26a1} Event pending \u{2014} respond now!".to_string()
                    } else if triggers.is_empty() {
                        "No events (safe mission)".to_string()
                    } else if let Some(&next_trigger) = triggers.iter().find(|&&t| t > progress) {
                        let secs_to_event =
                            ((next_trigger - progress) * total_secs as f64).round() as u64;
                        let resolved = mission.events.iter().filter(|e| e.is_resolved()).count();
                        format!(
                            "Next event in ~{}  ({}/{})",
                            format_hours(secs_to_event),
                            resolved,
                            triggers.len()
                        )
                    } else {
                        let resolved = mission.events.iter().filter(|e| e.is_resolved()).count();
                        format!("All events resolved ({}/{})", resolved, triggers.len())
                    };
                    let crew_hint = if squad_size > 1 {
                        format!("Lead {} +{}", leader, squad_size - 1)
                    } else {
                        format!("Lead {}", leader)
                    };
                    let line3 = format!("{}  \u{00b7}  {}", crew_hint, event_hint);
                    let line3_w = (card_right - (line_col + 2) - 1).max(12) as usize;
                    let line3_display = truncate_text(&line3, line3_w);
                    put_text(
                        buffer,
                        row,
                        line_col + 2,
                        &line3_display,
                        Color::Rgb(60, 90, 130),
                    );
                    // Recolor the pending event line
                    if mission.has_pending_event() {
                        put_text(buffer, row, line_col + 2, &line3_display, Color::Yellow);
                    }
                    row += 1;
                } else {
                    // Compact: bar + time on same row
                    let bar_width = 12usize;
                    let pct = (progress * 100.0) as u32;
                    render_progress_bar(buffer, row, line_col + 2, bar_width, progress, tc);
                    let time_str = if remaining_secs > 0 {
                        format!(" {}%  ~{}", pct, format_hours(remaining_secs))
                    } else {
                        format!(" {}%  done", pct)
                    };
                    put_text(
                        buffer,
                        row,
                        line_col + 2 + bar_width as i32,
                        &time_str,
                        Color::DarkGray,
                    );
                    row += 1;
                }
            }
        }

        // ── Warband log (below missions, last 5 entries) ──
        let log = &deep.prestige.warband_log;
        if !log.is_empty() && row + 2 < missions_bottom {
            render_section_rule(buffer, row, width, "WARBAND LOG", None);
            row += 1;
            for entry in log.iter().rev().take(5) {
                if row >= missions_bottom {
                    break;
                }
                let (icon, color) = match entry.outcome {
                    crate::deep::MissionOutcome::Success => ("\u{2713}", Color::Green),
                    crate::deep::MissionOutcome::PartialSuccess => ("\u{25cb}", Color::Yellow),
                    crate::deep::MissionOutcome::Failure => ("\u{2717}", Color::LightRed),
                };
                let line = format!(
                    "{} Layer {} \u{2014} {} \u{2014} {} Marks",
                    icon, entry.layer, entry.mission_name, entry.marks_earned
                );
                put_text(buffer, row, 1, &line, color);
                row += 1;
            }
        }
    }

    // ── Footer ──
    let footer = match ctx.tier {
        SizeTier::S => "[Tab]Switch  [Enter]Select  [Esc]Close",
        _ => "[Tab] Switch View  [Enter] Select  [Esc] Close",
    };
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);
    // [?] Help hint
    let help_hint = "[?] Help";
    let help_col = (width as i32 - help_hint.len() as i32 - 1).max(footer.len() as i32 + 2);
    put_text(
        buffer,
        height as i32 - 1,
        help_col,
        help_hint,
        Color::Rgb(50, 70, 100),
    );
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

    // ── Footer (context-sensitive: mission list vs squad staging) ──
    let footer = if ui.staging_mission_index.is_some() {
        if is_compact {
            "[\u{2191}/\u{2193}] Select  [Space] Toggle  [Enter] Launch  [Esc] Cancel"
        } else {
            "[\u{2191}/\u{2193}] Navigate  [Space] Toggle Merc  [Enter] Launch Mission  [Esc] Cancel"
        }
    } else if is_compact {
        "[\u{2191}/\u{2193}] Select  [Enter] Assign Squad  [Esc] Back"
    } else {
        "[\u{2191}/\u{2193}] Select Mission  [Enter] Assign Squad  [Esc] Back"
    };
    // Flash message (error/info) shown above the footer.
    if let Some(msg) = &ui.flash_message {
        put_text(buffer, height as i32 - 2, 1, msg, Color::LightRed);
    }
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);

    let content_top = 0i32;
    let content_bottom = height as i32 - 2; // leave room for flash + footer
    let content_height = (content_bottom - content_top).max(0) as usize;

    let available = &deep.prestige.available_missions;

    // Right-aligned: [?] Help + Marks balance in footer
    let marks_display = format!("\u{25c6} {} M", deep.prestige.warband_marks);
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
                "Recruit mercenaries in [Recruit] tab first.",
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
            put_text_centered(
                buffer,
                mid + 1,
                width,
                "Check back after current missions complete.",
                Color::Rgb(40, 55, 80),
            );
        } else {
            put_text_centered(
                buffer,
                mid,
                width,
                "Mission pool refreshes periodically.",
                Color::Rgb(50, 70, 100),
            );
            put_text_centered(
                buffer,
                mid + 1,
                width,
                "Return in a few minutes.",
                Color::Rgb(40, 55, 80),
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

/// Compact (S-tier) single-panel new mission view.
#[allow(clippy::too_many_arguments)]
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
        // Phase 1: mission list with detail for selected
        let mut row = content_top;
        for (i, m) in available.iter().enumerate() {
            if row >= content_bottom {
                break;
            }
            let is_sel = i == ui.selected_index;
            let cursor = if is_sel { "\u{25b6} " } else { "  " };
            let tc = mission_type_color(m.mission_type);
            let cost_str = if m.marks_cost > 0 {
                format!("  {}M", m.marks_cost)
            } else {
                String::new()
            };
            let rt = m.mission_type.risk_tier();
            let ri = risk_icon(rt);
            let line = format!(
                "{}{} [{}]  L{}  {}  {}{}",
                cursor,
                ri,
                mission_type_label(m.mission_type),
                m.layer,
                format_hours(m.duration_secs),
                risk_label(rt),
                cost_str,
            );
            put_text(buffer, row, 1, &line, tc);
            put_text(
                buffer,
                row,
                1,
                cursor,
                if is_sel { Color::Cyan } else { Color::DarkGray },
            );
            // Recolor risk icon
            put_text(buffer, row, 3, ri, risk_color(rt));
            row += 1;
        }
        // Compact Phase 1 detail for selected mission
        if let Some(m) = available.get(ui.selected_index) {
            row += 1;
            if row < content_bottom && !m.description.is_empty() {
                let desc_trunc: String = m.description.chars().take((_width - 4).max(10)).collect();
                put_text(buffer, row, 1, &desc_trunc, Color::DarkGray);
                row += 1;
            }
            if row < content_bottom {
                let mut detail = format!("Pwr: {}  ", m.min_squad_power);
                if let Some(req) = m.required_archetype {
                    let in_roster = deep.prestige.roster.iter().any(|r| r.archetype == req);
                    if in_roster {
                        detail.push_str(&format!("\u{2713} {} req  ", req.display_name()));
                    } else {
                        detail.push_str(&format!("\u{26a0} {} missing!  ", req.display_name()));
                    }
                }
                if m.marks_cost > 0 {
                    let marks = deep.prestige.warband_marks;
                    let afford = if marks >= m.marks_cost { "" } else { " LOW" };
                    detail.push_str(&format!("{}M (have {}{})", m.marks_cost, marks, afford));
                }
                put_text(buffer, row, 1, &detail, Color::DarkGray);
                // Color archetype warning
                if let Some(req) = m.required_archetype {
                    let in_roster = deep.prestige.roster.iter().any(|r| r.archetype == req);
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
                    format!("  {}M", m.marks_cost)
                } else {
                    String::new()
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
                for merc in deep.prestige.roster.iter() {
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
                let has_unavailable = deep.prestige.roster.iter().any(|m| !m.is_available());
                if has_unavailable && row < content_bottom - 2 {
                    let sep_str: String = "\u{2500} ".repeat((_width / 2).max(4));
                    put_text(
                        buffer,
                        row,
                        1,
                        &sep_str[..sep_str.len().min(_width - 2)],
                        Color::Rgb(40, 60, 80),
                    );
                    row += 1;
                    for merc in deep.prestige.roster.iter() {
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
    let list_width = (width * 40 / 100).max(18).min(width.saturating_sub(20));
    let detail_left = list_width as i32;
    let detail_width = width.saturating_sub(list_width);
    let staging = ui.staging_mission_index.is_some();

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
    let left_heading = if staging { "ASSIGN SQUAD" } else { "AVAILABLE" };
    let left_heading_color = if staging {
        Color::Rgb(95, 175, 235)
    } else {
        SECTION_LABEL_COLOR
    };
    put_text(buffer, content_top, 1, left_heading, left_heading_color);
    if staging {
        put_text(
            buffer,
            content_top,
            detail_left + 2,
            "SUMMARY (read-only)",
            Color::DarkGray,
        );
    }
    let list_inner_top = content_top + 1;

    if !staging {
        // Phase 1: mission list on left
        for (i, m) in available.iter().enumerate() {
            let row = list_inner_top + i as i32;
            if row >= content_bottom {
                break;
            }
            let is_sel = i == ui.selected_index;
            let cursor = if is_sel { "\u{25b6} " } else { "  " };
            let tc = mission_type_color(m.mission_type);
            let max_name_w = (list_width as i32 - 4).max(8) as usize;
            let type_name = mission_type_label(m.mission_type);
            let cost_str = if m.marks_cost > 0 {
                format!("  {}M", m.marks_cost)
            } else {
                String::new()
            };
            let rt = m.mission_type.risk_tier();
            let ri = risk_icon(rt);
            let line = format!(
                "{}{} [{:width$}]  L{}  {}{}",
                cursor,
                ri,
                &type_name[..type_name.len().min(max_name_w)],
                m.layer,
                format_hours(m.duration_secs),
                cost_str,
                width = 0,
            );
            put_text(buffer, row, 1, &line, tc);
            put_text(
                buffer,
                row,
                1,
                cursor,
                if is_sel { Color::Cyan } else { Color::DarkGray },
            );
            // Recolor risk icon
            put_text(buffer, row, 3, ri, risk_color(rt));
        }
    } else {
        // Phase 2: merc list on left with available/unavailable groups
        if let Some(mi) = ui.staging_mission_index {
            if let Some(m) = available.get(mi) {
                let tc = mission_type_color(m.mission_type);
                put_text(
                    buffer,
                    content_top,
                    left_heading.len() as i32 + 3,
                    &format!(
                        "{}  L{}  {}M",
                        mission_type_label(m.mission_type),
                        m.layer,
                        m.marks_cost
                    ),
                    tc,
                );
            }
        }

        let mut row = list_inner_top;

        // Available mercs (selectable)
        let mut avail_idx = 0usize;
        for merc in deep.prestige.roster.iter() {
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
        let has_unavailable = deep.prestige.roster.iter().any(|m| !m.is_available());
        if has_unavailable && row < content_bottom - 1 {
            let sep_str: String = "\u{2500} ".repeat((list_width / 2).max(4));
            put_text(
                buffer,
                row,
                1,
                &sep_str[..sep_str.len().min(list_width)],
                Color::Rgb(40, 60, 80),
            );
            row += 1;

            for merc in deep.prestige.roster.iter() {
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
                        &merc.name[..merc.name.len().min(14)],
                        merc.archetype.display_name(),
                        avail_str
                    ),
                    Color::Rgb(50, 60, 70),
                );
                row += 1;
            }
        }
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
        render_squad_summary_panel(
            buffer,
            deep,
            ui,
            m,
            detail_inner_left,
            detail_inner_w,
            content_top + 1,
            content_bottom,
        );
    }
}

/// One-line mission type description shown during first visits.
fn mission_type_hint(mt: MissionType) -> &'static str {
    match mt {
        MissionType::SupplyRun => "Safe income \u{2014} always returns, earns Marks reliably",
        MissionType::Recon => "Raises layer familiarity \u{2014} cuts future mission times",
        MissionType::Expedition => "Core rewards \u{2014} items, Marks, and merc XP",
        MissionType::Breakthrough => "Clears the frontier \u{2014} unlocks the next layer",
        MissionType::GatewayExpedition => "The final expedition \u{2014} breach the sealed gateway",
        MissionType::Construction(_) => {
            "Builds permanent infrastructure \u{2014} survives prestige"
        }
    }
}

/// Risk consequence description shown during first visits.
fn risk_consequence_hint(tier: u8) -> &'static str {
    match tier {
        0 => "no injuries, guaranteed return",
        1 => "rare injuries, Marks lost on failure",
        2 => "injuries likely on failure",
        3 => "injuries or death possible on failure",
        _ => "",
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
    mission_visit_count: u8,
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

    // Mission type name (colored) + first-visit type description
    let tc = mission_type_color(mission.mission_type);
    put_text(
        buffer,
        row,
        detail_inner_left,
        &mission_type_label(mission.mission_type),
        tc,
    );
    row += 1;

    if mission_visit_count < 5 && row < content_bottom {
        put_text(
            buffer,
            row,
            detail_inner_left,
            mission_type_hint(mission.mission_type),
            Color::Rgb(50, 80, 110),
        );
        row += 1;
    }

    // Description (word-wrapped, 2 lines max)
    if !mission.description.is_empty() {
        let max_w = (detail_inner_w - 1).max(10) as usize;
        let words: Vec<&str> = mission.description.split_whitespace().collect();
        let mut line_buf = String::new();
        let mut lines_rendered = 0;
        for word in &words {
            if lines_rendered >= 2 {
                break;
            }
            if line_buf.len() + word.len() + 1 > max_w && !line_buf.is_empty() {
                put_text(buffer, row, detail_inner_left, &line_buf, Color::DarkGray);
                row += 1;
                lines_rendered += 1;
                line_buf.clear();
            }
            if !line_buf.is_empty() {
                line_buf.push(' ');
            }
            line_buf.push_str(word);
        }
        if !line_buf.is_empty() && lines_rendered < 2 {
            put_text(buffer, row, detail_inner_left, &line_buf, Color::DarkGray);
            row += 1;
        }
    }
    row += 1;

    // Duration — show effective if modifiers apply, with breakdown
    let layer_record = deep.persistent.layer_record(mission.layer);
    let has_outpost = layer_record
        .map(|r| r.has_infrastructure(Infrastructure::Outpost))
        .unwrap_or(false);
    let familiarity = layer_record.map(|r| r.familiarity).unwrap_or(0);

    let tier = LayerTier::from_layer(mission.layer);
    let base_secs = base_mission_duration_secs(tier, mission.mission_type);

    let mods = DurationModifiers {
        has_outpost,
        familiarity,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective_secs = apply_duration_modifiers(base_secs, &mods);

    if row < content_bottom {
        if effective_secs != base_secs {
            let dur_str = format!(
                "Duration:  {}  \u{2192}  {} effective",
                format_hours(base_secs),
                format_hours(effective_secs)
            );
            put_text(buffer, row, detail_inner_left, &dur_str, Color::DarkGray);
            // Highlight the effective duration in cyan
            let eff_start = dur_str.find('\u{2192}').unwrap_or(0);
            put_text(
                buffer,
                row,
                detail_inner_left + eff_start as i32,
                &format!("\u{2192}  {} effective", format_hours(effective_secs)),
                Color::Cyan,
            );
            row += 1;

            // Stacked modifier breakdown
            if has_outpost && row < content_bottom {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    "  Outpost:     -25%",
                    Color::Rgb(60, 130, 90),
                );
                row += 1;
            }
            let fam_level = FamiliarityLevel::from_familiarity(familiarity);
            let fam_mod = match fam_level {
                FamiliarityLevel::Mapped => Some(("Mapped", "-10%")),
                FamiliarityLevel::Familiar => Some(("Familiar", "-20%")),
                FamiliarityLevel::Mastered => Some(("Mastered", "-30%")),
                FamiliarityLevel::Unknown => None,
            };
            if let Some((label, pct)) = fam_mod {
                if row < content_bottom {
                    put_text(
                        buffer,
                        row,
                        detail_inner_left,
                        &format!(
                            "  {}:{}{}",
                            label,
                            " ".repeat(10usize.saturating_sub(label.len())),
                            pct
                        ),
                        Color::Rgb(60, 130, 90),
                    );
                    row += 1;
                }
            }
        } else {
            let dur_str = format!("Duration:  {}", format_hours(base_secs));
            put_text(buffer, row, detail_inner_left, &dur_str, Color::DarkGray);
            row += 1;
        }
    }

    // Risk with first-visit consequence hint
    if row < content_bottom {
        let risk_tier = mission.mission_type.risk_tier();
        if mission_visit_count < 5 {
            let risk_str = format!(
                "Risk:      {} \u{2014} {}",
                risk_label(risk_tier),
                risk_consequence_hint(risk_tier)
            );
            put_text(
                buffer,
                row,
                detail_inner_left,
                &risk_str,
                risk_color(risk_tier),
            );
            let hint_part = format!("\u{2014} {}", risk_consequence_hint(risk_tier));
            let hint_col =
                detail_inner_left + format!("Risk:      {} ", risk_label(risk_tier)).len() as i32;
            put_text(buffer, row, hint_col, &hint_part, Color::DarkGray);
        } else {
            let risk_str = format!("Risk:      {}", risk_label(risk_tier));
            put_text(
                buffer,
                row,
                detail_inner_left,
                &risk_str,
                risk_color(risk_tier),
            );
        }
        row += 1;
    }

    // Cost with affordability
    if row < content_bottom {
        let marks = deep.prestige.warband_marks;
        if mission.marks_cost > 0 {
            let cost_str = format!("Cost:      {} Marks", mission.marks_cost);
            put_text(buffer, row, detail_inner_left, &cost_str, MARKS_COLOR);
            let (afford_str, afford_color) = if marks >= mission.marks_cost {
                (format!("  (have {})", marks), Color::Rgb(60, 180, 80))
            } else {
                (
                    format!("  (have {} \u{2014} INSUFFICIENT)", marks),
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
                "Cost:      Free",
                Color::Rgb(60, 180, 80),
            );
        }
        row += 1;
    }

    // Reward range (calculated from base_marks_earned with ±15% variance)
    if row < content_bottom {
        if matches!(mission.mission_type, MissionType::Construction(_)) {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "Reward:    Infrastructure (permanent)",
                Color::DarkGray,
            );
            row += 1;
        } else {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "Rewards (on success):",
                Color::Cyan,
            );
            row += 1;

            if row < content_bottom {
                let base = base_marks_earned(mission.mission_type, mission.layer);
                let min_marks = (base as f64 * 0.85).round() as u32;
                let max_marks = (base as f64 * 1.15).round() as u32;
                let marks_str = format!("  \u{25c6} {}\u{2013}{} Marks", min_marks, max_marks);
                put_text(buffer, row, detail_inner_left, &marks_str, MARKS_COLOR);
                row += 1;
            }

            if row < content_bottom {
                let xp = merc_xp_per_mission(mission.mission_type, mission.layer);
                let xp_str = format!("  \u{2605} XP per merc: ~{}", xp);
                put_text(buffer, row, detail_inner_left, &xp_str, Color::DarkGray);
                row += 1;
            }

            if row < content_bottom
                && matches!(
                    mission.mission_type,
                    MissionType::Expedition
                        | MissionType::Breakthrough
                        | MissionType::GatewayExpedition
                )
            {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    "  ? Item drop chance",
                    Color::DarkGray,
                );
                row += 1;
            }
        }
    }
    row += 1;

    // Requirements section
    if row < content_bottom {
        put_text(buffer, row, detail_inner_left, "Requires:", Color::Cyan);
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

    // Required archetype — check against full roster
    if let Some(req_arch) = mission.required_archetype {
        if row < content_bottom {
            let in_roster = deep.prestige.roster.iter().any(|m| m.archetype == req_arch);
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
            let in_roster = deep.prestige.roster.iter().any(|m| m.archetype == rec_arch);
            let (prefix, color) = if in_roster {
                ("\u{2605} ", Color::Cyan)
            } else {
                ("  ", Color::DarkGray)
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  {}{} recommended", prefix, rec_arch.display_name()),
                color,
            );
        }
    }

    // Action hint at bottom
    let hint_row = (content_bottom - 2).max(row + 1);
    if hint_row < content_bottom {
        put_text(
            buffer,
            hint_row,
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

    // Cost + balance header
    if mission.marks_cost > 0 {
        let header = format!("Cost: {} Marks     Balance: {}", mission.marks_cost, marks);
        put_text(buffer, row, detail_inner_left, &header, Color::White);
        let cost_color = if can_afford {
            Color::Green
        } else {
            Color::LightRed
        };
        put_text(
            buffer,
            row,
            detail_inner_left + 6,
            &format!("{}", mission.marks_cost),
            cost_color,
        );
    } else {
        put_text(
            buffer,
            row,
            detail_inner_left,
            "Cost: Free",
            Color::Rgb(60, 180, 80),
        );
    }
    row += 2;

    // Power meter
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
        format!("Squad Power:  {}", squad_power)
    } else {
        format!("Squad Power:  {} / {}  ({}%)", squad_power, min, ratio_pct)
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
    row += 2;

    // Archetype summary
    if row < content_bottom - 3 {
        put_text(
            buffer,
            row,
            detail_inner_left,
            "Archetypes in squad:",
            Color::Cyan,
        );
        row += 1;
    }

    let squad_archetypes: Vec<crate::deep::MercArchetype> = ui
        .staged_squad
        .iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.archetype)
        .collect();

    if squad_archetypes.is_empty() {
        if row < content_bottom - 3 {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "  (none selected)",
                Color::DarkGray,
            );
            row += 1;
        }
    } else {
        let mut seen = std::collections::HashSet::new();
        for &arch in &squad_archetypes {
            if seen.insert(arch) {
                if row >= content_bottom - 3 {
                    break;
                }
                let name = deep
                    .prestige
                    .roster
                    .iter()
                    .find(|m| m.archetype == arch && ui.staged_squad.contains(&m.id))
                    .map(|m| m.name.as_str())
                    .unwrap_or("");
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("  {} ({})", arch.display_name(), name),
                    archetype_color(arch),
                );
                row += 1;
            }
        }
    }

    // Required archetype check
    if let Some(req_arch) = mission.required_archetype {
        let req_present = squad_archetypes.contains(&req_arch);
        if row < content_bottom - 3 {
            let (prefix, color, suffix) = if req_present {
                ("\u{2713} ", Color::Green, " required \u{2014} present")
            } else {
                ("(!) ", Color::Yellow, " required \u{2014} missing!")
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("{}{}{}", prefix, req_arch.display_name(), suffix),
                color,
            );
            row += 1;
        }
    }

    // Recommended archetype check
    if let Some(rec_arch) = mission.recommended_archetype {
        let rec_present = squad_archetypes.contains(&rec_arch);
        if row < content_bottom - 3 {
            let (prefix, color, suffix) = if rec_present {
                ("\u{2605} ", Color::Cyan, " recommended \u{2014} present")
            } else {
                ("  ", Color::DarkGray, " recommended")
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("{}{}{}", prefix, rec_arch.display_name(), suffix),
                color,
            );
            row += 1;
        }
    }

    // Smart contextual hint
    row += 1;
    if row < content_bottom - 1 {
        let hint: Option<(String, Color)> = if !can_afford && mission.marks_cost > 0 {
            Some((
                "Earn Marks via Supply Runs (free)".to_string(),
                Color::DarkGray,
            ))
        } else if ui.staged_squad.is_empty() {
            Some(("Select mercs with [Space]".to_string(), Color::DarkGray))
        } else if ratio >= 1.5 {
            Some((
                "Overpowered \u{2014} mission will complete faster!".to_string(),
                Color::Rgb(80, 220, 120),
            ))
        } else if let Some(req_arch) = mission.required_archetype {
            if !squad_archetypes.contains(&req_arch) {
                let merc_with_arch = deep
                    .prestige
                    .roster
                    .iter()
                    .find(|m| m.archetype == req_arch && m.is_available());
                if let Some(m) = merc_with_arch {
                    Some((
                        format!(
                            "Add {} ({}) to meet requirement",
                            m.name,
                            req_arch.display_name()
                        ),
                        Color::Yellow,
                    ))
                } else {
                    Some((
                        "Check [Recruit] tab for required archetype".to_string(),
                        Color::Yellow,
                    ))
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some((hint_text, hint_color)) = hint {
            let hint_row = (content_bottom - 3).max(row);
            if hint_row < content_bottom - 1 {
                put_text(buffer, hint_row, detail_inner_left, &hint_text, hint_color);
            }
        }
    }

    // Launch action at bottom
    let launch_row = content_bottom - 1;
    if launch_row > row {
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
