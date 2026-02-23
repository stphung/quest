# The Deep — Quality Standards & Review Criteria

Quality standards, testing requirements, review checklists, acceptance criteria, and performance considerations for The Deep (Mercenary Expedition System).

---

## 1. Code Quality Standards

### Module Structure

Follow the established Quest module pattern:

```
src/deep/
├── mod.rs          # Public re-exports
├── types.rs        # Mercenary, Layer, Mission, Guild, DeepState, events
├── generation.rs   # Merc generation, mission generation, event generation
├── logic.rs        # Mission ticking, event resolution, squad validation
├── persistence.rs  # Save/load from ~/.quest/deep.json (account-level)
└── discovery.rs    # Discovery roll logic (P15+ tick-based)
```

Every file must have a module-level `//!` doc comment explaining its purpose (see any existing module for examples).

### Zero UI Imports in Game Logic

All files in `src/deep/` must have **zero imports from `src/ui/`**. This is the same constraint enforced on `src/core/tick.rs` and all other game logic modules. The separation works as follows:

- Game logic returns data (structs, enums, events) describing what happened.
- The presentation layer (`main.rs`, `src/ui/`) reads that data and renders it.
- No `ratatui`, `crossterm`, or UI type references in game logic.

### Explicit Parameter Injection (Haven Pattern)

Deep bonuses and state must be passed as explicit parameters, not accessed via globals or statics. This is the same pattern used by Haven, Enhancement, and Stormglass:

```rust
// GOOD: explicit parameter
pub fn resolve_mission(mission: &mut Mission, deep_state: &DeepState, rng: &mut R) -> MissionResult { ... }

// BAD: global access
pub fn resolve_mission(mission: &mut Mission) -> MissionResult {
    let state = DEEP_STATE.lock().unwrap(); // NEVER do this
}
```

### Serde Derives for All Persistent Types

Every type that is part of the save file (`~/.quest/deep.json`) must derive `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepState { ... }
```

Transient fields (UI state, animation ticks, cached computations) must be marked `#[serde(skip)]` with sensible defaults via `Default` implementation.

### Generic RNG for Testability

All functions that use randomness must accept `<R: Rng>` (or `&mut impl Rng`) as a parameter. This enables deterministic testing with seeded `ChaCha8Rng`:

```rust
pub fn generate_mercenary<R: Rng>(guild_rank: u32, rng: &mut R) -> Mercenary { ... }
pub fn roll_event_outcome<R: Rng>(event: &CheckInEvent, squad: &Squad, rng: &mut R) -> EventOutcome { ... }
```

Never use `rand::rng()` or `thread_rng()` directly inside game logic functions.

### Clippy and Formatting

- All code must pass `cargo clippy --all-targets -- -D warnings` with zero warnings.
- All code must pass `cargo fmt --check`.
- No `#[allow(clippy::...)]` without a justifying comment.
- No `unwrap()` or `expect()` in production code paths. Use `unwrap_or_default()`, `?`, or explicit error handling. (Test code may use `unwrap()`.)

### Naming Conventions

- Types: `PascalCase` (e.g., `Mercenary`, `MissionType`, `GuildRank`)
- Functions: `snake_case` (e.g., `generate_mercenary`, `resolve_mission`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `DEEP_MIN_PRESTIGE_RANK`, `MAX_ROSTER_SIZE`)
- Enum variants: `PascalCase` (e.g., `MissionType::SupplyRun`, `Archetype::Vanguard`)
- Module files: `snake_case.rs`

### Constants Organization

