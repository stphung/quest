//! Stats panel coordinator — delegates to submodules for rendering.

use super::responsive::{LayoutContext, SizeTier};
use super::stats_attributes::{attr_bg_color, attr_color, format_modifier};
use super::stats_equipment::draw_equipment_names_only;
use super::stats_prestige::{draw_deep_panel, format_eta};
use crate::character::attributes::AttributeType;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{get_adventurer_rank, get_prestige_tier};
use crate::core::game_logic::xp_for_next_level;
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

// Re-export for haven_scene.rs which uses super::stats_panel::enhancement_style
pub(super) use super::stats_equipment::enhancement_style;

/// Cached map of prestige rank → unlocked feature/zone names.
/// Built once from zone data + hardcoded feature unlocks, avoids
/// scanning all zones every frame.
fn prestige_unlock_names(rank: u32) -> &'static [&'static str] {
    use std::collections::HashMap;
    use std::sync::LazyLock;

    static MAP: LazyLock<HashMap<u32, Vec<&'static str>>> = LazyLock::new(|| {
        let mut m: HashMap<u32, Vec<&'static str>> = HashMap::new();
        for zone in crate::zones::get_all_zones() {
            if zone.prestige_requirement > 0 {
                m.entry(zone.prestige_requirement)
                    .or_default()
                    .push(zone.name);
            }
        }
        m.entry(10).or_default().push("Haven");
        m.entry(15).or_default().push("Soulforge");
        m.entry(15).or_default().push("Stormglass");
        m
    });

    MAP.get(&rank).map(|v| v.as_slice()).unwrap_or(&[])
}

/// Status indicator for the [D]eep shortcut in the footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeepIndicatorStatus {
    /// Deep not discovered.
    Hidden,
    /// Discovered, no missions running.
    Idle,
    /// At least one mission is running.
    Running,
    /// At least one mission has a pending event.
    EventPending,
    /// At least one mission has completed (result pending).
    Completed,
}

impl DeepIndicatorStatus {
    /// Compute the most urgent indicator status from deep state.
    pub fn from_deep(discovered: bool, deep: &crate::deep::DeepState) -> Self {
        if !discovered {
            return Self::Hidden;
        }
        let has_events = deep.prestige.has_any_pending_event();
        let has_results = !deep.prestige.pending_results.is_empty();
        let has_active = deep.prestige.active_mission_count() > 0;
        if has_events {
            Self::EventPending
        } else if has_results {
            Self::Completed
        } else if has_active {
            Self::Running
        } else {
            Self::Idle
        }
    }
}

