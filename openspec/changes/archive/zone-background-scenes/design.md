> Backported design record. Sources: docs/plans/2026-02-18-zone-background-scenes-design.md, docs/plans/2026-02-18-zone-background-scenes.md.

## 2026-02-18-zone-background-scenes-design.md

# Zone Background Scenes Design

**Date:** 2026-02-18
**Status:** Approved

## Summary

Upgrade the combat_3d enemy sprite backdrop from simple gradient+noise to rich, fishing-scene-quality layered zone scenery. Each of the 11 zones gets a unique atmospheric background with terrain silhouettes, sky treatment, weather effects, and optional per-zone overlays.

## Goals

- Each zone should feel visually distinct and thematically immersive
- Match the visual quality and layering depth of the fishing scene
- Render behind the enemy sprite in the existing combat_3d area
- Scale gracefully across all terminal sizes (S through XL)

## Architecture

### Approach: Data-driven core + per-zone overlays

A generic rendering engine processes a fixed set of layer types. Each zone provides configuration data for those layers. Zones that need unique effects (e.g. volcanic lava pulses, lightning flashes) provide an optional overlay function.

### Layer Pipeline

Six layers rendered in order into a shared `SceneCell` buffer:

```
Layer 1: Sky gradient       — Background colors only, fills entire buffer
Layer 2: Celestial/atmo     — Sparse fg chars (sun, moon, stars, auroras)
Layer 3: Far terrain         — Distant silhouette (hills, peaks, structures)
Layer 4: Near terrain        — Closer, taller, darker terrain
Layer 5: Ground detail       — Surface texture at bottom rows
Layer 6: Weather overlay     — Falling/rising particles (snow, embers, sparks)
```

Later layers overwrite earlier foreground chars while preserving background colors (via `put_cell()`).

### Data Model

```rust
struct ZoneSceneConfig {
    // Layer 1: Sky gradient
    sky_top: (u8, u8, u8),
    sky_bottom: (u8, u8, u8),

    // Layer 2: Celestial element
    celestial: CelestialType,  // Sun, Moon, Stars, Aurora, Overcast, None

    // Layer 3-4: Terrain silhouettes
    terrain_far: TerrainProfile,
    terrain_near: TerrainProfile,

    // Layer 5: Ground detail
    ground_glyphs: &'static [char],
    ground_color: (u8, u8, u8),
    ground_density: u32,

    // Layer 6: Weather particles
    weather: WeatherType,  // None, Snow, Rain, Ash, Embers, Sparks, Bubbles, Void

    // Optional per-zone overlay
    overlay: Option<fn(&mut [Vec<SceneCell>], f64)>,
}

struct TerrainProfile {
    glyph: char,
    color: (u8, u8, u8),
    base_height: f64,    // fraction of buffer height (0.0-1.0)
    amplitude: f64,      // wave amplitude
    frequency: f64,      // wave frequency
    speed: f64,          // horizontal drift speed
    fill: bool,          // fill below silhouette line
}

enum CelestialType {
    Sun { col_ratio: f64 },
    Moon { col_ratio: f64 },
    Stars { density: u32 },
    Aurora,
    BioLuminescent,
    Overcast,
    EmberGlow,
    None,
}

enum WeatherType {
    None,
    Snow { density: u32, speed: f64 },
    Rain { density: u32, speed: f64 },
    Ash { density: u32, speed: f64 },
    Embers { density: u32, speed: f64 },
    Sparks { density: u32, speed: f64 },
    Bubbles { density: u32, speed: f64 },
    VoidParticles { density: u32, speed: f64 },
    WindStreaks { density: u32, speed: f64 },
    Sparkles { density: u32, speed: f64 },
}
```

## Per-Zone Visual Design

### Zone 1: Meadow
- **Sky:** Bright blue gradient (top light blue → bottom green-tinted)
- **Celestial:** Sun with radiating dots, drifting clouds
- **Far terrain:** Gentle rolling hills, `~` glyph, muted green
- **Near terrain:** Taller grassy hills, `▒` glyph, darker green
- **Ground:** Grass and wildflower chars `.'"`, bright green
- **Weather:** None
- **Overlay:** None

### Zone 2: Dark Forest
- **Sky:** Dusk purple → dark (perpetual twilight)
- **Celestial:** Dim moon, sparse twinkling stars
- **Far terrain:** Jagged treeline, `▲^` glyphs, dark green
- **Near terrain:** Dense canopy silhouette, `█▓` glyphs, near-black green
- **Ground:** Roots and undergrowth `~;`, dim olive
- **Weather:** None
- **Overlay:** None

### Zone 3: Mountain Pass
- **Sky:** Grey-blue → slate (cold, overcast)
- **Celestial:** Stars visible through cloud gaps, snow clouds
- **Far terrain:** Distant peaked mountains, `╱╲△` glyphs, blue-grey
- **Near terrain:** Rocky crags and boulders, `▓▒` glyphs, dark slate
- **Ground:** Loose scree and gravel `.:,`, grey
- **Weather:** Falling snow `·*`, gentle drift
- **Overlay:** None

### Zone 4: Ancient Ruins
- **Sky:** Purple → dark violet (eerie dusk)
- **Celestial:** Eerie green floating wisps
- **Far terrain:** Broken pillars and arches, `║╖` glyphs, dusty purple
- **Near terrain:** Crumbling walls, `▒░` glyphs, dark stone
- **Ground:** Rubble and debris `:.;`, muted grey-brown
- **Weather:** None
- **Overlay:** Flickering ward glyphs that appear and fade

### Zone 5: Volcanic Wastes
- **Sky:** Deep red → black (perpetual fiery night)
- **Celestial:** Ember glow (no sun/moon), orange haze
- **Far terrain:** Jagged lava flows and ridges, `▓` glyph, dark red
- **Near terrain:** Obsidian spires, `█▓` glyphs, near-black red
- **Ground:** Cracked earth `=:`, dark orange
- **Weather:** Rising embers `·*`, upward drift
- **Overlay:** Lava glow pulses (periodic brightness surges)

### Zone 6: Frozen Tundra
- **Sky:** White-grey → pale blue (overcast blizzard)
- **Celestial:** Overcast (no celestial body visible)
- **Far terrain:** Ice ridges, `▒░` glyphs, pale blue-white
- **Near terrain:** Glacier walls, `█▓` glyphs, blue-grey
- **Ground:** Snowdrifts `~.`, white
- **Weather:** Falling snow `·*`, heavier than Mountain Pass
- **Overlay:** None

