//! The Loom of Worlds overlay — main UI renderer.
#![allow(dead_code)]
//!
//! Dispatches to different view renderers based on `LoomUiState::view`:
//!   - ArchetypeSelection: choose your archetype at unlock
//!   - FlowView:           pipeline diagram placeholder
//!   - ListDetail:         node list + detail panel placeholder
//!   - Codex:              recipe codex placeholder

use crate::loom::patterns::{active_pattern_requirement_status, all_patterns_complete};
use crate::loom::types::{LoomArchetype, LoomState, LoomUiState, LoomView};
use crate::ui::scene_fx::{current_millis, put_cell, put_text, render_buffer, SceneCell};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Set a cell with explicit foreground and background colors.
fn put_cell_bg(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, ch: char, fg: Color, bg: Color) {
    if row < 0 || col < 0 {
        return;
    }
    let row = row as usize;
    let col = col as usize;
    if row >= buffer.len() || col >= buffer[row].len() {
        return;
    }
    buffer[row][col] = SceneCell::new(ch, fg, bg);
}

/// Border color for the Loom overlay.
const LOOM_BORDER_COLOR: Color = Color::Rgb(180, 120, 220);

/// Background color for the Loom overlay interior.
const LOOM_BG: Color = Color::Rgb(10, 5, 18);

/// Width of a single node box in columns (including borders).
const NODE_BOX_WIDTH: usize = 28;
/// Height of a node box in rows (including borders).
const NODE_BOX_HEIGHT: usize = 4;

/// Per-node identity colors used for port labels and highlighting.
fn node_color(id: crate::loom::types::NodeId) -> Color {
    use crate::loom::types::NodeId;
    match id {
        NodeId::EmberSpindle => Color::Rgb(255, 140, 50),
        NodeId::VoidCondenser => Color::Rgb(160, 80, 220),
        NodeId::ReflectionLens => Color::Rgb(80, 200, 220),
        NodeId::MemoryArchive => Color::Rgb(220, 200, 80),
        NodeId::SilenceWell => Color::Rgb(140, 140, 160),
        NodeId::ResonanceForge => Color::Rgb(80, 140, 255),
    }
}

/// Single-letter abbreviation for port labels.
fn node_letter(id: crate::loom::types::NodeId) -> char {
    use crate::loom::types::NodeId;
    match id {
        NodeId::EmberSpindle => 'E',
        NodeId::VoidCondenser => 'V',
        NodeId::ReflectionLens => 'R',
        NodeId::MemoryArchive => 'M',
        NodeId::SilenceWell => 'S',
        NodeId::ResonanceForge => 'F',
    }
}

/// Extract NodeId from a LoomNodeRef, returning None for Refineries.
fn noderef_to_id(r: crate::loom::types::LoomNodeRef) -> Option<crate::loom::types::NodeId> {
    match r {
        crate::loom::types::LoomNodeRef::Extractor(id) => Some(id),
        crate::loom::types::LoomNodeRef::Refinery(_) => None,
    }
}

/// Color for a LoomNodeRef (gray fallback for refineries).
fn noderef_color(r: crate::loom::types::LoomNodeRef) -> Color {
    match noderef_to_id(r) {
        Some(id) => node_color(id),
        None => Color::Rgb(120, 100, 140),
    }
}

/// Letter for a LoomNodeRef ('?' fallback for refineries).
fn noderef_letter(r: crate::loom::types::LoomNodeRef) -> char {
    match noderef_to_id(r) {
        Some(id) => node_letter(id),
        None => '?',
    }
}

/// Name for a LoomNodeRef.
fn noderef_name(r: crate::loom::types::LoomNodeRef) -> &'static str {
    match noderef_to_id(r) {
        Some(id) => id.name(),
        None => "Refinery",
    }
}

/// Short resource name for recipe slot display inside node boxes.
fn resource_short(resource: &crate::loom::types::Resource) -> &'static str {
    use crate::loom::types::Resource;
    match resource {
        Resource::Ember => "Emb",
        Resource::Reflection => "Refl",
        Resource::VoidEssence => "Void",
        Resource::Memory => "Mem",
        Resource::Silence => "Slnc",
        Resource::Resonance => "Res",
        Resource::ForgedLight => "FrgLt",
        Resource::EchoGlass => "EchGl",
        Resource::StillbornSong => "StSng",
        Resource::CondensedEmber => "CndEm",
        Resource::EmberEcho => "EmbEc",
        Resource::PurifiedVoid => "PrVod",
        Resource::WovenReality => "WovRl",
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Render the Loom of Worlds overlay.
pub fn render_loom_overlay(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &mut LoomUiState,
) {
    ui.throbber_frame = ui.throbber_frame.wrapping_add(1);
    frame.render_widget(Clear, area);

    let view_name = match ui.view {
        LoomView::ArchetypeSelection => "Archetype Selection",
        LoomView::FlowView => "Flow View",
        LoomView::ListDetail => "Nodes",
        LoomView::Codex => "Recipe Codex",
    };

    let title = format!(" LOOM OF WORLDS \u{2014} {} ", view_name);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(LOOM_BORDER_COLOR));

    let inner = {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    };

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    match ui.view {
        LoomView::ArchetypeSelection => {
            render_archetype_selection(frame, inner, loom_state, ui);
        }
        LoomView::FlowView => {
            render_flow_view(frame, inner, loom_state, ui);
        }
        LoomView::ListDetail => {
            render_list_detail(frame, inner, loom_state, ui);
        }
        LoomView::Codex => {
            render_codex(frame, inner, loom_state, ui);
        }
    }

    // Render build overlay on top if active.
    if ui.build.is_some() {
        render_build_overlay(frame, inner, loom_state, ui);
    }

    render_nav_hints(frame, area, &*ui);
}

// ── View renderers ────────────────────────────────────────────────────────────

