# Act 2 Collection Achievements

## Why

Act II's achievement subsections are thin — The Voyage (2), The Ferry (3), The Era (2) — and none of the act's richest authored content (the 38-waypoint chart, rumors, refit doors, pilgrim ships, the crew's seven berths, the three Salvage yards) is recognized by any milestone. A feasibility investigation against the voyage engine (route graph reachability, per-crossing state resets, the CREW=7 structural cap, empirical era sweeps) identified seven achievements that are all genuinely earnable and that each steer players toward an under-touched mechanic. Naïve versions of several ("all six refits", "all eight soul arcs") are provably impossible; this change ships the verified-achievable set instead.

## What Changes

Seven new Vessel achievements (247 → 254 total), all dark-shipped behind the Act 2 kill-switch like the existing seven — visible-but-unearnable while dark, unlock paths confined to the voyage loop:

**The Voyage** (2 → 6):
- **Every Star a Harbor** — dock at all 38 waypoints, cumulative across crossings (per-crossing `visited` resets, so this adds a persistent union aggregate). Requires manually steering ferry-run junctions: autopilot repeats a fixed 26-waypoint path, and the 3-way Wandering Fair junction guarantees autopilot-only eras cap at 36/38. Grants the **Wayfarer** title.
- **Company on the Road** — hail all 5 pilgrim ships, cumulative across crossings (persistent union aggregate; single-crossing alignment is a brutal scheduling puzzle, the lifetime union is a fair chase).
- **Ear to the Water** — know all 8 rumors within a single crossing (rumors reset per crossing; 4 buyable at way-stations, the rest from night/pilgrim/arc/scene grants).
- **Three Doors Opened** — take a refit at all three shipyard doors in one crossing (maiden-voyage-only mechanic; requires routing through an optional third shipyard and never spending a door on hull mending).

**The Ferry** (3 → 5):
- **The Full Table** — reach the Tree with all seven berths filled (accept asks to a full crew, farewell no one; completes the 7 achievable soul arcs).
- **Heavy Lading** — deliver 2,500+ souls in a single crossing (`most_carried`). Cap Lv7 alone (180 × 1.46⁷ ≈ 2,554) or Lv6 + district bonuses clears it; a no-capacity line maxes at 810 (base + all districts) and correctly never qualifies.

**The Era** (2 → 3):
- **The Swift Passage** — complete a crossing in 8 sea-days or fewer (`fastest_days`, an era record; inclusive threshold). Drive-heavy lines floor at exactly 8 sea-days (port calls dominate once Drive compounds toward its 0.05 floor); ward-lean plateaus at 10, a no-drive line sits near 40.

Supporting changes:
- Two persistent union aggregates on `Achievements` (serde-default, save-compat safe): waypoints-docked bitmask, pilgrims-hailed bitmask.
- Idempotent poll-based unlock detection from the voyage loop in `main.rs` (observing `VoyageState`/`ColonyState`/`CrossingRecords`), keeping the voyage engine free of achievement coupling.
- Category/act partition tests, `VARIANT_COUNT`, sanity slices, and snapshot counts updated.

## Non-goals

- No new voyage mechanics, balance constants, or engine changes — the seven achievements observe existing state only.
- No "all six refits" / "all eight arcs" achievements (structurally impossible: refit doors are 3 one-of-two picks; CREW=7 < 8 souls with no re-boarding).
- No per-beat arc tracking (intermediate beats can be silently force-skipped at Chapter IV; recognizing "every beat seen" would require engine changes — deferred).
- No wiki page (player-facing wiki remains deferred per direction).
- No change to the existing seven Vessel achievements or their categories.

## Balance / progression impact

None on Act 1 or on voyage pacing: no engine constants change. The thresholds are recognition lines against the existing envelope (empirical sweep: balanced hits both Heavy Lading and Swift Passage mid-era; specialist lines each miss the opposite branch's achievement, which is the intent). Achievement points added: 325 across seven (tier scale only: 100/50/50/25/50/25/25) (max score rises accordingly; snapshot totals re-blessed).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `achievements`: total count 247 → 254; The Voyage/The Ferry/The Era category compositions grow; two new persistent collection aggregates with their update semantics; one new title (Wayfarer).
- `vessel-act2`: a new requirement that the voyage loop surfaces collection/records state to the achievement layer via idempotent observation (no engine hooks), preserving the visible-but-unearnable-while-dark invariant.

## Impact

- `src/achievements/types.rs` (7 new `AchievementId` variants, `VARIANT_COUNT`, aggregates), `data.rs` (7 defs + points + title), `handlers.rs` (new `on_*` observers + unit tests).
- `src/main.rs` voyage branch (poll-based unlock calls, modal drain already in place).
- Tests: partition/sanity/count tests, `vessel_visibility_tests`, save-compat corpus additions for the new serde-default fields, flag-on coverage for one unlock path, feasibility-pinning assertions (thresholds vs. formulas: Heavy Lading ≤ cap-Lv7 hold, Swift Passage ≥ port-call floor).
- Snapshots: achievement browser act row counts (0/7 → 0/14), overlay/browser snapshots re-blessed.
- Docs: `src/achievements/CLAUDE.md`, `src/vessel/CLAUDE.md` count references.
