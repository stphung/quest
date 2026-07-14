# Design — Act 2 Collection Achievements

## Context

Act 2's voyage engine (`src/vessel/`) already tracks everything the seven new achievements need — `VoyageState.visited/hailed/rumors/refits/souls`, `ColonyState.records.{most_carried,fastest_days}` — but three structural facts shape the design:

1. **Per-crossing resets.** `visited`, `hailed`, `rumors`, `refits` all live on `VoyageState` and reset at every `begin()`/`begin_ferry()`. The two "collect across the era" achievements (waypoints, pilgrims) therefore need persistent union aggregates outside the voyage state.
2. **The existing achievement integration is observational.** The voyage branch in `main.rs` calls `Achievements` handlers (`on_crossing_delivered`, `on_vessel_arrived`, …) at the delivery block (`take_finale_playback()`); unlocks drain through `take_modal_queue()` into `voyage_ui.moments`. The engine itself has zero achievement coupling — worth preserving.
3. **The dark-ship invariant.** All unlock paths must be confined to the voyage loop so the seven new rows stay visible-but-unearnable while `ACT2_ENABLED` is off (pinned by `vessel_achievements_stay_visible_while_act2_is_dark`).

Threshold evidence (RAMPDBG sweep of `run_era_with` policies, seed-stable): balanced reaches Drive Lv9 / ~7.3 sea-days steady state and carries 4,136+ by crossing 9; drive-only tops out at 810 carried (base 180 + all districts); cap-only never drops below ~43 sea-days; ward-lean plateaus at Drive Lv6 / ~9.2 sea-days, Cap Lv6 / 2,953 carried.

## Goals / Non-Goals

**Goals:**
- Seven achievable, playstyle-shaped achievements observing existing voyage state.
- Zero changes to voyage engine behavior or balance constants.
- Thresholds pinned by tests that fail if a future retune makes one unreachable.

**Non-Goals:**
- Per-beat arc recognition (Chapter IV force-skips intermediate beats silently — engine change required, deferred).
- Any "all six refits" / "all eight arcs" style impossible collection.
- Wiki documentation (deferred per direction).

## Decisions

### D1 — Observation over engine hooks
New `Achievements` handler methods are called from the `main.rs` voyage branch; `src/vessel/` is untouched. Two call shapes:

- **Per-frame observer** `on_voyage_observed(visited_mask: u64, hailed_mask: u8, rumors_known: usize, refits_taken: usize)` — called each voyage frame after stepping. Unions the two masks into the persistent aggregates and evaluates the four collection unlocks (Every Star a Harbor, Company on the Road, Ear to the Water, Three Doors Opened). `main.rs` converts `Vec<WaypointId>`/`Vec<u8>` to bitmasks (38 waypoints fits u64; 5 ships fits u8). All operations are O(1) bit math at 10 Hz — negligible.
- **Landfall observer** `on_landfall(carried: u32, days: u64, aboard: usize)` — called once per delivery, alongside the existing `on_crossing_delivered`, evaluating Heavy Lading (`carried >= 2_500`), The Swift Passage (`days <= 8`, inclusive — the drive-heavy floor is exactly 8 sea-days), and The Full Table (`aboard >= souls::CREW`).

*Alternative considered:* extending `on_crossing_delivered`'s signature — rejected; it already has three parameters with distinct semantics (lifetime max, per-crossing loss, name) and its tests pin the current shape. A second landfall method keeps both readable.

*Why per-frame rather than event-hooked:* rumor grants, hails, and docks happen deep inside `step_minute`/`arrive_at`; hooking each would thread achievement state through the engine. Polling the resulting state is idempotent, catches offline-resolved progress on the same frame it surfaces, and cannot miss a transition (the masks/lengths are monotone within a crossing).

### D2 — Completion targets reference vessel constants, not literals
The handler compares against `route::WAYPOINTS.len()`, `pilgrims::PILGRIMS.len()`, `route::RUMORS.len()`, `refits::REFIT_PAIRS.len()`, `souls::CREW` (same crate, always compiled). If authored content grows (a 39th waypoint), the achievement's target grows with it instead of silently completing early. The two tuned thresholds (2,500 souls; 8 sea-days) are new named constants in `achievements/data.rs` with doc comments carrying the empirical rationale.

### D3 — Aggregates live on `Achievements` (account-level), as bitmasks
`waypoints_docked_mask: u64` and `pilgrims_hailed_mask: u8`, both `#[serde(default)]` — same pattern and save-compat posture as `total_souls_delivered`. Account-level union across characters matches the existing Vessel aggregates' semantics (Ferryman tiers already accumulate account-wide). Bitmasks keep the save format compact and the union operation trivial.

*Trade-off:* an unlock between autosaves can be lost to a crash, but re-derivation is automatic (the masks re-union from the still-persisted voyage state next session) — same exposure as every existing per-frame achievement.

### D4 — Full Table condition is seat-count at landfall, not crossing number
`aboard >= CREW` can only first become true at the maiden finale (recruits only ask on crossing 1), and ferry-run arrivals with a full surviving crew re-satisfy it harmlessly (idempotent unlock). No crossing-number special-casing.

### D5 — Thresholds pinned empirically in `ferryman_tests`
A new assertion inside the existing era harness: the balanced policy's finished era must satisfy `records.most_carried >= HEAVY_LADING_SOULS` and `records.fastest_days <= SWIFT_PASSAGE_DAYS`, and the formula floor `BASE_CAPACITY × CAP_GROWTH⁷ ≥ HEAVY_LADING_SOULS` holds. A future retune of `CAP_GROWTH`/`DRIVE_DECAY`/port-call time that strands either achievement fails CI at the balance gate rather than shipping an unearnable milestone.

### D6 — Category placement and points
Voyage +4 (Every Star a Harbor 100 pts + **Wayfarer** title; Company on the Road 50; Ear to the Water 50; Three Doors Opened 25), Ferry +2 (The Full Table 50; Heavy Lading 25), Era +1 (The Swift Passage 25). Values come from the established 5/10/25/50/100/250/500 tier scale and match the existing seven (The Burn 50, Ferryman tiers 25/50/100). One new title only — Wayfarer for the chart-completion capstone, mirroring how Act 1 reserves titles for chase achievements.

## Risks / Trade-offs

- [Thresholds drift under future retunes] → D5's empirical pin in the era harness fails the Balance job.
- [Authored content grows (waypoints > 64)] → u64 mask overflows; a unit test asserts `WAYPOINTS.len() <= 64` with a pointer to widen the mask.
- [Per-frame handler called outside voyage (Act 1)] → it never is — the only call site is inside the voyage branch, which is unreachable while the kill-switch is off; the visibility pinning test keeps the rows browsable-but-locked.
- [Union masks span characters] → accepted; consistent with account-level achievement semantics (documented in the spec delta).

## Migration Plan

Pure addition: new serde-default fields, new enum variants appended (achievements serialize by name, not index — existing saves unaffected; save-compat corpus exercises the defaults). No rollback complexity: reverting the commit restores 247 achievements; saves carrying the two extra fields deserialize cleanly into older code only if fields are unknown-tolerated — they are (`serde_json` default behavior on the `Achievements` struct ignores unknown fields).

## Open Questions

(none — thresholds resolved empirically, placement resolved by D6)
