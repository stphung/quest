# Quest - Terminal-Based Idle RPG

A terminal-based idle RPG written in Rust. Your hero automatically battles enemies, gains XP, levels up, explores dungeons, and prestiges.

## Build & Run

```bash
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

### Core Module (`src/core/`)

- `game_state.rs` — Main character state struct (level, XP, prestige, combat state, equipment)
- `game_logic.rs` — XP curve (`100 × level^1.5`), leveling (+3 random attribute points), enemy spawning, offline progression
- `constants.rs` — Game balance constants (tick rate, attack interval, XP rates)

### Character Module (`src/character/`)

- `attributes.rs` — 6 RPG attributes (STR, DEX, CON, INT, WIS, CHA), modifier = `(value - 10) / 2`
- `derived_stats.rs` — Combat stats calculated from attributes (HP, damage, defense, crit, XP mult)
- `prestige.rs` — Prestige tiers (Bronze→Eternal) with XP multipliers (1.5× compounding) and attribute cap increases
- `manager.rs` — Character CRUD operations (create, delete, rename), JSON save/load in ~/.quest/, name validation
- `save.rs` — Legacy binary save/load (deprecated, used for migration only)

### Combat Module (`src/combat/`)

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

### Dungeon Module (`src/dungeon/`)

- `types.rs` — Room types (Entrance, Combat, Treasure, Elite, Boss), room state (Hidden, Revealed, Current, Cleared), dungeon sizes
- `generation.rs` — Procedural dungeon generation with connected rooms
- `logic.rs` — Dungeon navigation, room clearing, key system, safe death (no prestige loss)

**Dungeon Sizes:** Small 5×5, Medium 7×7, Large 9×9, Epic 11×11 (based on prestige)

### Fishing Module (`src/fishing/`)

- `types.rs` — Fish rarities (Common→Legendary), fishing phases (Casting, Waiting, Reeling), 30 ranks across 6 tiers
- `generation.rs` — Fish name generation and rarity rolling
- `logic.rs` — Fishing session tick processing

### Item Module (`src/items/`)

- `types.rs` — Core item data structures (7 equipment slots, 5 rarity tiers, 12 affix types)
- `equipment.rs` — Equipment container with slot management and iteration
- `generation.rs` — Rarity-based attribute/affix generation (Common: +1-2 attrs, Legendary: +8-15 attrs + 4-5 affixes)
- `drops.rs` — Drop system (15% base + 1% per prestige rank capped at 25%, continuous rarity distribution)
- `names.rs` — Procedural name generation with prefixes/suffixes
- `scoring.rs` — Smart weighted auto-equip scoring (attribute specialization bonus, affix type weights)

### Challenge Minigames (`src/challenges/`)

- `menu.rs` — Generic challenge menu system (pending challenges, extensible challenge types)
- `chess/` — Chess minigame (4 difficulty levels: Novice→Master, ~500-1350 ELO), requires P1+
- `go/` — Go (Territory Control) on 9×9 board, MCTS AI with heuristics (500-20k simulations), requires P1+
- `morris/` — Nine Men's Morris (board layout, mill detection, phases), requires P1+
- `gomoku/` — Gomoku (Five in a Row) on 15×15 board, minimax AI (depth 2-5)
- `minesweeper/` — Trap Detection, 4 difficulties (9×9 to 20×16)
- `rune/` — Rune Deciphering (Mastermind-style deduction), 4 difficulties

### Utilities (`src/utils/`)

- `build_info.rs` — Build metadata (commit, date) embedded at compile time
- `updater.rs` — Self-update functionality
- `debug_menu.rs` — Debug menu for testing discoveries (activate with `--debug` flag, toggle with backtick)

### UI (`src/ui/`)

- `mod.rs` — Layout coordinator (stats panel left 50%, combat scene right 50%)
- `stats_panel.rs` — Character name header, stats, attributes, derived stats, equipment display, prestige info, fishing rank
- `combat_scene.rs` — Combat view orchestration with HP bars
- `combat_3d.rs` — 3D ASCII first-person dungeon renderer
- `combat_effects.rs` — Visual effects (damage numbers, attack flashes, hit impacts)
- `enemy_sprites.rs` — ASCII enemy sprite templates
- `dungeon_map.rs` — Top-down dungeon minimap
- `fishing_scene.rs` — Fishing UI with phase display
- `prestige_confirm.rs` — Prestige confirmation dialog
- `challenge_menu_scene.rs` — Challenge menu list/detail view rendering
- `chess_scene.rs` — Chess board UI with move history and game-over overlay
- `go_scene.rs` — Go board UI with territory display and pass/forfeit controls
- `morris_scene.rs` — Nine Men's Morris board UI with help panel
- `gomoku_scene.rs` — Gomoku board UI with cursor navigation
- `minesweeper_scene.rs` — Minesweeper grid UI with game-over overlay
- `debug_menu_scene.rs` — Debug menu overlay rendering
- `throbber.rs` — Shared spinner/throbber animations and atmospheric waiting messages
- `character_select.rs` — Character selection screen with detailed preview panel
- `character_creation.rs` — Character creation with real-time name validation
- `character_delete.rs` — Delete confirmation requiring exact name typing
- `character_rename.rs` — Character renaming with validation

### Utilities

- `lib.rs` — Library crate exposing game logic modules for testing
- `build_info.rs` — Build metadata (commit, date) embedded at compile time
- `updater.rs` — Self-update functionality

## Key Constants

- Tick interval: 100ms (10 ticks/sec)
- Attack interval: 1.5s
- HP regen after kill: 2.5s
- Autosave: every 30s
- XP gain: Only from defeating enemies (200-400 XP per kill)
- Offline XP: 50% rate, max 7 days (simulates kills)
- Item drop rate: 15% base + 1% per prestige rank (capped at 25%)
- Boss spawn: After 10 kills in subzone
- Minigame discovery: ~2hr avg per challenge (0.000014 chance/tick), requires P1+

## Combat Mechanics

- **Death to Boss**: Resets encounter (fighting_boss=false, kills_in_subzone=0) but preserves prestige
- **Death in Dungeon**: Exits dungeon, no prestige loss
- **Weapon Gates**: Zone 10 final boss requires Stormbreaker weapon

## Project Structure

```
quest/
├── src/
│   ├── main.rs              # Entry point, game loop, input handling
│   ├── lib.rs               # Library crate for testing
│   ├── core/                # Core game systems
│   │   ├── constants.rs     # Game balance constants
│   │   ├── game_logic.rs    # XP, leveling, spawning
│   │   └── game_state.rs    # Main game state
│   ├── character/           # Character system
│   │   ├── attributes.rs    # 6 RPG attributes
│   │   ├── derived_stats.rs # Stats from attributes
│   │   ├── prestige.rs      # Prestige system
│   │   ├── manager.rs       # JSON saves
│   │   └── save.rs          # Legacy saves
│   ├── combat/              # Combat system
│   │   ├── types.rs         # Enemy, combat state
│   │   └── logic.rs         # Combat resolution
│   ├── zones/               # Zone system
│   │   ├── data.rs          # Zone definitions
│   │   └── progression.rs   # Zone progression
│   ├── dungeon/             # Dungeon system
│   │   ├── types.rs         # Room types, dungeon sizes
│   │   ├── generation.rs    # Procedural generation
│   │   └── logic.rs         # Navigation, clearing
│   ├── fishing/             # Fishing system
│   │   ├── types.rs         # Fish, phases, ranks
│   │   ├── generation.rs    # Fish generation
│   │   └── logic.rs         # Session processing
│   ├── items/               # Item system
│   │   ├── types.rs         # Items, slots, affixes
│   │   ├── equipment.rs     # Equipment container
│   │   ├── generation.rs    # Item generation
│   │   ├── drops.rs         # Drop system
│   │   ├── names.rs         # Name generation
│   │   └── scoring.rs       # Auto-equip scoring
│   ├── challenges/          # Challenge minigames
│   │   ├── menu.rs          # Challenge menu
│   │   ├── chess/           # Chess minigame
│   │   ├── go/              # Go (Territory Control)
│   │   ├── morris/          # Nine Men's Morris
│   │   ├── gomoku/          # Gomoku (Five in a Row)
│   │   ├── minesweeper/     # Trap Detection
│   │   └── rune/            # Rune Deciphering
│   ├── utils/               # Utilities
│   │   ├── build_info.rs    # Build metadata
│   │   ├── updater.rs       # Self-update
│   │   └── debug_menu.rs    # Debug menu
│   └── ui/                  # UI components
│       ├── stats_panel.rs   # Character stats
│       ├── combat_scene.rs  # Combat view
│       ├── combat_3d.rs     # 3D dungeon renderer
│       ├── *_scene.rs       # Various game scenes
│       └── character_*.rs   # Character management UI
├── tests/                   # Integration tests
├── .github/workflows/       # CI/CD pipeline
├── scripts/                 # Quality checks
├── docs/plans/              # Design documents
├── Makefile                 # Dev helpers
└── CLAUDE.md                # This file
```

## Dependencies

Ratatui 0.26, Crossterm 0.27, Serde (JSON), SHA2, Rand, Chrono, Directories, Chess-engine 0.1
