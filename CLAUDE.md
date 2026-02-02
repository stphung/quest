# Quest - Terminal-Based Idle RPG

A terminal-based idle RPG written in Rust. Your hero automatically battles enemies, gains XP, levels up, and prestiges.

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

## Pull Request Workflow

**IMPORTANT: When creating PRs, always validate CI status and fix any issues before marking as ready.**

1. **Create feature branch:**
   ```bash
   git checkout -b fix/description-of-fix
   # or
   git checkout -b feat/description-of-feature
   ```

2. **Make changes and commit:**
   ```bash
   git add <files>
   git commit -m "type: description"
   ```

3. **Push and create PR:**
   ```bash
   git push -u origin <branch-name>
   gh pr create --title "type: description" --body "..."
   ```

4. **Validate CI passes:**
   ```bash
   gh pr checks <pr-number>
   ```

5. **If CI fails, fix the issues:**
   - **Format failures:** Run `cargo fmt` and commit
   - **Clippy warnings:** Fix the warnings and commit
   - **Test failures:** Fix tests and commit
   - **Build failures:** Fix compilation errors and commit

6. **Push fixes and re-validate:**
   ```bash
   git push
   # Wait for CI to complete
   gh pr checks <pr-number>
   ```

7. **Confirm all checks pass** before requesting review or merging.

## CI/CD Pipeline

**On every PR:**
- Runs `scripts/ci-checks.sh` (format, lint, test, build, audit)
- Must pass to merge (if branch protection enabled)

**On push to main:**
- Runs all checks
- Builds release binaries for 4 platforms (Linux, macOS x86/ARM, Windows)
- Creates GitHub release with downloadable binaries

**Key insight:** Local `make check` runs the **exact same script** as CI, ensuring consistency.

## Architecture

Entry point: `src/main.rs` — runs a 100ms tick game loop using Ratatui + Crossterm.

### Core Modules

- `game_state.rs` — Main character state struct (level, XP, prestige, combat state)
- `attributes.rs` — 6 RPG attributes (STR, DEX, CON, INT, WIS, CHA), modifier = `(value - 10) / 2`
- `derived_stats.rs` — Combat stats calculated from attributes (HP, damage, defense, crit, XP mult)
- `combat.rs` — Enemy generation and combat state machine
- `combat_logic.rs` — Turn-based combat mechanics, damage calculation, event emission
- `game_logic.rs` — XP curve (`100 × level^1.5`), leveling (+3 random attribute points), enemy spawning, offline progression
- `prestige.rs` — Prestige ranks (Bronze→Celestial) with XP multipliers and attribute cap increases
- `save_manager.rs` — Legacy binary save/load (deprecated, used for migration only)
- `character_manager.rs` — JSON save/load in ~/.quest/ directory
- `constants.rs` — Game balance constants (tick rate, attack interval, XP rates)

### Item System

- `items.rs` — Core item data structures (7 equipment slots, 5 rarity tiers, 12 affix types)
- `equipment.rs` — Equipment container with slot management and iteration
- `item_generation.rs` — Rarity-based attribute/affix generation (Common: +1-2 attrs, Legendary: +8-15 attrs + 4-5 affixes)
- `item_drops.rs` — Prestige-scaled drop system (30% base + 5% per prestige rank, tier-based rarity distribution)
- `item_names.rs` — Procedural name generation with prefixes/suffixes
- `item_scoring.rs` — Smart weighted auto-equip scoring (attribute specialization bonus, affix type weights)

### Character System

- `character_manager.rs` — Character CRUD operations (create, delete, rename), JSON save/load with SHA256 checksums, name validation and sanitization
- `ui/character_select.rs` — Character selection screen with detailed preview panel
- `ui/character_creation.rs` — Character creation with real-time name validation
- `ui/character_delete.rs` — Delete confirmation requiring exact name typing
- `ui/character_rename.rs` — Character renaming with validation

### UI (`src/ui/`)

- `mod.rs` — Layout coordinator (stats panel left 50%, combat scene right 50%)
- `stats_panel.rs` — Character stats, attributes, derived stats, equipment display, prestige info
- `combat_scene.rs` — Combat view orchestration with HP bars
- `combat_3d.rs` — 3D ASCII first-person dungeon renderer
- `combat_effects.rs` — Visual effects (damage numbers, attack flashes, hit impacts)
- `enemy_sprites.rs` — ASCII enemy sprite templates
- `ascii_scaler.rs` — Sprite scaling algorithm
- `perspective.rs` — Wall/floor perspective rendering
- `zones.rs` — Zone progression display

## Key Constants

- Tick interval: 100ms (10 ticks/sec)
- Attack interval: 1.5s
- HP regen after kill: 2.5s
- Autosave: every 30s
- XP gain: Only from defeating enemies (200-400 XP per kill)
- Offline XP: 50% rate, max 7 days (simulates kills)
- Item drop rate: 30% base + 5% per prestige rank

## Project Structure

```
quest/
├── src/              # Rust source code
├── .github/
│   └── workflows/
│       └── ci.yml    # CI/CD pipeline (calls scripts/ci-checks.sh)
├── scripts/
│   ├── ci-checks.sh  # Single source of truth for all quality checks
│   └── README.md     # Scripts documentation
├── docs/
│   └── plans/        # Design documents and implementation plans
├── Makefile          # Development helpers (make check, make fmt, etc.)
└── CLAUDE.md         # This file
```

## Dependencies

Ratatui 0.26, Crossterm 0.27, Serde/Bincode, SHA2, Rand, Chrono, Directories
