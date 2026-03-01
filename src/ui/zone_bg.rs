//! Zone background rendering system.
//!
//! Provides a 6-layer compositing pipeline that paints rich, animated scenery
//! behind the combat sprite for each of the 11 game zones (plus a fallback).
//!
//! Layers (painted bottom-to-top):
//!   1. Sky gradient   -- vertical colour ramp with per-cell noise
//!   2. Celestial      -- sun / moon / stars / wisps / bioluminescence / etc.
//!   3. Far terrain    -- distant silhouette from a sine-wave profile
//!   4. Near terrain   -- foreground silhouette (taller, denser)
//!   5. Ground detail  -- sparse glyphs near the bottom
//!   6. Weather        -- animated particles (snow, embers, sparks, etc.)
//!
//! An optional per-zone overlay function runs last to add effects like
//! lightning flashes or pulsing void colours.

use ratatui::style::Color;

use super::scene_fx::{clamp_u8, current_millis, hash2d, lerp_rgb, put_cell, SceneCell};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Describes one terrain silhouette layer.
#[derive(Clone, Copy)]
struct TerrainProfile {
    /// Glyph used for the silhouette line (and fill if enabled).
    glyph: char,
    /// Foreground colour of the glyph.
    color: (u8, u8, u8),
    /// Base height of the silhouette as a fraction of buffer height (0.0 = top, 1.0 = bottom).
    base_height: f64,
    /// Amplitude of the primary sine wave (fraction of buffer height).
    amplitude: f64,
    /// Frequency multiplier for the primary sine wave.
    frequency: f64,
    /// Horizontal drift speed (millis divisor).
    speed: f64,
    /// If true, fill all rows from the silhouette line down to the bottom.
    fill: bool,
}

/// Overlay function signature: receives the scene buffer and current millis.
type OverlayFn = fn(&mut [Vec<SceneCell>], f64);

/// What to render in the sky above the terrain.
#[derive(Clone, Copy, PartialEq)]
enum CelestialType {
    Sun,
    Moon,
    Stars,
    Wisps,
    BioLuminescent,
    Overcast,
    EmberGlow,
    CrackedSky,
    VoidRift,
    Flicker,
    None,
}

/// Animated particle overlay type.
#[derive(Clone, Copy, PartialEq)]
enum WeatherType {
    None,
    Snow,
    Embers,
    Sparks,
    Bubbles,
    VoidParticles,
    WindStreaks,
    Sparkles,
    AshRain,
    GlassShards,
    DriftingAsh,
    DustMotes,
    StaticNoise,
    FractureMotes,
}

