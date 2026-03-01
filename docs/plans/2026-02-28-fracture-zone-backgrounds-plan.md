# Fracture Zone Backgrounds Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the generic fallback background for zones 12-30 with 19 unique themed backgrounds that follow a cosmic horror visual arc across six fracture chapters.

**Architecture:** All changes are in `src/ui/zone_bg.rs`. We add new enum variants to `WeatherType` and `CelestialType`, implement their paint functions, add 6 new overlay functions, add 19 new `config_*()` functions, and expand the `zone_scene_config()` match. No other files are modified.

**Tech Stack:** Rust, Ratatui (Color::Rgb), existing `scene_fx` utilities (`put_cell`, `hash2d`, `clamp_u8`, `lerp_rgb`, `current_millis`).

**Design doc:** `docs/plans/2026-02-28-fracture-zone-backgrounds-design.md`

---

## Task 1: Add New Enum Variants

**Files:**
- Modify: `src/ui/zone_bg.rs:48-71` (CelestialType and WeatherType enums)

**Step 1: Add CelestialType variants**

In `src/ui/zone_bg.rs`, add three new variants to the `CelestialType` enum (after `EmberGlow`, before `None`):

```rust
enum CelestialType {
    Sun,
    Moon,
    Stars,
    Wisps,
    BioLuminescent,
    Overcast,
    EmberGlow,
    CrackedSky,   // NEW: bright fracture lines across upper sky
    VoidRift,     // NEW: dark tear in sky with purple edge glow
    Flicker,      // NEW: points that blink in/out of existence
    None,
}
```

**Step 2: Add WeatherType variants**

Add six new variants to the `WeatherType` enum (after `Sparkles`):

```rust
enum WeatherType {
    None,
    Snow,
    Embers,
    Sparks,
    Bubbles,
    VoidParticles,
    WindStreaks,
    Sparkles,
    AshRain,       // NEW: dense downward grey/orange ash
    GlassShards,   // NEW: reflective falling fragments
    DriftingAsh,   // NEW: slow sparse dark ash floating laterally
    DustMotes,     // NEW: faint amber particles suspended in stillness
    StaticNoise,   // NEW: random flickering characters (visual snow)
    FractureMotes, // NEW: dim pulsing particles that appear/vanish
}
```

**Step 3: Add match arms for new celestial types in `paint_celestial()`**

In the `paint_celestial()` function (~line 167), add match arms for the three new types. For now, use placeholder empty bodies — they'll be implemented in Task 2.

```rust
CelestialType::CrackedSky => {
    paint_cracked_sky(buffer, width, height, millis);
}
CelestialType::VoidRift => {
    paint_void_rift(buffer, width, height, millis);
}
CelestialType::Flicker => {
    paint_flicker(buffer, width, height, millis);
}
```

**Step 4: Add match arms for new weather types in `paint_weather()`**

In the `paint_weather()` function (~line 473), add match arms:

```rust
WeatherType::AshRain => paint_ash_rain(buffer, millis, config.weather_intensity),
WeatherType::GlassShards => paint_glass_shards(buffer, millis, config.weather_intensity),
WeatherType::DriftingAsh => paint_drifting_ash(buffer, millis, config.weather_intensity),
WeatherType::DustMotes => paint_dust_motes(buffer, millis, config.weather_intensity),
WeatherType::StaticNoise => paint_static_noise(buffer, millis, config.weather_intensity),
WeatherType::FractureMotes => paint_fracture_motes(buffer, millis, config.weather_intensity),
```

**Step 5: Add stub functions for all new paint/overlay functions**

Add empty stub functions so the code compiles. Each stub takes the same signature as existing functions of its type. Place them in the appropriate sections of the file.