### Zone 7: Crystal Caverns
- **Sky:** Deep indigo → black (underground cavern)
- **Celestial:** Bioluminescent spots scattered on ceiling
- **Far terrain:** Crystal formations, `◆▸` glyphs, blue-purple
- **Near terrain:** Stalactites from above, `╲│╱` glyphs, dark crystal blue
- **Ground:** Crystal fragments and reflections `*:`, dim sparkle
- **Weather:** Floating sparkles that drift slowly
- **Overlay:** Color-shifting shimmer (hue rotation over time)

### Zone 8: Sunken Kingdom
- **Sky:** Deep blue → abyss black (underwater gradient)
- **Celestial:** Bioluminescent jellyfish-like floating lights
- **Far terrain:** Coral structures, `~≈` glyphs, muted teal
- **Near terrain:** Drowned spires and columns, `║▒` glyphs, dark blue-green
- **Ground:** Seafloor sand and shells `~.`, dark blue
- **Weather:** Rising bubbles `°o`, gentle upward drift
- **Overlay:** None

### Zone 9: Floating Isles
- **Sky:** Bright sky → cloud white (open sky, airy)
- **Celestial:** Sun with drifting clouds at multiple speeds
- **Far terrain:** Distant floating rock fragments, muted grey-blue
- **Near terrain:** Chain bridges and rock edges, `═─` glyphs, stone grey
- **Ground:** Cloud wisps `~`, white-grey
- **Weather:** Wind streaks `─`, horizontal fast drift
- **Overlay:** None

### Zone 10: Storm Citadel
- **Sky:** Dark amber → black (storm-charged atmosphere)
- **Celestial:** None (sky too turbulent)
- **Far terrain:** Lightning towers, `╫║` glyphs, amber-gold
- **Near terrain:** Crackling fortress walls, `▓▒` glyphs, dark amber
- **Ground:** Electrified floor `:`, dim yellow
- **Weather:** Electric sparks `*·`, erratic movement
- **Overlay:** Lightning flashes (brief full-screen brightness spikes)

### Zone 11: The Expanse
- **Sky:** Void purple → deep black (cosmic void)
- **Celestial:** Distant dying stars, very sparse
- **Far terrain:** Rift edges, `╱╲` glyphs, dim crimson-purple
- **Near terrain:** Reality tears, `░▒` glyphs, flickering
- **Ground:** Empty void / nothing
- **Weather:** Void particles `*:·`, drifting in all directions
- **Overlay:** Color-pulsing void (shifting between purple, red, and black)

## File Structure

```
src/ui/
├── zone_bg.rs           # NEW: ZoneSceneConfig, paint_zone_scene(), layer renderers
├── zone_bg_overlays.rs  # NEW: Per-zone overlay functions
├── combat_3d.rs         # MODIFIED: Remove ZoneBackdropTheme, call zone_bg::paint_zone_scene()
├── scene_fx.rs          # MODIFIED: Add draw_terrain_silhouette(), draw_falling_particles()
```

### Integration Changes

In `combat_3d.rs`:
- Remove `ZoneBackdropTheme`, `zone_backdrop_theme()`, `paint_zone_background()`, `lift_rgb()`, `clamp_u8()`
- Replace `paint_zone_background(&mut buffer, zone_id)` with `zone_bg::paint_zone_scene(&mut buffer, zone_id)`

In `scene_fx.rs`:
- Add `draw_terrain_silhouette(buffer, profile, millis)` — generic sine-wave terrain renderer
- Add `draw_weather_particles(buffer, weather_type, millis)` — generic particle system

In `ui/mod.rs`:
- Add `mod zone_bg;` and `mod zone_bg_overlays;`

## Performance

The fishing scene runs at 10 FPS (100ms tick) with no issues despite 6+ rendering layers. Zone backgrounds use the same buffer size and similar operations:
- Per-cell iteration for sky gradient: O(width * height)
- Per-column iteration for terrain silhouettes: O(width * 2)
- Sparse particle placement: O(width * height) with early skip via modulo
- Overlay effects: zone-specific, all lightweight

All well within the 100ms tick budget.

## Testing

- Edge case handling: `paint_zone_scene()` returns safely on empty or tiny buffers
- Existing behavior-lock tests are unaffected (they test game logic, not rendering)
- Manual visual verification using debug menu to jump between zones
- Regression: ensure enemy sprites still render correctly over new backgrounds

## Incremental Delivery

1. Core infrastructure (ZoneSceneConfig, layer renderers, scene_fx helpers)
2. Data-driven base for all 11 zones (layers 1-6, no overlays)
3. Per-zone overlays for standout zones (Volcanic, Crystal, Storm, Expanse)
4. Polish and tuning (color adjustments, animation speeds, particle density)

## 2026-02-18-zone-background-scenes.md

# Zone Background Scenes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the combat_3d zone backdrops from simple gradient+noise to rich, fishing-scene-quality layered scenery with terrain silhouettes, celestial elements, and weather effects for all 11 zones.

**Architecture:** Data-driven 6-layer rendering engine in a new `zone_bg.rs` module. Each zone provides a `ZoneSceneConfig` struct defining sky gradient, celestial type, two terrain silhouette profiles, ground detail, and weather particles. Optional per-zone overlay functions in `zone_bg_overlays.rs` handle unique effects (volcanic lava pulses, lightning flashes, etc.). The existing `paint_zone_background()` in `combat_3d.rs` is replaced with a single call to `zone_bg::paint_zone_scene()`.

**Tech Stack:** Rust, Ratatui (SceneCell buffer rendering via `scene_fx.rs` primitives)

**Design Doc:** `docs/plans/2026-02-18-zone-background-scenes-design.md`

---

### Task 1: Add scene_fx utility helpers

**Files:**
- Modify: `src/ui/scene_fx.rs` (add functions after line 150)

**Step 1: Add `clamp_u8` and `lift_rgb` to scene_fx**

These are currently private in `combat_3d.rs` (lines 500-510). Move them to `scene_fx.rs` as `pub` so both `zone_bg.rs` and `combat_3d.rs` can use them.

Add to the end of `src/ui/scene_fx.rs`:

