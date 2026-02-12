# Fishing System

Separate progression track where players discover fishing spots, catch fish for XP, find item drops, rank up through 40 tiers, and hunt the Storm Leviathan.

## Module Structure

```
src/fishing/
├── mod.rs         # Public re-exports
├── types.rs       # FishRarity, FishingPhase, FishingSession, FishingState, rank names/thresholds
├── generation.rs  # Rarity rolling, fish/session generation, Storm Leviathan encounter logic
└── logic.rs       # Tick processing, discovery, rank-ups, item drops, Haven bonus integration
```

## Key Types

### `FishRarity` (`types.rs`)
Five tiers matching item rarity: Common, Uncommon, Rare, Epic, Legendary. Ordered enum (0-4) used for drop chances and XP scaling.

### `FishingPhase` (`types.rs`)
State machine for one fish catch cycle:
- **Casting** (0.5-1.5s): Line being cast
- **Waiting** (1-8s): Waiting for a bite
- **Reeling** (0.5-3s): Fish on the line, reeling in

### `FishingSession` (`types.rs`)
Active fishing session at a discovered spot:
```rust
pub struct FishingSession {
    pub spot_name: String,           // One of 8 spot names
    pub total_fish: u32,             // 3-8 fish per session
    pub fish_caught: Vec<CaughtFish>,
    pub items_found: Vec<Item>,
    pub ticks_remaining: u32,        // Countdown for current phase
    pub phase: FishingPhase,
}
```

### `FishingState` (`types.rs`)
Persistent state saved with character:
```rust
pub struct FishingState {
    pub rank: u32,                   // 1-40
    pub total_fish_caught: u32,
    pub fish_toward_next_rank: u32,  // Excess carries over on rank-up
    pub legendary_catches: u32,
    pub leviathan_encounters: u8,    // 0-10 progressive hunt
}
```

### `HavenFishingBonuses` (`logic.rs`)
Haven bonuses passed as explicit parameter (not global):
```rust
pub struct HavenFishingBonuses {
    pub timer_reduction_percent: f64,     // Garden: reduces cast/wait/reel time
    pub double_fish_chance_percent: f64,  // Fishing Dock: chance to catch 2 fish
    pub max_fishing_rank_bonus: u32,      // Fishing Dock T4: +10 max rank
}
```

### `LeviathanResult` (`generation.rs`)
Result of Storm Leviathan roll: `None`, `Escaped { encounter_number }`, or `Caught`.

### `FishingTickResult` (`logic.rs`)
Return value from tick processing with messages, `caught_storm_leviathan` flag, and optional `leviathan_encounter` number.

## Rank System

40 ranks across 8 tiers with increasing fish-per-rank requirements:

| Tier | Ranks | Fish/Rank | Total Fish | Notes |
|------|-------|-----------|------------|-------|
| Novice | 1-5 | 100 | 500 | |
| Apprentice | 6-10 | 200 | 1,000 | |
| Journeyman | 11-15 | 400 | 2,000 | |
| Expert | 16-20 | 800 | 4,000 | |
| Master | 21-25 | 1,500 | 7,500 | |
| Grandmaster | 26-30 | 2,000 | 10,000 | Base max without Haven |
| Mythic | 31-35 | 4k-25k | 61,000 | Requires Fishing Dock T4 |
| Transcendent | 36-40 | 40k-250k | 615,000 | Storm Leviathan at rank 40 |

Cumulative: 25,000 fish to rank 30; 701,000 fish to rank 40.

## Rarity System

Base catch chances (adjusted by rank):
- Common: 60%, Uncommon: 25%, Rare: 10%, Epic: 4%, Legendary: 1%
- Every 5 ranks: -2% Common, +1% Uncommon, +0.5% Rare, +0.3% Epic, +0.2% Legendary
- Common floor: 10% minimum

XP rewards by rarity:
- Common: 50-100, Uncommon: 150-250, Rare: 400-600, Epic: 1,000-1,500, Legendary: 3,000-5,000

## Item Drops from Fishing

Drop chance by fish rarity (checked per catch):
- Common/Uncommon: 5%, Rare: 15%, Epic: 35%, Legendary: 75%

Fish rarity maps to item rarity: Common->Common, Uncommon->Magic, Rare->Rare, Epic->Epic, Legendary->Legendary. Item level based on current zone (`ilvl_for_zone(zone_id)`).

## Fishing Tick Flow

`tick_fishing_with_haven_result()` drives per-tick processing:

1. Take ownership of `state.active_fishing` (early return if None)
2. Decrement `ticks_remaining`
3. On timer reaching 0, process phase transition:
   - **Casting -> Waiting**: Roll waiting ticks (with Haven timer reduction)
   - **Waiting -> Reeling**: Roll reeling ticks (with Haven timer reduction)
   - **Reeling -> Catch**: Roll rarity, generate fish (with Leviathan check), award XP (with prestige multiplier), check item drop, check double fish (Haven), add to session. If all fish caught, end session. Otherwise, start Casting again.
4. Put session back into `state.active_fishing`

## Storm Leviathan Hunt

Progressive 10-encounter hunt, only available at rank 40 on legendary fish catches:

1. Each legendary catch at rank 40+ rolls against decreasing encounter chances:
   - Encounters 1-10: 8%, 6%, 5%, 4%, 3%, 2%, 1.5%, 1%, 0.5%, 0.25%
2. On encounter: Leviathan escapes, encounter counter increments
3. After 10 encounters: 25% catch chance per legendary fish
4. Catching awards 10,000-15,000 XP and enables Stormbreaker forging at Storm Forge

**Full path**: Max fishing rank (40) -> catch legendary fish -> 10 Leviathan encounters -> catch Leviathan -> build Storm Forge in Haven -> forge Stormbreaker -> fight Zone 10 final boss.

## Key Functions

### generation.rs
- `roll_fish_rarity(rank, rng) -> FishRarity` -- Rank-adjusted rarity roll
- `generate_fish(rarity, rng) -> CaughtFish` -- Random fish name + XP for rarity
- `generate_fish_with_rank(rarity, rank, leviathan_encounters, rng) -> (CaughtFish, LeviathanResult)` -- Fish generation with Leviathan hunt logic
- `is_storm_leviathan(fish) -> bool` -- Check if catch is the Storm Leviathan
- `generate_fishing_session(rng) -> FishingSession` -- New session (random spot, 3-8 fish, Casting phase)
- `roll_casting_ticks(rng)`, `roll_waiting_ticks(rng)`, `roll_reeling_ticks(rng)` -- Phase duration rolls

### logic.rs
- `tick_fishing_with_haven_result(state, rng, haven) -> FishingTickResult` -- Main tick processor (preferred)
- `tick_fishing_with_haven(state, rng, haven) -> Vec<String>` -- Returns messages only
- `tick_fishing(state, rng) -> Vec<String>` -- Legacy wrapper (no Haven bonuses)
- `try_discover_fishing(state, rng) -> Option<String>` -- 5% chance to discover a spot (blocked by active fishing/dungeon)
- `get_max_fishing_rank(fishing_rank_bonus) -> u32` -- Base 30 + bonus, capped at 40
- `check_rank_up_with_max(fishing_state, max_rank) -> Option<String>` -- Rank-up check with configurable cap
- `check_rank_up(fishing_state) -> Option<String>` -- Legacy rank-up (cap 30)

## Integration Points

- **Core** (`core/tick.rs`): Calls `tick_fishing_with_haven_result()` each tick, handles discovery via `try_discover_fishing()`, processes rank-ups and Leviathan events
- **Core** (`core/constants.rs`): `FISHING_DISCOVERY_CHANCE` (0.05), `BASE_MAX_FISHING_RANK` (30), `MAX_FISHING_RANK` (40)
- **Core** (`core/game_state.rs`): Owns `active_fishing: Option<FishingSession>` and `fishing: FishingState`
- **Character** (`character/prestige.rs`): Prestige multiplier applied to fish XP rewards
- **Items** (`items/generation.rs`, `items/drops.rs`): Item generation for fishing item drops
- **Haven** (`haven/types.rs`): Garden (timer reduction), Fishing Dock (double fish, max rank bonus), Storm Forge (Stormbreaker)
- **Achievements**: Tracks fishing rank milestones, legendary catches, Leviathan catch
- **UI** (`ui/fishing_scene.rs`): Fishing session display with phase indicator
- **Debug** (`utils/debug_menu.rs`): Can trigger fishing sessions for testing

## Extending the Fishing System

**Adding a new fish rarity**: Update `FishRarity` enum, add name array in `generation.rs`, add XP range to `XP_REWARDS`, add base chance to `BASE_CHANCES` and rank bonus to `RANK_BONUS_PER_5`, add drop chance constant and mapping in `logic.rs`.

**Adding a new Haven bonus**: Add field to `HavenFishingBonuses`, wire it in `tick_fishing_with_haven_result()`, define the room/upgrade in `haven/types.rs`.

**Adding a new fishing phase**: Add variant to `FishingPhase`, add timing constants, handle transitions in `tick_fishing_with_haven_result()` match block.
