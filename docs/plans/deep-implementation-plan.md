# The Deep — Implementation Plan

## Overview

This plan describes the phased implementation of The Deep Mercenary Expedition System for Quest. The Deep is a P15+ endgame feature where players recruit mercenaries and send squads on real-time missions (2-24 hours) that push deeper into an underground structure. It introduces wall-clock time progression — a fundamentally new pattern for Quest.

The plan is organized into 6 phases with clear dependencies, a complete file inventory, risk assessment, minimum viable feature definition, and testing strategy.

---

## Phase 1: Core Types and Data Model

**Blocks everything.** All subsequent phases depend on the type definitions established here.

**Tasks:** #1 (design) -> #8 (implement)

### 1.1 New Files

| File | Contents |
|------|----------|
| `src/deep/mod.rs` | Public re-exports (follows Haven/Enhancement pattern) |
| `src/deep/types.rs` | All core data structures (see below) |
| `src/deep/CLAUDE.md` | Module documentation (follows existing module CLAUDE.md pattern) |

### 1.2 Core Types to Define (`src/deep/types.rs`)

```
// ── Mercenaries ──
MercenaryArchetype       enum (Vanguard, Scout, Arcanist, Medic, Saboteur)
MercenaryStats           struct { power: u32, resilience: u32, expertise: u32 }
MercenaryStatus          enum (Available, OnMission, Injured { recover_at: i64 }, Lost)
Mercenary                struct { id, name, archetype, stats, level, xp, status }

// ── Layers ──
LayerTier                enum (Shallows, Warrens, Hollows, SunkenReach, Abyss, Void)
InfrastructureType       enum (Outpost, SupplyCache, Watchtower, Bridge)
LayerState               struct { layer_num, cleared, familiarity, infrastructure: [Option<InfrastructureType>; 2] }

// ── Missions ──
MissionType              enum (SupplyRun, Recon, Expedition, Breakthrough, Construction)
MissionStatus            enum (Active, PendingEvent, Completed, Failed)
MissionOutcome           enum (FullSuccess, PartialSuccess, Failure)
SquadSlot                struct { mercenary_id: u64 }
EventChoice              struct { label, archetype_bonus, outcome effects }
CheckInEvent             struct { event_id, description, choices, auto_resolve_choice, triggered_at, auto_resolve_at }
PendingEvent             struct { event: CheckInEvent, fired_at_progress: f64 }
Mission                  struct { id, mission_type, layer, squad, started_at: i64, duration_seconds, events, pending_event, status, outcome }

// ── Guild ──
GuildRank                enum/struct (Freelancers..Legion) with max_roster, concurrent_missions, rank requirements

// ── Economy ──
WarbandMarks             u64 (alias or newtype)

// ── Top-Level State ──
DeepState                struct { discovered, guild_rank, marks, roster: Vec<Mercenary>, layers: Vec<LayerState>,
                                  active_missions: Vec<Mission>, completed_missions: Vec<Mission>,
                                  frontier_layer, next_merc_id, recruit_pool, recruit_pool_refreshed_at,
                                  free_daily_used_at, last_tick_time }

// ── Constants ──
DEEP_MIN_PRESTIGE_RANK   = 15
DEEP_DISCOVERY_BASE_CHANCE, DEEP_DISCOVERY_RANK_BONUS (same pattern as Soulforge)
MAX_LAYERS               = 30 (soft cap, Void scales infinitely)
RECRUIT_POOL_SIZE        = 3-5
RECRUIT_REFRESH_HOURS    = 24
AUTO_RESOLVE_TIMEOUT_SECS = 7200 (2 hours)
```

### 1.3 Key Design Decisions

- **Wall-clock time stored as `i64` Unix timestamps** (like `last_save_time` in `GameState`). Avoids `SystemTime` serialization issues.
- **`DeepState` is account-level** (like Haven/Enhancement), persisted to `~/.quest/deep.json`.
- **Mercenary IDs are sequential `u64`** (monotonic counter in `DeepState.next_merc_id`), not UUIDs. Simpler and sufficient.
- **Infrastructure uses a `[Option<InfrastructureType>; 2]` fixed array** per layer — exactly 2 slots, enforced at the type level.
- **Missions store `started_at: i64` and `duration_seconds: u64`** rather than `ends_at`. This allows duration modifiers (Outpost -25%, Saboteur -15%) to be recalculated without stored dependency.
- **`LayerState` uses a `Vec<LayerState>` indexed by layer number (0-based internally, displayed 1-based)**. Only layers that have been reached need entries.

