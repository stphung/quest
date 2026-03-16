# Balance Simulator Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing game simulator to support full-lifecycle balance validation with strategy profiles, outcome injection, and configurable assertions.

**Architecture:** Split the monolithic `src/bin/simulator.rs` into a multi-file binary at `src/bin/simulator/`. Migrate from the deprecated `game_tick()` to `game_tick_with_context()` with persistent `TickContext`. Add strategy profiles that inject outcomes (challenge wins, enhancement, sigils, ascension, prestige) after each tick, and balance assertions for CI.

**Tech Stack:** Rust, rand/rand_chacha (seeded RNG), quest crate (game logic)

**Spec:** `docs/superpowers/specs/2026-03-15-balance-simulator-design.md`

---

## File Structure

```
src/bin/simulator/
├── main.rs           # Entry point, CLI parsing, tick loop, CSV, report orchestration
├── strategy.rs       # StrategyProfile enum, per-system rules, inject_outcomes()
├── assertions.rs     # Assertion struct, built-in assertions, check/report logic
├── stats.rs          # SimStats, TickProfile (extracted from current simulator.rs)
├── report.rs         # print_summary, print_multi_run_summary, print_final_equipment, helpers
```

The old `src/bin/simulator.rs` is deleted entirely. All its code is redistributed across the 5 new files.

---

## Chunk 1: Extract and Migrate

### Task 1: Create `src/bin/simulator/` directory and extract `stats.rs`

Extract `SimStats`, `TickProfile`, and their impls into a dedicated stats module. This is a pure extraction — no behavior change.

**Files:**
- Delete: `src/bin/simulator.rs`
- Create: `src/bin/simulator/main.rs`
- Create: `src/bin/simulator/stats.rs`
- Create: `src/bin/simulator/report.rs`

- [ ] **Step 1: Create the directory structure**

```bash
mkdir -p src/bin/simulator
```

- [ ] **Step 2: Create `stats.rs` with `SimStats` and `TickProfile`**

Extract from the current `simulator.rs` (lines 248–461 — the structs, their `Default` impls, and `SimStats` methods). Add `pub` visibility to everything that `main.rs` will use.

```rust
// src/bin/simulator/stats.rs
use quest::character::attributes::AttributeType;
use quest::core::game_state::GameState;
use quest::core::tick::TickEvent;
use quest::core::tick::TickResult;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SimStats {
    pub total_ticks: u64,
    pub total_kills: u64,
    pub total_deaths: u64,
    pub total_boss_kills: u64,
    pub total_crits: u64,
    pub total_xp_gained: u64,
    pub level_at_tick: HashMap<u32, u64>,
    pub zone_entry_tick: HashMap<(u32, u32), u64>,
    pub zone_boss_defeated_tick: HashMap<(u32, u32), u64>,
    pub deaths_per_zone: HashMap<(u32, u32), u64>,
    pub items_by_rarity: [u64; 5],
    pub items_equipped: u64,
    pub boss_items_dropped: u64,
    pub fish_caught: u64,
    pub fishing_rank_ups: u64,
    pub dungeons_completed: u64,
    pub dungeons_failed: u64,
    pub dungeons_discovered: u64,
    pub achievements_unlocked: u64,
    pub haven_discovered: bool,
    pub haven_rooms_built: u32,
    pub haven_prestige_spent: u32,
    pub haven_final_tiers: Vec<(String, u8)>,
    pub final_level: u32,
    pub final_xp: u64,
    pub final_prestige: u32,
    pub final_zone: (u32, u32),
    pub final_fishing_rank: u32,
    pub final_attributes: [u32; 6],
}
// ... Default impl, record_zone_entry, process_tick, finalize — identical to current code
```

Also extract `TickProfile` with its `record()`, `avg_us()`, `min_us()`, `max_us()` methods — identical to current lines 318–352.

- [ ] **Step 3: Create `report.rs` with all print/output functions**

Extract from current `simulator.rs`:
- `ticks_to_time()` (lines 691–703)
- `print_summary()` (lines 705–888)
- `print_multi_run_summary()` (lines 891–1017)
- `print_profile()` (lines 1020–1052)
- `print_final_equipment()` (lines 1120–1138)
- `print_tick_events()` (lines 631–687)

All functions become `pub`. They import `SimStats`, `TickProfile` from `super::stats`.

