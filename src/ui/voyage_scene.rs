//! The Crossing — Act 2's main screen (spec 2).
//!
//! Left: the living chart (route graph on a virtual canvas, viewport
//! following the Vessel, the Tree on the horizon). Right: the Vessel panel
//! (two gauges, trim, phase) and whichever surface is open (junction cards,
//! trim panel, rumor ledger). Everything renders read-only from
//! `VoyageState` + `VoyageUiState`; animation time comes from `ui::clock`.

use crate::vessel::junction::{current_junction_cards, RoadCard};
use crate::vessel::route::{self, Feature, WaypointId};
use crate::vessel::voyage::{
    hope_label, SceneState, Trim, VoyagePhase, VoyageState, DRIFT_RECOVERY_HOURS, MINUTES_PER_DAY,
    RUMOR_PRICE,
};
use crate::vessel::{SceneModal, VoyageUiState, VoyageView};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::responsive::{LayoutContext, SizeTier};
use super::vessel_scene::VESSEL_VIOLET;

const VIOLET_DIM: Color = Color::Rgb(120, 90, 160);
const SEA_DIM: Color = Color::Rgb(70, 80, 110);
const GOLD: Color = Color::Rgb(255, 215, 0);

/// Entry point: the whole Act 2 frame.
pub fn render_voyage(
    frame: &mut Frame,
    area: Rect,
    voyage: &VoyageState,
    ui: &VoyageUiState,
    ctx: &LayoutContext,
) {
    frame.render_widget(Clear, area);

    if voyage.intro_pending {
        render_intro(frame, area);
        return;
    }

    match ctx.tier {
        SizeTier::XL | SizeTier::L => render_full(frame, area, voyage, ui),
        _ => render_strip(frame, area, voyage, ui),
    }

    if let Some(modal) = &ui.scene_modal {
        render_scene_modal(frame, area, modal);
    }
}

// ── Intro (placeholder for the 5-beat transition, spec 4) ──────────────────

fn render_intro(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" \u{2726} The Crossing \u{2726} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VESSEL_VIOLET));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for _ in 0..inner.height.saturating_sub(12) / 2 {
        lines.push(Line::from(""));
    }
    for text in [
        "The Loom stills. A quarter million lives of glory, spent as fuel.",
        "",
        "The hull that was your fate lifts from the world that made it.",
        "",
        "Below: everything you were. Ahead: a living branch of Yggdrasil,",
        "and the long dark between.",
        "",
        "Three souls stand at the rail and do not look back.",
    ] {
        lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(VESSEL_VIOLET),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Cast off",
        Style::default().fg(GOLD),
    )));
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

// ── Full layout (XL/L) ──────────────────────────────────────────────────────

fn render_full(frame: &mut Frame, area: Rect, voyage: &VoyageState, ui: &VoyageUiState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(50), Constraint::Length(42)])
        .split(area);

    render_chart_panel(frame, cols[0], voyage);

    match ui.view {
        VoyageView::Junction { selected } => {
            render_junction_panel(frame, cols[1], voyage, selected)
        }
        VoyageView::Trim { selected } => render_trim_panel(frame, cols[1], voyage, selected),
        VoyageView::Rumors => render_rumor_panel(frame, cols[1], voyage),
        VoyageView::Chart => render_vessel_panel(frame, cols[1], voyage),
    }
}

// ── The chart ───────────────────────────────────────────────────────────────

fn render_chart_panel(frame: &mut Frame, area: Rect, voyage: &VoyageState) {
    let chapter = current_chapter(voyage);
    let title = format!(
        " \u{2726} The Crossing \u{2014} Chapter {} \u{00b7} {} ",
        chapter.numeral(),
        chapter.display_name()
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VESSEL_VIOLET));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // The Tree on the horizon, above the canvas.
    let stage = route::tree_stage_index(last_position(voyage));
    let tree_art = route::TREE_STAGES[stage];
    let tree_height = tree_art.lines().count() as u16 + 1;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tree_height),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(inner);

    let mut tree_lines: Vec<Line> = tree_art
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(GOLD))))
        .collect();
    tree_lines.push(Line::from(Span::styled(
        "\u{00b7} \u{00b7} \u{00b7}",
        Style::default().fg(SEA_DIM),
    )));
    frame.render_widget(
        Paragraph::new(tree_lines).alignment(Alignment::Center),
        rows[0],
    );

    let lines = chart_lines(voyage, rows[1].width, rows[1].height);
    frame.render_widget(Paragraph::new(lines), rows[1]);

    frame.render_widget(
        Paragraph::new(legend_line(rows[2].width)).alignment(Alignment::Center),
        rows[2],
    );
}

