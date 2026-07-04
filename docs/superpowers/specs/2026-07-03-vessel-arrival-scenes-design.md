# Arrival Scenes — Delivery, Payoffs, Refits, and the Threats

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 4 of 7
**Depends on:** spec 2 (shipped — waypoints carry `SceneRef` slots, drift
carries a recovery-scene flag), spec 3 (shipped — boarding asks, farewells,
`mark_lost`, counsel). **Feeds:** spec 6 (the Going-Dark is a scene), spec 7
(keepsakes and resolutions read into the manifest).

## Overview

Arrivals are **payoff scenes, never tests**. The player already made the
choice — at the junction, with the price on the card. The arrival is where
that choice pays: in provisions, in people, in one more piece of the world
seen before it ends. This spec defines the scene system that delivers them:
the beat format, the recoloring model, the real waypoint economy (replacing
the interim +20 gift), refits at shipyards, and the three named threats —
the only places a soul can be lost, and never by dice.

The one structural rule, inherited and absolute: **scenes contain no option
menus.** The only interactive moments inside scenes are *doors* the design
already sanctions — a soul's ask (board / decline), a farewell, and a refit
(a permanent A/B). Nothing in a scene is ever a gamble, a skill check, or a
timed prompt.

## The Scene Format

A scene is an authored sequence of **1–4 beats** — short paragraphs shown
one at a time in the scene view (`[Enter]` advances; the last beat shows
the payout in small print). Beats are text plus optional **color lines**
selected by state. Structure:

```rust
pub struct SceneDef {
    id: &'static str,               // matches route.rs SceneRef ids
    beats: &'static [SceneBeat],
    payout: ScenePayout,            // applied when the scene completes
    door: Option<SceneDoor>,        // Ask(soul) | Refit(pair) — at most one
}

pub struct SceneBeat {
    text: &'static str,
    colors: &'static [ColorLine],   // 0..n; matching lines append, in order
}

pub struct ColorLine {
    when: ColorKey,                 // deterministic, state-derived
    text: &'static str,
}

pub enum ColorKey {
    SoulAboard(SoulId),             // "Sefa steps forward and sings."
    ArrivedBy(RoadId),              // the road colors the landfall
    TrimIs(Trim),                   // arriving at Mourn reads differently
    KnowsRumor(RumorId),            // foreknowledge acknowledged
    ChapterIs(Chapter),
    HopeAtLeast(u8), HopeBelow(u8),
    Drifted,                        // the leg included a drift
}
```

> **Doc-alignment note (2026-07-04):** `HopeAtLeast`/`HopeBelow` never
> shipped — Hope was retired entirely (commit d39ad67) before this variant
> would have been used. Current `ColorKey` (`scenes.rs:18-23`) is
> `SoulAboard/ArrivedBy/TrimIs/KnowsRumor/Drifted` — no chapter or hope
> variants. Likewise every `hope: i8` payout field and every `hope ±N`
> beat described below (the payout table, the Ossuary Warden, the Silence
> itself, `mark_lost`) no longer prices anything — those beats are either
> free now or (Silence-bank specifically) replaced by a strain hit on the
> Helm soul, per the underway spec's alignment note. One refit is also now
> a no-op: **Lantern Mast** (row 3 below) has a name and blurb (`refits.rs`)
> but nothing reads it — unlike its siblings, it isn't wired into any
> gameplay effect. And **Mourning Colors**' stated "×0.80 provisions" was
> written against Mourn's old 0.90 base; Mourn's shipped base is already
> 0.80 post-Hope-retirement (`voyage.rs`), with the refit stacking it
> further to 0.70 — the number below no longer matches either value.

Rules that keep this writable and honest:

- **Determinism**: color lines are pure state functions — same save, same
  scene, always. No RNG anywhere in scenes.
- **Budget**: base beats ~40–80 words; color lines one sentence each. A
  waypoint's full scene tops out around a screen of text. 38 waypoints ×
  (2–3 beats + 3–6 color lines) is the writing cost — the route content
  map worksheet (below) tracks it.
