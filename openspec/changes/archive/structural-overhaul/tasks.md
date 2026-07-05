> Backported implementation plan (completed — this work shipped).

## 2026-02-27-structural-overhaul-plan.md

# Structural Overhaul Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor Quest's architecture for maintainability and extensibility using Module Facades + Decomposed State (Approach B from the design doc).

**Architecture:** Decompose the GameState god object into 4 composed sub-structs, introduce module facades with explicit input structs, extract shared forfeit helper for challenges, decompose large UI render functions, and simplify the tick engine with TickContext.

**Tech Stack:** Rust, Serde (custom Serialize/Deserialize for backward-compat JSON), Ratatui

**Design Doc:** `docs/plans/2026-02-27-structural-overhaul-design.md`

**Constraint:** Conservative — all 5,642+ existing tests must pass unchanged. No public API removals, only deprecations. JSON save format preserved. Rendering pixel-identical.

---

## Team Allocation

| Agent | Role | Tasks |
|-------|------|-------|
| `sys-arch-1` | Systems Architect | Tasks 1-2 (core struct + facade design) |
| `sys-arch-2` | Systems Architect | Tasks 3-4 (challenge + UI pattern design) |
| `dev-1` | Developer | Tasks 5-9 (GameState split + TickContext) |
| `dev-2` | Developer | Tasks 10-15 (module facades) |
| `dev-3` | Developer | Tasks 16-18 (forfeit helper + challenge cleanup) |
| `dev-4` | Developer | Tasks 19-22 (UI render decomposition) |
| `qa-1` | QA | Tasks 23-24 (core + tick tests) |
| `qa-2` | QA | Task 25 (combat + zone + dungeon + fishing tests) |
| `qa-3` | QA | Task 26 (challenge tests) |
| `qa-4` | QA | Task 27 (full make check + integration) |
| `eng-mgr-1` | Eng Manager | Coordinates core track (arch-1 → dev-1 → dev-2 → qa-1/2) |
| `eng-mgr-2` | Eng Manager | Coordinates systems track (arch-2 → dev-3 → dev-4 → qa-3/4) |
| `game-designer` | Game Designer | Task 28 (gameplay invariant audit) |

## Phase Dependencies

```
Phase 1 (parallel):  Tasks 1-4  (architects design)
Phase 2 (serial):    Tasks 5-9  (dev-1: GameState split — everything depends on this)
Phase 3 (parallel):  Tasks 10-22 (dev-2/3/4: facades, challenges, UI)
Phase 4 (parallel):  Tasks 23-27 (qa-1/2/3/4: validation)
Phase 5 (serial):    Task 28    (game-designer: final audit)
```

---

## Phase 1: Architecture Design

### Task 1: Design Core Sub-Structs (sys-arch-1)

**Files:**
- Create: `src/core/player_identity.rs`
- Create: `src/core/combat_context.rs`
- Create: `src/core/progression_state.rs`
- Create: `src/core/session_state.rs`
- Modify: `src/core/game_state.rs`

**Goal:** Define the 4 sub-structs that GameState will compose. Each sub-struct groups fields by domain.

**Step 1: Analyze current GameState field groupings**

Read `src/core/game_state.rs` (lines 48-134) and categorize every field into one of:
- `PlayerIdentity` — character identity, level, XP, attributes, prestige
- `CombatContext` — combat state, equipment, zones, dungeon, kill tracking
- `ProgressionState` — fishing, stormglass, sigils, challenges, minigames
- `SessionState` — transient UI/cache state, timers, ticker, derived stats

**Step 2: Write PlayerIdentity struct**

Create `src/core/player_identity.rs`:
```rust
use crate::character::attributes::Attributes;
use serde::{Deserialize, Serialize};

/// Character identity and progression fields.
/// Grouped for clarity — these define "who the character is."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerIdentity {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
}
```

**Step 3: Write CombatContext struct**

Create `src/core/combat_context.rs`:
```rust
use crate::combat::types::CombatState;
use crate::dungeon::types::Dungeon;
use crate::items::equipment::Equipment;
use crate::zones::ZoneProgression;
use serde::{Deserialize, Serialize};

/// Combat-related state: the fighting context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatContext {
    pub combat_state: CombatState,
    pub equipment: Equipment,
    #[serde(default)]
    pub zone_progression: ZoneProgression,
    #[serde(default)]
    pub active_dungeon: Option<Dungeon>,
    #[serde(skip)]
    pub session_kills: u64,
    #[serde(skip)]
    pub consecutive_deaths: u32,
}
```

**Step 4: Write ProgressionState struct**

Create `src/core/progression_state.rs`:
```rust
use crate::challenges::chess::ChessStats;
use crate::challenges::menu::ChallengeMenu;
use crate::challenges::ActiveMinigame;
use crate::challenges::MinigameWinInfo;
use crate::fishing::types::{FishingSession, FishingState};
use crate::stormglass::sigils::StormSigils;
use serde::{Deserialize, Serialize};

/// Non-combat progression: fishing, challenges, stormglass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionState {
    #[serde(default)]
    pub fishing: FishingState,
    #[serde(skip)]
    pub active_fishing: Option<FishingSession>,
    #[serde(default)]
    pub stormglass: u64,
    #[serde(default)]
    pub stormglass_discovered: bool,
    #[serde(default)]
    pub storm_sigils: StormSigils,
    #[serde(skip)]
    pub challenge_menu: ChallengeMenu,
    #[serde(skip)]
    pub chess_stats: ChessStats,
    #[serde(skip)]
    pub active_minigame: Option<ActiveMinigame>,
    #[serde(skip)]
    pub last_minigame_win: Option<MinigameWinInfo>,
}
```

