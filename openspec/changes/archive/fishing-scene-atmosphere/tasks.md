> Backported implementation plan (completed — this work shipped).

## 2026-03-07-fishing-scene-atmosphere-plan.md

# Fishing Scene Atmosphere Enhancement - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add stylized atmospheric depth to the fishing scene's mountains, sky, and horizon transition.

**Architecture:** All changes are within `SkyShape::draw()` in `src/ui/fishing_scene.rs`. The existing shoreline code (section 5, lines 294-320) is replaced with 3-layer mountains. New sky atmosphere and horizon effects are added as additional rendering passes in the existing pipeline. No new structs, files, or dependencies.

**Tech Stack:** Ratatui Canvas HalfBlock rendering, `scene_fx::{hash2d, lerp_rgb, lerp_channel}` helpers.

**Design doc:** `docs/plans/2026-03-07-fishing-scene-atmosphere-design.md`

---

### Task 1: 3-Layer Mountain Silhouettes

**Files:**
- Modify: `src/ui/fishing_scene.rs` — replace section 5 (lines 294-320) in `SkyShape::draw()`

**Step 1: Replace the shoreline silhouettes with 3-layer mountains**

Replace the existing section 5 comment and code block (lines 294-320) with:

```rust
        // 5. Mountain silhouettes — 3 depth layers with textured ridgelines
        // Layers: far (tallest, most faded), mid, near (shortest, most saturated)
        struct MountainLayer {
            // sine wave components: (frequency, amplitude, phase_offset)
            waves: [(f64, f64, f64); 3],
            base_height: f64,      // minimum height in pixels
            color_day: (u8, u8, u8),
            color_night: (u8, u8, u8),
            peak_tint_day: (u8, u8, u8),   // lighter color for peak pixels
            peak_tint_night: (u8, u8, u8),
            has_tree_fringe: bool,
        }

        let layers = [
            // Far range — gentle tall peaks, blue-grey, most faded
            MountainLayer {
                waves: [(0.08, 4.0, 0.0), (0.19, 2.0, 1.2), (0.37, 0.8, 3.7)],
                base_height: 3.0,
                color_day: (100, 120, 145),
                color_night: (50, 60, 82),
                peak_tint_day: (120, 140, 165),
                peak_tint_night: (62, 72, 95),
                has_tree_fringe: false,
            },
            // Mid range — moderate peaks, blue-green
            MountainLayer {
                waves: [(0.12, 3.0, 2.5), (0.26, 1.8, 0.8), (0.45, 0.6, 5.1)],
                base_height: 2.0,
                color_day: (78, 105, 120),
                color_night: (42, 58, 72),
                peak_tint_day: (95, 122, 138),
                peak_tint_night: (54, 70, 85),
                has_tree_fringe: false,
            },
            // Near range — sharp short peaks, dark green-grey, tree fringe
            MountainLayer {
                waves: [(0.15, 2.5, 1.0), (0.33, 1.5, 4.2), (0.52, 0.5, 2.3)],
                base_height: 1.5,
                color_day: (60, 85, 95),
                color_night: (35, 48, 62),
                peak_tint_day: (75, 100, 112),
                peak_tint_night: (45, 58, 74),
                has_tree_fringe: true,
            },
        ];

        for layer in &layers {
            let base_color = lerp_rgb(layer.color_day, layer.color_night, self.dusk);
            let peak_color = lerp_rgb(layer.peak_tint_day, layer.peak_tint_night, self.dusk);

            for col in 0..self.width {
                // Sum sine waves + hash jitter for ridgeline height
                let mut h = layer.base_height;
                for &(freq, amp, phase) in &layer.waves {
                    h += (col as f64 * freq + phase).sin() * amp;
                }
                // Per-column jitter from hash for irregularity
                let jitter = (hash2d(42 + col, layer.base_height as usize) % 5) as f64 * 0.4 - 0.8;
                h = (h + jitter).max(1.0);

                let pixels = (h * 2.0).round() as i32;
                for dy in 0..pixels {
                    let gy = sky_py as i32 - 1 - dy;
                    // Vertical gradient: base at bottom, peak tint at top
                    let vert_t = if pixels <= 1 {
                        1.0
                    } else {
                        dy as f64 / (pixels - 1) as f64
                    };
                    let mut color = lerp_rgb(base_color, peak_color, vert_t);

                    // Tree fringe: alternating hash-based texture on top 2 pixels
                    if layer.has_tree_fringe && dy >= pixels - 2 {
                        let is_dark = hash2d(col, dy as usize).is_multiple_of(3);
                        if is_dark {
                            color = (
                                color.0.saturating_sub(12),
                                color.1.saturating_sub(8),
                                color.2.saturating_sub(10),
                            );
                        } else {
                            color = (
                                color.0.saturating_add(6),
                                color.1.saturating_add(10),
                                color.2.saturating_add(4),
                            );
                        }
                    }

                    self.paint_px(painter, col as i32, gy, color);
                }
            }
        }
```

