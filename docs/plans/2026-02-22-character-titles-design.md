# Character Titles Design

## Overview

Characters can earn titles through achievements and display them as suffixes to their name. Titles are account-wide (stored in the Achievements system) and selected via a dedicated Titles overlay accessible from the achievement browser.

## Display Format

Title appears as a comma-separated suffix in the stats panel header:

```
┌─ Evaa, Eternal ⭐ ──────────────────────┐
│ Level 45   Prestige 12                   │
```

The existing badge icon system (⭐ etc.) remains independent — title is the text suffix, badge is the icon.

### Where titles appear

- **Stats panel header** (XL/L tier): `" Evaa, Eternal ⭐ "`
- **Compact stats bar** (M tier): `" Evaa, Eternal "`
- **Character select screen**: details panel and character list entries

### Where titles do NOT appear

- Combat scene (uses kill-count badges, not the name)
- Save file names (still `{character_name}.json`)

## Data Model

### Title Definitions

A static mapping from `AchievementId` to title display text. Only curated achievements grant titles — 29 total.

**Combat (6):**

| Achievement | Title | Unlock |
|---|---|---|
| SlayerV | Slayer | 10K kills |
| SlayerX | Destroyer | 10M kills |
| SlayerXV | Annihilator | 1B kills |
| BossHunterV | Boss Hunter | 1K bosses |
| BossHunterX | Bane of Bosses | 1M bosses |
| BossHunterXV | Godslayer | 10M bosses |

**Level & Prestige (8):**

| Achievement | Title | Unlock |
|---|---|---|
| Level250 | Legendary | Level 250 |
| Level500 | Mythic | Level 500 |
| Level1000 | Immortal | Level 1000 |
| Level1500 | Transcendent | Level 1500 |
| PrestigeXXV | Diamond | P25 |
| PrestigeL | Emerald | P50 |
| PrestigeLXX | Obsidian | P70 |
| Eternal | Eternal | P100 |

**Challenges (11):**

| Achievement | Title | Unlock |
|---|---|---|
| GrandChampion | Grand Champion | 100 challenges won |
| ChessMaster | Grandmaster | Chess at Master |
| GoMaster | Sovereign | Go at Master |
| MorrisMaster | Millwright | Morris at Master |
| GomokuMaster | Five-Stone Sage | Gomoku at Master |
| MinesweeperMaster | Trapbreaker | Minesweeper at Master |
| RuneMaster | Runeweaver | Rune at Master |
| FlappyMaster | Skypiercer | Flappy at Master |
| SnakeMaster | Serpent Lord | Snake at Master |
| ContainmentBreachMaster | Warden | JezzBall at Master |
| SigilSurgeMaster | Sigil Savant | Runic Shift at Master |

**Exploration (4):**

| Achievement | Title | Unlock |
|---|---|---|
| StormLeviathan | Leviathan Slayer | Catch Storm Leviathan |
| FishermanIV | Master Angler | Fishing rank 40 |
| HavenArchitect | Architect | Build all Haven rooms |
| SoulforgeX | Soulforged | +10 enhancement |

### Storage

- `selected_title: Option<AchievementId>` added to the `Achievements` struct
- Persisted in `~/.quest/achievements.json` with `#[serde(default)]`
- On load, if the selected achievement is not unlocked, silently clear the title

## Title Selection UI

### Access

Press `[T]` while in the achievement browser to open the Titles overlay.

### Layout

```
┌─ Titles ─────────────────────────────────────┐
│                                              │
│  > Eternal                   ✦ active        │
│    Grand Champion                            │
│    Slayer                                    │
│    Stormbreaker                              │
│    Dungeon Master                            │
│    Leviathan Slayer                          │
│                                              │
│  ┌─ Preview ───────────────────────────────┐ │
│  │  Evaa, Eternal                          │ │
│  └─────────────────────────────────────────┘ │
│                                              │
│  [Enter] Select  [Backspace] Clear  [Esc] Back│
└──────────────────────────────────────────────┘
```

### Behavior

- Lists only titles from **unlocked** achievements
- Sorted by a fixed display order (level/prestige → combat → challenges → exploration)
- `[Up/Down]` navigates the list
- Preview at bottom updates live: `{character_name}, {hovered_title}`
- `[Enter]` selects the highlighted title, returns to achievement browser
- `[Backspace]` clears the active title (no title displayed)
- `[Esc]` returns to achievement browser without changes
- Currently active title shows `✦ active` marker

### UI State

```rust
pub struct TitleBrowserState {
    pub showing: bool,
    pub selected_index: usize,
}
```

## Achievement Unlock Modal

No changes to the achievement unlock modal. Players discover new titles by browsing the title picker.

## Help Text Update

The achievement browser help bar updates to include the `[T]` key:

```
[</>] Category  [Up/Down] Select  [T] Titles  [Esc] Close
```
