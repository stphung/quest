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
