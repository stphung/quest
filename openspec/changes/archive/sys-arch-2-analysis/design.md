> Backported design record. Sources: docs/plans/sys-arch-2-analysis.md.

## sys-arch-2-analysis.md

# Sys Arch 2: Challenge, Character, Achievements, Items, UI Module Analysis

## Executive Summary

This analysis covers challenge minigames (10 games), achievements, character management, haven, combat types, and UI modules. Prior refactoring (PRs #292-#294) already extracted many submodules. The remaining opportunities focus on:

1. **Challenge `apply_game_result` duplication** — 10 near-identical functions that can be macro-generated
2. **Morris/Minesweeper logic.rs test bloat** — tests account for 50%+ of file length
3. **UI scene monster files** — fishing_scene and flappy_scene have 400-500 line render functions
4. **Achievement data.rs** — 1683 LOC of static data, resistant to meaningful splits
5. **Forfeit pattern duplication** — repeated across all 10 challenge input handlers

---

## 1. Challenge Minigames

### 1.1 Existing Infrastructure (Already DRYed)

The challenges module already has excellent shared infrastructure:

- **`difficulty_enum_impl!` macro** (mod.rs:7-31) — Generates `ALL`, `from_index()`, `name()` for all difficulty enums
- **`apply_challenge_rewards()`** (mod.rs:129-211) — Shared reward application: XP, prestige ranks, fishing ranks, combat log
- **`GameResultInfo` struct** (mod.rs:109-124) — Standardized result descriptor
- **`DifficultyInfo` trait** (menu.rs:172-194) — Shared difficulty display with `name()`, `reward()`, `extra_info()`, `difficulty_enum()`
- **`game_common.rs`** (UI) — Shared layout, status bars, game-over overlays
- **`ActiveMinigame` enum** (mod.rs:67-79) with `has_game_result()` method

### 1.2 `apply_game_result` Pattern (HIGH VALUE — estimated 300+ lines savings)

Every challenge has a nearly identical `apply_game_result` function. The pattern:

```rust
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    // 1. Extract game from state.active_minigame
    let game = match state.active_minigame.as_ref() {
        Some(ActiveMinigame::Variant(g)) => g,
        _ => return None,
    };
    // 2. Get result, difficulty, reward
    let result = game.game_result?;
    let difficulty = game.difficulty;
    let reward = difficulty.reward();
    // 3. Determine won/loss_message
    let (won, loss_message) = match result { ... };
    // 4. Call apply_challenge_rewards with GameResultInfo
    apply_challenge_rewards(state, GameResultInfo { ... })
}
```

**Files affected**: chess/logic.rs:221, morris/logic.rs:734, gomoku/logic.rs:231, minesweeper/logic.rs:281, rune/logic.rs:167, go/logic.rs:420, snake/logic.rs:229, flappy/logic.rs:234, jezzball/logic.rs:513, runic_shift/logic.rs:597

**Recommendation**: Create a macro `apply_game_result_impl!` in `mod.rs` that generates the entire function given:
- ActiveMinigame variant name
- Result enum type
- Win/Draw/Loss variant names
- MinigameType enum value
- Icon string
- Win message
- Loss message (default and forfeit)

This would reduce each `apply_game_result` from ~35 lines to a ~5 line macro invocation.

**Alternative** (simpler): Add a helper trait `MinigameState` with methods `game_result()`, `difficulty()`, `forfeit_pending()`, `icon()`, `win_message()`, `loss_message()`. Then a single generic `apply_minigame_result<T: MinigameState>()` function. This is cleaner than a macro but requires adding trait impls to each game struct.

### 1.3 Forfeit Pattern Duplication (MEDIUM VALUE — estimated 100+ lines savings)

Every challenge handles forfeit identically:
```rust
if game.forfeit_pending {
    game.game_result = Some(XxxResult::Loss);
} else {
    game.forfeit_pending = true;
}
```

And for non-forfeit keys during forfeit_pending:
```rust
if game.forfeit_pending {
    game.forfeit_pending = false;
}
```

This is duplicated in snake/logic.rs, flappy/logic.rs, jezzball/logic.rs, runic_shift/logic.rs, morris/logic.rs, minesweeper/logic.rs, go/logic.rs, gomoku/logic.rs, rune/logic.rs, chess/logic.rs.

**Recommendation**: NOT worth extracting into a shared function. The forfeit handling is interleaved with game-specific input processing in complex ways (e.g., checking `selected_position` before forfeit in morris, checking `active_wall` in jezzball). Attempting to abstract this would add complexity without meaningful simplification. The current pattern is clear and consistent — it's "duplication by convention" not "duplication by copy."

### 1.4 Morris logic.rs (1622 LOC)

**Structure**:
- Core game logic: lines 1-730 (~730 LOC)
  - `MoveUndo` struct, `MorrisInput` enum
  - Input processing: `process_input`, `process_cancel`, `process_human_enter` (~70 LOC)
  - Move generation: `get_legal_moves`, `get_placing_moves`, `get_movement_moves`, `get_capture_moves` (~95 LOC)
  - Move application: `apply_move`, `make_move_for_search`, `unmake_move` (~110 LOC)
  - Turn management: `end_turn`, `end_turn_for_search`, `check_win_condition` (~65 LOC)
  - AI: `process_ai_thinking`, `calculate_think_ticks`, `get_ai_move`, `minimax_optimized` (~120 LOC)
  - Board evaluation: `evaluate_board`, `count_mills`, `count_potential_mills`, `count_mobility` (~60 LOC)
  - Result application: `apply_game_result` (~40 LOC)
- Tests: lines 772-1622 (~850 LOC — **52% of file**)

**Recommendation**: Extract AI into `morris/ai.rs` (same pattern as gomoku/ai.rs and go/mcts.rs which already exist):
- Move `MoveUndo`, `make_move_for_search`, `unmake_move`, `end_turn_for_search` to `morris/ai.rs`
- Move `get_ai_move`, `minimax_optimized`, `evaluate_board`, `count_mills`, `count_potential_mills`, `count_mobility` to `morris/ai.rs`
- ~350 LOC moves to new file, reducing logic.rs from 1622 to ~1272

The AI tests would follow their functions. This mirrors the existing gomoku/ai.rs pattern.

### 1.5 Minesweeper logic.rs (1589 LOC)

**Structure**:
- Core game logic: lines 1-314 (~314 LOC)
  - `MinesweeperInput` enum
  - Input processing: `process_input` (~40 LOC)
  - Mine placement: `get_neighbors`, `place_mines`, `calculate_adjacent_counts` (~80 LOC)
  - Cell revealing: `reveal_cell`, `flood_fill_reveal`, `reveal_all_mines` (~60 LOC)
  - Flag/win logic: `toggle_flag`, `check_win_condition`, `handle_first_click` (~50 LOC)
  - Result: `apply_game_result` (~35 LOC)
- Tests: lines 316-1589 (~1273 LOC — **80% of file**)

**Recommendation**: The logic itself is already well-factored at 314 LOC. The test suite is extremely thorough (flood fill, boundary conditions, etc.) but accounts for 80% of the file. No code extraction needed — the tests are valuable and well-organized. The file size is due to comprehensive test coverage, not code bloat.

### 1.6 Challenge menu.rs (1265 LOC)

**Structure**:
- Input processing: `process_input`, `accept_selected_challenge`, `decline_selected_challenge` (~65 LOC)
- Reward types: `ChallengeReward`, `DifficultyInfo` trait, 10 DifficultyInfo impls (~260 LOC)
- Discovery system: `ChallengeWeight`, `CHALLENGE_TABLE`, `ChallengeMenu` struct, `try_discover_challenge_with_haven` (~155 LOC)
- Challenge creation: `create_challenge()` — 110 LOC of match arms with flavor text (~110 LOC)
- Tests: ~545 LOC

**Long function**: `create_challenge()` at 110 lines — one match arm per challenge type, each ~10 lines of flavor text.

**Recommendation**:
- Extract DifficultyInfo impls to a new `menu_rewards.rs` (~260 LOC). Each DifficultyInfo impl is tightly coupled to its challenge type but contributes significant bulk.
- `create_challenge()` is inherently a big match — no improvement possible without adding unnecessary abstraction (e.g., storing descriptions as data, which gains nothing).
- The existing file size (1265) is manageable and well-organized.

### 1.7 Remaining Challenge logic.rs Files (635-937 LOC)

| File | LOC | Code LOC | Test LOC | Long Functions |
|------|-----|----------|----------|----------------|
| runic_shift/logic.rs | 937 | ~580 | ~357 | `find_matches()` 83 lines |
| flappy/logic.rs | 838 | ~230 | ~608 | `step_physics()` 90 lines |
| go/logic.rs | 820 | ~510 | ~310 | none |
| jezzball/logic.rs | 812 | ~510 | ~302 | none |
| rune/logic.rs | 806 | ~200 | ~606 | none |
| gomoku/logic.rs | 778 | ~230 | ~548 | none |
| snake/logic.rs | 766 | ~250 | ~516 | none |
| chess/logic.rs | 635 | ~300 | ~335 | none |

**Observations**:
- All files are within reasonable size
- High test-to-code ratios (60-80% tests) inflate apparent file sizes
- `step_physics()` in flappy at 90 lines handles gravity, collision detection, pipe spawning — inherently complex, not worth splitting
- `find_matches()` in runic_shift at 83 lines checks horizontal/vertical/chain matches — inherently complex

**Recommendation**: No further extraction needed for these files. They are well-structured with clear separation.

### 1.8 Challenge types.rs Files

| File | LOC | Notes |
|------|-----|-------|
| morris/types.rs | 734 | MorrisGame struct, board layout (ADJACENCIES, MILLS constants), cursor movement |
| chess/types.rs | 734 | ChessGame struct wrapping chess-engine crate, move conversion, `try_move_to_cursor()` 95 LOC |

**chess/types.rs**: `try_move_to_cursor()` at 95 lines handles promotion detection, move legality checks, and move application. Could be split, but it's a single cohesive operation. No recommendation.

**morris/types.rs**: Contains `ADJACENCIES` and `MILLS` static arrays (~100 LOC of constants), `MorrisGame` struct with methods (~200 LOC), `CursorDirection` and cursor movement (~80 LOC), plus tests. Well-organized, no splits needed.

---

## 2. Achievement System

### 2.1 achievements/data.rs (1683 LOC)

**Structure**:
- `ALL_ACHIEVEMENTS` const slice: ~1170 LOC of `AchievementDef` entries
- Tests: ~510 LOC (comprehensive coverage tests)

**Long functions** (both in tests):
- `test_every_achievement_id_variant_has_definition()`: 184 lines — explicitly lists every AchievementId variant
- `test_challenge_achievements_cover_all_game_types_and_difficulties()`: 120 lines — exhaustive coverage check

**Recommendation**: **No split recommended**. This file is a single const array — splitting it by category (e.g., `data_combat.rs`, `data_challenges.rs`) would add module overhead without improving navigability. The file is pure data with no logic. It's large because there are 149 achievements. The tests are valuable for catching missing definitions.

### 2.2 achievements/types.rs (1153 LOC)

**Structure**:
- `AchievementCategory` enum + impls: ~40 LOC
- `AchievementId` enum (149 variants): ~222 LOC
- `AchievementDef`, `AchievementProgress`, `UnlockedAchievement` structs: ~30 LOC
- `Achievements` struct + methods: ~250 LOC
- Tests: ~610 LOC

**Note**: Prior refactoring already extracted `handlers.rs` (event handlers) and `milestones.rs` (threshold arrays) from this file.

**Recommendation**: The `Achievements` impl methods (unlock, is_unlocked, sync functions) are ~250 LOC and could theoretically be split, but they're all methods on the same struct and already have handlers.rs for the event-based methods. No further split recommended — the file is appropriately sized after the prior extractions.

---

## 3. Character Module

### 3.1 character/manager.rs (826 LOC)

**Structure**:
- `CharacterSaveData` struct: ~32 LOC
- `CharacterInfo` struct: ~15 LOC
- `CharacterManager` struct + `new()`, `with_dir()`: ~25 LOC
- `ACCOUNT_FILES` constant: ~1 LOC
- Tests: ~750 LOC

**Note**: Prior refactoring already extracted `persistence.rs` (file I/O), `name_validation.rs` (validation rules).

**Recommendation**: No further extraction. The actual code is only ~75 LOC (struct definitions + constructor). The tests are comprehensive filesystem tests that legitimately belong together. The file size is appropriate post-refactoring.

---

## 4. Haven Module

### 4.1 haven/types.rs (720 LOC)

**Structure**:
- Re-exports from `bonus` and `room_defs` modules: ~5 LOC
- `Haven` struct + Default impl: ~25 LOC
- Haven methods: `room_tier`, `is_room_unlocked`, `can_build`, `next_tier`, `fishing_rank_bonus`, `can_forge_stormbreaker`, `craft_stormbreaker`, `total_prestige_spent`, `vault_capacity`, `has_storm_forge`: ~120 LOC
- Tests: ~570 LOC

**Note**: Prior refactoring already extracted `room_defs.rs` (room metadata) and `bonus.rs` (bonus types and calculation).

**Recommendation**: No further extraction. The Haven struct methods are cohesive operations on the same data structure. The file size is appropriate post-refactoring.

---

## 5. Combat Types

### 5.1 combat/types.rs (670 LOC)

**Structure**:
- `Enemy` struct + methods: ~50 LOC
- `generate_enemy_name()`: ~20 LOC
- Zone enemy generation functions: ~300 LOC (zone_base_stats, generate_zone_enemy, generate_subzone_boss, generate_enemy_for_current_zone, generate_boss_for_current_zone, generate_dungeon_enemy/elite/boss)
- `get_zone_enemy_suffixes()`: 104 LOC — match on zone_id returning suffix arrays
- `CombatState` struct: ~50 LOC
- Tests: ~150 LOC

**Long function**: `get_zone_enemy_suffixes()` at 104 lines — one match arm per zone ID (11 zones), each with an array of ~8 suffix strings. This is pure data.

**Recommendation**:
- Extract enemy generation functions to `combat/enemy_generation.rs`: `zone_base_stats`, `generate_zone_enemy`, `generate_subzone_boss`, `generate_*_for_current_zone`, `generate_dungeon_*`, `get_zone_enemy_suffixes`, `generate_enemy_name`. (~430 LOC)
- Keep `Enemy` struct and `CombatState` struct in `types.rs` (~100 LOC)
- This follows the established pattern of `types.rs` for data structures and separate files for generation logic

---

## 6. UI Modules

### 6.1 enemy_sprites.rs (1428 LOC)

**Structure**:
- `EnemySprite` struct: ~23 LOC
- 8 legacy sprite constants (`SPRITE_INSECT` etc.): ~190 LOC
- Half-block sprite constants (10+ archetypes): ~450 LOC
- `get_enemy_sprite()` function: ~50 LOC
- `archetype_for_suffix()`: 107 LOC — maps enemy suffix to sprite archetype
- `suffix_is_known_for_zone()`: 149 LOC — maps zone_id to known enemy suffixes
- `get_zone_enemy_color()`: ~30 LOC
- Zone-specific sprite rendering helpers: ~100 LOC
- Tests: ~150 LOC

**Long functions**: `archetype_for_suffix()` (107) and `suffix_is_known_for_zone()` (149) — both are large match/map tables. Pure data mapping.

**Recommendation**:
- Extract sprite data constants to `enemy_sprite_data.rs` (~640 LOC: legacy sprites + half-block sprites + archetype/suffix mapping tables)
- Keep `get_enemy_sprite()` and rendering helpers in `enemy_sprites.rs` (~350 LOC)
- This separates static data from rendering logic

### 6.2 zone_bg.rs (1239 LOC)

**Structure**:
- Types: `TerrainProfile`, `CelestialType`, `WeatherType`, `ZoneSceneConfig`, `GroundDetail`: ~80 LOC
- 11 zone config functions (one per zone): ~550 LOC total (~50 LOC each)
- `get_zone_config()` dispatcher: ~30 LOC
- Rendering pipeline: `render_zone_background()`, `paint_sky_gradient`, `paint_celestial`, `paint_terrain`, `paint_ground_details`, `paint_weather`: ~350 LOC
- Overlay functions (lightning, void, etc.): ~230 LOC

**Recommendation**: No split recommended. The zone configs are data (could move to separate file), but they're tightly coupled with the rendering pipeline's struct types. The file is well-structured with clear layer separation. At 1239 LOC it's large but each section is cohesive.

### 6.3 stats_panel.rs (785 LOC)

**Structure**:
- Prior refactoring already extracted: `stats_attributes.rs`, `stats_equipment.rs`, `stats_prestige.rs`
- `draw_stats_panel()` coordinator: ~53 LOC
- `draw_header()`: 84 LOC
- `draw_footer()`: 107 LOC
- Helper functions: `format_play_time`, `highest_level_badge`, `draw_compact_stats_bar`, `draw_m_attributes`, `draw_m_xp_bar`: ~200 LOC
- Compact/M-tier renderers: ~150 LOC
- Tests: ~80 LOC

**Long functions**: `draw_header()` (84) and `draw_footer()` (107).

**Recommendation**: The prior refactoring already did the right thing. The remaining `draw_header()` and `draw_footer()` are within acceptable bounds. No further extraction needed.

### 6.4 flappy_scene.rs (778 LOC)

**Structure**:
- `render_flappy_scene()` orchestrator: ~30 LOC
- `render_play_field()`: **500 LOC** — builds ASCII bird, pipes, background, ground, renders to buffer
- `render_game_over()`: ~50 LOC
- `render_waiting_screen()`: ~50 LOC
- `render_info_panel()`: ~50 LOC
- `render_status_bar_content()`: ~30 LOC

**Critical long function**: `render_play_field()` at 500 lines is the largest function in the codebase.

**Recommendation**: Split `render_play_field()` into sub-functions:
- `render_bird()` — bird sprite and animation (~80 LOC)
- `render_pipes()` — pipe obstacle drawing (~100 LOC)
- `render_ground()` — ground layer (~40 LOC)
- `render_sky_and_background()` — background gradient (~60 LOC)
- Keep `render_play_field()` as orchestrator calling these sub-functions (~50 LOC)

These can stay in the same file — the goal is function decomposition, not file splitting.

### 6.5 fishing_scene.rs (715 LOC)

**Structure**:
- `render_fishing_scene()` orchestrator: ~25 LOC
- `draw_header()`: ~15 LOC
- `draw_water_scene()`: **406 LOC** — procedural ASCII water animation, bobber, fish sprites
- `draw_catch_progress()`: ~40 LOC
- `render_leviathan_encounter_modal()`: 99 LOC
- Helper functions: ~30 LOC

**Long function**: `draw_water_scene()` at 406 lines builds a multi-layer animated ASCII scene.

**Recommendation**: Similar to flappy_scene, decompose `draw_water_scene()`:
- `render_water_surface()` — wave animation and surface detail
- `render_underwater()` — underwater gradient and particles
- `render_bobber()` — bobber position and animation
- `render_fish_sprite()` — fish catch animation
- Keep `draw_water_scene()` as orchestrator

These can stay in the same file.

### 6.6 haven_scene.rs (684 LOC)

**Long functions**:
- `render_vault_selection()`: 218 LOC — renders full item details with attributes, affixes, descriptions
- `render_forge_confirmation()`: 89 LOC

**Recommendation**: No split recommended — the file was already refactored (haven_details.rs, haven_tree.rs extracted). At 684 LOC it's manageable. The `render_vault_selection()` function is long because it formats complex item data, but splitting would scatter context.

### 6.7 ui/mod.rs (703 LOC)

**Structure**:
- Module declarations: ~45 LOC
- `rarity_color()`: ~15 LOC
- `draw_ui_with_update()` entry point: ~60 LOC
- `draw_xl_l_layout()`: 112 LOC
- `draw_m_layout()`: ~80 LOC
- `draw_s_layout()`: ~80 LOC
- Scene dispatch helpers: ~100 LOC
- `minigame_priority_check()`: ~50 LOC
- Other helpers: ~80 LOC

**Recommendation**: No further split needed. This is the UI coordinator and needs to see all scene modules. At 703 LOC it's within bounds.

### 6.8 character_select.rs (672 LOC)

**Long function**: `draw_character_details()` at 106 LOC — formats character preview with attributes, equipment, prestige info.

**Recommendation**: No split needed — the file is a single concern (character selection UI) at a manageable size.

---

## 7. Summary of Recommended Refactorings

### High Value (Dev 3 — Challenge Refactoring)

| Action | Files Affected | LOC Reduction | Difficulty |
|--------|---------------|---------------|------------|
| **Macro or trait for `apply_game_result`** | All 10 challenge logic.rs files | ~300 LOC saved | Medium |
| **Extract morris/ai.rs** | morris/logic.rs | logic.rs: 1622 -> ~900 (+ ai.rs ~370 + tests) | Easy |

### Medium Value (Dev 4 — Achievements/Character/Haven)

| Action | Files Affected | LOC Reduction | Difficulty |
|--------|---------------|---------------|------------|
| **No further splits needed** | achievements, character, haven | Already well-refactored | N/A |

### Medium Value (Dev 5 — UI and Combat Types)

| Action | Files Affected | LOC Reduction | Difficulty |
|--------|---------------|---------------|------------|
| **Extract combat/enemy_generation.rs** | combat/types.rs | types.rs: 670 -> ~240 (+ enemy_generation.rs ~430) | Easy |
| **Extract ui/enemy_sprite_data.rs** | ui/enemy_sprites.rs | enemy_sprites.rs: 1428 -> ~790 (+ data ~640) | Easy |
| **Decompose render_play_field() in flappy_scene.rs** | ui/flappy_scene.rs | Same file, 500 LOC fn -> 5 smaller fns | Easy |
| **Decompose draw_water_scene() in fishing_scene.rs** | ui/fishing_scene.rs | Same file, 406 LOC fn -> 5 smaller fns | Easy |

### Low Value (Not Recommended)

| Action | Reason |
|--------|--------|
| Shared forfeit handler | Too interleaved with game-specific logic |
| Split achievements/data.rs | Pure static data, splitting adds overhead with no benefit |
| Split zone_bg.rs | Cohesive rendering pipeline, splitting scatters context |
| Extract menu.rs DifficultyInfo impls | Marginal benefit, adds import complexity |
| Split minesweeper/logic.rs tests | Tests are 80% of file but comprehensive and valuable |

---

## 8. Dependency Notes for Implementation

### Challenge `apply_game_result` Refactoring

If using a macro approach:
```rust
// In challenges/mod.rs
macro_rules! impl_apply_game_result {
    ($variant:ident, $result_type:ty, $game_type:expr, $icon:expr,
     $win_msg:expr, $loss_msg:expr, $forfeit_msg:expr $(, $draw_msg:expr)?) => {
        pub fn apply_game_result(
            state: &mut crate::core::game_state::GameState,
        ) -> Option<crate::challenges::MinigameWinInfo> {
            use crate::challenges::menu::DifficultyInfo;
            use crate::challenges::{apply_challenge_rewards, GameResultInfo, ActiveMinigame};

            let game = match state.active_minigame.as_ref() {
                Some(ActiveMinigame::$variant(g)) => g,
                _ => return None,
            };
            let result = game.game_result?;
            let difficulty = game.difficulty;
            let reward = difficulty.reward();
            let forfeit = game.forfeit_pending;
            // ... etc
        }
    };
}
```

### Morris AI Extraction

The key dependency: `make_move_for_search` and `unmake_move` call `game.forms_mill()` and read `game.board`, `game.pieces_to_place`, etc. — all methods/fields on `MorrisGame`. The AI module needs `use super::{MorrisGame, MorrisMove, MorrisPhase, MorrisResult, Player, MILLS, ADJACENCIES}`.

The `end_turn_for_search` function needs access to `get_legal_moves` from logic.rs, creating a circular dependency. Solution: keep `get_legal_moves` in logic.rs and import it from ai.rs, or move `get_legal_moves` to types.rs as a method on `MorrisGame`.

### Combat Enemy Generation Extraction

Clean dependency: `enemy_generation.rs` needs `use super::types::{Enemy, CombatState}` and `use crate::core::constants::*` and `use crate::zones::*`. No circular dependencies.

### UI Enemy Sprite Data Extraction

Clean dependency: data file just defines constants. The rendering file imports from data.
