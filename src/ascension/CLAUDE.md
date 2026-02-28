# Ascension System

Per-character combat power multiplier purchased with prestige ranks, gated by Deep layer milestones. Each Ascension level doubles all combat stats (damage, defense, HP) for levels I-VI, with diminishing returns at VII+.

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
- `DeepGateNotMet { needed_layer, current_layer }` -- Deep layer requirement not reached

## Key Functions

### `types.rs`

- `ascension_cost(level) -> u32` -- PR cost to reach the given level
- `ascension_deep_gate(level) -> Option<u32>` -- Deep layer requirement (None for levels 7+)
- `ascension_combat_multiplier(level) -> f64` -- Combat stat multiplier at given level

### `logic.rs`

- `can_ascend(ascension_level, prestige_rank, deepest_layer) -> bool` -- Check eligibility for next level
- `ascend(state, deepest_layer) -> AscendResult` -- Execute Ascension: validate, deduct PR, increment level

## Ascension Table

| Level | Deep Gate | PR Cost | Cumulative Mult |
|-------|-----------|---------|-----------------|
| I | Layer 3 (Shallows) | 10 PR | 2x |
| II | Layer 7 (Warrens) | 15 PR | 4x |
| III | Layer 12 (Hollows) | 25 PR | 8x |
| IV | Layer 18 (Sunken Reach) | 35 PR | 16x |
| V | Layer 25 (Abyss) | 50 PR | 32x |
| VI | Layer 30 (Gateway) | 65 PR | 64x |
| VII+ | None (PR only) | 80+ PR (+15 each) | x1.5 each |

Total PR for I-VI: 200 PR.

## Formulas

- **Cost**: Levels 1-6 from lookup table `[10, 15, 25, 35, 50, 65]`; levels 7+ = `65 + 15 * (level - 6)`
- **Deep gate**: Levels 1-6 from lookup table `[3, 7, 12, 18, 25, 30]`; levels 7+ = no gate
- **Multiplier**: Levels 1-6 = `2^level`; levels 7+ = `64 * 1.5^(level - 6)`

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
- **Achievements** (`achievements/handlers.rs`): `on_ascended(level)` unlocks `AscensionI` (I), `AscensionIII` (III), `AscensionVI` (VI)
- **UI** (`ui/stats_prestige.rs`): Shows "Asc N (Mx)" alongside prestige info when level > 0