/// Draws the stats panel
#[allow(clippy::too_many_arguments)]
pub fn draw_stats_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    ctx: &LayoutContext,
    enhancement_levels: &[u8; 7],
    achievements: &crate::achievements::Achievements,
    deep: &DeepState,
) {
    match ctx.height_tier {
        SizeTier::XL | SizeTier::L => {
            // Hero panel: 2 border + 2 header + 1 XP gauge + (4 or 5) prestige (incl sep) + 1 prestige gauge + 1 sep + 3 attrs
            let hero_height: u16 = if game_state.ascension_level > 0 {
                15 // 2 + 2 + 1 + 5 + 1 + 1 + 3
            } else {
                14 // 2 + 2 + 1 + 4 + 1 + 1 + 3
            };
            let deep_panel_height: u16 = if deep.persistent.discovered { 13 } else { 0 };

            let mut constraints = vec![
                Constraint::Length(hero_height), // Unified hero panel
                Constraint::Min(0),              // Equipment (includes sigils subsection)
            ];
            if deep_panel_height > 0 {
                constraints.push(Constraint::Length(deep_panel_height));
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            draw_hero_panel(frame, chunks[0], game_state, achievements);
            draw_equipment_names_only(
                frame,
                chunks[1],
                game_state,
                enhancement_levels,
                &game_state.storm_sigils,
            );
            if deep_panel_height > 0 {
                draw_deep_panel(frame, chunks[2], achievements, deep);
            }
        }
        _ => {}
    }
}

/// Draws the unified hero panel combining header, prestige, and attributes.
fn draw_hero_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    use super::stats_prestige::{highest_prestige_badge, prestige_ready_gauge_style, to_roman};
    use crate::ascension::types::ascension_combat_multiplier;
    use crate::character::prestige::get_next_prestige_tier;

    let dw = super::scene_fx::display_width;

    // Panel title: character name with optional title
    let title_suffix = achievements.selected_title.and_then(|id| {
        if achievements.is_unlocked(id) {
            crate::achievements::titles::get_title_text(id)
        } else {
            None
        }
    });
    let name_with_title = match title_suffix {
        Some(title) => format!("{}, {}", game_state.character_name, title),
        None => game_state.character_name.clone(),
    };
    let header_title = match title_badge(achievements) {
        Some(icon) => format!(" {} {} ", name_with_title, icon),
        None => format!(" {} ", name_with_title),
    };
    let block = Block::default().borders(Borders::ALL).title(header_title);
    let block = super::themed_block(block);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let width = inner.width as usize;
    let xp_needed = xp_for_next_level(game_state.character_level);

    // === Build header text lines (2 lines) ===
    let mut header_lines: Vec<Line> = Vec::new();

    // Row 1: Level + Power Level (right-aligned)
    let rank = get_adventurer_rank(game_state.character_level);
    let power = game_state.cached_power_rating as u64;
    let power_str = format!(
        "\u{26a1} Power Level: {}",
        super::game_common::format_number_short(power)
    );
    let level_str = format!("Level {} {}", game_state.character_level, rank);
    let gap = width.saturating_sub(dw(&level_str) + dw(&power_str));
    header_lines.push(Line::from(vec![
        Span::styled(
            level_str,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gap)),
        Span::styled(
            power_str,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Row 2: Play time + XP rate
    let play_time = format_play_time(game_state.play_time_seconds);
    let rate_suffix = match game_state.xp_per_hour() {
        Some(rate) => {
            let xp_remaining = xp_needed.saturating_sub(game_state.character_xp);
            let eta = if rate > 0 {
                let seconds = (xp_remaining as f64 / rate as f64 * 3600.0) as u64;
                format!(" ({})", format_eta(seconds))
            } else {
                String::new()
            };
            format!(
                " | {}/hr{}",
                super::game_common::format_number_short(rate),
                eta
            )
        }
        None => String::new(),
    };
    header_lines.push(Line::from(vec![
        Span::styled("\u{23f1}\u{fe0f} ", Style::default().fg(Color::Cyan)),
        Span::styled(play_time, Style::default().fg(Color::Cyan)),
        Span::styled(rate_suffix, Style::default().fg(Color::Cyan)),
    ]));

    // === Build prestige text lines (3-4 lines + titled separator) ===
    let mut prestige_lines: Vec<Line> = Vec::new();

    // Titled separator: ──── Prestige ⭐ ────
    {
        let label = match highest_prestige_badge(achievements) {
            Some(icon) => format!(" Prestige {} ", icon),
            None => " Prestige ".to_string(),
        };
        let label_w = dw(&label);
        let remaining = width.saturating_sub(label_w);
        let left_dashes = remaining / 2;
        let right_dashes = remaining - left_dashes;
        prestige_lines.push(Line::from(vec![
            Span::styled(
                "\u{2500}".repeat(left_dashes),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                label,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{2500}".repeat(right_dashes),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let tier = get_prestige_tier(game_state.prestige_rank);
    let cha_mod = game_state.attributes.modifier(AttributeType::Charisma);
    let effective_multiplier =
        DerivedStats::prestige_multiplier(tier.multiplier, &game_state.attributes);

    // Prestige row 1: Rank + prestige count
    let rank_spans = vec![
        Span::styled("🏆 Rank: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{} ({})", game_state.prestige_rank, tier.name),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("🔄 ", Style::default()),
        Span::styled(
            format!("{}", game_state.total_prestige_count),
            Style::default().fg(Color::Magenta),
        ),
    ];
    prestige_lines.push(Line::from(rank_spans));

    // Prestige row 2 (optional): Ascension
    if game_state.ascension_level > 0 {
        let asc_mult = ascension_combat_multiplier(game_state.ascension_level);
        prestige_lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "\u{2726} Ascension {} ",
                    to_roman(game_state.ascension_level)
                ),
                Style::default()
                    .fg(Color::Rgb(255, 215, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("\u{2014} ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("\u{00d7}{:.1} power", asc_mult),
                Style::default().fg(Color::Rgb(255, 215, 0)),
            ),
        ]));
    }

    // Prestige row 2b (optional): Vessel signal
    if game_state.vessel_signal_discovered {
        let violet = Color::Rgb(150, 120, 200);
        let hint_bright = super::clock::now_millis() % 1600 < 800;
        let hint_style = if hint_bright {
            Style::default().fg(violet).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(100, 75, 135))
        };
        let label = if game_state.vessel_launched {
            "\u{2726} The Vessel awaits "
        } else {
            "\u{2726} The branch withers... "
        };
        prestige_lines.push(Line::from(vec![
            Span::styled(label, Style::default().fg(violet)),
            Span::styled("[V] Vessel", hint_style),
        ]));
    }

    // Prestige row 3: XP multiplier
    let mut xp_spans = vec![
        Span::styled("⚡ XP: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{:.2}x", tier.multiplier),
            Style::default().fg(Color::Cyan),
        ),
    ];
    if cha_mod != 0 {
        xp_spans.push(Span::styled(
            format!(" +{:.1} CHA", cha_mod as f64 * 0.1),
            Style::default().fg(Color::Yellow),
        ));
    }
    xp_spans.push(Span::styled(
        " \u{2192} ",
        Style::default().fg(Color::DarkGray),
    ));
    xp_spans.push(Span::styled(
        format!("{:.2}x", effective_multiplier),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));
    prestige_lines.push(Line::from(xp_spans));

    // Prestige row 4: Unlock hint (uses cached lookup to avoid per-frame zone scan)
    let next_prestige = get_next_prestige_tier(game_state.prestige_rank);
    let mult_delta = next_prestige.multiplier - tier.multiplier;
    let unlock_hint = {
        let unlocks = prestige_unlock_names(next_prestige.rank);
        if unlocks.is_empty() {
            format!("\u{1f513} +{:.2}x mult", mult_delta)
        } else {
            format!(
                "\u{1f513} +{:.2}x mult \u{00b7} Unlocks: {}",
                mult_delta,
                unlocks.join(", ")
            )
        }
    };
    prestige_lines.push(Line::from(Span::styled(
        unlock_hint,
        Style::default().fg(Color::DarkGray),
    )));

    // === LAYOUT ===
    // header text (2) + XP gauge (1) + prestige text (separator + 3-4 lines) + prestige gauge (1) + separator (1) + attr gauges (3)
    let header_count = header_lines.len() as u16;
    let prestige_count = prestige_lines.len() as u16;

    let constraints = vec![
        Constraint::Length(header_count),   // header text
        Constraint::Length(1),              // XP gauge
        Constraint::Length(prestige_count), // prestige text (includes separator)
        Constraint::Length(1),              // prestige gauge
        Constraint::Length(1),              // separator
        Constraint::Length(1),              // attr row 1
        Constraint::Length(1),              // attr row 2
        Constraint::Length(1),              // attr row 3
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Render header text
    frame.render_widget(Paragraph::new(header_lines), chunks[0]);

    // XP gauge
    let xp_ratio = if xp_needed > 0 {
        (game_state.character_xp as f64 / xp_needed as f64).min(1.0)
    } else {
        0.0
    };
    let xp_label = format!(
        "XP: {}/{} ({:.1}%)",
        game_state.character_xp,
        xp_needed,
        xp_ratio * 100.0,
    );
    let xp_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(30, 28, 8))
                .add_modifier(Modifier::BOLD),
        )
        .label(xp_label)
        .ratio(xp_ratio);
    frame.render_widget(xp_gauge, chunks[1]);

    // Render prestige text
    frame.render_widget(Paragraph::new(prestige_lines), chunks[2]);

    // Prestige gauge
    let prestige_ratio =
        (game_state.character_level as f64 / next_prestige.required_level as f64).min(1.0);
    let prestige_eta = match game_state.xp_per_hour() {
        Some(rate) if rate > 0 && game_state.character_level < next_prestige.required_level => {
            let xp_remaining_current = xp_for_next_level(game_state.character_level)
                .saturating_sub(game_state.character_xp);
            let xp_future_levels: u64 = (game_state.character_level + 1
                ..next_prestige.required_level)
                .map(xp_for_next_level)
                .sum();
            let total_xp = xp_remaining_current + xp_future_levels;
            let seconds = (total_xp as f64 / rate as f64 * 3600.0) as u64;
            format!(" ({})", format_eta(seconds))
        }
        _ => String::new(),
    };
    let prestige_label = format!(
        "Lv {}/{} to {} (P{}){}",
        game_state.character_level,
        next_prestige.required_level,
        next_prestige.name,
        next_prestige.rank,
        prestige_eta
    );
    let prestige_ready = game_state.character_level >= next_prestige.required_level;
    let prestige_gauge = Gauge::default()
        .gauge_style(if prestige_ready {
            prestige_ready_gauge_style()
        } else {
            Style::default()
                .fg(Color::Rgb(180, 100, 255))
                .bg(Color::Rgb(20, 10, 30))
                .add_modifier(Modifier::BOLD)
        })
        .label(prestige_label)
        .ratio(prestige_ratio);
    frame.render_widget(prestige_gauge, chunks[3]);

    // Titled separator before attributes: ──── Attributes ────
    {
        let label = " Attributes ";
        let label_w = dw(label);
        let remaining = width.saturating_sub(label_w);
        let left_dashes = remaining / 2;
        let right_dashes = remaining - left_dashes;
        let sep = Paragraph::new(Line::from(vec![
            Span::styled(
                "\u{2500}".repeat(left_dashes),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                label,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{2500}".repeat(right_dashes),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        frame.render_widget(sep, chunks[4]);
    }

    // Attribute gauges (3 rows, 2 columns each)
    let cap = game_state.get_attribute_cap();
    let pairs = [
        (AttributeType::Strength, AttributeType::Intelligence),
        (AttributeType::Dexterity, AttributeType::Wisdom),
        (AttributeType::Constitution, AttributeType::Charisma),
    ];

    for (row_idx, (left, right)) in pairs.iter().enumerate() {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[5 + row_idx]);

        for (col_idx, at) in [left, right].iter().enumerate() {
            let val = game_state.attributes.get(**at);
            let modifier = game_state.attributes.modifier(**at);
            let mod_str = format_modifier(modifier);
            let fg = attr_color(**at);
            let bg = attr_bg_color(**at);
            let ratio = if cap > 0 {
                (val as f64 / cap as f64).min(1.0)
            } else {
                0.0
            };
            let label = format!("{} {}/{} ({})", at.abbrev(), val, cap, mod_str);
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD))
                .label(label)
                .ratio(ratio);
            frame.render_widget(gauge, cols[col_idx]);
        }
    }
}

/// Returns the accent color for a zone based on its ID.
pub(super) fn zone_color_for_id(zone_id: u32) -> Color {
    match zone_id {
        1..=2 => Color::Green,
        3..=4 => Color::Yellow,
        5..=6 => Color::Red,
        7..=8 => Color::Magenta,
        9..=10 => Color::Cyan,
        _ => Color::White,
    }
}

pub(super) fn draw_zone_info(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
    _ctx: &LayoutContext,
) {
    use super::scene_fx::{put_text_centered, put_text_centered_wrap, render_buffer, SceneCell};
    let prog = &game_state.zone_progression;

    let zone = crate::zones::get_zone(prog.current_zone_id);
    let subzone =
        crate::zones::get_subzone(prog.current_zone_id, prog.current_subzone_id).map(|(_, s)| s);

    let subzone_name = subzone.map(|s| s.name).unwrap_or("Unknown");
    let boss_name = subzone.map(|s| s.boss.name).unwrap_or("Unknown Boss");
    let total_subzones = zone.map(|z| z.subzones.len()).unwrap_or(0);

    let description_color = zone_description_tint(prog.current_zone_id);

    // No internal border — the unified right panel provides the outer frame.
    // Render directly into the given area.
    let width = area.width as usize;
    let height = area.height as usize;
    if width == 0 || height == 0 {
        return;
    }

    // --- Allocate buffer and paint dimmed zone background ---
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    super::zone_bg::paint_zone_track_bg(&mut buffer, prog.current_zone_id, 0.35);

    // --- Row 0: Subzone line (compact — zone name is in outer panel title) ---
    let badge = highest_zone_badge(achievements).unwrap_or("");
    let subzone_line = if badge.is_empty() {
        format!(
            "Subzone {}/{}: {}",
            prog.current_subzone_id, total_subzones, subzone_name
        )
    } else {
        format!(
            "{} Subzone {}/{}: {}",
            badge, prog.current_subzone_id, total_subzones, subzone_name
        )
    };
    put_text_centered(&mut buffer, 0, width, &subzone_line, Color::White);

    // --- Row 1: Boss progress ---
    if height > 1 {
        if let Some(weapon) = prog.boss_weapon_blocked(achievements) {
            let text = format!("\u{2694}\u{fe0f} BOSS: {} [Need {}!]", boss_name, weapon);
            put_text_centered(&mut buffer, 1, width, &text, Color::Yellow);
        } else if prog.fighting_boss {
            let text = format!("\u{2694}\u{fe0f} BOSS: {}", boss_name);
            put_text_centered(&mut buffer, 1, width, &text, Color::Red);
        } else {
            let kills_left = prog.kills_until_boss();
            let text = format!("[Boss in {} kills]", kills_left);
            put_text_centered(&mut buffer, 1, width, &text, Color::DarkGray);
        }
    }

    // --- Row 2+: Subzone description (wraps if too wide) ---
    if height > 2 {
        if let Some(sz) = subzone {
            put_text_centered_wrap(&mut buffer, 2, width, sz.description, description_color);
        }
    }

    render_buffer(frame, area, &buffer);

    // --- Rows 4-5: Zone progress gauge + gate hint ---
    if height > 4 {
        let fracture_cap = game_state.cached_fracture_zone_cap;
        let loom_cap = game_state.cached_loom_zone_cap;

        // Determine the highest visible zone ID (pattern/deep cap)
        let max_zone = if loom_cap >= 31 {
            loom_cap
        } else if fracture_cap >= 12 {
            fracture_cap
        } else {
            11
        };

        // Find the highest zone the player can actually enter
        // (meets prestige + ascension requirements)
        let mut highest_accessible: u32 = 0;
        for zid in 1..=max_zone {
            if prog.is_zone_unlocked(zid) {
                highest_accessible = zid;
            }
        }

        // Count completed zones (all subzone bosses defeated)
        let mut completed_zones: u32 = 0;
        for zid in 1..=max_zone {
            if let Some(z) = crate::zones::get_zone(zid) {
                if z.subzones.iter().all(|s| prog.is_boss_defeated(zid, s.id)) {
                    completed_zones += 1;
                }
            }
        }

        let ratio = if max_zone > 0 {
            (completed_zones as f64 / max_zone as f64).min(1.0)
        } else {
            0.0
        };

        // Color based on current era
        let gauge_fg = if prog.current_zone_id >= 31 {
            Color::Magenta // Loom zones
        } else if prog.current_zone_id >= 12 {
            Color::Cyan // Fracture zones
        } else {
            Color::Green // Expanse
        };
        let gauge_bg = Color::Rgb(30, 30, 30);

        let label = format!("Zone {} / {}", prog.current_zone_id, max_zone);

        let gauge_area = Rect {
            x: area.x + 1,
            y: area.y + 4,
            width: area.width.saturating_sub(2),
            height: 1,
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(gauge_fg).bg(gauge_bg))
            .label(label)
            .ratio(ratio);
        frame.render_widget(gauge, gauge_area);

        // --- Row 5: Gate hint (show first unmet requirement) ---
        if height > 5 && highest_accessible < max_zone {
            let first_gated = highest_accessible + 1;
            let gate_hint = zone_gate_hint(
                first_gated,
                game_state.prestige_rank,
                game_state.ascension_level,
            );
            if let Some(hint) = gate_hint {
                let hint_area = Rect {
                    x: area.x + 1,
                    y: area.y + 5,
                    width: area.width.saturating_sub(2),
                    height: 1,
                };
                let hint_paragraph = Paragraph::new(Line::from(Span::styled(
                    hint,
                    Style::default().fg(Color::DarkGray),
                )))
                .alignment(Alignment::Center);
                frame.render_widget(hint_paragraph, hint_area);
            }
        }
    }
}

/// Build a gate hint string for the first zone the player can't enter.
fn zone_gate_hint(zone_id: u32, prestige_rank: u32, ascension_level: u32) -> Option<String> {
    let zone = crate::zones::get_zone(zone_id)?;
    let mut reasons: Vec<String> = Vec::new();

    if prestige_rank < zone.prestige_requirement {
        reasons.push(format!("P{}", zone.prestige_requirement));
    }

    // Ascension gate for Loom zones
    let required_ascension = match zone_id {
        35..=38 => 7,
        39..=42 => 8,
        43..=46 => 9,
        47..=50 => 10,
        _ => 0,
    };
    if required_ascension > 0 && ascension_level < required_ascension {
        reasons.push(format!(
            "Asc {}",
            crate::ui::stats_prestige::to_roman(required_ascension)
        ));
    }

    if reasons.is_empty() {
        None
    } else {
        Some(format!(
            "Z{} locked \u{2014} need {}",
            zone_id,
            reasons.join(", ")
        ))
    }
}

/// Returns the icon of the highest unlocked zone completion achievement, if any.
fn highest_zone_badge(achievements: &crate::achievements::Achievements) -> Option<&'static str> {
    use crate::achievements::AchievementId;

    let zone_achievements = [
        AchievementId::BeyondInfinity,
        AchievementId::Zone10Complete,
        AchievementId::Zone9Complete,
        AchievementId::Zone8Complete,
        AchievementId::Zone7Complete,
        AchievementId::Zone6Complete,
        AchievementId::Zone5Complete,
        AchievementId::Zone4Complete,
        AchievementId::Zone3Complete,
        AchievementId::Zone2Complete,
        AchievementId::Zone1Complete,
    ];

    for id in zone_achievements {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

/// Returns a dim zone-themed tint for the subzone description text.
/// Derived from each zone's sky palette, kept very subtle (low saturation).
fn zone_description_tint(zone_id: u32) -> Color {
    let (r, g, b) = match zone_id {
        1 => (90, 120, 90),        // Meadow: soft green
        2 => (80, 70, 100),        // Dark Forest: dusky purple
        3 => (100, 105, 120),      // Mountain Pass: cool grey-blue
        4 => (95, 75, 110),        // Ancient Ruins: mystic purple
        5 => (130, 80, 60),        // Volcanic Wastes: warm ember
        6 => (90, 105, 120),       // Frozen Tundra: icy blue-grey
        7 => (80, 100, 120),       // Crystal Caverns: crystal blue
        8 => (70, 95, 115),        // Sunken Kingdom: deep aqua
        9 => (95, 115, 130),       // Floating Isles: sky blue
        10 => (100, 100, 115),     // Storm Citadel: storm grey
        11 => (85, 80, 110),       // The Expanse: void purple
        12..=14 => (120, 80, 70),  // Red Fault: warm red-brown
        15..=17 => (100, 95, 115), // Mirror Scar: cool crystal
        18..=20 => (95, 85, 80),   // Black Mouth: ashen grey
        21..=23 => (85, 90, 100),  // Hollow Throne: faded blue
        24..=26 => (100, 90, 100), // Wailing Reach: static haze
        27..=30 => (90, 75, 95),   // Origin Wound: deep wound purple
        31..=34 => (95, 110, 85),  // Thread Wilds: mossy green
        35..=38 => (110, 100, 80), // Woven Frontier: warm tan
        39..=42 => (115, 80, 90),  // The Unraveling: frayed crimson
        43..=46 => (90, 95, 110),  // Grand Design: blueprint blue
        47..=50 => (110, 95, 110), // Final Weave: cosmic purple
        _ => (100, 100, 100),      // fallback: neutral grey
    };
    Color::Rgb(r, g, b)
}

/// Returns the icon of the highest unlocked level achievement, if any.
/// Get the icon of the achievement that grants the selected title.
fn title_badge(achievements: &crate::achievements::Achievements) -> Option<&'static str> {
    let id = achievements.selected_title?;
    if !achievements.is_unlocked(id) {
        return None;
    }
    crate::achievements::titles::get_title_text(id)?;
    crate::achievements::data::get_achievement_def(id).map(|def| def.icon)
}

/// Draws a compact stats bar for M tier.
pub(super) fn draw_compact_stats_bar(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    _ctx: &LayoutContext,
    achievements: &crate::achievements::Achievements,
) {
    let tier = get_prestige_tier(game_state.prestige_rank);
    let effective_multiplier =
        DerivedStats::prestige_multiplier(tier.multiplier, &game_state.attributes);

    let prog = &game_state.zone_progression;
    let current_zone = crate::zones::get_zone(prog.current_zone_id);
    let zone_name = current_zone.map(|z| z.name).unwrap_or("???");
    let total_subzones = current_zone.map(|z| z.subzones.len()).unwrap_or(0);

    let compact_name = match achievements.selected_title.and_then(|id| {
        if achievements.is_unlocked(id) {
            crate::achievements::titles::get_title_text(id)
        } else {
            None
        }
    }) {
        Some(title) => format!(" {}, {} ", game_state.character_name, title),
        None => format!(" {} ", game_state.character_name),
    };
    let spans = vec![
        Span::styled(
            compact_name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("Lv.{}", game_state.character_level),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "P:{} {} {:.2}x",
                game_state.prestige_rank, tier.name, effective_multiplier
            ),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "Zone {}: {} ({}/{})",
                prog.current_zone_id, zone_name, prog.current_subzone_id, total_subzones
            ),
            Style::default().fg(Color::Green),
        ),
    ];

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

/// Draws all 6 attributes on a single line for M tier.
pub(super) fn draw_attributes_single_line(frame: &mut Frame, area: Rect, game_state: &GameState) {
    super::stats_attributes::draw_attributes_single_line(frame, area, game_state);
}

/// Draws a compact XP bar for M/S tier (borderless, single line).
pub(super) fn draw_xp_bar_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let xp_needed = xp_for_next_level(game_state.character_level);
    let xp_ratio = if xp_needed > 0 {
        (game_state.character_xp as f64 / xp_needed as f64).min(1.0)
    } else {
        0.0
    };

    let rate_suffix = match game_state.xp_per_hour() {
        Some(rate) => {
            let xp_remaining = xp_needed.saturating_sub(game_state.character_xp);
            let eta = if rate > 0 {
                let seconds = (xp_remaining as f64 / rate as f64 * 3600.0) as u64;
                format!(" ({})", format_eta(seconds))
            } else {
                String::new()
            };
            format!(
                " | {}/hr{}",
                super::game_common::format_number_short(rate),
                eta
            )
        }
        None => String::new(),
    };
    let xp_label = format!(
        "XP: {}/{} ({:.1}%){}",
        game_state.character_xp,
        xp_needed,
        xp_ratio * 100.0,
        rate_suffix
    );

    let xp_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(30, 28, 8))
                .add_modifier(Modifier::BOLD),
        )
        .label(xp_label)
        .ratio(xp_ratio);

    frame.render_widget(xp_gauge, area);
}

