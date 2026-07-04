> Backported implementation plan (completed — this work shipped).

## 2026-03-06-loom-implementation-plan.md

# Loom of Worlds Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the Loom of Worlds — a pipeline-and-recipe factory endgame system unlocked after Deep Layer 30 Gateway completion.

**Architecture:** New `src/loom/` module following the Deep module pattern (account-level state in `~/.quest/loom.json`, tick integration via `TickContext`, overlay UI via `loom_scene.rs`). Six processing nodes with explicit directional pipes, combinatorial recipes driven by node natures, and 18 woven pattern milestones as progression gates.

**Tech Stack:** Rust, Ratatui TUI, Serde JSON persistence, rand for RNG.

**Design docs:**
- [Loom of Worlds Design](2026-03-06-loom-of-worlds-design.md)
- [Starting Archetypes](2026-03-06-loom-starting-archetypes-design.md)
- [Pipeline & Recipe System](2026-03-06-loom-pipeline-recipe-system-design.md)

---

## Phase Overview

| Phase | What | Tasks |
|-------|------|-------|
| 1 | Foundation — types, persistence, discovery, empty UI shell | 1-5 |
| 2 | Archetype Selection — 3 archetypes, staggered unlock | 6-8 |
| 3 | Base Production — nodes produce native resources, upgrading | 9-11 |
| 4 | Pipelines — build/demolish/upgrade, split ratios, bandwidth | 12-15 |
| 5 | Backpressure — buffers, stalling, buffer UI | 16-17 |
| 6 | Node Natures & Recipes — combinatorial system, codex | 18-21 |
| 7 | Woven Patterns — 18 milestones, sustain timer, pattern bar | 22-24 |
| 8 | Integration & Polish — existing system bonuses, combat transition | 25-26 |

Each phase is independently testable and commitable. Phases build on each other sequentially.

---

## Phase 1: Foundation

### Task 1: Create Loom Module Scaffold

**Files:**
- Create: `src/loom/mod.rs`
- Create: `src/loom/types.rs`
- Modify: `src/lib.rs` — add `pub mod loom;`

**Step 1: Write the failing test**

```rust
// tests/loom_types_test.rs (or in src/loom/types.rs #[cfg(test)])
#[test]
fn test_loom_state_default() {
    let state = LoomState::new();
    assert!(!state.persistent.discovered);
    assert_eq!(state.persistent.nodes.len(), 6);
    assert!(state.persistent.pipes.is_empty());
    assert!(state.persistent.codex.is_empty());
    assert_eq!(state.persistent.active_pattern, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_loom_state_default`
Expected: FAIL — `LoomState` not found

**Step 3: Write types.rs**

```rust
// src/loom/types.rs
use serde::{Deserialize, Serialize};

/// Which archetype the player chose at Loom unlock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoomArchetype {
    BurnBright,  // Ember Spindle + Void Condenser
    ReachWide,   // Reflection Lens + Memory Archive
    RunDeep,     // Silence Well + Resonance Forge
}

/// The six node identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeId {
    EmberSpindle,
    ReflectionLens,
    VoidCondenser,
    MemoryArchive,
    SilenceWell,
    ResonanceForge,
}

impl NodeId {
    pub const ALL: [NodeId; 6] = [
        NodeId::EmberSpindle,
        NodeId::ReflectionLens,
        NodeId::VoidCondenser,
        NodeId::MemoryArchive,
        NodeId::SilenceWell,
        NodeId::ResonanceForge,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            NodeId::EmberSpindle => "Ember Spindle",
            NodeId::ReflectionLens => "Reflection Lens",
            NodeId::VoidCondenser => "Void Condenser",
            NodeId::MemoryArchive => "Memory Archive",
            NodeId::SilenceWell => "Silence Well",
            NodeId::ResonanceForge => "Resonance Forge",
        }
    }

    pub fn nature(&self) -> NodeNature {
        match self {
            NodeId::EmberSpindle => NodeNature::Heat,
            NodeId::ReflectionLens => NodeNature::Form,
            NodeId::VoidCondenser => NodeNature::Void,
            NodeId::MemoryArchive => NodeNature::Pattern,
            NodeId::SilenceWell => NodeNature::Stillness,
            NodeId::ResonanceForge => NodeNature::Vibration,
        }
    }
}

/// Node natures — the hidden ingredient in combinatorial recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeNature {
    Heat,
    Form,
    Void,
    Pattern,
    Stillness,
    Vibration,
}

/// The six base resources, each tied to a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Ember,
    Reflection,
    VoidEssence,
    Memory,
    Silence,
    Resonance,
    // Confluence resources
    ForgedLight,
    EchoGlass,
    StillbornSong,
    // Reaction products (discovered via combinatorial recipes)
    CondensedEmber,
    EmberEcho,
    PurifiedVoid,
    WovenReality,
    // Additional reaction products added as recipe design is finalized
}

/// A single processing node in the Loom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomNode {
    pub id: NodeId,
    pub level: u32,
    pub unlocked: bool,
    /// Current stock of the node's buffer.
    pub buffer: f64,
    /// Max buffer capacity (scales with level).
    pub buffer_capacity: f64,
    /// Base production rate per hour (native resource, no pipe inputs needed).
    pub base_rate: f64,
    /// Whether the node is currently stalled (buffer full, nowhere to send output).
    pub stalled: bool,
}

impl LoomNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            level: 1,
            unlocked: false,
            buffer: 0.0,
            buffer_capacity: 20.0, // 4 hours at 5/hr base
            base_rate: 5.0,
            stalled: false,
        }
    }
}

/// Pipe bandwidth tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipeTier {
    T1, // 5/hr
    T2, // 12/hr
    T3, // 25/hr
    T4, // 50/hr
}

impl PipeTier {
    pub fn bandwidth(&self) -> f64 {
        match self {
            PipeTier::T1 => 5.0,
            PipeTier::T2 => 12.0,
            PipeTier::T3 => 25.0,
            PipeTier::T4 => 50.0,
        }
    }
}

/// A directional pipe between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipe {
    pub from: NodeId,
    pub to: NodeId,
    pub tier: PipeTier,
    /// What fraction of the source node's output goes through this pipe (0.0-1.0).
    pub split_ratio: f64,
    /// Whether the pipe is still under construction.
    pub under_construction: bool,
    /// Ticks remaining for construction (0 = complete).
    pub construction_ticks_remaining: u32,
}

/// A discovered recipe in the codex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub inputs: Vec<Resource>,
    pub node_nature: NodeNature,
    pub output: Resource,
    pub output_amount: f64,
    pub discovered: bool,
}

/// Woven Pattern — a progression milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WovenPattern {
    pub index: u32,
    pub name: String,
    pub requirements: Vec<PatternRequirement>,
    pub sustain_seconds: u32,
    /// Seconds sustained so far (pauses if rates dip).
    pub sustained_seconds: u32,
    pub completed: bool,
}

/// A single requirement within a woven pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequirement {
    pub resource: Resource,
    pub rate_per_hour: f64,
}

/// All persistent Loom state (saved to loom.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomPersistent {
    pub discovered: bool,
    pub archetype: Option<LoomArchetype>,
    pub nodes: Vec<LoomNode>,
    pub pipes: Vec<Pipe>,
    pub codex: Vec<CodexEntry>,
    pub active_pattern: usize,
    pub patterns: Vec<WovenPattern>,
    /// Resource stockpiles (indexed by Resource enum).
    pub stockpiles: std::collections::HashMap<Resource, f64>,
    /// Seconds since second archetype node was unlocked (for staggered unlock).
    pub second_node_unlock_elapsed: Option<f64>,
}

impl Default for LoomPersistent {
    fn default() -> Self {
        Self {
            discovered: false,
            archetype: None,
            nodes: NodeId::ALL.iter().map(|&id| LoomNode::new(id)).collect(),
            pipes: Vec::new(),
            codex: Vec::new(),
            active_pattern: 0,
            patterns: Vec::new(),
            stockpiles: std::collections::HashMap::new(),
            second_node_unlock_elapsed: None,
        }
    }
}

/// Top-level Loom state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomState {
    pub persistent: LoomPersistent,
}

impl LoomState {
    pub fn new() -> Self {
        Self {
            persistent: LoomPersistent::default(),
        }
    }
}

impl Default for LoomState {
    fn default() -> Self {
        Self::new()
    }
}

/// Which view the Loom UI is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoomView {
    ArchetypeSelection,
    FlowView,
    ListDetail,
    Codex,
}

/// Runtime-only UI state (not serialized).
#[derive(Debug)]
pub struct LoomUiState {
    pub open: bool,
    pub view: LoomView,
    pub selected_node: usize,
    pub selected_pipe: usize,
    pub selected_archetype: usize,
}

impl LoomUiState {
    pub fn new() -> Self {
        Self {
            open: false,
            view: LoomView::FlowView,
            selected_node: 0,
            selected_pipe: 0,
            selected_archetype: 0,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }
}
```

**Step 4: Write mod.rs**

```rust
// src/loom/mod.rs
pub mod types;

pub use types::{
    CodexEntry, LoomArchetype, LoomNode, LoomPersistent, LoomState,
    LoomUiState, LoomView, NodeId, NodeNature, Pipe, PipeTier,
    Resource, WovenPattern,
};
```

**Step 5: Add to lib.rs**

Add `pub mod loom;` to `src/lib.rs` alongside the other module declarations.

**Step 6: Run test to verify it passes**

Run: `cargo test test_loom_state_default`
Expected: PASS

**Step 7: Commit**

```bash
git add src/loom/ src/lib.rs
git commit -m "feat(loom): add module scaffold with core types"
```

---

### Task 2: Persistence — Save/Load

**Files:**
- Create: `src/loom/persistence.rs`
- Modify: `src/loom/mod.rs` — add `pub mod persistence;` and re-exports

**Step 1: Write the failing test**

```rust
#[test]
fn test_loom_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("loom.json");

    let mut state = LoomState::new();
    state.persistent.discovered = true;
    state.persistent.archetype = Some(LoomArchetype::BurnBright);

    save_loom_to_path(&state, &path).unwrap();
    let loaded = load_loom_from_path(&path);

    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.archetype, Some(LoomArchetype::BurnBright));
    assert_eq!(loaded.persistent.nodes.len(), 6);
}

#[test]
fn test_loom_load_missing_file_returns_default() {
    let loaded = load_loom_from_path(std::path::Path::new("/nonexistent/loom.json"));
    assert!(!loaded.persistent.discovered);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_loom_save_load`
Expected: FAIL — functions not found

**Step 3: Write persistence.rs**

```rust
// src/loom/persistence.rs
use super::types::LoomState;
use std::{fs, io, path::{Path, PathBuf}};

pub fn loom_save_path() -> io::Result<PathBuf> {
    Ok(crate::core::paths::get_quest_dir()?.join("loom.json"))
}

pub fn load_loom() -> LoomState {
    let path = match loom_save_path() {
        Ok(p) => p,
        Err(_) => return LoomState::new(),
    };
    load_loom_from_path(&path)
}

pub fn load_loom_from_path(path: &Path) -> LoomState {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => LoomState::new(),
    }
}

pub fn save_loom(loom: &LoomState) -> io::Result<()> {
    let path = loom_save_path()?;
    save_loom_to_path(loom, &path)
}

pub fn save_loom_to_path(loom: &LoomState, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(loom)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}
```

**Step 4: Update mod.rs**

```rust
// Add to src/loom/mod.rs
pub mod persistence;
pub use persistence::{loom_save_path, load_loom, save_loom};
```

**Step 5: Run tests**

Run: `cargo test test_loom_save_load`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/persistence.rs src/loom/mod.rs
git commit -m "feat(loom): add save/load persistence"
```

---

### Task 3: Discovery Trigger

**Files:**
- Create: `src/loom/discovery.rs`
- Modify: `src/loom/mod.rs` — add `pub mod discovery;`
- Modify: `src/core/tick_types.rs` — add `LoomDiscovered` to `TickEvent`, `loom_changed` to `TickResult`
- Modify: `src/core/tick_context.rs` — add `loom` field to `TickContext`
- Modify: `src/core/tick_stages.rs` — add discovery check
- Modify: `src/core/tick.rs` — wire loom into tick call

The Loom is discovered when the Gateway Expedition at Deep Layer 30 completes. Find the existing code path where Deep mission completion is processed and add the Loom discovery trigger there.

**Step 1: Write the failing test**

```rust
#[test]
fn test_loom_discovery() {
    let mut loom = LoomState::new();
    assert!(!loom.persistent.discovered);

    complete_discovery(&mut loom);

    assert!(loom.persistent.discovered);
    // Should initialize the 18 woven patterns
    assert_eq!(loom.persistent.patterns.len(), 18);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_loom_discovery`
Expected: FAIL — `complete_discovery` not found

**Step 3: Write discovery.rs**

```rust
// src/loom/discovery.rs
use super::types::{LoomState, WovenPattern, PatternRequirement, Resource};

pub fn complete_discovery(loom: &mut LoomState) {
    loom.persistent.discovered = true;
    loom.persistent.patterns = create_pattern_sequence();
}

fn create_pattern_sequence() -> Vec<WovenPattern> {
    vec![
        // Teaching Arc (1-6)
        pattern(0, "First Thread", vec![(Resource::Ember, 2.0)], 1800),
        pattern(1, "The Bridge", vec![(Resource::Ember, 3.0), (Resource::Reflection, 1.0)], 3600),
        pattern(2, "Long Road", vec![(Resource::Ember, 2.0), (Resource::Memory, 1.0)], 3600),
        pattern(3, "Balancing Act", vec![(Resource::Ember, 2.0), (Resource::Reflection, 2.0), (Resource::VoidEssence, 2.0)], 5400),
        pattern(4, "Full Circle", vec![
            (Resource::Ember, 1.0), (Resource::Reflection, 1.0), (Resource::VoidEssence, 1.0),
            (Resource::Memory, 1.0), (Resource::Silence, 1.0), (Resource::Resonance, 1.0),
        ], 7200),
        pattern(5, "The Catalyst", vec![(Resource::CondensedEmber, 1.0)], 7200),
        // Mastery Arc (7-12)
        pattern(6, "Crossed Streams", vec![(Resource::CondensedEmber, 1.0), (Resource::EmberEcho, 1.0)], 7200),
        pattern(7, "The Diversion", vec![(Resource::ForgedLight, 1.0), (Resource::Ember, 3.0)], 9000),
        pattern(8, "Three Confluences", vec![
            (Resource::ForgedLight, 1.0), (Resource::EchoGlass, 1.0), (Resource::StillbornSong, 1.0),
        ], 10800),
        pattern(9, "Pressure Test", vec![(Resource::ForgedLight, 2.0), (Resource::EchoGlass, 2.0)], 10800),
        pattern(10, "The Bottleneck", vec![(Resource::StillbornSong, 3.0)], 10800),
        pattern(11, "Shifting Gears", vec![(Resource::ForgedLight, 3.0)], 7200), // first phase; second phase handled in logic
        // Endgame Arc (13-18)
        pattern(12, "Harmony", vec![
            (Resource::Ember, 5.0), (Resource::Reflection, 5.0), (Resource::VoidEssence, 5.0),
            (Resource::Memory, 5.0), (Resource::Silence, 5.0), (Resource::Resonance, 5.0),
        ], 14400),
        pattern(13, "The Triad", vec![
            (Resource::Ember, 3.0), (Resource::Reflection, 3.0), (Resource::VoidEssence, 3.0),
            (Resource::Memory, 3.0), (Resource::Silence, 3.0), (Resource::Resonance, 3.0),
            (Resource::ForgedLight, 3.0), (Resource::EchoGlass, 3.0), (Resource::StillbornSong, 3.0),
        ], 14400),
        pattern(14, "Razor's Edge", vec![(Resource::ForgedLight, 4.0), (Resource::EchoGlass, 4.0)], 14400),
        pattern(15, "Resonance Cascade", vec![(Resource::Resonance, 10.0)], 14400),
        pattern(16, "The Unraveling", vec![(Resource::WovenReality, 1.0)], 21600),
        pattern(17, "Mended Loom", vec![
            (Resource::WovenReality, 3.0),
            (Resource::Ember, 5.0), (Resource::Reflection, 5.0), (Resource::VoidEssence, 5.0),
            (Resource::Memory, 5.0), (Resource::Silence, 5.0), (Resource::Resonance, 5.0),
            (Resource::ForgedLight, 3.0), (Resource::EchoGlass, 3.0), (Resource::StillbornSong, 3.0),
        ], 28800),
    ]
}

fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64)>, sustain_seconds: u32) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, rate)| PatternRequirement {
                resource,
                rate_per_hour: rate,
            })
            .collect(),
        sustain_seconds,
        sustained_seconds: 0,
        completed: false,
    }
}
```

**Step 4: Update mod.rs, add re-exports**

**Step 5: Add `LoomDiscovered` variant to `TickEvent` in `src/core/tick_types.rs`**

Add `loom_changed: bool` field to `TickResult` with default `false`.

**Step 6: Add `loom: &'a mut LoomState` to `TickContext` in `src/core/tick_context.rs`**

**Step 7: Wire discovery into tick_stages.rs**

Find where Deep mission completion is processed (the Gateway expedition at Layer 30). After the existing Deep mission completion logic, add:

```rust
// After Gateway mission completes:
if !ctx.loom.persistent.discovered {
    loom::complete_discovery(ctx.loom);
    result.events.push(TickEvent::LoomDiscovered);
    result.loom_changed = true;
}
```

**Step 8: Run tests**

Run: `cargo test test_loom_discovery`
Expected: PASS

Run: `cargo test` (full suite)
Expected: PASS — existing tests still work with new TickContext field

**Step 9: Commit**

```bash
git add src/loom/discovery.rs src/loom/mod.rs src/core/
git commit -m "feat(loom): add discovery trigger on Gateway completion"
```

---

### Task 4: Wire Into Main Loop

**Files:**
- Modify: `src/main.rs` — load loom state, create UI state, pass to TickContext, handle events, save
- Modify: `src/main_helpers/persistence.rs` — add loom to save_files
- Modify: `src/main_helpers/game_context.rs` — add loom_ui to GameContext
- Modify: `src/input/types.rs` — add `LoomDiscovery` to `GameOverlay`

**Step 1: Add `LoomDiscovery` to `GameOverlay` in `src/input/types.rs`**

**Step 2: Load loom state in `main.rs`**

Near the other module state loading (Deep, Haven, etc.):
```rust
let mut loom_state = loom::load_loom();
let mut loom_ui = loom::LoomUiState::new();
```

**Step 3: Pass loom to TickContext construction**

Add `loom: &mut loom_state` to the TickContext struct literal.

**Step 4: Handle `LoomDiscovered` event in apply_tick_events**

Push `GameOverlay::LoomDiscovery` onto pending_overlays.

**Step 5: Handle `loom_changed` in save logic**

Add `|| tick_result.loom_changed` to the save condition.

**Step 6: Add loom to save_files signature and call**

**Step 7: Run full test suite and verify game compiles**

Run: `cargo build && cargo test`
Expected: PASS

**Step 8: Commit**

```bash
git add src/main.rs src/main_helpers/ src/input/types.rs
git commit -m "feat(loom): wire into main loop, persistence, and discovery overlay"
```

---

### Task 5: Empty UI Shell

**Files:**
- Create: `src/ui/loom_scene.rs`
- Modify: `src/ui/mod.rs` — add `pub mod loom_scene;`
- Create: `src/input/loom_input.rs`
- Modify: `src/input/mod.rs` — add loom input handling
- Modify: `src/main_helpers/overlay.rs` — render loom overlay

**Step 1: Create loom_scene.rs with minimal render**

```rust
// src/ui/loom_scene.rs
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use crate::loom::{LoomState, LoomUiState, LoomView};

pub fn render_loom_scene(
    frame: &mut Frame,
    area: Rect,
    loom: &LoomState,
    ui: &LoomUiState,
) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" The Loom of Worlds ")
        .borders(Borders::ALL);

    match ui.view {
        LoomView::ArchetypeSelection => {
            let text = Paragraph::new("Choose your archetype (coming soon)")
                .block(block);
            frame.render_widget(text, area);
        }
        LoomView::FlowView => {
            let text = Paragraph::new("Flow View (coming soon)")
                .block(block);
            frame.render_widget(text, area);
        }
        LoomView::ListDetail => {
            let text = Paragraph::new("List + Detail View (coming soon)")
                .block(block);
            frame.render_widget(text, area);
        }
        LoomView::Codex => {
            let text = Paragraph::new("Recipe Codex (coming soon)")
                .block(block);
            frame.render_widget(text, area);
        }
    }
}
```

**Step 2: Create loom_input.rs with minimal handler**

```rust
// src/input/loom_input.rs
use crossterm::event::{KeyCode, KeyEvent};
use crate::loom::{LoomState, LoomUiState, LoomView};
use super::types::InputResult;

pub fn handle_loom(
    key: KeyEvent,
    loom: &mut LoomState,
    ui: &mut LoomUiState,
) -> InputResult {
    match key.code {
        KeyCode::Esc => {
            ui.open = false;
            InputResult::Continue
        }
        KeyCode::Tab => {
            ui.view = match ui.view {
                LoomView::FlowView => LoomView::ListDetail,
                LoomView::ListDetail => LoomView::Codex,
                LoomView::Codex => LoomView::FlowView,
                LoomView::ArchetypeSelection => LoomView::ArchetypeSelection,
            };
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
```

**Step 3: Wire into input/mod.rs**

Add `mod loom_input;` and the loom overlay check in the input priority chain (after Deep, before Vault):
```rust
if loom_ui.open {
    return loom_input::handle_loom(key, loom_state, loom_ui);
}
```

Add keybinding in `handle_base_game()`:
```rust
KeyCode::Char('l') | KeyCode::Char('L') => {
    if loom_state.persistent.discovered {
        loom_ui.open();
    }
    InputResult::Continue
}
```

**Step 4: Wire into overlay.rs**

