---
name: perf-audit
description: Multi-agent performance audit — finds and fixes hot-path bottlenecks, adds criterion benchmarks and simulator profiling. Use when game feels slow, after adding features, or to establish performance baselines.
---

# Performance Audit

Multi-agent audit of runtime performance. Finds bottlenecks, auto-fixes safe patterns, and leaves behind profiling infrastructure.

## When to Use

- After landing new features or systems
- Game tick feels slow or laggy
- Before a release
- To establish performance baselines
- After significant code changes to hot paths

## Phase 1: Parallel Audit (3 Agents, Read-Only)

Spawn 3 Explore agents simultaneously:

**Agent 1 — Tick Loop & Combat**

Scope: `src/core/tick.rs`, `tick_stages.rs`, `tick_context.rs`, `combat/`, `enemy_spawning.rs`, `xp.rs`, `game_logic.rs`, `loom/logic.rs`

Search for:

| Pattern | Example | Fix |
|---------|---------|-----|
| Linear scans in per-tick code | `.iter().find()`, `.contains()` on Vec | HashMap/HashSet |
| Per-tick allocations | `String::new()`, `Vec::new()`, `format!()` in tick path | Pre-allocate, `&str`, reuse buffers |
| Redundant computation | Same calculation repeated across stages | Cache in TickContext or session_state |
| Unnecessary cloning | `.clone()` where borrow suffices | Borrow or reference |

**Agent 2 — UI Rendering**

Scope: `src/ui/`

Search for:

| Pattern | Example | Fix |
|---------|---------|-----|
| Per-frame allocations | `format!()`, `String::from()` in render functions | Static strings, write macros |
| Redundant layout computation | Same layout calculated multiple times per frame | Cache or compute once |
| Expensive sprite lookups | Linear search through sprite data per frame | Index or cache |

**Agent 3 — Discovery, Achievements & Persistence**

Scope: `src/achievements/`, `src/deep/`, `src/core/discoveries.rs`, `*/persistence.rs`

Search for:

| Pattern | Example | Fix |
|---------|---------|-----|
| Per-tick milestone checks | Iterating all milestones every tick | Early-exit, skip if counter unchanged |
| Linear achievement lookups | Scanning all achievements for status | HashMap or indexed |
| Unnecessary serialization work | Formatting/allocation in save path | Lazy or batched |

Each agent produces a ranked report: location, pattern, severity (HIGH/MEDIUM/LOW), suggested fix.

## Phase 2: Fix (Sequential)

Spawn fix agents based on audit findings.

### Auto-fix (no user approval needed)

- Replace linear scan with HashMap/HashSet lookup
- Add `LazyLock`/`OnceLock` caching for repeated computation
- Replace per-tick `String`/`Vec` allocation with pre-allocated or borrowed alternatives
- Replace `.clone()` with borrows where lifetime allows
- Add early-exit to loops when result is already determined

### Flag for user review

- Changes to public function signatures
- Changes that alter game behavior (even subtly)
- Changes to serialized types (save compatibility)
- Anything touching `constants.rs` values

## Phase 3: Infrastructure

### 3a. Criterion Benchmarks

If `benches/game_tick.rs` doesn't exist yet, create it:

1. Add `criterion` to `[dev-dependencies]` in `Cargo.toml`
2. Add `[[bench]]` section to `Cargo.toml`
3. Create `benches/game_tick.rs` with:
   - `bench_game_tick` — end-to-end `game_tick()` at 3 game states (Early L1/P0, Mid L50/P10, Endgame L100/P50)
   - Sub-function benchmarks for each confirmed bottleneck from Phase 1
   - All benchmarks use seeded `ChaCha8Rng` for deterministic state
4. Run `cargo bench` to establish baseline

### 3b. Simulator Profiling

Add `--profile` flag to `src/bin/simulator.rs`:

1. Add `profile: bool` to `SimConfig`
2. When active, wrap `game_tick()` in `Instant::now()` timing
3. Print per-tick timing breakdown at end of simulation
4. Zero overhead when `--profile` is not passed

## Phase 4: Verify

1. `make check` must pass
2. `cargo bench` runs without errors
3. Report summary of: findings, auto-fixes applied, items flagged for review, benchmark baselines

## Output

Report the PR URL and final status when done (use `/ship` skill).
