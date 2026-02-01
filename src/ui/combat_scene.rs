use crate::constants::{ATTACK_INTERVAL_SECONDS, HP_REGEN_DURATION_SECONDS};
use crate::game_state::GameState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Draws the combat scene showing player and enemy HP bars, combat status
pub fn draw_combat_scene(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let combat_block = Block::default()
        .borders(Borders::ALL)
        .title("Combat Arena");

    let inner = combat_block.inner(area);
    frame.render_widget(combat_block, area);

    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Player HP bar
            Constraint::Min(5),     // Combat area
            Constraint::Length(4), // Enemy HP bar
            Constraint::Length(3), // Combat status
        ])
        .split(inner);

    // Draw player HP bar
    draw_player_hp(frame, chunks[0], game_state);

    // Draw combat area with visual representation
    draw_combat_area(frame, chunks[1], game_state);

    // Draw enemy HP bar
    draw_enemy_hp(frame, chunks[2], game_state);

    // Draw combat status
    draw_combat_status(frame, chunks[3], game_state);
}

/// Draws the player HP bar
fn draw_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let hp_ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;

    let hp_color = if hp_ratio > 0.66 {
        Color::Green
    } else if hp_ratio > 0.33 {
        Color::Yellow
    } else {
        Color::Red
    };

    let label = format!(
        "Player HP: {}/{}",
        game_state.combat_state.player_current_hp, game_state.combat_state.player_max_hp
    );

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Player"))
        .gauge_style(Style::default().fg(hp_color).add_modifier(Modifier::BOLD))
        .label(label)
        .ratio(hp_ratio);

    frame.render_widget(gauge, area);
}

/// Draws the main combat area with player and enemy sprites
fn draw_combat_area(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let player_sprite = "🧙";
    let enemy_sprite = if game_state.combat_state.current_enemy.is_some() {
        "👹"
    } else {
        ""
    };

    // Show attack indicator during attack timer
    let attack_indicator = if game_state.combat_state.attack_timer < 0.3 {
        "⚔️"
    } else {
        " "
    };

    let combat_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("     "),
            Span::styled(
                player_sprite,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                attack_indicator,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                enemy_sprite,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    let combat_paragraph = Paragraph::new(combat_text).alignment(Alignment::Center);
    frame.render_widget(combat_paragraph, area);
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
        // No enemy present
        let empty_block = Block::default()
            .borders(Borders::ALL)
            .title("Enemy");

        let text = if game_state.combat_state.is_regenerating {
            vec![Line::from(Span::styled(
                "Regenerating...",
                Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC),
            ))]
        } else {
            vec![Line::from(Span::styled(
                "Spawning enemy...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
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
    let status_text = if game_state.combat_state.is_regenerating {
        let regen_progress = game_state.combat_state.regen_timer / HP_REGEN_DURATION_SECONDS;
        let remaining = HP_REGEN_DURATION_SECONDS - game_state.combat_state.regen_timer;
        vec![Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!(
                    "Regenerating HP... {:.1}s ({:.0}%)",
                    remaining,
                    regen_progress * 100.0
                ),
                Style::default().fg(Color::Green),
            ),
        ])]
    } else if game_state.combat_state.current_enemy.is_some() {
        let next_attack = ATTACK_INTERVAL_SECONDS - game_state.combat_state.attack_timer;
        vec![Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "In Combat",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | Next attack: {:.1}s", next_attack.max(0.0))),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "Waiting for enemy...",
                Style::default().fg(Color::Yellow),
            ),
        ])]
    };

    let status_paragraph = Paragraph::new(status_text).alignment(Alignment::Center);
    frame.render_widget(status_paragraph, area);
}
