use crate::attributes::AttributeType;
use crate::derived_stats::DerivedStats;
use crate::game_logic::xp_for_next_level;
use crate::game_state::GameState;
use crate::items::{Affix, AffixType, Rarity};
use crate::prestige::{get_adventurer_rank, get_prestige_tier};
use crate::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Formats an affix for display in the equipment panel.
fn format_affix(affix: &Affix) -> String {
    match affix.affix_type {
        AffixType::DamagePercent => format!("+{:.0}% DMG", affix.value),
        AffixType::CritChance => format!("+{:.0}% CRIT", affix.value),
        AffixType::CritMultiplier => format!("+{:.0}% CritMult", affix.value),
        AffixType::AttackSpeed => format!("+{:.0}% Speed", affix.value),
        AffixType::HPBonus => format!("+{:.0} HP", affix.value),
        AffixType::DamageReduction => format!("+{:.0}% DR", affix.value),
        AffixType::HPRegen => format!("+{:.0}% Regen", affix.value),
        AffixType::DamageReflection => format!("+{:.0}% Reflect", affix.value),
        AffixType::XPGain => format!("+{:.0}% XP", affix.value),
        AffixType::DropRate => format!("+{:.0}% Drops", affix.value),
        AffixType::PrestigeBonus => format!("+{:.0}% Prestige", affix.value),
        AffixType::OfflineRate => format!("+{:.0}% Offline", affix.value),
    }
}

/// Draws the stats panel with optional update notification
pub fn draw_stats_panel_with_update(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    next_update_check_secs: Option<u64>,
) {
    // Calculate update panel height: 4 base + changelog lines (max 5)
    let update_height = if let Some(info) = update_info {
        4 + info.changelog.len().min(5) as u16
    } else {
        0
    };

    // Main vertical layout: header, zone, attributes, derived stats, equipment, prestige, footer, [update]
    // When update panel is present, equipment gets smaller minimum to ensure update panel fits
    let constraints = if update_info.is_some() {
        vec![
            Constraint::Length(3),          // Header
            Constraint::Length(3),          // Zone info
            Constraint::Length(14),         // Attributes (6 attributes + borders)
            Constraint::Length(6),          // Derived stats (condensed)
            Constraint::Min(10),            // Equipment section (reduced min when update shown)
            Constraint::Length(6),          // Prestige info + fishing rank
            Constraint::Length(4),          // Footer (2 lines + borders)
            Constraint::Min(update_height), // Update panel (can shrink if needed)
        ]
    } else {
        vec![
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Zone info
            Constraint::Length(14), // Attributes (6 attributes + borders)
            Constraint::Length(6),  // Derived stats (condensed)
            Constraint::Min(16),    // Equipment section (grows to fit)
            Constraint::Length(6),  // Prestige info + fishing rank
            Constraint::Length(4),  // Footer (2 lines + borders)
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Draw header with character info
    draw_header(frame, chunks[0], game_state);

    // Draw zone info
    draw_zone_info(frame, chunks[1], game_state);

    // Draw attributes with progress bars
    draw_attributes(frame, chunks[2], game_state);

    // Draw derived stats
    draw_derived_stats(frame, chunks[3], game_state);

    // Draw equipment section
    draw_equipment_section(frame, chunks[4], game_state);

    // Draw prestige info
    draw_prestige_info(frame, chunks[5], game_state);

    // Draw footer with controls and update check countdown
    draw_footer(frame, chunks[6], game_state, next_update_check_secs);

    // Draw update panel if available
    if let Some(info) = update_info {
        draw_update_panel(frame, chunks[7], info);
    }
}

/// Draws the header with character level and XP
fn draw_header(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let xp_needed = xp_for_next_level(game_state.character_level);
    let xp_progress = if xp_needed > 0 {
        game_state.character_xp as f64 / xp_needed as f64
    } else {
        0.0
    };

    let rank = get_adventurer_rank(game_state.character_level);

    let header_text = vec![Line::from(vec![
        Span::styled(
            format!("Level {} {}", game_state.character_level, rank),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "XP: {}/{} ({:.1}%)",
                game_state.character_xp,
                xp_needed,
                xp_progress * 100.0
            ),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Play Time: {}s", game_state.play_time_seconds),
            Style::default().fg(Color::Green),
        ),
    ])];

    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(game_state.character_name.as_str()),
        )
        .alignment(Alignment::Center);

    frame.render_widget(header, area);
}