fn render_archetype_selection(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &LoomUiState,
) {
    let archetypes = [
        (
            LoomArchetype::BurnBright,
            "Burn Bright",
            "Ember Spindle + Void Condenser",
            "High throughput, volatile output",
        ),
        (
            LoomArchetype::ReachWide,
            "Reach Wide",
            "Reflection Lens + Memory Archive",
            "Broad coverage, pattern synergies",
        ),
        (
            LoomArchetype::RunDeep,
            "Run Deep",
            "Silence Well + Resonance Forge",
            "Efficient conversion, deep reactions",
        ),
    ];

    let already_chosen = loom_state.persistent.archetype;

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Choose your archetype to unlock the Loom:",
            Style::default().fg(Color::Rgb(200, 170, 240)),
        )),
        Line::from(""),
    ];

    for (i, (archetype, name, nodes, desc)) in archetypes.iter().enumerate() {
        let is_selected = ui.selected_archetype == i;
        let is_chosen = already_chosen == Some(*archetype);

        let prefix = if is_chosen {
            "\u{2713} "
        } else if is_selected {
            "\u{25b6} "
        } else {
            "  "
        };

        let name_color = if is_chosen {
            Color::Rgb(255, 215, 0)
        } else if is_selected {
            Color::White
        } else {
            Color::Rgb(140, 110, 180)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(name_color)),
            Span::styled(
                format!("[{}] {}", i + 1, name),
                Style::default().fg(name_color),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("     Nodes: {}", nodes),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            format!("     {}", desc),
            Style::default().fg(Color::Rgb(100, 80, 140)),
        )));
        lines.push(Line::from(""));
    }

    if already_chosen.is_none() {
        lines.push(Line::from(Span::styled(
            "[Enter] Confirm selection",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Archetype chosen. Use [Tab] to explore the Loom.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, area);
}

// ── Factory floor rendering (Tasks 2-7) ──────────────────────────────────────

/// Render the animated 2-row texture interior for a node.
#[allow(clippy::too_many_arguments)]
fn render_node_texture(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    width: usize,
    node_id: crate::loom::types::NodeId,
    stalled: bool,
    unlocked: bool,
    unlock_progress: f64,
) {
    use crate::loom::types::NodeId;

    if !unlocked {
        if unlock_progress > 0.0 {
            // Single row: progress bar with percentage
            let pct = (unlock_progress / 2.0).min(1.0);
            let bar_w = width.min(10);
            let filled = ((pct * bar_w as f64) as usize).min(bar_w);
            let empty = bar_w.saturating_sub(filled);
            let bar = format!(
                "{}{} {:.0}%",
                "\u{2593}".repeat(filled),
                "\u{2591}".repeat(empty),
                pct * 100.0
            );
            let start = col + (width as i32 - bar.chars().count() as i32) / 2;
            let progress_color = Color::Rgb(100, 80, 160);
            for (i, ch) in bar.chars().enumerate() {
                let fg = if i < filled {
                    progress_color
                } else {
                    Color::Rgb(60, 45, 80)
                };
                put_cell(buffer, row, start + i as i32, ch, fg);
            }
        } else {
            // Single row: "locked · needs neighbor"
            let lock_text = "locked \u{b7} needs neighbor";
            let lock_w = lock_text.len().min(width);
            let lock_str = &lock_text[..lock_w];
            let start = col + (width as i32 - lock_w as i32) / 2;
            put_text(buffer, row, start, lock_str, Color::Rgb(60, 45, 80));
        }
        return;
    }

    let millis = current_millis();
    let frame_idx = if stalled { 0 } else { (millis / 300) as usize };
    let color = if stalled {
        Color::Rgb(40, 30, 55)
    } else {
        node_color(node_id)
    };

    // Single texture row.
    for c in 0..width as i32 {
        let ch = match node_id {
            NodeId::EmberSpindle => {
                let offset = (frame_idx + c as usize) % 4;
                match offset {
                    0 | 2 => '~',
                    _ => ' ',
                }
            }
            NodeId::ReflectionLens => {
                let offset = (frame_idx + c as usize * 3) % 6;
                match offset {
                    0 | 4 => '.',
                    1 | 3 => '\u{b7}',
                    2 => '*',
                    _ => ' ',
                }
            }
            NodeId::VoidCondenser => {
                let offset = (frame_idx + c as usize * 2) % 4;
                match offset {
                    0 => ':',
                    2 => '\u{b7}',
                    _ => ' ',
                }
            }
            NodeId::MemoryArchive => {
                let offset = (frame_idx + c as usize) % 4;
                match offset {
                    0 | 2 => '\u{2573}',
                    _ => ' ',
                }
            }
            NodeId::SilenceWell => {
                let offset = (frame_idx + c as usize) % 6;
                match offset {
                    0 | 2 | 4 => '_',
                    _ => ' ',
                }
            }
            NodeId::ResonanceForge => {
                let offset = (frame_idx + c as usize) % 4;
                match offset {
                    0 | 2 => '\u{2248}',
                    _ => ' ',
                }
            }
        };
        put_cell(buffer, row, col + c, ch, color);
    }
}

/// Render recipe input slots inside a node box row.
fn render_recipe_slots(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    _width: usize,
    node: &crate::loom::types::LoomNode,
    _loom_state: &LoomState,
) {
    use crate::loom::recipes::recipes_by_nature;

    if !node.unlocked {
        return;
    }

    let nature = node.id.nature();
    let recipes = recipes_by_nature(nature);
    if recipes.is_empty() {
        return;
    }

    // In the direct-pull model, incoming resources are determined by refinery sources.
    // For extractor nodes, we show all possible recipes without pipe-based highlighting.
    let incoming_resources: std::collections::HashSet<crate::loom::types::Resource> =
        std::collections::HashSet::new();

    // Find the best recipe: prefer one with both inputs filled, then one with most.
    let best = recipes.iter().max_by_key(|r| {
        let has_a = incoming_resources.contains(&r.input_a) as u8;
        let has_b = incoming_resources.contains(&r.input_b) as u8;
        (has_a + has_b, (r.amount * 100.0) as u32)
    });

    if let Some(recipe) = best {
        let has_a = incoming_resources.contains(&recipe.input_a);
        let has_b = incoming_resources.contains(&recipe.input_b);
        let producing = has_a && has_b;

        let millis = current_millis();
        let pulse_bright = producing && (millis / 500).is_multiple_of(2);

        let filled_color = Color::Rgb(180, 220, 180);
        let empty_color = Color::Rgb(120, 50, 50);
        let output_color = if producing {
            Color::Rgb(220, 200, 255)
        } else {
            Color::Rgb(80, 60, 100)
        };

        let dot_a = if has_a { '\u{25cf}' } else { '\u{25cb}' };
        let dot_b = if has_b { '\u{25cf}' } else { '\u{25cb}' };
        let arrow = if pulse_bright { '\u{25b6}' } else { '>' };

        let slot_a = format!("[{}{}]", dot_a, resource_short(&recipe.input_a));
        let slot_b = format!("[{}{}]", dot_b, resource_short(&recipe.input_b));
        let output = resource_short(&recipe.output);

        let mut pos = 0i32;
        for ch in slot_a.chars() {
            put_cell(
                buffer,
                row,
                col + pos,
                ch,
                if has_a { filled_color } else { empty_color },
            );
            pos += 1;
        }
        put_cell(buffer, row, col + pos, ' ', LOOM_BG);
        pos += 1;
        for ch in slot_b.chars() {
            put_cell(
                buffer,
                row,
                col + pos,
                ch,
                if has_b { filled_color } else { empty_color },
            );
            pos += 1;
        }
        put_cell(buffer, row, col + pos, ' ', LOOM_BG);
        pos += 1;
        put_cell(buffer, row, col + pos, arrow, output_color);
        pos += 1;
        put_cell(buffer, row, col + pos, ' ', LOOM_BG);
        pos += 1;
        for ch in output.chars() {
            put_cell(buffer, row, col + pos, ch, output_color);
            pos += 1;
        }
    }
}

/// Render a single machine node box into the cell buffer.
/// Returns the row just below the box.
fn render_node_box(
    buffer: &mut [Vec<SceneCell>],
    top: i32,
    left: i32,
    node: &crate::loom::types::LoomNode,
    _loom_state: &LoomState,
    selected: bool,
) -> i32 {
    let w = NODE_BOX_WIDTH as i32;
    let inner_w = (w - 2) as usize;

    let (tl, tr, bl, br, h, v) = if selected {
        (
            '\u{250f}', '\u{2513}', '\u{2517}', '\u{251b}', '\u{2501}', '\u{2503}',
        )
    } else {
        (
            '\u{250c}', '\u{2510}', '\u{2514}', '\u{2518}', '\u{2500}', '\u{2502}',
        )
    };
    let border_color = if selected {
        Color::Rgb(220, 180, 255)
    } else if !node.unlocked {
        Color::Rgb(40, 30, 55)
    } else {
        Color::Rgb(80, 60, 110)
    };

    // Row 0: top border with title.
    put_cell(buffer, top, left, tl, border_color);
    for c in 1..w - 1 {
        put_cell(buffer, top, left + c, h, border_color);
    }
    put_cell(buffer, top, left + w - 1, tr, border_color);

    let title = if node.unlocked {
        format!(" {} Lv.{} ", node.id.name(), node.level)
    } else {
        format!(" {} ", node.id.name())
    };
    let title_color = if selected {
        Color::White
    } else if !node.unlocked {
        Color::Rgb(60, 45, 80)
    } else {
        node_color(node.id)
    };
    let title_start = left + 1;
    for (i, ch) in title.chars().enumerate().take(inner_w) {
        put_cell(buffer, top, title_start + i as i32, ch, title_color);
    }

    // Row 1: animated texture (1 row).
    put_cell(buffer, top + 1, left, v, border_color);
    put_cell(buffer, top + 1, left + w - 1, v, border_color);
    render_node_texture(
        buffer,
        top + 1,
        left + 1,
        inner_w,
        node.id,
        node.stalled,
        node.unlocked,
        node.unlock_progress,
    );

    // Row 2: "Ember 5/hr [gauge 0/20]" or unlock info.
    put_cell(buffer, top + 2, left, v, border_color);
    if node.unlocked {
        let fill = if node.buffer_capacity > 0.0 {
            (node.buffer / node.buffer_capacity).min(1.0)
        } else {
            0.0
        };

        // Text prefix: "Ember 5/hr "
        let resource = crate::loom::logic::node_native_resource(node.id);
        let res_name = resource_name(&resource);
        let rate = node.base_rate * crate::loom::logic::node_level_multiplier(node.level);
        let prefix = format!("{} {:.0}/hr ", res_name, rate);
        let prefix_len = prefix.len();

        // Render prefix text.
        let c = left + 1;
        for (i, ch) in prefix.chars().enumerate().take(inner_w) {
            put_cell(buffer, top + 2, c + i as i32, ch, Color::Rgb(140, 120, 170));
        }

        // Gauge fills the remaining space with "0/20" label centered.
        let gauge_w = inner_w.saturating_sub(prefix_len);
        if gauge_w >= 4 {
            let gauge_label = format!("{:.0}/{:.0}", node.buffer, node.buffer_capacity);
            let label_start = (gauge_w.saturating_sub(gauge_label.len())) / 2;
            let filled_cells = ((fill * gauge_w as f64) as usize).min(gauge_w);

            let bar_bg = if node.stalled || fill >= 0.90 {
                Color::Rgb(220, 60, 60)
            } else if fill >= 0.75 {
                Color::Rgb(220, 180, 60)
            } else {
                Color::Rgb(60, 200, 100)
            };
            let empty_bg = Color::Rgb(30, 20, 40);

            let gauge_col = c + prefix_len as i32;
            for i in 0..gauge_w {
                let is_filled = i < filled_cells;
                let bg = if is_filled { bar_bg } else { empty_bg };

                // Overlay the label text if we're in the label range.
                let label_idx = i.wrapping_sub(label_start);
                let ch = if label_idx < gauge_label.len() {
                    gauge_label.as_bytes()[label_idx] as char
                } else {
                    ' '
                };
                let fg = if is_filled {
                    Color::Rgb(10, 5, 18)
                } else {
                    Color::Rgb(160, 140, 190)
                };
                put_cell_bg(buffer, top + 2, gauge_col + i as i32, ch, fg, bg);
            }
        }
    } else {
        // Locked node: show unlock progress inline.
        let progress_text = if node.unlock_progress > 0.0 {
            format!("{:.1}/2.0h unlocking", node.unlock_progress)
        } else {
            "locked".to_string()
        };
        let fg = if node.unlock_progress > 0.0 {
            Color::Rgb(100, 80, 160)
        } else {
            Color::Rgb(60, 45, 80)
        };
        let start = left + 1 + (inner_w as i32 - progress_text.len() as i32) / 2;
        put_text(buffer, top + 2, start, &progress_text, fg);
    }
    put_cell(buffer, top + 2, left + w - 1, v, border_color);

    // Row 3: bottom border.
    put_cell(buffer, top + 3, left, bl, border_color);
    for c in 1..w - 1 {
        put_cell(buffer, top + 3, left + c, h, border_color);
    }
    put_cell(buffer, top + 3, left + w - 1, br, border_color);

    top + NODE_BOX_HEIGHT as i32
}

/// Two-letter abbreviation for a LoomNodeRef used in source badges.
fn noderef_short(r: crate::loom::types::LoomNodeRef, idx: usize) -> String {
    use crate::loom::types::{LoomNodeRef, NodeId};
    match r {
        LoomNodeRef::Extractor(id) => match id {
            NodeId::EmberSpindle => "ES".to_string(),
            NodeId::ReflectionLens => "RL".to_string(),
            NodeId::VoidCondenser => "VC".to_string(),
            NodeId::MemoryArchive => "MA".to_string(),
            NodeId::SilenceWell => "SW".to_string(),
            NodeId::ResonanceForge => "RF".to_string(),
        },
        LoomNodeRef::Refinery(_) => format!("R{}", idx),
    }
}

/// Braille throbber character for a given frame counter and tier.
/// T1: advances every 5 frames (slow), T2: every 3 frames, T3: every 1 frame.
/// Stalled: frozen at frame 0.
fn throbber_char(throbber_frame: u32, tier: u8, stalled: bool) -> char {
    const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    if stalled {
        return FRAMES[0];
    }
    let step = match tier {
        1 => 5u32,
        2 => 3u32,
        _ => 1u32,
    };
    let idx = ((throbber_frame / step) as usize) % FRAMES.len();
    FRAMES[idx]
}

/// Render a single refinery node box into the cell buffer.
///
/// Compact row format:
///   ⠹ T1 ForgedLight    Emb←[ES] Voi←[VC]  2.0/hr  ████░░░░░░
fn render_refinery_box(
    buffer: &mut [Vec<SceneCell>],
    top: i32,
    left: i32,
    refinery: &crate::loom::types::Refinery,
    selected: bool,
    index: usize,
    throbber_frame: u32,
) {
    let w = NODE_BOX_WIDTH as i32;
    let h = NODE_BOX_HEIGHT as i32;

    let (tl, tr, bl, br, horiz, vert) = if selected {
        (
            '\u{250f}', '\u{2513}', '\u{2517}', '\u{251b}', '\u{2501}', '\u{2503}',
        )
    } else {
        (
            '\u{250c}', '\u{2510}', '\u{2514}', '\u{2518}', '\u{2500}', '\u{2502}',
        )
    };
    let border_color = if selected {
        Color::Rgb(200, 170, 240)
    } else {
        Color::Rgb(50, 35, 65)
    };

    // Top border.
    put_cell(buffer, top, left, tl, border_color);
    for c in 1..w - 1 {
        put_cell(buffer, top, left + c, horiz, border_color);
    }
    put_cell(buffer, top, left + w - 1, tr, border_color);

    // Side borders for interior rows.
    for r in 1..h - 1 {
        put_cell(buffer, top + r, left, vert, border_color);
        put_cell(buffer, top + r, left + w - 1, vert, border_color);
    }

    // Bottom border.
    put_cell(buffer, top + h - 1, left, bl, border_color);
    for c in 1..w - 1 {
        put_cell(buffer, top + h - 1, left + c, horiz, border_color);
    }
    put_cell(buffer, top + h - 1, left + w - 1, br, border_color);

    let inner_w = (w - 2) as usize;
    let title_color = if selected {
        Color::White
    } else {
        Color::Rgb(160, 130, 190)
    };

    if refinery.under_construction {
        // Row 1: "R{i} [Building...]" with construction ticks.
        let ticks = refinery.construction_ticks_remaining;
        let secs = ticks / 10;
        let title = format!("R{} [Building... {}s]", index, secs);
        for (i, ch) in title.chars().enumerate().take(inner_w) {
            put_cell(
                buffer,
                top + 1,
                left + 1 + i as i32,
                ch,
                Color::Rgb(100, 80, 130),
            );
        }
        // Rows 2-4: empty interior during construction.
        return;
    }

    // Row 1 (top+1): "⠹ T{tier} {output_name}" — throbber + tier badge + output.
    let throb = throbber_char(throbber_frame, refinery.tier, refinery.stalled);
    let throb_color = if refinery.stalled {
        Color::Rgb(160, 60, 60)
    } else {
        Color::Rgb(140, 100, 200)
    };
    put_cell(buffer, top + 1, left + 1, throb, throb_color);
    put_cell(buffer, top + 1, left + 2, ' ', LOOM_BG);
    let tier_str = format!("T{}", refinery.tier);
    let tier_color = match refinery.tier {
        1 => Color::Rgb(100, 160, 100),
        2 => Color::Rgb(160, 140, 80),
        _ => Color::Rgb(180, 100, 200),
    };
    let mut col = left + 3;
    for ch in tier_str.chars() {
        put_cell(buffer, top + 1, col, ch, tier_color);
        col += 1;
    }
    put_cell(buffer, top + 1, col, ' ', LOOM_BG);
    col += 1;
    let out_name = resource_short(&refinery.output);
    for ch in out_name.chars() {
        if col >= left + w - 1 {
            break;
        }
        put_cell(buffer, top + 1, col, ch, title_color);
        col += 1;
    }

    // Row 2 (top+2): Source badges — "{ResourceShort}←[SourceShort]" for each source.
    col = left + 1;
    let source_label_color = Color::Rgb(100, 80, 140);
    let arrow_color = Color::Rgb(80, 60, 100);

    // sources_a first, then sources_b.
    let all_sources: Vec<(
        crate::loom::types::Resource,
        &crate::loom::types::LoomNodeRef,
    )> = {
        let mut v: Vec<(
            crate::loom::types::Resource,
            &crate::loom::types::LoomNodeRef,
        )> = Vec::new();
        for src in &refinery.sources_a {
            v.push((refinery.input_a, src));
        }
        for src in &refinery.sources_b {
            v.push((refinery.input_b, src));
        }
        v
    };

    for (res, src) in &all_sources {
        let short_src = match src {
            crate::loom::types::LoomNodeRef::Extractor(_) => noderef_short(**src, 0),
            crate::loom::types::LoomNodeRef::Refinery(ri) => format!("R{}", ri),
        };
        let res_short = resource_short(res);
        // Format: "Emb←[ES] "
        let badge = format!("{}←[{}] ", res_short, short_src);
        for ch in badge.chars() {
            if col >= left + w - 1 {
                break;
            }
            let fg = if ch == '[' || ch == ']' || ch == '\u{2190}' {
                arrow_color
            } else {
                source_label_color
            };
            put_cell(buffer, top + 2, col, ch, fg);
            col += 1;
        }
    }

    if all_sources.is_empty() {
        // Show "no sources" hint.
        let hint = "no sources";
        for (i, ch) in hint.chars().enumerate().take(inner_w) {
            put_cell(
                buffer,
                top + 2,
                left + 1 + i as i32,
                ch,
                Color::Rgb(60, 45, 80),
            );
        }
    }

    // Row 3 (top+3): Buffer bar.
    let fill_pct = if refinery.buffer_capacity > 0.0 {
        (refinery.buffer / refinery.buffer_capacity).min(1.0)
    } else {
        0.0
    };
    let bar_color = if refinery.stalled || fill_pct >= 0.90 {
        Color::Rgb(220, 60, 60)
    } else if fill_pct >= 0.75 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(60, 200, 100)
    };
    let bar_w = 10usize.min(inner_w.saturating_sub(8));
    let filled = ((fill_pct * bar_w as f64) as usize).min(bar_w);
    let empty = bar_w.saturating_sub(filled);
    let bar_str = format!(
        " {}{} {:>4.1}/{:.0}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        refinery.buffer,
        refinery.buffer_capacity
    );
    for (i, ch) in bar_str.chars().enumerate().take(inner_w) {
        let fg = if i == 0 || i > filled + empty {
            Color::Rgb(120, 100, 140)
        } else if i <= filled {
            bar_color
        } else {
            Color::Rgb(40, 30, 55)
        };
        put_cell(buffer, top + 3, left + 1 + i as i32, ch, fg);
    }

    // Row 4 (top+4): Recipe summary — "{input_a}+{input_b}▶{output}" with stall warning.
    let stall_suffix = if refinery.stalled {
        " \u{26a0}STALL"
    } else {
        ""
    };
    let recipe_text = format!(
        "{}+{}\u{25b6}{}{}",
        resource_short(&refinery.input_a),
        resource_short(&refinery.input_b),
        resource_short(&refinery.output),
        stall_suffix,
    );
    let recipe_color = if refinery.stalled {
        Color::Rgb(180, 80, 80)
    } else {
        Color::Rgb(100, 80, 130)
    };
    for (i, ch) in recipe_text.chars().enumerate().take(inner_w) {
        put_cell(buffer, top + 4, left + 1 + i as i32, ch, recipe_color);
    }
}

/// Render the sidebar detail panel for the selected node.
fn render_flow_sidebar(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::node_effective_rate;
    use crate::loom::recipes::recipes_by_nature;
    use crate::loom::types::NodeId;

    // If selected_node >= 6, show refinery detail instead of extractor detail.
    if ui.selected_node >= 6 {
        render_flow_sidebar_refinery(frame, area, loom_state, ui);
        return;
    }

    let nodes = NodeId::ALL;
    let selected_id = nodes[ui.selected_node.min(nodes.len() - 1)];
    let node = match loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == selected_id)
    {
        Some(n) => n,
        None => return,
    };

    let block = Block::default()
        .title(format!(" {} ", selected_id.name()))
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(80, 60, 110)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if !node.unlocked {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " [Locked]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        if node.unlock_progress > 0.0 {
            let pct = (node.unlock_progress / 2.0).min(1.0);
            let filled = ((pct * 10.0) as usize).min(10);
            let empty = 10usize.saturating_sub(filled);
            lines.push(Line::from(vec![
                Span::styled(" [", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "\u{2588}".repeat(filled),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
                Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Color::Rgb(40, 30, 55)),
                ),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.1}/2.0h", node.unlock_progress),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Unlocks when a neighbor's",
            Style::default().fg(Color::Rgb(60, 45, 80)),
        )));
        lines.push(Line::from(Span::styled(
            " buffer reaches 50%",
            Style::default().fg(Color::Rgb(60, 45, 80)),
        )));
        // Show which neighbors are unlocked
        let neighbors = crate::loom::node_neighbors(node.id);
        let unlocked_neighbors: Vec<&str> = neighbors
            .iter()
            .filter(|nid| {
                loom_state
                    .persistent
                    .nodes
                    .iter()
                    .any(|n| n.id == **nid && n.unlocked)
            })
            .map(|nid| nid.name())
            .collect();
        if !unlocked_neighbors.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Active neighbors:",
                Style::default().fg(Color::Rgb(80, 60, 110)),
            )));
            for name in &unlocked_neighbors {
                lines.push(Line::from(Span::styled(
                    format!("  \u{2022} {name}"),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                )));
            }
        } else {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " No active neighbors yet",
                Style::default().fg(Color::Rgb(60, 45, 80)),
            )));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!(" Lv.{}", node.level),
                Style::default().fg(Color::Rgb(120, 90, 160)),
            ),
            Span::styled(
                format!("  {}", node_nature_name(node.id.nature())),
                Style::default().fg(Color::Rgb(140, 100, 180)),
            ),
        ]));

        // Buffer bar.
        let fill = if node.buffer_capacity > 0.0 {
            (node.buffer / node.buffer_capacity).min(1.0)
        } else {
            0.0
        };
        let bar_color = if node.stalled || fill >= 0.90 {
            Color::Rgb(220, 60, 60)
        } else if fill >= 0.75 {
            Color::Rgb(220, 180, 60)
        } else {
            Color::Rgb(60, 200, 100)
        };
        let filled = ((fill * 10.0) as usize).min(10);
        let empty = 10usize.saturating_sub(filled);
        lines.push(Line::from(vec![
            Span::styled(" [", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
            Span::styled(
                "\u{2591}".repeat(empty),
                Style::default().fg(Color::Rgb(40, 30, 55)),
            ),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(Span::styled(
            format!(" {:.1}/{:.0}", node.buffer, node.buffer_capacity),
            Style::default().fg(bar_color),
        )));

        let rate = node_effective_rate(loom_state, node);
        lines.push(Line::from(Span::styled(
            format!(" +{:.1}/hr", rate),
            Style::default().fg(Color::Rgb(100, 200, 120)),
        )));
        lines.push(Line::from(""));

        // Recipe list.
        let nature = node.id.nature();
        let recipes = recipes_by_nature(nature);

        // In the direct-pull model, no pipe-based incoming tracking for extractors.
        let incoming: std::collections::HashSet<crate::loom::types::Resource> =
            std::collections::HashSet::new();

        if recipes.is_empty() {
            lines.push(Line::from(Span::styled(
                " No recipes",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!(" Recipes ({}):", node_nature_name(nature)),
                Style::default().fg(Color::Rgb(140, 100, 180)),
            )));
            for r in &recipes {
                let has_a = incoming.contains(&r.input_a);
                let has_b = incoming.contains(&r.input_b);
                let active = has_a && has_b;
                let dot_a = if has_a { "\u{25cf}" } else { "\u{25cb}" };
                let dot_b = if has_b { "\u{25cf}" } else { "\u{25cb}" };
                let color = if active {
                    Color::Rgb(200, 180, 240)
                } else {
                    Color::Rgb(70, 55, 90)
                };
                lines.push(Line::from(Span::styled(
                    format!(
                        " {}{} {}{}\u{25b6}{}",
                        dot_a,
                        resource_short(&r.input_a),
                        dot_b,
                        resource_short(&r.input_b),
                        resource_short(&r.output)
                    ),
                    Style::default().fg(color),
                )));
            }
        }

        // Upgrade cost and affordability.
        lines.push(Line::from(""));
        let cost = crate::loom::node_upgrade_cost(loom_state, node.id);
        let can_afford = node.buffer >= cost;
        let cost_color = if can_afford {
            Color::Rgb(100, 200, 120)
        } else {
            Color::Rgb(80, 60, 100)
        };
        let resource = crate::loom::logic::node_native_resource(node.id);
        lines.push(Line::from(vec![
            Span::styled(
                " [U]pgrade ",
                Style::default().fg(if can_afford {
                    Color::Rgb(200, 180, 240)
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(
                format!("{:.0} {}", cost, resource_name(&resource)),
                Style::default().fg(cost_color),
            ),
        ]));
    }

    lines.truncate(inner.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        inner,
    );
}

/// Render sidebar when a refinery (selected_node >= 6) is selected.
fn render_flow_sidebar_refinery(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &LoomUiState,
) {
    let refinery_idx = ui.selected_node - 6;
    let refineries = &loom_state.persistent.refineries;

    let title = if refinery_idx < refineries.len() {
        format!(" Refinery {} ", refinery_idx)
    } else {
        " Refinery ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(80, 60, 110)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if refinery_idx >= refineries.len() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " [No refinery]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        lines.truncate(inner.height as usize);
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            inner,
        );
        return;
    }

    let refinery = &refineries[refinery_idx];

    lines.push(Line::from(""));

    if refinery.under_construction {
        lines.push(Line::from(Span::styled(
            " [Under Construction]",
            Style::default().fg(Color::Rgb(160, 120, 200)),
        )));
        let ticks = refinery.construction_ticks_remaining;
        let secs = ticks / 10;
        lines.push(Line::from(Span::styled(
            format!(" ~{}s remaining", secs),
            Style::default().fg(Color::Rgb(100, 80, 130)),
        )));
    } else {
        // Tier and output.
        lines.push(Line::from(vec![
            Span::styled(
                format!(" Tier {}", refinery.tier),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            ),
            Span::styled(
                format!("  \u{2192} {}", resource_name(&refinery.output)),
                Style::default().fg(Color::Rgb(220, 180, 255)),
            ),
        ]));

        // Recipe.
        lines.push(Line::from(vec![
            Span::styled(" Recipe: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{} + {} \u{25b6} {}",
                    resource_name(&refinery.input_a),
                    resource_name(&refinery.input_b),
                    resource_name(&refinery.output)
                ),
                Style::default().fg(Color::Rgb(160, 120, 200)),
            ),
        ]));

        // Buffer bar.
        let fill = if refinery.buffer_capacity > 0.0 {
            (refinery.buffer / refinery.buffer_capacity).min(1.0)
        } else {
            0.0
        };
        let bar_color = if refinery.stalled || fill >= 0.90 {
            Color::Rgb(220, 60, 60)
        } else if fill >= 0.75 {
            Color::Rgb(220, 180, 60)
        } else {
            Color::Rgb(60, 200, 100)
        };
        let filled_cells = ((fill * 10.0) as usize).min(10);
        let empty_cells = 10usize.saturating_sub(filled_cells);
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" [", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "\u{2588}".repeat(filled_cells),
                Style::default().fg(bar_color),
            ),
            Span::styled(
                "\u{2591}".repeat(empty_cells),
                Style::default().fg(Color::Rgb(40, 30, 55)),
            ),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {:.1}/{:.0}", refinery.buffer, refinery.buffer_capacity),
                Style::default().fg(bar_color),
            ),
            if refinery.stalled {
                Span::styled(
                    " \u{26a0} STALLED",
                    Style::default().fg(Color::Rgb(220, 60, 60)),
                )
            } else {
                Span::raw("")
            },
        ]));

        // Amount multiplier.
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Yield: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("x{:.1}/cycle", refinery.amount),
                Style::default().fg(Color::Rgb(100, 200, 120)),
            ),
        ]));

        // Source connections — show per-input sources.
        lines.push(Line::from(""));
        if refinery.sources_a.is_empty() && refinery.sources_b.is_empty() {
            lines.push(Line::from(Span::styled(
                " No sources assigned",
                Style::default().fg(Color::Rgb(80, 60, 110)),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled(" In-A (", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    resource_short(&refinery.input_a),
                    Style::default().fg(Color::Rgb(180, 140, 220)),
                ),
                Span::styled("):", Style::default().fg(Color::DarkGray)),
            ]));
            if refinery.sources_a.is_empty() {
                lines.push(Line::from(Span::styled(
                    "   none",
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                )));
            } else {
                for src in &refinery.sources_a {
                    let src_name = match src {
                        crate::loom::types::LoomNodeRef::Extractor(id) => id.name(),
                        crate::loom::types::LoomNodeRef::Refinery(_) => "Refinery",
                    };
                    let src_color = noderef_color(*src);
                    lines.push(Line::from(vec![
                        Span::styled("   \u{2190} ", Style::default().fg(Color::DarkGray)),
                        Span::styled(src_name, Style::default().fg(src_color)),
                    ]));
                }
            }
            lines.push(Line::from(vec![
                Span::styled(" In-B (", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    resource_short(&refinery.input_b),
                    Style::default().fg(Color::Rgb(180, 140, 220)),
                ),
                Span::styled("):", Style::default().fg(Color::DarkGray)),
            ]));
            if refinery.sources_b.is_empty() {
                lines.push(Line::from(Span::styled(
                    "   none",
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                )));
            } else {
                for src in &refinery.sources_b {
                    let src_name = match src {
                        crate::loom::types::LoomNodeRef::Extractor(id) => id.name(),
                        crate::loom::types::LoomNodeRef::Refinery(_) => "Refinery",
                    };
                    let src_color = noderef_color(*src);
                    lines.push(Line::from(vec![
                        Span::styled("   \u{2190} ", Style::default().fg(Color::DarkGray)),
                        Span::styled(src_name, Style::default().fg(src_color)),
                    ]));
                }
            }
        }

        // Status line.
        let status_text = if refinery.stalled {
            " Status: STALLED"
        } else {
            " Status: Running"
        };
        let status_color = if refinery.stalled {
            Color::Rgb(220, 60, 60)
        } else {
            Color::Rgb(80, 200, 120)
        };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            status_text,
            Style::default().fg(status_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " [D]emolish",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.truncate(inner.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        inner,
    );
}

