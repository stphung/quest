# The Deep — Integration Architecture

**Date**: 2026-02-22
**Author**: sys-arch-2 (integration architecture agent)
**Status**: Draft

This document specifies exactly which existing files change, what new files are created, and how The Deep system integrates with every existing Quest subsystem.

---

## Overview

The Deep is an account-level system (like Haven and Soulforge) that persists across characters and prestiges. Its state is loaded at startup and saved alongside Haven and Enhancement. Missions run on wall-clock time so they progress while the game is closed. The integration surface is designed to be minimal and consistent with existing patterns.

---

## 1. New Module: `src/deep/`

Follow the standard module structure used by `src/haven/` and `src/enhancement/`.

### Files to Create

```
src/deep/
├── mod.rs          — Public re-exports + discovery roll function
├── types.rs        — All data structures (see Task #1 for full types spec)
├── generation.rs   — Merc generation, mission generation, event templates
├── logic.rs        — Mission ticking, event resolution, squad validation
├── persistence.rs  — Save/load from ~/.quest/deep.json
└── discovery.rs    — Discovery roll logic (try_discover_deep)
```

### `src/deep/mod.rs` — Public Re-exports

```rust
pub mod discovery;
pub mod generation;
pub mod logic;
pub mod persistence;
pub mod types;

pub use discovery::try_discover_deep;
pub use persistence::{load_deep, save_deep};
pub use types::DeepState;
```

### `src/deep/persistence.rs` — Save/Load

Mirrors `src/enhancement/persistence.rs` exactly. File: `~/.quest/deep.json`.

```rust
pub fn deep_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir()...;
    Ok(home_dir.join(".quest").join("deep.json"))
}

pub fn load_deep() -> DeepState {
    // Read file, serde_json::from_str, return DeepState::new() on error
}

pub fn save_deep(deep: &DeepState) -> io::Result<()> {
    // create_dir_all, to_string_pretty, write
}
```

### `src/deep/discovery.rs` — Discovery Roll

Mirrors the pattern from `src/haven/bonus.rs::haven_discovery_chance()` and `src/enhancement/logic.rs::try_discover_soulforge()`.

```rust
pub const DEEP_MIN_PRESTIGE_RANK: u32 = 15;
pub const DEEP_DISCOVERY_BASE_CHANCE: f64 = 0.000014; // same as Haven/Soulforge
pub const DEEP_DISCOVERY_RANK_BONUS: f64 = 0.000007;  // same as Haven/Soulforge

pub fn deep_discovery_chance(prestige_rank: u32) -> f64 {
    if prestige_rank < DEEP_MIN_PRESTIGE_RANK {
        return 0.0;
    }
    DEEP_DISCOVERY_BASE_CHANCE
        + (prestige_rank - DEEP_MIN_PRESTIGE_RANK) as f64 * DEEP_DISCOVERY_RANK_BONUS
}

pub fn try_discover_deep<R: Rng>(deep: &mut DeepState, prestige_rank: u32, rng: &mut R) -> bool {
    if deep.discovered {
        return false;
    }
    let chance = deep_discovery_chance(prestige_rank);
    if rng.random::<f64>() < chance {
        deep.discovered = true;
        true
    } else {
        false
    }
}
```

---

## 2. Modified File: `src/core/tick_types.rs`

### Add `deep_changed` Flag to `TickResult`

**File**: `/Users/stphung/workspace/quest3/src/core/tick_types.rs`

Add one field to `TickResult` (after `enhancement_changed`, following established pattern):

```rust
/// True if Deep state was modified (discovery) and should be persisted.
pub deep_changed: bool,
```

### Add `DeepDiscovered` Variant to `TickEvent`

Add to the Discovery section (after `SoulforgeDiscovered`):

```rust
/// The Deep was discovered (P15+ idle roll).
DeepDiscovered,
```

---

## 3. Modified File: `src/core/tick.rs`

### Add Deep Discovery Check (Stage 12 — New Stage)

The existing stages go to 12 (achievement modal accumulation). Add a new Stage 12 for Deep discovery, pushing achievement modal to Stage 13. Alternatively, insert it between Soulforge discovery (Stage 11) and achievement modal (Stage 12). The cleaner approach is to insert it as Stage 11b:

**Import additions at top of file**:

```rust
use crate::deep::DeepState;
```

