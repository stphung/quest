# Unified Deep Panel Design

**Date**: 2026-03-02 (updated 2026-03-03)
**Replaces**: Power Cores panel (`draw_power_cores_panel()` in `src/ui/stats_prestige.rs`)

## Overview

Merge the Power Cores panel and Deep status into a single unified panel titled "The Deep" in the stats column. Same 8-row height as the current Power Cores panel (6 content rows + 2 border rows). Features mission progress bar and aggregate core progress bar.

## Layout

```
┌─ The Deep ───────────────────────────────────┐
│ Row 1: Guild rank + Warband Marks            │
│ Row 2: Missions N/M [progress bar] ~ETA  ⚡N  │
│ Row 3: Crew glyphs + Frontier                │
│ Row 4: ─────────── separator ───────────────│
│ Row 5: Cores [aggregate bar] ~ETA    +N PR/d │
│ Row 6: Per-core rate·status pairs            │
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
│ Missions 2/2 [████████░░░░] ~45m     ⚡1     │
```
- Left: `Missions N/M` (Cyan)
- Center: 12-char progress bar showing nearest mission completion
  - Fill: `█` (Amber), empty: `░` (DarkGray), brackets in DarkGray
  - Progress = nearest active mission's elapsed / total duration
  - Only shown when active missions exist
- After bar: `~Xh Ym` time remaining (Yellow <15m, DarkGray otherwise)
- Right-aligned: `⚡N` in Yellow (omit if 0)
- No active missions: `Missions 0/N     ◷ idle` (no bar)

### Row 3: Crew + Frontier
```
│ ♦♦♦ ♢ ✝          Frontier L7                 │
```
- `♦` Green = Available, `♢` Cyan = On Mission, `✝` Red = Injured
- Groups separated by spaces, Lost mercs skipped
- Right-aligned: `Frontier LN` in Gray-blue
- Empty roster: blank left side

### Row 4: Separator
```
│─────────────────────────────────────────────│
```

### Row 5: Aggregate Core Progress Bar
```
│ Cores [████████░░░░] ~2h 45m         +8 PR/d │
```
- `Cores` label in DarkGray
- 12-char progress bar showing the **soonest-to-complete** core's fill ratio
  - Fill: `█` Amber (or Green when all ready), empty: `░` DarkGray
- After bar: time until next PR grant via `format_eta()`
- Right-aligned: `+N PR/d` in Amber
- All unlocked cores ready: bar fully filled Green, `All ready!` in Green+Bold
- No cores unlocked: `Cores: locked    First core at L3`

### Row 6: Per-Core Rate·Status Pairs
```
│ 2·✓  3·✓  5·2h  ◇L18  ◇L25  ◇L30            │
```
- Each unlocked core: `{pr_per_day}·{status}`
  - PR rate number in Amber — this IS the core's identity and conveys speed
  - `·` separator in DarkGray
  - Ready: `✓` in Green
  - Filling: `Xh` or `Xm` in DarkGray
- Locked cores: `◇LN` in DarkGray
- Two spaces between each entry

## State Examples

### Discovery (no mercs, no cores)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 0 Warband Marks          │
│ Missions 0/1     ◷ idle                      │
│                   Frontier L1                │
│─────────────────────────────────────────────│
│ Cores: locked    First core at L3            │
│ ◇L3  ◇L7  ◇L12  ◇L18  ◇L25  ◇L30            │
└──────────────────────────────────────────────┘
```

### Early game (1 core filling, 1 mission)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 80 Warband Marks         │
│ Missions 1/1 [████░░░░░░░░] ~3h 15m         │
│ ♦♦                Frontier L3                │
│─────────────────────────────────────────────│
│ Cores [██░░░░░░░░░░] ~4h 30m         +2 PR/d │
│ 2·4h  ◇L7  ◇L12  ◇L18  ◇L25  ◇L30           │
└──────────────────────────────────────────────┘
```

### Mid game (3 cores, mixed crew, events)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Company        ◆ 1250 Warband Marks       │
│ Missions 2/2 [████████░░░░] ~45m     ⚡1     │
│ ♦♦♦ ♢ ✝          Frontier L7                 │
│─────────────────────────────────────────────│
│ Cores [████████░░░░] ~2h 45m         +8 PR/d │
│ 2·✓  3·✓  5·2h  ◇L18  ◇L25  ◇L30            │
└──────────────────────────────────────────────┘
```

### Late game (all 6 cores, large roster)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 8200 Warband Marks       │
│ Missions 2/4 [██████████░░] ~1h 20m          │
│ ♦♦♦♦ ♢♢ ✝✝       Frontier L30               │
│─────────────────────────────────────────────│
│ Cores [████████░░░░] ~1h 05m        +48 PR/d │
│ 2·✓  3·1h  5·3h  8·5h  12·8h  18·11h         │
└──────────────────────────────────────────────┘
```

### All cores ready, no missions
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 12000 Warband Marks      │
│ Missions 0/4     ◷ idle                      │
│ ♦♦♦♦♦♦♦♦          Frontier L30               │
│─────────────────────────────────────────────│
│ Cores [████████████] All ready!     +48 PR/d │
│ 2·✓  3·✓  5·✓  8·✓  12·✓  18·✓               │
└──────────────────────────────────────────────┘
```

## Color Reference

| Element | Color | Notes |
|---------|-------|-------|
| `⬡` guild rank icon | White | |
| `◆` Warband Marks | Amber `Rgb(220,180,60)` | |
| `Missions N/M` | Cyan | |
| Mission bar fill `█` | Amber `Rgb(255,165,0)` | |
| Mission bar empty `░` | DarkGray | |
| Mission ETA text | Yellow (<15m), DarkGray (otherwise) | |
| `◷ idle` | DarkGray | |
| `♦` available merc | Green | |
| `♢` deployed merc | Cyan | |
| `✝` injured merc | Red | |
| Frontier label | Gray-blue `Rgb(120,140,170)` | |
| `⚡` events | Yellow | |
| `Cores` label | DarkGray | |
| Core bar fill (filling) `█` | Amber `Rgb(255,165,0)` | |
| Core bar fill (all ready) `█` | Green | |
| Core bar empty `░` | DarkGray | |
| Core ETA text | DarkGray | |
| `All ready!` | Green + Bold | |
| PR rate number | Amber `Rgb(255,165,0)` | |
| `·` separator | DarkGray | |
| `✓` ready | Green | |
| Core time remaining | DarkGray | |
| `◇` locked core | DarkGray | |
| `+N PR/d` | Amber `Rgb(255,165,0)` | |
| Panel border | Amber (themed) | |
| Separator | DarkGray | |

## Implementation Notes

- Panel function: `draw_deep_panel()` in `src/ui/stats_prestige.rs`
- Panel title: `" The Deep "`
- Height: `8` rows (6 content + 2 border)
- Visibility: `deep.persistent.discovered`
- Mission bar: 12 chars, uses nearest active mission's `progress(Utc::now())`
- Core aggregate bar: 12 chars, uses `next_ready_ratio` from `CoreSummary`
- Core rate labels: PR rate as identity (2, 3, 5, 8, 12, 18) — inherently conveys speed
- Crew glyphs: one per merc, grouped by MercStatus, Lost skipped
