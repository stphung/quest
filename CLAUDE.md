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

### Core Modules

- `game_state.rs` — Main character state struct (level, XP, prestige, combat state, equipment)
- `attributes.rs` — 6 RPG attributes (STR, DEX, CON, INT, WIS, CHA), modifier = `(value - 10) / 2`
- `derived_stats.rs` — Combat stats calculated from attributes (HP, damage, defense, crit, XP mult)
- `combat.rs` — Enemy generation and combat state machine
- `combat_logic.rs` — Turn-based combat mechanics, damage calculation, event emission
- `game_logic.rs` — XP curve (`100 × level^1.5`), leveling (+3 random attribute points), enemy spawning, offline progression
- `prestige.rs` — Prestige tiers (Bronze→Eternal) with XP multipliers (1.5× compounding) and attribute cap increases
- `constants.rs` — Game balance constants (tick rate, attack interval, XP rates)

### Zone System (`src/zones/`)

- `mod.rs` — Zone module exports and boss defeat result types
- `data.rs` — 10 zones with 3-4 subzones each, prestige requirements, boss definitions
- `progression.rs` — Zone/subzone progression state, kill tracking (10 kills → boss spawn), weapon gates

**Zone Tiers:**
- P0: Meadow, Dark Forest (3 subzones each)
- P5: Mountain Pass, Ancient Ruins (3 subzones each)
- P10: Volcanic Wastes, Frozen Tundra (4 subzones each)
- P15: Crystal Caverns, Sunken Kingdom (4 subzones each)
- P20: Floating Isles, Storm Citadel (4 subzones each, Zone 10 requires Stormbreaker)

### Dungeon System

- `dungeon.rs` — Room types (Entrance, Combat, Treasure, Elite, Boss), room state (Hidden, Revealed, Current, Cleared), dungeon sizes
- `dungeon_generation.rs` — Procedural dungeon generation with connected rooms
- `dungeon_logic.rs` — Dungeon navigation, room clearing, key system, safe death (no prestige loss)

**Dungeon Sizes:** Small 5×5, Medium 7×7, Large 9×9, Epic 11×11 (based on prestige)

### Fishing System

- `fishing.rs` — Fish rarities (Common→Legendary), fishing phases (Casting, Waiting, Reeling), 30 ranks across 6 tiers
- `fishing_generation.rs` — Fish name generation and rarity rolling
- `fishing_logic.rs` — Fishing session tick processing

### Item System

- `items.rs` — Core item data structures (7 equipment slots, 5 rarity tiers, 12 affix types)
- `equipment.rs` — Equipment container with slot management and iteration
- `item_generation.rs` — Rarity-based attribute/affix generation (Common: +1-2 attrs, Legendary: +8-15 attrs + 4-5 affixes)
- `item_drops.rs` — Drop system (15% base + 1% per prestige rank capped at 25%, continuous rarity distribution)
- `item_names.rs` — Procedural name generation with prefixes/suffixes
- `item_scoring.rs` — Smart weighted auto-equip scoring (attribute specialization bonus, affix type weights)

### Character System

- `character_manager.rs` — Character CRUD operations (create, delete, rename), JSON save/load in ~/.quest/, name validation
- `save_manager.rs` — Legacy binary save/load (deprecated, used for migration only)

### UI (`src/ui/`)

- `mod.rs` — Layout coordinator (stats panel left 50%, combat scene right 50%)
- `stats_panel.rs` — Character name header, stats, attributes, derived stats, equipment display, prestige info, fishing rank
- `combat_scene.rs` — Combat view orchestration with HP bars
- `combat_3d.rs` — 3D ASCII first-person dungeon renderer
- `combat_effects.rs` — Visual effects (damage numbers, attack flashes, hit impacts)
- `enemy_sprites.rs` — ASCII enemy sprite templates
- `perspective.rs` — Wall/floor perspective rendering
- `dungeon_map.rs` — Top-down dungeon minimap
- `fishing_scene.rs` — Fishing UI with phase display
- `prestige_confirm.rs` — Prestige confirmation dialog
- `character_select.rs` — Character selection screen with detailed preview panel
- `character_creation.rs` — Character creation with real-time name validation
- `character_delete.rs` — Delete confirmation requiring exact name typing
- `character_rename.rs` — Character renaming with validation

### Utilities

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

## Combat Mechanics

- **Death to Boss**: Resets encounter (fighting_boss=false, kills_in_subzone=0) but preserves prestige
- **Death in Dungeon**: Exits dungeon, no prestige loss
- **Weapon Gates**: Zone 10 final boss requires Stormbreaker weapon

## Project Structure

```
quest/
├── src/
│   ├── main.rs           # Entry point, game loop, input handling
│   ├── game_state.rs     # Core game state
│   ├── game_logic.rs     # XP, leveling, spawning
│   ├── combat.rs         # Enemy struct, combat state
│   ├── combat_logic.rs   # Combat resolution
│   ├── attributes.rs     # 6 RPG attributes
│   ├── derived_stats.rs  # Stats from attributes
│   ├── prestige.rs       # Prestige system
│   ├── constants.rs      # Game constants
│   ├── zones/            # Zone system
│   │   ├── mod.rs
│   │   ├── data.rs       # Zone definitions
│   │   └── progression.rs
│   ├── dungeon.rs        # Dungeon types
│   ├── dungeon_generation.rs
│   ├── dungeon_logic.rs
│   ├── fishing.rs        # Fishing types
│   ├── fishing_generation.rs
│   ├── fishing_logic.rs
│   ├── items.rs          # Item types
│   ├── equipment.rs
│   ├── item_generation.rs
│   ├── item_drops.rs
│   ├── item_names.rs
│   ├── item_scoring.rs
│   ├── character_manager.rs  # JSON saves
│   ├── save_manager.rs       # Legacy saves
│   └── ui/               # UI components
│       ├── mod.rs
│       ├── stats_panel.rs
│       ├── combat_scene.rs
│       ├── combat_3d.rs
│       ├── combat_effects.rs
│       ├── enemy_sprites.rs
│       ├── perspective.rs
│       ├── dungeon_map.rs
│       ├── fishing_scene.rs
│       ├── prestige_confirm.rs
│       ├── character_select.rs
│       ├── character_creation.rs
│       ├── character_delete.rs
│       └── character_rename.rs
├── .github/workflows/ci.yml  # CI/CD pipeline
├── scripts/ci-checks.sh      # Quality checks
├── docs/plans/               # Design documents
├── Makefile                  # Dev helpers
└── CLAUDE.md                 # This file
```

## Dependencies

Ratatui 0.26, Crossterm 0.27, Serde (JSON), SHA2, Rand, Chrono, Directories
