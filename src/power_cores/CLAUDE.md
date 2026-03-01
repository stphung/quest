# Power Cores System

Passive prestige rank generation tied to Deep layer milestones. Six Power Cores, each unlocked by clearing a specific Deep layer (the same layers that unlock fracture zones). Once active, a core passively generates prestige ranks over real wall-clock time.

## Module Structure

```
src/power_cores/
├── mod.rs           # Public re-exports
├── types.rs         # PassivesState, GeneratorTimer, PowerCoreDef, ALL_POWER_CORES, helper functions
├── tick.rs          # Per-tick processing (tick_power_cores), offline catchup (apply_offline_power_cores), init_new_core
└── persistence.rs   # Save/load from ~/.quest/passives.json
```

## Key Types

### `PassivesState` (`types.rs`)

Top-level persistent state for all passive generators. Saved to `~/.quest/passives.json`.

```rust
pub struct PassivesState {
    pub generators: HashMap<String, GeneratorTimer>,
}
```

Each entry is keyed by a stable string ID (e.g. `"power_core_1"`) that maps to a static definition. The struct is generic — future passive generator types share the same file and HashMap.

### `GeneratorTimer` (`types.rs`)

Timer state for a single passive generator: `last_granted_at: i64` (Unix timestamp). A value of `0` means "never granted". Stored as a struct (not bare `i64`) to allow adding `#[serde(default)]` fields later without a breaking schema change.

### `PowerCoreDef` (`types.rs`)

Static definition: `key` (&str), `achievement_id` (AchievementId), `name` (&str), `pr_per_day` (u32), `required_layer` (u32).

### `ALL_POWER_CORES` (`types.rs`)

Const slice of 6 core definitions:

| Core | Key | Name | Deep Layer | PR/Day | Fill Time |
|------|-----|------|-----------|--------|-----------|
| I | `power_core_1` | Red Fault | 3 | 2 | 12h |
| II | `power_core_2` | Mirror Scar | 7 | 3 | 8h |
| III | `power_core_3` | Black Mouth | 12 | 5 | ~4.8h |
| IV | `power_core_4` | Hollow Throne | 18 | 8 | 3h |
| V | `power_core_5` | Wailing Reach | 25 | 12 | 2h |
| VI | `power_core_6` | Origin Wound | 30 | 18 | ~1.3h |

Total at all 6 cores active: 48 PR/day.

## Key Functions

### `types.rs`

- `get_power_core_def(id) -> Option<&PowerCoreDef>` — Look up core by achievement ID
- `get_unlocked_cores(achievements) -> Vec<&PowerCoreDef>` — All cores whose achievement is unlocked
- `fill_duration_secs(pr_per_day) -> i64` — Seconds per cycle: `86400 / pr_per_day`

### `tick.rs`

- `tick_power_cores(state, passives, achievements, result)` — Per-tick processing. For each unlocked core, checks if fill timer elapsed and grants +1 PR per completed cycle. Sets `result.passives_changed`.
- `apply_offline_power_cores(state, passives, achievements) -> u32` — Offline catchup. Returns total PR granted.
- `init_new_core(passives, key: &str)` — Initialise a newly unlocked core's timestamp to now.

## Tick Behavior

Each game tick (100ms):
1. Get current wall-clock time
2. For each unlocked core:
   - If `last_granted_at == 0`: initialise to now (first cycle starts from unlock moment)
   - If elapsed >= fill duration: grant `completed_cycles` PR, advance timestamp, emit `PowerCoreGranted` events
3. Set `passives_changed = true` if any grant occurred

## Persistence

- **File**: `~/.quest/passives.json` (pretty-printed JSON via serde)
- **Load**: `load_passives()` — returns default if missing/corrupted
- **Save**: `save_passives()` — creates `~/.quest/` if needed
- **Trigger**: `main.rs` saves when `TickResult::passives_changed` is true

## Integration Points

- **Core** (`core/tick_types.rs`): `TickEvent::PowerCoreGranted { core_name }` variant; `TickResult::passives_changed` flag
- **Achievements** (`achievements/types.rs`): `PowerCoreI` through `PowerCoreVI` achievement IDs unlock each core
- **Deep** (`deep/`): Deep layer breakthroughs unlock the corresponding Power Core achievements
- **Main** (`main.rs`): Calls `tick_power_cores()` each tick, `apply_offline_power_cores()` on character load, saves state on change

## Extending with New Generators

To add a new passive generator type:
1. Define static definitions (like `ALL_POWER_CORES`) with stable string keys
2. Add a tick function that iterates your definitions, reads/writes `passives.generators` using your keys
3. Set `result.passives_changed = true` when state changes — the existing save path handles the rest