/// Full scene configuration for a single zone.
struct ZoneSceneConfig {
    /// (top_rgb, bottom_rgb) for the sky gradient.
    sky_top: (u8, u8, u8),
    sky_bottom: (u8, u8, u8),
    celestial: CelestialType,
    far_terrain: TerrainProfile,
    near_terrain: TerrainProfile,
    /// Glyphs scattered in the bottom portion of the scene.
    ground_glyphs: &'static [char],
    ground_color: (u8, u8, u8),
    weather: WeatherType,
    /// Extra weather intensity multiplier (e.g. heavy snow = 2.0).
    weather_intensity: f64,
    /// Optional post-processing overlay.
    overlay: Option<OverlayFn>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Paints a full 6-layer zone background into `buffer`.
///
/// Call this before drawing the enemy sprite so the sprite composites on top.
pub fn paint_zone_scene(buffer: &mut [Vec<SceneCell>], zone_id: u32) {
    if buffer.is_empty() || buffer[0].is_empty() {
        return;
    }

    let config = zone_scene_config(zone_id);
    let millis = current_millis() as f64;

    // Layer 1: sky gradient
    paint_sky_gradient(buffer, &config, millis);

    // Layer 2: celestial elements
    paint_celestial(buffer, &config, millis);

    // Layer 3: far terrain
    paint_terrain(buffer, &config.far_terrain, millis);

    // Layer 4: near terrain
    paint_terrain(buffer, &config.near_terrain, millis);

    // Layer 5: ground detail
    paint_ground_detail(buffer, &config, millis);

    // Layer 6: weather
    paint_weather(buffer, &config, millis);

    // Optional overlay
    if let Some(overlay_fn) = config.overlay {
        overlay_fn(buffer, millis);
    }
}

// ---------------------------------------------------------------------------
// Layer 1: Sky Gradient
// ---------------------------------------------------------------------------

fn paint_sky_gradient(buffer: &mut [Vec<SceneCell>], config: &ZoneSceneConfig, millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();

    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let base = lerp_rgb(config.sky_top, config.sky_bottom, t.powf(0.85));

        for (col, cell) in row_cells.iter_mut().enumerate().take(width) {
            // Per-cell noise for subtle variation
            let drift = (col as f64 * 0.13 + millis * 0.00038 + row as f64 * 0.09).sin()
                + (row as f64 * 0.17 - millis * 0.00027 + col as f64 * 0.07).cos();
            let jitter = (drift * 2.8).round() as i16;
            let r = clamp_u8(base.0 as i16 + jitter);
            let g = clamp_u8(base.1 as i16 + jitter);
            let b = clamp_u8(base.2 as i16 + jitter);
            cell.bg = Color::Rgb(r, g, b);
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 2: Celestial
// ---------------------------------------------------------------------------

fn paint_celestial(buffer: &mut [Vec<SceneCell>], config: &ZoneSceneConfig, millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();

    match config.celestial {
        CelestialType::Sun => {
            paint_sun(buffer, width, height, millis);
        }
        CelestialType::Moon => {
            paint_moon(buffer, width, height, millis);
        }
        CelestialType::Stars => {
            paint_stars(buffer, width, height, millis, false);
        }
        CelestialType::Wisps => {
            paint_wisps(buffer, width, height, millis);
        }
        CelestialType::BioLuminescent => {
            paint_bioluminescent(buffer, width, height, millis);
        }
        CelestialType::Overcast => {
            paint_overcast(buffer, width, height, millis);
        }
        CelestialType::EmberGlow => {
            paint_emberglow(buffer, width, height, millis);
        }
        CelestialType::CrackedSky => {
            paint_cracked_sky(buffer, width, height, millis);
        }
        CelestialType::VoidRift => {
            paint_void_rift(buffer, width, height, millis);
        }
        CelestialType::Flicker => {
            paint_flicker(buffer, width, height, millis);
        }
        CelestialType::None => {}
    }
}

fn paint_sun(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 1200.0;
    let cx = ((width as f64 * 0.75) + (t * 0.04).sin() * 3.0).round() as i32;
    let cy = ((height as f64 * 0.22) + (t * 0.03).sin() * 1.5).round() as i32;

    // Sun glow (halo)
    for &(dx, dy, ch, r, g, b) in &[
        (-1, 0, '\u{00b7}', 255u8, 240u8, 180u8), // middle-dot left
        (1, 0, '\u{00b7}', 255, 240, 180),        // middle-dot right
        (0, -1, '\u{00b7}', 255, 236, 170),       // middle-dot up
        (0, 1, '\u{00b7}', 255, 236, 170),        // middle-dot down
        (-2, 0, '.', 255, 220, 140),
        (2, 0, '.', 255, 220, 140),
    ] {
        put_cell(buffer, cy + dy, cx + dx, ch, Color::Rgb(r, g, b));
    }
    // Sun body
    put_cell(buffer, cy, cx, '\u{25cf}', Color::Rgb(255, 230, 160)); // ●
}

fn paint_moon(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 1800.0;
    let cx = ((width as f64 * 0.8) + (t * 0.025).sin() * 2.0).round() as i32;
    let cy = ((height as f64 * 0.18) + (t * 0.02).sin()).round() as i32;

    // Moon glow
    for &(dx, dy, ch, r, g, b) in &[
        (-1, 0, '\u{00b7}', 200u8, 210u8, 240u8),
        (1, 0, '\u{00b7}', 200, 210, 240),
        (0, -1, '.', 180, 190, 220),
    ] {
        put_cell(buffer, cy + dy, cx + dx, ch, Color::Rgb(r, g, b));
    }
    // Moon body (half moon)
    put_cell(buffer, cy, cx, '\u{25d1}', Color::Rgb(220, 230, 255)); // ◑
}

fn paint_stars(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    millis: f64,
    sparse: bool,
) {
    let twinkle_tick = (millis / 320.0) as usize;
    let threshold = if sparse { 157 } else { 89 };
    let sky_limit = (height as f64 * 0.55) as usize;

    for row in 0..sky_limit.min(height) {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            if !hash2d(row, col).is_multiple_of(threshold) {
                continue;
            }
            let bright = hash2d(row + twinkle_tick, col + twinkle_tick).is_multiple_of(3);
            let (ch, fg) = if bright {
                ('*', Color::Rgb(244, 246, 255))
            } else {
                ('.', Color::Rgb(160, 168, 210))
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_wisps(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 600.0;
    let sky_limit = (height as f64 * 0.5) as usize;

    for i in 0..5u32 {
        let base_x = i as f64 * width as f64 / 5.0 + 3.0;
        let base_y = i as f64 * sky_limit as f64 / 5.0 + 2.0;
        let x = (base_x + (t * 0.12 + i as f64 * 1.3).sin() * 4.0).round() as i32;
        let y = (base_y + (t * 0.08 + i as f64 * 0.9).cos() * 2.0).round() as i32;

        let phase = (t * 0.15 + i as f64 * 0.7).sin() * 0.5 + 0.5;
        let brightness = (140.0 + phase * 80.0) as u8;
        let fg = Color::Rgb(brightness, brightness / 2, brightness.saturating_add(20));

        put_cell(buffer, y, x, '\u{2726}', fg); // ✦
        put_cell(
            buffer,
            y,
            x - 1,
            '\u{00b7}',
            Color::Rgb(
                brightness / 2,
                brightness / 3,
                (brightness / 2).saturating_add(10),
            ),
        );
        put_cell(
            buffer,
            y,
            x + 1,
            '\u{00b7}',
            Color::Rgb(
                brightness / 2,
                brightness / 3,
                (brightness / 2).saturating_add(10),
            ),
        );
    }
}

fn paint_bioluminescent(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 500.0;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            if !hash2d(row + 7, col + 13).is_multiple_of(127) {
                continue;
            }
            let phase = (t * 0.09 + row as f64 * 0.3 + col as f64 * 0.2).sin();
            if phase < 0.3 {
                continue;
            }
            let brightness = (60.0 + phase * 100.0) as u8;
            let fg = Color::Rgb(brightness / 3, brightness, brightness.saturating_add(30));
            let ch = if phase > 0.7 { '\u{2022}' } else { '\u{00b7}' }; // bullet or middle-dot
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_overcast(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 400.0;
    let cloud_rows = (height as f64 * 0.4) as usize;

    for row in 0..cloud_rows.min(height) {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let wave = (col as f64 * 0.08 + t * 0.03 + row as f64 * 0.15).sin();
            if wave < 0.4 {
                continue;
            }
            if !hash2d(row + 3, col + 5).is_multiple_of(4) {
                continue;
            }
            let shade = (160.0 + wave * 40.0) as u8;
            put_cell(
                buffer,
                row as i32,
                col as i32,
                '~',
                Color::Rgb(shade, shade, shade.saturating_add(8)),
            );
        }
    }
}

fn paint_emberglow(buffer: &mut [Vec<SceneCell>], _width: usize, height: usize, millis: f64) {
    let t = millis / 800.0;
    // Pulsing warm glow in lower sky region
    let glow_start = (height as f64 * 0.3) as usize;

    for (row_idx, row_cells) in buffer
        .iter_mut()
        .enumerate()
        .skip(glow_start)
        .take(height - glow_start)
    {
        let row_t = (row_idx - glow_start) as f64 / (height - glow_start).max(1) as f64;
        let pulse = (t * 0.2 + row_t * 2.0).sin() * 0.5 + 0.5;
        let intensity = (pulse * 20.0 * row_t) as i16;

        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + intensity),
                    clamp_u8(g as i16 + intensity / 3),
                    b,
                );
            }
        }
    }
}

fn paint_cracked_sky(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 1500.0;
    let sky_limit = (height as f64 * 0.5) as usize;

    // Draw 3-4 fracture lines across the sky
    for i in 0..4u32 {
        let base_x = (i as f64 * width as f64 / 4.0 + width as f64 * 0.1) as i32;
        let base_y = (i as f64 * sky_limit as f64 / 6.0 + 1.0) as i32;

        let pulse = (t * 0.12 + i as f64 * 1.7).sin() * 0.5 + 0.5;
        let brightness = (120.0 + pulse * 135.0) as u8;
        let fg = Color::Rgb(brightness, brightness.saturating_sub(20), brightness.saturating_sub(60));

        // Main fracture point
        put_cell(buffer, base_y, base_x, '/', fg);
        put_cell(buffer, base_y, base_x + 1, '\\', fg);

        // Branch lines
        let dim = Color::Rgb(brightness / 2, brightness / 2, brightness.saturating_sub(40) / 2);
        put_cell(buffer, base_y - 1, base_x - 1, '/', dim);
        put_cell(buffer, base_y + 1, base_x + 2, '\\', dim);

        // Bright core glyph
        if pulse > 0.6 {
            put_cell(buffer, base_y, base_x, '\u{2726}', Color::Rgb(255, 240, 200)); // ✦
        }
    }
}

fn paint_void_rift(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let t = millis / 2000.0;
    let cx = (width as f64 * 0.5 + (t * 0.02).sin() * 3.0).round() as i32;
    let cy = (height as f64 * 0.2 + (t * 0.015).sin() * 1.5).round() as i32;

    // Rift edges (purple glow)
    let glow_pulse = (t * 0.1).sin() * 0.3 + 0.7;
    let glow_brightness = (80.0 * glow_pulse) as u8;
    let edge_fg = Color::Rgb(glow_brightness.saturating_add(40), glow_brightness / 4, glow_brightness);

    for dx in -3..=3i32 {
        put_cell(buffer, cy - 1, cx + dx, '\u{2500}', edge_fg); // ─
        put_cell(buffer, cy + 1, cx + dx, '\u{2500}', edge_fg); // ─
    }

    // Rift interior (dark void)
    for dx in -2..=2i32 {
        let void_fg = Color::Rgb(15, 5, 20);
        put_cell(buffer, cy, cx + dx, '\u{2591}', void_fg); // ░
    }

    // Corner accents
    put_cell(buffer, cy - 1, cx - 3, '\u{256d}', edge_fg); // ╭
    put_cell(buffer, cy - 1, cx + 3, '\u{256e}', edge_fg); // ╮
    put_cell(buffer, cy + 1, cx - 3, '\u{2570}', edge_fg); // ╰
    put_cell(buffer, cy + 1, cx + 3, '\u{256f}', edge_fg); // ╯
}

fn paint_flicker(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {
    let flicker_tick = (millis / 150.0) as usize;
    let sky_limit = (height as f64 * 0.6) as usize;

    for row in 0..sky_limit.min(height) {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            if !hash2d(row + 19, col + 37).is_multiple_of(157) {
                continue;
            }
            // Flicker: visible only on certain ticks
            let visible = hash2d(row + flicker_tick, col + flicker_tick * 3).is_multiple_of(5);
            if !visible {
                continue;
            }
            let bright = hash2d(row + flicker_tick / 2, col).is_multiple_of(3);
            let (ch, fg) = if bright {
                ('*', Color::Rgb(200, 180, 220))
            } else {
                ('.', Color::Rgb(100, 80, 130))
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 3 & 4: Terrain Silhouettes
// ---------------------------------------------------------------------------

fn paint_terrain(buffer: &mut [Vec<SceneCell>], profile: &TerrainProfile, millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let t = millis * profile.speed;

    let fg = Color::Rgb(profile.color.0, profile.color.1, profile.color.2);

    for col in 0..width {
        // Primary sine wave
        let primary = (col as f64 * profile.frequency + t).sin();
        // Secondary harmonic for variety
        let secondary = (col as f64 * profile.frequency * 2.3 + t * 1.4).sin() * 0.35;
        let wave = primary + secondary;

        let silhouette_row =
            (height as f64 * profile.base_height + wave * height as f64 * profile.amplitude) as i32;

        if profile.fill {
            // Fill from silhouette line to bottom
            for row in silhouette_row.max(0) as usize..height {
                if col < buffer[row].len() {
                    put_cell(buffer, row as i32, col as i32, profile.glyph, fg);
                }
            }
        } else {
            // Just draw the silhouette line itself
            put_cell(buffer, silhouette_row, col as i32, profile.glyph, fg);
            // Add a second line below for thickness
            put_cell(buffer, silhouette_row + 1, col as i32, profile.glyph, fg);
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 5: Ground Detail
// ---------------------------------------------------------------------------

fn paint_ground_detail(buffer: &mut [Vec<SceneCell>], config: &ZoneSceneConfig, millis: f64) {
    if config.ground_glyphs.is_empty() {
        return;
    }

    let height = buffer.len();
    let width = buffer[0].len();
    let ground_start = (height as f64 * 0.78) as usize;
    let phase = (millis / 240.0) as usize;

    let fg = Color::Rgb(
        config.ground_color.0,
        config.ground_color.1,
        config.ground_color.2,
    );

    for row in ground_start..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            // Only overwrite empty cells
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let seed = hash2d(row + phase / 80, col);
            if !seed.is_multiple_of(17) {
                continue;
            }
            let glyph_idx = seed as usize % config.ground_glyphs.len();
            put_cell(
                buffer,
                row as i32,
                col as i32,
                config.ground_glyphs[glyph_idx],
                fg,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 6: Weather
// ---------------------------------------------------------------------------

fn paint_weather(buffer: &mut [Vec<SceneCell>], config: &ZoneSceneConfig, millis: f64) {
    match config.weather {
        WeatherType::None => {}
        WeatherType::Snow => paint_snow(buffer, millis, config.weather_intensity),
        WeatherType::Embers => paint_embers(buffer, millis, config.weather_intensity),
        WeatherType::Sparks => paint_sparks(buffer, millis, config.weather_intensity),
        WeatherType::Bubbles => paint_bubbles(buffer, millis, config.weather_intensity),
        WeatherType::VoidParticles => {
            paint_void_particles(buffer, millis, config.weather_intensity);
        }
        WeatherType::WindStreaks => paint_wind_streaks(buffer, millis, config.weather_intensity),
        WeatherType::Sparkles => paint_sparkles(buffer, millis, config.weather_intensity),
        WeatherType::AshRain => paint_ash_rain(buffer, millis, config.weather_intensity),
        WeatherType::GlassShards => paint_glass_shards(buffer, millis, config.weather_intensity),
        WeatherType::DriftingAsh => paint_drifting_ash(buffer, millis, config.weather_intensity),
        WeatherType::DustMotes => paint_dust_motes(buffer, millis, config.weather_intensity),
        WeatherType::StaticNoise => paint_static_noise(buffer, millis, config.weather_intensity),
        WeatherType::FractureMotes => {
            paint_fracture_motes(buffer, millis, config.weather_intensity);
        }
    }
}

fn paint_snow(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (120.0 / intensity).max(20.0) as u32;
    let fall_phase = (millis / 180.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(
                row.wrapping_add(fall_phase),
                col.wrapping_add(fall_phase / 3),
            );
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let drift = (col as f64 * 0.15 + millis * 0.001).sin();
            let ch = if drift > 0.3 { '*' } else { '\u{00b7}' };
            let brightness = 180 + (seed % 60) as u8;
            put_cell(
                buffer,
                row as i32,
                col as i32,
                ch,
                Color::Rgb(brightness, brightness, brightness.saturating_add(10)),
            );
        }
    }
}

fn paint_embers(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (100.0 / intensity).max(15.0) as u32;
    let rise_phase = (millis / 150.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            // Embers rise upward, so offset row index by negative phase
            let seed = hash2d(
                row.wrapping_add(height.wrapping_sub(rise_phase % height)),
                col.wrapping_add(rise_phase / 4),
            );
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let hot = seed.is_multiple_of(3);
            let (ch, fg) = if hot {
                ('\u{2022}', Color::Rgb(255, 160, 40)) // bullet, bright orange
            } else {
                ('\u{00b7}', Color::Rgb(200, 80, 20)) // middle-dot, dim red
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_sparks(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (90.0 / intensity).max(12.0) as u32;
    let phase = (millis / 100.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row.wrapping_add(phase), col.wrapping_add(phase * 2));
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let bright = seed.is_multiple_of(4);
            let (ch, fg) = if bright {
                ('*', Color::Rgb(255, 255, 100))
            } else {
                ('\u{00b7}', Color::Rgb(255, 200, 60))
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_bubbles(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (110.0 / intensity).max(18.0) as u32;
    let rise_phase = (millis / 220.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(
                row.wrapping_add(height.wrapping_sub(rise_phase % height)),
                col.wrapping_add(rise_phase / 5),
            );
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let large = seed.is_multiple_of(5);
            let (ch, fg) = if large {
                ('\u{25cb}', Color::Rgb(140, 180, 220)) // ○
            } else {
                ('\u{00b0}', Color::Rgb(100, 150, 200)) // degree sign (small bubble)
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_void_particles(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (80.0 / intensity).max(10.0) as u32;
    let phase = (millis / 130.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row.wrapping_add(phase), col.wrapping_add(phase));
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let pulse = (millis * 0.003 + row as f64 * col as f64 * 0.01).sin();
            let brightness = (80.0 + pulse * 60.0) as u8;
            let (ch, fg) = if seed.is_multiple_of(3) {
                (
                    '*',
                    Color::Rgb(brightness.saturating_add(40), brightness / 3, brightness),
                )
            } else {
                (
                    '\u{00b7}',
                    Color::Rgb(brightness, brightness / 4, brightness / 2),
                )
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_wind_streaks(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (130.0 / intensity).max(20.0) as u32;
    let phase = (millis / 80.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row, col.wrapping_add(phase));
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let ch = if seed.is_multiple_of(3) { '-' } else { '~' };
            put_cell(
                buffer,
                row as i32,
                col as i32,
                ch,
                Color::Rgb(200, 210, 230),
            );
        }
    }
}

fn paint_sparkles(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (100.0 / intensity).max(14.0) as u32;
    let twinkle = (millis / 200.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row + 11, col + 23);
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let visible = hash2d(row + twinkle, col + twinkle).is_multiple_of(2);
            if !visible {
                continue;
            }
            let bright = seed.is_multiple_of(4);
            let (ch, fg) = if bright {
                ('\u{2726}', Color::Rgb(200, 180, 255)) // ✦
            } else {
                ('\u{00b7}', Color::Rgb(160, 140, 220))
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_ash_rain(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (90.0 / intensity).max(12.0) as u32;
    let fall_phase = (millis / 120.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(
                row.wrapping_add(fall_phase),
                col.wrapping_add(fall_phase / 5),
            );
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let hot = seed.is_multiple_of(4);
            let (ch, fg) = if hot {
                ('\u{00b7}', Color::Rgb(200, 140, 80)) // middle-dot, warm orange
            } else {
                ('.', Color::Rgb(140, 130, 120)) // dim grey ash
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_glass_shards(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (100.0 / intensity).max(16.0) as u32;
    let fall_phase = (millis / 160.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(
                row.wrapping_add(fall_phase),
                col.wrapping_add(fall_phase / 4),
            );
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            // Alternate between bright glint and dim shard
            let glint = (millis * 0.005 + row as f64 * 0.3 + col as f64 * 0.2).sin() > 0.2;
            let (ch, fg) = if glint {
                ('\u{2726}', Color::Rgb(240, 245, 255)) // ✦ bright glint
            } else {
                ('/', Color::Rgb(140, 160, 180)) // dim falling shard
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_drifting_ash(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (140.0 / intensity).max(22.0) as u32;
    let drift_phase = (millis / 250.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            // Horizontal drift: offset col by phase
            let seed = hash2d(row + 5, col.wrapping_add(drift_phase));
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let dark = seed.is_multiple_of(3);
            let (ch, fg) = if dark {
                ('\u{00b7}', Color::Rgb(60, 55, 50)) // dark ash
            } else {
                ('~', Color::Rgb(90, 85, 80)) // slightly lighter ash
            };
            put_cell(buffer, row as i32, col as i32, ch, fg);
        }
    }
}

fn paint_dust_motes(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (130.0 / intensity).max(20.0) as u32;
    let slow_phase = (millis / 800.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row + 3, col + slow_phase / 10);
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            // Gentle pulse
            let pulse = (millis * 0.002 + row as f64 * 0.4 + col as f64 * 0.3).sin();
            if pulse < 0.1 {
                continue;
            }
            let brightness = (60.0 + pulse * 50.0) as u8;
            let fg = Color::Rgb(
                brightness.saturating_add(30),
                brightness.saturating_add(10),
                brightness / 2,
            );
            put_cell(buffer, row as i32, col as i32, '\u{00b7}', fg);
        }
    }
}

fn paint_static_noise(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (70.0 / intensity).max(10.0) as u32;
    let phase = (millis / 60.0) as usize;

    let noise_chars = ['/', '\\', '|', '-', '.', '*', ':', ';'];

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row.wrapping_add(phase), col.wrapping_add(phase * 7));
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            let ch_idx = seed as usize % noise_chars.len();
            let brightness = 40 + (seed % 80) as u8;
            let fg = Color::Rgb(brightness, brightness.saturating_sub(10), brightness.saturating_add(20));
            put_cell(buffer, row as i32, col as i32, noise_chars[ch_idx], fg);
        }
    }
}

fn paint_fracture_motes(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let threshold = (160.0 / intensity).max(25.0) as u32;
    let phase = (millis / 300.0) as usize;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            let seed = hash2d(row + phase, col + 11);
            if !seed.is_multiple_of(threshold) {
                continue;
            }
            if buffer[row][col].ch != ' ' {
                continue;
            }
            // Only visible during part of pulse cycle
            let pulse = (millis * 0.003 + seed as f64 * 0.01).sin();
            if pulse < 0.4 {
                continue;
            }
            let brightness = (30.0 + pulse * 40.0) as u8;
            let fg = Color::Rgb(brightness.saturating_add(20), brightness / 3, brightness);
            put_cell(buffer, row as i32, col as i32, '\u{00b7}', fg);
        }
    }
}

// ---------------------------------------------------------------------------
// Overlay Functions
// ---------------------------------------------------------------------------

/// Flickering ward glyphs that fade in and out on a time/position hash.
fn overlay_ancient_wards(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let t = millis / 400.0;

    for row in 0..height {
        for col in 0..width {
            if col >= buffer[row].len() {
                continue;
            }
            if !hash2d(row + 31, col + 47).is_multiple_of(211) {
                continue;
            }
            let phase = (t * 0.18 + row as f64 * 0.4 + col as f64 * 0.3).sin();
            if phase < 0.5 {
                continue;
            }
            let brightness = (80.0 + phase * 140.0) as u8;
            put_cell(
                buffer,
                row as i32,
                col as i32,
                '\u{2726}', // ✦
                Color::Rgb(brightness, brightness / 2, brightness.saturating_add(20)),
            );
        }
    }
}

/// Periodic brightness surge on bottom third (warm lava glow).
fn overlay_lava_glow(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let glow_start = (height as f64 * 0.67) as usize;
    let pulse = ((millis * 0.002).sin() * 0.5 + 0.5) * 25.0;

    for (row_idx, row_cells) in buffer.iter_mut().enumerate().skip(glow_start) {
        let row_t = (row_idx - glow_start) as f64 / (height - glow_start).max(1) as f64;
        let amount = (pulse * row_t) as i16;

        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + amount),
                    clamp_u8(g as i16 + amount / 4),
                    b,
                );
            }
        }
    }
}

/// Slow hue-shifting across entire scene (purple/blue crystal tones).
fn overlay_crystal_shimmer(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let phase = (millis * 0.0008).sin();
    let shift_r = (phase * 8.0) as i16;
    let shift_b = (-phase * 6.0) as i16;

    for row_cells in buffer.iter_mut() {
        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + shift_r),
                    g,
                    clamp_u8(b as i16 + shift_b),
                );
            }
        }
    }
}

/// Brief full-screen brightness spike every ~4 seconds lasting ~80ms.
fn overlay_lightning_flash(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let cycle = 4000.0;
    let flash_duration = 80.0;
    let phase = millis % cycle;

    if phase > flash_duration {
        return;
    }

    let intensity = (1.0 - phase / flash_duration) * 60.0;
    let amount = intensity as i16;

    for row in buffer.iter_mut() {
        for cell in row.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + amount),
                    clamp_u8(g as i16 + amount),
                    clamp_u8(b as i16 + amount),
                );
            }
        }
    }
}

/// Pulsing colour shifts between purple and red on background.
fn overlay_void_pulse(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let pulse = (millis * 0.0015).sin() * 0.5 + 0.5; // 0..1
    let shift_r = (pulse * 12.0) as i16;
    let shift_b = ((1.0 - pulse) * 10.0) as i16;

    for row_cells in buffer.iter_mut() {
        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + shift_r),
                    clamp_u8(g as i16 - 2),
                    clamp_u8(b as i16 + shift_b),
                );
            }
        }
    }
}

fn overlay_heat_distortion(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let distort_start = (height as f64 * 0.5) as usize;

    for (row_idx, row_cells) in buffer.iter_mut().enumerate().skip(distort_start) {
        let row_t = (row_idx - distort_start) as f64 / (height - distort_start).max(1) as f64;
        let wave = (millis * 0.003 + row_idx as f64 * 0.4).sin();
        let shift = (wave * 8.0 * row_t) as i16;

        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + shift),
                    clamp_u8(g as i16 + shift / 3),
                    clamp_u8(b as i16 - shift / 2),
                );
            }
        }
    }
}

