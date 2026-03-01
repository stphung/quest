//! Stats panel coordinator — delegates to submodules for rendering.

use super::responsive::{LayoutContext, SizeTier};
use super::stats_attributes::draw_attributes_compact;
use super::stats_equipment::draw_equipment_names_only;
use super::stats_prestige::{draw_fishing_panel, draw_prestige_info, format_eta};
use super::stats_sigils::draw_sigils_panel;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{get_adventurer_rank, get_prestige_tier};
use crate::core::game_logic::xp_for_next_level;
use crate::core::game_state::GameState;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

// Re-export for haven_scene.rs which uses super::stats_panel::enhancement_style
pub(super) use super::stats_equipment::enhancement_style;

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
) {
    match ctx.height_tier {
        SizeTier::XL | SizeTier::L => {
            let etched = game_state.storm_sigils.etched_count();
            let prestige_height = if game_state.ascension_level > 0 {
                7 // 5 inner rows + 2 border
            } else {
                6 // 4 inner rows + 2 border
            };
            let mut constraints = vec![
                Constraint::Length(5), // Header (name, level+power, time+rate, XP gauge)
                Constraint::Length(prestige_height), // Prestige
                Constraint::Length(4), // Fishing
                Constraint::Length(5), // Attributes
            ];
            if etched > 0 {
                constraints.push(Constraint::Length(
                    crate::stormglass::sigils::MAX_SIGIL_SLOTS as u16 + 2,
                )); // Sigils (all 5 slots)
            }
            constraints.push(Constraint::Min(0)); // Equipment

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            let mut idx = 0;
            draw_header(frame, chunks[idx], game_state, achievements);
            idx += 1;
            draw_prestige_info(frame, chunks[idx], game_state, achievements);
            idx += 1;
            draw_fishing_panel(frame, chunks[idx], game_state, achievements);
            idx += 1;
            draw_attributes_compact(frame, chunks[idx], game_state);
            idx += 1;
            if etched > 0 {
                draw_sigils_panel(frame, chunks[idx], &game_state.storm_sigils);
                idx += 1;
            }
            draw_equipment_names_only(frame, chunks[idx], game_state, enhancement_levels);
        }
        _ => {}
    }
}

/// Draws the header with character level, XP bar, and play time
fn draw_header(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    let xp_needed = xp_for_next_level(game_state.character_level);
    let xp_ratio = if xp_needed > 0 {
        (game_state.character_xp as f64 / xp_needed as f64).min(1.0)
    } else {
        0.0
    };

    let rank = get_adventurer_rank(game_state.character_level);
    let play_time = format_play_time(game_state.play_time_seconds);

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
    let header_block = Block::default().borders(Borders::ALL).title(header_title);
    let header_block = super::themed_block(header_block);
    let inner = header_block.inner(area);
    frame.render_widget(header_block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    // Row 1: Level + Power rating (right-aligned)
    let power = game_state.cached_power_rating as u64;
    let power_str = format!(
        "\u{2694} Power: {}",
        super::game_common::format_number_short(power)
    );
    let level_str = format!("Level {} {}", game_state.character_level, rank);
    // Pad power to right-align within inner width
    let gap = (inner.width as usize).saturating_sub(level_str.len() + power_str.len());
    let row1 = Line::from(vec![
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
    ]);

    // Row 2: Play time + XP rate
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
    let row2 = Line::from(vec![
        Span::styled("\u{23f1}\u{fe0f} ", Style::default().fg(Color::Cyan)),
        Span::styled(play_time, Style::default().fg(Color::Cyan)),
        Span::styled(rate_suffix, Style::default().fg(Color::Cyan)),
    ]);

    // Row 3: XP gauge
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
                .add_modifier(Modifier::BOLD),
        )
        .label(xp_label)
        .ratio(xp_ratio);

    if inner.height >= 3 {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        frame.render_widget(Paragraph::new(vec![row1]), inner_chunks[0]);
        frame.render_widget(Paragraph::new(vec![row2]), inner_chunks[1]);
        frame.render_widget(xp_gauge, inner_chunks[2]);
    } else if inner.height >= 1 {
        frame.render_widget(Paragraph::new(vec![row1]), inner);
    }
}