### 1.4 Serde Considerations

- All types derive `Serialize, Deserialize` with `#[serde(default)]` on optional/new fields for backward compatibility.
- `MercenaryStatus::Injured { recover_at: i64 }` stores the wall-clock recovery timestamp.
- Event choices reference archetypes by enum, not string.

---

## Phase 2: Core Logic Modules

**Depends on:** Phase 1 (types)
**Can parallelize internally:** mercenary, mission, layer/economy, and event subsystems are independent.

**Tasks:** #9 (mercenaries), #10 (missions), #11 (layers/economy), #3 (events design)

### 2.1 New Files

| File | Contents |
|------|----------|
| `src/deep/mercenaries.rs` | Merc generation, recruitment, stat scaling, level-up, injury/recovery |
| `src/deep/missions.rs` | Mission generation, ticking, resolution, squad validation |
| `src/deep/events.rs` | Check-in event templates, event resolution, archetype bonuses |
| `src/deep/layers.rs` | Layer progression, infrastructure effects, familiarity, tier queries |
| `src/deep/economy.rs` | Mark earning/spending, guild rank upgrades, cost tables |
| `src/deep/rewards.rs` | Reward generation (XP, items, Stormglass, PR fragments) |

### 2.2 Mercenary System (`mercenaries.rs`)

**Key functions:**
- `generate_mercenary<R: Rng>(rng, archetype, guild_rank, layer_context) -> Mercenary` — stat generation scaled by guild rank
- `generate_recruit_pool<R: Rng>(rng, guild_rank, frontier) -> Vec<Mercenary>` — 3-5 candidates, rarer archetypes at higher ranks
- `recruit(deep: &mut DeepState, index: usize) -> Result<(), RecruitError>` — deducts marks, moves from pool to roster
- `level_up_mercenary(merc: &mut Mercenary)` — XP threshold check, stat growth
- `apply_injury(merc: &mut Mercenary, severity, current_time: i64)` — sets recovery time
- `check_recovery(merc: &mut Mercenary, current_time: i64) -> bool` — transitions Injured -> Available
- `starter_roster<R: Rng>(rng) -> Vec<Mercenary>` — 3 free starter mercs (Vanguard, Scout, Medic)

**Pattern:** All functions take `&mut R where R: Rng` for testability (follows `game_tick()` pattern).

### 2.3 Mission System (`missions.rs`)

**Key functions:**
- `generate_available_missions(deep: &DeepState) -> Vec<MissionTemplate>` — based on frontier, familiarity, guild rank
- `validate_squad(mission: &MissionTemplate, squad: &[&Mercenary]) -> SquadValidation` — checks requirements, computes power rating
- `start_mission(deep: &mut DeepState, template: MissionTemplate, squad: Vec<u64>, current_time: i64) -> Result<Mission, StartError>` — creates mission, marks mercs as OnMission
- `tick_missions(deep: &mut DeepState, current_time: i64) -> Vec<MissionTickResult>` — check all active missions for completion or pending events
- `resolve_mission<R: Rng>(rng, mission: &mut Mission, deep: &mut DeepState) -> MissionOutcome` — final resolution based on squad power, events, familiarity
- `cancel_missions_for_prestige(deep: &mut DeepState, current_time: i64) -> Vec<PartialReward>` — auto-cancel active missions with partial rewards

**Wall-clock ticking model:**
- `tick_missions()` is called from two places:
  1. **On game load** (like offline XP processing) — catches up all missions that completed while offline
  2. **Periodically during play** — every ~10 seconds (100 ticks), check mission progress

### 2.4 Check-In Events (`events.rs`)

**Key functions:**
- `check_event_triggers(mission: &Mission, current_time: i64) -> Option<PendingEvent>` — fires events at 25%/50%/75% progress
- `resolve_event(event: &CheckInEvent, choice_index: usize, squad: &[&Mercenary]) -> EventOutcome` — applies choice effects
- `auto_resolve_pending_events(mission: &mut Mission, current_time: i64) -> Vec<EventOutcome>` — auto-resolve after 2-hour timeout
- `generate_events_for_mission<R: Rng>(rng, mission_type, layer, tier) -> Vec<CheckInEvent>` — 0-5 events per mission