fn overlay_mirror_flash(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let width = if height > 0 { buffer[0].len() } else { return };

    // 3 flash points that cycle
    for i in 0..3u32 {
        let cycle = 3000.0 + i as f64 * 1100.0;
        let phase = millis % cycle;
        let flash_duration = 120.0;

        if phase > flash_duration {
            continue;
        }

        let intensity = (1.0 - phase / flash_duration) * 40.0;
        let amount = intensity as i16;

        let flash_row = (hash2d(i as usize + 7, (millis / cycle) as usize) % height as u32) as usize;
        let flash_col = (hash2d((millis / cycle) as usize, i as usize + 3) % width as u32) as usize;

        // Horizontal streak
        let streak_len = 4 + (i as usize % 3);
        for dc in 0..streak_len {
            let col = flash_col + dc;
            if flash_row < height && col < width {
                if let Color::Rgb(r, g, b) = buffer[flash_row][col].bg {
                    buffer[flash_row][col].bg = Color::Rgb(
                        clamp_u8(r as i16 + amount),
                        clamp_u8(g as i16 + amount),
                        clamp_u8(b as i16 + amount),
                    );
                }
            }
        }
    }
}

fn overlay_consuming_dark(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let pulse = (millis * 0.001).sin() * 0.15 + 0.7; // 0.55 to 0.85
    let dark_start = (height as f64 * pulse) as usize;

    for (row_idx, row_cells) in buffer.iter_mut().enumerate().skip(dark_start) {
        let row_t = (row_idx - dark_start) as f64 / (height - dark_start).max(1) as f64;
        let darken = (row_t * 30.0) as i16;

        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 - darken),
                    clamp_u8(g as i16 - darken),
                    clamp_u8(b as i16 - darken),
                );
            }
        }
    }
}