All balance constants for The Deep must live in `src/deep/types.rs` (following Enhancement's pattern) or in `src/core/constants.rs` if they're referenced by the tick engine. Group constants with doc comments:

```rust
// ── Discovery ──────────────────────────────────────
pub const DEEP_MIN_PRESTIGE_RANK: u32 = 15;
pub const DEEP_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
pub const DEEP_DISCOVERY_RANK_BONUS: f64 = 0.000007;
```

---

## 2. Testing Requirements

### Coverage Target

The Deep game logic modules (`src/deep/`) must maintain **90% line coverage**, consistent with the project-wide CI gate (`--fail-under-lines 90` in `scripts/ci-checks.sh`, which excludes `ui/`, `utils/updater`, `utils/build_info`, and `tick_events`).

### Test Determinism

All tests must use seeded RNG for reproducibility:

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}
```

No `thread_rng()`, no `SystemTime::now()` in tests. For wall-clock time testing, inject timestamps as parameters.

### Unit Tests by Module

#### `types.rs` Tests
- Default/new construction for all structs (DeepState, Mercenary, Mission, Guild, Layer)
- Serde roundtrip: serialize then deserialize all persistent types, assert equality
- Enum variant completeness: all variants of MissionType, Archetype, LayerTier have correct properties
- Boundary values: max roster size, max guild rank, max layer depth

#### `generation.rs` Tests
- Mercenary generation produces valid stats for each archetype
- Mercenary quality scales with guild rank (higher rank = higher base stats)
- Mission generation respects layer difficulty and intel level
- Mission pool size is within bounds (3-5 missions)
- Event generation produces correct number of events per mission type (1-2 for short, 3-5 for breakthroughs)
- Seeded RNG produces identical output across runs

#### `logic.rs` Tests
- **Squad validation**: rejects squads below minimum requirements, accepts valid squads
- **Mission ticking**: wall-clock elapsed time correctly advances mission progress
- **Event scheduling**: events fire at correct progress percentages (25%, 50%, 75%)
- **Auto-resolve**: always picks the safest option (never risks merc loss)
- **Mission resolution**: success/partial/failure outcomes based on squad power vs. layer difficulty
- **Merc injury/loss**: only occurs on frontier missions, never on supply runs or construction
- **Merc XP**: awarded on mission completion, scales with mission difficulty
- **Infrastructure effects**: outpost reduces duration by 25%, watchtower improves auto-resolve
- **Breakthrough**: unlocks next layer on success, does not unlock on failure

#### `discovery.rs` Tests
- Discovery chance is 0 below P15
- Discovery chance increases linearly with prestige rank above 15
- Discovery is blocked when dungeon, fishing, or minigame is active (same as Haven/Soulforge)
- Once discovered, `discovered` flag persists and no further rolls occur

#### `persistence.rs` Tests
- Save/load roundtrip preserves all fields
- Missing file returns default state
- Corrupted JSON returns default state (graceful degradation)
- Save creates `~/.quest/` directory if needed
- Account-level fields survive prestige reset; character-level fields are cleared

### Integration Tests

Integration test files go in `tests/` (following existing patterns like `tests/enhancement_test.rs`, `tests/haven_dungeon_coverage_test.rs`).

#### `tests/deep_integration_test.rs`
- **Discovery flow**: Simulate ticks at P15+ until discovery occurs; verify DeepState transitions from None to Some
- **Mission lifecycle**: Generate mission, assign squad, tick time forward, trigger events, resolve mission, verify rewards
- **Prestige reset**: Verify guild rank, cleared layers, and infrastructure persist; mercs, marks, and active missions reset
- **Offline resolution**: Set mission start time in the past, load game, verify missions resolved with auto-resolve for missed events

#### `tests/deep_economy_test.rs`
- **Mark earning**: Verify all mission types award correct Warband Marks
- **Mark spending**: Recruitment, infrastructure, guild rank upgrades deduct correctly
- **Insufficient marks**: Operations fail gracefully when player cannot afford them
- **Guild rank progression**: Layer breakthrough requirements, cost verification, roster/mission slot scaling

#### `tests/deep_tick_test.rs`
- **game_tick integration**: Verify Deep discovery stage integrates with existing tick pipeline
- **No interference**: Deep system does not affect combat, fishing, dungeon, or challenge systems
- **TickEvent emission**: Deep-related events (discovery, mission complete, event pending) emit correct TickEvent variants

### What NOT to Test

Following the project's testing philosophy:

- Do not test UI rendering (Ratatui widget construction, layout calculations)
- Do not test internal data structure implementation details (cache sizes, internal ordering)
- Do not test timing-dependent behavior with wall-clock assertions (use injected timestamps)
- Do not test framework internals (serde implementation details, rand distribution internals)

---

## 3. PR Review Checklist

Every PR touching `src/deep/` must pass this checklist before merge:

### Architecture
- [ ] No `ui::` imports in game logic modules (`types.rs`, `logic.rs`, `generation.rs`, `discovery.rs`, `persistence.rs`)
- [ ] Bonuses and state passed as explicit parameters (Haven injection pattern)
- [ ] All persistent types derive `Serialize, Deserialize`
- [ ] All random functions accept generic `<R: Rng>` parameter
- [ ] Transient fields marked `#[serde(skip)]` with `Default` impl

