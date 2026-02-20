//! Stormglass Exchange UI rendering: storm-themed modal overlay.

use crate::core::game_state::GameState;
use crate::stormglass::spending::prestige_rank_cost;
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
    current_millis, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};

/// Electric blue color used throughout Stormglass UI.
const ELECTRIC_BLUE: Color = Color::Rgb(100, 180, 255);

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

/// Write a string centered horizontally in the buffer.
fn put_text_centered(buffer: &mut [Vec<SceneCell>], row: i32, width: usize, text: &str, fg: Color) {
    let col = (width as i32 - text.chars().count() as i32) / 2;
    put_text(buffer, row, col, text, fg);
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
        ExchangePhase::InvokeTrial => render_invoke_trial(frame, area, exchange_ui),
        ExchangePhase::ChronoSurge => render_chrono_surge_select(frame, area, exchange_ui, state),
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

    let title = format!(
        " \u{26A1} Stormglass Exchange  [\u{26A1}{} SG] ",
        state.stormglass
    );
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

    // Menu items (rows 4-7)
    let pr_cost = prestige_rank_cost(state.prestige_rank);
    let pr_label = format!(
        "Prestige P{} \u{2192} P{}",
        state.prestige_rank,
        state.prestige_rank + 1
    );

    let items: [(String, String, bool); 3] = [
        (
            "Invoke Trial".to_string(),
            format!("{} SG", INVOKE_TRIAL_COST),
            state.stormglass >= INVOKE_TRIAL_COST,
        ),
        (
            pr_label,
            format!("{} SG", pr_cost),
            state.stormglass >= pr_cost,
        ),
        ("Chrono Surge".to_string(), ">>>".to_string(), true),
    ];

    let menu_start_row = 4i32;
    for (i, (name, cost, affordable)) in items.iter().enumerate() {
        let row = menu_start_row + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = i == exchange_ui.selected_item;

        let mut col = 0i32;

        // Cursor
        if is_selected {
            put_text(&mut buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        // Item name
        let name_fg = if *affordable {
            Color::White
        } else {
            Color::DarkGray
        };
        put_text(&mut buffer, row, col, name, name_fg);

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
            0 => "Spend Stormglass to unlock a choice of three challenges.",
            1 => "Spend Stormglass to advance your prestige rank, keeping all progress.",
            _ => "Bend time itself. Earn XP and loot, but no Stormglass.",
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
            " \u{26A1} Invoke Trial \u{26A1} ",
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
    let trial_start_row = 4i32;
    for (i, trial) in exchange_ui.trial_options.iter().enumerate() {
        let row = trial_start_row + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = i == exchange_ui.trial_selected;

        let mut col = 1i32;

        if is_selected {
            put_text(&mut buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        put_text(&mut buffer, row, col, &trial.display_name, Color::White);

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

    let title = format!(" \u{26A1} Chrono Surge  [\u{26A1}{} SG] ", state.stormglass);
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

    let progress = surge.progress();
    let pct = (progress * 100.0) as u32;

    // Progress bar
    let bar_width = banner_area.width.saturating_sub(4) as usize;
    let filled = ((bar_width as f64) * progress) as usize;
    let bar: String = format!(
        "[{}{}] {}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(bar_width.saturating_sub(filled)),
        pct,
    );

    let stats_line = format!(
        "\u{2694}{} kills  \u{2B06}+{} levels  \u{1F528}{} equipped",
        surge.kills, surge.levels_gained, surge.items_equipped,
    );

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "\u{26A1} Chrono Surge ",
                Style::default()
                    .fg(ELECTRIC_BLUE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar, Style::default().fg(ELECTRIC_BLUE)),
            Span::styled("  [Esc] Skip", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(Span::styled(
            stats_line,
            Style::default().fg(Color::Rgb(180, 200, 230)),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(ELECTRIC_BLUE)),
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
            " \u{26A1} Chrono Surge Complete \u{26A1} ",
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
        format!("\u{1F528}  Items equipped: {}", summary.items_equipped),
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
            "\u{26A1} Your unwanted gear crystallized into Stormglass!",
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
