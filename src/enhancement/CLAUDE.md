# Enhancement System (Soulforge)

Account-level equipment enhancement that persists across all characters. Each of 7 equipment slots can be enhanced from +0 to +10, boosting equipment stats via a multiplier applied in `derived_stats.rs`.

## Module Structure

```
src/enhancement/
├── mod.rs          # Public re-exports
├── types.rs        # EnhancementProgress, constants, SoulforgePhase, SoulforgeUiState, EnhancementResult
├── logic.rs        # Enhancement rolling, result application, Soulforge discovery
└── persistence.rs  # Save/load from ~/.quest/enhancement.json
```

## Key Types

### `EnhancementProgress` (`types.rs`)
```rust
pub struct EnhancementProgress {
    pub discovered: bool,
    pub levels: [u8; 7],           // Per-slot, 0-10, indexed by EquipmentSlot order
    pub total_attempts: u32,
    pub total_successes: u32,
    pub total_failures: u32,
    pub highest_level_reached: u8,
}
```

Methods:
- `level(slot_index)` -- Get enhancement level for a slot (0-6)
- `set_level(slot_index, level)` -- Set level, clamped to MAX_ENHANCEMENT_LEVEL, updates highest_level_reached

### `SoulforgePhase` (`types.rs`)
UI state machine: `Menu` -> `Confirming` -> `Hammering` -> `ResultSuccess` / `ResultFailure`.

### `SoulforgeUiState` (`types.rs`)
Overlay state with `open`, `selected_slot`, `phase`, `animation_tick`, `last_result: Option<EnhancementResult>`, and `soul_tithe: bool`.

### `EnhancementResult` (`types.rs`)
Display struct: `slot_index`, `success`, `old_level`, `new_level`, `cost`.

## Enhancement Mechanics

### Success Rates and Costs
| Target | Standard | Soul Tithe | Fail Penalty |
|--------|----------|-----------|-------------|
| +1-+4 | 1 PR / 100% | -- | 0 |
| +5 | 2 PR / 70% | 4 PR / 100% | -1 |
| +6 | 3 PR / 55% | 6 PR / 100% | -1 |
| +7 | 3 PR / 40% | 8 PR / 100% | -1 |
| +8 | 4 PR / 30% | 25 PR / 100% | -1 |
| +9 | 4 PR / 20% | 85 PR / 100% | -1 |
| +10 | 5 PR / 10% | 750 PR / 100% | -2 |

### Soul Tithe Mechanic
For +5 through +10, players can choose "Soul Tithe" mode on the confirmation screen (Left/Right arrows to toggle). Soul Tithe pays a higher PR cost for guaranteed 100% success. Not available for +1-+4 (already 100%).

Prices for +8-+10 are ~0.75x the expected PR cost of gambling that step -- attempt fees plus re-buying levels lost to failures via the tithes below -- matching the discount ratio the +5-+7 prices imply. A test (`test_soul_tithe_high_tier_prices_track_expected_gamble_cost`) asserts this relationship holds, so retuning rates/penalties requires repricing the tithes.

### Stat Multiplier Curve
Enhancement multiplier = `1.0 + cumulative_bonus / 100.0`:

| Level | Cumulative Bonus | Multiplier |
|-------|-----------------|------------|
| +0 | 0% | 1.00x |
| +1 | 5% | 1.05x |
| +2 | 10% | 1.10x |
| +3 | 15% | 1.15x |
| +4 | 20% | 1.20x |
| +5 | 30% | 1.30x |
| +6 | 40% | 1.40x |
| +7 | 55% | 1.55x |
| +8 | 75% | 1.75x |
| +9 | 100% | 2.00x |
| +10 | 150% | 2.50x |

### Color Tiers (for UI display)
- +0: DarkGray (128,128,128)
- +1-4: White (255,255,255)
- +5-7: Yellow (255,255,0)
- +8-9: Magenta (255,0,255)
- +10: Gold (255,215,0)

## Discovery

Soulforge is discovered randomly at P15+. Discovery chance per tick:

```
chance = 0.000014 + (prestige_rank - 15) * 0.000007
```