**Event template structure:**
- ~30-50 event templates organized by layer tier (Shallows, Warrens, etc.)
- Each event has 2-4 choices, some gated by archetype
- Events can chain (choice A in event 1 may unlock bonus choice in event 3)

### 2.5 Layer System (`layers.rs`)

**Key functions:**
- `get_layer_tier(layer_num: u32) -> LayerTier` — maps layer number to tier
- `layer_difficulty(layer_num: u32) -> f64` — scaling factor for mission duration/risk/reward
- `familiarity_duration_modifier(familiarity: f64) -> f64` — 0-30% reduction based on 0-100% familiarity
- `infrastructure_effects(layer: &LayerState) -> InfrastructureEffects` — aggregates Outpost/Cache/Watchtower/Bridge bonuses
- `build_infrastructure(deep: &mut DeepState, layer: u32, slot: usize, infra_type: InfrastructureType) -> Result<(), BuildError>` — validates slot availability, deducts marks
- `complete_breakthrough(deep: &mut DeepState, layer: u32)` — marks layer cleared, advances frontier

### 2.6 Economy (`economy.rs`)

**Key functions:**
- `mission_reward_marks(mission_type: MissionType, layer: u32, outcome: MissionOutcome) -> u64` — mark earning table
- `recruit_cost(archetype: MercenaryArchetype, guild_rank: GuildRank) -> u64` — 30-120 marks
- `infrastructure_cost(infra_type: InfrastructureType, layer: u32) -> u64` — 60-150 marks
- `guild_rank_cost(target_rank: GuildRank) -> u64` — 200/500/1200/3000 marks
- `can_upgrade_guild(deep: &DeepState) -> bool` — checks marks + layer requirement
- `upgrade_guild(deep: &mut DeepState) -> Result<(), UpgradeError>` — deducts marks, bumps rank
- `is_free_daily_available(deep: &DeepState, current_time: i64) -> bool` — one free supply run per calendar day

---

## Phase 3: Persistence and Discovery

**Depends on:** Phase 1 (types), Phase 2 (logic for prestige reset handling)
**Tasks:** #12

### 3.1 New Files

| File | Contents |
|------|----------|
| `src/deep/persistence.rs` | `load_deep()`, `save_deep()`, `deep_save_path()` — follows Enhancement pattern exactly |
| `src/deep/discovery.rs` | `deep_discovery_chance()`, `try_discover_deep()` — follows Soulforge discovery pattern |

### 3.2 Persistence Pattern (mirrors `enhancement/persistence.rs`)

```rust
fn deep_save_path() -> io::Result<PathBuf>  // ~/.quest/deep.json
fn load_deep() -> DeepState                  // Returns DeepState::new() on missing/corrupt
fn save_deep(deep: &DeepState) -> io::Result<()>  // Pretty-printed JSON
```

### 3.3 Discovery Pattern (mirrors Soulforge in `tick.rs`)

```rust
fn deep_discovery_chance(prestige_rank: u32) -> f64  // 0.000014 + (rank - 15) * 0.000007
fn try_discover_deep<R: Rng>(deep: &mut DeepState, prestige_rank: u32, rng: &mut R) -> bool
```

### 3.4 Prestige Reset Integration

Add to `perform_prestige()` flow (via a new function or callback pattern):
- `reset_deep_for_prestige(deep: &mut DeepState, current_time: i64)` — clears mercs, marks, active missions (with partial rewards), preserves guild rank, cleared layers, infrastructure, familiarity

### 3.5 Modified Files

| File | Change |
|------|--------|
| `src/main.rs` | Load `deep_state` at startup (like haven/enhancement), save on `deep_changed` flag |
| `src/main_helpers/persistence.rs` | Add `deep` parameter to `save_all()` |
| `src/core/tick_types.rs` | Add `DeepDiscovered` variant to `TickEvent`, add `deep_changed: bool` to `TickResult` |
| `src/core/tick.rs` | Add Stage 13: Deep discovery check (P15+, same guard pattern as Stage 11), add `deep: &mut DeepState` parameter to `game_tick()` |
| `src/character/prestige_actions.rs` | Call `reset_deep_for_prestige()` in prestige flow |
| `src/lib.rs` | Add `pub mod deep;` and re-exports |

