//! The Loom of Worlds overlay — main UI renderer.
#![allow(dead_code)]
//!
//! Dispatches to different view renderers based on `LoomUiState::view`:
//!   - FlowView:           pipeline diagram with extractors and shuttles
//!   - Codex:              recipe codex

use crate::loom::patterns::all_patterns_complete;
use crate::loom::types::{LoomState, LoomUiState, LoomView};
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

/// Extract NodeId from a LoomNodeRef, returning None for Shuttles.
fn noderef_to_id(r: crate::loom::types::LoomNodeRef) -> Option<crate::loom::types::NodeId> {
    match r {
        crate::loom::types::LoomNodeRef::Extractor(id) => Some(id),
        crate::loom::types::LoomNodeRef::Shuttle(_) => None,
    }
}

/// Color for a LoomNodeRef (gray fallback for shuttles).
fn noderef_color(r: crate::loom::types::LoomNodeRef) -> Color {
    match noderef_to_id(r) {
        Some(id) => node_color(id),
        None => Color::Rgb(120, 100, 140),
    }
}

/// Letter for a LoomNodeRef ('?' fallback for shuttles).
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
        None => "Shuttle",
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
        LoomView::FlowView => "Flow View",
        LoomView::Codex => "Recipe Codex",
    };

    let warp_label = if loom_state.time_warp > 1.0 {
        format!(" \u{23e9} {:.0}x ", loom_state.time_warp)
    } else {
        String::new()
    };
    let title = format!(" LOOM OF WORLDS \u{2014} {}{}", view_name, warp_label);
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
        LoomView::FlowView => {
            render_flow_view(frame, inner, loom_state, ui);
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
            format!("woven from {}", feeding[0].name())
        } else {
            "unwoven".to_string()
        };
        let hint_color = if !feeding.is_empty() {
            Color::Rgb(80, 60, 120)
        } else {
            Color::Rgb(50, 38, 65)
        };

        let progress_text = if node.unlock_progress > 0.0 {
            format!("{:.1}/2.0h weaving", node.unlock_progress)
        } else if feeding.is_empty() {
            "unwoven".to_string()
        } else {
            "weaving...".to_string()
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
        LoomNodeRef::Shuttle(_) => format!("R{}", idx),
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

/// Render a single shuttle node as ratatui widgets into a given Rect.
fn render_shuttle_widget(
    frame: &mut Frame,
    area: Rect,
    shuttle: &crate::loom::types::Shuttle,
    selected: bool,
    index: usize,
    throbber_frame: u32,
    time_warp: f64,
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

    // Title: "R0 T1 ForgedLight" or "R0 🔨 ~45m"
    let title = if shuttle.under_construction {
        let warp = time_warp.max(1.0) as u32;
        let total_secs = shuttle.construction_ticks_remaining / (10 * warp);
        let time_str = if total_secs >= 3600 {
            format!("~{}h", total_secs / 3600)
        } else if total_secs >= 60 {
            format!("~{}m", total_secs / 60)
        } else {
            format!("~{}s", total_secs)
        };
        format!(" R{} \u{1f528} {} ", index, time_str)
    } else {
        let out_name = resource_name(&shuttle.output);
        format!(" R{} T{} {} ", index, shuttle.tier, out_name)
    };

    let title_color = if shuttle.under_construction {
        Color::Rgb(100, 80, 130)
    } else if selected {
        Color::White
    } else {
        Color::Rgb(160, 130, 190)
    };
    let tier_color = match shuttle.tier {
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

    if inner.height == 0 || inner.width == 0 || shuttle.under_construction {
        return;
    }

    // Build content lines for the inner area.
    let mut content_lines: Vec<Line> = Vec::new();

    // Line 1: throbber + recipe summary
    let throb = throbber_char(throbber_frame, shuttle.tier, shuttle.stalled);
    let throb_color = if shuttle.stalled {
        Color::Rgb(160, 60, 60)
    } else {
        Color::Rgb(140, 100, 200)
    };
    let recipe_emoji = format!(
        "{}+{}\u{25b6}{}",
        resource_emoji(&shuttle.input_a),
        resource_emoji(&shuttle.input_b),
        resource_emoji(&shuttle.output),
    );
    let stall_suffix = if shuttle.stalled {
        " \u{26a0}STALL"
    } else {
        ""
    };
    content_lines.push(Line::from(vec![
        Span::styled(format!("{} ", throb), Style::default().fg(throb_color)),
        Span::styled(
            format!("T{} ", shuttle.tier),
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
    for src in &shuttle.sources_a {
        let short_src = match src {
            crate::loom::types::LoomNodeRef::Extractor(_) => noderef_short(*src, 0),
            crate::loom::types::LoomNodeRef::Shuttle(ri) => format!("R{}", ri),
        };
        let res_short = resource_name(&shuttle.input_a);
        source_spans.push(Span::styled(
            format!("{}\u{2190}[{}] ", res_short, short_src),
            Style::default().fg(source_label_color),
        ));
    }
    for src in &shuttle.sources_b {
        let short_src = match src {
            crate::loom::types::LoomNodeRef::Extractor(_) => noderef_short(*src, 0),
            crate::loom::types::LoomNodeRef::Shuttle(ri) => format!("R{}", ri),
        };
        let res_short = resource_name(&shuttle.input_b);
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
        let fill_pct = if shuttle.buffer_capacity > 0.0 {
            (shuttle.buffer / shuttle.buffer_capacity).min(1.0)
        } else {
            0.0
        };
        let bar_color = if shuttle.stalled || fill_pct >= 0.90 {
            Color::Rgb(220, 60, 60)
        } else if fill_pct >= 0.75 {
            Color::Rgb(220, 180, 60)
        } else {
            Color::Rgb(60, 200, 100)
        };
        let gauge_label = format!("{:.0}/{:.0}", shuttle.buffer, shuttle.buffer_capacity);
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

    // If selected_node >= 6, show shuttle detail instead of extractor detail.
    if ui.selected_node >= 6 {
        render_flow_sidebar_shuttle(frame, area, loom_state, ui);
        return;
    }

    // Must match grid_ids order in render_flow_grid() so the detail panel
    // shows the correct node for the cursor position.
    let grid_order = [
        NodeId::EmberSpindle,
        NodeId::ReflectionLens,
        NodeId::ResonanceForge,
        NodeId::VoidCondenser,
        NodeId::SilenceWell,
        NodeId::MemoryArchive,
    ];
    let selected_id = grid_order[ui.selected_node.min(grid_order.len() - 1)];
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
            format!(" Lv{} \u{2022} {:.0}/hr", node.level, rate),
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
                " Woven from:",
                Style::default().fg(Color::Rgb(80, 60, 110)),
            )));
            for nid in &unlocked_neighbors {
                lines.push(Line::from(Span::styled(
                    format!("  \u{25bc} {} {}", node_emoji(**nid), nid.name()),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                )));
            }
        } else {
            lines.push(Line::from(Span::styled(
                " Unwoven",
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

        // Consumers: shuttles pulling from this extractor.
        let node_ref = crate::loom::types::LoomNodeRef::Extractor(node.id);
        let consumers: Vec<(usize, &crate::loom::types::Shuttle)> = loom_state
            .persistent
            .shuttles
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                !s.under_construction
                    && (s.sources_a.contains(&node_ref) || s.sources_b.contains(&node_ref))
            })
            .collect();
        if !consumers.is_empty() {
            let rate = crate::loom::logic::node_effective_rate(loom_state, node);
            let total_consumers_on_node = consumers.len() as f64;
            let share = rate / total_consumers_on_node;
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" Consumers ({}):", consumers.len()),
                Style::default().fg(Color::Rgb(140, 100, 180)),
            )));
            for (i, s) in &consumers {
                let cap = crate::loom::logic::shuttle_effective_intake_cap(s.tier, s.level);
                let actual = share.min(cap);
                let eo = resource_emoji(&s.output);
                lines.push(Line::from(Span::styled(
                    format!("  R{} {} {:.0}/hr", i, eo, actual),
                    Style::default().fg(Color::Rgb(120, 100, 160)),
                )));
            }
            let used: f64 = consumers
                .iter()
                .map(|(_, s)| {
                    let cap = crate::loom::logic::shuttle_effective_intake_cap(s.tier, s.level);
                    share.min(cap)
                })
                .sum();
            lines.push(Line::from(Span::styled(
                format!("  Free: {:.0}/hr", (rate - used).max(0.0)),
                Style::default().fg(Color::Rgb(80, 160, 80)),
            )));
        }

        // Upgrade.
        let resource = crate::loom::logic::node_native_resource(node.id);
        let re = resource_emoji(&resource);
        lines.push(Line::from(""));
        if node.level < crate::loom::logic::MAX_NODE_LEVEL {
            let cost = crate::loom::node_upgrade_cost(loom_state, node.id);
            let can_afford = node.buffer >= cost;
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
                    format!("Lv{} ({:.0}{} cost)", node.level + 1, cost, re),
                    Style::default().fg(cost_color),
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                " Max Level",
                Style::default().fg(Color::Rgb(100, 200, 120)),
            )));
        }
    }

    lines.truncate(body_area.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        body_area,
    );
}

