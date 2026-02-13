# Responsive UI Audit: Current Layout Components

## Table of Contents

1. [Main Game Layout](#1-main-game-layout)
2. [Left Panel: Stats Panel](#2-left-panel-stats-panel)
3. [Right Panel: Activity Area](#3-right-panel-activity-area)
4. [Bottom Panel: Info Panel](#4-bottom-panel-info-panel)
5. [Footer](#5-footer)
6. [Overlays and Modals](#6-overlays-and-modals)
7. [Full-Screen Views](#7-full-screen-views)
8. [Shared Components](#8-shared-components)
9. [Summary: Hardcoded Dimensions](#9-summary-hardcoded-dimensions)
10. [Component Priority Classification](#10-component-priority-classification)

---

## 1. Main Game Layout

**File:** `src/ui/mod.rs:41-140` (`draw_ui_with_update()`)

The main game screen uses a vertical-then-horizontal layout:

```
+----------------------------------------------+
| [Challenge Banner - 1 line, conditional]      |
+----------------------+------------------------+
|                      |   Zone Info (4h)       |
|  Stats Panel (50%)   +------------------------+
|                      |   Right Content        |
|                      |   (Min 10h)            |
+----------------------+------------------------+
|  Loot Panel (50%)    |  Combat Log (50%)      | <- 8h fixed
+----------------------+------------------------+
|  Footer (3h)                                  |
+----------------------------------------------+
| [Update Drawer - 12h, conditional]            |
+----------------------------------------------+
```

**Hardcoded constraints:**
- Challenge banner: `Length(1)` (conditional)
- Main content: `Min(0)` (flexible)
- Info panel (loot + combat log): `Length(8)` fixed
- Update drawer: `Length(12)` (conditional)
- Footer: `Length(3)` fixed
- Horizontal split: `Percentage(50)` / `Percentage(50)`

**Minimum implied height:** 8 (info) + 3 (footer) + some content = ~20 rows minimum for usability

---

## 2. Left Panel: Stats Panel

**File:** `src/ui/stats_panel.rs:33-62` (`draw_stats_panel()`)

Vertical layout within the left 50%:

| Section | Constraint | Contents | Essential? |
|---------|-----------|----------|------------|
| Header (name, level, XP bar) | `Length(4)` | Character name, level, rank, playtime, XP gauge | YES |
| Prestige Info | `Length(7)` | Rank, multiplier, CHA bonus, resets, fishing rank, fishing progress bar | YES |
| Attributes | `Length(14)` | 6 attributes x 2 lines each (emoji, name, value, modifier, cap) + borders | YES |
| Derived Stats | `Length(6)` | Max HP, physical/magic damage, defense, crit%, XP mult + borders | MEDIUM |
| Equipment | `Min(16)` | 7 equipment slots, each 1-3 lines (name+rarity, attrs, affixes) | MEDIUM |

**Total fixed height:** 4 + 7 + 14 + 6 = 31 lines, plus equipment needs Min(16) = **47 lines minimum**

**Key observations:**
- The header gracefully degrades: at `inner.height >= 2` shows text + XP bar, at 1 shows just XP bar
- Prestige info gracefully degrades: at `inner.height >= 5` shows full layout, at 1 shows just fishing bar
- Attributes are rigid: 6 x `Length(2)` = 12 lines (no degradation)
- Equipment truncates item names at 28 chars
- No width constraints; relies on percentage-based parent

---

## 3. Right Panel: Activity Area

**File:** `src/ui/mod.rs:181-248` (`draw_right_panel()`, `draw_right_content()`)

The right panel always has a fixed zone info header:

| Section | Constraint | Contents |
|---------|-----------|----------|
| Zone Info | `Length(4)` | Zone name, subzone, boss progress, next zone status |
| Right Content | `Min(10)` | Dispatched by activity (see below) |

### 3a. Combat Scene

**File:** `src/ui/combat_scene.rs:15-48`

| Section | Constraint | Contents | Essential? |
|---------|-----------|----------|------------|
| Player HP bar | `Length(1)` | Gauge with HP values | YES |
| Sprite + 3D view | `Min(5)` | Enemy ASCII sprite (10 lines tall), centered | YES |
| Enemy HP bar | `Length(1)` | Gauge or "Regenerating..."/"Spawning..." | YES |
| Combat Status | `Length(1)` | Spinner, attack timers, DPS | MEDIUM |

- `combat_3d.rs:14`: Has explicit minimum check: `if area.height < 3 || area.width < 20` shows "Area too small"
- Enemy sprites are all 14 chars wide x 10 lines tall (from `enemy_sprites.rs`)

### 3b. Dungeon View

**File:** `src/ui/mod.rs:251-275` (`draw_dungeon_view()`)

| Section | Constraint | Contents |
|---------|-----------|----------|
| Dungeon Map | `Min(map_height)` | `grid_size * 2 + 3` height; grid sizes: 5/7/9/11/13 |
| Combat Scene | `Min(5)` | Same as combat scene above |

**Map heights by dungeon size:**
- Small (5x5): `5*2+3 = 13` rows
- Medium (7x7): `7*2+3 = 17` rows
- Large (9x9): `9*2+3 = 21` rows
- Epic (11x11): `11*2+3 = 25` rows
- Legendary (13x13): `13*2+3 = 29` rows

**Map widths:** `grid_size * 4` cells wide (emoji = 2 chars + 2 for corridor)
- Small: 20, Medium: 28, Large: 36, Epic: 44, Legendary: 52

### 3c. Fishing Scene

**File:** `src/ui/fishing_scene.rs:39-67`

| Section | Constraint | Contents | Essential? |
|---------|-----------|----------|------------|
| Header | `Length(3)` | Spot name, bordered | YES |
| Water Animation | `Min(6)` | ASCII water + bobber | YES |
| Catch Progress | `Length(4)` | Caught X/Y fish, phase status | YES |
| Rank Info | `Length(5)` | Rank name, progress bar | MEDIUM |

**Total minimum:** 3 + 6 + 4 + 5 = **18 lines**

### 3d. Challenge Menu

**File:** `src/ui/challenge_menu_scene.rs:21-29`

Two views:
- **List view:** Bordered list of pending challenges, help text at bottom
- **Detail view:** Description (4h) + spacer + difficulty selector (5h) + spacer + outcomes (1h) + spacer + help (1h). ~13 lines minimum

### 3e. Minigame Scenes (all use `create_game_layout`)

All minigames use the shared layout from `game_common.rs:44-80`:

```
+------ Title -------------------------+---- Info ---+
|                                      |             |
|   [content area]                     |  [info]     |
|                                      |             |
| [status bar - 2 lines]              |             |
+--------------------------------------+-------------+
```

**Layout constraints:**
- Horizontal: `Min(20)` for content | `Length(info_panel_width)` for info
- Vertical (left side): `Min(content_min_height)` | `Length(2)` for status bar

| Minigame | Content Min Height | Info Width | Board Dimensions |
|----------|-------------------|------------|-----------------|
| Chess | 19 (1 move history + 18 board) | 22 | 43w x 18h (grid: 5*8+3 = 43w) |
| Go | 11 | 24 | 25w x 9h (BOARD_SIZE=9) |
| Morris | 13 | 24 | 25w x 13h |
| Gomoku | 15 | 22 | 29w x 15h (BOARD_SIZE=15, cells=2w) |
| Minesweeper | 10 | 24 | Grid varies: 9x9=18w, 16x16=32w, 30x16=60w, 20x16=40w |
| Rune | 6 | 22 | Variable height (guess history grows) |
| Flappy Bird | 15 | 18 | GAME_WIDTH x GAME_HEIGHT (scalable) |
| Snake | 20 | 16 | grid_width x (grid_height/2) (half-block rendering) |

---

## 4. Bottom Panel: Info Panel

**File:** `src/ui/info_panel.rs:12-23`

**Fixed height:** 8 rows (set by parent layout)

Horizontal split: 50% / 50%:
- **Left: Loot Panel** - Recent drops with rarity colors, 2-line format for equipment (name + stats)
- **Right: Combat Log** - Newest-first combat entries, color-coded (green=player, red=enemy, yellow=crit)

Both panels use `inner.height` to calculate max visible entries. Truncates messages to `inner.width`.

**Essential?** YES - Primary feedback for idle gameplay

---

## 5. Footer

**File:** `src/ui/stats_panel.rs:778-877` (`draw_footer()`)

**Fixed height:** 3 rows (border + 1 content line + border)

Contains horizontally:
- `[Esc] Quit`
- `[P] Prestige (Available!)` or `[P] Prestige (Need Lv.X)`
- `[H] Haven` (conditional on discovery)
- `[A] Achievements` (with pending count)
- `[Tab] Challenges` (conditional on pending count)
- Update status (checking spinner / up to date / new version)
- Version info in title bar

**Essential?** YES - Primary navigation and status

---

## 6. Overlays and Modals

All overlays use `frame.render_widget(Clear, area)` before rendering, and are centered on screen.

| Overlay | File | Dimensions | Trigger |
|---------|------|-----------|---------|
| Prestige Confirm | `prestige_confirm.rs` | 50w x 18h (capped to screen-4) | Press [P] |
| Achievement Unlocked | `achievement_browser_scene.rs:302` | 50w x 9-20h | Achievement triggers |
| Haven Discovery | `haven_scene.rs:427` | 50w x 7h | First Haven discovery |
| Haven Build Confirm | `haven_scene.rs:473` | 45w x 9h | Build in Haven |
| Storm Forge Confirm | `haven_scene.rs:547` | 50w x 12h | Forge Stormbreaker |
| Leviathan Encounter | `fishing_scene.rs:350` | 64w x 16h | Leviathan event |
| Offline Welcome | `game_common.rs:310` | 44w x 10-11h | Return after offline |
| Debug Menu | `debug_menu_scene.rs:13` | 35w x (options+4)h | Backtick key (debug mode) |
| Game Over (full) | `game_common.rs:179` | Full area, centered content 7h | Minesweeper/Rune end |
| Game Over (banner) | `game_common.rs:239` | Full width, 4-5h at bottom | Chess/Go/Morris/Gomoku end |

**Essential?** All are essential when shown (modal interactions)

---

## 7. Full-Screen Views

These replace the entire terminal, not just the right panel:

### 7a. Character Select

**File:** `src/ui/character_select.rs:24-111`

Layout:
- Title: `Length(3)`
- Main content (40%/60% horizontal split): Character list | Character details
- Haven tree (conditional, discovered): `Length(19)` - 17 lines of diamond layout
- Controls: `Length(4)`
- Margin: 2 on all sides

**Minimum dimensions:** ~40w x 30h (without Haven), ~40w x 49h (with Haven tree)

### 7b. Character Creation

**File:** `src/ui/character_creation.rs:26-111`

Layout: Title(3) + spacer(1) + input(4) + spacer(1) + rules(4) + validation(2) + filler + controls(3) = ~18 minimum + margins

### 7c. Character Delete

**File:** `src/ui/character_delete.rs:26-109`

Layout: Title(3) + spacer(1) + details(min) + spacer(1) + warning(5) + spacer(1) + input(4) + spacer(1) + controls(3) = ~19 minimum + margins

### 7d. Character Rename

**File:** `src/ui/character_rename.rs:28-126`

Layout: Title(3) + spacer(1) + details(min) + spacer(1) + input(4) + spacer(1) + rules(4) + validation(2) + spacer(1) + controls(3) = ~20 minimum + margins

### 7e. Achievement Browser

**File:** `src/ui/achievement_browser_scene.rs:85-128`

Full-screen overlay with:
- Category tabs: `Length(3)`
- Content (45%/55% horizontal split): Achievement list | Achievement detail
- Help: `Length(1)`

### 7f. Haven Tree

**File:** `src/ui/haven_scene.rs:43-94`

Full-screen overlay with:
- Summary bar: `Length(2)`
- Main content (40%/60% horizontal split): Skill tree list | Room detail
- Help: `Length(1)`

### 7g. Vault Selection (during prestige)

**File:** `src/ui/haven_scene.rs:642-755`

Full-screen overlay with:
- Instructions: `Length(2)`
- Item list: `Min(0)`
- Help: `Length(1)`

---

## 8. Shared Components

### 8a. Throbber / Spinner

**File:** `src/ui/throbber.rs`

- `spinner_char()`: Braille spinner (10 frames, 100ms cycle)
- `waiting_message(seed)`: 20 atmospheric messages for idle time

**Dimensions:** Single character + message text. No size constraints.

### 8b. Game Common Layout

**File:** `src/ui/game_common.rs`

- `create_game_layout()`: Standardized minigame layout
- `render_status_bar()`: 2-line status with controls
- `render_thinking_status_bar()`: AI thinking spinner
- `render_forfeit_status_bar()`: Forfeit confirmation
- `render_game_over_overlay()`: Full-area game over
- `render_game_over_banner()`: Bottom banner (4-5h)
- `render_info_panel_frame()`: Bordered info panel
- `render_offline_welcome()`: Centered 44x10 modal
- `format_number_short()`: Number abbreviation (K/M/B/T/Q)

### 8c. Enemy Sprites

**File:** `src/ui/enemy_sprites.rs`

6 sprite templates, all 14 chars wide x 10 lines tall:
- `SPRITE_ORC`, `SPRITE_TROLL`, `SPRITE_DRAKE`
- `SPRITE_BEAST`, `SPRITE_HORROR`, `SPRITE_CRUSHER`

### 8d. Debug Indicators

**File:** `src/ui/debug_menu_scene.rs:73-131`

- `render_debug_indicator()`: "[DEBUG] Saves disabled" top-right (22 chars)
- `render_save_indicator()`: "Saved HH:MM AM" or spinner, top-right

---

## 9. Summary: Hardcoded Dimensions

### Fixed Heights (cannot shrink)

| Component | Height | Location |
|-----------|--------|----------|
| Challenge banner | 1 | `mod.rs:61` |
| Info panel (loot+combat) | 8 | `mod.rs:82,91` |
| Footer | 3 | `mod.rs:84,92` |
| Update drawer | 12 | `mod.rs:83` |
| Zone info | 4 | `mod.rs:191` |
| Stats header | 4 | `stats_panel.rs:40` |
| Stats prestige | 7 | `stats_panel.rs:41` |
| Stats attributes | 14 | `stats_panel.rs:42` |
| Stats derived | 6 | `stats_panel.rs:43` |
| Stats equipment (min) | 16 | `stats_panel.rs:44` |

### Fixed Widths

| Component | Width | Location |
|-----------|-------|----------|
| Prestige confirm modal | 50 | `prestige_confirm.rs:16` |
| Haven discovery modal | 50 | `haven_scene.rs:429` |
| Haven build confirm | 45 | `haven_scene.rs:481` |
| Storm forge confirm | 50 | `haven_scene.rs:554` |
| Leviathan modal | 64 | `fishing_scene.rs:357` |
| Offline welcome modal | 44 | `game_common.rs:312` |
| Debug menu | 35 | `debug_menu_scene.rs:15` |
| Chess board | 43 | `chess_scene.rs:45` |
| Morris board | 25 | `morris_scene.rs:78` |
| Go board | 25 | `go_scene.rs:36` (9*3-2) |
| Gomoku board | 29 | `gomoku_scene.rs:44` (15*2-1) |

### Percentage-Based Splits

| Split | Ratio | Location |
|-------|-------|----------|
| Stats / Right panel | 50% / 50% | `mod.rs:109-110` |
| Loot / Combat log | 50% / 50% | `info_panel.rs:15-16` |
| Haven tree / detail | 40% / 60% | `haven_scene.rs:76-77` |
| Character list / details | 40% / 60% | `character_select.rs:58-59` |
| Achievement list / detail | 45% / 55% | `achievement_browser_scene.rs:117-118` |
| Minigame content / info | `Min(20)` / `Length(W)` | `game_common.rs:64-65` |

---

## 10. Component Priority Classification

### Tier 1: Critical (must always be visible during gameplay)

- **Player HP** - survival feedback
- **Enemy HP / combat status** - combat feedback
- **XP bar** - progression feedback
- **Zone info** - location awareness
- **Footer controls** - navigation
- **Loot panel** - idle reward feedback
- **Combat log** - action feedback

### Tier 2: Important (should be visible when space allows)

- **Character level + name** - identity
- **Prestige rank + multiplier** - progression context
- **Attributes (condensed)** - build awareness
- **Equipment (names only)** - gear awareness
- **Fishing rank** - secondary progression
- **Challenge banner** - pending notification

### Tier 3: Nice-to-Have (can be hidden or accessed via overlay)

- **Derived stats** - calculable from attributes
- **Equipment affixes/details** - secondary item info
- **Attribute modifiers** - derived from values
- **Attribute caps** - rarely changes
- **Play time** - non-gameplay info
- **DPS calculation** - derived stat
- **Update status** - non-gameplay
- **Save indicator** - non-gameplay

### Tier 4: On-Demand Only (overlays/modals)

- **Prestige confirmation dialog**
- **Achievement browser**
- **Haven tree**
- **Vault selection**
- **Game over overlays**
- **Offline welcome**
- **Debug menu**
- **Character management screens**

### Currently Unresponsive Components

The following components have **zero graceful degradation** and will break at small sizes:

1. **Stats attributes section** - rigid 6x `Length(2)` = 12 lines
2. **Equipment section** - `Min(16)` with no condensed mode
3. **Info panel** - rigid `Length(8)` with no shrink
4. **Footer** - rigid `Length(3)`, all controls on one line (will truncate)
5. **Zone info** - rigid `Length(4)`
6. **All minigame boards** - fixed pixel dimensions
7. **Dungeon map** - scales with dungeon size, can overflow
8. **Modals** - hardcoded widths (35-64 chars), only cap to screen with `min()`
9. **50/50 split** - no breakpoint to stack vertically

### Components with Some Graceful Degradation

1. **Stats header** - degrades at `inner.height < 2` (XP bar only)
2. **Prestige info** - degrades at `inner.height < 5` (fishing bar only)
3. **Combat 3D** - shows "Area too small" at `height < 3 || width < 20`
4. **Flappy Bird / Snake** - scale render buffer to available area
5. **Prestige confirm modal** - caps to `screen - 4`
6. **Leviathan / Haven modals** - cap with `.min(area.width)` / `.min(area.height)`
