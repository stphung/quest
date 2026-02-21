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