**Function signature change** — add `deep: &mut DeepState` parameter:

```rust
pub fn game_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    enhancement: &mut crate::enhancement::EnhancementProgress,
    deep: &mut crate::deep::DeepState,          // NEW
    achievements: &mut Achievements,
    debug_mode: bool,
    rng: &mut R,
) -> TickResult
```

**Add discovery stage** (after Soulforge discovery block, before achievement modal):

```rust
// ── 11b. Deep discovery check ────────────────────────────
// Independent roll per tick, only when eligible (P15+, no active content)
if !deep.discovered
    && state.prestige_rank >= crate::deep::discovery::DEEP_MIN_PRESTIGE_RANK
    && state.active_dungeon.is_none()
    && state.active_fishing.is_none()
    && state.active_minigame.is_none()
    && crate::deep::try_discover_deep(deep, state.prestige_rank, rng)
{
    result.events.push(TickEvent::DeepDiscovered);
    result.deep_changed = true;
    if !debug_mode {
        result.achievements_changed = true;
    }
}
```

**Design rationale**: Deep discovery uses the same guard conditions as Haven and Soulforge (no active content). This ensures discoveries don't fire during dungeon/fishing/minigame sessions. The identical prestige threshold (P15+) matches Soulforge and Stormglass, making The Deep the third P15+ system alongside them.

---

## 4. Modified File: `src/main.rs`

### Load Deep State at Startup

After the Haven and Enhancement load calls (around line 178-180):

```rust
// Load account-level Deep state
let mut deep = deep::load_deep();
```

**Declare `mod deep`** at the top of `main.rs` (alongside other module declarations):

```rust
mod deep;
```

### Pass `deep` to `game_tick()`

Every call to `core::tick::game_tick()` in `main.rs` must add `&mut deep` as a parameter. There are three call sites:

1. **Normal game tick** (line ~752): Add `&mut deep,` before `&mut global_achievements`.
2. **Chrono Surge batch tick** (line ~678): Same.
3. **Chrono Surge skip/Esc tick loop** (line ~491): Same.

### Handle `deep_changed` Flag in Tick Result

After the existing flag checks in the normal tick path (around line 778-791), add:

```rust
if tick_result.deep_changed && !debug_mode {
    deep::save_deep(&deep).ok();
}
```

### Handle `DeepDiscovered` TickEvent

In `src/tick_events.rs` (the `apply_tick_events` function), add a case for `TickEvent::DeepDiscovered` that returns a flag, similar to how `HavenDiscovered` and `SoulforgeDiscovered` are handled.

Then in the main tick event processing block in `main.rs` (around line 796-804):

```rust
if tick_flags.deep_discovered {
    overlay = GameOverlay::DeepDiscovery;
}
```

### Save Deep in `save_all()`

**File**: `/Users/stphung/workspace/quest3/src/main_helpers/persistence.rs`

```rust
pub fn save_all(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    deep: &deep::DeepState,                   // NEW parameter
) {
    let _ = character_manager.save_character(state);
    achievements::save_achievements(global_achievements).ok();
    if haven.discovered {
        haven::save_haven(haven).ok();
    }
    if enhancement.discovered {
        enhancement::save_enhancement(enhancement).ok();
    }
    if deep.discovered {                       // NEW block
        deep::save_deep(deep).ok();
    }
}
```

Update all call sites of `save_all()` in `main.rs` to pass `&deep`.

### Resolve Completed Missions on Login

**File**: `/Users/stphung/workspace/quest3/src/main_helpers/offline.rs`

Add a parallel function to `apply_offline_xp()` for Deep offline resolution:

```rust
pub fn resolve_deep_offline(deep: &mut crate::deep::DeepState) -> Option<DeepOfflineReport> {
    crate::deep::logic::resolve_offline_missions(deep)
}
```

`resolve_offline_missions()` in `src/deep/logic.rs` inspects `last_resolved_at` timestamps on each active mission, completes missions whose `end_time` has passed, auto-resolves any check-in events that fired while offline (always picking the safe choice), and queues rewards for collection.

Call this from the character load path in `src/main_helpers/character_screens.rs` after offline XP is applied.

---

## 5. Modified File: `src/tick_events.rs`

**File**: `/Users/stphung/workspace/quest3/src/tick_events.rs`

The `apply_tick_events()` function returns a flags struct. Add a `deep_discovered` field:

```rust
pub struct TickFlags {
    pub haven_discovered: bool,
    pub soulforge_discovered: bool,
    pub stormglass_discovered: bool,
    pub deep_discovered: bool,              // NEW
}
```

Add match arm:

```rust
TickEvent::DeepDiscovered => {
    state.combat_state.add_log_entry(
        "\u{1F30A} A mercenary captain has found you. The Deep awaits...".to_string(),
        false,
        true,
    );
    flags.deep_discovered = true;
}
```

---

## 6. Modified File: `src/input/mod.rs`

### Add `DeepOverlay` Game Overlay State

**File**: `/Users/stphung/workspace/quest3/src/input/types.rs`

Add to the `GameOverlay` enum:

```rust
DeepDiscovery,
DeepOverlay,
```

### Add Deep Input Handler

Create **new file** `/Users/stphung/workspace/quest3/src/input/deep_input.rs`:

```rust
//! Input handling for The Deep overlay.
use crate::deep::DeepState;
use crate::input::types::{GameOverlay, InputResult};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_deep(
    key: KeyEvent,
    deep: &mut DeepState,
    overlay: &mut GameOverlay,
) -> InputResult {
    match key.code {
        KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
            *overlay = GameOverlay::None;
            InputResult::Continue
        }
        // TODO: Navigation, mission selection, event response
        _ => InputResult::Continue,
    }
}
```

Declare in `src/input/mod.rs`:

```rust
mod deep_input;
use deep_input::handle_deep;
```

### Add Deep Discovery Modal Handler

In `handle_game_input()` in `src/input/mod.rs`, add after the Stormglass discovery handler (around line 138-141):

```rust
// 1d. Deep discovery modal (blocks all other input)
if matches!(overlay, GameOverlay::DeepDiscovery) {
    return handle_deep_discovery(key, overlay);
}
```

Add the handler function:

```rust
fn handle_deep_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}
```

### Add Deep Overlay Handler in Input Priority Chain

In `handle_game_input()`, add after the Stormglass Exchange handler (around line 157-161):

```rust
// 2.8. Deep overlay
if matches!(overlay, GameOverlay::DeepOverlay) {
    return handle_deep(key, deep, overlay);
}
```

The function signature of `handle_game_input()` must add the `deep` parameter:

```rust
pub fn handle_game_input(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    exchange_ui: &mut ExchangeUiState,
    enhancement: &mut enhancement::EnhancementProgress,
    deep: &mut crate::deep::DeepState,            // NEW
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
    debug_mode: bool,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult
```

### Add `[D]` Keybind in Base Game Input

In `handle_base_game()` in `src/input/mod.rs`, add after the `[G]` Stormglass keybind (around line 385-390):

```rust
KeyCode::Char('d') | KeyCode::Char('D') => {
    if deep.discovered {
        *overlay = GameOverlay::DeepOverlay;
    }
    InputResult::Continue
}
```

The `handle_base_game()` signature must also receive `deep`:

```rust
fn handle_base_game(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    exchange_ui: &mut ExchangeUiState,
    enhancement: &enhancement::EnhancementProgress,
    deep: &crate::deep::DeepState,                // NEW
    overlay: &mut GameOverlay,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult
```

---

## 7. Modified File: `src/character/prestige_actions.rs`

### On Prestige: Reset Transient Deep State, Preserve Persistent

In `perform_prestige()`, add after the existing resets:

```rust
// Note: DeepState is account-level (not on GameState), so it's passed
// separately and handled in the prestige input handler, not here.
// The UI layer (prestige_input.rs) must call deep.on_prestige() after
// calling perform_prestige().
```

**File to modify**: `/Users/stphung/workspace/quest3/src/input/prestige_input.rs`

In `handle_prestige_confirm()`, after calling `perform_prestige()` or `perform_prestige_with_vault()`:

```rust
// Reset prestige-scoped Deep state (mercs, marks, active missions)
// while preserving guild rank, layer progression, infrastructure
deep.on_prestige();
```

This requires `deep: &mut crate::deep::DeepState` to be threaded through to `handle_prestige_confirm()`. The function signature becomes:

```rust
pub fn handle_prestige_confirm(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    deep: &mut crate::deep::DeepState,    // NEW
    overlay: &mut GameOverlay,
) -> InputResult
```

Update the call site in `handle_game_input()` to pass `deep`.

