use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Rect},
    style::Color,
    widgets::Paragraph,
    Frame,
};

use super::enemy_sprites::{
    detect_enemy_tier, get_sprite_for_enemy, zone_palette, EnemyTier, BOSS_CROWN, ZONE_BOSS_CROWN,
};
use super::scene_fx::{put_cell, render_buffer, SceneCell};

/// Returns the effective zone_id for the current combat context.
fn effective_zone_id(game_state: &GameState) -> u32 {
    game_state
        .active_dungeon
        .as_ref()
        .map(|d| d.zone_id)
        .unwrap_or(game_state.zone_progression.current_zone_id)
}

/// Eye characters that should be rendered in the secondary zone color.
const EYE_CHARS: &[char] = &['●', '◆'];

/// Renders the enemy sprite (borderless, no combat log)
pub fn render_combat_3d(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if area.height < 3 || area.width < 20 {
        let msg = Paragraph::new("Area too small").alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    render_simple_sprite(frame, area, game_state);
}

/// Renders a simple, centered enemy sprite with zone-based coloring,
/// two-tone eye rendering, tier decorations (crown), and tier-based name styling.
fn render_simple_sprite(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let width = area.width as usize;
    let height = area.height as usize;
    let zone_id = effective_zone_id(game_state);
    let mut buffer = vec![vec![SceneCell::default(); width]; height];

    super::zone_bg::paint_zone_scene(&mut buffer, zone_id);

    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let sprite_template = get_sprite_for_enemy(&enemy.name, zone_id);
        let sprite_art = sprite_template.base_art;
        let tier = detect_enemy_tier(game_state);
        let palette = zone_palette(zone_id);

        // Boss tiers get brighter bodies now that we render through a SceneCell buffer.
        let body_color = match tier {
            EnemyTier::Normal | EnemyTier::DungeonElite => palette.primary,
            EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss => palette.secondary,
            EnemyTier::ZoneBoss => Color::LightRed,
        };

        let available_height = area.height as usize;
        let sprite_height = sprite_art.lines().count();
        let has_crown = matches!(
            tier,
            EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss | EnemyTier::ZoneBoss
        );
        // crown (1) + sprite + blank (1) + name (1)
        let extra_lines = if has_crown { 3 } else { 2 };
        let total_content = sprite_height + extra_lines;
        let top_padding = (available_height.saturating_sub(total_content)) / 2;
        let mut row_cursor = top_padding as i32;

        if has_crown {
            let crown_text = if tier == EnemyTier::ZoneBoss {
                ZONE_BOSS_CROWN
            } else {
                BOSS_CROWN
            };
            write_centered_colored(&mut buffer, row_cursor, crown_text, |ch| {
                if ch == '\u{2605}' {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }
            });
            row_cursor += 1;
        }

        for line in sprite_art.lines() {
            write_sprite_line(&mut buffer, row_cursor, line, body_color, palette.secondary);
            row_cursor += 1;
        }

        row_cursor += 1;
        let name_style = match tier {
            EnemyTier::Normal => Color::Yellow,
            EnemyTier::DungeonElite => Color::LightRed,
            EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss => Color::White,
            EnemyTier::ZoneBoss => Color::LightRed,
        };
        write_centered_text(&mut buffer, row_cursor, &enemy.name, name_style);
    } else {
        use super::throbber::{spinner_char, waiting_message};

        let spinner = spinner_char();
        let message = waiting_message(game_state.character_xp);
        let text = format!("{} {}", spinner, message);
        write_centered_text(
            &mut buffer,
            (height / 2) as i32,
            &text,
            Color::Rgb(140, 146, 168),
        );
    }

    render_buffer(frame, area, &buffer);
}

fn write_sprite_line(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    line: &str,
    body_color: Color,
    eye_color: Color,
) {
    if buffer.is_empty() || row < 0 || row as usize >= buffer.len() {
        return;
    }

    let width = buffer[0].len();
    let text_width = line.chars().count();
    let start_col = (width.saturating_sub(text_width)) / 2;

    for (idx, ch) in line.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        let fg = if EYE_CHARS.contains(&ch) {
            eye_color
        } else {
            body_color
        };
        put_cell(buffer, row, (start_col + idx) as i32, ch, fg);
    }
}

fn write_centered_text(buffer: &mut [Vec<SceneCell>], row: i32, text: &str, fg: Color) {
    write_centered_colored(buffer, row, text, |_| fg);
}

fn write_centered_colored<F>(buffer: &mut [Vec<SceneCell>], row: i32, text: &str, mut color: F)
where
    F: FnMut(char) -> Color,
{
    if buffer.is_empty() || row < 0 || row as usize >= buffer.len() {
        return;
    }

    let width = buffer[0].len();
    let text_width = text.chars().count();
    let start_col = (width.saturating_sub(text_width)) / 2;

    for (idx, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        put_cell(buffer, row, (start_col + idx) as i32, ch, color(ch));
    }
}
