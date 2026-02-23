# The Deep — UI Design

## Overview

The Deep overlay follows the same modal architecture as Haven, Soulforge, and Stormglass: a full-screen Clear + border block with an animated backdrop, sub-views navigated by keybind, and a footer help bar. The overlay opens over the combat scene and closes with Esc.

File structure mirrors Haven:
- `src/ui/deep_scene.rs` — main overlay coordinator
- `src/ui/deep_missions.rs` — active missions panel
- `src/ui/deep_roster.rs` — mercenary roster sub-view
- `src/ui/deep_layers.rs` — layer/infrastructure sub-view
- `src/ui/deep_event.rs` — event response sub-view
- `src/ui/deep_results.rs` — mission complete modal

---

## Color Conventions

Follows Quest's existing `rarity_color()` pattern in `src/ui/mod.rs`.

### Layer Tier Colors

| Tier | Layers | Color | RGB |
|------|--------|-------|-----|
| The Shallows | 1-3 | `Color::Green` | — |
| The Warrens | 4-7 | `Color::Yellow` | — |
| The Hollows | 8-12 | `Color::Magenta` | — |
| The Sunken Reach | 13-18 | `Color::Cyan` | — |
| The Abyss | 19-25 | `Color::LightRed` | — |
| The Void | 26+ | `Color::Rgb(255, 215, 0)` | Gold |

### Mission Type Colors

| Type | Color | Rationale |
|------|-------|-----------|
| Supply Run | `Color::Green` | Safe, reliable |
| Recon | `Color::Cyan` | Information-gathering |
| Expedition | `Color::Yellow` | Core progression |
| Breakthrough | `Color::LightRed` | High stakes |
| Construction | `Color::Blue` | Infrastructure |

### Merc Archetype Colors

| Archetype | Color | Rationale |
|-----------|-------|-----------|
| Vanguard | `Color::LightRed` | Frontline aggression |
| Scout | `Color::Cyan` | Mobility and awareness |
| Arcanist | `Color::Magenta` | Magic/elemental |
| Medic | `Color::Green` | Healing |
| Saboteur | `Color::Yellow` | Trickery |

### Event Urgency Colors

| State | Color |
|-------|-------|
| No pending events | `Color::DarkGray` |
| Event waiting (auto-resolve soon) | `Color::Yellow` |
| Event auto-resolving in <5 min | `Color::LightRed` |
| Mission complete, rewards pending | `Color::Green` |

### Backdrop

Deep blue-black gradient, darker than Stormglass. Top `(5, 8, 20)`, bottom `(2, 3, 8)`. Drifting particles in pale blue-white `(60, 80, 140)` simulate dust motes in cave air. Themed cyan border: `Color::Rgb(80, 160, 220)`.

---

## Backdrop Theme

```rust
// paint_deep_backdrop parameters
top_rgb:    (5, 8, 20)     // near-black deep blue
bottom_rgb: (2, 3, 8)      // void black
particle_count: 10
particle_chars: ['·', '•', '∘']
particle_color_hot: (60, 80, 140)   // cave dust
particle_color_cool: (20, 30, 60)
```

Opening flourish (600ms): brief blue-white sheen sweeping top-to-bottom on overlay open, simulating descent into The Deep.

---

## View 1: Main Overlay

The default view on open. Shows guild status, active missions, and navigation footer.

### Layout (L/XL — 80x30+)