/// Render sidebar when a shuttle (selected_node >= 6) is selected.
fn render_flow_sidebar_shuttle(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &LoomUiState,
) {
    let shuttle_idx = ui.selected_node - 6;
    let shuttles = &loom_state.persistent.shuttles;

    let title = if shuttle_idx < shuttles.len() {
        format!(" Shuttle {} ", shuttle_idx)
    } else {
        " Shuttle ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(80, 60, 110)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if shuttle_idx >= shuttles.len() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " [No shuttle]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        lines.truncate(inner.height as usize);
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            inner,
        );
        return;
    }

    let shuttle = &shuttles[shuttle_idx];

    lines.push(Line::from(""));

    if shuttle.under_construction {
        lines.push(Line::from(Span::styled(
            " [Under Construction]",
            Style::default().fg(Color::Rgb(160, 120, 200)),
        )));
        let ticks = shuttle.construction_ticks_remaining;
        let warp = loom_state.time_warp.max(1.0) as u32;
        let secs = ticks / (10 * warp);
        lines.push(Line::from(Span::styled(
            format!(" ~{}s remaining", secs),
            Style::default().fg(Color::Rgb(100, 80, 130)),
        )));
        lines.truncate(inner.height as usize);
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            inner,
        );
        return;
    }

    {
        // Tier and output.
        lines.push(Line::from(vec![
            Span::styled(
                format!(" Tier {}", shuttle.tier),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            ),
            Span::styled(
                format!("  \u{2192} {}", resource_name(&shuttle.output)),
                Style::default().fg(Color::Rgb(220, 180, 255)),
            ),
        ]));

        // Recipe.
        lines.push(Line::from(vec![
            Span::styled(" Recipe: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{} + {} \u{25b6} {}",
                    resource_name(&shuttle.input_a),
                    resource_name(&shuttle.input_b),
                    resource_name(&shuttle.output)
                ),
                Style::default().fg(Color::Rgb(160, 120, 200)),
            ),
        ]));

        // Buffer gauge — rendered separately after the top lines paragraph.
        // We collect pre-gauge lines, render them, render the gauge, then continue with post-gauge lines.
        let pre_gauge_lines = lines;
        let pre_gauge_height = pre_gauge_lines.len() as u16;

        let gauge_height = 1u16;
        let post_start_y = inner.y + pre_gauge_height + gauge_height;
        let post_height = inner.height.saturating_sub(pre_gauge_height + gauge_height);

        // Render pre-gauge content.
        let pre_area = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            pre_gauge_height.min(inner.height),
        );
        frame.render_widget(
            Paragraph::new(pre_gauge_lines).style(Style::default().bg(LOOM_BG)),
            pre_area,
        );

        // Render gauge.
        if pre_gauge_height < inner.height {
            let gauge_area = Rect::new(
                inner.x,
                inner.y + pre_gauge_height,
                inner.width,
                gauge_height,
            );
            let fill = if shuttle.buffer_capacity > 0.0 {
                (shuttle.buffer / shuttle.buffer_capacity).min(1.0)
            } else {
                0.0
            };
            let bar_color = if fill >= 0.90 {
                Color::Rgb(220, 60, 60)
            } else if fill >= 0.75 {
                Color::Rgb(220, 180, 60)
            } else {
                Color::Rgb(60, 200, 100)
            };
            let gauge_label = format!("{:.0}/{:.0}", shuttle.buffer, shuttle.buffer_capacity);
            let gauge = Gauge::default()
                .ratio(fill)
                .label(gauge_label)
                .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)));
            frame.render_widget(gauge, gauge_area);
        }

        // Continue with post-gauge lines.
        let mut lines: Vec<Line> = Vec::new();

        // Amount multiplier.
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Yield: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("x{:.1}/cycle", shuttle.amount),
                Style::default().fg(Color::Rgb(100, 200, 120)),
            ),
        ]));

        // Input pull details — show actual throughput per source.
        let cap = crate::loom::logic::shuttle_effective_intake_cap(shuttle.tier, shuttle.level);

        // Count consumers per source across all active shuttles (mirrors tick logic).
        let consumer_counts: std::collections::HashMap<crate::loom::types::LoomNodeRef, usize> = {
            let mut counts = std::collections::HashMap::new();
            for s in &loom_state.persistent.shuttles {
                if !s.under_construction {
                    for src in s.sources_a.iter().chain(s.sources_b.iter()) {
                        *counts.entry(*src).or_insert(0) += 1;
                    }
                }
            }
            counts
        };

        // Helper: compute pull from a single source.
        let source_pull = |src: &crate::loom::types::LoomNodeRef| -> f64 {
            let available = match src {
                crate::loom::types::LoomNodeRef::Extractor(id) => {
                    let n = &loom_state.persistent.nodes[id.index()];
                    crate::loom::logic::node_effective_rate(loom_state, n)
                }
                crate::loom::types::LoomNodeRef::Shuttle(idx) => {
                    // Approximate: use the shuttle's output rate from rate tracker
                    loom_state
                        .persistent
                        .shuttles
                        .get(*idx)
                        .map(|s| {
                            loom_state
                                .rate_trackers
                                .get(&s.output)
                                .map(|t| t.rate_per_hour())
                                .unwrap_or(0.0)
                        })
                        .unwrap_or(0.0)
                }
            };
            let consumers = *consumer_counts.get(src).unwrap_or(&1) as f64;
            (available / consumers).min(cap)
        };

        let source_display = |src: &crate::loom::types::LoomNodeRef| -> (String, Color) {
            match src {
                crate::loom::types::LoomNodeRef::Extractor(id) => {
                    (id.name().to_string(), noderef_color(*src))
                }
                crate::loom::types::LoomNodeRef::Shuttle(idx) => {
                    let name = loom_state
                        .persistent
                        .shuttles
                        .get(*idx)
                        .map(|s| format!("R{} {}", idx, resource_name(&s.output)))
                        .unwrap_or_else(|| format!("R{}", idx));
                    (name, noderef_color(*src))
                }
            }
        };

        lines.push(Line::from(""));
        if shuttle.sources_a.is_empty() && shuttle.sources_b.is_empty() {
            lines.push(Line::from(Span::styled(
                " No sources assigned",
                Style::default().fg(Color::Rgb(80, 60, 110)),
            )));
        } else {
            // Input A
            let pull_a: f64 = shuttle
                .sources_a
                .iter()
                .map(&source_pull)
                .sum::<f64>()
                .min(cap);
            lines.push(Line::from(Span::styled(
                format!(" In A: {:.0}/hr (cap {:.0})", pull_a, cap),
                Style::default().fg(Color::Rgb(140, 100, 180)),
            )));
            for src in &shuttle.sources_a {
                let (name, color) = source_display(src);
                let pull = source_pull(src);
                lines.push(Line::from(vec![
                    Span::styled("  \u{2190} ", Style::default().fg(Color::DarkGray)),
                    Span::styled(name, Style::default().fg(color)),
                    Span::styled(
                        format!(" {:.0}/hr", pull),
                        Style::default().fg(Color::Rgb(100, 80, 130)),
                    ),
                ]));
            }

            // Input B
            let pull_b: f64 = shuttle
                .sources_b
                .iter()
                .map(&source_pull)
                .sum::<f64>()
                .min(cap);
            lines.push(Line::from(Span::styled(
                format!(" In B: {:.0}/hr (cap {:.0})", pull_b, cap),
                Style::default().fg(Color::Rgb(140, 100, 180)),
            )));
            for src in &shuttle.sources_b {
                let (name, color) = source_display(src);
                let pull = source_pull(src);
                lines.push(Line::from(vec![
                    Span::styled("  \u{2190} ", Style::default().fg(Color::DarkGray)),
                    Span::styled(name, Style::default().fg(color)),
                    Span::styled(
                        format!(" {:.0}/hr", pull),
                        Style::default().fg(Color::Rgb(100, 80, 130)),
                    ),
                ]));
            }

            // Output rate
            let output_rate = pull_a.min(pull_b) * shuttle.amount;
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    " Output: {:.0}/hr {}",
                    output_rate,
                    resource_name(&shuttle.output)
                ),
                Style::default().fg(Color::Rgb(100, 200, 120)),
            )));
        }

        // Status line.
        let status_text = if shuttle.stalled {
            " Status: STALLED"
        } else {
            " Status: Running"
        };
        let status_color = if shuttle.stalled {
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

        lines.truncate(post_height as usize);
        let post_area = Rect::new(inner.x, post_start_y, inner.width, post_height);
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            post_area,
        );
    }
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

    // ── Factory floor: branching tree layout ─────────────────────────────────
    // Diamond shape showing the two unlock paths from Ember Spindle:
    //   Row 0:       ES            (start node, centered)
    //              ╱    ╲
    //   Row 1:   RL      RF       (first neighbors)
    //            │        │
    //   Row 2:   VC      SW       (second neighbors)
    //              ╲    ╱
    //   Row 3:       MA            (final node, centered)

    // Grid index mapping for cursor navigation:
    //   0 = ES (row 0, center)
    //   1 = RL (row 1, left),  2 = RF (row 1, right)
    //   3 = VC (row 2, left),  4 = SW (row 2, right)
    //   5 = MA (row 3, center)
    let grid_ids = [
        NodeId::EmberSpindle,   // 0: top center
        NodeId::ReflectionLens, // 1: left branch tier 1
        NodeId::ResonanceForge, // 2: right branch tier 1
        NodeId::VoidCondenser,  // 3: left branch tier 2
        NodeId::SilenceWell,    // 4: right branch tier 2
        NodeId::MemoryArchive,  // 5: bottom center
    ];
    let selected_id = grid_ids[ui.selected_node.min(5)];

    // Determine shuttle section height.
    let shuttles = &loom_state.persistent.shuttles;
    let shuttle_row_count = if shuttles.is_empty() {
        0u16
    } else {
        let shuttle_grid_rows = shuttles.len().div_ceil(2);
        1 + (shuttle_grid_rows as u16) * (NODE_BOX_HEIGHT as u16 + 2)
    };

    // Layout: 4 extractor rows (single, pair, pair, single) + shuttles.
    let box_h = NODE_BOX_HEIGHT as u16;
    let gap_h = 1u16;
    let extractor_total_h = 4 * box_h + 3 * gap_h; // gaps used for vertical spacing only
    let content_h = extractor_total_h + shuttle_row_count;

    let v_pad = grid_area.height.saturating_sub(content_h) / 2;

    let mut row_constraints: Vec<Constraint> = Vec::new();
    row_constraints.push(Constraint::Length(v_pad));
    for i in 0..4 {
        row_constraints.push(Constraint::Length(box_h));
        if i < 3 {
            row_constraints.push(Constraint::Length(gap_h));
        }
    }
    if shuttle_row_count > 0 {
        row_constraints.push(Constraint::Length(shuttle_row_count));
    }
    row_constraints.push(Constraint::Min(0));

    let row_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_area);

    // Indices: 0=pad, 1=row0(ES), 2=gap, 3=row1(RL,RF), 4=gap, 5=row2(VC,SW), 6=gap, 7=row3(MA)
    // Then 8=shuttle section (if present), last=spacer
    let node_row_rects = [row_rects[1], row_rects[3], row_rects[5], row_rects[7]];
    let gap_rects = [row_rects[2], row_rects[4], row_rects[6]];
    let flow_arrow_color = Color::Rgb(120, 80, 180);

    let node_w = NODE_BOX_WIDTH as u16;
    let h_gap_w = 4u16.min(grid_area.width.saturating_sub(node_w * 2));
    let grid_total_w = node_w * 2 + h_gap_w;
    let h_pad = grid_area.width.saturating_sub(grid_total_w) / 2;

    // Helper: split a row into left/right columns with centering.
    let split_pair_row = |row_area: Rect| -> (Rect, Rect) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(h_pad),
                Constraint::Length(node_w),
                Constraint::Length(h_gap_w),
                Constraint::Length(node_w),
                Constraint::Min(0),
            ])
            .split(row_area);
        (cols[1], cols[3])
    };

    // Helper: center a single node in a row.
    let center_single_row = |row_area: Rect| -> Rect {
        let center_pad = row_area.width.saturating_sub(node_w) / 2;
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(center_pad),
                Constraint::Length(node_w),
                Constraint::Min(0),
            ])
            .split(row_area);
        cols[1]
    };

    // Row 0: Ember Spindle (centered).
    let es_rect = center_single_row(node_row_rects[0]);
    if let Some(node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == NodeId::EmberSpindle)
    {
        render_node_widget(
            frame,
            es_rect,
            node,
            loom_state,
            ui.selected_node < 6 && selected_id == NodeId::EmberSpindle,
        );
    }

    // Row 1: Reflection Lens (left) + Resonance Forge (right).
    let (rl_rect, rf_rect) = split_pair_row(node_row_rects[1]);
    if let Some(node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == NodeId::ReflectionLens)
    {
        render_node_widget(
            frame,
            rl_rect,
            node,
            loom_state,
            ui.selected_node < 6 && selected_id == NodeId::ReflectionLens,
        );
    }
    if let Some(node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == NodeId::ResonanceForge)
    {
        render_node_widget(
            frame,
            rf_rect,
            node,
            loom_state,
            ui.selected_node < 6 && selected_id == NodeId::ResonanceForge,
        );
    }

    // Row 2: Void Condenser (left) + Silence Well (right).
    let (vc_rect, sw_rect) = split_pair_row(node_row_rects[2]);
    if let Some(node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == NodeId::VoidCondenser)
    {
        render_node_widget(
            frame,
            vc_rect,
            node,
            loom_state,
            ui.selected_node < 6 && selected_id == NodeId::VoidCondenser,
        );
    }
    if let Some(node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == NodeId::SilenceWell)
    {
        render_node_widget(
            frame,
            sw_rect,
            node,
            loom_state,
            ui.selected_node < 6 && selected_id == NodeId::SilenceWell,
        );
    }

    // Row 3: Memory Archive (centered).
    let ma_rect = center_single_row(node_row_rects[3]);
    if let Some(node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == NodeId::MemoryArchive)
    {
        render_node_widget(
            frame,
            ma_rect,
            node,
            loom_state,
            ui.selected_node < 6 && selected_id == NodeId::MemoryArchive,
        );
    }

    // ── Arrows between rows ─────────────────────────────────────────────────

    // Gap 0 (ES → RL, ES → RF).
    if gap_rects[0].height > 0 {
        let rl_center_x = rl_rect.x + rl_rect.width / 2;
        let rf_center_x = rf_rect.x + rf_rect.width / 2;
        let gy = gap_rects[0].y;
        frame.render_widget(
            Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            Rect {
                x: rl_center_x,
                y: gy,
                width: 1,
                height: 1,
            },
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            Rect {
                x: rf_center_x,
                y: gy,
                width: 1,
                height: 1,
            },
        );
    }

    // Gap 1 (RL → VC, RF → SW).
    if gap_rects[1].height > 0 {
        let rl_center_x = rl_rect.x + rl_rect.width / 2;
        let rf_center_x = rf_rect.x + rf_rect.width / 2;
        let gy = gap_rects[1].y;
        frame.render_widget(
            Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            Rect {
                x: rl_center_x,
                y: gy,
                width: 1,
                height: 1,
            },
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            Rect {
                x: rf_center_x,
                y: gy,
                width: 1,
                height: 1,
            },
        );
    }

    // Gap 2 (VC → MA, SW → MA).
    if gap_rects[2].height > 0 {
        let ma_center_x = ma_rect.x + ma_rect.width / 2;
        let gy = gap_rects[2].y;
        frame.render_widget(
            Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            Rect {
                x: ma_center_x.saturating_sub(2),
                y: gy,
                width: 1,
                height: 1,
            },
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                "\u{25bc}",
                Style::default().fg(flow_arrow_color),
            ))
            .style(Style::default().bg(LOOM_BG)),
            Rect {
                x: ma_center_x + 2,
                y: gy,
                width: 1,
                height: 1,
            },
        );
    }

    // ── Shuttles: pipeline rows grouped by tier ────────────────────────────
    {
        let shuttle_section = row_rects[8];
        let mut y = shuttle_section.y;
        let section_end = shuttle_section.y + shuttle_section.height;

        // Shuttle count header.
        let max_shuttles = loom_state.persistent.max_shuttles();
        let count = shuttles.len();
        if y < section_end {
            let header = Paragraph::new(Line::from(Span::styled(
                format!(
                    "\u{2500}\u{2500} Shuttles ({}/{}) \u{2500}\u{2500}",
                    count, max_shuttles
                ),
                Style::default().fg(Color::Rgb(80, 60, 100)),
            )))
            .style(Style::default().bg(LOOM_BG));
            let row_rect = Rect {
                x: shuttle_section.x,
                y,
                width: shuttle_section.width,
                height: 1,
            };
            frame.render_widget(header, row_rect);
            y += 1;
        }

        if !shuttles.is_empty() {
            let render_row = |frame: &mut Frame, y: u16, line: Line, bg: Color| {
                if y < section_end {
                    let row_rect = Rect {
                        x: shuttle_section.x,
                        y,
                        width: shuttle_section.width,
                        height: 1,
                    };
                    frame.render_widget(
                        Paragraph::new(line).style(Style::default().bg(bg)),
                        row_rect,
                    );
                }
            };

            for tier in 1u8..=3 {
                let tier_shuttles: Vec<(usize, &crate::loom::types::Shuttle)> = shuttles
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.tier == tier)
                    .collect();
                if tier_shuttles.is_empty() {
                    continue;
                }

                if y >= section_end {
                    break;
                }

                // Tier header.
                render_row(
                    frame,
                    y,
                    Line::from(Span::styled(
                        format!("\u{2500}\u{2500} T{} Shuttles \u{2500}\u{2500}", tier),
                        Style::default().fg(Color::Rgb(80, 60, 100)),
                    )),
                    LOOM_BG,
                );
                y += 1;

                for (i, shuttle) in tier_shuttles {
                    if y >= section_end {
                        break;
                    }
                    let is_sel = ui.selected_node >= 6 && (ui.selected_node - 6) == i;

                    let ea = resource_emoji(&shuttle.input_a);
                    let eb = resource_emoji(&shuttle.input_b);
                    let eo = resource_emoji(&shuttle.output);
                    let out_name = resource_name(&shuttle.output);

                    let row_color = if is_sel {
                        Color::Rgb(200, 170, 240)
                    } else {
                        Color::Rgb(120, 100, 160)
                    };

                    let line = if shuttle.under_construction {
                        let warp = loom_state.time_warp.max(1.0) as u32;
                        let total_secs = shuttle.construction_ticks_remaining / (10 * warp);
                        let time_str = if total_secs >= 3600 {
                            format!("~{}h", total_secs / 3600)
                        } else if total_secs >= 60 {
                            format!("~{}m", total_secs / 60)
                        } else {
                            format!("~{}s", total_secs)
                        };
                        Line::from(Span::styled(
                            format!(
                                " {}+{} \u{2192} R{} {} {} \u{1f528} {}",
                                ea, eb, i, eo, out_name, time_str
                            ),
                            Style::default().fg(Color::Rgb(100, 80, 130)),
                        ))
                    } else {
                        // Compute this shuttle's individual output rate from its pull math.
                        let cap = crate::loom::logic::shuttle_effective_intake_cap(
                            shuttle.tier,
                            shuttle.level,
                        );
                        let mut consumer_counts: std::collections::HashMap<
                            crate::loom::types::LoomNodeRef,
                            usize,
                        > = std::collections::HashMap::new();
                        for s in &loom_state.persistent.shuttles {
                            if !s.under_construction {
                                for src in s.sources_a.iter().chain(s.sources_b.iter()) {
                                    *consumer_counts.entry(*src).or_insert(0) += 1;
                                }
                            }
                        }
                        let src_pull = |src: &crate::loom::types::LoomNodeRef| -> f64 {
                            let available = match src {
                                crate::loom::types::LoomNodeRef::Extractor(id) => {
                                    let n = &loom_state.persistent.nodes[id.index()];
                                    crate::loom::logic::node_effective_rate(loom_state, n)
                                }
                                crate::loom::types::LoomNodeRef::Shuttle(idx) => loom_state
                                    .rate_trackers
                                    .get(
                                        &loom_state
                                            .persistent
                                            .shuttles
                                            .get(*idx)
                                            .map(|s| s.output)
                                            .unwrap_or(crate::loom::types::Resource::Ember),
                                    )
                                    .map(|t| t.rate_per_hour())
                                    .unwrap_or(0.0),
                            };
                            let consumers = *consumer_counts.get(src).unwrap_or(&1) as f64;
                            (available / consumers).min(cap)
                        };
                        let pull_a: f64 = shuttle
                            .sources_a
                            .iter()
                            .map(&src_pull)
                            .sum::<f64>()
                            .min(cap);
                        let pull_b: f64 = shuttle
                            .sources_b
                            .iter()
                            .map(&src_pull)
                            .sum::<f64>()
                            .min(cap);
                        let output_rate = pull_a.min(pull_b) * shuttle.amount;

                        let fill = if shuttle.buffer_capacity > 0.0 {
                            (shuttle.buffer / shuttle.buffer_capacity).min(1.0)
                        } else {
                            0.0
                        };
                        let filled = ((fill * 5.0) as usize).min(5);
                        let empty = 5 - filled;
                        let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);

                        Line::from(vec![
                            Span::styled(
                                format!(" {}+{} \u{2192} R{} {} {}", ea, eb, i, eo, out_name),
                                Style::default().fg(row_color),
                            ),
                            Span::styled(
                                format!(" {:.0}/hr ", output_rate),
                                Style::default().fg(if output_rate > 0.0 {
                                    Color::Rgb(100, 200, 120)
                                } else {
                                    Color::Rgb(80, 60, 100)
                                }),
                            ),
                            Span::styled(bar, Style::default().fg(Color::Rgb(60, 200, 100))),
                        ])
                    };

                    let bg = if is_sel {
                        Color::Rgb(35, 25, 50)
                    } else {
                        LOOM_BG
                    };
                    render_row(frame, y, line, bg);
                    y += 1;
                }
            }
        }
    } // end shuttle section

    // ── Sidebar ─────────────────────────────────────────────────────────────
    render_flow_sidebar(frame, sidebar_area, loom_state, ui);

    // ── Pattern bar ─────────────────────────────────────────────────────────
    if has_patterns {
        render_pattern_bar(frame, pattern_area, loom_state);
    }
}

