# The Deep — Mercenary Expedition System

An endgame (P15+) system where players recruit and manage a mercenary company, sending squads on long-duration missions (2-24 hours, real-time wall-clock time) into a vast underground structure. Each prestige cycle represents a new generation of mercenaries inheriting the infrastructure, maps, and outposts left behind.

**Key differentiators from existing systems:**
- **Timescale**: Hours/days vs. seconds/minutes — a fundamentally different engagement rhythm
- **Generational theme**: Each prestige sends a new generation deeper, standing on the shoulders of those before
- **Optional engagement**: Check-in events reward attention but never punish absence (auto-resolve always picks the safe path)
- **Wall-clock time**: Missions progress in real time, including while the game is closed

## Module Structure

```
src/deep/
├── mod.rs          — Public API re-exports
├── types.rs        — All data structures, enums, constants, and helper methods
├── mercenaries.rs  — Merc generation, recruit pools, leveling, injuries, roster management
├── layers.rs       — Layer difficulty, familiarity system, infrastructure, mission durations
├── discovery.rs    — Discovery roll logic (try_discover_deep), starter roster init
└── persistence.rs  — Save/load from ~/.quest/deep.json
```

## Persistence Model

Two-tier persistence, mirroring the Haven and Soulforge pattern:

| Tier | Struct | Saved to | Survives prestige? |
|------|--------|----------|--------------------|
| Account-level | `DeepPersistent` | `~/.quest/deep.json` | Yes |
| Per-prestige | `DeepPrestige` | character save | No |

Both tiers are combined in `DeepState` for convenience. Call `DeepState::on_prestige()` to reset transient state while preserving account-level progress.

### Persists Across Prestiges
- `discovered` flag
- `guild_rank` (1-5)
- `deepest_layer_reached`
- Layer records: `familiarity` (0-100 intel), `infrastructure`, `cleared` status
- `merc_id_counter` and `mission_id_counter` (monotonically increasing across all prestiges)

### Resets on Prestige
- Mercenary roster (each generation starts fresh)
- Active and completed missions
- Available mission pool
- `warband_marks` currency balance
- Recruit pool

## Key Types

### `DeepState` (`types.rs`)
Top-level container. `persistent` is saved to `deep.json`; `prestige` is saved with the character.

```rust
pub struct DeepState {
    pub persistent: DeepPersistent,
    pub prestige: DeepPrestige,
}
```

Key methods:
- `on_prestige()` — Resets `prestige`, keeps `persistent` intact
- `is_active()` — Returns `persistent.discovered`

### `DeepPersistent` (`types.rs`)
Account-level state. Owned by the game's account-level save, not character saves.

Key methods:
- `layer_record_mut(index)` — Get or lazily create a `LayerRecord` (vec grows on demand)
- `layer_record(index)` — Read-only lookup; returns `None` for index 0 or unreached layers
- `frontier_layer()` — The deepest uncleared layer, or `deepest_layer_reached + 1` when all are cleared
- `next_merc_id()` / `next_mission_id()` — Monotonically increasing id assignment

### `DeepPrestige` (`types.rs`)
Per-prestige state. Resets fully on each prestige.

Key methods:
- `find_merc(id)` / `find_merc_mut(id)` — Lookup by `u64` id
- `find_mission_mut(id)` — Lookup active mission by id
- `available_merc_count()` — Count of `MercStatus::Available` mercs
- `active_mission_count()` — Count of `Active | EventPending` missions
- `has_any_pending_event()` — Whether any mission needs player response
- `spend_marks(amount)` — Deduct Warband Marks; returns false if insufficient

### `GuildRank` (`types.rs`)
Newtype `GuildRank(u8)` with values 1-5. Persists across prestiges.

| Rank | Name | Max Roster | Concurrent Missions | Required Breakthrough |
|------|------|-----------|---------------------|----------------------|
| 1 | Freelancers | 5 | 1 | Discovery |
| 2 | Sellswords | 7 | 1 | Layer 3 |
| 3 | Company | 9 | 2 | Layer 7 |
| 4 | Battalion | 12 | 3 | Layer 13 |
| 5 | Legion | 15 | 4 | Layer 19 |

Stats are driven from the `GUILD_RANK_STATS` constant array (index 0 = Rank 1). Do not call methods with `GuildRank(0)` — valid range is 1-5.

### `Mercenary` (`types.rs`)
Individual merc in the roster. Resets on prestige. `id` is a `u64` assigned via `DeepPersistent::next_merc_id()` and is unique across all generations.

