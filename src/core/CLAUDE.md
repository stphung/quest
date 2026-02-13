# Core Module

Central game state, game logic, balance constants, and the per-tick orchestration function that drives all gameplay systems.

## Module Structure

```
src/core/
├── mod.rs         # Public re-exports (GameState, constants, TickEvent, TickResult)
├── constants.rs   # All game balance constants (timing, XP, drops, discovery, zones)
├── game_state.rs  # GameState struct and RecentDrop display type
├── game_logic.rs  # XP curves, leveling, offline progression, enemy spawning
└── tick.rs        # game_tick() orchestration — the central per-tick function
```

## Key Types

### `GameState` (`game_state.rs`)

The main character state struct. Serialized to JSON for saves in `~/.quest/`.

```rust
pub struct GameState {
    // Persistent (saved to disk)
    pub character_id: String,          // UUID
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,        // 6 RPG attributes
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,           // Unix timestamp for offline XP
    pub play_time_seconds: u64,
    pub combat_state: CombatState,
    pub equipment: Equipment,
    pub active_dungeon: Option<Dungeon>,
    pub fishing: FishingState,
    pub zone_progression: ZoneProgression,
    pub chess_stats: ChessStats,

    // Transient (serde(skip), reset on load)
    pub active_fishing: Option<FishingSession>,
    pub challenge_menu: ChallengeMenu,
    pub active_minigame: Option<ActiveMinigame>,
    pub session_kills: u64,
    pub recent_drops: VecDeque<RecentDrop>,  // Capped at 10
    pub last_minigame_win: Option<MinigameWinInfo>,
}
```

Key methods:
- `new(name, time)` -- Creates fresh character with base stats (level 1, 50 HP, all attributes 10)
- `get_attribute_cap()` -- Returns `20 + prestige_rank * 5`
- `add_recent_drop(...)` -- Push to front of bounded deque (max 10, evicts oldest)
- `is_in_dungeon()` -- Checks `active_dungeon.is_some()`

### `RecentDrop` (`game_state.rs`)

Display-only struct for the Loot panel. Not serialized.

```rust
pub struct RecentDrop {
    pub name: String,
    pub rarity: Rarity,
    pub equipped: bool,
    pub icon: &'static str,   // Unicode emoji
    pub slot: String,          // "Weapon", "Armor", etc.
    pub stats: String,         // "+8 STR +3 DEX +Crit"
}
```

### `OfflineReport` (`game_logic.rs`)

Returned by `process_offline_progression()` to summarize what happened while the player was away.

```rust
pub struct OfflineReport {
    pub elapsed_seconds: i64,
    pub total_level_ups: u32,
    pub xp_gained: u64,
    pub level_before: u32,
    pub level_after: u32,
    pub offline_rate_percent: f64,
    pub haven_bonus_percent: f64,
}
```

### `TickEvent` (`tick.rs`)

Enum with 25+ variants describing everything that can happen in a single tick. The presentation layer (main.rs) maps these to combat log entries and visual effects. Game logic never touches UI types.

**Categories:**
- **Combat**: `PlayerAttack`, `PlayerAttackBlocked`, `EnemyAttack`, `EnemyDefeated`, `PlayerDied`, `PlayerDiedInDungeon`
- **Item Drops**: `ItemDropped` (with rarity, slot, stats, equipped flag)
- **Zone Progression**: `SubzoneBossDefeated` (with `BossDefeatResult`)
- **Dungeon**: `DungeonRoomEntered`, `DungeonTreasureFound`, `DungeonKeyFound`, `DungeonBossUnlocked`, `DungeonBossDefeated`, `DungeonEliteDefeated`, `DungeonFailed`, `DungeonCompleted`
- **Fishing**: `FishingMessage`, `FishCaught`, `FishingItemFound`, `FishingRankUp`, `StormLeviathanCaught`
- **Discovery**: `ChallengeDiscovered`, `DungeonDiscovered`, `FishingSpotDiscovered`, `HavenDiscovered`
- **Achievements**: `AchievementUnlocked`
- **Level Up**: `LeveledUp`

Each variant carries pre-formatted message strings with unicode escapes (e.g., `\u{2694}` for crossed swords). The presentation layer uses these directly for log entries.

### `TickResult` (`tick.rs`)

```rust
pub struct TickResult {
    pub events: Vec<TickEvent>,
    pub leviathan_encounter: Option<u8>,          // Encounter number 1-10
    pub achievements_changed: bool,                // Signal to persist to disk
    pub haven_changed: bool,                       // Signal to persist to disk
    pub achievement_modal_ready: Vec<AchievementId>, // Ready for overlay display
}
```

