//! Fishing scene UI rendering.
//!
//! Displays the active fishing session with animated water, catch progress,
//! caught fish list, and fishing rank progression.

use super::scene_fx::{current_millis, hash2d, lerp_channel, lerp_rgb};
use super::stats_prestige::{
    build_leviathan_tracker_line, build_leviathan_trophy_line, highest_fishing_badge,
};
use crate::achievements::Achievements;
use crate::core::game_state::GameState;
use crate::fishing::types::{FishingSession, FishingState, RANK_NAMES};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Painter, Shape},
        Block, Borders, Clear, Gauge, Paragraph, Wrap,
    },
    Frame,
};

/// Renders the fishing scene UI.
///
/// Layout: fishing header (rank + progress gauge, 2 rows) + water animation (fills rest).
/// The status strip (catch count + phase) is handled separately by the unified panel.
pub fn render_fishing_scene(
    frame: &mut Frame,
    area: Rect,
    session: &FishingSession,
    game_state: &GameState,
    achievements: &Achievements,
    _ctx: &super::responsive::LayoutContext,
) {
    // 3-part layout: header (2 rows) + divider (1 row) + water animation
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header: rank + progress (no border)
            Constraint::Length(1), // Divider line
            Constraint::Min(5),    // Water animation
        ])
        .split(area);

    // Draw header with rank and progress gauge
    draw_header(frame, chunks[0], session, game_state, achievements);

    // Draw horizontal divider matching the outer panel style
    let divider_char = super::panel_border_chars().h;
    let zone_color =
        super::stats_panel::zone_color_for_id(game_state.zone_progression.current_zone_id);
    let divider_str: String = std::iter::repeat_n(divider_char, chunks[1].width as usize).collect();
    let divider = Paragraph::new(divider_str.as_str())
        .style(Style::default().fg(super::themed_border_color(zone_color)));
    frame.render_widget(divider, chunks[1]);

    // Draw water animation with bobber
    draw_water_scene(frame, chunks[2], session);
}

/// Draws the fishing header: rank + progress gauge (or Leviathan tracker).
/// No border — separated from the water scene by a divider line rendered by the caller.
fn draw_header(
    frame: &mut Frame,
    area: Rect,
    _session: &FishingSession,
    game_state: &GameState,
    achievements: &Achievements,
) {
    use crate::achievements::AchievementId;

    if area.height < 1 {
        return;
    }

    let fishing = &game_state.fishing;
    let fish_required = FishingState::fish_required_for_rank(fishing.rank);
    let fish_progress = fishing.fish_toward_next_rank;
    let fish_ratio = if fish_required > 0 {
        (fish_progress as f64 / fish_required as f64).min(1.0)
    } else {
        0.0
    };

    let is_max_rank = fishing.rank as usize >= RANK_NAMES.len();
    let leviathan_caught = achievements.is_unlocked(AchievementId::StormLeviathan);
    let show_leviathan_hunt = is_max_rank && fishing.rank >= 40 && !leviathan_caught;
    let show_leviathan_trophy = is_max_rank && fishing.rank >= 40 && leviathan_caught;

    let badge = highest_fishing_badge(achievements);
    let rank_line = Line::from(vec![
        Span::styled(
            "\u{1F3A3} Rank: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ({})", fishing.rank_name(), fishing.rank),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(if let Some(icon) = badge {
            format!("  {}", icon)
        } else {
            String::new()
        }),
    ]);

    if area.height >= 2 {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        if show_leviathan_hunt {
            let hunt_line = Line::from(vec![Span::styled(
                "\u{1F40B} Storm Leviathan Hunt",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]);
            frame.render_widget(Paragraph::new(hunt_line), inner_chunks[0]);
            let tracker_line = build_leviathan_tracker_line(fishing);
            frame.render_widget(Paragraph::new(tracker_line), inner_chunks[1]);
        } else if show_leviathan_trophy {
            frame.render_widget(Paragraph::new(rank_line), inner_chunks[0]);
            let trophy_line = build_leviathan_trophy_line();
            frame.render_widget(Paragraph::new(trophy_line), inner_chunks[1]);
        } else {
            frame.render_widget(Paragraph::new(rank_line), inner_chunks[0]);
            let fish_label = if is_max_rank {
                "Max Rank".to_string()
            } else {
                let next_rank = fishing.rank + 1;
                let next_rank_name = RANK_NAMES[next_rank as usize - 1];
                format!(
                    "{}/{} to {} ({})",
                    fish_progress, fish_required, next_rank_name, next_rank
                )
            };
            let ratio = if is_max_rank { 1.0 } else { fish_ratio };
            let fish_gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Color::Blue)
                        .bg(Color::Rgb(8, 8, 28))
                        .add_modifier(Modifier::BOLD),
                )
                .label(fish_label)
                .ratio(ratio);
            frame.render_widget(fish_gauge, inner_chunks[1]);
        }
    } else {
        frame.render_widget(Paragraph::new(rank_line), area);
    }
}

/// Canvas HalfBlock sky: gradient, celestial body, stars, clouds, shoreline, sailboat.
struct SkyShape {
    width: usize,
    horizon: usize,
    wave_tick: f64,
    dusk: f64,
    mast_x: i32,
    mast_top: i32,
}

impl SkyShape {
    fn paint_px(&self, painter: &mut Painter, gx: i32, gy: i32, color: (u8, u8, u8)) {
        if gx >= 0 && (gx as usize) < self.width && gy >= 0 && (gy as usize) < self.horizon * 2 {
            painter.paint(
                gx as usize,
                gy as usize,
                Color::Rgb(color.0, color.1, color.2),
            );
        }
    }
}

