# Quest - Terminal-Based Idle RPG

A terminal-based idle RPG written in Rust. Your hero automatically battles enemies, gains XP, levels up, and prestiges.

## Build & Run

```bash
cargo build            # Build
cargo run              # Run the game
cargo test             # Run tests
cargo clippy           # Lint
```

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
- `save_manager.rs` — Binary save/load with SHA256 checksums, autosave, backward-compatible migration
- `constants.rs` — Game balance constants (tick rate, attack interval, XP rates)

### UI (`src/ui/`)

- `mod.rs` — Layout coordinator (stats panel left 50%, combat scene right 50%)
- `stats_panel.rs` — Character stats, attributes, derived stats, prestige info
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
- Offline XP: 50% rate, max 7 days

## Dependencies

Ratatui 0.26, Crossterm 0.27, Serde/Bincode, SHA2, Rand, Chrono, Directories
