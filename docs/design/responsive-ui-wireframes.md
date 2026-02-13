# Responsive UI Wireframes

Detailed ASCII wireframes for each terminal size tier, showing exactly how
the UI adapts. Synthesizes the [UI audit](responsive-ui-audit.md),
[breakpoints design](responsive-ui-breakpoints.md), and
[game information hierarchy](responsive-ui-game-priorities.md).

---

## Tier Overview

| Tier | Width | Height | Strategy |
|------|-------|--------|----------|
| XL | >= 120 | >= 40 | Current layout unchanged |
| L | 80-119 | 30-39 | Condensed stats, compact info panel |
| M | 60-79 | 24-29 | Stacked single-column layout |
| S | 40-59 | 16-23 | Minimal text-only layout |
| Too Small | < 40 | < 16 | "Resize terminal" message |

Width and height tiers are evaluated independently. A 100x22 terminal
would use L-width layout but M-height content density.

---

## 1. XL Layout (>= 120 cols x >= 40 rows)

**No changes from current layout.** This is the reference design.

```
120 cols
+----------------------------------------------------------+
| Challenge Banner (pending challenges notification)        | 1
+----------------------------+-----------------------------+
| Stats Panel (50%)          | Zone Info                    | 4
| +------------------------+ | Zone 3: Mountain Pass (2/3)  |
| | Hero (Lv.42 Warrior)   | | [Boss in 3 kills]           |
| | XP: ████████░░ 73.2%   | | Next: Zone 4 (unlocked)     |
| +------------------------+ +-----------------------------+
| | Prestige               | | Combat Scene                 |
| | Rank: 12 (Gold)        | |                              |
| | Mult: 2.50x + 0.30x   | |      /\                      |
| | Resets: 5              | |     /@@\                     |
| | Fishing: Expert (22)   | |    |@@@@|                    |
| | [fish progress bar]    | |    | o  o |                  |
| +------------------------+ |    | _/\_ |                  |
| | Attributes             | |    \------/                   |
| | STR: 24 (+7) [Cap:35]  | |     ||  ||                   |
| | DEX: 18 (+4) [Cap:35]  | |                              |
| | CON: 21 (+5) [Cap:35]  | | Player HP: ████████░░ 80%    |
| | INT: 15 (+2) [Cap:35]  | | Goblin:    ██░░░░░░ 25/90    |
| | WIS: 12 (+1) [Cap:35]  | | In Combat | You: 0.8s        |
| | CHA: 16 (+3) [Cap:35]  | |   Foe: 1.2s | DPS: 42        |
| +------------------------+ +-----------------------------+
| | Derived Stats          |
| | Max HP: 150            |
| | Physical: 24           |
| | Magic: 18              |
| | Defense: 12             |
| | Crit: 8%               |
| | XP Mult: 1.50x         |
| +------------------------+
| | Equipment              |
| | Weapon: Iron Sword     |
| |   +4 STR +2 DEX       |
| |   +5% DMG +3% Crit    |
| | Armor: Steel Plate     |
| |   +3 CON +2 STR       |
| | (... 5 more slots)     |
| +------------------------+
+----------------------------+-----------------------------+
| Loot Panel (50%)           | Combat Log (50%)            | 8
| [Rare] Darksteel Blade     | You deal 45 damage (CRIT!)  |
|   +5 STR +3 DEX  Equipped! | Goblin deals 12 damage      |
| [Common] Leather Cap       | You deal 23 damage           |
|                             | Goblin deals 15 damage       |
+----------------------------+-----------------------------+
| [Esc] Quit  [P] Prestige  [H] Haven  [A] Achievements   | 3
| [Tab] Challenges (2)  [U] Update (v1.5)                  |
| v2024-01-15 (abc123)                                      |
+----------------------------------------------------------+
```

**Total: ~47 rows used (fits in 40+ with some compression)**

---

## 2. L Layout (80-119 cols x 30-39 rows)

**Changes from XL:**
- Attributes condensed to 2-column format (6 rows -> 3 rows + border)
- Derived stats condensed to 2 lines
- Equipment shows names + rarity only (no attr bonuses/affixes)
- Info panel reduced from 8h to 6h
- Prestige section condensed (remove CHA breakdown, keep effective mult)

