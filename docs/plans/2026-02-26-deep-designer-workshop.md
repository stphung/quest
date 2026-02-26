# The Deep — Designer Workshop: Post-Review Improvement Proposals

> **Source:** 3 game designers analyzed the post-T1/T2 editorial review (84/100) and proposed improvements that preserve the system's core identity.
>
> **Review file:** `docs/reviews/the-deep-editorial-review-v2.md`
> **Previous design improvements:** `docs/plans/2026-02-25-deep-design-improvements.md`

---

## Review Findings Summary (84/100)

| Area | Score | Remaining Criticism |
|------|-------|---------------------|
| Discovery & Onboarding | 82 | Archetype intro through tooltips not play; auto-resolve window not visually flagged |
| Information Design | 81 | Familiarity thresholds invisible; Bridge ROI vague |
| Progression & Ceremony | 82 | Early prestige cycles feel like entry fees; post-Gateway vacuum persists |
| Abyss Identity | — | "Harder Sunken Reach" — no distinct mechanics or atmosphere |
| Post-Gateway | — | No second seal, no Descent narrative, static gold text |

---

## Tier 1: Quick Wins — Low Effort, High Impact

### 1. Time-Seeded Quote Rotation

**Problem:** Compact hub quotes are static within a session (`generation_counter % 5` = same quote until prestige).