```rust
// src/bin/simulator/report.rs
use super::stats::{SimStats, TickProfile};
use quest::core::game_state::GameState;
use quest::core::tick::{TickEvent, TickResult};

pub fn ticks_to_time(ticks: u64) -> String { /* identical */ }
pub fn print_summary(stats: &SimStats, seed: u64, config: &super::SimConfig) { /* identical */ }
pub fn print_multi_run_summary(all_stats: &[SimStats]) { /* identical */ }
pub fn print_profile(profile: &TickProfile, config: &super::SimConfig) { /* identical */ }
pub fn print_final_equipment(state: &GameState) { /* identical */ }
pub fn print_tick_events(tick: u64, result: &TickResult) { /* identical */ }
```

- [ ] **Step 4: Create `main.rs` with the remaining code**

Contains: `pub SimConfig` (must be `pub` so `report.rs` can reference `super::SimConfig`), `parse_args()`, `print_usage()`, `auto_build_haven()`, `run_simulation()`, `main()`. Imports from `mod stats` and `mod report`. Keep `HavenStrategy` here for now (it will be replaced by `StrategyProfile` in Task 3).

```rust
// src/bin/simulator/main.rs
mod stats;
mod report;

use stats::{SimStats, TickProfile};
// ... rest of imports, SimConfig, parse_args, run_simulation, main
// Identical logic, just referencing report:: and stats:: instead of local fns
```

- [ ] **Step 5: Delete the old `src/bin/simulator.rs`**

```bash
rm src/bin/simulator.rs
```

Cargo automatically discovers `src/bin/simulator/main.rs` as the binary entry point with the same name.

- [ ] **Step 6: Verify build and existing behavior**

Run:
```bash
cargo build --bin simulator 2>&1 | tail -5
```
Expected: builds successfully.

Run:
```bash
cargo run --bin simulator -- --ticks 1000 --seed 42 --quiet
```
Expected: identical output to before the split.

Run:
```bash
cargo test 2>&1 | grep -E "^test result:"
```
Expected: all tests pass (the simulator has no unit tests — this verifies no lib regressions).

- [ ] **Step 7: Commit**

```bash
git rm src/bin/simulator.rs && git add src/bin/simulator/
git commit -m "refactor: split simulator.rs into multi-file binary

Extract SimStats/TickProfile into stats.rs, all print functions into
report.rs. main.rs retains CLI, config, simulation loop. No behavior
change — same binary name, same output."
```

---

### Task 2: Migrate from `game_tick()` to `game_tick_with_context()`

Replace the deprecated `game_tick()` call with `game_tick_with_context()` using a persistent `TickContext`. This adds `LoomState` accumulation.

**Files:**
- Modify: `src/bin/simulator/main.rs`

- [ ] **Step 1: Update imports**

Replace:
```rust
#[allow(deprecated)]
use quest::core::tick::{game_tick, TickEvent, TickResult};
```
With:
```rust
use quest::core::tick::{game_tick_with_context, TickEvent, TickResult};
use quest::core::tick_context::TickContext;
use quest::loom::LoomState;
```

- [ ] **Step 2: Add `LoomState` allocation in `run_simulation()`**

After the existing `let mut achievements = Achievements::default();` line, add:

```rust
let mut loom = LoomState::new();
```

- [ ] **Step 3: Replace `game_tick()` calls with `game_tick_with_context()`**

Replace the tick call block (both the profiled and non-profiled paths). The current code passes 8 separate args; the new code builds a `TickContext` and passes it:

```rust
let mut ctx = TickContext {
    state: &mut state,
    tick_counter: &mut tick_counter,
    haven: &mut haven,
    enhancement: &mut enhancement,
    deep: &mut deep_state,
    achievements: &mut achievements,
    loom: &mut loom,
    debug_mode: false,
};
let result = game_tick_with_context(&mut ctx, &mut rng);
```

For the profiled path, wrap identically with `Instant::now()` / `profile.record()`.

- [ ] **Step 4: Fix borrow issues after TickContext drop**

The `TickContext` borrows `state`, `haven`, etc. mutably. After the tick call, the context must be dropped before we access those again. Since `ctx` is a local in the block, it drops at the end of the `let result = ...` expression. But the Haven auto-build code needs `&mut haven` and `&mut state.prestige_rank`. Ensure the `ctx` variable is scoped so it doesn't overlap:

