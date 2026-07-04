# Dossier: World & Narrative (the cross-act through-line)

Last refreshed: 2026-07-04 @ (seed)

> **Status: seed.** This dossier holds the *cross-act* story and world design —
> the through-line that belongs to no single change and so has no home in
> `openspec/specs/` or any one archived change. Grow it as Acts 3/4 take shape.
> Per-act mechanics live in their capability specs; per-feature design intent
> lives in `openspec/changes/archive/`. Act 2's experiential design has its own
> deep dossier in [`act2-pilgrimage.md`](act2-pilgrimage.md).

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
| **Act 1 — The Ascent** | 50 zones, bosses, the Stormbreaker gate at Z10, the endless **Expanse**, then Fracture (12–30) and Loom (31–50) bands | *Climb* | Shipped |
| **Act 2 — The Crossing** | Clear Z50 → a signal from a dying branch of Yggdrasil; burn 250,000 PR to launch **the Vessel**; the **Voyage** carries souls across the dark toward the Tree; arrival and the colony ferry loop | *Cross / ferry* | Shipped dark (kill-switch) |
| **Act 3 — ?** | TBD | ? | Not designed |
| **Act 4 — ?** | TBD | ? | Not designed |

The felt motion of the game inverts partway: Act 1 is a **climb** (up 50 zones);
the Deep is a **descent** (down numbered Layers to the Gateway); Act 2 is a
**crossing** (outward, across the dark). Each act so far re-frames the same hero
against a larger scale.

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

## Open narrative questions (feed these into Act 3/4 exploration)

- What is at the Tree — arrival as destination, or as a new threshold?
- Does the colony (Act 2's ferry loop) become Act 3's home base / faction?
- Does the "going dark" device pay off (contact re-established? never?)?
- How do the Norse eschatology beats (Ragnarök) map onto a final act, if at all?
- What is Act 3's *verb* — the climb/descent/crossing slot is open.

## Sources

- Act 2 experiential design: [`act2-pilgrimage.md`](act2-pilgrimage.md)
- Act 2 mechanics: `openspec/specs/vessel-act2/spec.md`
- Per-feature design intent: `openspec/changes/archive/` (esp. `the-vessel-act2/`)
- Act 1 systems: the capability specs under `openspec/specs/`
