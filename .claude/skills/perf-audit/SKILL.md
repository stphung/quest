---
name: perf-audit
description: Multi-agent performance audit — finds and fixes hot-path bottlenecks, adds criterion benchmarks and simulator profiling. Use when game feels slow, after adding features, or to establish performance baselines.
---

<!-- meta-audit scope-universe: src/*/ (thematic — flags a new top-level module with no agent assigned, not every file) -->

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

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 3,
     "scope": ["src/core/", "src/ui/"],
     "findings": [
       {
         "location": "src/core/tick_stages.rs:331",
         "claim": "...",
         "correct_value": "...",
         "severity": "MEDIUM",
         "category": "per-tick-allocation",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh perf-audit /tmp/perf-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh perf-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `perf-audit` next. If it
   prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.