fn render_flow_view(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::types::NodeId;

    // Split: factory floor (left) | sidebar (right, 22 cols).
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);
    let floor_area = h_chunks[0];
    let sidebar_area = h_chunks[1];

    // Split factory floor: node grid (top) | pattern bar (bottom, 4 rows).
    let has_patterns = !loom_state.persistent.patterns.is_empty();
    let pattern_h = if has_patterns { 4u16 } else { 0u16 };
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(pattern_h)])
        .split(floor_area);

    let grid_area = v_chunks[0];
    let pattern_area = v_chunks[1];

    // ── Factory floor: 3x2 grid of machine nodes ─────────────────────────────
    // Laid out as a snake following the cycle:
    //   Row 0: ES  →  RL        (cycle: ES→RL)
    //                  ↓         (cycle: RL→VC)
    //   Row 1: MA  ←  VC        (cycle: VC→MA)
    //          ↓                 (cycle: MA→SW)
    //   Row 2: SW  →  RF        (cycle: SW→RF, RF→ES via outer loop)

    let grid: [(NodeId, NodeId); 3] = [
        (NodeId::EmberSpindle, NodeId::ReflectionLens),
        (NodeId::MemoryArchive, NodeId::VoidCondenser),
        (NodeId::SilenceWell, NodeId::ResonanceForge),
    ];

    // ── Render factory floor using cell buffer ────────────────────────────────
    let rows = grid_area.height as usize;
    let cols = grid_area.width as usize;
    let mut buffer = vec![vec![SceneCell::new(' ', LOOM_BG, LOOM_BG); cols]; rows];

    // Calculate grid positions: 3 rows, 2 columns of node boxes.
    let col_spacing = if cols > (NODE_BOX_WIDTH * 2 + 4) {
        (cols - NODE_BOX_WIDTH * 2) / 3
    } else {
        1
    };
    // Each node row = NODE_BOX_HEIGHT (4) + 1 (gap for arrows) = 5 rows.
    let row_stride = NODE_BOX_HEIGHT + 1;

    // Grid-position-to-NodeId mapping for selection (matches input navigation: ±2 for up/down, ±1 for left/right).
    let grid_ids = [
        NodeId::EmberSpindle,   // 0: top-left
        NodeId::ReflectionLens, // 1: top-right
        NodeId::MemoryArchive,  // 2: mid-left
        NodeId::VoidCondenser,  // 3: mid-right
        NodeId::SilenceWell,    // 4: bottom-left
        NodeId::ResonanceForge, // 5: bottom-right
    ];
    let selected_id = grid_ids[ui.selected_node.min(5)];

    let flow_color = Color::Rgb(60, 45, 80);
    let flow_arrow_color = Color::Rgb(120, 80, 180);

    for (row_idx, (left_id, right_id)) in grid.iter().enumerate() {
        let top = (row_idx * row_stride) as i32;
        let left_col = col_spacing as i32;
        let right_col = (col_spacing + NODE_BOX_WIDTH + col_spacing) as i32;

        // Render left node.
        if let Some(left_node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *left_id)
        {
            let is_sel = ui.selected_node < 6 && *left_id == selected_id;
            render_node_box(&mut buffer, top, left_col, left_node, loom_state, is_sel);
        }

        // Render right node.
        if let Some(right_node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *right_id)
        {
            let is_sel = ui.selected_node < 6 && *right_id == selected_id;
            render_node_box(&mut buffer, top, right_col, right_node, loom_state, is_sel);
        }

        // ── Horizontal arrow: only on rows 0 (ES→RL) and 2 (RF→SW) ──
        let gap_start = left_col + NODE_BOX_WIDTH as i32;
        let gap_end = right_col;
        let arrow_row = top + (NODE_BOX_HEIGHT as i32) / 2;
        if gap_end > gap_start + 1 {
            match row_idx {
                0 => {
                    // Top: ES ───▶ RL
                    for c in gap_start..(gap_end - 1) {
                        put_cell(&mut buffer, arrow_row, c, '\u{2500}', flow_color);
                        // ─
                    }
                    put_cell(
                        &mut buffer,
                        arrow_row,
                        gap_end - 1,
                        '\u{25b6}',
                        flow_arrow_color,
                    ); // ▶
                }
                2 => {
                    // Bottom: SW ◀─── RF
                    put_cell(
                        &mut buffer,
                        arrow_row,
                        gap_start,
                        '\u{25c0}',
                        flow_arrow_color,
                    ); // ◀
                    for c in (gap_start + 1)..gap_end {
                        put_cell(&mut buffer, arrow_row, c, '\u{2500}', flow_color);
                        // ─
                    }
                }
                _ => {} // Row 1: no horizontal arrow (vertical flow only)
            }
        }
    }

    // ── Vertical arrows forming the ring sides ──
    // Right side goes DOWN: RL → VC → RF
    // Left side goes UP: SW → MA → ES
    {
        let right_col = (col_spacing + NODE_BOX_WIDTH + col_spacing) as i32;
        let left_col = col_spacing as i32;
        let mid_right = right_col + NODE_BOX_WIDTH as i32 / 2;
        let mid_left = left_col + NODE_BOX_WIDTH as i32 / 2;

        // Right side: RL ↓ VC (between row 0 and row 1)
        let gap_top = NODE_BOX_HEIGHT as i32;
        let gap_bot = row_stride as i32;
        for r in gap_top..gap_bot {
            if r == gap_bot - 1 {
                put_cell(&mut buffer, r, mid_right, '\u{25bc}', flow_arrow_color);
            // ▼
            } else {
                put_cell(&mut buffer, r, mid_right, '\u{2502}', flow_color); // │
            }
        }

        // Right side: VC ↓ RF (between row 1 and row 2)
        let gap_top = (row_stride + NODE_BOX_HEIGHT) as i32;
        let gap_bot = (2 * row_stride) as i32;
        for r in gap_top..gap_bot {
            if r == gap_bot - 1 {
                put_cell(&mut buffer, r, mid_right, '\u{25bc}', flow_arrow_color);
            // ▼
            } else {
                put_cell(&mut buffer, r, mid_right, '\u{2502}', flow_color); // │
            }
        }

        // Left side: SW ↑ MA (between row 2 and row 1, going UP)
        let gap_top = (row_stride + NODE_BOX_HEIGHT) as i32;
        let gap_bot = (2 * row_stride) as i32;
        for r in gap_top..gap_bot {
            if r == gap_top {
                put_cell(&mut buffer, r, mid_left, '\u{25b2}', flow_arrow_color);
            // ▲
            } else {
                put_cell(&mut buffer, r, mid_left, '\u{2502}', flow_color); // │
            }
        }

        // Left side: MA ↑ ES (between row 1 and row 0, going UP)
        let gap_top = NODE_BOX_HEIGHT as i32;
        let gap_bot = row_stride as i32;
        for r in gap_top..gap_bot {
            if r == gap_top {
                put_cell(&mut buffer, r, mid_left, '\u{25b2}', flow_arrow_color);
            // ▲
            } else {
                put_cell(&mut buffer, r, mid_left, '\u{2502}', flow_color); // │
            }
        }
    }

    // ── Refineries: render below the 3-row extractor grid ────────────────────
    let refineries = &loom_state.persistent.refineries;
    if !refineries.is_empty() {
        // Separator label just above the first refinery row.
        let sep_row = (3 * row_stride) as i32 - 1;
        put_text(
            &mut buffer,
            sep_row,
            col_spacing as i32,
            "\u{2500}\u{2500} Processing \u{2500}\u{2500}",
            Color::Rgb(80, 60, 100),
        );

        let refinery_row_start = 3 * row_stride;
        let refinery_cols = 2usize;

        for (i, refinery) in refineries.iter().enumerate() {
            let grid_row = i / refinery_cols;
            let grid_col = i % refinery_cols;
            let top = (refinery_row_start + grid_row * row_stride) as i32;
            let left_col = if grid_col == 0 {
                col_spacing as i32
            } else {
                (col_spacing + NODE_BOX_WIDTH + col_spacing) as i32
            };
            let is_sel = ui.selected_node >= 6 && (ui.selected_node - 6) == i;
            render_refinery_box(
                &mut buffer,
                top,
                left_col,
                refinery,
                is_sel,
                i,
                ui.throbber_frame,
            );
        }
    }

    render_buffer(frame, grid_area, &buffer);

    // ── Sidebar ─────────────────────────────────────────────────────────────
    render_flow_sidebar(frame, sidebar_area, loom_state, ui);

    // ── Pattern bar ─────────────────────────────────────────────────────────
    if has_patterns {
        render_pattern_bar(frame, pattern_area, loom_state);
    }
}

