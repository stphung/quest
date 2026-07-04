> Backported design record. Sources: docs/plans/2026-02-16-extended-combat-milestones-design.md.

## 2026-02-16-extended-combat-milestones-design.md

# Extended Combat Kill Milestones

## Summary

Extend the Slayer (enemy kills) and Boss Hunter (boss kills) achievement series from their current caps to tier XV each. Adds 13 new combat achievements with escalating thresholds and unique names for the top 3 tiers.

## New Achievements

### Slayer Series (enemy kills) -- 6 new tiers (X-XV)

| ID | Name | Kills | Icon | Description |
|----|------|-------|------|-------------|
| SlayerX | Slayer X | 2,500,000 | \u{2694} | Defeat 2,500,000 enemies |
| SlayerXI | Slayer XI | 10,000,000 | \u{2694} | Defeat 10,000,000 enemies |
| SlayerXII | Slayer XII | 50,000,000 | \u{2694} | Defeat 50,000,000 enemies |
| SlayerXIII | Harbinger | 100,000,000 | \u{2694} | Defeat 100,000,000 enemies |
| SlayerXIV | Reaper | 500,000,000 | \u{2694} | Defeat 500,000,000 enemies |
| SlayerXV | Death Incarnate | 1,000,000,000 | \u{2694} | Defeat 1,000,000,000 enemies |

### Boss Hunter Series (boss kills) -- 7 new tiers (IX-XV)

| ID | Name | Bosses | Icon | Description |
|----|------|--------|------|-------------|
| BossHunterIX | Boss Hunter IX | 25,000 | \u{1f451} | Defeat 25,000 bosses |
| BossHunterX | Boss Hunter X | 75,000 | \u{1f451} | Defeat 75,000 bosses |
| BossHunterXI | Boss Hunter XI | 250,000 | \u{1f451} | Defeat 250,000 bosses |
| BossHunterXII | Boss Hunter XII | 750,000 | \u{1f451} | Defeat 750,000 bosses |
| BossHunterXIII | Titan Breaker | 2,500,000 | \u{1f451} | Defeat 2,500,000 bosses |
| BossHunterXIV | Worldender | 5,000,000 | \u{1f451} | Defeat 5,000,000 bosses |
| BossHunterXV | The Absolute | 10,000,000 | \u{1f451} | Defeat 10,000,000 bosses |

## Design Decisions

- **Both series cap at tier XV** for consistency
- **Top 3 tiers get unique names** (Harbinger/Reaper/Death Incarnate and Titan Breaker/Worldender/The Absolute) to feel special and distinct from the numbered tiers
- **Thresholds use exponential spacing** with increasingly large gaps (2.5x-5x jumps) to create a long tail
- **Slayer XV (1 billion)** and **Boss Hunter XV (10 million)** are monument-tier achievements that may take months of idle time
- **No new infrastructure**: reuses existing `total_kills` and `total_bosses_defeated` counters, existing `on_enemy_killed()` handler, and existing `check_milestones()` pattern

## Files Changed

1. `src/achievements/types.rs` -- Add 13 new `AchievementId` enum variants; extend milestone arrays in `on_enemy_killed()` and `refresh_progress()`
2. `src/achievements/data.rs` -- Add 13 new `AchievementDef` entries in `ALL_ACHIEVEMENTS`; extend `test_every_achievement_id_variant_has_definition` test list

## Not Changed

- No new counters or tracking fields on `Achievements`
- No new event handlers
- No UI changes
- No save format changes
- No changes to tick.rs or combat logic