impl Shape for SkyShape {
    fn draw(&self, painter: &mut Painter) {
        let sky_py = self.horizon * 2;
        if sky_py == 0 || self.width == 0 {
            return;
        }

        let top = lerp_rgb((118, 196, 248), (20, 34, 66), self.dusk);
        let low = lerp_rgb((210, 232, 252), (116, 96, 132), self.dusk);

        // 1. Sky gradient — per-pixel row interpolation
        for gy in 0..sky_py {
            let py_t = if sky_py <= 1 {
                0.0
            } else {
                gy as f64 / (sky_py - 1) as f64
            };
            let bg = lerp_rgb(top, low, py_t);
            for gx in 0..self.width {
                painter.paint(gx, gy, Color::Rgb(bg.0, bg.1, bg.2));
            }
        }

        // 1b. Horizon color bleed — warm sunset/dusk glow near horizon
        if self.dusk > 0.35 && sky_py > 0 {
            let glow_strength = ((self.dusk - 0.35) / 0.65).min(1.0);
            let glow_band = 6.min(sky_py);
            let warm = if self.dusk < 0.65 {
                (255, 160, 80) // sunset orange
            } else {
                (180, 100, 140) // deep dusk purple-pink
            };
            for gy in (sky_py - glow_band)..sky_py {
                let band_t = 1.0 - (sky_py - 1 - gy) as f64 / glow_band.max(1) as f64;
                let blend = band_t * glow_strength * 0.35;
                let py_t = gy as f64 / (sky_py - 1).max(1) as f64;
                let base = lerp_rgb(top, low, py_t);
                let blended = lerp_rgb(base, warm, blend);
                for gx in 0..self.width {
                    painter.paint(gx, gy, Color::Rgb(blended.0, blended.1, blended.2));
                }
            }
        }

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
                        let base2 =
                            lerp_rgb(top, low, (row + 1) as f64 / (sky_py - 1).max(1) as f64);
                        let blended2 = lerp_rgb(base2, haze_tint, blend * 0.7);
                        painter.paint(gx, row + 1, Color::Rgb(blended2.0, blended2.1, blended2.2));
                    }
                }
            }
        }

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
                    self.paint_px(painter, gx, gy as i32, wisp_color);
                }
            }
        }

        // 2. Celestial body — pixel circle with halo
        let orb_col =
            (self.width as f64 * 0.78 + (self.wave_tick * 0.08).sin() * 3.0).round() as i32;
        let orb_row = (self.horizon as f64 * 0.32 + (self.wave_tick * 0.04).sin()).round() as i32;
        let orb_cy = orb_row * 2;
        let (orb_bright, orb_dim) = if self.dusk < 0.56 {
            ((255u8, 228, 152), (240u8, 216, 140))
        } else {
            ((232u8, 238, 255), (200u8, 210, 235))
        };
        for dy in -3..=3i32 {
            for dx in -3..=3i32 {
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                if dist <= 1.2 {
                    self.paint_px(painter, orb_col + dx, orb_cy + dy, orb_bright);
                } else if dist <= 2.5 {
                    self.paint_px(painter, orb_col + dx, orb_cy + dy, orb_dim);
                }
            }
        }

        // 3. Stars — single bright pixels at hash-based positions
        if self.dusk > 0.25 {
            let twinkle_tick = (self.wave_tick / 1.8) as usize;
            for row in 0..self.horizon.saturating_sub(1) {
                for col in 0..self.width {
                    if hash2d(row, col).is_multiple_of(97) {
                        let bright =
                            hash2d(row + twinkle_tick, col + twinkle_tick).is_multiple_of(3);
                        let color = if bright {
                            (244, 246, 255)
                        } else {
                            (184, 190, 232)
                        };
                        let gy = row as i32 * 2;
                        self.paint_px(painter, col as i32, gy, color);
                        if bright {
                            self.paint_px(painter, col as i32, gy + 1, color);
                        }
                    }
                }
            }
        }

        // 4. Clouds — pixel clusters drifting
        for &(base_x, y_ratio, speed, pat_len, shade) in &[
            (6.0f64, 0.18f64, 0.040f64, 4, 148u8),
            (18.0, 0.28, 0.032, 6, 140),
            (34.0, 0.16, 0.052, 8, 132),
            (12.0, 0.44, 0.026, 5, 126),
            (44.0, 0.36, 0.036, 4, 130),
        ] {
            let drift = (self.wave_tick * speed) % self.width as f64;
            let cx = (base_x - drift).rem_euclid(self.width as f64) as i32;
            let row = (self.horizon as f64 * y_ratio).round() as usize;
            let gy = (row * 2) as i32;
            let tint = lerp_channel(shade, 208, 1.0 - self.dusk);
            let top_color = (
                tint.saturating_add(12),
                tint.saturating_add(12),
                tint.saturating_add(18),
            );
            let bot_color = (tint.saturating_sub(8), tint.saturating_sub(8), tint);
            for dx in 0..pat_len {
                let gx = ((cx + dx) as usize % self.width) as i32;
                self.paint_px(painter, gx, gy, top_color);
                self.paint_px(painter, gx, gy + 1, bot_color);
            }
        }

        // 4b. Birds — depth-varied soaring silhouettes with altitude bob
        //
        // One close bird (5px, darker, more bob) and two distant birds
        // (3px, fainter, less bob). All have 4-phase wing cycle.
        {
            // (base_x, y_ratio, speed, flap_offset, is_close)
            let birds: [(f64, f64, f64, f64, bool); 3] = [
                (5.0, 0.22, 0.035, 0.0, true),   // close
                (25.0, 0.14, 0.028, 1.7, false), // distant
                (42.0, 0.30, 0.042, 3.4, false), // distant
            ];

            for &(base_x, y_ratio, speed, flap_offset, is_close) in &birds {
                // Color: close bird is dark silhouette, distant birds are fainter
                let bird_color = if is_close {
                    lerp_rgb((25, 22, 18), (60, 55, 50), self.dusk)
                } else {
                    lerp_rgb((65, 62, 58), (90, 85, 80), self.dusk)
                };

                let drift = (self.wave_tick * speed) % self.width as f64;
                let cx = (base_x + drift).rem_euclid(self.width as f64).round() as i32;
                let row = (self.horizon as f64 * y_ratio).round() as usize;

                // Altitude bob: close bird bobs more (2px), distant birds less (1px)
                let bob_amp = if is_close { 2.0 } else { 1.0 };
                let bob = (self.wave_tick * 0.04 + flap_offset).sin() * bob_amp;
                let gy = (row * 2) as i32 + bob.round() as i32;

                let phase = ((self.wave_tick * 0.11 + flap_offset) as usize) % 4;

                self.paint_px(painter, cx, gy, bird_color);

                if is_close {
                    // 5px wide: body + 2 wing segments each side
                    match phase {
                        0 => {
                            self.paint_px(painter, cx - 1, gy - 1, bird_color);
                            self.paint_px(painter, cx - 2, gy - 2, bird_color);
                            self.paint_px(painter, cx + 1, gy - 1, bird_color);
                            self.paint_px(painter, cx + 2, gy - 2, bird_color);
                        }
                        2 => {
                            self.paint_px(painter, cx - 1, gy + 1, bird_color);
                            self.paint_px(painter, cx - 2, gy + 2, bird_color);
                            self.paint_px(painter, cx + 1, gy + 1, bird_color);
                            self.paint_px(painter, cx + 2, gy + 2, bird_color);
                        }
                        _ => {
                            self.paint_px(painter, cx - 1, gy, bird_color);
                            self.paint_px(painter, cx - 2, gy, bird_color);
                            self.paint_px(painter, cx + 1, gy, bird_color);
                            self.paint_px(painter, cx + 2, gy, bird_color);
                        }
                    }
                } else {
                    // 3px wide: body + 1 wingtip each side
                    match phase {
                        0 => {
                            self.paint_px(painter, cx - 1, gy - 1, bird_color);
                            self.paint_px(painter, cx + 1, gy - 1, bird_color);
                        }
                        2 => {
                            self.paint_px(painter, cx - 1, gy + 1, bird_color);
                            self.paint_px(painter, cx + 1, gy + 1, bird_color);
                        }
                        _ => {
                            self.paint_px(painter, cx - 1, gy, bird_color);
                            self.paint_px(painter, cx + 1, gy, bird_color);
                        }
                    }
                }
            }
        }

        // 4c. Wind streaks — faint diagonal lines drifting across upper sky
        {
            let streaks: [(f64, f64, f64, i32); 4] = [
                // (base_x, y_ratio, speed, length)
                (3.0, 0.08, 0.07, 4),
                (19.0, 0.18, 0.06, 3),
                (35.0, 0.10, 0.08, 5),
                (50.0, 0.24, 0.055, 3),
            ];

            for &(base_x, y_ratio, speed, length) in &streaks {
                let drift = (self.wave_tick * speed) % self.width as f64;
                let start_x = (base_x + drift).rem_euclid(self.width as f64).round() as i32;
                let row = (sky_py as f64 * y_ratio).round() as usize;

                for dx in 0..length {
                    let gx = ((start_x + dx) as usize % self.width) as i32;
                    // Very slight diagonal: 1px down every 3px across
                    let gy = row as i32 + dx / 3;
                    if gy < 0 || gy as usize >= sky_py {
                        continue;
                    }
                    // Barely visible — just a few points brighter than sky
                    let py_t = gy as f64 / (sky_py - 1).max(1) as f64;
                    let base = lerp_rgb(top, low, py_t);
                    let streak_color = (
                        base.0.saturating_add(8),
                        base.1.saturating_add(8),
                        base.2.saturating_add(6),
                    );
                    self.paint_px(painter, gx, gy, streak_color);
                }
            }
        }

        // 5. Bob Ross mountains — snow-capped peaks, rocky faces, evergreen tree line
        //
        // Two layers of asymmetric peaks. Each column is divided into vertical
        // zones: snow cap (top), rocky face with highlight streaks (middle),
        // evergreen tree line (bottom, near layer only).

        // Peak: (center_ratio, height, left_width, right_width)
        type Peak = (f64, f64, f64, f64);

        let far_peaks: &[Peak] = &[
            (0.18, 6.0, 6.0, 12.0),
            (0.52, 5.0, 5.0, 10.0),
            (0.82, 7.0, 7.0, 14.0),
        ];
        let near_peaks: &[Peak] = &[
            (0.08, 3.5, 3.0, 8.0),
            (0.38, 4.5, 4.0, 9.0),
            (0.72, 5.0, 4.0, 10.0),
        ];

        // Compute height for a column given a set of peaks
        let peak_height = |col: usize, peaks: &[Peak], w: usize, floor: f64| -> f64 {
            let x = col as f64;
            let mut best = floor;
            for &(center_ratio, height, left_w, right_w) in peaks {
                let cx = center_ratio * w as f64;
                let dist = x - cx;
                let slope_w = if dist < 0.0 { left_w } else { right_w };
                let t = 1.0 - (dist.abs() / slope_w).min(1.0);
                let h = floor + (height - floor) * t * t * (3.0 - 2.0 * t);
                if h > best {
                    best = h;
                }
            }
            let jitter = (hash2d(col, (floor * 10.0) as usize) % 3) as f64 * 0.3;
            (best + jitter).max(floor)
        };

        // --- Far layer (taller, hazier, snow cap + smooth rock gradient) ---
        {
            let snow_day = (235u8, 240, 245);
            let snow_night = (140u8, 150, 170);
            let rock_base_day = (80u8, 90, 115);
            let rock_base_night = (38u8, 42, 58);
            let rock_top_day = (130u8, 140, 165);
            let rock_top_night = (65u8, 72, 92);

            let snow = lerp_rgb(snow_day, snow_night, self.dusk);
            let rock_base = lerp_rgb(rock_base_day, rock_base_night, self.dusk);
            let rock_top = lerp_rgb(rock_top_day, rock_top_night, self.dusk);

            for col in 0..self.width {
                let h = peak_height(col, far_peaks, self.width, 0.8);
                let pixels = (h * 2.0).round() as i32;
                if pixels <= 0 {
                    continue;
                }

                let snow_line = (pixels as f64 * 0.72).round() as i32;

                for dy in 0..pixels {
                    let gy = sky_py as i32 - 1 - dy;
                    let color = if dy >= snow_line {
                        snow
                    } else {
                        // Smooth gradient: dark at base, lighter toward snow line
                        let rock_t = if snow_line <= 0 {
                            0.0
                        } else {
                            dy as f64 / snow_line as f64
                        };
                        lerp_rgb(rock_base, rock_top, rock_t)
                    };
                    self.paint_px(painter, col as i32, gy, color);
                }
            }
        }

        // --- Near layer (shorter, darker, snow cap + smooth rock gradient) ---
        {
            let snow_day = (225u8, 230, 235);
            let snow_night = (120u8, 130, 150);
            let rock_base_day = (55u8, 62, 75);
            let rock_base_night = (28u8, 32, 42);
            let rock_top_day = (100u8, 110, 125);
            let rock_top_night = (52u8, 58, 72);

            let snow = lerp_rgb(snow_day, snow_night, self.dusk);
            let rock_base = lerp_rgb(rock_base_day, rock_base_night, self.dusk);
            let rock_top = lerp_rgb(rock_top_day, rock_top_night, self.dusk);

            for col in 0..self.width {
                let h = peak_height(col, near_peaks, self.width, 0.5);
                let pixels = (h * 2.0).round() as i32;
                if pixels <= 0 {
                    continue;
                }

                let snow_line = (pixels as f64 * 0.75).round() as i32;

                for dy in 0..pixels {
                    let gy = sky_py as i32 - 1 - dy;
                    let color = if dy >= snow_line {
                        snow
                    } else {
                        let rock_t = if snow_line <= 0 {
                            0.0
                        } else {
                            dy as f64 / snow_line as f64
                        };
                        lerp_rgb(rock_base, rock_top, rock_t)
                    };
                    self.paint_px(painter, col as i32, gy, color);
                }
            }
        }

        // 5b. Mountain light bleed — backlit glow where orb meets mountain edge
        {
            let orb_glow_color = if self.dusk < 0.56 {
                (255u8, 220, 140) // warm gold for sun
            } else {
                (200u8, 215, 240) // cool white for moon
            };

            let orb_col_f = self.width as f64 * 0.78 + (self.wave_tick * 0.08).sin() * 3.0;

            for col in 0..self.width {
                let dist_to_orb = (col as f64 - orb_col_f).abs();
                if dist_to_orb > 6.0 {
                    continue;
                }

                let h = peak_height(col, near_peaks, self.width, 0.5);
                let peak_gy = sky_py as i32 - (h * 2.0).round() as i32;

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

        // 6. Sailboat sky — pixel art for pennant, sail, mast, rigging
        let mx = self.mast_x;
        let mt = self.mast_top * 2; // pixel y of mast_top row
        let pennant: (u8, u8, u8) = (255, 96, 72);
        let sail_edge: (u8, u8, u8) = (245, 235, 215);
        let sail_fill: (u8, u8, u8) = (238, 228, 208);
        let mast_c: (u8, u8, u8) = (210, 168, 104);
        let rigging: (u8, u8, u8) = (160, 130, 80);

        // Pennant (2 pixels at mast_top row, col+1)
        self.paint_px(painter, mx + 1, mt, pennant);
        self.paint_px(painter, mx + 1, mt + 1, pennant);
        self.paint_px(painter, mx + 2, mt, (220, 72, 52));
        self.paint_px(painter, mx + 2, mt + 1, (220, 72, 52));

        // Mast — vertical pixels from mast_top to mast_top+2 (3 terminal rows = 6 pixels)
        for dy in 0..6 {
            self.paint_px(painter, mx, mt + dy, mast_c);
        }

        // Sail row 1 (mast_top+1): diagonal edge
        let r1 = mt + 2;
        self.paint_px(painter, mx - 1, r1, sail_edge);
        self.paint_px(painter, mx - 1, r1 + 1, sail_edge);

        // Sail row 2 (mast_top+2): wider diagonal + fill
        let r2 = mt + 4;
        self.paint_px(painter, mx - 2, r2, sail_edge);
        self.paint_px(painter, mx - 2, r2 + 1, sail_edge);
        self.paint_px(painter, mx - 1, r2, sail_fill);
        self.paint_px(painter, mx - 1, r2 + 1, sail_fill);

        // Rigging — aft diagonal (starboard side)
        self.paint_px(painter, mx + 1, r1, rigging);
        self.paint_px(painter, mx + 1, r1 + 1, rigging);
        self.paint_px(painter, mx + 2, r2, rigging);
        self.paint_px(painter, mx + 2, r2 + 1, rigging);
        // Extended aft rigging
        let r3 = mt + 6;
        self.paint_px(painter, mx + 3, r3, rigging);
        self.paint_px(painter, mx + 3, r3 + 1, rigging);

        // Forestay — front diagonal rope from mast top toward bow
        self.paint_px(painter, mx - 2, r1, rigging);
        self.paint_px(painter, mx - 2, r1 + 1, rigging);
        self.paint_px(painter, mx - 3, r2, rigging);
        self.paint_px(painter, mx - 3, r2 + 1, rigging);
    }
}

