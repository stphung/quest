# Unified Deep Panel Design

**Date**: 2026-03-02
**Replaces**: Power Cores panel (`draw_power_cores_panel()` in `src/ui/stats_prestige.rs`)

## Overview

Merge the Power Cores panel and Deep status into a single unified panel titled "The Deep" in the stats column. Same 8-row height as the current Power Cores panel (6 content rows + 2 border rows). Features mini progress bars for both missions and power cores.

## Layout

```
┌─ The Deep ───────────────────────────────────┐
│ Row 1: Guild rank + Warband Marks            │
│ Row 2: Msn N/M [progress bar] ~ETA   ⚡N     │
│ Row 3: Crew glyphs + Frontier                │
│ Row 4: ─────────── separator ───────────────│
│ Row 5: Core summary + PR/d                   │
│ Row 6: Per-core mini progress bars           │
└──────────────────────────────────────────────┘
```

## Row Details

### Row 1: Guild Rank + Currency
```
│ ⬡ Company        ◆ 1250 Warband Marks       │
```
- Left: `⬡` hex icon (White) + guild rank name
- Right-aligned: `◆` (Amber) + Warband Marks count

### Row 2: Missions + Progress Bar + Events
```
│ Msn 2/2 [████████░░░░] ~45m          ⚡1     │
```
- Left: `Msn N/M` (Cyan) — active count / max concurrent slots
- Center: 12-char progress bar showing nearest mission completion
  - Fill chars: `█` (Amber), empty: `░` (DarkGray)
  - Progress = nearest mission's elapsed / total duration
  - Only shown when active missions exist
- After bar: `~Xh Ym` time remaining
  - Yellow when < 15 minutes
  - DarkGray otherwise
- Right-aligned: `⚡N` in Yellow (pending events count, omit if 0)
- When no active missions: `Msn 0/N          ◷ idle` (no bar)

### Row 3: Crew + Frontier
```
│ ♦♦♦ ♢ ✝          Frontier L7                 │
```
- Left: One glyph per mercenary, grouped by status with spaces between groups:
  - `♦` Green — Available (ready for assignment)
  - `♢` Cyan — On Mission (deployed)
  - `✝` Red — Injured (benched)
- Right-aligned: `Frontier LN` in Gray-blue
- Empty roster: row shows only Frontier on the right, left side blank

### Row 4: Separator
```
│─────────────────────────────────────────────│
```

### Row 5: Core Summary
```
│ Cores: 2 ✓ (+4 PR)  Next: 2h 45m  +8 PR/d   │
```
- `Cores:` label in DarkGray
- Ready count + claimable PR in Green
- `Next: Xh Ym` — time until next core grants PR (DarkGray)
- Right-aligned: `+N PR/d` (Amber) — current daily PR generation rate
- When all unlocked cores ready: `All ready!` in Green+Bold (replaces Next)
- When no cores unlocked: `Cores: locked    First core at L3`

### Row 6: Per-Core Mini Progress Bars
```
│ ❂████ ❂████ ❂██░░ ◇L18 ◇L25 ◇L30           │
```
- Each unlocked core: `❂` (Amber) + 4-char progress bar
  - Fill chars: `█` (Amber for filling, Green for ready)
  - Empty chars: `░` (DarkGray)
  - Ready cores: all 4 filled in Green
  - Filling cores: proportional fill in Amber
- Locked cores: `◇LN` (DarkGray) with unlock layer
- Cores separated by spaces

## State Examples

### Discovery (first unlock, no mercs, no cores)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 0 Warband Marks          │
│ Msn 0/1          ◷ idle                      │
│                   Frontier L1                │
│─────────────────────────────────────────────│
│ Cores: locked    First core at L3            │
│ ◇L3 ◇L7 ◇L12 ◇L18 ◇L25 ◇L30               │
└──────────────────────────────────────────────┘
```

### Early game (1 core filling, 1 mission active)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 80 Warband Marks         │
│ Msn 1/1 [████░░░░░░░░] ~3h 15m              │
│ ♦♦                Frontier L3                │
│─────────────────────────────────────────────│
│ Cores: 0 ✓ Ready            Next: 4h 30m     │
│ ❂██░░ ◇L7 ◇L12 ◇L18 ◇L25 ◇L30    +2 PR/day │
└──────────────────────────────────────────────┘
```

