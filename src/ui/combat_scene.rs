use super::responsive::{LayoutContext, SizeTier};
use crate::combat::logic::effective_enemy_attack_interval;
use crate::core::constants::ATTACK_INTERVAL_SECONDS;
use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::combat::types::DAMAGE_FLASH_DURATION;

use super::combat_3d::render_combat_3d;
use super::enemy_sprites::zone_palette;

/// Draws the combat scene with 3D first-person view
pub fn draw_combat_scene(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    ctx: &LayoutContext,
) {
    match ctx.tier {
        SizeTier::M => {
            // Compact: no 3D sprite, just HP bars + status, with border
            draw_combat_compact(frame, area, game_state);
        }
        SizeTier::S => {
            // Minimal: no border, no sprite — HP bars handled by S layout directly
            // When called from S layout, just show combat status
            draw_combat_status(frame, area, game_state);
        }
        _ => {
            // Full layout with 3D sprite (XL/L)
            draw_combat_full(frame, area, game_state);
        }
    }
}

/// Full combat scene with 3D sprite (XL/L tier).
fn draw_combat_full(frame: &mut Frame, area: Rect, game_state: &GameState) {
    // Single outer border wrapping everything
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" \u{2694} Combat \u{2694} ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let is_regen = game_state.combat_state.is_regenerating;

    let mut constraints = vec![Constraint::Length(1)]; // Player HP
    if is_regen {
        constraints.push(Constraint::Length(1)); // Regen throbber
    }
    constraints.push(Constraint::Min(5)); // Sprite
    constraints.push(Constraint::Length(1)); // Enemy HP
    constraints.push(Constraint::Length(1)); // Status

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut idx = 0;
    draw_player_hp(frame, chunks[idx], game_state);
    idx += 1;

    if is_regen {
        draw_regen_throbber(frame, chunks[idx], game_state);
        idx += 1;
    }

    render_combat_3d(frame, chunks[idx], game_state);
    draw_floating_damage(frame, chunks[idx], game_state);
    idx += 1;

    draw_enemy_hp(frame, chunks[idx], game_state);
    idx += 1;

    draw_combat_status(frame, chunks[idx], game_state);
}

/// Compact combat scene for M tier: HP bars + sprite + status.
fn draw_combat_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Combat ");

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let is_regen = game_state.combat_state.is_regenerating;

    let mut constraints = vec![Constraint::Length(1)]; // Player HP
    if is_regen {
        constraints.push(Constraint::Length(1)); // Regen throbber
    }
    constraints.push(Constraint::Min(3)); // Sprite
    constraints.push(Constraint::Length(1)); // Enemy HP
    constraints.push(Constraint::Length(1)); // Status

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut idx = 0;
    draw_player_hp(frame, chunks[idx], game_state);
    idx += 1;

    if is_regen {
        draw_regen_throbber(frame, chunks[idx], game_state);
        idx += 1;
    }

    render_combat_3d(frame, chunks[idx], game_state);
    draw_floating_damage(frame, chunks[idx], game_state);
    idx += 1;

    draw_enemy_hp(frame, chunks[idx], game_state);
    idx += 1;

    draw_combat_status(frame, chunks[idx], game_state);
}

/// Draws the player HP bar (borderless, single line)
pub(super) fn draw_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
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

/// Draws a regen throbber line below the player HP bar (spinner + flavor text).
fn draw_regen_throbber(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use super::throbber::{regen_message, spinner_char};

    let spinner = spinner_char();
    let message = regen_message(game_state.character_xp);
    let text = Line::from(Span::styled(
        format!("{} {}", spinner, message),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::ITALIC),
    ));

    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Draws the enemy HP bar (borderless, single line) with zone-aware coloring
pub(super) fn draw_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let hp_ratio = enemy.current_hp as f64 / enemy.max_hp as f64;

        let label = format!("{}: {}/{}", enemy.name, enemy.current_hp, enemy.max_hp);

        let is_boss = game_state.zone_progression.fighting_boss;
        let is_dungeon_boss = enemy.name.starts_with("Boss ");
        let hp_color = if is_boss || is_dungeon_boss {
            Color::LightRed
        } else {
            let zone_id = game_state
                .active_dungeon
                .as_ref()
                .map(|d| d.zone_id)
                .unwrap_or(game_state.zone_progression.current_zone_id);
            zone_palette(zone_id).primary
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(hp_color).add_modifier(Modifier::BOLD))
            .label(label)
            .ratio(hp_ratio);

        frame.render_widget(gauge, area);
    }
}