```rust
/// Clamps an i16 to the u8 range [0, 255].
pub fn clamp_u8(value: i16) -> u8 {
    value.clamp(0, 255) as u8
}

/// Uniformly lifts an RGB tuple by `amount`, clamping to [0, 255].
pub fn lift_rgb(rgb: (u8, u8, u8), amount: i16) -> (u8, u8, u8) {
    (
        clamp_u8(rgb.0 as i16 + amount),
        clamp_u8(rgb.1 as i16 + amount),
        clamp_u8(rgb.2 as i16 + amount),
    )
}
```

**Step 2: Verify build compiles**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles with no errors (new functions are unused for now, but `pub` so no dead-code warning from outside)

**Step 3: Commit**

```bash
git add src/ui/scene_fx.rs
git commit -m "refactor: move clamp_u8 and lift_rgb to scene_fx"
```

---

### Task 2: Create zone_bg.rs with data model and sky gradient layer

**Files:**
- Create: `src/ui/zone_bg.rs`
- Modify: `src/ui/mod.rs` (add `mod zone_bg;` after line 28)

**Step 1: Create zone_bg.rs with types and sky gradient renderer**

Create `src/ui/zone_bg.rs` with the data model structs and the sky gradient layer. Start with just the `ZoneSceneConfig`, `TerrainProfile`, `CelestialType`, `WeatherType` types plus the sky gradient renderer and a stub `paint_zone_scene()` that only paints the sky.

