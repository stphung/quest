# Debug Menu Design

## Overview

Add a `--debug` flag and in-game menu to trigger chance-based discoveries for testing.

## Activation

```bash
cargo run -- --debug
```

When active, a `[DEBUG]` indicator shows in the UI corner.

## Menu Access

- Press backtick (`` ` ``) to toggle debug menu overlay
- Arrow keys to navigate, Enter to trigger, backtick to close

## Menu Options

1. **Trigger Dungeon** - Spawns a dungeon immediately
2. **Trigger Fishing** - Starts a fishing session immediately
3. **Trigger Chess Challenge** - Adds chess challenge to challenge menu
4. **Trigger Morris Challenge** - Adds morris challenge to challenge menu

## Implementation

### Files

- `src/debug_menu.rs` - Menu state and trigger logic
- `src/ui/debug_menu_scene.rs` - Menu rendering

### Changes to main.rs

- Parse `--debug` flag
- Store `debug_mode: bool`
- Handle backtick key when debug mode active
- Handle menu navigation when menu open

### Trigger Logic

Each option calls existing generation functions:
- Dungeon: `generate_dungeon()`
- Fishing: `generate_fishing_session()`
- Chess: `challenge_menu.add_challenge()` with chess challenge
- Morris: `challenge_menu.add_challenge()` with morris challenge

## UI Style

- Yellow border (matches challenge menu)
- Popup overlay centered on screen
- Help text at bottom
