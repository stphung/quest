# Terminal Size Breakpoints and Content Priority Tiers

## Reference: Current Layout Constraints

From the [UI audit](responsive-ui-audit.md), the current layout requires:
- Stats panel: 47+ lines minimum (left 50%)
- Info panel: 8 lines (fixed)
- Footer: 3 lines (fixed)
- Zone info: 4 lines (right panel header)
- Right content: Min(10) for combat scene
- Total minimum: ~47 lines tall, ~80 columns wide

---

## Terminal Size Tiers

### Tier Definitions

| Tier | Columns | Rows | Common Scenarios |
|------|---------|------|-----------------|
| **XL** (current) | >= 120 | >= 40 | Full-screen on large monitors, typical developer terminal |
| **L** (standard) | 80-119 | 30-39 | Standard 80x24 terminal, half-screen windows |
| **M** (compact) | 60-79 | 24-29 | Small terminal window, tmux pane, laptop half-screen |
| **S** (minimal) | 40-59 | 16-23 | Very small terminal, mobile SSH, extreme split |

**Detection:** Check `frame.size()` at the start of `draw_ui_with_update()`. Width and height are independently evaluated; a terminal can be L-width but M-height.

**Note:** Below 40x16, the game should display a "Terminal too small" message.

---

## Content Priority Ranking

Every UI element is assigned a priority level. Higher priority = shown at smaller sizes.

### Priority 1: Essential (always visible, all tiers)

These elements provide the absolute minimum gameplay feedback:

| Element | Purpose | Min Size |
|---------|---------|----------|
| Player HP bar | Survival awareness | 1 line |
| Enemy HP bar / status | Combat state | 1 line |
| XP bar with level | Progression | 1 line |
| Current activity label | What's happening | 1 line |
| Quit hint | How to exit | Part of condensed footer |

### Priority 2: Core Gameplay (S tier and above)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Zone / subzone name | Location | 1 line (condensed) |
| Boss progress | Zone progression | Combined with zone line |
| Loot feed (last 2-3 items) | Idle reward feedback | 3-4 lines |
| Combat log (last 2-3 entries) | Action feedback | 3-4 lines |
| Character name | Identity | Combined with level line |
| Prestige rank | Progression tier | 1 line |
| Key controls | Navigation | 1 line condensed |

### Priority 3: Important Context (M tier and above)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Attributes (condensed, 2 rows of 3) | Build awareness | 2-3 lines |
| Equipment summary (names only) | Gear awareness | 7 lines |
| Prestige multiplier details | XP context | 1-2 lines |
| Fishing rank + bar | Secondary progression | 2 lines |
| Challenge banner | Notification | 1 line |
| DPS display | Combat info | Part of status line |
| Derived stats (condensed) | Combat numbers | 2-3 lines |

### Priority 4: Full Detail (L tier and above)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Attributes with full layout (emojis, bars) | Detailed build | 12 lines |
| Equipment with stats + affixes | Full gear info | 16+ lines |
| Derived stats (full panel) | All combat numbers | 6 lines |
| Extended combat log (5+ entries) | Full battle history | 6+ lines |
| Extended loot feed (5+ items) | Full drop history | 6+ lines |
| Play time | Session tracking | 1 line |
| Attribute caps | Build limits | Part of attribute lines |
| Next zone status line | Forward progression | 1 line |

### Priority 5: Extra (XL tier only)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Full attribute modifiers display | Min-max detail | Part of attr lines |
| Equipment affixes | Item depth | Extra lines per item |
| Update check status | Meta-game | Part of footer |
| Spacious layout | Visual comfort | Padding/margins |

---

## Tier Layouts

### XL Layout (>= 120 cols x >= 40 rows) -- NO CHANGES

This is the current layout exactly as it exists today.

```
+----------------------------------------------+
| [Challenge Banner]                            |
+----------------------+------------------------+
|  Stats Panel (50%)   | Zone Info              |
|  - Header (4h)       +------------------------+
|  - Prestige (7h)     | Combat/Activity        |
|  - Attributes (14h)  |                        |
|  - Derived (6h)      |                        |
|  - Equipment (16h+)  |                        |
+----------------------+------------------------+
| Loot (50%)           | Combat Log (50%)       | 8h
+----------------------+------------------------+
| Footer (3h)                                   |
+----------------------------------------------+
```

