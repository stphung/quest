> Backported design record. Sources: docs/plans/sys-arch-1-analysis.md.

## sys-arch-1-analysis.md

# Module Refactoring Analysis — sys-arch-1

Analysis of game logic modules for refactoring opportunities. Produced as research for the `refactor/large-modules` branch.

**Important context**: PRs #292, #293, #294 already performed extensive submodule extraction across all major modules. This analysis reflects the *current* post-extraction state.

---

## 1. src/combat/logic.rs (2,654 LOC)

### Current Structure

| Lines | Content |
|-------|---------|
| 1-4 | Two `pub use` re-exports (backward compatibility) |
| 5-2654 | `#[cfg(test)] mod tests` (100% test code) |

Production code is **4 lines** of re-exports:
- `pub use super::attacks::effective_enemy_attack_interval;`
- `pub use super::orchestration::update_combat;`

All combat logic has been fully extracted to:
- `orchestration.rs` — `update_combat()` coordinator
- `player_attack.rs` — Player damage pipeline
- `enemy_attack.rs` — Enemy attack resolution
- `damage.rs` — Shared damage helpers, `handle_enemy_death()`
- `events.rs` — `CombatEvent`, `HavenCombatBonuses`, `GodItemCombatBonuses`
- `regen.rs` — HP regeneration
- `attacks.rs` — `effective_enemy_attack_interval()`

### Extractable Submodules

None for production code. Already fully extracted.

### Long Functions (>80 lines)

None in production code. The test module contains large integration tests but those are expected.

### Code Duplication

None identified.

### Coupling Issues

None. The re-exports provide a stable public API surface while internal modules are free to refactor.

### Refactoring Recommendations

**Low priority**: The ~2,650 lines of tests could be moved to a dedicated file under `tests/` (e.g., `tests/combat_logic_test.rs`) to reduce the file size. However, inline tests have the advantage of access to private items and co-location with the code they test. Since the production code in this file is only 4 lines, this is purely a file-size concern, not a maintainability concern.

**Verdict: No actionable refactoring needed.** This module is already well-structured.

---

## 2. src/fishing/logic.rs (1,321 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-20 | Imports + `apply_timer_reduction()` | Helper: timer reduction |
| 24-44 | `HavenFishingBonuses`, `FishingTickResult` | Structs |
| 56-224 | `tick_fishing_with_haven_result()` (~168 lines) | Main tick processor |
| 226-246 | `tick_fishing_with_haven()`, `tick_fishing()` | Legacy wrappers |
| 248-1321 | `#[cfg(test)] mod tests` (~1,073 lines) | Tests |

Production code: **~247 lines** (19% of file). Tests: **~1,073 lines** (81%).

Already-extracted submodules:
- `discovery.rs` — `try_discover_fishing()`
- `drops.rs` — `try_fishing_item_drop()`
- `rank.rs` — `check_rank_up_with_max()`, `get_max_fishing_rank()`
- `generation.rs` — Rarity rolling, fish generation, Leviathan hunt, session generation
- `types.rs` — `FishRarity`, `FishingPhase`, `FishingSession`, `FishingState`

### Long Functions (>80 lines)

- `tick_fishing_with_haven_result()` (lines 56-224, ~168 lines): This is the main fishing tick processor. It handles phase transitions (Casting -> Waiting -> Reeling -> Catch), double fish logic, Leviathan encounters, XP awards, item drops, and session completion. While long, it reads as a clear state machine with one match arm per phase. Breaking it into sub-functions would scatter the phase transition logic without improving clarity.

### Code Duplication

Minor: The timer reduction pattern `apply_timer_reduction(base_ticks, haven.timer_reduction_percent)` followed by `apply_timer_reduction(after_haven, god_item_fishing_reduction_percent)` appears 3 times (lines 85-87, 96-99, 212-215). Could be extracted to a 2-line helper like `apply_combined_timer_reduction(base, haven_pct, god_item_pct)`, but this is marginal.

### Coupling Issues

None. Haven bonuses are injected via explicit `HavenFishingBonuses` parameter. All generation logic is delegated to `generation.rs`.

### Refactoring Recommendations

1. **Optional — Extract combined timer reduction**: Create `fn apply_combined_reduction(base: u32, haven_pct: f64, god_item_pct: f64) -> u32` to DRY up the 3 occurrences. Very minor improvement.

2. **Optional — Move `HavenFishingBonuses` to types.rs**: The struct is a plain data type used across the module; moving it to `types.rs` would be consistent with the pattern of other modules.

**Verdict: Well-structured. No urgent refactoring needed.** The 168-line function is justified by the state machine nature of the code.

---