/// One-line chart legend. The journey reads bottom-to-top, so the legend
/// also anchors the direction of travel. Compact wording under narrow
/// panels (the L tier gives the chart ~38 columns).
fn legend_line(width: u16) -> Line<'static> {
    let entries: &[(&str, Color, &str)] = if width >= 70 {
        &[
            ("\u{25c6}", GOLD, " you"),
            ("\u{25c9}", VESSEL_VIOLET, " visited"),
            ("\u{25cb}", Color::Gray, " known ahead"),
            ("\u{25cc}", SEA_DIM, " unknown"),
            ("\u{2715}", Color::DarkGray, " passed by"),
        ]
    } else {
        &[
            ("\u{25c6}", GOLD, " you"),
            ("\u{25c9}", VESSEL_VIOLET, " past"),
            ("\u{25cb}", Color::Gray, " ahead"),
            ("\u{2715}", Color::DarkGray, " shut"),
        ]
    };
    let mut spans = Vec::new();
    for (i, (glyph, color, label)) in entries.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                "  \u{00b7}  ",
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::styled(*glyph, Style::default().fg(*color)));
        spans.push(Span::styled(*label, Style::default().fg(Color::DarkGray)));
    }
    if width >= 100 {
        spans.push(Span::styled(
            "  \u{00b7}  the Tree lies north",
            Style::default().fg(Color::DarkGray),
        ));
    }
    Line::from(spans)
}

