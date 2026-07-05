//! The Vessel launch-gate UI: discovery modal and construction overlay.
//!
//! Sub-project 1 of Act 2 — see
//! `openspec/specs/vessel-act2/spec.md`.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::core::game_state::GameState;
use crate::loom::LoomState;
use crate::ui::responsive::LayoutContext;
use crate::vessel;

/// The Vessel signal violet — matches the ticker whisper color.
pub(super) const VESSEL_VIOLET: Color = Color::Rgb(150, 120, 200);
const VESSEL_VIOLET_DIM: Color = Color::Rgb(120, 90, 160);
const GOLD: Color = Color::Rgb(255, 215, 0);

/// One-time celebration modal when the Zone 50 boss reveals the signal.
pub fn render_vessel_discovery_modal(frame: &mut Frame, area: Rect, _ctx: &LayoutContext) {
    let modal_width = 58u16.min(area.width.saturating_sub(4));
    let modal_height = 14u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{2726} A Signal From Beyond \u{2726} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(VESSEL_VIOLET)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        VESSEL_VIOLET,
        super::BorderFxContext,
    );

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "The Origin Thread lies quiet.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "In the silence, something answers — a living branch",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "of Yggdrasil, impossibly far away, is calling.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "The Loom resonates. It has always been answering.",
            Style::default().fg(VESSEL_VIOLET),
        )),
        Line::from(Span::styled(
            "The 28 patterns were never only sustaining this world.",
            Style::default().fg(VESSEL_VIOLET),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [V] to behold the Vessel.  [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

/// The 5-beat launch transition (spec 4): a full-screen, Enter-advanced
/// sequence played once between the launch burn and the first Voyage frame.
/// Unlike the other Vessel screens this has no border — the transition is a
/// sequence of full-screen renders, not an overlay over the game beneath it.
pub fn render_launch_transition(frame: &mut Frame, area: Rect, beat: u8) {
    use crate::vessel::transition::{self, BEAT_COUNT};

    frame.render_widget(Clear, area);
    frame.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let content = transition::beat(beat);
    // Beats darken/fragment/build/brighten in sequence — the fiction of the
    // old UI unweaving into the new one, without needing real animation.
    let color = match beat {
        1 => Color::DarkGray,
        2 => VESSEL_VIOLET_DIM,
        3 => VESSEL_VIOLET,
        4 => GOLD,
        _ => Color::White,
    };

    let mut lines: Vec<Line> = Vec::new();
    let content_height = content.lines.len() as u16 + 4;
    for _ in 0..area.height.saturating_sub(content_height) / 2 {
        lines.push(Line::from(""));
    }
    for text in content.lines {
        lines.push(Line::from(Span::styled(
            (*text).to_string(),
            Style::default().fg(color),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter]",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, area);

    // A small "N / 5" marker in the corner — the only chrome on an
    // otherwise bare screen, so the player knows a sequence is playing.
    let marker = format!(" {beat} / {BEAT_COUNT} \u{2014} {} ", content.heading);
    let marker_width = marker.len() as u16;
    if area.width > marker_width {
        let marker_area = Rect::new(
            area.x + area.width - marker_width - 1,
            area.y,
            marker_width,
            1,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(marker, Style::default().fg(Color::DarkGray))),
            marker_area,
        );
    }
}

/// Full-screen construction overlay opened with [V].
pub fn render_vessel_overlay(
    frame: &mut Frame,
    area: Rect,
    state: &GameState,
    loom: &LoomState,
    confirm_pending: bool,
    _ctx: &LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" \u{2726} The Vessel \u{2726} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(VESSEL_VIOLET)));
    let inner =
        super::render_themed_block(frame, area, block, VESSEL_VIOLET, super::BorderFxContext);

    let completed_patterns = loom.persistent.completed_pattern_count();
    let mut lines: Vec<Line> = Vec::new();

    // Vertically center: the construction view is ~20 content lines.
    let content_height = 20u16;
    for _ in 0..inner.height.saturating_sub(content_height) / 2 {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    let hull_woven = state.vessel_launched || vessel::can_launch(state, completed_patterns);
    for art in ship_art(hull_woven) {
        lines.push(Line::from(Span::styled(
            art,
            Style::default().fg(VESSEL_VIOLET),
        )));
    }
    lines.push(Line::from(""));

    if state.vessel_launched {
        render_launched_lines(&mut lines, state);
    } else {
        render_construction_lines(&mut lines, state, loom, completed_patterns);
    }

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);

    if confirm_pending && !state.vessel_launched {
        render_launch_confirm_modal(frame, area, state);
    }
}

fn ship_art(launched: bool) -> Vec<&'static str> {
    if launched {
        vec![
            "\u{00b7}   \u{00b7}       \u{00b7}     \u{00b7}",
            "   \u{2571}\u{2550}\u{2550}\u{2550}\u{2572}      \u{00b7}",
            "  \u{2571} \u{25c6}\u{25c6}\u{25c6} \u{2572}   \u{00b7}",
            " \u{2571}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2572}",
            "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}",
            "    \u{2551}\u{2551}\u{2551}\u{2551}",
        ]
    } else {
        vec![
            "   \u{2571}\u{2550}\u{2550}\u{2550}\u{2572}",
            "  \u{2571} \u{25c6}\u{25c6}\u{25c6} \u{2572}   \u{2504} \u{2504} \u{2504}",
            " \u{2571}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2572}  (unwoven)",
            "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}",
        ]
    }
}

fn render_construction_lines(
    lines: &mut Vec<Line<'_>>,
    state: &GameState,
    loom: &LoomState,
    completed_patterns: usize,
) {
    lines.push(Line::from(Span::styled(
        "The Loom's woven fate strains toward the beacon.",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(Span::styled(
        "Reality itself will be burned as fuel \u{2014} everything, in one breath.",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));

    lines.push(requirement_line(
        completed_patterns >= vessel::LAUNCH_REQUIRED_PATTERNS,
        "The Loom is complete",
        &format!("{} / 28 Woven Patterns", completed_patterns),
    ));
    lines.push(requirement_line(
        state.ascension_level >= vessel::LAUNCH_REQUIRED_ASCENSION,
        "The champion is ready",
        &format!(
            "Ascension {}",
            crate::ui::stats_prestige::to_roman(state.ascension_level.max(1))
        ),
    ));
    lines.push(requirement_line(
        true, // The overlay only opens after the signal — Z50 is cleared.
        "The Origin Thread has spoken",
        "Zone 50 conquered",
    ));
    lines.push(requirement_line(
        state.prestige_rank >= vessel::LAUNCH_PR_COST,
        "Fuel for the transformation",
        &format!(
            "P{} / {}",
            format_thousands(state.prestige_rank),
            format_thousands(vessel::LAUNCH_PR_COST)
        ),
    ));
    lines.push(Line::from(""));

    lines.push(progress_bar_line(state.prestige_rank));
    lines.push(Line::from(""));
    lines.push(income_line(state, loom));
    lines.push(Line::from(""));

    if vessel::can_launch(state, completed_patterns) {
        let bright = super::clock::now_millis() % 1600 < 800;
        let style = if bright {
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(160, 130, 30))
        };
        lines.push(Line::from(Span::styled(
            "[Enter] Launch into the Void",
            style,
        )));
        lines.push(Line::from(Span::styled(
            "[Esc] Not yet",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "[Esc] Return to the dying branch",
            Style::default().fg(Color::DarkGray),
        )));
    }
}

fn render_launched_lines(lines: &mut Vec<Line<'_>>, state: &GameState) {
    lines.push(Line::from(Span::styled(
        format!(
            "The burn is complete. {} Prestige Ranks became a hull.",
            format_thousands(vessel::LAUNCH_PR_COST)
        ),
        Style::default().fg(GOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "What remains of you: P{}",
            format_thousands(state.prestige_rank)
        ),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "The Vessel awaits the void.",
        Style::default()
            .fg(VESSEL_VIOLET)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "(Act 2 \u{2014} the voyage \u{2014} is coming soon.)",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Esc] Close",
        Style::default().fg(Color::DarkGray),
    )));
}

fn render_launch_confirm_modal(frame: &mut Frame, area: Rect, state: &GameState) {
    let modal_width = 56u16.min(area.width.saturating_sub(4));
    let modal_height = 13u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{2500}\u{2500} The Transformation \u{2500}\u{2500} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(VESSEL_VIOLET)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        VESSEL_VIOLET,
        super::BorderFxContext,
    );

    let after = state.prestige_rank.saturating_sub(vessel::LAUNCH_PR_COST);
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("P{}", format_thousands(state.prestige_rank)),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  \u{2192}  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("P{}", format_thousands(after)),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "{} Prestige Ranks are burned in one breath.",
                format_thousands(vessel::LAUNCH_PR_COST)
            ),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "The Loom unweaves itself into a hull.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "The voyage to the living branch begins.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "There is no return.",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[Enter] Confirm",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("      [Esc] Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

fn requirement_line(met: bool, label: &str, detail: &str) -> Line<'static> {
    let (mark, mark_color) = if met {
        ("\u{2713}", Color::Green)
    } else {
        ("\u{2717}", Color::Red)
    };
    Line::from(vec![
        Span::styled(format!("{mark} "), Style::default().fg(mark_color)),
        Span::styled(format!("{label}  "), Style::default().fg(Color::White)),
        Span::styled(
            detail.to_string(),
            Style::default().fg(if met { Color::DarkGray } else { GOLD }),
        ),
    ])
}

fn progress_bar_line(prestige_rank: u32) -> Line<'static> {
    const BAR_WIDTH: usize = 46;
    let ratio = (prestige_rank as f64 / vessel::LAUNCH_PR_COST as f64).min(1.0);
    let filled = (ratio * BAR_WIDTH as f64).round() as usize;
    let ready = prestige_rank >= vessel::LAUNCH_PR_COST;
    let bar_color = if ready { GOLD } else { VESSEL_VIOLET };
    let mut spans = vec![
        Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
        Span::styled(
            "\u{2591}".repeat(BAR_WIDTH - filled),
            Style::default().fg(Color::Rgb(60, 50, 80)),
        ),
    ];
    spans.push(Span::styled(
        if ready {
            "  READY".to_string()
        } else {
            format!("  {}%", (ratio * 100.0) as u32)
        },
        Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
    ));
    Line::from(spans)
}

fn income_line(state: &GameState, loom: &LoomState) -> Line<'static> {
    let wr_rate = loom
        .rate_trackers
        .get(&crate::loom::Resource::WovenReality)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    let pr_per_hour = crate::loom::wr_to_pr_per_hour(wr_rate);
    let pr_per_day = pr_per_hour as u64 * 24;
    let remaining = vessel::LAUNCH_PR_COST.saturating_sub(state.prestige_rank) as u64;

    let text = if remaining == 0 {
        "The weave is ready. The beacon holds steady.".to_string()
    } else if pr_per_day == 0 {
        format!(
            "{} PR remain \u{00b7} the Loom is quiet",
            format_thousands(remaining as u32)
        )
    } else {
        let days = remaining.div_ceil(pr_per_day);
        format!(
            "Income ~{} PR/day \u{00b7} the weave will be ready in ~{} day{}",
            format_thousands(pr_per_day as u32),
            days,
            if days == 1 { "" } else { "s" }
        )
    };
    Line::from(Span::styled(text, Style::default().fg(VESSEL_VIOLET_DIM)))
}

fn format_thousands(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}
