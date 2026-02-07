use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::enemy_sprites::get_sprite_for_enemy;

/// Renders the enemy sprite (borderless, no combat log)
pub fn render_combat_3d(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if area.height < 3 || area.width < 20 {
        let msg = Paragraph::new("Area too small").alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    render_simple_sprite(frame, area, game_state);
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

        // Render sprite (centered by Paragraph alignment)
        for line in sprite_art.lines() {
            sprite_lines.push(
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center),
            );
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