- **Scenes play once.** Waypoints are visited once per crossing; the scene
  state machine (spec 2's `Waiting → Played`) already enforces it.
- The existing one-line `placeholder` in `SceneRef` remains the fallback
  for any scene not yet authored — the game never blocks on writing.

## The Real Economy (retiring the +20 gift)

The interim uniform gift is replaced by **authored payouts** in the scene
data. Provisioning becomes part of the fiction: a market has stock, a reef
does not. Amounts by waypoint kind (authoring guideline, tuned by the
simulator's no-drift-at-Cruise target):

| Waypoint kind | Provisions payout | Notes |
|---------------|-------------------|-------|
| Harbor (start/end) | fill to cap | the pier gives what it has |
| WayStation | +25 | markets; rumors still cost 6 |
| RestStop | +20 | and breaks the Long Silence (shipped) |
| Shipyard | +15 | yards feed crews; refit door lives here |
| LanternSite | +12 | keepers' stores, shared |
| plain waypoint | +8..+15 authored | the scene says why |
| threat waypoints | +0..+5 | the reef gives nothing freely |

Other payout fields: `hope: i8` (a few scenes move it ±1, authored
sparingly), `rumor: Option<RumorId>` (scene-granted knowledge), and
`keepsake: Option<KeepsakeId>` — **keepsakes have no mechanics**: named
mementos ("a knot of Whale-Road baleen") that accumulate in the manifest
for spec 7. ~10 authored keepsakes across the route.

Simulator gate update: at Cruise, an attended crossing on any strategy
should complete with **at most one drift**; Quiet/Mourn with none. The
per-waypoint table above is tuned against that assertion.

## Refits — the three A/B doors

At the first **three distinct shipyards** visited, the yard offers one
permanent either/or refit (authored pairs, in a fixed sequence so route
choice decides *which yards*, not *which pairs*):

| # | A | B |
|---|---|---|
| 1 | **Storm Sail** — all legs 10% faster | **Long Hold** — provisions cap 150 |
| 2 | **Quiet Keel** — named-threat roads take their kinder ledger row | **Deep Larder** — drift recovery restores 40 |
| 3 | **Lantern Mast** — unknown waypoints (`◌`) within one junction show their names | **Mourning Colors** — Mourn trim also x0.80 provisions |

Refit rules: choosing A closes B forever (doors close); a refit is
configuration, not a consumable; effects compose into the same integer
prices everything else uses. Four shipyards are authored (Graywater,
Saint Elm's, Drift's End, the Ember Hold) and content parity guarantees
≥2 per route — so every crossing sees at least two refit doors, most
see three.

## The Threats — where loss lives

Three named threats exist (spec 2 authored them onto roads). Each is a
scene at the road's destination with a **ledger, not a roll**: the outcome
is a pure function of state the player controlled, and the junction card
told them what mattered (via tags, counsel, and rumors).

| Threat | Road | The ledger (checked in order) | Worst outcome |
|--------|------|-------------------------------|---------------|
| **The Ossuary Warden** | over the reef (R9) | Sefa aboard (she sings the office) → safe passage, keepsake · arrived at Quiet/Mourn trim → it takes provisions (−15) · else → it takes provisions (−15) **and hope (−2)** | priced, never a soul |
| **The Silence itself** | the silent road (R29) | any soul resting (unstationed) that leg → they anchor the crew, safe · else → hope −2 and the leg's log stays blank | priced, never a soul |
| **The Thorns** | the thorn run (R42) | Cormac aboard **and** at Helm → one clean line, keepsake · Quiet Keel refit → hull holds, provisions −10 · else → **a soul at a station is lost** (helm first, then tender, then watch; resting souls are never taken) | the game's only loss |

Loss design notes:

- The Thorns are the only loss in Act 2 v1 — one road, Chapter IV, the
  fast expensive option at the last junction, with a threat line on the
  card, Cormac's counsel speaking directly to it, and two independent
  outs (the pilot, the refit). A player who loses someone chose a named
  danger over a flowered road and sent a crewed ship through unprepared.
  That is "priced, chosen, and become story."
- Loss takes a *stationed* soul: the post was the exposure. Resting souls
  are below. This makes the stations trade physical, and it means the
  player's standing choices — not a table lookup of who's expendable —
  decide who was in harm's way.
- The scene calls spec 3's `mark_lost` (hope −3, hull carving, counsel
  silenced) and plays a memorial beat. The covenant is untouched: the
  ledger only ever runs when the player chose the road while present.

## Boarding, Farewell, and Recovery Scenes

Already mechanically shipped in spec 3 as modals; spec 4 gives them scene
bodies:

- **Boarding asks** become 2-beat scenes (the meeting, then the ask door).
  Per-soul, authored at each of their sites — Ysolt met at Saint Elm's is
  mending hulls; met at the Beacon Graveyard she is salvaging lenses.
- **Farewells** become 1-beat scenes with a color line per departing soul.
- **Drift recovery** gets four authored scenes, one per chapter (the same
  event reads differently in the Shallows than the Starless Deep).
- **Chapter gateways** (the Shallows Gate, Drift's End, Deepgate) get
  slightly longer scenes (3–4 beats) — the act breaks.

## UI

The one-line `SceneModal` grows into a **scene view**: centered column,
one beat at a time, `[Enter]` next / `[Esc]` skip-to-end, payout line in
small print on the final beat ("the hold gains 20 · a rumor learned").
Doors render as the existing ask/refit prompt after the last beat. The
log keeps every played scene's title (the crossing's story so far, one
line each, readable from the Rumors panel renamed **the Log**).

## Writing Pipeline — the route content map

A worksheet (`docs/superpowers/specs/act2-route-content-map.md`, authored
with the implementation) with one row per waypoint: kind, payout, scene
beats summary, color lines available there, door if any, keepsake if any,
threat ledger if any. It is the single place to see coverage and tone
drift, and the PR artifact reviewers read instead of 38 scene files.

## What This Spec Does NOT Add

No option menus in scenes. No dice. No new gauges or currencies. No
procedural text. No letters (spec 6), no finale (spec 7), no weather or
nights (spec 5).

## Testing

- Scene engine: every `SceneRef` id resolves to a `SceneDef` or falls back
  to the placeholder; color lines select deterministically; payouts apply
  exactly once (save/load mid-scene replays cleanly)
- Economy: simulator asserts ≤1 drift at Cruise, 0 at Quiet/Mourn, all
  strategies still arrive in the envelope
- Refits: three doors per crossing when three shipyards are visited; A
  closes B; effects compose into card integers (snapshot-tested)
- Threat ledgers: full outcome matrix per threat as table tests — every
  row reachable, the Thorns' loss row only with a stationed soul, and
  `mark_lost` never called from any other scene
- Covenant unchanged: the 60-day offline property test still passes (no
  scene runs without the player)

## Open Questions

- Whether the Warden's provisions toll should scale with hold contents
  (lean: flat — predictable prices are the game's whole grammar).
- Whether a second loss ledger should exist in Chapter III for players
  who want higher stakes earlier (lean: not in v1 — one perfect loss
  beats two adequate ones).
- Keepsake count and whether any appear in Act 3 (park for spec 7).
