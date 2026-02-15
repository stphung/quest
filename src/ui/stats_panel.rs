use super::responsive::{LayoutContext, SizeTier};
use crate::character::attributes::AttributeType;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{get_adventurer_rank, get_prestige_tier};
use crate::core::game_logic::xp_for_next_level;
use crate::core::game_state::GameState;
use crate::fishing::types::FishingState;
use crate::items::types::Rarity;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

/// Draws the stats panel
pub fn draw_stats_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    ctx: &LayoutContext,
    enhancement_levels: &[u8; 7],
) {
    match ctx.height_tier {
        SizeTier::XL | SizeTier::L => {
            // header(4) + prestige(5) + fishing(4) + attrs_compact(5) + equip_names(rest)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Length(5),
                    Constraint::Length(4),
                    Constraint::Length(5), // 3 pairs + 2 borders
                    Constraint::Min(0),
                ])
                .split(area);

            draw_header(frame, chunks[0], game_state);
            draw_prestige_info(frame, chunks[1], game_state);
            draw_fishing_panel(frame, chunks[2], game_state);
            draw_attributes_compact(frame, chunks[3], game_state);
            draw_equipment_names_only(frame, chunks[4], game_state, enhancement_levels);
        }
        _ => {
            // M and S don't use stats panel (handled by stacked layout in Phase 3)
        }
    }
}

