# Test Coverage Analysis

## Current State

**Total test count: ~1,067 tests** (932 unit + 135 integration)

### Unit tests by module

| Module | Files w/ tests | Test count | Assessment |
|--------|---------------|------------|------------|
| `core/` | 2 | 58 | Excellent |
| `character/` | 5 | 120 | Excellent |
| `combat/` | 2 | 73 | Good |
| `zones/` | 2 | 28 | Good |
| `dungeon/` | 3 | 73 | Good |
| `fishing/` | 3 | 48 | Good |
| `items/` | 6 | 62 | Good |
| `challenges/` | 13 | 307 | Excellent |
| `haven/` | 2 | 64 | Excellent |
| `achievements/` | 3 | 41 | Moderate |
| `utils/` | 3 | 18 | Light |
| `ui/` | 4 | 12 | Minimal |
| `input.rs` | 0 | 0 | None |

### Integration tests (8 files, 135 tests)

| Test file | Count | Covers |
|-----------|-------|--------|
| `item_pipeline_test.rs` | 30 | Drop rates, generation, scoring, auto-equip |
| `game_loop_test.rs` | 28 | Main tick loop, combat flow, level-ups, offline XP |
| `fishing_integration_test.rs` | 27 | Sessions, ranks, fish gen, haven bonuses, Storm Leviathan |
| `zone_progression_test.rs` | 17 | Boss spawning, weapon gates, zone unlocking |
| `dungeon_completion_test.rs` | 11 | Full runs, key system, safe death, room distribution |
| `chess_integration_test.rs` | 8 | Win/loss/draw, difficulty rewards, ELO |
| `prestige_cycle_test.rs` | 7 | Prestige flow, fishing preservation, XP multipliers |
| `storm_forge_test.rs` | 7 | Stormbreaker achievement, prestige costs |

---

## Gaps and Recommendations

### 1. Achievement unlock conditions have no integration test (HIGH)

`achievements/types.rs` has 35 unit tests for the data structure, but there is **no integration test verifying that achievements actually unlock during gameplay**. The `check_achievements()` function (or equivalent) that evaluates game state against unlock conditions is not tested end-to-end.

**Recommendation:** Add `tests/achievement_integration_test.rs` that:
- Creates a game state, simulates kills, and verifies combat achievements unlock at the correct thresholds (100, 500, 1000, 5000, 10000 kills)
- Simulates zone progression and checks zone-related achievements
- Simulates prestige and verifies prestige achievements
- Verifies achievements persist across prestiges (account-level)
- Tests that duplicate unlocks don't occur

### 2. Haven bonus application is not integration-tested (HIGH)

Haven has 64 solid unit tests for room construction and bonus calculation, but there is **no integration test verifying bonuses actually affect gameplay**. Haven bonuses are injected as parameters, so the wiring between Haven state and combat/fishing/XP systems is untested.

**Recommendation:** Add `tests/haven_integration_test.rs` that:
- Builds haven rooms and verifies XP multiplier bonus applies to combat XP gains
- Verifies drop rate bonus increases item drops
- Verifies rarity bonus shifts rarity distribution
- Verifies fishing gain bonus affects fishing rank XP
- Tests haven state survives prestige (account-level)

### 3. `input.rs` has zero tests (MEDIUM)

The top-level input router (`src/input.rs`) dispatches keyboard events to the correct handler based on game state. This is a critical coordination point with no test coverage. A wrong dispatch could silently break any game feature.

**Recommendation:** Add unit tests to `input.rs` that:
- Verify correct dispatch for each `GameMode` / overlay combination
- Test that minigame keys route to the correct handler
- Test that the debug menu toggle (`backtick`) only works with `--debug`
- Test edge cases like key events during state transitions

### 4. No integration tests for Go, Morris, Gomoku, Minesweeper, or Rune challenges (MEDIUM)

Chess has a dedicated `chess_integration_test.rs`, but the other 5 minigames have no integration tests. Their unit tests cover logic well (36-63 tests each), but the full flow from challenge discovery → play → reward is untested.

**Recommendation:** Add `tests/challenge_integration_test.rs` that for each minigame:
- Verifies a complete game from start to win/loss
- Confirms correct reward distribution (XP, prestige XP based on difficulty)
- Tests the forfeit pattern (first Esc → pending, second Esc → confirm, other key → cancel)
- Verifies prestige rank requirements are enforced (P1+ required)