## 3. src/dungeon/logic.rs (1,298 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-30 | `DungeonEvent` enum (7 variants) | Event types |
| 32-80 | `update_dungeon()` (~48 lines) | Main dungeon tick |
| 82-120 | `on_room_enemy_defeated()` (~38 lines) | Room combat completion |
| 122-140 | `on_elite_defeated()` (~18 lines) | Elite room key drop |
| 142-155 | `on_boss_defeated()` (~13 lines) | Boss room completion |
| 155-160 | `on_player_died_in_dungeon()` (~5 lines) | Death handling |
| 160-168 | `current_room_needs_combat()` (~8 lines) | Combat check |
| 168-171 | `get_enemy_stat_multiplier()` (~3 lines) | Stat lookup |
| 172-1298 | `#[cfg(test)] mod tests` (~1,126 lines) | Tests |

Production code: **~171 lines** (13% of file). Tests: **~1,126 lines** (87%).

Already-extracted submodules:
- `pathfinding.rs` — BFS pathfinding for dungeon navigation
- `rewards.rs` — Treasure/loot generation for dungeon rooms
- `generation.rs` — Procedural dungeon generation
- `types.rs` — Room, RoomType, RoomState, Dungeon, DungeonSize

### Long Functions (>80 lines)

None. All production functions are under 50 lines.

### Code Duplication

None identified.

### Coupling Issues

None. Dungeon logic is self-contained with clear integration points.

### Refactoring Recommendations

**Verdict: No refactoring needed.** The production code is compact (171 lines), well-factored into small focused functions, and the module structure is clean.

---

## 4. src/dungeon/generation.rs (709 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-30 | Imports, constants | Setup |
| 32-120 | `generate_dungeon()` (~88 lines) | Top-level dungeon generator |
| 122-210 | `generate_maze()` (~88 lines) | DFS maze carving |
| 212-250 | `add_extra_connections()` (~38 lines) | Loop creation for non-linear exploration |
| 252-270 | `find_dead_ends()` (~18 lines) | Dead-end room finder |
| 272-358 | `place_special_rooms()` (~86 lines) | Boss, Elite, Treasure placement |
| 360-378 | `distance_squared()`, `reveal_adjacent_rooms()` | Helpers |
| 380-709 | `#[cfg(test)] mod tests` (~329 lines) | Tests |

Production code: **~378 lines** (53% of file). Tests: **~329 lines** (47%).

### Long Functions (>80 lines)

- `generate_dungeon()` (lines 32-120, ~88 lines): Orchestrates all generation steps. Reads clearly as a pipeline: create grid, carve maze, place entrance, find dead ends, place boss, add connections, place specials, reveal adjacent. Already well-structured.
- `generate_maze()` (lines 122-210, ~88 lines): DFS maze carving algorithm. This is a single algorithm with clear loop structure. Breaking it up would not improve readability.
- `place_special_rooms()` (lines 272-358, ~86 lines): Places Boss, Elite, and Treasure rooms. The elite placement logic has several fallback strategies (viable dead ends -> furthest dead end -> furthest room -> last resort). Could potentially extract `find_best_elite_position()` but the fallback chain is clear as-is.

### Code Duplication

None significant.

### Coupling Issues

None. Generation depends only on `types.rs` (Room, Dungeon, DungeonSize) and `rand::Rng`.

### Refactoring Recommendations

1. **Optional — Extract `find_best_elite_position()`**: The elite room placement logic in `place_special_rooms()` has a 4-level fallback chain that could be a named function for clarity. Minor improvement.

**Verdict: Well-structured. No urgent refactoring.** The three ~88-line functions are algorithmic in nature and read well as-is.

---

## 5. src/fishing/generation.rs (707 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-40 | Imports, constants (fish names, XP ranges, rarity chances) | Data |
| 42-80 | `roll_fish_rarity()` | Rank-adjusted rarity rolling |
| 82-110 | `generate_fish()`, `generate_fish_with_rank()` | Fish creation with Leviathan check |
| 112-140 | Leviathan encounter logic | Progressive hunt system |
| 142-170 | `is_storm_leviathan()`, `generate_fishing_session()` | Helpers |
| 172-200 | `roll_casting_ticks()`, `roll_waiting_ticks()`, `roll_reeling_ticks()` | Phase timing |
| 202-300 | More constants and helpers | Data arrays |
| 302-707 | `#[cfg(test)] mod tests` (~405 lines) | Tests |

Production code: **~300 lines** (42% of file). Tests: **~405 lines** (58%).

### Long Functions (>80 lines)

None. All functions are concise.

### Code Duplication

None identified.

### Coupling Issues

None. Only depends on `types.rs` and `rand::Rng`.

### Refactoring Recommendations

**Verdict: No refactoring needed.** Clean, well-organized module.

---