```
┌─ THE DEEP ──────────────────────────────────────────────────────────┐
│                                                                     │
│  Guild: Sellswords (Rank 2)          Warband Marks: 1,240          │
│  Deepest Layer: 8 [The Hollows]      Mercs: 6/7                    │
│                                                                     │
│ ┌─ ACTIVE MISSIONS ─────────────────────────────────────────────┐  │
│ │ ► [Expedition]  Layer 8   12h elapsed / 16h                   │  │
│ │   [████████████████████████░░░░░░░] 78%                       │  │
│ │   Squad: Aldric (Vanguard), Sera (Scout), Thorne (Arcanist)   │  │
│ │   ⚡ Event pending! Press [Enter] to respond.                 │  │
│ │                                                               │  │
│ │   [Supply Run]  Layer 4    3h elapsed / 3h                    │  │
│ │   [████████████████████████████████] Done!                    │  │
│ │   Squad: Mira (Medic)                                         │  │
│ │   ✓ Complete — [Enter] to collect rewards.                    │  │
│ └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  No other missions running. [N] New Mission to deploy a squad.     │
│                                                                     │
│  [N] New Mission  [R] Roster  [L] Layers  [Esc] Close              │
└─────────────────────────────────────────────────────────────────────┘
```

**Cursor**: `►` marks the focused mission row. Up/Down navigate between missions.
**Enter on event**: opens Event Response sub-view.
**Enter on Done**: opens Mission Complete modal.

### Layout (M — 60x24)

```
┌─ THE DEEP ────────────────────────────────────────────┐
│ Sellswords Rank 2   Marks: 1,240   Mercs: 6/7        │
│ Layer 8 [The Hollows]                                 │
├───────────────────────────────────────────────────────┤
│ ► [Expedition] L8  ████████████░░░ 78%  ⚡ Event!   │
│   Aldric, Sera, Thorne                                │
│                                                       │
│   [Supply Run] L4  ████████████████ Done!            │
│   Mira                                                │
├───────────────────────────────────────────────────────┤
│ [N] New  [R] Roster  [L] Layers  [Esc] Close         │
└───────────────────────────────────────────────────────┘
```

### Layout (S — 40x16)

```
┌─ THE DEEP ──────────────────────────┐
│ Rank 2  Marks: 1,240  Mercs: 6/7   │
│                                     │
│ ► Expedition L8  78%  ⚡ Event!    │
│   Supply Run L4  Done!              │
│                                     │
│ [N]New [R]Roster [L]Layers [Esc]   │
└─────────────────────────────────────┘
```

**TooSmall (<40x16)**: Show "Terminal too small" per `render_too_small()` pattern.

---

## View 2: New Mission

Shown when player presses [N]. Left panel lists available missions; right panel shows details and squad assignment for the selected one.

### Layout (L/XL)

```
┌─ THE DEEP — New Mission ────────────────────────────────────────────┐
│                                                                     │
│  ┌─ AVAILABLE ───────────────────┐  ┌─ MISSION DETAIL ───────────┐ │
│  │ ► [Expedition]  Layer 8  12h │  │ Layer 8 — The Hollows      │ │
│  │   [Recon]       Layer 9   6h │  │                            │ │
│  │   [Supply Run]  Layer 4   2h │  │ Duration: 12-16h           │ │
│  │   [Construction] Layer 6  4h │  │ Risk:     Medium           │ │
│  │   [Recon]       Layer 8   5h │  │ Reward:   Marks + items    │ │
│  │                              │  │                            │ │
│  │                              │  │ Requires:                  │ │
│  │                              │  │  Min Power 40              │ │
│  │                              │  │  1+ Vanguard recommended   │ │
│  │                              │  │                            │ │
│  │                              │  │ ─ Assign Squad ──────────  │ │
│  │                              │  │  [✓] Aldric  Vanguard L4   │ │
│  │                              │  │  [✓] Sera    Scout    L3   │ │
│  │                              │  │  [ ] Mira    Medic    L2   │ │
│  │                              │  │        (on mission)         │ │
│  │                              │  │  [ ] Thorne  Arcanist L5   │ │
│  │                              │  │                            │ │
│  │                              │  │ Power: 72  ✓ Requirements  │ │
│  └──────────────────────────────┘  └────────────────────────────┘ │
│                                                                     │
│  [↑/↓] Select Mission  [Tab] Switch Panel  [Enter] Launch  [Esc]   │
└─────────────────────────────────────────────────────────────────────┘
```

