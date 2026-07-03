# The Arrival

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 7 of 7 — the last spec of the act.
**Depends on:** specs 2–6 (all shipped). Everything this spec renders
already exists in the save: `visited`, `untaken`, `keepsakes`, the Log,
the letters kept, the carved names, the soul roster with its resolved
arcs, and W37's authored three-beat scene behind `take_finale_playback()`.
**Feeds:** Act 3 (a single flag and a single face; nothing more).

## Overview

The crossing ends. This spec is about *how it ends being worth the
20–200 days it took*: the final approach staged as the act's longest
scene, and then — instead of a victory screen — three quiet rooms the
player can sit in for as long as they like. **The manifest** (who came,
who joined, who was lost and where their name is carved). **The keepsake
chart** (the whole crossing, the roads not taken crossed out forever).
**The record** (the Log, complete, with the letters bound in).

The design law that has governed every spec still governs the last one:
**memory, never score** (No Right Path, rule 6). There is no grade, no
percentage, no "souls saved: 6/8", no comparison against a route you did
not sail. The manifest is a ship's document, not a report card. The
numbers that appear (days at sea, letters kept) are facts a captain would
know, not metrics a game is judging.

And the act keeps its other promise to the very end: **what an untaken
road held is never revealed** (rule 3). The keepsake chart shows the shape
of the roads you crossed out, not their contents. The fog over unvisited
waypoints never lifts. You will wonder. That is the point.

## The Final Approach

Arrival at W37 currently fires the three authored beats ("The Tree." /
the sight of it / lines thrown, hands catching). Spec 7 extends the
finale playback — one staged sequence, read at the player's pace like any
scene, built at `take_finale_playback()` time from the save:

1. **The Tree** — W37's existing three beats, unchanged.
2. **The rail** — one authored line per soul *aboard*, in boarding order,
   each in their own voice, each colored by their resolved arc (a soul
   whose resolution fired at the sink still counts as resolved — the
   engine already guarantees no arc is left dangling). Souls who stepped
   ashore earlier get no rail beat; their goodbye already happened and is
   in the Log.
3. **The carved names** — if any soul was lost, one beat: the crew
   gathers at the rail where the names are carved, and reads them within
   sight of the Tree. Skipped entirely on a lossless crossing (absence of
   grief is not announced; it is simply a shorter scene).
4. **The harbor** — the Sister Verity is moored in the root-harbor
   (already authored: `dark_after: None`, "a face for Act 3"). Her
   mate waves the Vessel in. One beat, and she is the only outward hook
   this act plants.
5. **The lamp** — the closing beat. A door in the root-wall, closed, with
   a lamp lit beside it. Nobody says "Act 3". The beat says the harbor
   has rooms, the rooms have doors, and one of them is yours, later.

The whole sequence is still one `ScenePlayback` through the existing
pager — no new UI machinery. `finale_shown` still latches it to exactly
one showing; the harbor screen (below) is what you return to forever.

## The Harbor (the Arrived state's home screen)