```rust
// Celestial stubs (after paint_emberglow)
fn paint_cracked_sky(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {}
fn paint_void_rift(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {}
fn paint_flicker(buffer: &mut [Vec<SceneCell>], width: usize, height: usize, millis: f64) {}

// Weather stubs (after paint_sparkles)
fn paint_ash_rain(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {}
fn paint_glass_shards(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {}
fn paint_drifting_ash(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {}
fn paint_dust_motes(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {}
fn paint_static_noise(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {}
fn paint_fracture_motes(buffer: &mut [Vec<SceneCell>], millis: f64, intensity: f64) {}

// Overlay stubs (after overlay_void_pulse)
fn overlay_heat_distortion(buffer: &mut [Vec<SceneCell>], millis: f64) {}
fn overlay_mirror_flash(buffer: &mut [Vec<SceneCell>], millis: f64) {}
fn overlay_consuming_dark(buffer: &mut [Vec<SceneCell>], millis: f64) {}
fn overlay_hollow_echo(buffer: &mut [Vec<SceneCell>], millis: f64) {}
fn overlay_reality_tear(buffer: &mut [Vec<SceneCell>], millis: f64) {}
fn overlay_wound_pulse(buffer: &mut [Vec<SceneCell>], millis: f64) {}
```

**Step 6: Verify compilation**

Run: `cargo build 2>&1 | head -20`
Expected: compiles with only dead_code warnings for the stubs.

**Step 7: Commit**

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add enum variants and stubs for fracture zone FX"
```

---

## Task 2: Implement New Celestial Paint Functions

**Files:**
- Modify: `src/ui/zone_bg.rs` (replace celestial stubs)

**Step 1: Implement `paint_cracked_sky()`**

Bright fracture lines across the upper sky, like cracked glass. Uses diagonal line segments.

```rust
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
```

**Step 2: Implement `paint_void_rift()`**

A dark tear in the sky with faint purple edge glow.

```rust
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
```

**Step 3: Implement `paint_flicker()`**

Points that blink in and out of existence — reality failing.

```rust
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
```

**Step 4: Verify compilation**

Run: `cargo build 2>&1 | head -20`
Expected: compiles cleanly (stubs replaced with implementations).

**Step 5: Commit**

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: implement cracked sky, void rift, and flicker celestial types"
```

---

## Task 3: Implement New Weather Paint Functions (Part 1: AshRain, GlassShards, DriftingAsh)

**Files:**
- Modify: `src/ui/zone_bg.rs` (replace weather stubs)

**Step 1: Implement `paint_ash_rain()`**

Dense downward grey/orange ash particles. Similar to snow but with warm tones and faster fall.

```rust
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
```

**Step 2: Implement `paint_glass_shards()`**

Reflective falling fragments that alternate between bright and dim, like light catching shards of mirror.

```rust
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
```

**Step 3: Implement `paint_drifting_ash()`**

Slow sparse dark ash floating laterally (horizontal drift, not vertical fall).

```rust
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
```

**Step 4: Verify compilation**

Run: `cargo build 2>&1 | head -20`
Expected: compiles cleanly.

**Step 5: Commit**

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: implement ash rain, glass shards, and drifting ash weather types"
```

---

## Task 4: Implement New Weather Paint Functions (Part 2: DustMotes, StaticNoise, FractureMotes)

**Files:**
- Modify: `src/ui/zone_bg.rs` (replace remaining weather stubs)

**Step 1: Implement `paint_dust_motes()`**

Faint amber particles suspended in stillness — nearly motionless, just gently pulsing.

```rust
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
```

**Step 2: Implement `paint_static_noise()`**

Random flickering characters — visual snow / TV static effect.

```rust
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
```

**Step 3: Implement `paint_fracture_motes()`**

Dim pulsing particles that appear and vanish — the subtlest weather type.

```rust
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
```

**Step 4: Verify compilation**

Run: `cargo build 2>&1 | head -20`
Expected: compiles cleanly.

**Step 5: Commit**

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: implement dust motes, static noise, and fracture motes weather types"
```

---

## Task 5: Implement New Overlay Functions

**Files:**
- Modify: `src/ui/zone_bg.rs` (replace overlay stubs)

**Step 1: Implement `overlay_heat_distortion()`**

Subtle horizontal color waver in lower half of scene — heat shimmer effect.

```rust
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
```

**Step 2: Implement `overlay_mirror_flash()`**

Brief reflective bright streaks — light catching mirror shards.

```rust
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
```

**Step 3: Implement `overlay_consuming_dark()`**

Bottom-up darkness creep that pulses — the Black Mouth feeding.

```rust
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
```