```rust
let result = {
    let mut ctx = TickContext { /* ... */ };
    if let Some(ref mut profile) = tick_profile {
        let start = std::time::Instant::now();
        let r = game_tick_with_context(&mut ctx, &mut rng);
        profile.record(start.elapsed().as_nanos());
        r
    } else {
        game_tick_with_context(&mut ctx, &mut rng)
    }
};
// ctx is dropped here — state, haven, etc. are free to borrow again
```

- [ ] **Step 5: Remove the `#[allow(deprecated)]` attribute**

It's no longer needed since we're not calling `game_tick()`.

- [ ] **Step 6: Verify build and behavior**

Run:
```bash
cargo build --bin simulator 2>&1 | tail -5
```
Expected: builds without deprecation warnings.

Run:
```bash
cargo run --bin simulator -- --ticks 1000 --seed 42 --quiet
```
Expected: output may differ slightly (Loom state now accumulates), but should produce valid results.

Run:
```bash
cargo run --bin simulator -- --ticks 36000 --seed 42 --haven balanced --quiet
```
Expected: runs to completion without panics.

- [ ] **Step 7: Commit**

```bash
git add src/bin/simulator/main.rs
git commit -m "feat: migrate simulator to game_tick_with_context()

Replace deprecated game_tick() with game_tick_with_context() and a
persistent TickContext. LoomState now accumulates across ticks,
enabling Loom-dependent balance validation."
```

---

## Chunk 2: Strategy Profiles and Outcome Injection

### Task 3: Create `strategy.rs` with profile definitions and `inject_outcomes()`

This is the core new functionality. Defines the three strategy profiles and the outcome injection logic.

**Files:**
- Create: `src/bin/simulator/strategy.rs`
- Modify: `src/bin/simulator/main.rs` (add `mod strategy`, wire into tick loop)

- [ ] **Step 1: Define the `StrategyProfile` enum and per-system config structs**

```rust
// src/bin/simulator/strategy.rs
use quest::haven::HavenRoomId;

#[derive(Debug, Clone, Copy)]
pub enum StrategyProfile {
    Casual,
    Optimal,
    Speedrun,
}

impl StrategyProfile {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "casual" => Some(Self::Casual),
            "optimal" => Some(Self::Optimal),
            "speedrun" => Some(Self::Speedrun),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Casual => "casual",
            Self::Optimal => "optimal",
            Self::Speedrun => "speedrun",
        }
    }

    /// Haven room build priority order.
    pub fn haven_priority(&self) -> &'static [HavenRoomId] {
        match self {
            Self::Casual => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Bedroom,
                HavenRoomId::Garden,
                HavenRoomId::Library,
                HavenRoomId::FishingDock,
                HavenRoomId::Workshop,
                HavenRoomId::Vault,
            ],
            Self::Optimal => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::Bedroom,
                HavenRoomId::TrainingYard,
                HavenRoomId::Garden,
                HavenRoomId::TrophyHall,
                HavenRoomId::Library,
                HavenRoomId::Watchtower,
                HavenRoomId::FishingDock,
                HavenRoomId::AlchemyLab,
                HavenRoomId::Workshop,
                HavenRoomId::WarRoom,
                HavenRoomId::Vault,
            ],
            Self::Speedrun => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::Bedroom,
                HavenRoomId::TrainingYard,
                HavenRoomId::Garden,
                HavenRoomId::TrophyHall,
                HavenRoomId::Library,
                HavenRoomId::Watchtower,
                HavenRoomId::FishingDock,
                HavenRoomId::AlchemyLab,
                HavenRoomId::Workshop,
                HavenRoomId::WarRoom,
                HavenRoomId::Vault,
                HavenRoomId::StormForge,
            ],
        }
    }

    /// Ticks between challenge win injections.
    pub fn challenge_interval_ticks(&self) -> u64 {
        match self {
            Self::Casual => 36_000,   // 1 per hour
            Self::Optimal => 12_000,  // 3 per hour
            Self::Speedrun => 6_000,  // 6 per hour
        }
    }

    /// Prestige rank threshold to trigger enhancement injection.
    pub fn enhancement_pr_threshold(&self) -> u32 {
        match self {
            Self::Casual => 20,
            Self::Optimal => 15,
            Self::Speedrun => 10,
        }
    }

    /// Target enhancement level per slot.
    pub fn enhancement_target_level(&self) -> u8 {
        match self {
            Self::Casual => 4,
            Self::Optimal => 7,
            Self::Speedrun => 10,
        }
    }

    /// Stormglass threshold to trigger sigil etching.
    pub fn stormglass_threshold(&self) -> u64 {
        match self {
            Self::Casual => 5000,
            Self::Optimal => 3000,
            Self::Speedrun => 1500,
        }
    }

    /// Stormglass cost deducted per sigil etch cycle.
    pub fn stormglass_cost(&self) -> u64 {
        match self {
            Self::Casual => 2000,
            Self::Optimal => 3000,
            Self::Speedrun => 4000,
        }
    }

    /// Ticks of no zone progress before auto-prestige.
    pub fn prestige_stuck_ticks(&self) -> u64 {
        match self {
            Self::Casual => 18_000,
            Self::Optimal => 9_000,
            Self::Speedrun => 3_600,
        }
    }
}
```