/// Renders floating damage numbers over the sprite area.
/// Enemy damage floats UP from the bottom; player damage floats DOWN from the top.
/// Numbers fade from their original color to DarkGray as they travel.
fn draw_floating_damage(frame: &mut Frame, sprite_area: Rect, game_state: &GameState) {
    if sprite_area.height == 0 || sprite_area.width == 0 {
        return;
    }

    // Enemy damage floats UP from bottom of sprite area
    for float in &game_state.combat_state.enemy_damage_floats {
        let progress = 1.0 - (float.remaining / DAMAGE_FLASH_DURATION);
        let max_rise = (sprite_area.height as f64 * 0.15).max(1.0);
        let row_offset = (progress * max_rise) as u16;
        let y = sprite_area.bottom().saturating_sub(1 + row_offset);
        if y < sprite_area.y {
            continue;
        }
        render_float_text(frame, sprite_area, y, float, progress);
    }

    // Player damage floats DOWN from top of sprite area
    for float in &game_state.combat_state.player_damage_floats {
        let progress = 1.0 - (float.remaining / DAMAGE_FLASH_DURATION);
        let max_drop = (sprite_area.height as f64 * 0.15).max(1.0);
        let row_offset = (progress * max_drop) as u16;
        let y = sprite_area.y + row_offset;
        if y >= sprite_area.bottom() {
            continue;
        }
        render_float_text(frame, sprite_area, y, float, progress);
    }
}

/// Renders a single floating damage text at the given y position, right-aligned.
/// Fades the color based on progress (0.0 = fresh, 1.0 = expired).
fn render_float_text(
    frame: &mut Frame,
    sprite_area: Rect,
    y: u16,
    float: &crate::combat::types::DamageFlash,
    progress: f64,
) {
    let text_width = float.text.chars().count() as u16;
    if text_width == 0 || text_width >= sprite_area.width {
        return;
    }

    let style = if progress > 0.8 {
        // Final fade: dim gray
        Style::default().fg(Color::DarkGray)
    } else if progress > 0.6 {
        // Mid fade: original color, no bold
        Style::default().fg(float.color)
    } else {
        // Full intensity: original color + bold if applicable
        let mut s = Style::default().fg(float.color);
        if float.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    };

    let x = sprite_area.right().saturating_sub(text_width + 1);
    let rect = Rect::new(x, y, text_width, 1);
    let para = Paragraph::new(Span::styled(&float.text, style)).alignment(Alignment::Right);
    frame.render_widget(para, rect);
}

/// Draws the combat status information with DPS
pub(super) fn draw_combat_status(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use super::throbber::{spinner_char, waiting_message};
    let spinner = spinner_char();

    // Use cached derived stats (includes real enhancement levels)
    let derived = game_state.cached_derived_stats;
    let base_dps = derived.total_damage() as f64 / ATTACK_INTERVAL_SECONDS;
    let effective_dps = base_dps
        * (1.0 + (derived.crit_chance_percent as f64 / 100.0) * (derived.crit_multiplier - 1.0));
    let dps_span = Span::styled(
        format!(" | DPS: {:.0}", effective_dps),
        Style::default().fg(Color::DarkGray),
    );

    let status_text = if game_state.combat_state.current_enemy.is_some() {
        let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
        let player_next = (player_interval - game_state.combat_state.player_attack_timer).max(0.0);
        let enemy_interval = effective_enemy_attack_interval(game_state);
        let enemy_next = (enemy_interval - game_state.combat_state.enemy_attack_timer).max(0.0);

        let player_style = if player_next < 0.3 {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        let enemy_style = if enemy_next < 0.3 {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        vec![Line::from(vec![
            Span::styled(
                format!("{} In Combat", spinner),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(format!("You: {:.1}s", player_next), player_style),
            Span::raw("  "),
            Span::styled(format!("Foe: {:.1}s", enemy_next), enemy_style),
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
