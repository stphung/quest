use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Rect},
    style::Color,
    widgets::Paragraph,
    Frame,
};

use super::enemy_sprites::{
    detect_enemy_tier, get_archetype_for_enemy, sprite_color_palette, EnemyTier, PixelSprite,
    SpriteColorPalette, BOSS_CROWN, ZONE_BOSS_CROWN,
};
use super::scene_fx::{render_buffer, with_scene_buffer, SceneCell};

/// Returns the effective zone_id for the current combat context.
fn effective_zone_id(game_state: &GameState) -> u32 {
    game_state
        .active_dungeon
        .as_ref()
        .map(|d| d.zone_id)
        .unwrap_or(game_state.zone_progression.current_zone_id)
}

/// Renders the enemy sprite (borderless, no combat log)
pub fn render_combat_3d(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if area.height < 3 || area.width < 20 {
        let msg = Paragraph::new("Area too small").alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    render_simple_sprite(frame, area, game_state);
}

/// Renders a pixel-art enemy sprite with zone-based coloring,
/// half-block characters (▀/▄/█) for doubled vertical resolution,
/// tier decorations (crown), and tier-based name styling.
fn render_simple_sprite(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let width = area.width as usize;
    let height = area.height as usize;
    let zone_id = effective_zone_id(game_state);

    with_scene_buffer(width, height, |buffer| {
        super::zone_bg::paint_zone_scene(buffer, zone_id);

        if let Some(enemy) = &game_state.combat_state.current_enemy {
            let tier = detect_enemy_tier(game_state);
            let archetype = get_archetype_for_enemy(&enemy.name, zone_id);
            let pixel_sprite = archetype.pixel_sprite();
            let palette = sprite_color_palette(zone_id, tier);

            // Each pair of pixel rows renders to one cell row.
            let pixel_row_count = pixel_sprite.rows.len();
            let cell_rows = pixel_row_count.div_ceil(2);

            let has_crown = matches!(
                tier,
                EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss | EnemyTier::ZoneBoss
            );
            // crown (1) + sprite_cell_rows + blank (1) + name (1)
            let extra_lines = if has_crown { 3 } else { 2 };
            let total_content = cell_rows + extra_lines;
            let top_padding = (height.saturating_sub(total_content)) / 2;
            let mut row_cursor = top_padding as i32;

            if has_crown {
                let crown_text = if tier == EnemyTier::ZoneBoss {
                    ZONE_BOSS_CROWN
                } else {
                    BOSS_CROWN
                };
                write_centered_colored(buffer, row_cursor, crown_text, |ch| {
                    if ch == '\u{2605}' {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }
                });
                row_cursor += 1;
            }

            render_pixel_sprite(buffer, row_cursor, pixel_sprite, &palette);
            row_cursor += cell_rows as i32;

            row_cursor += 1;
            let name_style = match tier {
                EnemyTier::Normal => Color::Yellow,
                EnemyTier::DungeonElite => Color::LightRed,
                EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss => Color::White,
                EnemyTier::ZoneBoss => Color::LightRed,
            };
            write_centered_text(buffer, row_cursor, &enemy.name, name_style);
        } else {
            use super::throbber::{spinner_char, waiting_message};

            let spinner = spinner_char();
            let message = waiting_message(game_state.character_xp);
            let text = format!("{} {}", spinner, message);
            write_centered_text(
                buffer,
                (height / 2) as i32,
                &text,
                Color::Rgb(140, 146, 168),
            );
        }

        render_buffer(frame, area, buffer);
    });
}

/// Maps a pixel format character to the corresponding `SpriteColorPalette` color.
/// Returns `None` for transparent pixels (`.`).
fn pixel_color(pixel: char, palette: &SpriteColorPalette) -> Option<Color> {
    match pixel {
        'D' => Some(palette.dark),
        'B' => Some(palette.body),
        'S' => Some(palette.shade),
        'E' => Some(palette.eye),
        'H' => Some(palette.highlight),
        _ => None, // '.' = transparent
    }
}

/// Renders a `PixelSprite` into the scene buffer using half-block characters.
///
/// Processes pixel rows in pairs (row 0+1, row 2+3, …).  For each column pair:
///   - Both transparent → skip (keep background cell as-is)
///   - Top opaque, bottom transparent → `▀` fg=top_color bg=background_bg
///   - Top transparent, bottom opaque → `▄` fg=bottom_color bg=background_bg
///   - Both opaque → `▀` fg=top_color bg=bottom_color
///
/// For an odd-height sprite the last unpaired pixel row is treated as top-only.
fn render_pixel_sprite(
    buffer: &mut [Vec<SceneCell>],
    start_row: i32,
    sprite: &PixelSprite,
    palette: &SpriteColorPalette,
) {
    if buffer.is_empty() {
        return;
    }
    let buf_width = buffer[0].len();
    let pixel_rows = sprite.rows;
    let num_pixel_rows = pixel_rows.len();

    // Determine max pixel width across all rows for centering.
    let max_pixel_width = pixel_rows
        .iter()
        .map(|r| r.chars().count())
        .max()
        .unwrap_or(0);

    let mut cell_row: i32 = 0;
    let mut i = 0;
    while i < num_pixel_rows {
        let top_row_str = pixel_rows[i];
        let bottom_row_str = if i + 1 < num_pixel_rows {
            pixel_rows[i + 1]
        } else {
            "" // sentinel empty row for odd-height sprites
        };

        let dest_row = start_row + cell_row;
        if dest_row >= 0 && (dest_row as usize) < buffer.len() {
            let dest_row = dest_row as usize;

            // Center relative to max_pixel_width
            let start_col = (buf_width.saturating_sub(max_pixel_width)) / 2;

            // Collect pixels for this pair into aligned arrays
            let top_pixels: Vec<char> = top_row_str.chars().collect();
            let bot_pixels: Vec<char> = bottom_row_str.chars().collect();

            for col_idx in 0..max_pixel_width {
                let buf_col = start_col + col_idx;
                if buf_col >= buf_width {
                    break;
                }

                let top_px = top_pixels.get(col_idx).copied().unwrap_or('.');
                let bot_px = bot_pixels.get(col_idx).copied().unwrap_or('.');

                let top_color = pixel_color(top_px, palette);
                let bot_color = pixel_color(bot_px, palette);

                match (top_color, bot_color) {
                    (None, None) => {
                        // Both transparent – leave background as-is
                    }
                    (Some(fg), None) => {
                        // Top opaque, bottom transparent → ▀ upper half-block
                        let bg = buffer[dest_row][buf_col].bg;
                        buffer[dest_row][buf_col] = SceneCell::new('\u{2580}', fg, bg);
                    }
                    (None, Some(fg)) => {
                        // Top transparent, bottom opaque → ▄ lower half-block
                        let bg = buffer[dest_row][buf_col].bg;
                        buffer[dest_row][buf_col] = SceneCell::new('\u{2584}', fg, bg);
                    }
                    (Some(top_fg), Some(bot_fg)) => {
                        // Both opaque → ▀ with top as fg, bottom as bg
                        buffer[dest_row][buf_col] = SceneCell::new('\u{2580}', top_fg, bot_fg);
                    }
                }
            }
        }

        i += 2;
        cell_row += 1;
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
        let col = start_col + idx;
        if col >= width {
            break;
        }
        let row_u = row as usize;
        let fg = color(ch);
        let bg = buffer[row_u][col].bg;
        buffer[row_u][col] = SceneCell::new(ch, fg, bg);
    }
}
