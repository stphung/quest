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

## Architecture

Entry point: `src/main.rs` — runs a 100ms tick game loop using Ratatui + Crossterm.

### Module Documentation

Larger modules have their own `CLAUDE.md` with implementation patterns, integration points, and extension guides:

- [`src/challenges/CLAUDE.md`](src/challenges/CLAUDE.md) — Adding new minigames (step-by-step checklist)
- [`src/items/CLAUDE.md`](src/items/CLAUDE.md) — Item generation pipeline, scoring, drop rates
- [`src/character/CLAUDE.md`](src/character/CLAUDE.md) — Attributes, prestige, persistence
- [`src/combat/CLAUDE.md`](src/combat/CLAUDE.md) — Combat state machine, enemy generation
- [`src/dungeon/CLAUDE.md`](src/dungeon/CLAUDE.md) — Procedural generation, room system
- [`src/haven/CLAUDE.md`](src/haven/CLAUDE.md) — Account-level base building, bonus system
- [`src/ui/CLAUDE.md`](src/ui/CLAUDE.md) — Shared game layout components, color conventions

### Core Module (`src/core/`)

- `game_state.rs` — Main character state struct (level, XP, prestige, combat state, equipment)
- `game_logic.rs` — XP curve (`100 × level^1.5`), leveling (+3 random attribute points), enemy spawning, offline progression
- `constants.rs` — Game balance constants (tick rate, attack interval, XP rates, item drop rates, update check jitter)

### Character Module (`src/character/`) — [detailed docs](src/character/CLAUDE.md)

- `attributes.rs` — 6 RPG attributes (STR, DEX, CON, INT, WIS, CHA), modifier = `(value - 10) / 2`
- `derived_stats.rs` — Combat stats calculated from attributes (HP, damage, defense, crit, XP mult)
- `prestige.rs` — Prestige tiers (Bronze→Eternal) with XP multipliers (`1+0.5×rank^0.7`, diminishing returns) and attribute cap increases (`10+rank×5`)
- `manager.rs` — Character CRUD operations (create, delete, rename), JSON save/load in ~/.quest/, name validation
- `input.rs` — Character selection, creation, deletion, renaming input handling and UI states

### Combat Module (`src/combat/`) — [detailed docs](src/combat/CLAUDE.md)

- `types.rs` — Enemy generation and combat state machine
- `logic.rs` — Turn-based combat mechanics, damage calculation, event emission

### Zone System (`src/zones/`)

- `data.rs` — 10 zones with 3-4 subzones each, prestige requirements, boss definitions
- `progression.rs` — Zone/subzone progression state, kill tracking (10 kills → boss spawn), weapon gates

**Zone Tiers:**
- P0: Meadow, Dark Forest (3 subzones each)
- P5: Mountain Pass, Ancient Ruins (3 subzones each)
- P10: Volcanic Wastes, Frozen Tundra (4 subzones each)
- P15: Crystal Caverns, Sunken Kingdom (4 subzones each)
- P20: Floating Isles, Storm Citadel (4 subzones each, Zone 10 requires Stormbreaker)

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

- `types.rs` — Core item data structures (7 equipment slots, 5 rarity tiers, 9 affix types, ilvl scaling)
- `equipment.rs` — Equipment container with slot management and iteration
- `generation.rs` — Rarity-based attribute/affix generation with ilvl scaling (1.0x at ilvl 10 to 4.0x at ilvl 100)
- `drops.rs` — Separate mob/boss drop systems: mobs have 15% base drop chance (capped at Epic), bosses always drop (can drop Legendary)
- `names.rs` — Procedural name generation with prefixes/suffixes
- `scoring.rs` — Smart weighted auto-equip scoring (attribute specialization bonus, affix type weights)

### Challenge Minigames (`src/challenges/`) — [detailed docs](src/challenges/CLAUDE.md)

- `mod.rs` — Shared enums (`ChallengeDifficulty`, `ChallengeResult`, `MinigameInput`), `ActiveMinigame` enum, unified `start_minigame()` and `apply_minigame_result()` functions
- `menu.rs` — Challenge menu system, `ChallengeType` with reward tables, flavor text, and discovery weights
- `chess/` — Chess minigame (4 difficulty levels: Novice→Master, ~500-1350 ELO), requires P1+
- `go/` — Go (Territory Control) on 9×9 board, MCTS AI with heuristics (500-20k simulations), requires P1+
- `morris/` — Nine Men's Morris (board layout, mill detection, phases), requires P1+
- `gomoku/` — Gomoku (Five in a Row) on 15×15 board, minimax AI (depth 2-5)
- `minesweeper/` — Trap Detection, 4 difficulties (9×9 to 20×16)
- `rune/` — Rune Deciphering (Mastermind-style deduction), 4 difficulties

### Haven Module (`src/haven/`) — [detailed docs](src/haven/CLAUDE.md)

- `types.rs` — Haven struct, 14 room definitions in a skill tree, upgrade tiers, 15 bonus types, Storm Forge
- `logic.rs` — Room construction, upgrade logic, bonus calculation, prestige rank cost system

Account-level base building that persists across prestiges. 14 rooms in a two-branch skill tree (combat + QoL) with 3 capstones (War Room, Vault, Storm Forge). Rooms provide bonuses (damage, XP, drop rate, rarity, crit, HP regen, double strike, offline XP, fishing, discovery). Costs prestige ranks. Discovered at P10+.

### Achievement Module (`src/achievements/`)

- `types.rs` — AchievementId enum, categories, unlock tracking
- `data.rs` — Achievement database with descriptions and unlock conditions
- `persistence.rs` — Save/load from `~/.quest/achievements.json`