**Step 2: Build and visually verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles without errors.

Then run `cargo run`, start a fishing session, and confirm:
- 3 distinct mountain layers visible above the water
- Far layer tallest and most faded, near layer shortest and most saturated
- Ridgelines are jagged (not smooth sine waves)
- Near layer has subtle texture on top pixels
- Colors shift properly during the dusk cycle

**Step 3: Commit**

```bash
git add src/ui/fishing_scene.rs
git commit -m "feat(fishing): replace shoreline with 3-layer mountain silhouettes"
```

---

### Task 2: Sky Haze Bands

**Files:**
- Modify: `src/ui/fishing_scene.rs` — add new section after horizon color bleed (after line 222), before celestial body

**Step 1: Add haze bands between sections 1b and 2**

Insert after the closing brace of section 1b (horizon color bleed) and before the section 2 comment:

```rust
        // 1c. Haze bands — thin warm-tinted bands in lower sky, slow drift
        {
            let haze_bands: [(f64, f64, f64); 3] = [
                // (y_ratio in sky, drift_speed, opacity)
                (0.68, 0.020, 0.18),
                (0.76, 0.015, 0.15),
                (0.84, 0.022, 0.12),
            ];
            let haze_warm_day = (220u8, 210, 195);
            let haze_warm_dusk = (240u8, 170, 120);
            let haze_tint = lerp_rgb(haze_warm_day, haze_warm_dusk, self.dusk);

            for &(y_ratio, drift_speed, opacity) in &haze_bands {
                let row = (sky_py as f64 * y_ratio).round() as usize;
                if row >= sky_py {
                    continue;
                }
                let drift = (self.wave_tick * drift_speed) % self.width as f64;
                for gx in 0..self.width {
                    // Patchy coverage via sine modulation
                    let coverage = ((gx as f64 + drift) * 0.12).sin() * 0.5 + 0.5;
                    if coverage < 0.3 {
                        continue;
                    }
                    let blend = opacity * coverage;
                    let py_t = row as f64 / (sky_py - 1).max(1) as f64;
                    let base = lerp_rgb(top, low, py_t);
                    let blended = lerp_rgb(base, haze_tint, blend);
                    painter.paint(gx, row, Color::Rgb(blended.0, blended.1, blended.2));
                    // Paint second pixel row for 2px tall band
                    if row + 1 < sky_py {
                        let base2 = lerp_rgb(top, low, (row + 1) as f64 / (sky_py - 1).max(1) as f64);
                        let blended2 = lerp_rgb(base2, haze_tint, blend * 0.7);
                        painter.paint(gx, row + 1, Color::Rgb(blended2.0, blended2.1, blended2.2));
                    }
                }
            }
        }
```

**Step 2: Build and visually verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles without errors.

Then run `cargo run`, start fishing, and confirm:
- Faint warm-tinted horizontal bands visible in lower sky
- Bands drift slowly left/right
- Bands have patchy coverage (not solid lines)
- During dusk, bands pick up warmer orange tones

**Step 3: Commit**