### `DeepState::on_prestige()` in `src/deep/types.rs`

```rust
impl DeepState {
    pub fn on_prestige(&mut self) {
        // Preserve: guild_rank, layers (cleared status, intel, infrastructure)
        // Reset: mercenaries, warband_marks, active_missions, available_missions
        self.mercenaries.clear();
        self.warband_marks = 0;
        self.active_missions.clear();
        self.available_missions.clear();
        // guild_rank, layers, deepest_layer_reached — untouched
    }
}
```

---

## 8. Modified File: `src/ui/mod.rs` and New `src/ui/deep_scene.rs`

### New UI Scene

Create `/Users/stphung/workspace/quest3/src/ui/deep_scene.rs` following the pattern of `soulforge_scene.rs` and `haven_scene.rs`. Sub-modules for clarity:

```
src/ui/deep_scene.rs        — Main overlay coordinator
src/ui/deep_roster.rs       — Mercenary roster panel
src/ui/deep_missions.rs     — Active/available missions panel
src/ui/deep_infrastructure.rs — Layer infrastructure view
```

### Pending Event Indicator in Stats Panel

**File**: `/Users/stphung/workspace/quest3/src/ui/stats_panel.rs`

Add a subtle indicator when The Deep has a pending check-in event that needs player response. This follows the same pattern as achievement notification counts. The stats panel already reads `state` and a separate `Achievements` — it will also receive `deep: &crate::deep::DeepState` (read-only reference) to check `deep.has_pending_events()`.

Add to the stats panel rendering to show something like:

```
[D] The Deep ⚡ event
```

The indicator appears only when `deep.discovered && deep.has_pending_events()`.

### Discovery Overlay

**File**: `/Users/stphung/workspace/quest3/src/main_helpers/overlay.rs`

Add Deep discovery overlay rendering to `draw_game_overlays()`:

```rust
GameOverlay::DeepDiscovery => {
    ui::deep_scene::render_discovery_modal(frame, area);
}
GameOverlay::DeepOverlay => {
    ui::deep_scene::render_deep_overlay(frame, area, deep, &ctx);
}
```

The `draw_game_overlays()` function must receive `deep: &crate::deep::DeepState` as a new parameter.

---

## 9. Modified File: `src/main_helpers/character_screens.rs`

### Pass `deep` to Character Load Flow

When loading a character, the offline progression runs. Add Deep offline resolution here:

```rust
// Resolve completed Deep missions that finished while offline
if deep.discovered {
    let deep_report = crate::deep::logic::resolve_offline_missions(&mut deep);
    if let Some(report) = deep_report {
        // Queue rewards, show summary in overlay or combat log
    }
}
```

This requires `deep: &mut crate::deep::DeepState` to be threaded through the `handle_select_frame()` call chain to `LoadCharacter`.

---

## 10. Reward Integration: Existing Systems

### XP Flow

Mission rewards include XP. This is applied via `crate::core::xp::apply_tick_xp()` during offline mission resolution and reward collection, exactly as offline XP is applied in `src/core/offline.rs`. No changes to the XP system itself.

### Item Flow

Mission rewards include items. Generated items use the existing `src/items/generation.rs` pipeline. Abyssal equipment uses the same `Item` struct with a special affix type added to `src/items/types.rs`:

**File**: `/Users/stphung/workspace/quest3/src/items/types.rs`

Add affix variants to the existing `AffixType` enum:

```rust
// Abyssal (Deep-exclusive) affixes
AbyssalMissionSpeed,     // % faster mission timers
AbyssalSupplyYield,      // % more Warband Marks from supply runs
AbyssalResilience,       // bonus merc resilience
```

### Stormglass Flow

Expedition and breakthrough missions award Stormglass. This increments `state.stormglass` directly during reward collection, same as salvage and dungeon caches. No changes needed to the Stormglass system.

### Prestige Rank Fragments

Breakthrough missions on deep layers (19+) award fractional prestige ranks. This is a new concept — the cleanest integration is:

**File**: `/Users/stphung/workspace/quest3/src/core/game_state.rs`

Add a field to `GameState`:

```rust
/// Fractional prestige rank progress from Deep breakthroughs (0.0..1.0)
/// When it reaches 1.0, it is consumed to grant 1 prestige rank.
#[serde(default)]
pub deep_prestige_fragments: f64,
```

