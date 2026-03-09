# Achievement System

Account-level achievement tracking that persists across all characters. Tracks milestone progress for combat, leveling, prestige, zones, challenges, fishing, dungeons, and Haven building.

## Module Structure

```
src/achievements/
├── mod.rs            # Public re-exports
├── types.rs          # Data structures, AchievementId enum, Achievements state struct
├── data.rs           # Static achievement definitions (ALL_ACHIEVEMENTS constant)
├── handlers.rs       # Event handlers (on_enemy_killed, on_boss_killed, on_level_up, etc.), sync_* methods
├── milestones.rs     # MinigameType, MinigameDifficulty enums, milestone threshold arrays (SLAYER, BOSS_HUNTER, etc.)
├── modal.rs          # Modal notification queue, 500ms accumulation window management
├── notifications.rs  # Pending notification state, category-based notification counts
├── stats.rs          # Achievement statistics, unlock percentages, progress queries, category breakdowns, score computation
├── titles.rs         # Title definitions — maps curated achievements to display text, title selection/validation
├── unlock.rs         # Core unlock machinery (is_unlocked, unlock, unlock_with_name, check_milestones)
└── persistence.rs    # Save/load from ~/.quest/achievements.json
```

## Key Types

### `AchievementId` (`types.rs`)

Enum with 213 variants covering all trackable milestones. Organized by domain:

- **Combat**: `SlayerI`..`SlayerXV` (100 to 1B kills), `BossHunterI`..`BossHunterXV` (1 to 10M bosses)
- **Level**: `Level10`..`Level100000` (18 milestones)
- **Prestige**: `FirstPrestige`..`Prestige10000` (P1 to P10000, 19 milestones)
- **Zones**: `Zone1Complete`..`Zone10Complete`, `TheStormbreaker`, `StormsEnd`, `BeyondInfinity`, `FractureZone12`..`FractureZone30` (19 fracture zone completions)
- **Ascension**: `AscensionI`..`AscensionVI` (6 milestone achievements, one per level)
- **Power Cores**: `PowerCoreI`..`PowerCoreVI` (6 milestone achievements, unlocked at Deep Layers 3/7/12/18/25/30)
- **Challenges**: 4 difficulties per game type (chess, morris, gomoku, minesweeper, rune, go, flappy_bird, snake, jezzball, runic_shift) + `GrandChampion` (100 wins)
- **Enhancement**: `SoulforgeDiscovered`, `ApprenticeSmith` (+1), `FullyTempered` (+4 all), `JourneymanSmith` (+5), `SoulforgeAdept` (+6), `SoulforgeSavant` (+7), `SoulforgeMaster` (+8), `SoulforgeGrandmaster` (+9), `SoulforgeAscendant` (+10), `SoulConvergence` (+7 all), `PersistentHammering` (100 attempts)
- **Fishing**: `GoneFishing`, `FishermanI`..`FishermanIV` (rank milestones), `FishCatcherI`..`FishCatcherX` (100 to 100M fish catches), `StormLeviathan`
- **Dungeons**: `DungeonDiver`, `DungeonMasterI`..`DungeonMasterX` (10 to 1M dungeons)
- **Haven**: `HavenDiscovered`, `HavenBuilderI`..`HavenBuilderII`, `HavenArchitect`
- **Deep**: Discovery, first mission, mission count milestones (10/25/50/100), first breakthrough, layer milestones (Layers 5/10/15/20/25), VoidExplorer (Layer 26), guild rank milestones, first merc lost, gateway opened

### `AchievementCategory` (`types.rs`)

Nine categories for browsing: `Combat`, `Level`, `Prestige`, `Progression`, `Challenges`, `Exploration`, `Deep`, `Loom`, `Stats`.

### `AchievementDef` (`data.rs`)

Static definition with `id`, `name`, `description`, `category`, `icon`, and `points`. All definitions live in the `ALL_ACHIEVEMENTS` const slice. Points use a 7-tier system: Trivial (5), Easy (10), Medium (25), Hard (50), Very Hard (100), Elite (250), Pinnacle (500). 224 achievements total.

