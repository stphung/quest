# Pace & Rations — the naming pass

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 10 — a pure legibility pass on spec 5's Trim dial and
spec 8's rations toggle. No mechanics change.
**Depends on:** specs 5 and 8 (shipped/in-flight).

## The problem

The posture dial's four values — **Run · Cruise · Quiet · Mourn** — read
as *muddy*: they mix two metaphor families (two speeds, two moods), so
the eye can't sort them on one axis, and you can't tell which trades what.
Provisions, Hope, and Hull are legible because each names a single
concrete thing; the postures weren't.

## The fix

Name the dial and its values in **Oregon Trail's register** — by the toll
each takes on the party, not by an abstract speed. OT's own dials were
*Pace* (Steady/Strenuous/Grueling) and *Rations* (Filling/Meager/Bare
Bones); every word is a cost you can feel. We borrow the voice.

### Pace (was "Trim")

The panel label becomes **Pace**, and the four values sort top-to-bottom
as hardest→gentlest — which, being a pace, is also fastest→slowest:

| Was | Now | The toll (shown in the panel line) |
|-----|-----|------------------------------------|
| Run | **Grueling** | fastest — the hold empties and she scars |
| Cruise | **Steady** | the honest middle; never wrong, never best |
| Quiet | **Easy** | slower, sparing — quiet enough to hear the dark |
| Mourn | **Restful** | slowest — the crew mends and hope climbs |

The anti-muddy principle: **the name carries only the pace; the
one-line description under it carries the side-effect** (thrift,
listening, hope-healing). A name means one thing.

### Rations (was "Hard rations" toggle)

The two-state toggle from spec 8 is renamed in the same voice — still a
toggle, no third tier:

| State | Now | Effect (unchanged) |
|-------|-----|--------------------|
| off | **Filling** | the crew eats their fill |
| on | **Bare Bones** | burn ×0.75, hope −1/day |

Together, Act 2's supply layer now reads as a deliberate Oregon Trail
homage — a **Pace** dial and a **Rations** dial, both named by feel.

## Scope (what does NOT change)

- **No mechanics.** Every multiplier, gate, and effect is identical;
  this is strings only. The `Trim` enum, its variants (`Run`/`Cruise`/
  `Quiet`/`Mourn`), `hard_rations: bool`, and the `[T]` hotkey and
  `VoyageView::Trim` view all keep their **internal** names — engine
  terms, never shown. Only `display_name()`, the panel title/label, the
  description lines, and the rations row change.
- **No save impact** (no serialized field touched).
- A third rations tier (OT's *Meager* middle) is deliberately **not**
  added here — that's a mechanics change that would want sim re-tuning
  the Ferryman work (spec 9) will re-touch anyway. Parked.

## Testing

- UI snapshots re-blessed (Pace panel, vessel gauge line, strip); no
  test asserts the old display strings, so the rename is string-only
- `make check` green
