# Perf Audit Skill Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a `perf-audit` skill that spawns 3 parallel audit agents, auto-fixes safe bottleneck patterns, adds criterion benchmarks, and extends the simulator with `--profile`.

**Architecture:** A SKILL.md file in `.claude/skills/perf-audit/` drives the process. Criterion benchmarks live in `benches/game_tick.rs`. Simulator profiling is added to `src/bin/simulator.rs` behind a `--profile` flag.

**Tech Stack:** Rust, criterion (dev-dependency), `std::time::Instant` for profiling

---

### Task 1: Create the SKILL.md file

**Files:**
- Create: `.claude/skills/perf-audit/SKILL.md`

**Step 1: Write the skill file**

```markdown
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

1. Add `criterion` to `[dev-dependencies]` in `Cargo.toml`:
   ```toml
   criterion = { version = "0.5", features = ["html_reports"] }
   ```
   And add the `[[bench]]` section:
   ```toml
   [[bench]]
   name = "game_tick"
   harness = false
   ```

2. Create `benches/game_tick.rs` with:
   - `bench_game_tick` — end-to-end `game_tick()` at 3 game states:
     - Early: Level 1, Prestige 0
     - Mid: Level 50, Prestige 10
     - Endgame: Level 100, Prestige 50
   - Sub-function benchmarks for each confirmed bottleneck from Phase 1
   - All benchmarks use seeded `ChaCha8Rng` for deterministic state

3. Run benchmarks to establish baseline:
   ```bash
   cargo bench
   ```

### 3b. Simulator Profiling

Add `--profile` flag to `src/bin/simulator.rs`:

1. Add `profile: bool` field to `SimConfig`
2. Add `--profile` to `parse_args()` and `print_usage()`
3. Create `StageProfile` struct to accumulate per-stage timing
4. When `--profile` is active, wrap each `game_tick()` call with `Instant::now()` timing
5. At end of simulation, print per-stage timing breakdown:
   ```
   === Tick Profile (10,000 ticks, P20) ===
   Stage                    Avg (µs)   % Total
   ─────────────────────────────────────────
   Total per tick:          174.3 µs
   ```

Note: Per-stage timing requires modifying `game_tick()` to accept an optional profiler, or timing at the simulator level by running each stage group separately. Prefer the simpler approach: time the full `game_tick()` call at the simulator level, plus time the setup/event-processing around it. Do NOT modify `game_tick()` internals for profiling.

## Phase 4: Verify

1. `make check` must pass
2. `cargo bench` runs without errors
3. Report summary of: findings, auto-fixes applied, items flagged for review, benchmark baselines

## Output

Report the PR URL and final status when done (use `/ship` skill).
```

**Step 2: Verify the file was created**

```bash
cat .claude/skills/perf-audit/SKILL.md | head -5
```

Expected: The YAML frontmatter with name and description.

**Step 3: Commit**

```bash
git add .claude/skills/perf-audit/SKILL.md
git commit -m "feat: add perf-audit skill"
```

---

### Task 2: Add criterion benchmark infrastructure

**Files:**
- Modify: `Cargo.toml` (add criterion dev-dependency and bench target)
- Create: `benches/game_tick.rs`

**Step 1: Add criterion to Cargo.toml**

Add to `[dev-dependencies]`:
```toml
criterion = { version = "0.5", features = ["html_reports"] }
```

Add bench target at the end of the file:
```toml
[[bench]]
name = "game_tick"
harness = false
```

**Step 2: Create the benchmark file**

Create `benches/game_tick.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_state::GameState;
#[allow(deprecated)]
use quest::core::tick::game_tick;
use quest::enhancement::EnhancementProgress;
use quest::haven::Haven;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Create a game state at a given level and prestige rank.
fn make_state(level: u32, prestige: u32) -> (GameState, Haven, EnhancementProgress, quest::deep::DeepState, Achievements) {
    let mut state = GameState::new("Bench".to_string(), 0);
    state.character_level = level;
    state.prestige_rank = prestige;

    // Recalculate derived stats for correct HP/damage at this level
    let derived = DerivedStats::calculate_derived_stats(
        &state.attributes,
        &state.equipment,
        &[0; 7],
    );
    state.combat_state.player_max_hp = derived.max_hp;
    state.combat_state.player_current_hp = derived.max_hp;

    let haven = Haven::default();
    let enhancement = EnhancementProgress::new();
    let deep = quest::deep::DeepState::new();
    let achievements = Achievements::default();

    (state, haven, enhancement, deep, achievements)
}

fn bench_game_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick");

    // Early game: Level 1, P0
    group.bench_function("early_L1_P0", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach) = make_state(1, 0);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            game_tick(
                &mut state, &mut tick_counter, &mut haven,
                &mut enhancement, &mut deep, &mut ach,
                false, &mut rng,
            )
        });
    });

    // Mid game: Level 50, P10
    group.bench_function("mid_L50_P10", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach) = make_state(50, 10);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            game_tick(
                &mut state, &mut tick_counter, &mut haven,
                &mut enhancement, &mut deep, &mut ach,
                false, &mut rng,
            )
        });
    });

    // Endgame: Level 100, P50
    group.bench_function("endgame_L100_P50", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach) = make_state(100, 50);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            game_tick(
                &mut state, &mut tick_counter, &mut haven,
                &mut enhancement, &mut deep, &mut ach,
                false, &mut rng,
            )
        });
    });

    group.finish();
}

criterion_group!(benches, bench_game_tick);
criterion_main!(benches);
```

