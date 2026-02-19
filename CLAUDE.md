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
| `doc-health-audit` | "audit the docs", "update documentation" | Audits all docs/, CLAUDE.md files, and player-facing wiki (quest.wiki/) against the current codebase, fixes staleness |
| `test-health-audit` | "audit the tests", "fix flaky tests" | Parallel flakiness + performance audit, fixes, 10x verification run |
| `pr-validation` | "create a PR", "validate CI checks" | Creates PR, monitors CI, auto-fixes failures until green |

## Architecture

Entry point: `src/main.rs` — runs a 100ms tick game loop using Ratatui (with Crossterm backend).

### Module Documentation

Larger modules have their own `CLAUDE.md` with implementation patterns, integration points, and extension guides:

- [`src/core/CLAUDE.md`](src/core/CLAUDE.md) — Game tick engine, XP/leveling, offline progression, constants
- [`src/challenges/CLAUDE.md`](src/challenges/CLAUDE.md) — Adding new minigames (step-by-step checklist)
- [`src/items/CLAUDE.md`](src/items/CLAUDE.md) — Item generation pipeline, scoring, drop rates
- [`src/character/CLAUDE.md`](src/character/CLAUDE.md) — Attributes, prestige, persistence
- [`src/combat/CLAUDE.md`](src/combat/CLAUDE.md) — Combat state machine, enemy generation
- [`src/dungeon/CLAUDE.md`](src/dungeon/CLAUDE.md) — Procedural generation, room system
- [`src/fishing/CLAUDE.md`](src/fishing/CLAUDE.md) — Fishing sessions, ranks, Storm Leviathan
- [`src/zones/CLAUDE.md`](src/zones/CLAUDE.md) — Zone tiers, progression, weapon gates
- [`src/haven/CLAUDE.md`](src/haven/CLAUDE.md) — Account-level base building, bonus system
- [`src/achievements/CLAUDE.md`](src/achievements/CLAUDE.md) — Achievement tracking, persistence
- [`src/enhancement/CLAUDE.md`](src/enhancement/CLAUDE.md) — Soulforge enhancement system
- [`src/ui/CLAUDE.md`](src/ui/CLAUDE.md) — Shared game layout components, color conventions

### Core Module (`src/core/`)

- `game_state.rs` — Main character state struct (level, XP, prestige, combat state, equipment)
- `game_logic.rs` — XP curve (`100 × level^1.5`), leveling (+3 random attribute points), enemy spawning, offline progression
- `tick.rs` — Per-tick game engine: `game_tick<R: Rng>()` with 12 processing stages, returns `TickResult` with `Vec<TickEvent>` (30 variants). Zero UI imports, zero file I/O — fully decoupled from rendering
- `constants.rs` — Game balance constants (tick rate, attack intervals, XP rates, item drop rates, zone enemy stats, boss multipliers, prestige combat bonuses, update check jitter)

### Simulator (`src/bin/simulator.rs`)

Headless game balance simulator that calls the same `game_tick()` code with no UI and no tick delay. Useful for testing game balance across prestige levels and time horizons.

```bash
cargo run --release --bin simulator -- --ticks 36000 --seed 42 --prestige 10 --runs 3
cargo run --release --bin simulator -- --ticks 36000 --seed 42 --prestige 15 --haven combat
```

CLI: `--ticks N`, `--seed N`, `--prestige N`, `--runs N`, `--verbose`, `--csv FILE`, `--quiet`, `--stormbreaker` (force-unlocks TheStormbreaker achievement for Zone 10+ testing), `--haven STR` (auto-build Haven rooms using a named strategy)

**Haven auto-building:** `--haven <strategy>` enables automatic Haven room construction during simulation. Four strategies: `combat` (Armory/damage path), `qol` (Bedroom/fishing path), `balanced` (both branches), `full` (everything + StormForge). When enabled, Haven is force-discovered at start and prestige ranks are spent on rooms each tick following the strategy's priority order. This models the real gameplay trade-off between investing prestige in Haven vs keeping it for combat bonuses.

**Limitation:** Only exercises the combat/zone progression loop. Interactive systems (dungeons, fishing, challenges) are discovered but never activated (no player input). Haven bonuses are fully active when `--haven` is used. See issue #141 for auto-play policies.

