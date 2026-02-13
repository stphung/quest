# Responsive UI: Game Information Hierarchy

This document defines what information matters at each terminal size, which elements
can be hidden or simplified, and minimum size requirements for interactive content.

---

## 1. Information Priority Tiers

### Tier 1: Critical (must show at ANY size)

These elements answer the core idle-game question: **"Am I progressing?"**

| Element | Current Source | Why Critical |
|---------|---------------|-------------|
| Character name + level | `stats_panel::draw_header` | Identity and progress anchor |
| XP bar (ratio or %) | `stats_panel::draw_header` | Primary progress indicator |
| Player HP / Enemy HP | `combat_scene` | Are we alive? Are we winning? |
| Combat status (fighting/regen/idle) | `combat_scene::draw_combat_status` | What is happening right now |
| Current zone + subzone | `stats_panel::draw_zone_info` | Where are we |
| Footer hotkeys ([Esc], [P], [Tab]) | `stats_panel::draw_footer` | How to interact |

**Minimum viable display (~40 cols x 12 rows):**
```
Hero (Lv.15 Warrior)  XP: 73%
Zone 3: Mountain Pass (2/3)
HP: ████████░░ 80/100
Foe: ██░░░░░░░ 25/90
[fighting] DPS: 42
[Esc] [P] Prestige [Tab] Challenges
```

### Tier 2: Important (show at small+, ~60x20)

These elements significantly enhance gameplay understanding.

| Element | Current Source | Why Important |
|---------|---------------|-------------|
| Prestige rank + tier name | `stats_panel::draw_prestige_info` | Key progression metric |
| Prestige XP multiplier | `stats_panel::draw_prestige_info` | Explains progression speed |
| Boss progress (kills until boss) | `stats_panel::draw_zone_info` | Near-term goal |
| Combat log (last 2-3 entries) | `info_panel::draw_combat_log` | Feedback on combat events |
| Loot panel (last 1-2 drops) | `info_panel::draw_recent_gains` | Reward feedback |
| Fishing rank (if fishing) | `stats_panel::draw_prestige_info` | Fishing progress |
| Dungeon status (if in dungeon) | `dungeon_map::DungeonStatusWidget` | Dungeon progress |
| Enemy sprite (simplified) | `combat_3d::render_combat_3d` | Visual engagement |
| Next zone hint | `stats_panel::draw_zone_info` | Long-term goal |

### Tier 3: Nice-to-have (show at medium+, ~100x30)

These add depth but a player can survive without them.

| Element | Current Source | Why Nice |
|---------|---------------|---------|
| All 6 attributes with values | `stats_panel::draw_attributes` | Detailed build info |
| Derived stats (all 6) | `stats_panel::draw_derived_stats` | Combat math |
| Equipment list (names + rarity) | `stats_panel::draw_equipment_section` | Gear overview |
| Full combat log (6+ entries) | `info_panel::draw_combat_log` | Detailed combat history |
| Full loot panel (5+ drops) | `info_panel::draw_recent_gains` | Drop history |
| Play time | `stats_panel::draw_header` | Session tracking |
| Fishing progress bar | `stats_panel::draw_prestige_info` | Granular fishing progress |
| DPS calculation | `combat_scene::draw_combat_status` | Optimization info |
| Attack timers (You/Foe) | `combat_scene::draw_combat_status` | Combat timing |
| Challenge banner | `draw_challenge_banner` | Discovery notification |
| Dungeon minimap (small) | `dungeon_map::DungeonMapWidget` | Spatial awareness |

### Tier 4: Full Detail (large only, ~120x40+)

The current full layout, as designed.

| Element | Current Source | Notes |
|---------|---------------|-------|
| Equipment with attribute bonuses | `stats_panel::draw_equipment_section` | Lines 2-3 per item |
| Equipment affixes | `stats_panel::draw_equipment_section` | Detailed item stats |
| Attribute progress bars + caps | `stats_panel::draw_attribute_row` | Build ceiling info |
| Full dungeon map | `dungeon_map::DungeonMapWidget` | Large maps (9x9+) |
| 3D ASCII combat rendering | `combat_3d::render_combat_3d` | Atmospheric visuals |
| Update drawer (changelog) | `stats_panel::draw_update_drawer` | Version info |
| Prestige multiplier breakdown | `stats_panel::draw_prestige_info` | CHA bonus detail |
| Prestige reset count | `stats_panel::draw_prestige_info` | Historical stat |

---

## 2. Current Layout Dimensions

The current layout requires approximately **100 cols x 40 rows** minimum:

