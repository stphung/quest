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
use super::scene_fx::{
    current_millis, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};

/// Themed border color for The Deep overlay.
pub(super) const DEEP_BORDER_COLOR: Color = Color::Rgb(80, 160, 220);

// ── Backdrop ──────────────────────────────────────────────────────────────────

/// Paint the deep cave backdrop: dark blue gradient, stalactite ceiling,
/// drifting dust particles, bioluminescent glow patches, and water shimmer.
/// Visual parameters vary subtly based on the active view.
#[allow(clippy::needless_range_loop)]
pub(super) fn paint_deep_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128, view: DeepView) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // View-specific gradient tints: Infrastructure is darker/deeper,
    // EventResponse has a warmer tension tint, others use the default cave blue.
    let (top_rgb, bottom_rgb) = match view {
        DeepView::Infrastructure => ((4u8, 6u8, 16u8), (1u8, 2u8, 6u8)),
        DeepView::EventResponse => ((8u8, 6u8, 18u8), (4u8, 2u8, 8u8)),
        _ => ((5u8, 8u8, 20u8), (2u8, 3u8, 8u8)),
    };
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

    // 2. Stalactite ceiling features (subtle rock formations at rows 0-2)
    let stalactite_chars: &[char] = &['\u{2502}', '\u{2551}', '\u{00a6}']; // │ ║ ¦
    let stalactite_count = width / 8; // one every ~8 columns
    for i in 0..stalactite_count {
        let seed = hash2d(i, 41);
        let col = ((seed as usize) * 7 + i * 8) % width;
        let hang_len = 1 + (hash2d(i, 43) as usize % 3); // 1-3 rows
        let ch = stalactite_chars[(hash2d(i, 47) as usize) % stalactite_chars.len()];
        for r in 0..hang_len.min(height) {
            let t = r as f64 / 3.0;
            let rgb = lerp_rgb((18, 24, 40), (8, 12, 25), t);
            put_cell(
                buffer,
                r as i32,
                col as i32,
                ch,
                Color::Rgb(rgb.0, rgb.1, rgb.2),
            );
        }
    }

    // 3. Drifting dust particles (cave air) — rising slowly
    //    View-specific: Infrastructure has more particles (busy underground),
    //    EventResponse has faster, warmer particles (tension).
    let particle_chars: &[char] = &['\u{00b7}', '\u{2022}', '\u{2218}']; // · • ∘
    let (particle_count, particle_speed, particle_hot, particle_cool) = match view {
        DeepView::Infrastructure => (16, 1.2, (40u8, 60u8, 110u8), (15u8, 20u8, 45u8)),
        DeepView::EventResponse => (14, 2.5, (80u8, 60u8, 120u8), (30u8, 20u8, 50u8)),
        _ => (12, 1.5, (60u8, 80u8, 140u8), (20u8, 30u8, 60u8)),
    };

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

    // 4. Slow-falling water droplets (descending, sparse)
    let drop_chars: &[char] = &[',', '\''];
    let drop_count = 4;
    let drop_speed = 3.0;
    for i in 0..drop_count {
        let seed = hash2d(i, 53);
        let col = (seed as usize) % width.max(1);
        let ch = drop_chars[(hash2d(i, 59) as usize) % drop_chars.len()];

        let phase_offset = (seed as f64) * 0.47;
        let pos = (phase_offset + millis as f64 * drop_speed / 1000.0) % height as f64;
        let row = pos as i32;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb((40, 70, 120), (15, 25, 50), t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 5. Bioluminescent glow patches (faint pulsing spots — cave crystals/fungi)
    let glow_count = 5;
    for i in 0..glow_count {
        let seed = hash2d(i, 71);
        let col = (seed as usize) % width.max(1);
        let glow_row = (hash2d(i, 73) as usize) % height.max(1);

        // Slow independent pulse per patch
        let pulse_phase = millis as f64 / 2000.0 + (seed as f64) * 0.61;
        let pulse = (pulse_phase.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let brightness = 0.3 + pulse * 0.7;

        if glow_row < height && col < width {
            // Tint the background cell with a faint cyan-green glow
            if let Color::Rgb(r, g, b) = buffer[glow_row][col].bg {
                let glow_target = (10, 40, 50);
                let mixed = lerp_rgb((r, g, b), glow_target, brightness * 0.35);
                buffer[glow_row][col].bg = Color::Rgb(mixed.0, mixed.1, mixed.2);
            }
            // Render a faint glow character
            let glow_char = if pulse > 0.6 { '*' } else { '\u{00b7}' };
            let gc = lerp_rgb((15, 50, 60), (30, 100, 120), brightness);
            put_cell(
                buffer,
                glow_row as i32,
                col as i32,
                glow_char,
                Color::Rgb(gc.0, gc.1, gc.2),
            );
        }
    }

    // 6. Faint water-reflection shimmer (very subtle bg undulation in lower third)
    let shimmer_phase = millis as f64 / 200.0;
    let shimmer_start_row = (height * 2) / 3;
    for row in shimmer_start_row..height {
        for col in 0..width {
            if hash2d(row, col).is_multiple_of(11) {
                let shift =
                    ((shimmer_phase + row as f64 * 0.4 + col as f64 * 0.15).sin() * 4.0) as i16;
                if let Color::Rgb(r, g, b) = buffer[row][col].bg {
                    let new_b = (b as i16 + shift).clamp(0, 255) as u8;
                    buffer[row][col].bg = Color::Rgb(r, g, new_b);
                }
            }
        }
    }
}

/// Opening flourish: blue-white sheen sweeping top-to-bottom on overlay open (700ms),
/// plus a burst of cave debris particles that quickly settle.
pub(super) fn paint_opening_deep_fx(
    buffer: &mut [Vec<SceneCell>],
    millis: u128,
    open_elapsed_ms: u128,
) {
    const OPEN_FX_MS: f64 = 700.0;
    if open_elapsed_ms as f64 >= OPEN_FX_MS {
        return;
    }

    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = if buffer[0].is_empty() {
        return;
    } else {
        buffer[0].len()
    };

    let progress = (open_elapsed_ms as f64 / OPEN_FX_MS).clamp(0.0, 1.0);
    let strength = 1.0 - progress;

    // Cool blue-white sheen sweeping from top
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

    // Cave debris burst — particles that settle quickly (like disturbed cave dust)
    let debris_count = (2.0 + strength * 14.0) as usize;
    let debris_chars: &[char] = &['\u{00b7}', '\u{2022}', ':'];
    for i in 0..debris_count {
        let seed = hash2d(i, 83);
        let col = (seed as usize) % width;
        let phase = (seed as f64) * 0.31;
        let fall = (phase + millis as f64 * 0.009) % (height as f64 + 4.0);
        let row = fall.floor() as i32;
        if row < 0 || row >= height as i32 {
            continue;
        }

        let t = (fall / height.max(1) as f64).clamp(0.0, 1.0);
        let rgb = lerp_rgb((120, 170, 220), (30, 50, 80), t);
        let ch = debris_chars[(hash2d(i, 89) as usize) % debris_chars.len()];
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }
}

// ── Tab bar ──────────────────────────────────────────────────────────────────

/// Render the tab bar at row 0 of the buffer with state badges.
fn render_tab_bar(buffer: &mut [Vec<SceneCell>], width: usize, active: DeepView, deep: &DeepState) {
    let mut col = 1i32;
    let mut active_tab_start = 0i32;
    let mut active_tab_len = 0i32;
    for (i, &tab) in DeepView::TABS.iter().enumerate() {
        if i > 0 {
            put_text(buffer, 0, col, " ", Color::DarkGray);
            col += 1;
        }

        // Compute badge for this tab
        let (badge, badge_color) = match tab {
            DeepView::Hub => {
                let events = deep
                    .prestige
                    .active_missions
                    .iter()
                    .filter(|m| m.has_pending_event())
                    .count();
                let results = deep.prestige.pending_results.len();
                if events > 0 {
                    (format!("\u{26a1}{}", events), Color::Yellow)
                } else if results > 0 {
                    (format!("\u{2713}{}", results), Color::Green)
                } else {
                    (String::new(), Color::DarkGray)
                }
            }
            DeepView::NewMission => {
                let n = deep.prestige.available_missions.len();
                if n > 0 {
                    (format!("\u{00b7}{}", n), Color::Cyan)
                } else {
                    (String::new(), Color::DarkGray)
                }
            }
            DeepView::Roster => {
                let injured = deep
                    .prestige
                    .roster
                    .iter()
                    .filter(|m| {
                        matches!(
                            m.status,
                            crate::deep::MercStatus::Injured { .. } | crate::deep::MercStatus::Lost
                        )
                    })
                    .count();
                if injured > 0 {
                    (format!("!{}", injured), Color::Yellow)
                } else {
                    (String::new(), Color::DarkGray)
                }
            }
            DeepView::Recruit => {
                let n = deep.prestige.recruit_pool.candidates.len();
                if n > 0 {
                    ("\u{00b7}".to_string(), Color::Cyan)
                } else {
                    (String::new(), Color::DarkGray)
                }
            }
            _ => (String::new(), Color::DarkGray),
        };

        let label = tab.tab_label();
        let full = if badge.is_empty() {
            format!("[{}]", label)
        } else {
            format!("[{}{}]", label, badge)
        };

        let is_active = tab == active;
        let tab_color = if is_active {
            Color::Rgb(80, 160, 220)
        } else {
            Color::DarkGray
        };

        // Active tab gets a subtle highlighted background
        if is_active {
            let tab_len = full.len();
            for c in col..(col + tab_len as i32) {
                if c >= 0 && (c as usize) < width && !buffer.is_empty() {
                    buffer[0][c as usize].bg = Color::Rgb(15, 25, 45);
                }
            }
        }

        put_text(buffer, 0, col, &full, tab_color);

        // Overcolor badge portion when tab is not active
        if !badge.is_empty() && !is_active {
            let badge_col = col + 1 + label.len() as i32;
            put_text(buffer, 0, badge_col, &badge, badge_color);
        }

        // Track active tab position for underline
        if is_active {
            active_tab_start = col;
            active_tab_len = full.len() as i32;
        }
        col += full.len() as i32;
    }

    // Separator below tabs with active tab underline highlight
    let sep_color = Color::Rgb(40, 60, 80);
    let active_underline_color = Color::Rgb(80, 160, 220);
    for c in 1i32..(width as i32 - 1) {
        let ch = '\u{2500}'; // ─
        let color = if c >= active_tab_start && c < active_tab_start + active_tab_len {
            active_underline_color
        } else {
            sep_color
        };
        // Render separator on row 1 (below tab labels on row 0)
        if buffer.len() > 1 {
            put_cell(buffer, 1, c, ch, color);
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
    let inner = super::render_themed_block(
        frame,
        area,
        block,
        DEEP_BORDER_COLOR,
        super::BorderFxContext,
    );

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Build scene buffer and paint backdrop
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_deep_backdrop(&mut buffer, millis, ui.view);
    if let Some(elapsed) = open_elapsed_ms {
        paint_opening_deep_fx(&mut buffer, millis, elapsed);
    }

    // ── Tab bar (row 0) ──
    render_tab_bar(&mut buffer, width, ui.view, deep);

    // Sub-views render below the tab bar (row 0) and separator (row 1).
    // We pass a reduced height and offset the buffer slice by 2 rows.
    let content_height = height.saturating_sub(2);
    let content_buffer = &mut buffer[2..];

    // Dispatch to the appropriate sub-view
    match ui.view {
        DeepView::Hub => {
            super::deep_missions::render_hub(content_buffer, width, content_height, deep, ui, ctx);
        }
        DeepView::NewMission => {
            super::deep_missions::render_new_mission(
                content_buffer,
                width,
                content_height,
                deep,
                ui,
                ctx,
            );
        }
        DeepView::Roster => {
            super::deep_roster::render_roster(content_buffer, width, content_height, deep, ui, ctx);
        }
        DeepView::Recruit => {
            super::deep_roster::render_recruit(
                content_buffer,
                width,
                content_height,
                deep,
                ui,
                ctx,
            );
        }
        DeepView::Infrastructure => {
            super::deep_layers::render_layers(content_buffer, width, content_height, deep, ui, ctx);
        }
        DeepView::EventResponse => {
            super::deep_events::render_event_response(
                content_buffer,
                width,
                content_height,
                deep,
                ui,
                ctx,
            );
        }
    }

    // Help panel overlay ([?] toggle)
    if ui.show_help {
        render_help_panel(&mut buffer, width, height, ui.view, ctx);
    }

    // Flush buffer to frame
    render_buffer(frame, inner, &buffer);

    // Mission results modal is layered on top if there are pending results
    if !deep.prestige.pending_results.is_empty() {
        if let Some(mission) = deep.prestige.pending_results.first() {
            super::deep_results::render_mission_results(frame, area, mission, deep, ctx);
        }
    }

    // Farewell modal shown after prestige when there were mercs
    if !ui.farewell_mercs.is_empty() {
        render_farewell_modal(frame, area, &ui.farewell_mercs);
    }
}

// ── Farewell Modal ──────────────────────────────────────────────────────────

/// Render the prestige farewell modal listing lost mercenaries.
fn render_farewell_modal(frame: &mut Frame, area: Rect, mercs: &[(String, u32, u32)]) {
    use ratatui::{
        style::Modifier,
        text::{Line, Span},
        widgets::Paragraph,
    };

    let line_count = mercs.len().min(10);
    let modal_height = (6 + line_count as u16).min(area.height.saturating_sub(4));
    let modal_width = 54u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" The Deep Claims Another Generation ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

    let mut lines = vec![Line::from("")];

    for (name, level, _missions) in mercs.iter().take(10) {
        let flavor = match *level {
            1..=3 => "fell before their time",
            4..=6 => "served with courage",
            7..=9 => "served with honor",
            _ => "legends of the deep",
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} (Lv{})", name, level),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!(" \u{2014} {}", flavor),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    if mercs.len() > 10 {
        lines.push(Line::from(Span::styled(
            format!("  ...and {} more", mercs.len() - 10),
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Continue",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

// ── Help panel ──────────────────────────────────────────────────────────────

/// Per-view reference content for the [?] help panel.
fn help_content(view: DeepView) -> &'static [&'static str] {
    match view {
        DeepView::Hub => &[
            "THE DEEP \u{2014} Quick Reference",
            "",
            "Warband Marks",
            "Earned from missions. Resets on prestige.",
            "Spend on recruits and infrastructure.",
            "",
            "Guild Rank",
            "Rank 1 Freelancers  5 mercs  1 mission",
            "Rank 2 Sellswords   7 mercs  1 mission",
            "Rank 3 Company      9 mercs  2 missions",
            "Rank 4 Battalion   12 mercs  3 missions",
            "Rank 5 Legion      15 mercs  4 missions",
            "Advance by clearing breakthrough layers.",
            "",
            "Prestige Cycle",
            "Resets: mercs, Marks, active missions",
            "Survives: guild rank, cleared layers,",
            "          infrastructure, familiarity",
        ],
        DeepView::NewMission => &[
            "MISSION TYPES",
            "",
            "Supply Run   2-4h  Safe",
            "Safe income. Always returns.",
            "",
            "Recon        4-8h  Low",
            "Raises layer familiarity.",
            "Cuts future mission times.",
            "",
            "Expedition   8-16h  Medium",
            "Main rewards: items + Marks.",
            "",
            "Breakthrough 18-24h  High",
            "Clears frontier. Unlocks next.",
            "Earns 0.5 Prestige Rank.",
            "",
            "Construction 4-8h  Safe",
            "Builds infrastructure. Permanent.",
            "",
            "POWER GUIDE",
            ">= 150%: 95% success + faster",
            ">= 100%: 60-90% success",
            ">= 75%:  ~30% success",
            "<  75%:  likely fail",
        ],
        DeepView::Roster => &[
            "MERC STATS",
            "",
            "Power      \u{2014} mission success driver",
            "Resilience \u{2014} injury resistance",
            "Expertise  \u{2014} enables special choices",
            "",
            "ARCHETYPES",
            "Vanguard   high power, durable",
            "Scout      recon duration bonus",
            "Arcanist   expedition bonuses",
            "Medic      reduces squad injuries",
            "Saboteur   cuts mission time -10-15%",
            "",
            "LEVELING",
            "Complete missions to gain XP.",
            "Missions to level = 3 + level x 2",
            "Max level: 20",
            "",
            "INJURIES",
            "Light:     ~1 mission recovery",
            "Moderate:  ~2 missions recovery",
            "Severe:    ~3 missions recovery",
            "Injured mercs cannot be assigned.",
        ],
        DeepView::Infrastructure => &[
            "FAMILIARITY (Intel %)",
            "Unknown  0-24%   no reduction",
            "Mapped   25-49%  -10% durations",
            "Familiar 50-74%  -20% durations",
            "Mastered 75-100% -30% durations",
            "",
            "Gain familiarity by running missions.",
            "Recon gives +15%, others +5-10%.",
            "",
            "INFRASTRUCTURE (permanent)",
            "Outpost      -25% all mission times",
            "Supply Cache +50% Marks on Supply Runs",
            "Watchtower   +25 familiarity on build",
            "Bridge       -2h on deeper missions",
            "",
            "BUILD VIA Construction missions.",
            "Costs scale with layer depth.",
            "Max 4 infrastructure per layer.",
        ],
        DeepView::EventResponse => &[
            "EVENTS",
            "",
            "Events fire at checkpoints during",
            "missions. Choose a response or let",
            "the timer auto-resolve (safe path).",
            "",
            "Archetype-gated choices require",
            "that archetype in the squad.",
            "",
            "Risky choices may shorten missions",
            "but increase injury chance.",
            "",
            "Auto-resolve always picks the",
            "safest option with no penalty.",
        ],
        DeepView::Recruit => &[
            "RECRUITING",
            "",
            "Pool refreshes every 24 hours.",
            "Stronger archetypes appear at",
            "higher Guild Ranks.",
            "",
            "QUALITY TIERS",
            "Common    base stats",
            "Uncommon  +5 total stats",
            "Rare      +10 total stats",
            "Elite     +20 total stats",
            "",
            "Cost scales with quality.",
            "Higher rank = better recruits.",
        ],
    }
}

/// Render the [?] help reference panel over the scene buffer.
fn render_help_panel(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    view: DeepView,
    ctx: &super::responsive::LayoutContext,
) {
    let lines = help_content(view);
    let is_small = ctx.tier <= super::responsive::SizeTier::M;

    // Panel dimensions: full-width overlay for S/M, right panel for L/XL
    let (panel_left, panel_width) = if is_small {
        (0usize, width)
    } else {
        let pw = 38usize.min(width.saturating_sub(4));
        (width.saturating_sub(pw), pw)
    };

    let panel_top = 1usize; // below tab bar
    let panel_bottom = height.saturating_sub(1); // above footer row

    // Dark background for help panel
    let bg = super::scene_fx::SceneCell {
        ch: ' ',
        fg: Color::DarkGray,
        bg: Color::Rgb(5, 8, 15),
        wide_cont: false,
    };
    for row in panel_top..panel_bottom {
        for col in panel_left..panel_left + panel_width {
            if row < buffer.len() && col < buffer[row].len() {
                buffer[row][col] = bg;
            }
        }
    }

    // Render help content
    let content_left = panel_left as i32 + 1;
    let max_w = panel_width.saturating_sub(2);
    for (i, line) in lines.iter().enumerate() {
        let row = panel_top as i32 + i as i32;
        if row >= panel_bottom as i32 {
            break;
        }
        let display_line: String = line.chars().take(max_w).collect();
        // First line (title) in section label color, rest in info color
        let color = if i == 0
            || (line.chars().all(|c| {
                c.is_uppercase() || c == ' ' || c == '(' || c == ')' || c == '%' || c == '/'
            }) && !line.is_empty())
        {
            Color::Rgb(80, 160, 220)
        } else {
            Color::Rgb(60, 100, 140)
        };
        put_text(buffer, row, content_left, &display_line, color);
    }

    // Footer hint
    let footer_row = panel_bottom as i32;
    if footer_row < height as i32 {
        put_text(
            buffer,
            footer_row,
            panel_left as i32 + 1,
            "[?] Close help",
            Color::DarkGray,
        );
    }
}

/// Render The Deep discovery modal (shown once on discovery).
pub fn render_deep_discovery_modal(frame: &mut Frame, area: Rect, _ctx: &LayoutContext) {
    let modal_width = 56u16.min(area.width.saturating_sub(4));
    let modal_height = 14u16.min(area.height.saturating_sub(4));
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
            "Send mercenaries on hour-long expeditions.",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "Earn Marks, items, and Prestige Rank fragments.",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "Infrastructure you build here survives prestige.",
            Style::default().fg(Color::Cyan),
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
