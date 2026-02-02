use crate::attributes::AttributeType;
use crate::derived_stats::DerivedStats;
use crate::game_logic::xp_for_next_level;
use crate::game_state::GameState;
use crate::prestige::{get_adventurer_rank, get_prestige_tier};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draws the stats panel showing player attributes and derived stats
pub fn draw_stats_panel(frame: &mut Frame, area: Rect, game_state: &GameState) {
    // Main vertical layout: header, attributes, derived stats, prestige, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(14), // Attributes (6 attributes + borders)
            Constraint::Length(9),  // Derived stats
            Constraint::Length(5),  // Prestige info
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    // Draw header with character info
    draw_header(frame, chunks[0], game_state);

    // Draw attributes with progress bars
    draw_attributes(frame, chunks[1], game_state);

    // Draw derived stats
    draw_derived_stats(frame, chunks[2], game_state);

    // Draw prestige info
    draw_prestige_info(frame, chunks[3], game_state);

    // Draw footer with controls
    draw_footer(frame, chunks[4], game_state);
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
        .block(Block::default().borders(Borders::ALL).title("Character"))
        .alignment(Alignment::Center);

    frame.render_widget(header, area);
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
    let derived = DerivedStats::calculate_derived_stats(&game_state.attributes, &game_state.equipment);

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
    ];

    let prestige_paragraph = Paragraph::new(prestige_text);
    frame.render_widget(prestige_paragraph, inner);
}

/// Draws the footer with control instructions
fn draw_footer(frame: &mut Frame, area: Rect, game_state: &GameState) {
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

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
