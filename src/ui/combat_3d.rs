use crate::game_state::GameState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::ascii_scaler::{apply_depth_shading, scale_sprite};
use super::enemy_sprites::get_sprite_for_enemy;
use super::perspective::{render_ceiling, render_floor, render_walls};

/// Calculates enemy depth based on combat state (0.0 = far, 1.0 = close)
fn calculate_combat_depth(game_state: &GameState) -> f64 {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let player_hp_ratio = game_state.combat_state.player_current_hp as f64
            / game_state.combat_state.player_max_hp as f64;
        let enemy_hp_ratio = enemy.current_hp as f64 / enemy.max_hp as f64;

        // When player losing (low HP): enemy appears closer (higher depth)
        // When enemy losing: enemy appears farther (lower depth)
        let depth = 0.5 + (player_hp_ratio - enemy_hp_ratio) * 0.3;

        // Clamp to visible range
        depth.clamp(0.2, 0.9)
    } else {
        0.5 // Default middle distance
    }
}

/// Renders the full 3D combat scene
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

    if inner.height < 10 || inner.width < 30 {
        // Too small to render 3D view
        let msg = Paragraph::new("Area too small for 3D view").alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Layer composition
    let mut scene_lines: Vec<Line> = Vec::new();

    // Calculate heights for each layer
    let ceiling_height = inner.height / 4;
    let floor_height = inner.height / 4;
    let middle_height = inner.height - ceiling_height - floor_height;

    // 1. Render ceiling
    let ceiling = render_ceiling(inner.width as usize, ceiling_height as usize);
    scene_lines.extend(ceiling);

    // 2. Render middle section with walls and enemy
    let (left_walls, right_walls) = render_walls(inner.width as usize, middle_height as usize);

    if let Some(enemy) = &game_state.combat_state.current_enemy {
        // Get enemy sprite and scale it
        let sprite_template = get_sprite_for_enemy(&enemy.name);
        let depth = calculate_combat_depth(game_state);

        let target_height = (3.0 + depth * 17.0) as usize; // 3-20 lines
        let target_height = target_height.min(middle_height as usize);

        let scaled_sprite = scale_sprite(sprite_template.base_art, target_height);
        let shaded_sprite = apply_depth_shading(scaled_sprite, depth);

        // Render middle section with enemy centered
        for i in 0..middle_height as usize {
            let mut line_spans = Vec::new();

            // Left wall
            if i < left_walls.len() {
                line_spans.push(Span::styled(
                    left_walls[i].clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Center area with enemy
            let enemy_offset = (middle_height as usize).saturating_sub(target_height) / 2;
            if i >= enemy_offset && i < enemy_offset + shaded_sprite.len() {
                let sprite_line = &shaded_sprite[i - enemy_offset];
                let center_padding = (inner.width as usize)
                    .saturating_sub(left_walls[i].len() * 2)
                    .saturating_sub(sprite_line.len())
                    / 2;

                line_spans.push(Span::raw(" ".repeat(center_padding)));
                line_spans.push(Span::styled(
                    sprite_line.clone(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
                line_spans.push(Span::raw(" ".repeat(center_padding)));
            } else {
                // Empty center
                let center_width = (inner.width as usize).saturating_sub(left_walls[i].len() * 2);
                line_spans.push(Span::raw(" ".repeat(center_width)));
            }

            // Right wall
            if i < right_walls.len() {
                line_spans.push(Span::styled(
                    right_walls[i].clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            scene_lines.push(Line::from(line_spans));
        }
    } else {
        // No enemy - just render walls
        for i in 0..middle_height as usize {
            let mut line_spans = Vec::new();

            if i < left_walls.len() {
                line_spans.push(Span::styled(
                    left_walls[i].clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let center_width = (inner.width as usize).saturating_sub(left_walls[i].len() * 2);
            line_spans.push(Span::raw(" ".repeat(center_width)));

            if i < right_walls.len() {
                line_spans.push(Span::styled(
                    right_walls[i].clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            scene_lines.push(Line::from(line_spans));
        }
    }

    // 3. Render floor
    let floor = render_floor(inner.width as usize, floor_height as usize);
    scene_lines.extend(floor);

    // 4. Overlay visual effects on top of the scene
    if !game_state.combat_state.visual_effects.is_empty() {
        // Render effects in the middle section where the enemy is
        let effect_line_idx = ceiling_height as usize + (middle_height as usize / 3);

        for effect in &game_state.combat_state.visual_effects {
            if let Some(effect_line) = effect.render() {
                // Replace the line at effect position with the effect
                if effect_line_idx < scene_lines.len() {
                    scene_lines[effect_line_idx] = effect_line;
                }
            }
        }
    }

    // Render the composed scene
    let scene = Paragraph::new(scene_lines);
    frame.render_widget(scene, inner);
}
