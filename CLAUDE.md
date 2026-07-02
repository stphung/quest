# Quest - Terminal-Based Idle RPG

A terminal-based idle RPG written in Rust. Your hero automatically battles enemies, gains XP, levels up, explores dungeons, and prestiges.

## Build & Run

```bash
make setup             # First time: configure git hooks
cargo build            # Build
cargo run              # Run the game
make check             # Run all CI checks locally
make fmt               # Auto-fix formatting
```

## Development Workflow

**Use git worktrees for feature work.** Create isolated worktrees for branches instead of switching branches in the main workspace.

**Before pushing code, run:**
```bash
make check             # Runs scripts/ci-checks.sh (same as CI)
```

This runs all PR quality checks:
1. Format checking (`cargo fmt --check`)
2. Clippy linting (`cargo clippy --all-targets -- -D warnings`)
3. All tests (`cargo test`)
4. Progression check (`cargo run --release --bin simulator -- --check-progression`)
5. Security audit (`cargo audit --deny yanked`)

**Auto-fix formatting:**
```bash
make fmt               # Applies rustfmt to all code
```

### How to Verify Your Change

`make check` is the final gate before every push. But run the *targeted* verification for what you touched first — it is faster and catches what the generic gate can't. Pick the row(s) matching your change:

| You changed… | Verify with |
|--------------|-------------|
| `src/ui/` rendering | `cargo test snapshot` — review the diff; re-bless intentional changes with `INSTA_UPDATE=always cargo test snapshot` and commit the `.snap` diffs. For visual changes, also screenshot the real game with the `drive-game` skill |
| A full-screen overlay (Haven/Deep/Loom/Soulforge/Stormglass/Time Vault) | `cargo test overlay_snapshot` — same re-bless workflow; scenes render via their entry points in `src/ui/overlay_snapshot_tests.rs` |
| Combat, zones, XP, or balance constants (`core/constants.rs`, `ZONE_ENEMY_STATS`, multipliers) | `cargo test` + `cargo run --release --bin simulator -- --check-progression`. For balance questions beyond the CI gate ("can players still reach Z50?"), run the `balance-sim` skill |
| Core tick loop (`src/core/tick*.rs`) | `cargo test --test game_tick_tests` + progression check. Keep the loop seeded-RNG clean — `game_tick_with_context()` takes `rng: &mut R` for a reason |
| The Deep (`src/deep/`) | `cargo test --test deep_tests` + `cargo run --bin deep_simulator -- --hours 24 --seed 1 --strategy balanced` |
| Loom (`src/loom/`) | `cargo test --test loom_tests` + `cargo test overlay_snapshot` (graph renderer) + progression check (Loom unlocks are asserted in the endgame scenario) |
| Items, drops, generation (`src/items/`) | `cargo test --test item_tests` + `cargo test snapshot` (equipment panel renders names/tiers/colors). Keep `generate_item_with_rng` the single RNG entry point — fixtures depend on seeded generation |
| `GameState` fields / serde / persistence | `cargo test --test save_compat_tests` — loads the committed save corpus (`tests/fixtures/saves/`) through the real load paths; a failure means existing player saves break (account loaders silently wipe progress on parse failure, so this is the only red flag you get). Fix with `serde(default)`/`alias`/migration, never by editing the corpus. Also `cargo test --test character_tests --test history_tests` |
| A challenge minigame (`src/challenges/`) | That game's unit tests + `cargo test snapshot_all_minigames`. New minigame? Use the `add-challenge` skill — it covers all 15 integration points |
| Keyboard input (`src/input/`) | No automated coverage — drive the real game with the `drive-game` skill and exercise the key path you changed |
| Fixtures or the UI clock (`src/fixtures.rs`, `src/ui/clock.rs`) | Full `cargo test` — nearly every snapshot depends on them. If `snapshot_rendering_is_deterministic` fails, you introduced a wall-clock/RNG/ordering leak; fix that, never re-bless around it |
| Dependencies (`Cargo.toml`) | `cargo audit --deny yanked` (CI runs it even where local sandboxes can't) + the `dependency-audit` skill for a deeper pass |
| Docs / wiki | `doc-audit` / `wiki-audit` skills check for stale constants and broken links |

Two habits that make verification meaningful:
- **Verify the behavior, not just the build.** A green `cargo check` proves nothing about a gameplay change — run the row above that actually exercises it.
- **Snapshot diffs are the review, not noise.** When a `.snap` file changes, read the diff before re-blessing; an unexpected changed frame is the test working.

## CI/CD Pipeline

**On every PR:**
- Runs `scripts/ci-checks.sh` (format, lint, test, build, audit)
- Must pass to merge

**On push to main:**
- Runs all checks
- Builds release binaries for 3 platforms (Linux, macOS x86/ARM)
- Signs macOS binaries with ad-hoc signature (prevents Gatekeeper blocking)
- Creates GitHub release with downloadable binaries

**Key insight:** Local `make check` runs the **exact same script** as CI, ensuring consistency.

## Skills (`.claude/skills/`)

Agent-invocable skills — ask in natural language to trigger them.

| Skill | Trigger phrases | What it does |
|-------|----------------|--------------|
| `audit` | "full audit", "audit everything" | Runs all 5 audit skills in parallel on isolated worktrees (perf, test, doc, wiki, dependency) |
| `perf-audit` | "audit performance", "optimize hot paths" | Multi-agent perf audit: finds bottlenecks, auto-fixes, adds benchmarks |
| `test-audit` | "audit the tests", "fix flaky tests" | Multi-agent test audit: finds flakiness + perf issues by area, auto-fixes, 10x verification |
| `doc-audit` | "audit the docs", "update documentation" | Multi-agent docs audit: finds stale constants, missing files, outdated types across CLAUDE.md and docs/ |
| `wiki-audit` | "audit the wiki", "wiki is stale" | Multi-agent wiki audit: finds stale numbers, missing systems, broken links in quest.wiki/ |
| `dependency-audit` | "audit dependencies", "update deps" | Multi-agent dependency audit: outdated versions, unused deps, security advisories, feature hygiene |
| `add-challenge` | "add a challenge", "new minigame" | Checklist-driven agent for adding a new challenge minigame across all 15 integration points |
| `balance-sim` | "run the simulator", "check balance" | Multi-agent balance simulator: runs headless simulator across strategies/seeds, produces prioritized balance report |
| `clean-workspace` | "clean up workspace", "reset workspace" | Resets the repo to a fresh-clone-like state: removes stale branches and uncommitted files |
| `ship` | "ship it", `/ship` | Push branch, create PR with automerge, watch CI until merged, fix failures |
| `drive-game` | "drive the game", "screenshot the game" | Runs the real game in tmux against `mkstate` fixtures (isolated via `QUEST_DIR`), sends keystrokes, captures PNG screenshots for PR review |

## Architecture

Entry point: `src/main.rs` — runs a 100ms tick game loop using Ratatui (with Crossterm backend).

Larger modules have their own `CLAUDE.md` with implementation patterns, integration points, and extension guides. See the table below.

## Modules

| Module | Path | Docs | Purpose |
|--------|------|------|---------|
| Core | `src/core/` | [CLAUDE.md](src/core/CLAUDE.md) | Game tick engine, state, XP/leveling, constants |
| Combat | `src/combat/` | [CLAUDE.md](src/combat/CLAUDE.md) | Combat state machine, damage pipelines |
| Character | `src/character/` | [CLAUDE.md](src/character/CLAUDE.md) | Attributes, prestige, persistence |
| Zones | `src/zones/` | [CLAUDE.md](src/zones/CLAUDE.md) | 50 zones, fracture regions, Loom zones, progression |
| Dungeon | `src/dungeon/` | [CLAUDE.md](src/dungeon/CLAUDE.md) | Procedural generation, room system |
| Fishing | `src/fishing/` | [CLAUDE.md](src/fishing/CLAUDE.md) | Sessions, 40 ranks, Storm Leviathan |
| Items | `src/items/` | [CLAUDE.md](src/items/CLAUDE.md) | Generation, scoring, drop rates |
| Enhancement | `src/enhancement/` | [CLAUDE.md](src/enhancement/CLAUDE.md) | Soulforge equipment enhancement |
| Ascension | `src/ascension/` | [CLAUDE.md](src/ascension/CLAUDE.md) | Combat power multiplier system |
| Deep | `src/deep/` | [CLAUDE.md](src/deep/CLAUDE.md) | Mercenary expedition system |
| Stormglass | `src/stormglass/` | [CLAUDE.md](src/stormglass/CLAUDE.md) | Currency, Storm Sigils, daily rotation |
| Power Cores | `src/power_cores/` | [CLAUDE.md](src/power_cores/CLAUDE.md) | Passive PR generation |
| God Items | `src/god_items/` | [CLAUDE.md](src/god_items/CLAUDE.md) | 3 Norse mythology endgame items |
| Haven | `src/haven/` | [CLAUDE.md](src/haven/CLAUDE.md) | Account-level base building |
| Achievements | `src/achievements/` | [CLAUDE.md](src/achievements/CLAUDE.md) | Achievement tracking, titles, scores |
| Challenges | `src/challenges/` | [CLAUDE.md](src/challenges/CLAUDE.md) | 14 challenge minigames |
| History | `src/history/` | [CLAUDE.md](src/history/CLAUDE.md) | Git-based save versioning (Time Vault) |
| Input | `src/input/` | [CLAUDE.md](src/input/CLAUDE.md) | Keyboard input routing |
| UI | `src/ui/` | [CLAUDE.md](src/ui/CLAUDE.md) | Terminal UI components (Ratatui) |
| Utils | `src/utils/` | [CLAUDE.md](src/utils/CLAUDE.md) | Build info, updater, debug menu |
| Loom | `src/loom/` | [CLAUDE.md](src/loom/CLAUDE.md) | Resource production chains, direct-pull refineries |
| Main Helpers | `src/main_helpers/` | [CLAUDE.md](src/main_helpers/CLAUDE.md) | Orchestration between main.rs and domain modules |

### Simulators

**Game Simulator** (`src/bin/simulator/`): Headless game balance simulator calling `game_tick_with_context()` with no UI/delay. Supports `--ticks`, `--seed`, `--prestige`, `--runs`, `--strategy <profile>` (casual/optimal/speedrun), `--stormbreaker`, `--assertions`, `--check-progression`. Strategy profiles inject challenge wins, enhancement, sigils, ascension, and auto-prestige.

**Progression check** (`simulator --check-progression`): CI gate asserting the game still progresses. Runs 3 scenarios across multiple seeds — early-game (2h at P0), prestige-economy (6h, optimal strategy), endgame-systems (30h at P200 speedrun) — and asserts coarse progression facts (zone/level pacing, PR economy, Deep/Loom/Ascension unlocks). Scenarios and thresholds live in `src/bin/simulator/scenarios.rs`; thresholds carry ~2x headroom because the tick loop is not perfectly deterministic. Runs as the `Balance` job on every PR and as step 4 of `make check`.

**Deep Simulator** (`src/bin/deep_simulator.rs`): Headless Deep expedition simulator. Supports `--hours`, `--seed`, `--strategy` (rush/farm/balanced/infrastructure), `--guild-rank`.

**Fixture Generator** (`src/bin/mkstate.rs`): Writes character save fixtures for named scenarios (`fresh`, `midgame`, `endgame`, `boss`). Pair with the `QUEST_DIR` env var (honored by `core::paths::get_quest_dir()`) to run the game against an isolated save directory. Used by the `drive-game` skill for UI verification; `scripts/screenshot.sh` captures a tmux pane as a color PNG. The scenario builders live in `src/fixtures.rs` (`quest::fixtures`), shared with the UI snapshot tests — mkstate feeds them the wall clock and thread RNG, tests feed a fixed timestamp and a seeded RNG.

### UI Snapshot Tests (`src/ui/snapshot_tests.rs`)

Deterministic full-frame TUI snapshot tests (insta + ratatui `TestBackend`) covering each responsive size tier across fixture scenarios, plus the full-screen overlays (Haven, Deep, Loom, Soulforge, Stormglass, Time Vault) in `src/ui/overlay_snapshot_tests.rs`; committed snapshots live in `src/ui/snapshots/`. Part of `cargo test`, so they gate CI. After an intentional UI change: review the diff, re-bless with `INSTA_UPDATE=always cargo test snapshot`, and commit the `.snap` changes. This is the first line of verification for any `src/ui/` change — use the `drive-game` skill for visual/e2e confirmation. Determinism rules (freezable `ui/clock.rs`, no direct wall-clock reads in render code) are documented in [src/ui/CLAUDE.md](src/ui/CLAUDE.md).

### Library Crate (`src/lib.rs`)

Exposes all game logic modules for integration testing. UI module is private (terminal-coupled).

## Common Patterns

### Module Structure
Most game modules follow this layout:
```
module/
├── mod.rs         # Public API re-exports
├── types.rs       # Data structures and enums
├── logic.rs       # Business logic and state transitions
└── generation.rs  # (optional) Procedural generation
```

### Difficulty Tiers
All challenge minigames use 4 difficulty levels: Novice, Apprentice, Journeyman, Master.

### Forfeit Pattern
All interactive minigames: first Esc sets `forfeit_pending`, second Esc confirms, any other key cancels.

### Haven Bonus Injection
Haven bonuses are passed as explicit parameters rather than accessed globally. This keeps modules decoupled.

## Key Constants

- **Tick interval**: 100ms (10 ticks/sec)
- **Player attack interval**: 1.5s
- **Enemy attack intervals**: normal 2.0s, subzone boss 1.8s, zone boss 1.5s, dungeon elite 1.6s, dungeon boss 1.4s
- **HP regen after kill**: 2.5s
- **Autosave**: every 30s
- **Update check**: every 15min +/-5min jitter
- **XP gain**: Only from defeating enemies (200-400 XP per kill)
- **Offline XP**: 25% rate, max 7 days (simulates kills)
- **Mob item drop rate**: 15% base + 1% per prestige rank (capped at 25%), max rarity Epic
- **Boss item drops**: Guaranteed, can include Legendary (2% normal boss, 5% Zone 10 final boss)
- **Item level**: ilvl = zone_id x 10 (Zone 1 = ilvl 10, Zone 10 = ilvl 100)
- **Item tier**: T0-T9 quality roll (exponential curve: T0 38%, T9 0.1%). Stat multiplier: T0 0.40x to T9 1.00x
- **Boss spawn**: After 10 kills in subzone (kills reset to 0 after boss death)
- **Haven discovery**: requires P10+, base chance 0.000014/tick + 0.000007 per rank above 10
- **Challenge discovery**: ~2hr avg per challenge (requires P1+)
- **Soulforge discovery**: requires P15+, base chance 0.000014/tick + 0.000007 per rank above 15
- **The Deep discovery**: triggers on first Expanse cycle boss kill (The Endless) at P15+ (no per-tick RNG roll)
- **Enhancement levels**: 0-10, success rates 100% (+1-4), 70%/55%/40% (+5-7), 30%/20%/10% (+8-10)
- **Enhancement costs**: 1 PR (+1-4), 2/3/3 PR (+5-7), 4 PR (+8-9), 5 PR (+10)
- **Fracture zone stat scaling**: 1.6x per zone from Zone 11 base (FRACTURE_ZONE_STAT_MULTIPLIER)
- **Fracture zone unlock**: Deep Layer 3 -> Z12-14, Layer 7 -> Z15-17, Layer 12 -> Z18-20, Layer 18 -> Z21-23, Layer 25 -> Z24-26, Layer 30 -> Z27-30
- **Ascension cost**: [35, 65, 120, 200, 325, 500] PR for I-VI; [1500, 4000, 8000, 15000] PR for VII-X
- **Ascension multiplier**: 2^level for I-VI (2x to 64x); 64 * 1.5^(level-6) for VII+ (96x, 144x, 216x, 324x)
- **Ascension pattern gates**: VII = 8 patterns, VIII = 16, IX = 22, X = 28 completed Woven Patterns
- **Loom Zone stat scaling**: 1.25x per zone from Zone 30 base (LOOM_ZONE_STAT_MULTIPLIER)
- **Loom Zone unlock** (triple-gated: patterns + ascension + prestige): 4p/—/P2k -> Z31-34, 8p/VII/P5k -> Z35-38, 16p/VIII/P15k -> Z39-42, 22p/IX/P30k -> Z43-46, 28p/X/P50k -> Z47-50
- **WR→PR conversion**: PR/hr = WR × (1 + WR/100) — self-multiplying, ~1:1 at low rates, 2.3× at max (activates when all 28 patterns complete)
- **Shuttle level caps**: Asc 0-VI = 1, VII = 3, VIII = 5, IX = 7, X = 10
- **Power Cores**: 6 cores (2-18 PR/day), unlocked at Deep Layers 3/7/12/18/25/30, max 48 PR/day total

## Combat Mechanics

- **Enemy scaling**: Static zone-based stats from `ZONE_ENEMY_STATS` table. Fracture zones scale 1.6x per zone from Zone 11 base. Loom zones scale 1.25x per zone from Zone 30 base
- **Damage pipeline**: base -> Giant's Might % -> Haven % -> prestige flat -> ascension mult -> defense -> min 1 -> Bulwark DR -> crit
- **Defense pipeline**: base -> prestige flat -> ascension mult -> DR %
- **Death**: Boss death resets to subzone 1; dungeon death exits dungeon (no prestige loss)
- **Weapon gates**: Zone 10 final boss requires Stormbreaker

## Dependencies

Ratatui 0.30, Serde (JSON), serde_json 1.0, Rand 0.10, Rand_chacha 0.10 (seeded RNG for simulator), Chrono, dirs 6.0, Chess-engine 0.1, ureq 3.3, flate2 1.1, zip 8.5, unicode-width 0.2, git2 0.20 (vendored-openssl), tar 0.4, uuid 1.23, petgraph 0.8 (Loom DAG), tempfile 3 (dev), criterion 0.8 (dev/bench), insta 1 (dev, UI snapshot tests)
