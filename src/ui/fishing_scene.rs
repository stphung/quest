//! Fishing scene UI rendering.
//!
//! Displays the active fishing session with animated water, catch progress,
//! caught fish list, and fishing rank progression.

#![allow(dead_code)]

use super::scene_fx::{
    current_millis, draw_line, hash2d, lerp_channel, lerp_rgb, put_cell, render_buffer, SceneCell,
};
use crate::fishing::types::{FishingSession, FishingState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Renders the fishing scene UI.
///
/// # Layout
/// ```text
/// +---------------------------------------+
/// |  FISHING - [Spot Name]                |
/// +---------------------------------------+
/// |     ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~         |
/// |       ~~~~~~ O ~~~~~~                 |
/// |     ~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~           |
/// |              |                        |
/// +---------------------------------------+
/// |  Caught: X/Y fish                     |
/// +---------------------------------------+
/// |  [Uncommon] Trout - 180 XP            |
/// |  [Common] Carp - 65 XP                |
/// |  [Rare] Salmon - 520 XP  [Item]       |
/// +---------------------------------------+
/// |  Rank: [Rank Name] (N)                |
/// |  Progress: ████████░░ X/Y             |
/// +---------------------------------------+
/// ```
pub fn render_fishing_scene(
    frame: &mut Frame,
    area: Rect,
    session: &FishingSession,
    _fishing_state: &FishingState,
    _ctx: &super::responsive::LayoutContext,
) {
    // Main vertical layout (recent catches now shown in the Loot panel)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with spot name
            Constraint::Min(6),    // Water animation area
            Constraint::Length(4), // Catch progress + phase status
        ])
        .split(area);

    // Draw header with spot name
    draw_header(frame, chunks[0], session);

    // Draw water animation with bobber
    draw_water_scene(frame, chunks[1], session);

    // Draw catch progress
    draw_catch_progress(frame, chunks[2], session);
}