Add loom scene rendering in `draw_game_overlays()`:
```rust
if loom_ui.open {
    crate::ui::loom_scene::render_loom_scene(frame, frame.area(), &loom_state, &loom_ui);
}
```

Add `GameOverlay::LoomDiscovery` to the discovery modal dismiss block.

**Step 5: Wire into ui/mod.rs**

Add `pub mod loom_scene;`

**Step 6: Build and verify**

Run: `cargo build`
Expected: Compiles. Opening the Loom shows placeholder text. Esc closes. Tab cycles views.

**Step 7: Commit**

```bash
git add src/ui/loom_scene.rs src/input/loom_input.rs src/ui/mod.rs src/input/mod.rs src/main_helpers/overlay.rs
git commit -m "feat(loom): add empty UI shell with view cycling"
```

---

## Phase 2: Archetype Selection

### Task 6: Archetype Selection UI

**Files:**
- Modify: `src/ui/loom_scene.rs` — implement archetype selection rendering
- Modify: `src/input/loom_input.rs` — handle archetype selection input

Render the 3-archetype selection screen when `loom.persistent.archetype.is_none()` and the Loom is discovered. Arrow keys select, Enter confirms.

Refer to the selection screen mockup in `docs/plans/2026-03-06-loom-starting-archetypes-design.md`.

**Step 1: Write test for archetype selection logic**

```rust
#[test]
fn test_select_archetype_burn_bright() {
    let mut loom = LoomState::new();
    loom.persistent.discovered = true;

    select_archetype(&mut loom, LoomArchetype::BurnBright);

    assert_eq!(loom.persistent.archetype, Some(LoomArchetype::BurnBright));
    // First node (Ember Spindle) unlocked immediately
    let ember = loom.persistent.nodes.iter().find(|n| n.id == NodeId::EmberSpindle).unwrap();
    assert!(ember.unlocked);
    // Second node (Void Condenser) NOT yet unlocked (staggered)
    let void_n = loom.persistent.nodes.iter().find(|n| n.id == NodeId::VoidCondenser).unwrap();
    assert!(!void_n.unlocked);
    // Staggered unlock timer started
    assert_eq!(loom.persistent.second_node_unlock_elapsed, Some(0.0));
}
```

**Step 2: Implement `select_archetype()` in logic.rs**

**Step 3: Implement archetype selection UI rendering**

**Step 4: Implement archetype selection input handling (up/down to select, Enter to confirm)**

**Step 5: Auto-route to ArchetypeSelection view when archetype is None**

**Step 6: Run tests and verify**

**Step 7: Commit**

```bash
git commit -m "feat(loom): add archetype selection screen"
```

---

### Task 7: Archetype Passives

**Files:**
- Create: `src/loom/logic.rs`
- Modify: `src/loom/types.rs` — add passive-related fields if needed

Implement the 6 passive bonuses (one per archetype node):
- Ember Spindle: +50% throughput, neighbors unlock 30% slower
- Void Condenser: 2x conversion ratio at levels 1-3
- Reflection Lens: unlocks 3 neighbors instead of 2
- Memory Archive: starts with stockpile of 3 adjacent resources
- Silence Well: -25% upgrade costs for first 5 levels
- Resonance Forge: feedback loop at 50% strength before cycle closes

**Step 1: Write tests for each passive**

**Step 2: Implement passive application in logic.rs**

**Step 3: Run tests**

**Step 4: Commit**

```bash
git commit -m "feat(loom): implement archetype passive bonuses"
```

---

### Task 8: Staggered Second Node Unlock

**Files:**
- Modify: `src/loom/logic.rs` — tick the staggered unlock timer
- Modify: `src/core/tick_stages.rs` — call loom tick

The second archetype node unlocks ~4 hours (14400 seconds) after archetype selection. Implement the timer in the Loom tick function.

**Step 1: Write test**

```rust
#[test]
fn test_staggered_unlock_after_4_hours() {
    let mut loom = LoomState::new();
    loom.persistent.discovered = true;
    select_archetype(&mut loom, LoomArchetype::BurnBright);

    // Simulate 4 hours of ticks (14400 seconds)
    tick_loom_staggered_unlock(&mut loom, 14400.0);

    let void_n = loom.persistent.nodes.iter().find(|n| n.id == NodeId::VoidCondenser).unwrap();
    assert!(void_n.unlocked);
}
```

**Step 2: Implement staggered unlock tick logic**

**Step 3: Wire `tick_loom()` into `tick_stages.rs` and `tick.rs`**

**Step 4: Run tests**

**Step 5: Commit**

```bash
git commit -m "feat(loom): staggered second node unlock after 4 hours"
```

---

## Phase 3: Base Production

### Task 9: Node Base Production

Unlocked nodes produce their native resource each tick. Production rate = `base_rate * level_multiplier * passive_bonuses`. Output goes to the node's buffer.

### Task 10: Node Upgrading

Upgrade a node's level using resources. Cost scales with level. Implements the -25% cost passive for Silence Well.

### Task 11: Neighbor Unlocking

When a node produces enough, adjacent nodes begin unlocking. Implements the 3-neighbor passive for Reflection Lens and the 30% slower unlock for Ember Spindle.

---

## Phase 4: Pipelines

### Task 12: Pipe Data Model & Building

Build directional pipes between nodes. 2-hour construction time. Resource cost. Max 3 outgoing, 3 incoming per node.

### Task 13: Pipe Flow Simulation

Each tick, resources flow through pipes based on split ratios and bandwidth caps. Source buffer drains, destination buffer fills.

### Task 14: Split Ratio Adjustment UI

Free ratio adjustment via the List+Detail view. Arrow keys to select pipe, left/right to adjust ratio.

### Task 15: Pipe Upgrading & Demolishing

Upgrade bandwidth tier (instant, costs resources). Demolish pipe (instant, 50% refund).

---

## Phase 5: Backpressure

### Task 16: Buffer Stalling

When a node's buffer is full and no outgoing pipe can accept more, the node stalls. Production stops. Visual indicator in UI.

### Task 17: Buffer UI

Buffer bars on each node in Flow View showing fill percentage. Stall warning indicators.

---

## Phase 6: Node Natures & Recipes

### Task 18: Recipe Registry

Define the ~35-40 hand-designed recipes as a static registry. Each recipe maps (Input A, Input B, NodeNature) → (Output, Amount).

### Task 19: Reaction Processing

When a node receives two different resources via pipes, look up the recipe in the registry. If found, produce the output. If not found, no reaction (inputs accumulate in buffer).

### Task 20: Recipe Discovery & Codex

Track which recipes have been discovered. Show discovered recipes in the Codex view. Show "???" hints for adjacent undiscovered recipes.

### Task 21: Codex UI

Full codex view accessible via Tab cycling. Lists discovered recipes, shows input/output/node, hints for adjacent unknowns.

---

## Phase 7: Woven Patterns

### Task 22: Pattern Tracking

Each tick, check if the active pattern's requirements are all met (production rates sustained). If yes, increment the sustain timer. If any rate dips, pause the timer.

### Task 23: Pattern Completion & Progression

When sustain timer reaches the required duration, mark pattern complete. Unlock next pattern. Grant rewards (tier unlocks, node upgrade caps, etc.).

### Task 24: Pattern Bar UI

Always-visible bar at bottom of Flow View showing current pattern name, requirements with checkmarks, and sustain progress bar.

---

## Phase 8: Integration & Polish

### Task 25: Existing System Bonuses

Deep, Haven, Stormglass, and Ascension provide small production bonuses to Loom nodes. Meaningful early, negligible late.

### Task 26: Flow View & List+Detail View Polish

Full ASCII flow diagram with pipe routing, live rates, buffer bars, and confluence nodes. List+Detail view with upgrade controls, bottleneck indicators, and pipe management.

---

## Notes for Implementer

- **Follow Deep module as reference** for all patterns (persistence, tick integration, input, UI)
- **`#[serde(default)]`** on all persistent struct fields for backward compatibility
- **No `ui::` imports in tick/logic code** — tick.rs has zero UI coupling
- **Generic `<R: Rng>`** for any functions that need randomness
- **Run `make check` before every push** — same checks as CI
- Use **`cargo test` after every task** to verify no regressions

## 2026-03-07-loom-flow-view-plan.md

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

## 2026-03-07-loom-power-integration-plan.md

# Loom Power Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect the Loom of Worlds back to the main game: Ascension VII–X gated by pattern milestones, shuttle upgrades with progressive level caps, WR→PR generation at endgame, and 20 new Loom Zones (Z31–50) with 1.25x stat scaling.

**Architecture:** Extend the existing Ascension system to support levels 7–10 with a new pattern gate (alongside the existing Deep gate). Add a `completed_pattern_count()` helper to the Loom module. Add shuttle upgrade logic using the existing `Shuttle.level` field. Add WR→PR tick processing alongside Power Cores. Extend the zone data table from 30 to 50 entries with a new `LOOM_ZONE_STAT_MULTIPLIER` constant. Extend `sync_account_zone_unlocks` to handle Loom zone access.

**Tech Stack:** Rust, Ratatui, Serde (JSON persistence)

---

### Task 1: Add `completed_pattern_count()` helper to Loom

This helper is used by every downstream task (Ascension gating, shuttle caps, zone unlocks).

**Files:**
- Modify: `src/loom/types.rs`
- Modify: `src/loom/mod.rs`

**Step 1: Write the failing test**

Add to the bottom of the `#[cfg(test)] mod tests` block in `src/loom/types.rs`:

```rust
#[test]
fn test_completed_pattern_count_empty() {
    let state = LoomState::new();
    assert_eq!(state.persistent.completed_pattern_count(), 0);
}

#[test]
fn test_completed_pattern_count_some_completed() {
    let mut state = LoomState::new();
    state.persistent.patterns.push(WovenPattern {
        index: 0,
        name: "A".to_string(),
        requirements: vec![],
        completed: true,
    });
    state.persistent.patterns.push(WovenPattern {
        index: 1,
        name: "B".to_string(),
        requirements: vec![],
        completed: false,
    });
    assert_eq!(state.persistent.completed_pattern_count(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests::test_completed_pattern_count -- --no-capture`
Expected: FAIL with "no method named `completed_pattern_count`"

**Step 3: Write minimal implementation**

In `src/loom/types.rs`, add a method to the `impl LoomPersistent` block (right after `max_shuttles()`):

```rust
/// Number of completed Woven Patterns.
pub fn completed_pattern_count(&self) -> usize {
    self.patterns.iter().filter(|p| p.completed).count()
}
```

Note: `max_shuttles()` already does the same computation. Refactor `max_shuttles()` to call this:

```rust
pub fn max_shuttles(&self) -> usize {
    self.completed_pattern_count()
}
```

**Step 4: Add re-export in `src/loom/mod.rs`**

No re-export needed — `completed_pattern_count()` is a method on `LoomPersistent` which is already public.

**Step 5: Run test to verify it passes**

Run: `cargo test --lib loom::types::tests::test_completed_pattern_count`
Expected: PASS (both tests)

**Step 6: Commit**

```bash
git add src/loom/types.rs
git commit -m "feat(loom): add completed_pattern_count() helper"
```

---

### Task 2: Extend Ascension to support levels VII–X with pattern gates

Change `MAX_ASCENSION_LEVEL` from 6 to 10, add new cost table entries for levels 7–10, and add a new `ascension_pattern_gate()` function.

**Files:**
- Modify: `src/ascension/types.rs`
- Modify: `src/ascension/logic.rs`

**Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `src/ascension/types.rs`:

```rust
#[test]
fn test_ascension_cost_levels_7_through_10_loom() {
    assert_eq!(ascension_cost(7), 1500);
    assert_eq!(ascension_cost(8), 4000);
    assert_eq!(ascension_cost(9), 8000);
    assert_eq!(ascension_cost(10), 15000);
}

#[test]
fn test_ascension_pattern_gate() {
    assert_eq!(ascension_pattern_gate(1), None);
    assert_eq!(ascension_pattern_gate(6), None);
    assert_eq!(ascension_pattern_gate(7), Some(8));
    assert_eq!(ascension_pattern_gate(8), Some(16));
    assert_eq!(ascension_pattern_gate(9), Some(22));
    assert_eq!(ascension_pattern_gate(10), Some(28));
}

#[test]
fn test_ascension_combat_multiplier_levels_7_through_10() {
    assert!((ascension_combat_multiplier(7) - 96.0).abs() < 1e-10);
    assert!((ascension_combat_multiplier(8) - 144.0).abs() < 1e-10);
    assert!((ascension_combat_multiplier(9) - 216.0).abs() < 1e-10);
    assert!((ascension_combat_multiplier(10) - 324.0).abs() < 1e-10);
}

#[test]
fn test_max_shuttle_level_for_ascension() {
    assert_eq!(max_shuttle_level(0), 1);
    assert_eq!(max_shuttle_level(6), 1);
    assert_eq!(max_shuttle_level(7), 3);
    assert_eq!(max_shuttle_level(8), 5);
    assert_eq!(max_shuttle_level(9), 7);
    assert_eq!(max_shuttle_level(10), 10);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib ascension::types::tests -- --no-capture`
Expected: FAIL — `ascension_cost(7)` returns 575 (old formula), `ascension_pattern_gate` and `max_shuttle_level` don't exist.

**Step 3: Write the implementation**

In `src/ascension/types.rs`:

1. Change `MAX_ASCENSION_LEVEL` from 6 to 10.

2. Add the Loom Ascension cost table for levels 7–10. Replace the `ascension_cost()` function:

```rust
/// Loom-gated Ascension costs for levels 7-10.
const LOOM_ASCENSION_COSTS: [u32; 4] = [1500, 4000, 8000, 15000];

/// Loom pattern gates for Ascension levels 7-10.
const LOOM_ASCENSION_PATTERN_GATES: [usize; 4] = [8, 16, 22, 28];

/// Max shuttle level per Ascension tier (7-10).
const LOOM_SHUTTLE_LEVEL_CAPS: [u32; 4] = [3, 5, 7, 10];

/// Prestige rank cost to Ascend to the given level.
pub fn ascension_cost(level: u32) -> u32 {
    if (1..=6).contains(&level) {
        ASCENSION_COSTS[(level - 1) as usize]
    } else if (7..=10).contains(&level) {
        LOOM_ASCENSION_COSTS[(level - 7) as usize]
    } else {
        0
    }
}
```

3. Add the pattern gate function:

```rust
/// Woven Pattern gate for the given Ascension level.
/// Returns None for levels 1-6 (gated by Deep layers instead).
/// Returns Some(required_patterns) for levels 7-10.
pub fn ascension_pattern_gate(level: u32) -> Option<usize> {
    if (7..=10).contains(&level) {
        Some(LOOM_ASCENSION_PATTERN_GATES[(level - 7) as usize])
    } else {
        None
    }
}
```

4. Add the shuttle level cap function:

```rust
/// Maximum shuttle upgrade level allowed at the given Ascension level.
/// Returns 1 (no upgrades) for levels 0-6, progressive caps for 7-10.
pub fn max_shuttle_level(ascension_level: u32) -> u32 {
    if (7..=10).contains(&ascension_level) {
        LOOM_SHUTTLE_LEVEL_CAPS[(ascension_level - 7) as usize]
    } else {
        1
    }
}
```

5. Fix the existing test `test_ascension_cost_level_7_plus` — the old formula `500 + 75*(level-6)` no longer applies. Remove or update it:

```rust
#[test]
fn test_ascension_cost_level_0_returns_zero() {
    assert_eq!(ascension_cost(0), 0);
}

#[test]
fn test_ascension_cost_level_11_plus_returns_zero() {
    assert_eq!(ascension_cost(11), 0);
    assert_eq!(ascension_cost(100), 0);
}
```

6. Fix the existing test `test_total_pr_for_levels_1_through_6` — it should still pass (values 1-6 unchanged).

**Step 4: Update `src/ascension/logic.rs`**

The `can_ascend()` and `ascend()` functions need to check the pattern gate for levels 7+. They currently take `deepest_layer: u32`. Add a `completed_patterns: usize` parameter.

Update `can_ascend()`:

```rust
pub fn can_ascend(
    ascension_level: u32,
    prestige_rank: u32,
    deepest_layer: u32,
    completed_patterns: usize,
) -> bool {
    if ascension_level >= super::types::MAX_ASCENSION_LEVEL {
        return false;
    }
    let next = ascension_level + 1;
    let cost = super::types::ascension_cost(next);
    if prestige_rank < cost {
        return false;
    }
    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return false;
        }
    }
    if let Some(pattern_gate) = super::types::ascension_pattern_gate(next) {
        if completed_patterns < pattern_gate {
            return false;
        }
    }
    true
}
```

Add a new `AscendResult` variant:

```rust
/// Woven Pattern requirement not met.
PatternGateNotMet {
    needed_patterns: usize,
    current_patterns: usize,
},
```

Update `ascend()` to take `completed_patterns: usize` and check the pattern gate:

```rust
pub fn ascend(
    state: &mut crate::core::game_state::GameState,
    deepest_layer: u32,
    completed_patterns: usize,
) -> AscendResult {
    if state.ascension_level >= super::types::MAX_ASCENSION_LEVEL {
        return AscendResult::MaxLevelReached;
    }

    let next = state.ascension_level + 1;
    let cost = super::types::ascension_cost(next);

    if state.prestige_rank < cost {
        return AscendResult::InsufficientPR {
            needed: cost,
            have: state.prestige_rank,
        };
    }

    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return AscendResult::DeepGateNotMet {
                needed_layer: gate,
                current_layer: deepest_layer,
            };
        }
    }

    if let Some(pattern_gate) = super::types::ascension_pattern_gate(next) {
        if completed_patterns < pattern_gate {
            return AscendResult::PatternGateNotMet {
                needed_patterns: pattern_gate,
                current_patterns: completed_patterns,
            };
        }
    }

    state.prestige_rank -= cost;
    state.ascension_level = next;
    let multiplier = super::types::ascension_combat_multiplier(next);

    AscendResult::Success {
        new_level: next,
        multiplier,
    }
}
```

**Step 5: Fix all callers of `can_ascend()` and `ascend()`**

Search for all call sites. They need the extra `completed_patterns` parameter. For now, pass `0` where Loom state isn't available, or thread Loom state through. The call sites are:

- `src/ui/ascension_scene.rs` — `can_ascend(current_level, state.prestige_rank, deepest)` → add `completed_patterns` param. The render function needs to accept `&LoomState` (or just `completed_patterns: usize`).
- `src/input/` — wherever ascension input is handled. Search for `ascend(` calls.
- Any tests in `ascension/logic.rs`.

Run: `grep -rn "can_ascend\|ascend(" src/ --include="*.rs" | grep -v test | grep -v "//"`

Update each call site to pass `completed_patterns`. For UI rendering, add `loom: &LoomState` parameter to `render_ascension_confirm()` and use `loom.persistent.completed_pattern_count()`.

**Step 6: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/ascension/types.rs src/ascension/logic.rs src/ui/ascension_scene.rs
git commit -m "feat(ascension): extend to levels VII-X with Loom pattern gates"
```

---

### Task 3: Add shuttle upgrade logic

Implement shuttle level upgrades using the existing `Shuttle.level` field. Apply the level multiplier to intake caps. Gate upgrades behind Ascension VII+ with progressive level caps.

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/loom/mod.rs`

**Step 1: Write the failing tests**

Add to the test module in `src/loom/logic.rs`:

```rust
#[test]
fn test_shuttle_level_intake_multiplier() {
    // T1 shuttle at level 1: intake cap = 20.0
    assert!((shuttle_effective_intake_cap(1, 1) - 20.0).abs() < 0.001);
    // T1 shuttle at level 3: 20.0 * (1.0 + (3-1)*0.5) = 20.0 * 2.0 = 40.0
    assert!((shuttle_effective_intake_cap(1, 3) - 40.0).abs() < 0.001);
    // T3 shuttle at level 5: 40.0 * (1.0 + (5-1)*0.5) = 40.0 * 3.0 = 120.0
    assert!((shuttle_effective_intake_cap(3, 5) - 120.0).abs() < 0.001);
}

#[test]
fn test_upgrade_shuttle_success() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    setup_patterns(&mut loom, 8); // Need Asc VII for shuttle upgrades
    // Unlock all nodes for shuttle building
    for node in loom.persistent.nodes.iter_mut() {
        node.unlocked = true;
    }
    // Build a T1 shuttle
    let recipes = crate::loom::recipes::all_recipes();
    let t1_idx = recipes.iter().position(|r| r.tier == 1).unwrap();
    let _ = build_shuttle(
        &mut loom,
        t1_idx,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    // Give the shuttle enough buffer to afford the upgrade
    loom.persistent.shuttles[0].buffer = 500.0;
    loom.persistent.shuttles[0].under_construction = false;

    let result = upgrade_shuttle(&mut loom, 0, 7); // ascension_level = 7
    assert!(result.is_ok());
    assert_eq!(loom.persistent.shuttles[0].level, 2);
}

#[test]
fn test_upgrade_shuttle_blocked_by_ascension_cap() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    setup_patterns(&mut loom, 8);
    for node in loom.persistent.nodes.iter_mut() {
        node.unlocked = true;
    }
    let recipes = crate::loom::recipes::all_recipes();
    let t1_idx = recipes.iter().position(|r| r.tier == 1).unwrap();
    let _ = build_shuttle(
        &mut loom,
        t1_idx,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    loom.persistent.shuttles[0].buffer = 5000.0;
    loom.persistent.shuttles[0].under_construction = false;
    // Upgrade to level 3 (max for Asc VII)
    loom.persistent.shuttles[0].level = 3;

    let result = upgrade_shuttle(&mut loom, 0, 7); // at cap for Asc VII
    assert!(result.is_err());
}

#[test]
fn test_upgrade_shuttle_blocked_without_ascension_vii() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    setup_patterns(&mut loom, 1);
    for node in loom.persistent.nodes.iter_mut() {
        node.unlocked = true;
    }
    let recipes = crate::loom::recipes::all_recipes();
    let t1_idx = recipes.iter().position(|r| r.tier == 1).unwrap();
    let _ = build_shuttle(
        &mut loom,
        t1_idx,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    loom.persistent.shuttles[0].buffer = 5000.0;
    loom.persistent.shuttles[0].under_construction = false;

    let result = upgrade_shuttle(&mut loom, 0, 6); // no Asc VII
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::logic::tests::test_shuttle_level_intake -- --no-capture`
Expected: FAIL — functions don't exist yet.