```
Stats panel: 50% width (~50 cols)
  - Header:      4 rows
  - Prestige:    7 rows
  - Attributes: 14 rows
  - Derived:     6 rows
  - Equipment:  16+ rows

Right panel: 50% width (~50 cols)
  - Zone info:   4 rows
  - Combat/game: remaining (10+ rows)

Info panel: full width, 8 rows
  - Loot (50%) + Combat log (50%)

Footer: 3 rows
```

---

## 3. Minigame Size Requirements

Each minigame has hard minimum dimensions below which it cannot function.
The "content area" is the right panel minus zone info (4 rows) and borders.

### Board-based Games (need visual grid)

| Minigame | Board Size | Min Content WxH | Can Simplify? |
|----------|-----------|-----------------|--------------|
| Chess | 8x8 (5-char cells) | 44 x 21 | No - board is essential |
| Go | 9x9 (3-char cells) | 27 x 13 | No - board is essential |
| Gomoku | 15x15 (2-char cells) | 31 x 17 | Partial - could use scrolling viewport |
| Morris | 24 positions (fixed layout) | 27 x 15 | No - spatial layout is core mechanic |
| Minesweeper (Novice) | 9x9 (2-char cells) | 22 x 12 | No - grid is essential |
| Minesweeper (Master) | 20x16 (2-char cells) | 44 x 19 | Difficulty-dependent |
| Flappy Bird | 50x18 viewport | 54 x 21 | Could scale viewport |
| Snake | 26x26 (2-col cells) | 56 x 15 | Could scale grid |

### Non-visual Games (text-based interaction)

| Minigame | Min Content WxH | Can Simplify? |
|----------|-----------------|--------------|
| Rune Deciphering | 24 x 8 | Yes - already compact |

### Minigame Layout Overhead

Each minigame using `create_game_layout()` adds:
- Outer border: 2 cols + 2 rows
- Info panel: 22-24 cols on right side
- Status bar: 2 rows at bottom

So total overhead is ~26 cols wide, ~4 rows tall beyond the content area.

### Minimum Terminal Sizes for Minigames

| Minigame | Min Terminal Width | Min Terminal Height | Notes |
|----------|-------------------|--------------------|----|
| Chess | 120 (50 stats + 70 game) | 35 | Needs full board + info panel |
| Go | 100 (50 stats + 50 game) | 28 | 9x9 board is relatively compact |
| Gomoku | 110 (50 stats + 60 game) | 32 | 15x15 board is large |
| Morris | 100 (50 stats + 50 game) | 30 | Fixed position layout |
| Minesweeper | 96-120 (varies by diff) | 25-32 | Scales with difficulty |
| Rune | 96 (50 stats + 46 game) | 22 | Most compact minigame |
| Flappy Bird | 130 (50 stats + 80 game) | 35 | Wide viewport |
| Snake | 130 (50 stats + 80 game) | 35 | Large grid |

---

## 4. Size-Dependent Behavior Recommendations

### When terminal is too small for active minigame

If a minigame is active but the terminal is too small to render it:

1. **Show a "resize needed" message** with the minimum dimensions required
2. **Do NOT auto-forfeit** - the game should pause gracefully
3. **Keep game state** - resizing up should resume display immediately
4. **Show compact status** in place of the game board:
   ```
   Chess in progress (your move)
   Terminal too small: need 120x35, have 80x24
   [Esc] Forfeit
   ```

### Adaptive stats panel (left side)

| Terminal Height | Stats Panel Shows |
|----------------|------------------|
| < 15 rows | Name + level + XP bar only |
| 15-24 rows | + Prestige rank, + Zone info (moved to stats) |
| 25-34 rows | + Condensed attributes (2-col), + Condensed equipment |
| 35-44 rows | + Derived stats |
| 45+ rows | Full current layout |

| Terminal Width | Panel Split |
|---------------|-------------|
| < 60 cols | Single column, stacked layout |
| 60-79 cols | 40/60 split (narrower stats) |
| 80-99 cols | 50/50 split (current) |
| 100-119 cols | 50/50 split |
| 120+ cols | 50/50 or 40/60 (wider game area) |

### Adaptive info panel (bottom)

| Terminal Height | Info Panel |
|----------------|-----------|
| < 20 rows | Hidden entirely |
| 20-29 rows | 3 rows: single-line loot + combat log side by side |
| 30-39 rows | 5 rows: compact loot + combat |
| 40+ rows | 8 rows: full current layout |

### Adaptive footer

| Terminal Width | Footer Shows |
|---------------|-------------|
| < 40 cols | [Esc] [P] only |
| 40-59 cols | + [Tab] Challenges |
| 60-79 cols | + [H] Haven, [A] Achievements |
| 80+ cols | Full current footer |

---

## 5. Condensed Display Variants

### Condensed Attributes (for medium terminals)

