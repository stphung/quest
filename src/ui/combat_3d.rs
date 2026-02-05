use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::enemy_sprites::get_sprite_for_enemy;

/// Renders a simple, clean combat view with sprite and combat log
pub fn render_combat_3d(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let combat_block = Block::default()
        .borders(Borders::ALL)
        .title("⚔ COMBAT ⚔")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let inner = combat_block.inner(area);
    frame.render_widget(combat_block, area);

    if inner.height < 8 || inner.width < 20 {
        let msg = Paragraph::new("Area too small").alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Split into sprite area (top) and combat log (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),     // Sprite area
            Constraint::Length(12), // Combat log (expanded)
        ])
        .split(inner);

    // Render enemy sprite (simple, centered)
    render_simple_sprite(frame, chunks[0], game_state);

    // Render combat log
    render_combat_log(frame, chunks[1], game_state);
}

/// Renders a simple, centered enemy sprite
fn render_simple_sprite(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut sprite_lines: Vec<Line> = Vec::new();

    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let sprite_template = get_sprite_for_enemy(&enemy.name);
        let sprite_art = sprite_template.base_art;

        // Add padding at top
        let available_height = area.height as usize;
        let sprite_height = sprite_art.lines().count();
        let top_padding = (available_height.saturating_sub(sprite_height)) / 2;

        for _ in 0..top_padding {
            sprite_lines.push(Line::from(""));
        }

        // Render sprite centered
        for line in sprite_art.lines() {
            let line_width = line.chars().count();
            let padding = (area.width as usize).saturating_sub(line_width) / 2;

            sprite_lines.push(Line::from(vec![
                Span::raw(" ".repeat(padding)),
                Span::styled(
                    line,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // Add enemy name below sprite
        sprite_lines.push(Line::from(""));
        sprite_lines.push(
            Line::from(vec![Span::styled(
                enemy.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )])
            .alignment(Alignment::Center),
        );
    } else {
        // No enemy - show waiting message with spinner and rotating messages
        use super::throbber::{spinner_char, waiting_message};

        let spinner = spinner_char();
        let message = waiting_message(game_state.character_xp);

        let msg_line = (area.height / 2) as usize;
        for i in 0..area.height as usize {
            if i == msg_line {
                sprite_lines.push(
                    Line::from(vec![Span::styled(
                        format!("{} {}", spinner, message),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    )])
                    .alignment(Alignment::Center),
                );
            } else {
                sprite_lines.push(Line::from(""));
            }
        }
    }

    let sprite_paragraph = Paragraph::new(sprite_lines).alignment(Alignment::Center);
    frame.render_widget(sprite_paragraph, area);
}

/// Renders recent combat events as a log
fn render_combat_log(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut log_lines: Vec<Line> = Vec::new();

    // Add title
    log_lines.push(Line::from(vec![Span::styled(
        "─── Combat Log ───",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    // Show recent combat events from history (newest first, up to 8 entries)
    let max_entries = (area.height as usize).saturating_sub(3); // Leave room for title and status
    let history_entries = game_state
        .combat_state
        .combat_log
        .iter()
        .rev()
        .take(max_entries.saturating_sub(2));

    for entry in history_entries {
        let color = if entry.is_player_action {
            if entry.is_crit {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::Red
        };

        let modifier = if entry.is_crit {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        log_lines.push(Line::from(vec![Span::styled(
            entry.message.clone(),
            Style::default().fg(color).add_modifier(modifier),
        )]));
    }

    // Add separator
    log_lines.push(Line::from(""));

    // Show current status
    if let Some(_enemy) = &game_state.combat_state.current_enemy {
        if game_state.combat_state.attack_timer > 0.0 {
            let next_attack = crate::core::constants::ATTACK_INTERVAL_SECONDS
                - game_state.combat_state.attack_timer;
            log_lines.push(Line::from(vec![
                Span::styled("⏱ Next attack in ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.1}s", next_attack.max(0.0)),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    } else if game_state.combat_state.is_regenerating {
        log_lines.push(Line::from(vec![Span::styled(
            "❤ Regenerating HP...",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        )]));
        let regen_progress =
            game_state.combat_state.regen_timer / crate::core::constants::HP_REGEN_DURATION_SECONDS;
        log_lines.push(Line::from(vec![Span::styled(
            format!("   {:.0}% complete", regen_progress * 100.0),
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        log_lines.push(Line::from(vec![Span::styled(
            "⌛ Spawning enemy...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )]));
    }

    // Pad to fill the area
    while log_lines.len() < area.height as usize {
        log_lines.push(Line::from(""));
    }

    let log_paragraph = Paragraph::new(log_lines);
    frame.render_widget(log_paragraph, area);
}
