# Chess Puzzle Challenge Design

## Overview

Chess puzzles are a new challenge type where players solve tactical positions. Unlike the existing chess challenge (full game vs AI), puzzles present a specific position and ask the player to find the best move(s).

## Technical Constraint

The `chess_engine::Board` only starts from the standard position. Puzzles are defined as move sequences from the start position using `Move::Piece(Position::new(rank, file), Position::new(rank, file))` where rank=0-7 (rows 1-8) and file=0-7 (columns a-h).

### Coordinate Reference

```
Position::new(rank, file)
  rank: 0=row1, 1=row2, ..., 7=row8
  file: 0=a, 1=b, 2=c, 3=d, 4=e, 5=f, 6=g, 7=h

Standard starting position:
  Rank 0: Ra1 Nb1 Bc1 Qd1 Ke1 Bf1 Ng1 Rh1 (White)
  Rank 1: White pawns a2-h2
  Rank 6: Black pawns a7-h7
  Rank 7: Ra8 Nb8 Bc8 Qd8 Ke8 Bf8 Ng8 Rh8 (Black)
  White moves first
```

## Rewards

| Tier | Prestige | XP Bonus | Scoring Target |
|------|----------|----------|----------------|
| Novice | 1 | 50% | 4/6 puzzles |
| Apprentice | 2 | 100% | 4/6 puzzles |
| Journeyman | 3 | 75% | 4/6 puzzles |
| Master | 5 | 150% | 3/5 puzzles |

## Puzzle Types

- **Novice**: Mate-in-1 -- find the single checkmate move
- **Apprentice**: Simple Tactics -- find the best move (fork, pin, sacrifice)
- **Journeyman**: Mate-in-2 -- play move 1, opponent responds, deliver checkmate
- **Master**: Complex Tactics -- sacrifices, deflections, multi-move combinations

## Scoring System

- Each tier presents its puzzles in sequence
- Player must solve the target number to pass (e.g., 4/6 for Novice)
- Time limit per puzzle: 60s (Novice), 90s (Apprentice), 120s (Journeyman), 180s (Master)
- Wrong moves: 2 attempts allowed per puzzle before it is marked failed
- Hint: Available once per puzzle, no penalty for using

---

## Novice Puzzles (Mate-in-1)

### N1: Scholar's Mate

**Setup**: 1.e4 e5 2.Bc4 Nc6 3.Qh5 Nf6??

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 5), Position::new(3, 2)),  // Bf1-c4
    Move::Piece(Position::new(7, 1), Position::new(5, 2)),  // Nb8-c6
    Move::Piece(Position::new(0, 3), Position::new(4, 7)),  // Qd1-h5
    Move::Piece(Position::new(7, 6), Position::new(5, 5)),  // Ng8-f6
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(4, 7), Position::new(6, 5))` -- Qxf7# |
| Hint | "The f7 square is only defended by the king..." |
| Why | Queen takes f7, protected by Bc4. King has no escape. |

---

### N2: Fool's Mate

**Setup**: 1.f3 e5 2.g4??

```rust
vec![
    Move::Piece(Position::new(1, 5), Position::new(2, 5)),  // f2-f3
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(1, 6), Position::new(3, 6)),  // g2-g4
]
```

| Field | Value |
|-------|-------|
| Player | Black |
| Solution | `Move::Piece(Position::new(7, 3), Position::new(3, 7))` -- Qh4# |
| Hint | "White's king is exposed on a long diagonal." |
| Why | Qh4 checks via h4-e1 diagonal. King blocked by own pieces; f2 attacked by queen. |

---

### N3: Smothered Knight Mate

**Setup**: 1.e4 e5 2.Nf3 Nc6 3.Bc4 Nd4 4.Nxe5?? Qg5 5.Nxf7 Qxg2 6.Rf1 Qxe4+ 7.Be2

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(7, 1), Position::new(5, 2)),  // Nb8-c6
    Move::Piece(Position::new(0, 5), Position::new(3, 2)),  // Bf1-c4
    Move::Piece(Position::new(5, 2), Position::new(3, 3)),  // Nc6-d4
    Move::Piece(Position::new(2, 5), Position::new(4, 4)),  // Nf3xe5
    Move::Piece(Position::new(7, 3), Position::new(4, 6)),  // Qd8-g5
    Move::Piece(Position::new(4, 4), Position::new(6, 5)),  // Ne5xf7
    Move::Piece(Position::new(4, 6), Position::new(1, 6)),  // Qg5xg2
    Move::Piece(Position::new(0, 7), Position::new(0, 5)),  // Rh1-f1
    Move::Piece(Position::new(1, 6), Position::new(3, 4)),  // Qg2xe4+
    Move::Piece(Position::new(3, 2), Position::new(1, 4)),  // Bc4-e2
]
```