fn overlay_hollow_echo(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let width = if !buffer.is_empty() { buffer[0].len() } else { return };
    let wave_pos = ((millis * 0.0004).sin() * 0.5 + 0.5) * width as f64;

    for row_cells in buffer.iter_mut() {
        for (col, cell) in row_cells.iter_mut().enumerate() {
            let dist = ((col as f64 - wave_pos).abs() / width as f64).min(1.0);
            if dist > 0.15 {
                continue;
            }
            // Desaturate: pull all channels toward their average
            if let Color::Rgb(r, g, b) = cell.bg {
                let avg = ((r as u16 + g as u16 + b as u16) / 3) as i16;
                let blend = (1.0 - dist / 0.15) * 0.4; // max 40% desaturation
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + ((avg - r as i16) as f64 * blend) as i16),
                    clamp_u8(g as i16 + ((avg - g as i16) as f64 * blend) as i16),
                    clamp_u8(b as i16 + ((avg - b as i16) as f64 * blend) as i16),
                );
            }
        }
    }
}

fn overlay_reality_tear(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let phase = (millis / 200.0) as usize;

    for (row_idx, row_cells) in buffer.iter_mut().enumerate() {
        // Only affect certain rows, cycling rapidly
        let affected = hash2d(row_idx + phase, phase + 17).is_multiple_of(7);
        if !affected {
            continue;
        }

        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                // Partial inversion: shift toward complement
                cell.bg = Color::Rgb(
                    clamp_u8(255i16 - r as i16 * 2 / 3 - 85),
                    clamp_u8(g as i16), // keep green stable for less nausea
                    clamp_u8(255i16 - b as i16 * 2 / 3 - 85),
                );
            }
        }
    }
}