### Mid game (3 cores, mixed crew, events)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Company        ◆ 1250 Warband Marks       │
│ Msn 2/2 [████████░░░░] ~45m          ⚡1     │
│ ♦♦♦ ♢ ✝          Frontier L7                 │
│─────────────────────────────────────────────│
│ Cores: 2 ✓ (+4 PR)                +8 PR/d    │
│ ❂████ ❂████ ❂██░░ ◇L18 ◇L25 ◇L30           │
└──────────────────────────────────────────────┘
```

### Late game (all 6 cores, large roster)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 8200 Warband Marks       │
│ Msn 2/4 [██████████░░] ~1h 20m              │
│ ♦♦♦♦ ♢♢ ✝✝       Frontier L30               │
│─────────────────────────────────────────────│
│ Cores: 1 ✓ (+6 PR)  Next: 1h 05m  +48 PR/d  │
│ ❂████ ❂███░ ❂██░░ ❂█░░░ ❂░░░░ ❂░░░░         │
└──────────────────────────────────────────────┘
```

### All cores ready, no missions
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 12000 Warband Marks      │
│ Msn 0/4          ◷ idle                      │
│ ♦♦♦♦♦♦♦♦          Frontier L30               │
│─────────────────────────────────────────────│
│ Cores: 6 ✓ (+48 PR)  All ready!    +48 PR/d  │
│ ❂████ ❂████ ❂████ ❂████ ❂████ ❂████         │
└──────────────────────────────────────────────┘
```

### Mission about to complete (<15m)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Battalion      ◆ 3400 Warband Marks       │
│ Msn 1/3 [████████████] ~8m                   │
│ ♦♦♦♦♦             Frontier L18               │
│─────────────────────────────────────────────│
│ Cores: 3 ✓ (+12 PR)  All ready!   +12 PR/d   │
│ ❂████ ❂████ ❂████ ◇L18 ◇L25 ◇L30           │
└──────────────────────────────────────────────┘
```

## Color Reference

| Element | Color | Notes |
|---------|-------|-------|
| `⬡` guild rank icon | White | |
| `◆` Warband Marks | Amber `Rgb(220,180,60)` | |
| `Msn N/M` | Cyan | |
| Mission bar fill `█` | Amber `Rgb(255,165,0)` | |
| Mission bar empty `░` | DarkGray | |
| Mission ETA text | Yellow (<15m), DarkGray (otherwise) | |
| `◷ idle` | DarkGray | |
| `♦` available merc | Green | |
| `♢` deployed merc | Cyan | |
| `✝` injured merc | Red | |
| Frontier label | Gray-blue `Rgb(120,140,170)` | |
| `⚡` events | Yellow | |
| `❂` unlocked core | Amber `Rgb(255,165,0)` | |
| Core bar fill (filling) `█` | Amber `Rgb(255,165,0)` | |
| Core bar fill (ready) `█` | Green | |
| Core bar empty `░` | DarkGray | |
| `◇` locked core | DarkGray | |
| `All ready!` | Green + Bold | |
| `+N PR/d` | Amber `Rgb(255,165,0)` | |
| Panel border | Amber (themed) | Same as current Power Cores |
| Separator | DarkGray | |

## Implementation Notes

- Panel function: `draw_deep_panel()` in `src/ui/stats_prestige.rs`
- Panel title: `" The Deep "`
- Height: `8` rows (6 content + 2 border)
- Panel visibility: shown when `deep.persistent.discovered` is true
- Mission progress bar: 12 chars wide, uses nearest active mission's `progress(Utc::now())`
- Core mini bars: 4 chars wide per core, uses `fill_ratio()` from power_cores
- Crew glyphs: iterate `prestige.roster`, skip `MercStatus::Lost`, emit one glyph per merc grouped by status
- Merc statuses are mutually exclusive: `Available | OnMission(id) | Injured { missions_remaining } | Lost`