```
100 cols
+-------------------------------------------------------+
| [Challenge Banner]                                     | 1
+---------------------------+---------------------------+
| Stats Panel (50%)         | Zone Info                  | 3
| +-----------------------+ | Zone 3: Mountain (2/3)     |
| | Hero Lv.42 | 2h 15m   | | [Boss in 3 kills]          |
| | XP: ████████░░ 73.2%  | +---------------------------+
| +-----------------------+ | Combat Scene               |
| | P:12 Gold | 2.80x XP  | |                            |
| | Fish: Expert (22)      | |      /\                    |
| | [fish progress bar]    | |     /@@\                   |
| +-----------------------+ |    |@@@@|                   |
| | Attributes             | |    | o  o |                |
| | STR:24(+7) INT:15(+2) | |    | _/\_ |                |
| | DEX:18(+4) WIS:12(+1) | |    \------/                |
| | CON:21(+5) CHA:16(+3) | |     ||  ||                 |
| +-----------------------+ |                            |
| | HP:150 Phys:24 Mag:18 | | Player HP: ████████░░ 80%  |
| | Def:12 Crt:8% XP:1.5x | | Goblin:    ██░░░░░░ 25/90 |
| +-----------------------+ | In Combat | DPS: 42        |
| | Equipment              | +---------------------------+
| | Weapon [Rare] Iron Sw  |
| | Armor [Epic] Steel Pl  |
| | Helmet [Com] Leather   |
| | Gloves [Empty]         |
| | Boots [Mag] Swift San  |
| | Amulet [Empty]         |
| | Ring [Rare] Emerald    |
| +-----------------------+
+---------------------------+---------------------------+
| Loot (50%)                | Combat Log (50%)           | 6
| [Rare] Darksteel Blade    | You deal 45 damage (CRIT!) |
|   Equipped!                | Goblin deals 12 damage     |
| [Common] Leather Cap      | You deal 23 damage          |
+---------------------------+---------------------------+
| [Esc]Quit [P]Prestige [H]Haven [A]Ach [Tab]Chall(2)   | 3
| v2024-01-15 (abc123)                    Up to date     |
+-------------------------------------------------------+
```

**Total: ~33 rows (fits in 30-39)**

---

## 3. M Layout (60-79 cols x 24-29 rows)

**Major changes from L:**
- Single-column stacked layout (no 50/50 horizontal split)
- Stats become a compact header bar (2-3 lines)
- Attributes condensed to single line
- Equipment hidden (accessible via [E] key overlay)
- Derived stats hidden
- Footer condensed to 1 line (no border)
- Info panel reduced to 4 lines, full width
- Combat scene gets full width

```
70 cols
+--------------------------------------------------------------------+
| Hero Lv.42 | P:12 Gold 2.80x | Zone 3: Mountain (2/3)             | 1
+--------------------------------------------------------------------+
| STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16     XP: ████░░ 73%     | 1
+--------------------------------------------------------------------+
|                                                                     |
|                        /\                                           |
|                       /@@\                                          |
|                      |@@@@|                                         |
|                      | o  o |                                       |
|                      | _/\_ |                                       |
|                      \------/                                       |
|                       ||  ||                                        |
|                                                                     |
|              Player HP: ████████████░░░░ 80%                        | Min(10)
|              Goblin:    ██████░░░░░░░░░░ 28%                        |
|                  In Combat | You: 0.8s  Foe: 1.2s | DPS: 42        |
|                         [Boss in 3 kills]                           |
+--------------------------------------------------------------------+
| [Rare] Darksteel Blade equipped!   You deal 45 damage (CRIT!)      | 4
| [Common] Leather Cap               Goblin deals 12 damage          |
|                                     You deal 23 damage              |
+--------------------------------------------------------------------+
| [Esc]Quit [P]Prestige [H]Haven [A]Ach [Tab]Chall [E]Equip          | 1
+--------------------------------------------------------------------+
```

**Total: ~24 rows (fits in 24-29)**

**Note:** At M width, the info panel merges loot and combat log into a
single full-width area with loot on the left and combat on the right,
similar to L but without borders.

---

## 4. S Layout (40-59 cols x 16-23 rows)