/// Draws a compact footer for M tier.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_footer_compact(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
    deep_indicator: DeepIndicatorStatus,
    loom_discovered: bool,
    pending_achievements: usize,
) {
    use crate::character::prestige::can_prestige;

    let can_prestige_now = can_prestige(game_state);
    let prestige_span = if can_prestige_now {
        Span::styled(
            "[P]Prestige!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("[P]Prestige", Style::default().fg(Color::DarkGray))
    };

    let haven_span = if haven_discovered {
        Span::styled(" [H]Haven", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    let soulforge_span = if soulforge_discovered {
        Span::styled(" [S]oulforge", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let stormglass_span = if stormglass_discovered {
        Span::styled(" [G]lass", Style::default().fg(Color::Rgb(100, 180, 255)))
    } else {
        Span::raw("")
    };

    let deep_span = match deep_indicator {
        DeepIndicatorStatus::Hidden => Span::raw(""),
        DeepIndicatorStatus::Idle => Span::styled(" [D]eep", Style::default().fg(Color::DarkGray)),
        DeepIndicatorStatus::Running => Span::styled(
            " [D]eep\u{25cf}",
            Style::default().fg(Color::Rgb(80, 160, 220)),
        ),
        DeepIndicatorStatus::EventPending => Span::styled(
            " [D]eep\u{26a1}",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        DeepIndicatorStatus::Completed => {
            Span::styled(" [D]eep\u{2713}", Style::default().fg(Color::Green))
        }
    };

    let loom_span = if loom_discovered {
        Span::styled(" [L]Loom", Style::default().fg(Color::Rgb(180, 120, 220)))
    } else {
        Span::raw("")
    };

    let ach_span = if pending_achievements > 0 {
        Span::styled(
            format!(" [A]Ach({})", pending_achievements),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" [A]Ach", Style::default().fg(Color::Magenta))
    };

    let controls = Line::from(vec![
        prestige_span,
        haven_span,
        soulforge_span,
        stormglass_span,
        deep_span,
        loom_span,
        ach_span,
        Span::styled(" [T]Vault", Style::default().fg(Color::Cyan)),
        Span::styled(" [W]Wiki", Style::default().fg(Color::DarkGray)),
        Span::styled(" [!]Bug", Style::default().fg(Color::DarkGray)),
    ]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(area);

    let esc = Paragraph::new(Line::from(Span::styled(
        "[Esc]Quit",
        Style::default().fg(Color::Red),
    )));
    frame.render_widget(esc, chunks[0]);
    frame.render_widget(
        Paragraph::new(controls).alignment(Alignment::Center),
        chunks[1],
    );
}

/// Draws a minimal footer for S tier.
pub(super) fn draw_footer_minimal(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use crate::character::prestige::can_prestige;

    let can_prestige_now = can_prestige(game_state);
    let prestige_span = if can_prestige_now {
        Span::styled(
            " P:Prestige!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" P:Prestige", Style::default().fg(Color::DarkGray))
    };

    let controls = Line::from(vec![
        prestige_span,
        Span::styled(" !:Bug", Style::default().fg(Color::DarkGray)),
    ]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    let esc = Paragraph::new(Line::from(Span::styled(
        "Esc:Quit",
        Style::default().fg(Color::Red),
    )));
    frame.render_widget(esc, chunks[0]);
    frame.render_widget(
        Paragraph::new(controls).alignment(Alignment::Center),
        chunks[1],
    );
}

/// Formats play time as "Xmo Xw Xd Xh Xm Xs"
fn format_play_time(total_seconds: u64) -> String {
    const SECONDS_PER_MINUTE: u64 = 60;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_DAY: u64 = 86400;
    const SECONDS_PER_WEEK: u64 = 604800;
    const SECONDS_PER_MONTH: u64 = 2592000; // 30 days

    let months = total_seconds / SECONDS_PER_MONTH;
    let weeks = (total_seconds % SECONDS_PER_MONTH) / SECONDS_PER_WEEK;
    let days = (total_seconds % SECONDS_PER_WEEK) / SECONDS_PER_DAY;
    let hours = (total_seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR;
    let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = total_seconds % SECONDS_PER_MINUTE;

    if months > 0 {
        format!(
            "{}mo {}w {}d {}h {}m {}s",
            months, weeks, days, hours, minutes, seconds
        )
    } else if weeks > 0 {
        format!("{}w {}d {}h {}m {}s", weeks, days, hours, minutes, seconds)
    } else if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Draws the footer with control instructions and version info
#[allow(clippy::too_many_arguments)]
pub fn draw_footer(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    update_check_completed: bool,
    update_check_failed: bool,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
    deep_indicator: DeepIndicatorStatus,
    loom_discovered: bool,
    show_ascend: bool,
    pending_achievements: usize,
    _ctx: &LayoutContext,
) {
    use crate::character::prestige::can_prestige;
    use crate::utils::build_info::BUILD_COMMIT;

    let version_title = format!(" {} ", BUILD_COMMIT);

    let can_prestige_now = can_prestige(game_state);
    let prestige_text = if can_prestige_now {
        Span::styled(
            "[P] Prestige (Available!)",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )
    } else {
        let next_tier = get_prestige_tier(game_state.prestige_rank + 1);
        Span::styled(
            format!("[P] Prestige (Need Lv.{})", next_tier.required_level),
            Style::default().fg(Color::DarkGray),
        )
    };

    let update_status_text = if let Some(info) = update_info {
        let millis = super::clock::now_millis();
        // Sine-wave pulse between dim (60,50,0) and bright yellow (255,215,0)
        // Full cycle ~3.2 seconds for a breathing fade in/out
        let phase = (millis % 3200) as f64 / 3200.0 * std::f64::consts::TAU;
        let t = (phase.sin() + 1.0) / 2.0; // 0.0 to 1.0
        let r = (60.0 + t * 195.0) as u8;
        let g = (50.0 + t * 165.0) as u8;
        let b = 0;
        let color = Color::Rgb(r, g, b);
        Span::styled(
            format!(
                "    \u{2191} Update: {}",
                &info.new_commit[..7.min(info.new_commit.len())]
            ),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )
    } else if !update_check_completed {
        let spinner = super::throbber::spinner_char();
        Span::styled(
            format!("    {} Checking...", spinner),
            Style::default().fg(Color::DarkGray),
        )
    } else if update_check_failed {
        Span::raw("")
    } else {
        Span::styled("    \u{2713} Latest", Style::default().fg(Color::Green))
    };

    let haven_text = if haven_discovered {
        Span::styled("    [H] Haven", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    let soulforge_text = if soulforge_discovered {
        Span::styled("    [S] Soulforge", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let stormglass_text = if stormglass_discovered {
        Span::styled(
            "    [G] Stormglass Exchange",
            Style::default().fg(Color::Rgb(100, 180, 255)),
        )
    } else {
        Span::raw("")
    };

    let deep_text = match deep_indicator {
        DeepIndicatorStatus::Hidden => Span::raw(""),
        DeepIndicatorStatus::Idle => {
            Span::styled("    [D] The Deep", Style::default().fg(Color::DarkGray))
        }
        DeepIndicatorStatus::Running => Span::styled(
            "    [D] The Deep \u{25cf}",
            Style::default().fg(Color::Rgb(80, 160, 220)),
        ),
        DeepIndicatorStatus::EventPending => Span::styled(
            "    [D] The Deep \u{26a1}",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        DeepIndicatorStatus::Completed => Span::styled(
            "    [D] The Deep \u{2713}",
            Style::default().fg(Color::Green),
        ),
    };

    let loom_text = if loom_discovered {
        Span::styled(
            "    [L] Loom",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )
    } else {
        Span::raw("")
    };

    let achievements_text = if pending_achievements > 0 {
        Span::styled(
            format!("[A] Achievements (\u{1f3c6} {} new!)", pending_achievements),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("[A] Achievements", Style::default().fg(Color::Magenta))
    };

    let ascend_text = if show_ascend {
        Span::styled(
            "    [U] Ascend",
            Style::default().fg(Color::Rgb(255, 215, 0)),
        )
    } else {
        Span::raw("")
    };

    let footer_text = vec![
        Line::from(vec![
            prestige_text,
            haven_text,
            soulforge_text,
            stormglass_text,
            deep_text,
            loom_text,
            update_status_text,
        ]),
        Line::from(vec![
            achievements_text,
            Span::styled("    [T] Time Vault", Style::default().fg(Color::Cyan)),
            ascend_text,
            Span::styled("    [W] Wiki", Style::default().fg(Color::DarkGray)),
            Span::styled("    [!] Bug", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let footer_block =
        super::themed_block(Block::default().borders(Borders::ALL).title(version_title));
    let footer_inner = footer_block.inner(area);
    frame.render_widget(footer_block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    // Split inner: [Esc] bottom-left, controls centered
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(footer_inner);

    let esc_text = Paragraph::new(Line::from(Span::styled(
        "[Esc] Quit",
        Style::default().fg(Color::Red),
    )));
    frame.render_widget(esc_text, footer_chunks[0]);

    let controls = Paragraph::new(footer_text).alignment(Alignment::Center);
    frame.render_widget(controls, footer_chunks[1]);
}
