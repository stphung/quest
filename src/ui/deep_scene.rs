//! The Deep overlay — main coordinator with animated cave backdrop.
//!
//! Delegates sub-view rendering to:
//!   - deep_missions.rs  — Hub / NewMission views
//!   - deep_roster.rs    — Roster view
//!   - deep_layers.rs    — Infrastructure / Layer map view
//!   - deep_events.rs    — Event response view
//!   - deep_results.rs   — Mission complete results modal

use crate::deep::{DeepState, DeepUiState, DeepView};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
    Frame,
};

use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};

/// Themed border color for The Deep overlay.
pub(super) const DEEP_BORDER_COLOR: Color = Color::Rgb(80, 160, 220);

// ── Backdrop ──────────────────────────────────────────────────────────────────

/// Paint the deep cave backdrop: dark blue gradient with drifting dust particles.
pub(super) fn paint_deep_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // Background gradient: near-black deep blue top → void black bottom
    let top_rgb = (5u8, 8u8, 20u8);
    let bottom_rgb = (2u8, 3u8, 8u8);
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(top_rgb, bottom_rgb, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // Drifting dust particles (cave air)
    let particle_chars: &[char] = &['\u{00b7}', '\u{2022}', '\u{2218}']; // · • ∘
    let particle_count = 10;
    let particle_speed = 1.5;
    let particle_hot = (60u8, 80u8, 140u8);
    let particle_cool = (20u8, 30u8, 60u8);

    for i in 0..particle_count {
        let seed = hash2d(i, 7);
        let col = (seed as usize) % width.max(1);
        let ch = particle_chars[(hash2d(i, 13) as usize) % particle_chars.len()];

        let phase_offset = (seed as f64) * 0.81;
        let pos = (phase_offset + millis as f64 * particle_speed / 1000.0) % height as f64;
        let row = (height - 1) as f64 - pos;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(particle_hot, particle_cool, t);
        put_cell(
            buffer,
            row as i32,
            col as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }
}

/// Opening flourish: blue-white sheen sweeping top-to-bottom on overlay open (600ms).
pub(super) fn paint_opening_deep_fx(
    buffer: &mut [Vec<SceneCell>],
    _millis: u128,
    open_elapsed_ms: u128,
) {
    const OPEN_FX_MS: f64 = 600.0;
    if open_elapsed_ms as f64 >= OPEN_FX_MS {
        return;
    }

    let height = buffer.len();
    if height == 0 {
        return;
    }

    let progress = (open_elapsed_ms as f64 / OPEN_FX_MS).clamp(0.0, 1.0);
    let strength = 1.0 - progress;

    // Cool blue-white sheen
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let row_t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let sheen = (strength * (0.15 + 0.85 * (1.0 - row_t))).clamp(0.0, 1.0);
        for cell in row_cells.iter_mut() {
            let current_rgb = match cell.bg {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (5, 8, 20),
            };
            let lit = lerp_rgb(current_rgb, (80, 140, 210), sheen * 0.65);
            cell.bg = Color::Rgb(lit.0, lit.1, lit.2);
        }
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Render The Deep overlay.  Called from the scene dispatch in `mod.rs`.
pub fn render_deep_overlay(
    frame: &mut Frame,
    area: Rect,
    deep: &DeepState,
    ui: &DeepUiState,
    open_elapsed_ms: Option<u128>,
    ctx: &LayoutContext,
) {
    // Gracefully handle terminal-too-small
    if ctx.tier == SizeTier::TooSmall {
        return;
    }

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" THE DEEP ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(DEEP_BORDER_COLOR)));
    let inner =
        super::render_themed_block(frame, area, block, DEEP_BORDER_COLOR, super::BorderFxContext);

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Build scene buffer and paint backdrop
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_deep_backdrop(&mut buffer, millis);
    if let Some(elapsed) = open_elapsed_ms {
        paint_opening_deep_fx(&mut buffer, millis, elapsed);
    }

    // Dispatch to the appropriate sub-view
    match ui.view {
        DeepView::Hub => {
            super::deep_missions::render_hub(&mut buffer, width, height, deep, ui, ctx);
        }
        DeepView::NewMission => {
            super::deep_missions::render_new_mission(&mut buffer, width, height, deep, ui, ctx);
        }
        DeepView::Roster | DeepView::Recruit => {
            super::deep_roster::render_roster(&mut buffer, width, height, deep, ui, ctx);
        }
        DeepView::Infrastructure => {
            super::deep_layers::render_layers(&mut buffer, width, height, deep, ui, ctx);
        }
        DeepView::EventResponse => {
            super::deep_events::render_event_response(&mut buffer, width, height, deep, ui, ctx);
        }
    }

    // Flush buffer to frame
    render_buffer(frame, inner, &buffer);

    // Mission results modal is layered on top if there are pending results
    if !deep.prestige.pending_results.is_empty() {
        if let Some(mission) = deep.prestige.pending_results.first() {
            super::deep_results::render_mission_results(frame, area, mission, deep, ctx);
        }
    }
}

/// Render The Deep discovery modal (shown once on discovery).
pub fn render_deep_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &LayoutContext,
) {
    let modal_width = 56u16.min(area.width.saturating_sub(4));
    let modal_height = 11u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::Paragraph,
    };

    let block = Block::default()
        .title(" \u{25b6} New System Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "The Deep Discovered!",
            Style::default()
                .fg(Color::Rgb(80, 160, 220))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "A scarred mercenary captain approaches,",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "maps of underground passages in hand.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "\"The Deep goes further than you know.\"",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [D] to visit.  [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}
