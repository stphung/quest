# Underway — Weather, Trim, and the Watch

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 5 of 7 (replaces the old "Vessel Underway" scope)
**Mockups:** the "Traveling — Mid-Leg Check-In" set (four screens: return/log,
living chart, trim, watches)

## Overview

Traveling is not dead time with flavor text. While the Vessel is on a leg:
**the void has weather** (objects that move on the chart in real time),
**the ship has trim** (one posture dial, priced against today's weather),
and **the night has watches** (typed nights, scheduled souls, and a log the
nights write themselves). Other pilgrims' ships cross the same chart on
their own roads.

The design contract, inherited and kept: **no dice, no menus**. Every travel
effect is a standing choice with a stated price (trim, watch assignment) or
a world event that was visible before it arrived (weather on the chart,
night types in the forecast). All effects land in the two existing gauges,
time, rumors, or the log — no new resources.

**The check-in loop** (2–4 minutes, any time mid-leg):
1. **Read** — the log of the nights since you left (written, not tallied)
2. **Look** — the chart: what weather moved, who else is out there
3. **Trim** — one dial if the weather changed
4. **Watch** — fill any unassigned typed night in the forecast

Design budget: **at most one thing per check-in actually asks** (an
unassigned strange night, a current crossing the road). Everything else is
readable state.

---

## Void Weather

### Object types (v1: exactly three)

| Type | On the chart | Effect while it overlaps your leg | Trim interaction |
|------|--------------|-----------------------------------|------------------|
| **Current** (e.g. "the Northing") | `≋ ≋ ≋` band with a bearing, drifting across edges | None passively — currents are *opportunity* | **Run** rides it: big time gain at no extra provisions burn. Against-bearing currents make Run cost more instead |
| **Silence-bank** | `▒▒▒` shaded region | Hope −1/day while inside | **Quiet** nullifies the hope drain and can hear what the silence hides (a rumor, once per bank) |
| **Squall** | `≈≈*≈` flickering patch, fast-moving | Provisions −2/day while inside | **Mourn**/**Quiet** shelter through it (half cost); **Run** through a squall doubles its tax |

Weather never damages souls and never touches arcs. It prices time, hope,
and provisions — the things the player already trades.

### Lifecycle and visibility

- Weather objects live on the **route graph** (edges and small edge-regions),
  not free 2D space. They spawn, drift edge-to-edge on a bearing, and
  dissipate after **2–5 real days**.
- **At most 2** weather objects may affect the player's current leg at once;
  at most 4 are visible on the chart (the rest of the void's weather is
  simply not shown).
- The chart shows weather within one junction's distance — same horizon as
  the fog-of-route. Rumors can report weather further out ("the Northing
  runs the Tollgate road all season"), and such rumors are *forecasts*,
  aging out when the weather does.
- Movement steps happen on wall-clock hours (deterministic; see Determinism)
  so a check-in every few hours genuinely sees things move.

### Generation

Each chapter has an authored **weather deck**: 6–10 template cards
(type, strength, typical bearing, flavor name pool) with chapter character —
the Shallows run mild currents; the Starless Deep deals long silence-banks;
squalls cluster at chapter boundaries. Draws and drift steps are seeded
(see Determinism). Named story weather (e.g. a scripted silence-bank at the
Going-Dark) is placed by the route script, not drawn.

## Trim

One posture, settable any time aboard, persistent until changed
(including offline). Base table:

| Trim | Leg time | Leg provisions | Extra |
|------|----------|----------------|-------|
| **Run** | ×0.80 | ×1.30 | Pilgrim ships you're pacing fall behind |
| **Cruise** | ×1.00 | ×1.00 | The default; never wrong, never best |
| **Quiet** | ×1.20 | ×0.90 | *Hears more*: silence-banks yield their rumor; singing/strange nights resolve one grade kinder |
| **Mourn** | ×1.40 | ×0.90 | Hope +1/day; the only trim that raises hope at sea |

Weather multiplies on top (composition rule:
`time = base_time × trim_time × weather_time(trim, weather)`), and the trim
panel always shows the **final computed prices** — "arrive ~14h early ·
provisions −9 over the leg" — never the multipliers. Small integers on
screen; arithmetic stays behind the curtain (pillar: every number fits in
a sentence).

ETA in the footer re-derives from trim + live weather. The road's junction
card promised a *base* price; trim and weather are how the player does
better (or deliberately worse) than the promise.

## Nights and the Watch

### Night types

Every leg-night has a type, forecast **3 nights ahead** on the chart panel:

| Type | Frequency | Stood (any soul) | Unstood |
|------|-----------|------------------|---------|
| **Quiet night** | ~40% | a log line; nothing asked | — (quiet nights need no watch) |
| **Cold night** | ~15% | provisions −2 instead of −5 | provisions −5 |
| **Hungry night** | ~15% | provisions −3 instead of −6 | provisions −6 |
| **Singing night** | ~15% | a log line of the song | a log line of something missed |
| **Strange night** | ~15%, **max 1 per leg** | lore log line; nothing taken | hope −2, and "the night keeps whatever it was going to say" |

A **Watch-affine** soul (spec 3) standing any typed night resolves it one
grade kinder: cold/hungry costs drop to −1/−2, singing nights yield the
rumor or arc beat, strange nights pay out their lore/unique rumor.

Night suitability is not a separate table: nights read the soul's single
**affinity** axis (spec 3 — the same property that strengthens their
matching station). A Watch-affine soul (Runa, Maren) stands any typed
night one grade kinder; every other soul stands it at the base outcome.
No hidden stats, one axis per soul. The Watch-stationed soul covers typed
nights by default; the forecast panel edits per night.

### Scheduling rules

- One soul per night, assigned from the forecast panel; assignments persist
  and can be edited until the night begins.
- **A soul on watch is not resting, and arcs only advance on rest days.**
  This is the travel layer's standing trade-off: coverage vs story. A soul's
  arc panel shows rest-days-to-next-beat so the price is always visible.
- Unassigned typed nights resolve "unstood" (table above) — allowed, priced,
  never catastrophic. **No night ever injures or removes a soul.**
- Quiet nights need no assignment and don't interrupt anyone's rest.

### The log

Every night writes **one entry**: template pools keyed by
`(soul × night type × outcome)`, with weather and road salt mixed in.
Entries are first-person, 2–3 lines, in the soul's voice. Returning after
time away opens on the unread log — **idle time produces prose, not
tallies** — with any mechanical deltas appended in small print under each
entry.

The full log persists for the whole crossing and becomes part of the
arrival keepsake (alongside the chart). Content budget: ~12 templates per
soul-voice × 5 night types with outcome variants ≈ **~120 short entries**
per authored soul set — the largest single writing cost of this spec, and
the most-read text in Act 2.

## Other Pilgrims

**Five authored ships**, not a simulation. Each has a name, a silhouette,
a one-line character, and a **route script** (which roads, which real-day
windows, where her road coincides with plausible player routes).

- **On the chart** they render as moving lights with names, visible inside
  the same one-junction horizon.
- **Hail** (`[H]`, once per meeting): a short exchange — they trade rumors
  about roads *they* have sailed (pilgrim rumors are the only way to hear
  about roads behind you or parallel to you).
- **Matched course**: while your roads coincide and trims are compatible
  (not Run), watches are shared — one typed night per coincident stretch is
  covered by *their* crew, and the log entry arrives in a stranger's voice.
- **Scripted fates**: one ship goes dark in Chapter II (foreshadowing the
  Going-Dark); at least one survives to the Tree and stands in the harbor
  at arrival — a face for Act 3. Fates are authored, not simulated; the
  player's choices don't save or doom other ships (their story is weather,
  not consequence — this keeps the authoring bounded).

## Determinism, Offline, and the Covenant

- **Seeded**: weather draws/drift, night-type sequences, and template
  selection derive from `(voyage_seed, day_index)` — offline resolution
  and live play produce identical worlds; save-scumming changes nothing.
- **Offline**: trim holds; scheduled watches stand as assigned; unassigned
  typed nights resolve unstood; weather moves on its wall-clock steps;
  pilgrims follow their scripts; the log accumulates. Arrivals still hold
  station for the player (unchanged).
- **The covenant extends to travel**: nothing while the player is away —
  weather, nights, silence-banks — ever injures, removes, or advances-to-
  loss a soul. The worst offline outcome is priced provisions/hope and a
  night that kept its secret.
- **Anti-anxiety rule**: unread log entries never expire, and no travel
  choice has a deadline shorter than "before that night begins" /
  "while the weather lasts" — both visible on the forecast.

## UI Surfaces

| Surface | Content |
|---------|---------|
| Chart (main screen) | weather objects on edges, pilgrim lights, wake lanterns, trim + tonight's watch in the Vessel panel, weather summary block |
| Return view | unread log entries (auto-opens after >12h away), then to chart |
| Trim panel (`[T]`) | four postures with final computed prices against live weather |
| Watch panel (`[W]`) | 3-night forecast, affinity notes per soul, rest/arc status |
| Hail (`[H]`) | pilgrim exchange when one is in range |

## What This Spec Does NOT Add

No new gauges. No combat. No mini-games at watch change. No free 2D sailing
(the route graph stands). No procedural pilgrim AI. No weather that blocks
a road outright (weather prices roads; junctions choose them).

## Testing

- Weather lifecycle: spawn/drift/dissipate deterministic from seed; ≤2 on
  leg, ≤4 on chart invariants hold
- Trim composition: final price tables for all trim × weather pairs; ETA
  re-derivation matches footer
- Night resolution: each type × (suited / unsuited / unstood) outcome
  matrix; strange-night cap of 1 per leg; no-harm covenant (property test:
  no offline sequence reduces the soul roster)
- Offline equivalence: N days simulated offline == same N days ticked live
  with identical seeds (the determinism property, the spec's load-bearing
  test)
- Log: every (soul × night × outcome) key resolves to a template; unread
  queue survives save/load
- Pilgrim scripts: coincidence windows compute correctly against arbitrary
  player routes; hail-once-per-meeting enforced

## Open Questions

- Whether matched-course sharing should also ease provisions slightly
  (convoy economics) or stay watch-only (current lean: watch-only — one
  effect per system).
- Whether weather rumors (forecasts) occupy the same rumor inventory as
  road rumors or a separate short-lived slot (lean: same inventory, aging
  entries marked).
- Log volume tuning: one entry per night may be too chatty at 4-day legs ×
  5 months; possible digest mode ("three quiet nights passed") for
  quiet-night runs.
- Whether Mourn is selectable with nothing to mourn (mockup says yes — "the
  option is always here" — it reads as intent; confirm it can't be abused
  as a hope pump given ×1.40 time is the natural brake).