**Panel focus**: [Tab] switches focus between mission list (left) and squad picker (right).
**Mission list**: Up/Down selects mission; detail panel updates immediately.
**Squad picker**: Up/Down navigates mercs; [Space] toggles assignment.
**Greyed rows**: Mercs on active missions shown with "(on mission)" label, unselectable.
**Power display**: Updates live as mercs are toggled. Green when requirements met, red when not.
**Launch**: [Enter] launches the mission with assigned squad. Requires at least 1 merc.

### Layout (M)

```
┌─ New Mission ─────────────────────────────────────────┐
│ [↑/↓] Mission  [Tab] Switch  [Space] Toggle Merc     │
├──────────────────────────┬────────────────────────────┤
│ ► Expedition  L8  12h   │ Layer 8 — The Hollows      │
│   Recon       L9   6h   │ Risk: Medium   Marks +items │
│   Supply Run  L4   2h   │                            │
│   Construction L6   4h  │ Assign:                    │
│   Recon       L8   5h   │ [✓] Aldric Vanguard L4     │
│                          │ [✓] Sera   Scout   L3     │
│                          │ [ ] Mira   (on mission)   │
│                          │ Power: 72  ✓ OK           │
├──────────────────────────┴────────────────────────────┤
│             [Enter] Launch  [Esc] Back                │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

Single panel, toggle between list and squad views with [Tab]:

```
┌─ New Mission ───────────────────────┐
│ [Tab] List/Squad  [Esc] Back        │
│                                     │
│ ► Expedition  L8  12h  Medium      │
│   Recon       L9   6h  Low         │
│   Supply Run  L4   2h  Safe        │
│                                     │
│ Power: 72  ✓  [Enter] Launch       │
└─────────────────────────────────────┘
```

---

## View 3: Roster

Shown when player presses [R]. Merc list with stats, archetype, level, status.

### Layout (L/XL)

```
┌─ THE DEEP — Roster ─────────────────────────────────────────────────┐
│                                                                     │
│  Mercs: 6/7          Guild Rank: 2 (Sellswords)                    │
│                                                                     │
│  ┌─ MERCENARIES ─────────────────────────────────────────────────┐ │
│  │  Name          Archetype   Lvl   Power  Resilience  Status    │ │
│  │ ─────────────────────────────────────────────────────────────│ │
│  │ ► Aldric        Vanguard    4     52     High        On L8    │ │
│  │   Sera          Scout       3     38     Med         On L8    │ │
│  │   Thorne        Arcanist    5     61     Low         On L8    │ │
│  │   Mira          Medic       2     24     High        On L4    │ │
│  │   Brennan       Saboteur    1     18     Med         Ready    │ │
│  │   Lys           Vanguard    2     27     High        Ready    │ │
│  │                                                               │ │
│  │   [Recruit slot open — 240 Marks]                            │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─ DETAIL: Aldric ──────────────────────────────────────────────┐ │
│  │  Archetype: Vanguard   Level: 4   Power: 52   Resilience: 72  │ │
│  │  Missions completed: 8   Bonus: Reduces squad casualties       │ │
│  │  Current: Expedition Layer 8 — 12h elapsed / 16h (78%)        │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  [↑/↓] Navigate  [Enter] Recruit (on empty slot)  [Esc] Back      │
└─────────────────────────────────────────────────────────────────────┘
```

**Status column colors**: "On L8" in `Color::DarkGray`, "Ready" in `Color::Green`, "Injured" in `Color::Yellow`, "Lost" in `Color::Red`.
**Archetype column**: Colored per archetype conventions above.
**Detail panel**: Updates live as cursor moves. Shows merc bio, stats, current assignment.
**Recruit slot**: Available if roster not full. Shows cost. [Enter] on empty slot opens recruit confirm modal.
**Injured mercs**: Shown with injury timer: "Injured (2 missions)".

### Layout (M)

```
┌─ Roster ──────────────────────────────────────────────┐
│ Mercs: 6/7   Guild Rank 2                            │
├───────────────────────────────────────────────────────┤
│ ► Aldric      Vanguard L4  P:52  On L8              │
│   Sera        Scout    L3  P:38  On L8              │
│   Thorne      Arcanist L5  P:61  On L8              │
│   Mira        Medic    L2  P:24  On L4              │
│   Brennan     Saboteur L1  P:18  Ready              │
│   Lys         Vanguard L2  P:27  Ready              │
│   [Recruit slot — 240 Marks]                        │
├───────────────────────────────────────────────────────┤
│ Aldric · Vanguard · 8 missions · On L8 78%          │
│ [↑/↓] Navigate  [Enter] Recruit  [Esc] Back         │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

