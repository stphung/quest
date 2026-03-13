# Shard Fusion — Design Doc

**Date:** 2026-03-13
**Type:** New Challenge Minigame

## Overview

Shard Fusion is a 2048-style challenge minigame. The player slides tiles on a 4×4 grid, merging matching values to reach a target tile. Higher difficulties require reaching a higher target tile. No AI — pure puzzle skill.

## Gameplay

Standard 2048 rules:

- 4×4 grid, starts with 2 tiles (value 2 or 4)
- Arrow keys slide all tiles in one direction; adjacent matching tiles merge and double
- After each valid move, a new tile spawns in a random empty cell (90% → 2, 10% → 4)
- **Win**: reach the target tile value for the selected difficulty
- **Loss**: no valid moves remain (board full, no adjacent matches)
- **Score**: sum of all merge values (display only, no gameplay impact)

## Difficulty Tiers

| Difficulty | Target | PR Reward | Stormglass Reward |
|------------|--------|-----------|-------------------|
| Novice | 512 | 1 | 1,000 |
| Apprentice | 1024 | 1 | 2,500 |
| Journeyman | 2048 | 1 | 5,000 |
| Master | 4096 | 2 | 10,000 |

No move cap. The game ends naturally when no moves remain. Difficulty comes purely from the higher target.

## Animation

Each move triggers a two-phase animation sequence. Input is blocked during animation.

1. **Slide phase (~4 ticks / 400ms)**: tiles interpolate visually from their source cells to their destination cells
2. **Flash phase (~3 ticks / 300ms)**: merged cells render with a bright highlight color
3. New tile spawns, input unblocked

## Game State

```rust
pub struct ShardFusionGame {
    pub difficulty: ShardFusionDifficulty,
    pub board: [[u32; 4]; 4],
    pub anim_state: ShardFusionAnimState,  // Idle | Sliding(u32) | Flashing(u32)
    pub slide_moves: Vec<TileMove>,        // (from, to, value) for slide rendering
    pub merged_cells: Vec<(usize, usize)>, // cells that just merged (for flash)
    pub score: u32,
    pub game_result: Option<ShardFusionResult>,
    pub forfeit_pending: bool,
}
```

## UI

- **Layout**: standard `create_game_layout`
- **Main area**: 4×4 tile grid; tiles color-coded by value (dark for low, bright/saturated for high); empty cells shown as blank
- **Info panel**: current score, target tile, highest tile currently on board
- **Status bar**: `[Arrows] Slide  [Esc] Forfeit`
- **Border color**: `Color::Yellow`
- **Game over**: standard `render_game_over_overlay`

Tile color progression by value: `DarkGray` (2/4) → `White` (8/16) → `Yellow` (32/64) → `LightGreen` (128/256) → `LightCyan` (512/1024) → `LightMagenta` (2048/4096).

## Integration

- **Module**: `src/challenges/shard_fusion/` (`mod.rs`, `types.rs`, `logic.rs`)
- **UI scene**: `src/ui/shard_fusion_scene.rs`
- **Discovery weight**: 20 (same tier as Flappy Bird / Runic Shift)
- **`ActiveMinigame`**: `ShardFusion(ShardFusionGame)` variant
- **`ChallengeType`**: `ShardFusion` variant
- **Input**: `src/input/minigame_input.rs` — ignores input when `anim_state != Idle`
- **Debug menu**: "Trigger Shard Fusion Challenge" option
- **Achievements**: emits `MinigameWinInfo { game_type: "shard_fusion", difficulty: "..." }` on win
- **Macro**: uses `impl_apply_game_result!` in `mod.rs`