Account-level achievement system that persists across characters. 5 categories (Combat, Level, Progression, Challenges, Exploration). Tracks kills, boss kills, levels, prestige, zone completion, challenge wins, fishing ranks/catches, dungeon completions, and Haven building. Includes modal notification system with 500ms accumulation window.

### Input Handling (`src/input.rs`)

Routes keyboard input to the appropriate handler based on current game state. Dispatches to minigame input handlers, character management flows, haven overlay, and debug menu. Minigame input uses a unified key→`MinigameInput` mapping with a single dispatch to per-game `process_input()` functions.

### Utilities (`src/utils/`)

- `build_info.rs` — Build metadata (commit, date) embedded at compile time
- `updater.rs` — Self-update from GitHub releases (30min check interval ±5min jitter)
- `debug_menu.rs` — Debug menu for testing discoveries (activate with `--debug` flag, toggle with backtick). Options: trigger dungeons, fishing, all 6 challenge types, Haven discovery
- `persistence.rs` — Generic JSON save/load helpers (`load_json_or_default<T>()`, `save_json<T>()`), used by haven and achievements

### UI (`src/ui/`) — [detailed docs](src/ui/CLAUDE.md)

- `mod.rs` — Layout coordinator (stats panel left 50%, combat scene right 50%)
- `game_common.rs` — Shared minigame layout, status bars, game-over overlays
- `stats_panel.rs` — Character stats, attributes, equipment display, prestige info
- `info_panel.rs` — Full-width Loot + Combat log panels
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
- `chess_scene.rs`, `go_scene.rs`, `morris_scene.rs`, `gomoku_scene.rs`, `minesweeper_scene.rs`, `rune_scene.rs` — Minigame UIs
- `debug_menu_scene.rs` — Debug menu overlay
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
All challenge minigames use the shared `ChallengeDifficulty` enum (Novice, Apprentice, Journeyman, Master) defined in `challenges/mod.rs`. Game-specific difficulty parameters (search depth, board size, etc.) are methods on the game struct.

### Forfeit Pattern
All interactive minigames: first Esc sets `forfeit_pending`, second Esc confirms, any other key cancels.

### Haven Bonus Injection
Haven bonuses are passed as explicit parameters rather than accessed globally. This keeps modules decoupled.

## Key Constants

- Tick interval: 100ms (10 ticks/sec)
- Attack interval: 1.5s
- HP regen after kill: 2.5s
- Autosave: every 30s
- Update check: every 30min ±5min jitter
- XP gain: Only from defeating enemies (200-400 XP per kill)
- Offline XP: 25% rate, max 7 days (simulates kills)
- Mob item drop rate: 15% base + 1% per prestige rank (capped at 25%), max rarity Epic
- Boss item drops: Guaranteed, can include Legendary (5% normal boss, 10% Zone 10 final boss)
- Item level: ilvl = zone_id × 10 (Zone 1 = ilvl 10, Zone 10 = ilvl 100)
- Boss spawn: After 10 kills in subzone
- Haven discovery: requires P10+, base chance 0.000014/tick + 0.000007 per rank above 10
- Challenge discovery: ~2hr avg per challenge (requires P1+)

## Combat Mechanics

- **Death to Boss**: Resets encounter (fighting_boss=false, kills_in_subzone=0) but preserves prestige
- **Death in Dungeon**: Exits dungeon, no prestige loss
- **Weapon Gates**: Zone 10 final boss requires Stormbreaker (checked via TheStormbreaker achievement)
- **Stormbreaker Path**: Max fishing rank → catch Storm Leviathan (10 encounters) → build Storm Forge in Haven → forge Stormbreaker

## Project Structure

```
quest/
├── src/
│   ├── main.rs              # Entry point, game loop, input handling
│   ├── lib.rs               # Library crate for testing
│   ├── input.rs             # Keyboard input routing
│   ├── core/                # Core game systems
│   │   ├── constants.rs     # Game balance constants
│   │   ├── game_logic.rs    # XP, leveling, spawning
│   │   └── game_state.rs    # Main game state
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
│   │   └── scoring.rs       # Auto-equip scoring
│   ├── challenges/          # Challenge minigames [CLAUDE.md]
│   │   ├── mod.rs           # Shared enums, start/apply functions
│   │   ├── menu.rs          # Challenge menu, rewards, flavor text
│   │   ├── chess/           # Chess minigame
│   │   ├── go/              # Go (Territory Control)
│   │   ├── morris/          # Nine Men's Morris
│   │   ├── gomoku/          # Gomoku (Five in a Row)
│   │   ├── minesweeper/     # Trap Detection
│   │   └── rune/            # Rune Deciphering
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
│   │   ├── persistence.rs   # Generic JSON save/load
│   │   └── debug_menu.rs    # Debug menu
│   └── ui/                  # UI components [CLAUDE.md]
│       ├── game_common.rs   # Shared minigame layout
│       ├── stats_panel.rs   # Character stats
│       ├── combat_scene.rs  # Combat view
│       ├── combat_3d.rs     # 3D dungeon renderer
│       ├── *_scene.rs       # Various game scenes
│       └── character_*.rs   # Character management UI
├── tests/                   # Integration tests
├── .github/workflows/       # CI/CD pipeline
├── scripts/                 # Quality checks
├── docs/design/             # Consolidated design documents
├── docs/archive/            # Original dated design documents
├── docs/DECISIONS.md        # Key design decisions and rationale
├── Makefile                 # Dev helpers
└── CLAUDE.md                # This file
```

## Dependencies

Ratatui 0.26, Crossterm 0.27, Serde (JSON), Rand, Chrono, Directories, Chess-engine 0.1
