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

/// Draws the stats panel
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
            let mut constraints = vec![
                Constraint::Length(4), // Header
                Constraint::Length(5), // Prestige
                Constraint::Length(4), // Fishing
                Constraint::Length(5), // Attributes
            ];
            if etched > 0 {
                constraints.push(Constraint::Length(etched as u16 + 2)); // Sigils
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

    let header_title = match highest_level_badge(achievements) {
        Some(icon) => format!(" {} {} ", game_state.character_name, icon),
        None => format!(" {} ", game_state.character_name),
    };
    let header_block = Block::default().borders(Borders::ALL).title(header_title);
    let header_block = super::themed_block(header_block);
    let inner = header_block.inner(area);
    frame.render_widget(header_block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let header_text = vec![Line::from(vec![
        Span::styled(
            format!("Level {} {}", game_state.character_level, rank),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("\u{23f1}\u{fe0f} ", Style::default().fg(Color::Cyan)),
        Span::styled(play_time, Style::default().fg(Color::Cyan)),
    ])];

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

    if inner.height >= 2 {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let header_paragraph = Paragraph::new(header_text);
        frame.render_widget(header_paragraph, inner_chunks[0]);
        frame.render_widget(xp_gauge, inner_chunks[1]);
    } else if inner.height == 1 {
        let header_paragraph = Paragraph::new(header_text);
        frame.render_widget(header_paragraph, inner);
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

    let mut bar_spans: Vec<Span> = Vec::new();
    let mut label_spans: Vec<Span> = Vec::new();

    for zid in 1..=11u32 {
        if zid > 1 {
            bar_spans.push(Span::raw(" "));
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

        let filled = if is_completed {
            3
        } else if defeated_count == 0 {
            0
        } else {
            ((defeated_count as f64 / num_subzones as f64) * 3.0).ceil() as usize
        }
        .min(3);

        let (fill_char, empty_char, fg) = if is_completed {
            ("\u{2588}", "\u{2588}", Color::Green)
        } else if is_current {
            ("\u{2588}", "\u{2591}", Color::Yellow)
        } else if is_unlocked {
            ("\u{2591}", "\u{2591}", Color::White)
        } else {
            ("\u{2591}", "\u{2591}", Color::DarkGray)
        };

        let segment: String = fill_char.repeat(filled) + &empty_char.repeat(3 - filled);
        let segment_style = if is_current {
            Style::default().fg(fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg)
        };
        bar_spans.push(Span::styled(segment, segment_style));

        let label_fg = if is_current {
            Color::Yellow
        } else if is_completed {
            Color::Green
        } else if is_unlocked {
            Color::White
        } else {
            Color::DarkGray
        };
        if zid > 1 {
            let sep = if zid == 10 { "  " } else { " " };
            label_spans.push(Span::raw(sep));
        }
        let label = format!("{:^3}", zid);
        label_spans.push(Span::styled(label, Style::default().fg(label_fg)));
    }

    zone_lines.push(Line::from(""));
    zone_lines.push(Line::from(bar_spans));
    zone_lines.push(Line::from(label_spans));

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
fn highest_level_badge(achievements: &crate::achievements::Achievements) -> Option<&'static str> {
    use crate::achievements::AchievementId;

    let level_achievements = [
        AchievementId::Level1500,
        AchievementId::Level1000,
        AchievementId::Level750,
        AchievementId::Level500,
        AchievementId::Level250,
        AchievementId::Level200,
        AchievementId::Level150,
        AchievementId::Level100,
        AchievementId::Level50,
        AchievementId::Level25,
        AchievementId::Level10,
    ];

    for id in level_achievements {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

/// Draws a compact stats bar for M tier.
pub(super) fn draw_compact_stats_bar(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    _ctx: &LayoutContext,
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

    let spans = vec![
        Span::styled(
            format!(" {} ", game_state.character_name),
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
pub(super) fn draw_footer_compact(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
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

    let challenge_count = game_state.challenge_menu.challenges.len();
    let challenge_span = if challenge_count > 0 {
        Span::styled(
            format!(" [Tab]Chall({})", challenge_count),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled("[Esc]Quit", Style::default().fg(Color::Red)),
        Span::raw(" "),
        prestige_span,
        haven_span,
        soulforge_span,
        stormglass_span,
        ach_span,
        challenge_span,
        Span::styled(" [?]Help", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
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

    let line = Line::from(vec![
        Span::styled("Esc:Quit", Style::default().fg(Color::Red)),
        prestige_span,
        Span::styled(" ?:Help", Style::default().fg(Color::DarkGray)),
        Span::styled(" Tab:More", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
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

/// Draws the update drawer panel when expanded
pub fn draw_update_drawer(frame: &mut Frame, area: Rect, info: &UpdateInfo) {
    let mut lines = vec![
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("  New Version: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("v{}", info.new_version),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", info.new_commit),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "  What's New:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let max_items = 5;
    for item in info.changelog.iter().take(max_items) {
        lines.push(Line::from(vec![
            Span::styled("    \u{2022} ", Style::default().fg(Color::DarkGray)),
            Span::styled(item.clone(), Style::default().fg(Color::White)),
        ]));
    }

    if info.changelog_total > max_items {
        lines.push(Line::from(vec![Span::styled(
            format!("    (+{} more changes)", info.changelog_total - max_items),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    lines.push(Line::from(vec![]));
    lines.push(Line::from(vec![Span::styled(
        "  Run 'quest update' to install",
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  Wiki: {}", crate::core::constants::WIKI_URL),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("                              "),
        Span::styled("[U] Close", Style::default().fg(Color::Yellow)),
    ]));

    let drawer = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(super::themed_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)))
                .title(Span::styled(
                    " \u{1f195} Update Available ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
        ));

    frame.render_widget(drawer, area);
    super::apply_themed_border_fx(frame, area, Color::Yellow, super::BorderFxContext);
}

/// Draws the footer with control instructions and version info
#[allow(clippy::too_many_arguments)]
pub fn draw_footer(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    _update_expanded: bool,
    update_check_completed: bool,
    update_check_failed: bool,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
    pending_achievements: usize,
    _ctx: &LayoutContext,
) {
    use crate::character::prestige::can_prestige;
    use crate::utils::build_info::{BUILD_COMMIT, BUILD_DATE};

    let version_title = format!(" v{} ({}) ", BUILD_DATE, BUILD_COMMIT);

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
        Span::styled(
            format!("    \u{1f195} [U] Update (v{})", info.new_version),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if !update_check_completed {
        use super::throbber::spinner_char;
        Span::styled(
            format!("    {} Checking...", spinner_char()),
            Style::default().fg(Color::DarkGray),
        )
    } else if update_check_failed {
        Span::styled(
            "    ⚠ Update check failed",
            Style::default().fg(Color::LightRed),
        )
    } else {
        Span::styled(
            "    \u{2713} On latest version",
            Style::default().fg(Color::Green),
        )
    };

    let challenge_count = game_state.challenge_menu.challenges.len();
    let challenge_text = if challenge_count > 0 {
        Span::styled(
            format!("    [Tab] Challenges ({})", challenge_count),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
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

    let footer_text = vec![
        Line::from(vec![
            Span::styled("[Esc] Quit", Style::default().fg(Color::Red)),
            Span::raw("    "),
            prestige_text,
            haven_text,
            soulforge_text,
            stormglass_text,
        ]),
        Line::from(vec![
            achievements_text,
            challenge_text,
            update_status_text,
            Span::styled("    [?] Help", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .block(super::themed_block(
            Block::default().borders(Borders::ALL).title(version_title),
        ))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);
}
