> Backported design record. Sources: docs/plans/2026-02-20-storm-leviathan-timing-design.md, docs/plans/2026-02-20-storm-leviathan-timing.md.

## 2026-02-20-storm-leviathan-timing-design.md

# Storm Leviathan Encounter Rebalance

**Issue:** #323
**Date:** 2026-02-20
**Status:** Approved

## Problem

The Storm Leviathan hunt requires ~928 expected legendary catches, with 65% concentrated in encounters 9-10 (0.5% and 0.25% chance). At rank 40's ~2.4% legendary rate, this takes 25-82 hours. The final encounters feel like a brick wall rather than a climax.

## Design

Replace `LEVIATHAN_ENCOUNTER_CHANCES` in `src/fishing/generation.rs` with a narrative-arc curve that rises in the middle ("the beast grows bold") then tightens for the finale.

### New Encounter Chances

```
Current:  [0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.015, 0.01, 0.005, 0.0025]
New:      [0.05, 0.03, 0.04, 0.05, 0.04, 0.03, 0.02,  0.015, 0.01,  0.008]
```

### Narrative Arc

- Encounters 1-2: Elusive opening (5%, 3%) — "Ripples", "The Shadow"
- Encounters 3-5: The beast grows bold (4%, 5%, 4%) — "Emergence", "Known", "First Strike"
- Encounters 6-10: Tightening finale (3% → 0.8%) — "Fury" through "Legend"

### Expected Legendaries Per Encounter

| # | Name | Old Chance | New Chance | Old E[L] | New E[L] |
|---|------|-----------|-----------|---------|---------|
| 1 | Ripples | 8% | 5% | 13 | 20 |
| 2 | The Shadow | 6% | 3% | 17 | 33 |
| 3 | Emergence | 5% | 4% | 20 | 25 |
| 4 | Known | 4% | 5% | 25 | 20 |
| 5 | First Strike | 3% | 4% | 33 | 25 |
| 6 | Fury | 2% | 3% | 50 | 33 |
| 7 | Blood in Water | 1.5% | 2% | 67 | 50 |
| 8 | The Long Night | 1% | 1.5% | 100 | 67 |
| 9 | Exhaustion | 0.5% | 1% | 200 | 100 |
| 10 | Legend | 0.25% | 0.8% | 400 | 125 |
| | **Total** | | | **~928** | **~498** |

### Time Estimates

At rank 40 (~2.4% legendary rate), ~20,750 total fish needed:
- Without Haven bonuses: ~35-40 hours
- With Haven bonuses: ~14-20 hours

Worst single encounter (#10) needs ~125 legendaries vs. current 400 (69% reduction).

## Scope

- Change `LEVIATHAN_ENCOUNTER_CHANCES` array values
- Update `test_leviathan_encounter_chances_decreasing` — curve is no longer strictly decreasing (rises at encounters 3-4), change to valid-range check only
- Update doc comment on the array
- No changes to catch chance (25%), encounter count (10), or any other fishing mechanics

## 2026-02-20-storm-leviathan-timing.md

# Storm Leviathan Timing Rebalance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Flatten the Storm Leviathan encounter curve from ~928 to ~498 expected legendaries with a narrative arc (rises mid-hunt, tightens for finale).

**Architecture:** Single constant array change + test update. No structural changes.

**Tech Stack:** Rust

---

### Task 1: Update encounter chances and doc comment

**Files:**
- Modify: `src/fishing/generation.rs:170-184`

**Step 1: Update the constant array and comment**

Replace the `LEVIATHAN_ENCOUNTER_CHANCES` array and its doc comment with:

```rust
/// Progressive encounter chances for the Storm Leviathan hunt.
/// Narrative arc: elusive opening, the beast grows bold mid-hunt, tightening finale.
/// ~498 expected legendaries total (~14-20 hours with Haven, ~35-40 without).
const LEVIATHAN_ENCOUNTER_CHANCES: [f64; 10] = [
    0.05,  // Encounter 1: 5%   - "Ripples"
    0.03,  // Encounter 2: 3%   - "The Shadow"
    0.04,  // Encounter 3: 4%   - "Emergence"
    0.05,  // Encounter 4: 5%   - "Known"
    0.04,  // Encounter 5: 4%   - "First Strike"
    0.03,  // Encounter 6: 3%   - "Fury"
    0.02,  // Encounter 7: 2%   - "Blood in Water"
    0.015, // Encounter 8: 1.5% - "The Long Night"
    0.01,  // Encounter 9: 1%   - "Exhaustion"
    0.008, // Encounter 10: 0.8% - "Legend"
];
```

**Step 2: Run tests to see what breaks**

Run: `cargo test leviathan -- --nocapture`
Expected: `test_leviathan_encounter_chances_decreasing` FAILS (curve is no longer strictly decreasing)

---

### Task 2: Fix the broken test

**Files:**
- Modify: `src/fishing/generation.rs:530-542`

**Step 1: Replace the strictly-decreasing test with a valid-range + narrative-arc test**

Replace `test_leviathan_encounter_chances_decreasing` with:

```rust
    #[test]
    fn test_leviathan_encounter_chances_narrative_arc() {
        // All chances must be valid probabilities
        for (i, chance) in LEVIATHAN_ENCOUNTER_CHANCES.iter().enumerate() {
            assert!(
                *chance > 0.0 && *chance <= 1.0,
                "Encounter {} chance {} should be between 0 and 1",
                i + 1,
                chance
            );
        }

        // Narrative arc: encounters 3-4 should rise (beast grows bold)
        assert!(
            LEVIATHAN_ENCOUNTER_CHANCES[2] > LEVIATHAN_ENCOUNTER_CHANCES[1],
            "Encounter 3 should be higher than encounter 2 (rising arc)"
        );
        assert!(
            LEVIATHAN_ENCOUNTER_CHANCES[3] > LEVIATHAN_ENCOUNTER_CHANCES[2],
            "Encounter 4 should be higher than encounter 3 (rising arc)"
        );

        // Finale: encounters 5-10 should be strictly decreasing
        for i in 5..LEVIATHAN_ENCOUNTER_CHANCES.len() {
            assert!(
                LEVIATHAN_ENCOUNTER_CHANCES[i] < LEVIATHAN_ENCOUNTER_CHANCES[i - 1],
                "Encounter {} chance ({}) should be < encounter {} chance ({}) in finale",
                i + 1,
                LEVIATHAN_ENCOUNTER_CHANCES[i],
                i,
                LEVIATHAN_ENCOUNTER_CHANCES[i - 1]
            );
        }

        // Last encounter should still be a meaningful chance (not vanishingly small)
        assert!(
            *LEVIATHAN_ENCOUNTER_CHANCES.last().unwrap() >= 0.005,
            "Final encounter chance should be >= 0.5%"
        );
    }
```

**Step 2: Run all leviathan tests**

Run: `cargo test leviathan -- --nocapture`
Expected: All PASS

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All PASS

**Step 4: Commit**

```bash
git add src/fishing/generation.rs
git commit -m "feat: rebalance Storm Leviathan encounter curve (#323)

Narrative arc: elusive opening, beast grows bold mid-hunt, tightening
finale. Reduces expected legendaries from ~928 to ~498 (~14-40 hours
depending on Haven bonuses)."
```
