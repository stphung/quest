# The Price of Passage

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 8 — the act's first balance revision, after the queue.
**Depends on:** specs 2–7 (all shipped). **Instrument:** the voyage
simulator and the balance probe methodology (160 instrumented crossings,
2026-07-03) that produced the diagnosis below.

## Diagnosis

The act has decision surfaces but no shortage. Across 160 simulated
crossings (4 strategies × 4 check-in cadences × staffed/unstaffed × 5
seeds): the Long Silence fired **zero** times, every staffed strategy
ended at hope 10, a staffed cheap-route ship never drifted, and every
manifest read alike — full berths, sound crew, nothing carved. Junctions,
berths, refits, trim, and watches all exist; none of them charge a price
in a currency the player is short of.

The fix is not dice and not danger of failure. It is **scarcity plus
ledgers**: make the currencies short, give hope a spend, let neglect
accumulate on people and hull as visible, caused, waiting consequences —
and let the manifest say what the crossing cost.

## The Law (unchanged, sharpened)

The stake is never *whether* you arrive. It is **who arrives, and in
what state**. The crossing stays unlosable; the manifest becomes volatile.
Everything below is a pure function of prior choices with the cause shown
— no rolls, ever — and nothing heavy lands offline: consequences are
acquired at deterministic sim-time events (like nights) and *surface* at
the next check-in as moments, exactly like arc beats today.

## 1. Scarcity (the enabler)

Way-station and rest-stop payouts come down until the cheap route
genuinely squeezes:

- Way-stations: 25–28 → **20–23** provisions
- Rest stops: 18–25 → **15–20**
- Letters' parcels, threat tolls, drift recovery: unchanged (the
  covenant's floors are not the problem)

**Tuning target** (sim-gated): a staffed, attentive cheapest crossing
floors below 10 provisions at least once and still completes with 0
drifts — tense, not punished. An unstaffed or inattentive one drifts ~1–2
times, as today.

> **Doc-alignment note (2026-07-04):** this section's fix — giving Hope
> spends (Press the helm, Hard rations) so the gauge would finally engage —
> **did not hold**: balance-sim evidence later showed Hope still pinned at
> its maximum under every attentive strategy even with these sinks in
> place, and commit d39ad67 retired Hope entirely rather than add a third
> sink. Press the helm (`[P]`), Hard rations, and every `hope`-gated
> mechanic in this section are gone — see
> `docs/superpowers/specs/2026-07-03-vessel-ferryman-design.md`'s Ward
> follow-up and `docs/decisions.md` ("Act 2 Ferryman Era: Retiring Hope")
> for what replaced this diagnosis's fix. Sections 3–4 below (strain, hull
> wear/scars) are unaffected and shipped as designed
> (`HULL_WEAR_MAX`/`WEAR_BURN_PER_SCAR` in `voyage.rs`).

## 2. Hope becomes a wallet

Hope's sources stand; it gains **sinks** — real purchases at check-in
moments, both showing final prices (composition law unchanged):

1. **Press the helm** `[P]` — while Traveling, once per leg: −2 hope,
   the leg's *remaining* time ×0.85. The crew digs deep; days are what
   it buys. Pressing twice within one chapter strains the helm soul
   (see §3) — the sink feeds the ledger.
2. **Hard rations** — a standing toggle beside trim: provisions burn
   ×0.75 while on; hope −1 per full day on. Oregon Trail's dial, priced
   in the act's own currencies. Turning it off is free; the hunger isn't.

Guard: sinks require hope ≥ 3. You can be *worn down* into the Long
Silence; you cannot *buy* your way in. (The wind bonus at 8+ now has an
opportunity cost — holding bright hope means declining to spend it.)

## 3. The strain ledger (sickness without dice)

Per met soul: `strain: 0 | 1 | 2` — sound, **strained**, **worn**.
Every acquisition is announced with its cause, at the next check-in.

**Causes** (deterministic, sim-time):
- Standing **3 consecutive nights** on watch → the stander strains
- Crossing a **squall at Run** while on any post → the posted soul strains
- **Helm through a silence bank** un-Quiet → the helm soul strains
- **Pressing the helm twice in one chapter** → the helm soul strains

**Effects:**
- Strained (1): affinity stops counting — helm/tender multipliers revert
  to the unaffine values, an affine watcher counts as merely Stood; rest
  accrues at half pace (a hurt person heals before they story-tell)
- Worn (2): cannot hold a post; arc paused entirely