```rust
//! Zone-themed background scenes for combat_3d.
//!
//! Each zone gets a unique multi-layered ASCII background rendered into a
//! SceneCell buffer. Layers are painted in order: sky gradient, celestial,
//! far terrain, near terrain, ground detail, weather particles, overlay.

use super::scene_fx::{clamp_u8, current_millis, hash2d, lift_rgb, lerp_rgb, put_cell, SceneCell};
use ratatui::style::Color;

// ── Types ───────────────────────────────────────────────────────────

/// Configuration for a zone's terrain silhouette layer.
#[derive(Clone, Copy)]
pub struct TerrainProfile {
    pub glyph: char,
    pub color: (u8, u8, u8),
    /// Fraction of buffer height where the terrain baseline sits (0.0 = top, 1.0 = bottom).
    pub base_height: f64,
    /// Wave amplitude as fraction of buffer height.
    pub amplitude: f64,
    /// Horizontal wave frequency.
    pub frequency: f64,
    /// Horizontal drift speed (millis multiplier).
    pub speed: f64,
    /// If true, fill all rows below the silhouette line with the glyph.
    pub fill: bool,
}

/// Type of celestial/atmospheric element in the sky.
#[derive(Clone, Copy)]
pub enum CelestialType {
    /// Sun at a given horizontal position ratio (0.0-1.0).
    Sun { col_ratio: f64 },
    /// Moon at a given horizontal position ratio.
    Moon { col_ratio: f64 },
    /// Sparse twinkling stars with given density (higher = fewer).
    Stars { density: u32 },
    /// Eerie floating wisps.
    Wisps { density: u32 },
    /// Bioluminescent ceiling spots (underground).
    BioLuminescent { density: u32 },
    /// Overcast sky (no visible celestial body).
    Overcast,
    /// Ember glow haze (volcanic).
    EmberGlow,
    /// No celestial element.
    None,
}

/// Type of weather particle effect.
#[derive(Clone, Copy)]
pub enum WeatherType {
    None,
    /// Falling snow particles.
    Snow { density: u32, speed: f64 },
    /// Rising ember particles.
    Embers { density: u32, speed: f64 },
    /// Electric sparks (erratic movement).
    Sparks { density: u32, speed: f64 },
    /// Rising bubbles (underwater).
    Bubbles { density: u32, speed: f64 },
    /// Drifting void particles (all directions).
    VoidParticles { density: u32, speed: f64 },
    /// Horizontal wind streaks.
    WindStreaks { density: u32, speed: f64 },
    /// Floating sparkles (crystal caverns).
    Sparkles { density: u32, speed: f64 },
}

/// Full scene configuration for a zone's background.
pub struct ZoneSceneConfig {
    // Layer 1: Sky gradient
    pub sky_top: (u8, u8, u8),
    pub sky_bottom: (u8, u8, u8),
    /// Per-cell noise applied to sky gradient.
    pub sky_noise: f64,

    // Layer 2: Celestial
    pub celestial: CelestialType,

    // Layers 3-4: Terrain silhouettes
    pub terrain_far: TerrainProfile,
    pub terrain_near: TerrainProfile,

    // Layer 5: Ground detail
    pub ground_glyphs: &'static [char],
    pub ground_color: (u8, u8, u8),
    /// Ground glyph density (higher = fewer glyphs). 0 disables.
    pub ground_density: u32,
    /// Fraction of buffer height where ground starts (0.0 = top, 1.0 = bottom).
    pub ground_start: f64,

    // Layer 6: Weather
    pub weather: WeatherType,

    // Optional overlay
    pub overlay: Option<fn(&mut [Vec<SceneCell>], f64)>,
}

// ── Public API ──────────────────────────────────────────────────────

/// Paints the zone background into a SceneCell buffer.
/// This replaces `paint_zone_background()` from combat_3d.rs.
pub fn paint_zone_scene(buffer: &mut [Vec<SceneCell>], zone_id: u32) {
    if buffer.is_empty() || buffer[0].is_empty() {
        return;
    }

    let config = zone_scene_config(zone_id);
    let millis = current_millis() as f64;

    paint_sky_gradient(buffer, &config, millis, zone_id);
    paint_celestial(buffer, &config, millis, zone_id);
    paint_terrain(buffer, &config.terrain_far, millis, zone_id, 0);
    paint_terrain(buffer, &config.terrain_near, millis, zone_id, 1);
    paint_ground_detail(buffer, &config, millis, zone_id);
    paint_weather(buffer, &config, millis, zone_id);

    if let Some(overlay_fn) = config.overlay {
        overlay_fn(buffer, millis);
    }
}

// ── Layer Renderers ─────────────────────────────────────────────────

/// Layer 1: Sky gradient with per-cell noise.
fn paint_sky_gradient(
    buffer: &mut [Vec<SceneCell>],
    config: &ZoneSceneConfig,
    millis: f64,
    zone_id: u32,
) {
    let height = buffer.len();
    let top = lift_rgb(config.sky_top, 30);
    let bottom = lift_rgb(config.sky_bottom, 24);

    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let row_t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let base = lerp_rgb(top, bottom, row_t.powf(0.86));

        for (col, cell) in row_cells.iter_mut().enumerate() {
            let drift = (col as f64 * 0.13 + millis * 0.00042 + zone_id as f64 * 0.63).sin()
                + (row as f64 * 0.17 - millis * 0.00031 + zone_id as f64 * 0.29).cos();
            let jitter = ((drift * config.sky_noise).round() as i16).clamp(-6, 14);
            let r = clamp_u8(base.0 as i16 + jitter);
            let g = clamp_u8(base.1 as i16 + jitter);
            let b = clamp_u8(base.2 as i16 + jitter);
            cell.bg = Color::Rgb(r, g, b);
        }
    }
}

/// Layer 2: Celestial / atmospheric elements.
fn paint_celestial(
    buffer: &mut [Vec<SceneCell>],
    config: &ZoneSceneConfig,
    millis: f64,
    zone_id: u32,
) {
    let height = buffer.len();
    let width = buffer[0].len();
    let phase = (millis / 180.0) as usize;

    match config.celestial {
        CelestialType::Sun { col_ratio } => {
            let col = ((width as f64 * col_ratio) + (millis * 0.00008).sin() * 3.0).round() as i32;
            let row = ((height as f64 * 0.18) + (millis * 0.00004).sin()).round() as i32;
            // Sun body and glow
            for &(dx, dy, ch, r, g, b) in &[
                (0, 0, '\u{25CF}', 255u8, 228, 152),  // ● sun body
                (-1, 0, '\u{00B7}', 240, 226, 176),    // · glow
                (1, 0, '\u{00B7}', 240, 226, 176),
                (0, -1, '\u{00B7}', 236, 220, 170),
                (0, 1, '\u{00B7}', 236, 220, 170),
            ] {
                put_cell(buffer, row + dy, col + dx, ch, Color::Rgb(r, g, b));
            }
        }
        CelestialType::Moon { col_ratio } => {
            let col = ((width as f64 * col_ratio) + (millis * 0.00006).sin() * 2.0).round() as i32;
            let row = ((height as f64 * 0.15) + (millis * 0.00003).sin()).round() as i32;
            put_cell(buffer, row, col, '\u{25D1}', Color::Rgb(232, 238, 255)); // ◑
            put_cell(buffer, row, col - 1, '\u{00B7}', Color::Rgb(200, 210, 230));
            put_cell(buffer, row, col + 1, '\u{00B7}', Color::Rgb(200, 210, 230));
        }
        CelestialType::Stars { density } => {
            for row in 0..(height / 2) {
                for col in 0..width {
                    if hash2d(row + zone_id as usize * 7, col + zone_id as usize * 13)
                        .is_multiple_of(density)
                    {
                        let bright =
                            hash2d(row + phase, col + phase).is_multiple_of(3);
                        let ch = if bright { '*' } else { '.' };
                        let fg = if bright {
                            Color::Rgb(244, 246, 255)
                        } else {
                            Color::Rgb(184, 190, 232)
                        };
                        if buffer[row][col].ch == ' ' {
                            put_cell(buffer, row as i32, col as i32, ch, fg);
                        }
                    }
                }
            }
        }
        CelestialType::Wisps { density } => {
            for row in 0..height {
                for col in 0..width {
                    if hash2d(row + zone_id as usize * 11, col + zone_id as usize * 23)
                        .is_multiple_of(density)
                    {
                        let drift = (col as f64 * 0.08 + millis * 0.0003 + row as f64 * 0.5).sin();
                        if drift > 0.6 && buffer[row][col].ch == ' ' {
                            let bright = hash2d(row + phase, col + phase).is_multiple_of(2);
                            let fg = if bright {
                                Color::Rgb(120, 255, 160)
                            } else {
                                Color::Rgb(60, 180, 90)
                            };
                            put_cell(buffer, row as i32, col as i32, '\u{00B7}', fg);
                        }
                    }
                }
            }
        }
        CelestialType::BioLuminescent { density } => {
            // Glowing spots on ceiling (top third of buffer)
            let ceiling = height / 3;
            for row in 0..ceiling {
                for col in 0..width {
                    if hash2d(row + zone_id as usize * 19, col + zone_id as usize * 37)
                        .is_multiple_of(density)
                    {
                        let pulse = ((millis * 0.001 + col as f64 * 0.3).sin() * 0.5 + 0.5) as u8;
                        let base_bright = 120 + pulse * 80 / 255;
                        let fg = Color::Rgb(base_bright, base_bright + 40, base_bright + 80);
                        if buffer[row][col].ch == ' ' {
                            put_cell(buffer, row as i32, col as i32, '\u{2022}', fg); // •
                        }
                    }
                }
            }
        }
        CelestialType::Overcast => {
            // Drifting cloud bands in upper portion
            let cloud_zone = height / 3;
            for &(base_x, y_ratio, speed, pattern) in &[
                (6.0f64, 0.15, 0.035, "~~"),
                (20.0, 0.25, 0.028, "~~~"),
                (38.0, 0.10, 0.042, "~~"),
            ] {
                let drift = (millis * speed / 110.0) % width as f64;
                let cx = ((base_x - drift).rem_euclid(width as f64)) as usize;
                let row = ((cloud_zone as f64 * y_ratio).round() as usize).min(cloud_zone.saturating_sub(1));
                for (i, ch) in pattern.chars().enumerate() {
                    let col = (cx + i) % width;
                    if buffer[row][col].ch == ' ' {
                        put_cell(
                            buffer,
                            row as i32,
                            col as i32,
                            ch,
                            Color::Rgb(160, 168, 178),
                        );
                    }
                }
            }
        }
        CelestialType::EmberGlow => {
            // Warm haze bands in upper portion
            let haze_zone = height / 3;
            for row in 0..haze_zone {
                for col in 0..width {
                    let wave = (col as f64 * 0.06 + millis * 0.0002 + row as f64 * 0.3).sin();
                    if wave > 0.7
                        && hash2d(row + zone_id as usize * 5, col).is_multiple_of(8)
                        && buffer[row][col].ch == ' '
                    {
                        let intensity = ((wave - 0.7) * 200.0).min(80.0) as u8;
                        put_cell(
                            buffer,
                            row as i32,
                            col as i32,
                            '\u{2591}', // ░
                            Color::Rgb(120 + intensity, 60 + intensity / 2, 20),
                        );
                    }
                }
            }
        }
        CelestialType::None => {}
    }
}

/// Layers 3-4: Terrain silhouette.
fn paint_terrain(
    buffer: &mut [Vec<SceneCell>],
    profile: &TerrainProfile,
    millis: f64,
    zone_id: u32,
    layer_index: u32,
) {
    let height = buffer.len();
    let width = buffer[0].len();
    let color = lift_rgb(profile.color, if layer_index == 0 { 20 } else { 10 });
    let fg = Color::Rgb(color.0, color.1, color.2);

    for col in 0..width {
        let wave = (col as f64 * profile.frequency
            + millis * profile.speed
            + zone_id as f64 * 0.47
            + layer_index as f64 * 1.3)
            .sin();
        let secondary = (col as f64 * profile.frequency * 2.3
            + millis * profile.speed * 0.7
            + layer_index as f64 * 2.1)
            .sin();
        let combined = wave * 0.7 + secondary * 0.3;

        let base_row =
            (height as f64 * profile.base_height + combined * height as f64 * profile.amplitude)
                .round() as i32;

        if profile.fill {
            // Fill from silhouette line to bottom
            for row in base_row.max(0)..height as i32 {
                if (row as usize) < height
                    && (col as usize) < width
                    && buffer[row as usize][col].ch == ' '
                {
                    put_cell(buffer, row, col as i32, profile.glyph, fg);
                }
            }
        } else {
            // Just the silhouette line
            if base_row >= 0 && (base_row as usize) < height {
                put_cell(buffer, base_row, col as i32, profile.glyph, fg);
            }
        }
    }
}

/// Layer 5: Ground-level detail.
fn paint_ground_detail(
    buffer: &mut [Vec<SceneCell>],
    config: &ZoneSceneConfig,
    _millis: f64,
    zone_id: u32,
) {
    if config.ground_density == 0 || config.ground_glyphs.is_empty() {
        return;
    }

    let height = buffer.len();
    let width = buffer[0].len();
    let start_row = (height as f64 * config.ground_start).round() as usize;
    let color = lift_rgb(config.ground_color, 16);
    let fg = Color::Rgb(color.0, color.1, color.2);

    for row in start_row..height {
        for col in 0..width {
            let seed = hash2d(row + zone_id as usize * 23, col + zone_id as usize * 41);
            if seed.is_multiple_of(config.ground_density) && buffer[row][col].ch == ' ' {
                let glyph_idx = (seed as usize / config.ground_density as usize)
                    % config.ground_glyphs.len();
                put_cell(buffer, row as i32, col as i32, config.ground_glyphs[glyph_idx], fg);
            }
        }
    }
}

/// Layer 6: Weather particle effects.
fn paint_weather(
    buffer: &mut [Vec<SceneCell>],
    config: &ZoneSceneConfig,
    millis: f64,
    zone_id: u32,
) {
    let height = buffer.len();
    let width = buffer[0].len();

    match config.weather {
        WeatherType::None => {}
        WeatherType::Snow { density, speed } => {
            // Falling snow drifts downward
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 29, col + zone_id as usize * 43);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let fall = (row + tick + seed as usize * 3) % height;
                    let drift_x =
                        ((col as f64 + (millis * 0.0003).sin() * 2.0).round() as usize) % width;
                    if buffer[fall][drift_x].ch == ' ' {
                        let bright = seed.is_multiple_of(3);
                        let ch = if bright { '*' } else { '\u{00B7}' };
                        let fg = if bright {
                            Color::Rgb(220, 228, 240)
                        } else {
                            Color::Rgb(180, 190, 210)
                        };
                        put_cell(buffer, fall as i32, drift_x as i32, ch, fg);
                    }
                }
            }
        }
        WeatherType::Embers { density, speed } => {
            // Rising embers drift upward
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 31, col + zone_id as usize * 47);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let rise = (height - 1) - ((height - 1 - row + tick + seed as usize * 2) % height);
                    let drift_x = ((col as f64 + (millis * 0.0005 + row as f64 * 0.2).sin() * 1.5)
                        .round() as usize)
                        % width;
                    if buffer[rise][drift_x].ch == ' ' {
                        let bright = seed.is_multiple_of(3);
                        let ch = if bright { '*' } else { '\u{00B7}' };
                        let fg = if bright {
                            Color::Rgb(255, 160, 60)
                        } else {
                            Color::Rgb(200, 100, 40)
                        };
                        put_cell(buffer, rise as i32, drift_x as i32, ch, fg);
                    }
                }
            }
        }
        WeatherType::Sparks { density, speed } => {
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 37, col + zone_id as usize * 53);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let jitter_row = (row + tick + seed as usize) % height;
                    let jitter_col = (col + (seed as usize / 7) + tick / 3) % width;
                    if buffer[jitter_row][jitter_col].ch == ' ' {
                        let fg = if seed.is_multiple_of(2) {
                            Color::Rgb(255, 240, 120)
                        } else {
                            Color::Rgb(200, 180, 80)
                        };
                        put_cell(buffer, jitter_row as i32, jitter_col as i32, '*', fg);
                    }
                }
            }
        }
        WeatherType::Bubbles { density, speed } => {
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 41, col + zone_id as usize * 59);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let rise = (height - 1) - ((height - 1 - row + tick + seed as usize) % height);
                    let drift_x = ((col as f64 + (millis * 0.0002 + row as f64 * 0.4).sin() * 1.0)
                        .round() as usize)
                        % width;
                    if buffer[rise][drift_x].ch == ' ' {
                        let ch = if seed.is_multiple_of(4) { 'o' } else { '\u{00B0}' }; // ° or o
                        let fg = Color::Rgb(140, 200, 240);
                        put_cell(buffer, rise as i32, drift_x as i32, ch, fg);
                    }
                }
            }
        }
        WeatherType::VoidParticles { density, speed } => {
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 43, col + zone_id as usize * 61);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let drift_row = (row + tick / 2 + seed as usize) % height;
                    let drift_col = (col + tick / 3 + seed as usize / 5) % width;
                    if buffer[drift_row][drift_col].ch == ' ' {
                        let ch = if seed.is_multiple_of(3) { '*' } else { ':' };
                        let pulse = ((millis * 0.002 + seed as f64 * 0.1).sin() * 0.5 + 0.5) as u8;
                        let fg = Color::Rgb(160 + pulse * 40 / 255, 40 + pulse * 20 / 255, 100 + pulse * 60 / 255);
                        put_cell(buffer, drift_row as i32, drift_col as i32, ch, fg);
                    }
                }
            }
        }
        WeatherType::WindStreaks { density, speed } => {
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 47, col + zone_id as usize * 67);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let drift_col = (col + tick + seed as usize * 2) % width;
                    if buffer[row][drift_col].ch == ' ' {
                        let fg = Color::Rgb(180, 190, 210);
                        put_cell(buffer, row as i32, drift_col as i32, '\u{2500}', fg); // ─
                    }
                }
            }
        }
        WeatherType::Sparkles { density, speed } => {
            let tick = (millis * speed) as usize;
            for row in 0..height {
                for col in 0..width {
                    let seed = hash2d(row + zone_id as usize * 51, col + zone_id as usize * 71);
                    if !seed.is_multiple_of(density) {
                        continue;
                    }
                    let bright = hash2d(row + tick, col + tick).is_multiple_of(3);
                    if !bright {
                        continue;
                    }
                    if buffer[row][col].ch == ' ' {
                        let hue_shift = ((millis * 0.001 + col as f64 * 0.2).sin() * 0.5 + 0.5);
                        let r = (140.0 + hue_shift * 80.0) as u8;
                        let g = (120.0 + (1.0 - hue_shift) * 60.0) as u8;
                        let b = (180.0 + hue_shift * 60.0) as u8;
                        put_cell(buffer, row as i32, col as i32, '\u{2726}', Color::Rgb(r, g, b)); // ✦
                    }
                }
            }
        }
    }
}

// ── Zone Configs ────────────────────────────────────────────────────

/// Returns the scene configuration for a given zone.
fn zone_scene_config(zone_id: u32) -> ZoneSceneConfig {
    match zone_id {
        1 => meadow_config(),
        2 => dark_forest_config(),
        3 => mountain_pass_config(),
        4 => ancient_ruins_config(),
        5 => volcanic_wastes_config(),
        6 => frozen_tundra_config(),
        7 => crystal_caverns_config(),
        8 => sunken_kingdom_config(),
        9 => floating_isles_config(),
        10 => storm_citadel_config(),
        11 => the_expanse_config(),
        _ => meadow_config(), // fallback
    }
}

fn meadow_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (118, 196, 248),
        sky_bottom: (60, 140, 80),
        sky_noise: 2.6,
        celestial: CelestialType::Sun { col_ratio: 0.78 },
        terrain_far: TerrainProfile {
            glyph: '~',
            color: (50, 100, 55),
            base_height: 0.65,
            amplitude: 0.06,
            frequency: 0.12,
            speed: 0.00015,
            fill: true,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (35, 75, 40),
            base_height: 0.78,
            amplitude: 0.08,
            frequency: 0.09,
            speed: 0.00022,
            fill: true,
        },
        ground_glyphs: &['.', '\'', '"'],
        ground_color: (62, 114, 76),
        ground_density: 8,
        ground_start: 0.85,
        weather: WeatherType::None,
        overlay: None,
    }
}

fn dark_forest_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (30, 18, 50),
        sky_bottom: (12, 10, 22),
        sky_noise: 2.0,
        celestial: CelestialType::Moon { col_ratio: 0.82 },
        terrain_far: TerrainProfile {
            glyph: '\u{25B2}', // ▲
            color: (25, 55, 30),
            base_height: 0.50,
            amplitude: 0.12,
            frequency: 0.18,
            speed: 0.00010,
            fill: true,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (15, 35, 18),
            base_height: 0.65,
            amplitude: 0.10,
            frequency: 0.14,
            speed: 0.00016,
            fill: true,
        },
        ground_glyphs: &['~', ';', '.'],
        ground_color: (40, 60, 35),
        ground_density: 6,
        ground_start: 0.80,
        weather: WeatherType::None,
        overlay: None,
    }
}

fn mountain_pass_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (60, 72, 100),
        sky_bottom: (30, 35, 50),
        sky_noise: 3.8,
        celestial: CelestialType::Stars { density: 90 },
        terrain_far: TerrainProfile {
            glyph: '\u{25B3}', // △
            color: (70, 80, 100),
            base_height: 0.40,
            amplitude: 0.15,
            frequency: 0.22,
            speed: 0.00008,
            fill: false,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (45, 50, 65),
            base_height: 0.60,
            amplitude: 0.12,
            frequency: 0.16,
            speed: 0.00014,
            fill: true,
        },
        ground_glyphs: &['.', ':', ','],
        ground_color: (80, 85, 95),
        ground_density: 7,
        ground_start: 0.80,
        weather: WeatherType::Snow { density: 180, speed: 0.008 },
        overlay: None,
    }
}

fn ancient_ruins_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (40, 20, 55),
        sky_bottom: (18, 10, 30),
        sky_noise: 3.1,
        celestial: CelestialType::Wisps { density: 200 },
        terrain_far: TerrainProfile {
            glyph: '\u{2551}', // ║
            color: (70, 50, 85),
            base_height: 0.45,
            amplitude: 0.10,
            frequency: 0.25,
            speed: 0.00006,
            fill: false,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (50, 35, 60),
            base_height: 0.65,
            amplitude: 0.08,
            frequency: 0.19,
            speed: 0.00012,
            fill: true,
        },
        ground_glyphs: &[':', '.', ';'],
        ground_color: (70, 55, 65),
        ground_density: 7,
        ground_start: 0.82,
        weather: WeatherType::None,
        overlay: Some(overlay_ancient_wards),
    }
}

fn volcanic_wastes_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (60, 15, 10),
        sky_bottom: (20, 5, 3),
        sky_noise: 4.4,
        celestial: CelestialType::EmberGlow,
        terrain_far: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (80, 25, 15),
            base_height: 0.55,
            amplitude: 0.10,
            frequency: 0.20,
            speed: 0.00012,
            fill: true,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2588}', // █
            color: (40, 12, 8),
            base_height: 0.72,
            amplitude: 0.08,
            frequency: 0.15,
            speed: 0.00018,
            fill: true,
        },
        ground_glyphs: &['=', ':'],
        ground_color: (120, 60, 30),
        ground_density: 6,
        ground_start: 0.85,
        weather: WeatherType::Embers { density: 120, speed: 0.012 },
        overlay: Some(overlay_lava_glow),
    }
}

fn frozen_tundra_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (140, 150, 165),
        sky_bottom: (70, 85, 110),
        sky_noise: 2.8,
        celestial: CelestialType::Overcast,
        terrain_far: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (130, 145, 170),
            base_height: 0.55,
            amplitude: 0.06,
            frequency: 0.10,
            speed: 0.00010,
            fill: true,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (80, 95, 120),
            base_height: 0.70,
            amplitude: 0.08,
            frequency: 0.13,
            speed: 0.00015,
            fill: true,
        },
        ground_glyphs: &['~', '.'],
        ground_color: (180, 190, 210),
        ground_density: 5,
        ground_start: 0.82,
        weather: WeatherType::Snow { density: 100, speed: 0.012 },
        overlay: None,
    }
}

fn crystal_caverns_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (18, 15, 50),
        sky_bottom: (8, 6, 25),
        sky_noise: 3.5,
        celestial: CelestialType::BioLuminescent { density: 150 },
        terrain_far: TerrainProfile {
            glyph: '\u{25C6}', // ◆
            color: (60, 60, 130),
            base_height: 0.50,
            amplitude: 0.12,
            frequency: 0.22,
            speed: 0.00010,
            fill: false,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2502}', // │
            color: (40, 40, 90),
            base_height: 0.35,
            amplitude: 0.08,
            frequency: 0.28,
            speed: 0.00014,
            fill: false,
        },
        ground_glyphs: &['*', ':'],
        ground_color: (80, 80, 160),
        ground_density: 8,
        ground_start: 0.80,
        weather: WeatherType::Sparkles { density: 200, speed: 0.005 },
        overlay: Some(overlay_crystal_shimmer),
    }
}

fn sunken_kingdom_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (10, 30, 70),
        sky_bottom: (4, 12, 35),
        sky_noise: 2.4,
        celestial: CelestialType::BioLuminescent { density: 180 },
        terrain_far: TerrainProfile {
            glyph: '~',
            color: (30, 80, 80),
            base_height: 0.55,
            amplitude: 0.08,
            frequency: 0.08,
            speed: 0.00008,
            fill: true,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2551}', // ║
            color: (20, 50, 60),
            base_height: 0.68,
            amplitude: 0.06,
            frequency: 0.20,
            speed: 0.00012,
            fill: false,
        },
        ground_glyphs: &['~', '.'],
        ground_color: (30, 60, 90),
        ground_density: 6,
        ground_start: 0.82,
        weather: WeatherType::Bubbles { density: 200, speed: 0.006 },
        overlay: None,
    }
}

fn floating_isles_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (130, 180, 240),
        sky_bottom: (200, 215, 235),
        sky_noise: 2.2,
        celestial: CelestialType::Sun { col_ratio: 0.75 },
        terrain_far: TerrainProfile {
            glyph: '\u{2592}', // ▒
            color: (100, 110, 130),
            base_height: 0.45,
            amplitude: 0.10,
            frequency: 0.30,
            speed: 0.00020,
            fill: false,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2550}', // ═
            color: (80, 85, 100),
            base_height: 0.65,
            amplitude: 0.05,
            frequency: 0.12,
            speed: 0.00008,
            fill: false,
        },
        ground_glyphs: &['~'],
        ground_color: (180, 195, 220),
        ground_density: 10,
        ground_start: 0.85,
        weather: WeatherType::WindStreaks { density: 250, speed: 0.025 },
        overlay: None,
    }
}

fn storm_citadel_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (50, 40, 15),
        sky_bottom: (18, 14, 5),
        sky_noise: 3.0,
        celestial: CelestialType::None,
        terrain_far: TerrainProfile {
            glyph: '\u{256B}', // ╫
            color: (110, 90, 35),
            base_height: 0.45,
            amplitude: 0.08,
            frequency: 0.24,
            speed: 0.00010,
            fill: false,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (70, 55, 20),
            base_height: 0.65,
            amplitude: 0.06,
            frequency: 0.18,
            speed: 0.00016,
            fill: true,
        },
        ground_glyphs: &[':'],
        ground_color: (140, 120, 50),
        ground_density: 8,
        ground_start: 0.85,
        weather: WeatherType::Sparks { density: 160, speed: 0.018 },
        overlay: Some(overlay_lightning_flash),
    }
}

fn the_expanse_config() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (35, 10, 40),
        sky_bottom: (10, 3, 15),
        sky_noise: 4.8,
        celestial: CelestialType::Stars { density: 200 },
        terrain_far: TerrainProfile {
            glyph: '\u{2571}', // ╱
            color: (80, 30, 55),
            base_height: 0.55,
            amplitude: 0.10,
            frequency: 0.20,
            speed: 0.00020,
            fill: false,
        },
        terrain_near: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (50, 15, 35),
            base_height: 0.70,
            amplitude: 0.08,
            frequency: 0.26,
            speed: 0.00030,
            fill: false,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        ground_density: 0,
        ground_start: 1.0,
        weather: WeatherType::VoidParticles { density: 130, speed: 0.015 },
        overlay: Some(overlay_void_pulse),
    }
}

// ── Overlay Functions ───────────────────────────────────────────────

/// Ancient Ruins: Flickering ward glyphs that appear and fade.
fn overlay_ancient_wards(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let phase = (millis / 400.0) as usize;

    for row in 0..height {
        for col in 0..width {
            let seed = hash2d(row + 3, col + 7);
            if !seed.is_multiple_of(300) {
                continue;
            }
            let visible = hash2d(phase + seed as usize, col + row).is_multiple_of(4);
            if visible && buffer[row][col].ch == ' ' {
                let fg = Color::Rgb(80, 200, 120);
                put_cell(buffer, row as i32, col as i32, '\u{2726}', fg); // ✦
            }
        }
    }
}

/// Volcanic Wastes: Periodic lava glow brightness surges.
fn overlay_lava_glow(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let glow = ((millis * 0.0008).sin() * 0.5 + 0.5).powf(2.0);
    let boost = (glow * 12.0).round() as i16;

    if boost < 3 {
        return; // Only apply when the glow is visible
    }

    // Apply warm tint to bottom third of screen (lava region)
    let start = (height as f64 * 0.7).round() as usize;
    for row in start..height {
        for col in 0..width {
            let cell = &mut buffer[row][col];
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + boost),
                    clamp_u8(g as i16 + boost / 3),
                    b,
                );
            }
        }
    }
}

/// Crystal Caverns: Slow hue-shifting shimmer across the scene.
fn overlay_crystal_shimmer(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let hue = (millis * 0.0004).sin() * 0.5 + 0.5; // 0.0 to 1.0

    for row in 0..height {
        for col in 0..width {
            let cell = &mut buffer[row][col];
            if let Color::Rgb(r, g, b) = cell.bg {
                // Subtle hue shift toward purple or blue based on time
                let shift = ((hue * 6.0) as i16).clamp(0, 6);
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + shift),
                    g,
                    clamp_u8(b as i16 + shift * 2),
                );
            }
        }
    }
}

/// Storm Citadel: Brief lightning flashes.
fn overlay_lightning_flash(buffer: &mut [Vec<SceneCell>], millis: f64) {
    // Lightning strikes every ~4 seconds, lasts ~80ms
    let cycle = (millis % 4000.0) as u32;
    if cycle > 80 {
        return;
    }

    let height = buffer.len();
    let width = buffer[0].len();
    let intensity = if cycle < 30 { 20i16 } else { 10 };

    for row in 0..height {
        for col in 0..width {
            let cell = &mut buffer[row][col];
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.bg = Color::Rgb(
                    clamp_u8(r as i16 + intensity),
                    clamp_u8(g as i16 + intensity),
                    clamp_u8(b as i16 + intensity / 2),
                );
            }
        }
    }
}

/// The Expanse: Pulsing void color shifts between purple, red, and black.
fn overlay_void_pulse(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
    let width = buffer[0].len();
    let pulse = ((millis * 0.0005).sin() * 0.5 + 0.5).powf(1.5);
    let shift_r = (pulse * 8.0).round() as i16;
    let shift_b = ((1.0 - pulse) * 6.0).round() as i16;

    for row in 0..height {
        for col in 0..width {
            let cell = &mut buffer[row][col];
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
```