/// Pure chart rendering: the viewport around the Vessel as styled lines.
/// Exposed for snapshot tests.
pub fn chart_lines(voyage: &VoyageState, width: u16, height: u16) -> Vec<Line<'static>> {
    let (ship_x, ship_y) = ship_position(voyage);
    let w = width.max(10) as i32;
    let h = height.max(4) as i32;
    // Viewport top-left in canvas space, centered on the ship, clamped.
    let vx = (ship_x as i32 - w / 2).clamp(0, (120 - w).max(0));
    let vy = (ship_y as i32 - h / 2).clamp(0, (90 - h).max(0));

    let mut grid: Vec<Vec<(char, Color)>> = vec![vec![(' ', SEA_DIM); w as usize]; h as usize];
    let put = |x: i32, y: i32, c: char, color: Color, grid: &mut Vec<Vec<(char, Color)>>| {
        let (gx, gy) = (x - vx, y - vy);
        if gx >= 0 && gx < w && gy >= 0 && gy < h {
            grid[gy as usize][gx as usize] = (c, color);
        }
    };

    let known = known_waypoints(voyage);
    let visited: std::collections::HashSet<WaypointId> = voyage.visited.iter().copied().collect();

    // Roads first (waypoints and labels draw over them).
    for road in &route::ROADS {
        let from = route::waypoint(road.from).chart_pos;
        let to = route::waypoint(road.to).chart_pos;
        let taken = voyage
            .visited
            .windows(2)
            .any(|p| p[0] == road.from && p[1] == road.to)
            || voyage.current_road().is_some_and(|r| r.id == road.id);
        let color = if taken {
            VIOLET_DIM
        } else if voyage.untaken.contains(&road.id) {
            Color::DarkGray
        } else {
            SEA_DIM
        };
        for (x, y) in line_points(from, to) {
            put(x, y, '\u{00b7}', color, &mut grid);
        }
    }

    // Waypoints.
    for wp in &route::WAYPOINTS {
        let (x, y) = (wp.chart_pos.0 as i32, wp.chart_pos.1 as i32);
        let untaken_dest = voyage
            .untaken
            .iter()
            .any(|rid| route::road(*rid).to == wp.id && !visited.contains(&wp.id));
        let (glyph, color) = if visited.contains(&wp.id) {
            ('\u{25c9}', VESSEL_VIOLET) // ◉ visited
        } else if untaken_dest {
            ('\u{2715}', Color::DarkGray) // ✕ untaken, named forever
        } else if known.contains(&wp.id) {
            ('\u{25cb}', Color::Gray) // ○ known ahead
        } else {
            ('\u{25cc}', SEA_DIM) // ◌ beyond the horizon
        };
        put(x, y, glyph, color, &mut grid);

        // Labels: only what the fog allows — visited, known, untaken names.
        if visited.contains(&wp.id) || known.contains(&wp.id) || untaken_dest {
            let label_color = if untaken_dest {
                Color::DarkGray
            } else if known.contains(&wp.id) && !visited.contains(&wp.id) {
                Color::Gray
            } else {
                VIOLET_DIM
            };
            for (i, ch) in wp.name.chars().enumerate() {
                put(x + 2 + i as i32, y, ch, label_color, &mut grid);
            }
        }
    }

    // The Vessel, pulsing.
    let pulse = (super::clock::now_millis() / 600).is_multiple_of(2);
    let ship_glyph = if pulse { '\u{25c6}' } else { '\u{25c7}' }; // ◆ / ◇
    put(ship_x as i32, ship_y as i32, ship_glyph, GOLD, &mut grid);

    grid.into_iter()
        .map(|row| {
            Line::from(
                row.into_iter()
                    .map(|(c, color)| Span::styled(c.to_string(), Style::default().fg(color)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

/// The Vessel's position on the canvas: the held waypoint, or a point along
/// the current road proportional to progress.
fn ship_position(voyage: &VoyageState) -> (u16, u16) {
    match voyage.phase {
        VoyagePhase::HoldingStation { waypoint, .. } => route::waypoint(waypoint).chart_pos,
        VoyagePhase::Arrived { .. } => route::waypoint(route::ROUTE_SINK).chart_pos,
        VoyagePhase::Traveling {
            road,
            progress_days,
            ..
        }
        | VoyagePhase::Drifting {
            road,
            progress_days,
            ..
        } => {
            let r = route::road(road);
            let from = route::waypoint(r.from).chart_pos;
            let to = route::waypoint(r.to).chart_pos;
            let t = (progress_days / f64::from(r.base_days)).clamp(0.0, 1.0);
            (
                (f64::from(from.0) + (f64::from(to.0) - f64::from(from.0)) * t).round() as u16,
                (f64::from(from.1) + (f64::from(to.1) - f64::from(from.1)) * t).round() as u16,
            )
        }
    }
}

/// Waypoints inside the visibility horizon: the current road's destination
/// and card-visible stops; at a junction, every outgoing road's card.
fn known_waypoints(voyage: &VoyageState) -> std::collections::HashSet<WaypointId> {
    let mut known = std::collections::HashSet::new();
    match voyage.phase {
        VoyagePhase::Traveling { road, .. } | VoyagePhase::Drifting { road, .. } => {
            let r = route::road(road);
            known.insert(r.to);
            known.extend(r.known_stops.iter().copied());
        }
        VoyagePhase::HoldingStation { waypoint, .. } => {
            for r in route::roads_from(waypoint) {
                known.insert(r.to);
                known.extend(r.known_stops.iter().copied());
            }
            // Rumor-revealed stops are known too.
            for learned in &voyage.rumors {
                if let Some(stop) = route::rumor(learned.rumor).reveals_stop {
                    known.insert(stop);
                }
            }
        }
        VoyagePhase::Arrived { .. } => {}
    }
    known
}

/// Bresenham between two canvas points, endpoints excluded.
fn line_points(from: (u16, u16), to: (u16, u16)) -> Vec<(i32, i32)> {
    let (mut x0, mut y0) = (from.0 as i32, from.1 as i32);
    let (x1, y1) = (to.0 as i32, to.1 as i32);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut points = Vec::new();
    loop {
        if (x0, y0) == (x1, y1) {
            break;
        }
        if (x0, y0) != (from.0 as i32, from.1 as i32) {
            points.push((x0, y0));
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
    points
}

// ── The Vessel panel (right column, chart view) ─────────────────────────────

fn render_vessel_panel(frame: &mut Frame, area: Rect, voyage: &VoyageState) {
    let block = Block::default()
        .title(" \u{25c6} The Vessel ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VIOLET_DIM));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.extend(gauge_lines(voyage, inner.width));
    lines.push(Line::from(""));
    lines.extend(phase_lines(voyage));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("Rumors held: {}", voyage.rumors.len()),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    for key_line in footer_keys(voyage) {
        lines.push(key_line);
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn gauge_lines(voyage: &VoyageState, width: u16) -> Vec<Line<'static>> {
    let bar_width = (width.saturating_sub(20)).clamp(10, 20) as usize;
    let cap = voyage.provisions_cap.max(1.0);
    let filled = ((voyage.provisions / cap) * bar_width as f64).round() as usize;
    let filled = filled.min(bar_width);
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(bar_width - filled);
    let prov_color = if voyage.provisions < 20.0 {
        Color::LightRed
    } else if voyage.provisions < 40.0 {
        Color::Yellow
    } else {
        Color::Green
    };
    vec![
        Line::from(vec![
            Span::styled("Provisions ", Style::default().fg(Color::Gray)),
            Span::styled(bar, Style::default().fg(prov_color)),
            Span::styled(
                format!(" {}", voyage.provisions_display()),
                Style::default().fg(prov_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Hope       ", Style::default().fg(Color::Gray)),
            Span::styled(
                hope_label(voyage.hope).to_string(),
                Style::default()
                    .fg(VESSEL_VIOLET)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Trim       ", Style::default().fg(Color::Gray)),
            Span::styled(
                voyage.trim.display_name().to_string(),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("   Day {}", voyage.day_index() + 1),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ]
}

fn phase_lines(voyage: &VoyageState) -> Vec<Line<'static>> {
    match voyage.phase {
        VoyagePhase::Traveling { road, .. } => {
            let r = route::road(road);
            let to = route::waypoint(r.to);
            let eta = voyage.eta_minutes().unwrap_or(0);
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("Underway \u{2014} toward {}", to.name),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!("Arrives in {}", format_minutes(eta)),
                    Style::default().fg(Color::Gray),
                )),
            ];
            if !r.character.is_empty() {
                let tags: Vec<&str> = r.character.iter().map(|t| t.display_name()).collect();
                lines.push(Line::from(Span::styled(
                    format!("A {} road", tags.join(", ")),
                    Style::default().fg(VIOLET_DIM),
                )));
            }
            lines
        }
        VoyagePhase::Drifting { since_min, .. } => {
            let elapsed = voyage.processed_minutes.saturating_sub(since_min);
            let remaining = (DRIFT_RECOVERY_HOURS * 60).saturating_sub(elapsed);
            vec![
                Line::from(Span::styled(
                    "ADRIFT \u{2014} the hold is empty",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "The crew fishes, mends, and waits.",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    format!("Underway again in {}", format_minutes(remaining)),
                    Style::default().fg(Color::Gray),
                )),
            ]
        }
        VoyagePhase::HoldingStation {
            waypoint,
            scene_state,
            ..
        } => {
            let wp = route::waypoint(waypoint);
            let mut lines = vec![Line::from(Span::styled(
                format!("Holding station \u{2014} {}", wp.name),
                Style::default().fg(Color::White),
            ))];
            match scene_state {
                SceneState::Waiting => {
                    lines.push(Line::from(Span::styled(
                        "[Enter] Go ashore",
                        Style::default().fg(GOLD),
                    )));
                }
                SceneState::Played => {
                    let n = route::roads_from(waypoint).count();
                    lines.push(Line::from(Span::styled(
                        if n > 1 {
                            format!("[Enter] Chart a course \u{2014} {n} roads out")
                        } else {
                            "[Enter] Chart a course".to_string()
                        },
                        Style::default().fg(GOLD),
                    )));
                    if wp.has_feature(Feature::WayStation) && !voyage.rumor_bought_this_visit {
                        lines.push(Line::from(Span::styled(
                            format!("[B] Buy a rumor ({} provisions)", RUMOR_PRICE as u32),
                            Style::default().fg(Color::Gray),
                        )));
                    }
                }
            }
            lines
        }
        VoyagePhase::Arrived { at_min } => vec![
            Line::from(Span::styled(
                "The Roots of Light.",
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("The crossing took {} days.", at_min / MINUTES_PER_DAY),
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "(The finale arrives with a later update.)",
                Style::default().fg(Color::DarkGray),
            )),
        ],
    }
}

fn footer_keys(voyage: &VoyageState) -> Vec<Line<'static>> {
    let mut keys = vec!["[T] Trim", "[R] Rumors"];
    if matches!(
        voyage.phase,
        VoyagePhase::HoldingStation {
            scene_state: SceneState::Played,
            ..
        }
    ) {
        keys.insert(0, "[Enter] Course");
    }
    keys.push("[Q] Quit");
    vec![Line::from(Span::styled(
        keys.join("  \u{00b7}  "),
        Style::default().fg(Color::DarkGray),
    ))]
}

// ── Junction cards ──────────────────────────────────────────────────────────

fn render_junction_panel(frame: &mut Frame, area: Rect, voyage: &VoyageState, selected: usize) {
    let cards = current_junction_cards(voyage);
    let block = Block::default()
        .title(" \u{2725} Chart a course ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VESSEL_VIOLET));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for (i, card) in cards.iter().enumerate() {
        let is_selected = i == selected.min(cards.len().saturating_sub(1));
        lines.extend(card_lines(card, is_selected));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "\u{2191}\u{2193} choose  \u{00b7}  [Enter] commit  \u{00b7}  [Esc] back",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "Committing closes the other roads for good.",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn card_lines(card: &RoadCard, selected: bool) -> Vec<Line<'static>> {
    let marker = if selected { "\u{25b8} " } else { "  " };
    let title_style = if !card.selectable {
        Style::default().fg(Color::DarkGray)
    } else if selected {
        Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let mut lines = vec![Line::from(Span::styled(
        format!("{marker}{}", card.title),
        title_style,
    ))];

    let price_color = if card.affordable {
        Color::Green
    } else {
        Color::LightRed
    };
    let mut price_spans = vec![
        Span::styled(
            format!("    {} \u{00b7} ", card.days_label),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{} provisions", card.provisions_price),
            Style::default().fg(price_color),
        ),
    ];
    if !card.character.is_empty() {
        price_spans.push(Span::styled(
            format!(" \u{00b7} {}", card.character.join(", ")),
            Style::default().fg(VIOLET_DIM),
        ));
    }
    if !card.selectable {
        price_spans.push(Span::styled(
            " \u{00b7} locked",
            Style::default().fg(Color::LightRed),
        ));
    }
    lines.push(Line::from(price_spans));

    lines.push(Line::from(Span::styled(
        format!("    {}", card.blurb),
        Style::default().fg(Color::Gray),
    )));
    if !card.stops.is_empty() {
        let stops: Vec<String> = card
            .stops
            .iter()
            .map(|s| {
                if s.revealed_by_rumor {
                    format!("{}*", s.name)
                } else {
                    s.name.to_string()
                }
            })
            .collect();
        lines.push(Line::from(Span::styled(
            format!("    via {}", stops.join(", ")),
            Style::default().fg(Color::Gray),
        )));
    }
    if let Some(threat) = card.threat {
        lines.push(Line::from(Span::styled(
            format!("    \u{26a0} {threat}"),
            Style::default().fg(Color::LightRed),
        )));
    }
    for note in &card.annotations {
        lines.push(Line::from(Span::styled(
            format!("    \u{201c}{note}\u{201d}"),
            Style::default().fg(VESSEL_VIOLET),
        )));
    }
    lines
}

// ── Trim panel ──────────────────────────────────────────────────────────────

fn render_trim_panel(frame: &mut Frame, area: Rect, voyage: &VoyageState, selected: usize) {
    let block = Block::default()
        .title(" \u{2261} Trim ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VESSEL_VIOLET));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![Line::from("")];
    for (i, trim) in Trim::ALL.iter().enumerate() {
        let is_selected = i == selected.min(Trim::ALL.len() - 1);
        let is_current = *trim == voyage.trim;
        let marker = if is_selected { "\u{25b8} " } else { "  " };
        let name_style = if is_current {
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker}{}", trim.display_name()), name_style),
            Span::styled(
                if is_current { "  (set)" } else { "" }.to_string(),
                Style::default().fg(Color::Gray),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("    {}", trim_effect_line(voyage, *trim)),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "\u{2191}\u{2193} choose  \u{00b7}  [Enter] set  \u{00b7}  [Esc] back",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

/// Final computed prices, never multipliers (Underway's rule): what this
/// trim does to the rest of the current leg, or its standing character.
fn trim_effect_line(voyage: &VoyageState, trim: Trim) -> String {
    if let VoyagePhase::Traveling {
        road,
        progress_days,
        ..
    } = voyage.phase
    {
        let r = route::road(road);
        let remaining = (f64::from(r.base_days) - progress_days).max(0.0);
        let hours = (remaining * trim.time_mult() * 24.0).round() as u64;
        let provisions = (remaining * f64::from(r.base_provisions) / f64::from(r.base_days)
            * trim.provisions_mult())
        .round() as u64;
        let extra = match trim {
            Trim::Mourn => " · hope rises daily",
            Trim::Quiet => " · hears more",
            _ => "",
        };
        format!(
            "arrives in {} \u{00b7} {provisions} provisions{extra}",
            format_minutes(hours * 60)
        )
    } else {
        match trim {
            Trim::Run => "faster legs, a hungrier hold".to_string(),
            Trim::Cruise => "the default; never wrong, never best".to_string(),
            Trim::Quiet => "slower, thriftier, hears more".to_string(),
            Trim::Mourn => "slowest; the only trim that raises hope at sea".to_string(),
        }
    }
}

// ── Rumor ledger ────────────────────────────────────────────────────────────

fn render_rumor_panel(frame: &mut Frame, area: Rect, voyage: &VoyageState) {
    let block = Block::default()
        .title(" \u{2042} Rumors ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VESSEL_VIOLET));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![Line::from("")];
    if voyage.rumors.is_empty() {
        lines.push(Line::from(Span::styled(
            "Nothing heard yet. Way-stations sell what they hear.",
            Style::default().fg(Color::Gray),
        )));
    }
    for learned in &voyage.rumors {
        let def = route::rumor(learned.rumor);
        lines.push(Line::from(Span::styled(
            format!("\u{201c}{}\u{201d}", def.text),
            Style::default().fg(VESSEL_VIOLET),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "  \u{2014} heard at {}",
                route::waypoint(learned.learned_at).name
            ),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "[Esc] back",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

// ── Scene modal (placeholder arrival text until spec 4) ─────────────────────

fn render_scene_modal(frame: &mut Frame, area: Rect, modal: &SceneModal) {
    let width = area.width.clamp(30, 64);
    let height = 9u16.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let rect = Rect::new(x, y, width, height);
    frame.render_widget(Clear, rect);

    let block = Block::default()
        .title(format!(" {} ", modal.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GOLD));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            modal.body.clone(),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Esc] Close",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        inner,
    );
}

// ── Strip layout (S/M) ──────────────────────────────────────────────────────

fn render_strip(frame: &mut Frame, area: Rect, voyage: &VoyageState, _ui: &VoyageUiState) {
    let block = Block::default()
        .title(" \u{2726} The Crossing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(VESSEL_VIOLET));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![Line::from("")];
    let stage = route::tree_stage_index(last_position(voyage));
    lines.push(Line::from(Span::styled(
        format!(
            "Chapter {} \u{00b7} the Tree grows ({}/{})",
            current_chapter(voyage).numeral(),
            stage + 1,
            route::TREE_STAGES.len()
        ),
        Style::default().fg(GOLD),
    )));
    lines.push(Line::from(""));
    lines.extend(phase_lines(voyage));
    lines.push(Line::from(""));
    lines.extend(gauge_lines(voyage, inner.width));
    lines.push(Line::from(""));
    for key_line in footer_keys(voyage) {
        lines.push(key_line);
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

// ── Shared helpers ──────────────────────────────────────────────────────────

fn current_chapter(voyage: &VoyageState) -> route::Chapter {
    route::waypoint(last_position(voyage)).chapter
}

/// The most recently reached waypoint (for chapter/tree-stage display).
fn last_position(voyage: &VoyageState) -> WaypointId {
    voyage.visited.last().copied().unwrap_or(route::ROUTE_START)
}

fn format_minutes(minutes: u64) -> String {
    let days = minutes / MINUTES_PER_DAY;
    let hours = (minutes % MINUTES_PER_DAY) / 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h")
    } else {
        "under an hour".to_string()
    }
}
