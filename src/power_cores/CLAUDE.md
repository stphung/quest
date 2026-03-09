# Power Cores System

Passive prestige rank generation tied to Deep layer milestones. Six Power Cores, each unlocked by clearing a specific Deep layer (the same layers that unlock fracture zones). Once active, a core passively generates prestige ranks over real wall-clock time.

## Module Structure

```
src/power_cores/
├── mod.rs           # Public re-exports
├── types.rs         # PowerCoreDef, PowerCoreState, ALL_POWER_CORES, helper functions
└── tick.rs          # Per-tick processing (tick_power_cores), offline catchup (apply_offline_power_cores), init_new_core
```

## Key Types

### `PowerCoreDef` (`types.rs`)

Static definition: `achievement_id` (AchievementId), `name` (&str), `pr_per_day` (u32), `required_layer` (u32).

### `ALL_POWER_CORES` (`types.rs`)

Const slice of 6 core definitions:

| Core | Name | Deep Layer | PR/Day | Fill Time |
|------|------|-----------|--------|-----------|
| I | Red Fault | 3 | 2 | 12h |
| II | Mirror Scar | 7 | 3 | 8h |
| III | Black Mouth | 12 | 5 | ~4.8h |
| IV | Hollow Throne | 18 | 8 | 3h |
| V | Wailing Reach | 25 | 12 | 2h |
| VI | Origin Wound | 30 | 18 | ~1.3h |

Total at all 6 cores active: 48 PR/day.

### `PowerCoreState` (`types.rs`)

Runtime state: `last_granted_at: HashMap<AchievementId, i64>` — Unix timestamp per core for when it last granted a PR. Missing entries treated as "never granted" (timestamp = 0).

## Key Functions

### `types.rs`

- `get_power_core_def(id) -> Option<&PowerCoreDef>` — Look up core by achievement ID
- `get_unlocked_cores(achievements) -> Vec<&PowerCoreDef>` — All cores whose achievement is unlocked
- `fill_duration_secs(pr_per_day) -> i64` — Seconds per cycle: `86400 / pr_per_day`

### `tick.rs`

- `tick_power_cores(state, power_core_state, achievements, result)` — Per-tick processing. For each unlocked core, checks if fill timer elapsed and grants +1 PR per completed cycle. Sets `result.power_cores_changed`.
- `apply_offline_power_cores(state, power_core_state, achievements) -> u32` — Offline catchup. Returns total PR granted.
- `init_new_core(power_core_state, achievement_id)` — Initialise a newly unlocked core's timestamp to now.

## Tick Behavior

Each game tick (100ms):
1. Get current wall-clock time
2. For each unlocked core:
   - If `last_granted_at == 0`: initialise to now (first cycle starts from unlock moment)
   - If elapsed >= fill duration: grant `completed_cycles` PR, advance timestamp, emit `PowerCoreGranted` events
3. Set `power_cores_changed = true` if any grant occurred

## Persistence

Power Cores state is persisted as part of the Deep system. `DeepState.persistent.power_core_last_granted: HashMap<AchievementId, i64>` holds the Unix timestamp per core for when it last granted a PR. It is saved/loaded alongside `~/.quest/deep.json` by the Deep module's persistence functions.

## Integration Points

- **Core** (`core/tick_types.rs`): `TickEvent::PowerCoreGranted { core_name }` variant; `TickResult::power_cores_changed` flag
- **Achievements** (`achievements/types.rs`): `PowerCoreI` through `PowerCoreVI` achievement IDs unlock each core
- **Deep** (`deep/`): Deep layer breakthroughs unlock the corresponding Power Core achievements
- **Main** (`main.rs`): Calls `tick_power_cores()` each tick, `apply_offline_power_cores()` on character load, saves state on change