**Step 4: Implement `overlay_hollow_echo()`**

Periodic desaturation wave sweeping horizontally across the scene.

```rust
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
```

**Step 5: Implement `overlay_reality_tear()`**

Horizontal bands of color inversion flickering — reality failing.

```rust
fn overlay_reality_tear(buffer: &mut [Vec<SceneCell>], millis: f64) {
    let height = buffer.len();
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
```

**Step 6: Implement `overlay_wound_pulse()`**

Slow full-scene brightness oscillation between near-black and dim — the wound breathing.

```rust
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
```

**Step 7: Verify compilation**

Run: `cargo build 2>&1 | head -20`
Expected: compiles cleanly.

**Step 8: Commit**

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: implement 6 new overlay functions for fracture zone FX"
```

---

## Task 6: Add Zone Config Functions — Ch.1 The Red Fault (Z12-14)

**Files:**
- Modify: `src/ui/zone_bg.rs` (add config functions, update match)

**Step 1: Add `config_splintered_rim()` (Z12)**

```rust
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
```

**Step 2: Add `config_ember_ravine()` (Z13)**

```rust
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
```

**Step 3: Add `config_heart_of_the_fault()` (Z14)**

```rust
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
```

**Step 4: Add match arms for Z12-14**

In `zone_scene_config()`, add before the `_ =>` fallback:

```rust
12 => config_splintered_rim(),
13 => config_ember_ravine(),
14 => config_heart_of_the_fault(),
```

**Step 5: Verify and commit**

Run: `cargo build 2>&1 | head -20`

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add zone background configs for Ch.1 The Red Fault (Z12-14)"
```

---

## Task 7: Add Zone Config Functions — Ch.2 The Mirror Scar (Z15-17)

**Files:**
- Modify: `src/ui/zone_bg.rs`

**Step 1: Add `config_shard_fields()` (Z15)**

```rust
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
```

**Step 2: Add `config_refraction_steps()` (Z16)**

```rust
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
```

**Step 3: Add `config_hall_of_second_suns()` (Z17)**

```rust
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
```

**Step 4: Add match arms for Z15-17 and verify/commit**

```rust
15 => config_shard_fields(),
16 => config_refraction_steps(),
17 => config_hall_of_second_suns(),
```

Run: `cargo build 2>&1 | head -20`

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add zone background configs for Ch.2 The Mirror Scar (Z15-17)"
```

---

## Task 8: Add Zone Config Functions — Ch.3 The Black Mouth (Z18-20)

**Files:**
- Modify: `src/ui/zone_bg.rs`

**Step 1: Add `config_ashen_verge()` (Z18)**

```rust
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
```

**Step 2: Add `config_throat_of_the_world()` (Z19)**

```rust
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
```

**Step 3: Add `config_the_black_mouth()` (Z20)**

```rust
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
```

**Step 4: Add match arms for Z18-20 and verify/commit**

```rust
18 => config_ashen_verge(),
19 => config_throat_of_the_world(),
20 => config_the_black_mouth(),
```

Run: `cargo build 2>&1 | head -20`

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add zone background configs for Ch.3 The Black Mouth (Z18-20)"
```

---

## Task 9: Add Zone Config Functions — Ch.4 The Hollow Throne (Z21-23)

**Files:**
- Modify: `src/ui/zone_bg.rs`

**Step 1: Add `config_sunken_processional()` (Z21)**

```rust
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
```

**Step 2: Add `config_the_pale_archive()` (Z22)**

```rust
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
```

**Step 3: Add `config_the_hollow_throne()` (Z23)**

```rust
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
```

**Step 4: Add match arms for Z21-23 and verify/commit**

```rust
21 => config_sunken_processional(),
22 => config_the_pale_archive(),
23 => config_the_hollow_throne(),
```