**Step 5: Write SessionState struct**

Create `src/core/session_state.rs`:
```rust
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::core::recent_drops::RecentDrop;
use crate::core::ticker::Ticker;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

/// Transient session state: caches, timers, UI state.
/// Entirely #[serde(skip)] — none of this persists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    #[serde(skip)]
    pub chrono_surge_active: bool,
    #[serde(skip)]
    pub debug_force_overcharge: bool,
    #[serde(skip)]
    pub recent_drops: VecDeque<RecentDrop>,
    #[serde(skip)]
    pub xp_rate_samples: VecDeque<u64>,
    #[serde(skip)]
    pub xp_this_second: u64,
    #[serde(skip)]
    pub ticker: Ticker,
    #[serde(skip)]
    pub cached_derived_stats: DerivedStats,
    #[serde(skip)]
    pub cached_prestige_bonuses: PrestigeCombatBonuses,
    #[serde(skip)]
    pub derived_stats_dirty: bool,
    #[serde(skip)]
    pub combat_seconds_this_tick: bool,
    #[serde(skip)]
    pub game_over_shown_at: Option<Instant>,
}
```

**Step 6: Run tests to verify structs compile**

Run: `cargo build 2>&1 | head -20`
Expected: builds successfully (structs are defined but not yet used)

**Step 7: Commit**

```bash
git add src/core/player_identity.rs src/core/combat_context.rs \
        src/core/progression_state.rs src/core/session_state.rs
git commit -m "refactor(core): add GameState sub-structs (PlayerIdentity, CombatContext, ProgressionState, SessionState)"
```

---

### Task 2: Design TickContext and Facade Signatures (sys-arch-1)

**Files:**
- Create: `src/core/tick_context.rs`
- Create: `src/fishing/facade.rs`
- Create: `src/dungeon/facade.rs`
- Create: `src/combat/facade.rs`

**Goal:** Define the TickContext struct and all facade input struct signatures. Implementation comes later — this is just the type definitions.

**Step 1: Write TickContext**

Create `src/core/tick_context.rs`:
```rust
use crate::achievements::types::Achievements;
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use crate::enhancement::types::EnhancementProgress;
use crate::haven::types::Haven;

/// Bundles all mutable references needed by game_tick() into one parameter.
pub struct TickContext<'a> {
    pub state: &'a mut GameState,
    pub tick_counter: &'a mut u32,
    pub haven: &'a mut Haven,
    pub enhancement: &'a mut EnhancementProgress,
    pub deep: &'a mut DeepState,
    pub achievements: &'a mut Achievements,
    pub debug_mode: bool,
}
```

**Step 2: Write fishing facade input struct**

Create `src/fishing/facade.rs`:
```rust
use crate::fishing::logic::HavenFishingBonuses;
use crate::fishing::types::{FishingSession, FishingState};

/// Explicit inputs for the fishing tick facade.
pub struct FishingInput<'a> {
    pub fishing: &'a mut FishingState,
    pub active_fishing: &'a mut Option<FishingSession>,
    pub player_level: u32,
    pub prestige_rank: u32,
    pub haven_bonuses: HavenFishingBonuses,
    pub stormglass: &'a mut u64,
    pub storm_lure_active: bool,
}
```

**Step 3: Write dungeon facade input struct**

Create `src/dungeon/facade.rs`:
```rust
use crate::dungeon::types::Dungeon;

/// Explicit inputs for the dungeon tick facade.
pub struct DungeonInput<'a> {
    pub dungeon: &'a mut Option<Dungeon>,
    pub zone_id: u32,
    pub prestige_rank: u32,
    pub player_level: u32,
}
```

**Step 4: Write combat facade input struct**

Create `src/combat/facade.rs`:
```rust
use crate::combat::events::CombatBonuses;
use crate::combat::types::CombatState;
use crate::character::derived_stats::DerivedStats;

/// Explicit inputs for the combat update facade.
pub struct CombatInput<'a> {
    pub combat_state: &'a mut CombatState,
    pub bonuses: &'a CombatBonuses,
    pub derived: &'a DerivedStats,
    pub prestige_rank: u32,
}
```

**Step 5: Run tests to verify all type definitions compile**

Run: `cargo build 2>&1 | head -20`
Expected: builds successfully

**Step 6: Commit**

```bash
git add src/core/tick_context.rs src/fishing/facade.rs \
        src/dungeon/facade.rs src/combat/facade.rs
git commit -m "refactor: add TickContext and facade input struct definitions"
```

---

### Task 3: Design Forfeit Helper API (sys-arch-2)

**Files:**
- Modify: `src/challenges/mod.rs`

**Goal:** Design the shared forfeit handler functions. The `impl_apply_game_result!` macro already exists and handles result application. This task adds the missing forfeit flow helpers.

**Step 1: Read the existing forfeit patterns**

