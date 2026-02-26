# The Deep — Design Improvements from Editorial Review

> **Source:** Editorial review scored The Deep 81/100. Three game designers independently analyzed the review's criticisms and proposed improvements.
>
> **Review file:** `docs/reviews/the-deep-editorial-review.md`

---

## Review Findings Summary

| Area | Score | Key Criticism |
|------|-------|---------------|
| Discovery & First Impressions | 78 | First 48h too empty with 1 mission slot |
| Core Mechanic Depth | 78 | Information scattered across menus; infrastructure ROI opaque |
| Long-Term Progression | 84 | Post-Gateway purpose dissipates |
| Engagement Design | 83 | (Praised — no major criticism) |
| Aesthetics & Polish | 84 | Compact terminals lose atmospheric identity |

---

## Tier 1: High Impact, Low Effort (Ship First)

### 1. Full Effective Duration at Mission Launch

**Problem:** Mission detail panel shows base duration, not the fully-reduced effective duration after infrastructure, familiarity, and Saboteur modifiers. A player with Mastered + Outpost + Saboteur sees "8h" when the actual time is ~3h 12m.

**Change:** In `render_mission_detail_phase1()` in `deep_missions.rs`, compute and display the fully-modified duration:
```
Duration:  8h 0m  →  3h 12m effective
           (Outpost -25%, Mastered -30%, Saboteur -10%)
```
Breakdown line only appears when modifiers are active.

**Files:** `src/ui/deep_missions.rs` (rendering only)
**Effort:** Low — `apply_duration_modifiers()` already exists; pure rendering change.

### 2. Infrastructure Comparison Row in Layers Panel

**Problem:** Choosing between Watchtower vs. Supply Cache requires mental arithmetic across scattered menus.

**Change:** Add a "BUILD OPTIONS" section at the bottom of the infrastructure detail panel showing each unbuilt type with a one-line ROI summary:
```
BUILD OPTIONS
  Supply Cache   ~4 supply runs to break even  (need 178M)
  Watchtower     +25 fam immediately           (need 155M)
  Bridge         -10% duration on deeper pushes (need 195M)
```

**Files:** `src/ui/deep_layers.rs` (rendering only)
**Effort:** Low — one arithmetic heuristic per infrastructure type.

### 3. Compact Hub Mode for Small Terminals

**Problem:** On S-tier terminals (40x16), the cave backdrop disappears and the hub becomes a featureless scoreboard. The system's atmospheric identity is lost.

**Change:** Add a `SizeTier::S` branch in hub rendering with this layout:
```
 THE DEEP                    Gen.4
 "The tunnels breathe."
 ─────────────────────────────────
 GUILD  Legion (Rank 5)    L28
 MARKS  340 WM
 ─────────────────────────────────
 > Supply: L4    2h 14m  [evt!]
 > Recon:  L22   6h 03m
 ─────────────────────────────────
 [N]ew  [R]oster  [L]ayers  [?]
```
Key: atmospheric quote on line 2 carries the identity; generation counter right-aligned on title line; event indicator always inline.

**Files:** `src/ui/deep_missions.rs`, `src/ui/deep_scene.rs`
**Effort:** Low — layout branch within existing responsive framework.

### 4. Shallows Supply Run Duration: 30min → 20min

**Problem:** First Supply Run takes 30 minutes with no modifiers. Players check in, see it's still running, and close the game.

**Change:** Reduce `base_mission_duration_secs(Shallows, SupplyRun)` from 1800s to 1200s. Consider reducing `MIN_MISSION_DURATION_SECS` from 1800 to 900 for Shallows-tier missions.

**Files:** `src/deep/layers.rs`
**Effort:** Low — single constant change.
**Balance impact:** More runs per day in Rank 1 window, but mark yields per run are unchanged. Breakthrough/rank costs are absolute, so this reduces dead time without compressing time-to-rank.

---

## Tier 2: High Impact, Medium Effort

### 5. Split Guild Rank 2 Gate: Mission Slot at Breakthrough, Rank at Marks