## 6. src/core/tick_stages.rs (634 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-50 | Imports | Setup |
| 52-180 | `process_dungeon_events()` (~128 lines) | Stage 4: dungeon event processing |
| 182-290 | `process_fishing_tick()` (~108 lines) | Stage 5: fishing tick processing |
| 292-510 | `process_combat_events()` (~218 lines) | Stage 6: combat event -> TickEvent mapping |
| 512-566 | `process_item_drop()` (~54 lines) | Item drop processing |
| 568-590 | `process_discoveries()` (~22 lines) | Dungeon/fishing spot discovery |
| 592-620 | `process_zone_achievements()` (~28 lines) | Zone completion achievements |
| 622-634 | `collect_achievement_events()` (~12 lines) | Achievement event collection |

Production code: **634 lines** (100% — no inline tests).

### Long Functions (>80 lines)

- `process_dungeon_events()` (lines 52-180, ~128 lines): Processes `DungeonEvent` variants with match arms for room entry, treasure, key, boss unlock/defeat, elite defeat, death. Each arm has XP application, achievement tracking, and TickEvent emission. The repetitive XP+level-up pattern appears 3 times.

- `process_fishing_tick()` (lines 182-290, ~108 lines): Orchestrates fishing tick processing — calls `tick_fishing_with_haven_result()`, processes items/rank-ups/Leviathan, handles play time updates. Clear sequential flow.

- `process_combat_events()` (lines 292-510, ~218 lines): Maps `CombatEvent` variants to `TickEvent` variants. The largest function in this file. Each match arm handles XP, level-ups, achievement tracking, zone progression messages, item drops, and discoveries. The XP+level-up+achievement pattern repeats ~5 times.

### Code Duplication

**XP + level-up + achievement pattern** (repeated ~8 times across the file):
```rust
let level_before = state.character_level;
apply_tick_xp(state, xp_gained as f64);
if state.character_level > level_before {
    for lvl in (level_before + 1)..=state.character_level {
        achievements.on_level_up(lvl, Some(&state.character_name));
    }
    result.events.push(TickEvent::LeveledUp {
        new_level: state.character_level,
    });
}
```

This 8-line pattern appears in:
- `process_dungeon_events()`: Elite defeated, Boss defeated (2x)
- `process_combat_events()`: EnemyDefeated, DungeonEliteDefeated, BossDefeated, SubzoneBossDefeated (4x)

**Recommendation**: Extract `apply_xp_and_check_levelup(state, xp: f64, achievements, result)` helper to eliminate this duplication.

### Coupling Issues

This file couples together dungeon, fishing, combat, zone, item, and achievement systems. However, this is by design — it's the tick stage layer that maps domain events to tick events. The coupling is inherent to the orchestration role.

### Refactoring Recommendations

1. **Medium priority — Extract `apply_xp_and_check_levelup()` helper**: Would reduce the file by ~50 lines and eliminate the most repeated pattern.

2. **Optional — Split into 3 files**: `tick_stage_dungeon.rs`, `tick_stage_fishing.rs`, `tick_stage_combat.rs`. Each file would have a single stage function plus its helpers. This would reduce individual file sizes to ~130-270 lines each. However, the current single-file approach keeps all tick stages co-located, which aids comprehension.

**Verdict: One concrete refactoring opportunity** (the XP+levelup helper). File split is optional.

---

## 7. src/core/offline.rs (591 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-15 | Imports, `OfflineReport` struct | Data types |
| 17-55 | `calculate_offline_xp()` (~38 lines) | Pure XP calculation |
| 57-95 | `process_offline_progression()` (~38 lines) | Full offline processing |
| 96-591 | `#[cfg(test)] mod tests` (~495 lines) | Tests |

Production code: **~95 lines** (16% of file). Tests: **~495 lines** (84%).

### Long Functions (>80 lines)

None. Both production functions are ~38 lines.

### Code Duplication

None.

### Coupling Issues

None. Takes primitive parameters (elapsed seconds, prestige rank, modifiers, haven bonus). Very clean interface.

### Refactoring Recommendations

**Verdict: No refactoring needed.** Extremely clean module with minimal production code.

---

## 8. src/main.rs (695 LOC)

### Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-50 | Imports, `AppScreen` enum, constants | Setup |
| 52-120 | `main()` entry point (~68 lines) | Terminal setup, character selection |
| 122-400 | Game loop (~278 lines) | Tick processing, input, rendering |
| 402-500 | Helper functions | Autosave, update checks, soulforge animation |
| 500-695 | More game loop / event handling | TickEvent -> combat log mapping |

Production code: **695 lines** (100% — no inline tests).

Already-extracted to `main_helpers/`:
- `character_screens.rs` — Character screen handlers
- `input_routing.rs` — Game input routing
- `achievements.rs` — Achievement processing
- `offline.rs` — Offline progression handling
- `overlay.rs` — Overlay management
- `persistence.rs` — Save/load orchestration
- `scene.rs` — Scene rendering dispatch
- `update.rs` — Update checking

