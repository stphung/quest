# Design — Act 2 Era Pacing: the 3-Month Balanced Campaign

## Context

The release-hardening change froze the era's measured behavior into asserted gates (balanced 29 crossings / 4.0 mo / 87.5%). Direction is to hit the original ~3-month design target. The gates now make that a visible, verifiable retune instead of a hopeful constant nudge. Verification per root CLAUDE.md's vessel row: `cargo test --test ferryman_tests --release` + `cargo run --bin voyage_simulator` + `cargo test overlay_snapshot` (colony-derived numbers render in the Reckoning) + full `make check`.

## Goals / Non-Goals

**Goals:** balanced era ≈ 3 real months (~19–24 crossings, ≥85% saved); traps stay traps with a wide skill margin; maiden voyage untouched (~15 real days); gates tightened so the target is enforced, not aspirational.

**Non-Goals:** voyage clock, road/provision pricing, Riftglass/Dock constants, the Ward branch's era length, `INITIAL_SOULS`.

## Decisions

### D1 — Tune via `CAP_GROWTH`, not the clock, drive, or economy richness

Measured single-knob sensitivity (full deterministic era sweeps, 2026-07-12):

| Knob tried | Balanced result | Verdict |
|---|---|---|
| `SOULS_PER_SALVAGE` 30→24 (richer economy) | 32 crossings / **4.2 mo** — worse | rejected: enriches every line equally, balanced spend just buys more small steps |
| `DRIVE_DECAY` 0.70→0.67 (+`CAP` 1.42) | 30 crossings / 3.9 mo — no better than CAP alone | rejected: drive shortens crossings the balanced line has already made short |
| `CAP_GROWTH` 1.36→1.44 | 24 / 3.4 mo / 89.5% | right lever, not far enough |
| **`CAP_GROWTH` 1.36→1.46** | **22 / 3.1 mo / 90.1%** | chosen |

Era length is dominated by crossing *count* once Drive has compounded a few levels; the hold is what cuts the count. The clock (`GAME_MINUTES_PER_REAL_MINUTE`) was never a candidate — the two-real-week maiden voyage is stated design identity ("the ramp is earned, not compressed").

### D2 — Compensate the trap contrast with `DARK_TAKES_PER_DAY` 0.0006 → 0.0007

Wider holds alone lifted every line's % saved (cap-only reached 81.2%, blurring "pure Shipwright is a trap"). A slightly hungrier dark restores the spread at negligible cost to the tuned line: balanced 90.1% → 88.6% (matching the design's long-stated "~88%"), drive-only 70.5% → 67.1%, cap-only 78.5%. The margin between skilled and reckless play widens (23.0 points balanced-vs-drive-only, was 17.0).

### D3 — Tighten the gates to the new target

The envelope bands from the hardening change (20–40 crossings / 3–6 mo / ≥82%) would still pass a regression back to 4+ months. New bands center the target with ~±40% headroom: **15–30 crossings, 2.5–4.5 months, ≥84% saved** (balanced); `dock_time_across_charge_policies`' era-window assertion follows to 2.5–4.5. Ward-lean (≥90%, longer-than-balanced) and drive-only (≤74%) bands unchanged — both still hold with margin. The spec's envelope requirement is MODIFIED to match; the pre-existing `an_era_ferries...` 15–30 crossings / ≥78k assertions already agree.

### D4 — Fixture and snapshot fallout is accepted, reviewed, re-blessed

`fixtures::colony_midera()` (drive 10 / cap 10) now computes a larger `expedition_size` (180×1.46¹⁰ ≈ 7,940 vs ≈ 3,900), so Reckoning/Dock overlay snapshots showing colony-derived numbers change. These diffs are the intended, visible consequence of the retune — reviewed then re-blessed, never re-blessed blind. The frozen corpus `colony.json` stores levels, not derived values, so save-compat is unaffected.

## Risks / Trade-offs

- **[Cap-only trap is softer on % (78.5 vs 74.2)]** → it is now a trap on *two* axes: 10 points fewer souls AND twice the era length (6.3 vs 3.1 months); the gate keeps drive-only ≤74% as the hard floor and `skilled_play...` asserts the wide margin.
- **[Cap-lean optimal line shortens to ~2.2 months]** → proportionally identical to before (0.8–0.9 months under balanced); the optimal line being brisk is the reward for skill, and the ward branch remains the long game.
- **[Any hidden dependency on old constants]** → full `make check` plus the voyage simulator; the corpus fixtures pin serde shape, not balance.

## Migration Plan

Constants-only; no save migration (yard levels persist, their effects recompute). Ships in the same PR as the hardening change (same branch), with the PR description updated to declare the balance change explicitly.

## Open Questions

- The ward branch now runs ~7.2 months (was ~9). If a future pass wants it nearer the old "~5 months" note, the lever is the Ward price ladder — deliberately untouched here.
