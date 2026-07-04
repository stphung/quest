> Backported design record. Sources: docs/plans/2026-02-27-structural-overhaul-design.md.

## 2026-02-27-structural-overhaul-design.md

# Structural Overhaul Design — Module Facades + Decomposed State

**Date**: 2026-02-27
**Approach**: B (Module Facades + Decomposed State)
**Risk tolerance**: Conservative — preserve all public APIs, existing tests pass unchanged
**Scope**: Full structural overhaul (GameState, tick engine, challenges, UI)

---

## Problem Statement

The Quest codebase (107K LOC, 228 source files) has grown organically. Prior refactoring (PRs #292-294) extracted submodules well, but structural issues remain:

1. **GameState god object**: 40+ field struct that every module reaches into
2. **Tick engine coupling**: `game_tick()` takes 8 parameters and hardcodes 14 stages
3. **Challenge duplication**: 10 minigames repeat identical `apply_game_result` and forfeit patterns (~400+ lines)
4. **UI monolith**: 31K lines with render functions in the 400-600+ line range
5. **Implicit dependencies**: modules import `GameState` and access arbitrary fields

## Design Decisions

- **No traits/interfaces for system registration** — fights Rust's ownership model for marginal benefit
- **No event bus** — over-engineered for a single-threaded 100ms game loop
- **Facade pattern over direct access** — explicit input structs make dependencies visible
- **Deprecated wrappers for backward compat** — existing callers continue working, migrate at their own pace
- **Custom serde for save compat** — flattened JSON format preserved despite struct decomposition

---

## Section 1: GameState Decomposition

Split the monolithic `GameState` into four focused sub-structs, composed via fields.

### New Sub-Structs

```rust
pub struct PlayerIdentity {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
}

pub struct CombatContext {
    pub combat_state: CombatState,
    pub equipment: Equipment,
    pub zone_progression: ZoneProgression,
    pub active_dungeon: Option<Dungeon>,
    pub session_kills: u64,
    pub consecutive_deaths: u32,
}

pub struct ProgressionState {
    pub fishing: FishingState,
    pub active_fishing: Option<FishingSession>,
    pub stormglass: u64,
    pub stormglass_discovered: bool,
    pub storm_sigils: StormSigils,
    pub challenge_menu: ChallengeMenu,
    pub chess_stats: ChessStats,
    pub active_minigame: Option<ActiveMinigame>,
}

pub struct SessionState {
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub chrono_surge_active: bool,
    pub recent_drops: VecDeque<RecentDrop>,
    pub last_minigame_win: Option<MinigameWinInfo>,
    pub xp_rate_samples: VecDeque<u64>,
    pub xp_this_second: u64,
    pub ticker: Ticker,
    pub cached_derived_stats: DerivedStats,
    pub cached_prestige_bonuses: PrestigeCombatBonuses,
    pub derived_stats_dirty: bool,
    pub combat_seconds_this_tick: bool,
    pub game_over_shown_at: Option<Instant>,
}
```

### Composed GameState

```rust
pub struct GameState {
    pub player: PlayerIdentity,
    pub combat: CombatContext,
    pub progression: ProgressionState,
    pub session: SessionState,
}
```

### Backward Compatibility

- `#[deprecated]` accessor methods on `GameState` delegate to sub-structs (e.g., `state.character_level()` returns `state.player.character_level`)
- Custom `Serialize`/`Deserialize` impls flatten sub-structs to preserve existing JSON save format
- Existing code compiles unchanged via deprecated accessors; new code uses grouped paths

---

## Section 2: Module Facade Pattern

Each game system gets a facade function with explicit input structs instead of taking `&mut GameState`.

### Pattern

```rust
// fishing/facade.rs
pub struct FishingInput<'a> {
    pub fishing: &'a mut FishingState,
    pub active_fishing: &'a mut Option<FishingSession>,
    pub player_level: u32,
    pub prestige_rank: u32,
    pub haven_bonuses: HavenFishingBonuses,
}

pub fn tick_fishing(input: &mut FishingInput, rng: &mut impl Rng) -> FishingTickResult {
    // delegates to existing internal logic
}
```

### Facade Inventory

| Module | Facade function | Key inputs |
|--------|----------------|------------|
| `fishing` | `tick_fishing(FishingInput, rng)` | FishingState, level, prestige, haven bonuses |
| `dungeon` | `tick_dungeon(DungeonInput, rng)` | Dungeon, zone_id, equipment, prestige |
| `combat` | `update_combat(CombatInput, rng)` | CombatState, bonuses, derived stats |
| `challenges` | `tick_challenge_ai(ActiveMinigame)` | Just the active minigame |
| `deep` | `tick_deep(DeepInput)` | DeepState, prestige_rank |
| `discoveries` | `roll_discoveries(DiscoveryInput, rng)` | prestige_rank, zone, flags |

### Integration

- `game_tick()` constructs input structs from `GameState` sub-structs and calls facades
- No module imports `GameState` directly — they only see their input struct
- Existing public functions remain as deprecated wrappers

---

## Section 3: Challenge System Standardization

### 3a. `impl_apply_game_result!` Macro

Replaces ~35 lines per challenge with ~5 lines:

```rust
// challenges/mod.rs
macro_rules! impl_apply_game_result {
    ($variant:ident, $result_type:ty, $minigame_type:expr,
     $icon:expr, $win_msg:expr, $loss_msg:expr) => {
        pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
            let game = match state.active_minigame.as_ref() {
                Some(ActiveMinigame::$variant(g)) => g,
                _ => return None,
            };
            let result = game.game_result?;
            let difficulty = game.difficulty;
            let (won, loss_message) = /* match on result variants */;
            apply_challenge_rewards(state, GameResultInfo {
                won, minigame_type: $minigame_type,
                icon: $icon, win_message: $win_msg,
                loss_message, difficulty_name: difficulty.name(),
                reward: difficulty.reward(),
            })
        }
    };
}

// Usage in chess/logic.rs:
impl_apply_game_result!(Chess, ChessResult, MinigameType::Chess,
    "\u{265A}", "Checkmate!", "You were checkmated");
```

### 3b. Shared Forfeit Handler

```rust
pub fn handle_forfeit<R>(game_result: &mut Option<R>, forfeit_pending: &mut bool, loss: R) -> bool {
    if *forfeit_pending {
        *game_result = Some(loss);
        true
    } else {
        *forfeit_pending = true;
        false
    }
}

pub fn cancel_forfeit(forfeit_pending: &mut bool) {
    *forfeit_pending = false;
}
```

### Savings

~400+ lines of deduplication across 10 challenges. Canonical pattern established for adding challenge #11.

---

## Section 4: UI Module Decomposition

### 4a. Large Render Function Decomposition

Break 400-600+ line render functions into region-based helpers:

```rust
// AFTER: focused helpers in the same file
pub fn draw_stormglass_overlay(f: &mut Frame, area: Rect, state: &GameState, ...) {
    let layout = compute_stormglass_layout(area);
    draw_stormglass_header(f, layout.header, state);
    draw_sigil_grid(f, layout.grid, &state.storm_sigils);
    draw_sigil_detail(f, layout.detail, selected_sigil);
    draw_stormglass_animation(f, layout.anim, anim_state);
    draw_stormglass_footer(f, layout.footer, state);
}
```

### Target Files

| File | Current LOC | Approach |
|------|-------------|----------|
| `stormglass_scene.rs` | 3,196 | Split into region helpers |
| `deep_missions.rs` | 2,843 | Extract render helpers |
| `time_vault_scene.rs` | 1,291 | Split branch/commit/cloud panels |
| `zone_bg.rs` | 1,239 | One function per zone biome |
| `deep_roster.rs` | 1,333 | Extract card rendering helpers |

### 4b. Shared Layout Helpers

```rust
// ui/layout.rs
pub struct OverlayLayout {
    pub title_bar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub fn overlay_layout(area: Rect, title_height: u16) -> OverlayLayout { ... }

pub struct TwoPanelLayout {
    pub left: Rect,
    pub right: Rect,
}

pub fn two_panel_layout(area: Rect, left_pct: u16) -> TwoPanelLayout { ... }
```

Small composable building blocks, not a framework.

### Unchanged

`scene_fx.rs`, `game_common.rs`, `responsive.rs` — already well-factored.

---

## Section 5: Tick Engine Simplification

### 5a. TickContext Struct

```rust
pub struct TickContext<'a> {
    pub state: &'a mut GameState,
    pub tick_counter: &'a mut u32,
    pub haven: &'a mut Haven,
    pub enhancement: &'a mut EnhancementProgress,
    pub deep: &'a mut DeepState,
    pub achievements: &'a mut Achievements,
    pub debug_mode: bool,
}

pub fn game_tick<R: Rng>(ctx: &mut TickContext, rng: &mut R) -> TickResult {
    let mut result = TickResult::default();
    tick_challenge_ai(ctx, rng);
    tick_challenge_discovery(ctx, &mut result, rng);
    tick_sync_player_hp(ctx);
    tick_dungeon(ctx, &mut result, rng);
    if tick_fishing(ctx, &mut result, rng) { return result; }
    tick_combat(ctx, &mut result, rng);
    tick_enemy_spawn(ctx);
    tick_play_time(ctx);
    tick_achievement_collection(ctx, &mut result);
    tick_haven_discovery(ctx, &mut result, rng);
    tick_soulforge_discovery(ctx, &mut result, rng);
    tick_deep_events(ctx, &mut result);
    tick_achievement_modal(ctx, &mut result);
    result
}
```

### 5b. Named Stage Functions

Each inline stage block becomes a standalone function in `tick_stages.rs`. Each is independently testable.

### 5c. Backward Compatibility

Deprecated wrapper preserving the old 8-param signature delegates to `TickContext` version.

---

## Section 6: Team Structure

### Agents (13 total)

| Role | Agent | Responsibility |
|------|-------|---------------|
| Sys Architect | `sys-arch-1` | GameState decomposition, tick engine, facade pattern |
| Sys Architect | `sys-arch-2` | Challenge standardization, UI decomposition, pattern consistency |
| Developer | `dev-1` | Core: GameState split, TickContext, serde compat |
| Developer | `dev-2` | Facades: fishing, dungeon, combat, discoveries |
| Developer | `dev-3` | Challenges: macro, forfeit helper, 10 challenge migrations |
| Developer | `dev-4` | UI: render decomposition, layout helpers, 5 target files |
| QA | `qa-1` | Core + tick engine tests |
| QA | `qa-2` | Combat + zone + dungeon + fishing tests |
| QA | `qa-3` | Challenge minigame tests (all 10 games) |
| QA | `qa-4` | UI smoke tests, Deep tests, achievement tests, full `make check` |
| Eng Manager | `eng-mgr-1` | Core track: sys-arch-1 → dev-1 → dev-2 → qa-1 → qa-2 |
| Eng Manager | `eng-mgr-2` | Systems track: sys-arch-2 → dev-3 → dev-4 → qa-3 → qa-4 |
| Game Designer | `game-designer` | Gameplay invariant audit across all changes |

### Phasing

```
Phase 1: sys-arch-1 + sys-arch-2 produce designs (parallel)
Phase 2: dev-1 (GameState split) — everything else depends on this
Phase 3: dev-2 + dev-3 + dev-4 (parallel, independent modules)
Phase 4: qa-1 + qa-2 + qa-3 + qa-4 (parallel validation)
Phase 5: game-designer final audit
```

### Constraints

- All existing tests must pass unchanged
- JSON save format must be preserved (custom serde impls)
- No public API removals — only deprecations
- Rendering output must be pixel-identical
- Game balance constants, progression formulas, combat pipeline preserved exactly
