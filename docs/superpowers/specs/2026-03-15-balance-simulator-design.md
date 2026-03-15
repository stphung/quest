# Balance Simulator Design

Extend the existing game simulator (`src/bin/simulator.rs`) to support full-lifecycle balance validation by adding strategy profiles, outcome injection for interactive systems, and configurable assertions.

## Goals

1. **Progression curve validation** — Simulate P0 through endgame with all systems active, measuring time-to-zone, time-to-prestige, economy flow
2. **Economy consistency** — Verify PR income vs spending, Stormglass flow, enhancement cost curves through actual simulation data
3. **Regression detection** — Configurable assertions that fail CI when balance changes break expected ranges

Constant drift detection (code vs docs vs wiki) is explicitly out of scope — handled by existing doc-audit and wiki-audit skills.

## Approach

Extend the existing simulator binary (Approach A). No new binary — the simulator already runs `game_tick_with_context()` with seeded RNG, CSV export, multi-run support, and Haven auto-building. We generalize the Haven strategy pattern to cover all interactive systems.

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
- **Stormglass**: Spending pattern (hoard vs spend on sigils)
- **Ascension**: When to ascend (e.g., "ascend when affordable and gates met")
- **Prestige**: Auto-prestige trigger (e.g., "prestige when stuck for N ticks with no zone progress")

### 2. Outcome Injection

Each tick, after `game_tick_with_context()` returns, the simulator runs an `inject_outcomes()` phase that checks milestone triggers and directly mutates state.

#### Injection Mechanics Per System

- **Challenge wins**: Every N ticks (profile-configured), set `state.last_minigame_win = Some(MinigameWinInfo { ... })` with a difficulty from the profile's distribution. The achievement system picks it up naturally on the next tick.
- **Soulforge enhancement**: When prestige rank crosses a profile threshold, directly set `enhancement.levels[slot] = target_level`. Tests progression curves, not enhancement RNG.
- **Stormglass spending**: When balance exceeds a profile threshold, directly apply sigil bonuses by setting sigil state.
- **Ascension**: When prestige rank and pattern gates are met per profile rules, set `state.ascension_level += 1` and apply the multiplier.
- **Prestige**: When the character is stuck (no zone progress for N ticks), trigger prestige by calling the same prestige logic the game uses (reset level/zone, increment rank).

#### Key Principle

Inject at the state level, not the input level. We say "the player made this decision" and let real game logic handle downstream effects. No simulated keypresses, no AI players for minigames.

### 3. Assertions and Output

Both output modes are always available:

#### Data Dump (default)

Extends the existing report with new sections:

- **Economy Flow**: Total PR earned, PR spent (Haven, Ascension, Enhancement breakdown), net PR balance over time
- **Progression Milestones**: Time-to-zone for all zones reached, time-to-prestige-rank, time-to-ascension-level
- **System Activations**: When each system was discovered/activated, what was injected and when

CSV export gets new columns: `ascension_level`, `enhancement_avg`, `stormglass_balance`, `challenges_won`, `pr_earned`, `pr_spent`

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
