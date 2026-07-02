# Ascension System

Per-character combat power multiplier purchased with prestige ranks, gated by Deep layer milestones (I-VI) or Loom Woven Pattern counts (VII-X). Each Ascension level doubles all combat stats (damage, defense, HP) for levels I-VI, with diminishing returns at VII+. MAX_ASCENSION_LEVEL is 10.

## Module Structure

```
src/ascension/
├── mod.rs      # Public re-exports (can_ascend, ascend, AscendResult, helper functions)
├── types.rs    # Constants (cost table, gate table, multiplier formula), helper functions
└── logic.rs    # Eligibility checks (can_ascend), execution (ascend), AscendResult enum
```

## Key Types

### `AscendResult` (`logic.rs`)

Returned by `ascend()`:
- `Success { new_level, multiplier }` -- PR deducted, level incremented
- `InsufficientPR { needed, have }` -- not enough prestige ranks
- `DeepGateNotMet { needed_layer, current_layer }` -- Deep layer requirement not reached (levels I-VI)
- `PatternGateNotMet { needed_patterns, current_patterns }` -- Woven Pattern requirement not reached (levels VII-X)

## Key Functions

### `types.rs`

- `ascension_cost(level) -> u32` -- PR cost to reach the given level
- `ascension_deep_gate(level) -> Option<u32>` -- Deep layer requirement (None for levels 7+)
- `ascension_pattern_gate(level) -> Option<usize>` -- Woven Pattern requirement for levels 7-10 (None for levels 1-6)
- `ascension_combat_multiplier(level) -> f64` -- Combat stat multiplier at given level
- `max_shuttle_level(ascension_level) -> u32` -- Max Loom Shuttle upgrade level for given Ascension tier (1 for 0-VI, 3/5/7/10 for VII-X)

### `logic.rs`

- `can_ascend(ascension_level, prestige_rank, deepest_layer) -> bool` -- Check eligibility for next level
- `ascend(state, deepest_layer) -> AscendResult` -- Execute Ascension: validate, deduct PR, increment level

## Ascension Table

| Level | Deep Gate | PR Cost | Cumulative Mult |
|-------|-----------|---------|-----------------|
| I | Layer 3 (Shallows) | 35 PR | 2x |
| II | Layer 7 (Warrens) | 65 PR | 4x |
| III | Layer 12 (Hollows) | 120 PR | 8x |
| IV | Layer 18 (Sunken Reach) | 200 PR | 16x |
| V | Layer 25 (Abyss) | 325 PR | 32x |
| VI | Layer 30 (Gateway) | 500 PR | 64x |
| VII | 8 Patterns | 1,500 PR | 96x |
| VIII | 16 Patterns | 4,000 PR | 144x |
| IX | 22 Patterns | 8,000 PR | 216x |
| X | 28 Patterns | 15,000 PR | 324x |

Total PR for I-VI: 1,245 PR.

## Formulas

- **Cost**: Levels 1-6 from lookup table `[35, 65, 120, 200, 325, 500]`; levels 7-10 from `[1500, 4000, 8000, 15000]`
- **Deep gate**: Levels 1-6 from lookup table `[3, 7, 12, 18, 25, 30]`; levels 7+ = no Deep gate
- **Pattern gate**: Levels 7-10 require `[8, 16, 22, 28]` completed Woven Patterns; levels 1-6 = no pattern gate
- **Multiplier**: Levels 1-6 = `2^level`; levels 7+ = `64 * 1.5^(level - 6)`
- **Shuttle level cap**: Asc 0-VI = 1, VII = 3, VIII = 5, IX = 7, X = 10

## Persistence

- `ascension_level: u32` stored on `GameState` (per-character save) with `#[serde(default)]`
- New characters start at level 0 (1.0x multiplier)
- Ascension level survives prestige (PR spent is gone but level remains)

## Integration Points

- **Combat** (`combat/events.rs`): `CombatBonuses::ascension_multiplier` field applied to player damage, defense, and HP in the combat pipeline
- **Core** (`core/tick_stages.rs`): Builds `ascension_multiplier` from `ascension_combat_multiplier(state.ascension_level)` and injects into `CombatBonuses`; applies ascension multiplier to player max HP
- **Core** (`core/tick_types.rs`): `TickEvent::Ascended { level, message }` variant
- **Core** (`core/game_state.rs`): `ascension_level` field on `GameState`
- **Deep** (`deep/types.rs`): Deep layer milestones gate Ascension availability (account-level check)
- **Achievements** (`achievements/handlers.rs`): `on_ascended(level)` unlocks `AscensionI` through `AscensionVI` (one achievement per level)
- **UI** (`ui/stats_prestige.rs`): Shows "Asc N (Mx)" alongside prestige info when level > 0
- **Loom** (`loom/`): `completed_pattern_count()` provides pattern gate checks for VII-X; `max_shuttle_level(ascension_level)` gates shuttle upgrade caps
