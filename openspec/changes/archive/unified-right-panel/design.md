> Backported design record. Sources: docs/plans/2026-03-03-unified-right-panel-design.md.

## 2026-03-03-unified-right-panel-design.md

# Unified Right Panel Design

## Problem

The right side of the screen (combat, fishing, dungeon views) lacks visual consistency. Each scene renders with its own layout, borders, and status bar placement, making the UI feel disjointed.

## Solution

Wrap the entire right panel in a single bordered frame with inner horizontal dividers, creating three subpanels: zone info, content, and a context-sensitive status strip.

## Layout

```
┌─ The Volcanic Wastes ─────────────┐
│ Subzone 2/4 • Lava Pits           │  ← Zone Info (2-3 rows)
│ ████████░░░░ 7/10 kills [Boss: 3] │
├───────────────────────────────────┤
│                                   │
│  ░▒▓ Zone background + enemy ▓▒░  │  ← Content (fills remaining)
│  ░▒▓   sprite / fishing /    ▓▒░  │
│  ░▒▓   dungeon map here     ▓▒░  │
│                                   │
├───────────────────────────────────┤
│ HP 340/500 ██████░░ | Atk 0.8s    │  ← Status Strip (2 rows)
│ Foe: Fire Drake 120/200 ██░░░     │
└───────────────────────────────────┘
```

## Approach

**Approach A: Outer Block wrapper** — a single Ratatui `Block::bordered()` wraps the right panel area. The inner area splits into three vertical chunks (zone info, content, status strip) separated by horizontal dividers.

### Why This Approach

- Minimal change to existing scene renderers — they receive a slightly smaller `Rect` but render the same content.
- HP bars and status info extracted to a shared status strip function, eliminating per-scene status rendering.
- Zone backgrounds, enemy sprites, and all art remain identical.

## Subpanel Details

### Zone Info (top, 2-3 rows)

The zone name moves to the `Block` title. Inside the top subpanel:
- Row 1: Subzone name and progress (e.g., "Subzone 2/4 • Lava Pits")
- Row 2: Kill progress bar + boss countdown
- Zone completion dot track (if height allows, XL tier)

### Content (middle, fills remaining space)

Same scene dispatch as today, same priority order:
1. Minigames → **excluded** (keep their own frames)
2. Challenge menu
3. Fishing (water animation fills the area)
4. Dungeon (map fills the area)
5. Combat (3D sprite + zone background)

Each scene renderer receives the content `Rect` and renders exactly as today, minus its HP bars and status lines which move to the strip.

### Status Strip (bottom, 2 rows)

A shared `draw_status_strip()` function inspects game state and renders context-sensitive status:

| Activity | Row 1 | Row 2 |
|----------|-------|-------|
| Combat | `HP 340/500 ████░░ | Atk 0.8s DPS 42` | `Foe: Fire Drake 120/200 ███░ | Atk 1.2s` |
| Combat (boss) | Same + enrage timer | Same |
| Fishing | `Rank: Expert 22 ████░░ 3/5 fish` | `🐟 Reeling in! | Caught: 12/20` |
| Dungeon | `HP 340/500 ████░░ | Room 7/25 🔑×2` | `Foe: Dungeon Elite 80/150 ██░` |
| Idle | `HP 500/500 ████████ | Waiting...` | *(empty or zone flavor text)* |

## Border Style

- Outer border: `Block::bordered()` with `Borders::ALL`
- Block title: zone name (e.g., `" The Volcanic Wastes "`)
- Border color: zone accent color from `zone_bg.rs`, default `Color::DarkGray`
- Inner dividers: horizontal lines at zone info / content and content / status boundaries

## Scope

### In Scope (L/XL tiers only)
- `draw_right_panel()` in `src/ui/mod.rs` — add outer Block, split inner area into 3 chunks
- `draw_right_content()` — receives smaller content Rect, dispatches as before
- New `draw_status_strip()` — shared status rendering based on activity
- `combat_scene.rs` — remove HP bars and combat status from scene rendering
- `fishing_scene.rs` — remove bottom status box and header rank info from scene
- `draw_dungeon_view()` in `mod.rs` — remove HP bars and status; dungeon map fills content area
- `stats_panel.rs` `draw_zone_info()` — compact into the bordered zone info subpanel

### Out of Scope
- Minigame rendering (10 games keep their own frames via `game_common.rs`)
- M/S tier layouts (too small for borders, unchanged)
- Game logic (`game_tick()`, all modules) — zero impact
- Input handling — zero impact
- Overlays (Haven, Deep, Soulforge, Time Vault, etc.) — render on top, unaffected
- Ticker and footer — below the right panel, unchanged
- Zone background compositing pipeline — same, just smaller rect
- Enemy sprites — same art, same rendering