/// Canvas HalfBlock water surface with per-pixel depth gradient and wave animation.
struct WaterSurface {
    width: usize,
    water_rows: usize,
    horizon: usize,
    wave_tick: f64,
    dusk: f64,
    boat_x: f64,
    sky_pixels: usize, // gy offset = horizon * 2
}

impl Shape for WaterSurface {
    fn draw(&self, painter: &mut Painter) {
        let total_py = self.water_rows * 2;
        if total_py == 0 || self.width == 0 {
            return;
        }

        let near = lerp_rgb((48, 136, 192), (30, 70, 112), self.dusk);
        let deep = lerp_rgb((10, 60, 112), (5, 22, 52), self.dusk);
        let sky_low = lerp_rgb((210, 232, 252), (116, 96, 132), self.dusk);

        for gy in 0..total_py {
            let depth_t = if total_py <= 1 {
                0.0
            } else {
                gy as f64 / (total_py - 1) as f64
            };

            let mut base = lerp_rgb(near, deep, depth_t.powf(0.85));

            // Soften horizon seam: blend top 3 pixels toward sky horizon color
            if gy < 3 {
                let blend_t = (gy as f64 + 1.0) / 4.0;
                base = lerp_rgb(sky_low, base, blend_t);
            }

            let crest = lerp_rgb((154, 226, 248), (98, 184, 226), depth_t);
            let row_equiv = self.horizon as f64 + gy as f64 / 2.0;

            for gx in 0..self.width {
                let primary = (gx as f64 * (0.24 + depth_t * 0.06)
                    + self.wave_tick * (0.055 + depth_t * 0.045))
                    .sin();
                let secondary =
                    (gx as f64 * 0.11 - self.wave_tick * 0.043 + row_equiv * 0.92).cos();
                let wave = primary * 0.72 + secondary * 0.28;

                let mut color = if wave > 0.74 {
                    // Foam/whitecaps on wave crests
                    let crest_intensity = ((wave - 0.06) / 1.34).min(1.0);
                    let blended = lerp_rgb(base, crest, crest_intensity * 0.55);
                    let foam_t = ((wave - 0.74) / 0.26).min(1.0);
                    lerp_rgb(blended, (240, 248, 255), foam_t * 0.7)
                } else if wave > 0.06 {
                    let intensity = ((wave - 0.06) / 1.34).min(1.0);
                    let blended = lerp_rgb(base, crest, intensity * 0.55);
                    let shimmer = (((gx as f64 * 0.09 + self.wave_tick * 0.19).sin() + 1.0)
                        * 0.5
                        * 18.0
                        * intensity) as u8;
                    (
                        blended.0.saturating_add(shimmer),
                        blended.1.saturating_add(shimmer / 2),
                        blended.2.saturating_add(shimmer / 3),
                    )
                } else {
                    base
                };

                // Caustic light patterns — dancing bright spots near the surface
                let caustic_x = (gx as f64 * 0.31 + self.wave_tick * 0.17).sin();
                let caustic_y = (gy as f64 * 0.43 + self.wave_tick * 0.13).cos();
                let caustic = (caustic_x * caustic_y + 1.0) * 0.5;
                if caustic > 0.7 {
                    let boost = ((caustic - 0.7) / 0.3 * (1.0 - depth_t) * 0.15 * 255.0) as u8;
                    color = (
                        color.0.saturating_add(boost / 3),
                        color.1.saturating_add(boost),
                        color.2.saturating_add(boost / 2),
                    );
                }

                // Horizon foam line — animated bright strip at water's edge
                if gy < 3 {
                    let foam_wave = (gx as f64 * 0.18 + self.wave_tick * 0.09).sin();
                    if foam_wave > 0.2 {
                        let foam_t =
                            ((foam_wave - 0.2) / 0.8).min(1.0) * (1.0 - gy as f64 / 3.0) * 0.5;
                        color = lerp_rgb(color, (230, 245, 255), foam_t);
                    }
                }

                // Celestial body reflection — vertical shimmering streak
                let orb_col =
                    (self.width as f64 * 0.78 + (self.wave_tick * 0.08).sin() * 3.0).round();
                let reflect_wobble = (gy as f64 * 0.3 + self.wave_tick * 0.15).sin() * 1.5;
                let dist_from_orb = (gx as f64 - orb_col - reflect_wobble).abs();
                if dist_from_orb < 3.0 {
                    let reflect_t = 1.0 - dist_from_orb / 3.0;
                    let reflect_wave = (gy as f64 * 0.25 + self.wave_tick * 0.12).sin() * 0.5 + 0.5;
                    let reflect_strength = reflect_t * reflect_wave * (1.0 - depth_t * 0.7) * 0.35;
                    let orb_color = if self.dusk < 0.56 {
                        (255, 228, 152)
                    } else {
                        (200, 210, 240)
                    };
                    color = lerp_rgb(color, orb_color, reflect_strength);
                }

                // Lantern glow on water — warm tint near boat at night
                if self.dusk > 0.4 {
                    let dx = gx as f64 - self.boat_x;
                    let dy = gy as f64 * 0.5;
                    let lantern_dist = (dx * dx + dy * dy).sqrt();
                    if lantern_dist < 8.0 {
                        let glow_t =
                            (1.0 - lantern_dist / 8.0) * ((self.dusk - 0.4) / 0.6).min(1.0) * 0.2;
                        color = lerp_rgb(color, (255, 200, 80), glow_t);
                    }
                }

                // Wake pattern — V-shaped lighter streak behind boat hull
                let wake_x = gx as f64 - self.boat_x;
                let wake_depth = gy as f64;
                if wake_depth > 4.0 && wake_depth < 24.0 {
                    let arm_width = wake_depth * 0.3;
                    let dist_to_arm = (wake_x.abs() - arm_width).abs();
                    if dist_to_arm < 1.5 {
                        let fade = (1.0 - (wake_depth - 4.0) / 20.0) * 0.25;
                        let sharpness = 1.0 - dist_to_arm / 1.5;
                        color = lerp_rgb(color, (200, 240, 255), fade * sharpness);
                    }
                }

                painter.paint(
                    gx,
                    self.sky_pixels + gy,
                    Color::Rgb(color.0, color.1, color.2),
                );
            }
        }
    }
}

