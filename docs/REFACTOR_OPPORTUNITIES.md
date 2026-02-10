# Refactoring & DRY Opportunities

Analysis of the Quest codebase for code duplication and refactoring candidates.

## High Impact: Challenge Minigame System

The 6 challenge minigames (chess, go, morris, gomoku, minesweeper, rune) account for ~80% of identified duplication. Nearly every layer is copy-pasted with minor variations.

### 1. `apply_game_result()` — ~500 duplicated lines

Each game's `logic.rs` has a near-identical function that extracts the game from `ActiveMinigame`, calculates XP reward, grants prestige ranks, writes combat log entries, and returns `MinigameWinInfo`. Only flavor text differs.

**Files:**
- `src/challenges/chess/logic.rs`
- `src/challenges/go/logic.rs`
- `src/challenges/morris/logic.rs`
- `src/challenges/gomoku/logic.rs`
- `src/challenges/minesweeper/logic.rs`
- `src/challenges/rune/logic.rs`

**Fix:** Extract a shared `apply_minigame_result()` that takes a trait object or closure for game-specific parts (flavor text, result extraction). The XP calculation, prestige granting, and `MinigameWinInfo` construction are 100% identical.

### 2. 6 Identical Difficulty Enums

Every game defines `{Game}Difficulty { Novice, Apprentice, Journeyman, Master }` plus matching `to_str()` blocks.

**Files:** All 6 `types.rs` files + all 6 `logic.rs` files (match blocks)

**Fix:** Single `ChallengeDifficulty` enum in `src/challenges/mod.rs`. Per-game reward differentiation already exists in `DifficultyInfo` trait implementations.

### 3. 6 Similar Result Enums

All share Win/Loss; some add Draw and/or Forfeit. The UI layer already has a unified `GameResultType` in `game_common.rs:152`.

**Fix:** Unified `ChallengeResult { Win, Loss, Draw, Forfeit }` usable on both logic and UI sides.

### 4. 6 Similar Input Enums + Duplicated Routing

Each game defines its own input enum (Up, Down, Left, Right, action, cancel, other). `input.rs:461-568` has 6 near-identical blocks mapping `KeyCode` to these.

**Fix:** Shared `MinigameInput` enum with `Primary`/`Secondary` action slots. Single mapping function in `input.rs`.

### 5. Forfeit Double-Esc Pattern — ~70 duplicated lines

Identical `if game.forfeit_pending { match ... }` block in every `process_input()`.

**Fix:** Shared function or trait method, dependent on unifying input/result enums.

### 6. `start_xxx_game()` — 5 identical 2-liners

```rust
state.active_minigame = Some(ActiveMinigame::Xxx(XxxGame::new(difficulty)));
state.challenge_menu.close();
```

**Fix:** Generic start function parameterized by game constructor.

### Recommended Approach

Introduce a `Minigame` trait:

```rust
trait Minigame {
    fn game_result(&self) -> Option<ChallengeResult>;
    fn difficulty(&self) -> ChallengeDifficulty;
    fn forfeit_pending(&self) -> bool;
    fn set_forfeit_pending(&mut self, pending: bool);
    fn flavor_text(&self) -> MinigameFlavorText;
    fn process_input(&mut self, input: MinigameInput);
}
```

This collapses most per-game boilerplate while preserving unique board logic.

## Medium Impact

### 7. UI Scene Orchestration — ~100 duplicated lines

All 6 `render_xxx_scene()` functions follow the same structure: check game-over → create layout → render board → render status bar → render info panel. The status bar priority chain (AI thinking → forfeit → normal) is repeated 6 times.

**Files:** All 6 `src/ui/*_scene.rs` minigame files

**Fix:** Generic `render_minigame_scene()` that takes closures/trait methods for board rendering and status text.

### 8. Game-Over Rendering — ~200 duplicated lines

Each scene maps its result enum to `GameResultType`, constructs title/message/reward text, calls the shared overlay.

**Files:** All 6 `src/ui/*_scene.rs` minigame files

**Fix:** If result enums are unified (#3), this mapping disappears. A single `render_minigame_game_over()` can handle all games.

### 9. Save/Load JSON Boilerplate — ~60 duplicated lines

Three modules implement the same pattern: resolve `~/.quest/xxx.json`, deserialize with `unwrap_or_default()`, serialize with `to_string_pretty()`.

**Files:**
- `src/character/manager.rs:55-99`
- `src/haven/logic.rs:70-95`
- `src/achievements/persistence.rs:11-48`

**Fix:** Generic helpers in a shared persistence module:
```rust
fn save_json<T: Serialize>(filename: &str, data: &T) -> io::Result<()>
fn load_json_or_default<T: Default + DeserializeOwned>(filename: &str) -> T
```

### 10. Common Game Struct Fields

Every minigame struct repeats `difficulty`, `game_result`, `forfeit_pending`, `cursor`, and often `ai_thinking`/`ai_think_ticks`.

**Fix:** `MinigameCommon` struct embedded via composition.

## Low Impact

### 11. Parallel Rarity Systems

`items::Rarity` (Common/Magic/Rare/Epic/Legendary) vs `fishing::FishRarity` (Common/Uncommon/Rare/Epic/Legendary). Differ only in second-tier naming.

**Fix:** Shared base enum with display-name customization, or accept the minor divergence.

### 12. Duplicated Color Constants

`human_color`, `ai_color`, `cursor_color` etc. redefined in `gomoku_scene.rs`, `go_scene.rs`, `morris_scene.rs`.

**Fix:** Constants in `game_common.rs`.

### 13. Name Generation Pattern

Enemy, fish, and item name generation all use `rng.gen_range(0..array.len())` pick-from-array.

**Fix:** Small `pick_random` helper, or accept since the pattern is trivial.

## Intentionally Not Refactored

- **Haven bonus structs** (`HavenCombatBonuses`, `HavenFishingBonuses`): Separate by design for module decoupling.
- **Module directory structure** (`types.rs`/`logic.rs`/`generation.rs`): Good convention, not duplication.

## Priority Order

1. Unify difficulty/result/input enums (low effort, high payoff, unblocks other refactors)
2. Extract shared `apply_minigame_result()` (high duplication, medium effort)
3. Generic save/load helpers (low effort, immediate payoff)
4. UI scene orchestration (medium effort, cleans up 6 files)
5. Minigame trait + common struct (larger effort, biggest long-term benefit)
