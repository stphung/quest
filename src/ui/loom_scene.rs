//! The Loom of Worlds overlay — main UI renderer.
#![allow(dead_code)]
//!
//! Dispatches to different view renderers based on `LoomUiState::view`:
//!   - ArchetypeSelection: choose your archetype at unlock
//!   - FlowView:           pipeline diagram placeholder
//!   - ListDetail:         node list + detail panel placeholder
//!   - Codex:              recipe codex placeholder

use crate::loom::patterns::all_patterns_complete;
use crate::loom::types::{LoomArchetype, LoomState, LoomUiState, LoomView};
use crate::ui::scene_fx::current_millis;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};

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

/// Emoji icon for each resource, used as visual shorthand in sidebar and recipes.
fn resource_emoji(resource: &crate::loom::types::Resource) -> &'static str {
    use crate::loom::types::Resource;
    match resource {
        Resource::Ember => "\u{1f525}",          // 🔥
        Resource::Reflection => "\u{1f48e}",     // 💎
        Resource::VoidEssence => "\u{1f300}",    // 🌀
        Resource::Memory => "\u{1f4dc}",         // 📜
        Resource::Silence => "\u{1f311}",        // 🌑
        Resource::Resonance => "\u{1f514}",      // 🔔
        Resource::ForgedLight => "\u{2728}",     // ✨
        Resource::EchoGlass => "\u{1fa9e}",      // 🪞
        Resource::StillbornSong => "\u{1f3b5}",  // 🎵
        Resource::CondensedEmber => "\u{1f536}", // 🔶
        Resource::EmberEcho => "\u{1f538}",      // 🔸
        Resource::PurifiedVoid => "\u{1f49c}",   // 💜
        Resource::WovenReality => "\u{1f310}",   // 🌐
    }
}

/// Emoji icon for a node identity.
fn node_emoji(id: crate::loom::types::NodeId) -> &'static str {
    use crate::loom::types::NodeId;
    match id {
        NodeId::EmberSpindle => "\u{1f525}",   // 🔥
        NodeId::ReflectionLens => "\u{1f48e}", // 💎
        NodeId::VoidCondenser => "\u{1f300}",  // 🌀
        NodeId::MemoryArchive => "\u{1f4dc}",  // 📜
        NodeId::SilenceWell => "\u{1f311}",    // 🌑
        NodeId::ResonanceForge => "\u{1f514}", // 🔔
    }
}