Key methods:
- `is_available()` — True when `status == MercStatus::Available`
- `effective_power()` — Base power + `(level - 1) * 3`
- `effective_resilience()` — Base resilience + `(level - 1) * 2`
- `missions_to_next_level(level)` — `3 + level * 2`

### `MercArchetype` (`types.rs`)
Five archetypes, each with a distinct `base_stats()` tuple `(power, resilience, expertise)`:

| Archetype | Power | Resilience | Expertise | Role |
|-----------|-------|-----------|-----------|------|
| Vanguard | 14 | 12 | 4 | Frontline tank |
| Scout | 8 | 10 | 12 | Recon specialist |
| Arcanist | 10 | 6 | 14 | Elemental specialist |
| Medic | 6 | 14 | 10 | Healer |
| Saboteur | 10 | 8 | 12 | Trap/obstacle specialist |

Archetype availability by guild rank: Rank 1 = Vanguard/Scout/Medic, Rank 2 adds Arcanist, Rank 3+ adds Saboteur.

### `MercStatus` (`types.rs`)
```rust
pub enum MercStatus {
    Available,
    OnMission(u64),          // holds mission id
    Injured { missions_remaining: u32 },
    Lost,
}
```
Lost mercs are kept in roster for death notification display, then removed via `purge_lost_mercs()`.

### `LayerTier` (`types.rs`)
Computed from 1-based layer index via `LayerTier::from_layer(layer)`:

| Tier | Layers | Name |
|------|--------|------|
| Shallows | 1-3 | The Shallows |
| Warrens | 4-7 | The Warrens |
| Hollows | 8-12 | The Hollows |
| SunkenReach | 13-18 | The Sunken Reach |
| Abyss | 19-25 | The Abyss |
| Void | 26+ | The Void |

### `Layer` / `LayerRecord` (`types.rs`)
`LayerRecord` is the **persistent** per-layer record stored in `DeepPersistent::layers`. `Layer` is the **runtime view** combining `LayerRecord` data with computed fields like `theme_name` and `difficulty`.

`Layer` key methods:
- `total_duration_reduction()` — Sum of infrastructure duration reductions, capped at 0.75 (75%)
- `has_infrastructure(infra)` — Whether a given `Infrastructure` type is built
- `available_infrastructure_slots()` — How many more can be built (max 4 total)

### `Infrastructure` (`types.rs`)
Four types, each buildable once per layer (persists across prestiges):

| Type | Effect | Build Cost |
|------|--------|-----------|
| Outpost | -25% mission duration on this layer | 60 + 4*layer |
| SupplyCache | Supply runs yield bonus resources | 80 + 5*layer |
| Watchtower | Better intel, auto-resolve, +25 familiarity on build | 70 + 4*layer |
| Bridge | Shortcut — missions can skip this layer | 100 + 5*layer |

### `MissionType` (`types.rs`)
```rust
pub enum MissionType {
    SupplyRun,               // 2-4h, no risk, cleared layers
    Recon,                   // 4-8h, low risk, frontier
    Expedition,              // 8-16h, medium risk, frontier
    Breakthrough,            // 18-24h, high risk, frontier (once per layer)
    Construction(Infrastructure), // 4-8h, no risk, cleared layers
}
```

Note: `Construction` carries the infrastructure type being built as a payload.

### `Mission` (`types.rs`)
An active mission in `DeepPrestige::active_missions`. Uses wall-clock `DateTime<Utc>` for `started_at` / `ends_at`.

Key methods:
- `progress(now)` — Fraction elapsed 0.0-1.0
- `is_time_elapsed(now)` — Whether `now >= ends_at`
- `has_pending_event()` — Whether `status == EventPending`
- `unresolved_event_count()` — Count of unresolved `CheckInEvent`s

### `CheckInEvent` (`types.rs`)
Fires at scheduled points during missions. Auto-resolve uses `auto_resolve_choice` (always the safe option) when the player doesn't respond.

Key methods:
- `effective_choice()` — Player's choice or auto-resolve fallback
- `is_resolved()` — `resolved_choice.is_some()`

### `MissionResult` (`types.rs`)
Populated when a mission completes. Carries: `outcome`, `marks_earned`, `xp_earned`, `stormglass_earned`, optional `item_ilvl`, `prestige_fragment`, and lists of `injured_mercs` / `lost_mercs` / `merc_level_ups`.

### `DeepUiState` (`types.rs`)
Not serialized — pure runtime UI state. Manages which `DeepView` is shown and selection indices.

```rust
pub enum DeepView { Hub, NewMission, Roster, Infrastructure, EventResponse, Recruit }
```

## Key Functions

### `discovery.rs`
- `try_discover_deep(deep, prestige_rank, rng) -> bool` — Roll for discovery, initialise 3 starter mercs (Vanguard, Scout, Medic) on success