```bash
git add src/ui/fishing_scene.rs
git commit -m "feat(fishing): add drifting haze bands to lower sky"
```

---

### Task 3: High-Altitude Wisps

**Files:**
- Modify: `src/ui/fishing_scene.rs` — add after haze bands, before celestial body

**Step 1: Add wisp rendering after haze bands**

Insert after the haze bands block and before the section 2 comment:

```rust
        // 1d. High-altitude wisps — faint diagonal streaks in upper sky
        {
            let wisps: [(f64, f64, f64, i32, f64); 4] = [
                // (base_x, y_ratio, drift_speed, length, brightness_boost)
                (8.0, 0.15, 0.012, 5, 0.04),
                (22.0, 0.22, 0.010, 4, 0.03),
                (38.0, 0.12, 0.014, 6, 0.05),
                (52.0, 0.20, 0.009, 3, 0.03),
            ];

            for &(base_x, y_ratio, drift_speed, length, boost) in &wisps {
                let drift = (self.wave_tick * drift_speed) % self.width as f64;
                let start_x = (base_x + drift).rem_euclid(self.width as f64) as i32;
                let row = (sky_py as f64 * y_ratio).round() as usize;
                if row >= sky_py.saturating_sub(2) {
                    continue;
                }

                for dx in 0..length {
                    let gx = ((start_x + dx) as usize % self.width) as i32;
                    // Slight diagonal: every 2 pixels, shift down 1
                    let dy = dx / 2;
                    let gy = row + dy as usize;
                    if gy >= sky_py {
                        continue;
                    }

                    let py_t = gy as f64 / (sky_py - 1).max(1) as f64;
                    let base = lerp_rgb(top, low, py_t);
                    // Brighten slightly
                    let wisp_color = (
                        base.0.saturating_add((boost * 255.0) as u8),
                        base.1.saturating_add((boost * 255.0) as u8),
                        base.2.saturating_add((boost * 240.0) as u8),
                    );
                    self.paint_px(painter, gx, gy, wisp_color);
                }
            }
        }
```

**Step 2: Build and visually verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles without errors.

Run `cargo run`, start fishing, and confirm:
- Faint short diagonal streaks in upper sky
- Barely visible — just slightly brighter than sky behind them
- Drift very slowly (noticeably slower than clouds)

**Step 3: Commit**

```bash
git add src/ui/fishing_scene.rs
git commit -m "feat(fishing): add high-altitude wisp streaks to upper sky"
```

---

### Task 4: Mountain Light Bleed

**Files:**
- Modify: `src/ui/fishing_scene.rs` — add after mountain silhouettes, before sailboat

**Step 1: Add light bleed after mountains**

Insert after the mountain layers loop closes and before the section 6 (sailboat) comment:

```rust
        // 5b. Mountain light bleed — backlit glow where orb meets mountain edge
        {
            let orb_glow_color = if self.dusk < 0.56 {
                (255u8, 220, 140) // warm gold for sun
            } else {
                (200u8, 215, 240) // cool white for moon
            };

            // Recompute orb position (same as section 2)
            let orb_col_f =
                self.width as f64 * 0.78 + (self.wave_tick * 0.08).sin() * 3.0;

            for col in 0..self.width {
                let dist_to_orb = (col as f64 - orb_col_f).abs();
                if dist_to_orb > 6.0 {
                    continue;
                }

                // Find top of tallest mountain at this column (recompute near layer height)
                let near = &layers[2]; // near layer is tallest visually at horizon
                let mut h = near.base_height;
                for &(freq, amp, phase) in &near.waves {
                    h += (col as f64 * freq + phase).sin() * amp;
                }
                let jitter =
                    (hash2d(42 + col, near.base_height as usize) % 5) as f64 * 0.4 - 0.8;
                h = (h + jitter).max(1.0);
                let peak_gy = sky_py as i32 - (h * 2.0).round() as i32;

                // Paint glow on 1-2 pixels above the mountain peak
                let glow_strength = (1.0 - dist_to_orb / 6.0) * 0.20;
                for dy in 0..2i32 {
                    let gy = peak_gy - 1 - dy;
                    if gy < 0 || gy as usize >= sky_py {
                        continue;
                    }
                    let fade = glow_strength * (1.0 - dy as f64 * 0.5);
                    let py_t = gy as f64 / (sky_py - 1).max(1) as f64;
                    let base = lerp_rgb(top, low, py_t);
                    let blended = lerp_rgb(base, orb_glow_color, fade);
                    self.paint_px(painter, col as i32, gy, blended);
                }
            }
        }
```

