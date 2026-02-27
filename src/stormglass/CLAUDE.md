# Stormglass System

Character-level currency earned through gameplay and spent at the Stormglass Exchange overlay. Stormglass persists across prestiges but is per-character (not account-level).

## Module Structure

```
src/stormglass/
├── mod.rs       # Public re-exports
├── types.rs     # Constants, ExchangePhase, ExchangeUiState, ChronoSurgeState
├── sigils.rs    # SigilEffectType (11 types), Sigil, SigilGrade, StormSigils, daily rotation
├── earning.rs   # Salvage rates by rarity, dungeon cache sizes, soulforge consolation
└── spending.rs  # Invoke Challenge generation, chrono surge costs, Storm Lure
```

## Key Concepts

### Stormglass Currency
Earned passively through gameplay:
- **Item salvage**: Non-equipped item drops are auto-salvaged (Common/Magic 1, Rare 3, Epic 8, Legendary 25)
- **Dungeon treasure caches**: Found in dungeon treasure rooms (Small 5, Medium 15, Large 30, Epic 50, Legendary 75)
- **Soulforge consolation**: Failed enhancements at +5 and above award Stormglass (5-75 depending on target level)
- **Challenge rewards**: All challenges award Stormglass scaled by difficulty

### Stormglass Exchange
Overlay with four spending options:
1. **Invoke Challenge** (3,000 SG): Presents 3 random challenge types to choose from, bypassing normal discovery
2. **Chrono Surge** (500-16,000 SG): Fast-forwards game ticks with animated summary (15 min to 8 hours of gameplay)
3. **Storm Sigils**: Unlock slots, etch sigils, and reroll for persistent bonuses
4. **Storm Lure** (50,000 SG): Consumable that guarantees Storm Leviathan encounters on legendary fish catches at rank 40. Requires fishing rank 40 and no active lure

### Storm Sigils
Up to 5 sigil slots that provide permanent percentage-based bonuses. Character-level, persists through prestige.

**Slot unlock costs**: 25k, 50k, 100k, 200k, 400k Stormglass (exponential 2x curve)

**Etch/Reroll cost**: 25,000 Stormglass per attempt

**Daily rotation**: Each day, 5 of 11 effect types are available (deterministic via seeded RNG from calendar day). Players choose from these when etching or rerolling.

**Pick-1-of-3**: Each etch/reroll generates 3 random sigils from the daily pool. The player picks one.

### Sigil Effect Types (11 total)
| Effect | Range | Sigil Name |
|--------|-------|------------|
| XpPercent | 5-25% | Sigil of Wisdom |
| DamagePercent | 3-15% | Sigil of Fury |
| DamageReductionPercent | 1-5% | Sigil of the Bulwark |
| CritChancePercent | 2-8% | Sigil of Precision |
| DropRatePercent | 2-10% | Sigil of Fortune |
| MaxHpPercent | 3-15% | Sigil of Vitality |
| FishingSpeedPercent | 5-25% | Sigil of the Tide |
| OfflineXpPercent | 5-20% | Sigil of Echoes |
| AttackSpeedPercent | 2-10% | Sigil of Swiftness |
| DoubleStrikePercent | 1-5% | Sigil of the Twin Strike |
| RegenDelayPercent | 2-10% | Sigil of Renewal |

### Sigil Grades
Values are rolled on an exponential curve (`e^(3p) - 1`) that compresses low rolls and stretches high rolls. Grades are assigned by percentile (F- through S+), with 21 total grades across 7 letter tiers (F, E, D, C, B, A, S).

### SigilBonuses Aggregation
`SigilBonuses::compute(&StormSigils)` sums all etched sigil values by effect type into a single struct. These bonuses are injected into `CombatBonuses` (for combat effects) and other systems (fishing speed, offline XP, drop rate) via explicit parameter passing.

## Key Types

### `ExchangePhase` (`types.rs`)
UI state machine for the Exchange overlay: `Menu`, `InvokeTrialConfirm`, `InvokeTrial`, `ChronoSurge`, `SigilsList`, `SigilUnlockConfirm`, `SigilEtchConfirm`, `SigilRerollConfirm`, `SigilRolling`, `SigilPick`, `SigilResult`, and forfeit phases.

### `ExchangeUiState` (`types.rs`)
Overlay state with `open`, `selected_item`, `phase`, trial options, surge selection, and sigil UI state (selected slot, choices, pick selection, result, animation).

### `ChronoSurgeState` (`types.rs`)
Tracks an active surge: `ticks_remaining`, `ticks_total`, `batch_size` (computed for 10-second animation), `kills`, `levels_gained`, `items_equipped`.

### `StormSigils` (`sigils.rs`)
Character-level storage: `slots_unlocked` (0-5) and `sigils: Vec<Option<Sigil>>` (5 slots). Serialized via serde for persistence.

### `Sigil` (`sigils.rs`)
A single etched sigil: `effect: SigilEffectType`, `value: f64`, `grade: SigilGrade`.

## Constants

| Constant | Value | Notes |
|----------|-------|-------|
| `MAX_SIGIL_SLOTS` | 5 | Maximum sigil slots |
| `ETCH_COST` | 25,000 | SG cost to etch or reroll |
| `DAILY_POOL_SIZE` | 5 | Effect types available per day |
| `SLOT_UNLOCK_COSTS` | [25k, 50k, 100k, 200k, 400k] | Exponential 2x curve |
| `INVOKE_TRIAL_COST` | 3,000 | SG cost to invoke a challenge |
| `CHRONO_SURGE_OPTIONS` | 4 tiers | 500-16,000 SG for 15min-8hr |
| `SALVAGE_COMMON/MAGIC` | 1 | Lowest salvage value |
| `SALVAGE_LEGENDARY` | 25 | Highest salvage value |
| `STORM_LURE_COST` | 50,000 | SG cost for Storm Lure consumable |

## Integration Points

- **core/tick.rs**: Emits `StormglassDiscovered`, `StormglassSalvaged`, `StormglassDungeonCache` tick events during salvage and dungeon processing
- **core/tick.rs**: Sigil bonuses are computed via `SigilBonuses::compute()` and injected into the unified `CombatBonuses` struct each tick
- **combat/events.rs**: `CombatBonuses` includes sigil damage%, crit%, attack speed%, DR%, double strike%, and regen fields
- **challenges/menu.rs**: `ChallengeReward` struct includes `stormglass` field; all challenges award Stormglass
- **enhancement/logic.rs**: Failed enhancements award Stormglass consolation via `soulforge_consolation()`
- **ui/stormglass_scene.rs**: Exchange overlay rendering (menu, trial selection, surge animation, sigils list, pick-1-of-3)
- **input/stormglass_input.rs**: Exchange overlay input handling
- **fishing/types.rs**: `FishingState.storm_lure_active` flag, consumed on Leviathan encounter
- **spending.rs**: `can_purchase_storm_lure()` checks balance, active status, and fishing rank