Run: `cargo build 2>&1 | head -20`

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add zone background configs for Ch.4 The Hollow Throne (Z21-23)"
```

---

## Task 10: Add Zone Config Functions — Ch.5 The Wailing Reach (Z24-26)

**Files:**
- Modify: `src/ui/zone_bg.rs`

**Step 1: Add `config_the_stillborn_sea()` (Z24)**

```rust
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
            glyph: '\u{2592}', // ▒
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
```

**Step 2: Add `config_resonance_fault()` (Z25)**

```rust
/// Zone 25: Resonance Fault -- vibrating teal-purple, crystallized sound, light static.
fn config_resonance_fault() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (30, 80, 100),
        sky_bottom: (50, 20, 70),
        celestial: CelestialType::Flicker,
        far_terrain: TerrainProfile {
            glyph: '\u{2502}', // │
            color: (60, 120, 140),
            base_height: 0.46,
            amplitude: 0.08,
            frequency: 0.22,
            speed: 0.00010, // vibrating
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
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
```

**Step 3: Add `config_the_wailing_reach()` (Z26)**

```rust
/// Zone 26: The Wailing Reach -- flickering unstable reality, heavy static, reality tears.
fn config_the_wailing_reach() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (50, 45, 60),
        sky_bottom: (20, 15, 30),
        celestial: CelestialType::Flicker,
        far_terrain: TerrainProfile {
            glyph: '\u{2571}', // ╱
            color: (70, 50, 80),
            base_height: 0.48,
            amplitude: 0.10,
            frequency: 0.20,
            speed: 0.00012,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
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
```

**Step 4: Add match arms for Z24-26 and verify/commit**

```rust
24 => config_the_stillborn_sea(),
25 => config_resonance_fault(),
26 => config_the_wailing_reach(),
```

Run: `cargo build 2>&1 | head -20`

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add zone background configs for Ch.5 The Wailing Reach (Z24-26)"
```

---

## Task 11: Add Zone Config Functions — Ch.6 The Origin Wound (Z27-30)

**Files:**
- Modify: `src/ui/zone_bg.rs`

**Step 1: Add `config_the_scar_root()` (Z27)**

```rust
/// Zone 27: The Scar Root -- dark rust, petrified fracture roots, faint motes.
fn config_the_scar_root() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (50, 25, 15),
        sky_bottom: (12, 6, 4),
        celestial: CelestialType::Stars,
        far_terrain: TerrainProfile {
            glyph: '\u{2571}', // ╱
            color: (80, 35, 20),
            base_height: 0.48,
            amplitude: 0.09,
            frequency: 0.16,
            speed: 0.00006,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2593}', // ▓
            color: (35, 15, 10),
            base_height: 0.66,
            amplitude: 0.07,
            frequency: 0.12,
            speed: 0.00005,
            fill: true,
        },
        ground_glyphs: &[':', '.'],
        ground_color: (60, 30, 18),
        weather: WeatherType::FractureMotes,
        weather_intensity: 0.8,
        overlay: Some(overlay_wound_pulse),
    }
}
```

**Step 2: Add `config_echoing_abyss()` (Z28)**

```rust
/// Zone 28: Echoing Abyss -- pure black void, faint distant edges, sparse motes.
fn config_echoing_abyss() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (6, 4, 8),
        sky_bottom: (3, 2, 5),
        celestial: CelestialType::Flicker,
        far_terrain: TerrainProfile {
            glyph: ' ', // vast emptiness -- no far terrain
            color: (0, 0, 0),
            base_height: 0.50,
            amplitude: 0.0,
            frequency: 0.10,
            speed: 0.00005,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (18, 12, 22),
            base_height: 0.72,
            amplitude: 0.04,
            frequency: 0.10,
            speed: 0.00004,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::FractureMotes,
        weather_intensity: 0.4,
        overlay: Some(overlay_wound_pulse),
    }
}
```

**Step 3: Add `config_threshold_of_silence()` (Z29)**

```rust
/// Zone 29: Threshold of Silence -- light dying, a single fading horizon, emptiness.
fn config_threshold_of_silence() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (100, 95, 90),  // starts light
        sky_bottom: (5, 4, 3),    // fades to near-black
        celestial: CelestialType::None,
        far_terrain: TerrainProfile {
            glyph: '\u{2500}', // ─  single fading horizon
            color: (60, 55, 50),
            base_height: 0.55,
            amplitude: 0.01, // nearly flat
            frequency: 0.06,
            speed: 0.00002,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: ' ', // no near terrain
            color: (0, 0, 0),
            base_height: 0.90,
            amplitude: 0.0,
            frequency: 0.10,
            speed: 0.00005,
            fill: false,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::None, // silence
        weather_intensity: 1.0,
        overlay: Some(overlay_wound_pulse),
    }
}
```

**Step 4: Add `config_the_origin_wound()` (Z30)**

```rust
/// Zone 30: The Origin Wound -- deep void-purple, fracture cross pattern, the wound itself.
fn config_the_origin_wound() -> ZoneSceneConfig {
    ZoneSceneConfig {
        sky_top: (20, 5, 30),
        sky_bottom: (3, 1, 5),
        celestial: CelestialType::Flicker,
        far_terrain: TerrainProfile {
            glyph: '\u{2573}', // ╳
            color: (40, 12, 35),
            base_height: 0.50,
            amplitude: 0.08,
            frequency: 0.16,
            speed: 0.00008,
            fill: false,
        },
        near_terrain: TerrainProfile {
            glyph: '\u{2591}', // ░
            color: (15, 5, 18),
            base_height: 0.68,
            amplitude: 0.05,
            frequency: 0.12,
            speed: 0.00006,
            fill: true,
        },
        ground_glyphs: &[],
        ground_color: (0, 0, 0),
        weather: WeatherType::FractureMotes,
        weather_intensity: 0.3,
        overlay: Some(overlay_wound_pulse),
    }
}
```

**Step 5: Add match arms for Z27-30 and verify/commit**

```rust
27 => config_the_scar_root(),
28 => config_echoing_abyss(),
29 => config_threshold_of_silence(),
30 => config_the_origin_wound(),
```

Run: `cargo build 2>&1 | head -20`

```bash
git add src/ui/zone_bg.rs
git commit -m "feat: add zone background configs for Ch.6 The Origin Wound (Z27-30)"
```

---

## Task 12: Final Verification and Cleanup

**Files:**
- Modify: `src/ui/zone_bg.rs` (cleanup only)

**Step 1: Run full CI checks**

```bash
make check
```

Expected: All checks pass (fmt, clippy, tests, build, audit).

**Step 2: Fix any clippy warnings**

Address any clippy warnings about unused parameters, unused variables, or style issues in the new code.

**Step 3: Run `make fmt` to ensure formatting is correct**

```bash
make fmt
```

**Step 4: Verify the fallback is now only for truly unknown zone IDs**

Confirm that the `zone_scene_config()` match now covers zones 1-30 explicitly, with `_` only for zone IDs > 30.

**Step 5: Final commit if any changes needed**

```bash
git add src/ui/zone_bg.rs
git commit -m "chore: clean up fracture zone background code per clippy/fmt"
```

---

## Task Dependencies

```
Task 1 (enum stubs) ─┬─→ Task 2 (celestial FX)  ─┐
                      ├─→ Task 3 (weather pt1)     ├─→ Task 12 (verification)
                      ├─→ Task 4 (weather pt2)     │
                      ├─→ Task 5 (overlay FX)      │
                      ├─→ Task 6 (Ch.1 Z12-14)  ──┤
                      ├─→ Task 7 (Ch.2 Z15-17)  ──┤
                      ├─→ Task 8 (Ch.3 Z18-20)  ──┤
                      ├─→ Task 9 (Ch.4 Z21-23)  ──┤
                      ├─→ Task 10 (Ch.5 Z24-26) ──┤
                      └─→ Task 11 (Ch.6 Z27-30) ──┘
```

Task 1 must complete first (enum variants and stubs). Tasks 2-11 can then be parallelized across team members. Task 12 runs after all others complete.

## Team Assignment Suggestion

Given the requested team of 12:
- **Sys Architect (1)**: Task 1 (enum scaffolding), Task 12 (final verification)
- **Devs (5)**: Tasks 2-5 (FX implementations, 1 dev per task, 1 dev flex), Tasks 6-11 (zone configs)
- **QA (2)**: Visual verification of each chapter's zones after implementation
- **PM (1)**: Coordinate task dependencies and review progress
- **Game Designer (1)**: Review zone configs against narrative, tune palettes
- **Half-Block Artists (2)**: Tune terrain profiles, ground glyphs, and color palettes for each zone
