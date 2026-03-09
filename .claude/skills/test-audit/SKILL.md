---
name: test-audit
description: Multi-agent test health audit — finds flaky tests and performance bottlenecks by area, auto-fixes safe patterns, and verifies with 10x runs. Use when tests are flaky, slow, or before releases.
---

# Test Health Audit

Multi-agent audit of test suite health. Finds flakiness and performance issues, auto-fixes safe patterns, and verifies with 10x consecutive runs.

## When to Use

- Tests pass/fail inconsistently (flaky)
- Test suite is slow or getting slower
- Before a release or after a large feature lands
- CI is unreliable due to test issues
- After adding many new tests

## Phase 1: Parallel Audit (3 Agents, Read-Only)

Spawn 3 Explore agents simultaneously. Each agent checks for BOTH flakiness and performance anti-patterns in its scoped test files.

**Agent 1 — Core & Combat**

Scope: `tests/game_tick_tests/`, `tests/combat_tests/`, `tests/character_tests/`, `tests/zone_tests/`

**Agent 2 — Systems**

Scope: `tests/fishing_tests/`, `tests/deep_tests/`, `tests/haven_tests/`, `tests/enhancement_tests/`, `tests/stormglass_tests/`

**Agent 3 — Items, Achievements & Misc**

Scope: `tests/item_tests/`, `tests/achievement_tests/`, `tests/history_tests/`, `tests/misc_tests/`

### Flakiness Patterns

Each agent searches for:

| Pattern | Risk | Example | Fix |
|---------|------|---------|-----|
| Non-seeded RNG | HIGH | `rand::rng()`, `thread_rng()` in tests | Seeded `ChaCha8Rng` |
| Timing assertions | HIGH | `Instant::now()` + elapsed checks, timestamp comparisons | Add buffer (1-2s tolerance) |
| Shared filesystem paths | HIGH | Hardcoded paths without `tempfile` | `tempdir()` per test |
| Floating-point equality | MEDIUM | `assert_eq!` on f64 | `assert!((a - b).abs() < epsilon)` |
| Probabilistic assertions with tight margins | MEDIUM | Monte Carlo expecting exact distribution | Widen margins to 2-5x range |

### Performance Patterns

| Pattern | Fix |
|---------|-----|
| Excessive loop counts (1000+) | Reduce to minimum proving the point |
| Brute-force rare event triggering | Seeded RNG with known-good seed or boost parameters |
| Structural impossibility tests (P=0 by code) | 100 iterations max |
| Redundant coverage across files | Flag for consolidation |
| Heavy state setup repeated per test | Extract shared helper |

Each agent produces a ranked report: test name, file:line, pattern, severity (HIGH/MEDIUM/LOW), suggested fix, whether auto-fixable.

## Phase 2: Fix (Sequential)

Spawn fix agents based on audit findings.

### Auto-fix (no user approval needed)

- Add time buffer to timestamp assertions
- Reduce excessive loop counts
- Replace `thread_rng()` with seeded RNG in tests
- Widen probabilistic margins

### Flag for user review

- Removing or consolidating tests
- Changing what a test asserts (behavior change)
- Adding new dev-dependencies

## Phase 3: Verify

1. `make check` must pass
2. Run `cargo test` 10 times consecutively — 0 failures required:
   ```bash
   for i in $(seq 1 10); do echo "=== Run $i ==="; cargo test 2>&1 | grep "test result:"; done
   ```
3. Report summary of: findings, auto-fixes applied, items flagged for review

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Reducing iteration count too aggressively | Keep enough for statistical confidence (3x expected hits minimum) |
| Fixing flakiness by adding sleep/retry | Fix root cause (isolation, determinism), not symptoms |
| Modifying production code to fix tests | Only change test code unless the production code has a genuine bug |
| Ignoring Monte Carlo tests as "probably fine" | Check margins are generous (2-5x expected range) |
| Skipping the 10x verification | Flakiness is probabilistic; 1 run proves nothing |

## Output

Report the PR URL and final status when done (use `/ship` skill).
