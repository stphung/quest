---
name: test-audit
description: Multi-agent test health audit — finds flaky tests and performance bottlenecks by area, auto-fixes safe patterns, and verifies with 10x runs. Use when tests are flaky, slow, or before releases.
---

<!-- meta-audit scope-universe: tests/*_tests/ -->

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

Scope: `tests/fishing_tests/`, `tests/deep_tests/`, `tests/haven_tests/`, `tests/enhancement_tests/`, `tests/stormglass_tests/`, `tests/loom_tests/`

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
| Re-flagging the same ~40 brute-force RNG-search loops every run as if new | This category has recurred, unresolved, across every run since 2026-07-02 — fixing it needs seed research or a production-code RNG-injection change, both outside this skill's auto-fix scope. Note it as known, tracked debt rather than re-deriving it from scratch each cycle; only act on it if actually doing that research. |
| Bundling multiple locations under one aggregated finding without verifying each individually | A 2026-07-20 meta-audit re-verification found 7 of 20 findings from that run overstated uniformity across a cited location list: two outright misattributed a loop to the wrong same-file test (and the wrong file path) when two similarly-named tests sat near each other, and five more conflated distinct helpers/tests as one duplicated pattern or claimed uniform N-way duplication when only some cited locations actually matched. When a finding spans more than one location, read the exact test name and line at *each* one — don't extrapolate from the first-verified instance to the rest of the list. **Recurred in the 2026-08-31 meta-audit re-verification** (5 weeks after the guardrail above was added) in a subtler form: the *investigating* agent's own prose was correctly hedged (e.g. "verified 3 of 8 pairs, names match on the rest but not individually body-verified"), but that hedge was lost when compressed into the logged finding's one-line `claim` text ("8 tests duplicated verbatim") — the summary silently dropped the caveat and asserted uniformity the investigation never established. A second instance in the same pass claimed "Constitution=50 divergence in exactly 1 of 7 files" when the actual count was 3 of 7 (two different divergent values). **Stronger guardrail: when writing the `claim` field for a multi-location finding, the verified-count caveat must survive into the text itself** — write "N of M verified identical; remaining M-N name-matched only" rather than a blanket descriptor like "verbatim" or "exactly 1 of N" unless literally every cited location was individually re-read this run. |
| Narrating a plausible-sounding flakiness *mechanism* (race condition, tautological fallback) without tracing the exact code path | A 2026-08-31 meta-audit re-verification found two 2026-08-24 findings that pattern-matched a known anti-pattern shape without verifying the mechanism actually holds. (1) A timing test was flagged as a "wall-clock race that could flip under a loaded/throttled CI runner" — but the test backdates `Instant::now()` by *more* than the threshold and compares with a monotonic clock, so elapsed time can only grow, never shrink below the margin; added delay makes it pass more safely, the opposite of the claimed risk. (2) A brute-force loop's "silent fallback to a trivial/tautological assertion" was claimed for 3 cited tests, but 1 of the 3 had no fallback assertion at all in the miss case (just a discarded value) — a different (and arguably worse) defect than the one described. Before writing a flakiness-mechanism claim, trace the actual failure direction (does elapsed time in this construction ever decrease relative to the threshold?) and confirm an assertion is actually present in the fallback branch you're describing — don't infer either from the anti-pattern's name alone. |

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 3,
     "scope": ["tests/game_tick_tests/", "tests/combat_tests/"],
     "findings": [
       {
         "location": "tests/combat_tests/foo_test.rs:42",
         "claim": "...",
         "correct_value": "...",
         "severity": "HIGH",
         "category": "unseeded-rng",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh test-audit /tmp/test-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh test-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `test-audit` next. If it
   prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.