During mission reward collection, `deep_prestige_fragments += reward_amount`. When it crosses 1.0, subtract 1.0 and increment `prestige_rank` by 1 (without triggering a full prestige reset — this is a pure rank increment).

This is done in `src/deep/logic.rs::apply_mission_rewards()` which operates on `&mut GameState`.

---

## 11. Mission Timer Architecture

### Wall-Clock Time for Missions

Mission completion is determined by comparing `mission.start_time` (Unix timestamp, `i64`) against the current wall-clock time. This is the same approach used by `offline.rs` for offline XP:

```rust
// src/deep/types.rs
pub struct ActiveMission {
    pub mission_def: MissionDef,
    pub squad: Vec<MercId>,
    pub start_time: i64,           // Unix timestamp (Utc::now().timestamp())
    pub end_time: i64,             // start_time + mission_duration_seconds
    pub events: Vec<CheckInEvent>, // Pre-generated events with scheduled fire times
    pub event_choices: Vec<Option<usize>>, // Player's choices (None = auto-resolve)
    pub last_resolved_at: i64,     // Tracks partial resolution
}
```

### Tick-Based Mission Progress Updates

During the game tick (in `src/core/tick.rs`), mission timers do NOT use game ticks. Instead, missions are checked on each tick by comparing current wall-clock time against `end_time`. This is fast (just timestamp comparison, no simulation).

Add a lightweight mission check to `game_tick()`:

**Stage 13** (after achievement modal): Check for pending Deep check-in events that fired during this tick (events with `fire_time <= Utc::now().timestamp()` that have no choice yet). If found, set a flag in `TickResult`:

```rust
/// If Some, a Deep check-in event is ready for player response.
pub deep_event_ready: bool,
```

This flag is used by `main.rs` to show the event indicator in the UI (not to open the overlay automatically — the player presses `[D]` to respond).

Completed missions (where `end_time <= now`) are not resolved during the tick — they are resolved during offline resolution on load, or when the player opens The Deep overlay. This keeps the tick stage minimal and avoids item generation during the game loop.

---

## 12. Debug Menu Integration

**File**: `/Users/stphung/workspace/quest3/src/utils/debug_menu.rs`

Add a "Deep" category to the debug menu with options:
- "Discover The Deep" (sets `deep.discovered = true`, shows discovery modal)
- "Add 1000 Warband Marks"
- "Complete all missions"
- "Force next layer breakthrough"

This follows the existing pattern in `debug_menu.rs` (tabbed categories: Challenges, World, Resources, Items — add "Deep" as a fifth category).

---

## 13. Complete Change Summary

### New Files

| File | Description |
|------|-------------|
| `src/deep/mod.rs` | Module root, public re-exports |
| `src/deep/types.rs` | DeepState, Mercenary, Mission, Layer, Guild structs |
| `src/deep/generation.rs` | Merc/mission/event generation |
| `src/deep/logic.rs` | Mission ticking, event resolution, reward application |
| `src/deep/persistence.rs` | load_deep() / save_deep() |
| `src/deep/discovery.rs` | try_discover_deep(), discovery constants |
| `src/input/deep_input.rs` | handle_deep() input handler |
| `src/ui/deep_scene.rs` | Deep overlay coordinator |
| `src/ui/deep_roster.rs` | Mercenary roster panel |
| `src/ui/deep_missions.rs` | Missions panel |
| `src/ui/deep_infrastructure.rs` | Infrastructure/layer view |

### Modified Files

