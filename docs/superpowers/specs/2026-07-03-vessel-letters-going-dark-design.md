# Letters From Home & the Going-Dark

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 6 of 7
**Depends on:** specs 2–5 (shipped — arrivals, the Log, the moments queue,
the two-gauge economy). **Feeds:** spec 7 (the manifest keeps the letters;
the Going-Dark shapes what the arrival means).

## Overview

Act 1's world does not vanish at launch — it writes. **Letters from home**
arrive at every port in the first half of the crossing: the Haven's new
warden, the Deep's guild, the fisher-fleets, the Loom-tenders you left at
the wheel. Each letter is a voice, a small parcel, and proof that the
world you spent a quarter-million lives building is still holding the
lamp for you.

Then, at the Threshold, the postmaster hands you the last one. And
somewhere past the Last Lantern comes the crossing's quietest, heaviest
beat: **the night the mail does not come.** The world behind you has gone
out. From there to the Tree, the only lights are the ones you carry.

This spec also formally settles Act 1's runtime fate (deferred twice):
after launch, Act 1 is **frozen fiction** — its state never ticks again,
and the letters are its only interface. The Going-Dark closes even that.

## Letters From Home

### Delivery model — no timers, no anxiety

Letters are delivered **at arrivals**, not on clocks: the mail catches up
to you at ports (the fiction), and nothing can be missed by being away
(the covenant). One letter waits at each waypoint arrival while the mail
still flows:

- **Flows**: Chapters I and II, and the Threshold (W23) — the final letter.
- **Never flows**: anywhere in Chapter III past the Threshold, or Chapter
  IV. The Going-Dark is one-way.

Delivery mechanics: after the arrival scene plays, the letter surfaces as
a **moment** (the existing one-line modal queue), and is kept forever —
the Log grows a "Letters kept" section, and spec 7's manifest binds them.

### The letters themselves

An authored, **sequenced** table (letter N is the Nth received — the
sequence tells a story of home slowly changing), ~12 letters plus the
final one. Senders rotate through the world the player actually built:

| Sender | Voice | Example beat |
|--------|-------|--------------|
| The Haven's new warden | formal, trying too hard | the rooms you built, kept warm; your bunk left made |
| The Deep guild | terse, professional | Layer 30 holds; the mercenaries drink to Torvald |
| The fisher-fleets | weather-worn, fond | Runa's old skiff found a new pair of hands |
| The Loom-tenders | reverent, precise | the Loom still hums your patterns; they send its "surplus" |
| Children of the harbor | crayon-honest | drawings of the Vessel, dictated postscripts |

Each letter carries:

- **Text** (2–4 sentences, authored; the sequence darkens subtly — later
  letters mention dimming zones, quieter harbors — foreshadowing without
  announcing)