Current: 14 rows (6 attrs x 2 rows + borders)

Condensed 2-column (6 rows):
```
Attributes
STR: 18 (+4)  INT: 12 (+1)
DEX: 15 (+2)  WIS: 14 (+2)
CON: 16 (+3)  CHA: 11 (+0)
```

### Condensed Equipment (for medium terminals)

Current: 16+ rows (1-3 lines per item)

Condensed (9 rows):
```
Equipment
Weapon: Iron Sword [Rare]
 Armor: Steel Plate [Epic]
Helmet: Leather Cap [Common]
Gloves: [Empty]
 Boots: Swift Sandals [Magic]
Amulet: [Empty]
  Ring: Emerald Band [Rare]
```

### Condensed Derived Stats (for medium terminals)

Current: 6 rows

Inline (2 rows):
```
HP:150 Phys:24 Mag:18 Def:12
Crit:8% XP:1.50x DPS:42
```

### Condensed Combat Scene (for small terminals)

Current: Player HP + 3D sprite + Enemy HP + Status (8+ rows)

Compact (4 rows):
```
You: ████████░░ 80/100
Foe: ██░░░░░░░ 25/90 (Goblin)
In Combat | You: 0.8s  Foe: 1.2s
DPS: 42 | Boss in 3 kills
```

### Condensed Zone Info (for very small terminals)

Current: 4 rows with borders

Inline (1 row):
```
Zone 3: Mountain Pass (2/3) [Boss in 3 kills]
```

---

## 6. Minigame Simplification Analysis

### Games that could have simplified/text-only versions

**Rune Deciphering**: Already nearly text-only. Could render in as little as
24x8 characters. Minimal adaptation needed.

**Minesweeper**: The grid is essential but could use single-character cells
instead of 2-char cells at small sizes, halving width requirements.

**Chess**: Could theoretically use a compact notation view showing the position
as FEN or a tiny 16x8 board (2-char cells), but this degrades the experience
significantly. Better to require minimum size.

### Games that cannot work small

**Go**: The 9x9 board with stone placement and territory visualization needs
spatial rendering. A text-only version would be unplayable.

**Gomoku**: 15x15 board is inherently large. Even with 1-char cells it needs
15x15 minimum. Could add viewport scrolling but pattern recognition across
the full board is core gameplay.

**Morris**: The board positions have specific spatial relationships (lines forming
mills). A non-graphical version would require the player to memorize positions
by number, making it unplayable in practice.

**Flappy Bird**: Real-time visual game. Cannot function without visual rendering.
Could potentially scale the viewport proportionally.

**Snake**: Real-time visual game. Could scale the grid down but needs a minimum
visual area. A 13x13 grid (half size) could work as a "compact mode."

### Recommendation

For terminals too small for a minigame:
- Do not offer simplified versions (the gameplay suffers too much)
- Instead, show a clear minimum size requirement before starting
- Allow the challenge to remain pending until the player resizes
- The menu itself (`challenge_menu_scene`) can render at any size since
  it is just a list

---

## 7. Idle-Specific Design Considerations

### The idle game paradox

This game auto-plays. At very small terminal sizes, the player's primary need is
passive monitoring: "Is my character progressing?" This means:

1. **XP bar is king** - the single most important visual element
2. **Level-up notifications** matter more than combat details
3. **Item drops** should be visible (even as single-line notifications)
4. **Zone advancement** is the big milestone indicator

### What players do at each engagement level

| Engagement | What they need to see | Terminal size |
|-----------|----------------------|--------------|
| Glance (1 sec) | Level, XP%, zone | Any size |
| Check-in (10 sec) | + HP, combat status, recent drops | Small |
| Active monitoring (1 min) | + Equipment, attributes, full logs | Medium |
| Active play (minigame) | Full minigame board + game info | Large |
| Deep review (haven/achievements) | Full overlay screens | Large |

### Auto-save indicator

At all sizes, if an autosave just occurred, a brief flash or indicator should
be visible so the player knows their progress is preserved. This is especially
important at small sizes where the player may be running the game in a small
tmux pane.

---

## 8. Summary: Size Tier Definitions

| Tier | Min Size | Layout Strategy |
|------|----------|----------------|
| Micro | 40x12 | Single column: name+XP, zone, HP bars, footer |
| Small | 60x20 | Single column: + prestige, compact combat, compact logs |
| Medium | 80x30 | Two columns: condensed stats + combat scene, compact logs |
| Large | 100x40 | Two columns: full stats + full combat, full logs (current layout) |
| XLarge | 120x45+ | Current layout with extra space for minigames |

The responsive system should detect terminal size on each frame and select
the appropriate tier, gracefully degrading or upgrading as the terminal
is resized.
