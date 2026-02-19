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
