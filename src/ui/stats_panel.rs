use crate::attributes::AttributeType;
use crate::derived_stats::DerivedStats;
use crate::game_logic::xp_for_next_level;
use crate::game_state::GameState;
use crate::items::{Affix, AffixType, Rarity};
use crate::prestige::{get_adventurer_rank, get_prestige_tier};
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

/// Draws the stats panel showing player attributes and derived stats
pub fn draw_stats_panel(frame: &mut Frame, area: Rect, game_state: &GameState) {
    // Main vertical layout: header, zone, attributes, derived stats, equipment, prestige, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Zone info
            Constraint::Length(14), // Attributes (6 attributes + borders)
            Constraint::Length(6),  // Derived stats (condensed)
            Constraint::Length(8),  // Equipment section
            Constraint::Length(6),  // Prestige info + fishing rank
            Constraint::Length(3),  // Footer
        ])
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

    // Draw footer with controls
    draw_footer(frame, chunks[6], game_state);
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

    // Choose color based on attribute type
    let color = match attr_type {
        AttributeType::Strength => Color::Red,
        AttributeType::Dexterity => Color::Green,
        AttributeType::Constitution => Color::Magenta,
        AttributeType::Intelligence => Color::Blue,
        AttributeType::Wisdom => Color::Cyan,
        AttributeType::Charisma => Color::Yellow,
    };

    // Format modifier with sign
    let mod_str = if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    };

    let text = vec![Line::from(vec![
        Span::styled(
            format!("{}: ", attr_type.abbrev()),
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
            Span::styled("Max HP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", derived.max_hp),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Physical Damage: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", derived.physical_damage),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Magic Damage: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", derived.magic_damage),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::styled("Defense: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", derived.defense),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Crit Chance: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}%", derived.crit_chance_percent),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "XP Multiplier: ",
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
            Span::styled("Rank: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} ({})", game_state.prestige_rank, tier.name),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Multiplier: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
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
            Span::styled(
                "Total Prestiges: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
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

            // First line: icon, slot, name, rarity, stars
            let stars = "⭐".repeat(item.rarity as usize + 1);
            let item_name = if item.display_name.len() > 25 {
                format!("{}...", &item.display_name[..22])
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

            // Second line: attribute bonuses and affixes (indented)
            let mut bonuses = Vec::new();

            // Add attribute bonuses
            let attr_bonuses = [
                (item.attributes.str, "STR"),
                (item.attributes.dex, "DEX"),
                (item.attributes.con, "CON"),
                (item.attributes.int, "INT"),
                (item.attributes.wis, "WIS"),
                (item.attributes.cha, "CHA"),
            ];
            for (value, name) in attr_bonuses {
                if value > 0 {
                    bonuses.push(format!("+{}{}", value, name));
                }
            }

            // Add affixes
            for affix in &item.affixes {
                bonuses.push(format_affix(affix));
            }

            if !bonuses.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("             "),
                    Span::styled(bonuses.join(", "), Style::default().fg(Color::Gray)),
                ]));
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

/// Draws the footer with control instructions and version info
fn draw_footer(frame: &mut Frame, area: Rect, game_state: &GameState) {
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

    let footer_text = vec![Line::from(vec![
        Span::styled("Controls: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            "Q",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" = Quit | "),
        prestige_text,
    ])];

    // Build version string for the title
    let version_title = format!("v{} ({}) ", BUILD_DATE, BUILD_COMMIT);

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title(version_title))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