---

## Phase 4: UI Overlay and Input Handling

**Depends on:** Phase 1 (types for rendering)
**Can run in parallel with:** Phase 2 (logic) — UI can be built against type stubs

**Tasks:** #5 (design), #13 (UI implementation), #14 (input handling)

### 4.1 New Files

| File | Contents |
|------|----------|
| `src/ui/deep_scene.rs` | Main Deep overlay renderer (delegates to sub-modules) |
| `src/ui/deep_roster.rs` | Roster sub-view — merc list with stats, status, archetype |
| `src/ui/deep_missions.rs` | Mission sub-view — active missions, progress bars, squad details |
| `src/ui/deep_layers.rs` | Layer sub-view — layer list, infrastructure, familiarity |
| `src/ui/deep_events.rs` | Event response sub-view — check-in event choices |
| `src/ui/deep_recruit.rs` | Recruitment sub-view — available candidates, cost display |
| `src/ui/deep_mission_setup.rs` | Mission setup sub-view — mission selection, squad picker |
| `src/input/deep_input.rs` | Deep overlay input routing (follows Haven/Soulforge pattern) |

### 4.2 UI State Types (in `src/deep/types.rs` or `src/input/types.rs`)

```rust
enum DeepView { Main, Roster, Missions, Layers, EventResponse, Recruitment, MissionSetup }
struct DeepUiState {
    open: bool,
    view: DeepView,
    selected_index: usize,        // For list navigation
    squad_selection: Vec<u64>,     // Merc IDs selected for squad
    selected_mission: Option<usize>,
    // ... additional per-view state
}
```

### 4.3 Overlay Integration Pattern

Follows the exact pattern used by Haven/Soulforge/Stormglass:
1. `GameOverlay` enum in `src/input/types.rs` gets a new `Deep` variant
2. `DeepUiState` manages open/closed and sub-view navigation
3. Keybind (e.g., `d` or `g`) toggles the overlay
4. Stats panel shows notification indicator when events are pending (e.g., `[D] Event!`)

### 4.4 Modified Files

| File | Change |
|------|--------|
| `src/input/mod.rs` | Add `deep_input` module, route Deep overlay input |
| `src/input/types.rs` | Add `Deep` variant to `GameOverlay`, add `DeepUiState` |
| `src/ui/mod.rs` | Add `deep_scene` and sub-module declarations |
| `src/ui/stats_panel.rs` | Add Deep notification indicator (pending events, completed missions) |
| `src/main.rs` | Add `deep_ui_state` to game loop, render Deep overlay |
| `src/main_helpers/overlay.rs` | Add Deep overlay to overlay draw dispatch |
| `src/main_helpers/scene.rs` | Include Deep in scene kind checks |
| `src/main_helpers/input_routing.rs` | Route Deep keybind and overlay input |

---

## Phase 5: Game State Integration

**Depends on:** Phase 1 (types), Phase 2 (logic), Phase 3 (persistence)
**Tasks:** #2 (architecture), #10 (mission ticking integration)

### 5.1 Tick Integration

The Deep requires a different ticking model than existing systems:

**Wall-clock ticking** (NOT game-tick-based):
- Missions progress based on `SystemTime::now()` vs `mission.started_at`, not `tick_counter`
- This is similar to `offline.rs` but runs while the game is open too

**Integration point in `game_tick()`:**
- New Stage 13 (or extend Stage 11): Deep discovery check
- New Stage 14: Deep mission ticking — call `tick_missions()` every ~10 seconds (guard with a simple modulo on `tick_counter`)
- Emit new `TickEvent` variants for mission completions, events pending, etc.

**On game load (main.rs):**
- After loading `DeepState`, call `catch_up_missions(deep, current_time)` to process all missions that completed while offline
- Auto-resolve any events that timed out (> 2 hours old)
- Queue completed mission results for display

### 5.2 New TickEvent Variants

```rust
// Add to TickEvent enum in tick_types.rs
DeepDiscovered,
DeepMissionCompleted { mission_type: MissionType, layer: u32, outcome: MissionOutcome, message: String },
DeepEventPending { mission_id: u64, message: String },
DeepEventAutoResolved { message: String },
DeepBreakthroughCompleted { layer: u32, message: String },
```

### 5.3 Modified Files (Integration Touchpoints)

