# Balance Simulator Design

Extend the existing game simulator (`src/bin/simulator.rs`) to support full-lifecycle balance validation by adding strategy profiles, outcome injection for interactive systems, and configurable assertions.

## Goals

1. **Progression curve validation** — Simulate P0 through endgame with all systems active, measuring time-to-zone, time-to-prestige, economy flow
2. **Economy consistency** — Verify PR income vs spending, Stormglass flow, enhancement cost curves through actual simulation data
3. **Regression detection** — Configurable assertions that fail CI when balance changes break expected ranges

Constant drift detection (code vs docs vs wiki) is explicitly out of scope — handled by existing doc-audit and wiki-audit skills.

## Approach

Extend the existing simulator binary (Approach A). No new binary — the simulator already runs the game's tick loop with seeded RNG, CSV export, multi-run support, and Haven auto-building. We generalize the Haven strategy pattern to cover all interactive systems.

### Prerequisite: Migrate to `game_tick_with_context()`

The current simulator calls the deprecated `game_tick()` function, which internally creates a throwaway `LoomState` each tick. This means Loom state (patterns, shuttles, WR→PR conversion, zone unlocks) is never accumulated. Before adding strategy profiles, the simulator must be migrated to `game_tick_with_context()` with a persistent `TickContext` that includes a real `LoomState` alongside the existing `Haven`, `EnhancementProgress`, and `DeepState` allocations.

## Design

### 1. Strategy Profiles

A new `--strategy <profile>` flag replaces the existing `--haven <strategy>` flag. Each profile defines milestone-triggered rules for all interactive systems.

#### Three Profiles

| Profile | Philosophy | Target audience |
|---|---|---|
| `casual` | Slow spending, conservative enhancement, few challenge wins | "What does the game feel like for a relaxed player?" |
| `optimal` | Efficient spending, smart enhancement timing, steady challenge wins | "What does ideal-but-human play look like?" |
| `speedrun` | Aggressive spending, max enhancement ASAP, frequent challenge wins | "What's the fastest possible progression?" |

#### Per-System Rules

Each profile defines:

- **Haven**: Room build priority order (replaces the old `--haven` flag)
- **Soulforge**: Prestige rank thresholds for enhancement, target levels per slot
- **Challenges**: Win rate (challenges won per simulated hour), difficulty distribution
- **Stormglass**: Spending pattern (see Stormglass section below for per-profile tables)
- **Ascension**: When to ascend — call `ascension::logic::ascend()` when affordable and gates met
- **Prestige**: Auto-prestige trigger with per-profile stuck thresholds (see Prestige section below)

### 2. Outcome Injection

Each tick, after `game_tick_with_context()` returns, the simulator runs an `inject_outcomes()` phase that checks milestone triggers and directly mutates state.

#### Injection Mechanics Per System

**Challenge wins**: Every N ticks (profile-configured), call `achievements.on_minigame_won(game_type, difficulty, character_name)` directly. This is the correct injection site — `last_minigame_win` is a transient field consumed by `main.rs` between ticks, not by the tick engine. Also apply the challenge rewards (PR and Stormglass) directly to `state.prestige_rank` and `state.stormglass` to keep economy metrics accurate.

**Soulforge enhancement**: When prestige rank crosses a profile threshold, directly set `enhancement.levels[slot] = target_level`. Tests progression curves, not enhancement RNG.

**Stormglass sigil spending**: When `state.stormglass` exceeds a profile threshold, etch sigils by setting `state.storm_sigils.slots_unlocked` and populating `state.storm_sigils.sigils` with concrete `Sigil { effect, value, grade }` entries, then deduct the equivalent Stormglass cost from `state.stormglass`. Use `roll_sigil(effect, deterministic_roll)` with a fixed roll value per grade tier to produce representative `value` fields, or construct `Sigil` structs directly with values from the grade's expected range midpoint.

Per-profile sigil tables (using actual `SigilEffectType` and `SigilGrade` enum variants):

| Profile | Threshold | Effects | Grade | Cost deducted |
|---|---|---|---|---|
| `casual` | 5000 SG | `DamagePercent`, `DamageReductionPercent` | `C` | 2000 SG |
| `optimal` | 3000 SG | `DamagePercent`, `DamageReductionPercent`, `CritChancePercent` | `A` | 3000 SG |
| `speedrun` | 1500 SG | `DamagePercent`, `CritChancePercent`, `MaxHpPercent` | `SPlus` | 4000 SG |

(The `SigilGrade` enum uses a letter-grade system: `FMinus` through `SPlus`. The `SigilEffectType` enum has percent-based variants: `XpPercent`, `DamagePercent`, `DamageReductionPercent`, `CritChancePercent`, `DropRatePercent`, `MaxHpPercent`, `FishingSpeedPercent`, `OfflineXpPercent`, `AttackSpeedPercent`, `DoubleStrikePercent`, `RegenDelayPercent`, `ChronoOverchargePercent`. Exact effect choices per profile may be adjusted during implementation.)