fn render_list_detail(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::types::NodeId;
    use crate::loom::{node_effective_rate, node_upgrade_cost};

    // Split into left list (60%) and right detail panel (40%).
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);
    let list_area = h_chunks[0];
    let detail_area = h_chunks[1];

    let nodes = NodeId::ALL;
    let selected_node_id = nodes[ui.selected_node.min(nodes.len() - 1)];

    // ── Left: node list ────────────────────────────────────────────────────────
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Nodes",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )),
        Line::from(""),
    ];

    for (i, node_id) in nodes.iter().enumerate() {
        let Some(node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *node_id)
        else {
            continue;
        };
        let is_selected = ui.selected_node == i;
        let prefix = if is_selected { "\u{25b6} " } else { "  " };

        let color = if !node.unlocked {
            Color::Rgb(60, 45, 80)
        } else if is_selected {
            Color::White
        } else {
            Color::Rgb(120, 80, 160)
        };

        let stall_marker = if node.stalled {
            Span::styled(" \u{26a0}", Style::default().fg(Color::Rgb(220, 60, 60)))
        } else {
            Span::raw("")
        };
        let lock_marker = if !node.unlocked {
            Span::styled(" [locked]", Style::default().fg(Color::Rgb(60, 45, 80)))
        } else {
            Span::raw("")
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(
                format!("{:<16}", node_id.name()),
                Style::default().fg(color),
            ),
            if node.unlocked {
                Span::styled(
                    format!("L{}", node.level),
                    Style::default().fg(Color::Rgb(100, 75, 140)),
                )
            } else {
                Span::raw("")
            },
            stall_marker,
            lock_marker,
        ]));

        // Show refinery consumers of this node inline in the list.
        if is_selected && node.unlocked {
            let consumer_count = loom_state
                .persistent
                .refineries
                .iter()
                .filter(|r| {
                    let node_ref = crate::loom::types::LoomNodeRef::Extractor(*node_id);
                    r.sources_a.contains(&node_ref) || r.sources_b.contains(&node_ref)
                })
                .count();
            if consumer_count == 0 {
                lines.push(Line::from(Span::styled(
                    "     no refinery consumers",
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("     {} refinery consumer(s)", consumer_count),
                    Style::default().fg(Color::Rgb(100, 70, 130)),
                )));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(Color::Rgb(10, 5, 18))),
        list_area,
    );

    // ── Right: selected node detail panel ─────────────────────────────────────
    let detail_block = Block::default()
        .title(format!(" {} ", selected_node_id.name()))
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(80, 60, 110)));
    let detail_inner = detail_block.inner(detail_area);
    frame.render_widget(detail_block, detail_area);

    let Some(selected_node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == selected_node_id)
    else {
        return;
    };

    let mut detail_lines: Vec<Line> = vec![Line::from("")];

    if !selected_node.unlocked {
        detail_lines.push(Line::from(Span::styled(
            " [Locked]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        detail_lines.push(Line::from(Span::styled(
            " Unlock via neighbor",
            Style::default().fg(Color::Rgb(60, 45, 80)),
        )));
        detail_lines.push(Line::from(""));
        // Show unlock progress bar if partially unlocked.
        if selected_node.unlock_progress > 0.0 {
            let prog_pct = (selected_node.unlock_progress / 2.0).min(1.0);
            let filled = ((prog_pct * 10.0) as usize).min(10);
            let empty = 10usize.saturating_sub(filled);
            detail_lines.push(Line::from(vec![
                Span::styled(" Progress [", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "\u{2588}".repeat(filled),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
                Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Color::Rgb(40, 30, 55)),
                ),
                Span::styled("]", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:.1}h", selected_node.unlock_progress),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
            ]));
        }
    } else {
        // Nature and native resource.
        let nature_name = node_nature_name(selected_node_id.nature());
        let native = crate::loom::node_native_resource(selected_node_id);
        detail_lines.push(Line::from(vec![
            Span::styled(" Nature: ", Style::default().fg(Color::DarkGray)),
            Span::styled(nature_name, Style::default().fg(Color::Rgb(180, 140, 220))),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(" Produces: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                resource_name(&native),
                Style::default().fg(Color::Rgb(220, 180, 255)),
            ),
        ]));
        detail_lines.push(Line::from(""));

        // Buffer bar.
        let fill = if selected_node.buffer_capacity > 0.0 {
            (selected_node.buffer / selected_node.buffer_capacity).min(1.0)
        } else {
            0.0
        };
        let bar_color = if selected_node.stalled || fill >= 0.90 {
            Color::Rgb(220, 60, 60)
        } else if fill >= 0.75 {
            Color::Rgb(220, 180, 60)
        } else {
            Color::Rgb(60, 200, 100)
        };
        let filled_cells = ((fill * 12.0) as usize).min(12);
        let empty_cells = 12usize.saturating_sub(filled_cells);
        detail_lines.push(Line::from(vec![
            Span::styled(" Buffer [", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "\u{2588}".repeat(filled_cells),
                Style::default().fg(bar_color),
            ),
            Span::styled(
                "\u{2591}".repeat(empty_cells),
                Style::default().fg(Color::Rgb(40, 30, 55)),
            ),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(
                format!(
                    " {:.1}/{:.1}",
                    selected_node.buffer, selected_node.buffer_capacity
                ),
                Style::default().fg(bar_color),
            ),
            if selected_node.stalled {
                Span::styled(
                    " \u{26a0} STALLED",
                    Style::default().fg(Color::Rgb(220, 60, 60)),
                )
            } else {
                Span::raw("")
            },
        ]));
        detail_lines.push(Line::from(""));

        // Production rate.
        let rate = node_effective_rate(loom_state, selected_node);
        detail_lines.push(Line::from(vec![
            Span::styled(" Rate: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}/hr", rate),
                Style::default().fg(Color::Rgb(100, 200, 120)),
            ),
            Span::styled(
                format!(" (Lv{})", selected_node.level),
                Style::default().fg(Color::Rgb(80, 60, 100)),
            ),
        ]));
        detail_lines.push(Line::from(""));

        // Upgrade cost.
        let upgrade_cost = node_upgrade_cost(loom_state, selected_node_id);
        let can_upgrade = selected_node.buffer >= upgrade_cost;
        let upgrade_color = if can_upgrade {
            Color::Rgb(180, 255, 180)
        } else {
            Color::Rgb(100, 75, 80)
        };
        detail_lines.push(Line::from(vec![
            Span::styled(" Upgrade: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0} {}", upgrade_cost, resource_name(&native)),
                Style::default().fg(upgrade_color),
            ),
        ]));
        if can_upgrade {
            detail_lines.push(Line::from(Span::styled(
                " [U] Upgrade node",
                Style::default().fg(Color::Rgb(120, 200, 120)),
            )));
        } else {
            detail_lines.push(Line::from(Span::styled(
                " (insufficient buffer)",
                Style::default().fg(Color::Rgb(80, 60, 70)),
            )));
        }

        // Refinery consumer count for this node.
        let node_ref = crate::loom::types::LoomNodeRef::Extractor(selected_node_id);
        let consumer_count = loom_state
            .persistent
            .refineries
            .iter()
            .filter(|r| r.sources_a.contains(&node_ref) || r.sources_b.contains(&node_ref))
            .count();
        if consumer_count > 0 {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled(" Consumers: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} refiner{}",
                        consumer_count,
                        if consumer_count == 1 { "y" } else { "ies" }
                    ),
                    Style::default().fg(Color::Rgb(140, 100, 180)),
                ),
            ]));
        }
    }

    detail_lines.truncate(detail_inner.height as usize);
    frame.render_widget(
        Paragraph::new(detail_lines).style(Style::default().bg(Color::Rgb(10, 5, 18))),
        detail_inner,
    );
}