**Step 3: Write the implementation**

In `src/loom/logic.rs`:

1. Add `shuttle_effective_intake_cap()`:

```rust
/// Effective intake cap for a shuttle, applying the level multiplier.
/// Formula: tier_intake_cap(tier) × node_level_multiplier(level)
pub fn shuttle_effective_intake_cap(tier: u8, level: u32) -> f64 {
    tier_intake_cap(tier) * node_level_multiplier(level)
}
```

2. Modify `tick_shuttle_pull()` to use `shuttle_effective_intake_cap()` instead of `tier_intake_cap()`. In the shuttle processing loop, change:

```rust
let cap = tier_intake_cap(r.tier);
```

to:

```rust
let cap = shuttle_effective_intake_cap(r.tier, r.level);
```

3. Add the shuttle upgrade function:

```rust
/// Error type for shuttle upgrade failures.
#[derive(Debug, Clone, PartialEq)]
pub enum ShuttleUpgradeError {
    /// Invalid shuttle index.
    InvalidIndex,
    /// Shuttle is under construction.
    UnderConstruction,
    /// Ascension level too low for shuttle upgrades (need VII+).
    AscensionTooLow,
    /// Shuttle already at max level for current Ascension tier.
    AtMaxLevel,
    /// Not enough output resource in shuttle buffer.
    InsufficientBuffer { needed: f64, have: f64 },
}

/// Attempt to upgrade a shuttle's level.
/// Cost is the same formula as node upgrades: 100 × level^1.5, paid from shuttle buffer.
/// Max level is capped by the player's Ascension level via max_shuttle_level().
pub fn upgrade_shuttle(
    loom: &mut LoomState,
    shuttle_idx: usize,
    ascension_level: u32,
) -> Result<(), ShuttleUpgradeError> {
    let max_level = crate::ascension::types::max_shuttle_level(ascension_level);
    if max_level <= 1 {
        return Err(ShuttleUpgradeError::AscensionTooLow);
    }

    let shuttle = loom
        .persistent
        .shuttles
        .get(shuttle_idx)
        .ok_or(ShuttleUpgradeError::InvalidIndex)?;

    if shuttle.under_construction {
        return Err(ShuttleUpgradeError::UnderConstruction);
    }

    if shuttle.level >= max_level {
        return Err(ShuttleUpgradeError::AtMaxLevel);
    }

    let cost = 100.0 * (shuttle.level as f64).powf(1.5);
    if shuttle.buffer < cost {
        return Err(ShuttleUpgradeError::InsufficientBuffer {
            needed: cost,
            have: shuttle.buffer,
        });
    }

    let shuttle = loom.persistent.shuttles.get_mut(shuttle_idx).unwrap();
    shuttle.buffer -= cost;
    shuttle.level += 1;

    Ok(())
}
```

4. Add re-exports in `src/loom/mod.rs`:

```rust
pub use logic::{shuttle_effective_intake_cap, upgrade_shuttle, ShuttleUpgradeError};
```

**Step 4: Run tests**

Run: `cargo test --lib loom::logic::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs
git commit -m "feat(loom): add shuttle upgrade logic with Ascension-gated level caps"
```

---

### Task 4: Add WR→PR tick processing

Add a per-tick WR→PR conversion that runs when all 28 patterns are complete (Ascension X unlocked). Uses a tiered bracket system. Follows the same architecture as `tick_power_cores()`.

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/loom/mod.rs`
- Modify: `src/core/tick_types.rs` (new TickEvent variant)
- Modify: `src/core/tick_stages.rs` (call the new function from `tick_loom`)

**Step 1: Write the failing tests**

Add to the test module in `src/loom/logic.rs`:

```rust
#[test]
fn test_wr_to_pr_per_day_zero_rate() {
    assert_eq!(wr_to_pr_per_day(0.0), 0);
}

#[test]
fn test_wr_to_pr_per_day_low_bracket() {
    // 5 WR/hr → 5 * 5 = 25 PR/day
    assert_eq!(wr_to_pr_per_day(5.0), 25);
}

#[test]
fn test_wr_to_pr_per_day_mid_bracket() {
    // 20 WR/hr → (10 * 5) + (10 * 10) = 50 + 100 = 150 PR/day
    assert_eq!(wr_to_pr_per_day(20.0), 150);
}

#[test]
fn test_wr_to_pr_per_day_high_bracket() {
    // 60 WR/hr → (10 * 5) + (15 * 10) + (35 * 15) = 50 + 150 + 525 = 725 PR/day
    assert_eq!(wr_to_pr_per_day(60.0), 725);
}

#[test]
fn test_wr_to_pr_per_day_exact_bracket_boundary() {
    // 10 WR/hr → 10 * 5 = 50 PR/day
    assert_eq!(wr_to_pr_per_day(10.0), 50);
    // 25 WR/hr → (10 * 5) + (15 * 10) = 50 + 150 = 200 PR/day
    assert_eq!(wr_to_pr_per_day(25.0), 200);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::logic::tests::test_wr_to_pr_per_day -- --no-capture`
Expected: FAIL — function doesn't exist.

**Step 3: Write the implementation**

In `src/loom/logic.rs`, add the bracket calculator:

```rust
/// Calculate PR generated per day from a given WR production rate (units/hr).
///
/// Tiered brackets:
/// - 0–10 WR/hr: 5 PR per WR/hr per day
/// - 10–25 WR/hr: 10 PR per WR/hr per day
/// - 25+ WR/hr: 15 PR per WR/hr per day
pub fn wr_to_pr_per_day(wr_per_hour: f64) -> u32 {
    if wr_per_hour <= 0.0 {
        return 0;
    }

    let mut pr = 0.0;
    let mut remaining = wr_per_hour;

    // Bracket 1: 0–10 at 5 PR per WR/hr
    let b1 = remaining.min(10.0);
    pr += b1 * 5.0;
    remaining -= b1;

    // Bracket 2: 10–25 at 10 PR per WR/hr
    if remaining > 0.0 {
        let b2 = remaining.min(15.0);
        pr += b2 * 10.0;
        remaining -= b2;
    }

    // Bracket 3: 25+ at 15 PR per WR/hr
    if remaining > 0.0 {
        pr += remaining * 15.0;
    }

    pr.round() as u32
}
```

**Step 4: Run tests**

Run: `cargo test --lib loom::logic::tests::test_wr_to_pr_per_day`
Expected: PASS

**Step 5: Add the tick function**

In `src/loom/logic.rs`, add the per-tick WR→PR grant function. This follows the `tick_power_cores()` pattern using wall-clock time:

```rust
/// Tick WR→PR conversion. Called from tick_loom() each game tick.
///
/// Only active when all 28 patterns are complete. Reads the WR production
/// rate from the rate tracker, calculates PR/day, and grants PR at the
/// appropriate wall-clock interval.
///
/// Returns the number of PR granted this tick (0 in most ticks).
pub fn tick_wr_to_pr(
    loom: &LoomState,
    state: &mut crate::core::game_state::GameState,
) -> u32 {
    if !crate::loom::all_patterns_complete(&loom.persistent) {
        return 0;
    }

    // Get current WR production rate from rate tracker.
    let wr_rate = loom
        .rate_trackers
        .get(&Resource::WovenReality)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);

    let pr_per_day = wr_to_pr_per_day(wr_rate);
    if pr_per_day == 0 {
        return 0;
    }

    // Use the same wall-clock interval pattern as Power Cores.
    // PR is granted once per fill cycle: 86400 / pr_per_day seconds.
    let fill_secs = 86400i64 / pr_per_day as i64;
    let now = chrono::Utc::now().timestamp();
    let last = loom.persistent.wr_pr_last_granted_at;

    if last == 0 {
        // First tick with WR→PR active — don't grant, just initialise.
        return 0; // Caller sets the timestamp.
    }

    let elapsed = now - last;
    if elapsed < fill_secs {
        return 0;
    }

    let completed_cycles = (elapsed / fill_secs) as u32;
    state.prestige_rank = state.prestige_rank.saturating_add(completed_cycles);
    state.recalculate_prestige_bonuses();
    state.derived_stats_dirty = true;

    completed_cycles
}
```

**Step 6: Add persistence field**

In `src/loom/types.rs`, add to `LoomPersistent`:

```rust
/// Unix timestamp of last WR→PR grant (wall-clock, like Power Cores).
#[serde(default)]
pub wr_pr_last_granted_at: i64,
```

And in the `Default` impl:

```rust
wr_pr_last_granted_at: 0,
```

**Step 7: Add TickEvent variant**

In `src/core/tick_types.rs`, add a new variant to the `TickEvent` enum:

```rust
/// Woven Reality production granted prestige ranks.
WovenRealityPRGranted { pr_amount: u32, wr_rate: f64 },
```

**Step 8: Wire into tick_loom()**

In `src/core/tick_stages.rs`, at the end of `tick_loom()` (after the pattern sustain block), add:

```rust
// Tick WR→PR conversion (active after all 28 patterns complete).
if crate::loom::all_patterns_complete(&loom.persistent) {
    let now = chrono::Utc::now().timestamp();
    // Initialise timestamp on first tick.
    if loom.persistent.wr_pr_last_granted_at == 0 {
        loom.persistent.wr_pr_last_granted_at = now;
        result.loom_changed = true;
    }

    let wr_rate = loom
        .rate_trackers
        .get(&crate::loom::Resource::WovenReality)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);

    let pr_per_day = crate::loom::wr_to_pr_per_day(wr_rate);
    if pr_per_day > 0 {
        let fill_secs = 86400i64 / pr_per_day as i64;
        let last = loom.persistent.wr_pr_last_granted_at;
        let elapsed = now - last;
        if elapsed >= fill_secs {
            let completed_cycles = (elapsed / fill_secs) as u32;
            // Need GameState access — this function currently doesn't have it.
            // We'll need to add state parameter to tick_loom() — see integration note below.
        }
    }
}
```

**Integration note:** `tick_loom()` currently takes `(deep, loom, result)`. It needs `state: &mut GameState` to grant PR. Update the signature and the call in `src/core/tick.rs`:

In `src/core/tick_stages.rs`, change:
```rust
pub(super) fn tick_loom(
    deep: &crate::deep::DeepState,
    loom: &mut crate::loom::LoomState,
    state: &mut crate::core::game_state::GameState,
    result: &mut TickResult,
)
```

In `src/core/tick.rs`, update the call:
```rust
tick_stages::tick_loom(ctx.deep, ctx.loom, ctx.state, &mut result);
```

The full WR→PR grant logic in `tick_loom()`:

```rust
// Tick WR→PR conversion (active after all 28 patterns complete).
if crate::loom::all_patterns_complete(&loom.persistent) {
    let now = chrono::Utc::now().timestamp();
    if loom.persistent.wr_pr_last_granted_at == 0 {
        loom.persistent.wr_pr_last_granted_at = now;
        result.loom_changed = true;
    } else {
        let wr_rate = loom
            .rate_trackers
            .get(&crate::loom::Resource::WovenReality)
            .map(|t| t.rate_per_hour())
            .unwrap_or(0.0);
        let pr_per_day = crate::loom::wr_to_pr_per_day(wr_rate);
        if pr_per_day > 0 {
            let fill_secs = 86400i64 / pr_per_day as i64;
            let last = loom.persistent.wr_pr_last_granted_at;
            let elapsed = now - last;
            if elapsed >= fill_secs {
                let completed_cycles = (elapsed / fill_secs) as u32;
                state.prestige_rank = state.prestige_rank.saturating_add(completed_cycles);
                state.recalculate_prestige_bonuses();
                state.derived_stats_dirty = true;
                loom.persistent.wr_pr_last_granted_at = last + fill_secs * completed_cycles as i64;
                for _ in 0..completed_cycles {
                    result.events.push(TickEvent::WovenRealityPRGranted {
                        pr_amount: 1,
                        wr_rate,
                    });
                }
                result.loom_changed = true;
            }
        }
    }
}
```

**Step 9: Add re-export**

In `src/loom/mod.rs`:
```rust
pub use logic::wr_to_pr_per_day;
```

**Step 10: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 11: Commit**

```bash
git add src/loom/logic.rs src/loom/types.rs src/loom/mod.rs src/core/tick_types.rs src/core/tick_stages.rs src/core/tick.rs
git commit -m "feat(loom): add WR→PR tiered bracket generation system"
```

---

### Task 5: Add Loom Zones (Z31–50) data

Extend the zone data table with 20 new Loom-themed zones. Add zone definitions, enemy stat scaling at 1.25x per zone, and the `LOOM_ZONE_STAT_MULTIPLIER` constant.

**Files:**
- Modify: `src/core/constants.rs`
- Modify: `src/zones/data.rs`

**Step 1: Write the failing tests**

Add to `src/core/constants.rs` tests:

```rust
#[test]
fn test_zone_enemy_stats_has_50_entries() {
    assert_eq!(ZONE_ENEMY_STATS.len(), 50);
}

#[test]
fn test_loom_zone_stat_multiplier() {
    assert!((LOOM_ZONE_STAT_MULTIPLIER - 1.25).abs() < 1e-10);
}

#[test]
fn test_loom_zone_constants() {
    assert_eq!(FIRST_LOOM_ZONE_ID, 31);
    assert_eq!(LAST_LOOM_ZONE_ID, 50);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib core::constants::tests`
Expected: FAIL — array has 30 entries, constants don't exist.

**Step 3: Write the implementation**

In `src/core/constants.rs`:

1. Add new constants:

```rust
pub const FIRST_LOOM_ZONE_ID: u32 = 31;
pub const LAST_LOOM_ZONE_ID: u32 = 50;
pub const LOOM_ZONE_STAT_MULTIPLIER: f64 = 1.25;
```

2. Extend `ZONE_ENEMY_STATS` from 30 to 50 entries. Calculate each zone's stats as `Zone30_base × 1.25^(zone_id - 30)`. Zone 30 base stats are `(37778883, 3022365, 3777930, 604515, 1888978, 226688)`.

Zone 31 = Zone 30 × 1.25:
```
(47223604, 3777956, 4722413, 755644, 2361223, 283360)
```

Continue the pattern for all 20 zones. Use a comment block:

```rust
// Loom Zones — 1.25x exponential scaling from Zone 30
(47_223_604, 3_777_956, 4_722_413, 755_644, 2_361_223, 283_360),  // Zone 31
(59_029_505, 4_722_445, 5_903_016, 944_555, 2_951_529, 354_200),  // Zone 32
// ... (calculate all 20 entries)
```

**Important**: Use `u32` max (4,294,967,295) as a ceiling. Some stats for very high zones may exceed u32 range — use u64 if needed, OR cap at u32::MAX. Check whether the existing enemy generation code uses u32 — it does (see `calc_zone_enemy_stats` returning `(u32, u32, u32)`). Zone 50 at 1.25^20 ≈ 86.7x Zone 30 base:
- HP: 37778883 × 86.7 ≈ 3.27 billion (fits u32 max of 4.29 billion)
- DMG: 3777930 × 86.7 ≈ 327 million (fits u32)

All values fit within u32. Calculate all 20 entries precisely:

For each zone z (31–50): `stat = (Zone30_stat as f64 × 1.25^(z-30)).round() as u32`

3. Fix the existing test `test_zone_enemy_stats_has_30_entries` — change to 50.

**Step 4: Add zone definitions in `src/zones/data.rs`**

Add 20 new zone entries to the `ALL_ZONES` LazyLock vec. Each zone needs:
- `id: 31..=50`
- `name` and `description` — Loom-themed names (woven realms)
- `subzones: Vec<Subzone>` — 5 subzones each (like fracture zones)
- `prestige_requirement` — progressive: P400 (Z31-34), P500 (Z35-38), P600 (Z39-42), P700 (Z43-46), P800 (Z47-50)
- `min_level` and `max_level` — continuing from Z30
- `requires_weapon: false`
- `weapon_name: None`

Loom-themed zone names (20 zones across 5 chapters):

**Ch.7: The Thread Wilds (Z31-34)**
- Z31: Threadbare Wastes
- Z32: Spindle Hollow
- Z33: The Weft Expanse
- Z34: Heart of the Thread Wilds

**Ch.8: The Woven Frontier (Z35-38)**
- Z35: Loom's Edge
- Z36: Shuttle Run
- Z37: The Pattern Gate
- Z38: Heart of the Woven Frontier

**Ch.9: The Unraveling (Z39-42)**
- Z39: Frayed Reaches
- Z40: The Loose Ends
- Z41: Tangle of Fates
- Z42: Heart of the Unraveling

**Ch.10: The Grand Design (Z43-46)**
- Z43: The Blueprint Halls
- Z44: Architect's Loom
- Z45: Tapestry of Stars
- Z46: Heart of the Grand Design

**Ch.11: The Final Weave (Z47-50)**
- Z47: The Last Shuttle
- Z48: Reality's Seam
- Z49: The World Loom
- Z50: The Origin Thread

**Step 5: Run tests**

Run: `cargo test`
Expected: PASS

**Step 6: Commit**

```bash
git add src/core/constants.rs src/zones/data.rs
git commit -m "feat(zones): add 20 Loom Zones (Z31-50) with 1.25x stat scaling"
```

---

### Task 6: Add Loom Zone unlock gating

Extend `sync_account_zone_unlocks` to handle Loom zone access based on completed pattern count. Add a `loom_zone_cap` concept similar to `fracture_zone_cap`.

**Files:**
- Modify: `src/zones/access.rs`
- Modify: `src/loom/logic.rs` (add `loom_zone_cap_for_patterns()`)
- Modify: `src/loom/mod.rs`

**Step 1: Write the failing tests**

Add to `src/loom/logic.rs` tests:

```rust
#[test]
fn test_loom_zone_cap_for_patterns() {
    assert_eq!(loom_zone_cap_for_patterns(0), 30);  // No Loom zones
    assert_eq!(loom_zone_cap_for_patterns(3), 30);  // Not enough
    assert_eq!(loom_zone_cap_for_patterns(4), 34);  // First tier
    assert_eq!(loom_zone_cap_for_patterns(7), 34);  // Still first tier
    assert_eq!(loom_zone_cap_for_patterns(8), 38);  // Second tier
    assert_eq!(loom_zone_cap_for_patterns(15), 38); // Still second tier
    assert_eq!(loom_zone_cap_for_patterns(16), 42); // Third tier
    assert_eq!(loom_zone_cap_for_patterns(21), 42); // Still third tier
    assert_eq!(loom_zone_cap_for_patterns(22), 46); // Fourth tier
    assert_eq!(loom_zone_cap_for_patterns(27), 46); // Still fourth tier
    assert_eq!(loom_zone_cap_for_patterns(28), 50); // All patterns = all zones
}
```

Add to `src/zones/access.rs` tests:

```rust
#[test]
fn test_sync_unlocks_loom_zones_31_to_34() {
    let mut prog = ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 400, 34);
    assert!(prog.is_zone_unlocked(31));
    assert!(prog.is_zone_unlocked(34));
    assert!(!prog.is_zone_unlocked(35));
}

#[test]
fn test_sync_does_not_unlock_loom_zones_beyond_cap() {
    let mut prog = ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 800, 34);
    assert!(prog.is_zone_unlocked(34));
    assert!(!prog.is_zone_unlocked(35));
}
```

**Step 2: Run tests to verify they fail**

Expected: FAIL — functions don't exist.

**Step 3: Write the implementation**

In `src/loom/logic.rs`:

```rust
/// Returns the highest zone ID unlocked by the given completed pattern count.
///
/// | Patterns | Zones Unlocked |
/// |----------|----------------|
/// | 4        | Z31–34         |
/// | 8        | Z35–38         |
/// | 16       | Z39–42         |
/// | 22       | Z43–46         |
/// | 28       | Z47–50         |
pub fn loom_zone_cap_for_patterns(completed_patterns: usize) -> u32 {
    if completed_patterns >= 28 {
        50
    } else if completed_patterns >= 22 {
        46
    } else if completed_patterns >= 16 {
        42
    } else if completed_patterns >= 8 {
        38
    } else if completed_patterns >= 4 {
        34
    } else {
        30 // No Loom zones
    }
}
```

In `src/loom/mod.rs`:
```rust
pub use logic::loom_zone_cap_for_patterns;
```

In `src/zones/access.rs`, add a `loom_zone_cap` parameter to `sync_account_zone_unlocks`:

```rust
pub fn sync_account_zone_unlocks(
    prog: &mut ZoneProgression,
    storms_end_unlocked: bool,
    fracture_zone_cap: u32,
    prestige_rank: u32,
    loom_zone_cap: u32,
) {
    // ... existing logic for zones 11 and 12..=fracture_zone_cap ...

    // Loom zones 31..=loom_zone_cap
    let zones = crate::zones::data::get_all_zones();
    for zone_id in 31..=loom_zone_cap {
        if let Some(zone) = zones.iter().find(|z| z.id == zone_id) {
            if prestige_rank >= zone.prestige_requirement {
                prog.unlock_zone(zone_id);
            }
        }
    }
}
```

**Step 4: Fix all callers of `sync_account_zone_unlocks`**

Search for call sites and add the `loom_zone_cap` parameter. The call sites need access to Loom state to compute `loom_zone_cap_for_patterns(loom.persistent.completed_pattern_count())`. Initially, callers without Loom access can pass `30` (no Loom zones).

The main call site in `tick_stages.rs` (fracture region unlock) already has access to `loom` — pass the computed cap.

**Step 5: Wire zone cap computation into tick_loom**

In `tick_stages.rs`, when a pattern completes, recompute the zone cap and call `sync_account_zone_unlocks` if it changed:

```rust
if pattern_completed {
    result.loom_changed = true;
    let new_cap = crate::loom::loom_zone_cap_for_patterns(
        loom.persistent.completed_pattern_count()
    );
    // Store and compare — if changed, sync zone unlocks
    // This requires access to state.zone_progression — add state param
}
```

**Step 6: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs src/zones/access.rs src/core/tick_stages.rs
git commit -m "feat(zones): add Loom zone unlock gating (Z31-50 via pattern milestones)"
```