**Change:** Replace generation-keyed index with time-based rotation using `current_millis() / 12_000`. Rotates every 12 seconds (slower than full hub's 8s for readability at compact density).

```rust
// Before (deep_missions.rs ~line 141):
let quote_idx = (deep.persistent.generation_counter as usize) % quotes.len();

// After:
let millis = super::scene_fx::current_millis();
let quote_idx = (millis / 12_000) as usize % quotes.len();
```

*Designers 1 & 3 both proposed this; Designer 3's millis approach is simpler (no new field needed).*

**Files:** `src/ui/deep_missions.rs` (~line 141)
**Effort:** Low — single line change.

### 2. Injury Flavor by Archetype

**Problem:** Injury messages are generic ("Gareth is injured — 2 missions"). A Medic being injured feels the same as a Vanguard. Missed attachment opportunity.

**Change:** Add `injury_flavor(archetype, severity) -> &'static str` pure function in roster rendering. Append 3-5 word flavor to injury display:

| Archetype | Light | Moderate | Severe |
|-----------|-------|----------|--------|
| Vanguard | "bruised but standing" | "shield arm broken" | "took the hit for the squad" |
| Scout | "twisted ankle in the dark" | "fell into a fissure" | "barely made it back" |
| Arcanist | "overchanneled, stabilizing" | "ward collapsed inward" | "mind touched something old" |
| Medic | "healed others first" | "no one left to tend her wounds" | "spent everything keeping them alive" |
| Saboteur | "trap misfired nearby" | "caught in his own device" | "the mechanism wasn't done yet" |

Display: `Gareth the Ironwall — Lv3 — took the hit for the squad (3 missions)`

**Files:** `src/ui/deep_roster.rs`
**Effort:** Low — pure render function, no state changes.

### 3. Post-Gateway Atmospheric Rotation

**Problem:** "The Gateway stands open. The Wellspring waits." is a single static gold line. Post-Gateway atmosphere becomes frozen at the moment it should be most alive.

**Change:** Replace the single string with 4 rotating messages cycling every 10 seconds, all in gold:

```
"The Gateway stands open. The Wellspring waits."
"The Wellspring has seen this before. It is patient."
"What waits below the Wellspring is not a reward. It is an answer."
"Your predecessors went as far as this. You have gone further."
```

The fourth message uses the generational theme — post-Gateway players have genuinely outrun every previous generation.

**Files:** `src/ui/deep_missions.rs` (~line 502)
**Effort:** Low — array swap only.

---

## Tier 2: Narrative Identity — Medium Effort

### 4. Abyss Narrative Concept: "The Compression"

**Problem:** The Abyss lacks a unified narrative concept. Its messages are "things are weird" without cohering into an identity distinct from Sunken Reach.

**Change:** Give the Abyss a unified concept: **compression** — space, time, and identity compress near the Wellspring. Replace atmospheric messages with specific symptoms felt through mercenaries:

```
"Mira returned with six days of rations consumed. She was gone four hours."
"The Vanguard's battle-axe is two inches shorter. The edge is sharper."
"Sound travels wrong here — you hear your orders before you give them."
"Your Medic's wound records don't match. She was injured on missions she hasn't run."
"The Wellspring doesn't call to you. It recognizes you."
```

Optionally use `{merc}` placeholder replaced at render time with an actual roster merc name (capped at 10 chars) for personal connection.

**Files:** `src/ui/deep_missions.rs` (tier_atmosphere_messages), `src/ui/deep_scene.rs`
**Effort:** Low-Medium — message swap + optional name replacement.

### 5. Abyss Visual Identity — Backdrop Tint

**Problem:** The cave backdrop uses the same blue gradient at L19 as at L1. No visual signal you've entered different territory.

**Change:** In `paint_deep_backdrop()`, when frontier layer is Abyss tier, shift the gradient from pure blue to warm-purple:

```rust
// Normal:  top (5, 8, 20) → bottom (2, 3, 8)
// Abyss:   top (8, 6, 18) → bottom (5, 2, 6)   // warm-purple tint
```

Requires passing `current_tier` to the backdrop function — one field addition to the signature.

**Files:** `src/ui/deep_scene.rs`
**Effort:** Low — parameter addition + color match arm.
**Risk:** Must not make text unreadable. Test at S/M tiers.

### 6. Bridge ROI — Concrete Time Savings

**Problem:** BUILD OPTIONS shows Bridge as "-10% duration on deeper pushes" — vague compared to Supply Cache and Watchtower descriptions.

**Change:** Compute and display concrete time savings in BUILD OPTIONS:

```
Bridge  — 150M
  Skip this layer on deeper missions
  With 2 bridges built: -19% on Abyss missions (~1h 32m saved per Expedition)
  ROI: recoups after ~3 deeper missions
```

Show cumulative bridge count and effective savings. Display savings line in `Color::Cyan`.

**Files:** `src/ui/deep_layers.rs`, `src/deep/layers.rs` (expose `bridge_duration_savings_secs()` helper)
**Effort:** Low-Medium.

---

## Tier 3: Mechanical Depth — Medium-High Effort

### 7. Abyss Echoes (Layer-Specific Passive Modifiers)

**Problem:** L19-25 use the same mission types with no mechanical distinction from earlier tiers.

**Change:** Each Abyss layer has one permanent "Echo" — a fixed passive modifier that fires on mission completion:

| Layer | Echo | Effect |
|-------|------|--------|
| L19 | The Hunger | Supply Runs +20% marks, mercs gain 5 Fatigue |
| L21 | The Pressure | Breakthroughs +15 familiarity auto-gain, +2h duration |
| L23 | The Silence | Recons give double intel, but no check-in events fire |
| L25 | The Current | One random merc gains +1 permanent stat point |

Echoes are fixed per layer (not random). Players learn and plan around them.

**Files:** `src/deep/types.rs` (AbyssEcho enum), `src/deep/layers.rs` (echo resolution)
**Effort:** Medium.
**Risk:** Fatigue stacking on L19 — cap at 25 to prevent cascade injuries.

### 8. Void Pillars (Persistent Milestones)

**Problem:** The Void is an infinite treadmill with no waypoints.

**Change:** Every 5 Void layers (L30, L35, L40...) is a named "Pillar" — permanently recorded in `DeepPersistent`. Reaching a new Pillar:
- Records `deepest_pillar: u32` (persists across prestige)
- Displays a golden marker on the layer list
- Grants one bonus infrastructure slot on that layer (5 instead of 4)
- Shows an epitaph line in the Hub header

**Files:** `src/deep/types.rs`, `src/ui/deep_layers.rs`
**Effort:** Low-Medium.

### 9. The Second Seal (Post-Gateway Objective)

**Problem:** Post-Gateway purpose vacuum — no horizon after L30.

**Change:** After the Gateway fires, the Hub displays: "The Second Seal stirs — Layer 50." Reaching L50 triggers a 5-event sequence (reuses Gateway event engine) with new narrative text. Awards:
- Second golden epitaph in Hub
- Unique guild title "Void-Touched"
- No mechanical reward beyond the narrative moment

Frame as a legend, not an expectation — the Hub text reads as lore, not a quest marker.

**Files:** `src/deep/types.rs` (`second_seal_completed: bool`), `src/deep/layers.rs`, `src/ui/deep_scene.rs`
**Effort:** Low-Medium (reuses Gateway infrastructure).
**Risk:** L50 may be unreachable for most players. Framing is critical.

### 10. Gateway Crossed Badge

**Problem:** Players who clear L30 and enter the Void have no persistent visual marker of that achievement.

**Change:** Add `gateway_crossed: bool` to `DeepPersistent`. When `mark_layer_cleared(30)` fires, set the flag. Display in Hub header:

```
Frontier: Layer 34 (The Void)  [GATEWAY CROSSED]
  What lies beyond has no name.
```

Badge in `Color::Rgb(120, 80, 180)` (deep violet). Rotating Void subtitle from 5 strings.

**Files:** `src/deep/types.rs`, `src/deep/layers.rs`, `src/ui/deep_missions.rs`
**Effort:** Low-Medium.

---

## Preserved Design Principles

All proposals were audited against these non-negotiables:

- **"Reward attention, never punish absence"** — No proposals add time pressure or FOMO. Echoes are passive; Pillars are permanent; quotes are cosmetic.
- **Two-tier persistence** — New persistent fields (`gateway_crossed`, `deepest_pillar`, `second_seal_completed`) use `#[serde(default)]`. Session-only state (quote index) is not persisted.
- **Patience as design material** — No proposals reduce meaningful wait time. They reduce the perception of dead time through responsive atmosphere.
- **Atmospheric identity** — All proposals reinforce the cave's identity. The Abyss tint, compression narrative, and injury flavors deepen the world rather than replacing it.
- **Idle rhythm** — All changes are visible during normal 2-4 daily check-ins. No additional player action required.

---

## Recommended Implementation Roadmap

**Sprint 1 (Polish — 1 day):** Items 1-3 — Quote rotation, injury flavor, post-Gateway rotation
**Sprint 2 (Identity — 2-3 days):** Items 4-6 — Abyss compression narrative, backdrop tint, Bridge ROI
**Sprint 3 (Depth — 3-5 days):** Items 7-10 — Abyss Echoes, Void Pillars, Second Seal, Gateway badge

**Target score improvement:** 84 → 88-90 if all three sprints ship. Sprint 1 alone should move polish scores by +2-3 points.

---

## Designer Credits

| Designer | Focus | Key Proposals |
|----------|-------|---------------|
| 1 | Onboarding & Session Structure | Quote cycling, Bridge ROI, Abyss visual signal, Gateway badge |
| 2 | Abyss & Endgame Depth | Abyss Echoes, Void Pillars, Second Seal |
| 3 | Atmosphere & Polish | Time-seeded quotes, Compression narrative, post-Gateway rotation, injury flavor |

*Workshop output synthesized from 3 independent design proposals analyzing the 84/100 editorial review.*
