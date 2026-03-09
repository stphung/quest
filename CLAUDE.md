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
4. Build verification (`cargo build --all-targets`)
5. Security audit (`cargo audit --deny yanked`)

**Auto-fix formatting:**
```bash
make fmt               # Applies rustfmt to all code
```

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
| `update-docs` | "audit the docs", "update documentation", "restructure docs" | Audits CLAUDE.md files and docs/ for structural health and content accuracy |
| `update-wiki` | "update the wiki", "wiki is stale" | Updates player-facing wiki (quest.wiki/) to match current game |
| `test-health-audit` | "audit the tests", "fix flaky tests" | Parallel flakiness + performance audit, fixes, 10x verification run |
| `perf-audit` | "audit performance", "optimize hot paths", "profile the game" | Multi-agent perf audit: finds bottlenecks, auto-fixes, adds benchmarks |
| `ship` | "ship it", "push and merge", "land this" | Push branch, create PR with automerge, watch CI until merged, fix failures |

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
| Challenges | `src/challenges/` | [CLAUDE.md](src/challenges/CLAUDE.md) | 10 challenge minigames |
| History | `src/history/` | [CLAUDE.md](src/history/CLAUDE.md) | Git-based save versioning (Time Vault) |
| Input | `src/input/` | [CLAUDE.md](src/input/CLAUDE.md) | Keyboard input routing |
| UI | `src/ui/` | [CLAUDE.md](src/ui/CLAUDE.md) | Terminal UI components (Ratatui) |
| Utils | `src/utils/` | [CLAUDE.md](src/utils/CLAUDE.md) | Build info, updater, debug menu |
| Loom | `src/loom/` | [CLAUDE.md](src/loom/CLAUDE.md) | Resource production chains, direct-pull refineries |
| Main Helpers | `src/main_helpers/` | [CLAUDE.md](src/main_helpers/CLAUDE.md) | Orchestration between main.rs and domain modules |

### Simulators

**Game Simulator** (`src/bin/simulator.rs`): Headless game balance simulator calling `game_tick()` with no UI/delay. Supports `--ticks`, `--seed`, `--prestige`, `--runs`, `--haven <strategy>`, `--stormbreaker`. Only exercises combat/zone loop (no interactive systems).

**Deep Simulator** (`src/bin/deep_simulator.rs`): Headless Deep expedition simulator. Supports `--hours`, `--seed`, `--strategy` (rush/balanced/infrastructure), `--guild-rank`.

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
- **Boss spawn**: After 10 kills in subzone (5 kills to retry after boss death)
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
- **WR→PR brackets**: 0-10 WR/hr = 5 PR/WR/hr/day, 10-25 = 10, 25+ = 15 (activates when all 28 patterns complete)
- **Shuttle level caps**: Asc 0-VI = 1, VII = 3, VIII = 5, IX = 7, X = 10
- **Power Cores**: 6 cores (2-18 PR/day), unlocked at Deep Layers 3/7/12/18/25/30, max 48 PR/day total

## Combat Mechanics

- **Enemy scaling**: Static zone-based stats from `ZONE_ENEMY_STATS` table. Fracture zones scale 1.6x per zone from Zone 11 base. Loom zones scale 1.25x per zone from Zone 30 base
- **Damage pipeline**: base -> Giant's Might % -> Haven % -> prestige flat -> ascension mult -> defense -> min 1 -> Bulwark DR -> crit
- **Defense pipeline**: base -> prestige flat -> ascension mult -> DR %
- **Death**: Boss death resets to subzone 1; dungeon death exits dungeon (no prestige loss)
- **Weapon gates**: Zone 10 final boss requires Stormbreaker

## Dependencies

Ratatui 0.30, Serde (JSON), serde_json 1.0, Rand 0.10, Rand_chacha 0.10 (seeded RNG for simulator), Chrono, dirs 6.0, Chess-engine 0.1, ureq 3.2, flate2 1.1, zip 8.0, unicode-width 0.2, git2 0.20 (vendored-openssl), tar 0.4, uuid 1.21, tempfile 3 (dev)