### Data Model
- [ ] Types match the design doc (`docs/plans/2026-02-22-the-deep-mercenary-expedition-design.md`)
- [ ] Persistence correctly split: guild/layers/infrastructure persist; mercs/marks/missions reset on prestige
- [ ] Wall-clock timestamps use `chrono::Utc::now().timestamp()` (i64), not game ticks
- [ ] No panics: out-of-bounds access returns defaults, invalid state handled gracefully

### Balance
- [ ] Mission durations match design doc ranges (Supply 2-4h, Recon 4-8h, Expedition 8-16h, Breakthrough 18-24h, Construction 4-8h)
- [ ] Guild rank costs and unlock requirements match design doc
- [ ] Roster size caps match design doc (5/7/9/12/15 by rank)
- [ ] Concurrent mission slots match design doc (1/1/2/3/4 by rank)
- [ ] Infrastructure effects match design doc (outpost -25% duration, etc.)

### Safety
- [ ] Auto-resolve always picks the safest choice (never risks merc loss)
- [ ] Supply runs and construction missions never cause injury or loss
- [ ] Merc loss only possible on frontier missions with explicit risk
- [ ] No `unwrap()` or `expect()` in production code paths
- [ ] Offline resolution handles edge cases (very long absences, clock skew)

### Integration
- [ ] tick.rs changes are minimal (add discovery stage, similar to Haven/Soulforge pattern)
- [ ] DeepState passed to `game_tick()` the same way Haven and Enhancement are
- [ ] TickEvent variants added for Deep events (discovery, mission complete, event pending)
- [ ] TickResult gets `deep_changed: bool` flag for persistence signaling
- [ ] Achievement hooks called for Deep milestones (if applicable)

### Quality Gates
- [ ] `make check` passes (fmt, clippy, tests, build, audit)
- [ ] New tests added for all public functions
- [ ] Existing tests still pass (no regressions)
- [ ] Coverage maintained at 90%+ for game logic

---

## 4. Acceptance Criteria

The Deep is considered complete when all of the following are verified:

### Discovery
- [ ] Not available below P15
- [ ] Discovered via tick-based random roll (same pattern as Haven at P10+, Soulforge at P15+)
- [ ] Discovery blocked when dungeon, fishing, or minigame is active
- [ ] Discovery emits `TickEvent::DeepDiscovered` and sets `deep_changed` flag
- [ ] Once discovered, overlay is accessible via keybind

### Mercenaries
- [ ] 5 archetypes functional (Vanguard, Scout, Arcanist, Medic, Saboteur)
- [ ] Mercenary stats (Power, Resilience, Expertise, Level) calculated correctly per archetype
- [ ] Recruitment from rotating pool, cost in Warband Marks
- [ ] Roster size limited by guild rank
- [ ] Mercs gain XP from completed missions, level up correctly
- [ ] Mercs can be injured (unavailable for 1-2 missions) or lost on failed frontier missions
- [ ] Merc levels and roster reset on prestige

### Missions
- [ ] All 5 mission types functional (Supply Run, Recon, Expedition, Breakthrough, Construction)
- [ ] Mission generation produces 3-5 options matching current frontier/cleared layers
- [ ] Squad assignment validates requirements and recommendations
- [ ] Missions progress on wall-clock time (not game ticks)
- [ ] Missions progress while game is closed (offline resolution on load)
- [ ] Concurrent mission limit enforced per guild rank

### Check-In Events
- [ ] Events fire at scheduled progress percentages
- [ ] Archetype-specific options available when squad includes matching archetype
- [ ] Auto-resolve picks safest option when player does not respond
- [ ] Missed events (offline) auto-resolve correctly
- [ ] Events can chain (risky choice affects later events)

### Layers
- [ ] Layer tiers follow design doc (Shallows 1-3, Warrens 4-7, Hollows 8-12, Sunken Reach 13-18, Abyss 19-25, Void 26+)
- [ ] Breakthrough mission on Layer N unlocks Layer N+1
- [ ] Cleared layers persist across prestiges
- [ ] Infrastructure buildable on cleared layers
- [ ] Intel level accumulates from missions on a layer

### Economy
- [ ] Warband Marks earned from all mission types, scaling with depth and difficulty
- [ ] Marks spent on recruitment, infrastructure, guild rank upgrades
- [ ] Marks reset on prestige
- [ ] Guild rank persists across prestiges
- [ ] Guild rank costs and unlock requirements match design doc

### Persistence
- [ ] Save/load from `~/.quest/deep.json` works correctly
- [ ] Account-level data persists: guild rank, cleared layers, infrastructure, intel
- [ ] Character-level data resets on prestige: mercs, marks, active missions
- [ ] Corrupted save file falls back to default state (no crash)
- [ ] Active missions auto-cancel on prestige