/// Number of completed patterns determines which recipe tiers are visible.
fn visible_recipe_tier(completed_patterns: usize) -> u8 {
    if completed_patterns >= 15 {
        3
    } else if completed_patterns >= 8 {
        2
    } else {
        1
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

// ── Factory floor rendering ──────────────────────────────────────────────────

/// Build the animated texture string for an extractor node.
fn node_texture_string(node_id: crate::loom::types::NodeId, width: usize, stalled: bool) -> String {
    use crate::loom::types::NodeId;
    let millis = current_millis();
    let frame_idx = if stalled { 0 } else { (millis / 300) as usize };
    let mut s = String::with_capacity(width);
    for c in 0..width {
        let ch = match node_id {
            NodeId::EmberSpindle => {
                if (frame_idx + c) % 4 == 0 || (frame_idx + c) % 4 == 2 {
                    '~'
                } else {
                    ' '
                }
            }
            NodeId::ReflectionLens => match (frame_idx + c * 3) % 6 {
                0 | 4 => '.',
                1 | 3 => '\u{b7}',
                2 => '*',
                _ => ' ',
            },
            NodeId::VoidCondenser => match (frame_idx + c * 2) % 4 {
                0 => ':',
                2 => '\u{b7}',
                _ => ' ',
            },
            NodeId::MemoryArchive => {
                if (frame_idx + c) % 4 == 0 || (frame_idx + c) % 4 == 2 {
                    '\u{2573}'
                } else {
                    ' '
                }
            }
            NodeId::SilenceWell => {
                let m = (frame_idx + c) % 6;
                if m == 0 || m == 2 || m == 4 {
                    '_'
                } else {
                    ' '
                }
            }
            NodeId::ResonanceForge => {
                if (frame_idx + c) % 4 == 0 || (frame_idx + c) % 4 == 2 {
                    '\u{2248}'
                } else {
                    ' '
                }
            }
        };
        s.push(ch);
    }
    s
}

/// Render a single extractor node as ratatui widgets into a given Rect.
///
/// Layout (4 rows):
///   Row 0: border + title (handled by Block)
///   Row 1: animated texture line
///   Row 2: resource + gauge bar (or locked-node info)
///   Row 3: bottom border (handled by Block)
fn render_node_widget(
    frame: &mut Frame,
    area: Rect,
    node: &crate::loom::types::LoomNode,
    loom_state: &LoomState,
    selected: bool,
) {
    if area.height < 3 || area.width < 6 {
        return;
    }

    let border_color = if selected {
        Color::Rgb(220, 180, 255)
    } else if !node.unlocked {
        Color::Rgb(40, 30, 55)
    } else {
        Color::Rgb(80, 60, 110)
    };
    let title_color = if selected {
        Color::White
    } else if !node.unlocked {
        Color::Rgb(60, 45, 80)
    } else {
        node_color(node.id)
    };

    let emoji = node_emoji(node.id);
    let title = if node.unlocked {
        format!(" {} {} Lv.{} ", emoji, node.id.name(), node.level)
    } else {
        format!(" {} {} ", emoji, node.id.name())
    };

    let border_type = if selected {
        ratatui::widgets::BorderType::Thick
    } else {
        ratatui::widgets::BorderType::Rounded
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(title_color)))
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(LOOM_BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let inner_w = inner.width as usize;

    if !node.unlocked {
        // Locked node: line 1 = source hint, line 2 = progress.
        let neighbors = crate::loom::node_neighbors(node.id);
        let feeding: Vec<&crate::loom::types::NodeId> = neighbors
            .iter()
            .filter(|nid| {
                loom_state
                    .persistent
                    .nodes
                    .iter()
                    .any(|n| n.id == **nid && n.unlocked)
            })
            .collect();

        let hint = if !feeding.is_empty() {
            format!("\u{2190} {} (50%)", feeding[0].name())
        } else {
            "dormant".to_string()
        };
        let hint_color = if !feeding.is_empty() {
            Color::Rgb(80, 60, 120)
        } else {
            Color::Rgb(50, 38, 65)
        };

        let progress_text = if node.unlock_progress > 0.0 {
            format!("{:.1}/2.0h unlocking", node.unlock_progress)
        } else if feeding.is_empty() {
            "no active neighbors".to_string()
        } else {
            "waiting for 50% buffer".to_string()
        };
        let progress_color = if node.unlock_progress > 0.0 {
            Color::Rgb(100, 80, 160)
        } else {
            Color::Rgb(50, 38, 65)
        };

        let lines = vec![
            Line::from(Span::styled(hint, Style::default().fg(hint_color))),
            Line::from(Span::styled(
                progress_text,
                Style::default().fg(progress_color),
            )),
        ];
        let para = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(LOOM_BG));
        frame.render_widget(para, inner);
    } else {
        // Unlocked node: line 1 = texture, line 2 = resource + gauge.
        let texture_color = if node.stalled {
            Color::Rgb(40, 30, 55)
        } else {
            node_color(node.id)
        };
        let texture = node_texture_string(node.id, inner_w, node.stalled);

        // Split inner into texture row (1) and data row (rest).
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner);

        let texture_para = Paragraph::new(Line::from(Span::styled(
            texture,
            Style::default().fg(texture_color),
        )))
        .style(Style::default().bg(LOOM_BG));
        frame.render_widget(texture_para, rows[0]);

        // Data row: "Ember 50/hr" label + Gauge for buffer.
        let resource = crate::loom::logic::node_native_resource(node.id);
        let res_name = resource_name(&resource);
        let rate = node.base_rate * crate::loom::logic::node_level_multiplier(node.level);
        let label = format!("{} {:.0}/hr", res_name, rate);
        let label_len = (label.len() + 1) as u16;

        let data_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(label_len), Constraint::Min(4)])
            .split(rows[1]);

        let label_para = Paragraph::new(Span::styled(
            label,
            Style::default().fg(Color::Rgb(140, 120, 170)),
        ))
        .style(Style::default().bg(LOOM_BG));
        frame.render_widget(label_para, data_cols[0]);

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
        let gauge_label = format!("{:.0}/{:.0}", node.buffer, node.buffer_capacity);
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)))
            .label(Span::styled(
                gauge_label,
                Style::default().fg(Color::Rgb(200, 180, 220)),
            ))
            .ratio(fill);
        frame.render_widget(gauge, data_cols[1]);
    }
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

