use crate::game_logic::{xp_for_next_level, xp_gain_per_tick};
use crate::game_state::{GameState, StatType};
use crate::ui::zones::get_current_zone;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Draws the stats panel showing player stats with progress bars
pub fn draw_stats_panel(frame: &mut Frame, area: Rect, game_state: &GameState) {
    // Main vertical layout: header, stats, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),     // Stats area
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Draw header
    draw_header(frame, chunks[0], game_state);

    // Draw stats with progress bars
    draw_stats(frame, chunks[1], game_state);

    // Draw footer with controls
    draw_footer(frame, chunks[2], game_state);
}

/// Draws the header with game title and prestige rank
fn draw_header(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let zone = get_current_zone(game_state);

    let header_text = vec![Line::from(vec![
        Span::styled("Idle RPG", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(
            format!("Zone: {}", zone.name),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Prestige Rank: {}", game_state.prestige_rank),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Play Time: {}s", game_state.play_time_seconds),
            Style::default().fg(Color::Green),
        ),
    ])];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Game Info"))
        .alignment(Alignment::Center);

    frame.render_widget(header, area);
}

/// Draws the stats section with 4 progress bars
fn draw_stats(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title("Character Stats");

    let inner = stats_block.inner(area);
    frame.render_widget(stats_block, area);

    // Layout for 4 stat rows
    let stat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Strength
            Constraint::Length(3), // Agility
            Constraint::Length(3), // Intelligence
            Constraint::Length(3), // Vitality
        ])
        .split(inner);

    // Draw each stat with progress bar
    let stat_types = StatType::all();
    for (i, stat_type) in stat_types.iter().enumerate() {
        if i < stat_chunks.len() {
            let stat = game_state.get_stat(*stat_type);
            draw_stat_row(frame, stat_chunks[i], stat, stat_type, game_state);
        }
    }
}

/// Draws a single stat row with name, level, and progress bar
fn draw_stat_row(
    frame: &mut Frame,
    area: Rect,
    stat: &crate::game_state::Stat,
    stat_type: &StatType,
    game_state: &GameState,
) {
    let xp_needed = xp_for_next_level(stat.level);

    // Calculate progress percentage (0.0 to 1.0)
    let progress = if xp_needed > 0 {
        stat.current_xp as f64 / xp_needed as f64
    } else {
        0.0
    };

    // Calculate XP per second (10 ticks per second)
    let xp_per_tick = xp_gain_per_tick(game_state.prestige_rank);
    let xp_per_second = xp_per_tick * 10.0;

    // Choose color based on stat type
    let color = match stat_type {
        StatType::Strength => Color::Red,
        StatType::Magic => Color::Blue,
        StatType::Wisdom => Color::Cyan,
        StatType::Vitality => Color::Magenta,
    };

    // Get full stat name
    let stat_name = match stat_type {
        StatType::Strength => "Strength",
        StatType::Magic => "Magic",
        StatType::Wisdom => "Wisdom",
        StatType::Vitality => "Vitality",
    };

    // Create label with stat info including XP/s and percentage
    let percentage = (progress * 100.0) as u32;
    let label = format!(
        "{} Lv.{} ({}/{} | {}% | {:.1} XP/s)",
        stat_name, stat.level, stat.current_xp, xp_needed, percentage, xp_per_second
    );

    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .label(label)
        .ratio(progress);

    frame.render_widget(gauge, area);
}

/// Draws the footer with control instructions
fn draw_footer(frame: &mut Frame, area: Rect, _game_state: &GameState) {
    let footer_text = vec![Line::from(vec![
        Span::styled("Controls: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" = Quit | "),
        Span::styled("P", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" = Prestige"),
    ])];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