- [ ] **Step 2: Define `InjectionState` to track injection bookkeeping**

```rust
/// Mutable state used by inject_outcomes() across ticks.
pub struct InjectionState {
    pub last_challenge_tick: u64,
    pub enhancement_applied: bool,
    pub sigils_etched: bool,
    pub last_new_zone_tick: u64,
    pub last_zone_id: u32,
}

impl InjectionState {
    pub fn new() -> Self {
        Self {
            last_challenge_tick: 0,
            enhancement_applied: false,
            sigils_etched: false,
            last_new_zone_tick: 0,
            last_zone_id: 1,
        }
    }
}
```

- [ ] **Step 3: Implement `inject_outcomes()`**

This is the core injection function called after each tick. It checks each system's trigger conditions and mutates state accordingly.

```rust
use quest::achievements::milestones::{MinigameDifficulty, MinigameType};
use quest::achievements::Achievements;
use quest::ascension::logic::{ascend, can_ascend};
use quest::character::prestige_actions::{can_prestige, perform_prestige};
use quest::core::game_state::GameState;
use quest::deep::DeepState;
use quest::enhancement::EnhancementProgress;
use quest::stormglass::sigils::{roll_sigil, SigilEffectType, SigilGrade};
use quest::zones::sync_account_zone_unlocks;

pub fn inject_outcomes(
    profile: &StrategyProfile,
    state: &mut GameState,
    enhancement: &mut EnhancementProgress,
    deep: &mut DeepState,
    achievements: &mut Achievements,
    loom: &quest::loom::LoomState,
    injection: &mut InjectionState,
    tick: u64,
    verbose: bool,
) {
    // -- Challenge wins --
    if tick > 0
        && tick - injection.last_challenge_tick >= profile.challenge_interval_ticks()
    {
        // Rotate through game types for variety
        let game_types = [
            MinigameType::Chess, MinigameType::Minesweeper,
            MinigameType::Rune, MinigameType::Go,
        ];
        let idx = ((tick / profile.challenge_interval_ticks()) as usize) % game_types.len();
        let game_type = game_types[idx];
        let difficulty = match profile {
            StrategyProfile::Casual => MinigameDifficulty::Novice,
            StrategyProfile::Optimal => MinigameDifficulty::Journeyman,
            StrategyProfile::Speedrun => MinigameDifficulty::Master,
        };

        achievements.on_minigame_won(game_type, difficulty, Some("Simulator"));

        // Apply challenge rewards directly (PR + Stormglass)
        let reward = challenge_reward_for_difficulty(&difficulty);
        state.prestige_rank += reward.0;
        state.stormglass += reward.1;

        injection.last_challenge_tick = tick;

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Challenge win ({game_type:?}/{difficulty:?}) +{} PR +{} SG",
                reward.0, reward.1
            );
        }
    }

    // -- Soulforge enhancement --
    if !injection.enhancement_applied
        && state.prestige_rank >= profile.enhancement_pr_threshold()
    {
        let target = profile.enhancement_target_level();
        for slot in 0..7 {
            enhancement.levels[slot] = target;
        }
        injection.enhancement_applied = true;
        state.invalidate_bonuses();

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Enhancement all slots -> +{target}"
            );
        }
    }

    // -- Stormglass sigil spending --
    if !injection.sigils_etched
        && state.stormglass >= profile.stormglass_threshold()
    {
        inject_sigils(profile, state);
        injection.sigils_etched = true;

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Sigils etched ({} profile), {} SG deducted",
                profile.name(),
                profile.stormglass_cost()
            );
        }
    }

    // -- Ascension --
    let deepest_layer = deep.persistent.max_layer_reached;
    let completed_patterns = loom.persistent.completed_pattern_count();
    if can_ascend(
        state.ascension_level,
        state.prestige_rank,
        deepest_layer,
        completed_patterns,
    ) {
        let result = ascend(state, deepest_layer, completed_patterns);
        if let quest::ascension::logic::AscendResult::Success { new_level, multiplier } = &result {
            // Sync zone unlocks after ascension — recompute loom_zone_cap fresh
            let storms_end_unlocked = achievements
                .is_unlocked(quest::achievements::AchievementId::TheStormbreaker);
            let loom_zone_cap = quest::loom::loom_zone_cap_for_patterns(
                loom.persistent.completed_pattern_count(),
            );
            sync_account_zone_unlocks(
                &mut state.zone_progression,
                storms_end_unlocked,
                deep.persistent.fracture_zone_cap,
                state.prestige_rank,
                loom_zone_cap,
                state.ascension_level,
            );
            state.invalidate_bonuses();

            if verbose {
                println!(
                    "[t={tick:>6}] INJECT: Ascension -> level {new_level} ({multiplier:.0}x)"
                );
            }
        }
    }

    // -- Auto-prestige (stuck detection) --
    let current_zone = state.zone_progression.current_zone_id;
    if current_zone != injection.last_zone_id {
        injection.last_new_zone_tick = tick;
        injection.last_zone_id = current_zone;
    }

    if tick - injection.last_new_zone_tick >= profile.prestige_stuck_ticks()
        && can_prestige(state)
    {
        perform_prestige(state);
        injection.last_new_zone_tick = tick;
        injection.last_zone_id = state.zone_progression.current_zone_id;
        state.invalidate_bonuses();

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Auto-prestige -> P{}",
                state.prestige_rank
            );
        }
    }
}

/// Challenge rewards by difficulty. PR values reflect that most game
/// types award 0 PR at lower difficulties; we use conservative averages
/// across game types. SG values approximate the mid-range for each tier.
fn challenge_reward_for_difficulty(diff: &MinigameDifficulty) -> (u32, u64) {
    match diff {
        MinigameDifficulty::Novice => (0, 500),
        MinigameDifficulty::Apprentice => (0, 1500),
        MinigameDifficulty::Journeyman => (1, 3000),
        MinigameDifficulty::Master => (1, 6000),
    }
}

/// Etch sigils per profile spec.
fn inject_sigils(profile: &StrategyProfile, state: &mut GameState) {
    use SigilEffectType::*;
    use SigilGrade::*;

    let (effects, grade): (&[SigilEffectType], SigilGrade) = match profile {
        StrategyProfile::Casual => (
            &[DamagePercent, DamageReductionPercent],
            C,
        ),
        StrategyProfile::Optimal => (
            &[DamagePercent, DamageReductionPercent, CritChancePercent],
            A,
        ),
        StrategyProfile::Speedrun => (
            &[DamagePercent, CritChancePercent, MaxHpPercent],
            SPlus,
        ),
    };

    // Ensure enough slots
    let needed = effects.len() as u8;
    if state.storm_sigils.slots_unlocked < needed {
        state.storm_sigils.slots_unlocked = needed;
    }

    // Resize sigils vec if needed
    while state.storm_sigils.sigils.len() < needed as usize {
        state.storm_sigils.sigils.push(None);
    }

    // Use roll_sigil with a grade-midpoint roll value
    let uniform_roll = match grade {
        C => 0.50,    // mid-range C grade
        A => 0.85,    // mid-range A grade
        SPlus => 0.99, // top of S+ range
        _ => 0.50,
    };

    for (i, &effect) in effects.iter().enumerate() {
        let mut sigil = roll_sigil(effect, uniform_roll);
        sigil.grade = grade; // Override grade to match profile spec
        state.storm_sigils.sigils[i] = Some(sigil);
    }

    state.stormglass = state.stormglass.saturating_sub(profile.stormglass_cost());
}
```

