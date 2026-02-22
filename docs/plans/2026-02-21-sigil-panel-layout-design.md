# Storm Sigil Panel Layout Redesign

## Problem

Storm Sigils are currently rendered as a sub-panel nested inside the Equipment block in `stats_equipment.rs`. This is thematically wrong — sigils are permanent soul-inscribed progression bonuses, not gear. They deserve their own section.

## Design

Move Storm Sigils to a dedicated top-level panel in the stats panel, positioned between Attributes and Equipment.

### Layout

```
┌─ Header ────────────────────────────┐  (4h)
├── Prestige ─────────────────────────┤  (5h)
├── Fishing ──────────────────────────┤  (4h)
├── Attributes ───────────────────────┤  (5h)
├── ᚱ Storm Sigils (2/5) ────────────┤  (dynamic: etched + 2)
│  📖 Wisdom   +12.3% XP          A+ │
│  🔥 Fury      +8.0% Damage       B │
├── Equipment ────────────────────────┤  (Min(0))
│ Weapon  Stormblade     Legendary T7 │
│ ...                                 │
└─────────────────────────────────────┘
```

### Visibility Rules

- Panel only appears when at least 1 sigil is etched
- Only etched sigils are shown (no empty/locked slots — those live in the Exchange overlay)
- Dynamic height: `Constraint::Length(etched_count + 2)` (content + borders)
- When no sigils etched, equipment gets the full `Min(0)` space

### Panel Details

- Title: ` ᚱ Storm Sigils (etched/unlocked) ` with electric blue border (`Color::Rgb(100, 180, 255)`)
- Row format: same as current — `icon shortname    +value% stat  grade`
- Grade colors: S=gold, A=green, B=cyan, C=white, D=gray, E=darkgray, F=red
- Grade styling: `+` variants bold, `-` variants dim

## Changes

1. **`stats_panel.rs`**: Add conditional sigil panel constraint between attributes and equipment
2. **New `stats_sigils.rs`**: Top-level sigil panel renderer (extracted from `stats_equipment.rs`)
3. **`stats_equipment.rs`**: Remove sigil sub-panel splitting logic — equipment is purely equipment again
4. **`ui/mod.rs`**: Add `mod stats_sigils;`
