//! Soulforge UI rendering: equipment enhancement overlay with animations.

use crate::enhancement::{EnhancementProgress, SoulforgePhase, SoulforgeUiState};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};

/// Parameters controlling the forge backdrop appearance.
struct ForgeBackdropParams {
    bottom_rgb: (u8, u8, u8),
    top_rgb: (u8, u8, u8),
    ember_count: usize,
    ember_speed: f64,
    ember_upward: bool,
    ember_hot: (u8, u8, u8),
    ember_cool: (u8, u8, u8),
    shimmer: bool,
}

impl ForgeBackdropParams {
    /// Standard warm forge glow (Menu, Confirming phases).
    fn normal() -> Self {
        Self {
            bottom_rgb: (120, 40, 15),
            top_rgb: (15, 8, 5),
            ember_count: 10,
            ember_speed: 5.0,
            ember_upward: true,
            ember_hot: (255, 160, 40),
            ember_cool: (80, 20, 5),
            shimmer: true,
        }
    }

    /// Intensified forge during hammering.
    fn hot() -> Self {
        Self {
            bottom_rgb: (180, 60, 20),
            top_rgb: (25, 12, 8),
            ember_count: 14,
            ember_speed: 7.0,
            ember_upward: true,
            ember_hot: (255, 200, 60),
            ember_cool: (120, 40, 10),
            shimmer: true,
        }
    }
}

