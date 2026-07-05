> Backported design record. Sources: docs/plans/2026-03-01-title-screen-stats-design.md.

## 2026-03-01-title-screen-stats-design.md

# Title Screen Stats: Deep Layer Badge & Ascension Level

**Date:** 2026-03-01

## Problem

The character select / title screen stat line only shows Prestige rank, Level, and equipment-based power. Players who have unlocked The Deep and Ascension have no visibility into these important progression milestones from the title screen.

## Design

### 1. Deep Layer Badge (Account-level, badges row)

Add a Deep Layer badge to the account journey badges row, positioned after Haven and before Cloud sync.

- **Icon:** `⛏` (U+26CF)
- **Format:** `⛏ L{deepest_layer_reached}`
- **Visibility:** Only when `deep.persistent.discovered == true`
- **Color:** Icon in amber/dark yellow, value in Gray
- **Data source:** `DeepPersistent.deepest_layer_reached` (account-level)

### 2. Ascension Level (Per-character, stat line)

Add Ascension level to the per-character stat line, between level and power.

- **Format:** `Asc {roman_numeral}` (I, II, III, IV, V, VI, VII, VIII, IX, X)
- **Visibility:** Only when `ascension_level > 0`
- **Position:** After `P{rank} Lv{level}`, before `{pwr} pwr`
- **Data source:** `GameState.ascension_level` (per-character, persists through prestige)

### 3. Character Power Rating (Behavior change)

Replace equipment-only power sum with full `compute_power_rating()` — the geometric mean of effective DPS and effective HP. This reflects actual combat strength including attributes, equipment, prestige bonuses, Haven, enhancement, ascension, and sigils.

- **Current:** `equipment.iter_equipped().map(|i| i.power()).sum()`
- **New:** `compute_power_rating(derived_stats, combat_bonuses, max_hp)`
- **Display:** Rounded integer, same `{pwr} pwr` format

## Visual Mockups

### Early game (no systems unlocked)
```
  ☆ 12 pts   · offline

  Heroes
  │ ▶ Warrior (Lv12 · 450 pwr)
```

### Mid game (Deep discovered, no Ascension)
```
  ☆ 215 pts   ⌂ 5/14   ⛏ L3   ☁ synced

  Heroes
  │ ▶ Warrior (P15 Lv180 · 2,450 pwr)
```

### Late game (Deep + Ascension)
```
  ☆ 847 pts   ⚒ +4/+4/+4   ⌂ 8/14   ⛏ L12   ☁ synced

  Heroes
  │ ▶ Warrior, Eternal (P50 Lv685 · Asc III · 12,400 pwr)
```

### Endgame (maxed)
```
  ☆ 4215 pts   ⚒ +10/+10/+10   ⌂ 14/14   ⛏ L30   ☁ synced

  Heroes
  │ ▶ Warrior, Eternal (P300 Lv4k · Asc VI · 45,000 pwr)
```

## Data Changes

### `CharacterInfo` struct (`src/character/manager.rs`)

Add `ascension_level: u32` field. Populated from `GameState.ascension_level` during `list_characters()` in `persistence.rs`.

### `build_startup_splash_text()` (`src/main_helpers/update.rs`)

- Add `deep: &DeepState` parameter for the Deep Layer badge
- Compute full power rating per character using:
  - `DerivedStats::calculate_derived_stats()` from character attributes + equipment + enhancement levels
  - `CombatBonuses` built from prestige rank, haven bonuses, god items, sigils, ascension level
  - `compute_power_rating(derived, bonuses, max_hp)`

### Roman numeral formatting

Add a small helper for Ascension level display (I through X+). Levels beyond X use the pattern: XI, XII, etc.

## Files Changed

| File | Change |
|------|--------|
| `src/character/manager.rs` | Add `ascension_level: u32` to `CharacterInfo` |
| `src/character/persistence.rs` | Populate `ascension_level` in `list_characters()` |
| `src/main_helpers/update.rs` | Add Deep badge, Ascension in stat line, compute power rating |
| `src/main.rs` (or caller) | Pass `&deep` to `build_startup_splash_text()` |