**Step 2: Build and visually verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles without errors.

Run `cargo run`, start fishing, and confirm:
- Subtle warm glow visible on sky pixels just above mountain peaks near the sun/moon
- Glow fades as distance from celestial body increases
- Color is gold during day, cool white at night

**Step 3: Commit**

```bash
git add src/ui/fishing_scene.rs
git commit -m "feat(fishing): add mountain light bleed from celestial body"
```

---

### Task 5: Horizon Mist

**Files:**
- Modify: `src/ui/fishing_scene.rs` — add after light bleed, before sailboat

**Step 1: Add mist layer after light bleed**

Insert after the light bleed block and before the section 6 (sailboat) comment:

```rust
        // 5c. Horizon mist — semi-transparent fog patches at mountain base
        {
            let mist_color_day = (180u8, 200, 215);
            let mist_color_night = (100u8, 110, 130);
            let mist_tint = lerp_rgb(mist_color_day, mist_color_night, self.dusk);
            let mist_drift = (self.wave_tick * 0.024) % self.width as f64;
            let mist_band = 4.min(sky_py); // 4 pixels tall max

            for gy in (sky_py - mist_band)..sky_py {
                let band_t = (sky_py - gy) as f64 / mist_band.max(1) as f64; // 1.0 at top, 0.0 at bottom
                let base_opacity = 0.28 * band_t; // stronger at top, fading toward water

                for gx in 0..self.width {
                    // Patchy sine coverage with drift
                    let patch = ((gx as f64 + mist_drift) * 0.09).sin()
                        * ((gx as f64 + mist_drift * 1.3) * 0.17).cos();
                    if patch < 0.1 {
                        continue;
                    }
                    let opacity = base_opacity * ((patch - 0.1) / 0.9).min(1.0);

                    let py_t = gy as f64 / (sky_py - 1).max(1) as f64;
                    let base = lerp_rgb(top, low, py_t);
                    let blended = lerp_rgb(base, mist_tint, opacity);
                    painter.paint(gx, gy, Color::Rgb(blended.0, blended.1, blended.2));
                }
            }
        }
```

**Step 2: Build and visually verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles without errors.

Run `cargo run`, start fishing, and confirm:
- Patchy mist visible at the base of the mountains
- Mist drifts slowly sideways
- Gaps in the mist reveal mountain base behind
- Mist is stronger/more opaque at the top of the band, fading toward water
- Color shifts with dusk cycle

**Step 3: Commit**

```bash
git add src/ui/fishing_scene.rs
git commit -m "feat(fishing): add horizon mist patches at mountain base"
```

---

### Task 6: Final Build Verification

**Files:**
- None (verification only)

**Step 1: Run full CI checks**

Run: `make check`
Expected: all checks pass (format, clippy, tests, build, audit).

**Step 2: Visual smoke test**

Run `cargo run`, start a fishing session, and watch for one full dusk cycle (~16 seconds). Confirm:
- Mountains have 3 visible depth layers with color gradients
- Near-layer tree fringe is subtle, not distracting
- Haze bands drift slowly in lower sky, patchy
- Wisps are barely visible in upper sky
- Light bleed appears when sun/moon is near mountain peaks
- Mist drifts at mountain base with gaps
- Water scene, boat, bobber, and fishing line all still work correctly
- No visual artifacts or flickering

**Step 3: Commit (if any final fixes needed)**

```bash
git add src/ui/fishing_scene.rs
git commit -m "fix(fishing): polish atmosphere rendering"
```