pub(super) fn draw_zone_info(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
    _ctx: &LayoutContext,
) {
    use crate::zones::get_all_zones;

    let zones = get_all_zones();
    let prog = &game_state.zone_progression;

    let zone = zones.iter().find(|z| z.id == prog.current_zone_id);
    let subzone = zone.and_then(|z| z.subzones.iter().find(|s| s.id == prog.current_subzone_id));

    let zone_name = zone.map(|z| z.name).unwrap_or("Unknown");
    let subzone_name = subzone.map(|s| s.name).unwrap_or("Unknown");
    let boss_name = subzone.map(|s| s.boss.name).unwrap_or("Unknown Boss");
    let total_subzones = zone.map(|z| z.subzones.len()).unwrap_or(0);

    let zone_color = match prog.current_zone_id {
        1..=2 => Color::Green,
        3..=4 => Color::Yellow,
        5..=6 => Color::Red,
        7..=8 => Color::Magenta,
        9..=10 => Color::Cyan,
        _ => Color::White,
    };

    let boss_progress = if let Some(weapon) = prog.boss_weapon_blocked(achievements) {
        Span::styled(
            format!(" \u{2694}\u{fe0f} BOSS: {} [Need {}!] ", boss_name, weapon),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )
    } else if prog.fighting_boss {
        Span::styled(
            format!(" \u{2694}\u{fe0f} BOSS: {} ", boss_name),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )
    } else {
        let kills_left = prog.kills_until_boss();
        Span::styled(
            format!(" [Boss in {} kills]", kills_left),
            Style::default().fg(Color::DarkGray),
        )
    };

    let mut zone_lines = vec![Line::from(vec![
        Span::styled(
            format!("Zone {}: ", prog.current_zone_id),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            zone_name,
            Style::default().fg(zone_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(subzone_name, Style::default().fg(Color::White)),
        Span::styled(
            format!(" ({}/{})", prog.current_subzone_id, total_subzones),
            Style::default().fg(Color::DarkGray),
        ),
    ])];
    zone_lines.push(Line::from(boss_progress));

    if let Some(sz) = subzone {
        zone_lines.push(Line::from(vec![Span::styled(
            sz.description,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    }

    // Dot track: ● = completed (green), ○ = current (yellow), · = unlocked (white), · = locked (gray)
    // Chapter separators │ between base zones, zone 11, and each fracture chapter.
    // Second row shows zone range labels aligned under each group.
    let max_fracture_zone = (12..=30u32)
        .rev()
        .find(|&zid| prog.is_zone_unlocked(zid))
        .unwrap_or(0);

    // Define zone groups: (start, end) inclusive
    let mut groups: Vec<(u32, u32)> = vec![(1, 11)];
    if max_fracture_zone >= 12 {
        // Fracture chapter boundaries
        let chapter_starts: &[u32] = &[12, 15, 18, 21, 24, 27];
        let chapter_ends: &[u32] = &[14, 17, 20, 23, 26, 30];
        for (&start, &end) in chapter_starts.iter().zip(chapter_ends.iter()) {
            if max_fracture_zone >= start {
                groups.push((start, end.min(max_fracture_zone)));
            }
        }
    }

    let mut dot_spans: Vec<Span> = Vec::new();
    let mut label_parts: Vec<(usize, String)> = Vec::new(); // (char_offset, label)
    let mut char_pos: usize = 0;

    for (gi, &(g_start, g_end)) in groups.iter().enumerate() {
        // Separator between groups
        if gi > 0 {
            dot_spans.push(Span::styled(
                " \u{2502} ",
                Style::default().fg(Color::DarkGray),
            ));
            char_pos += 3; // " │ " is 3 chars
        }

        let group_start_pos = char_pos;

        for zid in g_start..=g_end {
            if zid > g_start {
                dot_spans.push(Span::raw(" "));
                char_pos += 1;
            }

            let zone_data = zones.iter().find(|z| z.id == zid);
            let num_subzones = zone_data.map(|z| z.subzones.len()).unwrap_or(3);
            let defeated_count = zone_data
                .map(|z| {
                    z.subzones
                        .iter()
                        .filter(|s| prog.is_boss_defeated(zid, s.id))
                        .count()
                })
                .unwrap_or(0);

            let is_current = zid == prog.current_zone_id;
            let is_completed = defeated_count == num_subzones;
            let is_unlocked = if zid == 11 {
                achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd)
            } else {
                prog.is_zone_unlocked(zid)
            };

            let (dot, fg, bold) = if is_current {
                ("\u{25cb}", Color::Yellow, true) // ○
            } else if is_completed {
                ("\u{25cf}", Color::Green, false) // ●
            } else if is_unlocked {
                ("\u{00b7}", Color::White, false) // ·
            } else {
                ("\u{00b7}", Color::DarkGray, false) // ·
            };

            let style = if bold {
                Style::default().fg(fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg)
            };
            dot_spans.push(Span::styled(dot, style));
            char_pos += 1; // each dot is 1 char wide
        }

        // Build label for this group
        let label = if g_start == g_end {
            format!("{}", g_start)
        } else {
            format!("{}-{}", g_start, g_end)
        };
        let group_width = char_pos - group_start_pos;
        label_parts.push((group_start_pos, center_label(&label, group_width)));
    }

    // Build the label line by placing each label at its offset
    let total_width = char_pos;
    let mut label_chars: Vec<u8> = vec![b' '; total_width];
    for (offset, label) in &label_parts {
        for (i, ch) in label.bytes().enumerate() {
            let pos = offset + i;
            if pos < total_width {
                label_chars[pos] = ch;
            }
        }
    }
    let label_str = String::from_utf8(label_chars).unwrap_or_default();

    zone_lines.push(Line::from(""));
    zone_lines.push(Line::from(dot_spans));
    zone_lines.push(Line::from(Span::styled(
        label_str,
        Style::default().fg(Color::DarkGray),
    )));

    let location_title = match highest_zone_badge(achievements) {
        Some(icon) => format!(" Location {} ", icon),
        None => " Location ".to_string(),
    };
    let zone_widget = Paragraph::new(zone_lines)
        .block(super::themed_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(super::themed_border_color(zone_color)))
                .title(location_title),
        ))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(zone_widget, area);
    super::apply_themed_border_fx(frame, area, zone_color, super::BorderFxContext);
}

/// Center a label within a given width, padding with spaces.
fn center_label(label: &str, width: usize) -> String {
    if label.len() >= width {
        return label[..width].to_string();
    }
    let pad = (width - label.len()) / 2;
    format!("{:>width$}", label, width = pad + label.len())
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
    use crate::zones::get_all_zones;

    let tier = get_prestige_tier(game_state.prestige_rank);
    let effective_multiplier =
        DerivedStats::prestige_multiplier(tier.multiplier, &game_state.attributes);

    let zones = get_all_zones();
    let prog = &game_state.zone_progression;
    let zone_name = zones
        .iter()
        .find(|z| z.id == prog.current_zone_id)
        .map(|z| z.name)
        .unwrap_or("???");
    let total_subzones = zones
        .iter()
        .find(|z| z.id == prog.current_zone_id)
        .map(|z| z.subzones.len())
        .unwrap_or(0);

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
        use std::time::{SystemTime, UNIX_EPOCH};
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
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

    let ascend_text = if !matches!(deep_indicator, DeepIndicatorStatus::Hidden) {
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