/// Draws the current zone and subzone info
fn draw_zone_info(frame: &mut Frame, area: Rect, game_state: &GameState) {
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

    // Build the boss progress display
    let boss_progress = if let Some(weapon) = prog.boss_weapon_blocked() {
        // Fighting boss that requires weapon we don't have
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

    let zone_text = vec![Line::from(vec![
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

    let zone_widget = Paragraph::new(zone_text)
        .block(Block::default().borders(Borders::ALL).title("Location"))
        .alignment(Alignment::Center);

    frame.render_widget(zone_widget, area);
}

/// Draws all 6 attributes with their values and caps
fn draw_attributes(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let attrs_block = Block::default().borders(Borders::ALL).title("Attributes");

    let inner = attrs_block.inner(area);
    frame.render_widget(attrs_block, area);

    // Layout for 6 attribute rows
    let attr_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // STR
            Constraint::Length(2), // DEX
            Constraint::Length(2), // CON
            Constraint::Length(2), // INT
            Constraint::Length(2), // WIS
            Constraint::Length(2), // CHA
        ])
        .split(inner);

    let cap = game_state.get_attribute_cap();

    // Draw each attribute
    for (i, attr_type) in AttributeType::all().iter().enumerate() {
        if i < attr_chunks.len() {
            draw_attribute_row(frame, attr_chunks[i], game_state, *attr_type, cap);
        }
    }
}

/// Draws a single attribute row
fn draw_attribute_row(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    attr_type: AttributeType,
    cap: u32,
) {
    let value = game_state.attributes.get(attr_type);
    let modifier = game_state.attributes.modifier(attr_type);

    // Choose color and emoji based on attribute type
    let (color, emoji) = match attr_type {
        AttributeType::Strength => (Color::Red, "💪"),
        AttributeType::Dexterity => (Color::Green, "🏃"),
        AttributeType::Constitution => (Color::Magenta, "❤️"),
        AttributeType::Intelligence => (Color::Blue, "🧠"),
        AttributeType::Wisdom => (Color::Cyan, "👁️"),
        AttributeType::Charisma => (Color::Yellow, "✨"),
    };

    // Format modifier with sign
    let mod_str = if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    };

    let text = vec![Line::from(vec![
        Span::styled(
            format!("{} {}: ", emoji, attr_type.abbrev()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:2}", value),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" ({:>3}) ", mod_str)),
        Span::styled(
            format!("[Cap: {}]", cap),
            Style::default().fg(Color::DarkGray),
        ),
    ])];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

/// Draws derived stats calculated from attributes
fn draw_derived_stats(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let derived =
        DerivedStats::calculate_derived_stats(&game_state.attributes, &game_state.equipment);

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title("Derived Stats");

    let inner = stats_block.inner(area);
    frame.render_widget(stats_block, area);

    let stats_text = vec![
        Line::from(vec![
            Span::styled("💚 Max HP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", derived.max_hp),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "⚔️ Physical: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", derived.physical_damage),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(vec![
            Span::styled("🔮 Magic: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", derived.magic_damage),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "🛡️ Defense: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", derived.defense),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("💥 Crit: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}%", derived.crit_chance_percent),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "📈 XP Mult: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:.2}x", derived.xp_multiplier),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_text);
    frame.render_widget(stats_paragraph, inner);
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
            Span::styled("🔄 Total: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", game_state.total_prestige_count),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "🎣 Fishing: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} ({})",
                    game_state.fishing.rank_name(),
                    game_state.fishing.rank
                ),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];

    let prestige_paragraph = Paragraph::new(prestige_text);
    frame.render_widget(prestige_paragraph, inner);
}

/// Draws equipment section with all 7 equipment slots
fn draw_equipment_section(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let equipment_block = Block::default().borders(Borders::ALL).title("Equipment");

    let inner = equipment_block.inner(area);
    frame.render_widget(equipment_block, area);

    let mut lines = Vec::new();

    // Draw each equipment slot
    let slots = vec![
        (game_state.equipment.weapon.as_ref(), "⚔️ Weapon"),
        (game_state.equipment.armor.as_ref(), "🛡 Armor"),
        (game_state.equipment.helmet.as_ref(), "🪖 Helmet"),
        (game_state.equipment.gloves.as_ref(), "🧤 Gloves"),
        (game_state.equipment.boots.as_ref(), "👢 Boots"),
        (game_state.equipment.amulet.as_ref(), "📿 Amulet"),
        (game_state.equipment.ring.as_ref(), "💍 Ring"),
    ];

    for (item, slot_label) in slots {
        if let Some(item) = item {
            // Get rarity color
            let rarity_color = match item.rarity {
                Rarity::Common => Color::White,
                Rarity::Magic => Color::Blue,
                Rarity::Rare => Color::Yellow,
                Rarity::Epic => Color::Magenta,
                Rarity::Legendary => Color::LightRed,
            };

            // Line 1: icon, name, rarity, stars
            let stars = "⭐".repeat(item.rarity as usize + 1);
            let item_name = if item.display_name.len() > 28 {
                format!("{}...", &item.display_name[..25])
            } else {
                item.display_name.clone()
            };

            lines.push(Line::from(vec![
                Span::raw(format!("{} ", slot_label)),
                Span::styled(item_name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", item.rarity.name()),
                    Style::default().fg(rarity_color),
                ),
                Span::raw(format!(" {}", stars)),
            ]));

            // Line 2: attribute bonuses with colored emojis
            let attr_bonuses = [
                (item.attributes.str, "💪", Color::Red),
                (item.attributes.dex, "🏃", Color::Green),
                (item.attributes.con, "❤️", Color::Magenta),
                (item.attributes.int, "🧠", Color::Blue),
                (item.attributes.wis, "👁️", Color::Cyan),
                (item.attributes.cha, "✨", Color::Yellow),
            ];

            let mut attr_spans: Vec<Span> = vec![Span::raw("   ")];
            let mut has_attrs = false;
            for (value, emoji, color) in attr_bonuses {
                if value > 0 {
                    if has_attrs {
                        attr_spans.push(Span::raw(" "));
                    }
                    attr_spans.push(Span::styled(
                        format!("{}+{}", emoji, value),
                        Style::default().fg(color),
                    ));
                    has_attrs = true;
                }
            }

            if has_attrs {
                lines.push(Line::from(attr_spans));
            }

            // Line 3: affixes (if any)
            if !item.affixes.is_empty() {
                let mut affix_spans: Vec<Span> = vec![Span::raw("   ")];
                for (i, affix) in item.affixes.iter().enumerate() {
                    if i > 0 {
                        affix_spans.push(Span::styled(" ", Style::default()));
                    }
                    affix_spans.push(Span::styled(
                        format_affix(affix),
                        Style::default().fg(Color::Gray),
                    ));
                }
                lines.push(Line::from(affix_spans));
            }
        } else {
            // Empty slot
            lines.push(Line::from(vec![
                Span::raw(slot_label),
                Span::raw(" "),
                Span::styled("[Empty]", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let equipment_paragraph = Paragraph::new(lines);
    frame.render_widget(equipment_paragraph, inner);
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
fn draw_footer(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    next_update_check_secs: Option<u64>,
) {
    use crate::build_info::{BUILD_COMMIT, BUILD_DATE};
    use crate::prestige::can_prestige;

    let can_prestige_now = can_prestige(game_state);
    let prestige_text = if can_prestige_now {
        Span::styled(
            "P = Prestige (AVAILABLE!)",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )
    } else {
        let next_tier = get_prestige_tier(game_state.prestige_rank + 1);
        Span::styled(
            format!("P = Prestige (Need Lv.{})", next_tier.required_level),
            Style::default().fg(Color::DarkGray),
        )
    };

    // Build update check countdown text (minutes only)
    let update_check_text = if let Some(secs) = next_update_check_secs {
        let mins = secs.div_ceil(60); // Round up to nearest minute
        format!(" | Update: {}m", mins)
    } else {
        String::new()
    };

    // Build play time text
    let play_time_text = format_play_time(game_state.play_time_seconds);

    // Line 1: Controls
    let controls_line = Line::from(vec![
        Span::styled("Controls: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            "Q",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" = Quit | "),
        prestige_text,
    ]);

    // Line 2: Play time and update check
    let stats_line = Line::from(vec![
        Span::styled("⏱️ ", Style::default().fg(Color::Cyan)),
        Span::styled(play_time_text, Style::default().fg(Color::Cyan)),
        Span::styled(update_check_text, Style::default().fg(Color::DarkGray)),
    ]);

    let footer_text = vec![controls_line, stats_line];

    // Build version string for the title
    let version_title = format!("v{} ({}) ", BUILD_DATE, BUILD_COMMIT);

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title(version_title))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

/// Draws the update notification panel
fn draw_update_panel(frame: &mut Frame, area: Rect, update_info: &UpdateInfo) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("New version: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{} ({})", update_info.new_version, update_info.new_commit),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    // Add changelog entries
    if !update_info.changelog.is_empty() {
        for entry in &update_info.changelog {
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(Color::DarkGray)),
                Span::styled(entry.as_str(), Style::default().fg(Color::White)),
            ]));
        }
    }

    // Add install instruction
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Run 'quest update' to install",
        Style::default().fg(Color::DarkGray),
    )]));

    let update_panel = Paragraph::new(lines).block(
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

    frame.render_widget(update_panel, area);
}