| Field | Value |
|-------|-------|
| Player | Black |
| Solution | `Move::Piece(Position::new(3, 3), Position::new(2, 5))` -- Nf3# |
| Hint | "The king is surrounded by its own army. A knight delivers the final blow." |
| Why | Nd4-f3 smothered mate. King on e1 boxed in by Qd1, d2-pawn, Be2, Rf1, f2-pawn. |

---

### N4: Caro-Kann Smothered Mate

**Setup**: 1.e4 c6 2.d4 d5 3.Nc3 dxe4 4.Nxe4 Nd7 5.Qe2 Ngf6??

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 2), Position::new(5, 2)),  // c7-c6
    Move::Piece(Position::new(1, 3), Position::new(3, 3)),  // d2-d4
    Move::Piece(Position::new(6, 3), Position::new(4, 3)),  // d7-d5
    Move::Piece(Position::new(0, 1), Position::new(2, 2)),  // Nb1-c3
    Move::Piece(Position::new(4, 3), Position::new(3, 4)),  // d5xe4
    Move::Piece(Position::new(2, 2), Position::new(3, 4)),  // Nc3xe4
    Move::Piece(Position::new(7, 1), Position::new(6, 3)),  // Nb8-d7
    Move::Piece(Position::new(0, 3), Position::new(1, 4)),  // Qd1-e2
    Move::Piece(Position::new(7, 6), Position::new(5, 5)),  // Ng8-f6
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(3, 4), Position::new(5, 3))` -- Nd6# |
| Hint | "Your knight can jump deep into enemy territory." |
| Why | Ne4-d6 checks king on e8. King entombed by Qd8, Nd7, Bf8, f7-pawn. |

---

### N5: Englund Gambit Mate

**Setup**: 1.d4 e5 2.dxe5 Bc5 3.Nf3 Qh4 4.Nc3??

```rust
vec![
    Move::Piece(Position::new(1, 3), Position::new(3, 3)),  // d2-d4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(3, 3), Position::new(4, 4)),  // d4xe5
    Move::Piece(Position::new(7, 5), Position::new(4, 2)),  // Bf8-c5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(7, 3), Position::new(3, 7)),  // Qd8-h4
    Move::Piece(Position::new(0, 1), Position::new(2, 2)),  // Nb1-c3
]
```

| Field | Value |
|-------|-------|
| Player | Black |
| Solution | `Move::Piece(Position::new(3, 7), Position::new(1, 5))` -- Qxf2# |
| Hint | "The king is boxed in by its own army. Strike at f2." |
| Why | Queen captures f2, supported by Bc5 diagonal. King on e1 surrounded by Qd1, d2-pawn, e2-pawn, Bf1. |

---

### N6: Dutch Defense Bishop Mate

**Setup**: 1.d4 f5 2.Bg5 h6 3.Bf4 g5 4.Bg3 f4 5.e3 h5 6.Bd3 Rh6 7.Qxh5 Rxh5

```rust
vec![
    Move::Piece(Position::new(1, 3), Position::new(3, 3)),  // d2-d4
    Move::Piece(Position::new(6, 5), Position::new(4, 5)),  // f7-f5
    Move::Piece(Position::new(0, 2), Position::new(4, 6)),  // Bc1-g5
    Move::Piece(Position::new(6, 7), Position::new(5, 7)),  // h7-h6
    Move::Piece(Position::new(4, 6), Position::new(3, 5)),  // Bg5-f4
    Move::Piece(Position::new(6, 6), Position::new(4, 6)),  // g7-g5
    Move::Piece(Position::new(3, 5), Position::new(2, 6)),  // Bf4-g3
    Move::Piece(Position::new(4, 5), Position::new(3, 5)),  // f5-f4
    Move::Piece(Position::new(1, 4), Position::new(2, 4)),  // e2-e3
    Move::Piece(Position::new(5, 7), Position::new(4, 7)),  // h6-h5
    Move::Piece(Position::new(0, 5), Position::new(2, 3)),  // Bf1-d3
    Move::Piece(Position::new(7, 7), Position::new(5, 7)),  // Rh8-h6
    Move::Piece(Position::new(0, 3), Position::new(4, 7)),  // Qd1xh5
    Move::Piece(Position::new(5, 7), Position::new(4, 7)),  // Rh6xh5
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(2, 3), Position::new(5, 6))` -- Bg6# |
| Hint | "Your bishop has a clear diagonal to a devastating square." |
| Why | Bd3-g6 (path e4/f5 empty). Attacks e8 through empty f7. King entombed: Qd8, d7/e7 pawns, Bf8. |

---

## Apprentice Puzzles (Simple Tactics)

Find ONE winning tactical move that wins material or creates a decisive advantage.

### A1: Central Knight Fork

**Setup**: 1.e4 e5 2.Nf3 Qf6 3.Nc3 Bc5

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(7, 3), Position::new(5, 5)),  // Qd8-f6
    Move::Piece(Position::new(0, 1), Position::new(2, 2)),  // Nb1-c3
    Move::Piece(Position::new(7, 5), Position::new(4, 2)),  // Bf8-c5
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(2, 2), Position::new(4, 3))` -- Nd5 |
| Hint | "Your knight can attack two pieces at once from the center." |
| Why | Nd5 forks Qf6 and threatens Nxc7+ forking king and Ra8. Wins the rook. |