### `mercenaries.rs`

**Generation:**
- `generate_mercenary(id, archetype, quality, rng) -> Mercenary` — Create a merc with quality bonuses and +/-10% stat variance
- `generate_recruit_pool(guild_rank, next_id, rng) -> RecruitPool` — Generate 3-5 daily recruit candidates (size scales with rank)
- `generate_starter_roster(guild_rank, next_id, rng) -> Vec<Mercenary>` — 3 Common-quality starter mercs
- `generate_merc_name(archetype, rng) -> String` — Thematic name from 40 first names x 10 archetype-specific epithets
- `roll_recruit_quality(guild_rank, rng) -> MercQuality` — Quality distribution shifts with rank (Rank 1: 100% Common, Rank 5: 0% Common, 30% Elite)
- `roll_recruit_cost(quality, rng) -> u32` — Warband Marks cost, rounded to nearest 5

**Leveling:**
- `stats_at_level(archetype, base_p, base_r, base_e, level) -> (u32, u32, u32)` — Level-scaled stats using per-archetype growth rates
- `xp_to_next_level(level) -> u32` — `200 * level^1.3` XP curve
- `apply_merc_xp(merc, levels_gained) -> u32` — Apply level-ups, scale stats proportionally preserving quality variance

**Injuries:**
- `injure_merc(merc, severity, rng)` — Set injury status with recovery countdown (Light: 4-8h, Moderate: 8-12h, Severe: 12-16h)
- `mark_merc_lost(merc)` — Mark as permanently lost
- `tick_merc_injury(merc) -> bool` — Decrement injury counter, return true if recovered

**Roster:**
- `roster_has_capacity(roster, guild_rank) -> bool` — Whether roster is below max
- `available_mercs(roster) -> Vec<&Mercenary>` — Filter to Available status
- `purge_lost_mercs(roster) -> u32` — Remove acknowledged lost mercs

### `layers.rs`

**Difficulty:**
- `layer_power_thresholds(layer) -> LayerPowerThresholds` — Power requirements by layer (lookup table L1-25, linear scaling L26+)
- `mission_power_threshold(layer, mission_type) -> u32` — Convenience wrapper selecting the right threshold for a mission type

**Durations:**
- `base_mission_duration_secs(tier, mission_type) -> u64` — Base duration before modifiers (2h Supply Run to 24h Breakthrough)
- `apply_duration_modifiers(base_secs, mods) -> u64` — Full pipeline: Outpost (-25%) * Familiarity (-10/-20/-30%) * Saboteur (-10/-15%) * Overpower (-10%), clamped to 30min floor

**Familiarity:**
- `familiarity_gain(mission_type) -> u8` — Per-mission gain (Supply Run: 5, Recon: 15, Expedition: 10, Breakthrough: 15, Construction: 5)
- `apply_familiarity_gain(record, mission_type)` — Apply gain capped at 100
- `FamiliarityLevel::from_familiarity(pct)` — Unknown (0-24%), Mapped (25-49%), Familiar (50-74%), Mastered (75-100%)

**Infrastructure:**
- `infrastructure_build_cost(infra, layer) -> u32` — Warband Marks cost (scales with depth)
- `build_infrastructure(record, infra) -> Result<(), InfrastructureBuildError>` — Validate cleared + not duplicate, apply. Watchtower grants +25 familiarity on build.
- `mark_layer_cleared(persistent, layer)` — Set cleared flag and update deepest reached
- `is_frontier_layer(persistent, layer) -> bool` / `is_safe_layer(persistent, layer) -> bool` — Layer state queries

### `persistence.rs`
- `load_deep() -> DeepState` — Read from `~/.quest/deep.json`, return default on error
- `save_deep(deep) -> io::Result<()>` — Write pretty-printed JSON

## Discovery

The Deep is discovered at P15+ via a per-tick random roll, identical in pattern to Haven (P10+) and Soulforge (P15+):

```
chance_per_tick = 0.000014 + (prestige_rank - 15) * 0.000007
```

At P15 this gives ~0.000014/tick. With 10 ticks/sec, average discovery time is ~2 hours.

On discovery, `try_discover_deep()` sets `deep.persistent.discovered = true` and creates 3 starter mercs (Vanguard "Gareth", Scout "Lyra", Medic "Aldric"). The discovery roll is blocked when dungeon, fishing, or minigame is active.

## Wall-Clock Time Model

Missions use `DateTime<Utc>` from chrono. On game load, offline resolution processes elapsed time for all active missions, resolving completions and auto-resolving missed events — the same pattern as offline XP progression in `core/offline.rs`.