**Major changes from M:**
- Ultra-compact: single merged status line at top
- No attributes display
- Combat scene minimal: HP bars only, no sprite
- Loot and combat log merge into single scrolling feed
- Footer is minimal key hints
- No borders around sections

```
50 cols
Hero Lv.42 P:12 Gold  Zone 3: Mountain  1
XP: ████████████████░░░░░░ 73.2%         1
You: ████████████░░░░ 80/100 HP          1
Foe: ██████░░░░░░░░░░ 25/90 Goblin       1
In Combat | You: 0.8s | DPS: 42          1
[Boss in 3 kills]                        1
                                          |
         (empty space for activity)       | Min(4)
                                          |
[Rare] Darksteel Blade equipped!          |
You deal 45 damage (CRIT!)               | 5
Goblin deals 12 damage                    | (merged
You deal 23 damage                        |  feed)
Goblin deals 15 damage                    |
                                          |
Esc:Quit P:Prestige Tab:More              1
```

**Total: ~16 rows (fits in 16-23)**

**Key design decisions for S:**
- No borders or chrome anywhere (every row counts)
- HP bars use full-width gauge for readability
- Activity feed interleaves loot and combat entries chronologically
- Tab opens a quick menu for Haven/Achievements/Equipment
- Zone info merged into top status line

---

## 5. Too Small (< 40 cols or < 16 rows)

```
+--------------------------------------+
|                                       |
|    Terminal too small for Quest       |
|                                       |
|    Current: 35 x 12                   |
|    Minimum: 40 x 16                   |
|                                       |
|    Please resize your terminal.       |
|                                       |
+--------------------------------------+
```

---

## 6. Activity-Specific Wireframes

### 6a. Fishing Scene by Tier

**XL/L:** Current layout (fits in right panel)

**M:** Full-width fishing scene
```
70 cols
+--------------------------------------------------------------------+
| Hero Lv.42 | P:12 Gold | Zone 3: Mountain     Fish: Expert (22)   | 1
+--------------------------------------------------------------------+
| FISHING - Crystal Lake                                              | 1
|                                                                     |
|     ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~                                     |
|       ~~~~~~ O ~~~~~~                                               |
|     ~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~                                       | Min(6)
|              |                                                      |
|                                                                     |
| Caught: 3/8 fish      Waiting for bite...                          | 2
| Rank: Expert (22)     [████████░░] 14/20                           | 2
+--------------------------------------------------------------------+
| [Rare] Starfish +180 XP            Reeling in...                    | 4
| [Common] Trout +65 XP                                               |
+--------------------------------------------------------------------+
| Esc:Quit                                                             | 1
+--------------------------------------------------------------------+
```

**S:** Compact fishing
```
50 cols
Fish: Expert (22) Crystal Lake         1
Caught: 3/8 | Waiting for bite...       1
~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~             |
   ~~~~~~ O ~~~~~~                      | Min(4)
~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~               |
         |                              |
Rank: [████████░░] 14/20               1
[Rare] Starfish +180 XP                4
[Common] Trout +65 XP
Esc:Quit                                1
```

### 6b. Dungeon View by Tier

**XL/L:** Current layout (map + combat in right panel)

**M:** Full-width, map on top, combat below
```
70 cols
+--------------------------------------------------------------------+
| Hero Lv.42 | P:12 | Medium Dungeon | Rooms: 8/25 | [KEY]          | 1
+--------------------------------------------------------------------+
|                                                                     |
|        [Dungeon Map - centered, grid_size * 4 wide]                 |
|        [emoji rooms with corridors]                                 | map_h
|                                                                     |
+--------------------------------------------------------------------+
| You: ████████░░ 80/100    Skeleton: ██░░░░ 25/90                   | 2
| In Combat | DPS: 42                                                 |
+--------------------------------------------------------------------+
| You deal 45 damage           Skeleton deals 12 damage               | 3
+--------------------------------------------------------------------+
| Esc:Quit  Arrow:Move  Enter:Clear room                              | 1
+--------------------------------------------------------------------+
```