/// Build a compact inline ratio bar (e.g. "████░░░░░░" for 40%).
fn build_ratio_bar(ratio: f64, width: usize) -> String {
    let filled = ((ratio.clamp(0.0, 1.0) * width as f64) as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

fn render_codex(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::recipes::all_recipes;

    let discovered_count = loom_state
        .persistent
        .codex
        .iter()
        .filter(|e| e.discovered)
        .count();
    let total_recipes = all_recipes().len();
    let hint_indices = crate::loom::logic::codex_hint_indices(&loom_state.persistent.codex);
    let hint_count = hint_indices.len();

    let count_label = format!("  {}/{} Discovered", discovered_count, total_recipes);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Recipe Codex",
                Style::default().fg(Color::Rgb(180, 120, 220)),
            ),
            Span::styled(count_label, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
    ];

    if discovered_count == 0 && hint_count == 0 {
        lines.push(Line::from(Span::styled(
            "No recipes discovered yet.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Process resources through nodes to reveal reactions.",
            Style::default().fg(Color::Rgb(80, 60, 100)),
        )));
    } else {
        // Discovered recipes with proper nature names and output multiplier.
        for entry in loom_state.persistent.codex.iter().filter(|e| e.discovered) {
            let inputs: Vec<&str> = entry.inputs.iter().map(|r| resource_name(r)).collect();
            let nature = node_nature_name(entry.node_nature);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} @ {}", inputs.join(" + "), nature),
                    Style::default().fg(Color::Rgb(160, 100, 200)),
                ),
                Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    resource_name(&entry.output),
                    Style::default().fg(Color::Rgb(220, 180, 255)),
                ),
                Span::styled(
                    format!(" (x{:.1})", entry.output_amount),
                    Style::default().fg(Color::Rgb(130, 90, 170)),
                ),
            ]));
        }

        // "???" hints for adjacent undiscovered recipes — show all of them (scrollable).
        if hint_count > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    " {} adjacent reaction{} hinted:",
                    hint_count,
                    if hint_count == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::DarkGray),
            )));
            for _ in 0..hint_count {
                lines.push(Line::from(Span::styled(
                    " ??? + ??? @ ??? \u{2192} ???",
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                )));
            }
        }
    }

    // Clamp scroll so it can't scroll past the last line.
    let total_lines = lines.len();
    let visible_rows = area.height as usize;
    let scroll = ui
        .codex_scroll
        .min(total_lines.saturating_sub(visible_rows));

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)))
        .scroll((scroll as u16, 0));
    frame.render_widget(para, area);
}

