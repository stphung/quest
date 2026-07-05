# Dossier: World & Narrative (the cross-act through-line)

Last refreshed: 2026-07-05 @ (seed)

> **Status: seed.** This dossier holds the *cross-act* story and world design —
> the through-line that belongs to no single change and so has no home in
> `openspec/specs/` or any one archived change. Grow it as Acts 3/4 take shape.
> Per-act mechanics live in their capability specs; per-feature design intent
> lives in `openspec/changes/archive/`. Act 2's experiential design has its own
> deep dossier in [`act2-pilgrimage.md`](act2-pilgrimage.md). Act 3 now has a
> **concept direction** — pre-commitment exploration only, not designed or
> built — worked out interactively in
> [`../explorations/2026-07-05-act-3-4-story-arc.md`](../explorations/2026-07-05-act-3-4-story-arc.md)
> and summarized below; Act 4 is still fully open.

## Cosmology

Quest sits in a Norse frame. The world-tree **Yggdrasil** is literal: its
branches are worlds, and the game is a journey up, through, and between them.
The three god-items are Norse artifacts (Asprika, the Æsir's armor; Sleipnir,
the eight-legged; Megingjörð, the belt of giant strength). Endgame currencies
and systems keep the register — the Storm, the Deep, the Loom (of Worlds),
Ascension, the Vessel.

## The arc across acts

| Act | Shape | Verb | Status |
|-----|-------|------|--------|
| **Act 1 — The Ascent** | 50 zones, bosses, the Stormbreaker gate at Z10, the endless **Expanse**, then Fracture (12–30) and Loom (31–50) bands | *Climb* | Shipped — deep dossier: [`act1-ascent.md`](act1-ascent.md) |
| **Act 2 — The Crossing** | Clear Z50 → a signal from a dying branch of Yggdrasil; burn 250,000 PR to launch **the Vessel**; the **Voyage** carries souls across the dark toward the Tree; arrival and the colony ferry loop | *Cross / ferry* | Shipped dark (kill-switch) |
| **Act 3 — The Wellspring** *(concept)* | `last_crossing_complete` fires → a cold open pays off the going-dark silence and the Sister Verity's wait → **the Grove** (a new colony-growth system) and **Wyrd** (a light fate currency) unlock → a narrative-beat ramp opens **Root Zones**, a finite band fought against a purpose-built decay/rot bestiary | *Root / graft* | Concept only — not designed or built; see [exploration](../explorations/2026-07-05-act-3-4-story-arc.md) |
| **Act 4 — ?** | TBD — Ragnarök surfaced as a strong candidate (ties the colony to the Líf/Lífthrasir survival myth Act 2's ending already re-tells without naming it) but was deliberately left undecided | ? | Not designed |

The felt motion of the game inverts partway: Act 1 is a **climb** (up 50 zones);
the Deep is a **descent** (down numbered Layers to the Gateway); Act 2 is a
**crossing** (outward, across the dark); Act 3's concept direction is inward —
descending *into* Yggdrasil itself rather than moving across it. Each act so
far re-frames the same hero against a larger scale.

## Recurring motifs (the palette to keep pulling from)

- **Rebirth cycles.** Prestige is death-and-return: a run resets, Prestige Rank
  persists. Ascension is a further shedding. The story treats each cycle as a
  deliberate sacrifice for permanence.
- **Souls and the ferryman.** Act 2 makes the hero a ferryman of souls across
  the dark; the "reckoning" and "going dark" beats are its emotional core.
- **Going dark.** Letters home thin out and stop as the Vessel gets further from
  the shore — distance as loss of contact. A strong, reusable device.
- **The Storm.** Stormglass, Storm Sigils, the Storm Leviathan, Stormbreaker —
  weather as threshold and reward throughout Act 1.
- **The endless.** The Expanse cycles forever; the Deep's first boss is "The
  Endless." Infinity as both progression engine and dread.

## Design guardrails (from Act 1's benchmarks)

- Idle-first: story is delivered through automatic play and overlays, never
  gated behind required input (the forfeit/whisper/scene patterns respect this).
- Numbers are narrative: gates (250k PR, 28 patterns, Ascension X) are pitched
  as momentous *because* they cost real progression, not flavor text.
- Ship dark when unsure: Act 2 lives behind `ACT2_ENABLED` + `QUEST_ACT2`. New
  acts can incubate the same way.

## Open narrative questions

Act 3's questions now carry a working answer from the exploration pass
(concept only — not designed or built, and worth revisiting before anything
ships); Act 4's stay fully open.

- ~~What is at the Tree — arrival as destination, or as a new threshold?~~
  **Concept**: a threshold — the roots, and a living branch (the Grove) for
  the colony to graft onto.
- ~~Does the colony (Act 2's ferry loop) become Act 3's home base / faction?~~
  **Concept**: yes — the Grove is that base, a purpose-built new system
  rather than a Haven branch.
- ~~Does the "going dark" device pay off (contact re-established? never?)?~~
  **Concept**: yes — Act 3's cold open, before any new system unlocks.
- ~~What is Act 3's *verb* — the climb/descent/crossing slot is open.~~
  **Concept**: Root / graft — the unclaimed vector was *in*, not another
  spatial direction across the map.
- How do the Norse eschatology beats (Ragnarök) map onto a final act, if at
  all? **Still open** — a strong Act 4 candidate, deliberately left
  undecided pending more shape on Act 3.

## Sources

- Act 1 experiential design: [`act1-ascent.md`](act1-ascent.md)
- Act 2 experiential design: [`act2-pilgrimage.md`](act2-pilgrimage.md)
- Act 2 mechanics: `openspec/specs/vessel-act2/spec.md`
- Per-feature design intent: `openspec/changes/archive/` (esp. `the-vessel-act2/`)
- Act 1 systems: the capability specs under `openspec/specs/`
- Act 3/4 concept exploration: [`../explorations/2026-07-05-act-3-4-story-arc.md`](../explorations/2026-07-05-act-3-4-story-arc.md)