After the finale, the voyage screen's Arrived state stops being a stub
and becomes a small permanent place: the chart panel shows the Vessel
moored at the Tree (sea calm, no weather objects — `weather_at` simply
isn't consulted), and the side panel lists what there is to sit with:

| Key | Room |
|-----|------|
| `[M]` | The manifest |
| `[K]` | The keepsake chart |
| `[R]` | The record (the Log, complete) |
| `[Q]` | Back to the title flow, as today |

Nothing ticks. Provisions and hope gauges are retired from the panel
(the crossing is over; the gauges were the crossing). Time-at-the-Tree
is not measured. This screen is deliberately a museum, not a lobby.

## The Manifest

One scrollable panel, a ship's document in four parts:

1. **The crossing** — vessel name, the launch date and arrival date (game
   dates), days at sea (from `at_min`), the chapters crossed. Facts only.
2. **The souls** — every soul *met*, grouped by how their story ended:
   - **Came ashore** (aboard at arrival): name, station history's last
     post, and their arc's resolution beat *title* (the Log has the text).
   - **Went their own way** (ashore/farewell): name, the waypoint where
     they stepped off.
   - **Carved** (lost): name, and the line already used at the memorial.
   Souls never met are **not listed** — the manifest records the crossing
   that happened, not the cast that exists (rule 3 again, applied to
   people).
3. **The hold** — keepsakes, in the order collected, each with the
   waypoint it came from; then "Letters kept: N" and the senders (the
   letters themselves remain readable in the record).
4. **The wake** — rumors heard, refits taken (and, for each refit taken,
   the name of the door that closed — the one place the manifest admits a
   road not taken, because the player chose it looking at both).

No totals, no ranks, no stars.

## The Keepsake Chart

`[K]` opens the chart full-screen — the same renderer, three changes:

- **Pan** with arrow keys (the full canvas is bigger than any terminal;
  until now the viewport followed the Vessel; now it follows the player).
- **The sailed route** renders bright: visited waypoints ◉, roads sailed
  solid, in visit order discernible by the line style already used.
- **Untaken roads** keep their `✕` forever, and unvisited waypoints keep
  their fog glyph and their namelessness. The chart never becomes a map
  of the world; it stays a map of *your crossing* through it.

No new data: `visited` and `untaken` are already the full record.

## The Act 3 Gate

One new persisted fact and nothing else:

```rust
// GameState (serde default: false — every existing save loads clean)
pub vessel_arrived: bool,
```

Set exactly once, when the finale playback is taken (same latch as
`finale_shown`, but on the account-visible side: `main.rs` sets it and
saves when it surfaces the finale). Act 3, whenever it is designed, keys
off `vessel_arrived` the way Act 2 keyed off `vessel_launched` — and the
Sister Verity in the harbor is its authored face. This spec deliberately
plants *no other* Act 3 machinery: no currencies, no unlocks, no teaser
menu. The closed door with the lamp is a sentence in a scene, not a
locked UI element (locked UI is a promise with a countdown; a lit lamp is
a promise without one).

The kill-switch discipline is unchanged: all of this ships dark behind
`ACT2_ENABLED = false`, and enabling the act remains the deliberate
two-line PR it has been since spec 1.

## Data Model (build scope)

```rust
// GameState — the only cross-act surface
vessel_arrived: bool,                    // serde(default)

// src/vessel/souls.rs — authored additions
pub struct SoulDef {
    // ...existing...
    /// One line at the rail, in their voice, read at the finale
    /// if they are aboard at arrival.
    pub rail_line: &'static str,
}

// src/vessel/voyage.rs
// take_finale_playback() grows from "W37's scene" to the staged
// sequence above (rail beats + carved names + harbor + lamp appended
// to the authored beats). Pure function of VoyageState — offline ==
// live, chunking-invariant, same as everything else.

// Manifest/chart/record are pure render-side reads of VoyageState —
// no new serialized voyage fields at all.
```

UI: `VoyageView` grows `Manifest` and a pannable chart offset for the
Arrived state; input routes `[M]`/`[K]`/`[R]` only when
`voyage.arrived()`. The record reuses the existing Log panel rendering
with the pager.

## What This Spec Does NOT Add

No score, grade, rank, or completion percentage. No reveal of untaken
roads or unmet souls. No new-game-plus, no second crossing, no "sail
again" prompt. No Act 3 content beyond one flag and one moored ship. No
post-arrival economy — nothing at the Tree costs or pays anything. No
changes to Act 1 (still frozen fiction; whether its numbers ever render
again is Act 3's question, unanswered here on purpose).

## Testing

- The finale sequence is a pure function of the save: same VoyageState →
  identical beats (snapshot the playback for a fixture crossing with a
  loss, a farewell, and a full hold; and for a minimal lossless one).
- Rail beats appear for exactly the souls aboard at arrival, in boarding
  order; the carved-names beat appears iff `carved_names()` is non-empty.
- `vessel_arrived` is set exactly once, persists, and old saves load
  with it false (save-compat corpus).
- Manifest lists only met souls; groups match roster statuses; keepsakes
  and letters match the hold; no numeric field beyond days and counts.
- Keepsake chart: pan clamps to canvas; fog glyphs and `✕` render for a
  fixture with untaken junctions; no unvisited waypoint name appears
  anywhere in the buffer (grep the frame — this is rule 3 as a test).
- Overlay snapshots for harbor screen, manifest, and keepsake chart at
  XL and S tiers; input tests for [M]/[K]/[R] gating on `arrived()`.
- The full-crossing simulator still lands inside the 20–200 day envelope
  and `finale_shown`/`vessel_arrived` latch once across save/load.

## Open Questions

- Whether the record `[R]` should paginate the letters as full re-readable
  text or keep the Log's one-line "kept" entries (lean: full text — the
  letters are the act's best writing and the Going-Dark made them finite).
- Whether the harbor screen should show the Sister Verity's lamp on the
  chart as a persistent glyph (lean: yes, one ☀ at the sink — cheap, and
  it keeps the Act 3 face visible without a single word of UI).
- Whether arrival should be announced to the Act 1 title screen (e.g. the
  character select shows "— arrived —" instead of a zone). Lean: yes but
  trivial; decide at build time.