| File | Change |
|------|--------|
| `src/core/tick.rs` | Add `deep: &mut DeepState` parameter, add discovery + mission tick stages |
| `src/core/tick_types.rs` | Add `DeepDiscovered` and mission-related TickEvent variants, add `deep_changed: bool` to TickResult |
| `src/tick_events.rs` | Map new TickEvent variants to combat log entries |
| `src/character/prestige_actions.rs` | Accept `&mut DeepState` in prestige functions for reset |
| `src/main.rs` | Load/save DeepState, pass to `game_tick()`, handle Deep-related events, catch up missions on load |
| `src/main_helpers/persistence.rs` | Include Deep in `save_all()` |
| `src/main_helpers/offline.rs` | Add Deep mission catch-up alongside offline XP |
| `src/bin/simulator.rs` | Add `--deep` flag for simulator (optional, lower priority) |
| `src/achievements/types.rs` | Add Deep-related achievement IDs |
| `src/achievements/handlers.rs` | Add Deep event handlers (on_deep_discovered, on_breakthrough, etc.) |
| `src/achievements/data.rs` | Add Deep achievement definitions |
| `src/utils/debug_menu.rs` | Add Deep debug options (discover, grant marks, force breakthrough) |
| `src/ui/debug_menu_scene.rs` | Add Deep tab to debug menu |

### 5.4 Signature Change: `game_tick()`

The most impactful change is adding `deep: &mut DeepState` to `game_tick()`. This follows the exact pattern used when Enhancement was added (Enhancement was added as a parameter after Haven).

```rust
// Before:
pub fn game_tick<R: Rng>(state, tick_counter, haven, enhancement, achievements, debug_mode, rng) -> TickResult

// After:
pub fn game_tick<R: Rng>(state, tick_counter, haven, enhancement, deep, achievements, debug_mode, rng) -> TickResult
```

**Impact:** Every call site for `game_tick()` must be updated:
- `src/main.rs`
- `src/bin/simulator.rs`
- All integration tests calling `game_tick()` (30+ test files)
- Tests inside `src/core/tick.rs`

---

## Phase 6: Testing

**Progressive — starts after each module is implemented.**
**Tasks:** #15, #16, #17, #18, #19, #20, #21

### 6.1 Unit Tests Per Module

| Module | Test File | Key Test Cases |
|--------|-----------|----------------|
| `deep/mercenaries.rs` | Inline `#[cfg(test)]` | Generate merc stats by archetype, recruit pool generation, level-up thresholds, injury/recovery timing, starter roster composition |
| `deep/missions.rs` | Inline `#[cfg(test)]` | Mission generation by layer/rank, squad validation (requirements/recommendations), mission duration calculations with infrastructure modifiers, resolution outcomes |
| `deep/events.rs` | Inline `#[cfg(test)]` | Event trigger timing (25/50/75%), archetype bonus unlocking, auto-resolve picks safest, event chaining |
| `deep/layers.rs` | Inline `#[cfg(test)]` | Layer tier mapping, familiarity duration modifier curve, infrastructure slot constraints (max 2), breakthrough advances frontier |
| `deep/economy.rs` | Inline `#[cfg(test)]` | Mark earning rates by mission/layer, recruitment costs, guild rank prerequisites, free daily supply run logic |
| `deep/persistence.rs` | Inline `#[cfg(test)]` | Round-trip serialize/deserialize, backward compat with missing fields, corrupt file fallback |
| `deep/discovery.rs` | Inline `#[cfg(test)]` | Discovery probability curve, blocked when not P15+, blocked during active content |

### 6.2 Integration Tests

| Test File | Purpose | Depends On |
|-----------|---------|-----------|
| `tests/deep_mercenary_test.rs` | Merc generation, recruitment, level-up across missions | Phase 2 |
| `tests/deep_mission_test.rs` | Full mission lifecycle: create -> tick -> events -> resolve | Phase 2, 5 |
| `tests/deep_economy_test.rs` | Economy balance: earning vs spending across guild ranks | Phase 2 |
| `tests/deep_persistence_test.rs` | Save/load round-trip, migration from older save formats | Phase 3 |
| `tests/deep_prestige_test.rs` | Prestige reset preserves guild/layers/infra, clears mercs/marks/missions | Phase 3, 5 |
| `tests/deep_discovery_test.rs` | Discovery gating, tutorial flow, initial state | Phase 3, 5 |
| `tests/deep_integration_test.rs` | End-to-end: discover -> recruit -> mission -> breakthrough -> prestige -> resume | Phase 5 |