## game_tick() Architecture

```rust
pub fn game_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    achievements: &mut Achievements,
    debug_mode: bool,
    rng: &mut R,
) -> TickResult
```

**Why generic `<R: Rng>`**: The `rand::Rng` trait is not dyn-compatible, so we use a generic parameter. Pass `&mut rand::thread_rng()` in production, or a seeded `ChaCha8Rng` in tests for deterministic behavior.

### Processing Stages

| Stage | What it does |
|-------|-------------|
| 1. Challenge AI | Ticks AI thinking for active Chess, Morris, Gomoku, or Go games |
| 2. Challenge discovery | Rolls for new challenge discovery (P1+ required, Haven bonus applied) |
| 3. Sync player HP | Recalculates `DerivedStats`, computes `PrestigeCombatBonuses::from_rank()`, applies `flat_hp` to `combat_state.player_max_hp` |
| 4. Dungeon exploration | Calls `update_dungeon()`, processes room entry, treasure, keys, boss unlock, completion/failure |
| 5. Fishing | If fishing active: ticks session, handles catches/items/rank-ups/Leviathan, updates play time, **returns early** (skips combat) |
| 6. Combat | Calls `update_combat(state, dt, haven, prestige_bonuses, achievements)`, maps `CombatEvent` to `TickEvent`, applies XP, handles kills/deaths, processes item drops and discoveries |
| 7. Enemy spawn | Calls `spawn_enemy_if_needed()` if no enemy and not regenerating |
| 8. Play time | Increments tick counter; at 10 ticks, increments `play_time_seconds` |
| 9. Achievement collection | Drains newly unlocked achievements into `TickResult.events` |
| 10. Haven discovery | Rolls for Haven discovery (P10+, no active content) |
| 11. Achievement modal | Checks if 500ms accumulation window has elapsed for modal display |

**Important**: Stage 5 (fishing) returns early, skipping stages 6-7. Fishing and combat are mutually exclusive.

### Helper Functions (private)

- `process_item_drop(state, haven, result)` -- Rolls mob/boss drops, auto-equips, adds to recent drops
- `process_discoveries(state, rng, result)` -- Rolls dungeon and fishing spot discovery after kills
- `process_zone_achievements(defeat_result, achievements, name)` -- Tracks zone completion achievements
- `collect_achievement_events(achievements, result)` -- Drains unlock queue into TickResult events

## Key Functions (`game_logic.rs`)

### XP and Leveling

| Function | Signature | Purpose |
|----------|-----------|---------|
| `xp_for_next_level` | `(level: u32) -> u64` | XP curve: `100 * level^1.5` |
| `prestige_multiplier` | `(rank: u32, cha_modifier: i32) -> f64` | Base from prestige tier + CHA bonus (0.1 per modifier point) |
| `xp_gain_per_tick` | `(prestige_rank, wis_mod, cha_mod) -> f64` | `1.0 * prestige_mult * (1 + wis_mod * 0.05)` |
| `apply_tick_xp` | `(state, xp: f64) -> (levelups, attrs)` | Applies XP, processes level-ups in a loop, distributes +3 attribute points per level |
| `distribute_level_up_points` | `(state) -> Vec<AttributeType>` | Randomly distributes 3 points among non-capped attributes |
| `combat_kill_xp` | `(passive_rate, haven_bonus) -> u64` | Random 200-400 ticks of XP per kill, with Haven Training Yard bonus |

### Offline Progression

| Function | Signature | Purpose |
|----------|-----------|---------|
| `calculate_offline_xp` | `(elapsed, rank, wis, cha, haven_bonus) -> f64` | Simulates kills at 25% rate, capped at 7 days |
| `process_offline_progression` | `(state, haven_bonus) -> OfflineReport` | Full offline XP processing, updates `last_save_time` |

Offline XP formula: `(elapsed_seconds / 5.0) * 0.25 * xp_per_kill * (1 + haven_bonus/100)`

### Enemy Spawning

| Function | Signature | Purpose |
|----------|-----------|---------|
| `spawn_enemy_if_needed` | `(state)` | Spawns zone or dungeon enemy if no enemy and not regenerating. Uses zone-based generators (`generate_enemy_for_current_zone`, `generate_boss_for_current_zone`) |
| `spawn_dungeon_enemy` | `(state)` (private) | Spawns Combat/Elite/Boss enemy via `generate_dungeon_enemy(zone_id)`, `generate_dungeon_elite(zone_id)`, `generate_dungeon_boss(zone_id)` |
| `try_discover_dungeon` | `(state) -> bool` | 2% chance per call, generates dungeon via `generate_dungeon(level, prestige_rank, zone_id)` |