/// Canvas HalfBlock sailboat hull, deck, keel, and sail bottom row.
/// Drawn after WaterSurface so boat pixels overwrite water.
struct BoatShape {
    mast_x: usize,
    top_py: usize,
    dusk: f64,
    width: usize,
    total_py: usize,
}

impl Shape for BoatShape {
    fn draw(&self, painter: &mut Painter) {
        let mx = self.mast_x as i32;

        // Color palette
        let hull_dark: (u8, u8, u8) = (94, 58, 36);
        let hull_mid: (u8, u8, u8) = (140, 90, 52);
        let hull_light: (u8, u8, u8) = (188, 140, 86);
        let deck: (u8, u8, u8) = (188, 148, 96);
        let mast: (u8, u8, u8) = (210, 168, 104);
        let sail_edge: (u8, u8, u8) = (245, 235, 215);
        let sail_fill: (u8, u8, u8) = (238, 228, 208);
        let sail_edge_lo: (u8, u8, u8) = (235, 225, 205);
        let sail_fill_lo: (u8, u8, u8) = (232, 222, 200);
        let mast_lo: (u8, u8, u8) = (200, 158, 94);

        // Lantern vs mast at sail position
        let (mast_top, mast_bot) = if self.dusk > 0.4 {
            let g = ((self.dusk - 0.4) / 0.6).min(1.0);
            (
                (
                    (180.0 + g * 75.0) as u8,
                    (140.0 + g * 60.0) as u8,
                    (40.0 + g * 40.0) as u8,
                ),
                (
                    (160.0 + g * 55.0) as u8,
                    (120.0 + g * 40.0) as u8,
                    (30.0 + g * 30.0) as u8,
                ),
            )
        } else {
            (mast, mast_lo)
        };

        // Pixel table: (dx from mast_x, dpy from top_py, color)
        let pixels: &[(i32, usize, (u8, u8, u8))] = &[
            // py0: sail bottom upper half
            (-2, 0, sail_edge),
            (-1, 0, sail_fill),
            (0, 0, mast_top),
            // py1: sail bottom lower half
            (-2, 1, sail_edge_lo),
            (-1, 1, sail_fill_lo),
            (0, 1, mast_bot),
            // py2: deck upper half
            (-3, 2, hull_light),
            (-2, 2, deck),
            (-1, 2, deck),
            (0, 2, mast),
            (1, 2, deck),
            (2, 2, deck),
            (3, 2, deck),
            // py3: deck lower half (full width with taper)
            (-6, 3, deck),
            (-5, 3, hull_mid),
            (-4, 3, hull_mid),
            (-3, 3, hull_dark),
            (-2, 3, hull_dark),
            (-1, 3, hull_dark),
            (0, 3, hull_dark),
            (1, 3, hull_dark),
            (2, 3, hull_dark),
            (3, 3, hull_dark),
            (4, 3, hull_mid),
            (5, 3, hull_mid),
            (6, 3, deck),
            // py4: hull upper
            (-4, 4, hull_mid),
            (-3, 4, hull_dark),
            (-2, 4, hull_dark),
            (-1, 4, hull_dark),
            (0, 4, hull_dark),
            (1, 4, hull_dark),
            (2, 4, hull_dark),
            (3, 4, hull_dark),
            (4, 4, hull_mid),
            // py5: hull lower
            (-3, 5, hull_dark),
            (-2, 5, hull_dark),
            (-1, 5, hull_dark),
            (0, 5, hull_dark),
            (1, 5, hull_dark),
            (2, 5, hull_dark),
            (3, 5, hull_dark),
            // py6: keel upper
            (-2, 6, hull_dark),
            (-1, 6, hull_dark),
            (0, 6, hull_dark),
            (1, 6, hull_dark),
            (2, 6, hull_dark),
            // py7: keel lower
            (-1, 7, hull_dark),
            (0, 7, hull_dark),
            (1, 7, hull_dark),
        ];

        for &(dx, dpy, color) in pixels {
            let gx = mx + dx;
            let gy = self.top_py + dpy;
            if gx >= 0 && (gx as usize) < self.width && gy < self.total_py {
                painter.paint(gx as usize, gy, Color::Rgb(color.0, color.1, color.2));
            }
        }

        // Waterline highlight — bright refraction pixels at hull edges
        let waterline_color = Color::Rgb(160, 210, 240);
        let waterline_py = self.top_py + 4;
        if waterline_py < self.total_py {
            for dx in [-7, -6, 7, 8] {
                let gx = mx + dx;
                if gx >= 0 && (gx as usize) < self.width {
                    painter.paint(gx as usize, waterline_py, waterline_color);
                }
            }
        }

        // Keel reflection glow — soft light beneath the hull
        let keel_glow = Color::Rgb(80, 140, 180);
        let keel_py = self.top_py + 8;
        if keel_py < self.total_py {
            for dx in -2..=2 {
                let gx = mx + dx;
                if gx >= 0 && (gx as usize) < self.width {
                    painter.paint(gx as usize, keel_py, keel_glow);
                }
            }
        }
    }
}