Read these files to confirm the pattern:
- `src/challenges/flappy/logic.rs` lines 20-56
- `src/challenges/gomoku/logic.rs` lines 24-58
- `src/challenges/jezzball/logic.rs` lines 34-95

**Step 2: Design the shared forfeit API**

Add to `src/challenges/mod.rs` (after the existing macro):

```rust
/// Shared forfeit confirmation handler.
/// Returns true if the forfeit was confirmed (game_result set to loss).
/// Call this when the player presses Esc/Forfeit.
pub fn handle_forfeit<R>(
    game_result: &mut Option<R>,
    forfeit_pending: &mut bool,
    loss_variant: R,
) -> bool {
    if *forfeit_pending {
        *game_result = Some(loss_variant);
        true
    } else {
        *forfeit_pending = true;
        false
    }
}

/// Cancel a pending forfeit. Call this on any non-Esc input
/// when forfeit_pending is true.
pub fn cancel_forfeit_if_pending(forfeit_pending: &mut bool) {
    if *forfeit_pending {
        *forfeit_pending = false;
    }
}
```

**Step 3: Run tests**

Run: `cargo test --lib 2>&1 | tail -5`
Expected: all tests pass (new functions are defined but not yet called)

**Step 4: Commit**

```bash
git add src/challenges/mod.rs
git commit -m "refactor(challenges): add shared forfeit handler functions"
```

---

### Task 4: Design UI Layout Helpers (sys-arch-2)

**Files:**
- Create: `src/ui/overlay_layout.rs`

**Goal:** Define shared layout computation helpers for overlay scenes. These are pure layout math — no rendering.

**Step 1: Survey existing overlay layout patterns**

Read these files to identify the common centering/splitting patterns:
- `src/ui/stormglass_scene.rs` lines 410-425 (overlay sizing)
- `src/ui/deep_roster.rs` lines 343-356 (list/detail split)
- `src/ui/time_vault_scene.rs` lines 252-260 (overlay centering)

**Step 2: Write the shared layout helpers**

Create `src/ui/overlay_layout.rs`:
```rust
use ratatui::layout::Rect;

/// Layout for a centered overlay with title and footer.
pub struct OverlayLayout {
    pub outer: Rect,
    pub title_bar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

/// Compute a centered overlay layout.
/// `max_width` and `max_height` are the maximum overlay dimensions.
/// `title_height` is the number of rows for the title bar (typically 1-3).
/// `footer_height` is the number of rows for the footer (typically 1-2).
pub fn centered_overlay(
    area: Rect,
    max_width: u16,
    max_height: u16,
    title_height: u16,
    footer_height: u16,
) -> OverlayLayout {
    let w = max_width.min(area.width);
    let h = max_height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let outer = Rect::new(x, y, w, h);
    let title_bar = Rect::new(x, y, w, title_height);
    let content_y = y + title_height;
    let content_h = h.saturating_sub(title_height + footer_height);
    let content = Rect::new(x, content_y, w, content_h);
    let footer = Rect::new(x, y + h - footer_height, w, footer_height);
    OverlayLayout { outer, title_bar, content, footer }
}

/// Layout for a two-panel split (e.g., list on left, detail on right).
pub struct TwoPanelLayout {
    pub left: Rect,
    pub right: Rect,
}

/// Split an area into two panels.
/// `left_pct` is 0-100, the percentage of width for the left panel.
pub fn two_panel_split(area: Rect, left_pct: u16) -> TwoPanelLayout {
    let left_w = (area.width as u32 * left_pct as u32 / 100) as u16;
    let right_w = area.width.saturating_sub(left_w);
    TwoPanelLayout {
        left: Rect::new(area.x, area.y, left_w, area.height),
        right: Rect::new(area.x + left_w, area.y, right_w, area.height),
    }
}
```

**Step 3: Run tests**

Run: `cargo build 2>&1 | head -20`
Expected: builds successfully

**Step 4: Commit**

```bash
git add src/ui/overlay_layout.rs
git commit -m "refactor(ui): add shared overlay layout helpers"
```

---

## Phase 2: GameState Decomposition (Blocking)

### Task 5: Add Sub-Struct Fields to GameState (dev-1)

**Files:**
- Modify: `src/core/game_state.rs`
- Modify: `src/core/mod.rs`

**Goal:** Add the 4 sub-struct fields to GameState alongside the existing flat fields. This is the additive step — nothing is removed yet.

**Step 1: Register the new modules in core/mod.rs**

Read `src/core/mod.rs` and add:
```rust
pub mod player_identity;
pub mod combat_context;
pub mod progression_state;
pub mod session_state;
pub mod tick_context;
```

**Step 2: Add composed fields to GameState**

In `src/core/game_state.rs`, add after the existing fields (inside the struct):
```rust
    // === Composed sub-structs (Phase 2 refactoring) ===
    // These group existing fields for clearer module boundaries.
    // During migration, both flat fields and sub-struct fields exist.
    #[serde(skip)]
    pub player: Option<()>,  // placeholder — will be populated in Task 7
    #[serde(skip)]
    pub combat_ctx: Option<()>,
    #[serde(skip)]
    pub prog: Option<()>,
    #[serde(skip)]
    pub sess: Option<()>,
```

Note: These are initially placeholders. The actual migration happens in Tasks 7-8.

**Step 3: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

**Step 4: Commit**