---

### Task 7: Add Loom zone cycling (boss defeat behavior)

Extend boss defeat logic so Loom cap zones cycle like fracture cap zones. Add a `LoomZoneCycle` variant to `BossDefeatResult`.

**Files:**
- Modify: `src/zones/boss_defeat.rs`
- Modify: `src/zones/advancement.rs` (if boss defeat flow goes through here)

**Step 1: Write the failing tests**

Add to `src/zones/boss_defeat.rs` tests:

```rust
#[test]
fn test_loom_zone_cap_cycles() {
    let mut prog = ZoneProgression::new();
    // Unlock zones up to 34
    for z in 1..=34 {
        prog.unlock_zone(z);
    }
    prog.current_zone_id = 34;
    prog.current_subzone_id = 5; // Last subzone
    prog.fighting_boss = true;

    let result = on_boss_defeated_with_cap(
        &mut prog,
        400,
        &mut Achievements::default(),
        30,  // fracture_zone_cap
        34,  // loom_zone_cap
    );

    // Zone 34 is the Loom cap — should cycle back to subzone 1
    assert!(matches!(result, BossDefeatResult::LoomZoneCycle { zone_id: 34 }));
    assert_eq!(prog.current_subzone_id, 1);
}
```

**Step 2: Run to verify failure**

**Step 3: Implement**

Add `LoomZoneCycle { zone_id: u32 }` variant to `BossDefeatResult` enum.

In `on_boss_defeated_with_cap()`, add a `loom_zone_cap: u32` parameter. After the fracture cycling check, add:

```rust
// If this is the Loom zone cap, cycle back to subzone 1
if zone_id >= 31 && zone_id == loom_zone_cap && zone_id <= 50 {
    prog.current_subzone_id = 1;
    prog.kills_in_subzone = 0;
    prog.fighting_boss = false;
    return BossDefeatResult::LoomZoneCycle { zone_id };
}
```

**Step 4: Fix all callers** of `on_boss_defeated_with_cap()` to pass `loom_zone_cap`.

**Step 5: Run tests, commit**

```bash
git add src/zones/boss_defeat.rs src/zones/mod.rs
git commit -m "feat(zones): add Loom zone cycling for cap zones"
```

---

### Task 8: Update Ascension UI for Loom gates

Show pattern requirement for Ascension VII–X in the confirmation dialog. Show "Requires: N Woven Patterns" instead of "No Deep requirement" for levels 7+.

**Files:**
- Modify: `src/ui/ascension_scene.rs`

**Step 1: Update render function signature**

Add `loom: &crate::loom::LoomState` to `render_ascension_confirm()`.

**Step 2: Add pattern gate display**

After the Deep gate display block, add:

```rust
if let Some(required_patterns) = ascension_pattern_gate(next_level) {
    let current_patterns = loom.persistent.completed_pattern_count();
    let met = current_patterns >= required_patterns;
    let color = if met { Color::Green } else { Color::Red };
    lines.push(Line::from(vec![
        Span::styled("Requires: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{} Woven Patterns", required_patterns),
            Style::default().fg(color),
        ),
        Span::styled(
            format!("  (completed: {})", current_patterns),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
}
```

**Step 3: Fix all callers** that call `render_ascension_confirm()` — pass the Loom state.

**Step 4: Run `cargo test`, verify compilation**

**Step 5: Commit**

```bash
git add src/ui/ascension_scene.rs
git commit -m "feat(ui): show Woven Pattern gate in Ascension dialog for VII-X"
```

---

### Task 9: Update input handling for ascension with pattern gates

Update the ascension input handler to pass `completed_patterns` to `can_ascend()` and `ascend()`.

**Files:**
- Search for: `grep -rn "can_ascend\|ascend(" src/input/ --include="*.rs"`
- Modify: whichever input file handles the ascension 'Y' key press

**Step 1: Find the call site**

Run: `grep -rn "ascend(" src/input/ --include="*.rs"`

**Step 2: Update the call**

Pass `loom.persistent.completed_pattern_count()` as the `completed_patterns` parameter to both `can_ascend()` and `ascend()`.

**Step 3: Run tests, commit**

```bash
git commit -m "feat(input): pass completed patterns to ascension gating"
```

---

### Task 10: Add Loom zone enemy name prefixes/suffixes

Add Loom-themed enemy name generation for zones 31–50 so combat feels distinct.

**Files:**
- Modify: `src/combat/enemy_generation.rs`

**Step 1: Add name data**

In `get_zone_enemy_prefixes()`, add match arms for zones 31–50:

```rust
31..=34 => &["Threadbare", "Woven", "Spindle", "Weft", "Loom"],
35..=38 => &["Shuttle", "Pattern", "Weave", "Fabric", "Tapestry"],
39..=42 => &["Frayed", "Unraveled", "Tangled", "Knotted", "Snarled"],
43..=46 => &["Grand", "Architect", "Blueprint", "Design", "Schema"],
47..=50 => &["Final", "Origin", "Reality", "World", "Infinite"],
```

Similarly for `get_zone_enemy_suffixes()`:

```rust
31..=34 => &["Weaver", "Spinner", "Threader", "Bobbin", "Shuttle"],
35..=38 => &["Loomguard", "Weftwalker", "Patternborn", "Fabricant", "Threadseeker"],
39..=42 => &["Unmaker", "Raveler", "Tanglefoe", "Knotter", "Splicer"],
43..=46 => &["Architect", "Designer", "Schemer", "Artificer", "Crafter"],
47..=50 => &["Worldweaver", "Realityborn", "Originkeeper", "Threadmaster", "Loombinder"],
```

**Step 2: Run tests, commit**

```bash
git add src/combat/enemy_generation.rs
git commit -m "feat(combat): add Loom zone enemy name prefixes/suffixes for Z31-50"
```

---

### Task 11: Update CLAUDE.md documentation

Update the module documentation files to reflect all new systems.

**Files:**
- Modify: `src/ascension/CLAUDE.md`
- Modify: `src/loom/CLAUDE.md`
- Modify: `src/zones/CLAUDE.md`
- Modify: `CLAUDE.md` (root)

**Step 1: Update Ascension CLAUDE.md**

Add Ascension VII–X to the table:

```markdown
| VII | 8 Patterns | 1,500 PR | 96x |
| VIII | 16 Patterns | 4,000 PR | 144x |
| IX | 22 Patterns | 8,000 PR | 216x |
| X | 28 Patterns | 15,000 PR | 324x |
```

Add `ascension_pattern_gate(level)` and `max_shuttle_level(ascension_level)` to the Key Functions section. Update MAX_ASCENSION_LEVEL to 10.

**Step 2: Update Loom CLAUDE.md**

Add sections for:
- Shuttle upgrades (level multiplier on intake cap, Ascension-gated level caps)
- WR→PR generation (tiered brackets, activation condition)
- `completed_pattern_count()`, `loom_zone_cap_for_patterns()`, `wr_to_pr_per_day()`, `upgrade_shuttle()`, `shuttle_effective_intake_cap()`

**Step 3: Update Zones CLAUDE.md**

Add Loom Zones section:
- Ch.7–11 zone names and prestige requirements
- 1.25x stat scaling multiplier
- Pattern-gated unlock table
- `LoomZoneCycle` boss defeat result

**Step 4: Update root CLAUDE.md**

Add key constants:
- Loom Zone stat multiplier: 1.25x
- Ascension VII–X costs and multipliers
- WR→PR brackets
- Shuttle level caps per Ascension tier

**Step 5: Commit**

```bash
git add src/ascension/CLAUDE.md src/loom/CLAUDE.md src/zones/CLAUDE.md CLAUDE.md
git commit -m "docs: update module docs for Loom power integration"
```

---

### Task 12: Run full CI checks

**Step 1: Run `make check`**

```bash
make check
```

Expected: All checks pass (format, clippy, tests, build, audit).

**Step 2: Fix any issues found**

Address any compilation errors, clippy warnings, or test failures.

**Step 3: Commit fixes if needed**

```bash
git commit -m "fix: address CI check issues from Loom power integration"
```

## 2026-03-07-loom-refineries-plan.md

# Loom Refineries Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add buildable, recipe-locked processing nodes (Refineries) to the Loom of Worlds, creating multi-step Factorio-style production chains below the existing 6 Extractor nodes.

**Architecture:** Introduce a unified `LoomNodeRef` addressing type that covers both fixed Extractors (`NodeId`) and dynamic Refineries (by index). Refineries are stored in a new `Vec<Refinery>` on `LoomPersistent`. Pipes, flow simulation, and reactions all operate on `LoomNodeRef` instead of `NodeId`. The UI renders Refineries in a scrollable processing area below the existing 3x2 Extractor grid. Pattern completion gates which Refinery tiers can be built; resource costs gate each instance.

**Tech Stack:** Rust, Serde (JSON persistence), Ratatui (terminal UI), existing `scene_fx` cell buffer rendering

---

## Context for the Implementer

### Current Architecture

The Loom has 6 fixed nodes identified by the `NodeId` enum (`EmberSpindle`, `VoidCondenser`, etc.). Each is a `LoomNode` struct stored in `loom.persistent.nodes: Vec<LoomNode>`. Pipes connect nodes using `NodeId` for `from`/`to` fields. The pipe flow system (`pipes.rs`) finds nodes by iterating `loom.persistent.nodes` and matching on `NodeId`. Recipes are looked up by `(input_a, input_b, node_nature)`.

### Key Files

| File | What it does |
|------|-------------|
| `src/loom/types.rs` | `NodeId` enum, `LoomNode`, `Pipe`, `LoomPersistent`, `LoomState`, `LoomUiState` |
| `src/loom/pipes.rs` | Pipe building, flow simulation, split ratios — all use `NodeId` |
| `src/loom/logic.rs` | Base production, stall detection, node upgrades, neighbor unlocking, reactions |
| `src/loom/recipes.rs` | Recipe registry, `lookup_recipe(a, b, nature)`, `recipes_by_nature()` |
| `src/loom/patterns.rs` | Woven Pattern sustain timer and completion |
| `src/loom/discovery.rs` | 18 patterns defined in `create_pattern_sequence()` |
| `src/ui/loom_scene.rs` | Flow View rendering with cell buffer, sidebar, node boxes |
| `src/input/loom_input.rs` | Keyboard navigation (2D grid for FlowView, list for ListDetail) |
| `src/core/tick_stages.rs:987` | `tick_loom()` — calls base production, pipe flow, reactions, patterns |

### Design Decisions

1. **Refineries are recipe-locked**: Each Refinery runs exactly one recipe, chosen at build time. It has no base production — it only processes piped-in resources.
2. **3-tier production chains**: T1 Refineries process base→derived recipes, T2 process derived→derived, T3 process high-tier recipes. Tier is determined by the recipe's tier field.
3. **Pattern-gated unlocks**: Completing Woven Patterns unlocks the ability to build Refinery tiers. T1 after pattern 1, T2 after pattern 6, T3 after pattern 12.
4. **Resource costs**: Building a Refinery costs stockpile resources. T1: 25 of a base resource. T2: 15 of a T1 product. T3: 10 of a T2 product.
5. **Refinery limit**: Max Refineries = number of completed patterns. Starts at 1 after first pattern, grows to 18 max.
6. **Layout**: Extractors stay in fixed top 3x2 grid. Refineries appear in a scrollable processing area below, 2 columns wide.

---

### Task 1: Add LoomNodeRef and Refinery types

**Files:**
- Modify: `src/loom/types.rs`

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `src/loom/types.rs`:

```rust
#[test]
fn test_loom_node_ref_equality() {
    let ext_a = LoomNodeRef::Extractor(NodeId::EmberSpindle);
    let ext_b = LoomNodeRef::Extractor(NodeId::EmberSpindle);
    let ref_a = LoomNodeRef::Refinery(0);
    let ref_b = LoomNodeRef::Refinery(0);
    let ref_c = LoomNodeRef::Refinery(1);
    assert_eq!(ext_a, ext_b);
    assert_eq!(ref_a, ref_b);
    assert_ne!(ext_a, ref_a);
    assert_ne!(ref_a, ref_c);
}

#[test]
fn test_refinery_new() {
    use super::Resource;
    let r = Refinery::new(
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        Resource::ForgedLight,
        1.0,
        1,
    );
    assert_eq!(r.input_a, Resource::Ember);
    assert_eq!(r.input_b, Resource::VoidEssence);
    assert_eq!(r.nature, NodeNature::Heat);
    assert_eq!(r.output, Resource::ForgedLight);
    assert!((r.amount - 1.0).abs() < 0.001);
    assert_eq!(r.tier, 1);
    assert!(!r.stalled);
    assert!((r.buffer - 0.0).abs() < 0.001);
    assert!((r.buffer_capacity - 20.0).abs() < 0.001);
    assert_eq!(r.level, 1);
}

#[test]
fn test_loom_state_default_has_empty_refineries() {
    let state = LoomState::new();
    assert!(state.persistent.refineries.is_empty());
}

#[test]
fn test_refinery_limit_zero_with_no_patterns() {
    let state = LoomState::new();
    assert_eq!(state.persistent.max_refineries(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `LoomNodeRef`, `Refinery`, `max_refineries` not defined

**Step 3: Write minimal implementation**

Add the following types to `src/loom/types.rs` (before the `LoomNode` struct):

```rust
/// Unified address for any node in the Loom — either a fixed Extractor or a built Refinery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoomNodeRef {
    /// One of the 6 fixed extractor nodes.
    Extractor(NodeId),
    /// A player-built refinery, identified by index in `LoomPersistent::refineries`.
    Refinery(usize),
}

/// A player-built processing node that runs a single locked recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinery {
    /// First input resource for this refinery's locked recipe.
    pub input_a: Resource,
    /// Second input resource for this refinery's locked recipe.
    pub input_b: Resource,
    /// The nature catalyst for this refinery's recipe.
    pub nature: NodeNature,
    /// Output resource produced.
    pub output: Resource,
    /// Output amount multiplier from the recipe.
    pub amount: f64,
    /// Recipe tier (1, 2, or 3).
    pub tier: u8,
    /// Current buffer level (holds output resource).
    #[serde(default)]
    pub buffer: f64,
    /// Buffer capacity.
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    /// Refinery level (for future upgrades).
    #[serde(default = "default_node_level")]
    pub level: u32,
    /// Whether this refinery is stalled (missing inputs).
    #[serde(default)]
    pub stalled: bool,
    /// Whether currently under construction.
    #[serde(default)]
    pub under_construction: bool,
    /// Ticks remaining for construction.
    #[serde(default)]
    pub construction_ticks_remaining: u32,
}

impl Refinery {
    pub fn new(
        input_a: Resource,
        input_b: Resource,
        nature: NodeNature,
        output: Resource,
        amount: f64,
        tier: u8,
    ) -> Self {
        Self {
            input_a,
            input_b,
            nature,
            output,
            amount,
            tier,
            buffer: 0.0,
            buffer_capacity: 20.0,
            level: 1,
            stalled: false,
            under_construction: false,
            construction_ticks_remaining: 0,
        }
    }
}
```

Add `refineries` field to `LoomPersistent`:

```rust
#[serde(default)]
pub refineries: Vec<Refinery>,
```

And add the `max_refineries` method to `LoomPersistent`:

```rust
impl LoomPersistent {
    /// Maximum number of Refineries the player can build.
    /// Equal to the number of completed Woven Patterns.
    pub fn max_refineries(&self) -> usize {
        self.patterns.iter().filter(|p| p.completed).count()
    }
}
```

Update `Default for LoomPersistent` to include `refineries: Vec::new()`.

Update the `pub use types::` line in `src/loom/mod.rs` to include `LoomNodeRef` and `Refinery`.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::types -- --nocapture 2>&1 | tail -10`
Expected: PASS — all 5 type tests pass

**Step 5: Commit**

```bash
git add src/loom/types.rs src/loom/mod.rs
git commit -m "feat(loom): add LoomNodeRef and Refinery types"
```

---

### Task 2: Migrate Pipe from/to from NodeId to LoomNodeRef

**Files:**
- Modify: `src/loom/types.rs` (Pipe struct)
- Modify: `src/loom/pipes.rs` (all pipe functions)
- Modify: `src/loom/logic.rs` (process_reactions, node_native_resource usage)
- Modify: `src/ui/loom_scene.rs` (port labels, sidebar pipe display)
- Modify: `src/input/loom_input.rs` (pipe selection)

This is the largest task — it touches every file that references `pipe.from` or `pipe.to`. The migration is mechanical: change `Pipe::from` and `Pipe::to` from `NodeId` to `LoomNodeRef`, then update every call site to construct `LoomNodeRef::Extractor(node_id)` where it previously used bare `NodeId`.

**Step 1: Change the Pipe struct**

In `src/loom/types.rs`, change:
```rust
pub struct Pipe {
    pub from: LoomNodeRef,
    pub to: LoomNodeRef,
    // ... rest unchanged
}
```

**Step 2: Update pipes.rs function signatures**

Change `build_pipe` to accept `LoomNodeRef` for `from`/`to`. Update all helper functions (`outgoing_pipe_count`, `incoming_pipe_count`, `pipe_exists`, `total_split_ratio`, `normalize_split_ratios`) to accept `LoomNodeRef`.

In the flow simulation (`tick_pipe_flow`), the node lookup must now handle both `LoomNodeRef::Extractor(id)` (finds in `loom.persistent.nodes`) and `LoomNodeRef::Refinery(idx)` (indexes into `loom.persistent.refineries`). Add a helper:

```rust
/// Resolve a LoomNodeRef to buffer/capacity/rate info.
fn resolve_node_ref(loom: &LoomState, node_ref: LoomNodeRef) -> Option<(f64, f64, f64, bool)> {
    match node_ref {
        LoomNodeRef::Extractor(id) => {
            let node = loom.persistent.nodes.iter().find(|n| n.id == id)?;
            if !node.unlocked { return None; }
            let rate = crate::loom::logic::node_effective_rate(loom, node);
            Some((node.buffer, node.buffer_capacity, rate, node.unlocked))
        }
        LoomNodeRef::Refinery(idx) => {
            let r = loom.persistent.refineries.get(idx)?;
            if r.under_construction { return None; }
            Some((r.buffer, r.buffer_capacity, 0.0, true))
        }
    }
}
```

Similarly add `resolve_node_ref_mut` for applying transfers.

**Step 3: Update logic.rs**

In `process_reactions`, when looking up pipe destinations, match on `LoomNodeRef::Extractor(id)` to get the node's nature for recipe lookup. For `LoomNodeRef::Refinery(idx)`, use the refinery's `nature` field.

In `node_native_resource`, this remains `NodeId`-based (only Extractors have native resources). Pipe flow for Refineries uses the refinery's `output` as the resource it sends downstream.

**Step 4: Update UI files**

In `src/ui/loom_scene.rs`, port labels use `pipe.from`/`pipe.to` to show colored letters. For `LoomNodeRef::Extractor(id)`, use `node_letter(id)`. For `LoomNodeRef::Refinery(idx)`, use `R` with a numeric suffix (e.g., `R1`, `R2`).

In `src/input/loom_input.rs`, the pipe selection logic uses `NodeId::ALL[loom_ui.selected_node]`. Wrap this in `LoomNodeRef::Extractor(...)` for Extractor selection. Refinery selection will be handled in Task 6.

**Step 5: Fix all compilation errors**

Run `cargo build` and fix each error. Most are mechanical `NodeId` → `LoomNodeRef::Extractor(NodeId)` wrapping.

**Step 6: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: All existing tests pass (no behavior change, just type widening)

**Step 7: Commit**

```bash
git add src/loom/
git commit -m "refactor(loom): migrate Pipe from/to from NodeId to LoomNodeRef"
```

---

### Task 3: Add Refinery building logic

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/loom/mod.rs` (re-exports)

**Step 1: Write the failing tests**

Add to `src/loom/logic.rs` test module:

```rust
#[test]
fn test_build_refinery_success() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // Give pattern completion for capacity.
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true;
    // Stock resources.
    *loom.persistent.stockpiles.entry(Resource::Ember).or_insert(0.0) += 50.0;

    let result = build_refinery(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
    );
    assert!(result.is_ok());
    assert_eq!(loom.persistent.refineries.len(), 1);
    let r = &loom.persistent.refineries[0];
    assert_eq!(r.output, Resource::ForgedLight);
    assert!(r.under_construction);
}

#[test]
fn test_build_refinery_fails_at_capacity() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // No patterns completed → max_refineries = 0.
    let result = build_refinery(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}

#[test]
fn test_build_refinery_fails_insufficient_resources() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true;
    // No stockpile resources.

    let result = build_refinery(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}

#[test]
fn test_build_refinery_fails_invalid_recipe() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true;
    *loom.persistent.stockpiles.entry(Resource::Ember).or_insert(0.0) += 50.0;

    // WovenReality + WovenReality has no recipe.
    let result = build_refinery(
        &mut loom,
        Resource::WovenReality,
        Resource::WovenReality,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}