### Long Functions (>80 lines)

The game loop in `main()` is the longest section but it's an inherent characteristic of the game's entry point. It handles:
- Terminal initialization
- Character screen flow
- Tick scheduling
- Input polling
- TickEvent processing
- Rendering dispatch
- Autosave timing
- Update check timing
- Soulforge animation ticks

### Code Duplication

None significant. The TickEvent processing loop has some repetitive `add_log_entry()` calls but each variant needs distinct formatting.

### Coupling Issues

`main.rs` necessarily couples to all systems (UI, core, combat, dungeon, fishing, etc.) as the entry point. This is expected and unavoidable.

### Refactoring Recommendations

1. **Optional — Extract TickEvent processing**: The TickEvent -> combat log mapping could be extracted to a `tick_events.rs` helper module (note: this file already exists at `src/tick_events.rs`). If the mapping logic in `main.rs` is duplicating what's in `tick_events.rs`, consolidation may help.

**Verdict: Reasonably well-structured for an entry point.** The 695 LOC is manageable and most extractable logic has already been moved to `main_helpers/`.

---

## 9. src/zones/progression.rs (216 LOC) + src/zones/data.rs (748 LOC)

### zones/progression.rs — Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-30 | `ZoneProgression` struct + constructors | State |
| 32-70 | `record_kill()`, `should_spawn_boss()`, `kills_until_boss()` | Kill tracking |
| 72-90 | `unlock_zone()`, `defeat_boss()` | Zone unlocking |
| 92-115 | `current_location_names()` | Display helper |
| 116-216 | `#[cfg(test)] mod tests` (~100 lines) | Tests |

Production code: **~115 lines**. Tests: **~100 lines**.

Already-extracted submodules:
- `boss_defeat.rs` — `on_boss_defeated()` logic
- `advancement.rs` — Zone advancement logic
- `gates.rs` — Weapon gates and prestige requirements

### zones/data.rs — Current Structure

| Lines | Content | Responsibility |
|-------|---------|---------------|
| 1-15 | Imports, type definitions | Setup |
| 17-584 | `ALL_ZONES: LazyLock<Vec<Zone>>` | Static zone data (11 zones) |
| 586-602 | `get_all_zones()`, `get_zone()`, `get_subzone()` | Lookup functions |
| 604-748 | `#[cfg(test)] mod tests` (~144 lines) | Tests |

Production code: **~602 lines** (mostly static data declarations). Tests: **~144 lines**.

### Long Functions (>80 lines)

None. The `ALL_ZONES` initializer is a large data literal (~567 lines) but it's not a function — it's declarative zone data.

### Code Duplication

The zone definition pattern is repetitive (each zone has the same struct fields) but this is inherent to data declaration — not logic duplication.

### Coupling Issues

None. Pure data module with no external dependencies beyond its own types.

### Refactoring Recommendations

**Verdict: No refactoring needed.** Both files are clean. `progression.rs` is compact at 216 LOC with boss defeat, advancement, and gate logic already extracted. `data.rs` is mostly static data that cannot be meaningfully refactored.

---

## Summary

### Modules That Need No Further Refactoring (7 of 9)

| Module | Production LOC | Test LOC | Status |
|--------|---------------|----------|--------|
| combat/logic.rs | 4 | 2,650 | Fully extracted; only re-exports remain |
| dungeon/logic.rs | 171 | 1,126 | Compact, well-factored functions |
| dungeon/generation.rs | 378 | 329 | Clean algorithmic code |
| fishing/logic.rs | 247 | 1,073 | Clear state machine |
| fishing/generation.rs | 300 | 405 | Clean, concise functions |
| core/offline.rs | 95 | 495 | Minimal, clean interface |
| zones/ (both files) | 717 | 244 | Already well-extracted |

### Modules With Minor Opportunities (2 of 9)

| Module | Production LOC | Opportunity | Priority |
|--------|---------------|-------------|----------|
| core/tick_stages.rs | 634 | Extract `apply_xp_and_check_levelup()` helper (eliminates 8x duplication of XP+levelup pattern) | Medium |
| main.rs | 695 | Consolidate TickEvent mapping with existing `tick_events.rs` if overlapping | Low |

### Key Insight

**The prior refactoring (PRs #292-294) was highly effective.** Most "large" files are large due to inline test modules, not production code. The actual production code in these modules is compact and well-structured:

- combat/logic.rs: 4 lines of production code (99.8% tests)
- dungeon/logic.rs: 171 lines (87% tests)
- fishing/logic.rs: 247 lines (81% tests)
- core/offline.rs: 95 lines (84% tests)

The one concrete refactoring worth pursuing is the **XP+levelup helper extraction in `tick_stages.rs`**, which would eliminate ~50 lines of repeated boilerplate across 8 call sites.
