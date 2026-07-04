> Backported design record. Sources: docs/plans/2026-02-28-scaffold-wiring-design.md.

## 2026-02-28-scaffold-wiring-design.md

# PR #424 Scaffold Wiring — Design Document

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Complete the structural overhaul scaffold from PR #424 by wiring all 6 facades, switching main.rs to TickContext, populating sub-structs with real types, and adding custom serde for backward-compatible saves.

**Architecture:** Wire-and-Delegate — facades call lower-level internal functions (not `&mut GameState` wrappers), TickContext becomes the primary game_tick entry point, sub-structs coexist alongside flat fields, custom serde via FlatGameState preserves JSON format.

**Risk model:** Conservative. All 5,704+ tests must pass after every change. JSON save format preserved exactly. Gameplay invariants verified via simulator.

---

## 1. Facade Wiring

Each of the 6 facade files gets its input struct expanded to include all missing fields, the `todo!()` body replaced with a delegation to lower-level internal functions.

### 1a. Challenge AI Facade (`src/challenges/facade.rs`)

**Current state:** Takes `&mut Option<ActiveMinigame>`, returns `Option<()>`.

**Changes:**
- Add generic `<R: Rng>` parameter and `rng: &mut R` argument
- Body: replicate the 4-arm match dispatch from `tick_stages::tick_challenge_ai`, calling each game's `process_ai_thinking(game, rng)` directly
- Return type: `()` (no meaningful return)

**Target internals:** `chess::logic::process_ai_thinking`, `morris::logic::process_ai_thinking`, `gomoku::logic::process_ai_thinking`, `go::process_ai_thinking` — all are `pub` and take `(game: &mut XxxGame, rng: &mut R)`.

### 1b. Fishing Facade (`src/fishing/facade.rs`)

**Current state:** `FishingInput` with 7 fields, returns `Option<()>`.

**Expand input struct:**
- Add `god_item_fishing_reduction_percent: f64` (Sleipnir speed reduction)
- Remove `storm_lure_active: bool` (already accessible via `fishing.storm_lure_active`)
- Add `character_name: &'a str` (for achievement attribution if needed downstream)

**Changes:**
- Add generic `<R: Rng>` (already present)
- Return type: `FishingTickResult`
- Body: call the internal fishing tick logic. The current `tick_fishing_with_haven_result` in `logic.rs` takes `&mut GameState` — the facade must replicate its orchestration using the decomposed input fields (take active_fishing, tick phases, apply catches, put back)

**Internal functions called:**
- `generation::roll_fish_rarity(rank, rng)`, `generation::generate_fish_with_rank(rarity, rank, leviathan_encounters, rng)`
- `drops::try_fishing_item_drop(rarity, zone_id, rng)`
- `rank::check_rank_up_with_max(fishing_state, max_rank)`
- Phase transition timing via `generation::roll_casting_ticks`, `roll_waiting_ticks`, `roll_reeling_ticks`

### 1c. Combat Facade (`src/combat/facade.rs`)

**Current state:** `CombatInput` with 4 fields, returns `Option<()>`.

**Expand input struct:**
- Add `zone_progression: &'a mut ZoneProgression`
- Add `equipment: &'a Equipment`
- Add `consecutive_deaths: &'a mut u32`
- Add `delta_time: f64`
- Add `achievements: &'a mut Achievements`
- Remove `prestige_rank: u32` (redundant — bonuses already carry prestige data)

**Changes:**
- Add generic `<R: Rng>` parameter
- Return type: `Vec<CombatEvent>`
- Body: call `orchestration::update_combat_inner()` or replicate the combat orchestration using the decomposed inputs

**Key challenge:** `update_combat` in `orchestration.rs` currently takes `&mut GameState`. The facade must call the same player_attack/enemy_attack/regen sub-functions directly, or a new `update_combat_decomposed()` function must be extracted that takes the individual references.

**Preferred approach:** Extract an `update_combat_core()` in `orchestration.rs` that takes the decomposed fields. The existing `update_combat(&mut GameState, ...)` becomes a thin wrapper calling `update_combat_core()` with fields extracted from GameState. The facade also calls `update_combat_core()`. This avoids reimplementation while achieving decoupling.

### 1d. Dungeon Facade (`src/dungeon/facade.rs`)

