# Loom Flow View Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the text-table Loom Flow View with an animated factory floor using scene_fx cell buffers, featuring living machine nodes with unique textures, recipe input slots, port labels, and a sidebar detail panel.

**Architecture:** The new Flow View renders to a `SceneCell` buffer (like zone backgrounds and combat scenes) for per-character animation control. Six machine nodes are placed at fixed grid positions in a 3x2 layout. A Paragraph-based sidebar on the right shows selected node detail. The existing `render_flow_view` function is replaced; all other views (ArchetypeSelection, ListDetail, Codex) remain unchanged.

**Tech Stack:** Rust, Ratatui, scene_fx cell buffer system (`SceneCell`, `put_text`, `put_cell`, `render_buffer`, `current_millis`)

---

### Task 1: Add `recipes_by_nature` helper to recipe registry

**Files:**
- Modify: `src/loom/recipes.rs`

**Step 1: Write the failing test**

Add to the existing `#[cfg(test)] mod tests` block in `src/loom/recipes.rs`:

```rust
#[test]
fn test_recipes_by_nature_heat() {
    let heat_recipes = recipes_by_nature(NodeNature::Heat);
    assert!(!heat_recipes.is_empty(), "Heat should have recipes");
    for r in &heat_recipes {
        assert_eq!(r.node_nature, NodeNature::Heat);
    }
}

#[test]
fn test_recipes_by_nature_returns_all_natures() {
    use NodeNature::*;
    for nature in [Heat, Form, Void, Pattern, Stillness, Vibration] {
        let recipes = recipes_by_nature(nature);
        assert!(!recipes.is_empty(), "{:?} should have at least one recipe", nature);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::recipes::tests::test_recipes_by_nature`
Expected: FAIL — `recipes_by_nature` not found.

**Step 3: Write implementation**

Add to `src/loom/recipes.rs` after the existing `recipes_using` function:

```rust
/// Returns all recipes that use a given node nature as catalyst.
pub fn recipes_by_nature(nature: NodeNature) -> Vec<Recipe> {
    all_recipes()
        .into_iter()
        .filter(|r| r.node_nature == nature)
        .collect()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::recipes::tests::test_recipes_by_nature`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/recipes.rs
git commit -m "feat(loom): add recipes_by_nature helper for sidebar recipe list"
```

---

### Task 2: Add node color constants and abbreviation helpers

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Add constants and helper functions**

Add near the top of `src/ui/loom_scene.rs`, after the existing `LOOM_BORDER_COLOR` constant:

```rust
use crate::ui::scene_fx::{current_millis, put_cell, put_text, render_buffer, SceneCell};

/// Background color for the Loom overlay interior.
const LOOM_BG: Color = Color::Rgb(10, 5, 18);