#[test]
fn test_build_refinery_tier_gating() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true; // Only 1 pattern done.
    *loom.persistent.stockpiles.entry(Resource::ForgedLight).or_insert(0.0) += 50.0;
    *loom.persistent.stockpiles.entry(Resource::EchoGlass).or_insert(0.0) += 50.0;

    // T3 recipe requires 12 patterns. Should fail with only 1.
    let result = build_refinery(
        &mut loom,
        Resource::ForgedLight,
        Resource::EchoGlass,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::logic -- test_build_refinery 2>&1 | tail -10`
Expected: FAIL — `build_refinery` not defined

**Step 3: Write minimal implementation**

Add to `src/loom/logic.rs`:

```rust
/// Error conditions for refinery building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineryError {
    /// No matching recipe exists for the given inputs + nature.
    InvalidRecipe,
    /// The recipe's tier is not yet unlocked (need more pattern completions).
    TierLocked,
    /// Player has reached the max refinery count (= completed patterns).
    AtCapacity,
    /// Not enough stockpile resources to pay the build cost.
    InsufficientResources,
}

/// Build cost for a refinery based on its recipe tier.
/// T1: 25 of input_a. T2: 15 of input_a. T3: 10 of input_a.
fn refinery_build_cost(tier: u8) -> f64 {
    match tier {
        1 => 25.0,
        2 => 15.0,
        _ => 10.0,
    }
}

/// Pattern completion count required to unlock a recipe tier.
/// T1: 1 pattern. T2: 6 patterns. T3: 12 patterns.
fn refinery_tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 6,
        _ => 12,
    }
}

/// Attempt to build a new Refinery locked to the recipe matching (input_a, input_b, nature).
///
/// Validates:
/// 1. A recipe exists for the inputs + nature
/// 2. The recipe's tier is unlocked (enough completed patterns)
/// 3. Player hasn't reached max refinery count
/// 4. Stockpile has enough of input_a to pay the build cost
///
/// On success, creates a Refinery under construction and deducts cost.
/// Returns Ok(refinery_index) or Err(RefineryError).
pub fn build_refinery(
    loom: &mut LoomState,
    input_a: Resource,
    input_b: Resource,
    nature: NodeNature,
) -> Result<usize, RefineryError> {
    // Look up recipe.
    let recipe = crate::loom::recipes::find_recipe(input_a, input_b, nature)
        .ok_or(RefineryError::InvalidRecipe)?;

    // Check tier gating.
    let completed_patterns = loom.persistent.patterns.iter().filter(|p| p.completed).count();
    if completed_patterns < refinery_tier_unlock_threshold(recipe.tier) {
        return Err(RefineryError::TierLocked);
    }

    // Check capacity.
    if loom.persistent.refineries.len() >= loom.persistent.max_refineries() {
        return Err(RefineryError::AtCapacity);
    }

    // Check and deduct cost from stockpile.
    let cost = refinery_build_cost(recipe.tier);
    let stockpile = loom.persistent.stockpiles.entry(input_a).or_insert(0.0);
    if *stockpile < cost {
        return Err(RefineryError::InsufficientResources);
    }
    *stockpile -= cost;

    // Create refinery.
    let refinery = Refinery::new(
        recipe.input_a,
        recipe.input_b,
        recipe.node_nature,
        recipe.output,
        recipe.amount,
        recipe.tier,
    );
    let mut r = refinery;
    r.under_construction = true;
    r.construction_ticks_remaining = crate::loom::pipes::PIPE_CONSTRUCTION_TICKS; // Same 2hr timer.
    loom.persistent.refineries.push(r);
    Ok(loom.persistent.refineries.len() - 1)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::logic -- test_build_refinery 2>&1 | tail -10`
Expected: PASS — all 5 tests pass

**Step 5: Commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs
git commit -m "feat(loom): add Refinery building logic with tier gating and resource costs"
```

---

### Task 4: Add Refinery ticking (construction, processing, stall detection)

**Files:**
- Modify: `src/loom/logic.rs` (new tick functions)
- Modify: `src/core/tick_stages.rs` (wire into tick loop)

**Step 1: Write the failing tests**

```rust
#[test]
fn test_refinery_construction_completes() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.refineries.push({
        let mut r = Refinery::new(
            Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
            Resource::ForgedLight, 1.0, 1,
        );
        r.under_construction = true;
        r.construction_ticks_remaining = 1;
        r
    });

    let completed = tick_refinery_construction(&mut loom);
    assert_eq!(completed.len(), 1);
    assert!(!loom.persistent.refineries[0].under_construction);
}

#[test]
fn test_refinery_processing_produces_output() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // Add a completed refinery that turns Ember+VoidEssence → ForgedLight.
    loom.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    // Simulate deliveries: both inputs arrived this tick.
    let deliveries = vec![
        (LoomNodeRef::Refinery(0), Resource::Ember, 5.0),
        (LoomNodeRef::Refinery(0), Resource::VoidEssence, 3.0),
    ];

    let reactions = process_refinery_reactions(&mut loom, deliveries);
    assert!(!reactions.is_empty());
    // Output should be min(5.0, 3.0) * 1.0 = 3.0 ForgedLight in buffer.
    assert!((loom.persistent.refineries[0].buffer - 3.0).abs() < 0.01);
}

#[test]
fn test_refinery_stall_when_buffer_full() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    let mut r = Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    );
    r.buffer = r.buffer_capacity; // Full.
    loom.persistent.refineries.push(r);

    tick_refinery_stall_detection(&mut loom);
    assert!(loom.persistent.refineries[0].stalled);
}
```

**Step 2: Run tests to verify failure, then implement**

Add three functions to `src/loom/logic.rs`:

```rust
/// Tick construction for all refineries under construction.
/// Returns indices of refineries that completed this tick.
pub fn tick_refinery_construction(loom: &mut LoomState) -> Vec<usize> {
    let mut completed = Vec::new();
    for (i, r) in loom.persistent.refineries.iter_mut().enumerate() {
        if !r.under_construction { continue; }
        r.construction_ticks_remaining = r.construction_ticks_remaining.saturating_sub(1);
        if r.construction_ticks_remaining == 0 {
            r.under_construction = false;
            completed.push(i);
        }
    }
    completed
}

/// Process reactions at refineries from pipe deliveries.
/// Unlike Extractor reactions (which use node nature from NodeId),
/// Refineries have their recipe baked in — just check both inputs arrived.
pub fn process_refinery_reactions(
    loom: &mut LoomState,
    deliveries: Vec<(LoomNodeRef, Resource, f64)>,
) -> Vec<(usize, Resource, f64)> {
    let mut results = Vec::new();

    // Group deliveries by refinery index.
    let mut refinery_inputs: std::collections::HashMap<usize, Vec<(Resource, f64)>> =
        std::collections::HashMap::new();
    for (node_ref, resource, amount) in deliveries {
        if let LoomNodeRef::Refinery(idx) = node_ref {
            refinery_inputs.entry(idx).or_default().push((resource, amount));
        }
    }

    for (idx, inputs) in refinery_inputs {
        let Some(r) = loom.persistent.refineries.get(idx) else { continue };
        if r.under_construction { continue; }

        // Find amounts for each required input.
        let amt_a: f64 = inputs.iter().filter(|(res, _)| *res == r.input_a).map(|(_, a)| a).sum();
        let amt_b: f64 = inputs.iter().filter(|(res, _)| *res == r.input_b).map(|(_, a)| a).sum();

        if amt_a > 0.0 && amt_b > 0.0 {
            let output_amount = amt_a.min(amt_b) * r.amount;
            let cap = r.buffer_capacity;
            let r = &mut loom.persistent.refineries[idx];
            let space = (cap - r.buffer).max(0.0);
            let actual = output_amount.min(space);
            r.buffer += actual;
            r.stalled = false;
            results.push((idx, r.output, actual));
        }
    }

    results
}

/// Update stall flags for all refineries.
pub fn tick_refinery_stall_detection(loom: &mut LoomState) {
    for r in &mut loom.persistent.refineries {
        if r.under_construction { continue; }
        if r.buffer >= r.buffer_capacity {
            r.stalled = true;
        }
    }
}
```

Wire into `tick_loom()` in `src/core/tick_stages.rs`, after the existing `tick_pipe_construction` and before `tick_stall_detection`:

```rust
// Tick refinery construction.
let completed_refineries = crate::loom::tick_refinery_construction(loom);
if !completed_refineries.is_empty() {
    result.loom_changed = true;
}

// After tick_pipe_flow: process refinery reactions from deliveries.
let refinery_deliveries: Vec<_> = deliveries.iter()
    .filter(|(nr, _, _)| matches!(nr, crate::loom::LoomNodeRef::Refinery(_)))
    .cloned()
    .collect();
let _refinery_reactions = crate::loom::process_refinery_reactions(loom, refinery_deliveries);

// After tick_stall_detection:
crate::loom::tick_refinery_stall_detection(loom);
```

**Step 3: Run tests, verify pass**

Run: `cargo test --lib loom -- test_refinery 2>&1 | tail -10`
Expected: PASS

**Step 4: Commit**

```bash
git add src/loom/logic.rs src/core/tick_stages.rs src/loom/mod.rs
git commit -m "feat(loom): add Refinery ticking — construction, processing, stall detection"
```

---

### Task 5: Add Refinery demolishing

**Files:**
- Modify: `src/loom/logic.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_demolish_refinery() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    // Add a pipe pointing to this refinery.
    loom.persistent.pipes.push(Pipe {
        from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
        to: LoomNodeRef::Refinery(0),
        tier: PipeTier::T1,
        split_ratio: 1.0,
        under_construction: false,
        construction_ticks_remaining: 0,
    });

    demolish_refinery(&mut loom, 0);
    assert!(loom.persistent.refineries.is_empty());
    // Pipe should also be removed.
    assert!(loom.persistent.pipes.is_empty());
}
```

**Step 2: Implement**

```rust
/// Demolish a refinery by index.
/// Removes the refinery and all pipes connected to/from it.
/// Also re-indexes any LoomNodeRef::Refinery references in remaining pipes
/// that pointed to higher-indexed refineries.
pub fn demolish_refinery(loom: &mut LoomState, idx: usize) {
    if idx >= loom.persistent.refineries.len() {
        return;
    }

    // Remove pipes connected to this refinery.
    let ref_node = LoomNodeRef::Refinery(idx);
    loom.persistent.pipes.retain(|p| p.from != ref_node && p.to != ref_node);

    // Remove the refinery.
    loom.persistent.refineries.remove(idx);

    // Re-index pipe references for refineries above the removed index.
    for pipe in &mut loom.persistent.pipes {
        if let LoomNodeRef::Refinery(ref mut i) = pipe.from {
            if *i > idx { *i -= 1; }
        }
        if let LoomNodeRef::Refinery(ref mut i) = pipe.to {
            if *i > idx { *i -= 1; }
        }
    }
}
```

**Step 3: Run tests, verify pass, commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs
git commit -m "feat(loom): add Refinery demolishing with pipe cleanup and re-indexing"
```

---

### Task 6: Render Refineries in the Flow View processing area

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Update `render_flow_view` layout**

After the Extractor grid rendering, add a processing area section. The processing area uses the same cell buffer approach, rendered below the Extractors:

```rust
// ── Processing area: Refineries ─────────────────────────────────────
let refineries = &loom_state.persistent.refineries;
if !refineries.is_empty() {
    // Calculate scroll: if refineries exceed visible area, scroll based on selection.
    let refinery_row_start = 3 * row_stride; // Below 3 rows of extractors.
    let refinery_cols = 2; // 2 columns, same as extractors.

    for (i, refinery) in refineries.iter().enumerate() {
        let grid_row = i / refinery_cols;
        let grid_col = i % refinery_cols;
        let top = (refinery_row_start + grid_row * row_stride) as i32;
        let left_col = if grid_col == 0 {
            col_spacing as i32
        } else {
            (col_spacing + NODE_BOX_WIDTH + col_spacing) as i32
        };

        // Render refinery box (similar to extractor but with recipe info).
        let is_sel = loom_ui_selected_is_refinery(ui, i);
        render_refinery_box(&mut buffer, top, left_col, refinery, is_sel);
    }
}
```

**Step 2: Add `render_refinery_box` function**

Similar to `render_node_box` but shows:
- Title: recipe output name + tier badge
- Texture: gear/cog animation (distinct from extractors)
- Buffer bar: same as extractors
- Recipe slots: both input indicators always shown (filled/empty based on active pipes)

```rust
fn render_refinery_box(
    buffer: &mut [Vec<SceneCell>],
    top: i32,
    left: i32,
    refinery: &crate::loom::types::Refinery,
    selected: bool,
) -> i32 {
    // Same box structure as render_node_box but with refinery-specific content.
    // Title: "T1 → ForgedLight" or recipe output name.
    // Texture rows: gear animation (⚙ pattern cycling).
    // Buffer bar: same green/yellow/red coloring.
    // Recipe line: [●Emb] [●Void] > FrgLt (always shows locked recipe).
    // ... (implementation mirrors render_node_box)
    top + NODE_BOX_HEIGHT as i32
}
```

**Step 3: Update navigation to include Refineries**

In `LoomUiState`, the `selected_node` index needs to cover both Extractors (0-5) and Refineries (6+). When `selected_node >= 6`, it refers to `refineries[selected_node - 6]`.

**Step 4: Update sidebar**

When a Refinery is selected, the sidebar shows:
- Refinery identity (recipe, tier)
- Buffer + rate
- Input status (which inputs are connected via pipes)
- Controls: [D]emolish

**Step 5: Run `cargo build`, verify no errors, commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): render Refineries in Flow View processing area"
```

---

### Task 7: Add Refinery input handling (build, demolish, navigation)

**Files:**
- Modify: `src/input/loom_input.rs`

**Step 1: Extend navigation**

Down arrow past the last Extractor row enters the Refinery area. Up arrow from the first Refinery row returns to Extractors. Left/Right works within each row (2 columns).

```rust
// In FlowView, total selectable items = 6 extractors + refineries.len()
let total_nodes = 6 + loom_state.persistent.refineries.len();
```

**Step 2: Add build keybinding**

Add `B` key handling in FlowView:
- Opens a recipe selection sub-menu (or builds at current selection)
- For now: `B` when an Extractor is selected opens a list of unlocked recipes that use that Extractor's native resource
- Player picks a recipe → `build_refinery()` is called

**Step 3: Add demolish keybinding**

`D` key when a Refinery is selected calls `demolish_refinery()`.

**Step 4: Write tests for navigation bounds**

```rust
#[test]
fn test_navigation_extends_to_refineries() {
    let mut state = LoomState::new();
    select_archetype(&mut state, LoomArchetype::BurnBright);
    state.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    let mut ui = make_ui(LoomView::FlowView);
    ui.selected_node = 4; // Bottom-left extractor.

    handle_loom(key(KeyCode::Down), &mut state, &mut ui);
    assert_eq!(ui.selected_node, 6, "should enter refinery area");

    handle_loom(key(KeyCode::Up), &mut state, &mut ui);
    assert_eq!(ui.selected_node, 4, "should return to extractors");
}
```

**Step 5: Run tests, verify pass, commit**

```bash
git add src/input/loom_input.rs
git commit -m "feat(loom): add Refinery input handling — build, demolish, navigation"
```

---

### Task 8: Add debug menu actions for Refineries

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add debug actions**

Add to the Loom section of the debug menu:
- "Build T1 Refinery (Ember+Void→ForgedLight)" — instant build, skips cost/construction
- "Build T2 Refinery (ForgedLight+Reflection→EchoGlass)" — instant build
- "Clear All Refineries" — removes all refineries and their pipes

**Step 2: Implement**

```rust
Self::LoomBuildTestRefinery(tier) => {
    let (a, b, nature) = match tier {
        1 => (Resource::Ember, Resource::VoidEssence, NodeNature::Heat),
        2 => (Resource::ForgedLight, Resource::Reflection, NodeNature::Form),
        _ => (Resource::ForgedLight, Resource::EchoGlass, NodeNature::Heat),
    };
    if let Some(recipe) = crate::loom::recipes::find_recipe(a, b, nature) {
        let r = crate::loom::types::Refinery::new(
            recipe.input_a, recipe.input_b, recipe.node_nature,
            recipe.output, recipe.amount, recipe.tier,
        );
        loom.persistent.refineries.push(r);
        "Refinery built (debug)."
    } else {
        "No recipe found."
    }
}
```

**Step 3: Run `cargo build`, verify, commit**

```bash
git add src/utils/debug_menu.rs
git commit -m "feat(loom): add debug menu actions for Refineries"
```

---

### Task 9: Update Refinery pipe flow integration

**Files:**
- Modify: `src/loom/pipes.rs`

**Step 1: Update `tick_pipe_flow` to handle Refinery buffers**

The flow simulation currently only reads/writes `loom.persistent.nodes` buffers. After Task 2's `LoomNodeRef` migration, the resolve helper handles both. This task ensures:

1. Pipes from Extractors to Refineries drain Extractor buffer, add to Refinery input tracking
2. Pipes from Refineries to other nodes drain Refinery buffer (output resource)
3. Refinery output resource is determined by `refinery.output`, not `node_native_resource()`

**Step 2: Write test**

```rust
#[test]
fn test_pipe_flow_extractor_to_refinery() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // Fill Ember Spindle buffer.
    loom.persistent.nodes.iter_mut()
        .find(|n| n.id == NodeId::EmberSpindle).unwrap()
        .buffer = 10.0;
    // Add a completed refinery.
    loom.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    // Add pipe from Ember to Refinery 0.
    loom.persistent.pipes.push(Pipe {
        from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
        to: LoomNodeRef::Refinery(0),
        tier: PipeTier::T1,
        split_ratio: 1.0,
        under_construction: false,
        construction_ticks_remaining: 0,
    });

    let deliveries = tick_pipe_flow(&mut loom, 3600.0); // 1 hour.
    assert!(!deliveries.is_empty(), "should have transferred resources");
}
```

**Step 3: Implement, test, commit**

```bash
git add src/loom/pipes.rs
git commit -m "feat(loom): integrate Refinery buffers into pipe flow simulation"
```

---

### Task 10: Visual polish and full integration test

**Files:**
- Modify: `src/ui/loom_scene.rs` (refinery texture animation)
- Run: `make check`

**Step 1: Add refinery-specific texture**

Refineries use a gear/cog animation pattern distinct from Extractor textures:
```
⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙   (frame 0)
∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙   (frame 1)
```

**Step 2: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build)

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add Refinery texture animation and visual polish"
```

---

### Task 11: Update CLAUDE.md documentation

**Files:**
- Modify: `src/loom/CLAUDE.md` (if exists, otherwise skip)

Document:
- New types: `LoomNodeRef`, `Refinery`, `RefineryError`
- New functions: `build_refinery`, `demolish_refinery`, `tick_refinery_construction`, `process_refinery_reactions`, `tick_refinery_stall_detection`
- New UI: Processing area layout, Refinery node boxes, extended navigation
- Design decisions: recipe-locked, 3-tier, pattern-gated, resource-cost

**Commit:**

```bash
git add src/loom/CLAUDE.md
git commit -m "docs(loom): document Refinery system in CLAUDE.md"
```

## 2026-03-07-loom-remove-pipes-plan.md

# Loom Simplification: Remove Pipes, Direct-Pull Refineries — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove pipes from the Loom and replace with direct-pull refineries that declare their input sources, plus convert patterns from rate×time to raw amounts.

**Architecture:** Delete `pipes.rs` and all pipe types/fields. Add `sources_a`/`sources_b` vectors to `Refinery`. Replace `tick_pipe_flow()` with `tick_refinery_pull()` that calculates contention and pulls from sources. Convert `PatternRequirement` from `rate_per_hour` to `amount` with accumulator. Update UI, input, tick stages, debug menu, and persistence.

**Tech Stack:** Rust, Serde (JSON persistence), Ratatui (terminal UI)

**Design doc:** `docs/plans/2026-03-07-loom-remove-pipes-design.md`

---

## Task 1: Convert Patterns from Rate×Time to Raw Amounts

**Files:**
- Modify: `src/loom/types.rs` — `PatternRequirement` and `WovenPattern` structs
- Modify: `src/loom/discovery.rs` — `create_pattern_sequence()` and `pattern()` helper
- Modify: `src/loom/patterns.rs` — all rate-checking and sustain logic
- Test: inline `#[cfg(test)]` modules in each file

**Context:** Currently patterns require sustaining a `rate_per_hour` for `sustain_seconds`. We're converting to: produce a total `amount` of each resource. The accumulator increments based on actual production rate each tick. This is a self-contained change with no pipe dependencies.

**Step 1: Update `PatternRequirement` in `types.rs`**

Change the struct from:

```rust
pub struct PatternRequirement {
    pub resource: Resource,
    pub rate_per_hour: f64,
}
```

to:

```rust
pub struct PatternRequirement {
    pub resource: Resource,
    /// Total amount of this resource needed to complete the pattern.
    pub amount: f64,
    /// Accumulated production so far.
    #[serde(default)]
    pub accumulated: f64,
}
```

Also in `WovenPattern`, remove `sustain_seconds`, `sustained_seconds`, and `sustained_seconds_frac` fields. The completion check moves to "all requirements accumulated >= amount".

New `WovenPattern`:

```rust
pub struct WovenPattern {
    pub index: u32,
    pub name: String,
    pub requirements: Vec<PatternRequirement>,
    #[serde(default)]
    pub completed: bool,
}
```

**Step 2: Update `discovery.rs` — convert all 18 patterns**

Change the `pattern()` helper from `(index, name, reqs_with_rates, sustain_seconds)` to `(index, name, reqs_with_amounts)`.

Converted amounts (computed as `rate_per_hour * sustain_seconds / 3600`):

