# The Souls — Roster, Stations, Arcs, and the Wind

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 3 of 7
**Depends on:** spec 2 (shipped — route graph, voyage state machine, junction
cards). **Feeds:** spec 4 (arrival scenes are where souls board, speak, and
are lost), spec 5 (night suitability, watch-vs-rest), spec 7 (the manifest).

## Overview

Act 1 was a solo power fantasy. Act 2's cast is the point: **the Vessel
carries people, and the game's second gauge measures how they are doing.**
This spec defines the roster (who exists, where they are met), stations
(the standing assignment loop), arcs (each soul's personal thread), hope's
one mechanical effect (the wind), and loss (authored, priced, memorialized,
and never offline).

Everything here follows the parent pillars: small numbers, no hidden
arithmetic on screen, doors close, and the covenant — nothing harms a soul
while the player is away.

## The Roster

### Three board at launch

Faces from the systems the player mastered. Their dossiers are the tutorial
for every soul mechanic.

| Soul | Origin | Voice (one line) | Station affinity | Night suitability |
|------|--------|------------------|------------------|-------------------|
| **Torvald** | the Deep's guild captain | "I've been lower than dark. It blinks first." | Helm | steady on strange nights (nothing bad, learns nothing) |
| **Eir** | the Haven's warden | "A ship is a house that argues with the sea." | Tender | careful in the cold (cold nights cost less) |
| **Runa** | the fisher | "Everything worth catching sings first." | Watch | answers singing nights (earns the rumor/beat) |

### Five found along the route

Each recruitable soul has **one site per branch arm** at the junction diamond
where they are met, so every route meets every recruitable — *different
scenes, same person* (content-parity rule 5: different souls never means
fewer souls). Sites use the existing `Feature::SoulCandidate` waypoints.

| Soul | Met at (any one of) | Who they are | Affinity |
|------|---------------------|--------------|----------|
| **Maren** | the Lightship Vigil (W1, spine) | the last lightship keeper; asks to see one more lit lantern | Watch |
| **Sefa** | the Drowned Choir (W3) · the Kelp Meadows (W5) | the last cantor of the drowned parishes | — (sings; singing nights) |
| **Ysolt** | Saint Elm's Rest (W14) · the Beacon Graveyard (W16) · the Pilgrims' Buoy (W18) | a mender of hulls and of the other kind of damage | Tender |
| **Cormac** | the Whale Roads (W20) · the Smugglers' Slip (W21) | a pilot who knows the roads nobody charts | Helm |
| **Brother Wren** | the Choir of Bones (W26) · the Sleepers' Trench (W28) | woken from a years-long sleep; remembers the deep from inside | Keel |

The remaining soul-candidate waypoints (W9 the Ossuary Reef, W12 the
Wandering Fair) host **arc beats**, not recruitments.

### Berths: seven

3 launch + 5 found = **8 possible asks against 7 berths.** A player who says
yes to everyone faces exactly one **farewell** — naming who steps ashore at
the next waypoint (a scene, a manifest line, and a hope cost; never a
death). Declining an ask is permanent: that soul's door closes, their name
stays on the chart. The berth question is always *who*, never upkeep —
souls eat nothing.

Boarding and farewell are **scenes** (spec 4 delivers them); this spec owns
the state machine: `Ask → Aboard | Declined`, `Aboard → Ashore (farewell) |
Lost (authored only) | Arrived`.

## Stations

Four standing posts, assigned from the Souls panel (`[S]`), persistent
until changed (including offline). One soul per station; unassigned
stations simply lack the effect. **A soul at a station is not resting** —
the same coverage-vs-story trade as the night watch (spec 5), and the core
reason a 7-berth roster matters: four posts, and arcs only move for the
souls you let rest.

| Station | Standing effect (any soul) | With affinity soul |
|---------|---------------------------|--------------------|
| **Helm** | legs 4% faster | 8% faster |
| **Tender** | leg provisions −5% | −10%, and cold/hungry nights resolve one grade kinder |
| **Watch** | default assignee for typed nights (spec 5) | singing/strange nights resolve one grade kinder |
| **Keel** | dark-tagged roads don't fray hope | named-threat roads re-price one step kinder (hook consumed by spec 4 scenes) |

Rules that keep the arithmetic invisible on screen:

- Effects surface **only as final prices** — the junction card and trim
  panel already print composed integers; stations change those integers,
  never show a multiplier.
- Composition: `time = base × trim × wind × helm`, provisions likewise
  with tender. Order is fixed and documented in code.
- Counsel, not buffs, is the visible face: at a junction, each soul aboard
  may contribute **one line of counsel** on one card, in their voice
  ("Torvald: 'The narrows are honest. Hungry, but honest.'") — authored per
  (soul × road), a bounded writing table (~2 lines per soul per junction
  where they have an opinion; most souls are silent at most junctions).

## Arcs

One personal thread per soul: **three beats and a resolution**, unfolding
at particular places and paid for in rest days.

- **Beat advancement**: a beat becomes *ready* when its trigger is met
  (reach a tagged place, sail a tagged road, a chapter boundary — all
  authored per-soul); a ready beat **fires after the soul accumulates 2
  rest days** (days neither stationed nor standing watch). The Souls panel
  shows `rest days to next beat: 1 of 2` — the price is always visible.
- **Beat payout**: a log entry in the soul's voice, and one of: hope +1,
  a rumor (their private knowledge becomes a chart annotation), or a
  junction counsel upgrade. Resolutions pay hope +2 and a manifest line.
- **Arcs resolve before the Roots**: any unresolved arc fast-forwards its
  resolution scene during the Chapter IV approach — nobody's story is left
  dangling at the finale (spec 7 reads resolutions into the manifest).
- Beats never punish: an ignored arc simply waits. The only cost of
  neglect is time (and the finale reflecting what was and wasn't heard).

Example (Sefa): boards singing a lament (beat 0) → *the Ossuary Reef or
any LanternSite*: she asks to sing for the dead (beat 1, hope +1) → *first
silence-bank survived* (Chapter III): her voice is the thing the silence
couldn't take (beat 2, rumor) → resolution: she teaches the crew the
evening office; singing nights are kind to everyone forever (resolution,
hope +2, manifest line).

## Hope is the Wind

Hope gets its one mechanical effect (parent spec: "high hope is wind").
Bands, composed into leg time like trim:

| Hope | Label range | Wind |
|------|-------------|------|
| 8–10 | high / singing / radiant | legs 10% faster |
| 5–7 | steady / warm / bright | — |
| 3–4 | low / uneasy | legs 10% slower |
| 1–2 | guttering / failing | legs 25% slower |
| 0 | ashen | **the Long Silence** |

**The Long Silence**: legs crawl (+40%) and arcs pause. It breaks at the
next RestStop arrival (the scene plays a fire relit) — hope returns to 3
("low"). It is a valley, not a fail state; the crossing still cannot be
lost.

What moves hope (consolidated; all sources land in the two-gauge economy):

- **Up**: arc beats (+1) and resolutions (+2), Mourn trim at sea (+1/day),
  kept letters (spec 6), a handful of authored scene payoffs (spec 4)
- **Down**: hold-station past grace (−1/day, floor "steady"), soul loss
  (−3), farewell (−1), a named-threat scene gone badly (authored, priced
  on the card), silence-banks unshielded (spec 5)

## Loss and the Memorial

- Souls are lost **only in authored scenes** attached to named threats —
  the threat was on the junction card, the road was chosen, and the scene
  offered a priced alternative. No dice: loss follows from a choice whose
  stakes were stated (parent rule: catastrophes are priced, chosen, and
  become story).
- **The covenant, mechanical form**: no tick-driven code path may reduce
  the roster. Nights, weather, drift, hold-station, offline resolution —
  none touch souls. CI enforces this as a property test (simulate
  arbitrary offline windows; roster count is invariant).
- A loss carves the soul's name into the hull: the ship art gains a
  carved-name line for the rest of the game (Act 3 included), the arc
  becomes a memorial manifest line, hope −3, and their counsel lines go
  silent — the junction feels emptier, which is the design intent.

## UI Surfaces

| Surface | Content |
|---------|---------|
| Souls panel (`[S]`) | roster with faces/voice lines, station assignment, arc status ("rest days to next beat"), dossier per soul |
| Junction cards | counsel lines (soul voice + road read) appended to existing annotations |
| Chart | recruit sites render their `SoulCandidate` marker with a small `☺`-style accent once known |
| Boarding ask / farewell | scene modals (spec 4 shapes; engine provides the state transitions) |
| Ship art | carved names after losses |
| Vessel panel | hope line gains its wind arrow (`hope: bright ↑` when wind aids, `↓` when it drags) |

## Data Model (engine-side, this spec's build scope)

```rust
pub struct SoulDef {            // authored table, like route.rs
    id: SoulId,
    name: &'static str,
    voice: &'static str,        // the one-line personality
    origin: &'static str,
    affinity: Option<Station>,
    night_note: &'static str,   // dossier text; spec 5 consumes the typed form
    sites: &'static [WaypointId],  // empty = boards at launch
    arc: &'static [ArcBeat],    // trigger + payout per beat
    counsel: &'static [(RoadId, &'static str)],
}

pub struct SoulState {          // in voyage.json
    soul: SoulId,
    status: SoulStatus,         // Aboard | Declined | Ashore | Lost | (implicit: NotMet)
    station: Option<Station>,
    arc_beat: u8,
    rest_day_minutes: u64,      // accumulates only while resting
}
```

Constants: `BERTHS = 7`, `ARC_BEAT_REST_DAYS = 2`, wind bands above,
station multipliers above, `LOSS_HOPE_COST = 3`, `FAREWELL_HOPE_COST = 1`.

## What This Spec Does NOT Add

No soul stats, levels, or equipment. No morale-per-soul (hope is one
shared gauge). No procedural souls — eight authored people, full stop. No
soul death outside authored scenes. No upkeep.

## Testing

- Roster invariants: every maximal route passes at least one site of every
  recruitable soul (per-soul cut across all 96 routes — extends the
  existing parity test)
- Berth overflow: 8 asks against 7 berths forces exactly one
  farewell-or-decline; both paths permanent
- Covenant property test: no sequence of offline ticks changes roster
  count or fires a loss
- Wind: hope bands compose into leg time deterministically; Long Silence
  pauses arcs and breaks at a RestStop; offline == live still bitwise
- Arc engine: beats gate on trigger + rest days; stationed souls
  accumulate none; resolution fast-forward at Chapter IV
- Station effects: final card prices shift by the documented composition;
  snapshot tests cover the Souls panel and counsel-bearing junction cards

## Open Questions

- Should the Watch station soul auto-cover *all* typed nights, or only
  when the forecast panel is untouched (lean: default assignee, editable
  per night — keeps spec 5's panel meaningful)?
- Farewell timing: immediate at the current waypoint vs "at the next
  harbor" (lean: next waypoint with a scene, so it lands as story).
- Whether counsel lines should ever disagree with the card's own tags
  (lean: yes, rarely, and authored — Torvald being wrong once is worth
  more than him being a UI hint forever).
- Hull carving for *farewelled* souls (lean: no — carvings are for the
  lost; the manifest remembers the ashore).