### 6.3 Testing Strategy

- **All RNG-dependent logic uses generic `<R: Rng>`** for deterministic seeded testing (follows existing `game_tick()` pattern)
- **Wall-clock time is parameterized** — functions accept `current_time: i64` rather than calling `SystemTime::now()`. Tests pass synthetic timestamps.
- **No hardware-dependent timing tests** — test functional outcomes (mission completed? event triggered?) not elapsed wall-clock durations
- **Backward compatibility tests** — deserializing older `DeepState` JSON (with missing new fields) produces valid defaults

---

## Minimum Viable Feature (Smallest Playable Slice)

The MVP is the smallest subset that delivers a playable loop:

### MVP Scope

1. **Discovery** — P15+ tick-based discovery, combat log message
2. **Starter roster** — 3 free mercs (Vanguard, Scout, Medic)
3. **Layer 1-3** (The Shallows) — basic difficulty, no environmental hazards
4. **Supply Run missions only** — 2-4h, safe, no events, no risk
5. **Basic Breakthrough mission** — unlock Layer 2, then Layer 3. 1 event each, simple choices.
6. **Main overlay** — shows guild rank, marks, active missions with progress bars
7. **Roster sub-view** — list mercs, show stats and status
8. **Warband Marks** — earn from supply runs, spend on recruitment
9. **Persistence** — save/load DeepState to `~/.quest/deep.json`
10. **Prestige reset** — clears mercs/marks, preserves cleared layers

### MVP Excludes (Add Later)

- Recon, Expedition, Construction mission types
- Check-in events (beyond basic breakthrough event)
- Infrastructure building
- Familiarity system
- Guild rank upgrades (start at Rank 1 permanently for MVP)
- Recruitment pool (rotating daily candidates) — MVP just gives starter mercs
- Merc injury/loss system
- Rewards flowing to main game (XP, items, Stormglass, PR fragments)
- Achievements
- Debug menu integration
- Simulator integration
- Layer sub-view
- Event response sub-view
- Layers 4+

### MVP Implementation Order

1. `types.rs` — full type definitions (even for post-MVP features, to avoid schema changes)
2. `persistence.rs` + `discovery.rs` — can discover and persist
3. `mercenaries.rs` (starter roster only)
4. `missions.rs` (Supply Run + basic Breakthrough only)
5. `economy.rs` (mark earning from supply runs, recruit cost)
6. `layers.rs` (Layer 1-3 definitions, breakthrough logic)
7. `deep_input.rs` + `deep_scene.rs` (main overlay + roster view)
8. Integration: `tick.rs` discovery, `main.rs` load/save, prestige reset

---

## Risk Assessment

### 1. Wall-Clock Time Implementation (HIGH RISK)

**Risk:** Quest has never used wall-clock time for game mechanics. All existing systems run on the 100ms tick loop. Introducing real-time missions creates new complexity around:
- Time zone handling (use UTC consistently)
- Clock manipulation/cheating (accept it — idle game convention)
- Offline catch-up (missions completed while game was closed)
- Multiple events firing simultaneously on catch-up

**Mitigation:**
- All time as `i64` Unix timestamps (UTC), never `SystemTime` in persisted state
- `tick_missions()` accepts `current_time` parameter (testable, no wall-clock dependency in logic)
- Offline catch-up processes missions in chronological order, not all-at-once
- Auto-resolve always picks safest option — no negative surprise from offline events

### 2. `game_tick()` Signature Change (MEDIUM RISK)

**Risk:** Adding `deep: &mut DeepState` to `game_tick()` touches every call site — main.rs, simulator, and 30+ integration test files.

**Mitigation:**
- This is the same mechanical change that was done when `enhancement` was added
- Can be done as a single atomic commit before any Deep logic is added
- Tests only need a `DeepState::default()` stub initially

### 3. Mission State Persistence Complexity (MEDIUM RISK)

**Risk:** Active missions carry complex state (pending events, squad assignments, timestamps). Serialization bugs could corrupt saves or lose in-progress missions.

