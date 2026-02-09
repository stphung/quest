# Terminal Idle RPG - Design Document

**Date:** 2026-01-31
**Type:** Fantasy RPG incremental game
**Platform:** Terminal (cross-platform)
**Language:** Rust

## Overview

A fantasy RPG incremental game that runs in the terminal with minimal player interaction. The game features autonomous skill progression, animated combat scenes, and a deep prestige system. Players watch their character grow, fight enemies, and progress through zones with occasional prestige decisions.

## Core Features

- **4 Core Stats**: Strength, Magic, Wisdom, Vitality
- **Autonomous Progression**: All stats train simultaneously with no player input required
- **Hybrid Time System**: Real-time progression while running, offline catch-up when returning
- **Deep Prestige**: 15+ reincarnation ranks with exponential multipliers
- **Animated Combat**: Visual combat scene with enemies, attacks, and zone progression
- **Secure Saves**: Binary serialization with SHA256 checksumming

## Architecture

### High-Level Structure

Single-threaded event loop using ratatui with 100ms tick interval (10 ticks/second):

1. Initialize terminal, load save file (or create new game)
2. Enter event loop with tick-based progression
3. Each iteration: handle input → advance game state → render UI
4. On exit: save game state to disk

**Core Components:**
- `GameState` - Holds all game data (stats, levels, prestige rank, timestamps)
- `GameLogic` - Handles progression calculations (XP gains, level-ups, offline catch-up)
- `UI` - Renders terminal interface using ratatui
- `SaveManager` - Handles serialization/deserialization with checksums
- `PrestigeSystem` - Manages prestige tiers, unlocks, and multipliers

**Data Flow:**
Tick event → GameLogic updates GameState → UI reads GameState → renders to terminal

## Stats and Progression

### Core Stats

1. **Strength (STR)** - Physical power, combat prowess
2. **Magic (MAG)** - Magical ability, arcane knowledge
3. **Wisdom (WIS)** - Mental fortitude, insight
4. **Vitality (VIT)** - Health, endurance, resilience

### Leveling Mechanics

- Each stat has independent level (starts at 1) and XP
- **XP curve**: `xp_needed = 100 * (level ^ 1.5)`
  - Level 1→2: 100 XP
  - Level 2→3: 282 XP
  - Level 10→11: 3,162 XP
- **XP gain rate**: Base 1 XP per tick per stat (10 XP/second, 36,000 XP/hour)
- All stats train simultaneously (fully autonomous)
- Multiplied by prestige bonuses

### Visual Feedback

- Each stat shows: current level, XP bar with percentage, XP/sec rate
- Level-ups trigger brief highlight animation
- Adventurer Rank titles at milestones (Novice at 10, Adept at 25, Master at 50)

## Prestige System

### Reincarnation Ranks

15+ prestige tiers with exponential requirements and bonuses:

1. **Rank 1 - "First Rebirth"**: All stats level 20+ → 2x XP multiplier
2. **Rank 2 - "Second Rebirth"**: All stats level 30+ → 3x XP multiplier
3. **Rank 3 - "Third Rebirth"**: All stats level 40+ → 4.5x XP multiplier
4. **Rank 5**: Level 60+ → 10x multiplier
5. **Rank 10**: Level 100+ → 50x multiplier
6. **Rank 15**: Level 150+ → 250x multiplier
7. Continues scaling...

### Mechanics

- Manual trigger when requirements met (press 'P')
- Resets all stats to level 1
- Grants permanent multiplicative XP bonuses
- Unlocks new titles and potentially mythical zones at high ranks
- Creates satisfying loop: grind → prestige → faster grinding → repeat

## Time and Progression

### Active (Real-time) Progression

- Game loop runs at 100ms tick interval (10 ticks/second)
- Each tick: apply XP gains based on current multipliers
- Example: At Rank 3 (4.5x), each tick adds 4.5 XP to each stat
- Smooth progress bar animations
- `last_save_time` timestamp updated periodically

### Offline Progression

- On load, calculate `elapsed_seconds = now - last_save_time`
- Apply offline multiplier (0.5x) to balance against active play
- Calculate total offline XP: `(elapsed_seconds * 10) * base_xp * prestige_mult * 0.5`
- Distribute to each stat and process level-ups
- Display "Welcome back" message with summary
- Cap offline gains at 7 days maximum

### Edge Cases

- Clock moved backwards: Treat as 0 elapsed time
- Very long absence (>7 days): Cap at 7 days worth of progress
- First launch: Initialize `last_save_time` to current time

### Save Timing

- Auto-save every 30 seconds during active play
- Always save on clean exit
- Crash recovery: lose at most 30 seconds of progress

## Save File Format

### Location

`~/.config/idle-rpg/save.dat` (platform-appropriate config directory)

### Binary Structure

Using **bincode** for serialization:

```
[8 bytes: version magic]
[4 bytes: data length]
[N bytes: serialized GameState]
[32 bytes: SHA256 checksum]
```

### GameState Structure