| File | Change |
|------|--------|
| `src/main.rs` | Add `mod deep`, load/save Deep, pass to game_tick, handle discovery overlay, resolve offline missions, update save_all calls |
| `src/core/tick.rs` | Add `deep: &mut DeepState` param, add Stage 11b (Deep discovery), add Stage 13 (check event ready) |
| `src/core/tick_types.rs` | Add `TickEvent::DeepDiscovered`, `TickResult::deep_changed`, `TickResult::deep_event_ready` |
| `src/tick_events.rs` | Handle `DeepDiscovered` event, add `deep_discovered` to `TickFlags` |
| `src/input/mod.rs` | Declare `deep_input`, add `deep` parameter, add discovery modal handler, add overlay handler, add `[D]` keybind, add deep to handle_base_game |
| `src/input/types.rs` | Add `GameOverlay::DeepDiscovery` and `GameOverlay::DeepOverlay` variants |
| `src/input/prestige_input.rs` | Add `deep: &mut DeepState` param, call `deep.on_prestige()` on prestige |
| `src/main_helpers/persistence.rs` | Add `deep: &crate::deep::DeepState` param to save_all(), call save_deep() |
| `src/main_helpers/offline.rs` | Add resolve_deep_offline() function |
| `src/main_helpers/overlay.rs` | Add Deep overlay rendering cases to draw_game_overlays() |
| `src/main_helpers/character_screens.rs` | Pass deep through LoadCharacter path, call resolve_offline_missions() |
| `src/character/prestige_actions.rs` | Add doc comment noting DeepState prestige handling is in prestige_input.rs |
| `src/items/types.rs` | Add AbyssalMissionSpeed, AbyssalSupplyYield, AbyssalResilience to AffixType |
| `src/core/game_state.rs` | Add `deep_prestige_fragments: f64` field |
| `src/ui/mod.rs` | Declare deep_scene, deep_roster, deep_missions, deep_infrastructure |
| `src/ui/stats_panel.rs` | Add pending event indicator when deep.has_pending_events() |
| `src/utils/debug_menu.rs` | Add "Deep" category with test triggers |

---

## 14. Dependency Graph for Implementation Tasks

The following dependencies exist between implementation tasks:

```
Task #8 (types.rs)
    └── blocks Task #9 (merc system)
    └── blocks Task #10 (mission system)
    └── blocks Task #11 (layer/economy)
    └── blocks Task #12 (persistence/discovery) ← This doc (integration arch) blocks #12
    └── blocks Task #13 (UI)
    └── blocks Task #14 (input)

Task #12 (persistence/discovery)
    └── blocks main.rs integration

Task #9 + #10 + #11
    └── block Task #12 (logic.rs depends on all types being defined)

Task #12 + #13 + #14
    └── block Task #15-21 (tests)
```

The integration document (this file) blocks Task #12 because the persistence integration pattern and discovery mechanism must be defined before implementation begins.

---

## 15. Architectural Decisions

### Why Account-Level (Not Character-Level)?

Deep state is account-level (like Haven and Soulforge) because:
1. Layer progression and infrastructure persist across prestiges by design
2. Guild rank is a permanent unlock
3. Multiple characters would share the same mercenary company narrative

This means `deep: DeepState` lives in `main.rs` alongside `haven` and `enhancement`, not embedded in `GameState`.

### Why Wall-Clock Time for Missions?

Missions are intended to progress while the game is closed. Using `last_save_time` (Unix timestamps) for missions follows exactly the same pattern as offline XP in `src/core/offline.rs`. The game tick does not simulate mission progress — it only checks for pending events and completion on the next frame after the game is already running.

### Why Not Extend `GameState`?

Adding `deep_state: Option<DeepState>` to `GameState` would couple Deep to character saves, causing The Deep to be reset when a character is deleted. As an account-level system, it must be independent. This is consistent with how Haven and Enhancement are handled.

The only Deep data that belongs in `GameState` is `deep_prestige_fragments` (fractional PR from missions), because prestige rank is character-level state. All other Deep data stays in the standalone `DeepState`.

### Why `[D]` Keybind?

Currently assigned keybinds: `[H]` Haven, `[S]` Soulforge, `[G]` Stormglass, `[A]` Achievements, `[P]` Prestige, `[Tab]` Challenges, `[?]` Help, `[U]` Updates, `[!]` Bug report.

`[D]` for "The Deep" is natural and unoccupied. The existing `[D]` key in the challenge menu is scoped to the challenge menu context only (Decline), so it does not conflict with the base game keybind.

### Discovery: Same Rate as Soulforge

The Deep uses the same discovery rate formula as Soulforge and Haven: `0.000014 + (rank - 15) * 0.000007` per tick. This gives an average discovery time of roughly 2 hours at P15 (same as Soulforge). This is intentional — all three P15+ systems are meant to be discovered at roughly the same progression milestone.

### Mission Resolution: Offline, Not On-Tick

Completed missions are not resolved during the game tick — they are resolved:
1. On game load (in the character load path, same as offline XP)
2. When the player opens The Deep overlay

This keeps the tick function minimal. The tick only checks for pending check-in events (a simple timestamp comparison, no item generation or complex logic).