```bash
git add src/core/mod.rs src/core/game_state.rs
git commit -m "refactor(core): register sub-struct modules and add placeholder fields"
```

---

### Task 6: Implement Custom Serde for Flat JSON Compatibility (dev-1)

**Files:**
- Create: `src/core/game_state_serde.rs`
- Modify: `src/core/game_state.rs`

**Goal:** Write a helper module that serializes/deserializes GameState as a flat JSON object (preserving the current save format) even after the struct is decomposed.

**Step 1: Write the serde helper**

Create `src/core/game_state_serde.rs`:

```rust
//! Custom serde implementation for GameState that maintains flat JSON format
//! after the struct is decomposed into sub-structs.
//!
//! This module is used during the migration period where GameState fields
//! are being moved into sub-structs. It ensures saves remain compatible.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;

/// Flat JSON representation matching the original GameState format.
/// Used as an intermediate for serialization/deserialization.
#[derive(Serialize, Deserialize)]
pub(crate) struct FlatGameState {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: crate::character::attributes::Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub combat_state: crate::combat::types::CombatState,
    pub equipment: crate::items::equipment::Equipment,
    #[serde(default)]
    pub active_dungeon: Option<crate::dungeon::types::Dungeon>,
    #[serde(default)]
    pub fishing: crate::fishing::types::FishingState,
    #[serde(default)]
    pub zone_progression: crate::zones::ZoneProgression,
    #[serde(default)]
    pub stormglass: u64,
    #[serde(default)]
    pub stormglass_discovered: bool,
    #[serde(default)]
    pub storm_sigils: crate::stormglass::sigils::StormSigils,
}
```

**Step 2: Write a round-trip test**

Add at the bottom of `game_state_serde.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_game_state_round_trip() {
        // Create a FlatGameState, serialize to JSON, deserialize back
        // Verify all fields survive the round trip
        let json = r#"{"character_id":"test","character_name":"Hero","character_level":5,"character_xp":1000,"attributes":{"strength":12,"dexterity":10,"constitution":14,"intelligence":10,"wisdom":10,"charisma":10},"prestige_rank":2,"total_prestige_count":3,"last_save_time":1000000,"play_time_seconds":3600,"combat_state":{"player_hp":100,"player_max_hp":100,"current_enemy":null,"combat_log":[],"damage_flash_timer":0.0,"player_attack_timer":0.0,"regen_timer":null,"boss_enrage_timer":null,"is_retreating":false},"equipment":{"weapon":null,"armor":null,"helmet":null,"gloves":null,"boots":null,"amulet":null,"ring":null}}"#;
        let flat: FlatGameState = serde_json::from_str(json).unwrap();
        assert_eq!(flat.character_name, "Hero");
        assert_eq!(flat.character_level, 5);
        assert_eq!(flat.prestige_rank, 2);
        let re_json = serde_json::to_string(&flat).unwrap();
        let flat2: FlatGameState = serde_json::from_str(&re_json).unwrap();
        assert_eq!(flat2.character_name, flat.character_name);
        assert_eq!(flat2.character_level, flat.character_level);
    }
}
```

**Step 3: Run the test**

Run: `cargo test game_state_serde 2>&1 | tail -10`
Expected: test passes

**Step 4: Commit**

```bash
git add src/core/game_state_serde.rs
git commit -m "refactor(core): add FlatGameState serde helper for save format compatibility"
```

---

### Task 7: Add Accessor Methods to GameState (dev-1)

**Files:**
- Modify: `src/core/game_state.rs`

**Goal:** Add convenience accessor methods on GameState that will later delegate to sub-structs. For now they just return the flat fields directly. This establishes the API surface that callers will eventually migrate to.

**Step 1: Add grouped accessor methods**

Add a new `impl GameState` block in `game_state.rs`:

```rust
/// Grouped accessors — these provide the same data organized by domain.
/// New code should prefer these over direct field access.
impl GameState {
    // --- Player Identity ---
    pub fn player_level(&self) -> u32 { self.character_level }
    pub fn player_xp(&self) -> u64 { self.character_xp }
    pub fn player_name(&self) -> &str { &self.character_name }
    pub fn player_prestige_rank(&self) -> u32 { self.prestige_rank }

    // --- Combat Context ---
    pub fn current_zone_id(&self) -> u32 { self.zone_progression.current_zone_id() }
}
```

Note: Keep this minimal. Only add accessors that provide meaningful abstraction (e.g., `current_zone_id()` hides the zone_progression nesting). Don't add trivial wrappers for every field — that's busywork.

**Step 2: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

**Step 3: Commit**

```bash
git add src/core/game_state.rs
git commit -m "refactor(core): add grouped accessor methods on GameState"
```

---

### Task 8: Implement TickContext (dev-1)

**Files:**
- Modify: `src/core/tick.rs`
- Modify: `src/core/tick_context.rs`

**Goal:** Add a `game_tick_with_context()` function that takes `TickContext` and delegates to the existing `game_tick()`. Then swap the callers.

**Step 1: Write game_tick_with_context**

In `src/core/tick.rs`, add:
```rust
use super::tick_context::TickContext;

/// New entry point using TickContext. Delegates to the existing game_tick().
pub fn game_tick_with_context<R: Rng>(ctx: &mut TickContext, rng: &mut R) -> TickResult {
    game_tick(
        ctx.state,
        ctx.tick_counter,
        ctx.haven,
        ctx.enhancement,
        ctx.deep,
        ctx.achievements,
        ctx.debug_mode,
        rng,
    )
}
```