**Recovery:** RestStop arrivals heal every **off-post** soul by one
level (posts are not rest — relieving someone is the decision). Surfaced
as a moment: "Runa mends at the Mirrorcalm."

**The teeth:** threat-road ledgers take the **most strained stationed
soul first** (ties break helm→tender→watch as today). The Thorns row
changes from "exposure is the post" to "exposure is the post, weakest
first." A loss now traces through three visible decisions — who stood
too many watches, who wasn't rested, which road you then chose.

**Manifest:** arrival state shows it forever — "came ashore, worn."

## 4. Hull wear (the wagon axle)

`hull_wear: 0..=6` — **scars**, counted in words on the vessel panel
("sound", "scarred ×3"), never a bar.

**Sources** (+1, announced with cause): a drift; a squall crossed at
Run; the threat rows where the ship takes the road on her own skin
(the Thorns quiet-keel and all-below rows, the Warden's hurried row);
pressing the helm at wear ≥ 4.

**Effect:** provisions burn ×(1 + 0.05 × wear). A six-scarred hull eats
30% more — wear compounds into the scarcity axis, not into speed. She
always sails; she just gets hungry.

**Repair — the third door:** shipyards offer **A / B / mend her**.
Mending zeroes wear and *closes that yard's refit pair forever* (the
doors-close pillar, now load-bearing: a hard-driven ship may finish with
one refit where a gentle one carries three). This is the only repair;
wear otherwise rides to the Tree and into the manifest ("she arrived
carrying four scars").

## Data Model (build scope)

```rust
// SoulState — serde(default)
strain: u8,                       // 0 sound, 1 strained, 2 worn
consecutive_watches: u8,          // resets on a night not stood

// VoyageState — serde(default)
hull_wear: u8,                    // 0..=6
hard_rations: bool,
pressed_this_leg: bool,
presses_this_chapter: u8,         // resets at chapter boundary
strain_events: Vec<StrainEvent>,  // surfaced like soul_events

// Composition law grows one term each, still shown as final integers:
// time = base × trim × wind × helm × press (× StormSail)
// provisions = base × trim × tender × rations × wear (× MourningColors)
```

Threat ledgers read strain; `choose_refit` gains the mend arm; RestStop
arrival heals; nights/squalls/banks acquire strain in `step_minute` /
`resolve_night` (chunking-invariant, offline == live bitwise, as ever).

## UI

| Surface | Change |
|---------|--------|
| Vessel panel | hull line ("scarred ×2 — the hold pays 10% more"); rations state; `[P] Press the helm` when available |
| Trim panel | hard-rations toggle row, priced like trims (final numbers) |
| Souls panel | strain shown per soul with its cause; worn souls can't cycle onto posts |
| Watch panel | "third night in a row" warning before it costs |
| Shipyard modal | three doors: A / B / mend |
| Moments | strain acquired/healed, wear taken — one line, cause named |
| Manifest / finale | strain state and scars recorded; carved-names beat unchanged |

## What This Spec Does NOT Add

No dice. No offline decay — everything acquires at deterministic
sim-time events and waits at the next check-in. No losable crossing —
drift recovery, the affordability invariant, and the never-locked
cheapest road all stand. No HP bars, no percentages — strain and wear
are small integers with words. No new berths math, no new route content.

## Testing / Sim Gates

- All strategies still arrive; 20–200 day envelope holds
- Cheapest staffed + attentive: 0 drifts, min provisions < 10 (the
  squeeze exists), 0 strains with good watch rotation
- Neglect profile (unstaffed, 48h check-ins): ≥1 strain and ≥2 wear at
  arrival — the manifest finally varies
- Hard-rations-abuse profile: hope min ≤ 2 (the gauge finally bites);
  Long Silence reachable but only through sustained choice
- Run-everywhere profile: ≥2 wear; mend-vs-refit decision reached
- Strain determinism: same seed + same assignments → identical ledger;
  offline == live bitwise through save/load
- Threat exposure: staged crossings prove most-strained-first ordering
- Save compat: all new fields serde(default); pre-spec-8 voyage.json
  loads sound, unworn, full-rationed

## Open Questions

- Whether Mourn trim should also heal strain at sea (lean: no — RestStops
  keep their monopoly on mending people; Mourn already owns hope)
- Whether wear should show on the chart's ship glyph at high scars
  (lean: yes at 4+, a color shift, no new glyph)
- Whether the Outfitting (menu item 6) and ford/ferry/caulk junctions
  (item 5) become spec 9 after this lands (lean: evaluate with fresh
  probe data — scarcity may make existing junctions bite enough)