/// Render a single refinery node as ratatui widgets into a given Rect.
fn render_refinery_widget(
    frame: &mut Frame,
    area: Rect,
    refinery: &crate::loom::types::Refinery,
    selected: bool,
    index: usize,
    throbber_frame: u32,
) {
    if area.height < 3 || area.width < 6 {
        return;
    }

    let border_color = if selected {
        Color::Rgb(200, 170, 240)
    } else {
        Color::Rgb(50, 35, 65)
    };
    let border_type = if selected {
        ratatui::widgets::BorderType::Thick
    } else {
        ratatui::widgets::BorderType::Rounded
    };

    // Title: "R0 T1 ForgedLight" or "R0 [Building... 3s]"
    let title = if refinery.under_construction {
        let secs = refinery.construction_ticks_remaining / 10;
        format!(" R{} [Building... {}s] ", index, secs)
    } else {
        let out_name = resource_name(&refinery.output);
        format!(" R{} T{} {} ", index, refinery.tier, out_name)
    };

    let title_color = if refinery.under_construction {
        Color::Rgb(100, 80, 130)
    } else if selected {
        Color::White
    } else {
        Color::Rgb(160, 130, 190)
    };
    let tier_color = match refinery.tier {
        1 => Color::Rgb(100, 160, 100),
        2 => Color::Rgb(160, 140, 80),
        _ => Color::Rgb(180, 100, 200),
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(title_color)))
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(LOOM_BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 || refinery.under_construction {
        return;
    }

    // Build content lines for the inner area.
    let mut content_lines: Vec<Line> = Vec::new();

    // Line 1: throbber + recipe summary
    let throb = throbber_char(throbber_frame, refinery.tier, refinery.stalled);
    let throb_color = if refinery.stalled {
        Color::Rgb(160, 60, 60)
    } else {
        Color::Rgb(140, 100, 200)
    };
    let recipe_emoji = format!(
        "{}+{}\u{25b6}{}",
        resource_emoji(&refinery.input_a),
        resource_emoji(&refinery.input_b),
        resource_emoji(&refinery.output),
    );
    let stall_suffix = if refinery.stalled {
        " \u{26a0}STALL"
    } else {
        ""
    };
    content_lines.push(Line::from(vec![
        Span::styled(format!("{} ", throb), Style::default().fg(throb_color)),
        Span::styled(
            format!("T{} ", refinery.tier),
            Style::default().fg(tier_color),
        ),
        Span::styled(recipe_emoji, Style::default().fg(Color::Rgb(160, 130, 190))),
        Span::styled(
            stall_suffix.to_string(),
            Style::default().fg(Color::Rgb(180, 80, 80)),
        ),
    ]));

    // Line 2: source badges
    let source_label_color = Color::Rgb(100, 80, 140);
    let mut source_spans: Vec<Span> = Vec::new();
    for src in &refinery.sources_a {
        let short_src = match src {
            crate::loom::types::LoomNodeRef::Extractor(_) => noderef_short(*src, 0),
            crate::loom::types::LoomNodeRef::Refinery(ri) => format!("R{}", ri),
        };
        let res_short = resource_short(&refinery.input_a);
        source_spans.push(Span::styled(
            format!("{}\u{2190}[{}] ", res_short, short_src),
            Style::default().fg(source_label_color),
        ));
    }
    for src in &refinery.sources_b {
        let short_src = match src {
            crate::loom::types::LoomNodeRef::Extractor(_) => noderef_short(*src, 0),
            crate::loom::types::LoomNodeRef::Refinery(ri) => format!("R{}", ri),
        };
        let res_short = resource_short(&refinery.input_b);
        source_spans.push(Span::styled(
            format!("{}\u{2190}[{}] ", res_short, short_src),
            Style::default().fg(source_label_color),
        ));
    }
    if source_spans.is_empty() {
        source_spans.push(Span::styled(
            "no sources".to_string(),
            Style::default().fg(Color::Rgb(60, 45, 80)),
        ));
    }
    content_lines.push(Line::from(source_spans));

    // Render text content (lines 1-2).
    let text_h = content_lines.len() as u16;
    if inner.height > 1 {
        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(1).min(text_h),
        };
        let para = Paragraph::new(content_lines).style(Style::default().bg(LOOM_BG));
        frame.render_widget(para, content_area);
    }

    // Last row: buffer gauge.
    if inner.height >= 2 {
        let gauge_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
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
        let gauge_label = format!("{:.0}/{:.0}", refinery.buffer, refinery.buffer_capacity);
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)))
            .label(Span::styled(
                gauge_label,
                Style::default().fg(Color::Rgb(200, 180, 220)),
            ))
            .ratio(fill_pct);
        frame.render_widget(gauge, gauge_area);
    }
}