### 5. Offline progression has thin testing (MEDIUM)

`game_loop_test.rs` has a basic offline XP test, but the offline progression system (50% rate, 7-day cap, simulated kills) involves several interacting systems. Edge cases around the cap and interaction with prestige multipliers are not covered.

**Recommendation:** Expand offline progression tests:
- Verify 7-day cap is enforced exactly
- Verify 50% rate vs online rate
- Test offline progression with various prestige multipliers
- Test offline progression with haven XP bonus
- Test that being offline for 0 seconds produces no XP
- Test serialization round-trip of the "last seen" timestamp

### 6. MCTS AI has only 4 tests (MEDIUM)

The Go module's Monte Carlo Tree Search (`src/challenges/go/mcts.rs`) is a complex algorithm with only 4 tests (basic move generation, suicide avoidance, and 2 UCT math tests). For an algorithm that drives AI behavior, this is thin.

**Recommendation:** Add tests for:
- MCTS with different simulation counts (verify higher counts → better moves)
- Known Go positions where the best move is deterministic
- Performance regression (ensure MCTS completes within time budget)
- Edge cases: full board, single legal move, ko situations

### 7. `achievements/persistence.rs` only has 3 tests (MEDIUM)

The save/load system has minimal tests and relies on the real filesystem (`~/.quest/`). There are no tests for corruption recovery, concurrent access, or migration from older formats.

**Recommendation:**
- Test loading malformed JSON (should return default, not panic)
- Test loading JSON with unknown fields (forward compatibility)
- Test save/load round-trip preserves all achievement state
- Use `tempdir` instead of real home directory to avoid polluting user state

### 8. No property-based or fuzz testing (LOW)

The codebase relies entirely on example-based tests. For procedural generation systems (items, dungeons, fish, enemy stats), property-based testing would catch edge cases that hand-written examples miss.

**Recommendation:** Add `proptest` dependency and property tests for:
- Item generation: all generated items have valid slots, positive attributes, correct affix counts for their rarity
- Dungeon generation: all rooms reachable from entrance, boss room always exists, room count matches size
- XP curve: always monotonically increasing, never overflows for valid levels
- Prestige multiplier: always >= 1.0, monotonically increasing with rank
- Enemy generation: HP and damage always positive, scale with level

### 9. RNG non-determinism makes test failures hard to reproduce (LOW)

Tests use `rand::thread_rng()` directly, making failures non-reproducible. Statistical tests use wide tolerance ranges to compensate.

**Recommendation:**
- Create a test helper that provides a seeded `StdRng` for deterministic tests
- Use seeded RNG for all generation tests (items, dungeons, enemies, fish)
- Keep a few statistical tests with real RNG for distribution validation, but gate them behind a feature flag or `#[ignore]` so they don't cause flaky CI

### 10. No shared test utilities (LOW)

Each module creates its own test setup (`GameState::new("Test Hero", 0)` repeated dozens of times). There is no `tests/common/mod.rs` or test helper crate.

**Recommendation:** Create `tests/common/mod.rs` with:
- `fn test_game_state() -> GameState` — pre-configured state for common test scenarios
- `fn test_game_state_at_level(level: u32) -> GameState`
- `fn test_game_state_with_prestige(rank: u32) -> GameState`
- `fn test_game_state_in_dungeon() -> GameState`
- Seeded RNG helper

---

## Priority Summary

| Priority | Area | Effort | Impact |
|----------|------|--------|--------|
| HIGH | Achievement unlock integration test | Medium | Catches unlock logic bugs |
| HIGH | Haven bonus integration test | Medium | Catches bonus wiring bugs |
| MEDIUM | `input.rs` dispatch tests | Low | Prevents silent routing breaks |
| MEDIUM | Non-chess minigame integration tests | Medium | Covers 5 untested game flows |
| MEDIUM | Offline progression edge cases | Low | Prevents XP exploits/bugs |
| MEDIUM | MCTS AI expanded tests | Low | Catches AI regressions |
| MEDIUM | Persistence corruption handling | Low | Prevents save data loss |
| LOW | Property-based testing | High | Catches procedural gen edge cases |
| LOW | Deterministic RNG in tests | Medium | Enables failure reproduction |
| LOW | Shared test utilities | Low | Reduces duplication |