### Character Module (`src/character/`) — [detailed docs](src/character/CLAUDE.md)

- `attributes.rs` — 6 RPG attributes (STR, DEX, CON, INT, WIS, CHA), modifier = `(value - 10) / 2`
- `derived_stats.rs` — Combat stats calculated from attributes and enhancement levels (HP, damage, defense, crit, XP mult)
- `prestige.rs` — Prestige tiers (Bronze→Eternal) with XP multipliers (`1+0.5×rank^0.7`, diminishing returns), attribute cap increases (`10+rank×5`), and `PrestigeCombatBonuses` (flat damage/defense/crit/HP from rank)
- `manager.rs` — Character CRUD operations (create, delete, rename), JSON save/load in ~/.quest/, name validation
- `input.rs` — Character selection, creation, deletion, renaming input handling and UI states

### Combat Module (`src/combat/`) — [detailed docs](src/combat/CLAUDE.md)

- `types.rs` — Enemy struct (with defense field), zone-based enemy generators, combat state machine
- `logic.rs` — Turn-based combat mechanics with prestige bonuses and god item passives, damage pipeline (Giant's Might % -> Haven % -> prestige flat -> enemy defense -> Divine Bulwark DR -> crit), `GodItemCombatBonuses` struct, event emission

### Zone System (`src/zones/`)

- `data.rs` — 11 zones with 3-4 subzones each, prestige requirements, boss definitions
- `progression.rs` — Zone/subzone progression state, kill tracking (10 kills → boss spawn, 5 kills to retry after boss death), weapon gates

**Zone Tiers:**
- P0: Meadow, Dark Forest (3 subzones each)
- P5: Mountain Pass, Ancient Ruins (3 subzones each)
- P10: Volcanic Wastes, Frozen Tundra (4 subzones each)
- P15: Crystal Caverns, Sunken Kingdom (4 subzones each)
- P20: Floating Isles, Storm Citadel (4 subzones each, Zone 10 requires Stormbreaker)
- Post-game: The Expanse (Zone 11, 4 subzones, cycles infinitely, endgame difficulty wall)

### Dungeon Module (`src/dungeon/`) — [detailed docs](src/dungeon/CLAUDE.md)

- `types.rs` — Room types (Entrance, Combat, Treasure, Elite, Boss), room state (Hidden, Revealed, Current, Cleared), dungeon sizes
- `generation.rs` — Procedural dungeon generation with connected rooms
- `logic.rs` — Dungeon navigation, room clearing, key system, safe death (no prestige loss)

**Dungeon Sizes:** Small 5×5, Medium 7×7, Large 9×9, Epic 11×11 (based on prestige)

### Fishing Module (`src/fishing/`)

- `types.rs` — Fish rarities (Common→Legendary), fishing phases (Casting, Waiting, Reeling), 40 ranks across 8 tiers, Storm Leviathan encounter tracking
- `generation.rs` — Fish name generation, rarity rolling, Storm Leviathan progressive hunt
- `logic.rs` — Fishing session tick processing, Haven bonus integration, item drops from fishing

**Fishing Ranks:** 40 ranks across 8 tiers (Novice 1-5, Apprentice 6-10, Journeyman 11-15, Expert 16-20, Master 21-25, Grandmaster 26-30 base max, Mythic 31-35, Transcendent 36-40 with Fishing Dock T4). Storm Leviathan encounter at rank 40.

**Storm Leviathan:** A 10-encounter progressive hunt. At max fishing rank, legendary fish catches may trigger Leviathan encounters. After 10 encounters, the player catches it, unlocking the ability to forge Stormbreaker at the Storm Forge.

### Item Module (`src/items/`) — [detailed docs](src/items/CLAUDE.md)

- `types.rs` — Core item data structures (7 equipment slots, 6 rarity tiers including God/Mythic, 9 affix types + Unknown fallback, ilvl scaling, tier (T0-T9), `god_item_id` field, `power()` intrinsic score)
- `equipment.rs` — Equipment container with slot management and iteration
- `generation.rs` — Rarity-based attribute/affix generation with ilvl scaling (1.0x at ilvl 10 to 4.0x at ilvl 100) and tier quality multiplier (T0 0.40x to T9 1.00x)
- `drops.rs` — Separate mob/boss drop systems: mobs have 15% base drop chance (capped at Epic), bosses always drop (can drop Legendary)
- `names.rs` — Procedural name generation with prefixes/suffixes
- `scoring.rs` — Affix power weights (`affix_power_weight()`) and power-based auto-equip (`auto_equip_if_better()` uses intrinsic `power()` score). God (Mythic) items are never auto-replaced by lower rarity

### Enhancement Module (`src/enhancement/`) — [detailed docs](src/enhancement/CLAUDE.md)

- `types.rs` — EnhancementProgress (per-slot levels 0-10), SoulforgePhase, SoulforgeUiState, constants (success rates, costs, penalties, multiplier curve)
- `logic.rs` — Enhancement rolling, result application, Soulforge discovery chance/roll
- `persistence.rs` — Save/load from `~/.quest/enhancement.json`

Account-level equipment enhancement system (Soulforge) that persists across characters. Each of 7 equipment slots can be enhanced from +0 to +10. Levels +1-4 are 100% success rate; +5-10 have decreasing success rates (60% down to 10%) and failure penalties (-1 or -2 levels). Costs prestige ranks. Discovered at P15+. Enhancement multipliers boost equipment stats in `derived_stats.rs`.

### God Items Module (`src/god_items/`)

- `types.rs` — 3 god items (Asprika, Sleipnir, Megingjord) with unique passives and bonuses, `GodItemId` enum, `GodItemDefinition` struct, helper functions for querying equipped god item effects

Three Norse mythology-themed endgame items with `Rarity::Mythic` (displayed as "God"). Each has unique combat passives and non-combat bonuses:

| God Item | Slot | Attributes | Passive | Bonuses |
|----------|------|-----------|---------|---------|
| Asprika | Armor | +40 CON, +20 WIS | Divine Bulwark: 30% damage reduction | +40% XP |
| Sleipnir | Boots | +40 DEX, +20 WIS | Windborne: 100% attack speed | Swiftstrider (50% regen reduction), Swiftfoot (50% dungeon speed), NimbleHands (50% fishing speed) |
| Megingjord | Ring | +40 STR, +20 CON | Giant's Might: 150% damage | +40% XP |

God items are created via debug menu (discovery/forging system not yet designed, see issue #235). Items have `god_item_id: Option<GodItemId>` field; auto-equip never replaces a god item with a lower rarity.

### Challenge Minigames (`src/challenges/`) — [detailed docs](src/challenges/CLAUDE.md)

- `menu.rs` — Generic challenge menu system (pending challenges, extensible challenge types)
- `chess/` — Chess minigame (4 difficulty levels: Novice→Master, ~500-1350 ELO), requires P1+
- `go/` — Go (Territory Control) on 9×9 board, MCTS AI with heuristics (500-20k simulations), requires P1+
- `morris/` — Nine Men's Morris (board layout, mill detection, phases), requires P1+
- `gomoku/` — Gomoku (Five in a Row) on 15×15 board, minimax AI (depth 2-5)
- `minesweeper/` — Trap Detection, 4 difficulties (9×9 to 20×16)
- `rune/` — Rune Deciphering (Mastermind-style deduction), 4 difficulties
- `snake/` — Serpent's Path (Snake) on 26×26 grid, 4 difficulties (Novice 10 food/200ms, Master 25 food/90ms), real-time ~60 FPS
- `flappy/` — Skyward Gauntlet (Flappy Bird) on 50×18 area, 4 difficulties, gravity/flap physics, pipe obstacles with gap sizes (7→4 rows), 3 lives, real-time ~60 FPS
- `jezzball/` — Containment Breach (JezzBall) on 34×22 grid, 4 difficulties (2-5 balls), wall-building to capture area, 3 lives, real-time ~60 FPS
- `runic_shift/` — Sigil Surge (panel-matching) on 6×12 grid, 5 rune colors, 3 lives, 4 difficulties (rise interval 7000-3000ms), real-time ~60 FPS

### Haven Module (`src/haven/`) — [detailed docs](src/haven/CLAUDE.md)

- `types.rs` — Haven struct, 14 room definitions in a skill tree, upgrade tiers, 15 bonus types, Storm Forge
- `logic.rs` — Room construction, upgrade logic, bonus calculation, prestige rank cost system

Account-level base building that persists across prestiges. 14 rooms in a two-branch skill tree (combat + QoL) with 3 capstones (War Room, Vault, Storm Forge). Rooms provide bonuses (damage, XP, drop rate, rarity, crit, HP regen, double strike, offline XP, fishing, discovery). Costs prestige ranks. Discovered at P10+.

### Achievement Module (`src/achievements/`)

- `types.rs` — AchievementId enum, categories, unlock tracking
- `data.rs` — Achievement database with descriptions and unlock conditions
- `persistence.rs` — Save/load from `~/.quest/achievements.json`

Account-level achievement system that persists across characters. 6 categories (Combat, Level, Progression, Challenges, Exploration, Stats). Tracks kills, boss kills, levels, prestige, zone completion, challenge wins, fishing ranks/catches, dungeon completions, Haven building, and Soulforge enhancements. Includes modal notification system with 500ms accumulation window.

### Input Handling (`src/input.rs`)

Routes keyboard input to the appropriate handler based on current game state. Dispatches to minigame input handlers, character management flows, haven overlay, and debug menu. When quitting with pending challenges, shows a confirmation dialog ([Enter] Leave / [Esc] Stay).

### Utilities (`src/utils/`)

- `build_info.rs` — Build metadata (commit, date) embedded at compile time
- `updater.rs` — Self-update from GitHub releases (30min check interval ±5min jitter)
- `debug_menu.rs` — Debug menu for testing discoveries (activate with `--debug` flag, toggle with backtick). Options: trigger dungeons, fishing, all 10 challenge types, Haven discovery, Soulforge discovery, forge god items (Asprika, Sleipnir, Megingjord)

### UI (`src/ui/`) — [detailed docs](src/ui/CLAUDE.md)

- `mod.rs` — Layout coordinator (stats panel left 50%, combat scene right 50%), centralized `rarity_color()` function
- `game_common.rs` — Shared minigame layout, status bars, game-over overlays
- `stats_panel.rs` — Character stats, attributes, equipment display, prestige info, XP rate + ETA
- `ticker.rs` — Scrolling loot ticker with independent per-entry scrolling
- `combat_scene.rs` — Combat view with HP bars and enemy sprites
- `combat_3d.rs` — 3D ASCII first-person dungeon renderer
- `combat_effects.rs` — Visual effects (damage numbers, attack flashes)
- `enemy_sprites.rs` — ASCII enemy sprite templates
- `dungeon_map.rs` — Top-down dungeon minimap with fog of war
- `fishing_scene.rs` — Fishing UI with phase display
- `haven_scene.rs` — Haven base building overlay
- `prestige_confirm.rs` — Prestige confirmation dialog
- `achievement_browser_scene.rs` — Achievement browsing and tracking
- `challenge_menu_scene.rs` — Challenge menu list/detail view
- `responsive.rs` — Responsive layout with 5 size tiers (TooSmall/S/M/L/XL)
- `soulforge_scene.rs` — Soulforge enhancement overlay
- `chess_scene.rs`, `go_scene.rs`, `morris_scene.rs`, `gomoku_scene.rs`, `minesweeper_scene.rs`, `rune_scene.rs`, `snake_scene.rs`, `flappy_scene.rs`, `jezzball_scene.rs`, `runic_shift_scene.rs` — Minigame UIs
- `scene_fx.rs` — Shared utilities for layered ASCII scene rendering (scene buffer, backdrop effects)
- `zone_bg.rs` — Stylized zone background scenes with 6-layer compositing pipeline for all 11 zones
- `debug_menu_scene.rs` — Debug menu overlay
- `help_overlay.rs` — Help/controls overlay
- `throbber.rs` — Shared spinner animations and atmospheric messages
- `character_select.rs`, `character_creation.rs`, `character_delete.rs`, `character_rename.rs` — Character management UI

### Library Crate (`src/lib.rs`)

Exposes all game logic modules for integration testing. UI module is private (terminal-coupled). Re-exports commonly used types at crate root.

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

- Tick interval: 100ms (10 ticks/sec)
- Player attack interval: 1.5s
- Enemy attack intervals: normal 2.0s, subzone boss 1.8s, zone boss 1.5s, dungeon elite 1.6s, dungeon boss 1.4s
- HP regen after kill: 2.5s
- Autosave: every 30s
- Update check: every 30min ±5min jitter
- XP gain: Only from defeating enemies (200-400 XP per kill)
- Offline XP: 25% rate, max 7 days (simulates kills)
- Mob item drop rate: 15% base + 1% per prestige rank (capped at 25%), max rarity Epic
- Boss item drops: Guaranteed, can include Legendary (2% normal boss, 5% Zone 10 final boss)
- Item level: ilvl = zone_id × 10 (Zone 1 = ilvl 10, Zone 10 = ilvl 100)
- Item tier: T0-T9 quality roll (exponential curve: T0 38%, T9 0.1%). Stat multiplier: T0 0.40x to T9 1.00x. God items always T9
- Boss spawn: After 10 kills in subzone (5 kills to retry after boss death)
- Haven discovery: requires P10+, base chance 0.000014/tick + 0.000007 per rank above 10
- Challenge discovery: ~2hr avg per challenge (requires P1+)
- Soulforge discovery: requires P15+, base chance 0.000014/tick + 0.000007 per rank above 15
- Enhancement levels: 0-10, success rates 100% (+1-4), 60%/50%/40% (+5-7), 30%/20%/10% (+8-10)
- Enhancement costs: 1 PR (+1-4), 3 PR (+5-7), 5 PR (+8-9), 10 PR (+10)

## Combat Mechanics

- **Enemy scaling**: Static zone-based stats from `ZONE_ENEMY_STATS` table (not player-HP-based). Each zone has `(base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)` tuples; subzone depth adds incremental stats
- **Prestige combat bonuses**: `PrestigeCombatBonuses::from_rank()` provides flat damage, flat defense, crit chance, and flat HP that scale with prestige rank via power-law formulas
- **God item combat bonuses**: `GodItemCombatBonuses` struct injected into `update_combat()` — carries damage %, attack speed %, damage reduction %, and regen reduction % from equipped god items
- **Damage pipeline**: base damage → Giant's Might % → Haven Armory % → prestige flat damage → enemy defense → min 1 → Divine Bulwark DR → crit (2x)
- **Enemy attack intervals**: Vary by tier (2.0s normal, 1.8s boss, 1.5s zone boss, 1.6s dungeon elite, 1.4s dungeon boss)
- **Death to Boss**: Sets `kills_in_subzone = KILLS_FOR_BOSS - KILLS_FOR_BOSS_RETRY` (only 5 more kills to retry), preserves prestige
- **Death in Dungeon**: Exits dungeon, no prestige loss
- **Weapon Gates**: Zone 10 final boss requires Stormbreaker (checked via TheStormbreaker achievement)
- **Stormbreaker Path**: Max fishing rank → catch Storm Leviathan (10 encounters) → build Storm Forge in Haven → forge Stormbreaker
- **Zone 11 (The Expanse)**: Endgame wall with ~6.2x HP, ~4.6x DMG, ~4.8x DEF over Zone 10. Requires very high prestige (P50+) to farm comfortably

## Project Structure

```
quest/
├── src/
│   ├── main.rs              # Entry point, game loop, input handling
│   ├── lib.rs               # Library crate for testing
│   ├── input.rs             # Keyboard input routing
│   ├── tick_events.rs         # TickEvent → combat log mapping
│   ├── bin/
│   │   └── simulator.rs     # Headless game balance simulator
│   ├── core/                # Core game systems
│   │   ├── constants.rs     # Game balance constants
│   │   ├── game_logic.rs    # XP, leveling, spawning
│   │   ├── game_state.rs    # Main game state
│   │   └── tick.rs          # Per-tick game engine (game_tick)
│   ├── character/           # Character system [CLAUDE.md]
│   │   ├── attributes.rs    # 6 RPG attributes
│   │   ├── derived_stats.rs # Stats from attributes
│   │   ├── prestige.rs      # Prestige system
│   │   ├── manager.rs       # JSON saves
│   │   └── input.rs         # Character management input
│   ├── combat/              # Combat system [CLAUDE.md]
│   │   ├── types.rs         # Enemy, combat state
│   │   └── logic.rs         # Combat resolution
│   ├── zones/               # Zone system
│   │   ├── data.rs          # Zone definitions
│   │   └── progression.rs   # Zone progression
│   ├── dungeon/             # Dungeon system [CLAUDE.md]
│   │   ├── types.rs         # Room types, dungeon sizes
│   │   ├── generation.rs    # Procedural generation
│   │   └── logic.rs         # Navigation, clearing
│   ├── fishing/             # Fishing system
│   │   ├── types.rs         # Fish, phases, ranks
│   │   ├── generation.rs    # Fish generation
│   │   └── logic.rs         # Session processing
│   ├── items/               # Item system [CLAUDE.md]
│   │   ├── types.rs         # Items, slots, affixes
│   │   ├── equipment.rs     # Equipment container
│   │   ├── generation.rs    # Item generation
│   │   ├── drops.rs         # Drop system
│   │   ├── names.rs         # Name generation
│   │   └── scoring.rs       # Power scoring and auto-equip scoring
│   ├── enhancement/         # Soulforge enhancement system [CLAUDE.md]
│   │   ├── types.rs         # Enhancement progress, constants, UI state
│   │   ├── logic.rs         # Enhancement rolling, discovery
│   │   └── persistence.rs   # Save/load
│   ├── god_items/           # God Items system
│   │   └── types.rs         # 3 god items, passives, bonuses, helper queries
│   ├── challenges/          # Challenge minigames [CLAUDE.md]
│   │   ├── menu.rs          # Challenge menu
│   │   ├── chess/           # Chess minigame
│   │   ├── go/              # Go (Territory Control)
│   │   ├── morris/          # Nine Men's Morris
│   │   ├── gomoku/          # Gomoku (Five in a Row)
│   │   ├── minesweeper/     # Trap Detection
│   │   ├── rune/            # Rune Deciphering
│   │   ├── snake/           # Serpent's Path (Snake)
│   │   ├── flappy/          # Skyward Gauntlet (Flappy Bird)
│   │   ├── jezzball/        # Containment Breach (JezzBall)
│   │   └── runic_shift/     # Sigil Surge (panel-matching)
│   ├── haven/               # Haven base building [CLAUDE.md]
│   │   ├── types.rs         # Room definitions, bonuses
│   │   └── logic.rs         # Construction, upgrades
│   ├── achievements/        # Achievement system
│   │   ├── types.rs         # Achievement definitions
│   │   ├── data.rs          # Achievement database
│   │   └── persistence.rs   # Save/load
│   ├── utils/               # Utilities
│   │   ├── build_info.rs    # Build metadata
│   │   ├── updater.rs       # Self-update
│   │   └── debug_menu.rs    # Debug menu
│   └── ui/                  # UI components [CLAUDE.md]
│       ├── game_common.rs   # Shared minigame layout
│       ├── responsive.rs    # Responsive layout tiers
│       ├── stats_panel.rs   # Character stats
│       ├── ticker.rs        # Scrolling loot ticker
│       ├── combat_scene.rs  # Combat view
│       ├── combat_3d.rs     # 3D dungeon renderer
│       ├── snake_scene.rs   # Snake UI
│       ├── flappy_scene.rs  # Flappy Bird UI
│       ├── jezzball_scene.rs # JezzBall UI
│       ├── soulforge_scene.rs # Soulforge enhancement UI
│       ├── scene_fx.rs       # Shared utilities for layered ASCII scene rendering
│       ├── zone_bg.rs        # Stylized zone background scenes (6-layer compositing)
│       ├── *_scene.rs       # Various game scenes
│       └── character_*.rs   # Character management UI
├── tests/                   # Integration tests (16 test files, 2,900+ tests)
│   ├── game_loop_orchestration_test.rs  # 36 behavior-locking tests for game_tick
│   ├── tick_integration_test.rs         # Tick module integration tests
│   ├── zone_progression_test.rs         # Zone advancement tests
│   └── ...                              # Chess, fishing, dungeon, prestige, items, etc.
├── .github/workflows/       # CI/CD pipeline
├── scripts/                 # Quality checks
├── docs/                    # System design, balance, decisions, and per-system design docs
├── docs/archive/            # Original dated design documents
├── Makefile                 # Dev helpers
└── CLAUDE.md                # This file
```

## Dependencies

Ratatui 0.30, Serde (JSON), Rand 0.10, Rand_chacha 0.10 (seeded RNG for simulator), Chrono, Directories, Chess-engine 0.1, ureq 3.2, flate2 1.1, zip 8.0