fn overlay_wound_pulse(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let pulse = (millis * 0.0006).sin() * 0.5 + 0.5; // 0..1, very slow
    let darken = ((1.0 - pulse) * 20.0) as i16;

    for row_cells in buffer.iter_mut() {
        for cell in row_cells.iter_mut() {
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 - darken),
                    clamp_u8(g as i16 - darken),
                    clamp_u8(b as i16 - darken),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Zone Configs
// ---------------------------------------------------------------------------

fn zone_scene_config(zone_id: u32) -> ZoneSceneConfig {
    match zone_id {
        1 => config_meadow(),
        2 => config_dark_forest(),
        3 => config_mountain_pass(),
        4 => config_ancient_ruins(),
        5 => config_volcanic_wastes(),
        6 => config_frozen_tundra(),
        7 => config_crystal_caverns(),
        8 => config_sunken_kingdom(),
        9 => config_floating_isles(),
        10 => config_storm_citadel(),
        11 => config_the_expanse(),
        // Chapter 1: The Red Fault
        12 => config_splintered_rim(),
        13 => config_ember_ravine(),
        14 => config_heart_of_the_fault(),
        // Chapter 2: The Mirror Scar
        15 => config_shard_fields(),
        16 => config_refraction_steps(),
        17 => config_hall_of_second_suns(),
        // Chapter 3: The Black Mouth
        18 => config_ashen_verge(),
        19 => config_throat_of_the_world(),
        20 => config_the_black_mouth(),
        // Chapter 4: The Hollow Throne
        21 => config_sunken_processional(),
        22 => config_the_pale_archive(),
        23 => config_the_hollow_throne(),
        // Chapter 5: The Wailing Reach
        24 => config_the_stillborn_sea(),
        25 => config_resonance_fault(),
        26 => config_the_wailing_reach(),
        // Chapter 6: The Origin Wound
        27 => config_the_scar_root(),
        28 => config_echoing_abyss(),
        29 => config_threshold_of_silence(),
        30 => config_the_origin_wound(),
        _ => config_fallback(),
    }
}

/// Zone 1: Meadow -- bright, pastoral scene.
fn config_meadow() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (100, 160, 230),
        sky_bottom: (60, 140, 80),
        celestial: CelestialType::Sun,
        far_terrain: TerrainProfile {
            glyph: '~',
            color: (50, 120, 50),
            base_height: 0.60,
            amplitude: 0.06,
            frequency: 0.12,
            speed: 0.00015,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (40, 100, 40),
            base_height: 0.72,
            amplitude: 0.08,
            frequency: 0.09,
            speed: 0.00010,
            fill: true,
        },
        ground_glyphs: &['.', '\'', '"'],
        ground_color: (70, 130, 60),
        weather: WeatherType::None,
        weather_intensity: 1.0,
        overlay: None,
    }
}

