# Unified Deep Panel Design

**Date**: 2026-03-02
**Replaces**: Power Cores panel (`draw_power_cores_panel()` in `src/ui/stats_prestige.rs`)

## Overview

Merge the Power Cores panel and Deep status into a single unified panel titled "The Deep" in the stats column. Same 8-row height as the current Power Cores panel (6 content rows + 2 border rows).

## Layout

```
┌─ The Deep ───────────────────────────────────┐
│ Row 1: Guild rank + Warband Marks            │
│ Row 2: Mission slots + next completion       │
│ Row 3: Crew glyphs + Frontier + events       │
│ Row 4: ─────────── separator ───────────────│
│ Row 5: Core summary (ready count, next ETA)  │
│ Row 6: Per-core status badges                │
└──────────────────────────────────────────────┘
```

## Row Details

### Row 1: Guild Rank + Currency
```
│ ⬡ Company        ◆ 1250 Warband Marks       │
```
- Left: `⬡` hex icon (White) + guild rank name
- Right-aligned: `◆` (Amber) + Warband Marks count

### Row 2: Missions + Timer
```
│ Missions 2/2     ◷ Next: ~45m               │
```
- Left: `Missions N/M` (Cyan) — active count / max concurrent slots
- Right: `◷ Next: ~Xh Ym` — time until next mission completes
  - Yellow when < 15 minutes
  - DarkGray otherwise
  - `◷ idle` when no active missions

### Row 3: Crew + Frontier + Events
```
│ ♦♦♦ ♢ ✝          Frontier L7    ⚡1          │
```
- Left: One glyph per mercenary, grouped by status with spaces between groups:
  - `♦` Green — Available (ready for assignment)
  - `♢` Cyan — On Mission (deployed)
  - `✝` Red — Injured (benched)
- Center-right: `Frontier LN` in Gray-blue
- Far right: `⚡N` in Yellow (pending events count, omit if 0)
- Empty roster: row shows only Frontier on the right, left side blank

### Row 4: Separator
```
│─────────────────────────────────────────────│
```

### Row 5: Core Summary
```
│ Cores: 2 ✓ Ready (+4 PR)  ·  Next: 2h 45m   │
```
- Ready count in Green + total PR available to collect
- `Next: Xh Ym` — time until next core grants PR
- When all unlocked cores are ready: `All ready!` in Green+Bold
- When no cores unlocked: `Cores: locked    First core at L3`

### Row 6: Per-Core Badges
```
│ ❂✓ ❂✓ ❂2h ◇L18 ◇L25 ◇L30     +8 PR/day     │
```
- `❂` (Amber) = unlocked core, followed by:
  - `✓` (Green) if ready
  - `Xh` (DarkGray) time remaining
- `◇` (DarkGray) = locked core, followed by unlock layer (e.g., `L18`)
- Right-aligned: `+N PR/day` (Amber) — current total daily PR generation rate

## State Examples

### Discovery (first unlock, no mercs, no cores)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 0 Warband Marks          │
│ Missions 0/1     ◷ idle                      │
│                   Frontier L1                │
│─────────────────────────────────────────────│
│ Cores: locked    First core at L3            │
│ ◇L3 ◇L7 ◇L12 ◇L18 ◇L25 ◇L30               │
└──────────────────────────────────────────────┘
```

### Early game (1 core, small crew)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 80 Warband Marks         │
│ Missions 0/1     ◷ idle                      │
│ ♦♦                Frontier L3                │
│─────────────────────────────────────────────│
│ Cores: 0 ✓ Ready            Next: 4h 30m     │
│ ❂4h ◇L7 ◇L12 ◇L18 ◇L25 ◇L30   +2 PR/day    │
└──────────────────────────────────────────────┘
```

### Mid game (3 cores, mixed crew, events)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Company        ◆ 1250 Warband Marks       │
│ Missions 2/2     ◷ Next: ~45m               │
│ ♦♦♦ ♢ ✝          Frontier L7    ⚡1          │
│─────────────────────────────────────────────│
│ Cores: 2 ✓ Ready (+4 PR)  ·  Next: 2h 45m   │
│ ❂✓ ❂✓ ❂2h ◇L18 ◇L25 ◇L30     +8 PR/day     │
└──────────────────────────────────────────────┘
```

### Late game (all 6 cores, large roster)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 8200 Warband Marks       │
│ Missions 2/4     ◷ Next: ~1h 20m            │
│ ♦♦♦♦ ♢♢ ✝✝       Frontier L30               │
│─────────────────────────────────────────────│
│ Cores: 1 ✓ Ready (+6 PR)  ·  Next: 1h 05m   │
│ ❂✓ ❂1h ❂3h ❂5h ❂8h ❂11h      +48 PR/day    │
└──────────────────────────────────────────────┘
```

### All cores ready
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 12000 Warband Marks      │
│ Missions 0/4     ◷ idle                      │
│ ♦♦♦♦♦♦♦♦          Frontier L30               │
│─────────────────────────────────────────────│
│ Cores: 6 ✓ Ready (+48 PR) ·  All ready!      │
│ ❂✓ ❂✓ ❂✓ ❂✓ ❂✓ ❂✓            +48 PR/day     │
└──────────────────────────────────────────────┘
```

## Color Reference

| Element | Color | Notes |
|---------|-------|-------|
| `⬡` guild rank icon | White | |
| `◆` Warband Marks | Amber `Rgb(220,180,60)` | |
| Mission count | Cyan | |
| `◷` timer | Yellow (<15m), DarkGray (otherwise) | |
| `◷ idle` | DarkGray | |
| `♦` available merc | Green | |
| `♢` deployed merc | Cyan | |
| `✝` injured merc | Red | |
| Frontier label | Gray-blue | |
| `⚡` events | Yellow | |
| `❂` unlocked core | Amber | |
| `◇` locked core | DarkGray | |
| `✓` ready | Green | |
| Core time remaining | DarkGray | |
| `All ready!` | Green + Bold | |
| `+N PR/day` | Amber | |
| Panel border | Amber (themed) | Same as current Power Cores |
| Separator | DarkGray | |

## Implementation Notes

- Replaces `draw_power_cores_panel()` in `src/ui/stats_prestige.rs`
- Panel title changes from `" Power Cores "` to `" The Deep "`
- Height remains `8` rows (same `Constraint` in `draw_stats_panel()`)
- Panel visibility: shown when Deep is discovered (same condition as current Power Cores)
- Crew glyphs: iterate `prestige.roster`, skip `MercStatus::Lost`, emit one glyph per merc grouped by status
- Core badges: iterate `ALL_POWER_CORES`, check unlock status and fill progress
- PR/day: sum of `core.pr_per_day()` for all unlocked cores
- Merc statuses are mutually exclusive: `Available | OnMission(id) | Injured { missions_remaining } | Lost`
