# Achievement System

Account-level achievement tracking that persists across all characters. Tracks milestone progress for combat, leveling, prestige, zones, challenges, fishing, dungeons, and Haven building.

## Module Structure

```
src/achievements/
├── mod.rs          # Public re-exports
├── types.rs        # Data structures, AchievementId enum, Achievements state, event handlers
├── data.rs         # Static achievement definitions (ALL_ACHIEVEMENTS constant)
└── persistence.rs  # Save/load from ~/.quest/achievements.json
```

## Key Types

### `AchievementId` (`types.rs`)

Enum with 80+ variants covering all trackable milestones. Organized by domain:

- **Combat**: `SlayerI`..`SlayerIX` (100 to 1M kills), `BossHunterI`..`BossHunterVIII` (1 to 10K bosses)
- **Level**: `Level10`..`Level1500` (11 milestones)
- **Prestige**: `FirstPrestige`..`Eternal` (P1 to P100, 12 milestones)
- **Zones**: `Zone1Complete`..`Zone10Complete`, `TheStormbreaker`, `StormsEnd`, `ExpanseCycleI`..`ExpanseCycleIV`
- **Challenges**: 4 difficulties per game type (chess, morris, gomoku, minesweeper, rune, go, flappy_bird, snake, jezzball) + `GrandChampion` (100 wins)
- **Fishing**: `GoneFishing`, `FishermanI`..`FishermanIV` (rank milestones), `FishCatcherI`..`FishCatcherIV` (catch counts), `StormLeviathan`
- **Dungeons**: `DungeonDiver`, `DungeonMasterI`..`DungeonMasterVI`
- **Haven**: `HavenDiscovered`, `HavenBuilderI`..`HavenBuilderII`, `HavenArchitect`

### `AchievementCategory` (`types.rs`)

Five categories for browsing: `Combat`, `Level`, `Progression`, `Challenges`, `Exploration`.

### `AchievementDef` (`data.rs`)

Static definition with `id`, `name`, `description`, `category`, and `icon`. All definitions live in the `ALL_ACHIEVEMENTS` const slice.

### `Achievements` (`types.rs`)

Main state struct (serialized to disk). Contains:

- `unlocked: HashMap<AchievementId, UnlockedAchievement>` -- which achievements are unlocked and when
- `progress: HashMap<AchievementId, AchievementProgress>` -- current/target for multi-stage achievements
- Aggregate counters: `total_kills`, `total_bosses_defeated`, `total_fish_caught`, `total_dungeons_completed`, `total_minigame_wins`, `highest_prestige_rank`, `highest_level`, `highest_fishing_rank`, `zones_fully_cleared`, `expanse_cycles_completed`
- Transient fields (`#[serde(skip)]`): `pending_notifications`, `newly_unlocked`, `modal_queue`, `accumulation_start`

## How Achievements Are Unlocked

### Event Handler Pattern

`Achievements` exposes `on_*` methods that game systems call to report events. Each handler increments counters and checks milestone thresholds using a shared `check_milestones()` helper:

```rust
// Called from tick.rs when an enemy dies
achievements.on_enemy_killed(is_boss, Some(&state.character_name));

// Called from tick.rs on level up
achievements.on_level_up(new_level, Some(&state.character_name));
```

Event handlers: `on_enemy_killed`, `on_level_up`, `on_prestige`, `on_zone_fully_cleared`, `on_storms_end`, `on_dungeon_completed`, `on_minigame_won`, `on_fish_caught`, `on_fishing_rank_up`, `on_storm_leviathan_caught`, `on_haven_discovered`, `on_haven_all_t1`, `on_haven_all_t2`, `on_haven_architect`.

### Unlock Flow

1. Event handler calls `unlock()` with the achievement ID and character name
2. `unlock()` checks for duplicates, inserts into `unlocked` map with timestamp
3. ID is pushed to three transient lists: `pending_notifications`, `newly_unlocked`, `modal_queue`
4. `accumulation_start` timer begins on first unlock in a batch

### Retroactive Sync

When loading a character, `sync_from_game_state()` retroactively unlocks achievements for milestones already passed (e.g., loading a level 120 character unlocks Level10 through Level100). Similarly, `sync_from_haven()` syncs Haven tier achievements. Note: kill/boss/dungeon counters cannot be synced retroactively since they are stored in the achievements file, not character saves.

## Modal Notification System

Achievements use a 500ms accumulation window to batch notifications:

1. First unlock in a batch sets `accumulation_start = Some(Instant::now())`
2. Subsequent unlocks within 500ms are added to `modal_queue`
3. `is_modal_ready()` returns true after 500ms has elapsed
4. `take_modal_queue()` drains the queue and resets the timer
5. `tick.rs` checks `is_modal_ready()` and passes IDs to `TickResult::achievement_modal_ready`
6. `main.rs` sets `GameOverlay::AchievementUnlocked` to display the modal

Additionally, `newly_unlocked` is drained each tick by `collect_achievement_events()` in `tick.rs`, which emits `TickEvent::AchievementUnlocked` events. These are logged to the combat log in `main.rs`.

`pending_notifications` tracks unviewed achievements for the UI indicator badge. Cleared when the user opens the achievement browser.

## Persistence

- **File**: `~/.quest/achievements.json` (pretty-printed JSON via serde)
- **Load**: `load_achievements()` returns `Achievements::default()` if file is missing or corrupted
- **Save**: `save_achievements()` creates the `~/.quest/` directory if needed
- **Trigger**: `main.rs` saves whenever `TickResult::achievements_changed` is true (set by `collect_achievement_events()` in `tick.rs`)
- **Also saved**: on prestige, minigame win, quit, and character switch

## Integration Points

- **tick.rs** (`core/tick.rs`): Calls `on_*` handlers during combat, fishing, dungeon, and discovery processing. Collects `TickEvent::AchievementUnlocked` events. Checks modal readiness.
- **main.rs**: Loads/saves achievements. Syncs on character load. Handles prestige and minigame win achievements. Routes `TickEvent::AchievementUnlocked` to combat log. Displays modal overlay from `achievement_modal_ready`.
- **game_logic.rs** (`core/game_logic.rs`): `update_combat()` takes `&mut Achievements` and calls `on_enemy_killed` on kills.
- **haven** (`haven/logic.rs`): Haven upgrades trigger `on_haven_all_t1/t2/architect` checks.
- **zones** (`zones/data.rs`): `sync_zone_completions()` uses zone definitions to check which zones are fully cleared.
- **UI** (`ui/achievement_browser_scene.rs`): Achievement browser overlay and unlock modal rendering.

## Adding a New Achievement

1. Add variant to `AchievementId` enum in `types.rs`
2. Add `AchievementDef` entry to `ALL_ACHIEVEMENTS` in `data.rs` (with name, description, category, icon)
3. Add unlock logic: either add a threshold to an existing `check_milestones()` call, or create a new `on_*` handler
4. Call the handler from `tick.rs` or `main.rs` at the appropriate point
5. Tests: add milestone test in `types.rs` following the existing pattern