---

### A2: Tempo Development

**Setup**: 1.e4 d5 2.exd5 Qxd5

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 3), Position::new(4, 3)),  // d7-d5
    Move::Piece(Position::new(3, 4), Position::new(4, 3)),  // e4xd5
    Move::Piece(Position::new(7, 3), Position::new(4, 3)),  // Qd8xd5
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(0, 1), Position::new(2, 2))` -- Nc3 |
| Hint | "Develop a piece with an attack on the enemy queen." |
| Why | Nc3 develops with tempo. Black must waste a move retreating the queen. |

---

### A3: Continuing Attack with Check

**Setup**: 1.e4 e5 2.Nf3 f6?? 3.Nxe5 fxe5 4.Qh5+ Ke7 5.Qxe5+ Kf7

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(6, 5), Position::new(5, 5)),  // f7-f6
    Move::Piece(Position::new(2, 5), Position::new(4, 4)),  // Nf3xe5
    Move::Piece(Position::new(5, 5), Position::new(4, 4)),  // f6xe5
    Move::Piece(Position::new(0, 3), Position::new(4, 7)),  // Qd1-h5+
    Move::Piece(Position::new(7, 4), Position::new(6, 4)),  // Ke8-e7
    Move::Piece(Position::new(4, 7), Position::new(4, 4)),  // Qh5xe5+
    Move::Piece(Position::new(6, 4), Position::new(5, 5)),  // Ke7-f7
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(0, 5), Position::new(3, 2))` -- Bc4+ |
| Hint | "Develop with check to keep the pressure on the exposed king." |
| Why | Bc4+ checks king on f7. Continues the attack with massive development lead. |

---

### A4: Ruy Lopez Pin

**Setup**: 1.e4 e5 2.Nf3 Nc6

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(7, 1), Position::new(5, 2)),  // Nb8-c6
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(0, 5), Position::new(4, 1))` -- Bb5 |
| Hint | "Pin the defender of the e5 pawn to the king." |
| Why | Bb5 pins Nc6 to king. Creates lasting strategic pressure. |

---

### A5: Fried Liver Pawn Grab

**Setup**: 1.e4 e5 2.Nf3 Nc6 3.Bc4 Nf6 4.Ng5 d5

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(7, 1), Position::new(5, 2)),  // Nb8-c6
    Move::Piece(Position::new(0, 5), Position::new(3, 2)),  // Bf1-c4
    Move::Piece(Position::new(7, 6), Position::new(5, 5)),  // Ng8-f6
    Move::Piece(Position::new(2, 5), Position::new(4, 6)),  // Nf3-g5
    Move::Piece(Position::new(6, 3), Position::new(4, 3)),  // d7-d5
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(3, 4), Position::new(4, 3))` -- exd5 |
| Hint | "Capture the pawn. If the knight retakes, a devastating fork awaits." |
| Why | exd5 wins a pawn. If Nxd5, Nxf7! forks king and queen (Fried Liver). |

---

### A6: Sacrifice on f7

**Setup**: 1.e4 e5 2.Nf3 d6 3.Bc4 Bg4 4.Nc3 Nd7

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(6, 3), Position::new(5, 3)),  // d7-d6
    Move::Piece(Position::new(0, 5), Position::new(3, 2)),  // Bf1-c4
    Move::Piece(Position::new(7, 2), Position::new(3, 6)),  // Bc8-g4
    Move::Piece(Position::new(0, 1), Position::new(2, 2)),  // Nb1-c3
    Move::Piece(Position::new(7, 1), Position::new(6, 3)),  // Nb8-d7
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Solution | `Move::Piece(Position::new(3, 2), Position::new(6, 5))` -- Bxf7+ |
| Hint | "A sacrifice on the weakest square exposes the king." |
| Why | Bxf7+! After Kxf7, Ng5+ forks king and Bg4. Wins piece + pawn. |

