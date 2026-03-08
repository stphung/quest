# Performance Optimization Review

Comprehensive audit of the Quest codebase identifying performance optimization opportunities, organized by priority and impact.

## Summary

The game runs at a 100ms tick interval (10 ticks/sec). Most individual inefficiencies are small (~0.01–1ms), but they compound across the tick loop + UI render path. The highest-impact areas are:

1. **Per-frame string allocations in UI rendering** — hundreds of `.repeat()`, `format!()`, and `.clone()` calls every 100ms
2. **Ticker character-width counting** — O(n) UTF-8 `.chars().count()` on every render, uncached
3. **Combat event string cloning** — enemy names cloned 4+ times per combat event
4. **Deep mission sorting every frame** — full sort of mission display order on every render
5. **Background save cloning all state** — 7 large struct clones every 30s

---

## Critical Priority — Per-Tick / Per-Frame Hot Path

### 1. Ticker Character Counting (Every Render)
**File**: `src/core/ticker.rs:93-177`

`entry_char_len()` calls `.chars().count()` multiple times per entry on icon, segments, and text fields. With up to 30 ticker entries, this runs ~90+ `.chars().count()` calls per render frame.

**Fix**: Cache character length in `TickerEntry` at creation time. One extra `usize` field eliminates all repeated counting.

### 2. Combat Enemy Name Cloning (Every Tick During Combat)
**File**: `src/core/tick_stages.rs:336-407`

`current_enemy_name` is `.clone()`'d 4 times across `PlayerAttack`, `EnemyAttack`, `EnemyDied`, `EliteDefeated`, and `BossDefeated` event handling. Each clone allocates on the heap.

**Fix**: Use `&str` references or `Rc<String>` to share the name without cloning. Alternatively, clone once and move into a local that can be referenced.

### 3. Deep Mission Sorting Every Frame
**File**: `src/ui/deep_missions.rs:810-819`

Creates a `display_order` vec and sorts all missions by pending events + remaining time on every single render frame, even when nothing has changed.

**Fix**: Cache the sorted order in the UI state. Only re-sort when a mission state changes (new mission, completion, event trigger).

### 4. String `.repeat()` Allocations in UI (Every Frame)
**Files**: `src/ui/stats_panel.rs`, `src/ui/stats_equipment.rs`, `src/ui/loom_scene.rs`, `src/ui/deep_missions.rs`, `src/ui/achievement_details.rs`, `src/ui/deep_results.rs`

Hundreds of `" ".repeat(n)` and `"\u{2500}".repeat(n)` calls for padding and separators. Each allocates a new `String` every frame. Progress bars use patterns like `"\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty)` which allocate two strings then concatenate.

**Fix**: For fixed-width separators, use `const` or `lazy_static` cached strings. For dynamic-width items like progress bars, use `String::with_capacity()` and `push_str()` in a single allocation.

### 5. Item Drop String Allocations (Per Kill, 15-25% Chance)
**File**: `src/core/tick_stages.rs:524-546`

Each item drop clones `display_name`, calls `slot_name().to_string()`, and `stat_summary()` (which builds a new String). Then clones these 3 more times for different event paths. Total: 5+ heap string allocations per item drop.

**Fix**: Build strings once, pass references to event constructors rather than cloning at each branch.

---

## High Priority — Frequent but Guarded Paths

### 6. Derived Stats Recalculation
**File**: `src/character/calculation.rs:12-131`

`calculate_derived_stats()` iterates all 7 equipment slots twice (once for attributes, once for affixes), computing enhancement multipliers each iteration.

**Status**: Already has a `derived_stats_dirty` flag — verify it's consistently checked before calling. **Additional fix**: merge the two passes into a single loop to halve iteration overhead.

### 7. Combat Bonuses Recomputation
**File**: `src/core/tick_stages.rs:46-47`

`compute_merged_bonuses()` rebuilds `CombatBonuses` from Haven, Sigils, and prestige sources. Already has a `bonuses_dirty` flag in `GameState`.

**Fix**: Ensure the dirty flag is checked before recomputing. Only update when Haven, Sigils, or prestige change.

### 8. Zone Achievement O(n) Lookup
**File**: `src/core/tick_stages.rs:616-620`

On every subzone boss defeat, does a linear search through all 50 zones by string name:
```rust
get_all_zones().iter().find(|z| z.name == *old_zone)
```

**Fix**: Use zone ID (already available in `BossDefeatResult`) for direct lookup instead of string comparison.

### 9. Background Save State Cloning (Every 30s)
**File**: `src/main_helpers/persistence.rs:98-118`

`spawn_background_save()` clones 7 structures: `CharacterManager`, `GameState`, `Achievements`, `Haven`, `EnhancementProgress`, `DeepState`, `LoomState`. The `GameState` is particularly large with nested dungeon, fishing, equipment, and zone progression data.

**Status**: Already guarded by `is_finished()` check. **Future optimization**: Consider `Arc`-based snapshots or copy-on-write for the largest structs to reduce clone cost.

### 10. `FlatGameState` Conversion for Serialization
**File**: `src/core/game_state_serde.rs:40-62`

`From<&GameState> for FlatGameState` clones `combat_state`, `equipment`, `active_dungeon`, `fishing`, `zone_progression`, and `storm_sigils`. This happens on every save.

**Fix**: Serialize directly from `&GameState` using custom `Serialize` impl to avoid intermediate clones.

---

## Medium Priority — Allocations in Rendering Loops

### 11. Equipment Name Cloning in UI Loop
**File**: `src/ui/stats_equipment.rs:119`

`item.display_name.clone()` runs for all 7 equipment slots every frame.