### L Layout (80-119 cols x 30-39 rows)

**Changes from XL:**
- Attributes condensed: 2 rows of 3 (STR/DEX/CON then INT/WIS/CHA) instead of 6 x 2-line rows -> saves 6 lines
- Derived stats condensed: 2 rows (HP/Phys/Magic on one, Def/Crit/XP on another) -> saves 3 lines
- Equipment: names only, no attr bonuses or affixes -> saves ~7 lines
- Info panel: reduce to `Length(6)` -> saves 2 lines
- Net savings: ~18 lines

```
+----------------------------------------------+
| [Challenge Banner]                            |
+----------------------+------------------------+
|  Stats Panel (50%)   | Zone Info (3h)         |
|  - Header (4h)       +------------------------+
|  - Prestige (5h)     | Combat/Activity        |
|  - Attrs compact(4h) |                        |
|  - Derived cmpct(4h) |                        |
|  - Equip names (9h)  |                        |
+----------------------+------------------------+
| Loot (50%)           | Combat Log (50%)       | 6h
+----------------------+------------------------+
| Footer (3h)                                   |
+----------------------------------------------+
```

### M Layout (60-79 cols x 24-29 rows)

**Changes from L:**
- Stats and activity panels **stacked vertically** instead of 50/50 horizontal
- Stats panel becomes a compact horizontal bar at top (2-3 lines)
- Zone info merged into stats bar
- Info panel reduced to 4 lines
- Footer condensed to 1 line (no border)
- Equipment hidden (accessible via overlay key)
- Derived stats hidden (accessible via overlay key)

```
+----------------------------------------------+
| Lv42 Warrior | P:12 Gold | Zone 5: Mountain  | 2h (stats bar)
+----------------------------------------------+
| STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16   | 1h (attrs)
+----------------------------------------------+
|                                               |
|          Combat / Activity Area               | Min(8)
|         (full width, more vertical)           |
|                                               |
+----------------------------------------------+
| Loot (50%)           | Combat Log (50%)       | 4h
+----------------------------------------------+
| [Esc]Quit [P]Prestige [H]Haven [A]Achievem.  | 1h
+----------------------------------------------+
```

### S Layout (40-59 cols x 16-23 rows)

**Changes from M:**
- Stats bar reduced to single essential line
- Attributes hidden
- Info panel shows either loot OR combat log (tabbed), reduced to 3 lines
- Footer reduced to minimal key hints
- Activity area gets maximum space

```
+----------------------------------------------+
| Lv42 Gold P:12 | Zone 5 [Boss in 3]          | 1h
+----------------------------------------------+
| HP [========    ] 156/200                     | 1h
+----------------------------------------------+
|                                               |
|          Combat / Activity Area               | Min(6)
|         (full width, maximum space)           |
|                                               |
+----------------------------------------------+
| Foe HP [======= ] 89/120                     | 1h
+----------------------------------------------+
| [Epic] Darksteel Blade equipped!              | 3h (loot OR log)
| You deal 45 damage (CRIT!)                   |
| Enemy deals 12 damage                        |
+----------------------------------------------+
| Esc:Quit P:Prestige Tab:More                  | 1h
+----------------------------------------------+
```

---

## Breakpoint Decision Matrix

For each component, which tiers show it:

