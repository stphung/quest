# UI Module

Terminal UI rendering using Ratatui + Crossterm. All rendering is separated from game logic — UI files only read state and draw frames.

## Snapshot Tests (`snapshot_tests.rs`)

Full-frame snapshot tests render `draw_ui_with_update()` into a ratatui `TestBackend` and assert the buffer (characters + style runs, so color changes fail too) against committed [insta](https://insta.rs) snapshots in `snapshots/`. They run as part of `cargo test` — this is the primary way to verify UI changes without eyeballing screenshots.

- **Determinism**: the UI clock is frozen via `clock::freeze_at_millis()`, fixtures come from `crate::fixtures` with a fixed timestamp and seeded RNG, and `BUILD_COMMIT` in the footer is masked by an insta filter. `snapshot_rendering_is_deterministic` guards against regressions — if it fails, a wall-clock read, thread RNG, or unordered iteration leaked into the render path; fix that, never re-bless around it.
- **After an intentional UI change**: run `cargo test snapshot`, review the diff insta prints, then re-bless with `INSTA_UPDATE=always cargo test snapshot` (or `cargo insta review`) and commit the updated `.snap` files. For visual changes, also screenshot the real game with the `drive-game` skill.
- **Adding coverage**: build a state with `crate::fixtures` (or extend it), then call `assert_frame_snapshot("name", &state, w, h)`. One representative size per tier is enough.
- **Full-screen overlays** (`overlay_snapshot_tests.rs`): scenes (Haven, Deep, Loom, Soulforge, Stormglass, Time Vault, Vessel launch transition, Voyage) are snapshotted by calling their render entry points directly. Most are dispatched from `main_helpers/overlay.rs`'s `draw_game_overlays()`, but `vessel_scene::render_launch_transition` and `voyage_scene::render_voyage` are instead called directly from `main.rs`. Each assertion renders twice from independently built state and requires byte-identical frames first — this catches hidden mutable animation state (e.g. the Loom scene advances `throbber_frame` during render). Known exclusions and why are documented in the file header (Deep roster HashMap ordering, character-select splash `Utc::now()`, Stormglass rolling phases).

### UI Clock (`clock.rs`)

All wall-clock reads in rendering code go through `ui::clock` (`now_millis()`, `now_secs()`, `now_utc()`) so tests can freeze time. **Never call `SystemTime::now()`, `Utc::now()`, or `Instant::now()` directly from UI code** — it breaks snapshot determinism. Spinners/pulses should derive from `clock::now_millis()` (or `scene_fx::current_millis()`, which delegates to it).

## Module Structure

```
src/ui/
├── mod.rs                      # Main draw_ui_with_update(), layout coordinator
├── clock.rs                    # Freezable UI animation clock — sole source of wall-clock time for rendering
├── snapshot_tests.rs           # Full-frame insta snapshot tests of the main layout (committed snapshots/ dir)
├── overlay_snapshot_tests.rs   # Snapshot tests for full-screen overlays (Haven/Deep/Loom/Soulforge/Stormglass/Time Vault/Vessel/Voyage)
├── responsive.rs               # Responsive layout: SizeTier enum, LayoutContext, size thresholds
├── game_common.rs              # Shared minigame layout components (IMPORTANT)
├── stats_panel.rs              # Left panel: layout orchestration (delegates to helpers below)
├── stats_attributes.rs         # Attribute rendering helpers for stats panel
├── stats_equipment.rs          # Equipment rendering helpers for stats panel
├── stats_prestige.rs           # Prestige and fishing panel rendering helpers
├── ticker.rs                   # Scrolling loot ticker with independent per-entry scrolling
├── throbber.rs                 # Spinner animations and atmospheric messages
│
├── combat_scene.rs             # Combat view orchestration
├── combat_3d.rs                # First-person 3D ASCII dungeon renderer
├── combat_effects.rs           # Visual effects (damage numbers, flashes)
├── enemy_sprites.rs            # ASCII enemy sprite templates and archetype logic
├── enemy_sprite_data.rs        # Enemy sprite constant data, archetype mapping tables, zone suffix lookups
├── fracture_sprites_1.rs       # ASCII sprites for fracture zone enemies (zones 12-20)
├── fracture_sprites_2.rs       # ASCII sprites for fracture zone enemies (zones 21-30)
├── dungeon_map.rs              # Top-down dungeon minimap with fog of war
├── fishing_scene.rs            # Fishing UI with phase display
├── prestige_confirm.rs         # Prestige confirmation dialog
├── haven_scene.rs              # Haven base building overlay (delegates to helpers below)
├── haven_details.rs            # Haven room detail panel rendering
├── haven_tree.rs               # Haven skill tree panel rendering
├── achievement_browser_scene.rs # Achievement browsing (Combat/Level/Prestige/...) and stats tab
├── achievement_details.rs      # Achievement browser detail panel and split Level/Prestige stats view
├── achievement_list.rs         # Achievement browser list panel
├── achievement_tabs.rs         # Achievement browser category tabs, counts, and recent badges
├── title_browser_scene.rs      # Title browser overlay (select display title from unlocked achievements)
├── deep_scene.rs               # The Deep overlay coordinator, backdrop, view routing
├── deep_missions.rs            # Deep active missions panel and new mission creation
├── deep_roster.rs              # Deep mercenary roster sub-view
├── deep_layers.rs              # Deep layer map and infrastructure sub-view
├── deep_events.rs              # Deep check-in event response sub-view
├── deep_results.rs             # Deep mission complete modal
├── deep_shared.rs              # Shared Deep UI helpers (draw_deep_card, format_hours, truncate_text, render_progress_bar)
├── debug_menu_scene.rs         # Debug menu overlay with tabbed categories (Challenges, World, Resources, Items, Deep, Zones, Character, Soulforge, Loom, Borders)
├── bug_report_scene.rs         # Bug report overlay with game-state preview and clipboard status
│
├── challenge_menu_scene.rs     # Challenge menu list/detail view
├── chess_scene.rs              # Chess board with letter notation (K/Q/R/B/N/P)
├── go_scene.rs                 # Go board with territory display
├── morris_scene.rs             # Nine Men's Morris with help panel
├── gomoku_scene.rs             # Gomoku board with cursor
├── minesweeper_scene.rs        # Minesweeper grid with game-over overlay
├── rune_scene.rs               # Rune Deciphering with guess history
├── flappy_scene.rs             # Flappy Bird side-scroller (cyan border, pipe obstacles, bird)
├── snake_scene.rs              # Snake game (green border, 26×26 grid, body gradient, food)
├── jezzball_scene.rs           # JezzBall game (containment breach, wall-building, ball physics)
├── runic_shift_scene.rs        # Sigil Surge game (panel-matching, rune grid)
├── sudoku_scene.rs             # Sudoku (Sigil Matrix) puzzle with pencil marks
├── shard_fusion_scene.rs       # Shard Fusion (2048-style tile merging)
├── runic_lights_scene.rs       # Runic Lights pattern puzzle
├── vault_warden_scene.rs       # Vault Warden security puzzle
├── soulforge_scene.rs          # Soulforge enhancement overlay (delegates to helpers below)
├── soulforge_effects.rs        # Soulforge hammering/success/failure animation effects
├── soulforge_slots.rs          # Soulforge slot selection menu
├── loom_scene.rs               # Loom of Worlds overlay (graph view + detail panel)
├── loom_graph.rs               # Canvas-based Loom DAG graph renderer (edges with glow/particle animation, resource-colored nodes, selection highlighting)
├── ascension_scene.rs          # Ascension overlay UI (level display, cost/gate info, ascend confirmation)
├── stormglass_scene.rs         # Stormglass Exchange overlay with animations (Invoke Trial rolling, Chrono Surge speed ramp/fast-forward, Storm Sigils daily rotation, Storm Lure)
├── time_vault_scene.rs         # Time Vault overlay (branch/commit browser, restore, fork, GitHub cloud sync)
├── vessel_scene.rs             # Vessel discovery modal and construction/launch-confirmation overlay (Act 2 kill-switch gated)
├── vessel_transition_fx.rs     # 5-beat launch transition animation (Act 2 kill-switch gated)
├── voyage_scene.rs             # Act 2 "Crossing" main screen (chart/junction/trim/souls/watch/farewell views); imports vessel_scene::VESSEL_VIOLET
├── scene_fx.rs                 # Shared utilities for layered ASCII scene rendering (wide char support, SceneCell::new(), put_text_centered(), display_width())
├── overlay_layout.rs            # Shared overlay layout helpers for consistent overlay rendering
├── zone_bg.rs                  # Stylized zone background scenes (6-layer compositing, unique backgrounds for all 30 zones)
│
├── character_select.rs         # Character list with preview panel
├── character_creation.rs       # Name input with real-time validation
├── character_delete.rs         # Delete confirmation (type name to confirm)
└── character_rename.rs         # Rename with validation
```

## Responsive Layout (`responsive.rs`)

Terminal size is classified into 5 tiers, computed once per frame in a `LayoutContext`:

| Tier | Min Size | Layout |
|------|----------|--------|
| TooSmall | < 40×16 | Error message ("Terminal too small") |
| S (Small) | 40×16+ | Single-column, text-only combat |
| M (Medium) | 60×24+ | Stacked single-column with compact stats bar |
| L (Large) | 80×30+ | 2-column (stats left 50%, activity right 50%) |
| XL (Extra Large) | 120×40+ | 2-column with taller stats and equipment panels |

`LayoutContext` tracks independent `width_tier` and `height_tier` plus an effective `tier = min(width, height)`. Raw `cols`/`rows` are also available for fine-grained decisions.

Layout dispatch in `draw_ui_with_update()`:
- **XL/L**: `draw_xl_l_layout()` — full 2-column with zone info, info panels, footer
- **M**: `draw_m_layout()` — compact stats bar + optional attributes + XP bar + full-width activity + compact info + footer
- **S**: `draw_s_layout()` — minimal text: status line + XP + player HP + enemy HP + combat status + merged feed + footer. Special activities (minigames, fishing, dungeons) get nearly full screen.

## Main Layout (XL/L tiers)

```
┌──────────────────────────────────────────────┐
│ [Challenge Banner - 1 line, if pending]      │
├───────────────────────┬──────────────────────┤
│                       │                      │
│   Stats Panel (50%)   │  Combat Scene (50%)  │
│                       │                      │
├───────────────────────┴──────────────────────┤
│  Scrolling Ticker (1 line)                   │
├──────────────────────────────────────────────┤
│  Footer (4 lines / 2 content rows)           │
└──────────────────────────────────────────────┘
```

When a minigame is active, the right panel is replaced by the minigame scene.

## Combat HUD in Dungeons

When a dungeon is active, the right panel renders a single "Dungeon" panel with player/enemy HP bars, dungeon status, the map, and combat status all integrated inside one bordered block (no separate combat panel split). The dungeon uses a plain black background for clarity.

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
    let layout = create_game_layout(frame, area, " Title ", Color::Cyan, 15, 22, ctx);

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

Rarity colors are centralized in `mod.rs` via `pub fn rarity_color(rarity: Rarity) -> Color`. All UI code should use this function instead of inline matches.

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
| Rarity: Mythic | `Color::Rgb(255, 215, 0)` (Gold) |

Each minigame scene uses a unique border color (Cyan, Green, Yellow, Magenta, etc.).

## Rendering Principles

1. **Read-only**: UI functions only read `GameState` — never mutate it
2. **Frame-based**: Every tick renders the full frame; no incremental updates
3. **Ratatui widgets**: Use `Paragraph`, `Block`, `Borders`, `Layout`, `Span`/`Line` for all rendering
4. **Visibility control**: `pub` vs `mod` in `mod.rs` controls which scenes are accessible from outside
5. **Clear before render**: Use `frame.render_widget(Clear, area)` for overlays
