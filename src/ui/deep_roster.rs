//! The Deep — Roster and Recruit sub-view rendering.

use crate::deep::{DeepState, DeepUiState, MercArchetype, Mercenary};
use chrono::Utc;
use ratatui::style::Color;

use super::deep_missions::archetype_color;
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{put_cell, put_text, put_text_centered, SceneCell};

/// Infer quality tier glyph + color from base stats vs archetype baseline.
pub(super) fn quality_glyph(merc: &Mercenary) -> (char, Color) {
    use crate::deep::MercQuality;
    match merc.quality {
        MercQuality::Elite => ('\u{2726}', Color::Rgb(255, 215, 0)), // ✦ gold
        MercQuality::Rare => ('\u{2605}', Color::Yellow),            // ★
        MercQuality::Uncommon => ('\u{25c9}', Color::Green),         // ◉
        MercQuality::Common => ('\u{25cf}', Color::White),           // ●
    }
}

/// Quality tier label text.
fn quality_label(merc: &Mercenary) -> (&'static str, Color) {
    use crate::deep::MercQuality;
    match merc.quality {
        MercQuality::Elite => ("Elite", Color::Rgb(255, 215, 0)),
        MercQuality::Rare => ("Rare", Color::Yellow),
        MercQuality::Uncommon => ("Uncommon", Color::Green),
        MercQuality::Common => ("Common", Color::White),
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

/// Injury status label with remaining recovery time (e.g. "Injured 3h 42m")
/// and a color that eases from red toward yellow as recovery approaches.
pub(super) fn injury_status_display(
    recover_at: chrono::DateTime<Utc>,
    now: chrono::DateTime<Utc>,
) -> (String, Color) {
    let remaining = (recover_at - now).num_seconds().max(0);
    if remaining == 0 {
        return ("Recovering".to_string(), Color::Yellow);
    }
    let color = if remaining <= 4 * 3600 {
        Color::Yellow
    } else if remaining <= 10 * 3600 {
        Color::LightRed
    } else {
        Color::Red
    };
    (format!("Injured {}", format_countdown(remaining)), color)
}

// Roster rendering is handled inline in deep_missions.rs (Status tab).

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
    let roster_count = deep.session.roster.len();
    let max_roster = rank.max_roster() as usize;
    let pool = &deep.session.recruit_pool;
    let marks = deep.session.warband_marks;

    // ── Summary header ──
    let now = Utc::now();
    let refresh_secs = (pool.refreshed_at + chrono::Duration::hours(24) - now)
        .num_seconds()
        .max(0);
    let capacity_color = if roster_count < max_roster {
        Color::Green
    } else {
        Color::LightRed
    };
    let pool_count = pool.candidates.len();
    let summary = format!(
        "RECRUITS ({} available)    Roster: {}/{}    \u{25c6} {} Warband Marks",
        pool_count, roster_count, max_roster, marks,
    );
    put_text(buffer, 0, 1, &summary, Color::DarkGray);
    // Highlight pool count
    let count_str = format!("{}", pool_count);
    put_text(
        buffer,
        0,
        11,
        &count_str,
        if pool_count > 0 {
            Color::Cyan
        } else {
            Color::DarkGray
        },
    );
    // Highlight capacity portion
    let cap_start = summary.find("Roster:").unwrap_or(0) + 8;
    let cap_str = format!("{}/{}", roster_count, max_roster);
    put_text(buffer, 0, 1 + cap_start as i32, &cap_str, capacity_color);
    // Highlight marks in amber
    let marks_str = format!("\u{25c6} {} Warband Marks", marks);
    if let Some(marks_pos) = summary.find('\u{25c6}') {
        put_text(
            buffer,
            0,
            1 + marks_pos as i32,
            &marks_str,
            super::deep_missions::MARKS_COLOR,
        );
    }

    // ── Footer ──
    let footer = if ctx.tier <= SizeTier::S {
        "[\u{2191}/\u{2193}] Select  [Enter] Recruit  [Esc] Close"
    } else {
        "[\u{2191}/\u{2193}] Navigate  [Enter] Recruit  [Esc] Close"
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

    // Refresh countdown on row 1
    let refresh_color = if refresh_secs < 3600 {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    put_text(
        buffer,
        1,
        1,
        &format!("New recruits in {}", format_countdown(refresh_secs)),
        refresh_color,
    );

    let content_top = 2i32;
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
    let pool = &deep.session.recruit_pool;
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
        let arch_str: String = candidate.archetype.display_name().chars().take(8).collect();

        let line = format!(
            "{}{:14} {:8} {:3}  {} Warband Marks",
            cursor,
            candidate.name.chars().take(14).collect::<String>(),
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
            &arch_str,
            if can_afford {
                archetype_color(candidate.archetype)
            } else {
                Color::DarkGray
            },
        );
        // Cost colored (green if affordable, red if not)
        let cost_str = format!("{} Warband Marks", cost);
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
    let pool = &deep.session.recruit_pool;
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
        let arch_str: String = candidate.archetype.display_name().chars().take(8).collect();

        let line = format!(
            "{}{:14} {:8}   {:3}  {} Warband Marks",
            cursor,
            candidate.name.chars().take(14).collect::<String>(),
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
        // Archetype colored (base col 1 + cursor(2) + name(14) + sp(1) = 18)
        put_text(
            buffer,
            row,
            18,
            &arch_str,
            if can_afford {
                archetype_color(candidate.archetype)
            } else {
                Color::DarkGray
            },
        );
        // Cost colored
        let cost_str = format!("{} Warband Marks", cost);
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

    // Stats with "vs Best" comparison
    row += 1;
    // Find the best roster merc of the same archetype for comparison
    let best_same_arch = deep
        .session
        .roster
        .values()
        .filter(|m| m.archetype == candidate.archetype)
        .max_by_key(|m| m.effective_power() + m.effective_resilience() + m.expertise);

    if let Some(best) = best_same_arch {
        let header = format!(
            "Stats:          Recruit  vs Best {}",
            candidate.archetype.display_name()
        );
        put_text(buffer, row, detail_inner_left, &header, Color::Cyan);
        // Highlight archetype name
        let arch_col = detail_inner_left + "Stats:          Recruit  vs Best ".len() as i32;
        put_text(
            buffer,
            row,
            arch_col,
            candidate.archetype.display_name(),
            archetype_color(candidate.archetype),
        );
        row += 1;

        let stats: [(&str, u32, u32); 3] = [
            (
                "Power:",
                candidate.effective_power(),
                best.effective_power(),
            ),
            (
                "Resilience:",
                candidate.effective_resilience(),
                best.effective_resilience(),
            ),
            ("Expertise:", candidate.expertise, best.expertise),
        ];
        for (label, recruit_val, best_val) in stats {
            if row >= content_bottom {
                break;
            }
            let delta = recruit_val as i32 - best_val as i32;
            let (delta_str, delta_color) = if delta > 0 {
                (format!("(+{}) \u{25b2}", delta), Color::Green)
            } else if delta < 0 {
                (format!("({}) \u{25bc}", delta), Color::LightRed)
            } else {
                ("(=)".to_string(), Color::DarkGray)
            };
            let line = format!(
                "  {:12} {:3}      {:3}  {}",
                label, recruit_val, best_val, delta_str
            );
            put_text(buffer, row, detail_inner_left, &line, Color::White);
            // Recolor delta portion
            let delta_col = detail_inner_left
                + format!("  {:12} {:3}      {:3}  ", label, recruit_val, best_val).len() as i32;
            put_text(buffer, row, delta_col, &delta_str, delta_color);
            row += 1;
        }

        // Verdict summary
        if row < content_bottom {
            let recruit_total = candidate.effective_power()
                + candidate.effective_resilience()
                + candidate.expertise;
            let best_total = best.effective_power() + best.effective_resilience() + best.expertise;
            let verdict = if recruit_total > best_total + 5 {
                ("Clear upgrade", Color::Green)
            } else if recruit_total >= best_total {
                ("Comparable", Color::Cyan)
            } else if candidate.expertise > best.expertise {
                ("Better expertise, weaker combat", Color::Yellow)
            } else if candidate.effective_power() > best.effective_power() {
                ("Better power, weaker elsewhere", Color::Yellow)
            } else {
                ("Weaker than current", Color::DarkGray)
            };
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  Verdict: {}", verdict.0),
                verdict.1,
            );
            row += 1;
        }
    } else {
        // No existing merc of this archetype — show plain stats with "first of type" note
        put_text(buffer, row, detail_inner_left, "Stats:", Color::Cyan);
        let arch_note = format!(
            "  (First {} in roster!)",
            candidate.archetype.display_name()
        );
        put_text(buffer, row, detail_inner_left + 7, &arch_note, Color::Green);
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
    }

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