**Step 2: Register module in mod.rs**

Add `mod zone_bg;` to `src/ui/mod.rs` after line 28 (after `mod scene_fx;`).

**Step 3: Verify build compiles**

Run: `cargo build 2>&1 | head -30`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/ui/zone_bg.rs src/ui/mod.rs
git commit -m "feat: add zone_bg module with 6-layer scene rendering for all 11 zones"
```

---

### Task 3: Integrate zone_bg into combat_3d

**Files:**
- Modify: `src/ui/combat_3d.rs`

**Step 1: Replace paint_zone_background call with zone_bg::paint_zone_scene**

In `src/ui/combat_3d.rs` line 66, replace:
```rust
paint_zone_background(&mut buffer, zone_id);
```
with:
```rust
super::zone_bg::paint_zone_scene(&mut buffer, zone_id);
```

**Step 2: Remove old backdrop code from combat_3d.rs**

Delete the following from `combat_3d.rs`:
- Lines 26-45: `ZoneBackdropTheme` struct
- Lines 139-225: `paint_zone_background()` function
- Lines 279-497: `zone_backdrop_theme()` function
- Lines 500-510: `clamp_u8()` and `lift_rgb()` functions (now in `scene_fx.rs`)

Also clean up the import on line 12. The following imports are no longer needed in combat_3d.rs after removing paint_zone_background:
- `current_millis` — no longer used directly
- `hash2d` — no longer used directly
- `lerp_rgb` — no longer used directly

The remaining import line should be:
```rust
use super::scene_fx::{put_cell, render_buffer, SceneCell};
```

**Step 3: Verify build compiles and passes all tests**

Run: `cargo build && cargo test 2>&1 | tail -20`
Expected: Build succeeds, all tests pass

**Step 4: Commit**

```bash
git add src/ui/combat_3d.rs
git commit -m "refactor: replace old zone backdrop with zone_bg scene rendering"
```

---

### Task 4: Run full CI checks

**Files:** None (verification only)

**Step 1: Run make check**

Run: `make check`
Expected: All checks pass (fmt, clippy, test, build, audit)

**Step 2: Fix any clippy warnings**

If clippy flags anything (unused imports, unnecessary clones, etc.), fix them.

**Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: resolve clippy warnings in zone_bg"
```