/// Returns the display name for a NodeNature.
fn node_nature_name(nature: crate::loom::types::NodeNature) -> &'static str {
    use crate::loom::types::NodeNature;
    match nature {
        NodeNature::Heat => "Heat",
        NodeNature::Form => "Form",
        NodeNature::Void => "Void",
        NodeNature::Pattern => "Pattern",
        NodeNature::Stillness => "Stillness",
        NodeNature::Vibration => "Vibration",
    }
}

// ── Buffer bar helpers ────────────────────────────────────────────────────────

/// Build a single line showing a node's name, lock status, buffer bar, and stall warning.
fn build_node_buffer_line(node: &crate::loom::types::LoomNode, width: u16) -> Line<'static> {
    if !node.unlocked {
        return Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:<18}", node.id.name()),
                Style::default().fg(Color::Rgb(60, 45, 80)),
            ),
            Span::styled("[locked]", Style::default().fg(Color::Rgb(60, 45, 80))),
        ]);
    }

    let fill_ratio = if node.buffer_capacity > 0.0 {
        (node.buffer / node.buffer_capacity).min(1.0)
    } else {
        0.0
    };

    // Color: green normal, yellow >75%, red >90% or stalled.
    let bar_color = if node.stalled || fill_ratio >= 0.90 {
        Color::Rgb(220, 60, 60)
    } else if fill_ratio >= 0.75 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(60, 200, 100)
    };

    // Bar width: total width minus name(18) minus brackets(2) minus spacing(4) minus stock text(12).
    let bar_width = (width as usize)
        .saturating_sub(18 + 2 + 4 + 12)
        .clamp(4, 20);
    let filled = ((fill_ratio * bar_width as f64) as usize).min(bar_width);
    let empty = bar_width - filled;

    let stall_marker = if node.stalled { " \u{26a0}STALL" } else { "" };

    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{:<18}", node.id.name()),
            Style::default().fg(Color::Rgb(180, 140, 220)),
        ),
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
        Span::styled(
            "\u{2591}".repeat(empty),
            Style::default().fg(Color::Rgb(40, 30, 55)),
        ),
        Span::styled("]", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                " {:>5.1}/{:<5.1}{}",
                node.buffer, node.buffer_capacity, stall_marker
            ),
            Style::default().fg(bar_color),
        ),
    ])
}