The game tick does **not** simulate mission progress. It only checks for pending check-in events (a simple timestamp comparison against `Utc::now()`). Completed missions are resolved on load or when the player opens The Deep overlay.

## Integration Points

- **`core/tick.rs`**: `game_tick()` takes `deep: &mut DeepState`. Stage 11b rolls for Deep discovery (`try_discover_deep()`), emits `TickEvent::DeepDiscovered`, sets `deep_changed` flag. Stage 13 checks for pending check-in events.
- **`core/tick_types.rs`**: `TickEvent::DeepDiscovered` variant, `TickResult::deep_changed` and `deep_event_ready` flags
- **`tick_events.rs`**: `TickFlags::deep_discovered` field, combat log message on discovery
- **`input/mod.rs`**: `[D]` keybind opens overlay when `deep.discovered`. Routes to `handle_deep()`.
- **`input/deep_input.rs`**: Deep overlay input handler (navigation, mission selection, event response)
- **`input/types.rs`**: `GameOverlay::DeepDiscovery` and `GameOverlay::DeepOverlay` variants
- **`input/prestige_input.rs`**: Calls `deep.on_prestige()` after `perform_prestige()`
- **`main.rs`**: Loads/saves Deep state alongside Haven and Enhancement. Passes `&mut deep` to `game_tick()` and `save_all()`
- **`main_helpers/persistence.rs`**: `save_all()` includes `deep` parameter, calls `save_deep()` when discovered
- **`main_helpers/offline.rs`**: `resolve_deep_offline()` completes missions that finished while game was closed
- **`ui/deep_scene.rs`**: Deep overlay rendering (Hub, NewMission, Roster, Infrastructure, EventResponse, Recruit views)
- **`ui/stats_panel.rs`**: Pending event indicator when `deep.prestige.has_any_pending_event()`
- **`achievements/`**: Deep-related achievements (discovery, layer milestones, guild ranks)
- **`items/types.rs`**: Abyssal affix types (`AbyssalMissionSpeed`, `AbyssalSupplyYield`, `AbyssalResilience`)
- **`core/game_state.rs`**: `deep_prestige_fragments: f64` for fractional PR from Breakthrough missions

## Constants and Balance Reference

### Discovery
| Constant | Value | Notes |
|----------|-------|-------|
| `DEEP_MIN_PRESTIGE_RANK` | 15 | Same gate as Soulforge and Stormglass |
| `DEEP_DISCOVERY_BASE_CHANCE` | 0.000014 | Per-tick chance at exactly P15 |
| `DEEP_DISCOVERY_RANK_BONUS` | 0.000007 | Additional chance per rank above 15 |

### Layer Ranges
| Constant | Value |
|----------|-------|
| `SHALLOWS_LAYERS` | 1..=3 |
| `WARRENS_LAYERS` | 4..=7 |
| `HOLLOWS_LAYERS` | 8..=12 |
| `SUNKEN_REACH_LAYERS` | 13..=18 |
| `ABYSS_LAYERS` | 19..=25 |
| `VOID_START_LAYER` | 26 |

### Power Thresholds (Breakthrough)
Layers 1-25 use a lookup table. Void (26+) scales linearly at +80/layer. Sample values:

| Layer | Breakthrough | Expedition | Recon | Supply Run |
|-------|-------------|-----------|-------|-----------|
| 1 | 25 | 20 | 15 | 10 |
| 7 | 130 | 100 | 75 | 50 |
| 13 | 295 | 220 | 165 | 110 |
| 19 | 545 | 410 | 310 | 205 |
| 25 | 930 | 700 | 525 | 350 |
| 26+ | 930+80n | 700+60n | 525+45n | 350+30n |

### Mission Durations (Base, Before Modifiers)
| Tier | Supply Run | Recon | Expedition | Breakthrough | Construction |
|------|-----------|-------|------------|--------------|-------------|
| Shallows | 2.0h | 4.0h | 8.0h | 18.0h | 4.0h |
| Warrens | 2.5h | 5.0h | 10.0h | 20.0h | 5.0h |
| Hollows | 3.0h | 6.0h | 12.0h | 22.0h | 6.0h |
| Sunken Reach | 3.5h | 7.0h | 14.0h | 24.0h | 7.0h |
| Abyss/Void | 4.0h | 8.0h | 16.0h | 24.0h | 8.0h |

Minimum duration floor: 30 minutes (`MIN_MISSION_DURATION_SECS = 1800`).

