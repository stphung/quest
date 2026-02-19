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
use super::scene_fx::{
    clamp_u8, current_millis, hash2d, lerp_rgb, lift_rgb, put_cell, render_buffer, SceneCell,
};

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

/// Zone backdrop palette and glyph motif.
#[derive(Clone, Copy)]
struct ZoneBackdropTheme {
    top: (u8, u8, u8),
    bottom: (u8, u8, u8),
    dim_fg: (u8, u8, u8),
    bright_fg: (u8, u8, u8),
    glyph_a: char,
    glyph_b: char,
    noise_freq_x: f64,
    noise_freq_y: f64,
    noise_speed_x: f64,
    noise_speed_y: f64,
    jitter_scale: f64,
    band_wave_freq: f64,
    band_speed: f64,
    band_amp: f64,
    haze_mod: u32,
    particle_mod: u32,
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

/// Renders a simple, centered enemy sprite with zone-based coloring,
/// two-tone eye rendering, tier decorations (crown), and tier-based name styling.
fn render_simple_sprite(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let width = area.width as usize;
    let height = area.height as usize;
    let zone_id = effective_zone_id(game_state);
    let mut buffer = vec![vec![SceneCell::default(); width]; height];

    paint_zone_background(&mut buffer, zone_id);

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

fn paint_zone_background(buffer: &mut [Vec<SceneCell>], zone_id: u32) {
    if buffer.is_empty() || buffer[0].is_empty() {
        return;
    }

    let theme = zone_backdrop_theme(zone_id);
    let millis = current_millis() as f64;
    let phase = (millis / 180.0) as usize;
    let height = buffer.len();
    let width = buffer[0].len();
    let lifted_top = lift_rgb(theme.top, 30);
    let lifted_bottom = lift_rgb(theme.bottom, 24);
    let lifted_dim_fg = lift_rgb(theme.dim_fg, 34);
    let lifted_bright_fg = lift_rgb(theme.bright_fg, 28);

    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let row_t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let base_rgb = lerp_rgb(lifted_top, lifted_bottom, row_t.powf(0.86));

        for (col, cell) in row_cells.iter_mut().enumerate() {
            let drift = (col as f64 * theme.noise_freq_x
                + millis * theme.noise_speed_x
                + zone_id as f64 * 0.63)
                .sin()
                + (row as f64 * theme.noise_freq_y - millis * theme.noise_speed_y
                    + zone_id as f64 * 0.29)
                    .cos();
            let jitter = ((drift * theme.jitter_scale).round() as i16).clamp(-6, 14);
            let r = clamp_u8(base_rgb.0 as i16 + jitter);
            let g = clamp_u8(base_rgb.1 as i16 + jitter);
            let b = clamp_u8(base_rgb.2 as i16 + jitter);
            cell.bg = Color::Rgb(r, g, b);
        }
    }

    let flow = millis * theme.band_speed;
    for band in 0..3usize {
        let anchor = (height as f64 * (0.26 + band as f64 * 0.24)).round();
        for col in 0..width {
            let wave = (col as f64 * theme.band_wave_freq
                + flow * (1.0 + band as f64 * 0.18)
                + band as f64 * 0.7)
                .sin();
            let row = (anchor + wave * (height as f64 * theme.band_amp)) as i32;
            if row < 0 || row >= height as i32 {
                continue;
            }
            if hash2d(row as usize + phase + band * 7, col + zone_id as usize * 13)
                .is_multiple_of(theme.haze_mod)
            {
                let haze = if band == 1 {
                    theme.glyph_b
                } else {
                    theme.glyph_a
                };
                put_cell(
                    buffer,
                    row,
                    col as i32,
                    haze,
                    Color::Rgb(lifted_dim_fg.0, lifted_dim_fg.1, lifted_dim_fg.2),
                );
            }
        }
    }

    for row in 0..height {
        for col in 0..width {
            let seed = hash2d(row + zone_id as usize * 17, col + zone_id as usize * 31);
            if !seed.is_multiple_of(theme.particle_mod) {
                continue;
            }
            let bright = hash2d(row + phase, col + phase).is_multiple_of(3);
            let fg = if bright {
                Color::Rgb(lifted_bright_fg.0, lifted_bright_fg.1, lifted_bright_fg.2)
            } else {
                Color::Rgb(lifted_dim_fg.0, lifted_dim_fg.1, lifted_dim_fg.2)
            };
            let ch = if bright { theme.glyph_b } else { theme.glyph_a };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
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

fn zone_backdrop_theme(zone_id: u32) -> ZoneBackdropTheme {
    match zone_id {
        1 => ZoneBackdropTheme {
            top: (12, 28, 18),
            bottom: (7, 16, 11),
            dim_fg: (34, 78, 48),
            bright_fg: (62, 114, 76),
            glyph_a: '.',
            glyph_b: '\'',
            noise_freq_x: 0.11,
            noise_freq_y: 0.15,
            noise_speed_x: 0.00042,
            noise_speed_y: 0.00031,
            jitter_scale: 2.6,
            band_wave_freq: 0.13,
            band_speed: 0.00018,
            band_amp: 0.040,
            haze_mod: 5,
            particle_mod: 185,
        },
        2 => ZoneBackdropTheme {
            top: (13, 24, 18),
            bottom: (7, 13, 10),
            dim_fg: (30, 62, 44),
            bright_fg: (52, 92, 66),
            glyph_a: '^',
            glyph_b: '.',
            noise_freq_x: 0.09,
            noise_freq_y: 0.27,
            noise_speed_x: 0.00030,
            noise_speed_y: 0.00052,
            jitter_scale: 2.0,
            band_wave_freq: 0.09,
            band_speed: 0.00012,
            band_amp: 0.025,
            haze_mod: 3,
            particle_mod: 120,
        },
        3 => ZoneBackdropTheme {
            top: (16, 20, 30),
            bottom: (8, 10, 18),
            dim_fg: (52, 62, 88),
            bright_fg: (76, 88, 116),
            glyph_a: '.',
            glyph_b: '*',
            noise_freq_x: 0.18,
            noise_freq_y: 0.10,
            noise_speed_x: 0.00068,
            noise_speed_y: 0.00026,
            jitter_scale: 3.8,
            band_wave_freq: 0.22,
            band_speed: 0.00028,
            band_amp: 0.060,
            haze_mod: 6,
            particle_mod: 170,
        },
        4 => ZoneBackdropTheme {
            top: (18, 12, 24),
            bottom: (9, 6, 14),
            dim_fg: (64, 42, 80),
            bright_fg: (92, 60, 112),
            glyph_a: ';',
            glyph_b: '+',
            noise_freq_x: 0.15,
            noise_freq_y: 0.22,
            noise_speed_x: 0.00048,
            noise_speed_y: 0.00044,
            jitter_scale: 3.1,
            band_wave_freq: 0.19,
            band_speed: 0.00024,
            band_amp: 0.045,
            haze_mod: 4,
            particle_mod: 145,
        },
        5 => ZoneBackdropTheme {
            top: (26, 12, 10),
            bottom: (14, 6, 4),
            dim_fg: (90, 48, 34),
            bright_fg: (130, 72, 46),
            glyph_a: ':',
            glyph_b: '`',
            noise_freq_x: 0.22,
            noise_freq_y: 0.16,
            noise_speed_x: 0.00088,
            noise_speed_y: 0.00033,
            jitter_scale: 4.4,
            band_wave_freq: 0.27,
            band_speed: 0.00042,
            band_amp: 0.055,
            haze_mod: 5,
            particle_mod: 132,
        },
        6 => ZoneBackdropTheme {
            top: (9, 20, 32),
            bottom: (5, 10, 18),
            dim_fg: (56, 80, 112),
            bright_fg: (88, 116, 148),
            glyph_a: '.',
            glyph_b: '*',
            noise_freq_x: 0.12,
            noise_freq_y: 0.14,
            noise_speed_x: 0.00035,
            noise_speed_y: 0.00021,
            jitter_scale: 2.8,
            band_wave_freq: 0.11,
            band_speed: 0.00014,
            band_amp: 0.070,
            haze_mod: 7,
            particle_mod: 210,
        },
        7 => ZoneBackdropTheme {
            top: (14, 16, 30),
            bottom: (8, 10, 20),
            dim_fg: (60, 68, 118),
            bright_fg: (86, 98, 154),
            glyph_a: '=',
            glyph_b: ':',
            noise_freq_x: 0.20,
            noise_freq_y: 0.19,
            noise_speed_x: 0.00054,
            noise_speed_y: 0.00048,
            jitter_scale: 3.5,
            band_wave_freq: 0.24,
            band_speed: 0.00032,
            band_amp: 0.050,
            haze_mod: 5,
            particle_mod: 160,
        },
        8 => ZoneBackdropTheme {
            top: (6, 12, 28),
            bottom: (3, 8, 18),
            dim_fg: (36, 62, 108),
            bright_fg: (60, 90, 140),
            glyph_a: '~',
            glyph_b: '.',
            noise_freq_x: 0.08,
            noise_freq_y: 0.25,
            noise_speed_x: 0.00026,
            noise_speed_y: 0.00057,
            jitter_scale: 2.4,
            band_wave_freq: 0.07,
            band_speed: 0.00010,
            band_amp: 0.080,
            haze_mod: 4,
            particle_mod: 140,
        },
        9 => ZoneBackdropTheme {
            top: (20, 22, 30),
            bottom: (10, 12, 18),
            dim_fg: (72, 82, 104),
            bright_fg: (106, 118, 142),
            glyph_a: '.',
            glyph_b: '*',
            noise_freq_x: 0.26,
            noise_freq_y: 0.09,
            noise_speed_x: 0.00072,
            noise_speed_y: 0.00020,
            jitter_scale: 3.6,
            band_wave_freq: 0.30,
            band_speed: 0.00040,
            band_amp: 0.045,
            haze_mod: 6,
            particle_mod: 168,
        },
        10 => ZoneBackdropTheme {
            top: (30, 24, 12),
            bottom: (16, 12, 5),
            dim_fg: (104, 84, 30),
            bright_fg: (146, 120, 46),
            glyph_a: ':',
            glyph_b: '.',
            noise_freq_x: 0.17,
            noise_freq_y: 0.13,
            noise_speed_x: 0.00060,
            noise_speed_y: 0.00029,
            jitter_scale: 3.0,
            band_wave_freq: 0.20,
            band_speed: 0.00026,
            band_amp: 0.042,
            haze_mod: 5,
            particle_mod: 150,
        },
        11 => ZoneBackdropTheme {
            top: (26, 9, 18),
            bottom: (13, 4, 11),
            dim_fg: (116, 44, 70),
            bright_fg: (160, 62, 96),
            glyph_a: '*',
            glyph_b: ':',
            noise_freq_x: 0.24,
            noise_freq_y: 0.21,
            noise_speed_x: 0.00096,
            noise_speed_y: 0.00070,
            jitter_scale: 4.8,
            band_wave_freq: 0.32,
            band_speed: 0.00058,
            band_amp: 0.065,
            haze_mod: 3,
            particle_mod: 118,
        },
        _ => ZoneBackdropTheme {
            top: (16, 12, 12),
            bottom: (8, 8, 8),
            dim_fg: (78, 56, 56),
            bright_fg: (110, 72, 72),
            glyph_a: '.',
            glyph_b: ':',
            noise_freq_x: 0.14,
            noise_freq_y: 0.18,
            noise_speed_x: 0.00048,
            noise_speed_y: 0.00036,
            jitter_scale: 3.0,
            band_wave_freq: 0.17,
            band_speed: 0.00022,
            band_amp: 0.045,
            haze_mod: 5,
            particle_mod: 151,
        },
    }
}