```rust
struct GameState {
    stats: [Stat; 4],           // STR, MAG, WIS, VIT
    prestige_rank: u32,
    total_prestige_count: u64,
    last_save_time: i64,        // Unix timestamp
    play_time_seconds: u64,
    // ... other metadata
}

struct Stat {
    level: u32,
    current_xp: u64,
}
```

### Integrity Checking

- On save: compute SHA256 hash of (version + length + data), append to file
- On load: recompute hash, compare with stored hash
- Mismatch → reject load, offer "start new game"
- Prevents casual tampering without encryption complexity

## Terminal UI Design

### Layout Structure

```
┌──────────────────────────────────────────────────────────┐
│ IDLE RPG - Rank 3: Third Rebirth (4.5x XP) - DARK FOREST│
├──────────────────────────────────────────────────────────┤
│                    🌲  🌲    🌲                          │
│          ⚔️                                              │
│     [HERO]  ─→ ─→ ─→  💥  [GOBLIN]                     │
│       ⚔️                                                 │
│                    🌲      🌲  🌲                        │
│                                                          │
│  STR 42 ████████░░ 68%  MAG 38 ███████░░░ 52%          │
│  WIS 40 █████████░ 73%  VIT 41 ██████░░░░ 45%          │
│                                                          │
│  Adventurer: MASTER  |  Playtime: 2h 34m                │
├──────────────────────────────────────────────────────────┤
│  [P] Prestige (Ready!)  [Q] Quit                        │
└──────────────────────────────────────────────────────────┤
```

### Combat Scene (Top Section)

**Elements:**
- **Hero sprite**: ASCII art character (left side), changes with prestige rank
- **Enemy sprites**: Zone-appropriate enemies, respawn every 2-3 seconds
- **Environment**: Zone-specific decorations (trees, rocks, ruins, etc.)
- **Combat effects**:
  - Attack animations: Arrows `─→` traveling from hero to enemy
  - Impact effects: `💥`, `✨`, floating damage numbers
  - Enemy defeat: Dissolve animation, new enemy spawns
- **Zone transitions**: Environment updates when entering new zones

### Zones & Progression

Zones unlock based on average stat level:

1. **Meadow** (Lvl 1-10): Grassland, slimes, rats
2. **Dark Forest** (Lvl 10-25): Trees, goblins, wolves
3. **Mountain Pass** (Lvl 25-50): Rocks, orcs, trolls
4. **Ancient Ruins** (Lvl 50-75): Pillars, undead, wraiths
5. **Volcanic Wastes** (Lvl 75-100): Lava, demons, dragons
6. Higher prestige ranks: Mythical zones

### Stats Panel

- **Progress bars**: `█` for filled, `░` for empty
- Smooth updates every tick (100ms)
- Flash/highlight on level-up
- Color-coded by stat (STR=red, MAG=blue, WIS=purple, VIT=green)

### Status Area

- Current adventurer rank title
- Total playtime
- Prestige availability indicator

### Color Scheme

- Strength: Red/orange tones
- Magic: Blue/cyan tones
- Wisdom: Purple/magenta tones
- Vitality: Green tones
- Prestige ready: Bright yellow highlight

## Project Structure

### Dependencies (Cargo.toml)

```toml
[dependencies]
ratatui = "0.26"           # Terminal UI framework
crossterm = "0.27"         # Terminal control
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"            # Binary serialization
sha2 = "0.10"              # Checksumming
chrono = "0.4"             # Time handling
rand = "0.8"               # Random enemy spawns
directories = "5.0"        # Cross-platform config dirs
```

### Module Structure

```
src/
├── main.rs              # Entry point, event loop
├── game_state.rs        # GameState struct, stat definitions
├── game_logic.rs        # Progression calculations, level-ups
├── prestige.rs          # Prestige tier definitions, unlock logic
├── save_manager.rs      # Serialization, checksums, file I/O
├── ui/
│   ├── mod.rs           # UI coordinator
│   ├── combat_scene.rs  # Combat visualization, animations
│   ├── stats_panel.rs   # Progress bars, stat display
│   └── zones.rs         # Zone definitions, enemy spawns
└── constants.rs         # XP curves, tick rate, multipliers
```

### Key Module Responsibilities

- `main.rs`: Initialize terminal, run event loop, coordinate components
- `game_logic.rs`: Pure functions for XP calculations, offline catch-up
- `ui/combat_scene.rs`: ASCII art, animation states, combat rendering
- `ui/zones.rs`: Zone unlock conditions, environment, enemy pools
- `save_manager.rs`: Isolated file I/O, checksumming

## Development Approach

1. **Phase 1**: Core progression (game_state + game_logic + save_manager)
2. **Phase 2**: Basic UI with stats panel (no combat scene yet)
3. **Phase 3**: Combat scene and zone system
4. **Phase 4**: Polish animations and visual effects

## Design Principles

- **YAGNI**: No features beyond what's specified
- **Simplicity**: Single-threaded, tick-based is sufficient
- **Satisfying**: Smooth animations, clear progression feedback
- **Respectful**: Offline progress rewards checking back
- **Secure enough**: Save tampering deterred but not impossible