**S:** Map hidden, text-only dungeon progress
```
50 cols
Dungeon (Med) Rooms:8/25 [KEY]          1
Skeleton Room | You: 80/100 HP          1
Foe: 25/90 | Combat | DPS: 42          1
                                         |
You deal 45 damage (CRIT!)              | Min(4)
Skeleton deals 12 damage                |
                                         |
Esc:Quit Arrow:Move                      1
```

### 6c. Minigame Scenes by Tier

**XL:** Current layout (stats left, game right with info panel)

**L:** Stats panel narrower or hidden, game gets more space
```
100 cols
+-------------------------------------------------------+
| Hero Lv.42 P:12 Gold | Chess (Apprentice)             | 1
+-------------------------------------------------------+
| Moves: 4.Ba4 Nf6  3.Bb5 a6  2.Nf3 Nc6  1.e4 e5      | 1
|                                                         |
|   +----+----+----+----+----+----+----+----+             |
| 8 | R  | N  | B  | Q  | K  | B  | N  | R  |  Info     |
|   +----+----+----+----+----+----+----+----+  RULES     |
| 7 | P  | P  | P  | P  | P  | P  | P  | P  |  Check    |
|   +----+----+----+----+----+----+----+----+  mate the  | 18
| 6 |    |    |    |    |    |    |    |    |  enemy      |
|   +----+----+----+----+----+----+----+----+  king.     |
| 5 |    |    |    |    |    |    |    |    |             |
|   +----+----+----+----+----+----+----+----+  You: KQRB |
| 4 |    |    |    |    |    |    |    |    |  Foe: KQRB  |
|   +----+----+----+----+----+----+----+----+             |
| 3 |    |    |    |    |    |    |    |    |             |
|   +----+----+----+----+----+----+----+----+             |
| 2 | p  | p  | p  | p  | p  | p  | p  | p  |           |
|   +----+----+----+----+----+----+----+----+             |
| 1 | r  | n  | b  | q  | k  | b  | n  | r  |           |
|   +----+----+----+----+----+----+----+----+             |
|     A    B    C    D    E    F    G    H                 |
|                                                         |
| Your move                                               | 2
| [Arrows] Move  [Enter] Select  [Esc] Forfeit            |
+-------------------------------------------------------+
```

**M:** Stats panel hidden during minigame, game gets full width
```
70 cols
+--------------------------------------------------------------------+
| Chess (Apprentice) | Hero Lv.42                                     | 1
+--------------------------------------------------------------------+
| [Chess board at full width, info panel below or hidden]             |
|                                                                     |
| (board renders with available space, info panel collapsed to        |
|  a single status line below the board if height is tight)           |
|                                                                     |
+--------------------------------------------------------------------+
| Your move | [Arrows] Move [Enter] Select [Esc] Forfeit              | 1
+--------------------------------------------------------------------+
```

**S:** Terminal too small message for board games
```
50 cols
+------------------------------------------------+
|                                                 |
|  Chess in progress (your move)                  |
|                                                 |
|  Terminal too small to display board.            |
|  Need: 65 x 24   Have: 50 x 18                 |
|                                                 |
|  Please resize your terminal.                   |
|                                                 |
|  [Esc] Forfeit                                  |
+------------------------------------------------+
```

**Exception:** Rune Deciphering works at M and possibly S:
```
50 cols (S tier, rune game)
Rune Deciphering (Apprentice)           1
 1: A B C D   * o . .                   |
 2: D A B C   . * o .                   |
                                         | 6+
 3: B C _ _                             |
Runes: A B C D E F                      |
                                         |
Deciphering... 4 left                    1
[<>]Move [^v]Cycle [Enter]Go [Esc]Quit  1
```

### 6d. Haven Overlay by Tier

**XL/L:** Current full-screen overlay with skill tree + detail panel

**M:** Simplified list view
```
70 cols
+--------------------------------------------------------------------+
| Haven (8/14 rooms built)                                            |
+--------------------------------------------------------------------+
| Active: +15% DMG  +10% XP  +5% Drops  +3% Crit                    | 2
+--------------------------------------------------------------------+
| > Hearthstone ★★★   +15% DMG                          Cost: --    |
|   Armory ★★·        +10% Drop Rate                    Cost: 3 PR   |
|   TrainingYard ★··   +5% Crit                         Cost: 5 PR   |
|   Watchtower ···     Locked (needs TrainingYard)                    | Min(8)
|   Bedroom ★★★       +20% XP                           Cost: --    |
|   Garden ★··         +3% Discovery                    Cost: 4 PR   |
|   (... scrollable)                                                  |
+--------------------------------------------------------------------+
| [Up/Down] Navigate  [Enter] Build  [Esc] Close                     | 1
+--------------------------------------------------------------------+
```

