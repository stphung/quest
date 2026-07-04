> Backported design record. Sources: docs/plans/2026-02-22-time-vault-ux-design.md.

## 2026-02-22-time-vault-ux-design.md

# Time Vault UX Overhaul Design

## Overview

Upgrade the Time Vault overlay from a plain two-panel browser to a visually polished experience with an animated backdrop, timeline graph, event-type icons/colors, and branch panel improvements.

## Layout

Two-panel layout (branches left, snapshots right) — same as current. No third preview panel. Animated backdrop rendered behind both panels using the `scene_fx.rs` SceneCell buffer pipeline.

```
╔══════════════════════════ TIME VAULT ══════════════════════════╗
║▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓║
║▓ Branches ▓▓▓▓▓▓ Snapshots ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓║
║▓┌──────────────┐ ┌──────────────────────────────────────────┐▓║
║▓│ ● main (12)  │ │  ● ⚔ Defeated Dark Forest boss          │▓║
║▓│ ○ speedrun   │ │  │   Feb 22, 2026  3:42 PM              │▓║
║▓│              │ │  │   Lv30 · P5 · Zone 5 · 10h 15m       │▓║
║▓│              │ │  │ ─────────────────────────────────     │▓║
║▓│              │ │  ○ ★ Prestige to rank 5                  │▓║
║▓│              │ │  │   Feb 22, 2026  2:18 PM              │▓║
║▓│              │ │  │   Lv25 · P5 · Zone 1 · 9h 40m        │▓║
║▓│              │ │  │ ─────────────────────────────────     │▓║
║▓│              │ │  ○ ♟ Won Chess at Journeyman             │▓║
║▓│              │ │  │   Feb 22, 2026  1:05 PM              │▓║
║▓│              │ │  │   Lv25 · P4 · Zone 4 · 8h 30m        │▓║
║▓│              │ │  │ ─────────────────────────────────     │▓║
║▓│              │ │  ○ ◆ Completed medium dungeon            │▓║
║▓│              │ │      Feb 21, 2026  11:30 PM             │▓║
║▓│              │ │      Lv22 · P4 · Zone 3 · 7h 00m        │▓║
║▓└──────────────┘ └──────────────────────────────────────────┘▓║
║▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓║
║  [Enter] Restore  [F] Fork  [Tab] Branches  [Esc] Close      ║
╚══════════════════════════════════════════════════════════════╝
```

## 1. Animated Backdrop

- Deep blue/cyan temporal theme
- Dark navy gradient top-to-bottom behind both panels
- Subtle cyan particles drifting slowly (fewer and slower than Stormglass — vault atmosphere, not storm)
- Uses `scene_fx.rs` SceneCell buffer rendering pipeline (`paint_storm_backdrop` pattern adapted)
- Panels rendered on top with dark backgrounds

### Backdrop Parameters

- Top RGB: `(8, 12, 35)` (dark navy)
- Bottom RGB: `(3, 5, 15)` (near black)
- Particle count: 5 (subtle, not busy)
- Particle speed: 0.8 (slow drift)
- Shimmer: true (faint temporal shimmer)

## 2. Timeline Graph (Snapshot Panel)

- Vertical `│` line connecting commits in the right panel
- `●` for selected commit, `○` for unselected
- Last commit has no trailing `│` — timeline ends
- Selected `●` and text: yellow + bold
- Unselected `○`: cyan
- Connecting `│` line: dim cyan `Color::Rgb(40, 80, 120)`
- Thin `─────` separators between cards instead of blank lines (dim cyan)

### Card Layout (4 lines per card)

```
  ● ⚔ Defeated Dark Forest boss
  │   Feb 22, 2026  3:42 PM
  │   Lv30 · P5 · Zone 5 · 10h 15m
  │ ─────────────────────────────
```

Last card omits the separator and trailing `│`.

## 3. Event Icons and Colors

Each commit gets a type-specific icon parsed from the commit message prefix.

| Pattern match (message starts with) | Icon | Color |
|--------------------------------------|------|-------|
| "Defeated" | `⚔` | LightRed |
| "Prestige" | `★` | Rgb(255, 215, 0) (Gold) |
| "Won " (challenge) | `♟` | Magenta |
| "Completed" (dungeon) | `◆` | Green |
| "Caught" / "Fishing" | `~` | Blue |
| "Built" / "Upgraded" (Haven) | `⌂` | Yellow |
| "Enhanced" (Soulforge) | `⚒` | Cyan |
| "Achievement" | `✦` | White |
| "Chrono Surge" | `⏩` | Cyan |
| Other/fallback | `·` | DarkGray |

- Icon color is applied to the icon character only
- Event text stays white (selected) or dim white (unselected)

## 4. Branch Panel Polish

- `●` for active branch (green), `○` for inactive (dim white)
- Commit count shown next to branch name: `● main (12)`
- Selected branch: yellow + bold (unchanged)
- Commit count requires data — `TimelineInfo` already has `head_commit` but not count; we either count from the commits list or add a count field

## 5. Controls Bar

- Same keybindings, no logic changes
- `·` dot separators between control groups for visual polish

## Implementation Notes

- The main rendering function `draw_time_vault` switches from direct widget rendering to the SceneCell buffer approach (like `stormglass_scene.rs`)
- Panel contents are painted onto the buffer after the backdrop
- Branch panel and snapshot panel content rendered with `put_text` / `put_cell` helpers from `scene_fx.rs`
- Controls bar can stay as a regular Paragraph widget below the buffer area
- Scrolling logic for commit list stays the same