| Component | XL | L | M | S | Notes |
|-----------|:--:|:-:|:-:|:-:|-------|
| Player HP bar | Y | Y | Y | Y | Always |
| Enemy HP bar | Y | Y | Y | Y | Always |
| XP bar + level | Y | Y | Y | Y | Condensed at S |
| Zone name + boss | Y | Y | Y | Y | Condensed at S |
| Character name | Y | Y | Y | N | Merged into stats bar at M |
| Prestige rank | Y | Y | Y | Y | Condensed |
| Prestige multiplier detail | Y | Y | N | N | |
| Attributes (full 6x2) | Y | N | N | N | |
| Attributes (compact 2x3) | N | Y | N | N | |
| Attributes (single line) | N | N | Y | N | |
| Derived stats (full) | Y | N | N | N | |
| Derived stats (compact) | N | Y | N | N | |
| Equipment (full) | Y | N | N | N | |
| Equipment (names only) | N | Y | N | N | |
| Fishing rank + bar | Y | Y | N | N | Overlay access at M/S |
| Combat log (full) | Y | Y | Y | Y* | *Merged with loot at S |
| Loot panel (full) | Y | Y | Y | Y* | *Merged with combat at S |
| Info panel border/chrome | Y | Y | Y | N | |
| Footer (full 3h) | Y | Y | N | N | |
| Footer (compact 1h) | N | N | Y | Y | |
| Challenge banner | Y | Y | Y | N | |
| Play time | Y | Y | N | N | |
| DPS display | Y | Y | Y | N | |
| Update status | Y | Y | N | N | |
| Save indicator | Y | Y | N | N | |
| Spacious padding | Y | N | N | N | |

---

## Minigame Breakpoints

Minigames present a special challenge because their boards have fixed pixel dimensions.

| Minigame | Min Width | Min Height | Strategy at Small Sizes |
|----------|-----------|-----------|------------------------|
| Chess | 43 + 22 = 65w | 21h | Board cannot shrink; hide info panel below M |
| Go | 25 + 24 = 49w | 13h | Board fits at M; info panel collapses |
| Morris | 25 + 24 = 49w | 15h | Board fits at M; info panel collapses |
| Gomoku | 29 + 22 = 51w | 17h | Board fits at M; info panel collapses |
| Minesweeper | 18-60 + 24w | 11-18h | Varies by difficulty; some fit at M |
| Rune | Variable + 22w | 8h+ | Fits at most sizes |
| Flappy Bird | Scalable | Scalable | Already scales; works at M |
| Snake | Scalable | Scalable | Already scales; works at M |

**Strategy:** At M and below, minigame info panels move below the board or are hidden, giving the board full width. At S, chess is the only minigame that truly cannot fit and should display a "Terminal too small for this challenge" message.

---

## Implementation Constants

Proposed constants for `src/core/constants.rs` or a new `src/ui/responsive.rs`:

```rust
// Terminal size tier thresholds
pub const TIER_XL_MIN_COLS: u16 = 120;
pub const TIER_XL_MIN_ROWS: u16 = 40;
pub const TIER_L_MIN_COLS: u16 = 80;
pub const TIER_L_MIN_ROWS: u16 = 30;
pub const TIER_M_MIN_COLS: u16 = 60;
pub const TIER_M_MIN_ROWS: u16 = 24;
pub const TIER_S_MIN_COLS: u16 = 40;
pub const TIER_S_MIN_ROWS: u16 = 16;

// Absolute minimum (show "Terminal too small" below this)
pub const MIN_USABLE_COLS: u16 = 40;
pub const MIN_USABLE_ROWS: u16 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TerminalTier {
    TooSmall,
    S,      // 40x16+
    M,      // 60x24+
    L,      // 80x30+
    XL,     // 120x40+
}

impl TerminalTier {
    pub fn from_size(cols: u16, rows: u16) -> Self {
        if cols < MIN_USABLE_COLS || rows < MIN_USABLE_ROWS {
            TerminalTier::TooSmall
        } else if cols >= TIER_XL_MIN_COLS && rows >= TIER_XL_MIN_ROWS {
            TerminalTier::XL
        } else if cols >= TIER_L_MIN_COLS && rows >= TIER_L_MIN_ROWS {
            TerminalTier::L
        } else if cols >= TIER_M_MIN_COLS && rows >= TIER_M_MIN_ROWS {
            TerminalTier::M
        } else {
            TerminalTier::S
        }
    }
}
```

---

## Migration Strategy

The breakpoint system should be implemented incrementally:

1. **Phase 1:** Add `TerminalTier` detection, pass tier to all draw functions. No behavior changes.
2. **Phase 2:** Implement L tier (condensed stats) -- minimal visual change, most compatible.
3. **Phase 3:** Implement M tier (stacked layout) -- significant layout change.
4. **Phase 4:** Implement S tier (minimal layout) -- maximum information density.

Each phase should be a separate PR to keep changes reviewable.