**S:** Compact haven status only
```
50 cols
Haven (8/14 rooms)                       1
Bonuses: +15%DMG +10%XP +5%Drop +3%Crit 1
                                          |
> Hearthstone ★★★                        |
  Armory ★★·  [3 PR to upgrade]         |
  TrainingYard ★··                       | Min(6)
  Watchtower ··· [Locked]                |
  Bedroom ★★★                           |
  Garden ★··                             |
                                          |
Up/Down:Move Enter:Build Esc:Close        1
```

### 6e. Achievement Browser by Tier

**XL/L:** Current full overlay with tabs + list + detail

**M:** Tabs + list only, detail on Enter
```
70 cols
+--------------------------------------------------------------------+
| Achievements (42.5% Complete)                                       |
+--------------------------------------------------------------------+
| Combat(3/8) Level(4/6) Progress(2/5) Challenge(1/4) Explore(0/3)  | 2
+--------------------------------------------------------------------+
| [X] First Blood - Defeat your first enemy                          |
| [X] Warrior - Defeat 100 enemies                                   |
| [ ] Champion - Defeat 1000 enemies (Progress: 456/1000)           | Min(8)
| [ ] Slayer - Defeat 5000 enemies                                   |
| > [ ] Legend - Defeat 10000 enemies                                 |
|                                                                     |
+--------------------------------------------------------------------+
| [</>] Category  [Up/Down] Select  [Enter] Detail  [Esc] Close     | 1
+--------------------------------------------------------------------+
```

---

## 7. Transition Points

### Width Transitions

| At Width | Change |
|----------|--------|
| 120 -> 119 | Enter L tier: condense attributes, equipment, derived stats |
| 80 -> 79 | Enter M tier: switch to stacked layout, hide equipment |
| 60 -> 59 | Enter S tier: remove all borders, minimal text layout |
| 40 -> 39 | Too small: show resize message |

### Height Transitions

| At Height | Change |
|-----------|--------|
| 40 -> 39 | L tier: reduce info panel 8->6, condense prestige section |
| 30 -> 29 | M tier: compact header bar, hide derived stats |
| 24 -> 23 | Aggressive M: hide attributes line |
| 16 -> 15 | Too small for gameplay |

### Hysteresis

To prevent flickering when the terminal is exactly at a breakpoint,
use a 2-unit hysteresis buffer:

```rust
// Upgrade threshold = breakpoint value
// Downgrade threshold = breakpoint value - 2
// Example: L->XL at 120 cols, XL->L at 118 cols
```

---

## 8. Overlay Adaptation

All modals (prestige confirm, haven discovery, achievement unlock, etc.)
should adapt to available space:

| Modal | XL/L Size | M Size | S Size |
|-------|-----------|--------|--------|
| Prestige Confirm | 50x18 centered | 50x18 or full-width | Full-screen |
| Achievement Unlock | 50x9 centered | 50x9 centered | Full-width, compact |
| Haven Discovery | 50x7 centered | 50x7 centered | Full-width |
| Leviathan Encounter | 64x16 centered | Full-width x 16 | Full-width, truncated |
| Offline Welcome | 44x10 centered | 44x10 centered | Full-width |

**Rule:** If a modal's hardcoded width exceeds 80% of terminal width,
render it as full-width with 1-column padding on each side.

---

## 9. Summary of Line Budgets

| Tier | Total Rows | Chrome/Fixed | Content |
|------|-----------|-------------|---------|
| XL | 40+ | ~14 (banner+info+footer+zone) | 26+ |
| L | 30-39 | ~10 (banner+info+footer) | 20-29 |
| M | 24-29 | ~4 (header+attrs+info+footer) | 20-25 |
| S | 16-23 | ~3 (status+footer) | 13-20 |