/// Zone 2: Dark Forest -- dusk-purple tones, jagged treelines.
fn config_dark_forest() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (50, 30, 70),
        sky_bottom: (15, 10, 25),
        celestial: CelestialType::Moon,
        far_terrain: TerrainProfile {
            glyph: '\u{25b2}', // ▲
            color: (30, 50, 30),
            base_height: 0.50,
            amplitude: 0.10,
            frequency: 0.18,
            speed: 0.00008,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (20, 35, 20),
            base_height: 0.65,
            amplitude: 0.12,
            frequency: 0.14,
            speed: 0.00006,
            fill: true,
        },
        ground_glyphs: &['~', ';', '.'],
        ground_color: (40, 60, 35),
        weather: WeatherType::None,
        weather_intensity: 1.0,
        overlay: None,
    }
}

/// Zone 3: Mountain Pass -- grey-blue heights, snow.
fn config_mountain_pass() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (80, 90, 120),
        sky_bottom: (50, 55, 70),
        celestial: CelestialType::Stars,
        far_terrain: TerrainProfile {
            glyph: '\u{25b3}', // △
            color: (100, 110, 130),
            base_height: 0.42,
            amplitude: 0.12,
            frequency: 0.10,
            speed: 0.00005,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (70, 75, 90),
            base_height: 0.62,
            amplitude: 0.10,
            frequency: 0.08,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &['.', ':', ','],
        ground_color: (90, 95, 110),
        weather: WeatherType::Snow,
        weather_intensity: 1.0,
        overlay: None,
    }
}

/// Zone 4: Ancient Ruins -- purple mystical with flickering wards.
fn config_ancient_ruins() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (60, 30, 80),
        sky_bottom: (25, 12, 40),
        celestial: CelestialType::Wisps,
        far_terrain: TerrainProfile {
            glyph: '\u{2551}', // ║
            color: (80, 60, 100),
            base_height: 0.48,
            amplitude: 0.08,
            frequency: 0.22,
            speed: 0.00003,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (60, 45, 75),
            base_height: 0.65,
            amplitude: 0.06,
            frequency: 0.16,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &[':', '.', ';'],
        ground_color: (70, 50, 90),
        weather: WeatherType::None,
        weather_intensity: 1.0,
        overlay: Some(overlay_ancient_wards),
    }
}

/// Zone 5: Volcanic Wastes -- deep reds, lava, embers.
fn config_volcanic_wastes() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (100, 30, 15),
        sky_bottom: (20, 5, 5),
        celestial: CelestialType::EmberGlow,
        far_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (140, 50, 20),
            base_height: 0.50,
            amplitude: 0.08,
            frequency: 0.14,
            speed: 0.00010,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2588}', // █
            color: (40, 10, 5),
            base_height: 0.68,
            amplitude: 0.10,
            frequency: 0.11,
            speed: 0.00007,
            fill: true,
        },
        ground_glyphs: &['=', ':'],
        ground_color: (160, 60, 20),
        weather: WeatherType::Embers,
        weather_intensity: 1.0,
        overlay: Some(overlay_lava_glow),
    }
}

/// Zone 6: Frozen Tundra -- white-grey to pale blue, heavy snow.
fn config_frozen_tundra() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (160, 165, 175),
        sky_bottom: (120, 140, 170),
        celestial: CelestialType::Overcast,
        far_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (180, 190, 210),
            base_height: 0.52,
            amplitude: 0.07,
            frequency: 0.10,
            speed: 0.00004,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (140, 155, 180),
            base_height: 0.68,
            amplitude: 0.08,
            frequency: 0.08,
            speed: 0.00003,
            fill: true,
        },
        ground_glyphs: &['~', '.'],
        ground_color: (170, 180, 200),
        weather: WeatherType::Snow,
        weather_intensity: 2.0,
        overlay: None,
    }
}

/// Zone 7: Crystal Caverns -- deep indigo, bioluminescence, sparkles.
fn config_crystal_caverns() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (20, 15, 60),
        sky_bottom: (5, 5, 20),
        celestial: CelestialType::BioLuminescent,
        far_terrain: TerrainProfile {
            glyph: '\u{25c6}', // ◆
            color: (80, 60, 160),
            base_height: 0.45,
            amplitude: 0.10,
            frequency: 0.20,
            speed: 0.00005,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2502}', // │
            color: (50, 40, 120),
            base_height: 0.60,
            amplitude: 0.08,
            frequency: 0.25,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &['*', ':'],
        ground_color: (100, 80, 180),
        weather: WeatherType::Sparkles,
        weather_intensity: 1.0,
        overlay: Some(overlay_crystal_shimmer),
    }
}

/// Zone 8: Sunken Kingdom -- deep blue, coral, bubbles.
fn config_sunken_kingdom() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (10, 30, 80),
        sky_bottom: (5, 10, 35),
        celestial: CelestialType::BioLuminescent,
        far_terrain: TerrainProfile {
            glyph: '~',
            color: (40, 80, 120),
            base_height: 0.52,
            amplitude: 0.07,
            frequency: 0.09,
            speed: 0.00008,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2551}', // ║
            color: (30, 55, 90),
            base_height: 0.65,
            amplitude: 0.09,
            frequency: 0.15,
            speed: 0.00006,
            fill: true,
        },
        ground_glyphs: &['~', '.'],
        ground_color: (50, 90, 130),
        weather: WeatherType::Bubbles,
        weather_intensity: 1.0,
        overlay: None,
    }
}

/// Zone 9: Floating Isles -- bright sky, wind.
fn config_floating_isles() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (120, 170, 240),
        sky_bottom: (180, 200, 230),
        celestial: CelestialType::Sun,
        far_terrain: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (100, 120, 140),
            base_height: 0.50,
            amplitude: 0.10,
            frequency: 0.13,
            speed: 0.00012,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2550}', // ═
            color: (80, 90, 110),
            base_height: 0.65,
            amplitude: 0.07,
            frequency: 0.10,
            speed: 0.00008,
            fill: false,
        },
        ground_glyphs: &['~'],
        ground_color: (130, 150, 180),
        weather: WeatherType::WindStreaks,
        weather_intensity: 1.0,
        overlay: None,
    }
}

/// Zone 10: Storm Citadel -- dark amber-black, lightning.
fn config_storm_citadel() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (60, 45, 15),
        sky_bottom: (10, 8, 5),
        celestial: CelestialType::None,
        far_terrain: TerrainProfile {
            glyph: '\u{256b}', // ╫
            color: (100, 80, 30),
            base_height: 0.45,
            amplitude: 0.09,
            frequency: 0.16,
            speed: 0.00006,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (60, 50, 20),
            base_height: 0.62,
            amplitude: 0.08,
            frequency: 0.12,
            speed: 0.00005,
            fill: true,
        },
        ground_glyphs: &[':'],
        ground_color: (90, 70, 25),
        weather: WeatherType::Sparks,
        weather_intensity: 1.0,
        overlay: Some(overlay_lightning_flash),
    }
}