**Step 2: Write a test for the new entry point**

Add to the tests in `tick.rs`:
```rust
#[test]
fn test_game_tick_with_context_matches_direct_call() {
    // Create identical game states and verify both paths produce same result
    let mut state1 = GameState::new("Test1".to_string(), 0);
    let mut state2 = state1.clone();
    let mut tc1: u32 = 0;
    let mut tc2: u32 = 0;
    let mut haven1 = Haven::default();
    let mut haven2 = haven1.clone();
    let mut enh1 = EnhancementProgress::default();
    let mut enh2 = enh1.clone();
    let mut deep1 = DeepState::default();
    let mut deep2 = deep1.clone();
    let mut ach1 = Achievements::default();
    let mut ach2 = ach1.clone();
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    let result1 = game_tick(&mut state1, &mut tc1, &mut haven1, &mut enh1, &mut deep1, &mut ach1, false, &mut rng1);

    let mut ctx = TickContext {
        state: &mut state2,
        tick_counter: &mut tc2,
        haven: &mut haven2,
        enhancement: &mut enh2,
        deep: &mut deep2,
        achievements: &mut ach2,
        debug_mode: false,
    };
    let result2 = game_tick_with_context(&mut ctx, &mut rng2);

    assert_eq!(result1.events.len(), result2.events.len());
}
```

**Step 3: Run the test**

Run: `cargo test test_game_tick_with_context 2>&1 | tail -10`
Expected: PASS

**Step 4: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

**Step 5: Commit**

```bash
git add src/core/tick.rs src/core/tick_context.rs
git commit -m "refactor(core): add game_tick_with_context() using TickContext"
```

---

### Task 9: Extract Tick Stages into Named Functions (dev-1)

**Files:**
- Modify: `src/core/tick.rs`
- Modify: `src/core/tick_stages.rs`

**Goal:** Each of the 14 inline stages in `game_tick()` becomes a named function. The body of `game_tick()` becomes a clean sequence of function calls.

**Step 1: Read the current game_tick() body**

Read `src/core/tick.rs` lines 53-335 to identify all 14 stages and their boundaries.

**Step 2: Extract stages that are still inline**

Some stages are already in `tick_stages.rs`. For each remaining inline block:
1. Cut the code from `game_tick()`
2. Create a function in `tick_stages.rs` with signature `fn tick_stage_name(state: &mut GameState, ..., result: &mut TickResult, rng: &mut R)`
3. Replace the inline block with a function call

Target: `game_tick()` body should read as ~20 lines of sequential function calls.

**Step 3: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass — behavior identical

**Step 4: Run the game to verify visually**

Run: `cargo run -- --debug` briefly to confirm the game runs normally.

**Step 5: Commit**

```bash
git add src/core/tick.rs src/core/tick_stages.rs
git commit -m "refactor(core): extract all tick stages into named functions"
```

---

## Phase 3: Parallel Workstreams

### Task 10: Fishing Facade (dev-2)

**Files:**
- Modify: `src/fishing/facade.rs`
- Modify: `src/fishing/mod.rs`

**Goal:** Implement the fishing facade function that delegates to existing `tick_fishing_with_haven_result()`.

**Step 1: Implement the facade**

In `src/fishing/facade.rs`, add:
```rust
use crate::fishing::logic::{tick_fishing_with_haven_result, FishingTickResult, HavenFishingBonuses};
use crate::fishing::types::{FishingSession, FishingState};
use rand::Rng;

pub struct FishingInput<'a> {
    pub fishing: &'a mut FishingState,
    pub active_fishing: &'a mut Option<FishingSession>,
    pub player_level: u32,
    pub prestige_rank: u32,
    pub haven_bonuses: HavenFishingBonuses,
}

/// Facade: tick the fishing system with explicit inputs.
/// Delegates to the existing tick_fishing_with_haven_result() internally.
pub fn tick_fishing_facade<R: Rng>(
    input: &mut FishingInput,
    _rng: &mut R,
) -> FishingTickResult {
    // For now, delegate to existing function.
    // The existing function accesses GameState directly — this facade
    // will be the migration target when we decouple fishing from GameState.
    todo!("Wire to existing tick_fishing_with_haven_result")
}
```

Note: The actual wiring depends on how `tick_fishing_with_haven_result` currently takes its parameters. Read the function signature in `src/fishing/logic.rs` and adapt accordingly.

**Step 2: Re-export from mod.rs**

In `src/fishing/mod.rs`, add:
```rust
pub mod facade;
```

**Step 3: Run tests**

Run: `cargo build 2>&1 | head -20`
Expected: builds (the todo!() is fine since nothing calls it yet)

**Step 4: Commit**

```bash
git add src/fishing/facade.rs src/fishing/mod.rs
git commit -m "refactor(fishing): add facade module with FishingInput struct"
```

---

### Task 11: Dungeon Facade (dev-2)

**Files:**
- Modify: `src/dungeon/facade.rs`
- Modify: `src/dungeon/mod.rs`

**Goal:** Same pattern as Task 10 for the dungeon system.