| # | Name | Requirements (resource, amount) |
|---|------|-------------------------------|
| 0 | First Thread | Ember 1.0 |
| 1 | The Bridge | Ember 3.0, Reflection 1.0 |
| 2 | Long Road | Ember 2.0, Memory 1.0 |
| 3 | Balancing Act | Ember 3.0, Reflection 3.0, VoidEssence 3.0 |
| 4 | Full Circle | All 6 base × 2.0 |
| 5 | The Catalyst | CondensedEmber 2.0 |
| 6 | Crossed Streams | CondensedEmber 2.0, EmberEcho 2.0 |
| 7 | The Diversion | ForgedLight 2.5, Ember 7.5 |
| 8 | Three Confluences | ForgedLight 3.0, EchoGlass 3.0, StillbornSong 3.0 |
| 9 | Pressure Test | ForgedLight 6.0, EchoGlass 6.0 |
| 10 | The Bottleneck | StillbornSong 9.0 |
| 11 | Shifting Gears | ForgedLight 6.0 |
| 12 | Harmony | All 6 base × 20.0 |
| 13 | The Triad | All 6 base × 12.0, all 3 confluence × 12.0 |
| 14 | Razor's Edge | ForgedLight 16.0, EchoGlass 16.0 |
| 15 | Resonance Cascade | Resonance 40.0 |
| 16 | The Unraveling | WovenReality 6.0 |
| 17 | Mended Loom | WovenReality 24.0, base×40.0, confluence×24.0 |

New helper signature:

```rust
fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, amount)| PatternRequirement {
                resource,
                amount,
                accumulated: 0.0,
            })
            .collect(),
        completed: false,
    }
}
```

**Step 3: Rewrite `patterns.rs` — accumulator-based completion**

Replace `active_pattern_requirements_met()`:

```rust
pub fn active_pattern_requirements_met(persistent: &LoomPersistent) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern.requirements.iter().all(|req| req.accumulated >= req.amount)
}
```

Replace `tick_pattern_sustain()` — now takes `rates: &HashMap<Resource, f64>` and accumulates:

```rust
pub fn tick_pattern_sustain(
    persistent: &mut LoomPersistent,
    rates: &HashMap<Resource, f64>,
    delta_seconds: f64,
) -> bool {
    let Some(pattern) = persistent.patterns.get_mut(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }

    let delta_hours = delta_seconds / 3600.0;

    // Accumulate production for each requirement.
    for req in &mut pattern.requirements {
        let rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        req.accumulated = (req.accumulated + rate * delta_hours).min(req.amount);
    }

    // Check if all requirements are met.
    if pattern.requirements.iter().all(|req| req.accumulated >= req.amount) {
        complete_active_pattern(persistent);
        return true;
    }

    false
}
```

Replace `active_pattern_requirement_status()`:

```rust
pub fn active_pattern_requirement_status(
    persistent: &LoomPersistent,
) -> Vec<(f64, f64)> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| (req.accumulated, req.amount))
        .collect()
}
```

**Step 4: Update tests in all three files**

- `types.rs` tests: Remove references to `sustain_seconds`, `sustained_seconds`, `sustained_seconds_frac`.
- `discovery.rs` tests: Update `test_first_pattern_sustain_is_30_minutes` → test first pattern amount is 1.0. Update `test_all_requirement_rates_are_positive` → test all amounts are positive. Remove sustain-related tests.
- `patterns.rs` tests: Rewrite all tests to use accumulator model. Key tests:
  - `test_accumulates_production_each_tick` — rate 2.0/hr, 0.1s tick → accumulated increases
  - `test_completes_when_all_accumulated` — all reqs at 100% → completion
  - `test_partial_accumulation_no_completion` — some reqs at 100%, others not → no completion
  - `test_zero_rate_no_accumulation` — 0 rate → accumulated stays at 0
  - `test_accumulated_capped_at_amount` — can't exceed target

**Step 5: Update callers**

The `tick_stages.rs` call to `tick_pattern_sustain` already passes `rates` and `delta_seconds` — the signature is compatible. But `active_pattern_requirements_met` now takes no `rates` parameter (requirements are checked internally via `accumulated`). Search for all callers of `active_pattern_requirements_met` and `active_pattern_requirement_status` and update.

Also update `render_pattern_bar()` in `loom_scene.rs` to show `37/60` format instead of time remaining.

**Step 6: Run tests and commit**

Run: `cargo test -p quest --lib -- loom`
Expected: All loom tests pass.

```bash
git add src/loom/types.rs src/loom/discovery.rs src/loom/patterns.rs src/ui/loom_scene.rs src/core/tick_stages.rs
git commit -m "feat(loom): convert patterns from rate×time to raw amounts"
```

---

## Task 2: Add Source Fields to Refinery, Remove Pipe Types

**Files:**
- Modify: `src/loom/types.rs` — `Refinery` struct, remove `Pipe`/`PipeTier`, remove `pipes` from `LoomPersistent`
- Modify: `src/loom/mod.rs` — remove pipe re-exports
- Delete: `src/loom/pipes.rs`
- Test: inline `#[cfg(test)]` in `types.rs`

**Context:** This is the core data model change. Refineries get `sources_a`/`sources_b` vectors. All pipe types are removed. The `pipes.rs` file is deleted entirely.

**Step 1: Add source fields to `Refinery` in `types.rs`**

```rust
pub struct Refinery {
    pub input_a: Resource,
    pub input_b: Resource,
    pub nature: NodeNature,
    pub output: Resource,
    pub amount: f64,
    pub tier: u8,
    /// Sources for input A — extractors or lower-tier refineries.
    #[serde(default)]
    pub sources_a: Vec<LoomNodeRef>,
    /// Sources for input B — extractors or lower-tier refineries.
    #[serde(default)]
    pub sources_b: Vec<LoomNodeRef>,
    #[serde(default)]
    pub buffer: f64,
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    #[serde(default = "default_node_level")]
    pub level: u32,
    #[serde(default)]
    pub stalled: bool,
    #[serde(default)]
    pub under_construction: bool,
    #[serde(default)]
    pub construction_ticks_remaining: u32,
}
```

Update `Refinery::new()` to accept `sources_a` and `sources_b`:

```rust
pub fn new(
    input_a: Resource, input_b: Resource, nature: NodeNature,
    output: Resource, amount: f64, tier: u8,
    sources_a: Vec<LoomNodeRef>, sources_b: Vec<LoomNodeRef>,
) -> Self {
    Self {
        input_a, input_b, nature, output, amount, tier,
        sources_a, sources_b,
        buffer: 0.0, buffer_capacity: 20.0, level: 1,
        stalled: false, under_construction: false,
        construction_ticks_remaining: 0,
    }
}
```

**Step 2: Remove `Pipe`, `PipeTier`, and `pipes` field**

Delete these from `types.rs`:
- `PipeTier` enum (lines 208-226)
- `Pipe` struct (lines 228-240)
- `pipes: Vec<Pipe>` field from `LoomPersistent` (line 286)
- `pipes: Vec::new()` from `LoomPersistent::default()` (line 320)

**Step 3: Delete `src/loom/pipes.rs`**

Remove the entire file (~600+ LOC including tests).

**Step 4: Update `src/loom/mod.rs`**

- Remove `pub mod pipes;`
- Remove entire `pub use pipes::{...}` block
- Remove `Pipe`, `PipeTier` from the `pub use types::{...}` block

**Step 5: Fix all compilation errors**

Every file that references `Pipe`, `PipeTier`, `pipes`, pipe functions, or `Refinery::new()` (which now takes sources) will fail to compile. The main files to fix:
- `src/loom/logic.rs` — `demolish_refinery` pipe cleanup, `tick_stall_detection` pipe references, `process_reactions` (pipe delivery based), `process_refinery_reactions` (pipe delivery based), `build_refinery` uses `PIPE_CONSTRUCTION_TICKS`
- `src/core/tick_stages.rs` — `tick_pipe_construction`, `tick_pipe_flow`, pipe delivery processing
- `src/input/loom_input.rs` — pipe selection, split ratio adjustment, `P` hotkey
- `src/ui/loom_scene.rs` — pipe rendering, port labels
- `src/utils/debug_menu.rs` — `LoomClearRefineries` pipe cleanup

For now, **stub out** the removed functions so the code compiles. The next tasks will implement the replacements.

**Step 6: Update all `Refinery::new()` call sites**

Search for all `Refinery::new(` calls and add empty source vectors: `vec![], vec![]`. Call sites:
- `src/loom/logic.rs` `build_refinery()`
- `src/utils/debug_menu.rs` `LoomBuildTestRefineryT1`, `LoomBuildTestRefineryT2`
- All test code creating refineries

**Step 7: Run tests and commit**

Run: `cargo test -p quest --lib -- loom`
Expected: Pipe tests gone, remaining loom tests pass (some logic tests may need updating).

```bash
git add -A
git commit -m "feat(loom): remove pipes, add source fields to Refinery"
```

---

## Task 3: Implement Direct-Pull Tick (`tick_refinery_pull`)

**Files:**
- Modify: `src/loom/logic.rs` — add `tick_refinery_pull()`, add contention calculation, add source validation
- Modify: `src/core/tick_stages.rs` — replace pipe tick calls with `tick_refinery_pull()`
- Test: inline `#[cfg(test)]` in `logic.rs`

**Context:** This is the new core simulation. Each tick, refineries pull resources directly from their sources. Contention splits source output evenly among consumers.

**Step 1: Add intake cap constant**

In `logic.rs`:

```rust
/// Max intake rate per input, by refinery tier (units/hour).
pub fn tier_intake_cap(tier: u8) -> f64 {
    match tier {
        1 => 2.0,
        2 => 3.0,
        3 => 4.0,
        _ => 2.0,
    }
}
```

**Step 2: Add source validation**

```rust
/// Check if a source is valid for a refinery's tier.
/// T1: extractors only. T2: extractors + T1. T3: extractors + T1 + T2.
pub fn valid_source_for_tier(
    source: LoomNodeRef,
    refinery_tier: u8,
    refineries: &[Refinery],
) -> bool {
    match source {
        LoomNodeRef::Extractor(_) => true, // all tiers can pull from extractors
        LoomNodeRef::Refinery(idx) => {
            if let Some(source_ref) = refineries.get(idx) {
                source_ref.tier < refinery_tier
            } else {
                false
            }
        }
    }
}
```

**Step 3: Implement `tick_refinery_pull()`**

```rust
/// Direct-pull tick: each refinery pulls from its sources, respecting contention and intake caps.
///
/// Returns a map of resource → total produced this tick (for pattern tracking).
pub fn tick_refinery_pull(
    loom: &mut LoomState,
    delta_seconds: f64,
) -> std::collections::HashMap<Resource, f64> {
    use std::collections::HashMap;
    let delta_hours = delta_seconds / 3600.0;
    let mut produced: HashMap<Resource, f64> = HashMap::new();

    // Step 1: Count consumers per source (for contention).
    let mut consumer_count: HashMap<LoomNodeRef, usize> = HashMap::new();
    for r in &loom.persistent.refineries {
        if r.under_construction {
            continue;
        }
        for src in r.sources_a.iter().chain(r.sources_b.iter()) {
            *consumer_count.entry(*src).or_insert(0) += 1;
        }
    }

    // Step 2: Calculate available output per source.
    let mut source_output: HashMap<LoomNodeRef, f64> = HashMap::new();
    for node in &loom.persistent.nodes {
        if !node.unlocked {
            continue;
        }
        let rate = node_effective_rate(loom, node);
        source_output.insert(LoomNodeRef::Extractor(node.id), rate);
    }
    // Need to avoid borrow conflict — collect node data first, then use loom reference.
    // (Node rates are already in source_output.)
    // Refinery outputs as sources: use their buffer drain rate (output rate from previous tick).
    // For simplicity, use the refinery's current buffer as the available pool.
    // Actually, the design says refineries pull from source's *output rate*, not buffer.
    // For extractors: use effective_rate. For refineries as sources: use their last-tick output rate.
    // We'll compute in two passes: first T1 (from extractors), then T2 (from extractors+T1), then T3.

    // Simplified: process by tier order to ensure lower tiers produce before higher tiers pull.
    let refinery_indices_by_tier: Vec<Vec<usize>> = {
        let mut by_tier: Vec<Vec<usize>> = vec![vec![], vec![], vec![]];
        for (i, r) in loom.persistent.refineries.iter().enumerate() {
            if !r.under_construction {
                let tier_idx = (r.tier as usize).saturating_sub(1).min(2);
                by_tier[tier_idx].push(i);
            }
        }
        by_tier
    };

    let mut refinery_output_rates: HashMap<usize, f64> = HashMap::new();

    for tier_group in &refinery_indices_by_tier {
        for &idx in tier_group {
            let r = &loom.persistent.refineries[idx];
            let cap = tier_intake_cap(r.tier);

            // Pull input A
            let mut total_pull_a = 0.0;
            for src in &r.sources_a {
                let available = match src {
                    LoomNodeRef::Extractor(nid) => {
                        source_output.get(&LoomNodeRef::Extractor(*nid)).copied().unwrap_or(0.0)
                    }
                    LoomNodeRef::Refinery(ri) => {
                        refinery_output_rates.get(ri).copied().unwrap_or(0.0)
                    }
                };
                let consumers = consumer_count.get(src).copied().unwrap_or(1).max(1);
                let share = available / consumers as f64;
                total_pull_a += share.min(cap);
            }

            // Pull input B
            let mut total_pull_b = 0.0;
            for src in &r.sources_b {
                let available = match src {
                    LoomNodeRef::Extractor(nid) => {
                        source_output.get(&LoomNodeRef::Extractor(*nid)).copied().unwrap_or(0.0)
                    }
                    LoomNodeRef::Refinery(ri) => {
                        refinery_output_rates.get(ri).copied().unwrap_or(0.0)
                    }
                };
                let consumers = consumer_count.get(src).copied().unwrap_or(1).max(1);
                let share = available / consumers as f64;
                total_pull_b += share.min(cap);
            }

            // Output = min(pull_a, pull_b) * recipe_amount
            let output_rate = total_pull_a.min(total_pull_b) * r.amount;
            refinery_output_rates.insert(idx, output_rate);

            // Add to buffer
            let actual = output_rate * delta_hours;
            let r_mut = &mut loom.persistent.refineries[idx];
            let space = (r_mut.buffer_capacity - r_mut.buffer).max(0.0);
            let deposited = actual.min(space);
            r_mut.buffer += deposited;
            if deposited > 0.0 {
                r_mut.stalled = false;
            }

            *produced.entry(r_mut.output).or_insert(0.0) += deposited;
        }
    }

    produced
}
```

**Step 4: Update `tick_stages.rs`**

In `tick_loom()`, replace the pipe tick section:

```rust
// OLD (remove):
// let completed_pipes = crate::loom::tick_pipe_construction(loom);
// ...
// let deliveries = crate::loom::tick_pipe_flow(loom, TICK_SECONDS);
// let refinery_deliveries = ...
// let _reactions = crate::loom::process_reactions(loom, deliveries);
// let _refinery_reactions = crate::loom::process_refinery_reactions(loom, refinery_deliveries);

// NEW:
let refinery_produced = crate::loom::tick_refinery_pull(loom, TICK_SECONDS);
```

Merge `refinery_produced` into the `base_produced` rates map for pattern sustain.

**Step 5: Remove `process_reactions()` and `process_refinery_reactions()`**

These functions process pipe deliveries — no longer needed. Remove from `logic.rs` and `mod.rs`.

**Step 6: Update stall detection**

`tick_stall_detection()` currently checks pipes. Simplify to: an extractor is stalled when its buffer is full (no refineries pulling from it, or all pulling refineries have full buffers). The refinery stall detection (`tick_refinery_stall_detection`) already works correctly (buffer >= capacity).

For extractors, simplify `tick_stall_detection`:

```rust
pub fn tick_stall_detection(loom: &mut LoomState) -> Vec<NodeId> {
    let mut changed = Vec::new();
    for node in &mut loom.persistent.nodes {
        if !node.unlocked {
            continue;
        }
        let should_stall = node.buffer >= node.buffer_capacity;
        if node.stalled != should_stall {
            node.stalled = should_stall;
            changed.push(node.id);
        }
    }
    changed
}
```

**Step 7: Write tests**

Key tests for `tick_refinery_pull`:
- `test_single_refinery_pulls_from_extractors` — T1 with two extractor sources, verify output rate
- `test_contention_splits_evenly` — two T1s pulling from same extractor, each gets half
- `test_tier_intake_cap_limits_pull` — extractor produces 10/hr, T1 caps at 2.0
- `test_multi_source_merge` — refinery with two sources for input A, sums their shares
- `test_t2_pulls_from_t1_output` — T2 sources include a T1 refinery
- `test_stalled_refinery_no_buffer_overflow` — buffer full = no more deposits
- `test_source_validation_t1_only_extractors` — T1 can't pull from other refineries
- `test_source_validation_t2_from_t1` — T2 can pull from T1 but not T2

**Step 8: Run tests and commit**

Run: `cargo test -p quest --lib -- loom`

```bash
git add src/loom/logic.rs src/core/tick_stages.rs
git commit -m "feat(loom): implement direct-pull tick with contention"
```

---

## Task 4: Update `build_refinery()` and `demolish_refinery()`

**Files:**
- Modify: `src/loom/logic.rs` — update build/demolish functions

**Step 1: Update `build_refinery()` to accept sources**

New signature:

```rust
pub fn build_refinery(
    loom: &mut LoomState,
    input_a: Resource,
    input_b: Resource,
    nature: NodeNature,
    sources_a: Vec<LoomNodeRef>,
    sources_b: Vec<LoomNodeRef>,
) -> Result<usize, RefineryError>
```

Add source validation: each source must be valid for the recipe tier (using `valid_source_for_tier`). Each source must actually produce the required resource. Add `RefineryError::InvalidSource` variant.

Replace `PIPE_CONSTRUCTION_TICKS` reference with a local constant:

```rust
pub const REFINERY_CONSTRUCTION_TICKS: u32 = 72_000; // 2 hours at 100ms/tick
```

**Step 2: Simplify `demolish_refinery()`**

Remove all pipe cleanup code. Also re-index source references in remaining refineries:

```rust
pub fn demolish_refinery(loom: &mut LoomState, idx: usize) {
    if idx >= loom.persistent.refineries.len() {
        return;
    }
    loom.persistent.refineries.remove(idx);

    // Re-index source references in remaining refineries.
    for r in &mut loom.persistent.refineries {
        reindex_sources(&mut r.sources_a, idx);
        reindex_sources(&mut r.sources_b, idx);
    }
}

fn reindex_sources(sources: &mut Vec<LoomNodeRef>, removed_idx: usize) {
    sources.retain(|s| !matches!(s, LoomNodeRef::Refinery(i) if *i == removed_idx));
    for s in sources.iter_mut() {
        if let LoomNodeRef::Refinery(ref mut i) = s {
            if *i > removed_idx {
                *i -= 1;
            }
        }
    }
}
```

**Step 3: Write tests and commit**

```bash
git commit -m "feat(loom): update build/demolish for direct-pull model"
```

---

## Task 5: Update Input Handling

**Files:**
- Modify: `src/input/loom_input.rs` — remove pipe input, add source editing

**Step 1: Remove pipe-related input**

- Remove `P` hotkey handler (pipe cycling)
- Remove `adjust_selected_pipe()` function
- Remove Left/Right split ratio adjustment in ListDetail view
- Remove `selected_pipe` resets (keep the field for now, clean up later)

**Step 2: Update tests**

- Remove `test_left_right_adjusts_split_ratio`
- Remove `test_left_right_no_op_when_no_pipes`
- Remove `test_p_cycles_pipe_selection`
- Remove `test_up_down_resets_selected_pipe` pipe assertions (keep navigation test)
- Remove all `Pipe` / `PipeTier` imports from test code

**Step 3: Run tests and commit**

```bash
git commit -m "feat(loom): remove pipe input handling"
```

---

## Task 6: Update Debug Menu

**Files:**
- Modify: `src/utils/debug_menu.rs` — update refinery debug actions

**Step 1: Update `LoomClearRefineries`**

Remove pipe cleanup from `LoomClearRefineries`:

```rust
// OLD:
loom.persistent.pipes.retain(|p| { ... });
loom.persistent.refineries.clear();

// NEW:
loom.persistent.refineries.clear();
```

**Step 2: Update `LoomBuildTestRefineryT1` and `LoomBuildTestRefineryT2`**

Add source vectors to `Refinery::new()` calls. Use sensible defaults — pull from the first two extractors:

```rust
// T1 test refinery: Ember+Void → ForgedLight, sources from EmberSpindle and VoidCondenser
let r = Refinery::new(
    recipe.input_a, recipe.input_b, recipe.node_nature,
    recipe.output, recipe.amount, recipe.tier,
    vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
    vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
);
```

**Step 3: Run tests and commit**

```bash
git commit -m "feat(loom): update debug menu for direct-pull model"
```

---

## Task 7: Update UI Rendering

**Files:**
- Modify: `src/ui/loom_scene.rs` — update Flow View, sidebar, pattern bar

**Step 1: Remove pipe rendering**

- Remove `render_port_labels()` function calls
- Remove pipe connection drawing in `render_flow_view()`

**Step 2: Update refinery rows in Flow View**

In the processing area below extractors, render each refinery as a compact row:

```
⠹ T1 ForgedLight    Emb←[ES] Voi←[VC]  2.0/hr  ████░░░░░░
```

Components:
- Throbber character (braille spinner, speed based on tier)
- Tier badge
- Output resource name (short form)
- Source badges: `ResourceShort←[SourceShort]` for each source
- Current output rate
- Buffer bar

**Step 3: Add throbber animation**

Add a throbber state to `LoomUiState`:

```rust
pub throbber_frame: u32, // incremented each render, used for spinner animation
```

Throbber frame rate per tier:
- T1: advance every 5 frames (500ms at 100ms render)
- T2: advance every 3 frames (300ms)
- T3: advance every 1-2 frames (150ms)

Braille chars: `['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']`

**Step 4: Update sidebar detail panel**

When a refinery is selected, show:
- Recipe info (inputs → output × amount)
- Per-source pull rates with contention info
- Buffer level
- Bottleneck diagnosis

When an extractor is selected, show:
- Consumer count and contention split

**Step 5: Update pattern bar**

Change from `██████░░░░ 12:30` (time remaining) to `██████░░░░ 37/60` (accumulated/total).