/// Zone 11: The Expanse -- void purple, reality tears.
fn config_the_expanse() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (40, 10, 50),
        sky_bottom: (8, 2, 12),
        celestial: CelestialType::Stars,
        far_terrain: TerrainProfile {
            glyph: '\u{2571}', // ╱
            color: (90, 30, 70),
            base_height: 0.48,
            amplitude: 0.10,
            frequency: 0.18,
            speed: 0.00010,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (60, 15, 45),
            base_height: 0.62,
            amplitude: 0.08,
            frequency: 0.22,
            speed: 0.00008,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::VoidParticles,
        weather_intensity: 1.0,
        overlay: Some(overlay_void_pulse),
    }
}

// ---------------------------------------------------------------------------
// Chapter 1: The Red Fault (Zones 12-14)
// ---------------------------------------------------------------------------

/// Zone 12: Splintered Rim -- cracked volcanic ridge, smoky, ash falling.
fn config_splintered_rim() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (120, 70, 35),
        sky_bottom: (40, 20, 10),
        celestial: CelestialType::CrackedSky,
        far_terrain: TerrainProfile {
            glyph: '\u{25b2}', // ▲
            color: (140, 60, 30),
            base_height: 0.48,
            amplitude: 0.10,
            frequency: 0.16,
            speed: 0.00008,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (60, 25, 15),
            base_height: 0.65,
            amplitude: 0.08,
            frequency: 0.12,
            speed: 0.00006,
            fill: true,
        },
        ground_glyphs: &['=', ':'],
        ground_color: (130, 70, 35),
        weather: WeatherType::AshRain,
        weather_intensity: 0.8,
        overlay: Some(overlay_heat_distortion),
    }
}

/// Zone 13: Ember Ravine -- deep molten ravine, heavy embers, lava glow.
fn config_ember_ravine() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (150, 40, 10),
        sky_bottom: (25, 5, 5),
        celestial: CelestialType::EmberGlow,
        far_terrain: TerrainProfile {
            glyph: '~',
            color: (180, 80, 20),
            base_height: 0.52,
            amplitude: 0.06,
            frequency: 0.10,
            speed: 0.00010,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2588}', // █
            color: (35, 10, 5),
            base_height: 0.68,
            amplitude: 0.10,
            frequency: 0.11,
            speed: 0.00007,
            fill: true,
        },
        ground_glyphs: &['=', '~'],
        ground_color: (170, 70, 25),
        weather: WeatherType::Embers,
        weather_intensity: 2.0,
        overlay: Some(overlay_lava_glow),
    }
}

/// Zone 14: Heart of the Fault -- the wound's burning core, crimson sky, heavy ash.
fn config_heart_of_the_fault() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (180, 30, 20),
        sky_bottom: (60, 8, 8),
        celestial: CelestialType::EmberGlow,
        far_terrain: TerrainProfile {
            glyph: '\u{2571}', // ╱
            color: (200, 60, 30),
            base_height: 0.46,
            amplitude: 0.09,
            frequency: 0.18,
            speed: 0.00010,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (50, 15, 10),
            base_height: 0.64,
            amplitude: 0.08,
            frequency: 0.14,
            speed: 0.00006,
            fill: true,
        },
        ground_glyphs: &[':', '.'],
        ground_color: (160, 50, 20),
        weather: WeatherType::AshRain,
        weather_intensity: 1.8,
        overlay: Some(overlay_heat_distortion),
    }
}

// ---------------------------------------------------------------------------
// Chapter 2: The Mirror Scar (Zones 15-17)
// ---------------------------------------------------------------------------

/// Zone 15: Shard Fields -- scattered mirror crystals, cold blue, glass shards falling.
fn config_shard_fields() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (140, 160, 190),
        sky_bottom: (60, 70, 90),
        celestial: CelestialType::CrackedSky,
        far_terrain: TerrainProfile {
            glyph: '\u{25c6}', // ◆
            color: (120, 150, 200),
            base_height: 0.46,
            amplitude: 0.09,
            frequency: 0.20,
            speed: 0.00005,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2502}', // │
            color: (80, 100, 150),
            base_height: 0.62,
            amplitude: 0.07,
            frequency: 0.25,
            speed: 0.00004,
            fill: false,
        },
        ground_glyphs: &['*', '.'],
        ground_color: (140, 160, 200),
        weather: WeatherType::GlassShards,
        weather_intensity: 0.8,
        overlay: Some(overlay_mirror_flash),
    }
}

/// Zone 16: Refraction Steps -- bending light, prismatic, bioluminescent.
fn config_refraction_steps() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (180, 200, 240),
        sky_bottom: (30, 20, 80),
        celestial: CelestialType::BioLuminescent,
        far_terrain: TerrainProfile {
            glyph: '\u{25bd}', // ▽
            color: (100, 140, 200),
            base_height: 0.50,
            amplitude: 0.08,
            frequency: 0.16,
            speed: 0.00006,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (70, 100, 170),
            base_height: 0.64,
            amplitude: 0.06,
            frequency: 0.20,
            speed: 0.00005,
            fill: true,
        },
        ground_glyphs: &[':', '*'],
        ground_color: (110, 140, 210),
        weather: WeatherType::GlassShards,
        weather_intensity: 1.2,
        overlay: Some(overlay_crystal_shimmer),
    }
}

/// Zone 17: Hall of Second Suns -- blinding prismatic light, heavy sparkles.
fn config_hall_of_second_suns() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (220, 230, 250),
        sky_bottom: (80, 160, 180),
        celestial: CelestialType::CrackedSky,
        far_terrain: TerrainProfile {
            glyph: '\u{2550}', // ═
            color: (200, 210, 240),
            base_height: 0.48,
            amplitude: 0.05,
            frequency: 0.12,
            speed: 0.00008,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (120, 160, 200),
            base_height: 0.66,
            amplitude: 0.06,
            frequency: 0.15,
            speed: 0.00005,
            fill: true,
        },
        ground_glyphs: &['*', '\u{00b7}'],
        ground_color: (180, 200, 240),
        weather: WeatherType::Sparkles,
        weather_intensity: 2.0,
        overlay: Some(overlay_mirror_flash),
    }
}

// ---------------------------------------------------------------------------
// Chapter 3: The Black Mouth (Zones 18-20)
// ---------------------------------------------------------------------------

/// Zone 18: Ashen Verge -- dark grey, rolling ash dunes, consuming darkness.
fn config_ashen_verge() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (70, 65, 60),
        sky_bottom: (25, 22, 20),
        celestial: CelestialType::Overcast,
        far_terrain: TerrainProfile {
            glyph: '~',
            color: (80, 75, 70),
            base_height: 0.52,
            amplitude: 0.07,
            frequency: 0.10,
            speed: 0.00006,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (40, 38, 35),
            base_height: 0.68,
            amplitude: 0.08,
            frequency: 0.09,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &['.', ';'],
        ground_color: (65, 60, 55),
        weather: WeatherType::DriftingAsh,
        weather_intensity: 0.8,
        overlay: Some(overlay_consuming_dark),
    }
}

/// Zone 19: Throat of the World -- near-black, downward stalactites, heavy drifting ash.
fn config_throat_of_the_world() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (20, 18, 15),
        sky_bottom: (5, 4, 3),
        celestial: CelestialType::None,
        far_terrain: TerrainProfile {
            glyph: '\u{25bc}', // ▼
            color: (40, 35, 30),
            base_height: 0.40,
            amplitude: 0.10,
            frequency: 0.14,
            speed: 0.00005,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2588}', // █
            color: (15, 12, 10),
            base_height: 0.65,
            amplitude: 0.08,
            frequency: 0.10,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::DriftingAsh,
        weather_intensity: 1.8,
        overlay: Some(overlay_consuming_dark),
    }
}