**Step 1: Implement the facade**

Read `src/dungeon/logic.rs` to understand the current `update_dungeon()` signature, then create a facade in `src/dungeon/facade.rs` with `DungeonInput` and a delegation function.

**Step 2: Re-export from mod.rs**

**Step 3: Run build, commit**

```bash
git commit -m "refactor(dungeon): add facade module with DungeonInput struct"
```

---

### Task 12: Combat Facade (dev-2)

**Files:**
- Modify: `src/combat/facade.rs`
- Modify: `src/combat/mod.rs`

**Goal:** Same pattern for the combat system. Note: combat's `update_combat()` already takes fairly explicit parameters (CombatBonuses, DerivedStats), so this facade may be thinner.

**Step 1: Read `src/combat/orchestration.rs` to understand current signature**

**Step 2: Create CombatInput and facade function**

**Step 3: Run build, commit**

```bash
git commit -m "refactor(combat): add facade module with CombatInput struct"
```

---

### Task 13: Discovery Facade (dev-2)

**Files:**
- Create: `src/core/discovery_facade.rs`

**Goal:** Create a DiscoveryInput struct and facade for the discovery rolls (dungeon, fishing spot, haven, soulforge).

**Step 1: Read `src/core/discoveries.rs` to understand current functions**

**Step 2: Create DiscoveryInput and facade**

```rust
pub struct DiscoveryInput {
    pub prestige_rank: u32,
    pub character_level: u32,
    pub current_zone_id: u32,
    pub has_active_dungeon: bool,
    pub has_active_fishing: bool,
    pub has_active_minigame: bool,
}
```

**Step 3: Run build, commit**

```bash
git commit -m "refactor(core): add discovery facade with DiscoveryInput struct"
```

---

### Task 14: Challenge AI Facade (dev-2)

**Files:**
- Create: `src/challenges/facade.rs`

**Goal:** Create a thin facade for `tick_challenge_ai()` that only needs the ActiveMinigame.

**Step 1: Read the AI tick functions across challenge modules**

**Step 2: Create facade**

**Step 3: Run build, commit**

```bash
git commit -m "refactor(challenges): add facade for challenge AI ticking"
```

---

### Task 15: Deep Facade (dev-2)

**Files:**
- Create: `src/deep/facade.rs`

**Goal:** Create DeepInput and facade for Deep tick processing.

**Step 1: Read `src/deep/missions.rs` for tick-relevant functions**

**Step 2: Create facade**

**Step 3: Run build, commit**

```bash
git commit -m "refactor(deep): add facade module with DeepInput struct"
```

---

### Task 16: Implement Shared Forfeit Handler (dev-3)

**Files:**
- Modify: `src/challenges/mod.rs`

**Goal:** The forfeit handler functions were defined in Task 3. This task adds unit tests.

**Step 1: Write tests for handle_forfeit**

Add to `src/challenges/mod.rs` tests:
```rust
#[cfg(test)]
mod forfeit_tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestResult { Win, Loss }

    #[test]
    fn test_handle_forfeit_first_press_sets_pending() {
        let mut result: Option<TestResult> = None;
        let mut pending = false;
        let confirmed = handle_forfeit(&mut result, &mut pending, TestResult::Loss);
        assert!(!confirmed);
        assert!(pending);
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_forfeit_second_press_confirms() {
        let mut result: Option<TestResult> = None;
        let mut pending = true;
        let confirmed = handle_forfeit(&mut result, &mut pending, TestResult::Loss);
        assert!(confirmed);
        assert_eq!(result, Some(TestResult::Loss));
    }

    #[test]
    fn test_cancel_forfeit_clears_pending() {
        let mut pending = true;
        cancel_forfeit_if_pending(&mut pending);
        assert!(!pending);
    }

    #[test]
    fn test_cancel_forfeit_noop_when_not_pending() {
        let mut pending = false;
        cancel_forfeit_if_pending(&mut pending);
        assert!(!pending);
    }
}
```

**Step 2: Run the tests**

Run: `cargo test forfeit_tests 2>&1 | tail -10`
Expected: all 4 tests pass

**Step 3: Commit**

```bash
git add src/challenges/mod.rs
git commit -m "test(challenges): add unit tests for shared forfeit handler"
```

---

### Task 17: Migrate Flappy Bird to Shared Forfeit Handler (dev-3)

**Files:**
- Modify: `src/challenges/flappy/logic.rs`

**Goal:** Replace inline forfeit logic with calls to `handle_forfeit()` and `cancel_forfeit_if_pending()`. This is the first migration — validates the pattern.

**Step 1: Read `src/challenges/flappy/logic.rs` lines 20-56**

**Step 2: Replace forfeit handling**

```rust
// BEFORE (inline):
FlappyBirdInput::Forfeit => {
    if game.forfeit_pending {
        game.game_result = Some(FlappyBirdResult::Loss);
    } else {
        game.forfeit_pending = true;
    }
}

// AFTER (shared):
FlappyBirdInput::Forfeit => {
    crate::challenges::handle_forfeit(
        &mut game.game_result,
        &mut game.forfeit_pending,
        FlappyBirdResult::Loss,
    );
}
```

And for cancel:
```rust
// BEFORE:
if game.forfeit_pending { game.forfeit_pending = false; }

// AFTER:
crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
```