---

## Journeyman Puzzles (Mate-in-2)

Player plays move 1, opponent responds (forced), player delivers checkmate.

### J1: Legal's Mate

**Setup**: 1.e4 e5 2.Nf3 d6 3.Bc4 Bg4 4.Nc3 g6 5.Nxe5 Bxd1??

```rust
vec![
    Move::Piece(Position::new(1, 4), Position::new(3, 4)),  // e2-e4
    Move::Piece(Position::new(6, 4), Position::new(4, 4)),  // e7-e5
    Move::Piece(Position::new(0, 6), Position::new(2, 5)),  // Ng1-f3
    Move::Piece(Position::new(6, 3), Position::new(5, 3)),  // d7-d6
    Move::Piece(Position::new(0, 5), Position::new(3, 2)),  // Bf1-c4
    Move::Piece(Position::new(7, 2), Position::new(3, 6)),  // Bc8-g4
    Move::Piece(Position::new(0, 1), Position::new(2, 2)),  // Nb1-c3
    Move::Piece(Position::new(6, 6), Position::new(5, 6)),  // g7-g6
    Move::Piece(Position::new(2, 5), Position::new(4, 4)),  // Nf3xe5
    Move::Piece(Position::new(3, 6), Position::new(0, 3)),  // Bg4xd1
]
```

| Field | Value |
|-------|-------|
| Player | White |
| Move 1 | `Move::Piece(Position::new(3, 2), Position::new(6, 5))` -- Bxf7+ |
| Response | `Move::Piece(Position::new(7, 4), Position::new(6, 4))` -- Ke7 |
| Move 2 | `Move::Piece(Position::new(2, 2), Position::new(4, 3))` -- Nd5# |
| Hint | "Sacrifice the bishop to lure the king out, then the knight delivers." |
| Why | Bxf7+ forces Ke7 (Kd7 attacked by Ne5). Nd5# -- all escapes blocked: d6=pawn, d7=Ne5, d8=queen, e6/e8=Bf7, f6=Nd5, f7=bishop, f8=bishop. |

**Note**: Kf8 is a legal alternative to Ke7, but avoids the immediate mate.

---

### J2-J6: Pending Engine Verification

The following puzzles need to be constructed with engine-assisted verification due to the difficulty of hand-verifying long sequences. Themes are defined here; implementations should use the puzzle test harness to validate.

**J2: Arabian Mate** -- Rook + knight trap king in corner. Requires castling setup (20+ moves).

**J3: Anastasia's Mate** -- Knight + rook create mating net on h-file.

**J4: Boden's Mate** -- Two bishops on criss-crossing diagonals after queenside castling.

**J5: Epaulette Mate** -- Queen mates while king's own pieces block escape on both sides.

**J6: Back Rank Mate** -- Rook delivers checkmate on the 8th rank while pawns block the king.

---

## Master Puzzles (Complex Tactics)

Multi-move combinations involving sacrifices, deflections, and deep calculation.

### M1-M5: Pending Engine Verification

**M1: Queen Sacrifice + Back Rank Mate** -- Sacrifice queen to deflect defender, then rook mates.

**M2: Clearance Sacrifice** -- Move a piece with tempo to clear a square for a mating piece.

**M3: Discovered Attack** -- Moving one piece reveals a devastating attack from another.

**M4: Deflection** -- Force a defender away from its duty, then exploit the weakness.

**M5: Windmill** -- Alternating discovered checks winning material on each move.

---

## Implementation Notes

### Puzzle Data Structure

Each puzzle consists of:
1. **Setup moves**: `Vec<Move>` played from the standard position
2. **Player color**: White or Black
3. **Solution**: For mate-in-1, one `Move`. For mate-in-2, `(Move, Move, Move)` = (player1, response, player2). For tactics, one `Move`.
4. **Hint**: `&str` displayed on request
5. **Title**: Short name for the puzzle
6. **Difficulty tier**: Determines rewards and time limits

### Validation Requirements

Before adding any puzzle:
1. Play all setup moves through `Board::play_move()` -- each must return `GameResult::Continuing`
2. For mate-in-1: solution move must return `GameResult::Victory`
3. For mate-in-2: move 1 returns `Continuing`, response is a legal move, move 2 returns `Victory`
4. For tactics: solution move must be legal
5. Write a `#[test]` for each puzzle verifying the above

### Implementation Priority

1. Build puzzle engine with N1-N6 and A1-A6 (13 verified puzzles)
2. Add J1 (Legal's Mate)
3. Use test harness to construct and verify J2-J6 and M1-M5
4. Total target: 23 puzzles across 4 tiers