/// Zone 20: The Black Mouth -- deep purple-black void, consuming everything.
fn config_the_black_mouth() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (30, 10, 35),
        sky_bottom: (5, 2, 8),
        celestial: CelestialType::VoidRift,
        far_terrain: TerrainProfile {
            glyph: '\u{2572}', // ╲
            color: (50, 20, 40),
            base_height: 0.48,
            amplitude: 0.08,
            frequency: 0.18,
            speed: 0.00008,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (20, 8, 18),
            base_height: 0.64,
            amplitude: 0.06,
            frequency: 0.15,
            speed: 0.00006,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::VoidParticles,
        weather_intensity: 1.2,
        overlay: Some(overlay_consuming_dark),
    }
}

// ---------------------------------------------------------------------------
// Chapter 4: The Hollow Throne (Zones 21-23)
// ---------------------------------------------------------------------------

/// Zone 21: Sunken Processional -- amber-grey pillared halls, dust motes, hollow echoes.
fn config_sunken_processional() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (100, 85, 60),
        sky_bottom: (35, 30, 22),
        celestial: CelestialType::Wisps,
        far_terrain: TerrainProfile {
            glyph: '\u{2551}', // ║
            color: (120, 100, 60),
            base_height: 0.44,
            amplitude: 0.06,
            frequency: 0.22,
            speed: 0.00003,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (70, 60, 40),
            base_height: 0.66,
            amplitude: 0.05,
            frequency: 0.16,
            speed: 0.00003,
            fill: true,
        },
        ground_glyphs: &[':', '.'],
        ground_color: (90, 75, 50),
        weather: WeatherType::DustMotes,
        weather_intensity: 0.8,
        overlay: Some(overlay_hollow_echo),
    }
}

/// Zone 22: The Pale Archive -- bone-white library, faded, dusty silence.
fn config_the_pale_archive() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (160, 155, 145),
        sky_bottom: (80, 75, 65),
        celestial: CelestialType::None,
        far_terrain: TerrainProfile {
            glyph: '\u{2502}', // │
            color: (130, 125, 115),
            base_height: 0.46,
            amplitude: 0.05,
            frequency: 0.24,
            speed: 0.00002,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (110, 105, 95),
            base_height: 0.65,
            amplitude: 0.04,
            frequency: 0.18,
            speed: 0.00003,
            fill: true,
        },
        ground_glyphs: &['\u{00b7}', ';'],
        ground_color: (120, 115, 105),
        weather: WeatherType::DustMotes,
        weather_intensity: 1.2,
        overlay: Some(overlay_hollow_echo),
    }
}

/// Zone 23: The Hollow Throne -- cold grey to void-black, obsidian palace, void rift above.
fn config_the_hollow_throne() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (60, 55, 65),
        sky_bottom: (8, 5, 12),
        celestial: CelestialType::VoidRift,
        far_terrain: TerrainProfile {
            glyph: '\u{256b}', // ╫
            color: (80, 70, 90),
            base_height: 0.44,
            amplitude: 0.07,
            frequency: 0.18,
            speed: 0.00004,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (30, 25, 35),
            base_height: 0.64,
            amplitude: 0.06,
            frequency: 0.14,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::DustMotes,
        weather_intensity: 0.5,
        overlay: Some(overlay_hollow_echo),
    }
}

// ---------------------------------------------------------------------------
// Chapter 5: The Wailing Reach (Zones 24-26)
// ---------------------------------------------------------------------------

/// Zone 24: The Stillborn Sea -- flat grey lifeless sea, absolute stillness, no weather.
fn config_the_stillborn_sea() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (85, 85, 88),
        sky_bottom: (70, 70, 73),
        celestial: CelestialType::None,
        far_terrain: TerrainProfile {
            glyph: '~',
            color: (80, 80, 85),
            base_height: 0.55,
            amplitude: 0.02, // nearly flat -- still water
            frequency: 0.08,
            speed: 0.00001, // barely moving
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '▒', // ▒
            color: (65, 65, 70),
            base_height: 0.70,
            amplitude: 0.03,
            frequency: 0.06,
            speed: 0.00001,
            fill: true,
        },
        ground_glyphs: &['~', '.'],
        ground_color: (75, 75, 80),
        weather: WeatherType::None, // absolute stillness
        weather_intensity: 1.0,
        overlay: Some(overlay_hollow_echo),
    }
}

/// Zone 25: Resonance Fault -- vibrating teal-purple, crystallized sound, light static.
fn config_resonance_fault() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (30, 80, 100),
        sky_bottom: (50, 20, 70),
        celestial: CelestialType::Flicker,
        far_terrain: TerrainProfile {
            glyph: '│', // │
            color: (60, 120, 140),
            base_height: 0.46,
            amplitude: 0.08,
            frequency: 0.22,
            speed: 0.00010, // vibrating
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '░', // ░
            color: (40, 80, 100),
            base_height: 0.63,
            amplitude: 0.06,
            frequency: 0.18,
            speed: 0.00008,
            fill: true,
        },
        ground_glyphs: &['*', ':'],
        ground_color: (50, 100, 120),
        weather: WeatherType::StaticNoise,
        weather_intensity: 0.6,
        overlay: Some(overlay_reality_tear),
    }
}

/// Zone 26: The Wailing Reach -- flickering unstable reality, heavy static, reality tears.
fn config_the_wailing_reach() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (50, 45, 60),
        sky_bottom: (20, 15, 30),
        celestial: CelestialType::Flicker,
        far_terrain: TerrainProfile {
            glyph: '╱', // ╱
            color: (70, 50, 80),
            base_height: 0.48,
            amplitude: 0.10,
            frequency: 0.20,
            speed: 0.00012,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '░', // ░
            color: (40, 30, 50),
            base_height: 0.64,
            amplitude: 0.08,
            frequency: 0.16,
            speed: 0.00010,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::StaticNoise,
        weather_intensity: 1.6,
        overlay: Some(overlay_reality_tear),
    }
}

// ---------------------------------------------------------------------------
// Chapter 6: The Origin Wound (Zones 27-30)
// ---------------------------------------------------------------------------

fn config_the_scar_root() -> ZoneSceneConfig {
    config_fallback()
}

fn config_echoing_abyss() -> ZoneSceneConfig {
    config_fallback()
}

fn config_threshold_of_silence() -> ZoneSceneConfig {
    config_fallback()
}

fn config_the_origin_wound() -> ZoneSceneConfig {
    config_fallback()
}

/// Fallback for unknown zone ids.
fn config_fallback() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (60, 60, 70),
        sky_bottom: (20, 20, 25),
        celestial: CelestialType::Stars,
        far_terrain: TerrainProfile {
            glyph: '~',
            color: (70, 70, 80),
            base_height: 0.55,
            amplitude: 0.07,
            frequency: 0.12,
            speed: 0.00006,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (50, 50, 60),
            base_height: 0.68,
            amplitude: 0.08,
            frequency: 0.10,
            speed: 0.00005,
            fill: true,
        },
        ground_glyphs: &['.', ':'],
        ground_color: (60, 60, 70),
        weather: WeatherType::None,
        weather_intensity: 1.0,
        overlay: None,
    }
}
