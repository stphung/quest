//! Stormglass Exchange UI rendering: storm-themed modal overlay.

use crate::core::game_state::GameState;
use crate::stormglass::types::{
    ChronoSurgeState, ChronoSurgeSummary, ExchangePhase, ExchangeUiState, CHRONO_SURGE_OPTIONS,
    INVOKE_TRIAL_COST,
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::scene_fx::{
    current_millis, display_width, hash2d, lerp_rgb, put_cell, put_text, put_text_centered,
    render_buffer, SceneCell,
};

/// Electric blue color used throughout Stormglass UI.
const ELECTRIC_BLUE: Color = Color::Rgb(100, 180, 255);

/// Rune circle purple color.
const RUNE_PURPLE: Color = Color::Rgb(180, 160, 255);
/// Inscription flash color (warm white).
const INSCRIPTION_FLASH: Color = Color::Rgb(255, 255, 200);
/// Invoke Challenge animation timing boundaries in milliseconds.
const INVOKE_PHASE1_END_MS: u128 = 1300;
const INVOKE_PHASE2_END_MS: u128 = 2900;
const INVOKE_PHASE3_END_MS: u128 = 4200;
/// Phase timing boundaries in milliseconds.
const PHASE1_END_MS: u128 = 1400;
const PHASE2_END_MS: u128 = 2800;
const PHASE3_END_MS: u128 = 3500;

/// Temporal-tech accent colors for Chrono Surge visuals.
const TIMEWARP_CYAN: Color = Color::Rgb(120, 220, 255);
const TIMEWARP_MID: Color = Color::Rgb(70, 150, 230);
const TIMEWARP_DEEP: Color = Color::Rgb(20, 40, 90);

/// Parameters controlling the storm backdrop appearance.
struct StormBackdropParams {
    top_rgb: (u8, u8, u8),
    bottom_rgb: (u8, u8, u8),
    particle_count: usize,
    particle_speed: f64,
    shimmer: bool,
}

impl StormBackdropParams {
    fn normal() -> Self {
        Self {
            top_rgb: (10, 15, 40),
            bottom_rgb: (5, 5, 15),
            particle_count: 8,
            particle_speed: 1.5,
            shimmer: true,
        }
    }
}

/// Paint the storm backdrop: dark gradient, lightning particles, shimmer.
fn paint_storm_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128, params: &StormBackdropParams) {
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

    // 2. Subtle particles drifting downward
    let particle_chars: &[char] = &['\u{00b7}', '\u{2022}', '\u{2726}', '*'];
    let particle_hot: (u8, u8, u8) = (120, 170, 230);
    let particle_cool: (u8, u8, u8) = (30, 50, 100);
    for i in 0..params.particle_count {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = particle_chars[(hash2d(i, 1) as usize) % particle_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * params.particle_speed / 1000.0) % height as f64;
        let row = pos as i32;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(particle_hot, particle_cool, t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 3. Occasional lightning flash (random bright cells)
    if params.shimmer {
        let flash_phase = (millis / 80) as usize;
        for i in 0..3 {
            let seed = hash2d(flash_phase.wrapping_add(i), 77);
            let row = (seed as usize) % height;
            let col = (hash2d(flash_phase.wrapping_add(i), 88) as usize) % width;
            let brightness = 60 + ((seed % 40) as u8);
            put_cell(
                buffer,
                row as i32,
                col as i32,
                '\u{00b7}',
                Color::Rgb(brightness, brightness + 40, 255),
            );
        }
    }
}

/// Clear particle characters from a row, preserving only the background gradient.
fn clear_row_chars(buffer: &mut [Vec<SceneCell>], row: i32) {
    if row >= 0 && (row as usize) < buffer.len() {
        for cell in buffer[row as usize].iter_mut() {
            cell.ch = ' ';
            cell.fg = Color::Reset;
        }
    }
}

/// Convert surge ticks (10 ticks/sec) to a compact duration label.
fn chrono_ticks_to_duration_label(ticks: u64) -> String {
    let total_mins = ticks / 600;
    let hours = total_mins / 60;
    let mins = total_mins % 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Render the full Stormglass Exchange overlay as a centered modal.
pub fn render_stormglass_exchange(
    frame: &mut Frame,
    area: Rect,
    exchange_ui: &ExchangeUiState,
    state: &GameState,
    _ctx: &super::responsive::LayoutContext,
) {
    match exchange_ui.phase {
        ExchangePhase::Menu => render_exchange_menu(frame, area, exchange_ui, state),
        ExchangePhase::InvokeTrialConfirm => render_invoke_trial_confirm(frame, area, state),
        ExchangePhase::InvokeTrialRolling => render_invoke_trial_rolling(frame, area, exchange_ui),
        ExchangePhase::InvokeTrial => render_invoke_trial(frame, area, exchange_ui),
        ExchangePhase::InvokeTrialForfeitConfirm => {
            // Render the trial selection underneath, then overlay the forfeit confirm
            render_invoke_trial(frame, area, exchange_ui);
            render_invoke_trial_forfeit_confirm(frame, area);
        }
        ExchangePhase::ChronoSurge => render_chrono_surge_select(frame, area, exchange_ui, state),
        ExchangePhase::SigilsList => render_sigils_list(frame, area, exchange_ui, state),
        ExchangePhase::SigilUnlockConfirm => render_sigil_unlock_confirm(frame, area, state),
        ExchangePhase::SigilEtchConfirm => render_sigil_etch_confirm(frame, area, state),
        ExchangePhase::SigilRerollConfirm => {
            render_sigil_reroll_confirm(frame, area, exchange_ui, state)
        }
        ExchangePhase::SigilRolling => render_sigil_rolling(frame, area, exchange_ui),
        ExchangePhase::SigilPick => render_sigil_pick(frame, area, exchange_ui),
        ExchangePhase::SigilForfeitConfirm => {
            render_sigil_pick(frame, area, exchange_ui);
            render_sigil_forfeit_confirm(frame, area);
        }
        ExchangePhase::SigilResult => render_sigil_result(frame, area, exchange_ui),
    }
}

fn render_exchange_menu(
    frame: &mut Frame,
    area: Rect,
    exchange_ui: &ExchangeUiState,
    state: &GameState,
) {
    // Center overlay: 52 wide, 16 tall (or fit to terminal)
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 16u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let title = " \u{1F48E} Stormglass Exchange \u{1F48E} ";
    let block = Block::default()
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    // Clear particle chars from rows that will have text (preserves bg gradient)
    clear_row_chars(&mut buffer, 1); // flavor line 1 (rendered as widget overlay after buffer)
    clear_row_chars(&mut buffer, 2); // flavor line 2
    for i in 0..3 {
        clear_row_chars(&mut buffer, 4 + i); // menu items
    }
    clear_row_chars(&mut buffer, 8); // description line 1
    clear_row_chars(&mut buffer, 9); // description line 2
    clear_row_chars(&mut buffer, (h as i32) - 1); // help

    // Menu items (rows 4-6): (display_text, cost_text, affordable)
    // Wide emoji icons (2 terminal cols) get 1 space before label;
    // narrow icons (1 terminal col) get 2 spaces — so labels align.
    let items: [(&str, String, bool); 3] = [
        (
            "\u{1F3B2} Invoke Challenge", // 🎲 (2-wide) + 1 space
            format!("{} SG", INVOKE_TRIAL_COST),
            state.stormglass >= INVOKE_TRIAL_COST,
        ),
        (
            "\u{231B} Chrono Surge", // ⌛ (2-wide) + 1 space
            ">>>".to_string(),
            true,
        ),
        (
            "\u{16B1}  Etch Storm Sigils", // ᚱ (1-wide) + 2 spaces
            ">>>".to_string(),
            true,
        ),
    ];

    let menu_start_row = 4i32;
    let text_col = 2i32;
    for (i, (display, cost, affordable)) in items.iter().enumerate() {
        let row = menu_start_row + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = i == exchange_ui.selected_item;

        // Cursor
        if is_selected {
            put_text(&mut buffer, row, 0, "> ", Color::Yellow);
        }

        let fg = if *affordable {
            Color::White
        } else {
            Color::DarkGray
        };

        // Icon + label as one string so terminal width is handled naturally
        put_text(&mut buffer, row, text_col, display, fg);

        // Right-aligned cost
        let cost_fg = if *affordable {
            ELECTRIC_BLUE
        } else {
            Color::DarkGray
        };
        let cost_col = (w as i32) - cost.chars().count() as i32 - 1;
        put_text(&mut buffer, row, cost_col, cost, cost_fg);

        // Selected row highlight
        if is_selected {
            let highlight_bg = Color::Rgb(15, 25, 55);
            if (row as usize) < h {
                for cell in buffer[row as usize].iter_mut() {
                    cell.bg = highlight_bg;
                }
            }
        }
    }

    // Help row at bottom
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[\u{2191}\u{2193}] Select  [Enter] Exchange  [Esc] Close",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    // Flavor text overlay (rows 1-2) — rendered as Paragraph widget for italic + wrapping
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "Each shard of Stormglass holds a tempest waiting to be unleashed.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }

    // Description overlay (rows 8-9) — changes based on selected item
    if h > 9 {
        let desc = match exchange_ui.selected_item {
            0 => "Spend Stormglass to invoke a choice of three challenges.",
            1 => "Bend time itself. Earn XP and loot, but no Stormglass.",
            _ => "Etch sigils of power onto your soul. Permanent bonuses.",
        };
        let desc_area = Rect::new(inner.x, inner.y + 8, inner.width, 2);
        let desc_widget = Paragraph::new(Span::styled(
            desc,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(desc_widget, desc_area);
    }
}

fn render_invoke_trial_confirm(frame: &mut Frame, area: Rect, state: &GameState) {
    // Center overlay: 52 wide, 14 tall
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{1F3B2} Invoke Challenge? \u{1F3B2} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    // Clear text rows
    clear_row_chars(&mut buffer, 1); // flavor
    clear_row_chars(&mut buffer, 2); // flavor
    clear_row_chars(&mut buffer, 4); // balance
    clear_row_chars(&mut buffer, 5); // cost
    clear_row_chars(&mut buffer, 6); // after
    clear_row_chars(&mut buffer, 8); // description
    clear_row_chars(&mut buffer, (h as i32) - 1); // help

    // Balance / Cost / After breakdown (rows 4-6)
    let balance = state.stormglass;
    let after = balance.saturating_sub(INVOKE_TRIAL_COST);

    let balance_str = format!("Balance:  {} SG", balance);
    put_text(&mut buffer, 4, 4, &balance_str, Color::White);

    let cost_str = format!("Cost:    -{} SG", INVOKE_TRIAL_COST);
    put_text(&mut buffer, 5, 4, &cost_str, Color::LightRed);

    let after_str = format!("After:    {} SG", after);
    put_text(&mut buffer, 6, 4, &after_str, ELECTRIC_BLUE);

    // Description
    put_text_centered(
        &mut buffer,
        8,
        w,
        "You will choose one of three random challenges.",
        Color::DarkGray,
    );

    // Help row
    let help_row = (h as i32) - 1;
    // Build the help text with colored segments
    let help_y_col = 4i32;
    put_text(&mut buffer, help_row, help_y_col, "[", Color::DarkGray);
    put_text(&mut buffer, help_row, help_y_col + 1, "Y", Color::Green);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 2,
        "] Invoke  [",
        Color::DarkGray,
    );
    put_text(&mut buffer, help_row, help_y_col + 13, "N", Color::LightRed);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 14,
        "] Cancel",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    // Flavor text overlay (rows 1-2)
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "The storm fractures. Three trials will emerge.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }
}

fn render_invoke_trial_forfeit_confirm(frame: &mut Frame, area: Rect) {
    // Small modal overlay: ~42 wide, ~8 tall, centered
    let modal_width = 42u16.min(area.width.saturating_sub(4));
    let modal_height = 8u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " Abandon Trial? ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "3,000 SG already spent.",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "The Stormglass cannot be reclaimed.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("] Leave  [", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("] Stay", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

fn render_invoke_trial_rolling(frame: &mut Frame, area: Rect, exchange_ui: &ExchangeUiState) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{1F3B2} Casting the Lot \u{1F3B2} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let millis = current_millis();
    let anim_start = exchange_ui.invoke_animation_start_ms.unwrap_or(millis);
    let elapsed = millis.saturating_sub(anim_start);

    let intensity = (elapsed as f64 / INVOKE_PHASE3_END_MS as f64).clamp(0.0, 1.0);
    let params = StormBackdropParams {
        top_rgb: lerp_rgb((10, 15, 40), (20, 24, 55), intensity),
        bottom_rgb: lerp_rgb((5, 5, 15), (8, 10, 28), intensity),
        particle_count: 10 + (intensity * 10.0) as usize,
        particle_speed: 1.7 + intensity * 1.4,
        shimmer: true,
    };
    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    paint_storm_backdrop(&mut buffer, millis, &params);

    if elapsed < INVOKE_PHASE1_END_MS {
        render_invoke_rolling_phase1(&mut buffer, w, h, millis, elapsed);
    } else if elapsed < INVOKE_PHASE2_END_MS {
        let phase_elapsed = elapsed - INVOKE_PHASE1_END_MS;
        render_invoke_rolling_phase2(&mut buffer, w, h, exchange_ui, millis, phase_elapsed);
    } else {
        render_invoke_rolling_phase3(&mut buffer, w, h, exchange_ui, millis);
    }

    if elapsed > 200 {
        let help_row = (h as i32) - 1;
        clear_row_chars(&mut buffer, help_row);
        put_text_centered(&mut buffer, help_row, w, "[Any key] Skip", Color::DarkGray);
    }

    render_buffer(frame, inner, &buffer);
}

fn render_invoke_rolling_phase1(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    millis: u128,
    elapsed: u128,
) {
    for row in 0..h {
        clear_row_chars(buffer, row as i32);
    }

    let progress = (elapsed as f64 / INVOKE_PHASE1_END_MS as f64).clamp(0.0, 1.0);
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    let dice = [
        '\u{2680}', '\u{2681}', '\u{2682}', '\u{2683}', '\u{2684}', '\u{2685}',
    ];
    let orbit_speed = millis as f64 / 260.0;
    let radius = 4.8 - progress * 2.8;

    for i in 0..6 {
        let angle = orbit_speed + (i as f64 * std::f64::consts::TAU / 6.0);
        let x = cx + angle.cos() * radius * 2.0;
        let y = cy + angle.sin() * radius;
        let face = dice[((millis / 90 + i as u128) % dice.len() as u128) as usize];
        let glow = lerp_rgb((120, 150, 210), (220, 240, 255), progress);
        put_cell(
            buffer,
            y.round() as i32,
            x.round() as i32,
            face,
            Color::Rgb(glow.0, glow.1, glow.2),
        );
    }

    put_cell(
        buffer,
        cy.round() as i32,
        cx.round() as i32,
        '\u{2726}',
        ELECTRIC_BLUE,
    );
    put_text_centered(buffer, 1, w, "Casting storm-lot...", ELECTRIC_BLUE);
    put_text_centered(
        buffer,
        2,
        w,
        "Thunder picks three trials.",
        Color::Rgb(170, 200, 240),
    );
}

fn render_invoke_rolling_phase2(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
    millis: u128,
    phase_elapsed: u128,
) {
    for row in 0..h {
        clear_row_chars(buffer, row as i32);
    }

    put_text_centered(buffer, 1, w, "Reading the fracture...", ELECTRIC_BLUE);
    put_text_centered(
        buffer,
        2,
        w,
        "Names coalesce from static.",
        Color::Rgb(150, 180, 220),
    );

    let row_start = 4i32;
    let stagger_ms: u128 = 170;
    let phase_duration = INVOKE_PHASE2_END_MS - INVOKE_PHASE1_END_MS;
    let reveal_duration = (phase_duration.saturating_sub(stagger_ms * 2)).max(1);
    for (i, trial) in exchange_ui.trial_options.iter().enumerate() {
        let row = row_start + i as i32;
        if row >= h as i32 {
            break;
        }

        let row_elapsed = phase_elapsed.saturating_sub(stagger_ms * i as u128);
        let reveal_t = (row_elapsed as f64 / reveal_duration as f64).clamp(0.0, 1.0);
        let total_chars = trial.display_name.chars().count();
        let reveal_chars = ((total_chars as f64) * reveal_t).round() as usize;
        let row_seed = (millis as usize / 80).wrapping_add(i * 97);
        let scrambled = scramble_trial_name(&trial.display_name, reveal_chars, row_seed);

        let fg_rgb = lerp_rgb((90, 110, 150), (235, 245, 255), reveal_t);
        put_text(
            buffer,
            row,
            3,
            &scrambled,
            Color::Rgb(fg_rgb.0, fg_rgb.1, fg_rgb.2),
        );

        if reveal_t < 1.0 {
            let scan_col = 3 + ((display_width(&trial.display_name) as f64 * reveal_t) as i32);
            put_cell(buffer, row, scan_col, '\u{2502}', ELECTRIC_BLUE);
        }
    }
}

fn render_invoke_rolling_phase3(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
    millis: u128,
) {
    for row in 0..h {
        clear_row_chars(buffer, row as i32);
    }

    put_text_centered(
        buffer,
        1,
        w,
        "The fracture stabilizes. Choose a trial.",
        Color::Rgb(190, 220, 255),
    );
    render_trial_option_rows(buffer, h, exchange_ui, Some(0));

    // Pulse selected row border glyphs to hint imminent transition.
    let pulse = ((millis as f64 / 180.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let pulse_rgb = lerp_rgb((120, 140, 190), (220, 240, 255), pulse);
    put_cell(
        buffer,
        4,
        0,
        '\u{25B8}',
        Color::Rgb(pulse_rgb.0, pulse_rgb.1, pulse_rgb.2),
    );
}

fn scramble_trial_name(name: &str, reveal_chars: usize, seed: usize) -> String {
    let masks = ['?', '*', ':', '+', '/', '\\', '|'];
    name.chars()
        .enumerate()
        .map(|(i, ch)| {
            if i < reveal_chars || ch == ' ' || ch == '\'' || ch == '-' {
                ch
            } else {
                masks
                    [hash2d(seed.wrapping_add(i), i.wrapping_add(seed * 13)) as usize % masks.len()]
            }
        })
        .collect()
}

fn render_trial_option_rows(
    buffer: &mut [Vec<SceneCell>],
    h: usize,
    exchange_ui: &ExchangeUiState,
    selected: Option<usize>,
) {
    let trial_start_row = 4i32;
    for (i, trial) in exchange_ui.trial_options.iter().enumerate() {
        let row = trial_start_row + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = selected.is_some_and(|idx| idx == i);

        if is_selected {
            put_text(buffer, row, 1, "> ", Color::Yellow);
        }
        put_text(buffer, row, 3, &trial.display_name, Color::White);

        if is_selected && (row as usize) < h {
            let highlight_bg = Color::Rgb(15, 25, 55);
            for cell in buffer[row as usize].iter_mut() {
                cell.bg = highlight_bg;
            }
        }
    }
}

fn render_invoke_trial(frame: &mut Frame, area: Rect, exchange_ui: &ExchangeUiState) {
    // Center overlay: 52 wide, 14 tall
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{1F3B2} Invoke Challenge \u{1F3B2} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    // Clear particle chars from text rows (preserves bg gradient)
    clear_row_chars(&mut buffer, 1); // flavor line 1
    clear_row_chars(&mut buffer, 2); // flavor line 2
    for i in 0..3 {
        clear_row_chars(&mut buffer, 4 + i); // trial options
    }
    clear_row_chars(&mut buffer, (h as i32) - 1); // help

    // Flavor text
    put_text_centered(
        &mut buffer,
        1,
        w,
        "Stormglass shatters. Trials emerge",
        Color::Rgb(150, 200, 255),
    );
    put_text_centered(
        &mut buffer,
        2,
        w,
        "from the fracture.",
        Color::Rgb(150, 200, 255),
    );

    // Trial options (rows 4-6)
    render_trial_option_rows(
        &mut buffer,
        h,
        exchange_ui,
        Some(exchange_ui.trial_selected),
    );

    // Help row at bottom
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[Enter] Select  [Esc] Forfeit",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);
}

/// Render the Chrono Surge duration selection screen.
fn render_chrono_surge_select(
    frame: &mut Frame,
    area: Rect,
    exchange_ui: &ExchangeUiState,
    state: &GameState,
) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 16u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let title = " \u{231B} Chrono Surge \u{231B} ";
    let block = Block::default()
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    // Clear text rows
    clear_row_chars(&mut buffer, 1);
    clear_row_chars(&mut buffer, 2);
    for i in 0..4 {
        clear_row_chars(&mut buffer, 4 + i);
    }
    clear_row_chars(&mut buffer, (h as i32) - 1);

    // Duration options (rows 4-7)
    let options_start = 4i32;
    for (i, (_, cost, label)) in CHRONO_SURGE_OPTIONS.iter().enumerate() {
        let row = options_start + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = i == exchange_ui.surge_selected;
        let affordable = state.stormglass >= *cost;

        let mut col = 0i32;
        if is_selected {
            put_text(&mut buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        let name_fg = if affordable {
            Color::White
        } else {
            Color::DarkGray
        };
        put_text(&mut buffer, row, col, label, name_fg);

        let cost_str = format!("{} SG", cost);
        let cost_fg = if affordable {
            ELECTRIC_BLUE
        } else {
            Color::DarkGray
        };
        let cost_col = (w as i32) - cost_str.chars().count() as i32 - 1;
        put_text(&mut buffer, row, cost_col, &cost_str, cost_fg);

        if is_selected {
            let highlight_bg = Color::Rgb(15, 25, 55);
            if (row as usize) < h {
                for cell in buffer[row as usize].iter_mut() {
                    cell.bg = highlight_bg;
                }
            }
        }
    }

    // Help row
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[\u{2191}\u{2193}] Select  [Enter] Activate  [Esc] Back",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    // Flavor text overlay
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "Bend time itself. Earn XP and loot, but no Stormglass.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }
}

/// Render the active Chrono Surge status banner along the bottom of the screen.
/// The normal game view remains visible and updates in fast-forward.
fn draw_single_overlay_glyph(frame: &mut Frame, area: Rect, x: i32, y: i32, ch: char, fg: Color) {
    let min_x = area.x as i32;
    let max_x = min_x + area.width as i32;
    let min_y = area.y as i32;
    let max_y = min_y + area.height as i32;
    if x < min_x || x >= max_x || y < min_y || y >= max_y {
        return;
    }
    let glyph = Paragraph::new(Span::styled(ch.to_string(), Style::default().fg(fg)))
        .alignment(Alignment::Center);
    frame.render_widget(glyph, Rect::new(x as u16, y as u16, 1, 1));
}

type StreamLayer = (usize, u128, usize, char, (u8, u8, u8), (u8, u8, u8));

/// Render a global temporal-tech overlay while surge is fast-forwarding gameplay.
/// Kept non-destructive (foreground glyphs only) so core gameplay remains readable.
pub fn render_chrono_surge_time_warp(
    frame: &mut Frame,
    area: Rect,
    surge: &ChronoSurgeState,
    _ctx: &super::responsive::LayoutContext,
) {
    if area.width < 24 || area.height < 8 {
        return;
    }

    let millis = current_millis();
    let progress = surge.progress();
    let pct = (progress * 100.0) as u32;
    let velocity_scalar = surge.speed_multiplier();
    let accel = ((velocity_scalar - 0.85) / (3.40 - 0.85)).clamp(0.0, 1.0);
    let pulse_t = ((millis as f64 / 260.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let pulse_rgb = lerp_rgb((120, 220, 255), (235, 245, 255), pulse_t);
    let pulse_color = Color::Rgb(pulse_rgb.0, pulse_rgb.1, pulse_rgb.2);
    let accent_t = ((millis as f64 / 190.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let accent_rgb = lerp_rgb((90, 160, 245), (170, 225, 255), accent_t);
    let accent_color = Color::Rgb(accent_rgb.0, accent_rgb.1, accent_rgb.2);

    let badge = format!("⏩ x{} warp {:>3}%", surge.current_batch_size(), pct);
    let badge_w = display_width(&badge) as u16;
    let badge_x = area
        .x
        .saturating_add(area.width.saturating_sub(badge_w.saturating_add(1)));
    let badge_y = area.y.saturating_add(1);
    let badge_area = Rect::new(badge_x, badge_y, badge_w.min(area.width), 1);
    let badge_widget = Paragraph::new(Span::styled(
        badge,
        Style::default()
            .fg(pulse_color)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(badge_widget, badge_area);

    let top = area.y.saturating_add(2) as i32;
    let bottom = (area.y + area.height.saturating_sub(4)) as i32;
    if bottom <= top {
        return;
    }
    let left = area.x as i32 + 1;
    let right = (area.x + area.width.saturating_sub(2)) as i32;
    let inner_w = (right - left + 1).max(1);
    let field_h = (bottom - top + 1).max(1);

    // Parallax streak layers moving right-to-left.
    let layers: [StreamLayer; 4] = [
        (1, 18, 12, '─', (220, 248, 255), (90, 150, 220)),
        (2, 30, 9, '╌', (180, 230, 255), (70, 120, 190)),
        (3, 45, 7, '·', (145, 205, 245), (55, 95, 160)),
        (4, 65, 5, '·', (120, 185, 230), (45, 80, 140)),
    ];

    for (layer_idx, (row_step, speed_ms, streak_len, body_ch, hot_rgb, cool_rgb)) in
        layers.iter().enumerate()
    {
        let start_row = top + (layer_idx as i32 % *row_step as i32);
        let mut row_count = 0usize;
        let dynamic_len = *streak_len + (accel * (*streak_len as f64 * 0.95)).round() as usize;
        for y in (start_row..=bottom).step_by(*row_step) {
            let phase_span = inner_w + dynamic_len as i32 + 8;
            let accel_phase =
                (accel * phase_span as f64 * (4.0 + layer_idx as f64 * 1.8)).round() as i32;
            let phase = (((millis / *speed_ms) as i32)
                + accel_phase
                + row_count as i32 * (4 + layer_idx as i32)
                + layer_idx as i32 * 13)
                .rem_euclid(phase_span);
            let head_x = right - phase;

            for seg in 0..dynamic_len {
                let x = head_x + seg as i32;
                if x < left || x > right {
                    continue;
                }
                let seg_t = seg as f64 / dynamic_len.max(1) as f64;
                let rgb = lerp_rgb(*hot_rgb, *cool_rgb, seg_t);
                let color = Color::Rgb(rgb.0, rgb.1, rgb.2);
                let ch = if seg == 0 {
                    if accel > 0.72 {
                        '◉'
                    } else if layer_idx <= 1 {
                        '✦'
                    } else {
                        '•'
                    }
                } else if seg <= 2 {
                    '•'
                } else {
                    *body_ch
                };
                draw_single_overlay_glyph(frame, area, x, y, ch, color);
            }
            row_count += 1;
        }
    }

    // Sparse temporal gates that sweep right-to-left.
    let gate_row_step = if accel > 0.55 { 2 } else { 3 };
    for gate in 0..4i32 {
        let phase = (((millis / 95) as i32)
            + gate * ((inner_w / 4).max(8))
            + (accel * (inner_w + 12) as f64 * 5.0).round() as i32)
            .rem_euclid(inner_w + 12);
        let x = right - phase;
        if x >= left && x <= right {
            for y in (top..=bottom).step_by(gate_row_step) {
                let ch = if (y + gate) % 2 == 0 { '┊' } else { '╎' };
                draw_single_overlay_glyph(frame, area, x, y, ch, TIMEWARP_MID);
            }
        }
    }

    // Fast shards with tiny trails.
    let shard_count = 16 + (accel * 24.0).round() as usize;
    let shard_trail = 2 + (accel * 4.0).round() as i32;
    for i in 0..shard_count {
        let seed = hash2d(i, (millis / 55) as usize);
        let y = top + (seed as i32 % field_h);
        let phase = (((millis / 24) as i32)
            + (seed as i32 % 41)
            + (accel * (inner_w + 10) as f64 * 7.0).round() as i32)
            .rem_euclid(inner_w + 10);
        let x = right - phase;
        if x < left || x > right {
            continue;
        }
        let color = if i % 3 == 0 {
            pulse_color
        } else {
            accent_color
        };
        let shard_head = if accel > 0.75 { '◁' } else { '◀' };
        draw_single_overlay_glyph(frame, area, x, y, shard_head, color);
        for t in 1..=shard_trail {
            let trail_ch = if t <= 2 { '·' } else { '╌' };
            draw_single_overlay_glyph(frame, area, x + t, y, trail_ch, TIMEWARP_MID);
        }
    }
}

pub fn render_chrono_surge_banner(
    frame: &mut Frame,
    area: Rect,
    surge: &ChronoSurgeState,
    _ctx: &super::responsive::LayoutContext,
) {
    let banner_h = 3u16;
    if area.height < banner_h + 4 {
        return;
    }
    let banner_area = Rect::new(
        area.x,
        area.y + area.height - banner_h,
        area.width,
        banner_h,
    );

    frame.render_widget(Clear, banner_area);

    let millis = current_millis();
    let progress = surge.progress();
    let pct = (progress * 100.0) as u32;

    // Progress bar as a "time-rift ribbon" with traveling pulse nodes.
    let bar_width = (banner_area.width as usize)
        .saturating_sub(28)
        .clamp(12, 120);
    let filled = ((bar_width as f64) * progress) as usize;
    let mut bar_cells = vec!['┄'; bar_width];
    let fill_len = filled.min(bar_width);
    for (i, cell) in bar_cells.iter_mut().take(fill_len).enumerate() {
        let dist_to_head = fill_len.saturating_sub(i);
        *cell = if dist_to_head <= 2 {
            '▓'
        } else if dist_to_head <= 4 {
            '▒'
        } else {
            '█'
        };
    }
    if bar_width > 0 {
        let pulse_a = ((millis / 55) as usize) % bar_width;
        let pulse_b = (pulse_a + (bar_width / 3).max(1)) % bar_width;
        for pulse in [pulse_a, pulse_b] {
            if pulse < fill_len {
                bar_cells[pulse] = '◉';
            } else {
                bar_cells[pulse] = '◌';
            }
        }
        if fill_len < bar_width {
            bar_cells[fill_len] = '⟫';
        } else {
            bar_cells[bar_width - 1] = '◆';
        }
    }
    let bar = format!(
        "[{}] {:>3}%",
        bar_cells.into_iter().collect::<String>(),
        pct
    );

    let ticks_left_label = chrono_ticks_to_duration_label(surge.ticks_remaining);

    let stats_line = format!(
        "\u{2694}{} kills  \u{2B06}+{} levels  \u{1F528}{} equipped  \u{23F3} {} left",
        surge.kills, surge.levels_gained, surge.items_equipped, ticks_left_label,
    );

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "\u{231B} Chrono Surge",
                Style::default()
                    .fg(TIMEWARP_CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(bar, Style::default().fg(TIMEWARP_CYAN)),
            Span::raw("  "),
            Span::styled(
                format!("\u{26A1}x{}", surge.current_batch_size()),
                Style::default().fg(Color::Rgb(170, 230, 255)),
            ),
            Span::styled("  [Esc] Skip", Style::default().fg(Color::Gray)),
        ]),
        Line::from(Span::styled(
            stats_line,
            Style::default().fg(Color::Rgb(180, 210, 240)),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(TIMEWARP_MID))
            .style(Style::default().bg(TIMEWARP_DEEP)),
    );
    frame.render_widget(paragraph, banner_area);
}

/// Render the Chrono Surge summary modal after completion.
pub fn render_chrono_surge_summary(
    frame: &mut Frame,
    area: Rect,
    summary: &ChronoSurgeSummary,
    _ctx: &super::responsive::LayoutContext,
) {
    let overlay_width = 48u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{231B} Chrono Surge Complete \u{231B} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    // Clear text rows
    for i in 0..h {
        clear_row_chars(&mut buffer, i as i32);
    }

    // Stats
    let stats = [
        format!("\u{2694}  Kills: {}", summary.kills),
        format!("\u{2B06}  Levels gained: +{}", summary.levels_gained),
        format!("\u{1F528} Items equipped: {}", summary.items_equipped),
    ];

    let start_row = 2i32;
    for (i, line) in stats.iter().enumerate() {
        let row = start_row + i as i32;
        if row < h as i32 {
            put_text(&mut buffer, row, 3, line, Color::White);
        }
    }

    // Duration info
    let dur_ticks = summary.ticks_completed;
    let dur_mins = dur_ticks / 600; // 10 ticks/sec * 60 sec/min = 600 ticks/min
    let dur_text = if dur_mins >= 60 {
        format!("Duration: {}h {}m", dur_mins / 60, dur_mins % 60)
    } else {
        format!("Duration: {}m", dur_mins)
    };
    let dur_row = start_row + 5;
    if dur_row < h as i32 {
        put_text(&mut buffer, dur_row, 3, &dur_text, Color::DarkGray);
    }

    // Help at bottom
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[Enter] Continue",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);
}

/// Map sigil grade tier letter to a terminal color.
fn sigil_grade_color(grade: crate::stormglass::sigils::SigilGrade) -> Color {
    match grade.tier_letter() {
        'S' => Color::Rgb(255, 215, 0), // Gold
        'A' => Color::Green,
        'B' => Color::Cyan,
        'C' => Color::White,
        'D' => Color::Gray,
        'E' => Color::DarkGray,
        _ => Color::Red, // F
    }
}

/// Render the sigils list screen with 5 slots.
fn render_sigils_list(
    frame: &mut Frame,
    area: Rect,
    exchange_ui: &ExchangeUiState,
    state: &GameState,
) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 17u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let title = " \u{16B1} Etch Storm Sigils \u{16B1} ";
    let block = Block::default()
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    // Clear text rows
    clear_row_chars(&mut buffer, 1); // flavor
    clear_row_chars(&mut buffer, 2); // blank
    for i in 0..5 {
        clear_row_chars(&mut buffer, 3 + i); // slot rows
    }
    clear_row_chars(&mut buffer, 9); // daily rotation line 1
    clear_row_chars(&mut buffer, 10); // daily rotation line 2
    clear_row_chars(&mut buffer, 11); // rotation countdown
    clear_row_chars(&mut buffer, (h as i32) - 1); // help

    let sigils = &state.storm_sigils;

    // Slot rows (rows 3-7)
    let slot_start_row = 3i32;
    use crate::stormglass::sigils::MAX_SIGIL_SLOTS;

    for i in 0..MAX_SIGIL_SLOTS {
        let row = slot_start_row + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = i == exchange_ui.sigil_selected_slot;

        // Cursor
        let mut col = 1i32;
        if is_selected {
            put_text(&mut buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        let slot_label = format!("Slot {}: ", i + 1);
        put_text(&mut buffer, row, col, &slot_label, Color::DarkGray);
        col += slot_label.len() as i32;

        if i < sigils.slots_unlocked as usize {
            // Unlocked slot
            if let Some(sigil) = &sigils.sigils[i] {
                // Etched sigil: icon + name + value + grade
                let icon = sigil.effect.icon();
                let icon_name = format!("{} {}", icon, sigil.effect.sigil_name());
                put_text(&mut buffer, row, col, &icon_name, Color::White);

                // Right-aligned: value + grade (pad grade to 2 chars for alignment)
                let value_str = sigil.effect.format_value(sigil.value);
                let grade_str = sigil.grade.label();
                let grade_padded = format!("{:<2}", grade_str);
                let right_text = format!("{}  {}", value_str, grade_padded);
                let right_col = (w as i32) - right_text.len() as i32 - 1;

                // Value in white
                put_text(&mut buffer, row, right_col, &value_str, Color::White);

                // Grade with tier color
                let grade_col = (w as i32) - grade_padded.len() as i32 - 1;
                let grade_fg = sigil_grade_color(sigil.grade);
                put_text(&mut buffer, row, grade_col, &grade_padded, grade_fg);
            } else {
                // Empty slot
                put_text(&mut buffer, row, col, "(empty)", Color::DarkGray);
            }
        } else if i == sigils.slots_unlocked as usize {
            // Next unlockable slot
            if let Some(cost) = sigils.next_unlock_cost() {
                let lock_text = format!("\u{1F512} Unlock: {} SG", cost);
                let affordable = state.stormglass >= cost;
                let fg = if affordable {
                    ELECTRIC_BLUE
                } else {
                    Color::DarkGray
                };
                put_text(&mut buffer, row, col, &lock_text, fg);
            } else {
                put_text(&mut buffer, row, col, "\u{1F512}", Color::DarkGray);
            }
        } else {
            // Locked (beyond next)
            put_text(&mut buffer, row, col, "\u{1F512}", Color::DarkGray);
        }

        // Selected row highlight
        if is_selected {
            let highlight_bg = Color::Rgb(15, 25, 55);
            if (row as usize) < h {
                for cell in buffer[row as usize].iter_mut() {
                    cell.bg = highlight_bg;
                }
            }
        }
    }

    // Daily rotation lines (rows 9-11) — show today's available sigils with countdown
    if 11 < h as i32 {
        let pool = crate::stormglass::sigils::daily_sigil_pool();
        let labels: Vec<String> = pool
            .iter()
            .map(|e| format!("{} {}", e.icon(), e.short_name()))
            .collect();
        // Split into two lines: first 3, then remaining
        let line1_parts: Vec<&str> = labels[..3.min(labels.len())]
            .iter()
            .map(|s| s.as_str())
            .collect();
        let line1 = line1_parts.join(" \u{00b7} ");
        put_text_centered(&mut buffer, 9, w, &line1, Color::DarkGray);
        if labels.len() > 3 {
            let line2_parts: Vec<&str> = labels[3..].iter().map(|s| s.as_str()).collect();
            let line2 = line2_parts.join(" \u{00b7} ");
            put_text_centered(&mut buffer, 10, w, &line2, Color::DarkGray);
        }

        // Countdown to next rotation (midnight UTC)
        let now = chrono::Utc::now();
        let tomorrow = (now.date_naive() + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let remaining = tomorrow - now.naive_utc();
        let hours = remaining.num_hours();
        let minutes = remaining.num_minutes() % 60;
        let seconds = remaining.num_seconds() % 60;
        let countdown = format!("\u{29d6} {hours}h {minutes:02}m {seconds:02}s until rotation");
        put_text_centered(&mut buffer, 11, w, &countdown, Color::DarkGray);
    }

    // Help row at bottom
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[\u{2191}\u{2193}] Select  [Enter] Action  [Esc] Back",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    // Flavor text overlay (row 1)
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
        let flavor = Paragraph::new(Span::styled(
            "Sigils of power etched into your soul.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(flavor, flavor_area);
    }
}

/// Render the unlock confirmation dialog.
fn render_sigil_unlock_confirm(frame: &mut Frame, area: Rect, state: &GameState) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Unlock Sigil Slot? \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    clear_row_chars(&mut buffer, 1);
    clear_row_chars(&mut buffer, 2);
    clear_row_chars(&mut buffer, 4);
    clear_row_chars(&mut buffer, 5);
    clear_row_chars(&mut buffer, 6);
    clear_row_chars(&mut buffer, 8);
    clear_row_chars(&mut buffer, (h as i32) - 1);

    let cost = state.storm_sigils.next_unlock_cost().unwrap_or(0);
    let balance = state.stormglass;
    let after = balance.saturating_sub(cost);

    put_text(
        &mut buffer,
        4,
        4,
        &format!("Balance:  {} SG", balance),
        Color::White,
    );
    put_text(
        &mut buffer,
        5,
        4,
        &format!("Cost:    -{} SG", cost),
        Color::LightRed,
    );
    put_text(
        &mut buffer,
        6,
        4,
        &format!("After:    {} SG", after),
        ELECTRIC_BLUE,
    );

    put_text_centered(
        &mut buffer,
        8,
        w,
        "Unlock the next sigil slot?",
        Color::DarkGray,
    );

    let help_row = (h as i32) - 1;
    let help_y_col = 4i32;
    put_text(&mut buffer, help_row, help_y_col, "[", Color::DarkGray);
    put_text(&mut buffer, help_row, help_y_col + 1, "Y", Color::Green);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 2,
        "] Unlock  [",
        Color::DarkGray,
    );
    put_text(&mut buffer, help_row, help_y_col + 13, "N", Color::LightRed);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 14,
        "] Cancel",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    // Flavor text
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "A new sigil slot awaits within the storm.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }
}

/// Render the etch confirmation dialog.
fn render_sigil_etch_confirm(frame: &mut Frame, area: Rect, state: &GameState) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 16u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Etch Sigil? \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    clear_row_chars(&mut buffer, 1);
    clear_row_chars(&mut buffer, 2);
    clear_row_chars(&mut buffer, 4);
    clear_row_chars(&mut buffer, 5);
    clear_row_chars(&mut buffer, 6);
    clear_row_chars(&mut buffer, 8);
    clear_row_chars(&mut buffer, 10);
    clear_row_chars(&mut buffer, 11);
    clear_row_chars(&mut buffer, (h as i32) - 1);

    let cost = crate::stormglass::sigils::ETCH_COST;
    let balance = state.stormglass;
    let after = balance.saturating_sub(cost);

    put_text(
        &mut buffer,
        4,
        4,
        &format!("Balance:  {} SG", balance),
        Color::White,
    );
    put_text(
        &mut buffer,
        5,
        4,
        &format!("Cost:    -{} SG", cost),
        Color::LightRed,
    );
    put_text(
        &mut buffer,
        6,
        4,
        &format!("After:    {} SG", after),
        ELECTRIC_BLUE,
    );

    put_text_centered(
        &mut buffer,
        8,
        w,
        "You will choose from today's pool:",
        Color::DarkGray,
    );

    // Daily sigil pool names with icons (rows 10-11, with padding rows 9 and 12)
    let pool = crate::stormglass::sigils::daily_sigil_pool();
    let names: Vec<String> = pool
        .iter()
        .map(|e| format!("{} {}", e.icon(), e.short_name()))
        .collect();
    let line1_names = &names[..3.min(names.len())];
    let line1 = line1_names.join("  ");
    put_text_centered(&mut buffer, 10, w, &line1, Color::DarkGray);
    if names.len() > 3 {
        let line2_names = &names[3..];
        let line2 = line2_names.join("  ");
        put_text_centered(&mut buffer, 11, w, &line2, Color::DarkGray);
    }

    let help_row = (h as i32) - 1;
    let help_y_col = 4i32;
    put_text(&mut buffer, help_row, help_y_col, "[", Color::DarkGray);
    put_text(&mut buffer, help_row, help_y_col + 1, "Y", Color::Green);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 2,
        "] Etch  [",
        Color::DarkGray,
    );
    put_text(&mut buffer, help_row, help_y_col + 9, "N", Color::LightRed);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 10,
        "] Cancel",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "The storm fractures. Three sigils will emerge.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }
}

/// Render the reroll confirmation dialog.
fn render_sigil_reroll_confirm(
    frame: &mut Frame,
    area: Rect,
    exchange_ui: &ExchangeUiState,
    state: &GameState,
) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Reroll Sigil? \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    clear_row_chars(&mut buffer, 1);
    clear_row_chars(&mut buffer, 2);
    clear_row_chars(&mut buffer, 4);
    clear_row_chars(&mut buffer, 5);
    clear_row_chars(&mut buffer, 6);
    clear_row_chars(&mut buffer, 8);
    clear_row_chars(&mut buffer, 9);
    clear_row_chars(&mut buffer, (h as i32) - 1);

    let cost = crate::stormglass::sigils::ETCH_COST;
    let balance = state.stormglass;
    let after = balance.saturating_sub(cost);

    put_text(
        &mut buffer,
        4,
        4,
        &format!("Balance:  {} SG", balance),
        Color::White,
    );
    put_text(
        &mut buffer,
        5,
        4,
        &format!("Cost:    -{} SG", cost),
        Color::LightRed,
    );
    put_text(
        &mut buffer,
        6,
        4,
        &format!("After:    {} SG", after),
        ELECTRIC_BLUE,
    );

    // Show current sigil being destroyed
    let slot = exchange_ui.sigil_target_slot;
    if let Some(sigil) = &state.storm_sigils.sigils[slot] {
        let current_text = format!(
            "Destroying: {} {} {}",
            sigil.effect.icon(),
            sigil.effect.sigil_name(),
            sigil.effect.format_value(sigil.value)
        );
        put_text_centered(&mut buffer, 8, w, &current_text, Color::LightRed);
    }
    put_text_centered(
        &mut buffer,
        9,
        w,
        "Current sigil will be lost forever.",
        Color::DarkGray,
    );

    let help_row = (h as i32) - 1;
    let help_y_col = 4i32;
    put_text(&mut buffer, help_row, help_y_col, "[", Color::DarkGray);
    put_text(&mut buffer, help_row, help_y_col + 1, "Y", Color::Green);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 2,
        "] Reroll  [",
        Color::DarkGray,
    );
    put_text(&mut buffer, help_row, help_y_col + 13, "N", Color::LightRed);
    put_text(
        &mut buffer,
        help_row,
        help_y_col + 14,
        "] Cancel",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "The old sigil shatters as the storm reshapes.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }
}

/// Render the sigil rolling animation (4 phases over 4 seconds).
fn render_sigil_rolling(frame: &mut Frame, area: Rect, exchange_ui: &ExchangeUiState) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Sigil Inscription \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let millis = current_millis();
    let anim_start = exchange_ui.sigil_animation_start_ms.unwrap_or(millis);
    let elapsed = millis.saturating_sub(anim_start);

    // Intensified storm backdrop: more particles and faster speed during phase 1
    let intensity = if elapsed < PHASE1_END_MS {
        (elapsed as f64 / PHASE1_END_MS as f64).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let params = StormBackdropParams {
        top_rgb: (
            lerp_rgb((10, 15, 40), (20, 10, 50), intensity).0,
            lerp_rgb((10, 15, 40), (20, 10, 50), intensity).1,
            lerp_rgb((10, 15, 40), (20, 10, 50), intensity).2,
        ),
        bottom_rgb: (
            lerp_rgb((5, 5, 15), (10, 5, 30), intensity).0,
            lerp_rgb((5, 5, 15), (10, 5, 30), intensity).1,
            lerp_rgb((5, 5, 15), (10, 5, 30), intensity).2,
        ),
        particle_count: 8 + (intensity * 12.0) as usize,
        particle_speed: 1.5 + intensity * 2.5,
        shimmer: true,
    };

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    paint_storm_backdrop(&mut buffer, millis, &params);

    if elapsed < PHASE1_END_MS {
        render_rolling_phase1(&mut buffer, w, h, millis, elapsed);
    } else if elapsed < PHASE2_END_MS {
        let phase_elapsed = elapsed - PHASE1_END_MS;
        render_rolling_phase2(&mut buffer, w, h, exchange_ui, phase_elapsed);
    } else if elapsed < PHASE3_END_MS {
        let phase_elapsed = elapsed - PHASE2_END_MS;
        render_rolling_phase3(&mut buffer, w, h, exchange_ui, phase_elapsed);
    } else {
        render_rolling_phase4(&mut buffer, w, h, exchange_ui);
    }

    // Help row: "[Any key] Skip" after 200ms
    if elapsed > 200 {
        let help_row = (h as i32) - 1;
        clear_row_chars(&mut buffer, help_row);
        put_text_centered(&mut buffer, help_row, w, "[Any key] Skip", Color::DarkGray);
    }

    render_buffer(frame, inner, &buffer);
}

/// Phase 1: Energy Gathering (0-1400ms) — rune circle orbiting center, pulsing text.
fn render_rolling_phase1(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    millis: u128,
    elapsed: u128,
) {
    let progress = (elapsed as f64 / PHASE1_END_MS as f64).clamp(0.0, 1.0);

    // Center of the buffer
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    // Clear center rows for text
    for r in 0..h {
        clear_row_chars(buffer, r as i32);
    }

    // Orbiting rune circle characters with progressive detail
    let circle_chars: &[char] = if progress < 0.25 {
        &['\u{00b7}'] // ·
    } else if progress < 0.5 {
        &['\u{00b7}', '\u{25e6}'] // · ◦
    } else if progress < 0.75 {
        &['\u{25e6}', '\u{25cb}'] // ◦ ○
    } else {
        &['\u{25cb}', '\u{25ce}'] // ○ ◎
    };

    let radius = 3.0 + progress * 2.0;
    let orbit_speed = millis as f64 / 400.0;
    let num_runes = 8 + (progress * 4.0) as usize;

    for i in 0..num_runes {
        let angle = orbit_speed + (i as f64 * std::f64::consts::TAU / num_runes as f64);
        // Converge inward as progress increases
        let r = radius * (1.0 - progress * 0.3);
        let rx = cx + angle.cos() * r * 2.0; // x stretched for terminal aspect ratio
        let ry = cy + angle.sin() * r;
        let ch = circle_chars[i % circle_chars.len()];

        let fade = 0.4 + 0.6 * progress;
        let rgb = lerp_rgb((80, 60, 160), (180, 160, 255), fade);
        put_cell(
            buffer,
            ry as i32,
            rx as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }

    // Orbiting glyphs converging inward
    let glyphs = ['\u{26A1}', '\u{2726}', '\u{25C8}']; // ⚡ ✦ ◈
    for (i, &glyph) in glyphs.iter().enumerate() {
        let angle = orbit_speed * 0.7 + (i as f64 * std::f64::consts::TAU / 3.0);
        let glyph_r = (5.0 - progress * 3.5).max(1.0);
        let gx = cx + angle.cos() * glyph_r * 2.0;
        let gy = cy + angle.sin() * glyph_r;
        put_cell(buffer, gy as i32, gx as i32, glyph, ELECTRIC_BLUE);
    }

    // Center text: "Gathering energy..." with pulsing purple
    let pulse = ((millis as f64 / 300.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let pulse_rgb = lerp_rgb((120, 100, 200), (220, 200, 255), pulse);
    let pulse_fg = Color::Rgb(pulse_rgb.0, pulse_rgb.1, pulse_rgb.2);

    let text = "Gathering energy...";
    let center_row = cy as i32;
    put_text_centered(buffer, center_row, w, text, pulse_fg);
}

/// Phase 2: Sigil Etching (1400-2800ms) — sigil names etch character-by-character.
/// Uses icon + sigil name format matching the slots screen.
fn render_rolling_phase2(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
    phase_elapsed: u128,
) {
    // Clear all rows for clean rendering
    for r in 0..h {
        clear_row_chars(buffer, r as i32);
    }

    // Header
    put_text_centered(buffer, 1, w, "Inscribing sigils...", RUNE_PURPLE);

    let choice_start_row = 4i32;
    let stagger_ms: u128 = 150;
    let phase_duration = PHASE2_END_MS - PHASE1_END_MS; // 1400ms

    for (i, choice) in exchange_ui.sigil_choices.iter().enumerate() {
        let row = choice_start_row + i as i32;
        if row >= h as i32 {
            break;
        }

        if let Some(sigil) = choice {
            // Match slots screen format: icon + sigil name
            let icon = sigil.effect.icon();
            let display_str = format!("{} {}", icon, sigil.effect.sigil_name());
            let total_chars = display_str.chars().count();
            let sigil_start = stagger_ms * i as u128;

            if phase_elapsed < sigil_start {
                // Not started yet
                continue;
            }

            let sigil_elapsed = phase_elapsed - sigil_start;
            // Time per character: distribute across available time
            let char_duration = (phase_duration - stagger_ms * 2) / total_chars.max(1) as u128;
            let chars_visible = if char_duration == 0 {
                total_chars
            } else {
                ((sigil_elapsed / char_duration) as usize).min(total_chars)
            };

            let partial: String = display_str.chars().take(chars_visible).collect();
            // Match slots col: cursor marker (2) + start at col 3
            let col = 3i32;
            put_text(buffer, row, col, &partial, Color::White);

            // Typing cursor at end of partial text (use display width for position)
            if chars_visible < total_chars {
                let cursor_col = col + display_width(&partial) as i32;
                put_cell(buffer, row, cursor_col, '\u{2588}', Color::White); // █
            }

            // Brief inscription flash when a sigil completes
            let complete = chars_visible >= total_chars;
            if complete {
                let flash_window = 120; // ms
                let complete_time = sigil_start + char_duration * total_chars as u128;
                if phase_elapsed < complete_time + flash_window {
                    // Flash the row
                    if (row as usize) < h {
                        for cell in buffer[row as usize].iter_mut() {
                            if cell.ch != ' ' {
                                cell.fg = INSCRIPTION_FLASH;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Phase 3: Grade Reveal (2800-3500ms) — full names shown, value + grade fade in right-aligned.
/// Matches slots screen layout: icon + sigil name left, value + grade right.
fn render_rolling_phase3(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
    phase_elapsed: u128,
) {
    for r in 0..h {
        clear_row_chars(buffer, r as i32);
    }

    put_text_centered(buffer, 1, w, "Sigils revealed!", RUNE_PURPLE);

    let choice_start_row = 4i32;
    let stagger_ms: u128 = 150;
    let phase_duration = PHASE3_END_MS - PHASE2_END_MS; // 700ms

    for (i, choice) in exchange_ui.sigil_choices.iter().enumerate() {
        let row = choice_start_row + i as i32;
        if row >= h as i32 {
            break;
        }

        if let Some(sigil) = choice {
            // Match slots screen: icon + sigil name on left
            let icon = sigil.effect.icon();
            let name_str = format!("{} {}", icon, sigil.effect.sigil_name());
            let col = 3i32;
            put_text(buffer, row, col, &name_str, Color::White);

            // Right-aligned value + grade fade in with stagger
            let grade_start = stagger_ms * i as u128;
            if phase_elapsed >= grade_start {
                let grade_elapsed = phase_elapsed - grade_start;
                let fade_duration = (phase_duration - stagger_ms * 2).max(1);
                let fade = (grade_elapsed as f64 / fade_duration as f64).clamp(0.0, 1.0);

                let value_str = sigil.effect.format_value(sigil.value);
                let grade_str = sigil.grade.label();
                let grade_fg = sigil_grade_color(sigil.grade);

                // Value fades in white
                let value_rgb = lerp_rgb((60, 60, 60), (200, 200, 200), fade);
                let value_fg = Color::Rgb(value_rgb.0, value_rgb.1, value_rgb.2);

                // Grade lerps from dark gray to final color
                let base_rgb = (60, 60, 60);
                let target_rgb = match grade_fg {
                    Color::Rgb(r, g, b) => (r, g, b),
                    Color::Green => (0, 200, 0),
                    Color::Cyan => (0, 200, 200),
                    Color::White => (200, 200, 200),
                    Color::Gray => (128, 128, 128),
                    Color::DarkGray => (80, 80, 80),
                    Color::Red => (200, 0, 0),
                    _ => (200, 200, 200),
                };
                let faded_rgb = lerp_rgb(base_rgb, target_rgb, fade);
                let faded_fg = Color::Rgb(faded_rgb.0, faded_rgb.1, faded_rgb.2);

                // Right-aligned: "value  grade" (pad grade to 2 chars)
                let grade_padded = format!("{:<2}", grade_str);
                let right_text = format!("{}  {}", value_str, grade_padded);
                let right_col = (w as i32) - right_text.len() as i32 - 1;
                put_text(buffer, row, right_col, &value_str, value_fg);
                let grade_col = (w as i32) - grade_padded.len() as i32 - 1;
                put_text(buffer, row, grade_col, &grade_padded, faded_fg);
            }
        }
    }
}

/// Phase 4: Transition (3500-4000ms) — everything solidified, cursor on first choice.
/// Renders identically to the pick screen for a seamless transition.
fn render_rolling_phase4(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
) {
    for r in 0..h {
        clear_row_chars(buffer, r as i32);
    }

    put_text_centered(
        buffer,
        1,
        w,
        "The storm fractures. Three sigils emerge.",
        RUNE_PURPLE,
    );

    render_sigil_choice_rows(buffer, w, h, exchange_ui, 0);
}

/// Shared helper: render 3 sigil choice rows matching the slots screen layout.
/// Icon + sigil name on the left, value + grade right-aligned.
/// `selected` is the currently highlighted row index.
fn render_sigil_choice_rows(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
    selected: usize,
) {
    let choice_start_row = 4i32;

    for (i, choice) in exchange_ui.sigil_choices.iter().enumerate() {
        let row = choice_start_row + i as i32;
        if row >= h as i32 {
            break;
        }

        if let Some(sigil) = choice {
            let is_selected = i == selected;

            // Cursor
            let col = 1i32;
            if is_selected {
                put_text(buffer, row, col, "> ", Color::Yellow);
            }
            let name_col = col + 2;

            // Icon + sigil name (left side)
            let icon = sigil.effect.icon();
            let name_str = format!("{} {}", icon, sigil.effect.sigil_name());
            put_text(buffer, row, name_col, &name_str, Color::White);

            // Right-aligned: value + grade (matching slots screen)
            // Pad grade to 2 chars so the letter column stays aligned
            // ("A" → "A ", "S+" → "S+", "B-" → "B-")
            let value_str = sigil.effect.format_value(sigil.value);
            let grade_str = sigil.grade.label();
            let grade_padded = format!("{:<2}", grade_str);
            let right_text = format!("{}  {}", value_str, grade_padded);
            let right_col = (w as i32) - right_text.len() as i32 - 1;

            // Value in white
            put_text(buffer, row, right_col, &value_str, Color::White);

            // Grade with tier color (at fixed 2-char-wide position)
            let grade_col = (w as i32) - grade_padded.len() as i32 - 1;
            let grade_fg = sigil_grade_color(sigil.grade);
            put_text(buffer, row, grade_col, &grade_padded, grade_fg);

            // Selected row highlight
            if is_selected && (row as usize) < h {
                let highlight_bg = Color::Rgb(15, 25, 55);
                for cell in buffer[row as usize].iter_mut() {
                    cell.bg = highlight_bg;
                }
            }
        }
    }
}

/// Render the pick-1-of-3 sigil selection screen.
fn render_sigil_pick(frame: &mut Frame, area: Rect, exchange_ui: &ExchangeUiState) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Choose a Sigil \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    clear_row_chars(&mut buffer, 1);
    clear_row_chars(&mut buffer, 2);
    for i in 0..3 {
        clear_row_chars(&mut buffer, 4 + i);
    }
    clear_row_chars(&mut buffer, (h as i32) - 1);

    // 3 sigil choices — uses shared helper matching slots screen layout
    render_sigil_choice_rows(
        &mut buffer,
        w,
        h,
        exchange_ui,
        exchange_ui.sigil_pick_selected,
    );

    // Help row
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[\u{2191}\u{2193}] Select  [Enter] Etch  [Esc] Forfeit",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);

    // Flavor text
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let flavor_rgb = lerp_rgb((100, 140, 200), (150, 200, 255), pulse_t);
    let flavor_fg = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    if h > 2 {
        let flavor_area = Rect::new(inner.x, inner.y + 1, inner.width, 2);
        let flavor = Paragraph::new(Span::styled(
            "The storm fractures. Three sigils emerge.",
            Style::default()
                .fg(flavor_fg)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(flavor, flavor_area);
    }
}

/// Render the forfeit confirmation overlay (shown on top of pick screen).
fn render_sigil_forfeit_confirm(frame: &mut Frame, area: Rect) {
    let modal_width = 42u16.min(area.width.saturating_sub(4));
    let modal_height = 8u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " Abandon Sigil? ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "25,000 SG already spent.",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "The Stormglass cannot be reclaimed.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("] Leave  [", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("] Stay", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the sigil result screen after inscribing.
fn render_sigil_result(frame: &mut Frame, area: Rect, exchange_ui: &ExchangeUiState) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 14u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Sigil Etched! \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_storm_backdrop(&mut buffer, millis, &StormBackdropParams::normal());

    for i in 0..h {
        clear_row_chars(&mut buffer, i as i32);
    }

    if let Some(sigil) = &exchange_ui.sigil_result {
        // Sigil icon + name centered
        let name = format!("{} {}", sigil.effect.icon(), sigil.effect.sigil_name());
        let grade_fg = sigil_grade_color(sigil.grade);
        put_text_centered(&mut buffer, 3, w, &name, grade_fg);

        // Value centered
        let value_str = sigil.effect.format_value(sigil.value);
        put_text_centered(&mut buffer, 5, w, &value_str, Color::White);

        // Grade centered with color
        let grade_str = sigil.grade.label();
        let grade_col = ((w as i32) - grade_str.len() as i32) / 2;
        for (j, ch) in grade_str.chars().enumerate() {
            let col = grade_col + j as i32;
            if col >= 0 && (col as usize) < w {
                let cell = &mut buffer[7][col as usize];
                cell.ch = ch;
                cell.fg = grade_fg;
            }
        }

        // Range info
        let (min, max) = sigil.effect.range();
        let range_str = format!("Range: {:.0}-{:.0}%", min, max);
        put_text_centered(&mut buffer, 9, w, &range_str, Color::DarkGray);
    }

    // Help
    let help_row = (h as i32) - 1;
    put_text_centered(
        &mut buffer,
        help_row,
        w,
        "[Enter] Continue",
        Color::DarkGray,
    );

    render_buffer(frame, inner, &buffer);
}

/// Render the Stormglass discovery modal.
pub fn render_stormglass_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
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
            "\u{1F48E} Your unwanted gear crystallized into Stormglass!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Salvaged gear now generates Stormglass.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "Spend it at the Stormglass Exchange.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [G] to visit. [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
