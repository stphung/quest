# Performance Audit Skill Design

## Overview

A multi-agent skill that audits the game for runtime performance bottlenecks, auto-fixes safe patterns, and leaves behind profiling infrastructure (criterion benchmarks + simulator profiling).

**Skill name**: `perf-audit`
**Trigger phrases**: "audit performance", "optimize hot paths", "profile the game", "perf audit"

## Process Flow

```
Phase 1: Parallel Audit (3 agents, read-only)
  ├── Agent 1: Tick loop & combat
  ├── Agent 2: UI rendering & stats
  └── Agent 3: Discovery, achievements & persistence

Phase 2: Fix (sequential)
  ├── Auto-fix safe patterns
  └── Flag risky changes for user review

Phase 3: Infrastructure (sequential)
  ├── Add criterion benchmarks for game_tick() + confirmed bottleneck functions
  └── Extend simulator.rs with --profile flag

Phase 4: Verify
  ├── make check
  ├── Run criterion benchmarks for baseline
  └── Report findings summary
```

## Phase 1: Audit Agents

### Agent 1 — Tick Loop & Combat

**Scope**: `src/core/tick.rs`, `tick_stages.rs`, `tick_context.rs`, `combat/`, `enemy_spawning.rs`, `xp.rs`, `game_logic.rs`, `loom/logic.rs`

| Pattern | Example | Fix |
|---------|---------|-----|
| Linear scans in per-tick code | `.iter().find()`, `.contains()` on Vec | HashMap/HashSet |
| Per-tick allocations | `String::new()`, `Vec::new()`, `format!()` | Pre-allocate, `&str`, reuse buffers |
| Redundant computation | Same calculation repeated across stages | Cache in TickContext or session_state |
| Unnecessary cloning | `.clone()` where borrow suffices | Borrow or reference |

### Agent 2 — UI Rendering

**Scope**: `src/ui/`

| Pattern | Example | Fix |
|---------|---------|-----|
| Per-frame allocations | `format!()`, `String::from()` in render | Static strings, write macros |
| Redundant layout computation | Same layout calculated multiple times per frame | Cache or compute once |
| Expensive sprite lookups | Linear search through sprite data per frame | Index or cache |

### Agent 3 — Discovery, Achievements & Persistence

**Scope**: `src/achievements/`, `src/deep/`, `src/core/discoveries.rs`, `*/persistence.rs`

| Pattern | Example | Fix |
|---------|---------|-----|
| Per-tick milestone checks | Iterating all milestones every tick | Early-exit, skip if counter unchanged |
| Linear achievement lookups | Scanning all achievements for status | HashMap or indexed |
| Unnecessary serialization work | Formatting/allocation in save path | Lazy or batched |

## Phase 2: Fix Guardrails

### Auto-fix (no user approval needed)

- Replace linear scan with HashMap/HashSet lookup
- Add `LazyLock`/`OnceLock` caching for repeated computation
- Replace per-tick `String`/`Vec` allocation with pre-allocated or borrowed alternatives
- Replace `.clone()` with borrows where lifetime allows
- Add early-exit to loops when result is already determined

### Flag for user review

- Changes to public function signatures
- Changes that alter game behavior
- Changes to serialized types (save compatibility)
- Anything touching `constants.rs` values

## Phase 3: Infrastructure

### Criterion Benchmarks (`benches/game_tick.rs`)

| Benchmark | What it measures | Game states |
|-----------|-----------------|-------------|
| `bench_game_tick` | Full `game_tick()` end-to-end | Early (L1/P0), Mid (L50/P10), Endgame (L100/P50) |
| `bench_<bottleneck>` | Per-function for confirmed hot spots | Same 3 states |

Sub-function benchmarks are determined by audit findings, not pre-specified. Each benchmark uses seeded `ChaCha8Rng` for deterministic game states.

### Simulator Profiling (`--profile` flag)

```
cargo run --bin simulator -- --profile --ticks 10000 --prestige 20
```

Output: per-stage timing breakdown showing where tick time is spent. Implementation wraps each tick stage in `Instant::now()` / `elapsed()` timing, accumulates into a `StageProfile` struct. Zero overhead when `--profile` is not passed.

## Phase 4: Verification

1. `make check` must pass (fmt, clippy, test, build, audit)
2. Run criterion benchmarks to establish baseline
3. Report: findings summary, what was auto-fixed, what needs review