**Current state:** `DungeonInput` with 4 fields, returns `Option<()>`.

**Expand input struct:**
- Add `combat_state: &'a CombatState` (to check if room is cleared)
- Add `delta_time: f64`
- Add `god_item_dungeon_speed_percent: f64` (Sleipnir dungeon speed)

**Changes:**
- Return type: `Vec<DungeonEvent>` (or similar)
- Body: call `logic::update_dungeon_core()` — same extraction pattern as combat

### 1e. Discovery Facade (`src/core/discovery_facade.rs`)

**Current state:** `DiscoveryInput` with 6 read-only fields, returns `Option<()>`.

**Expand input struct:**
- Add `active_dungeon: &'a mut Option<Dungeon>` (dungeon discovery writes here)
- Add `rng: &mut R` as generic parameter

**Changes:**
- Return type: `DiscoveryResult` struct with `dungeon_discovered: bool`, `fishing_spot_discovered: bool`
- Body: call `discoveries::try_discover_dungeon(rng, ...)` with decomposed inputs

### 1f. Deep Facade (`src/deep/facade.rs`)

**Current state:** `DeepInput` with 2 fields, returns `Option<()>`.

**Expand input struct:**
- Add `character_name: &'a str` (for achievement attribution)
- Add `achievements: &'a mut Achievements`
- Add `debug_mode: bool`
- Remove `prestige_rank: u32` (redundant — DeepState carries this)

**Changes:**
- Add generic `<R: Rng>` parameter
- Return type: `DeepTickResult` struct (missions completed, events, marks, etc.)
- Body: call `missions::tick_all_missions()` and `missions::maybe_refresh_mission_pool()` directly

---

## 2. TickContext Switch

### 2a. Switch main.rs call sites

3 call sites in `main.rs` currently call `game_tick(state, tick_counter, haven, enhancement, deep, achievements, debug_mode, rng)`.

Replace each with:
```rust
let mut ctx = TickContext {
    state: &mut state,
    tick_counter: &mut tick_counter,
    haven: &mut haven,
    enhancement: &mut enhancement,
    deep: &mut deep_state,
    achievements: &mut global_achievements,
    debug_mode,
};
let tick_result = game_tick_with_context(&mut ctx, &mut rng);
```

### 2b. Deprecate old entry point

- Remove `#[allow(dead_code)]` from `game_tick_with_context`
- Add `#[deprecated(note = "Use game_tick_with_context instead")]` to `game_tick()`
- Keep `game_tick()` callable — tests and simulator use it directly

---

## 3. Sub-Struct Population

### 3a. Replace placeholder types

In `game_state.rs`, change:
```rust
pub player: Option<()>      →  pub player: PlayerIdentity
pub combat_ctx: Option<()>  →  pub combat_ctx: CombatContext
pub prog: Option<()>        →  pub prog: ProgressionState
pub sess: Option<()>        →  pub sess: SessionState
```

Remove `#[serde(skip)]` from these fields — they are now part of the struct, but serde is handled by custom impl (Section 4).

### 3b. Populate in GameState::new()

After setting flat fields, populate sub-structs from the same values:
```rust
let player = PlayerIdentity {
    character_id: character_id.clone(),
    character_name: character_name.clone(),
    // ... etc
};
```

### 3c. Populate in load_character()

In `persistence.rs`, after constructing the GameState from deserialized flat fields, populate sub-structs from those same fields.

### 3d. Add sync_sub_structs() method

A `pub fn sync_sub_structs(&mut self)` method on GameState that copies flat field values into sub-struct fields. Called after any mutation that changes flat fields (e.g., level-up, prestige, equip item). This ensures sub-structs stay in sync during the migration period.

### 3e. Update construction sites

Two production struct literal sites need updating:
- `src/character/persistence.rs` line 58 (`load_character`)
- `src/character/manager.rs` line 96 (test helper `make_test_state`)

Plus `GameState::new()` itself.

---

## 4. Custom Serde via FlatGameState

### 4a. Remove derived Serialize/Deserialize

Replace `#[derive(Debug, Clone, Serialize, Deserialize)]` on GameState with just `#[derive(Debug, Clone)]`.

### 4b. Implement From conversions