**Ascension**: When prestige rank and pattern gates are met per profile rules, call `ascension::logic::ascend(state, deepest_layer, completed_patterns)` — the public function that handles PR deduction and level increment. After `ascend()` returns, the caller must also call `zones::access::sync_account_zone_unlocks()` to update Loom/fracture zone caps, since `ascend()` itself does not perform this sync. Do not mutate `ascension_level` directly.

**Prestige**: When the character is stuck, call `perform_prestige(state)` (non-vault variant, since the simulator has no vault state). "Stuck" is defined as: no new `zone_id` entered in the last N ticks AND `can_prestige(state)` returns true.

Per-profile stuck thresholds:

| Profile | N (ticks) | Equivalent time |
|---|---|---|
| `casual` | 18000 | 30 minutes |
| `optimal` | 9000 | 15 minutes |
| `speedrun` | 3600 | 6 minutes |

#### Key Principle

Inject at the state level, not the input level. Use the game's own public functions (`ascend()`, `perform_prestige()`, `on_minigame_won()`) wherever they exist. Only fall back to direct state mutation when no public function exists (enhancement levels, sigils).

### 3. Assertions and Output

Both output modes are always available:

#### Data Dump (default)

Extends the existing report with new sections:

- **Economy Flow**: Total PR earned, PR spent (Haven, Ascension, Enhancement breakdown), net PR balance over time
- **Progression Milestones**: Time-to-zone for all zones reached, time-to-prestige-rank, time-to-ascension-level
- **System Activations**: When each system was discovered/activated, what was injected and when

CSV export gets new columns: `ascension_level`, `enhancement_avg`, `stormglass_balance`, `challenges_won`, `pr_earned`, `pr_spent`

**PR tracking strategy**: Use a single unified approach — snapshot `state.prestige_rank` before and after each tick (including `inject_outcomes()`) to derive the delta. Positive deltas accumulate into `pr_earned`, negative deltas into `pr_spent`. This captures all PR sources (challenge rewards, Power Cores, WR→PR conversion) and all PR sinks (Haven builds, Ascension) without needing to enumerate individual `TickEvent` variants or separately track injection costs. The `inject_outcomes()` function does NOT separately track its own PR costs — the snapshot delta handles everything uniformly.

#### Assertions (`--assertions` flag)

Range checks that print PASS/FAIL and exit non-zero on failure.

Built-in assertion examples:
```
Zone 5 reachable within 30min at P0
Zone 10 reachable within 2hr at P0 with Stormbreaker
Level 50 reachable within 1hr at P0
PR income exceeds PR spending by tick 50000
No zone with >50% death rate sustained for >1000 ticks
```

Assertions are defined as a struct: `{ metric, op (<=, >=, ==), value, condition }`. Initial set is hardcoded in the binary. External config file (TOML/JSON) is a straightforward future extension but not needed now.

Exit code: 0 if all assertions pass (or no assertions requested), 1 if any fail.

### 4. File Organization

Split `simulator.rs` into a multi-file binary using Cargo's `src/bin/simulator/` convention:

```
src/bin/simulator/
├── main.rs           # Entry point, CLI parsing, tick loop
├── strategy.rs       # Strategy profiles, milestone triggers, inject_outcomes()
├── assertions.rs     # Assertion definitions, checking, PASS/FAIL output
├── stats.rs          # SimStats, TickProfile (extracted from current simulator.rs)
├── report.rs         # Print/summary functions (extracted from current simulator.rs)
```

Cargo produces a `simulator` binary from `src/bin/simulator/main.rs` — same binary name, same CLI, no breaking change for existing users (except `--haven` removal).

### Breaking Changes

- `--haven <strategy>` flag removed. Use `--strategy optimal` (or `casual`/`speedrun`) instead. All three profiles include Haven room priorities.
- Any CI workflow steps referencing `--haven` must be updated to `--strategy` before this lands. The old Haven strategies (`combat`, `qol`, `balanced`, `full`) are folded into the three new profiles — `balanced` maps to `optimal`, `full` maps to `speedrun`, `combat`/`qol` map to `casual` with different Haven room priorities.

### CLI Summary

```
cargo run --bin simulator -- [OPTIONS]

Existing flags (unchanged):
  --ticks N          Ticks to simulate (default: 36000 = 1 hour)
  --seed N           RNG seed (default: 42)
  --prestige N       Starting prestige rank (default: 0)
  --runs N           Number of runs with incrementing seeds (default: 1)
  --verbose          Per-tick event logging
  --csv FILE         Write time-series CSV
  --quiet            Only final summary line
  --stormbreaker     Unlock Stormbreaker achievement
  --profile          Print per-tick timing profile

New flags:
  --strategy STR     Strategy profile: casual, optimal, speedrun
  --assertions       Run balance assertions and exit with pass/fail

Removed flags:
  --haven STR        (replaced by --strategy)
```

### What This Does NOT Cover

- Dynamic difficulty adjustment or AI players for minigames
- Testing UI rendering or input routing
- Constant drift detection (use doc-audit / wiki-audit)
- The Deep simulator integration (remains a separate binary for mission-level simulation)