**Step 3: Run flappy tests**

Run: `cargo test flappy 2>&1 | tail -10`
Expected: all flappy tests pass

**Step 4: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

**Step 5: Commit**

```bash
git add src/challenges/flappy/logic.rs
git commit -m "refactor(challenges): migrate flappy bird to shared forfeit handler"
```

---

### Task 18: Migrate Remaining 9 Challenges to Shared Forfeit Handler (dev-3)

**Files:**
- Modify: `src/challenges/chess/logic.rs`
- Modify: `src/challenges/go/logic.rs`
- Modify: `src/challenges/morris/logic.rs`
- Modify: `src/challenges/gomoku/logic.rs`
- Modify: `src/challenges/minesweeper/logic.rs`
- Modify: `src/challenges/rune/logic.rs`
- Modify: `src/challenges/snake/logic.rs`
- Modify: `src/challenges/jezzball/logic.rs`
- Modify: `src/challenges/runic_shift/logic.rs`

**Goal:** Apply the same pattern from Task 17 to all remaining challenges.

**Important notes:**
- Morris has a special case: Esc first clears selection, then triggers forfeit. Read `process_cancel()` in morris/logic.rs carefully — the shared handler only applies to the pure forfeit case, not the selection-clearing.
- Some games (Gomoku, Go) check `ai_thinking` before forfeit — keep that guard, only replace the forfeit body.

**Step 1: Migrate each challenge one at a time**

For each file:
1. Read the current forfeit handling
2. Replace with `handle_forfeit()` / `cancel_forfeit_if_pending()`
3. Run `cargo test <game_name>` to verify
4. Move to next

**Step 2: Run all challenge tests**

Run: `cargo test challenges 2>&1 | tail -10`
Expected: all challenge tests pass

**Step 3: Run full test suite**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

**Step 4: Commit**

```bash
git add src/challenges/*/logic.rs
git commit -m "refactor(challenges): migrate all 9 remaining challenges to shared forfeit handler"
```

---

### Task 19: Register UI Layout Module (dev-4)

**Files:**
- Modify: `src/ui/mod.rs`

**Goal:** Register the `overlay_layout` module created in Task 4.

**Step 1: Add module declaration**

In `src/ui/mod.rs`, add:
```rust
pub mod overlay_layout;
```

**Step 2: Run build**

Run: `cargo build 2>&1 | head -20`
Expected: builds successfully

**Step 3: Commit**

```bash
git add src/ui/mod.rs
git commit -m "refactor(ui): register overlay_layout module"
```

---

### Task 20: Decompose deep_roster.rs render_roster_split (dev-4)

**Files:**
- Modify: `src/ui/deep_roster.rs`

**Goal:** Extract the merc detail panel (~280 lines, lines 433-712) from `render_roster_split()` into a separate `render_merc_detail_panel()` function.

**Step 1: Read `src/ui/deep_roster.rs` lines 343-712**

Identify the natural boundary between list rendering (343-430) and detail rendering (433-712).

**Step 2: Extract render_merc_detail_panel()**

Cut lines 433-712 into a new private function:
```rust
fn render_merc_detail_panel(
    buffer: &mut [Vec<SceneCell>],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    merc: &Mercenary,
    // ... other needed params
) {
    // ... extracted detail rendering code
}
```

Replace the inline block in `render_roster_split()` with a call to the new function.

**Step 3: Run build**

Run: `cargo build 2>&1 | head -20`
Expected: builds successfully (UI code has no dedicated tests but compilation verifies correctness)

**Step 4: Run full test suite**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

**Step 5: Commit**

```bash
git add src/ui/deep_roster.rs
git commit -m "refactor(ui): extract render_merc_detail_panel from deep_roster"
```

---

### Task 21: Decompose deep_missions.rs render_new_mission_split (dev-4)

**Files:**
- Modify: `src/ui/deep_missions.rs`

**Goal:** Extract 3-4 region helpers from `render_new_mission_split()` (~282 lines):
- `render_mission_type_selector()` — Mission type horizontal menu
- `render_layer_selector()` — Layer tier vertical list
- `render_squad_assembly()` — Merc selection panel
- `render_mission_preview()` — Duration/cost/risk summary

**Step 1: Read `src/ui/deep_missions.rs` lines 1821-2103**

Identify region boundaries.

**Step 2: Extract each region into a helper function**

Each helper takes the buffer, coordinates, and the specific data it needs.

**Step 3: Run build and full test suite**

**Step 4: Commit**

```bash
git add src/ui/deep_missions.rs
git commit -m "refactor(ui): decompose deep_missions render_new_mission_split into region helpers"
```

---

### Task 22: Decompose stormglass_scene.rs render_exchange_menu (dev-4)

**Files:**
- Modify: `src/ui/stormglass_scene.rs`

**Goal:** Extract region helpers from `render_exchange_menu()` (~191 lines):
- `render_menu_items()` — 4 menu items with cursor/affordability
- `render_overlay_text_layers()` — Flavor text and description panels

**Step 1: Read `src/ui/stormglass_scene.rs` lines 410-601**

**Step 2: Extract helpers**

**Step 3: Run build and full test suite**

**Step 4: Commit**

```bash
git add src/ui/stormglass_scene.rs
git commit -m "refactor(ui): decompose stormglass render_exchange_menu into region helpers"
```

