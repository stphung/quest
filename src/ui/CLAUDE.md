# UI Module

Terminal UI rendering using Ratatui + Crossterm. All rendering is separated from game logic — UI files only read state and draw frames.

## Module Structure

```
src/ui/
├── mod.rs                    # Main draw_ui_with_update(), layout coordinator
├── game_common.rs            # Shared minigame layout components (IMPORTANT)
├── stats_panel.rs            # Left panel: character stats, attributes, equipment
├── info_panel.rs             # Full-width Loot + Combat log panels
├── throbber.rs               # Spinner animations and atmospheric messages
│
├── combat_scene.rs           # Combat view orchestration
├── combat_3d.rs              # First-person 3D ASCII dungeon renderer
├── combat_effects.rs         # Visual effects (damage numbers, flashes)
├── enemy_sprites.rs          # ASCII enemy sprite templates
├── dungeon_map.rs            # Top-down dungeon minimap with fog of war
├── fishing_scene.rs          # Fishing UI with phase display
├── prestige_confirm.rs       # Prestige confirmation dialog
├── haven_scene.rs            # Haven base building overlay
├── achievement_browser_scene.rs # Achievement browsing
├── debug_menu_scene.rs       # Debug menu overlay
│
├── challenge_menu_scene.rs   # Challenge menu list/detail view
├── chess_scene.rs            # Chess board with move history
├── go_scene.rs               # Go board with territory display
├── morris_scene.rs           # Nine Men's Morris with help panel
├── gomoku_scene.rs           # Gomoku board with cursor
├── minesweeper_scene.rs      # Minesweeper grid with game-over overlay
├── rune_scene.rs             # Rune Deciphering with guess history
├── flappy_scene.rs           # Flappy Bird side-scroller with pipe obstacles
├── snake_scene.rs            # Snake game with grid and growing snake
│
├── character_select.rs       # Character list with preview panel
├── character_creation.rs     # Name input with real-time validation
├── character_delete.rs       # Delete confirmation (type name to confirm)
└── character_rename.rs       # Rename with validation
```

## Main Layout (`mod.rs`)

The main game screen is laid out as:
```
┌──────────────────────────────────────────────┐
│ [Challenge Banner - 1 line, if pending]      │
├───────────────────────┬──────────────────────┤
│                       │                      │
│   Stats Panel (50%)   │  Combat Scene (50%)  │
│                       │                      │
├───────────────────────┴──────────────────────┤
│  Loot Panel + Combat Log (full-width, 8h)    │
├──────────────────────────────────────────────┤
│  Footer (1 line)                             │
└──────────────────────────────────────────────┘
```

When a minigame is active, the right panel is replaced by the minigame scene.

## Shared Game Components (`game_common.rs`)

This is the most important file for implementing new minigame UIs. It provides:

### `create_game_layout()`
Standardized layout for all minigame scenes:
```
┌─ Title ─────────────────────────┬─ Info ──────┐
│                                 │             │
│   [content area]                │  [info]     │
│                                 │             │
│ [status bar - 2 lines]          │             │
└─────────────────────────────────┴─────────────┘
```

Returns `GameLayout { content, status_bar, info_panel }`.

### Status Bar Renderers
- `render_status_bar()` — Normal controls display with key hints
- `render_thinking_status_bar()` — AI thinking spinner
- `render_forfeit_status_bar()` — Forfeit confirmation prompt

### Game Over Overlay
- `render_game_over_overlay()` — Win/Loss/Draw overlay with result-specific colors

## Adding a New Minigame Scene

Follow the pattern established by existing scenes:

```rust
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_status_bar, render_thinking_status_bar, GameResultType,
};

pub fn render_newgame_scene(frame: &mut Frame, area: Rect, game: &NewGameGame) {
    // 1. Game over overlay takes priority
    if game.game_result.is_some() {
        render_game_over(frame, area, game);
        return;
    }

    // 2. Create standardized layout
    let layout = create_game_layout(frame, area, " Title ", Color::Cyan, 15, 22);

    // 3. Render board/content
    render_board(frame, layout.content, game);

    // 4. Render status bar (AI thinking → forfeit → normal)
    render_status_bar_content(frame, layout.status_bar, game);

    // 5. Render info panel
    render_info(frame, layout.info_panel, game);
}
```

Then register in `mod.rs` and dispatch from `draw_ui_with_update()`.

## Color Conventions

| Element | Color |
|---------|-------|
| Player pieces/text | `Color::White` |
| AI/enemy pieces | `Color::LightRed` |
| Cursor highlight | `Color::Yellow` |
| Last move highlight | `Color::Green` |
| Grid/board lines | `Color::DarkGray` |
| Win result | `Color::Green` |
| Loss result | `Color::Red` |
| Draw result | `Color::Yellow` |
| Rarity: Common | `Color::White` |
| Rarity: Magic | `Color::Blue` |
| Rarity: Rare | `Color::Yellow` |
| Rarity: Epic | `Color::Magenta` |
| Rarity: Legendary | `Color::LightRed` |

Each minigame scene uses a unique border color (Cyan, Green, Yellow, Magenta, etc.).

## Rendering Principles

1. **Read-only**: UI functions only read `GameState` — never mutate it
2. **Frame-based**: Every tick renders the full frame; no incremental updates
3. **Ratatui widgets**: Use `Paragraph`, `Block`, `Borders`, `Layout`, `Span`/`Line` for all rendering
4. **Visibility control**: `pub` vs `mod` in `mod.rs` controls which scenes are accessible from outside
5. **Clear before render**: Use `frame.render_widget(Clear, area)` for overlays