Blocked when dungeon, fishing, or minigame is active. Once discovered, `enhancement.discovered = true` persists permanently.

## Key Functions

### logic.rs
- `roll_enhancement(current_level, rng) -> (success, new_level)` -- Roll outcome without modifying state
- `apply_enhancement_result(enhancement, slot_index, new_level, success)` -- Apply pre-rolled result, update counters
- `soulforge_discovery_chance(prestige_rank) -> f64` -- Calculate discovery probability
- `try_discover_soulforge(enhancement, prestige_rank, rng) -> bool` -- Roll for discovery

### types.rs (helper functions)
- `success_rate(target_level) -> f64` -- Lookup success rate for target level
- `enhancement_cost(target_level) -> u32` -- Lookup standard PR cost for target level
- `soul_tithe_cost(target_level) -> Option<u32>` -- Lookup soul tithe PR cost (Some for +5 through +10, None otherwise)
- `fail_penalty(target_level) -> u8` -- Lookup failure penalty for target level
- `enhancement_multiplier(level) -> f64` -- Calculate stat multiplier for current level
- `enhancement_prefix(level) -> String` -- Format display prefix (e.g., "+5 " or "")
- `enhancement_color_tier(level) -> u8` -- Color tier (0-4) for UI rendering
- `enhancement_color_rgb(level) -> (u8, u8, u8)` -- RGB color for UI rendering

## Persistence

- **File**: `~/.quest/enhancement.json` (pretty-printed JSON via serde)
- **Load**: `load_enhancement()` returns `EnhancementProgress::new()` if file is missing or corrupted
- **Save**: `save_enhancement()` creates the `~/.quest/` directory if needed
- **Trigger**: `main.rs` saves whenever `TickResult::enhancement_changed` is true

## Integration Points

- **character/derived_stats.rs**: `DerivedStats::calculate_derived_stats()` takes `enhancement_levels: &[u8; 7]` and applies `enhancement_multiplier()` to scale equipment attribute bonuses and affix values per slot
- **core/tick.rs**: Stage 3 passes `enhancement.levels` to `calculate_derived_stats()`. Stage 11 rolls for Soulforge discovery (`try_discover_soulforge()`), emits `TickEvent::SoulforgeDiscovered`, and sets `enhancement_changed` flag
- **core/tick.rs**: `game_tick()` takes `enhancement: &mut EnhancementProgress` as a parameter
- **main.rs**: Loads/saves enhancement state. Handles Soulforge input routing and overlay management via `SoulforgeUiState`
- **achievements/handlers.rs**: `on_soulforge_discovered()` and `on_enhancement_upgraded()` track Soulforge-related milestones
- **ui/soulforge_scene.rs**: Soulforge overlay rendering (slot list, enhancement animation, result display)

## Constants (`types.rs`)

| Constant | Value | Notes |
|----------|-------|-------|
| `MAX_ENHANCEMENT_LEVEL` | 10 | Hard cap per slot |
| `SOULFORGE_MIN_PRESTIGE_RANK` | 15 | Discovery requires P15+ |
| `SOULFORGE_DISCOVERY_BASE_CHANCE` | 0.000014 | Per tick |
| `SOULFORGE_DISCOVERY_RANK_BONUS` | 0.000007 | Per rank above 15 |
| `ENHANCEMENT_SUCCESS_RATES` | [1.0, 1.0, 1.0, 1.0, 0.7, 0.55, 0.4, 0.3, 0.2, 0.1] | Indexed by target level - 1 |
| `ENHANCEMENT_COSTS` | [1, 1, 1, 1, 2, 3, 3, 4, 4, 5] | Standard PR cost per target level |
| `ENHANCEMENT_SOUL_TITHE_COSTS` | [None×4, Some(4), Some(6), Some(8), Some(25), Some(85), Some(750)] | Soul Tithe PR cost (100% success) |
| `ENHANCEMENT_FAIL_PENALTY` | [0, 0, 0, 0, 1, 1, 1, 1, 1, 2] | Level loss on failure |
| `ENHANCEMENT_CUMULATIVE_BONUS` | [0, 5, 10, 15, 20, 30, 40, 55, 75, 100, 150] | % bonus at each level |