## Constants (`constants.rs`)

### Timing
| Constant | Value | Notes |
|----------|-------|-------|
| `TICK_INTERVAL_MS` | 100 | 10 ticks/sec |
| `ATTACK_INTERVAL_SECONDS` | 1.5 | |
| `HP_REGEN_DURATION_SECONDS` | 2.5 | After kill |
| `AUTOSAVE_INTERVAL_SECONDS` | 30 | |
| `UPDATE_CHECK_INTERVAL_SECONDS` | 1800 | 30 minutes |
| `UPDATE_CHECK_JITTER_SECONDS` | 300 | +/- 5 min |

### XP and Leveling
| Constant | Value | Notes |
|----------|-------|-------|
| `XP_CURVE_BASE` | 100.0 | `100 * level^1.5` |
| `XP_CURVE_EXPONENT` | 1.5 | |
| `COMBAT_XP_MIN_TICKS` | 200 | XP per kill range |
| `COMBAT_XP_MAX_TICKS` | 400 | |
| `OFFLINE_MULTIPLIER` | 0.25 | 25% of online rate |
| `MAX_OFFLINE_SECONDS` | 604800 | 7 days |

### Character
| Constant | Value | Notes |
|----------|-------|-------|
| `BASE_ATTRIBUTE_VALUE` | 10 | Starting value for all 6 attributes |
| `BASE_ATTRIBUTE_CAP` | 20 | Cap at prestige 0 |
| `ATTRIBUTE_CAP_PER_PRESTIGE` | 5 | +5 cap per prestige rank |
| `LEVEL_UP_ATTRIBUTE_POINTS` | 3 | Random distribution |

### Item Drops
| Constant | Value | Notes |
|----------|-------|-------|
| `ITEM_DROP_BASE_CHANCE` | 0.15 | 15% |
| `ITEM_DROP_PRESTIGE_BONUS` | 0.01 | +1% per rank |
| `ITEM_DROP_MAX_CHANCE` | 0.25 | Hard cap |
| `ZONE_ILVL_MULTIPLIER` | 10 | ilvl = zone_id * 10 |

### Discovery
| Constant | Value | Notes |
|----------|-------|-------|
| `DUNGEON_DISCOVERY_CHANCE` | 0.02 | 2% per kill |
| `FISHING_DISCOVERY_CHANCE` | 0.05 | 5% per kill |
| `CHALLENGE_DISCOVERY_CHANCE` | 0.000014 | ~2hr avg |
| `HAVEN_DISCOVERY_BASE_CHANCE` | 0.000014 | Per tick |
| `HAVEN_DISCOVERY_RANK_BONUS` | 0.000007 | Per rank above 10 |
| `HAVEN_MIN_PRESTIGE_RANK` | 10 | |

### Zone Enemy Stats
| Constant | Value | Notes |
|----------|-------|-------|
| `ZONE_ENEMY_STATS` | `[(u32,u32,u32,u32,u32,u32); 11]` | Per-zone `(base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)`. Index 0=Zone 1, Index 10=Zone 11. Steps are per-subzone-depth increments above depth 1 |

Zone 11 (The Expanse) is an endgame wall: `(5000, 400, 500, 80, 250, 30)` — roughly 6.2x HP, 4.6x DMG, 4.8x DEF over Zone 10.

### Boss Multipliers
| Constant | Value | Notes |
|----------|-------|-------|
| `SUBZONE_BOSS_MULTIPLIERS` | `(3.0, 1.5, 1.8)` | (HP, DMG, DEF) multipliers |
| `ZONE_BOSS_MULTIPLIERS` | `(5.0, 1.8, 2.5)` | |
| `DUNGEON_ELITE_MULTIPLIERS` | `(2.2, 1.5, 1.6)` | |
| `DUNGEON_BOSS_MULTIPLIERS` | `(3.5, 1.8, 2.0)` | |

### Prestige Combat Bonuses
| Constant | Value | Notes |
|----------|-------|-------|
| `PRESTIGE_FLAT_DAMAGE_FACTOR` | 5.0 | `floor(5.0 * rank^0.7)` |
| `PRESTIGE_FLAT_DAMAGE_EXPONENT` | 0.7 | |
| `PRESTIGE_FLAT_DEFENSE_FACTOR` | 3.0 | `floor(3.0 * rank^0.6)` |
| `PRESTIGE_FLAT_DEFENSE_EXPONENT` | 0.6 | |
| `PRESTIGE_CRIT_PER_RANK` | 0.5 | +0.5% per rank |
| `PRESTIGE_CRIT_CAP` | 15.0 | Max bonus crit % |
| `PRESTIGE_FLAT_HP_FACTOR` | 15.0 | `floor(15.0 * rank^0.6)` |
| `PRESTIGE_FLAT_HP_EXPONENT` | 0.6 | |