/// Per-node identity colors used for port labels and highlighting.
fn node_color(id: crate::loom::types::NodeId) -> Color {
    use crate::loom::types::NodeId;
    match id {
        NodeId::EmberSpindle => Color::Rgb(255, 140, 50),    // orange
        NodeId::VoidCondenser => Color::Rgb(160, 80, 220),   // purple
        NodeId::ReflectionLens => Color::Rgb(80, 200, 220),  // cyan
        NodeId::MemoryArchive => Color::Rgb(220, 200, 80),   // yellow
        NodeId::SilenceWell => Color::Rgb(140, 140, 160),    // gray
        NodeId::ResonanceForge => Color::Rgb(80, 140, 255),  // blue
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

/// Short resource name (max 5 chars) for recipe slot display inside node boxes.
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
```

**Step 2: Verify compilation**

Run: `cargo build 2>&1 | tail -5`
Expected: Compiles with no errors (unused warnings are OK at this stage).

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add node color, letter, and resource_short helpers"
```

---

### Task 3: Implement node texture animation system

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Add texture rendering function**

Add a function that writes an animated 2-row texture into a SceneCell buffer at a given position. Each node type has a unique pattern that shifts over time.

```rust
/// Render the animated 2-row texture interior for a node.
/// `row` and `col` are the top-left of the texture area (inside the box border).
/// `width` is the number of columns available for the texture.
fn render_node_texture(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    width: usize,
    node_id: crate::loom::types::NodeId,
    stalled: bool,
    unlocked: bool,
) {
    use crate::loom::types::NodeId;

    if !unlocked {
        // Locked: dim lock icon centered on both rows.
        let lock_text = "locked";
        let start = col + (width as i32 - lock_text.len() as i32) / 2;
        let dim = Color::Rgb(40, 30, 55);
        put_text(buffer, row, start, lock_text, dim);
        put_text(buffer, row + 1, start, "  \u{1f512}   ", dim);
        return;
    }

    let millis = current_millis();
    let frame = if stalled { 0 } else { (millis / 300) as usize };
    let color = if stalled {
        Color::Rgb(40, 30, 55)
    } else {
        node_color(node_id)
    };

    for r in 0..2i32 {
        for c in 0..width as i32 {
            let ch = match node_id {
                NodeId::EmberSpindle => {
                    // Horizontal shifting wave
                    let offset = (frame + c as usize + r as usize) % 4;
                    match offset { 0 => '~', 1 => ' ', 2 => '~', _ => ' ' }
                }
                NodeId::ReflectionLens => {
                    // Twinkling dots
                    let offset = (frame + c as usize * 3 + r as usize * 7) % 6;
                    match offset { 0 => '.', 1 => '\u{b7}', 2 => '*', 3 => '\u{b7}', 4 => '.', _ => ' ' }
                }
                NodeId::VoidCondenser => {
                    // Dripping colons
                    let offset = (frame + c as usize * 2 + r as usize) % 4;
                    match offset { 0 => ':', 1 => ' ', 2 => '\u{b7}', _ => ' ' }
                }
                NodeId::MemoryArchive => {
                    // Crosshatch
                    let offset = (frame + c as usize + r as usize) % 4;
                    match offset { 0 => '\u{2573}', 1 => ' ', 2 => '\u{2573}', _ => ' ' }
                }
                NodeId::SilenceWell => {
                    // Calm ripple
                    let offset = (frame + c as usize) % 6;
                    match offset { 0 => '_', 1 => ' ', 2 => '_', 3 => ' ', 4 => '_', _ => ' ' }
                }
                NodeId::ResonanceForge => {
                    // Vibrating waves
                    let offset = (frame + c as usize + r as usize * 3) % 4;
                    match offset { 0 => '\u{2248}', 1 => ' ', 2 => '\u{2248}', _ => ' ' }
                }
            };
            put_cell(buffer, row + r, col + c, ch, color);
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build 2>&1 | tail -5`
Expected: Compiles (unused warning OK).

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add animated node texture renderer"
```

---

### Task 4: Implement node box renderer

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Add the node box rendering function**

This renders a complete node box into the cell buffer at a given position. The box includes: title bar with border, 2-row animated texture, buffer bar, recipe slots, and port labels below.

```rust
/// Width of a single node box in columns (including borders).
const NODE_BOX_WIDTH: usize = 28;
/// Height of a node box in rows (including borders, excluding port label row).
const NODE_BOX_HEIGHT: usize = 6;

/// Render a single machine node box into the cell buffer.
/// `top` and `left` are the top-left corner of the box.
/// Returns the row just below the box (where port labels go).
fn render_node_box(
    buffer: &mut [Vec<SceneCell>],
    top: i32,
    left: i32,
    node: &crate::loom::types::LoomNode,
    loom_state: &LoomState,
    selected: bool,
) -> i32 {
    use crate::loom::types::NodeId;
    let w = NODE_BOX_WIDTH as i32;
    let inner_w = (w - 2) as usize; // width inside borders

    // Border characters and color.
    let (tl, tr, bl, br, h, v) = if selected {
        ('\u{250f}', '\u{2513}', '\u{2517}', '\u{251b}', '\u{2501}', '\u{2503}')
    } else {
        ('\u{250c}', '\u{2510}', '\u{2514}', '\u{2518}', '\u{2500}', '\u{2502}')
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
    // Fill top border.
    for c in 1..w - 1 {
        put_cell(buffer, top, left + c, h, border_color);
    }
    // Overlay title text.
    let title_start = left + 1;
    for (i, ch) in title.chars().enumerate().take(inner_w) {
        put_cell(buffer, top, title_start + i as i32, ch, title_color);
    }
    put_cell(buffer, top, left + w - 1, tr, border_color);

    // Rows 1-2: animated texture.
    for r in 1..=2 {
        put_cell(buffer, top + r, left, v, border_color);
        put_cell(buffer, top + r, left + w - 1, v, border_color);
    }
    render_node_texture(buffer, top + 1, left + 1, inner_w, node.id, node.stalled, node.unlocked);

    // Row 3: buffer bar.
    put_cell(buffer, top + 3, left, v, border_color);
    if node.unlocked {
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
        let bar_w = 10usize.min(inner_w.saturating_sub(8));
        let filled = ((fill * bar_w as f64) as usize).min(bar_w);
        let empty = bar_w.saturating_sub(filled);
        let bar_str = format!(
            " {}{} {:>4.1}/{:.0}",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
            node.buffer,
            node.buffer_capacity
        );
        let col = left + 1;
        for (i, ch) in bar_str.chars().enumerate().take(inner_w) {
            let c = if i == 0 || i > filled + empty { Color::Rgb(120, 100, 140) } else if i <= filled { bar_color } else { Color::Rgb(40, 30, 55) };
            put_cell(buffer, top + 3, col + i as i32, ch, c);
        }
    }
    put_cell(buffer, top + 3, left + w - 1, v, border_color);

    // Row 4: recipe slots.
    put_cell(buffer, top + 4, left, v, border_color);
    render_recipe_slots(buffer, top + 4, left + 1, inner_w, node, loom_state);
    put_cell(buffer, top + 4, left + w - 1, v, border_color);

    // Row 5: bottom border.
    put_cell(buffer, top + 5, left, bl, border_color);
    for c in 1..w - 1 {
        put_cell(buffer, top + 5, left + c, h, border_color);
    }
    put_cell(buffer, top + 5, left + w - 1, br, border_color);

    top + NODE_BOX_HEIGHT as i32 // return row below box
}
```

**Step 2: Add recipe slot renderer (called by node box)**

```rust
/// Render recipe input slots inside a node box row.
/// Shows the best candidate recipe: [*A] [*B] > Output  or  [oA] [*B] > Output?
fn render_recipe_slots(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    width: usize,
    node: &crate::loom::types::LoomNode,
    loom_state: &LoomState,
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

    // Determine which resources are arriving at this node via pipes.
    let incoming_resources: std::collections::HashSet<crate::loom::types::Resource> = loom_state
        .persistent
        .pipes
        .iter()
        .filter(|p| p.to == node.id && !p.under_construction)
        .map(|p| crate::loom::node_native_resource(p.from))
        .collect();

    // Find the best recipe: prefer one with both inputs filled, then one with at least one.
    let best = recipes
        .iter()
        .max_by_key(|r| {
            let has_a = incoming_resources.contains(&r.input_a) as u8;
            let has_b = incoming_resources.contains(&r.input_b) as u8;
            (has_a + has_b, (r.amount * 100.0) as u32)
        });

    if let Some(recipe) = best {
        let has_a = incoming_resources.contains(&recipe.input_a);
        let has_b = incoming_resources.contains(&recipe.input_b);
        let producing = has_a && has_b;

        let millis = current_millis();
        let pulse_bright = producing && (millis / 500) % 2 == 0;

        let slot_a = format!(
            "[{}{}]",
            if has_a { "\u{25cf}" } else { "\u{25cb}" },
            resource_short(&recipe.input_a)
        );
        let slot_b = format!(
            "[{}{}]",
            if has_b { "\u{25cf}" } else { "\u{25cb}" },
            resource_short(&recipe.input_b)
        );
        let arrow = if pulse_bright { "\u{25b6}" } else { ">" };
        let output = resource_short(&recipe.output);
        let text = format!("{} {} {} {}", slot_a, slot_b, arrow, output);

        let filled_color = Color::Rgb(180, 220, 180);
        let empty_color = Color::Rgb(120, 50, 50);
        let output_color = if producing { Color::Rgb(220, 200, 255) } else { Color::Rgb(80, 60, 100) };

        // Write character by character with appropriate colors.
        let mut pos = 0i32;
        // Slot A
        for ch in slot_a.chars() {
            let c = if has_a { filled_color } else { empty_color };
            put_cell(buffer, row, col + pos, ch, c);
            pos += 1;
        }
        put_cell(buffer, row, col + pos, ' ', LOOM_BG);
        pos += 1;
        // Slot B
        for ch in slot_b.chars() {
            let c = if has_b { filled_color } else { empty_color };
            put_cell(buffer, row, col + pos, ch, c);
            pos += 1;
        }
        put_cell(buffer, row, col + pos, ' ', LOOM_BG);
        pos += 1;
        // Arrow
        for ch in arrow.chars() {
            put_cell(buffer, row, col + pos, ch, output_color);
            pos += 1;
        }
        put_cell(buffer, row, col + pos, ' ', LOOM_BG);
        pos += 1;
        // Output name
        for ch in output.chars() {
            put_cell(buffer, row, col + pos, ch, output_color);
            pos += 1;
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build 2>&1 | tail -5`
Expected: Compiles (unused warnings OK).

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add node box and recipe slot renderers"
```

---

### Task 5: Implement port label renderer

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Add port label rendering function**

```rust
/// Render port labels below a node box.
/// Format: outgoing on left (->V ->R), incoming on right (<-E <-F).
/// `row` is the row to render on, `left` is the left edge of the node box.
fn render_port_labels(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    left: i32,
    node_id: crate::loom::types::NodeId,
    loom_state: &LoomState,
    selected: bool,
    selected_node_id: crate::loom::types::NodeId,
) {
    let millis = current_millis();
    let mut pos = left + 1;

    // Outgoing pipes.
    for pipe in &loom_state.persistent.pipes {
        if pipe.from != node_id {
            continue;
        }
        let is_highlighted = selected
            || selected_node_id == pipe.to; // highlight if this node or connected node is selected
        let color = if is_highlighted {
            node_color(pipe.to)
        } else {
            Color::Rgb(60, 50, 70)
        };
        let blink = pipe.under_construction && (millis / 500) % 2 != 0;
        if !blink {
            let arrow_color = if is_highlighted { Color::Rgb(100, 90, 110) } else { Color::Rgb(45, 38, 55) };
            put_cell(buffer, row, pos, '\u{2192}', arrow_color);
            put_cell(buffer, row, pos + 1, node_letter(pipe.to), color);
            put_cell(buffer, row, pos + 2, ' ', LOOM_BG);
        }
        pos += 3;
    }

    // Gap.
    pos += 2;

    // Incoming pipes.
    for pipe in &loom_state.persistent.pipes {
        if pipe.to != node_id || pipe.under_construction {
            continue;
        }
        let is_highlighted = selected
            || selected_node_id == pipe.from;
        let color = if is_highlighted {
            node_color(pipe.from)
        } else {
            Color::Rgb(60, 50, 70)
        };
        let arrow_color = if is_highlighted { Color::Rgb(100, 90, 110) } else { Color::Rgb(45, 38, 55) };
        put_cell(buffer, row, pos, '\u{2190}', arrow_color);
        put_cell(buffer, row, pos + 1, node_letter(pipe.from), color);
        put_cell(buffer, row, pos + 2, ' ', LOOM_BG);
        pos += 3;
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build 2>&1 | tail -5`
Expected: Compiles.

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add port label renderer with selection highlighting"
```

---

### Task 6: Implement sidebar detail panel

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Add sidebar rendering function**

This renders the right sidebar as a `Paragraph` (no cell buffer needed). Shows: node identity, buffer, rate, recipe list with input slots, pipe list, controls.

```rust
/// Render the sidebar detail panel for the selected node.
fn render_flow_sidebar(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &LoomUiState,
) {
    use crate::loom::types::NodeId;
    use crate::loom::recipes::recipes_by_nature;
    use crate::loom::{node_effective_rate, node_upgrade_cost, pipe_flow_rate};

    let nodes = NodeId::ALL;
    let selected_id = nodes[ui.selected_node.min(nodes.len() - 1)];
    let node = match loom_state.persistent.nodes.iter().find(|n| n.id == selected_id) {
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
                Span::styled("\u{2588}".repeat(filled), Style::default().fg(Color::Rgb(100, 80, 160))),
                Span::styled("\u{2591}".repeat(empty), Style::default().fg(Color::Rgb(40, 30, 55))),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.1}h", node.unlock_progress), Style::default().fg(Color::Rgb(100, 80, 160))),
            ]));
        }
    } else {
        // Identity.
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(format!(" Lv.{}", node.level), Style::default().fg(Color::Rgb(120, 90, 160))),
            Span::styled(format!("  {}", node_nature_name(node.id.nature())), Style::default().fg(Color::Rgb(140, 100, 180))),
        ]));

        // Buffer bar.
        let fill = if node.buffer_capacity > 0.0 { (node.buffer / node.buffer_capacity).min(1.0) } else { 0.0 };
        let bar_color = if node.stalled || fill >= 0.90 { Color::Rgb(220, 60, 60) } else if fill >= 0.75 { Color::Rgb(220, 180, 60) } else { Color::Rgb(60, 200, 100) };
        let filled = ((fill * 10.0) as usize).min(10);
        let empty = 10usize.saturating_sub(filled);
        lines.push(Line::from(vec![
            Span::styled(" [", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
            Span::styled("\u{2591}".repeat(empty), Style::default().fg(Color::Rgb(40, 30, 55))),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(Span::styled(
            format!(" {:.1}/{:.0}", node.buffer, node.buffer_capacity),
            Style::default().fg(bar_color),
        )));

        // Rate.
        let rate = node_effective_rate(loom_state, node);
        lines.push(Line::from(Span::styled(
            format!(" +{:.1}/hr", rate),
            Style::default().fg(Color::Rgb(100, 200, 120)),
        )));
        lines.push(Line::from(""));

        // Recipe list.
        let nature = node.id.nature();
        let recipes = recipes_by_nature(nature);

        // Determine incoming resources.
        let incoming: std::collections::HashSet<crate::loom::types::Resource> = loom_state
            .persistent.pipes.iter()
            .filter(|p| p.to == node.id && !p.under_construction)
            .map(|p| crate::loom::node_native_resource(p.from))
            .collect();

        if recipes.is_empty() {
            lines.push(Line::from(Span::styled(" No recipes", Style::default().fg(Color::DarkGray))));
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
                let color = if active { Color::Rgb(200, 180, 240) } else { Color::Rgb(70, 55, 90) };
                lines.push(Line::from(Span::styled(
                    format!("  {}{} {}{} \u{25b6} {}",
                        dot_a, resource_short(&r.input_a),
                        dot_b, resource_short(&r.input_b),
                        resource_short(&r.output)),
                    Style::default().fg(color),
                )));
            }
        }

        lines.push(Line::from(""));

        // Pipe list.
        let outgoing: Vec<_> = loom_state.persistent.pipes.iter().enumerate()
            .filter(|(_, p)| p.from == selected_id && !p.under_construction)
            .collect();
        let incoming_pipes: Vec<_> = loom_state.persistent.pipes.iter().enumerate()
            .filter(|(_, p)| p.to == selected_id && !p.under_construction)
            .collect();

        if !outgoing.is_empty() {
            lines.push(Line::from(Span::styled(" Out:", Style::default().fg(Color::Rgb(100, 80, 130)))));
            for (idx, pipe) in &outgoing {
                let flow = pipe_flow_rate(loom_state, *idx);
                lines.push(Line::from(Span::styled(
                    format!("  \u{2192}{} {:.1} {:?}", node_letter(pipe.to), flow, pipe.tier),
                    Style::default().fg(node_color(pipe.to)),
                )));
            }
        }
        if !incoming_pipes.is_empty() {
            lines.push(Line::from(Span::styled(" In:", Style::default().fg(Color::Rgb(100, 80, 130)))));
            for (idx, pipe) in &incoming_pipes {
                let flow = pipe_flow_rate(loom_state, *idx);
                lines.push(Line::from(Span::styled(
                    format!("  \u{2190}{} {:.1} {:?}", node_letter(pipe.from), flow, pipe.tier),
                    Style::default().fg(node_color(pipe.from)),
                )));
            }
        }

        lines.push(Line::from(""));

        // Controls.
        lines.push(Line::from(Span::styled(" [B]uild  [U]pgr", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled(" [D]emol  [S]plit", Style::default().fg(Color::DarkGray))));
    }

    lines.truncate(inner.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        inner,
    );
}
```

**Step 2: Verify compilation**

Run: `cargo build 2>&1 | tail -5`
Expected: Compiles.

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add flow sidebar detail panel with recipes and pipes"
```

---

### Task 7: Replace `render_flow_view` with new factory floor renderer

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Rewrite `render_flow_view`**

Replace the existing `render_flow_view` function body. The new version:
1. Splits the area horizontally: factory floor (left) + sidebar (right, 22 cols).
2. Splits the factory floor vertically: node grid (top) + pattern bar (bottom, 4 rows).
3. Creates a `SceneCell` buffer for the node grid area.
4. Renders 6 node boxes in 3x2 grid positions with port labels below each.
5. Flushes the buffer with `render_buffer`.
6. Calls `render_flow_sidebar` for the right panel.
7. Calls the existing `render_pattern_bar` for the bottom strip.

The function signature needs `ui: &LoomUiState` added as a parameter.

```rust
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

    // Create cell buffer for the grid area.
    let rows = grid_area.height as usize;
    let cols = grid_area.width as usize;
    let mut buffer = vec![vec![SceneCell::new(' ', Color::Reset, LOOM_BG); cols]; rows];

    // Node grid: 3 rows x 2 columns.
    let grid: [(NodeId, NodeId); 3] = [
        (NodeId::EmberSpindle, NodeId::VoidCondenser),
        (NodeId::ReflectionLens, NodeId::MemoryArchive),
        (NodeId::SilenceWell, NodeId::ResonanceForge),
    ];

    let nodes_arr = NodeId::ALL;
    let selected_id = nodes_arr[ui.selected_node.min(nodes_arr.len() - 1)];

    // Calculate vertical spacing: each node row needs NODE_BOX_HEIGHT + 1 (port labels).
    let row_height = NODE_BOX_HEIGHT + 1; // box + port label row
    let total_grid_height = row_height * 3;
    let v_offset = if rows > total_grid_height { ((rows - total_grid_height) / 2) as i32 } else { 0 };

    // Calculate horizontal spacing: two boxes per row with gap.
    let gap = 4i32;
    let pair_width = NODE_BOX_WIDTH as i32 * 2 + gap;
    let h_offset = if cols as i32 > pair_width { (cols as i32 - pair_width) / 2 } else { 0 };

    for (row_idx, (left_id, right_id)) in grid.iter().enumerate() {
        let top = v_offset + (row_idx * row_height) as i32;
        let left_col = h_offset;
        let right_col = h_offset + NODE_BOX_WIDTH as i32 + gap;

        // Find nodes.
        let left_node = loom_state.persistent.nodes.iter().find(|n| n.id == *left_id);
        let right_node = loom_state.persistent.nodes.iter().find(|n| n.id == *right_id);

        if let Some(node) = left_node {
            let port_row = render_node_box(&mut buffer, top, left_col, node, loom_state, node.id == selected_id);
            render_port_labels(&mut buffer, port_row, left_col, node.id, loom_state, node.id == selected_id, selected_id);
        }

        if let Some(node) = right_node {
            let port_row = render_node_box(&mut buffer, top, right_col, node, loom_state, node.id == selected_id);
            render_port_labels(&mut buffer, port_row, right_col, node.id, loom_state, node.id == selected_id, selected_id);
        }
    }

    // Flush cell buffer to frame.
    render_buffer(frame, grid_area, &buffer);

    // Sidebar.
    render_flow_sidebar(frame, sidebar_area, loom_state, ui);

    // Pattern bar.
    if has_patterns {
        render_pattern_bar(frame, pattern_area, loom_state);
    }
}
```

**Step 2: Update the call site in `render_loom_overlay`**

In `render_loom_overlay`, change the `FlowView` dispatch to pass `ui`:

```rust
LoomView::FlowView => {
    render_flow_view(frame, inner, loom_state, ui);
}
```

**Step 3: Remove old helper functions that are no longer called**

Remove these functions that were only used by the old Flow View:
- `build_flow_node_header_line`
- `build_flow_buffer_line`
- `build_flow_rate_line`
- `render_stockpiles_panel`

Check with grep that they aren't called from anywhere else first.

**Step 4: Verify compilation**

Run: `cargo build 2>&1 | tail -10`
Expected: Compiles. There may be dead code warnings for the removed functions' helpers — that's expected.

**Step 5: Run all Loom tests**

Run: `cargo test --lib loom:: 2>&1 | tail -10`
Expected: All tests pass. The UI changes are render-only, no logic affected.

**Step 6: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): replace Flow View with animated factory floor"
```

---

### Task 8: Update input handling for 2D grid navigation

**Files:**
- Modify: `src/input/loom_input.rs`

**Step 1: Read the current input handler**

Read `src/input/loom_input.rs` to understand the current Up/Down arrow handling for `selected_node`.

**Step 2: Update arrow key navigation for 3x2 grid**

Currently `selected_node` cycles 0-5 linearly. Update to support 2D grid navigation:
- Up/Down: move between rows (selected_node +/- 2)
- Left/Right: move between columns within a row (selected_node +/- 1, clamped to row bounds)

The grid layout is:
```
Index: 0  1
       2  3
       4  5