```
┌─ Roster ────────────────────────────────┐
│ Mercs: 6/7                             │
│ ► Aldric   Vanguard L4   On L8        │
│   Sera     Scout    L3   On L8        │
│   Thorne   Arcanist L5   On L8        │
│   Mira     Medic    L2   On L4        │
│   Brennan  Saboteur L1   Ready        │
│   Lys      Vanguard L2   Ready        │
│ [Esc] Back                             │
└─────────────────────────────────────────┘
```

---

## View 4: Layers

Shown when player presses [L]. Layer-by-layer infrastructure status and progression.

### Layout (L/XL)

```
┌─ THE DEEP — Layers ─────────────────────────────────────────────────┐
│                                                                     │
│  Frontier: Layer 8 (The Hollows)     Deepest ever: Layer 8         │
│                                                                     │
│  ┌─ LAYER MAP ──────────────────┐  ┌─ LAYER DETAIL ──────────────┐ │
│  │  L1  The Shallows  [CLEAR]  │  │ Layer 4 — The Warrens       │ │
│  │  L2  The Shallows  [CLEAR]  │  │ Tier: The Warrens           │ │
│  │  L3  The Shallows  [CLEAR]  │  │ Status: Cleared             │ │
│  │  L4  The Warrens   [CLEAR]  │  │ Intel: 100%                 │ │
│  │  L5  The Warrens   [CLEAR]  │  │                             │ │
│  │  L6  The Warrens   [CLEAR]  │  │ Infrastructure:             │ │
│  │  L7  The Warrens   [CLEAR]  │  │  [✓] Outpost      (-25% t)  │ │
│  │ ►L8  The Hollows  [FRNTIR] │  │  [✓] Supply Cache (+yield)  │ │
│  │  L9  The Hollows  [??????] │  │  [ ] Watchtower   (locked)  │ │
│  │  L10 The Hollows  [??????] │  │  [ ] Bridge       (locked)  │ │
│  │  ...                       │  │                             │ │
│  │                             │  │ Available: Construction     │ │
│  │                             │  │  Watchtower — 300 Marks     │ │
│  └─────────────────────────────┘  └─────────────────────────────┘ │
│                                                                     │
│  [↑/↓] Navigate Layers  [Enter] Send Construction Mission  [Esc]   │
└─────────────────────────────────────────────────────────────────────┘
```

**Layer list colors**: Cleared layers use `Color::DarkGray` for number, `Color::Green` for `[CLEAR]`. Frontier layer uses `Color::Yellow` for `[FRNTIR]`. Unknown layers use `Color::DarkGray` for `[??????]`.
**Familiarity/Intel bar**: Rendered as `████░░░ 65%` where filled portion is `Color::Cyan`.
**Infrastructure checkboxes**: Built items in `Color::Green`, unbuilt in `Color::DarkGray`.
**Enter on cleared layer**: If a construction option is available and affordable, queues a construction mission selection flow.

### Layout (M)