/// Paint the forge backdrop into the buffer: gradient background, drifting embers, heat shimmer.
fn paint_forge_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128, params: &ForgeBackdropParams) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient (top to bottom)
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(params.top_rgb, params.bottom_rgb, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Drifting embers
    let ember_chars: &[char] = &['\u{00b7}', '\u{2022}', '*', '\u{2726}'];
    for i in 0..params.ember_count {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = ember_chars[(hash2d(i, 1) as usize) % ember_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * params.ember_speed / 1000.0) % height as f64;
        let row_f = if params.ember_upward {
            (height - 1) as f64 - pos
        } else {
            pos
        };
        let row = row_f as i32;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(params.ember_hot, params.ember_cool, t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 3. Heat shimmer
    if params.shimmer {
        let shimmer_phase = millis as f64 / 150.0;
        for (row, row_cells) in buffer.iter_mut().enumerate() {
            for (col, cell) in row_cells.iter_mut().enumerate() {
                if hash2d(row, col).is_multiple_of(7) {
                    let shift =
                        ((shimmer_phase + row as f64 * 0.3 + col as f64 * 0.2).sin() * 8.0) as i16;
                    if let Color::Rgb(r, g, b) = cell.bg {
                        let new_r = (r as i16 + shift).clamp(0, 255) as u8;
                        cell.bg = Color::Rgb(new_r, g, b);
                    }
                }
            }
        }
    }
}

/// Render the soulforge overlay
pub fn render_soulforge(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center overlay: 62 wide, 24 tall (or fit to terminal)
    let overlay_width = 62u16.min(area.width.saturating_sub(4));
    let overlay_height = 24u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let title = format!(
        " \u{2692} The Soulforge  [Prestige Ranks: {}] ",
        prestige_rank
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    match soulforge_ui.phase {
        SoulforgePhase::Menu => {
            render_menu(frame, inner, soulforge_ui, enhancement, prestige_rank);
        }
        SoulforgePhase::Confirming => {
            render_confirming(frame, inner, soulforge_ui, enhancement, prestige_rank);
        }
        SoulforgePhase::Hammering => {
            render_hammering(frame, inner, soulforge_ui, enhancement);
        }
        SoulforgePhase::ResultSuccess => {
            render_success(frame, inner, soulforge_ui);
        }
        SoulforgePhase::ResultFailure => {
            render_failure(frame, inner, soulforge_ui);
        }
    }
}

/// Render the equipment slot menu using scene buffer with forge backdrop.
fn render_menu(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_forge_backdrop(&mut buffer, millis, &ForgeBackdropParams::normal());

    super::soulforge_slots::render_menu_content(
        &mut buffer,
        millis,
        soulforge_ui,
        enhancement,
        prestige_rank,
    );

    render_buffer(frame, area, &buffer);
}

/// Render the confirmation phase using scene buffer with intensified forge backdrop.
fn render_confirming(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    let mut params = ForgeBackdropParams::normal();
    params.bottom_rgb = (150, 50, 18);
    params.ember_count = 12;
    paint_forge_backdrop(&mut buffer, millis, &params);

    super::soulforge_slots::render_confirming_content(
        &mut buffer,
        millis,
        soulforge_ui,
        enhancement,
        prestige_rank,
    );

    render_buffer(frame, area, &buffer);
}

/// Render the hammering animation
fn render_hammering(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_forge_backdrop(&mut buffer, millis, &ForgeBackdropParams::hot());

    super::soulforge_effects::render_hammering_content(&mut buffer, soulforge_ui, enhancement);

    render_buffer(frame, area, &buffer);
}

/// Render the success animation with golden burst and sparkle effects.
fn render_success(frame: &mut Frame, area: Rect, soulforge_ui: &SoulforgeUiState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let intensity = (tick as f64 / 30.0).min(1.0);
    let params = ForgeBackdropParams {
        bottom_rgb: lerp_rgb((120, 40, 15), (200, 170, 50), intensity),
        top_rgb: lerp_rgb((15, 8, 5), (40, 30, 10), intensity),
        ember_count: 10 + (10.0 * intensity) as usize,
        ember_speed: 5.0 + 3.0 * intensity,
        ember_upward: true,
        ember_hot: lerp_rgb((255, 160, 40), (255, 230, 100), intensity),
        ember_cool: lerp_rgb((80, 20, 5), (200, 150, 30), intensity),
        shimmer: true,
    };
    paint_forge_backdrop(&mut buffer, millis, &params);

    super::soulforge_effects::render_success_content(&mut buffer, soulforge_ui);

    render_buffer(frame, area, &buffer);
}

/// Render the failure animation with ash decay and crack effects.
fn render_failure(frame: &mut Frame, area: Rect, soulforge_ui: &SoulforgeUiState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let cool_t = (tick as f64 / 8.0).min(1.0);
    let params = ForgeBackdropParams {
        bottom_rgb: lerp_rgb((120, 40, 15), (40, 40, 45), cool_t),
        top_rgb: lerp_rgb((15, 8, 5), (15, 15, 18), cool_t),
        ember_count: 10 - (6.0 * cool_t) as usize,
        ember_speed: 5.0 - 3.0 * cool_t,
        ember_upward: cool_t < 1.0,
        ember_hot: lerp_rgb((255, 160, 40), (80, 30, 10), cool_t),
        ember_cool: lerp_rgb((80, 20, 5), (30, 15, 10), cool_t),
        shimmer: cool_t <= 0.5,
    };
    paint_forge_backdrop(&mut buffer, millis, &params);

    // Crack characters
    let max_cracks = 8usize;
    let active_cracks = ((cool_t * max_cracks as f64).ceil() as usize).min(max_cracks);
    let crack_char = '\u{2573}';
    let crack_rgb = lerp_rgb((180, 60, 20), (80, 80, 90), cool_t);
    let crack_fg = Color::Rgb(crack_rgb.0, crack_rgb.1, crack_rgb.2);
    for i in 0..active_cracks {
        let seed = hash2d(i + 42, i * 7 + 13);
        let row = (seed as usize) % h;
        let col = (hash2d(i + 99, i * 3 + 5) as usize) % w;
        put_cell(&mut buffer, row as i32, col as i32, crack_char, crack_fg);
    }

    super::soulforge_effects::render_failure_content(&mut buffer, soulforge_ui);

    render_buffer(frame, area, &buffer);
}

/// Render the Soulforge discovery modal
pub fn render_soulforge_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center the modal
    let modal_width = 52u16.min(area.width.saturating_sub(4));
    let modal_height = 10u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{25b6} New System Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "\u{2692}\u{fe0f} You've uncovered an ancient Soulforge!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Ancient runes pulse with forgotten power.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "This forge tempers the soul, not the steel.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "All that you wield will strike truer.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [S] to visit. [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
