> Backported design record. Sources: docs/design/debug-menu-shortening-mockups.md.

## debug-menu-shortening-mockups.md

# Debug Menu Shortening Mockups

Date: 2026-02-22
Current menu: 22 flat options in one scroll list (`src/utils/debug_menu.rs`).

## Goals

- Reduce visible length at first glance
- Keep all existing debug actions available
- Preserve keyboard-first flow (Up/Down/Enter/Esc)

## Option A: Tabbed Categories

### Idea
Split actions into top tabs. Only show one category list at a time.

### Wireframe

```text
+------------------------------------------------------------------+
| Debug Menu                                                       |
| [Challenges] [World] [Resources] [Items]                        |
+------------------------------------------------------------------+
| > Trigger Chess Challenge                                        |
|   Trigger Morris Challenge                                       |
|   Trigger Gomoku Challenge                                       |
|   Trigger Minesweeper Challenge                                  |
|   Trigger Rune Challenge                                         |
|   Trigger Go Challenge                                           |
|   Trigger Flappy Bird Challenge                                  |
|   Trigger JezzBall Challenge                                     |
|   Trigger Snake Challenge                                        |
|   Trigger Sigil Surge Challenge                                  |
+------------------------------------------------------------------+
| [Tab/Shift+Tab] Category  [Up/Down] Navigate  [Enter] Trigger   |
+------------------------------------------------------------------+
```

### Suggested grouping

- `Challenges` (10)
- `World` (Dungeon, Fishing, Haven, Soulforge)
- `Resources` (stormglass + sigils)
- `Items` (forge 3 god items)

### Tradeoffs

- Pros: clear mental model, shortest visible list per screen
- Cons: one extra key action when switching categories

## Option B: Two-Stage Drilldown

### Idea
Stage 1 shows only categories. Enter opens stage 2 action list.

### Wireframe (stage 1)

```text
+------------------------------------------------------+
| Debug Menu                                           |
+------------------------------------------------------+
| > Challenges (10)                                    |
|   World Events (4)                                   |
|   Resources (5)                                      |
|   God Items (3)                                      |
+------------------------------------------------------+
| [Enter] Open  [Esc] Close                            |
+------------------------------------------------------+
```

### Wireframe (stage 2)

```text
+------------------------------------------------------+
| Debug Menu > Resources                               |
+------------------------------------------------------+
| > Grant 1000 Stormglass                              |
|   Discover Stormglass                                |
|   Grant 100k Stormglass                              |
|   Etch Random Sigils (All Slots)                    |
|   Etch S+ Sigil (Slot 1)                             |
+------------------------------------------------------+
| [Esc] Back  [Up/Down] Navigate  [Enter] Trigger      |
+------------------------------------------------------+
```

### Tradeoffs

- Pros: shortest menu on open; very scalable as options grow
- Cons: two steps to trigger any action

## Option C: Quick Picks + Full List

### Idea
Top section for frequent actions, plus compact "More actions..." entry.

### Wireframe

```text
+------------------------------------------------------------------+
| Debug Menu                                                       |
+------------------------------------------------------------------+
| Quick Picks                                                      |
| > Trigger Dungeon                                                |
|   Trigger Fishing                                                |
|   Trigger Chess Challenge                                        |
|   Trigger Haven Discovery                                        |
|   Grant 1000 Stormglass                                          |
|                                                                  |
| More                                                             |
|   Browse All Actions...                                          |
+------------------------------------------------------------------+
| [Enter] Trigger/Open  [/] Search  [Esc] Close                    |
+------------------------------------------------------------------+
```

### Wireframe ("Browse All Actions")

```text
+------------------------------------------------------------------+
| All Debug Actions                                                |
| Filter: "storm"                                                  |
+------------------------------------------------------------------+
| > Grant 1000 Stormglass                                          |
|   Discover Stormglass                                            |
|   Grant 100k Stormglass                                          |
+------------------------------------------------------------------+
| [Type] Filter  [Up/Down] Navigate  [Esc] Back                    |
+------------------------------------------------------------------+
```

### Tradeoffs

- Pros: fastest for common actions; still supports long tail actions
- Cons: requires deciding "quick picks" defaults; filter adds complexity

## Recommendation

If priority is "shorter immediately with low implementation risk": pick **Option A**.

If priority is "scales best as debug actions keep growing": pick **Option B**.

If priority is "fast for daily testing workflow": pick **Option C**.