```
┌─ Layers ──────────────────────────────────────────────┐
│ Frontier: Layer 8   Deepest: Layer 8                 │
├──────────────────────────┬────────────────────────────┤
│  L1  Shallows  [CLEAR]  │ Layer 4 — The Warrens      │
│  L2  Shallows  [CLEAR]  │ Cleared   Intel: 100%       │
│  L3  Shallows  [CLEAR]  │                            │
│  L4  Warrens   [CLEAR]  │ [✓] Outpost  [✓] Supply    │
│  L5  Warrens   [CLEAR]  │ [ ] Watchtower             │
│  L6  Warrens   [CLEAR]  │ Watchtower — 300 Marks     │
│  L7  Warrens   [CLEAR]  │                            │
│ ►L8  Hollows  [FRNTIR] │                            │
│  L9  Hollows  [??????] │                            │
├──────────────────────────┴────────────────────────────┤
│     [↑/↓] Navigate  [Enter] Build  [Esc] Back         │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

```
┌─ Layers ────────────────────────────────┐
│  L1  Shallows  CLEAR                   │
│  L4  Warrens   CLEAR                   │
│  L7  Warrens   CLEAR                   │
│ ►L8  Hollows   FRONTIER               │
│  L9  Hollows   ???                     │
│ [↑/↓] Navigate  [Esc] Back             │
└─────────────────────────────────────────┘
```

---

## View 5: Event Response

Shown when a mission has a pending check-in event. Accessible from Main view by pressing [Enter] on a mission with `⚡ Event pending!`. Can also be reached by pressing [E] from the main overlay.

### Layout (L/XL)

```
┌─ THE DEEP — Event ──────────────────────────────────────────────────┐
│                                                                     │
│  Mission: Expedition Layer 8   Progress: 78% (12h / 16h)           │
│  Squad: Aldric (Vanguard), Sera (Scout), Thorne (Arcanist)         │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │                                                                 │ │
│ │                     CAVE-IN AHEAD                               │ │
│ │                                                                 │ │
│ │   Your squad encounters a collapsed tunnel blocking the         │ │
│ │   main path. The ceiling groans under the weight above.         │ │
│ │   Dust and pebbles rain down from the passage ahead.           │ │
│ │                                                                 │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ► [Vanguard] Dig through       — 3h delay, safe                   │
│    [Saboteur] Find alternate    — no delay, moderate risk           │
│    [Arcanist] Blast through     — 1h delay, costs supplies         │
│    [Auto]     Let them decide   — always safe, worst outcome        │
│                                                                     │
│  Auto-resolve in: 23m   (safe choice will be selected)             │
│                                                                     │
│  [↑/↓] Choose  [Enter] Confirm  [Esc] Back (auto-resolve later)   │
└─────────────────────────────────────────────────────────────────────┘
```

**Choice rows**: Archetype tag colored per archetype (Vanguard = `Color::LightRed`, etc.). Options not available due to missing archetype shown in `Color::DarkGray` with `[--]` instead of archetype label.
**Auto-resolve timer**: `Color::Yellow` when >10 min remaining, `Color::LightRed` when <5 min.
**Esc from event**: Returns to main view. Event remains pending (timer continues).
**Consequence preview**: Shown inline after the choice text (delay, risk level, resource cost).

### Layout (M)

```
┌─ Event — Layer 8 ─────────────────────────────────────┐
│ Expedition 78%   Squad: Aldric, Sera, Thorne          │
├───────────────────────────────────────────────────────┤
│                                                       │
│               CAVE-IN AHEAD                           │
│                                                       │
│   Your squad encounters a collapsed tunnel            │
│   blocking the main path.                             │
│                                                       │
├───────────────────────────────────────────────────────┤
│ ► [Vanguard] Dig through    3h delay, safe           │
│   [Saboteur] Alternate route  no delay, risk         │
│   [Arcanist] Blast through  1h delay, supply cost    │
│   [Auto]     Let them decide  always safe            │
│                                                       │
│ Auto-resolve in: 23m                                  │
├───────────────────────────────────────────────────────┤
│     [↑/↓] Choose  [Enter] Confirm  [Esc] Back         │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