---

## Phase 4: Validation

### Task 23: Core + Tick Engine Test Validation (qa-1)

**Goal:** Verify all core and tick-related tests pass after the refactoring.

**Step 1: Run core module tests**

Run: `cargo test core:: 2>&1`
Expected: all pass

**Step 2: Run tick integration tests**

Run: `cargo test --test tick_integration_test 2>&1`
Expected: all pass

**Step 3: Run game loop orchestration tests**

Run: `cargo test --test game_loop_orchestration_test 2>&1`
Expected: all 36 behavior-locking tests pass

**Step 4: Verify TickContext equivalence**

Run: `cargo test test_game_tick_with_context 2>&1`
Expected: pass

**Step 5: Document results**

Record pass/fail counts for each test group.

---

### Task 24: Serde Round-Trip Validation (qa-1)

**Goal:** Verify save file compatibility is preserved.

**Step 1: Run serialization tests**

Run: `cargo test serialization 2>&1`
Expected: all round-trip tests pass

**Step 2: Run character persistence tests**

Run: `cargo test persistence 2>&1`
Expected: all pass

**Step 3: Manual save/load test**

Run the game, create a character, play briefly, quit. Relaunch and verify the character loads correctly.

---

### Task 25: Combat + Zone + Dungeon + Fishing Test Validation (qa-2)

**Goal:** Verify all combat-related systems work correctly.

**Step 1: Run combat tests**

Run: `cargo test combat 2>&1`
Expected: all pass

**Step 2: Run zone progression tests**

Run: `cargo test --test zone_progression_test 2>&1`
Expected: all pass

**Step 3: Run dungeon tests**

Run: `cargo test dungeon 2>&1`
Expected: all pass

**Step 4: Run fishing tests**

Run: `cargo test fishing 2>&1`
Expected: all pass

---

### Task 26: Challenge Minigame Test Validation (qa-3)

**Goal:** Verify all challenge tests pass after forfeit handler migration.

**Step 1: Run all challenge tests**

Run: `cargo test challenges 2>&1`
Expected: all pass

**Step 2: Run individual challenge test suites**

For each of the 10 challenges:
Run: `cargo test chess && cargo test morris && cargo test gomoku && cargo test minesweeper && cargo test rune && cargo test go:: && cargo test snake && cargo test flappy && cargo test jezzball && cargo test runic_shift`
Expected: all pass

**Step 3: Verify forfeit handler tests**

Run: `cargo test forfeit 2>&1`
Expected: all 4 unit tests pass

---

### Task 27: Full Integration Validation (qa-4)

**Goal:** Run the complete CI check suite — the final quality gate.

**Step 1: Run make check (identical to CI)**

Run: `make check 2>&1`
Expected: all 5 checks pass:
1. Format check (`cargo fmt --check`)
2. Clippy lint (`cargo clippy --all-targets -- -D warnings`)
3. All tests (`cargo test`)
4. Build verification (`cargo build --all-targets`)
5. Security audit (`cargo audit --deny yanked`)

**Step 2: Run the simulator**

Run: `cargo run --release --bin simulator -- --ticks 3600 --seed 42 --prestige 10 --runs 1`
Expected: completes without error, output matches expected format

**Step 3: Run the deep simulator**

Run: `cargo run --release --bin deep_simulator -- --hours 24 --seed 42 --strategy balanced`
Expected: completes without error

**Step 4: Brief manual play test**

Run: `cargo run -- --debug` and verify:
- Combat works normally
- Debug menu opens/closes
- Challenge discovery works
- No visual glitches

---

## Phase 5: Final Audit

### Task 28: Gameplay Invariant Audit (game-designer)

**Goal:** Verify that no gameplay mechanics were altered by the refactoring.

**Step 1: Verify combat damage pipeline**

Read `src/combat/player_attack.rs` and `src/combat/enemy_attack.rs` — confirm the damage formula order is: base → Giant's Might % → Haven Armory % → prestige flat → enemy defense → min 1 → Divine Bulwark DR → crit (2x)

**Step 2: Verify XP formulas**

Read `src/core/xp.rs` — confirm:
- `xp_for_next_level(level) = 100 * level^1.5`
- Kill XP: 200-400 ticks
- Offline XP: 25% rate, max 7 days

**Step 3: Verify prestige multipliers**

Read `src/character/prestige.rs` — confirm `1 + 0.5 * rank^0.7` formula

**Step 4: Verify challenge rewards**

Read `src/challenges/mod.rs` `apply_challenge_rewards()` — confirm Stormglass + prestige rank + fishing rank rewards are unchanged

**Step 5: Verify item drop rates**

Read `src/core/constants.rs` — confirm:
- Mob: 15% base + 1% per prestige (cap 25%), max Epic
- Boss: guaranteed, 2% Legendary (5% Zone 10 final boss)

**Step 6: Run simulator comparison**

Run the simulator with identical seeds before and after refactoring:
```bash
cargo run --release --bin simulator -- --ticks 36000 --seed 42 --prestige 10 --runs 3
```
Compare output to pre-refactoring baseline. All numbers should be identical (same RNG seeds, same logic).

**Step 7: Document audit results**

Write findings to `docs/plans/2026-02-27-structural-overhaul-audit.md`.