**Problem:** The second concurrent mission slot — "the single most satisfying mechanical moment" per the review — is gated behind both Layer 3 Breakthrough AND 200 Marks. The 200-mark requirement adds 12-24h of single-slot farming after the player has earned the real milestone.

**Change:** Decouple the two rewards:
- **Layer 3 Breakthrough** → immediately unlocks 2 concurrent mission slots (no mark cost)
- **200 Marks** → formal Sellswords rank (roster expands to 7, Arcanist archetype unlocks)

**Implementation:** Add a helper that checks `persistent.deepest_layer_reached >= 3` for concurrent missions independently of `guild_rank`. Or add a `bonus_concurrent_slots` field.

**Files:** `src/deep/types.rs`, `src/deep/economy.rs`, `src/ui/deep_missions.rs`
**Effort:** Medium — requires separating concurrent mission logic from rank logic.
**Balance impact:** Second slot arrives ~24h earlier. The 200-mark gate still paces roster expansion and archetype unlocks.

### 6. Starter "First Orders" Mission

**Problem:** After discovering The Deep and launching a first Supply Run, there's nothing to collect on the second visit.

**Change:** On first discovery, auto-queue a special one-time 20-minute Recon: "First Orders — Scout the Shallows." Returns +30 familiarity for Layer 1, +15 marks, and a brief narrative fragment. The starter trio is "already on their way."

**Implementation:** Add `first_orders_completed: bool` to `DeepPrestige` (reset on prestige — only fires on first prestige post-discovery, controlled by a `first_orders_ever: bool` on `DeepPersistent`).

**Files:** `src/deep/types.rs`, `src/deep/discovery.rs`, `src/deep/missions.rs`
**Effort:** Medium — one-time hardcoded mission with narrative text.

### 7. Abyss Entry Familiarity Bonus

**Problem:** Entering the Abyss (L19) at Unknown familiarity means the first Recon takes 5h at full duration — maximum resource pressure coinciding with maximum wait.

**Change:** When Layer 18 Breakthrough completes, award +25 familiarity on Layer 19 automatically. Narrative: "Your scouts recognize patterns from the Sunken Reach. Layer 19 starts Mapped."

**Implementation:** In `mark_layer_cleared()`, when cleared layer is 18, set L19 familiarity to 25 (Mapped). One-time, applied during the existing breakthrough resolution.

**Files:** `src/deep/layers.rs`
**Effort:** Low-Medium — small code change, but needs a persistence flag to avoid re-triggering.

### 8. Generation Records (Layer Echoes)

**Problem:** The `generation_counter` increments but has no mechanical weight. Post-Gateway, each generation inherits a static monument.

**Change:** On prestige, record the generation's stats: marks earned, missions completed, mercs lost, deepest new layer reached, whether gateway was opened. Display previous two generations' records in the Hub. Infrastructure built by notable generations gets commemorative tags in the layer map.

**New type:**
```rust
pub struct GenerationRecord {
    pub generation: u32,
    pub marks_earned: u32,
    pub missions_completed: u32,
    pub mercs_lost: u32,
    pub deepest_layer_reached: u32,
    pub gateway_opened_this_generation: bool,
}
```
Add `generation_records: Vec<GenerationRecord>` to `DeepPersistent` (capped at 10).

**Files:** `src/deep/types.rs`, `src/deep/missions.rs` (on prestige), `src/ui/deep_missions.rs`
**Effort:** Medium — record-keeping and UI display, no new systems.

---

## Tier 3: High Impact, High Effort (Future Milestones)

### 9. The Descent — Post-Gateway Narrative Progression

**Problem:** After opening the Gateway, there's no destination. The review says: "The system needs a post-Gateway objective — a second seal, a deeper mystery, something to point toward."

**Change:** Five Descent stages, each unlocking after a post-Gateway prestige cycle with specific infrastructure requirements:

| Stage | Name | Requirement | Reward |
|-------|------|-------------|--------|
| 1 | The First Step | Outposts on all Shallows layers | +10% Warband Marks yield, permanent |
| 2 | The Memory Hall | Full infra on L1-12 | Mercs start at Level 3 each prestige |
| 3 | The Wellspring's Edge | Full infra on L1-18 | -15% infrastructure build costs |
| 4 | The Old Tongue | Full infra on L1-25 | Auto-resolve picks bonus path, not safe |
| 5 | The Source | Full infra on all 30 layers | Ascendant rank (20 roster, 5 concurrent, Elite-only pool) |

Each stage is a narrative modal triggered at prestige. The infrastructure requirements mean post-Gateway generations have a specific goal: complete the monument. The hero's personal Descent is the payoff for what your mercs prepared.

**Files:** `src/deep/types.rs` (new persistent fields), `src/deep/missions.rs` (stage checks), `src/ui/deep_scene.rs` (modals)
**Effort:** High — 5 narrative event sequences, new guild rank, 3-4 persistent fields.

### 10. Abyss Pulse Events

**Problem:** The Abyss uses the same verbs as earlier tiers at higher costs. It lacks distinct identity.

**Change:** Every 48h of real time with active Abyss missions, a Pulse fires — a hub-level notification with one of three effects:
- **Temporal Surge:** Active Abyss missions complete 20% faster for 6h
- **Resonant Echo:** Familiarity gain doubled on Abyss layers for 12h; Mastering during Echo awards +50 bonus Marks
- **Void Tithe:** Sacrifice 100 Marks for +2 effective power on all Abyss mercs for 24h (optional)

**Files:** `src/deep/types.rs` (timer field), `src/deep/missions.rs` (pulse logic), `src/ui/deep_missions.rs` (notification)
**Effort:** Medium-High — new event system scoped to Abyss tier.

### 11. Depth Anchor — Abyss-Only Infrastructure

**Problem:** Infrastructure ROI inverts at depth. Outpost costs 199-235 Marks in the Abyss but provides the same -25% as at Layer 4.

**Change:** New infrastructure type available only on L19-25: **Depth Anchor**. Reduces power thresholds for Expedition and Breakthrough by 8% (floor: 90% of base). Costs 280 + 8*layer Marks. Requires Familiar familiarity (50+) to build.

**Files:** `src/deep/types.rs` (new enum variant), `src/deep/layers.rs` (validation, threshold modification)
**Effort:** High — new infrastructure with unique validation and threshold interaction. Needs balance testing.

### 12. Wellspring Essence — Cross-System Currency (Future)

**Problem:** Post-Gateway Deep has no connection to Quest's broader meta-progression.

**Change:** Deep Void missions (L31+) occasionally yield Wellspring Essence — a rare currency spent in a Wellspring Exchange for permanent character bonuses (+XP, +enhancement cap, +sigil slot, new Haven room).

**Effort:** Very High — new currency, new UI, cross-system integration. Ship only after Descent is validated.

---

## Preserved Design Principles

All proposals were evaluated against these non-negotiables from the review:

- **"Reward attention, never punish absence"** — No proposal introduces time pressure or FOMO mechanics
- **Two-tier persistence** — Infrastructure persists, currency resets. No changes to this contract
- **Discovery moment weight** — No changes to the P15+ discovery roll timing
- **Atmospheric identity** — The cave, the text rotation, the visual register shift are preserved and extended (not replaced) in compact mode
- **Patience as design material** — Proposals reduce dead time, not meaningful wait time

---

## Recommended Implementation Roadmap

**Sprint 1 (Polish):** Items 1-4 — Information clarity + compact mode + duration tuning
**Sprint 2 (Onboarding):** Items 5-7 — Split rank gate + starter mission + Abyss entry bonus
**Sprint 3 (Depth):** Items 8, 10 — Generation records + Abyss Pulses
**Sprint 4 (Endgame):** Item 9 — The Descent narrative progression
**Future:** Items 11, 12 — Depth Anchor + Wellspring Essence

---

*Generated from 3 independent game designer analyses of the editorial review (81/100). Proposals target moving the score to 86-90 by addressing the four specific criticisms: early pacing (78→84), information design (78→85), Abyss dead zone, and post-Gateway purpose.*