```
┌─ Event ─────────────────────────────────┐
│ L8 Expedition  78%  Auto: 23m          │
│                                         │
│   CAVE-IN AHEAD                        │
│                                         │
│ ► [Vanguard] Dig through  safe         │
│   [Saboteur] Alternate    risk         │
│   [Arcanist] Blast        supply       │
│   [Auto]     Decide                    │
│                                         │
│ [↑/↓] [Enter] Confirm  [Esc] Back     │
└─────────────────────────────────────────┘
```

---

## View 6: Mission Complete

Modal overlay displayed when entering the main view with a completed mission (Done! state). Pressing [Enter] collects rewards and dismisses.

### Layout (L/XL — centered modal, ~55x18)

```
┌─ Mission Complete ──────────────────────────────────────┐
│                                                        │
│  Supply Run — Layer 4 — The Warrens                   │
│  Duration: 3h   Result: SUCCESS                        │
│                                                        │
│  Rewards:                                              │
│   + 380 Warband Marks                                  │
│   + 1 Rare item (Iron Raider's Chestguard)            │
│   + 240 XP (Mira +1 mission)                          │
│                                                        │
│  Squad:                                                │
│   ✓ Mira (Medic L2) — returned safely                 │
│                                                        │
│                                                        │
│           [Enter] Collect and Close                    │
└────────────────────────────────────────────────────────┘
```

**Result line color**: SUCCESS in `Color::Green`, PARTIAL SUCCESS in `Color::Yellow`, FAILURE in `Color::Red`.
**Item rarity**: Colored via `rarity_color()`.
**Merc status**: `✓` in `Color::Green` for safe return, `!` in `Color::Yellow` for injured, `✗` in `Color::Red` for lost.

### Breakthrough variant (larger rewards)

```
┌─ Mission Complete ──────────────────────────────────────┐
│                                                        │
│  Breakthrough — Layer 8 — The Hollows                 │
│  Duration: 18h   Result: SUCCESS                       │
│                                                        │
│  Layer 9 unlocked! (The Hollows continue deeper)       │
│                                                        │
│  Rewards:                                              │
│   + 1,840 Warband Marks                               │
│   + 1 Legendary item (Abyssal Voidmantle)             │
│   + 0.5 Prestige Rank                                 │
│   + 1,200 XP (all squad members)                      │
│                                                        │
│  Squad:                                                │
│   ✓ Aldric (Vanguard L5) — returned safely            │
│   ✓ Sera (Scout L4) — returned safely                 │
│   ! Thorne (Arcanist L5) — injured (2 missions)       │
│                                                        │
│           [Enter] Collect and Close                    │
└────────────────────────────────────────────────────────┘
```

---

## Stats Panel Integration

The existing stats panel in `src/ui/stats_panel.rs` needs a small indicator when The Deep is active.

### [D] Indicator Placement

Added to the header section (`draw_header()`) alongside existing system indicators. Appears after the character name/level line:

```
┌──────────────────────────────────┐
│ Aldric the Bold  Lv.142  P23    │
│ ████████████████████░░ 87% XP   │
│ [H] Haven  [J] Forge  [D] Deep  │  ← indicator row
└──────────────────────────────────┘
```

**States for [D] Deep indicator**:

| State | Display | Color |
|-------|---------|-------|
| Not discovered | (hidden) | — |
| Discovered, no missions | `[D] Deep` | `Color::DarkGray` |
| Mission running | `[D] Deep ●` | `Color::Cyan` |
| Event pending | `[D] Deep ⚡` | `Color::Yellow` |
| Mission complete | `[D] Deep ✓` | `Color::Green` |

The indicator uses the most urgent state if multiple missions are running (event pending > mission complete > running > idle).

### M-tier stats bar

On M-tier, the compact stats bar adds a single character to the activity line:

```
P23 | H✓ | J+5 | D⚡ | Zone 8 | Lv.142 ...
```

Where `D⚡` collapses to the most urgent Deep state character. Colors follow the same table above.

### S-tier

