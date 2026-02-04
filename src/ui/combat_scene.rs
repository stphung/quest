use crate::constants::ATTACK_INTERVAL_SECONDS;
use crate::game_state::GameState;
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
    // Split into 3D view and status bars
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Player HP
            Constraint::Min(10),   // 3D Combat View
            Constraint::Length(3), // Enemy HP
            Constraint::Length(3), // Status
        ])
        .split(area);

    // Draw player HP bar
    draw_player_hp(frame, chunks[0], game_state);

    // Draw 3D combat scene
    render_combat_3d(frame, chunks[1], game_state);

    // Draw enemy HP bar
    draw_enemy_hp(frame, chunks[2], game_state);

    // Draw combat status
    draw_combat_status(frame, chunks[3], game_state);
}

/// Draws the player HP bar
fn draw_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let hp_ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;

    let label = format!(
        "Player HP: {}/{}",
        game_state.combat_state.player_current_hp, game_state.combat_state.player_max_hp
    );

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Player"))
        .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .label(label)
        .ratio(hp_ratio);

    frame.render_widget(gauge, area);
}

/// Draws the enemy HP bar
fn draw_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let hp_ratio = enemy.current_hp as f64 / enemy.max_hp as f64;

        let label = format!("{}: {}/{}", enemy.name, enemy.current_hp, enemy.max_hp);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Enemy"))
            .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .label(label)
            .ratio(hp_ratio);

        frame.render_widget(gauge, area);
    } else {
        let empty_block = Block::default().borders(Borders::ALL).title("Enemy");
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

        let paragraph = Paragraph::new(text)
            .block(empty_block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

/// Draws the combat status information
fn draw_combat_status(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use super::throbber::{spinner_char, waiting_message};

    let spinner = spinner_char();

    let status_text = if game_state.combat_state.is_regenerating {
        let message = waiting_message(game_state.character_xp);
        vec![Line::from(vec![Span::styled(
            format!("{} {}", spinner, message),
            Style::default().fg(Color::Yellow),
        )])]
    } else if game_state.combat_state.current_enemy.is_some() {
        let next_attack = ATTACK_INTERVAL_SECONDS - game_state.combat_state.attack_timer;
        vec![Line::from(vec![
            Span::styled(
                format!("{} In Combat", spinner),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | Next attack: {:.1}s", next_attack.max(0.0))),
        ])]
    } else {
        let message = waiting_message(game_state.character_xp);

        vec![Line::from(vec![Span::styled(
            format!("{} {}", spinner, message),
            Style::default().fg(Color::Yellow),
        )])]
    };

    let status_paragraph = Paragraph::new(status_text).alignment(Alignment::Center);
    frame.render_widget(status_paragraph, area);
}