```

So: Left column = even indices, Right column = odd indices.
- Up: subtract 2 (if >= 2)
- Down: add 2 (if <= 3)
- Left: subtract 1 (if odd)
- Right: add 1 (if even and < 5)

Only apply this when `view == FlowView`. Other views keep their current behavior.

**Step 3: Verify compilation and test**

Run: `cargo build 2>&1 | tail -5`
Run: `cargo test --lib loom:: 2>&1 | tail -5`
Expected: Both pass.

**Step 4: Commit**

```bash
git add src/input/loom_input.rs
git commit -m "feat(loom): add 2D grid navigation for Flow View"
```

---

### Task 9: Visual polish and testing

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Run the game and test visually**

Run: `cargo run`
- Open the Loom (L key or debug menu)
- Verify 3x2 grid of node boxes renders correctly
- Verify animated textures cycle (check Ember waves shift)
- Verify buffer bars show correct fill levels
- Verify port labels appear below nodes with correct letters
- Verify selection highlighting works (arrow keys, bright border)
- Verify sidebar shows recipe list with filled/empty dots
- Verify stalled nodes have frozen/dim textures
- Verify locked nodes show lock icon
- Verify pattern bar renders at bottom

**Step 2: Fix any visual issues found**

Adjust spacing, colors, or layout constants as needed.

**Step 3: Run full CI checks**

Run: `make check`
Expected: All checks pass (fmt, clippy, tests, build).

**Step 4: Commit final polish**

```bash
git add src/ui/loom_scene.rs
git commit -m "fix(loom): visual polish for factory floor Flow View"
```

---

### Task 10: Push and update PR

**Step 1: Push to remote**

```bash
git push
```

**Step 2: Verify CI passes**

Run: `gh run list --limit 1`
Check that the latest run is passing.