- [ ] **Step 4: Verify `strategy.rs` compiles**

Add `mod strategy;` to `main.rs` and run:
```bash
cargo build --bin simulator 2>&1 | tail -10
```
Expected: compiles. Fix any import issues.

- [ ] **Step 5: Commit**

```bash
git add src/bin/simulator/strategy.rs src/bin/simulator/main.rs
git commit -m "feat: add strategy profiles and outcome injection

Three profiles (casual/optimal/speedrun) define milestone-triggered
rules for challenges, enhancement, sigils, ascension, and prestige.
inject_outcomes() runs after each game tick to mutate state."
```

---

### Task 4: Wire strategy profiles into the tick loop and replace `--haven`

Replace `HavenStrategy` and `--haven` flag with `StrategyProfile` and `--strategy`. Wire `inject_outcomes()` into the tick loop with PR snapshot tracking.

**Files:**
- Modify: `src/bin/simulator/main.rs`

- [ ] **Step 1: Replace `HavenStrategy` with `StrategyProfile` in `SimConfig`**

Remove the `HavenStrategy` enum entirely. Update `SimConfig`:

```rust
struct SimConfig {
    ticks: u64,
    seed: u64,
    prestige: u32,
    runs: u32,
    verbose: bool,
    csv_path: Option<String>,
    quiet: bool,
    stormbreaker: bool,
    strategy: Option<strategy::StrategyProfile>,  // was haven_strategy
    profile: bool,
    assertions: bool,  // new
}
```