---

### Task 5: Visual verification and tuning

**Files:** Possibly modify `src/ui/zone_bg.rs` for color/parameter adjustments

**Step 1: Run the game and check each zone background**

Run: `cargo run -- --debug`
Use the debug menu (backtick key) to trigger zone changes and verify each zone's background looks correct:
- Zone 1 (Meadow): Blue sky, sun, green rolling hills, grass at bottom
- Zone 2 (Dark Forest): Dark purple sky, moon, jagged treeline
- Zone 3 (Mountain Pass): Grey sky, stars, mountain peaks, falling snow
- Zone 4 (Ancient Ruins): Purple sky, green wisps, pillars, flickering wards
- Zone 5 (Volcanic Wastes): Red/black sky, ember glow, lava terrain, rising embers, lava glow pulses
- Zone 6 (Frozen Tundra): Overcast grey, ice ridges, heavy snow
- Zone 7 (Crystal Caverns): Dark indigo, bioluminescent ceiling, crystal formations, sparkles, shimmer
- Zone 8 (Sunken Kingdom): Deep blue, bioluminescent lights, coral, rising bubbles
- Zone 9 (Floating Isles): Bright sky, sun, floating rocks, wind streaks
- Zone 10 (Storm Citadel): Dark amber, lightning towers, sparks, lightning flashes
- Zone 11 (The Expanse): Void purple/black, distant stars, void particles, color pulses

**Step 2: Tune parameters**

Adjust colors, speeds, densities, and amplitudes in the zone config functions as needed based on visual review. Common adjustments:
- Sky gradient colors too bright/dark
- Terrain silhouettes too high/low or too jagged/smooth
- Weather particle density too sparse/dense
- Overlay effects too subtle/aggressive

**Step 3: Commit any tuning changes**

```bash
git add src/ui/zone_bg.rs
git commit -m "polish: tune zone background parameters after visual review"
```

---

### Task 6: Final CI check and branch completion

**Step 1: Run make check one final time**

Run: `make check`
Expected: All checks pass

**Step 2: Branch is ready for PR or merge**

Use `superpowers:finishing-a-development-branch` to decide on merge/PR strategy.