/// Draws the header with character level, XP bar, and play time
fn draw_header(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let xp_needed = xp_for_next_level(game_state.character_level);
    let xp_ratio = if xp_needed > 0 {
        (game_state.character_xp as f64 / xp_needed as f64).min(1.0)
    } else {
        0.0
    };

    let rank = get_adventurer_rank(game_state.character_level);
    let play_time = format_play_time(game_state.play_time_seconds);

    // Create block and get inner area
    let header_block = Block::default()
        .borders(Borders::ALL)
        .title(game_state.character_name.as_str());
    let inner = header_block.inner(area);
    frame.render_widget(header_block, area);

    // Header text line
    let header_text = vec![Line::from(vec![
        Span::styled(
            format!("Level {} {}", game_state.character_level, rank),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("⏱️ ", Style::default().fg(Color::Cyan)),
        Span::styled(play_time, Style::default().fg(Color::Cyan)),
    ])];

    // XP progress bar
    let xp_label = format!(
        "XP: {}/{} ({:.1}%)",
        game_state.character_xp,
        xp_needed,
        xp_ratio * 100.0
    );

    let xp_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .label(xp_label)
        .ratio(xp_ratio);

    // Render based on available height
    if inner.height >= 2 {
        // Split inner area: header text + XP bar
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let header_paragraph = Paragraph::new(header_text);
        frame.render_widget(header_paragraph, inner_chunks[0]);
        frame.render_widget(xp_gauge, inner_chunks[1]);
    } else if inner.height == 1 {
        // Only room for one line — show level
        let header_paragraph = Paragraph::new(header_text);
        frame.render_widget(header_paragraph, inner);
    }
}

/// Draws the current zone and subzone info
/// Zone completion status for the second line of the zone info panel.
pub(super) enum ZoneCompletionStatus {
    /// Zone complete but next zone requires higher prestige
    Gated {
        next_zone_id: u32,
        next_zone_name: &'static str,
        required_prestige: u32,
    },
    /// Zone complete and player meets the requirement
    Unlocked {
        next_zone_id: u32,
        next_zone_name: &'static str,
        required_prestige: u32,
    },
    /// No next zone — unknown territory
    Mystery,
}

pub(super) fn compute_zone_completion(game_state: &GameState) -> ZoneCompletionStatus {
    use crate::zones::get_all_zones;

    let zones = get_all_zones();
    let prog = &game_state.zone_progression;

    // Always show next zone status when a next zone exists
    match zones.iter().find(|z| z.id == prog.current_zone_id + 1) {
        Some(next) if next.prestige_requirement > game_state.prestige_rank => {
            ZoneCompletionStatus::Gated {
                next_zone_id: next.id,
                next_zone_name: next.name,
                required_prestige: next.prestige_requirement,
            }
        }
        Some(next) => ZoneCompletionStatus::Unlocked {
            next_zone_id: next.id,
            next_zone_name: next.name,
            required_prestige: next.prestige_requirement,
        },
        None => ZoneCompletionStatus::Mystery,
    }
}

pub(super) fn draw_zone_info(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    zone_completion: &ZoneCompletionStatus,
    achievements: &crate::achievements::Achievements,
    _ctx: &LayoutContext,
) {
    use crate::zones::get_all_zones;

    let zones = get_all_zones();
    let prog = &game_state.zone_progression;

    // Get current zone and subzone info
    let zone = zones.iter().find(|z| z.id == prog.current_zone_id);
    let subzone = zone.and_then(|z| z.subzones.iter().find(|s| s.id == prog.current_subzone_id));

    let zone_name = zone.map(|z| z.name).unwrap_or("Unknown");
    let subzone_name = subzone.map(|s| s.name).unwrap_or("Unknown");
    let boss_name = subzone.map(|s| s.boss.name).unwrap_or("Unknown Boss");
    let total_subzones = zone.map(|z| z.subzones.len()).unwrap_or(0);

    // Color based on zone tier
    let zone_color = match prog.current_zone_id {
        1..=2 => Color::Green,   // Tier 1
        3..=4 => Color::Yellow,  // Tier 2
        5..=6 => Color::Red,     // Tier 3
        7..=8 => Color::Magenta, // Tier 4
        9..=10 => Color::Cyan,   // Tier 5
        _ => Color::White,
    };

    // Build the boss progress display — always show boss status
    let boss_progress = if let Some(weapon) = prog.boss_weapon_blocked(achievements) {
        Span::styled(
            format!(" ⚔️ BOSS: {} [Need {}!] ", boss_name, weapon),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )
    } else if prog.fighting_boss {
        Span::styled(
            format!(" ⚔️ BOSS: {} ", boss_name),
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
        boss_progress,
    ])];

    // Add subzone flavor text
    if let Some(sz) = subzone {
        zone_lines.push(Line::from(vec![Span::styled(
            sz.description,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    }

    // Blank line separator between flavor text and next-zone status
    zone_lines.push(Line::from(""));

    // Add second line based on completion status
    match zone_completion {
        ZoneCompletionStatus::Gated {
            next_zone_id,
            next_zone_name,
            required_prestige,
        } => {
            zone_lines.push(Line::from(vec![
                Span::styled("🔒 Next: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("Zone {}: {}", next_zone_id, next_zone_name),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" — requires Prestige {}", required_prestige),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
        }
        ZoneCompletionStatus::Unlocked {
            next_zone_id,
            next_zone_name,
            required_prestige,
        } => {
            let line = if *required_prestige == 0 {
                // No prestige needed — just show the arrow
                Line::from(vec![
                    Span::styled("➡ Next: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("Zone {}: {}", next_zone_id, next_zone_name),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("✅ Next: ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("Zone {}: {}", next_zone_id, next_zone_name),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" — Prestige {} requirement met!", required_prestige),
                        Style::default().fg(Color::Green),
                    ),
                ])
            };
            zone_lines.push(line);
        }
        ZoneCompletionStatus::Mystery => {
            zone_lines.push(Line::from(vec![Span::styled(
                "❓ Next: Zone ???: ????????",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            )]));
        }
    }

    let zone_widget = Paragraph::new(zone_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(zone_color))
                .title("Location"),
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(zone_widget, area);
}

/// Draws prestige information with CHA bonus
fn draw_prestige_info(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let prestige_block = Block::default().borders(Borders::ALL).title("Prestige");

    let inner = prestige_block.inner(area);
    frame.render_widget(prestige_block, area);

    let tier = get_prestige_tier(game_state.prestige_rank);
    let cha_mod = game_state.attributes.modifier(AttributeType::Charisma);
    let effective_multiplier =
        DerivedStats::prestige_multiplier(tier.multiplier, &game_state.attributes);

    let prestige_text = vec![
        Line::from(vec![
            Span::styled("🏆 Rank: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} ({})", game_state.prestige_rank, tier.name),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("⚡ Mult: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:.2}x", tier.multiplier),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" + "),
            Span::styled(
                format!("{:.2}x (CHA)", cha_mod as f64 * 0.1),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" = "),
            Span::styled(
                format!("{:.2}x", effective_multiplier),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("🔄 Resets: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", game_state.total_prestige_count),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    // Show as many lines as fit, rank first
    let lines_to_show = inner.height as usize;
    let truncated: Vec<Line> = prestige_text.into_iter().take(lines_to_show).collect();
    let prestige_paragraph = Paragraph::new(truncated);
    frame.render_widget(prestige_paragraph, inner);
}

/// Draws the fishing panel with rank and progress bar.
fn draw_fishing_panel(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let block = Block::default().borders(Borders::ALL).title("Fishing");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let fish_required = FishingState::fish_required_for_rank(game_state.fishing.rank);
    let fish_progress = game_state.fishing.fish_toward_next_rank;
    let fish_ratio = if fish_required > 0 {
        (fish_progress as f64 / fish_required as f64).min(1.0)
    } else {
        0.0
    };

    let rank_line = Line::from(vec![
        Span::styled("🎣 Rank: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "{} ({})",
                game_state.fishing.rank_name(),
                game_state.fishing.rank
            ),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    let fish_label = format!("{}/{}", fish_progress, fish_required);
    let fish_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .label(fish_label)
        .ratio(fish_ratio);

    if inner.height >= 2 {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let rank_paragraph = Paragraph::new(rank_line);
        frame.render_widget(rank_paragraph, inner_chunks[0]);
        frame.render_widget(fish_gauge, inner_chunks[1]);
    } else if inner.height >= 1 {
        // Only room for one line — show rank
        let rank_paragraph = Paragraph::new(rank_line);
        frame.render_widget(rank_paragraph, inner);
    }
}

/// Returns the style for an enhancement prefix based on its color tier.
fn enhancement_style(level: u8) -> Style {
    let (r, g, b) = crate::enhancement::enhancement_color_rgb(level);
    let tier = crate::enhancement::enhancement_color_tier(level);
    let style = Style::default().fg(Color::Rgb(r, g, b));
    match tier {
        2..=4 => style.add_modifier(Modifier::BOLD),
        _ => style,
    }
}

/// Draws attributes in a compact 2-column layout.
/// 3 rows: STR/INT, DEX/WIS, CON/CHA with modifiers.
fn draw_attributes_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let attrs_block = Block::default().borders(Borders::ALL).title("Attributes");
    let inner = attrs_block.inner(area);
    frame.render_widget(attrs_block, area);

    let cap = game_state.get_attribute_cap();

    // Pair attributes: STR/INT, DEX/WIS, CON/CHA
    let pairs = [
        (AttributeType::Strength, AttributeType::Intelligence),
        (AttributeType::Dexterity, AttributeType::Wisdom),
        (AttributeType::Constitution, AttributeType::Charisma),
    ];

    let mut lines = Vec::new();
    for (left, right) in &pairs {
        let l_val = game_state.attributes.get(*left);
        let l_mod = game_state.attributes.modifier(*left);
        let r_val = game_state.attributes.get(*right);
        let r_mod = game_state.attributes.modifier(*right);

        let l_color = attr_color(*left);
        let r_color = attr_color(*right);

        let l_mod_str = format_modifier(l_mod);
        let r_mod_str = format_modifier(r_mod);

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", left.abbrev()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:2}", l_val), Style::default().fg(l_color)),
            Span::raw(format!(" ({:>3})  ", l_mod_str)),
            Span::styled(
                format!("{}: ", right.abbrev()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:2}", r_val), Style::default().fg(r_color)),
            Span::raw(format!(" ({:>3})  ", r_mod_str)),
            Span::styled(
                format!("[Cap:{}]", cap),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Draws equipment with name + rarity color only, one line per slot (L tier).
fn draw_equipment_names_only(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    enhancement_levels: &[u8; 7],
) {
    let block = Block::default().borders(Borders::ALL).title("Equipment");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    use crate::items::EquipmentSlot;
    let slot_order = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    for (idx, slot_enum) in slot_order.iter().enumerate() {
        let item = game_state.equipment.get(*slot_enum);
        let slot_label = slot_enum.name();
        if let Some(item) = item {
            let rarity_color = match item.rarity {
                Rarity::Common => Color::White,
                Rarity::Magic => Color::Blue,
                Rarity::Rare => Color::Yellow,
                Rarity::Epic => Color::Magenta,
                Rarity::Legendary => Color::LightRed,
            };

            let enh_level = enhancement_levels[idx];
            let prefix = crate::enhancement::enhancement_prefix(enh_level);

            let item_name = if item.display_name.len() > 20 {
                format!("{}...", &item.display_name[..17])
            } else {
                item.display_name.clone()
            };

            let mut name_spans = vec![Span::styled(
                format!("{:>6}: ", slot_label),
                Style::default().add_modifier(Modifier::BOLD),
            )];
            if !prefix.is_empty() {
                name_spans.push(Span::styled(prefix, enhancement_style(enh_level)));
            }
            name_spans.push(Span::styled(item_name, Style::default().fg(rarity_color)));
            name_spans.push(Span::raw(" "));
            name_spans.push(Span::styled(
                format!("[{}]", item.rarity.name()),
                Style::default().fg(rarity_color),
            ));

            lines.push(Line::from(name_spans));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>6}: ", slot_label),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled("[Empty]", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Draws a compact stats bar for M tier: single line with name, level, prestige, zone.
/// Format: "Hero Lv.42 | P:12 Gold 2.80x | Zone 3: Mountain (2/3)"
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
/// Format: "STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16"
pub(super) fn draw_attributes_single_line(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut spans = Vec::new();

    for (i, attr_type) in AttributeType::all().iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let value = game_state.attributes.get(*attr_type);
        let color = attr_color(*attr_type);
        spans.push(Span::styled(
            format!("{}:", attr_type.abbrev()),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{}", value),
            Style::default().fg(color),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Draws a compact XP bar for M/S tier (borderless, single line).
pub(super) fn draw_xp_bar_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let xp_needed = xp_for_next_level(game_state.character_level);
    let xp_ratio = if xp_needed > 0 {
        (game_state.character_xp as f64 / xp_needed as f64).min(1.0)
    } else {
        0.0
    };

    let xp_label = format!(
        "XP: {}/{} ({:.1}%)",
        game_state.character_xp,
        xp_needed,
        xp_ratio * 100.0
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

/// Draws a compact footer for M tier (1 row, no borders).
/// Format: "[Esc]Quit [P]Prestige [H]Haven [A]Ach [Tab]Chall"
pub(super) fn draw_footer_compact(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    haven_discovered: bool,
    soulforge_discovered: bool,
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
        ach_span,
        challenge_span,
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Draws a minimal footer for S tier (1 row, minimal keybindings).
/// Format: "Esc:Quit P:Prestige Tab:More"
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
        Span::styled(" Tab:More", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Returns the display color for an attribute type.
fn attr_color(attr_type: AttributeType) -> Color {
    match attr_type {
        AttributeType::Strength => Color::Red,
        AttributeType::Dexterity => Color::Green,
        AttributeType::Constitution => Color::Magenta,
        AttributeType::Intelligence => Color::Blue,
        AttributeType::Wisdom => Color::Cyan,
        AttributeType::Charisma => Color::Yellow,
    }
}

/// Formats a modifier value with a sign prefix.
fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
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

    // Add changelog items as bullet points (up to 5)
    let max_items = 5;
    for item in info.changelog.iter().take(max_items) {
        lines.push(Line::from(vec![
            Span::styled("    • ", Style::default().fg(Color::DarkGray)),
            Span::styled(item.clone(), Style::default().fg(Color::White)),
        ]));
    }

    // Show remaining count if there are more
    if info.changelog_total > max_items {
        lines.push(Line::from(vec![Span::styled(
            format!("    (+{} more changes)", info.changelog_total - max_items),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    // Add empty line and footer
    lines.push(Line::from(vec![]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Run 'quest update' to install",
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("                              "),
        Span::styled("[U] Close", Style::default().fg(Color::Yellow)),
    ]));

    let drawer = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                " 🆕 Update Available ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_widget(drawer, area);
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
    haven_discovered: bool,
    soulforge_discovered: bool,
    pending_achievements: usize,
    _ctx: &LayoutContext,
) {
    use crate::character::prestige::can_prestige;
    use crate::utils::build_info::{BUILD_COMMIT, BUILD_DATE};

    // Build version string for the title
    let version_title = format!("v{} ({}) ", BUILD_DATE, BUILD_COMMIT);

    // Normal footer (update drawer is drawn separately when expanded)
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

    // Build update status text
    let update_status_text = if let Some(info) = update_info {
        Span::styled(
            format!("    🆕 [U] Update (v{})", info.new_version),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if update_check_completed {
        Span::styled("    ✓ Up to date", Style::default().fg(Color::Green))
    } else {
        use super::throbber::spinner_char;
        Span::styled(
            format!("    {} Checking...", spinner_char()),
            Style::default().fg(Color::DarkGray),
        )
    };

    // Build challenge notification text
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

    // Build Haven hint text
    let haven_text = if haven_discovered {
        Span::styled("    [H] Haven", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    // Build Soulforge hint text
    let soulforge_text = if soulforge_discovered {
        Span::styled("    [S] Soulforge", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    // Achievements hint (with pending count if any)
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
        ]),
        Line::from(vec![achievements_text, challenge_text, update_status_text]),
    ];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title(version_title))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
