# Runic Lights — Design Spec

## Overview

Runic Lights is a Lights Out puzzle challenge. The player is presented with a grid of runes, some lit, some dark. Toggling a rune flips it and its 4 orthogonal neighbors. Goal: extinguish all runes.

## Core Mechanic

- Toggle a cell: flips that cell + up/down/left/right neighbors (cross pattern)
- Cells on edges/corners have fewer neighbors (2-3 instead of 4)
- Win condition: all cells dark
- Lose condition: exceed move limit (3x par)

## Difficulty Tiers

| Tier | Grid | Solution Depth | Par | Move Limit (3x par) |
|------|------|---------------|-----|---------------------|
| Novice | 3x3 | 3-5 toggles | 5 | 15 |
| Apprentice | 4x4 | 5-8 toggles | 8 | 24 |
| Journeyman | 5x5 | 8-12 toggles | 12 | 36 |
| Master | 6x6 | 12-16 toggles | 16 | 48 |

- Par = maximum solution depth for the tier (upper bound of the range)
- Move limit = 3x par, generous enough that only aimless clicking triggers a loss

## Puzzle Generation

Reverse generation ensures solvability:

1. Start from all-off (solved) state
2. Apply N random unique cell toggles (N = solution depth, sampled from tier range)
3. Each toggle flips the cross pattern, producing the starting board
4. The puzzle is guaranteed solvable in at most N moves
5. Par is set to the tier's upper bound (not the actual N), keeping it consistent per difficulty

"Unique cell toggles" means each cell is toggled at most once during generation (toggling the same cell twice cancels out). This guarantees the puzzle is solvable in at most N moves. Note: for 5x5+ boards, the null space of the toggle matrix over GF(2) means shorter solutions may exist. Par is set conservatively to the tier's upper bound, so this works in the player's favor.

## Input

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Enter | Toggle cell (flip cross pattern) |
| Esc | Forfeit (double-tap to confirm) |

No undo. Players commit to their moves.

## UI Layout

Uses `create_game_layout()` with the standard game layout pattern:

```
+-- Runic Lights ---------------------+-- Info ----------+
|                                     |                  |
|      [grid with cursor]             |  Difficulty      |
|                                     |  Journeyman      |
|      lit = filled diamond           |                  |
|      dark = hollow diamond          |  Grid: 5x5      |
|      cursor = yellow highlight      |                  |
|                                     |  Moves: 7/36    |
|                                     |  Par: 12         |
|                                     |                  |
|                                     |  Lit: 13/25     |
|                                     |                  |
| Extinguishing...   [Arrows] Move  [Enter] Toggle  [Esc] Forfeit |
+---------------------------------+--+------------------+
```

- Border color: Cyan
- Lit cells: filled diamond, bright color
- Dark cells: hollow diamond, dim color
- Cursor: yellow background highlight
- Info panel: difficulty, grid size, moves/limit, par, lit count

## Win/Loss Screens

**Win:** "All runes extinguished!" with move count vs par. Under-par gets a star indicator.

**Loss:** "Too many moves!" when move count exceeds 3x par.

## Rewards

| Tier | Stormglass | Prestige | Fishing |
|------|-----------|----------|---------|
| Novice | 400 | 0 | 0 |
| Apprentice | 1,200 | 0 | 0 |
| Journeyman | 3,000 | 1 | 0 |
| Master | 6,000 | 2 | 0 |

Reward curve matches Sudoku (comparable moderate puzzle).

## Achievements

4 achievements following the standard challenge pattern:

| Achievement ID | Name | Description | Points |
|---------------|------|-------------|--------|
| RunicLightsNovice | Runic Lights Novice | Extinguish all runes on Novice difficulty | 10 |
| RunicLightsApprentice | Runic Lights Apprentice | Extinguish all runes on Apprentice difficulty | 25 |
| RunicLightsJourneyman | Runic Lights Journeyman | Extinguish all runes on Journeyman difficulty | 50 |
| RunicLightsMaster | Runic Lights Master | Extinguish all runes on Master difficulty | 100 |

All in `AchievementCategory::Challenges`. Icon: diamond symbol.

Wins also increment `total_minigame_wins` toward the Grand Champion milestone (100 wins).

## Module Structure

```
src/challenges/runic_lights/
  mod.rs      — public exports, impl_apply_game_result! macro
  types.rs    — RunicLightsGame, RunicLightsDifficulty, RunicLightsResult, RunicLightsInput
  logic.rs    — puzzle generation, input processing, toggle logic, win/loss checks
```

## Integration Points

1. **ChallengeType::RunicLights** in `src/challenges/menu.rs` — enum variant, discovery weight, icon, description
2. **ActiveMinigame::RunicLights** in `src/challenges/mod.rs` — active game wrapper
3. **MinigameType::RunicLights** in `src/achievements/milestones.rs` — achievement game type
4. **AchievementId** variants in `src/achievements/types.rs` — 4 tier achievements
5. **AchievementDef** entries in `src/achievements/data.rs` — definitions with points
6. **on_minigame_won()** match arms in `src/achievements/handlers.rs` — unlock logic
7. **Input mapping** in `src/input/minigame_input.rs` — KeyCode to RunicLightsInput
8. **UI scene** in `src/ui/runic_lights_scene.rs` — grid rendering
9. **AI facade** in `src/challenges/facade.rs` — no AI needed; `ai_thinking` is always false, `tick_game()` is a no-op
10. **Debug menu** in `src/utils/debug_menu.rs` — debug trigger for testing

## Persistence

`RunicLightsGame` does not need `Serialize`/`Deserialize`. Mid-game quit triggers the standard forfeit warning. If the player force-quits, the challenge is lost (same as all other challenges).

## Discovery Weight

Suggested weight: 20 (moderate, similar to Shard Fusion).
