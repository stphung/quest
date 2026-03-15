---
name: add-challenge
description: Complete checklist for adding a new challenge minigame to Quest. Use this skill whenever someone wants to add a new challenge, minigame, puzzle, or game — even if they don't say "challenge" explicitly. Covers all 15 integration points across the codebase so nothing gets missed. Use PROACTIVELY when implementing any new challenge from a spec or plan.
---

# Add Challenge

Complete integration checklist for adding a new challenge minigame to Quest. This skill exists because challenges touch 15+ files across 6 modules, and it's easy to miss one — causing compiler errors or runtime bugs that are hard to trace back to a missing match arm.

The checklist below is ordered so that each step compiles (or at least doesn't break unrelated code). Follow the order.

## Before You Start

You need a design spec or clear description covering:
- Game name and mechanic
- Difficulty scaling (grid size, speed, depth, etc.)
- Win/loss conditions
- Rewards per difficulty tier (stormglass, prestige_ranks, fishing_ranks)
- Achievement names and point values
- Unicode icon for combat log messages
- Discovery weight (see `CHALLENGE_TABLE` in `src/challenges/menu.rs`)

If you don't have a spec, use the `brainstorming` skill first.

## Integration Checklist

### Phase 1: Core Game Logic (3 files to create)

#### 1. Create `src/challenges/<game>/types.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum <Game>Difficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

// Generates ALL, name(), and iteration support
difficulty_enum_impl!(<Game>Difficulty);
```

Also define:
- `<Game>Result` enum (Win, Loss, optionally Draw)
- `<Game>Game` struct with at minimum: `difficulty`, `game_result: Option<Result>`, `forfeit_pending: bool`
- `<Game>Input` enum (Up, Down, Left, Right, Select/Toggle, Forfeit, Other)
- `impl <Game>Game { pub fn new(difficulty) -> Self }` constructor

If the game has no AI, omit `ai_thinking` and `ai_think_ticks` fields.

#### 2. Create `src/challenges/<game>/logic.rs`

Required functions:
- `pub fn process_input(game: &mut <Game>Game, input: <Game>Input)` — must handle forfeit flow first (check `forfeit_pending`, then match input)
- `pub fn start_<game>_game(difficulty, rng) -> ActiveMinigame` (or without rng if not needed)
- `impl DifficultyInfo for <Game>Difficulty` — provides `name()`, `reward()`, `extra_info()`

Use shared forfeit helpers:
```rust
crate::challenges::handle_forfeit(&mut game.game_result, &mut game.forfeit_pending, <Game>Result::Loss);
crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
```

If the game has AI, add `pub fn process_ai_thinking(game, rng)` (use this exact name).

If the game has no AI, `tick_game` is a no-op and `ai_thinking` is always false.

#### 3. Create `src/challenges/<game>/mod.rs`

```rust
pub mod logic;
pub mod types;

pub use logic::*;
pub use types::*;

impl_apply_game_result! {
    variant: <Game>;
    result_body: |result, _state, _reward| {
        use <game>::types::<Game>Result;
        let (won, loss_message) = match result {
            <Game>Result::Win => (true, ""),
            <Game>Result::Loss => (false, "The runes remain!"),
        };
        (won, loss_message)
    }
    game_type: "<game>";
    icon: "\u{XXXX}";
    win_message: "Victory message!";
}
```

### Phase 2: Wire Into Challenge System (3 files to modify)

#### 4. `src/challenges/mod.rs`

- Add `pub mod <game>;` to module declarations
- Add re-export to the `pub use` block
- Add `ActiveMinigame::<Game>(<Game>Game)` variant
- Add match arm in `has_game_result()`: `ActiveMinigame::<Game>(g) => g.game_result.is_some()`

#### 5. `src/challenges/menu.rs`

- Add `ChallengeType::<Game>` variant to enum
- Add import for `<game>::types::<Game>Difficulty`
- Add arm in `accept_selected_challenge()` to call `start_<game>_game()`
- Add entry in `create_challenge()` with icon, description, discovery flavor text
- Add entry in `CHALLENGE_TABLE` with discovery weight

#### 6. `src/challenges/facade.rs`

- If game has AI: add match arm in `tick_challenge_ai_facade()` calling `process_ai_thinking()`
- If no AI: add match arm with empty body `=> {}` (required — match is exhaustive)

### Phase 3: Input & UI (4 files to create/modify)

#### 7. `src/input/minigame_input.rs`

Add match arm in `handle_minigame()`:
```rust
ActiveMinigame::<Game>(game) => {
    if game.game_result.is_some() {
        apply_<game>_result(state);
        return InputResult::Continue;
    }
    let input = match key.code {
        KeyCode::Up => <Game>Input::Up,
        KeyCode::Down => <Game>Input::Down,
        KeyCode::Left => <Game>Input::Left,
        KeyCode::Right => <Game>Input::Right,
        KeyCode::Enter => <Game>Input::Toggle, // or Select
        KeyCode::Esc => <Game>Input::Forfeit,
        _ => <Game>Input::Other,
    };
    process_input(game, input);
}
```

#### 8. Create `src/ui/<game>_scene.rs`

Use the standard game layout pattern:
```rust
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_status_bar, GameResultType,
};
```

Key functions:
- `pub fn render_<game>(frame, area, game, ctx, show_dismiss_hint)` — main entry
- Private `render_game_over()` — calls `render_game_over_overlay()` with 7 args: frame, area, result_type, title, message, reward, show_dismiss_hint
- Private `render_board()` / `render_grid()` — game content
- Private `render_info()` — info panel (difficulty, score, etc.)
- Private `render_status_bar_content()` — forfeit check, then normal controls

The `render_game_over_overlay()` title and message args are `String`, not `&str`. The reward arg uses `format!()`.

Each game gets a unique border color (check existing games to avoid duplicates).

#### 9. `src/ui/mod.rs`

- Add `mod <game>_scene;` declaration
- Add match arm in `render_active_minigame()` dispatching to `<game>_scene::render_<game>()`

#### 10. `src/ui/challenge_menu_scene.rs`

Add match arm in the difficulty selector block (around line 155) and in `preferred_difficulty_height()`:
```rust
ChallengeType::<Game> => {
    render_difficulty_selector(frame, detail_area, challenge, <Game>Difficulty::ALL, ctx);
}
```

### Phase 4: Achievements (3 files to modify)

#### 11. `src/achievements/milestones.rs`

Add variant to `MinigameType` enum:
```rust
pub enum MinigameType {
    // ...existing...
    <Game>,
}
```

#### 12. `src/achievements/types.rs`

- Add 4 `AchievementId` variants: `<Game>Novice`, `<Game>Apprentice`, `<Game>Journeyman`, `<Game>Master`
- Update `VARIANT_COUNT` (add 4 to the current value)

#### 13. `src/achievements/data.rs`

Add 4 `AchievementDef` entries following the standard pattern:
- Points: 10 (Novice), 25 (Apprentice), 50 (Journeyman), 100 (Master)
- Category: `AchievementCategory::Challenges`
- Description: "[Action verb] on [Difficulty] difficulty"

#### 14. `src/achievements/handlers.rs`

Add 4 match arms in `on_minigame_won()`:
```rust
(MinigameType::<Game>, "novice") => self.try_unlock(AchievementId::<Game>Novice, notifications),
(MinigameType::<Game>, "apprentice") => self.try_unlock(AchievementId::<Game>Apprentice, notifications),
(MinigameType::<Game>, "journeyman") => self.try_unlock(AchievementId::<Game>Journeyman, notifications),
(MinigameType::<Game>, "master") => self.try_unlock(AchievementId::<Game>Master, notifications),
```

### Phase 5: Remaining Integration (2 files to modify)

#### 15. `src/stormglass/spending.rs`

Add match arm in `challenge_type_name()`:
```rust
ChallengeType::<Game> => "<Full Display Name>",
```

#### 16. `src/ui/achievement_details.rs`

Add entry to the `challenge_games` array in the challenges mastered stats section (around line 545):
```rust
(
    "<Display Name>",
    [
        AchievementId::<Game>Novice,
        AchievementId::<Game>Apprentice,
        AchievementId::<Game>Journeyman,
        AchievementId::<Game>Master,
    ],
),
```

#### 17. `src/utils/debug_menu.rs`

Add a "Trigger <Game> Challenge" option to `DEBUG_OPTIONS` and a handler function.

## Verification

After all integration points are wired:

```bash
make check   # Runs fmt, clippy, tests, build, audit
```

Common issues:
- **Missing match arm**: The compiler will tell you exactly which file and match. These are exhaustive matches.
- **VARIANT_COUNT wrong**: Achievement tests assert the count matches. If tests fail with a count mismatch, update the constant.
- **Import errors**: Each new type needs to be imported where it's used. Follow the existing import patterns in each file.

## Quick Reference: Files Touched

| # | File | Action | What |
|---|------|--------|------|
| 1 | `src/challenges/<game>/types.rs` | Create | Difficulty, Game, Result, Input types |
| 2 | `src/challenges/<game>/logic.rs` | Create | Game logic, input processing, DifficultyInfo |
| 3 | `src/challenges/<game>/mod.rs` | Create | Re-exports, impl_apply_game_result! |
| 4 | `src/challenges/mod.rs` | Modify | pub mod, ActiveMinigame variant, has_game_result |
| 5 | `src/challenges/menu.rs` | Modify | ChallengeType, create/accept/table |
| 6 | `src/challenges/facade.rs` | Modify | AI tick dispatch (even if no-op) |
| 7 | `src/input/minigame_input.rs` | Modify | Key-to-input mapping |
| 8 | `src/ui/<game>_scene.rs` | Create | Rendering |
| 9 | `src/ui/mod.rs` | Modify | Scene dispatch |
| 10 | `src/ui/challenge_menu_scene.rs` | Modify | Difficulty selector |
| 11 | `src/achievements/milestones.rs` | Modify | MinigameType variant |
| 12 | `src/achievements/types.rs` | Modify | 4 AchievementId variants + VARIANT_COUNT |
| 13 | `src/achievements/data.rs` | Modify | 4 AchievementDef entries |
| 14 | `src/achievements/handlers.rs` | Modify | 4 on_minigame_won arms |
| 15 | `src/stormglass/spending.rs` | Modify | challenge_type_name() |
| 16 | `src/ui/achievement_details.rs` | Modify | Challenges mastered display |
| 17 | `src/utils/debug_menu.rs` | Modify | Debug trigger |