```rust
impl From<&GameState> for FlatGameState { ... }  // flatten sub-structs to flat fields
impl FlatGameState {
    fn into_game_state(self) -> GameState { ... } // inflate flat to both flat + sub-structs
}
```

### 4c. Manual Serialize/Deserialize impls

```rust
impl Serialize for GameState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        FlatGameState::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let flat = FlatGameState::deserialize(deserializer)?;
        Ok(flat.into_game_state())
    }
}
```

### 4d. Round-trip test

Existing `test_flat_game_state_round_trip` validates FlatGameState alone. Add new test:
- Serialize a fully-populated `GameState` to JSON
- Deserialize back to `GameState`
- Verify all flat fields AND sub-struct fields match

---

## 5. Accessor Methods

Add ~15 forwarding methods on GameState. These delegate to sub-struct fields and establish the future migration path.

**PlayerIdentity accessors:** `player_id()`, `player_attributes()`, `player_attributes_mut()`, `total_prestige_count()`

**CombatContext accessors:** `is_fighting()`, `is_regenerating()`, `current_subzone_id()`

**ProgressionState accessors:** `is_fishing()`, `fishing_rank()`, `stormglass_balance()`, `has_active_minigame()`

**SessionState accessors:** `save_time()`, `play_time()`

All marked `#[allow(dead_code)]` during migration — they establish the API surface for future callers.

---

## 6. Team Allocation

| Role | Count | Assignment |
|------|-------|------------|
| sys-arch-1 | 1 | Design facade input struct expansions + _core() function extractions |
| sys-arch-2 | 1 | Design custom serde + sub-struct sync strategy |
| dev-1 | 1 | Challenge AI facade + Deep facade wiring |
| dev-2 | 1 | Fishing facade wiring (most complex — full orchestration) |
| dev-3 | 1 | Combat facade wiring (extract update_combat_core) |
| dev-4 | 1 | Dungeon facade + Discovery facade wiring |
| dev-5 | 1 | TickContext switch + sub-struct population + custom serde |
| qa-1 | 1 | Core module tests + tick engine validation |
| qa-2 | 1 | Combat + dungeon + fishing validation |
| qa-3 | 1 | Challenge system validation |
| qa-4 | 1 | Deep + stormglass validation |
| qa-5 | 1 | Full integration (make check) + serde round-trip |
| eng-mgr-1 | 1 | Phase 1-2 coordination (design + wire) |
| eng-mgr-2 | 1 | Phase 3-5 coordination (populate + validate + polish) |
| game-designer | 1 | Gameplay invariant audit — simulator comparisons |

## 7. Phase Structure

### Phase 1: Design (sys-arch)
- Expand facade input struct definitions
- Design _core() function extraction for combat, dungeon, fishing
- Design custom Serialize/Deserialize impls

### Phase 2: Wire Facades (dev-1 through dev-4, parallel)
- Challenge AI + Deep facades (dev-1)
- Fishing facade with internal orchestration (dev-2)
- Combat facade + extract update_combat_core (dev-3)
- Dungeon + Discovery facades (dev-4)
- QA validates each facade independently

### Phase 3: TickContext + Sub-Structs (dev-5)
- Switch main.rs to game_tick_with_context (3 call sites)
- Replace Option<()> with real sub-struct types
- Add sync_sub_structs() method
- Update construction sites

### Phase 4: Custom Serde (dev-5)
- Remove derived Serialize/Deserialize
- Implement From conversions
- Write manual Serialize/Deserialize impls
- Add comprehensive round-trip tests

### Phase 5: Validation (all QA + game-designer)
- Full test suite (5,704+ tests)
- make check (format, clippy, test, build, audit)
- Simulator comparison (headless + Deep)
- Gameplay invariant audit (damage pipeline, XP, drop rates, prestige)

### Phase 6: Polish (dev)
- Add accessor methods
- Remove #[allow(dead_code)] from wired code
- Deprecate old game_tick() entry point

## 8. Success Criteria

- All 6 facade todo!() panics eliminated — replaced with working delegations
- main.rs uses game_tick_with_context exclusively
- GameState sub-struct fields populated with real types (not Option<()>)
- JSON save format byte-identical for all persisted fields
- All 5,704+ tests pass with 0 regressions
- Simulator output matches pre-refactoring baseline (seed 42, P10, 36k ticks)
- Zero clippy warnings