On S-tier, add `[D]` to the footer keybind line only. No inline indicator (too little space).

---

## Interaction Flow Diagrams

### Flow 1: Discovery to First Mission

```
[Idle — P15+]
      │
      ▼ (tick-based discovery roll fires)
[Combat log: "A scarred mercenary captain approaches..."]
[Discovery modal — "The Deep Unlocked! Press [D] to open."]
      │ [Enter] or [D]
      ▼
[The Deep Main View — first open]
  Guild: Freelancers (Rank 1)   Marks: 0
  Mercs: 4/5 (starter roster)
  No active missions.
  [N] New Mission ...
      │ [N]
      ▼
[New Mission View]
  Available missions:
  ► [Supply Run] Layer 1  2h  Safe    ← only safe options shown first
    [Recon]     Layer 1  4h  Low
      │ select mission, assign mercs
      │ [Enter]
      ▼
[Main View — mission launched]
  ► Supply Run L1  ░░░░░░░░░░░░ 0%
      │ (real time passes)
      ▼
[Main View — mission complete]
  ► Supply Run L1  ████████████ Done! ✓
      │ [Enter]
      ▼
[Mission Complete Modal]
  + 120 Marks  + Common item  + XP
      │ [Enter]
      ▼
[Main View — rewards collected]
```

### Flow 2: Mission Launch

```
[Main View]
      │ [N]
      ▼
[New Mission View — left panel focused]
      │ [↑/↓] select mission
      ▼
[Mission detail updates in right panel]
      │ [Tab] switch to squad picker
      ▼
[Squad picker focused]
      │ [↑/↓] navigate mercs
      │ [Space] toggle assignment
      ▼
[Power display updates, requirement met/not]
      │ [Enter] (requirements met)
      ▼
[Confirmation: "Launch Expedition L8 with Aldric, Sera, Thorne? [Enter]/[Esc]"]
      │ [Enter]
      ▼
[Main View — new mission appears in active list]
```

### Flow 3: Event Response

```
[Main View — event pending indicator ⚡]
      │ [↑/↓] cursor on mission
      │ [Enter]
      ▼
[Event Response View]
  Event text + 4 choices displayed
      │ [↑/↓] select choice
      │ [Enter] confirm
      ▼
[Main View — event resolved]
  Mission continues (timer adjusted by choice outcome)
      │ OR player presses [Esc] without responding
      ▼
[Main View — event still pending, timer ticking]
      │ (auto-resolve timer expires)
      ▼
[Auto-resolve applies safe default choice silently]
```

### Flow 4: Prestige Transition

When player prestiges while Deep missions are active:

```
[Prestige Confirm Screen — standard]
      │ If Deep missions are active:
      ▼
[Warning line added to prestige confirm dialog]
  "Active Deep missions will be cancelled."
  "Guild rank, infrastructure, and layer progress persist."
      │ [Enter] confirm prestige
      ▼
[Prestige executes]
  - All active missions cancelled (no rewards)
  - Warband Marks reset to 0
  - Mercenaries dismissed (fresh roster on next open)
  - Guild rank preserved
  - Cleared layers preserved
  - Infrastructure preserved
      │ (new prestige begins)
      ▼
[The Deep Main View — first open after prestige]
  Guild: [same rank]   Marks: 0
  Mercs: 0/[max] — recruit fresh mercenaries
  [N] New Mission (re-enabled immediately)
```

---

## Module Structure

```
src/ui/
├── deep_scene.rs       — Main overlay coordinator, backdrop, tab routing
├── deep_missions.rs    — Active missions panel and mission list rendering
├── deep_roster.rs      — Roster sub-view
├── deep_layers.rs      — Layer map sub-view
├── deep_event.rs       — Event response sub-view
└── deep_results.rs     — Mission complete modal
```

### deep_scene.rs responsibilities