- [ ] **Step 2: Update `parse_args()` to replace `--haven` with `--strategy`**

Replace the `"--haven"` match arm with:
```rust
"--strategy" => {
    i += 1;
    if i >= args.len() {
        eprintln!("--strategy requires a profile: casual, optimal, speedrun");
        std::process::exit(1);
    }
    config.strategy = Some(
        strategy::StrategyProfile::from_str(&args[i]).unwrap_or_else(|| {
            eprintln!(
                "Unknown strategy: {}. Options: casual, optimal, speedrun",
                args[i]
            );
            std::process::exit(1);
        }),
    );
}
"--assertions" => config.assertions = true,
```

- [ ] **Step 3: Update `print_usage()` to reflect new CLI**

Replace `--haven` line with:
```
\x20 --strategy STR  Strategy profile: casual, optimal, speedrun\n\
\x20 --assertions    Run balance assertions and exit with pass/fail\n\
```

- [ ] **Step 4: Wire injection into `run_simulation()`**

After the existing tick call and zone-change detection, add:

```rust
// PR snapshot tracking
let pr_before = state.prestige_rank;

// ... existing tick call ...

// Haven auto-build (now uses strategy priority)
if let Some(ref strat) = config.strategy {
    if haven.discovered {
        if let Some((room, new_tier, cost)) =
            auto_build_haven(&mut haven, &mut state.prestige_rank, strat.haven_priority())
        {
            state.invalidate_bonuses();
            stats.haven_rooms_built += 1;
            stats.haven_prestige_spent += cost;
            if config.verbose { /* ... */ }
        }
    }

    // Outcome injection
    let prev_challenge_tick = injection_state.last_challenge_tick;
    strategy::inject_outcomes(
        strat,
        &mut state,
        &mut enhancement,
        &mut deep_state,
        &mut achievements,
        &loom,
        &mut injection_state,
        tick,
        config.verbose,
    );
    // Track challenge wins (detect when last_challenge_tick changed)
    if injection_state.last_challenge_tick != prev_challenge_tick {
        stats.challenges_won += 1;
    }
}

// PR delta tracking
let pr_after = state.prestige_rank;
if pr_after > pr_before {
    stats.pr_earned += (pr_after - pr_before) as u64;
} else if pr_before > pr_after {
    stats.pr_spent += (pr_before - pr_after) as u64;
}
```

Add `InjectionState` initialization before the loop:
```rust
let mut injection_state = strategy::InjectionState::new();
```

- [ ] **Step 5: Add PR tracking fields to `SimStats`**

In `stats.rs`, add to `SimStats`:
```rust
pub pr_earned: u64,
pub pr_spent: u64,
pub ascension_level: u32,
pub challenges_won: u64,
pub stormglass_balance: u64,
```

Update `Default` impl and `finalize()` to populate `ascension_level` and `stormglass_balance` from final state.

- [ ] **Step 6: Update `report.rs` to display new stats**