/// Draws the header with fishing spot name.
fn draw_header(frame: &mut Frame, area: Rect, session: &FishingSession) {
    let title = format!(" FISHING - {} ", session.spot_name);

    let header_text = vec![Line::from(vec![Span::styled(
        format!("Fishing at {}", session.spot_name),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .alignment(Alignment::Center);

    frame.render_widget(header, area);
}

/// Draws the ASCII water scene with bobber.
fn draw_water_scene(frame: &mut Frame, area: Rect, session: &FishingSession) {
    let water_block = Block::default().borders(Borders::LEFT | Borders::RIGHT);
    let inner = water_block.inner(area);
    frame.render_widget(water_block, area);

    if inner.width < 12 || inner.height < 5 {
        return;
    }

    let width = inner.width as usize;
    let height = inner.height as usize;
    let mut buffer = vec![vec![SceneCell::default(); width]; height];

    let millis = current_millis() as f64;
    let wave_tick = millis / 110.0;
    let dusk = ((millis / 16_000.0).sin() * 0.5 + 0.5).powf(1.2);
    let horizon = ((height as f64 * 0.34).round() as usize).clamp(1, height.saturating_sub(2));

    render_sky_and_shoreline(&mut buffer, width, height, horizon, wave_tick, dusk);
    render_water_surface(&mut buffer, height, horizon, wave_tick);
    render_sailboat(&mut buffer, width, horizon, height, wave_tick);
    render_bobber_and_line(&mut buffer, session, width, height, horizon, wave_tick);

    render_buffer(frame, inner, &buffer);
}

/// Sky gradient, celestial body, stars, clouds, and distant shoreline silhouettes.
fn render_sky_and_shoreline(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    horizon: usize,
    wave_tick: f64,
    dusk: f64,
) {
    // Sky and water background gradients
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let bg_rgb = if row < horizon {
            let row_t = if horizon <= 1 {
                0.0
            } else {
                row as f64 / (horizon - 1) as f64
            };
            let top = lerp_rgb((118, 196, 248), (20, 34, 66), dusk);
            let low = lerp_rgb((210, 232, 252), (116, 96, 132), dusk);
            lerp_rgb(top, low, row_t)
        } else {
            let row_t = if height - horizon <= 1 {
                0.0
            } else {
                (row - horizon) as f64 / (height - horizon - 1) as f64
            };
            let near = lerp_rgb((48, 136, 192), (30, 70, 112), dusk);
            let deep = lerp_rgb((10, 60, 112), (5, 22, 52), dusk);
            lerp_rgb(near, deep, row_t.powf(0.85))
        };

        let bg = Color::Rgb(bg_rgb.0, bg_rgb.1, bg_rgb.2);
        for cell in row_cells {
            cell.bg = bg;
        }
    }

    // Celestial body
    let orb_col = ((width as f64 * 0.78) + (wave_tick * 0.08).sin() * 3.0).round() as i32;
    let orb_row = ((horizon as f64 * 0.32) + (wave_tick * 0.04).sin()).round() as i32;
    let (orb_char, orb_color) = if dusk < 0.56 {
        ('●', Color::Rgb(255, 228, 152))
    } else {
        ('◑', Color::Rgb(232, 238, 255))
    };
    for (dx, dy, ch, fg) in [
        (0, 0, orb_char, orb_color),
        (-1, 0, '·', Color::Rgb(240, 226, 176)),
        (1, 0, '·', Color::Rgb(240, 226, 176)),
        (0, -1, '·', Color::Rgb(236, 220, 170)),
        (0, 1, '·', Color::Rgb(236, 220, 170)),
    ] {
        put_cell(&mut buffer[..], orb_row + dy, orb_col + dx, ch, fg);
    }

    // Stars appear as dusk settles
    if dusk > 0.25 {
        let twinkle_tick = (wave_tick / 1.8) as usize;
        for (row, row_cells) in buffer
            .iter_mut()
            .enumerate()
            .take(horizon.saturating_sub(1))
        {
            for (col, cell) in row_cells.iter_mut().enumerate() {
                if hash2d(row, col).is_multiple_of(97) && cell.ch == ' ' {
                    let bright = hash2d(row + twinkle_tick, col + twinkle_tick).is_multiple_of(3);
                    *cell = SceneCell::new(
                        if bright { '*' } else { '.' },
                        if bright {
                            Color::Rgb(244, 246, 255)
                        } else {
                            Color::Rgb(184, 190, 232)
                        },
                        cell.bg,
                    );
                }
            }
        }
    }

    // Sky clouds
    for &(base_x, y_ratio, speed, pattern, shade) in &[
        (6.0, 0.18, 0.040, "~~", 148u8),
        (18.0, 0.28, 0.032, "~~~", 140),
        (34.0, 0.16, 0.052, "~~~~", 132),
        (12.0, 0.44, 0.026, "~ ~", 126),
        (44.0, 0.36, 0.036, "~~", 130),
    ] {
        let drift = (wave_tick * speed) % width as f64;
        let cx = ((base_x - drift).rem_euclid(width as f64)) as usize;
        let row = ((horizon as f64 * y_ratio).round() as usize).min(horizon.saturating_sub(1));
        for (i, ch) in pattern.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let col = (cx + i) % width;
            if buffer[row][col].ch == ' ' {
                let tint = lerp_channel(shade, 208, 1.0 - dusk);
                buffer[row][col] = SceneCell::new(
                    ch,
                    Color::Rgb(tint, tint, tint.saturating_add(8)),
                    buffer[row][col].bg,
                );
            }
        }
    }

    // Distant shoreline silhouettes
    for col in 0..width {
        let far_h =
            (1.0 + ((col as f64 * 0.24 + wave_tick * 0.012).sin() + 1.0) * 1.1).round() as i32;
        let near_h = (1.0 + ((col as f64 * 0.15 + wave_tick * 0.020 + 1.5).sin() + 1.0) * 1.5)
            .round() as i32;

        for (height_delta, ch, color) in [
            (
                far_h,
                '░',
                Color::Rgb(
                    lerp_channel(84, 62, dusk),
                    lerp_channel(110, 90, dusk),
                    lerp_channel(126, 114, dusk),
                ),
            ),
            (
                near_h,
                '▒',
                Color::Rgb(
                    lerp_channel(70, 52, dusk),
                    lerp_channel(96, 74, dusk),
                    lerp_channel(110, 98, dusk),
                ),
            ),
        ] {
            let top = horizon as i32 - height_delta;
            for row in top.max(0)..=horizon as i32 {
                put_cell(&mut buffer[..], row, col as i32, ch, color);
            }
        }
    }
}

/// Layered animated water surface with wave patterns and shimmer.
fn render_water_surface(
    buffer: &mut [Vec<SceneCell>],
    height: usize,
    horizon: usize,
    wave_tick: f64,
) {
    for (row, row_cells) in buffer.iter_mut().enumerate().skip(horizon) {
        let depth_t = if height - horizon <= 1 {
            0.0
        } else {
            (row - horizon) as f64 / (height - horizon - 1) as f64
        };
        let crest = lerp_rgb((154, 226, 248), (98, 184, 226), depth_t);
        for (col, cell) in row_cells.iter_mut().enumerate() {
            let primary = (col as f64 * (0.24 + depth_t * 0.06)
                + wave_tick * (0.055 + depth_t * 0.045))
                .sin();
            let secondary = (col as f64 * 0.11 - wave_tick * 0.043 + row as f64 * 0.92).cos();
            let wave = primary * 0.72 + secondary * 0.28;

            let ch = if wave > 0.74 {
                '≈'
            } else if wave > 0.38 {
                '~'
            } else if wave > 0.06 {
                '-'
            } else if wave > -0.34 {
                '·'
            } else {
                ' '
            };

            if ch != ' ' {
                let shimmer =
                    (((col as f64 * 0.09 + wave_tick * 0.19).sin() + 1.0) * 0.5 * 18.0) as u8;
                *cell = SceneCell::new(
                    ch,
                    Color::Rgb(
                        crest.0.saturating_add(shimmer),
                        crest.1.saturating_add(shimmer / 2),
                        crest.2.saturating_add(shimmer / 3),
                    ),
                    cell.bg,
                );
            }
        }
    }
}

/// Halfblock sailboat with gentle bob near the horizon.
fn render_sailboat(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    horizon: usize,
    height: usize,
    wave_tick: f64,
) {
    let boat_center = ((width as f64 * 0.35) + (wave_tick * 0.04).sin() * 2.0).round() as i32;
    let deck_row = (horizon + 1).min(height.saturating_sub(4)) as i32;
    let mast_x = boat_center;
    let mast_top = deck_row - 3;

    // Colour palette
    let hull_dark = (94, 58, 36);
    let hull_mid = (140, 90, 52);
    let hull_light = (188, 140, 86);
    let deck_color = (188, 148, 96);
    let mast_color = Color::Rgb(210, 168, 104);
    let sail_color = Color::Rgb(245, 235, 215);
    let pennant_color = Color::Rgb(255, 96, 72);

    // Helper: set a cell with explicit fg, bg, and char
    let set = |buf: &mut [Vec<SceneCell>],
               r: i32,
               c: i32,
               ch: char,
               fg: (u8, u8, u8),
               bg: (u8, u8, u8)| {
        if r < 0 || c < 0 {
            return;
        }
        let (ru, cu) = (r as usize, c as usize);
        if ru < buf.len() && cu < buf[ru].len() {
            buf[ru][cu] = SceneCell::new(
                ch,
                Color::Rgb(fg.0, fg.1, fg.2),
                Color::Rgb(bg.0, bg.1, bg.2),
            );
        }
    };

    // Helper: get bg color tuple at a buffer position
    let bg_at = |buf: &[Vec<SceneCell>], r: i32, c: i32| -> (u8, u8, u8) {
        if r < 0 || c < 0 {
            return (0, 0, 0);
        }
        let (ru, cu) = (r as usize, c as usize);
        if ru < buf.len() && cu < buf[ru].len() {
            match buf[ru][cu].bg {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (0, 0, 0),
            }
        } else {
            (0, 0, 0)
        }
    };

    // Row 0: pennant at mast top
    put_cell(buffer, mast_top, mast_x + 1, '▸', pennant_color);

    // Rows 1-2: sail (triangular) + mast
    put_cell(buffer, mast_top + 1, mast_x - 1, '╱', sail_color);
    put_cell(buffer, mast_top + 1, mast_x, '│', mast_color);

    put_cell(buffer, mast_top + 2, mast_x - 2, '╱', sail_color);
    let sail_fill = Color::Rgb(238, 228, 208);
    put_cell(buffer, mast_top + 2, mast_x - 1, '░', sail_fill);
    put_cell(buffer, mast_top + 2, mast_x, '│', mast_color);

    // Row 3: deck -- halfblock tapered edges blending into surrounding bg
    let dr = deck_row;
    let bg0 = bg_at(buffer, dr, mast_x - 6);
    set(buffer, dr, mast_x - 6, '▄', deck_color, bg0);
    let bg1 = bg_at(buffer, dr, mast_x - 5);
    set(buffer, dr, mast_x - 5, '▄', hull_mid, bg1);
    let bg2 = bg_at(buffer, dr, mast_x - 4);
    set(buffer, dr, mast_x - 4, '▟', hull_mid, bg2);
    // Solid deck section
    for dx in -3..=3 {
        let c = if dx == 0 { '│' } else { '█' };
        let fg = if dx == 0 {
            match mast_color {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => deck_color,
            }
        } else {
            deck_color
        };
        set(buffer, dr, mast_x + dx, c, fg, hull_dark);
    }
    // Right taper
    let bg3 = bg_at(buffer, dr, mast_x + 4);
    set(buffer, dr, mast_x + 4, '▙', hull_mid, bg3);
    let bg4 = bg_at(buffer, dr, mast_x + 5);
    set(buffer, dr, mast_x + 5, '▄', hull_mid, bg4);
    let bg5 = bg_at(buffer, dr, mast_x + 6);
    set(buffer, dr, mast_x + 6, '▄', deck_color, bg5);

    // Row 4: hull bottom
    let hr = deck_row + 1;
    let bg_l3 = bg_at(buffer, hr, mast_x - 4);
    set(buffer, hr, mast_x - 4, '▀', hull_mid, bg_l3);
    let bg_l2 = bg_at(buffer, hr, mast_x - 3);
    set(buffer, hr, mast_x - 3, '▄', hull_dark, bg_l2);
    for dx in -2..=2 {
        set(buffer, hr, mast_x + dx, '█', hull_dark, hull_dark);
    }
    let bg_r2 = bg_at(buffer, hr, mast_x + 3);
    set(buffer, hr, mast_x + 3, '▄', hull_dark, bg_r2);
    let bg_r3 = bg_at(buffer, hr, mast_x + 4);
    set(buffer, hr, mast_x + 4, '▀', hull_mid, bg_r3);

    // Row 5: keel/waterline
    let kr = deck_row + 2;
    for dx in -1..=1 {
        let bg_k = bg_at(buffer, kr, mast_x + dx);
        set(buffer, kr, mast_x + dx, '▀', hull_dark, bg_k);
    }

    // Small highlight stripe on upper hull for depth
    let bg_hl = bg_at(buffer, dr, mast_x - 3);
    set(buffer, dr, mast_x - 3, '▓', hull_light, bg_hl);
}

/// Bobber, fishing line, ripples, and splash effects.
fn render_bobber_and_line(
    buffer: &mut [Vec<SceneCell>],
    session: &FishingSession,
    width: usize,
    height: usize,
    horizon: usize,
    wave_tick: f64,
) {
    use crate::fishing::types::FishingPhase;

    // Sailboat mast position (must match render_sailboat for line origin)
    let boat_center = ((width as f64 * 0.35) + (wave_tick * 0.04).sin() * 2.0).round() as i32;
    let deck_row = (horizon + 1).min(height.saturating_sub(4)) as i32;
    let mast_x = boat_center;
    let mast_top = deck_row - 3;

    let (bobber_ratio, bobber_amp, bobber_sink, bobber_char, bobber_color, ripple_radius) =
        match session.phase {
            FishingPhase::Casting => (0.66, 0.45, 0.0, '○', Color::Rgb(240, 248, 255), 1),
            FishingPhase::Waiting => (0.58, 0.60, 0.2, '◉', Color::Rgb(255, 236, 214), 1),
            FishingPhase::Reeling => (0.50, 1.10, 0.9, '●', Color::Rgb(255, 96, 82), 2),
        };

    let bobber_x = ((width as f64 * bobber_ratio) + (wave_tick * 0.065).sin() * 2.5).round() as i32;
    let bobber_base = horizon + ((height - horizon).max(2) / 5);
    let bobber_y = (bobber_base as f64 + (wave_tick * 0.085).sin() * bobber_amp + bobber_sink)
        .round()
        .clamp((horizon + 1) as f64, (height.saturating_sub(2)) as f64) as i32;

    // Fishing line: shadow then bright
    draw_line(
        buffer,
        mast_x + 1,
        mast_top.max(0),
        bobber_x + 1,
        bobber_y,
        Color::Rgb(38, 52, 78),
    );
    draw_line(
        buffer,
        mast_x,
        mast_top.max(0),
        bobber_x,
        bobber_y,
        Color::Rgb(255, 208, 122),
    );

    // Bobber
    put_cell(buffer, bobber_y, bobber_x, bobber_char, bobber_color);

    // Ripples around bobber
    for ring in 1..=ripple_radius {
        let ch = if ring == 1 { 'o' } else { '.' };
        let fg = if ring == 1 {
            Color::Rgb(196, 236, 255)
        } else {
            Color::Rgb(146, 206, 238)
        };
        for &(dx, dy) in &[
            (-ring, 0),
            (ring, 0),
            (0, -ring),
            (0, ring),
            (-ring, -ring),
            (ring, -ring),
            (-ring, ring),
            (ring, ring),
        ] {
            put_cell(buffer, bobber_y + dy, bobber_x + dx, ch, fg);
        }
    }

    // Splash effect during reeling
    if session.phase == FishingPhase::Reeling {
        let splash_row = bobber_y - 1;
        let splash_x = bobber_x + if ((wave_tick as i32) & 1) == 0 { 4 } else { -4 };
        for (i, ch) in "<><".chars().enumerate() {
            put_cell(
                buffer,
                splash_row,
                splash_x + i as i32,
                ch,
                Color::Rgb(255, 224, 118),
            );
        }
        for &(dx, dy, ch) in &[(-1, -1, '\''), (2, -1, '\''), (0, -2, '`')] {
            put_cell(
                buffer,
                splash_row + dy,
                splash_x + dx,
                ch,
                Color::Rgb(216, 242, 255),
            );
        }
    }
}

/// Draws the catch progress indicator with current phase.
fn draw_catch_progress(frame: &mut Frame, area: Rect, session: &FishingSession) {
    use crate::fishing::types::FishingPhase;

    use super::throbber::spinner_char;

    let spinner = spinner_char();

    let caught = session.fish_caught.len() as u32;
    let total = session.total_fish;

    // Get phase text and color
    let (phase_text, phase_color) = match session.phase {
        FishingPhase::Casting => (format!("{} Casting line...", spinner), Color::White),
        FishingPhase::Waiting => (format!("{} Waiting for bite...", spinner), Color::Cyan),
        FishingPhase::Reeling => ("🐟 FISH ON! Reeling in!".to_string(), Color::Yellow),
    };

    let progress_text = vec![
        Line::from(vec![
            Span::styled("Caught: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}/{}", caught, total),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" fish"),
        ]),
        Line::from(vec![Span::styled(
            phase_text,
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let progress_block = Block::default().borders(Borders::ALL).title(" Status ");

    let progress_paragraph = Paragraph::new(progress_text)
        .block(progress_block)
        .alignment(Alignment::Center);

    frame.render_widget(progress_paragraph, area);
}

/// Data for each Storm Leviathan encounter stage.
struct LeviathanEncounterData {
    title: &'static str,
    flavor: &'static str,
    status: &'static str,
    health_bar: &'static str,
}

/// The 10 progressive encounters with the Storm Leviathan.
const LEVIATHAN_ENCOUNTERS: [LeviathanEncounterData; 10] = [
    LeviathanEncounterData {
        title: "RIPPLES",
        flavor: "Something disturbed the deep. A shadow vast as a ship passes beneath you. Before you can react, it vanishes into the abyss.",
        status: "UNTOUCHED",
        health_bar: "████████████████████",
    },
    LeviathanEncounterData {
        title: "THE SHADOW",
        flavor: "The Leviathan surfaces for a heartbeat - scales like storm clouds, eyes like lightning. It knows you now. Then it's gone.",
        status: "AWARE",
        health_bar: "██████████████████░░",
    },
    LeviathanEncounterData {
        title: "EMERGENCE",
        flavor: "It breaches! The beast roars - a sound like thunder over the waves. Your boat rocks violently as it dives deep.",
        status: "AGITATED",
        health_bar: "████████████████░░░░",
    },
    LeviathanEncounterData {
        title: "KNOWN",
        flavor: "It circles your position. Watching. Waiting. This is no mere fish - it's deciding if YOU are worthy prey.",
        status: "HUNTING",
        health_bar: "██████████████░░░░░░",
    },
    LeviathanEncounterData {
        title: "FIRST STRIKE",
        flavor: "Your hook finds flesh! The beast screams - a sound that will haunt your dreams. It dives, trailing darkness and blood.",
        status: "WOUNDED",
        health_bar: "████████████░░░░░░░░",
    },
    LeviathanEncounterData {
        title: "FURY",
        flavor: "It rams your boat in rage! You barely hold on as waves crash over the deck. But in its fury, it expends precious strength.",
        status: "RAGING",
        health_bar: "██████████░░░░░░░░░░",
    },
    LeviathanEncounterData {
        title: "BLOOD IN WATER",
        flavor: "Wounded and bleeding, it circles. You are both predator and prey now. Neither will yield. Neither can escape.",
        status: "BLEEDING",
        health_bar: "████████░░░░░░░░░░░░",
    },
    LeviathanEncounterData {
        title: "THE LONG NIGHT",
        flavor: "Hours pass. The beast surfaces less often, its movements slower. Stars wheel overhead. Dawn approaches. You will not sleep until this ends.",
        status: "TIRING",
        health_bar: "██████░░░░░░░░░░░░░░",
    },
    LeviathanEncounterData {
        title: "EXHAUSTION",
        flavor: "It can barely surface now. Each breath is labored, each dive shorter. Victory is close. You can taste it.",
        status: "EXHAUSTED",
        health_bar: "████░░░░░░░░░░░░░░░░",
    },
    LeviathanEncounterData {
        title: "LEGEND",
        flavor: "With a final, defiant bellow, it succumbs. Your line holds. Your arms burn. But you've done the impossible.",
        status: "DEFEATED",
        health_bar: "██░░░░░░░░░░░░░░░░░░",
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
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

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