### Enemy Attack Intervals
| Constant | Value | Notes |
|----------|-------|-------|
| `ENEMY_ATTACK_INTERVAL_SECONDS` | 2.0 | Normal mobs |
| `ENEMY_BOSS_ATTACK_INTERVAL_SECONDS` | 1.8 | Subzone bosses |
| `ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS` | 1.5 | Zone bosses |
| `ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS` | 1.6 | Dungeon elites |
| `ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS` | 1.4 | Dungeon bosses |

### Zone Progression / Fishing
| Constant | Value | Notes |
|----------|-------|-------|
| `KILLS_FOR_BOSS` | 10 | Kills per subzone before boss |
| `KILLS_FOR_BOSS_RETRY` | 5 | Kills needed to retry after boss death |
| `BASE_MAX_FISHING_RANK` | 30 | Without Haven |
| `MAX_FISHING_RANK` | 40 | With Fishing Dock T4 |

## Integration Points

### tick.rs depends on (inputs)
- **combat** (`combat::logic`): `update_combat(state, dt, haven, prestige_bonuses, achievements)` returns `Vec<CombatEvent>`, `HavenCombatBonuses` struct
- **character** (`character::prestige`): `PrestigeCombatBonuses::from_rank()` — computed each tick for combat bonuses
- **character** (`character::derived_stats`): `DerivedStats::calculate_derived_stats()`
- **dungeon** (`dungeon::logic`): `update_dungeon()`, `on_room_enemy_defeated()`, `on_elite_defeated()`, `on_boss_defeated()`, `add_dungeon_xp()`, `calculate_boss_xp_reward()`, `on_treasure_room_entered()`
- **fishing** (`fishing::logic`): `tick_fishing_with_haven_result()`, `check_rank_up_with_max()`, `get_max_fishing_rank()`, `HavenFishingBonuses` struct
- **challenges** (`challenges::*::logic`): `process_ai_thinking()` per game type, `try_discover_challenge_with_haven()`
- **haven** (`haven`): `Haven`, `HavenBonusType`, `try_discover_haven()`
- **achievements** (`achievements`): `Achievements` with `on_*()` tracking methods
- **items** (`items::drops`): `try_drop_from_mob()`, `try_drop_from_boss()`; (`items::scoring`): `auto_equip_if_better()`
- **zones** (`zones`): `BossDefeatResult`, `get_zone()`, `get_all_zones()`

### game_logic.rs depends on
- **character**: `Attributes`, `AttributeType`, `DerivedStats`, `prestige::get_prestige_tier()`
- **combat**: `generate_enemy_for_current_zone()`, `generate_boss_for_current_zone()`, `generate_dungeon_enemy()`, `generate_dungeon_elite()`, `generate_dungeon_boss()`
- **dungeon**: `generate_dungeon(level, prestige_rank, zone_id)`, `RoomType`
- **zones**: `ZoneProgression`

### Other modules depend on core
- **main.rs**: Calls `game_tick()`, processes `TickResult`, handles IO (save, visual effects, log entries)
- **character/manager.rs**: Creates/loads `GameState`, calls `process_offline_progression()`
- **UI modules**: Read `GameState` fields for display (read-only)

## Design Decisions

- **tick.rs has zero `ui::` imports**: All UI updates happen in main.rs by mapping `TickEvent` variants to log entries and visual effects.
- **tick.rs DOES call `add_recent_drop()`**: For fishing catches and item drops. This is state mutation, not UI.
- **Messages are pre-formatted**: TickEvent variants carry message strings with unicode escapes. The presentation layer uses them directly rather than formatting again.
- **`achievements_changed` / `haven_changed` flags**: Signal that IO is needed. The presentation layer (main.rs) owns the actual file writes.
- **debug_mode suppresses save signals**: When `--debug` is active, achievement and haven save flags are suppressed to avoid polluting saves during testing.
- **Fishing early return**: Stage 5 returns early when fishing is active, making fishing and combat mutually exclusive within a tick.

## Known Issues

- `combat_kill_xp()` and `distribute_level_up_points()` use internal `thread_rng()` (not yet parameterized with generic `R: Rng`)
- `try_discover_dungeon()` also uses internal `thread_rng()`