**Step 3: Verify benchmarks compile and run**

```bash
cargo bench -- --quick
```

Expected: 3 benchmarks run successfully with timing output.

**Step 4: Commit**

```bash
git add Cargo.toml benches/game_tick.rs
git commit -m "perf: add criterion benchmarks for game_tick"
```

---

### Task 3: Add simulator --profile flag

**Files:**
- Modify: `src/bin/simulator.rs`

**Step 1: Add profile field to SimConfig**

Add `profile: bool` to the `SimConfig` struct (default `false`).
Add `--profile` parsing to `parse_args()`.
Add `--profile` line to `print_usage()`.

**Step 2: Add StageProfile struct and timing**

```rust
#[derive(Default)]
struct StageProfile {
    tick_count: u64,
    total_tick_ns: u128,
    min_tick_ns: u128,
    max_tick_ns: u128,
}

impl StageProfile {
    fn record(&mut self, elapsed_ns: u128) {
        self.tick_count += 1;
        self.total_tick_ns += elapsed_ns;
        if self.tick_count == 1 || elapsed_ns < self.min_tick_ns {
            self.min_tick_ns = elapsed_ns;
        }
        if elapsed_ns > self.max_tick_ns {
            self.max_tick_ns = elapsed_ns;
        }
    }

    fn avg_us(&self) -> f64 {
        if self.tick_count == 0 { return 0.0; }
        (self.total_tick_ns as f64 / self.tick_count as f64) / 1000.0
    }

    fn min_us(&self) -> f64 {
        self.min_tick_ns as f64 / 1000.0
    }

    fn max_us(&self) -> f64 {
        self.max_tick_ns as f64 / 1000.0
    }
}
```

**Step 3: Wrap game_tick() with timing in simulation loop**

In `run_simulation()`, when `config.profile` is true, wrap the `game_tick()` call:

```rust
let profile = if config.profile {
    let start = std::time::Instant::now();
    let result = game_tick(...);
    let elapsed = start.elapsed().as_nanos();
    stage_profile.record(elapsed);
    result
} else {
    game_tick(...)
};
```

Return `StageProfile` alongside `SimStats`.

**Step 4: Print profile summary**

After simulation completes, if `config.profile`, print:

```
=== Tick Profile ({ticks} ticks, P{prestige}) ===
Metric               Value
─────────────────────────────
Avg per tick:        XX.X µs
Min per tick:        XX.X µs
Max per tick:        XX.X µs
Total wall time:     X.XXX s
Ticks/second:        XXX,XXX
```

**Step 5: Verify**

```bash
cargo run --release --bin simulator -- --profile --ticks 10000 --prestige 20 --quiet
```

Expected: Profile summary printed after simulation.

**Step 6: Commit**

```bash
git add src/bin/simulator.rs
git commit -m "perf: add --profile flag to simulator for tick timing"
```

---

### Task 4: Update root CLAUDE.md skills table

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add perf-audit to skills table**

Add row to the skills table in root `CLAUDE.md`:

```markdown
| `perf-audit` | "audit performance", "optimize hot paths", "profile the game" | Multi-agent perf audit: finds bottlenecks, auto-fixes, adds benchmarks |
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add perf-audit skill to CLAUDE.md skills table"
```

---

### Task 5: Verify everything works

**Step 1: Run make check**

```bash
make check
```

Expected: All checks pass (fmt, clippy, test, build, audit).

**Step 2: Run benchmarks**

```bash
cargo bench -- --quick
```

Expected: 3 game_tick benchmarks run successfully.

**Step 3: Run simulator with --profile**

```bash
cargo run --release --bin simulator -- --profile --ticks 10000 --quiet
```

Expected: Profile summary printed.

**Step 4: Commit any fixes if needed, then ship**

Use `/ship` skill to push, create PR, and watch CI.