/// Build a compact inline ratio bar (e.g. "████░░░░░░" for 40%).
fn build_ratio_bar(ratio: f64, width: usize) -> String {
    let filled = ((ratio.clamp(0.0, 1.0) * width as f64) as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

/// Resource lists for each codex column.
const CODEX_BASE: [crate::loom::types::Resource; 6] = {
    use crate::loom::types::Resource;
    [
        Resource::Ember,
        Resource::VoidEssence,
        Resource::Reflection,
        Resource::Memory,
        Resource::Silence,
        Resource::Resonance,
    ]
};

const CODEX_CONFLUENCE: [crate::loom::types::Resource; 6] = {
    use crate::loom::types::Resource;
    [
        Resource::ForgedLight,
        Resource::CondensedEmber,
        Resource::EmberEcho,
        Resource::PurifiedVoid,
        Resource::EchoGlass,
        Resource::StillbornSong,
    ]
};

const CODEX_TERMINAL: [crate::loom::types::Resource; 1] =
    [crate::loom::types::Resource::WovenReality];

/// Get the resource at a given (column, row) in the codex graph.
fn codex_resource_at(col: usize, row: usize) -> Option<crate::loom::types::Resource> {
    match col {
        0 => CODEX_BASE.get(row).copied(),
        1 => CODEX_CONFLUENCE.get(row).copied(),
        2 => CODEX_TERMINAL.get(row).copied(),
        _ => None,
    }
}

/// Number of resources in a codex column.
fn codex_column_len(col: usize) -> usize {
    match col {
        0 => 6,
        1 => 6,
        2 => 1,
        _ => 0,
    }
}

/// Check if a recipe is discovered in the codex.
fn is_recipe_discovered(
    codex: &[crate::loom::types::CodexEntry],
    recipe: &crate::loom::recipes::Recipe,
) -> bool {
    codex.iter().any(|e| {
        e.discovered
            && e.output == recipe.output
            && e.node_nature == recipe.node_nature
            && e.inputs.len() == 2
            && ((e.inputs[0] == recipe.input_a && e.inputs[1] == recipe.input_b)
                || (e.inputs[0] == recipe.input_b && e.inputs[1] == recipe.input_a))
    })
}

/// Check if a confluence resource has been discovered (at least one recipe producing it is known).
fn is_resource_discovered(
    codex: &[crate::loom::types::CodexEntry],
    resource: crate::loom::types::Resource,
) -> bool {
    use crate::loom::types::Resource;
    // Base resources are always discovered.
    matches!(
        resource,
        Resource::Ember
            | Resource::Reflection
            | Resource::VoidEssence
            | Resource::Memory
            | Resource::Silence
            | Resource::Resonance
    ) || codex.iter().any(|e| e.discovered && e.output == resource)
}

fn render_codex(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::recipes::{all_recipes, recipes_producing, recipes_using};

    let discovered_count = loom_state
        .persistent
        .codex
        .iter()
        .filter(|e| e.discovered)
        .count();
    let total_recipes = all_recipes().len();

    // Split area into top (topology graph ~60%) and bottom (detail panel ~40%).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(58),
            Constraint::Percentage(38),
            Constraint::Length(1), // footer
        ])
        .split(area);
    let graph_area = chunks[0];
    let detail_area = chunks[1];
    let footer_area = chunks[2];

    // ── Top half: Topology graph ──────────────────────────────────────────────
    let mut graph_lines: Vec<Line<'static>> = Vec::new();

    graph_lines.push(Line::from(""));

    // Build header line with column labels.
    {
        let mut spans = vec![Span::raw("  ")];
        let header_style = Style::default().fg(Color::Rgb(140, 100, 180));
        spans.push(Span::styled(format!("{:<24}", "BASE"), header_style));
        spans.push(Span::styled(format!("{:<26}", "CONFLUENCE"), header_style));
        spans.push(Span::styled("TERMINAL", header_style));
        graph_lines.push(Line::from(spans));
    }
    graph_lines.push(Line::from(""));

    // Build the graph rows. We'll render the maximum column length (6 rows).
    let max_rows = 6usize;
    let codex = &loom_state.persistent.codex;

    // Primary connection map: base resources → their primary confluence target.
    // Used for rendering connection indicators.
    let connections: &[(usize, usize)] = &[
        (0, 0), // Ember → ForgedLight
        (1, 0), // VoidEssence → ForgedLight
        (3, 4), // Memory → EchoGlass
        (4, 4), // Silence → EchoGlass
        (5, 5), // Resonance → StillbornSong
    ];
    // Confluence → Terminal connections.
    let conf_to_term: &[usize] = &[0, 4]; // ForgedLight, EchoGlass → WovenReality

    for row in 0..max_rows {
        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::raw("  "));

        // ── Base column ──
        if let Some(res) = codex_resource_at(0, row) {
            let is_selected = ui.codex_column == 0 && ui.codex_row == row;
            let name = resource_name(&res);
            let emoji = resource_emoji(&res);
            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 200))
                    .bg(Color::Rgb(60, 40, 80))
            } else {
                Style::default().fg(Color::Rgb(180, 140, 220))
            };
            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            spans.push(Span::styled(format!("{}{} {}", cursor, emoji, name), style));
            // Pad to column width.
            let label_len = 2 + 2 + 1 + name.len(); // cursor(2) + emoji(2) + space(1) + name
            let pad = 22usize.saturating_sub(label_len);

            // Connection indicator: check if this base resource connects to a confluence.
            let has_connection = connections.iter().any(|(b, _)| *b == row);
            if has_connection {
                let pad_str = " ".repeat(pad.saturating_sub(3));
                spans.push(Span::styled(
                    format!("{}\u{2500}\u{2500}\u{25b6}", pad_str),
                    Style::default().fg(Color::Rgb(80, 60, 120)),
                ));
            } else {
                spans.push(Span::raw(" ".repeat(pad)));
            }
        } else {
            spans.push(Span::raw("                        "));
        }

        // ── Confluence column ──
        if let Some(res) = codex_resource_at(1, row) {
            let is_selected = ui.codex_column == 1 && ui.codex_row == row;
            let discovered = is_resource_discovered(codex, res);
            let name = if discovered {
                resource_name(&res)
            } else {
                "???"
            };
            let emoji = if discovered {
                resource_emoji(&res)
            } else {
                "\u{2753}"
            }; // ❓
            let tier_mark = if discovered { "\u{2726} " } else { "  " }; // ✦

            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 200))
                    .bg(Color::Rgb(60, 40, 80))
            } else if discovered {
                Style::default().fg(Color::Rgb(200, 160, 240))
            } else {
                Style::default().fg(Color::Rgb(60, 45, 80))
            };

            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            spans.push(Span::styled(
                format!("{}{}{} {}", cursor, tier_mark, emoji, name),
                style,
            ));
            // Pad and add terminal connection if applicable.
            let label_len = 2 + 2 + 2 + 1 + name.len();
            let pad = 24usize.saturating_sub(label_len);

            let has_term_conn = conf_to_term.contains(&row);
            if has_term_conn {
                let pad_str = " ".repeat(pad.saturating_sub(3));
                spans.push(Span::styled(
                    format!("{}\u{2500}\u{2500}\u{25b6}", pad_str),
                    Style::default().fg(Color::Rgb(80, 60, 120)),
                ));
            } else {
                spans.push(Span::raw(" ".repeat(pad)));
            }
        } else {
            spans.push(Span::raw("                          "));
        }

        // ── Terminal column ──
        if let Some(res) = codex_resource_at(2, row) {
            let is_selected = ui.codex_column == 2 && ui.codex_row == row;
            let discovered = is_resource_discovered(codex, res);
            let name = if discovered {
                resource_name(&res)
            } else {
                "???"
            };
            let emoji = if discovered {
                resource_emoji(&res)
            } else {
                "\u{2753}"
            };
            let star = if discovered { "\u{2605} " } else { "  " }; // ★

            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 200))
                    .bg(Color::Rgb(60, 40, 80))
            } else if discovered {
                Style::default().fg(Color::Rgb(255, 215, 0))
            } else {
                Style::default().fg(Color::Rgb(60, 45, 80))
            };

            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            spans.push(Span::styled(
                format!("{}{}{} {}", cursor, star, emoji, name),
                style,
            ));
        }

        graph_lines.push(Line::from(spans));
    }

    let graph_para = Paragraph::new(graph_lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(LOOM_BG));
    frame.render_widget(graph_para, graph_area);

    // ── Bottom half: Detail panel ─────────────────────────────────────────────
    let selected_resource = codex_resource_at(ui.codex_column, ui.codex_row);
    let mut detail_lines: Vec<Line<'static>> = Vec::new();

    if let Some(res) = selected_resource {
        let res_discovered = is_resource_discovered(codex, res);
        let display_name = if res_discovered {
            format!("{} {}", resource_emoji(&res), resource_name(&res))
        } else {
            "???".to_string()
        };

        // Separator + header.
        let sep_width = area.width as usize;
        let sep_line = format!(
            " \u{2500}\u{2500}\u{2500} {} {}",
            display_name,
            "\u{2500}".repeat(sep_width.saturating_sub(display_name.len() + 6))
        );
        detail_lines.push(Line::from(Span::styled(
            sep_line,
            Style::default().fg(Color::Rgb(100, 70, 140)),
        )));

        // Split detail area into two sub-columns.
        let producing = recipes_producing(res);
        let using = recipes_using(res);

        // "Made from:" column
        let made_from_header = Line::from(vec![
            Span::styled(
                "  Made from:                        ",
                Style::default().fg(Color::Rgb(140, 100, 180)),
            ),
            Span::styled("Used in:", Style::default().fg(Color::Rgb(140, 100, 180))),
        ]);
        detail_lines.push(made_from_header);

        let max_detail_rows = (detail_area.height as usize).saturating_sub(3);
        let max_entries = producing.len().max(using.len()).min(max_detail_rows);

        for i in 0..max_entries {
            let mut spans: Vec<Span<'static>> = Vec::new();

            // Left sub-column: "Made from" recipes.
            if i < producing.len() {
                let recipe = &producing[i];
                let disc = is_recipe_discovered(codex, recipe);
                if disc {
                    let text = format!(
                        "   {} + {} @ {} (x{:.1}) \u{2713}",
                        resource_name(&recipe.input_a),
                        resource_name(&recipe.input_b),
                        node_nature_name(recipe.node_nature),
                        recipe.amount,
                    );
                    // Pad to ~36 chars.
                    let padded = format!("{:<36}", text);
                    spans.push(Span::styled(
                        padded,
                        Style::default().fg(Color::Rgb(160, 130, 200)),
                    ));
                } else {
                    let text = "   ??? + ??? @ ??? \u{2192} ???";
                    let padded = format!("{:<36}", text);
                    spans.push(Span::styled(
                        padded,
                        Style::default().fg(Color::Rgb(60, 45, 80)),
                    ));
                }
            } else {
                spans.push(Span::raw("                                    "));
            }

            // Right sub-column: "Used in" recipes.
            if i < using.len() {
                let recipe = &using[i];
                let disc = is_recipe_discovered(codex, recipe);
                if disc {
                    let other_input = if recipe.input_a == res {
                        resource_name(&recipe.input_b)
                    } else {
                        resource_name(&recipe.input_a)
                    };
                    let output_discovered = is_resource_discovered(codex, recipe.output);
                    let output_name = if output_discovered {
                        resource_name(&recipe.output)
                    } else {
                        "???"
                    };
                    let text = format!(
                        "+ {} @ {} \u{2192} {} (x{:.1})",
                        other_input,
                        node_nature_name(recipe.node_nature),
                        output_name,
                        recipe.amount,
                    );
                    spans.push(Span::styled(
                        text,
                        Style::default().fg(Color::Rgb(160, 130, 200)),
                    ));
                } else {
                    spans.push(Span::styled(
                        "+ ??? @ ??? \u{2192} ???",
                        Style::default().fg(Color::Rgb(60, 45, 80)),
                    ));
                }
            }

            detail_lines.push(Line::from(spans));
        }

        // Show remaining undiscovered counts if we couldn't fit them all.
        let remaining_producing = producing
            .iter()
            .skip(max_entries)
            .filter(|r| !is_recipe_discovered(codex, r))
            .count();
        let remaining_using = using
            .iter()
            .skip(max_entries)
            .filter(|r| !is_recipe_discovered(codex, r))
            .count();

        if remaining_producing > 0 || remaining_using > 0 {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if remaining_producing > 0 {
                let text = format!("   + {} undiscovered...", remaining_producing);
                spans.push(Span::styled(
                    format!("{:<36}", text),
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                ));
            } else {
                spans.push(Span::raw("                                    "));
            }
            if remaining_using > 0 {
                spans.push(Span::styled(
                    format!("+ {} undiscovered...", remaining_using),
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                ));
            }
            detail_lines.push(Line::from(spans));
        }
    }

    let detail_para = Paragraph::new(detail_lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(LOOM_BG));
    frame.render_widget(detail_para, detail_area);

    // ── Footer ────────────────────────────────────────────────────────────────
    let footer_line = Line::from(vec![
        Span::styled(
            format!("  {}/{} Discovered", discovered_count, total_recipes),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("                    "),
        Span::styled(
            "\u{2190}\u{2192} Column  \u{2191}\u{2193} Resource  Tab: Flow",
            Style::default().fg(Color::Rgb(80, 60, 120)),
        ),
    ]);
    let footer_para = Paragraph::new(vec![footer_line])
        .alignment(Alignment::Left)
        .style(Style::default().bg(LOOM_BG));
    frame.render_widget(footer_para, footer_area);
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
                " \u{2728} Loom Mended \u{2014} All 28 Patterns Complete ",
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

    // Rows 0..N: single-line gauge per requirement with all info in the label.
    // Format: "🔥 Ember  ▓▓▓▓░░░░░░░░  0:48/10:00  170/hr ≥ 20 ▶"
    for (i, req) in pattern.requirements.iter().enumerate() {
        let row_area = rows[i];
        if row_area.height == 0 {
            continue;
        }

        let ratio = if req.sustain_duration_secs > 0.0 {
            (req.sustained_secs / req.sustain_duration_secs).min(1.0)
        } else {
            1.0
        };
        let met = req.completed;

        let current_rate = loom_state
            .rate_trackers
            .get(&req.resource)
            .map(|t| t.rate_per_hour())
            .unwrap_or(0.0);
        let advancing = !met && current_rate >= req.required_rate;

        let sustained_mins = (req.sustained_secs / 60.0) as u32;
        let duration_mins = (req.sustain_duration_secs / 60.0) as u32;
        let time_label = format!(
            "{}:{:02}/{}:{:02}",
            sustained_mins / 60,
            sustained_mins % 60,
            duration_mins / 60,
            duration_mins % 60,
        );

        let state_icon = if met {
            "\u{2713}"
        } else if advancing {
            "\u{25B6}"
        } else {
            "\u{23F8}"
        };

        let emoji = resource_emoji(&req.resource);
        let res_name = resource_name(&req.resource);

        // Build gauge label: "🔥 Ember  0:48/10:00  170 of 25/hr ▶"
        let gauge_label = format!(
            "{} {:14} {}  {:.0} of {:.0}/hr {}",
            emoji, res_name, time_label, current_rate, req.required_rate, state_icon
        );

        let gauge_color = if met {
            Color::Rgb(80, 200, 120)
        } else if advancing {
            Color::Rgb(100, 180, 100)
        } else {
            Color::Rgb(180, 140, 40)
        };

        let gauge = Gauge::default()
            .ratio(ratio)
            .label(gauge_label)
            .gauge_style(Style::default().fg(gauge_color).bg(Color::Rgb(30, 20, 40)));
        frame.render_widget(gauge, row_area);
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
                    if req.sustain_duration_secs > 0.0 {
                        req.sustained_secs / req.sustain_duration_secs
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
                Constraint::Length(14),
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

// ── Build Shuttle Overlay ────────────────────────────────────────────────────

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
            let unlocked_tiers = crate::loom::unlocked_tiers(loom_state);
            let tier_labels: String = unlocked_tiers
                .iter()
                .map(|t| {
                    if *t == build.tier {
                        format!("[T{}]", t)
                    } else {
                        format!(" T{} ", t)
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            lines.push(Line::from(Span::styled(
                format!(" {} Recipes:", tier_labels),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            if unlocked_tiers.len() > 1 {
                lines.push(Line::from(Span::styled(
                    " [Tab] Switch Tier",
                    Style::default().fg(Color::DarkGray),
                )));
            }
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
                        "{}{} + {} \u{2192} {} ({:.1}x)",
                        marker,
                        resource_name(&r.input_a),
                        resource_name(&r.input_b),
                        resource_name(&r.output),
                        r.amount,
                    ),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            let cost = crate::loom::shuttle_build_cost_public(build.tier);
            lines.push(Line::from(Span::styled(
                format!(" Build cost: {:.0} of input A resource", cost),
                Style::default().fg(Color::Rgb(100, 80, 130)),
            )));
            (" Build Shuttle ", lines)
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
            let cost = crate::loom::shuttle_build_cost_public(build.tier);
            let available = crate::loom::logic::available_resource(loom_state, r.input_a);
            let can_afford = available >= cost;
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
                    available
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
        crate::loom::types::LoomNodeRef::Shuttle(idx) => {
            if let Some(r) = loom_state.persistent.shuttles.get(*idx) {
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
        Resource::VoidEssence => "Void Essence",
        Resource::Memory => "Memory",
        Resource::Silence => "Silence",
        Resource::Resonance => "Resonance",
        Resource::ForgedLight => "Forged Light",
        Resource::EchoGlass => "Echo Glass",
        Resource::StillbornSong => "Stillborn Song",
        Resource::CondensedEmber => "Condensed Ember",
        Resource::EmberEcho => "Ember Echo",
        Resource::PurifiedVoid => "Purified Void",
        Resource::WovenReality => "Woven Reality",
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