In `print_summary()`, add an "Economy Flow" section after the existing sections:
```rust
// Economy flow (only when strategy is active)
if stats.pr_earned > 0 || stats.pr_spent > 0 {
    println!("--- Economy Flow ---");
    println!(
        "PR earned: {}  |  PR spent: {}  |  Net: {}",
        stats.pr_earned,
        stats.pr_spent,
        stats.pr_earned as i64 - stats.pr_spent as i64
    );
    if stats.ascension_level > 0 {
        println!("Ascension level: {}", stats.ascension_level);
    }
    if stats.challenges_won > 0 {
        println!("Challenges won: {}", stats.challenges_won);
    }
    if stats.stormglass_balance > 0 {
        println!("Stormglass balance: {}", stats.stormglass_balance);
    }
    println!();
}
```

Update the quiet one-liner and multi-run summary to include the new fields.

- [ ] **Step 7: Update CSV header and row writing**

Add new columns to the CSV header:
```
...,ascension_level,enhancement_avg,stormglass_balance,challenges_won,pr_earned,pr_spent
```

And the corresponding values in the row writer.

- [ ] **Step 8: Update Haven strategy name in report**

Replace the Haven section's strategy name display to use `strategy.name()` instead of the old `haven_strategy.name()`.

- [ ] **Step 9: Verify build and new behavior**

```bash
cargo build --bin simulator 2>&1 | tail -5
```

Test without strategy (should work as before):
```bash
cargo run --bin simulator -- --ticks 1000 --seed 42 --quiet
```

Test with strategy:
```bash
cargo run --bin simulator -- --ticks 36000 --seed 42 --strategy optimal --verbose 2>&1 | head -50
```
Expected: should see INJECT log lines for challenges, enhancement, etc.

Test prestige + strategy:
```bash
cargo run --bin simulator -- --ticks 360000 --seed 42 --prestige 20 --strategy speedrun --quiet
```
Expected: runs to completion, shows non-zero PR earned/spent.

- [ ] **Step 10: Commit**

```bash
git add src/bin/simulator/
git commit -m "feat: wire strategy profiles into simulator tick loop

Replace --haven flag with --strategy (casual/optimal/speedrun).
inject_outcomes() runs after each tick when a strategy is active.
PR tracking via prestige_rank snapshot deltas. New CSV columns:
ascension_level, enhancement_avg, stormglass_balance, challenges_won,
pr_earned, pr_spent."
```

---

## Chunk 3: Assertions and Cleanup

### Task 5: Create `assertions.rs` with balance assertions

**Files:**
- Create: `src/bin/simulator/assertions.rs`
- Modify: `src/bin/simulator/main.rs` (add `mod assertions`, wire into main)

- [ ] **Step 1: Define assertion types**

```rust
// src/bin/simulator/assertions.rs
use super::stats::SimStats;

#[derive(Debug)]
pub enum AssertionOp {
    LessOrEqual,
    GreaterOrEqual,
    Equal,
}

#[derive(Debug)]
pub struct Assertion {
    pub name: &'static str,
    pub metric: fn(&SimStats) -> f64,
    pub op: AssertionOp,
    pub value: f64,
}

impl Assertion {
    pub fn check(&self, stats: &SimStats) -> bool {
        let actual = (self.metric)(stats);
        match self.op {
            AssertionOp::LessOrEqual => actual <= self.value,
            AssertionOp::GreaterOrEqual => actual >= self.value,
            AssertionOp::Equal => (actual - self.value).abs() < f64::EPSILON,
        }
    }
}
```

- [ ] **Step 2: Define built-in assertions**

```rust
pub fn builtin_assertions() -> Vec<Assertion> {
    vec![
        Assertion {
            name: "Zone 5 reachable within 30min at P0",
            metric: |s| {
                s.zone_entry_tick
                    .keys()
                    .filter(|(z, _)| *z >= 5)
                    .filter_map(|k| s.zone_entry_tick.get(k))
                    .copied()
                    .min()
                    .unwrap_or(u64::MAX) as f64
            },
            op: AssertionOp::LessOrEqual,
            value: 18_000.0, // 30 min in ticks
        },
        Assertion {
            name: "Level 50 reachable within 1hr at P0",
            metric: |s| {
                s.level_at_tick
                    .get(&50)
                    .copied()
                    .unwrap_or(u64::MAX) as f64
            },
            op: AssertionOp::LessOrEqual,
            value: 36_000.0, // 1 hour in ticks
        },
        Assertion {
            name: "PR income exceeds PR spending by tick 50000",
            metric: |s| {
                if s.total_ticks >= 50_000 {
                    s.pr_earned as f64 - s.pr_spent as f64
                } else {
                    1.0 // pass if sim didn't run long enough
                }
            },
            op: AssertionOp::GreaterOrEqual,
            value: 0.0,
        },
    ]
}
```