- **A parcel**: +5 provisions ("the Loom's surplus", "the fleet's salt
  catch") — small on purpose; letters are a hope economy, not a second
  pantry
- **Hope +1 on roughly every third letter** (authored per letter, not
  rolled)

Soul color: letters may carry one postscript line keyed to a soul aboard
(`ColorKey`-style, reusing spec 4's machinery): the guild writes to
Torvald; a child asks after Runa's skiff.

### Act 1, settled

- At launch, Act 1's state is frozen exactly as the burn left it. The
  game tick never runs for that character again (already true in code;
  now it is design law, not a deferral).
- The letters are the *fictional interface* to the frozen world: the
  Loom's "surplus" in the parcels is narrative, not a computed WR figure.
- The stats panel's Act 1 numbers, if ever shown again, are a museum
  (spec 7 / Act 3 decide whether to show them).

## The Going-Dark

Three authored beats, all **location-triggered** (never time-triggered —
nothing this heavy happens while the player is away):

1. **The Threshold (W23)** — already authored in spec 4 ("the last place a
   letter can reach you"). Its letter is **the Last Letter**: longer,
   co-signed by every sender, and it knows what it is. The scene's
   existing Mourn color line stands.
2. **The Last Lantern (W24)** — the keeper sells stores and stories, and
   mentions, carefully, that the mail packet is late. First time that has
   happened. (One line added to the existing scene; dread, not event.)
3. **The night the mail does not come** — the first arrival *after* W24
   (whichever road was chosen): the crew gathers on deck out of habit at
   mail-hour, and nothing comes. The world behind has gone out. Authored
   moment, **hope −2** (priced, once, and the Long Silence is reachable
   from it if hope was already low — the design accepts this; Chapter III
   is the dark chapter), and the Log's entry is the act's shortest:
   *"No letters. There will be no more letters."*

After the third beat: `gone_dark = true`. No letters, ever again. The
chart's southern edge (home) dims a shade in the palette — the one
cosmetic touch.

### Why this works on the two gauges

The letters' +5 parcels quietly supplement Chapters I–II (the safe water).
Their removal lands exactly as Chapter III's roads get expensive — the
chapter *feels* darker because the economy genuinely tightened, without a
single number on a card changing. Hope loses its steady letter drip at
the same time, making Mourn trim and rest-day arcs the chapter's hope
engines — the people become the light, which is the act's whole argument.

## Data Model (build scope)

```rust
// src/vessel/letters.rs — authored table
pub struct LetterDef {
    sender: &'static str,
    text: &'static str,
    hope: u8,                       // 0 or 1; the Last Letter pays 2
    postscript: Option<(SoulId, &'static str)>,
}
pub const LETTERS: [LetterDef; 12];
pub const LAST_LETTER: LetterDef;   // delivered at the Threshold
pub const LETTER_PARCEL_PROVISIONS: f64 = 5.0;
pub const MAIL_FAILS_HOPE_COST: u8 = 2;

// VoyageState (serde defaults; old saves continue cleanly)
letters_received: u8,               // index into the sequence
gone_dark: bool,                    // set by the third beat
```

Engine: `arrive_at` delivers (queue a `SoulEvent`-style letter event for
the UI; parcel + hope apply at delivery, exactly once); the third beat
triggers on the first arrival whose predecessor-in-visited is W24-or-later
in Chapter III. All lazily evaluated, chunking-invariant, offline == live
bitwise.

## UI

| Surface | Content |
|---------|---------|
| Moments queue | each letter as a titled moment ("A letter from the Haven"), read like arc beats |
| The Log | new "Letters kept: N" line; letters listed with senders |
| Chart | home edge dims after the Going-Dark |
| Vessel panel | nothing — letters are moments, not a gauge |

## What This Spec Does NOT Add

No timers or missable mail. No reply mechanic. No new gauges. No
resurrection of Act 1 ticking. No Going-Dark choice — it is weather, the
act's largest weather, and it happens to everyone.

## Testing

- A letter at every Chapter I/II arrival, in sequence; parcels and hope
  apply exactly once; save/load mid-sequence resumes correctly
- The Last Letter delivers at W23 and nowhere else; no letter ever
  delivers past it
- The mail-fails beat fires exactly once, at the first arrival after W24,
  with its hope price; `gone_dark` is permanent
- Economy gates re-run with parcels in (Cruise ≤1 drift, Mourn 0 — the
  parcels may buy back a notch of Chapter I/II tuning if needed)
- Offline equivalence bitwise with letters in play; the covenant test
  extended: crossing into Chapter III offline queues the beats, harms
  nothing beyond their stated prices

## Open Questions

- Letter count: 12 + Last (lean) vs scaling with route length (~14 max
  arrivals before W23 on the longest route; extra arrivals past 12 simply
  get no letter — "the mail is thinning" — which foreshadows for free).
- Whether the mail-fails beat's hope −2 should be softened if hope is
  already ≤2 (lean: no — the floor mechanics already catch it, and the
  Long Silence in Chapter III is thematically correct).
- Whether Act 3 reopens anything of home (park for the Act 3 elevation;
  nothing in this spec forecloses it).