**Mitigation:**
- Use `#[serde(default)]` on all fields for backward compatibility
- Round-trip serialization tests for every state combination
- `load_deep()` returns `DeepState::new()` on any parse failure (same as Enhancement)
- Completed missions are moved to a separate `completed_missions` vec (bounded, e.g., last 20)

### 4. UI Complexity — Multiple Sub-Views (MEDIUM RISK)

**Risk:** The Deep overlay has 7 sub-views (Main, Roster, Missions, Layers, EventResponse, Recruitment, MissionSetup). This is significantly more complex than Haven (2 panels) or Soulforge (slot list + animation).

**Mitigation:**
- Each sub-view is a separate file (follows Haven's haven_tree.rs/haven_details.rs pattern)
- MVP ships with only 2 sub-views (Main + Roster)
- State machine in `DeepView` enum prevents invalid transitions
- Sub-views are stateless renderers — all state lives in `DeepUiState`

### 5. Balancing Mission Durations and Rewards (LOW RISK)

**Risk:** Getting the feel right for 2-24 hour missions requires playtesting that can't be unit-tested.

**Mitigation:**
- All duration/reward values are constants in `economy.rs` and `types.rs` — easy to tune
- Simulator can be extended with `--deep` flag for accelerated testing
- MVP starts with conservative values (shorter durations, higher rewards) and tunes down

### 6. Prestige Reset Interaction (LOW RISK)

**Risk:** The split persistence model (some state persists, some resets) could have edge cases.

**Mitigation:**
- The exact same pattern is used by Haven (persists) + GameState (resets)
- `reset_deep_for_prestige()` is a single function with clear documentation
- Integration test covers: prestige -> verify guild/layers preserved, mercs/marks cleared

---

## Task Dependency Graph

```
Phase 1 (Types):
  #1 Design types ──────────────► #8 Implement types.rs
                                      │
                                      ▼
Phase 2 (Logic):            ┌────────────────────────┐
  #3 Event design ─────────►│                        │
  #4 Balance design ────────►│  #9  Mercenaries      │
                             │  #10 Missions          │ (parallelizable)
                             │  #11 Layers/Economy    │
                             │                        │
                             └────────┬───────────────┘
                                      │
Phase 3 (Persistence):               ▼
  #2 Integration arch ─────► #12 Persistence + Discovery
                                      │
Phase 4 (UI):                         │
  #5 UI design ────────────► #13 UI Overlay ──────────┤
                             #14 Input Handling ──────┤
                                                      │
Phase 5 (Integration):                                ▼
                             Full game_tick() integration
                             (combines Phases 2-4)
                                      │
Phase 6 (Testing):                    ▼
  #15 Merc unit tests ◄───── #9
  #16 Mission unit tests ◄── #10
  #17 Economy unit tests ◄── #11
  #18 Persistence tests ◄─── #12
  #19 Discovery flow tests ◄─ #12, #13, #14
  #20 E2E mission tests ◄─── #10, #13
  #21 Prestige reset tests ◄─ #12, #11
```

---

## Complete File Inventory

### New Files (17 files)

| File | Phase | Description |
|------|-------|-------------|
| `src/deep/mod.rs` | 1 | Module declaration and re-exports |
| `src/deep/types.rs` | 1 | All core data structures, constants |
| `src/deep/CLAUDE.md` | 1 | Module documentation |
| `src/deep/mercenaries.rs` | 2 | Merc generation, recruitment, leveling, injury |
| `src/deep/missions.rs` | 2 | Mission lifecycle: generation, ticking, resolution |
| `src/deep/events.rs` | 2 | Check-in event system and templates |
| `src/deep/layers.rs` | 2 | Layer progression, infrastructure, familiarity |
| `src/deep/economy.rs` | 2 | Warband Marks economy, guild ranks, costs |
| `src/deep/rewards.rs` | 2 | Reward generation flowing into existing systems |
| `src/deep/persistence.rs` | 3 | Save/load `~/.quest/deep.json` |
| `src/deep/discovery.rs` | 3 | Discovery roll (P15+ tick-based) |
| `src/ui/deep_scene.rs` | 4 | Main overlay renderer |
| `src/ui/deep_roster.rs` | 4 | Roster sub-view |
| `src/ui/deep_missions.rs` | 4 | Missions sub-view |
| `src/ui/deep_layers.rs` | 4 | Layers sub-view |
| `src/ui/deep_events.rs` | 4 | Event response sub-view |
| `src/ui/deep_recruit.rs` | 4 | Recruitment sub-view |
| `src/ui/deep_mission_setup.rs` | 4 | Mission setup + squad picker |
| `src/input/deep_input.rs` | 4 | Deep overlay input routing |

### New Test Files (7 files)

| File | Phase |
|------|-------|
| `tests/deep_mercenary_test.rs` | 6 |
| `tests/deep_mission_test.rs` | 6 |
| `tests/deep_economy_test.rs` | 6 |
| `tests/deep_persistence_test.rs` | 6 |
| `tests/deep_prestige_test.rs` | 6 |
| `tests/deep_discovery_test.rs` | 6 |
| `tests/deep_integration_test.rs` | 6 |

### Modified Files (19 files)

| File | Phase | Change Summary |
|------|-------|---------------|
| `src/main.rs` | 3, 4, 5 | Add `mod deep`, load/save DeepState, pass to game_tick, render overlay, catch-up missions on load |
| `src/lib.rs` | 1 | Add `pub mod deep;` and re-exports |
| `src/core/tick.rs` | 3, 5 | Add `deep` param to `game_tick()`, add discovery stage, add mission tick stage |
| `src/core/tick_types.rs` | 3, 5 | Add `DeepDiscovered` + mission TickEvent variants, add `deep_changed` flag |
| `src/tick_events.rs` | 5 | Map new Deep TickEvent variants to combat log entries |
| `src/character/prestige_actions.rs` | 3 | Add Deep reset to prestige flow |
| `src/main_helpers/persistence.rs` | 3 | Add `deep` to `save_all()` |
| `src/main_helpers/offline.rs` | 5 | Add Deep mission catch-up on game load |
| `src/main_helpers/overlay.rs` | 4 | Add Deep overlay rendering dispatch |
| `src/main_helpers/scene.rs` | 4 | Include Deep in scene kind checks |
| `src/main_helpers/input_routing.rs` | 4 | Route Deep keybind and overlay input |
| `src/input/mod.rs` | 4 | Add `deep_input` module, route overlay input |
| `src/input/types.rs` | 4 | Add `Deep` variant to `GameOverlay`, add `DeepUiState` |
| `src/ui/mod.rs` | 4 | Add deep scene module declarations |
| `src/ui/stats_panel.rs` | 4 | Add Deep notification indicator |
| `src/achievements/types.rs` | 5 | Add Deep achievement IDs |
| `src/achievements/handlers.rs` | 5 | Add Deep event handlers |
| `src/achievements/data.rs` | 5 | Add Deep achievement definitions |
| `src/utils/debug_menu.rs` | 5 | Add Deep debug options |
| `src/ui/debug_menu_scene.rs` | 5 | Add Deep tab to debug menu |
| `src/bin/simulator.rs` | 5 | Update `game_tick()` call signature (at minimum), optional `--deep` flag |

### Total: ~18 new source files + 7 test files + ~21 modified files

---

## Implementation Sequencing Recommendation

### Suggested Order (Critical Path)

1. **Types first** (#8) — unblocks everything
2. **`game_tick()` signature change** — add `deep: &mut DeepState` parameter with `DeepState::default()` stubs in all call sites. Do this early as a standalone commit to avoid merge conflicts with other work.
3. **Persistence + Discovery** (#12) — small, self-contained, enables testing the discovery flow
4. **Mercenaries + Economy** (#9, #11 partial) — enables recruitment loop
5. **Missions** (#10) — the core gameplay loop
6. **Basic UI** (#13 partial) — main overlay + roster, enough to play
7. **Input handling** (#14) — make it interactive
8. **Events** (#10 partial) — check-in events for missions
9. **Full UI sub-views** (#13 complete) — layers, events, recruitment, mission setup
10. **Achievements + Debug** — polish
11. **Integration tests** — end-to-end validation

### Parallelization Opportunities

- **Tasks #9, #10, #11** can be implemented in parallel by different developers (they share types but have independent logic)
- **Task #13 (UI) and #14 (input)** can proceed in parallel with Phase 2 logic
- **All unit test tasks (#15, #16, #17)** can be written alongside their corresponding modules
- **Integration tests (#18-#21)** must wait for their dependencies but can run in parallel with each other
