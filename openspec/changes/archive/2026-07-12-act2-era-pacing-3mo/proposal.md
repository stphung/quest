# Act 2 Era Pacing — 3-Month Balanced Campaign

## Why

The ferry era's balanced line measured 29 crossings / 4.0 real months — a month past the design's original "~19–24 crossings, ~3 real months" target, a drift accepted-by-documentation during the release-hardening pass. Per direction (2026-07-12), the campaign is being tuned back to the ~3-month target before release, now that the asserted balance gates make a constants change visible and safe.

## What Changes

- **`CAP_GROWTH` 1.36 → 1.46**: each Shipwright level widens the hold more, so the balanced era completes in fewer, fuller crossings — the dominant lever for era length (measured sensitivity: balanced 4.0 → 3.1 months).
- **`DARK_TAKES_PER_DAY` 0.0006 → 0.0007**: a slightly hungrier dark, compensating the bigger holds so the naive extremes stay traps and the skill margin stays wide (drive-only 67.1% saved, cap-only 78.5% — both clearly inferior to balanced 88.6%).
- **Balance gates tightened** to enforce the new target: balanced 15–30 crossings / 2.5–4.5 real months (was 20–40 / 3–6), ≥84% saved (was ≥82%).
- Measured landing (deterministic sweep, 2026-07-12): balanced **22 crossings / 3.1 mo / 88.6%**; cap-lean (souls-first) 12 / 2.2 / 90.1%; ward-lean 44 / 7.2 / 93.2%; drive-only 99 / 11.1 / 67.1%; cap-only 10 / 6.3 / 78.5%. C1 stays ≈15 real days (the voyage clock is untouched).

## Non-goals

- **No change to the voyage clock** (`GAME_MINUTES_PER_REAL_MINUTE`) — the two-real-week maiden voyage is a design identity and stays exactly as-is.
- **No change to Riftglass/Dock constants** — dock time is ~0.1 month of the era; not the lever.
- **No change to the Ward branch's identity** — it remains the deliberate "slower era, most souls saved" line (measured 7.2 months / 93.2%); compressing it toward the old ~5-month note is out of scope.
- **Not flipping `ACT2_ENABLED`.**

## Balance/progression impact

This IS a balance change, to the Act 2 ferry era only: the balanced campaign shortens from ~4.0 to ~3.1 real months and saves slightly more of the world (87.5% → 88.6%); the reckless lines get slightly worse (the hungrier dark). Act 1 progression is untouched (`--check-progression` unaffected). Voyage-level pacing (road days, provisions, drift) is untouched — only the colony economy above it.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `vessel-act2`: the **Ferry-Era Balance Envelope** requirement's normative bands tighten to the 3-month target (15–30 crossings, 2.5–4.5 months, ≥84% saved balanced).

## Impact

- **Code**: `src/vessel/colony.rs` (two constants + doc comments).
- **Tests**: `tests/ferryman_tests.rs` (tightened bands, refreshed measured-value comments); any overlay snapshots that render colony-derived numbers (re-blessed after review).
- **Specs/docs**: `openspec/specs/vessel-act2/spec.md` (envelope requirement), `src/vessel/CLAUDE.md` (constants table + tension paragraph), `docs/dossiers/act2-pilgrimage.md` (balance evidence refresh note).
