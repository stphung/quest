use crate::core::constants::{ATTACK_INTERVAL_SECONDS, ENEMY_ATTACK_INTERVAL_SECONDS};
use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use super::combat_3d::render_combat_3d;

/// Draws the combat scene with 3D first-person view
pub fn draw_combat_scene(frame: &mut Frame, area: Rect, game_state: &GameState) {
    // Single outer border wrapping everything
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" ⚔ Combat ⚔ ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split inner area — no individual borders
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Player HP
            Constraint::Min(5),    // Sprite + Combat log
            Constraint::Length(1), // Enemy HP
            Constraint::Length(1), // Status
        ])
        .split(inner);

    // Draw player HP bar (borderless)
    draw_player_hp(frame, chunks[0], game_state);

    // Draw 3D combat scene (borderless)
    render_combat_3d(frame, chunks[1], game_state);

    // Draw enemy HP bar (borderless)
    draw_enemy_hp(frame, chunks[2], game_state);

    // Draw combat status
    draw_combat_status(frame, chunks[3], game_state);
}

/// Draws the player HP bar (borderless, single line)
fn draw_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let hp_ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;

    let label = format!(
        "Player HP: {}/{}",
        game_state.combat_state.player_current_hp, game_state.combat_state.player_max_hp
    );

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .label(label)
        .ratio(hp_ratio);

    frame.render_widget(gauge, area);
}

/// Draws the enemy HP bar (borderless, single line)
fn draw_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let hp_ratio = enemy.current_hp as f64 / enemy.max_hp as f64;

        let label = format!("{}: {}/{}", enemy.name, enemy.current_hp, enemy.max_hp);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .label(label)
            .ratio(hp_ratio);

        frame.render_widget(gauge, area);
    } else {
        let text = if game_state.combat_state.is_regenerating {
            vec![Line::from(Span::styled(
                "Regenerating...",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::ITALIC),
            ))]
        } else {
            vec![Line::from(Span::styled(
                "Spawning enemy...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))]
        };

        let paragraph = Paragraph::new(text).alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

/// Draws the combat status information with DPS
fn draw_combat_status(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use super::throbber::{spinner_char, waiting_message};
    use crate::character::derived_stats::DerivedStats;

    let spinner = spinner_char();

    // Calculate DPS for display
    let derived =
        DerivedStats::calculate_derived_stats(&game_state.attributes, &game_state.equipment);
    let base_dps = derived.total_damage() as f64 / ATTACK_INTERVAL_SECONDS;
    let effective_dps = base_dps
        * (1.0 + (derived.crit_chance_percent as f64 / 100.0) * (derived.crit_multiplier - 1.0));
    let dps_span = Span::styled(
        format!(" | DPS: {:.0}", effective_dps),
        Style::default().fg(Color::DarkGray),
    );

    let status_text = if game_state.combat_state.is_regenerating {
        let message = waiting_message(game_state.character_xp);
        vec![Line::from(vec![
            Span::styled(
                format!("{} {}", spinner, message),
                Style::default().fg(Color::Yellow),
            ),
            dps_span,
        ])]
    } else if game_state.combat_state.current_enemy.is_some() {
        let effective_player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
        let player_next = effective_player_interval - game_state.combat_state.attack_timer;
        let enemy_next = ENEMY_ATTACK_INTERVAL_SECONDS - game_state.combat_state.enemy_attack_timer;
        vec![Line::from(vec![
            Span::styled(
                format!("{} In Combat", spinner),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" | ⚔️ {:.1}s", player_next.max(0.0)),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!(" | 👹 {:.1}s", enemy_next.max(0.0)),
                Style::default().fg(Color::Red),
            ),
            dps_span,
        ])]
    } else {
        let message = waiting_message(game_state.character_xp);
        vec![Line::from(vec![
            Span::styled(
                format!("{} {}", spinner, message),
                Style::default().fg(Color::Yellow),
            ),
            dps_span,
        ])]
    };

    let status_paragraph = Paragraph::new(status_text).alignment(Alignment::Center);
    frame.render_widget(status_paragraph, area);
}