**Fix**: Use `&str` references in `Span` construction (Ratatui supports borrowed strings).

### 12. Word-Wrapping Vec Allocations
**Files**: `src/ui/deep_missions.rs:1610,1865,1956,2242`, `src/ui/deep_events.rs:159`

`split_whitespace().collect::<Vec<_>>()` allocates a vec per mission/event for word wrapping on every frame.

**Fix**: Use iterator-based word wrapping that doesn't collect into intermediate vecs.

### 13. Unnecessary `.collect()` Before `.chunks()`
**File**: `src/ui/achievement_details.rs:693`

```rust
slot_names.iter().enumerate().collect::<Vec<_>>().chunks(2)
```

The intermediate `collect()` is unnecessary — use `Iterator::array_chunks()` or manual chunking.

### 14. `visible_entries()` Allocates Vec Every Frame
**File**: `src/core/ticker.rs:148`

Returns `Vec<(&TickerEntry, isize)>` — allocates on every render call.

**Fix**: Return an iterator, or use a `SmallVec<[_; 16]>` since visible entry count is bounded.

### 15. Width Calculations via `.chars().count()` in UI
**Files**: `src/ui/ticker.rs:42,187,201,277`, `src/ui/mod.rs:1032`, `src/ui/achievement_tabs.rs:45,185`

Multiple `.iter().map(|s| s.content.chars().count()).sum()` patterns for layout width calculation. Each is O(n) on UTF-8 content, repeated every frame.

**Fix**: Cache widths when spans are created, or use `unicode_width::UnicodeWidthStr` (already a dependency) which better handles actual display widths.

---

## Low Priority — Infrequent or Bounded

### 16. Item Generation Affix Array
**File**: `src/items/generation.rs:104-133`

`generate_affixes()` creates `all_affix_types` array on every call. Item generation rate is ~1-2/sec.

**Fix**: Use a `static` const array. Pre-allocate `Vec::with_capacity(4)` based on expected affix count.

### 17. Loom Production Double Iteration
**File**: `src/loom/logic.rs:118-150`

`tick_base_production()` collects node data into a `Vec`, then iterates again. Also uses `HashMap::new()` without capacity hint.

**Fix**: Single-pass iteration. Use `HashMap::with_capacity(6)` (max 6 resource types).

### 18. Fishing Message String Parsing
**File**: `src/core/tick_stages.rs:220-248`

Parses fish names and item names from message strings using `.split().nth(1)` and `.to_string()`.

**Fix**: Return structured data from the fishing module instead of parsing display strings.

### 19. Recent Drops VecDeque Capacity
**File**: `src/core/game_state.rs:161`

`VecDeque::with_capacity(5)` but max capacity is 10 entries. Causes reallocation.

**Fix**: Use `VecDeque::with_capacity(10)` to match the actual cap, or use a fixed-size circular buffer.

---

## Build & Dependency Optimizations

### 20. Cargo Profile Tuning
**File**: `Cargo.toml`

Current config is already good:
- `opt-level = 2` for dependencies in dev builds
- `split-debuginfo = "unpacked"` for faster macOS linking
- `codegen-units = 256` for test parallelism

**Potential addition**: Consider `[profile.release] lto = "thin"` for smaller/faster release binaries if not already set.

### 21. `chess-engine` Dependency
**File**: `Cargo.toml:11`

The `chess-engine` crate is pulled in for a challenge minigame. It's a relatively heavy dependency for a single feature.

**Potential**: If build times are a concern, evaluate if a simpler chess implementation would suffice.

### 22. `serde_json` Pretty vs Compact
If save files use `serde_json::to_string_pretty()`, switching to `to_string()` reduces file size by ~30-40% and speeds up serialization. Only matters if save files are large.

---

## Architecture Recommendations

### A. String Interning for Repeated Values
Enemy names, zone names, and item slot names are cloned frequently. A string interner (or `&'static str` for fixed values) would eliminate most combat-path allocations.

### B. Event System Refactor
`TickEvent` variants carry owned `String` fields that get cloned through the pipeline. Switching to `Cow<'static, str>` or an enum-based message type would eliminate heap allocation for the ~80% of messages that are static templates.

### C. UI Dirty Flags
Most UI panels re-render identical content frame after frame. Adding dirty flags per panel (similar to the existing `derived_stats_dirty` pattern) would allow skipping unchanged panels entirely.

### D. SmallVec for Bounded Collections
Many `Vec` allocations are for collections with known small upper bounds (equipment slots = 7, affixes = 5, sigils = 5, visible ticker entries ~10). `SmallVec` would keep these on the stack.

---

## Estimated Impact

| Category | Per-Tick Cost | Fix Effort | Priority |
|----------|--------------|------------|----------|
| Ticker char counting | ~0.5-1ms/frame | Low | Critical |
| Combat string clones | ~0.05ms/tick | Low | Critical |
| Deep mission sort | ~0.1-0.5ms/frame | Low | Critical |
| UI `.repeat()` allocs | ~0.5-2ms/frame | Medium | High |
| Item drop strings | ~0.1ms/drop | Medium | High |
| Derived stats | ~0.1ms/tick (when dirty) | Low | High |
| Save cloning | ~5-10ms/30s | High | Medium |
| Serialization | ~2-5ms/save | High | Medium |
| Vec allocs in UI | ~0.2-0.5ms/frame | Medium | Medium |
| String interning | ~1-2ms/tick total | High | Future |

**Total estimated waste**: ~2-5ms per tick/frame, against a 100ms budget. Not crisis-level, but addressing the Critical items would noticeably improve responsiveness on lower-end hardware and reduce GC pressure from short-lived allocations.
