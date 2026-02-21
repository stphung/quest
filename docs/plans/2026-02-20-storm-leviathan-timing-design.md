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