### UI
- [ ] Overlay opens/closes cleanly (like Haven/Soulforge overlays)
- [ ] Active missions panel shows progress, time remaining, pending events
- [ ] New Mission sub-view shows available missions with requirements
- [ ] Roster sub-view shows merc list with stats and status
- [ ] Infrastructure sub-view shows per-layer build status
- [ ] Event Response sub-view shows choices with archetype-gated options
- [ ] Pending event indicator visible in main stats panel

### CI
- [ ] `make check` passes (fmt, clippy, tests, build, audit)
- [ ] No new clippy warnings
- [ ] Coverage maintained at 90%+ for game logic modules
- [ ] All new public APIs have tests

---

## 5. Performance Considerations

### Wall-Clock Time Processing

The Deep uses wall-clock time (real seconds) rather than game ticks for mission progression. This introduces performance constraints not present in tick-based systems.

**On game load (offline resolution):**
- Calculate elapsed time since last save for all active missions.
- Resolve any events that would have fired during offline period.
- This must complete in under **100ms** to avoid visible load delay (the game targets 100ms tick intervals).
- Strategy: iterate active missions (max 4 concurrent), each with at most 5 events. This is O(missions * events), bounded at 20 operations -- trivially fast.

**Per-tick processing:**
- The Deep should add **minimal overhead** to the existing game tick.
- Per tick, only check: (1) has any mission's wall-clock timer advanced past an event threshold? (2) has any mission completed?
- This is O(active_missions), bounded at 4. No expensive computation per tick.
- Discovery roll is a single random check per tick (same as Haven/Soulforge).

**Avoid:**
- Re-computing mission state from scratch every tick. Cache progress percentages and only recalculate when timestamp changes significantly.
- Sorting or searching large collections per tick. Mission and merc counts are small (max 15 mercs, max 4 missions).

### Persistence Overhead

**Save frequency:**
- Save `deep.json` only when `deep_changed` flag is set in TickResult (same pattern as Haven/Enhancement).
- Events that trigger saves: discovery, mission completion, event response, recruitment, construction, guild rank change.
- Do NOT save on every tick (mission progress is derived from timestamps, not stored progress percentages).

**Save file size:**
- Estimated maximum: ~50-100KB for a fully developed Deep state (15 mercs, 26+ layers with infrastructure/intel, guild rank, mission history).
- This is well within acceptable bounds -- Haven and Enhancement saves are under 5KB.
- Use `serde_json::to_string_pretty()` for human-readable saves (consistent with other Quest save files).

**Load performance:**
- Deserialization of ~100KB JSON is sub-millisecond. No concern here.
- Post-load offline resolution (see above) bounded at ~20 operations.

### Memory Usage

- DeepState is an account-level struct, single instance in memory.
- Mercenary roster: max 15 mercs, each ~200 bytes = ~3KB.
- Layer data: 26+ layers, each ~500 bytes with infrastructure and intel = ~15KB.
- Mission data: max 4 active, each ~1KB with events = ~4KB.
- Total in-memory footprint: **under 25KB**. Negligible compared to game state.

### Tick Budget

The game runs at 10 ticks/second (100ms per tick). The Deep's per-tick cost must fit within the existing tick budget alongside combat, fishing, dungeon, challenge AI, and other systems.

- **Target**: Deep tick processing should take under **1ms** per tick.
- **Measurement**: Use `cargo bench` or `std::time::Instant` timing in debug builds to verify.
- **If exceeded**: Profile and optimize the hot path (likely event scheduling or mission state checks).

### Simulator Compatibility

The Deep should integrate with the headless simulator (`src/bin/simulator.rs`):
- The simulator uses `game_tick()` with no UI and no tick delay.
- Wall-clock missions will need a simulated clock (injected timestamps) rather than `Utc::now()`.
- Add a `--deep` CLI flag (similar to `--haven`) for auto-managing Deep state during simulation runs.
- This is not required for initial implementation but should be planned for.

---

## References

- [Design doc](2026-02-22-the-deep-mercenary-expedition-design.md)
- [CI checks script](../../scripts/ci-checks.sh)
- [Enhancement system](../../src/enhancement/CLAUDE.md) -- closest architectural precedent
- [Haven system](../../src/haven/CLAUDE.md) -- bonus injection pattern precedent
- [Core tick engine](../../src/core/CLAUDE.md) -- tick integration pattern