/// Canvas HalfBlock fishing line, fish shadows, and ripple rings.
/// Drawn after water but before boat so the line emerges from under the hull.
struct FishingDetailsShape {
    line_x0: i32,
    line_y0: i32,
    line_x1: i32,
    line_y1: i32,
    bobber_gx: i32,
    bobber_gy: i32,
    ripple_radius: i32,
    wave_tick: f64,
    width: usize,
    water_rows: usize,
    sky_pixels: usize,
    shadow_bobber_x: i32,
    shadow_bobber_wy: i32,
    is_waiting: bool,
    total_py: usize,
}

impl FishingDetailsShape {
    fn paint_pixel(&self, painter: &mut Painter, gx: i32, gy: i32, color: Color) {
        if gx >= 0 && (gx as usize) < self.width && gy >= 0 && (gy as usize) < self.total_py {
            painter.paint(gx as usize, gy as usize, color);
        }
    }

    fn paint_bresenham(
        &self,
        painter: &mut Painter,
        mut x0: i32,
        mut y0: i32,
        x1: i32,
        y1: i32,
        color: Color,
    ) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.paint_pixel(painter, x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_shadows(&self, painter: &mut Painter) {
        // Oval offsets: 5 wide × 3 tall fish shape
        let core: &[(i32, i32)] = &[(-1, 0), (0, -1), (0, 0), (0, 1), (1, 0)];
        let wings: &[(i32, i32)] = &[(-2, 0), (2, 0)];

        for i in 0..3 {
            let seed = i as f64 * 137.5;
            let speed = 0.018 * (i as f64 * 0.4 + 1.0);

            let mut fx = ((seed + self.wave_tick * speed).sin() * self.width as f64 * 0.35
                + self.width as f64 * 0.5)
                .round() as i32;
            let mut fy = ((seed * 2.3 + self.wave_tick * speed * 0.8).sin()
                * (self.water_rows as f64 * 0.3)
                + self.water_rows as f64 * 0.4
                + i as f64 * 2.0)
                .round() as i32;

            // Shadow 0 veers toward bobber during Waiting
            if self.is_waiting && i == 0 {
                let approach = ((self.wave_tick * 0.03).sin() * 0.5 + 0.5).min(1.0) * 0.6;
                fx += ((self.shadow_bobber_x - fx) as f64 * approach).round() as i32;
                fy += ((self.shadow_bobber_wy - fy) as f64 * approach * 0.5).round() as i32;
            }

            // Depth-based fade: deeper shadows blend toward deep water color
            let depth_t = fy as f64 / self.water_rows.max(1) as f64;
            let fade = (depth_t * 0.6).clamp(0.0, 1.0);
            let dark = lerp_rgb((18, 38, 68), (10, 50, 100), fade);
            let light = lerp_rgb((24, 48, 80), (14, 54, 104), fade);
            let shadow_dark = Color::Rgb(dark.0, dark.1, dark.2);
            let shadow_light = Color::Rgb(light.0, light.1, light.2);

            // Convert to scene-global pixel coords
            let gy_center = self.sky_pixels as i32 + fy * 2;
            for &(dx, dy) in core {
                self.paint_pixel(painter, fx + dx, gy_center + dy, shadow_dark);
            }
            for &(dx, dy) in wings {
                self.paint_pixel(painter, fx + dx, gy_center + dy, shadow_light);
            }
        }
    }

    fn draw_bobber_trail(&self, painter: &mut Painter) {
        // Only during reeling (ripple_radius == 2 signals reeling phase)
        if self.ripple_radius < 2 {
            return;
        }

        let trail: &[(i32, i32, (u8, u8, u8))] = &[
            (3, 0, (160, 220, 245)),
            (5, 1, (130, 200, 235)),
            (8, -1, (110, 185, 225)),
            (11, 0, (90, 170, 215)),
            (14, 1, (70, 155, 205)),
        ];
        for &(dx, dy, c) in trail {
            let wobble = ((self.wave_tick * 0.3 + dx as f64 * 0.5).sin() * 1.0).round() as i32;
            let gx = self.bobber_gx + dx;
            let gy = self.bobber_gy + dy + wobble;
            self.paint_pixel(painter, gx, gy, Color::Rgb(c.0, c.1, c.2));
        }
    }

    fn draw_ripples(&self, painter: &mut Painter) {
        use std::f64::consts::PI;

        for ring in 1..=self.ripple_radius {
            let pr = ring * 2; // pixel radius
            let num_points = (pr * 6).max(12) as usize;
            let color = if ring == 1 {
                Color::Rgb(196, 236, 255)
            } else {
                Color::Rgb(146, 206, 238)
            };
            for i in 0..num_points {
                let theta = i as f64 * 2.0 * PI / num_points as f64;
                let rx = (self.bobber_gx as f64 + pr as f64 * theta.cos()).round() as i32;
                let ry = (self.bobber_gy as f64 + pr as f64 * theta.sin()).round() as i32;
                self.paint_pixel(painter, rx, ry, color);
            }
        }
    }
}

impl Shape for FishingDetailsShape {
    fn draw(&self, painter: &mut Painter) {
        // 1. Fish shadows (underneath everything)
        self.draw_shadows(painter);

        // 2. Fishing line: shadow then bright
        self.paint_bresenham(
            painter,
            self.line_x0 + 1,
            self.line_y0,
            self.line_x1 + 1,
            self.line_y1,
            Color::Rgb(38, 52, 78),
        );
        self.paint_bresenham(
            painter,
            self.line_x0,
            self.line_y0,
            self.line_x1,
            self.line_y1,
            Color::Rgb(255, 208, 122),
        );

        // 3. Bobber wake trail (reeling only)
        self.draw_bobber_trail(painter);

        // 4. Ripple rings around bobber
        self.draw_ripples(painter);
    }
}

/// Write a character with fg color onto the frame buffer, preserving the existing bg.
fn overlay_fg(frame: &mut Frame, area: Rect, row: i32, col: i32, ch: char, fg: Color) {
    if row < 0 || col < 0 {
        return;
    }
    let (r, c) = (row as u16, col as u16);
    if r >= area.height || c >= area.width {
        return;
    }
    let cell = &mut frame.buffer_mut()[(area.x + c, area.y + r)];
    cell.set_char(ch);
    cell.set_fg(fg);
}

/// Shared bobber parameters by fishing phase.
fn bobber_params(session: &FishingSession) -> (f64, f64, f64, char, Color, i32) {
    use crate::fishing::types::FishingPhase;
    match session.phase {
        FishingPhase::Casting => (0.66, 0.45, 0.0, '○', Color::Rgb(240, 248, 255), 1),
        FishingPhase::Waiting => (0.58, 0.60, 0.2, '◉', Color::Rgb(255, 236, 214), 1),
        FishingPhase::Reeling => (0.50, 1.10, 0.9, '●', Color::Rgb(255, 96, 82), 2),
    }
}

/// Draws the full fishing scene as a single Canvas HalfBlock surface.
fn draw_water_scene(frame: &mut Frame, area: Rect, session: &FishingSession) {
    let water_block = Block::default().borders(Borders::LEFT | Borders::RIGHT);
    let inner = water_block.inner(area);
    frame.render_widget(water_block, area);

    if inner.width < 12 || inner.height < 5 {
        return;
    }

    let width = inner.width as usize;
    let height = inner.height as usize;
    let millis = current_millis() as f64;
    let wave_tick = millis / 110.0;
    let dusk = ((millis / 16_000.0).sin() * 0.5 + 0.5).powf(1.2);
    let horizon = ((height as f64 * 0.34).round() as usize).clamp(1, height.saturating_sub(2));
    let water_rows = height - horizon;
    let total_py = height * 2;
    let sky_pixels = horizon * 2;

    // Shared boat geometry
    let boat_center = ((width as f64 * 0.35) + (wave_tick * 0.04).sin() * 2.0).round() as i32;
    let deck_row = (horizon + 1).min(height.saturating_sub(4)) as i32;
    let mast_x = boat_center;
    let mast_top = deck_row - 3;

    // Bobber position (shared by Canvas shapes and character overlays)
    let (bobber_ratio, bobber_amp, bobber_sink, _, _, ripple_radius) = bobber_params(session);
    let bobber_x = ((width as f64 * bobber_ratio) + (wave_tick * 0.065).sin() * 2.5).round() as i32;
    let bobber_base = horizon + ((height - horizon).max(2) / 5);
    let bobber_y = (bobber_base as f64 + (wave_tick * 0.085).sin() * bobber_amp + bobber_sink)
        .round()
        .clamp((horizon + 1) as f64, (height.saturating_sub(2)) as f64) as i32;
    let is_waiting = matches!(session.phase, crate::fishing::types::FishingPhase::Waiting);

    // === Canvas shapes (all in scene-global pixel coords) ===
    let sky = SkyShape {
        width,
        horizon,
        wave_tick,
        dusk,
        mast_x,
        mast_top,
    };
    let water = WaterSurface {
        width,
        water_rows,
        horizon,
        wave_tick,
        dusk,
        boat_x: mast_x as f64,
        sky_pixels,
    };
    let fishing = FishingDetailsShape {
        line_x0: mast_x,
        line_y0: (mast_top * 2).max(0),
        line_x1: bobber_x,
        line_y1: bobber_y * 2,
        bobber_gx: bobber_x,
        bobber_gy: bobber_y * 2,
        ripple_radius,
        wave_tick,
        width,
        water_rows,
        sky_pixels,
        shadow_bobber_x: bobber_x,
        shadow_bobber_wy: bobber_y - horizon as i32,
        is_waiting,
        total_py,
    };
    let boat = BoatShape {
        mast_x: mast_x.max(0) as usize,
        top_py: sky_pixels,
        dusk,
        width,
        total_py,
    };

    let canvas = Canvas::default()
        .x_bounds([0.0, width as f64])
        .y_bounds([0.0, total_py as f64])
        .marker(Marker::HalfBlock)
        .background_color(Color::Rgb(10, 30, 60))
        .paint(|ctx| {
            ctx.draw(&sky);
            ctx.draw(&water);
            ctx.draw(&fishing);
            ctx.draw(&boat);
        });
    frame.render_widget(canvas, inner);

    // === CHARACTER OVERLAYS (bobber symbol + splash effects) ===
    overlay_bobber_effects(frame, inner, session, width, height, horizon, wave_tick);
}

/// Bobber character and splash/bubble effects overlaid on the full-scene Canvas.
#[allow(clippy::too_many_arguments)]
fn overlay_bobber_effects(
    frame: &mut Frame,
    area: Rect,
    session: &FishingSession,
    width: usize,
    height: usize,
    horizon: usize,
    wave_tick: f64,
) {
    use crate::fishing::types::FishingPhase;

    let (bobber_ratio, bobber_amp, bobber_sink, bobber_char, bobber_color, _) =
        bobber_params(session);

    let bobber_x = ((width as f64 * bobber_ratio) + (wave_tick * 0.065).sin() * 2.5).round() as i32;
    let bobber_base = horizon + ((height - horizon).max(2) / 5);
    let bobber_y = (bobber_base as f64 + (wave_tick * 0.085).sin() * bobber_amp + bobber_sink)
        .round()
        .clamp((horizon + 1) as f64, (height.saturating_sub(2)) as f64) as i32;

    // Bobber character (area covers full scene, so scene row = overlay row)
    overlay_fg(frame, area, bobber_y, bobber_x, bobber_char, bobber_color);

    // Splash effect and bubble trail during reeling
    if session.phase == FishingPhase::Reeling {
        // Bubble trail below bobber
        let trail_chars = ['°', '·', '∘'];
        for i in 1..=3i32 {
            let wobble = if (wave_tick as i64 + i as i64) % 2 == 0 {
                1
            } else {
                -1
            };
            let fade = (1.0 - i as f64 / 4.0).max(0.0);
            let c = (
                (80.0 + fade * 120.0) as u8,
                (160.0 + fade * 80.0) as u8,
                255u8,
            );
            overlay_fg(
                frame,
                area,
                bobber_y + i,
                bobber_x + wobble,
                trail_chars[(i - 1) as usize],
                Color::Rgb(c.0, c.1, c.2),
            );
        }

        // Main splash
        let splash_row = bobber_y - 1;
        let splash_x = bobber_x + if ((wave_tick as i64) & 1) == 0 { 4 } else { -4 };
        for (i, ch) in "<><".chars().enumerate() {
            overlay_fg(
                frame,
                area,
                splash_row,
                splash_x + i as i32,
                ch,
                Color::Rgb(255, 224, 118),
            );
        }
        for &(dx, dy, ch) in &[(-1, -1, '\''), (2, -1, '\''), (0, -2, '`')] {
            overlay_fg(
                frame,
                area,
                splash_row + dy,
                splash_x + dx,
                ch,
                Color::Rgb(216, 242, 255),
            );
        }

        // Enhanced spray: wider arc of droplets around the bobber
        let spray_phase = (wave_tick * 0.5) as i64;
        let spray_offsets: &[(i32, i32, char)] = &[
            (-3, -1, '\''),
            (3, -1, '`'),
            (-2, -2, '·'),
            (2, -2, '·'),
            (-4, 0, '·'),
            (4, 0, '·'),
        ];
        for &(dx, dy, ch) in spray_offsets {
            // Alternate visibility per tick for animation
            let visible = (spray_phase + dx.abs() as i64) % 3 != 0;
            if visible {
                overlay_fg(
                    frame,
                    area,
                    bobber_y + dy,
                    bobber_x + dx,
                    ch,
                    Color::Rgb(196, 236, 255),
                );
            }
        }
    }
}

/// Data for each Storm Leviathan encounter stage.
struct LeviathanEncounterData {
    title: &'static str,
    flavor: &'static str,
    status: &'static str,
    health_bar: &'static str,
    lure_line: &'static str,
}

/// The 10 progressive encounters with the Storm Leviathan.
const LEVIATHAN_ENCOUNTERS: [LeviathanEncounterData; 10] = [
    LeviathanEncounterData {
        title: "RIPPLES",
        flavor: "Something disturbed the deep. A shadow vast as a ship passes beneath you. Before you can react, it vanishes into the abyss.",
        status: "UNTOUCHED",
        health_bar: "████████████████████",
        lure_line: "\u{26A1} Lure pulled into the depths",
    },
    LeviathanEncounterData {
        title: "THE SHADOW",
        flavor: "The Leviathan surfaces for a heartbeat - scales like storm clouds, eyes like lightning. It knows you now. Then it's gone.",
        status: "AWARE",
        health_bar: "██████████████████░░",
        lure_line: "\u{26A1} Lure swallowed by the shadow",
    },
    LeviathanEncounterData {
        title: "EMERGENCE",
        flavor: "It breaches! The beast roars - a sound like thunder over the waves. Your boat rocks violently as it dives deep.",
        status: "AGITATED",
        health_bar: "████████████████░░░░",
        lure_line: "\u{26A1} Lure torn away in the breach",
    },
    LeviathanEncounterData {
        title: "KNOWN",
        flavor: "It circles your position. Watching. Waiting. This is no mere fish - it's deciding if YOU are worthy prey.",
        status: "HUNTING",
        health_bar: "██████████████░░░░░░",
        lure_line: "\u{26A1} Lure crushed in its jaws",
    },
    LeviathanEncounterData {
        title: "FIRST STRIKE",
        flavor: "Your hook finds flesh! The beast screams - a sound that will haunt your dreams. It dives, trailing darkness and blood.",
        status: "WOUNDED",
        health_bar: "████████████░░░░░░░░",
        lure_line: "\u{26A1} Lure shattered on impact",
    },
    LeviathanEncounterData {
        title: "FURY",
        flavor: "It rams your boat in rage! You barely hold on as waves crash over the deck. But in its fury, it expends precious strength.",
        status: "RAGING",
        health_bar: "██████████░░░░░░░░░░",
        lure_line: "\u{26A1} Lure lost in the chaos",
    },
    LeviathanEncounterData {
        title: "BLOOD IN WATER",
        flavor: "Wounded and bleeding, it circles. You are both predator and prey now. Neither will yield. Neither can escape.",
        status: "BLEEDING",
        health_bar: "████████░░░░░░░░░░░░",
        lure_line: "\u{26A1} Lure dissolved in the churn",
    },
    LeviathanEncounterData {
        title: "THE LONG NIGHT",
        flavor: "Hours pass. The beast surfaces less often, its movements slower. Stars wheel overhead. Dawn approaches. You will not sleep until this ends.",
        status: "TIRING",
        health_bar: "██████░░░░░░░░░░░░░░",
        lure_line: "\u{26A1} Lure faded with the light",
    },
    LeviathanEncounterData {
        title: "EXHAUSTION",
        flavor: "It can barely surface now. Each breath is labored, each dive shorter. Victory is close. You can taste it.",
        status: "EXHAUSTED",
        health_bar: "████░░░░░░░░░░░░░░░░",
        lure_line: "\u{26A1} Lure snapped from the line",
    },
    LeviathanEncounterData {
        title: "LEGEND",
        flavor: "With a final, defiant bellow, it succumbs. Your line holds. Your arms burn. But you've done the impossible.",
        status: "DEFEATED",
        health_bar: "██░░░░░░░░░░░░░░░░░░",
        lure_line: "\u{26A1} Lure spent \u{2014} the beast falls",
    },
];

/// Renders the Storm Leviathan encounter modal.
///
/// This modal appears when the player encounters the Leviathan during fishing.
/// The encounter number (1-10) determines which stage of the hunt is shown.
pub fn render_leviathan_encounter_modal(
    frame: &mut Frame,
    area: Rect,
    encounter_number: u8,
    lure_consumed: bool,
    _ctx: &super::responsive::LayoutContext,
) {
    if encounter_number == 0 || encounter_number > 10 {
        return;
    }

    let data = &LEVIATHAN_ENCOUNTERS[(encounter_number - 1) as usize];

    let modal_width = 64;
    let modal_height = 16;

    // Center the modal
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    frame.render_widget(Clear, modal_area);

    let title = format!(" ⛈️  {}  ⛈️ ", data.title);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Cyan)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Cyan,
        super::BorderFxContext,
    );

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🐋",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Flavor text (wrapped)
    lines.push(Line::from(Span::styled(
        data.flavor,
        Style::default().fg(Color::White),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Health bar
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(data.health_bar, Style::default().fg(Color::Red)),
        Span::raw("  "),
        Span::styled(
            data.status,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}/10", encounter_number),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    if lure_consumed {
        lines.push(Line::from(Span::styled(
            data.lure_line,
            Style::default()
                .fg(Color::Rgb(80, 160, 220))
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines.push(Line::from(""));

    // Hint text based on encounter
    let hint = if encounter_number < 10 {
        "\"The beast learns. Each escape makes it warier...\""
    } else {
        "\"This is your moment. The hunt ends now.\""
    };
    lines.push(Line::from(Span::styled(
        hint,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] to continue",
        Style::default().fg(Color::DarkGray),
    )));

    let para = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(para, inner);
}

/// Renders the Storm Leviathan catch-miss modal.
///
/// Shown when the Leviathan appears during the catch phase but isn't caught.
pub fn render_leviathan_catch_miss_modal(
    frame: &mut Frame,
    area: Rect,
    lure_consumed: bool,
    _ctx: &super::responsive::LayoutContext,
) {
    let modal_width = 58u16;
    let modal_height = 12u16;

    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{26C8}\u{FE0F}  THE BEAST ELUDES YOU  \u{26C8}\u{FE0F} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Cyan)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Cyan,
        super::BorderFxContext,
    );

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "\u{1F40B}",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "The Leviathan surfaces, thrashing against your line...",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "but breaks free at the last moment!",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    if lure_consumed {
        lines.push(Line::from(Span::styled(
            "\u{26A1} Lure dragged into the abyss",
            Style::default()
                .fg(Color::Rgb(80, 160, 220))
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] to continue",
        Style::default().fg(Color::DarkGray),
    )));

    let para = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(para, inner);
}