- [ ] **Step 3: Add `run_assertions()` function**

```rust
pub fn run_assertions(stats: &SimStats) -> bool {
    let assertions = builtin_assertions();
    let mut all_pass = true;

    println!();
    println!("=== Balance Assertions ===");
    println!();

    for assertion in &assertions {
        let passed = assertion.check(stats);
        let actual = (assertion.metric)(stats);
        let status = if passed { "PASS" } else { "FAIL" };
        let icon = if passed { "\u{2714}" } else { "\u{2718}" };

        println!("{icon} [{status}] {}", assertion.name);
        if !passed {
            println!(
                "         actual={actual:.0}, expected {:?} {:.0}",
                assertion.op, assertion.value
            );
            all_pass = false;
        }
    }

    println!();
    if all_pass {
        println!("All assertions passed.");
    } else {
        println!("Some assertions FAILED.");
    }

    all_pass
}
```

- [ ] **Step 4: Wire assertions into `main()`**

In `main()`, after printing the summary, add:
```rust
if config.assertions {
    let pass = if config.runs == 1 {
        assertions::run_assertions(&all_stats[0])
    } else {
        // For multi-run: assert on each run, fail if any fails
        all_stats.iter().all(|s| assertions::run_assertions(s))
    };
    if !pass {
        std::process::exit(1);
    }
}
```

- [ ] **Step 5: Verify assertions**

Run with assertions on a short sim (should pass trivially or show meaningful output):
```bash
cargo run --bin simulator -- --ticks 36000 --seed 42 --strategy optimal --assertions
```

Run with assertions that should fail (too few ticks):
```bash
cargo run --bin simulator -- --ticks 100 --seed 42 --assertions
echo "Exit code: $?"
```
Expected: exit code 1 (zone 5 not reached in 100 ticks).

- [ ] **Step 6: Commit**

```bash
git add src/bin/simulator/assertions.rs src/bin/simulator/main.rs
git commit -m "feat: add balance assertions to simulator

Built-in assertions check zone reachability, level progression, and
PR economy health. --assertions flag prints PASS/FAIL and exits
non-zero on failure for CI integration."
```

---

### Task 6: Update docs and CLAUDE.md references

**Files:**
- Modify: `CLAUDE.md` (update simulator description)
- Modify: `docs/infrastructure.md` (if it references `--haven`)

- [ ] **Step 1: Update `CLAUDE.md` simulator description**

Replace the Game Simulator entry:
```
**Game Simulator** (`src/bin/simulator/`): Headless game balance simulator calling `game_tick_with_context()` with no UI/delay. Supports `--ticks`, `--seed`, `--prestige`, `--runs`, `--strategy <profile>` (casual/optimal/speedrun), `--stormbreaker`, `--assertions`. Strategy profiles inject challenge wins, enhancement, sigils, ascension, and auto-prestige.
```

- [ ] **Step 2: Update any `--haven` references in docs**

Check `docs/infrastructure.md` and update any `--haven` references to `--strategy`.

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md docs/
git commit -m "docs: update simulator references for --strategy flag

Replace --haven references with --strategy in CLAUDE.md and docs."
```

---

### Task 7: Final verification

- [ ] **Step 1: Run `make check`**

```bash
make check
```
Expected: all CI checks pass (format, clippy, tests, build, audit).

- [ ] **Step 2: Run a full balance simulation**

```bash
cargo run --bin simulator -- --ticks 360000 --seed 42 --prestige 10 --strategy optimal --csv /tmp/balance.csv --assertions
```
Expected: runs to completion, assertions report printed, CSV written.

- [ ] **Step 3: Verify CSV output has new columns**

```bash
head -1 /tmp/balance.csv
```
Expected: header includes `ascension_level,enhancement_avg,stormglass_balance,challenges_won,pr_earned,pr_spent`.

- [ ] **Step 4: Run all three profiles**

```bash
for p in casual optimal speedrun; do
  echo "=== $p ==="
  cargo run --bin simulator -- --ticks 360000 --seed 42 --prestige 10 --strategy $p --quiet
done
```
Expected: different progression numbers per profile. Speedrun should reach further than casual.

- [ ] **Step 5: Commit any final fixes**

If any issues were found in steps 1-4, fix and commit.
