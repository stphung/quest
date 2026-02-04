# Rune Deciphering Challenge Design

## Overview

A Mastermind-style logic/deduction minigame themed as decoding ancient rune tablets. The player guesses hidden sequences of runes and receives feedback on correctness. Discovered while adventuring (requires P1+, ~2hr average discovery time).

## Theme

The hero discovers ancient rune tablets while adventuring. Each tablet contains a hidden sequence of runes that must be decoded through logical deduction. Feedback after each guess reveals which runes are correct, misplaced, or absent.

## Difficulty Levels

| Difficulty | Runes | Slots | Guesses | Dupes | Combinations |
|-----------|-------|-------|---------|-------|-------------|
| Novice | 5 | 3 | 10 | No | 60 |
| Apprentice | 6 | 4 | 10 | No | 360 |
| Journeyman | 6 | 4 | 8 | Yes | 1,296 |
| Master | 8 | 5 | 8 | Yes | 32,768 |

Key difficulty jumps:
- Novice/Apprentice: No duplicates, generous guesses. Hard to lose.
- Journeyman: Duplicates allowed. Qualitative shift in deduction strategy, fewer guesses.
- Master: More runes, more slots, duplicates. Genuine failure risk.

## Rewards

| Difficulty | Reward |
|-----------|--------|
| Novice | +25% XP |
| Apprentice | +50% XP |
| Journeyman | +1 Fish Rank, +75% XP |
| Master | +2 Fish Ranks, +1 Prestige |

Fishing-focused identity at higher tiers. No other challenge rewards fishing ranks. Prestige only at Master where difficulty justifies it.

## Rune Symbols

Eight terminal-friendly rune characters: `᛭ ᚦ ᛟ ᚱ ᛊ ᚹ ᛏ ᚲ`

Fallback if unicode rendering is problematic: single-letter labels (A-H).

## Feedback System

After each guess, feedback markers appear next to the submitted sequence:
- `●` (bright/white) — Correct rune in correct position
- `○` (yellow/dim) — Correct rune, wrong position
- `·` (dark) — Rune not in the code

Feedback markers are shown in sorted order (exact matches first, then misplaced, then wrong) to avoid leaking positional information.

## Controls

- **Left/Right arrows**: Move between slots
- **Up/Down arrows**: Cycle through available rune symbols in current slot
- **Enter**: Submit guess (all slots must be filled)
- **F**: Clear current guess (reset all slots)
- **Esc (x2)**: Forfeit (double-tap with confirmation, same pattern as Minesweeper)

## UI Layout

Same layout pattern as Minesweeper and Chess: grid area on left, info panel on right (24 chars wide).

### Grid Area (Left)

Displays guess history and current input:
- Each past guess shows: guess number, rune sequence, feedback markers
- Current guess shows filled/empty slots with cursor indicator
- Available rune symbols listed below the guess area

### Info Panel (Right)

- Title: "Rune Deciphering"
- Difficulty name
- Rune count, slot count
- Guesses remaining
- Duplicates allowed (yes/no)
- Status text (Deciphering... / Forfeit game?)
- Controls reference

### Game Over Overlay

Centered on the grid area (not full area). Shows:
- Win: "Runes Deciphered!" in green, reward text
- Loss: "Runes Remain Hidden" in red, reveals the hidden code, "No reward"
- "[Any key to continue]" to dismiss

## Game Logic

### Code Generation

- Randomly select `slots` runes from the available `runes` pool
- If duplicates not allowed, sample without replacement
- If duplicates allowed, sample with replacement
- Code generated when challenge is created (before first guess)

### Guess Validation

- All slots must be filled before submitting
- If duplicates not allowed, guess must contain unique runes (reject with message if duplicated)

### Feedback Calculation

Standard Mastermind scoring algorithm:
1. Count exact matches (correct rune, correct position)
2. For remaining runes, count misplaced matches (correct rune, wrong position)
3. Remaining slots are wrong guesses

### Win/Loss Conditions

- **Win**: All slots are exact matches (all `●`)
- **Loss**: Used all guesses without solving

## Data Structures

```rust
pub enum RuneDifficulty {
    Novice,      // 5 runes, 3 slots, 10 guesses, no dupes
    Apprentice,  // 6 runes, 4 slots, 10 guesses, no dupes
    Journeyman,  // 6 runes, 4 slots, 8 guesses, yes dupes
    Master,      // 8 runes, 5 slots, 8 guesses, yes dupes
}

pub enum RuneResult {
    Win,
    Loss,
}

pub enum FeedbackMark {
    Exact,     // ● correct rune, correct position
    Misplaced, // ○ correct rune, wrong position
    Wrong,     // · not in code
}

pub struct RuneGuess {
    pub runes: Vec<usize>,              // indices into RUNE_SYMBOLS
    pub feedback: Vec<FeedbackMark>,    // sorted: Exact first, then Misplaced, then Wrong
}

pub struct RuneGame {
    pub difficulty: RuneDifficulty,
    pub secret_code: Vec<usize>,        // indices into RUNE_SYMBOLS
    pub guesses: Vec<RuneGuess>,        // submitted guesses
    pub current_guess: Vec<Option<usize>>, // current input (None = empty slot)
    pub cursor_slot: usize,             // which slot cursor is on
    pub max_guesses: usize,
    pub num_runes: usize,               // how many rune symbols available
    pub num_slots: usize,
    pub allow_duplicates: bool,
    pub game_result: Option<RuneResult>,
    pub forfeit_pending: bool,
}
```

## Files

| File | Purpose |
|------|---------|
| `src/rune.rs` | Data structures, difficulty config, rune symbols |
| `src/rune_logic.rs` | Code generation, feedback calculation, input handling |
| `src/ui/rune_scene.rs` | Grid rendering, info panel, game over overlay |

## Integration Points

- `challenge_menu.rs`: Add `ChallengeType::Rune` variant, reward impl, weighted discovery
- `game_state.rs`: Add `active_rune: Option<RuneGame>` field
- `main.rs`: Input handling for active rune game, rendering dispatch
- `debug_menu.rs`: Add "Trigger Rune Challenge" option
- `lib.rs`: Export `rune` and `rune_logic` modules
