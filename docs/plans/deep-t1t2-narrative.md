# The Deep — Narrative and Atmospheric Text

**Author:** Game Designer (agent)
**Date:** 2026-02-25
**Scope:** Task #12 — Atmospheric quotes, First Orders mission text, generation record headers, Abyss entry flavor

---

## 1. Compact Hub Atmospheric Quotes

These rotate on the second line of the compact hub (S-tier terminals). Max ~35 characters each. One is chosen per session or per refresh cycle. They should feel like ambient observations — overheard, not announced.

```
"Stone remembers every step."
"The tunnels go further than the maps."
"Marks don't spend themselves down here."
"Silence is a warning, not a gift."
"Every layer was someone's frontier."
"The guild holds. The mercs don't always."
"Familiarity is just survived ignorance."
"Infrastructure outlasts everyone."
"Deeper is not the same as better."
"The Void has no bottom on the maps."
```

**Usage notes for implementers:**
- Select one at random when the compact hub renders for the first time per session.
- Rotate on prestige (the next generation gets a fresh quote).
- Maximum 35 characters verified for each quote above.
- Render in `Color::Rgb(60, 80, 110)` — muted blue-grey, subordinate to status information.

---

## 2. First Orders Mission Text

The "First Orders" starter mission is a Recon on Layer 1. It is auto-queued on discovery (Task #4). The starter trio — Gareth (Vanguard), Lyra (Scout), Aldric (Medic) — are already deployed when the player first opens the overlay.

### Mission Description (shown in mission list detail panel, 1-2 lines)

```
Scout the Shallows. Mark what moves.
The captain's maps are a generation old.
```

**Alternate (single-line, for compact mode):**
```
Your scouts are already in the tunnels.
```

### Active Mission Status Text (shown on hub card while mission is running)

This replaces or supplements the standard `[Recon] Layer 1 — The Shallows` label:

```
First Orders — Layer 1
```

Display in `Color::Cyan` to distinguish from normal generated missions.

### Completion Narrative (shown in mission result modal, 2-3 lines)

Displayed above the rewards section when the First Orders mission resolves:

```
Gareth, Lyra, and Aldric return with maps
and notes. The Shallows are charted now.
This is the beginning of something longer.
```

**Notes:**
- Appears only once — the first mission result the player ever collects.
- After dismissal, subsequent missions use standard result formatting.
- No special mechanical effect; purely atmospheric.
- Familiarity gain (+15% Recon) still applies and is shown normally below this text.

---

## 3. Generation Record Display Headers

The Generation Records section in the hub shows a compact history of past prestige cycles.

### Section Header

```
PAST GENERATIONS
```

Render in `Color::Rgb(80, 160, 220)` (same as `SECTION_LABEL_COLOR`).

**Alternate if vertical space is very tight (S-tier compact):**
```
LEGACY
```

### Per-Generation Summary Format

One line per generation, left-aligned, fixed-width columns:

```
Gen.3  L12 reached  847M earned  2 lost
```

**Column layout:**
| Column | Width | Content | Color |
|--------|-------|---------|-------|
| `Gen.N` | 5 chars | Generation number | `Color::DarkGray` |
| `LN reached` | 11 chars | Deepest layer reached | `Color::White` |
| `NM earned` | 10 chars | Total Marks earned | `Color::Yellow` |
| `N lost` | 7 chars | Mercs permanently lost | `Color::Rgb(160, 80, 80)` |

**Zero-loss variant** (no mercs lost):
```
Gen.1  L3 reached   120M earned  —
```
Use `—` (em dash, U+2014) in the lost column when count is 0. Color the `—` in `Color::DarkGray`.

**Example block with three generations:**
```
PAST GENERATIONS
Gen.3  L12 reached  847M earned  2 lost
Gen.2  L7 reached   392M earned  1 lost
Gen.1  L3 reached   120M earned  —
```

Show most recent generation first. If more than 5 generations exist, show the 5 most recent and omit older ones.

---

## 4. Abyss Entry Bonus Flavor Text

Shown as a one-line message in the Layer 19 detail panel (or as a flash message) when the L18 Breakthrough completes, explaining the automatic +25 familiarity bonus on L19:

```
Patterns from the Sunken Reach echo here.
The Abyss is not entirely unknown.
```

**Single-line variant (for flash messages or compact displays):**
```
Sunken Reach experience carries over.
```

**Implementation note:**
- Triggered in the L18 Breakthrough mission result when the +25 familiarity bonus on L19 is applied (Task #2 / T2-7).
- Render the two-line version in the result modal below the standard reward block.
- Render in `Color::Rgb(60, 90, 160)` — cooler blue than standard text, suggesting depth.
- The bonus line in the rewards section remains: `+ Familiarity on Layer 19: +25% (now 25%, Mapped!)`.

---

## 5. Additional Atmospheric Text (Bonus)

These are extras that implementers can use where space and context allow.

### Discovery Modal Flavor (line 2 of revised discovery modal)

The three mechanically explanatory lines in the discovery modal (from the onboarding doc) benefit from a tonal opening. The existing line `"The Deep goes further than you know."` works well. No changes needed here — the existing quote lands.

### Injury Notification Flavor (in mission result modal, when mercs are injured)

When a mission result includes injured mercs, prefix the injury list with one of these:

- `"Not everyone came back the same."`  — general
- `"The tunnels extract a price."`       — for moderate/severe injuries
- `"Walk it off. Or don't."`            — for light injuries only

Select based on worst injury severity in the result.

### Merc Lost Notification Flavor (when a merc is permanently lost)

Shown above the `Lost:` line in the mission result modal:

```
Some don't come back. Remember the name.
```

Single line. Render in `Color::Rgb(160, 80, 80)` — same muted red as the lost-merc count in generation records.

### Breakthrough Cleared Celebration (Layer N)

The onboarding doc calls for `★ LAYER N CLEARED — Layer N+1 Unlocked! ★` in gold. The flavor underneath (if vertical space allows, 1 line):

- Layers 1-3 (Shallows cleared):   `"The entrance is yours. The depth is not."`
- Layers 4-7 (Warrens cleared):    `"The corridors open into something older."`
- Layers 8-12 (Hollows cleared):   `"Vast. The word doesn't cover it."`
- Layers 13-18 (Sunken Reach):     `"The water recedes. Something remains."`
- Layers 19-25 (Abyss cleared):    `"You went where the maps end. Keep going."`
- Layer 26+ (Void):                `"The guild's name means nothing here."`

These are optional flavor lines rendered in `Color::DarkGray` beneath the gold breakthrough banner. Skip if vertical space is insufficient.