- `render_deep_overlay(frame, area, deep_state, ctx)` — top-level entry point
- `paint_deep_backdrop(buffer, millis)` — cave-blue gradient + dust particles
- `paint_opening_deep_fx(buffer, millis, elapsed)` — 600ms descent sheen
- Routes to sub-views based on `DeepUiState` (Main, NewMission, Roster, Layers, EventResponse)
- Renders border with title " THE DEEP "
- Renders footer help bar for current sub-view

### Color helper (add to src/ui/mod.rs)

```rust
pub fn layer_tier_color(layer: u32) -> Color {
    match layer {
        1..=3   => Color::Green,
        4..=7   => Color::Yellow,
        8..=12  => Color::Magenta,
        13..=18 => Color::Cyan,
        19..=25 => Color::LightRed,
        _       => Color::Rgb(255, 215, 0),
    }
}

pub fn merc_archetype_color(archetype: MercArchetype) -> Color {
    match archetype {
        MercArchetype::Vanguard  => Color::LightRed,
        MercArchetype::Scout     => Color::Cyan,
        MercArchetype::Arcanist  => Color::Magenta,
        MercArchetype::Medic     => Color::Green,
        MercArchetype::Saboteur  => Color::Yellow,
    }
}

pub fn mission_type_color(mission_type: MissionType) -> Color {
    match mission_type {
        MissionType::SupplyRun    => Color::Green,
        MissionType::Recon        => Color::Cyan,
        MissionType::Expedition   => Color::Yellow,
        MissionType::Breakthrough => Color::LightRed,
        MissionType::Construction => Color::Blue,
    }
}
```

---

## Keybind Conventions

| Key | Action |
|-----|--------|
| `d` | Toggle The Deep overlay (from main game) |
| Esc | Close overlay / back to parent view |
| N | New Mission sub-view |
| R | Roster sub-view |
| L | Layers sub-view |
| E | Event (if pending) sub-view |
| Enter | Confirm selection / collect rewards |
| Space | Toggle merc in squad picker |
| Tab | Switch panel focus (New Mission view) |
| Up/Down | Navigate list items |

The `d` keybind follows the same convention as `h` (Haven), `j` (Soulforge) — single lowercase letter toggles the overlay.

---

## Implementation Notes for Developers

### Scene buffer pattern

All sub-views render into a `Vec<Vec<SceneCell>>` buffer using `put_text()` and `put_cell()` from `scene_fx.rs`, then flush via `render_buffer()`. This matches Haven, Soulforge, and Stormglass.

### Sub-view switching

`DeepUiState` (defined in `src/deep/types.rs` or `src/input/types.rs`) tracks which sub-view is active. The input handler routes keys based on the current state. Pattern from `StormglassUiState` and `SoulforgeUiState`.

### Progress bar rendering

Mission progress bars use the same `Gauge` widget as the XP bar in `stats_panel.rs`:
```rust
let ratio = elapsed_secs as f64 / total_secs as f64;
let gauge = Gauge::default()
    .gauge_style(Style::default().fg(mission_type_color(mission.mission_type)))
    .ratio(ratio.clamp(0.0, 1.0));
```

Or inline via `put_text()` with block characters `████░░░` when rendering inside a scene buffer.

### Wall-clock time display

Format elapsed/remaining time using the existing `format_eta()` helper from `stats_prestige.rs`:
- `format_eta(remaining_secs)` → "~3h 20m"
- For elapsed: same function with elapsed_secs

### Responsive layout dispatch

```rust
match ctx.tier {
    SizeTier::TooSmall => render_too_small(frame, ctx),
    SizeTier::S        => render_deep_s(frame, area, state, ctx),
    SizeTier::M        => render_deep_m(frame, area, state, ctx),
    SizeTier::L | SizeTier::XL => render_deep_lxl(frame, area, state, ctx),
}
```

### Discovery modal

Follows `render_haven_discovery_modal()` pattern. Centered ~52x10 modal with Yellow border, flavor text, keybind hint. Trigger phrase in combat log: "A scarred mercenary captain approaches, maps of underground passages spilling from worn satchels."