/// Render the sidebar detail panel for the selected node (Option H layout).
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
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(80, 60, 110)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 || inner.width < 8 {
        return;
    }

    // Split: header (3 lines) | gauge (1 line) | body (rest).
    let gauge_row_count = if node.unlocked { 1u16 } else { 0 };
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),               // header: emoji+name, rate/level
            Constraint::Length(gauge_row_count), // gauge bar
            Constraint::Min(0),                  // recipes + upgrade
        ])
        .split(inner);
    let header_area = v_chunks[0];
    let gauge_area = v_chunks[1];
    let body_area = v_chunks[2];

    // ── Header ──
    let emoji = node_emoji(selected_id);
    let mut header_lines = vec![Line::from(Span::styled(
        format!("{} {}", emoji, selected_id.name()),
        Style::default().fg(node_color(selected_id)),
    ))];

    if !node.unlocked {
        header_lines.push(Line::from(Span::styled(
            " [Locked]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        if node.unlock_progress > 0.0 {
            header_lines.push(Line::from(Span::styled(
                format!(" {:.1}/2.0h unlocking", node.unlock_progress),
                Style::default().fg(Color::Rgb(100, 80, 160)),
            )));
        } else {
            header_lines.push(Line::from(""));
        }
    } else {
        let rate = node_effective_rate(loom_state, node);
        header_lines.push(Line::from(Span::styled(
            format!(" +{:.0}/hr", rate),
            Style::default().fg(Color::Rgb(100, 200, 120)),
        )));
        header_lines.push(Line::from(""));
    }
    frame.render_widget(
        Paragraph::new(header_lines).style(Style::default().bg(LOOM_BG)),
        header_area,
    );

    // ── Gauge (unlocked only) ──
    if node.unlocked && gauge_area.height > 0 {
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
        let label = format!("{:.0}/{:.0}", node.buffer, node.buffer_capacity);
        let gauge = Gauge::default()
            .ratio(fill)
            .label(label)
            .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)));
        frame.render_widget(gauge, gauge_area);
    }

    // ── Body ──
    let mut lines: Vec<Line> = Vec::new();

    if !node.unlocked {
        // Locked node body: neighbor info.
        let neighbors = crate::loom::node_neighbors(node.id);
        let unlocked_neighbors: Vec<&crate::loom::types::NodeId> = neighbors
            .iter()
            .filter(|nid| {
                loom_state
                    .persistent
                    .nodes
                    .iter()
                    .any(|n| n.id == **nid && n.unlocked)
            })
            .collect();
        lines.push(Line::from(""));
        if !unlocked_neighbors.is_empty() {
            lines.push(Line::from(Span::styled(
                " Active neighbors:",
                Style::default().fg(Color::Rgb(80, 60, 110)),
            )));
            for nid in &unlocked_neighbors {
                lines.push(Line::from(Span::styled(
                    format!("  {} {}", node_emoji(**nid), nid.name()),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                )));
            }
        } else {
            lines.push(Line::from(Span::styled(
                " No active neighbors",
                Style::default().fg(Color::Rgb(60, 45, 80)),
            )));
        }
    } else {
        // Unlocked node body: recipes + upgrade.
        let nature = node.id.nature();
        let max_tier = visible_recipe_tier(
            loom_state
                .persistent
                .patterns
                .iter()
                .filter(|p| p.completed)
                .count(),
        );
        let recipes: Vec<_> = recipes_by_nature(nature)
            .into_iter()
            .filter(|r| r.tier <= max_tier)
            .collect();

        lines.push(Line::from(""));
        if recipes.is_empty() {
            lines.push(Line::from(Span::styled(
                " No recipes yet",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!(" {} recipes:", node_nature_name(nature)),
                Style::default().fg(Color::Rgb(140, 100, 180)),
            )));
            for r in &recipes {
                let ea = resource_emoji(&r.input_a);
                let eb = resource_emoji(&r.input_b);
                let eo = resource_emoji(&r.output);
                let out_name = resource_name(&r.output);
                lines.push(Line::from(Span::styled(
                    format!(" {}+{}\u{2192}{} {}", ea, eb, eo, out_name),
                    Style::default().fg(Color::Rgb(120, 100, 160)),
                )));
            }
        }

        // Upgrade.
        lines.push(Line::from(""));
        let cost = crate::loom::node_upgrade_cost(loom_state, node.id);
        let can_afford = node.buffer >= cost;
        let resource = crate::loom::logic::node_native_resource(node.id);
        let re = resource_emoji(&resource);
        let cost_color = if can_afford {
            Color::Rgb(100, 200, 120)
        } else {
            Color::Rgb(80, 60, 100)
        };
        lines.push(Line::from(vec![
            Span::styled(
                " [U] ",
                Style::default().fg(if can_afford {
                    Color::Rgb(200, 180, 240)
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(
                format!("{:.0}{} to Lv{}", cost, re, node.level + 1),
                Style::default().fg(cost_color),
            ),
        ]));
    }

    lines.truncate(body_area.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        body_area,
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
                format!(" {:.0}/{:.0}", refinery.buffer, refinery.buffer_capacity),
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

    // Split: factory floor (left) | sidebar (right, 28 cols).
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(28)])
        .split(area);
    let floor_area = h_chunks[0];
    let sidebar_area = h_chunks[1];

    // Split factory floor: node grid (top) | pattern bar (bottom).
    let has_patterns = !loom_state.persistent.patterns.is_empty();
    let req_count = loom_state
        .persistent
        .patterns
        .get(loom_state.persistent.active_pattern)
        .map(|p| p.requirements.len())
        .unwrap_or(0);
    let pattern_h = if has_patterns {
        // Bordered block: 2 (borders) + req_count + 1 (blank) + 1 (overall gauge).
        (4 + req_count as u16).min(14)
    } else {
        0u16
    };
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(pattern_h)])
        .split(floor_area);

    let grid_area = v_chunks[0];
    let pattern_area = v_chunks[1];

    // Fill grid background.
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(LOOM_BG)),
        grid_area,
    );

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

    let grid_ids = [
        NodeId::EmberSpindle,   // 0: top-left
        NodeId::ReflectionLens, // 1: top-right
        NodeId::MemoryArchive,  // 2: mid-left
        NodeId::VoidCondenser,  // 3: mid-right
        NodeId::SilenceWell,    // 4: bottom-left
        NodeId::ResonanceForge, // 5: bottom-right
    ];
    let selected_id = grid_ids[ui.selected_node.min(5)];

    // Determine refinery section height.
    let refineries = &loom_state.persistent.refineries;
    let refinery_row_count = if refineries.is_empty() {
        0u16
    } else {
        let refinery_grid_rows = refineries.len().div_ceil(2);
        // 1 for separator + rows * (NODE_BOX_HEIGHT+2) for refinery boxes
        1 + (refinery_grid_rows as u16) * (NODE_BOX_HEIGHT as u16 + 2)
    };

    // Split grid_area into 3 extractor rows + arrow gaps + refinery section.
    // Each extractor row = NODE_BOX_HEIGHT, each arrow gap = 2 rows (│ + ▼/▲).
    let box_h = NODE_BOX_HEIGHT as u16;
    let gap_h = 2u16;
    let extractor_total_h = 3 * box_h + 2 * gap_h;
    let content_h = extractor_total_h + refinery_row_count;

    // Vertical centering: split remaining space equally above and below.
    let v_pad = grid_area.height.saturating_sub(content_h) / 2;

    let mut row_constraints: Vec<Constraint> = Vec::new();
    row_constraints.push(Constraint::Length(v_pad)); // top padding
    for i in 0..3 {
        row_constraints.push(Constraint::Length(box_h)); // node row
        if i < 2 {
            row_constraints.push(Constraint::Length(gap_h)); // arrow gap
        }
    }
    if refinery_row_count > 0 {
        row_constraints.push(Constraint::Length(refinery_row_count));
    }
    row_constraints.push(Constraint::Min(0)); // absorb remaining space (bottom padding)

    let row_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_area);

    // Indices into row_rects: 0=top pad, 1=extractor row 0, 2=gap, 3=extractor row 1, 4=gap, 5=extractor row 2
    // Then 6=refinery section (if present), last=spacer
    let extractor_row_rects = [row_rects[1], row_rects[3], row_rects[5]];
    let gap_rects = [row_rects[2], row_rects[4]];

    let flow_color = Color::Rgb(60, 45, 80);
    let flow_arrow_color = Color::Rgb(120, 80, 180);

    // Horizontal centering: compute padding to center the 2-column grid.
    let node_w = NODE_BOX_WIDTH as u16;
    let h_gap_w = 4u16.min(grid_area.width.saturating_sub(node_w * 2)); // 4-col gap between nodes
    let grid_total_w = node_w * 2 + h_gap_w;
    let h_pad = grid_area.width.saturating_sub(grid_total_w) / 2;

    // Render each extractor row (2 nodes + horizontal arrow gap).
    for (row_idx, (left_id, right_id)) in grid.iter().enumerate() {
        let row_area = extractor_row_rects[row_idx];

        // Split row into: pad | left_node | h_gap | right_node | pad
        let row_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(h_pad),
                Constraint::Length(node_w),
                Constraint::Length(h_gap_w),
                Constraint::Length(node_w),
                Constraint::Min(0),
            ])
            .split(row_area);
        let left_rect = row_cols[1];
        let h_gap_rect = row_cols[2];
        let right_rect = row_cols[3];

        // Render left node.
        if let Some(left_node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *left_id)
        {
            let is_sel = ui.selected_node < 6 && *left_id == selected_id;
            render_node_widget(frame, left_rect, left_node, loom_state, is_sel);
        }

        // Render right node.
        if let Some(right_node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *right_id)
        {
            let is_sel = ui.selected_node < 6 && *right_id == selected_id;
            render_node_widget(frame, right_rect, right_node, loom_state, is_sel);
        }

        // Horizontal arrow in gap (only rows 0 and 2).
        if h_gap_rect.width > 0 && h_gap_rect.height > 0 {
            // Place arrow at vertical center of the gap.
            let arrow_y = h_gap_rect.y + h_gap_rect.height / 2;
            let arrow_area = Rect {
                x: h_gap_rect.x,
                y: arrow_y,
                width: h_gap_rect.width,
                height: 1,
            };
            let arrow_text = match row_idx {
                0 => {
                    // ES ──▶ RL
                    let w = arrow_area.width as usize;
                    let mut s = "\u{2500}".repeat(w.saturating_sub(1));
                    s.push('\u{25b6}');
                    Line::from(vec![
                        Span::styled(
                            s[..s.len().saturating_sub(3)].to_string(),
                            Style::default().fg(flow_color),
                        ),
                        Span::styled(
                            "\u{25b6}".to_string(),
                            Style::default().fg(flow_arrow_color),
                        ),
                    ])
                }
                2 => {
                    // SW ◀── RF
                    let w = arrow_area.width as usize;
                    let dashes = "\u{2500}".repeat(w.saturating_sub(1));
                    Line::from(vec![
                        Span::styled(
                            "\u{25c0}".to_string(),
                            Style::default().fg(flow_arrow_color),
                        ),
                        Span::styled(dashes, Style::default().fg(flow_color)),
                    ])
                }
                _ => Line::from(""),
            };
            let para = Paragraph::new(arrow_text).style(Style::default().bg(LOOM_BG));
            frame.render_widget(para, arrow_area);
        }
    }

    // Vertical arrows in the gap rows between extractor rows.
    // With gap_h=2, we render: row 0 = │ (pipe), row 1 = ▼ or ▲ (arrowhead).
    for gap_rect in &gap_rects {
        if gap_rect.width == 0 || gap_rect.height == 0 {
            continue;
        }
        // Split gap the same way as node rows to find node centers.
        let gap_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(h_pad),
                Constraint::Length(node_w),
                Constraint::Length(h_gap_w),
                Constraint::Length(node_w),
                Constraint::Min(0),
            ])
            .split(*gap_rect);

        // Right side: down arrows (RL→VC, VC→RF).
        let right_center_x = gap_cols[3].x + gap_cols[3].width / 2;
        if right_center_x >= gap_rect.x && right_center_x < gap_rect.x + gap_rect.width {
            // Pipe character on first row of gap.
            if gap_rect.height >= 2 {
                let pipe_rect = Rect {
                    x: right_center_x,
                    y: gap_rect.y,
                    width: 1,
                    height: 1,
                };
                let pipe =
                    Paragraph::new(Span::styled("\u{2502}", Style::default().fg(flow_color)))
                        .style(Style::default().bg(LOOM_BG));
                frame.render_widget(pipe, pipe_rect);
            }
            // Arrowhead on last row of gap.
            let arrow_rect = Rect {
                x: right_center_x,
                y: gap_rect.y + gap_rect.height - 1,
                width: 1,
                height: 1,
            };
            let para = Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG));
            frame.render_widget(para, arrow_rect);
        }

        // Left side: up arrows (SW→MA, MA→ES).
        let left_center_x = gap_cols[1].x + gap_cols[1].width / 2;
        if left_center_x >= gap_rect.x && left_center_x < gap_rect.x + gap_rect.width {
            // Arrowhead on first row of gap.
            let arrow_rect = Rect {
                x: left_center_x,
                y: gap_rect.y,
                width: 1,
                height: 1,
            };
            let para = Paragraph::new(Span::styled(
                "\u{25b2}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG));
            frame.render_widget(para, arrow_rect);
            // Pipe character on second row of gap.
            if gap_rect.height >= 2 {
                let pipe_rect = Rect {
                    x: left_center_x,
                    y: gap_rect.y + gap_rect.height - 1,
                    width: 1,
                    height: 1,
                };
                let pipe =
                    Paragraph::new(Span::styled("\u{2502}", Style::default().fg(flow_color)))
                        .style(Style::default().bg(LOOM_BG));
                frame.render_widget(pipe, pipe_rect);
            }
        }
    }

    // ── Refineries: render below the extractor grid ──────────────────────────
    if !refineries.is_empty() {
        let refinery_section = row_rects[6]; // index 6 = after top pad + 3 extractor rows + 2 gaps

        // Separator label.
        let sep_rect = Rect {
            x: refinery_section.x,
            y: refinery_section.y,
            width: refinery_section.width,
            height: 1,
        };
        let sep = Paragraph::new(Line::from(Span::styled(
            "\u{2500}\u{2500} Processing \u{2500}\u{2500}",
            Style::default().fg(Color::Rgb(80, 60, 100)),
        )))
        .style(Style::default().bg(LOOM_BG));
        frame.render_widget(sep, sep_rect);

        // Refinery boxes in 2-column grid below separator.
        let ref_box_h = NODE_BOX_HEIGHT as u16 + 2; // slightly taller for content

        for (i, refinery) in refineries.iter().enumerate() {
            let grid_row = i / 2;
            let grid_col = i % 2;

            let ref_y = refinery_section.y + 1 + (grid_row as u16) * ref_box_h;
            if ref_y + ref_box_h > refinery_section.y + refinery_section.height {
                break; // out of visible area
            }

            let ref_x = if grid_col == 0 {
                refinery_section.x
            } else {
                refinery_section.x + refinery_section.width.saturating_sub(node_w)
            };

            let ref_rect = Rect {
                x: ref_x,
                y: ref_y,
                width: node_w.min(refinery_section.width),
                height: ref_box_h,
            };

            let is_sel = ui.selected_node >= 6 && (ui.selected_node - 6) == i;
            render_refinery_widget(frame, ref_rect, refinery, is_sel, i, ui.throbber_frame);
        }
    }

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
                    " {:.0}/{:.0}",
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
                format!("{:.0}/hr", rate),
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
    if area.height < 3 {
        return;
    }

    let all_done = all_patterns_complete(&loom_state.persistent);

    if all_done {
        let block = Block::default()
            .title(Span::styled(
                " \u{2728} Loom Mended \u{2014} All 18 Patterns Complete ",
                Style::default().fg(Color::Rgb(255, 215, 0)),
            ))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(100, 60, 140)))
            .style(Style::default().bg(LOOM_BG));
        frame.render_widget(block, area);
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

    // Bordered block with pattern name as title.
    let title = format!(
        " Pattern {}/{}: \"{}\" ",
        active_idx + 1,
        pattern_count,
        pattern.name
    );
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(Color::Rgb(200, 160, 240)),
        ))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(100, 60, 140)))
        .style(Style::default().bg(LOOM_BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Layout: per-requirement gauge rows + blank line + overall gauge.
    let req_count = pattern.requirements.len();
    let mut constraints: Vec<Constraint> = Vec::with_capacity(req_count + 2);
    for _ in 0..req_count {
        constraints.push(Constraint::Length(1)); // per-resource gauge
    }
    constraints.push(Constraint::Length(1)); // blank separator
    constraints.push(Constraint::Length(1)); // overall gauge
    constraints.push(Constraint::Min(0)); // absorb extra

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Rows 0..N: per-resource Gauge.
    let label_w = 14u16; // width for "emoji+name" column
    for (i, req) in pattern.requirements.iter().enumerate() {
        let row_area = rows[i];
        if row_area.height == 0 {
            continue;
        }

        let ratio = if req.amount > 0.0 {
            (req.accumulated / req.amount).min(1.0)
        } else {
            1.0
        };
        let met = req.accumulated >= req.amount;

        // Split: label | gauge | count.
        let count_label = format!("{:.0}/{:.0}", req.accumulated, req.amount);
        let count_w = count_label.len() as u16 + 3; // space + text + check
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(label_w),
                Constraint::Min(6),
                Constraint::Length(count_w),
            ])
            .split(row_area);

        // Label: emoji + resource name.
        let emoji = resource_emoji(&req.resource);
        let res_name = resource_name(&req.resource);
        let label_text = format!(" {} {}", emoji, res_name);
        let label_color = if met {
            Color::Rgb(80, 200, 120)
        } else {
            Color::Rgb(180, 150, 210)
        };
        frame.render_widget(
            Paragraph::new(Span::styled(label_text, Style::default().fg(label_color)))
                .style(Style::default().bg(LOOM_BG)),
            cols[0],
        );

        // Gauge.
        let gauge_color = if met {
            Color::Rgb(80, 200, 120)
        } else {
            Color::Rgb(160, 100, 220)
        };
        let gauge = Gauge::default()
            .ratio(ratio)
            .gauge_style(Style::default().fg(gauge_color).bg(Color::Rgb(30, 20, 40)));
        frame.render_widget(gauge, cols[1]);

        // Count + check mark.
        let check = if met { " \u{2713}" } else { "" };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {}{}", count_label, check),
                Style::default().fg(label_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            cols[2],
        );
    }

    // Blank separator row (rows[req_count]) — just background, nothing to render.

    // Overall progress row.
    let overall_row = rows[req_count + 1];
    if overall_row.height > 0 {
        let overall_ratio = if pattern.requirements.is_empty() {
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
                .sum::<f64>()
                / pattern.requirements.len() as f64
        };

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(label_w),
                Constraint::Min(6),
                Constraint::Length(6),
            ])
            .split(overall_row);

        frame.render_widget(
            Paragraph::new(Span::styled(
                " Overall",
                Style::default().fg(Color::Rgb(200, 160, 240)),
            ))
            .style(Style::default().bg(LOOM_BG)),
            cols[0],
        );

        let label = format!("{:.0}%", overall_ratio * 100.0);
        let gauge = Gauge::default()
            .ratio(overall_ratio.min(1.0))
            .label(label)
            .gauge_style(
                Style::default()
                    .fg(Color::Rgb(200, 160, 240))
                    .bg(Color::Rgb(30, 20, 40)),
            );
        frame.render_widget(gauge, cols[1]);
    }
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
                    " Cost: {:.0} {} (have {:.0})",
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
            format!("{} ({:.0}/hr)", node_id.name(), rate)
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