// ── Pattern bar ───────────────────────────────────────────────────────────────

/// Render the always-visible pattern progress bar at the bottom of Flow View.
///
/// Shows: pattern name + index, per-requirement checkmarks, sustain progress bar.
/// When all patterns are complete, shows a completion message.
fn render_pattern_bar(frame: &mut Frame, area: Rect, loom_state: &LoomState) {
    if area.height == 0 {
        return;
    }

    let req_status = active_pattern_requirement_status(&loom_state.persistent);
    let all_done = all_patterns_complete(&loom_state.persistent);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(100, 60, 140)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if all_done {
        let line = Line::from(Span::styled(
            " \u{2728} Loom Mended \u{2014} All 18 Patterns Complete",
            Style::default().fg(Color::Rgb(255, 215, 0)),
        ));
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(Color::Rgb(10, 5, 18))),
            inner,
        );
        return;
    }

    let Some(pattern) = loom_state
        .persistent
        .patterns
        .get(loom_state.persistent.active_pattern)
    else {
        return;
    };

    let pattern_count = loom_state.persistent.patterns.len();
    let active_idx = loom_state.persistent.active_pattern;

    // Line 1: pattern name and index.
    let pattern_title = format!(
        " PATTERN {}/{}: \"{}\"",
        active_idx + 1,
        pattern_count,
        pattern.name
    );

    // Line 2: per-requirement status chips (accumulated/amount + resource name).
    let mut req_spans: Vec<Span> = vec![Span::raw(" ")];
    for (i, req) in pattern.requirements.iter().enumerate() {
        let (accumulated, amount) = req_status.get(i).copied().unwrap_or((0.0, 0.0));
        let met = accumulated >= amount;
        let check = if met { "\u{2713}" } else { "\u{2717}" };
        let color = if met {
            Color::Rgb(80, 200, 120)
        } else {
            Color::Rgb(200, 80, 80)
        };
        let res_name = resource_name(&req.resource);
        req_spans.push(Span::styled(
            format!("{} {} {:.1}/{:.1} ", check, res_name, accumulated, amount),
            Style::default().fg(color),
        ));
        if i + 1 < pattern.requirements.len() {
            req_spans.push(Span::styled(
                "\u{2502} ",
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    // Line 3: progress bar — use the minimum completion ratio across all requirements.
    let min_ratio = if pattern.requirements.is_empty() {
        0.0
    } else {
        pattern
            .requirements
            .iter()
            .map(|req| {
                if req.amount > 0.0 {
                    req.accumulated / req.amount
                } else {
                    1.0
                }
            })
            .fold(f64::INFINITY, f64::min)
            .min(1.0)
    };
    let progress_line = build_progress_line_ratio(min_ratio, inner.width);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            pattern_title,
            Style::default().fg(Color::Rgb(200, 160, 240)),
        )),
        Line::from(req_spans),
        progress_line,
    ];
    lines.truncate(inner.height as usize);

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(Color::Rgb(10, 5, 18))),
        inner,
    );
}

/// Build the pattern progress bar line from a completion ratio in [0.0, 1.0].
fn build_progress_line_ratio(ratio: f64, width: u16) -> Line<'static> {
    let label_prefix = " Weaving: ";
    let pct_suffix = format!("  {:.0}%", (ratio * 100.0).min(100.0));
    let bar_width = (width as usize)
        .saturating_sub(label_prefix.len() + pct_suffix.len() + 2)
        .min(40);

    let filled = ((ratio * bar_width as f64) as usize).min(bar_width);
    let empty = bar_width.saturating_sub(filled);

    let bar = format!(
        "{}{}{}{}",
        label_prefix,
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        pct_suffix,
    );

    Line::from(Span::styled(
        bar,
        Style::default().fg(Color::Rgb(160, 100, 220)),
    ))
}