Achievement score is computed at runtime by summing the point values of all unlocked achievements. Score is displayed in four locations: browser title bar, achievement unlock modal, achievement detail panel, and stats view.

### `Achievements` (`types.rs`)

Main state struct (serialized to disk). Contains:

- `unlocked: HashMap<AchievementId, UnlockedAchievement>` -- which achievements are unlocked and when
- `progress: HashMap<AchievementId, AchievementProgress>` -- current/target for multi-stage achievements
- Aggregate counters: `total_kills`, `total_bosses_defeated`, `total_fish_caught`, `total_dungeons_completed`, `total_minigame_wins`, `highest_prestige_rank`, `highest_level`, `highest_fishing_rank`, `zones_fully_cleared`, `expanse_cycles_completed`
- `ui_border_style: UiBorderStyle` -- global border style for panel UI
- `selected_title: Option<AchievementId>` -- currently selected character title (account-wide)
- Transient fields (`#[serde(skip)]`): `pending_notifications`, `newly_unlocked`, `modal_queue`, `recently_unlocked`, `accumulation_start`

## How Achievements Are Unlocked

### Event Handler Pattern (`handlers.rs`)

`Achievements` exposes `on_*` methods (implemented in `handlers.rs`) that game systems call to report events. Each handler increments counters and checks milestone thresholds against arrays defined in `milestones.rs`:

```rust
// Called from tick.rs when an enemy dies
achievements.on_enemy_killed(is_boss, Some(&state.character_name));

// Called from tick.rs on level up
achievements.on_level_up(new_level, Some(&state.character_name));
```

Event handlers: `on_enemy_killed`, `on_level_up`, `on_prestige`, `on_zone_fully_cleared`, `on_storms_end`, `on_dungeon_completed`, `on_minigame_won`, `on_fish_caught`, `on_fishing_rank_up`, `on_storm_leviathan_caught`, `on_haven_discovered`, `on_haven_all_t1`, `on_haven_all_t2`, `on_haven_architect`, `on_soulforge_discovered`, `on_enhancement_upgraded`, `on_deep_discovered`, `on_deep_breakthrough`, `on_deep_guild_rank_up`, `on_deep_mission_complete`, `on_deep_merc_lost`, `on_deep_gateway_opened`, `on_fracture_boss_defeated`, `on_ascended`.

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
- **deep** (`deep/`): Deep milestones trigger `on_deep_discovered`, `on_deep_breakthrough`, `on_deep_guild_rank_up`, `on_deep_mission_complete`, `on_deep_merc_lost`, `on_deep_gateway_opened`.
- **UI** (`ui/achievement_browser_scene.rs`): Achievement browser overlay and unlock modal rendering.

## Title System (`titles.rs`)

Titles are display names earned by unlocking specific achievements. Players can select one title to display after their character name (e.g., "Hero, Godslayer"). Titles are account-wide and persist in `selected_title` on the `Achievements` struct.

- `ALL_TITLES`: const slice of `TitleDef { achievement_id, title_text }` — 64 curated titles across level, prestige, combat, challenges, exploration, enhancement, fracture, and Deep categories
- `get_title_text(id)`: returns the title text for an achievement, if it grants a title
- `get_unlocked_titles(achievements)`: returns all titles the player has earned, in display order
- `validate_selected_title(achievements)`: clears `selected_title` if the achievement isn't unlocked or doesn't grant a title (called on load)

Title browser UI: `ui/title_browser_scene.rs` — overlay opened with [T] from the achievement browser. Shows unlocked titles, preview of name + title, select with Enter, clear with Backspace.

Titles are shown in: stats panel header, character select screen, and achievement browser (✦ indicator on achievements that grant titles).

## Adding a New Achievement

1. Add variant to `AchievementId` enum in `types.rs`
2. Add `AchievementDef` entry to `ALL_ACHIEVEMENTS` in `data.rs` (with name, description, category, icon, points)
3. Add unlock logic: either add a threshold to a milestone array in `milestones.rs`, or create a new `on_*` handler in `handlers.rs`
4. Call the handler from `tick.rs` or `main.rs` at the appropriate point
5. Tests: add milestone test following the existing pattern