Show per-requirement progress:
```
ForgedLight: 37/60  (+2.0/hr) ✓
EchoGlass: 12/60  (+1.5/hr)
```

**Step 6: Run full test suite and commit**

Run: `cargo test -p quest --lib`
Run: `cargo clippy --all-targets -- -D warnings`

```bash
git commit -m "feat(loom): update UI for direct-pull model and amount patterns"
```

---

## Task 8: Update Persistence and CLAUDE.md

**Files:**
- Modify: `src/loom/persistence.rs` — verify backward compatibility
- Modify: `src/loom/CLAUDE.md` — update documentation
- Modify: `CLAUDE.md` — update module description if needed

**Step 1: Verify backward compatibility**

All new fields use `#[serde(default)]`, so old save files without `sources_a`/`sources_b`/`accumulated` will deserialize correctly (empty vecs, 0.0). The removed `pipes` field also has `#[serde(default)]`, so old saves with `pipes` data will simply ignore it (serde's `deny_unknown_fields` is not enabled).

Old `PatternRequirement` saves with `rate_per_hour` field: since we're renaming to `amount`, we need `#[serde(alias = "rate_per_hour")]` on the `amount` field for backward compatibility:

```rust
pub struct PatternRequirement {
    pub resource: Resource,
    #[serde(alias = "rate_per_hour")]
    pub amount: f64,
    #[serde(default)]
    pub accumulated: f64,
}
```

Similarly, old `WovenPattern` saves have `sustain_seconds`, `sustained_seconds`, `sustained_seconds_frac` — these will be ignored since serde skips unknown fields by default.

**Step 2: Update `src/loom/CLAUDE.md`**

Remove all pipe documentation. Update:
- Module Structure (remove `pipes.rs`)
- Key Types (remove `Pipe`, `PipeTier`, update `Refinery` description)
- Node Addressing section (update to mention refinery sources instead of pipe endpoints)
- Production Chain Flow (remove pipe step)
- Refinery System (add source fields, contention, intake caps)
- Add "Direct-Pull System" section explaining contention model
- Update Input section (remove `P` hotkey)
- Update Debug Menu section
- Update Integration Points (remove pipe tick references)

**Step 3: Run `make check` and commit**

```bash
make check
git commit -m "docs(loom): update CLAUDE.md for pipe removal"
```

---

## Task 9: Full CI Check and Cleanup

**Files:**
- All modified files

**Step 1: Run full CI checks**

```bash
make check
```

This runs: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --all-targets`, `cargo audit --deny yanked`.

**Step 2: Fix any issues**

- Remove any remaining dead code warnings from pipe-related imports
- Clean up any `#[allow(dead_code)]` that are no longer needed
- Remove `selected_pipe` from `LoomUiState` if no longer used anywhere

**Step 3: Final commit**

```bash
git commit -m "chore(loom): cleanup dead code after pipe removal"
```

## 2026-03-07-loom-sustained-patterns-plan.md

# Loom of Worlds — Sustained Rate Pattern Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the accumulated-totals pattern system with sustained flow rate tracking, expand from 18 to 28 patterns, fix extractor buffer overflow, and shift tier gates.

**Architecture:** Add a `RateTracker` struct for 60-second rolling window rate measurement. Redesign `PatternRequirement` to use rate thresholds and sustain durations instead of accumulated amounts. Each requirement completes independently when its sustain timer reaches the required duration. Fix extractors to auto-drain excess production instead of stalling.

**Tech Stack:** Rust, serde (JSON persistence), Ratatui (UI)

---

## Summary of Changes

| Area | What changes |
|------|-------------|
| `types.rs` | New `RateTracker` struct, redesigned `PatternRequirement` with rate/duration fields |
| `patterns.rs` | Rewrite `tick_pattern_sustain()` for rate-based sustain logic with pause model |
| `discovery.rs` | Replace 18 patterns with 28, using rate thresholds and durations from design doc |
| `logic.rs` | Fix extractor buffer overflow (auto-drain), shift tier gate thresholds |
| `tick_stages.rs` | Pass per-resource production amounts to rate tracker each tick |
| `loom_scene.rs` | Update pattern bar to show rate/duration/state instead of accumulated/amount |
| `mod.rs` | Re-export new types |
| `persistence.rs` | No structural changes (serde handles new fields via defaults) |
| `CLAUDE.md` | Update Loom module documentation |

---

### Task 1: Add RateTracker struct to types.rs

**Files:**
- Modify: `src/loom/types.rs`

**Context:** The `RateTracker` measures a 60-second rolling window average of production for a single resource. It uses a circular buffer of 600 ticks (at 100ms/tick = 60 seconds). The running sum gives O(1) per-tick updates. This struct is NOT serialized — on load, it starts empty and ramps up over 60 seconds (negligible).

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `src/loom/types.rs`:

```rust
#[test]
fn test_rate_tracker_new_is_empty() {
    let tracker = RateTracker::new();
    assert!((tracker.rate_per_hour()).abs() < 1e-9);
}

#[test]
fn test_rate_tracker_push_single_value() {
    let mut tracker = RateTracker::new();
    // Push a single tick's production: 1.0 units produced in 0.1s
    tracker.push(1.0);
    // Window has 1 sample out of 600, so average per tick = 1.0/600
    // Rate per hour = (sum / window_size) * ticks_per_hour
    // = (1.0 / 600) * 36000 = 60.0/hr
    let rate = tracker.rate_per_hour();
    assert!((rate - 60.0).abs() < 1e-6, "rate was {}", rate);
}

#[test]
fn test_rate_tracker_full_window_steady() {
    let mut tracker = RateTracker::new();
    // Simulate 600 ticks at 50/hr = 50/36000 per tick ≈ 0.001389 per tick
    let per_tick = 50.0 / 36000.0;
    for _ in 0..600 {
        tracker.push(per_tick);
    }
    let rate = tracker.rate_per_hour();
    assert!((rate - 50.0).abs() < 0.1, "rate was {}", rate);
}

#[test]
fn test_rate_tracker_evicts_old_values() {
    let mut tracker = RateTracker::new();
    // Fill 600 ticks with high production
    for _ in 0..600 {
        tracker.push(1.0);
    }
    // Now push 600 ticks of zero — should evict all old values
    for _ in 0..600 {
        tracker.push(0.0);
    }
    assert!((tracker.rate_per_hour()).abs() < 1e-9);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests::test_rate_tracker -- --nocapture`
Expected: FAIL — `RateTracker` type does not exist yet.

**Step 3: Write minimal implementation**

Add above the `#[cfg(test)]` block in `src/loom/types.rs`:

```rust
/// 60-second rolling window rate tracker.
///
/// Measures production rate using a circular buffer of the last 600 ticks
/// (at 100ms/tick = 60 seconds). Maintains a running sum for O(1) updates.
///
/// Not serialized — reconstructed from scratch on load (60s ramp-up is negligible).
#[derive(Debug, Clone)]
pub struct RateTracker {
    buffer: std::collections::VecDeque<f64>,
    sum: f64,
}

const RATE_WINDOW_SIZE: usize = 600; // 600 ticks × 0.1s = 60 seconds
const TICKS_PER_HOUR: f64 = 36_000.0; // 3600s / 0.1s

impl RateTracker {
    pub fn new() -> Self {
        Self {
            buffer: std::collections::VecDeque::with_capacity(RATE_WINDOW_SIZE),
            sum: 0.0,
        }
    }

    /// Push one tick's production amount into the window.
    pub fn push(&mut self, amount: f64) {
        if self.buffer.len() >= RATE_WINDOW_SIZE {
            self.sum -= self.buffer.pop_front().unwrap_or(0.0);
        }
        self.buffer.push_back(amount);
        self.sum += amount;
    }

    /// Current production rate in units/hour, averaged over the 60-second window.
    pub fn rate_per_hour(&self) -> f64 {
        // Average production per tick × ticks per hour
        (self.sum / RATE_WINDOW_SIZE as f64) * TICKS_PER_HOUR
    }
}

impl Default for RateTracker {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::types::tests::test_rate_tracker -- --nocapture`
Expected: PASS — all 4 tests pass.

**Step 5: Commit**

```bash
git add src/loom/types.rs
git commit -m "feat(loom): add RateTracker for 60-second rolling window rate measurement"
```

---

### Task 2: Redesign PatternRequirement for sustained rates

**Files:**
- Modify: `src/loom/types.rs`

**Context:** Currently `PatternRequirement` has `amount` (total needed) and `accumulated` (progress so far). We need to replace this with rate-based fields: `required_rate` (units/hr threshold), `sustain_duration_secs` (total seconds the rate must be sustained), and `sustained_secs` (seconds sustained so far). The `accumulated` and `amount` fields are removed. The `RateTracker` is transient (not serialized). Each requirement also gets a `completed` flag so requirements can complete independently.

**Important serde migration note:** Old saves have `amount` and `accumulated` fields. Use `#[serde(default)]` on all new fields so old saves load without crashing. The old `amount` field should be kept with `#[serde(default)]` so old JSON doesn't fail deserialization — it just won't be used.

**Step 1: Write the failing test**

Add to `src/loom/types.rs` tests:

```rust
#[test]
fn test_pattern_requirement_rate_fields() {
    let req = PatternRequirement {
        resource: Resource::Ember,
        required_rate: 25.0,
        sustain_duration_secs: 7200.0,
        sustained_secs: 0.0,
        completed: false,
        // Legacy fields for serde compat
        amount: 0.0,
        accumulated: 0.0,
    };
    assert_eq!(req.resource, Resource::Ember);
    assert!((req.required_rate - 25.0).abs() < 1e-9);
    assert!((req.sustain_duration_secs - 7200.0).abs() < 1e-9);
    assert!((req.sustained_secs).abs() < 1e-9);
    assert!(!req.completed);
}

#[test]
fn test_pattern_requirement_completes_independently() {
    let mut req = PatternRequirement {
        resource: Resource::Ember,
        required_rate: 25.0,
        sustain_duration_secs: 100.0,
        sustained_secs: 100.0,
        completed: true,
        amount: 0.0,
        accumulated: 0.0,
    };
    assert!(req.completed);
    assert!(req.sustained_secs >= req.sustain_duration_secs);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests::test_pattern_requirement_rate -- --nocapture`
Expected: FAIL — fields `required_rate`, `sustain_duration_secs`, `sustained_secs`, `completed` don't exist.

**Step 3: Write minimal implementation**

Replace the `PatternRequirement` struct in `src/loom/types.rs`:

```rust
/// A single requirement within a woven pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequirement {
    pub resource: Resource,
    /// Minimum production rate (units/hr) that must be sustained.
    #[serde(default)]
    pub required_rate: f64,
    /// Total seconds the rate must be sustained to complete this requirement.
    #[serde(default)]
    pub sustain_duration_secs: f64,
    /// Seconds sustained so far (timer advances when rate >= threshold, pauses otherwise).
    #[serde(default)]
    pub sustained_secs: f64,
    /// Whether this individual requirement is complete (locks when sustain timer finishes).
    #[serde(default)]
    pub completed: bool,
    /// Legacy field — total amount needed (accumulated totals system). Kept for serde compat.
    #[serde(default, alias = "rate_per_hour")]
    pub amount: f64,
    /// Legacy field — accumulated production so far. Kept for serde compat.
    #[serde(default)]
    pub accumulated: f64,
}
```

**Step 4: Fix all compilation errors**

After changing `PatternRequirement`, code in `patterns.rs`, `discovery.rs`, `loom_scene.rs`, and tests will fail to compile. Fix each by adding the new fields with defaults. This is expected — subsequent tasks will properly update the logic. For now, just make it compile:

- In `discovery.rs` `pattern()` helper: add `required_rate: 0.0, sustain_duration_secs: 0.0, sustained_secs: 0.0, completed: false` to `PatternRequirement` construction (the old 18 patterns won't use these yet — Task 5 replaces them entirely).
- In test files: add the new fields where `PatternRequirement` is constructed directly.

**Step 5: Run all loom tests to verify they pass**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS — all existing tests should still pass with the added fields.

**Step 6: Commit**

```bash
git add src/loom/types.rs src/loom/discovery.rs src/loom/patterns.rs
git commit -m "feat(loom): add rate-based fields to PatternRequirement"
```

---

### Task 3: Rewrite tick_pattern_sustain() for sustained rate logic

**Files:**
- Modify: `src/loom/patterns.rs`

**Context:** The current `tick_pattern_sustain()` accumulates `rate * delta_hours` per tick. The new version should:
1. For each non-completed requirement in the active pattern:
   - Look up the current measured rate for that resource (from a `HashMap<Resource, f64>` of per-resource rates).
   - If rate >= `required_rate`: advance `sustained_secs` by `delta_seconds`.
   - If rate < `required_rate`: do nothing (simple pause — no decay).
   - If `sustained_secs >= sustain_duration_secs`: mark requirement `completed = true`.
2. If ALL requirements are `completed`: mark the pattern as completed and advance to the next one.

The function signature changes: the `rates` parameter now contains **measured rates** (from RateTracker), not raw per-tick production amounts.

**Step 1: Write the failing tests**

Replace the accumulator tests in `src/loom/patterns.rs` with rate-based tests. Keep the existing test helper functions. Add new tests:

```rust
#[test]
fn test_sustain_advances_when_rate_meets_threshold() {
    let mut state = state_with_patterns();
    // Pattern 0 requires Ember 25/hr for some duration.
    // Provide rate of 30/hr (above threshold).
    let r = rates(&[(Resource::Ember, 30.0)]);
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    let req = &state.persistent.patterns[0].requirements[0];
    assert!(req.sustained_secs > 0.0, "should advance when rate >= threshold");
    assert!((req.sustained_secs - 1.0).abs() < 1e-9, "should advance by delta_seconds");
}

#[test]
fn test_sustain_pauses_when_rate_below_threshold() {
    let mut state = state_with_patterns();
    // Pre-set some sustained progress.
    state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
    // Provide rate below threshold.
    let r = rates(&[(Resource::Ember, 10.0)]); // below 25/hr
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    let req = &state.persistent.patterns[0].requirements[0];
    assert!((req.sustained_secs - 100.0).abs() < 1e-9, "should not advance when rate < threshold");
}

#[test]
fn test_sustain_never_decays() {
    let mut state = state_with_patterns();
    state.persistent.patterns[0].requirements[0].sustained_secs = 50.0;
    let r = rates(&[]); // zero rate
    tick_pattern_sustain(&mut state.persistent, &r, 10.0);
    let req = &state.persistent.patterns[0].requirements[0];
    assert!(req.sustained_secs >= 50.0, "sustained_secs must never decrease");
}

#[test]
fn test_requirement_completes_when_duration_reached() {
    let mut state = state_with_patterns();
    let req = &mut state.persistent.patterns[0].requirements[0];
    req.sustained_secs = req.sustain_duration_secs - 0.5;
    let r = rates(&[(Resource::Ember, 100.0)]); // well above threshold
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    assert!(state.persistent.patterns[0].requirements[0].completed);
}

#[test]
fn test_pattern_completes_when_all_requirements_complete() {
    let mut state = state_with_patterns();
    // Set all requirements to just below completion.
    for req in &mut state.persistent.patterns[0].requirements {
        req.sustained_secs = req.sustain_duration_secs - 0.1;
    }
    // Provide high rates for all required resources.
    let r = rates(&[(Resource::Ember, 100.0)]);
    let completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    assert!(completed);
    assert!(state.persistent.patterns[0].completed);
}

#[test]
fn test_requirement_independent_completion() {
    let mut state = state_with_patterns();
    // Skip to a multi-requirement pattern (pattern index 4 = "Mirror and Void" in new set,
    // but for now use whatever multi-req pattern exists).
    // We'll test this properly after Task 5 replaces patterns.
    // For now, just verify the single-req pattern 0 works.
    let req = &mut state.persistent.patterns[0].requirements[0];
    req.sustained_secs = req.sustain_duration_secs;
    req.completed = true;
    assert!(active_pattern_requirements_met(&state.persistent));
}

#[test]
fn test_already_completed_requirement_not_advanced() {
    let mut state = state_with_patterns();
    state.persistent.patterns[0].requirements[0].completed = true;
    state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
    let r = rates(&[(Resource::Ember, 100.0)]);
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    // Should not advance past where it was.
    assert!((state.persistent.patterns[0].requirements[0].sustained_secs - 100.0).abs() < 1e-9);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::patterns::tests -- --nocapture`
Expected: FAIL — the current implementation accumulates `rate * delta_hours` instead of advancing a sustain timer.

**Step 3: Rewrite tick_pattern_sustain()**

Replace the function body in `src/loom/patterns.rs`:

```rust
/// Tick the sustain timer for the active pattern.
///
/// Called once per game tick. `delta_seconds` is wall-clock time elapsed
/// since the last tick (typically 0.1s for a 100ms tick interval).
/// `rates` maps each resource to its current measured production rate in units/hour.
///
/// For each non-completed requirement:
/// - If the measured rate >= required_rate: advance sustained_secs by delta_seconds.
/// - Otherwise: do nothing (simple pause — no decay).
/// - If sustained_secs >= sustain_duration_secs: mark requirement completed.
///
/// When ALL requirements are completed, the pattern completes.
///
/// Returns `true` if a pattern was completed during this tick.
pub fn tick_pattern_sustain(
    persistent: &mut LoomPersistent,
    rates: &HashMap<Resource, f64>,
    delta_seconds: f64,
) -> bool {
    let Some(pattern) = persistent.patterns.get_mut(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }

    for req in &mut pattern.requirements {
        if req.completed {
            continue;
        }
        let rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        if rate >= req.required_rate {
            req.sustained_secs += delta_seconds;
            if req.sustained_secs >= req.sustain_duration_secs {
                req.sustained_secs = req.sustain_duration_secs;
                req.completed = true;
            }
        }
        // Simple pause: do nothing when rate < threshold. No decay.
    }

    if pattern.requirements.iter().all(|req| req.completed) {
        complete_active_pattern(persistent);
        return true;
    }
    false
}
```

Also update `active_pattern_requirements_met()`:

```rust
pub fn active_pattern_requirements_met(persistent: &LoomPersistent) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern.requirements.iter().all(|req| req.completed)
}
```

And update `active_pattern_requirement_status()` to return the new fields:

```rust
/// Returns `(sustained_secs, sustain_duration_secs, completed)` for each requirement.
pub fn active_pattern_requirement_status(
    persistent: &LoomPersistent,
) -> Vec<(f64, f64, bool)> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| (req.sustained_secs, req.sustain_duration_secs, req.completed))
        .collect()
}
```

**Step 4: Update all existing tests that reference the old behavior**

Remove or rewrite tests that check `accumulated` or `amount` directly. The tests from Step 1 replace them. Keep structural tests like `test_advance_skips_already_completed_patterns` and `test_all_complete_when_every_pattern_marked`.

Update `active_pattern_requirement_status` tests to match the new return type `Vec<(f64, f64, bool)>`.

**Step 5: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/patterns.rs
git commit -m "feat(loom): rewrite pattern sustain logic for rate-based tracking"
```

---

### Task 4: Fix extractor buffer overflow (auto-drain)

**Files:**
- Modify: `src/loom/logic.rs`

**Context:** Currently in `tick_base_production()`, when a node's buffer reaches capacity, the node stalls (`node.stalled = true; continue;`). This silently breaks sustained rate patterns because the extractor stops producing. The fix: always produce at full rate. If the buffer is full, discard the excess (auto-drain). The extractor never stalls due to a full buffer — the buffer is a reservoir, not a gate.

**Step 1: Write the failing test**

Add to `src/loom/logic.rs` tests (find the existing `mod tests` block):

```rust
#[test]
fn test_extractor_produces_at_full_rate_when_buffer_full() {
    let mut loom = LoomState::new();
    // Unlock the Ember Spindle.
    loom.persistent.nodes[0].unlocked = true;
    loom.persistent.nodes[0].buffer = loom.persistent.nodes[0].buffer_capacity; // full buffer

    let produced = tick_base_production(&mut loom, 0.1);

    // Should still report production (for rate tracking) even when buffer is full.
    let ember_produced = produced.get(&Resource::Ember).copied().unwrap_or(0.0);
    assert!(ember_produced > 0.0, "extractor should report production even with full buffer");
    // Node should NOT be marked stalled.
    assert!(!loom.persistent.nodes[0].stalled, "extractor should not stall from full buffer");
}

#[test]
fn test_extractor_buffer_does_not_exceed_capacity() {
    let mut loom = LoomState::new();
    loom.persistent.nodes[0].unlocked = true;
    loom.persistent.nodes[0].buffer = loom.persistent.nodes[0].buffer_capacity - 0.001;

    tick_base_production(&mut loom, 0.1);

    assert!(
        loom.persistent.nodes[0].buffer <= loom.persistent.nodes[0].buffer_capacity + 1e-9,
        "buffer should not exceed capacity"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::logic::tests::test_extractor_produces_at_full_rate -- --nocapture`
Expected: FAIL — currently the node stalls and produces 0 when buffer is full.

**Step 3: Fix tick_base_production()**

In `src/loom/logic.rs`, change the `tick_base_production()` function. Replace the stall check block:

**Old code (around line 260-265):**
```rust
// If buffer is at capacity, node stalls — no production.
if node.buffer >= capacity {
    node.stalled = true;
    continue;
}
```

**New code:**
```rust
// Always produce at full rate for rate tracking.
// Buffer caps at capacity — excess is auto-drained (discarded).
// The extractor never stalls from a full buffer.
```

And change the production calculation to always report the full amount but only add to buffer what fits:

```rust
let amount = rate * delta_hours;
let new_buffer = (node.buffer + amount).min(capacity);
node.buffer = new_buffer;
node.stalled = false;

// Report full production amount for rate tracking, not just what fit in buffer.
if amount > 0.0 {
    let resource = node_native_resource(node_id);
    *produced.entry(resource).or_insert(0.0) += amount;
}
```

**Step 4: Update existing tests that check stall behavior**

Any test that expects `stalled = true` when buffer is full needs to be updated. The extractor no longer stalls from buffer fullness.

**Step 5: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/logic.rs
git commit -m "fix(loom): auto-drain extractor buffers instead of stalling on full"
```

---

### Task 5: Replace 18 patterns with 28 in discovery.rs

**Files:**
- Modify: `src/loom/discovery.rs`

**Context:** Replace the entire `create_pattern_sequence()` function with the 28 patterns from the design doc. Each pattern now uses `required_rate` (units/hr) and `sustain_duration_secs` instead of `amount`. The `pattern()` helper needs updating.

**Step 1: Update the pattern() helper function**

```rust
fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, rate, duration_hours)| PatternRequirement {
                resource,
                required_rate: rate,
                sustain_duration_secs: duration_hours * 3600.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            })
            .collect(),
        completed: false,
    }
}
```

Note: The third element in each tuple is duration **in hours** (converted to seconds internally).

**Step 2: Replace create_pattern_sequence() with all 28 patterns**

```rust
fn create_pattern_sequence() -> Vec<WovenPattern> {
    vec![
        // ── Teaching Arc (1-8) ── ~3 days (72 hours) ──
        pattern(0, "First Thread", vec![
            (Resource::Ember, 25.0, 2.0),
        ]),
        pattern(1, "Still Waters", vec![
            (Resource::Silence, 25.0, 2.0),
        ]),
        pattern(2, "Echoing Halls", vec![
            (Resource::Memory, 25.0, 4.0),
        ]),
        pattern(3, "Harmonic Pulse", vec![
            (Resource::Resonance, 25.0, 4.0),
        ]),
        pattern(4, "Mirror and Void", vec![
            (Resource::Reflection, 30.0, 6.0),
            (Resource::VoidEssence, 30.0, 6.0),
        ]),
        pattern(5, "Full Circle", vec![
            (Resource::Ember, 20.0, 10.0),
            (Resource::Reflection, 20.0, 10.0),
            (Resource::VoidEssence, 20.0, 10.0),
            (Resource::Memory, 20.0, 10.0),
            (Resource::Silence, 20.0, 10.0),
            (Resource::Resonance, 20.0, 10.0),
        ]),
        pattern(6, "The Catalyst", vec![
            (Resource::CondensedEmber, 8.0, 16.0),
        ]),
        pattern(7, "Echo of Flame", vec![
            (Resource::EmberEcho, 8.0, 28.0),
        ]),
        // ── Mastery Arc (9-16) ── ~10 days (236 hours) ──
        pattern(8, "Forged in Fire", vec![
            (Resource::ForgedLight, 15.0, 16.0),
        ]),
        pattern(9, "Glass Resonance", vec![
            (Resource::EchoGlass, 15.0, 16.0),
        ]),
        pattern(10, "The Unsung", vec![
            (Resource::StillbornSong, 15.0, 24.0),
        ]),
        pattern(11, "Void Distillation", vec![
            (Resource::PurifiedVoid, 10.0, 24.0),
        ]),
        pattern(12, "Crossed Streams", vec![
            (Resource::ForgedLight, 12.0, 24.0),
            (Resource::EchoGlass, 12.0, 24.0),
        ]),
        pattern(13, "The Asymmetry", vec![
            (Resource::ForgedLight, 25.0, 36.0),
            (Resource::StillbornSong, 8.0, 36.0),
        ]),
        pattern(14, "Pressure Test", vec![
            (Resource::CondensedEmber, 15.0, 36.0),
            (Resource::EmberEcho, 10.0, 36.0),
            (Resource::PurifiedVoid, 10.0, 36.0),
        ]),
        pattern(15, "Three Confluences", vec![
            (Resource::ForgedLight, 18.0, 60.0),
            (Resource::EchoGlass, 18.0, 60.0),
            (Resource::StillbornSong, 18.0, 60.0),
        ]),
        // ── Endgame Arc (17-28) ── ~22 days (534 hours) ──
        pattern(16, "The Amplifier", vec![
            (Resource::ForgedLight, 35.0, 18.0),
        ]),
        pattern(17, "Purified Cascade", vec![
            (Resource::PurifiedVoid, 20.0, 24.0),
            (Resource::ForgedLight, 20.0, 24.0),
        ]),
        pattern(18, "Resonance Cascade", vec![
            (Resource::Resonance, 150.0, 24.0),
            (Resource::StillbornSong, 25.0, 24.0),
        ]),
        pattern(19, "First Weave", vec![
            (Resource::WovenReality, 5.0, 30.0),
        ]),
        pattern(20, "The Unraveling", vec![
            (Resource::WovenReality, 15.0, 36.0),
            (Resource::PurifiedVoid, 15.0, 36.0),
        ]),
        pattern(21, "Grand Harmony", vec![
            (Resource::Ember, 100.0, 36.0),
            (Resource::Reflection, 100.0, 36.0),
            (Resource::VoidEssence, 100.0, 36.0),
            (Resource::Memory, 100.0, 36.0),
            (Resource::Silence, 100.0, 36.0),
            (Resource::Resonance, 100.0, 36.0),
            (Resource::ForgedLight, 30.0, 36.0),
            (Resource::EchoGlass, 30.0, 36.0),
            (Resource::StillbornSong, 30.0, 36.0),
        ]),
        pattern(22, "The Knot", vec![
            (Resource::ForgedLight, 25.0, 36.0),
            (Resource::PurifiedVoid, 15.0, 36.0),
            (Resource::CondensedEmber, 12.0, 36.0),
        ]),
        pattern(23, "Strange Alchemy", vec![
            (Resource::ForgedLight, 30.0, 42.0),
            (Resource::EchoGlass, 30.0, 42.0),
            (Resource::StillbornSong, 30.0, 42.0),
            (Resource::Ember, 80.0, 42.0),
            (Resource::VoidEssence, 80.0, 42.0),
        ]),
        pattern(24, "Refined Purpose", vec![
            (Resource::PurifiedVoid, 30.0, 48.0),
            (Resource::ForgedLight, 25.0, 48.0),
        ]),
        pattern(25, "The Flood", vec![
            (Resource::WovenReality, 35.0, 48.0),
        ]),
        pattern(26, "Everything Flows", vec![
            (Resource::Ember, 50.0, 72.0),
            (Resource::Reflection, 50.0, 72.0),
            (Resource::VoidEssence, 50.0, 72.0),
            (Resource::Memory, 50.0, 72.0),
            (Resource::Silence, 50.0, 72.0),
            (Resource::Resonance, 50.0, 72.0),
            (Resource::ForgedLight, 20.0, 72.0),
            (Resource::EchoGlass, 20.0, 72.0),
            (Resource::StillbornSong, 20.0, 72.0),
            (Resource::CondensedEmber, 10.0, 72.0),
            (Resource::EmberEcho, 10.0, 72.0),
            (Resource::PurifiedVoid, 10.0, 72.0),
            (Resource::WovenReality, 5.0, 72.0),
        ]),
        pattern(27, "Mended Loom", vec![
            (Resource::WovenReality, 20.0, 120.0),
            (Resource::ForgedLight, 40.0, 120.0),
            (Resource::EchoGlass, 40.0, 120.0),
            (Resource::StillbornSong, 40.0, 120.0),
            (Resource::Ember, 80.0, 120.0),
            (Resource::Silence, 80.0, 120.0),
            (Resource::Resonance, 80.0, 120.0),
        ]),
    ]
}
```

**Step 3: Update all discovery tests**

Update tests that reference `18` patterns to use `28`. Update tests that check specific pattern amounts to check `required_rate` and `sustain_duration_secs` instead:

```rust
#[test]
fn test_loom_discovery() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    assert!(loom.persistent.discovered);
    assert_eq!(loom.persistent.patterns.len(), 28);
}

#[test]
fn test_first_pattern_requires_ember_at_25_per_hour() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    let first = &loom.persistent.patterns[0];
    assert_eq!(first.requirements.len(), 1);
    assert_eq!(first.requirements[0].resource, Resource::Ember);
    assert!((first.requirements[0].required_rate - 25.0).abs() < 1e-9);
    assert!((first.requirements[0].sustain_duration_secs - 7200.0).abs() < 1e-9); // 2 hours
}

// Update the count check
#[test]
fn test_discovery_does_not_re_discover() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    assert_eq!(loom.persistent.patterns.len(), 28);
    loom.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
    complete_discovery(&mut loom);
    assert_eq!(loom.persistent.patterns.len(), 28);
    assert!((loom.persistent.patterns[0].requirements[0].sustained_secs - 100.0).abs() < 1e-9);
}
```

Also update `test_final_pattern_has_largest_total_amount` to check `sustain_duration_secs` instead of `amount`:

```rust
#[test]
fn test_final_pattern_has_longest_duration() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    let last = loom.persistent.patterns.last().unwrap();
    let last_duration = last.requirements[0].sustain_duration_secs;
    // Mended Loom = 120 hours = 432000 seconds — longest in the set.
    assert!((last_duration - 432_000.0).abs() < 1e-9);
}
```

**Step 4: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/discovery.rs
git commit -m "feat(loom): replace 18 accumulated patterns with 28 sustained rate patterns"
```

---

### Task 6: Shift tier gate thresholds

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/ui/loom_scene.rs`

**Context:** The design shifts tier gates from T1@1, T2@6, T3@12 to T1@1, T2@8, T3@15. Two places need updating:
1. `refinery_tier_unlock_threshold()` in `logic.rs` (gates building refineries)
2. `visible_recipe_tier()` in `loom_scene.rs` (gates recipe visibility in UI)

**Step 1: Write the failing test**

Add to `logic.rs` tests:

```rust
#[test]
fn test_tier_gates_shifted() {
    let mut loom = LoomState::new();
    crate::loom::complete_discovery(&mut loom);
    // 0 complete → no tiers
    assert!(unlocked_tiers(&loom).is_empty());
    // 1 complete → T1 only
    loom.persistent.patterns[0].completed = true;
    assert_eq!(unlocked_tiers(&loom), vec![1]);
    // 7 complete → still T1 only (threshold is 8 for T2)
    for i in 1..7 {
        loom.persistent.patterns[i].completed = true;
    }
    assert_eq!(unlocked_tiers(&loom), vec![1]);
    // 8 complete → T1 + T2
    loom.persistent.patterns[7].completed = true;
    assert_eq!(unlocked_tiers(&loom), vec![1, 2]);
    // 14 complete → still T1 + T2 (threshold is 15 for T3)
    for i in 8..14 {
        loom.persistent.patterns[i].completed = true;
    }
    assert_eq!(unlocked_tiers(&loom), vec![1, 2]);
    // 15 complete → T1 + T2 + T3
    loom.persistent.patterns[14].completed = true;
    assert_eq!(unlocked_tiers(&loom), vec![1, 2, 3]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::logic::tests::test_tier_gates_shifted -- --nocapture`
Expected: FAIL — current thresholds are 1/6/12, not 1/8/15.

**Step 3: Update the threshold function**

In `src/loom/logic.rs`, change `refinery_tier_unlock_threshold()`:

```rust
fn refinery_tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 8,
        _ => 15,
    }
}
```

In `src/ui/loom_scene.rs`, change `visible_recipe_tier()`:

```rust
fn visible_recipe_tier(completed_patterns: usize) -> u8 {
    if completed_patterns >= 15 {
        3
    } else if completed_patterns >= 8 {
        2
    } else {
        1
    }
}
```

**Step 4: Update any existing tests that relied on old thresholds**

Search for tests that use `6` or `12` as tier gate values and update them to `8` and `15`.

**Step 5: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/logic.rs src/ui/loom_scene.rs
git commit -m "feat(loom): shift tier gates to T2@8 and T3@15 patterns"
```

---

### Task 7: Wire RateTracker into tick_stages.rs

**Files:**
- Modify: `src/core/tick_stages.rs`
- Modify: `src/loom/types.rs` (add rate trackers to LoomState)

**Context:** Currently `tick_loom()` passes per-node effective rates to `tick_pattern_sustain()`. The new system needs measured rates from a `RateTracker` per resource. The trackers live on `LoomState` (transient, not serialized). Each tick, we push the per-tick production amount for each resource into its tracker, then read the measured rate from each tracker to pass to `tick_pattern_sustain()`.

**Step 1: Add rate trackers to LoomState**

In `src/loom/types.rs`, add a field to `LoomState`:

```rust
pub struct LoomState {
    pub persistent: LoomPersistent,
    /// Per-resource rolling rate trackers (transient, not serialized).
    #[serde(skip)]
    pub rate_trackers: HashMap<Resource, RateTracker>,
}
```

Update `LoomState::new()` and `Default`:

```rust
impl LoomState {
    pub fn new() -> Self {
        Self {
            persistent: LoomPersistent::default(),
            rate_trackers: HashMap::new(),
        }
    }
}
```

**Step 2: Update tick_loom() to use rate trackers**

In `src/core/tick_stages.rs`, modify the `tick_loom()` function. Replace the `rates` computation (lines ~1042-1058) with:

```rust
// Push per-tick production amounts into rate trackers.
for (resource, amount) in &produced {
    loom.rate_trackers
        .entry(*resource)
        .or_insert_with(RateTracker::new)
        .push(*amount);
}
// Also push 0.0 for resources that weren't produced this tick
// (so their rate decays naturally in the rolling window).
for resource in &[
    Resource::Ember, Resource::Reflection, Resource::VoidEssence,
    Resource::Memory, Resource::Silence, Resource::Resonance,
    Resource::ForgedLight, Resource::EchoGlass, Resource::StillbornSong,
    Resource::CondensedEmber, Resource::EmberEcho, Resource::PurifiedVoid,
    Resource::WovenReality,
] {
    if !produced.contains_key(resource) {
        loom.rate_trackers
            .entry(*resource)
            .or_insert_with(RateTracker::new)
            .push(0.0);
    }
}

// Read measured rates from trackers for pattern sustain.
let rates: std::collections::HashMap<crate::loom::Resource, f64> = loom
    .rate_trackers
    .iter()
    .map(|(resource, tracker)| (*resource, tracker.rate_per_hour()))
    .collect();

let pattern_completed =
    crate::loom::tick_pattern_sustain(&mut loom.persistent, &rates, TICK_SECONDS);
if pattern_completed {
    result.loom_changed = true;
}
```

**Step 3: Add necessary imports**

Add `use crate::loom::types::RateTracker;` (or import via `crate::loom::RateTracker`) at the top of `tick_stages.rs` if needed.

**Step 4: Update mod.rs re-exports**

In `src/loom/mod.rs`, add `RateTracker` to the re-exports from `types`:

```rust
pub use types::{
    ..., RateTracker, ...
};
```

**Step 5: Run all tests**

Run: `cargo test --lib -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/types.rs src/core/tick_stages.rs src/loom/mod.rs
git commit -m "feat(loom): wire RateTracker into tick loop for measured production rates"
```

---

### Task 8: Update pattern bar UI for rate-based display

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Context:** The pattern bar currently shows `accumulated/amount` per requirement. Update it to show:
- Rate display: `52/hr (need 25/hr)` with green/yellow coloring
- Time display: `15:00/30:00` sustain progress bar
- State indicator: `✓` (advancing) or `⏸` (paused)
- Completed requirements show a checkmark and full bar
- Update "All 18 Patterns Complete" to "All 28 Patterns Complete"

**Step 1: Read the current render_pattern_bar function**

Already read above (lines 1994-2167). The function renders per-requirement Gauge widgets showing `accumulated/amount`.

**Step 2: Update render_pattern_bar()**

Replace the per-requirement rendering in `render_pattern_bar()`. The key changes:

1. Change completion message from "18" to "28":
```rust
" \u{2728} Loom Mended \u{2014} All 28 Patterns Complete ",
```

2. Change per-requirement display. The gauge ratio becomes `sustained_secs / sustain_duration_secs`. The count label becomes time format `HH:MM/HH:MM`. Add rate display from `loom_state.rate_trackers`.

```rust
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

    // Get current measured rate from rate trackers.
    let current_rate = loom_state
        .rate_trackers
        .get(&req.resource)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    let advancing = !met && current_rate >= req.required_rate;

    // Format time: sustained / duration in HH:MM
    let sustained_mins = (req.sustained_secs / 60.0) as u32;
    let duration_mins = (req.sustain_duration_secs / 60.0) as u32;
    let time_label = format!(
        "{}:{:02}/{}:{:02}",
        sustained_mins / 60, sustained_mins % 60,
        duration_mins / 60, duration_mins % 60,
    );

    // Rate label: "52/hr (25/hr)"
    let rate_label = format!("{:.0}/hr ({:.0}/hr)", current_rate, req.required_rate);

    // State indicator
    let state_icon = if met {
        " \u{2713}" // ✓
    } else if advancing {
        " \u{25B6}" // ▶
    } else {
        " \u{23F8}" // ⏸
    };

    // ... render emoji, gauge, labels using existing layout pattern ...
}
```

3. Update the count label column to show `time_label + rate_label + state_icon`:
```rust
let count_label = format!("{} {}{}", time_label, rate_label, state_icon);
```

4. Color the rate green when advancing, yellow/amber when paused, and bright green when completed.

5. Update the overall progress bar to average `sustained_secs / sustain_duration_secs` across all requirements.

**Step 3: Run the project to verify visually**

Run: `cargo build`
Expected: compiles successfully.

**Step 4: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): update pattern bar UI for rate-based sustained display"
```

---

### Task 9: Update max_refineries cap and hardcoded "18" references

**Files:**
- Modify: `src/loom/types.rs` — `max_refineries()` is fine (counts completed patterns dynamically), but cap should be 28 now
- Modify: `src/loom/patterns.rs` — update comment "all 18 patterns"
- Modify: `src/loom/CLAUDE.md` — update documentation
- Search for any other hardcoded "18" references

**Step 1: Find all "18" references in loom module**

Search for `18` in the loom module and UI:

```bash
grep -rn "18" src/loom/ src/ui/loom_scene.rs | grep -i "pattern\|18"
```

**Step 2: Update each reference**

- `src/loom/patterns.rs:105` — comment "all 18 patterns" → "all 28 patterns"
- `src/loom/CLAUDE.md:14` — "18 woven patterns" → "28 woven patterns"
- `src/loom/CLAUDE.md:83` — "max 18" → "max 28"
- `src/loom/CLAUDE.md` — update pattern system description from accumulated amounts to sustained rates
- `src/ui/loom_scene.rs:2004` — already updated in Task 8

**Step 3: Update CLAUDE.md documentation**

Update `src/loom/CLAUDE.md` to reflect:
- 28 patterns instead of 18
- Sustained rate mechanic instead of accumulated totals
- New `RateTracker` type
- Updated `PatternRequirement` fields
- Tier gates at 1/8/15 instead of 1/6/12
- Auto-drain buffer behavior

**Step 4: Run all tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/patterns.rs src/loom/CLAUDE.md
git commit -m "docs(loom): update references from 18 to 28 patterns and document sustained rate system"
```

---

### Task 10: Update pattern tests in patterns.rs for 28-pattern coverage

**Files:**
- Modify: `src/loom/patterns.rs`

**Context:** After Tasks 3 and 5, the test suite needs updating to exercise the 28-pattern set with sustained rate mechanics. Key scenarios to test:
- Multi-requirement pattern with independent completion (e.g., pattern 4 "Mirror and Void")
- Pattern completion advances to next pattern correctly through all 28
- `all_patterns_complete()` works with 28 patterns

**Step 1: Add/update multi-requirement tests**

```rust
#[test]
fn test_multi_requirement_independent_completion() {
    let mut state = state_with_patterns();
    // Pattern 4 "Mirror and Void": Reflection 30/hr for 6hr, VoidEssence 30/hr for 6hr
    for i in 0..4 {
        state.persistent.patterns[i].completed = true;
    }
    state.persistent.active_pattern = 4;

    // Complete only the first requirement.
    state.persistent.patterns[4].requirements[0].sustained_secs =
        state.persistent.patterns[4].requirements[0].sustain_duration_secs;
    state.persistent.patterns[4].requirements[0].completed = true;

    // Pattern should NOT be complete yet (second requirement pending).
    assert!(!active_pattern_requirements_met(&state.persistent));

    // Complete second requirement.
    state.persistent.patterns[4].requirements[1].sustained_secs =
        state.persistent.patterns[4].requirements[1].sustain_duration_secs;
    state.persistent.patterns[4].requirements[1].completed = true;

    // Now pattern should be complete.
    assert!(active_pattern_requirements_met(&state.persistent));
}

#[test]
fn test_all_28_patterns_complete() {
    let mut state = state_with_patterns();
    for p in &mut state.persistent.patterns {
        p.completed = true;
    }
    assert!(all_patterns_complete(&state.persistent));
    assert_eq!(state.persistent.patterns.len(), 28);
}

#[test]
fn test_sustained_rate_exact_threshold_advances() {
    let mut state = state_with_patterns();
    // Rate exactly equal to required_rate should advance.
    let threshold = state.persistent.patterns[0].requirements[0].required_rate;
    let r = rates(&[(Resource::Ember, threshold)]);
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    assert!(state.persistent.patterns[0].requirements[0].sustained_secs > 0.0);
}
```

**Step 2: Run all tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git add src/loom/patterns.rs
git commit -m "test(loom): add sustained rate tests for 28-pattern coverage"
```

---

### Task 11: Full integration verification

**Files:** None (read-only verification)

**Step 1: Run full CI check suite**

```bash
make check
```

This runs: formatting, clippy, all tests, build, and audit.

Expected: All checks pass.

**Step 2: Verify loom tests specifically**

```bash
cargo test --lib loom:: -- --nocapture 2>&1 | tail -20
```

Expected: All loom tests pass.

**Step 3: Build and verify no warnings**

```bash
cargo build 2>&1 | grep -i warning
```

Expected: No new warnings (existing `dead_code` allows are fine).

**Step 4: Commit any remaining fixes**

If any issues surfaced during verification, fix and commit them.