// ── Build Refinery Overlay ────────────────────────────────────────────────────

fn render_build_overlay(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    let build = match &ui.build {
        Some(b) => b,
        None => return,
    };

    let popup_w = area.width.clamp(30, 50);
    let popup_h = area.height.clamp(10, 20);
    let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let popup = Rect::new(popup_x, popup_y, popup_w, popup_h);

    frame.render_widget(Clear, popup);

    let recipes = crate::loom::recipes::all_recipes();

    let (title, lines) = match &build.step {
        crate::loom::BuildStep::SelectRecipe { cursor } => {
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(" T{} Recipes:", build.tier),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            for (i, &ridx) in build.available_recipes.iter().enumerate() {
                let r = &recipes[ridx];
                let marker = if i == *cursor { "\u{25b6} " } else { "  " };
                let color = if i == *cursor {
                    Color::White
                } else {
                    Color::Rgb(140, 110, 170)
                };
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}{} + {} \u{2192} {}",
                        marker,
                        resource_name(&r.input_a),
                        resource_name(&r.input_b),
                        resource_name(&r.output),
                    ),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            let cost = crate::loom::refinery_build_cost_public(build.tier);
            lines.push(Line::from(Span::styled(
                format!(" Build cost: {:.0} of input A resource", cost),
                Style::default().fg(Color::Rgb(100, 80, 130)),
            )));
            (" Build Refinery ", lines)
        }
        crate::loom::BuildStep::SelectSourcesA { cursor, toggle } => {
            let r = &recipes[build.recipe_index];
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(
                    " {} + {} \u{2192} {}",
                    resource_name(&r.input_a),
                    resource_name(&r.input_b),
                    resource_name(&r.output)
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" Select sources for {}:", resource_name(&r.input_a)),
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for (i, src) in build.eligible_sources_a.iter().enumerate() {
                let marker = if i == *cursor { "\u{25b6}" } else { " " };
                let check = if toggle[i] { "[\u{2713}]" } else { "[ ]" };
                let name = source_display_name(src, loom_state);
                let color = if i == *cursor {
                    Color::White
                } else {
                    Color::Rgb(140, 110, 170)
                };
                lines.push(Line::from(Span::styled(
                    format!(" {} {} {}", marker, check, name),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " [Space] Toggle  [Enter] Next",
                Style::default().fg(Color::DarkGray),
            )));
            (" Sources: Input A ", lines)
        }
        crate::loom::BuildStep::SelectSourcesB { cursor, toggle } => {
            let r = &recipes[build.recipe_index];
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(
                    " {} + {} \u{2192} {}",
                    resource_name(&r.input_a),
                    resource_name(&r.input_b),
                    resource_name(&r.output)
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" Select sources for {}:", resource_name(&r.input_b)),
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for (i, src) in build.eligible_sources_b.iter().enumerate() {
                let marker = if i == *cursor { "\u{25b6}" } else { " " };
                let check = if toggle[i] { "[\u{2713}]" } else { "[ ]" };
                let name = source_display_name(src, loom_state);
                let color = if i == *cursor {
                    Color::White
                } else {
                    Color::Rgb(140, 110, 170)
                };
                lines.push(Line::from(Span::styled(
                    format!(" {} {} {}", marker, check, name),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " [Space] Toggle  [Enter] Next",
                Style::default().fg(Color::DarkGray),
            )));
            (" Sources: Input B ", lines)
        }
        crate::loom::BuildStep::Confirm => {
            let r = &recipes[build.recipe_index];
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(
                    " {} + {} \u{2192} {}",
                    resource_name(&r.input_a),
                    resource_name(&r.input_b),
                    resource_name(&r.output)
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Sources A:",
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for src in &build.selected_sources_a {
                lines.push(Line::from(Span::styled(
                    format!("   {}", source_display_name(src, loom_state)),
                    Style::default().fg(Color::Rgb(120, 100, 160)),
                )));
            }
            lines.push(Line::from(Span::styled(
                " Sources B:",
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for src in &build.selected_sources_b {
                lines.push(Line::from(Span::styled(
                    format!("   {}", source_display_name(src, loom_state)),
                    Style::default().fg(Color::Rgb(120, 100, 160)),
                )));
            }
            lines.push(Line::from(""));
            let cost = crate::loom::refinery_build_cost_public(build.tier);
            let stockpile = loom_state
                .persistent
                .stockpiles
                .get(&r.input_a)
                .copied()
                .unwrap_or(0.0);
            let can_afford = stockpile >= cost;
            let cost_color = if can_afford {
                Color::Rgb(100, 180, 100)
            } else {
                Color::Rgb(180, 80, 80)
            };
            lines.push(Line::from(Span::styled(
                format!(
                    " Cost: {:.0} {} (have {:.1})",
                    cost,
                    resource_name(&r.input_a),
                    stockpile
                ),
                Style::default().fg(cost_color),
            )));
            lines.push(Line::from(""));
            if can_afford {
                lines.push(Line::from(Span::styled(
                    " [Enter] Build  [Esc] Cancel",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    " Insufficient resources. [Esc] Cancel",
                    Style::default().fg(Color::Rgb(180, 80, 80)),
                )));
            }
            (" Confirm Build ", lines)
        }
        crate::loom::BuildStep::Blocked { message } => {
            let mut lines = Vec::new();
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" {}", message),
                Style::default().fg(Color::Rgb(180, 120, 80)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Press any key to dismiss",
                Style::default().fg(Color::DarkGray),
            )));
            (" Cannot Build ", lines)
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(160, 120, 200)));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn source_display_name(src: &crate::loom::types::LoomNodeRef, loom_state: &LoomState) -> String {
    match src {
        crate::loom::types::LoomNodeRef::Extractor(node_id) => {
            let rate = loom_state
                .persistent
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .map(|n| crate::loom::node_effective_rate(loom_state, n))
                .unwrap_or(0.0);
            format!("{} ({:.1}/hr)", node_id.name(), rate)
        }
        crate::loom::types::LoomNodeRef::Refinery(idx) => {
            if let Some(r) = loom_state.persistent.refineries.get(*idx) {
                format!("R{} {} ({:.1}x)", idx, resource_name(&r.output), r.amount)
            } else {
                format!("R{} (unknown)", idx)
            }
        }
    }
}

// ── Navigation hints ──────────────────────────────────────────────────────────

fn render_nav_hints(frame: &mut Frame, area: Rect, ui: &LoomUiState) {
    if area.height < 3 {
        return;
    }

    let hints = if ui.build.is_some() {
        " [Up/Down] Select  [Space] Toggle  [Enter] Confirm  [Esc] Cancel "
    } else if ui.view == LoomView::ArchetypeSelection {
        " [Up/Down] Select  [Enter] Confirm  [Esc] Close "
    } else if ui.view == LoomView::ListDetail {
        " [Tab] Switch View  [Up/Down] Node  [U] Upgrade  [B] Build  [Esc] Close "
    } else if ui.view == LoomView::FlowView {
        " [Tab] Switch View  [Arrows] Navigate  [U] Upgrade  [B] Build  [D] Demolish  [Esc] Close "
    } else {
        " [Tab] Switch View  [Up/Down] Navigate  [Esc] Close "
    };

    let hint_line = Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)));

    let hint_area = Rect::new(
        area.x + 1,
        area.y + area.height - 1,
        area.width.saturating_sub(2),
        1,
    );

    let para = Paragraph::new(hint_line).alignment(Alignment::Center);
    frame.render_widget(para, hint_area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn resource_name(resource: &crate::loom::types::Resource) -> &'static str {
    use crate::loom::types::Resource;
    match resource {
        Resource::Ember => "Ember",
        Resource::Reflection => "Reflection",
        Resource::VoidEssence => "VoidEssence",
        Resource::Memory => "Memory",
        Resource::Silence => "Silence",
        Resource::Resonance => "Resonance",
        Resource::ForgedLight => "ForgedLight",
        Resource::EchoGlass => "EchoGlass",
        Resource::StillbornSong => "StillbornSong",
        Resource::CondensedEmber => "CondensedEmber",
        Resource::EmberEcho => "EmberEcho",
        Resource::PurifiedVoid => "PurifiedVoid",
        Resource::WovenReality => "WovenReality",
    }
}

// ── Discovery modal ─────────────────────────────────────────────────────────

pub fn render_loom_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
    let modal_width = 56u16.min(area.width.saturating_sub(4));
    let modal_height = 12u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{25b6} New System Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(LOOM_BORDER_COLOR)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        LOOM_BORDER_COLOR,
        super::BorderFxContext,
    );

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "\u{1f9f5} The Loom of Worlds awakens!",
            Style::default().fg(Color::LightMagenta),
        )),
        Line::from(""),
        Line::from("A vast pipeline network hums with"),
        Line::from("potential. Six nodes await your command."),
        Line::from(""),
        Line::from(Span::styled(
            "Press [L] to open the Loom.",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter/Esc] Dismiss",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