### Duration Modifiers (Multiplicative)
| Source | Reduction |
|--------|-----------|
| Outpost | -25% |
| Familiarity: Mapped | -10% |
| Familiarity: Familiar | -20% |
| Familiarity: Mastered | -30% |
| Saboteur (base) | -10% |
| Saboteur (Lv10+) | -15% |
| Overpowered squad (>150%) | -10% |

### Mercenary Stats
| Archetype | Growth/Level (P/R/E) | Level 10 Stats (from base) |
|-----------|---------------------|---------------------------|
| Vanguard | 4.0 / 3.5 / 2.0 | ~48 / ~46 / ~26 |
| Scout | 3.0 / 3.0 / 3.5 | ~35 / ~37 / ~44 |
| Arcanist | 3.5 / 2.0 / 4.0 | ~42 / ~24 / ~50 |
| Medic | 2.0 / 3.5 / 3.0 | ~24 / ~46 / ~37 |
| Saboteur | 3.0 / 2.5 / 4.0 | ~37 / ~31 / ~48 |

XP curve: `200 * level^1.3` per level. Max level: 20. Stat variance: +/-10%.

### Recruit Quality Distribution by Rank
| Rank | Common | Uncommon | Rare | Elite | Pool Size |
|------|--------|----------|------|-------|-----------|
| 1 | 100% | 0% | 0% | 0% | 3 |
| 2 | 60% | 40% | 0% | 0% | 4 |
| 3 | 30% | 50% | 20% | 0% | 4 |
| 4 | 0% | 40% | 50% | 10% | 5 |
| 5 | 0% | 20% | 50% | 30% | 5 |

### Familiarity Gain Per Mission
| Mission Type | Gain |
|-------------|------|
| Supply Run | +5 |
| Recon | +15 |
| Expedition | +10 |
| Breakthrough | +15 |
| Construction | +5 |

## Extension Guide

### Adding a New Layer Tier
1. Add variant to `LayerTier` enum in `types.rs`
2. Add range constant in `types.rs` (e.g., `NEW_TIER_LAYERS: RangeInclusive<u32>`)
3. Update `LayerTier::from_layer()` match arm with new range
4. Add `display_name()` match arm
5. Add power thresholds to `layer_power_thresholds()` lookup table in `layers.rs`
6. Add base durations to `base_mission_duration_secs()` match in `layers.rs`
7. Add event templates for the new tier in event generation

### Adding a New Infrastructure Type
1. Add variant to `Infrastructure` enum and its `ALL` slice in `types.rs`
2. Implement `display_name()`, `description()`, `duration_reduction()` match arms in `types.rs`
3. Add cost formula to `infrastructure_build_cost()` in `layers.rs`
4. Add special-on-build logic (like Watchtower's familiarity bonus) to `build_infrastructure()` in `layers.rs`
5. Add a `Construction(Infrastructure::YourType)` mission template in generation
6. Update UI to display the new infrastructure type

### Adding a New Check-In Event Template
1. Define event with title, description, and 2-3 `EventChoice` entries
2. Set `auto_resolve_choice` to the safest non-gated choice (must not have `is_risky: true`)
3. Set `required_archetype` on archetype-gated choices
4. Mark risky choices with `is_risky: true` and optionally `unlocks_bonus_event: true` for chaining
5. Register the template in the event selection pool for appropriate layer tiers

### Adding a New MercArchetype
1. Add variant to `MercArchetype` enum and `ALL` slice in `types.rs`
2. Add `display_name()` and `base_stats()` match arms in `types.rs`
3. Add archetype primary flags in `archetype_primary_flags()` in `mercenaries.rs`
4. Add growth rates in `archetype_growth_per_level()` in `mercenaries.rs`
5. Add name epithets table in `mercenaries.rs`
6. Update `available_archetypes()` rank gating in `mercenaries.rs`
7. Add archetype-gated event choices in event templates

## Known Invariants and Gotchas

- **`GuildRank(0)` is invalid** — Methods index into `GUILD_RANK_STATS` with `self.0 - 1`, which panics for rank 0. Always use `GuildRank::MIN` (rank 1) as the minimum.
- **`frontier_layer()` when all layers cleared** — Returns `deepest_layer_reached + 1`, not 1.
- **`layer_record(0)` returns `None`** — Layers are 1-based throughout. Index 0 is explicitly handled.
- **Serde for `DateTime<Utc>`** — Requires the `serde` feature in the `chrono` dependency (`Cargo.toml`).
- **Recruit cost rounding** — All Warband Marks costs for recruits are rounded to the nearest 5 for cleaner display.
- **Injury recovery uses missions-remaining, not wall-clock** — Despite missions being wall-clock, injury countdown is in missions (1 mission ~ 6 hours average).
