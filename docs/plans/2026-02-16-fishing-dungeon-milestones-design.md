# Extended Fishing & Dungeon Milestones

## Summary

Extend the Fish Catcher (fish caught) series from tier IV to tier X and the Dungeon Master (dungeons completed) series from tier VI to tier X. Adds 10 new achievements with escalating thresholds and unique names for the top 3 tiers of each series.

## New Achievements

### Fish Catcher Series (fish caught) -- 6 new tiers (V-X)

| ID | Name | Fish | Icon | Description |
|----|------|------|------|-------------|
| FishCatcherV | Fish Catcher V | 500,000 | \u{1f40b} | Catch 500,000 fish |
| FishCatcherVI | Fish Catcher VI | 1,000,000 | \u{1f40b} | Catch 1,000,000 fish |
| FishCatcherVII | Fish Catcher VII | 5,000,000 | \u{1f40b} | Catch 5,000,000 fish |
| FishCatcherVIII | Leviathan's Rival | 10,000,000 | \u{1f988} | Catch 10,000,000 fish |
| FishCatcherIX | Poseidon's Hand | 50,000,000 | \u{1f30a} | Catch 50,000,000 fish |
| FishCatcherX | Lord of the Deep | 100,000,000 | \u{1f531} | Catch 100,000,000 fish |

### Dungeon Master Series (dungeons completed) -- 4 new tiers (VII-X)

| ID | Name | Dungeons | Icon | Description |
|----|------|----------|------|-------------|
| DungeonMasterVII | Dungeon Master VII | 25,000 | \u{1f451} | Complete 25,000 dungeons |
| DungeonMasterVIII | Labyrinth Lord | 100,000 | \u{1f3db}\u{fe0f} | Complete 100,000 dungeons |
| DungeonMasterIX | Abyss Walker | 500,000 | \u{1f300} | Complete 500,000 dungeons |
| DungeonMasterX | The Undying Delver | 1,000,000 | \u{1f451} | Complete 1,000,000 dungeons |

## Design Decisions

- **Both series cap at tier X** for consistency
- **Top 3 tiers get unique names** following the pattern from Extended Combat Milestones
- **Fish Catcher icons**: numbered tiers continue with \u{1f40b} (whale), top 3 use "Rising Power" theme (\u{1f988}/\u{1f30a}/\u{1f531})
- **Dungeon Master icons**: numbered tier continues with \u{1f451} (crown), top 3 use "Labyrinth Deep" theme (\u{1f3db}\u{fe0f}/\u{1f300}/\u{1f451})
- **Thresholds use exponential spacing** with long-tail gaps
- **No new infrastructure**: reuses existing `total_fish_caught` and `total_dungeons_completed` counters

## Files Changed

1. `src/achievements/types.rs` -- Add 10 new `AchievementId` enum variants; extend milestone arrays in `on_fish_caught()`, `on_dungeon_completed()`, and `refresh_progress()`
2. `src/achievements/data.rs` -- Add 10 new `AchievementDef` entries; extend test list

## Not Changed

- No new counters or tracking fields on `Achievements`
- No new event handlers
- No UI changes
- No save format changes
- No changes to tick.rs or combat logic
