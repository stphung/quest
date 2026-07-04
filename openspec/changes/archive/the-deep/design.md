> Backported design record. Sources: docs/plans/2026-02-22-the-deep-mercenary-expedition-design.md, docs/plans/2026-02-24-deep-narrative-design.md, docs/plans/2026-02-25-deep-design-improvements.md, docs/plans/2026-02-26-deep-designer-workshop.md, docs/plans/2026-02-26-deep-discovery-redesign.md, docs/plans/2026-02-26-deep-hub-ui-improvements.md, docs/plans/2026-03-02-deep-panel-design.md, docs/plans/deep-balance-design.md, docs/plans/deep-events-design.md, docs/plans/deep-integration-architecture.md, docs/plans/deep-quality-standards.md, docs/plans/deep-t1t2-narrative.md, docs/plans/deep-ui-audit.md, docs/plans/deep-ui-design.md, docs/plans/deep-ui-hub-missions-design.md, docs/plans/deep-ui-onboarding-design.md, docs/plans/deep-ui-roster-layers-design.md.

## 2026-02-22-the-deep-mercenary-expedition-design.md

# The Deep — Mercenary Expedition System

## Overview

An endgame (P15+) system where players recruit and manage a mercenary company, sending squads on long-duration missions (2-24 hours real-time) that push deeper into a vast underground structure called The Deep. Each prestige cycle represents a new generation of mercenaries inheriting the infrastructure, maps, and outposts left behind by previous generations.

**Key differentiators from existing systems:**
- **Timescale**: Hours/days vs. seconds/minutes — a fundamentally different engagement rhythm
- **Generational theme**: Each prestige sends a new generation deeper, standing on the shoulders of those before
- **Strategic depth**: Squad composition, risk/reward, infrastructure investment — decisions that matter over days
- **Optional engagement**: Check-in events reward attention but never punish absence
- **Wall-clock time**: Missions progress in real time, including while the game is closed

## Discovery & Gate

- Requires P15+ (same tier as Soulforge and Stormglass)
- Discovered via tick-based random roll (same pattern as Haven/Soulforge discovery)
- On discovery, a combat log message introduces the mercenary captain and The Deep
- Unlocks The Deep overlay (accessible via keybind, like Haven/Soulforge/Stormglass)

## Mercenaries

### Archetypes

Each mercenary has an archetype that determines their stat distribution and unlocks unique options during check-in events:

| Archetype | Primary Stats | Role | Special Ability |
|-----------|--------------|------|-----------------|
| Vanguard | STR/CON | Frontline tank | Reduces squad casualties |
| Scout | DEX/WIS | Recon specialist | Reveals events earlier, better auto-resolve |
| Arcanist | INT | Elemental specialist | Counters environmental hazards |
| Medic | WIS/CON | Healer | Reduces injury severity, prevents merc loss |
| Saboteur | DEX/INT | Trap/obstacle specialist | Speeds up missions, unlocks alternate routes |

### Stats

- **Power** — combat effectiveness (derived from archetype + level)
- **Resilience** — survival chance when things go wrong
- **Expertise** — bonus to missions matching their specialty
- **Level** — gained from completing missions (resets on prestige)

### Recruitment

- Mercs recruited from a rotating pool (refreshes daily or on prestige)
- Cost: Warband Marks
- Higher guild rank = better quality recruits (higher base stats, rarer archetypes)
- Starting roster: 4-5 mercs at discovery, max scales with guild rank (up to ~15)

### Merc Loss

- On failed frontier missions, mercs can be **injured** (unavailable for 1-2 missions) or **lost** (permanently gone)
- Medics and high-resilience squads reduce this risk
- Supply runs on cleared layers are always safe
- Losing a high-level merc creates real stakes for frontier pushes

## The Deep — Layer Structure

A vast underground structure of unknown origin. Each layer is deeper, stranger, and more dangerous.

### Layer Tiers

```
Layer 1-3:   The Shallows     (introductory)
Layer 4-7:   The Warrens      (branching tunnels, first real challenges)
Layer 8-12:  The Hollows      (open caverns, environmental hazards)
Layer 13-18: The Sunken Reach (flooded/corrupted, specialist mercs shine)
Layer 19-25: The Abyss        (extreme danger, multi-event missions)
Layer 26+:   The Void         (infinite scaling, endgame prestige sink)
```

### Layer Properties

Each layer has:
- **Theme** — determines hazard types, enemy archetypes, and check-in event flavor
- **Difficulty rating** — scales mission duration, risk, and reward
- **Infrastructure slots** — things previous generations can build that make future missions easier
- **Intel level** — starts at 0 (unknown), increases from completing missions on that layer

### Layer Progression

- To unlock Layer N+1, complete a **breakthrough mission** on Layer N
- Breakthrough missions are the hardest type: 18-24h, multiple events, boss encounter
- Once broken through, a layer becomes "cleared" — available for supply runs and construction
- Cleared layers persist permanently across prestiges

### Infrastructure (persists across prestiges)

Built by sending mercs on construction missions (shorter, safe, costs Warband Marks):

| Infrastructure | Effect |
|---------------|--------|
| Outpost | Reduces mission duration on this layer by 25% |
| Supply Cache | Supply run missions yield bonus resources |
| Watchtower | Reveals more intel, improves auto-resolve outcomes |
| Bridge | Unlocks shortcut, missions can skip this layer when pushing deeper |

## Missions

### Mission Types

| Type | Duration | Risk | Available On | Purpose |
|------|----------|------|-------------|---------|
| Supply Run | 2-4h | None | Cleared layers | Resource farming, safe merc XP |
| Recon | 4-8h | Low | Frontier layer | Gather intel, reveal events |
| Expedition | 8-16h | Medium | Frontier layer | Main progression |
| Breakthrough | 18-24h | High | Frontier layer (once) | Unlock next layer, boss encounter |
| Construction | 4-8h | None | Cleared layers | Build infrastructure |

### Mission Generation

- 3-5 missions available at a time, refreshing as missions complete
- Pool depends on: current frontier layer, intel level, guild rank
- Higher intel = better mission options (shorter duration, better rewards, less risk)

### Squad Assignment

- Each mission has **requirements** (e.g., "needs 1+ Arcanist", "minimum squad Power 50")
- And **recommendations** (e.g., "Scout recommended — improves event outcomes")
- Player picks 3-5 mercs. Assigned mercs unavailable until mission ends.
- Overpowered squads finish faster, better outcomes. Underpowered squads risk injury/loss.

### Check-In Events

Events fire at scheduled intervals during missions (e.g., 25%, 50%, 75% through):

```
[6h into 16h Expedition, Layer 9]

    CAVE-IN AHEAD

    Your squad encounters a collapsed
    tunnel blocking the main path.

    > [Vanguard] Dig through (3h delay, safe)
    > [Saboteur] Find alternate route (no delay, risk)
    > [Arcanist] Blast through (1h delay, costs supplies)

    Auto-resolve (if ignored): Dig through
```

- 1-2 events on short missions, 3-5 on breakthroughs
- Having the right archetype unlocks better options
- Auto-resolve always picks the safest choice (never risks merc loss)
- Events can chain — risky choice in event 1 may create bonus opportunity in event 3

### Mission Resolution

When timer completes, mission resolves based on: squad power vs. layer difficulty, event choices, small random factor.

- **Success** — full rewards
- **Partial success** — reduced rewards, possible injuries
- **Failure** — minimal rewards, injuries/losses

## Economy & Rewards

### Warband Marks (resets on prestige)

Primary currency for the merc system:
- Earned from all mission types (more from harder/deeper missions)
- Spent on: recruiting mercs, building infrastructure, upgrading guild rank

### Guild Rank (persists across prestiges)

| Rank | Name | Max Roster | Concurrent Missions | Unlock Requirement |
|------|------|-----------|---------------------|-------------------|
| 1 | Freelancers | 5 | 1 | Discovery |
| 2 | Sellswords | 7 | 1 | Layer 3 breakthrough |
| 3 | Company | 9 | 2 | Layer 7 breakthrough |
| 4 | Battalion | 12 | 3 | Layer 13 breakthrough |
| 5 | Legion | 15 | 4 | Layer 19 breakthrough |

Guild rank costs Warband Marks to upgrade, but the rank itself persists. Each prestige you start at your earned rank with full roster size and mission slots.

### Rewards Flowing Into Existing Systems

| Mission Type | Rewards |
|-------------|---------|
| Supply runs | Modest XP, common items |
| Expeditions | Good XP, uncommon-rare items, Stormglass |
| Breakthroughs | Large XP, rare-legendary items, Stormglass, prestige rank fragments |
| Construction | Infrastructure (no direct loot) |
| Deep layers (19+) | Chance at unique merc-exclusive hero equipment |

### Merc-Exclusive Hero Rewards

- **Abyssal equipment** — items only found through The Deep. Unique affixes not available elsewhere (e.g., "merc mission speed +10%", "supply run yield +25%", or Deep-themed combat affixes)
- **Layer trophies** — cosmetic achievements tracking deepest push across all generations

### Prestige Rank Fragments

- Breakthrough missions on deep layers award fractional prestige ranks (0.5 or 1 PR)
- Creates an alternate path to earning prestige ranks alongside the combat loop

## Persistence Model

### Persists Across Prestiges
- Guild rank
- Deepest layer reached (cleared layers stay cleared)
- Infrastructure built on each layer
- Intel levels per layer
- Campaign/narrative progress
- Layer trophies and achievements

### Resets On Prestige
- Individual mercenaries (recruit fresh each generation)
- Active missions (auto-cancel on prestige)
- Warband Marks currency
- Merc gear/levels

## Time Model

- Missions run on **wall-clock time**, not game ticks
- Missions progress while the game is closed
- On login, completed missions show results; missed events auto-resolved
- Check-in events that fired while offline use the safe auto-resolve default

## UI

### The Deep Overlay

Modal panel (like Haven/Soulforge/Stormglass) opened via keybind:

```
┌─ THE DEEP ──────────────────────────────────────────────┐
│                                                         │
│  Guild: Sellswords (Rank 2)     Warband Marks: 1,240   │
│  Deepest Layer: 8 (The Hollows) Mercs: 6/7             │
│                                                         │
│  ┌─ ACTIVE MISSIONS ─────────────────────────────────┐  │
│  │ ► Expedition L8    12h  [████████░░] 78%  Squad A │  │
│  │   ⚡ Event pending! Press [Enter] to respond      │  │
│  │ ► Supply Run L4     3h  [██████████] Done!        │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  [N] New Mission  [R] Roster  [I] Infrastructure  [Esc] │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Sub-Views

- **New Mission** — available missions, select one, pick squad
- **Roster** — merc list with stats, archetype, level, status
- **Infrastructure** — layer-by-layer view of built infrastructure
- **Event Response** — check-in event choices when an event is pending

### Integration With Main Game

- Overlay is modal, opens over the combat scene
- Missions tick regardless of whether overlay is open
- Pending events show a subtle indicator in the main stats panel
- Completed missions queue rewards for collection

## Architecture Integration

### New Module: `src/deep/`

Following existing module patterns:
- `types.rs` — Mercenary, Layer, Mission, Guild, DeepState structs
- `generation.rs` — Merc generation, mission generation, event generation
- `logic.rs` — Mission ticking, event resolution, squad assignment validation
- `persistence.rs` — Save/load from `~/.quest/deep.json` (account-level)
- `discovery.rs` — Discovery roll logic

### Game State Integration

- Add `deep_state: Option<DeepState>` to account-level persistence (like Haven, Soulforge)
- Discovery via tick-based roll in `tick.rs` (new stage or extend existing discovery stage)
- Wall-clock mission ticking handled on game load (similar to offline XP) and periodically during play

### Input & UI

- New input handler: `src/input/deep_input.rs`
- New UI scene: `src/ui/deep_scene.rs` (with sub-modules for roster, missions, infrastructure views)
- Keybind to toggle overlay (like `h` for Haven, `j` for Soulforge)

## 2026-02-24-deep-narrative-design.md

# The Deep — Narrative Design: The Sealed Root

**Date:** 2026-02-24
**Status:** Approved
**Tone:** Dark fantasy — grim, dangerous, morally grey

## Overview

The Deep is not a geological formation. It is a root system — metaphysical, not mineral. The Expanse's "raw, unformed reality" was never formless. It was the canopy of something buried below. The Abyssal Rift in Zone 11 is the literal entrance — a wound in reality that goes *down*.

Millennia ago, a civilization discovered the Wellspring — a source of power beneath everything. They built downward to reach it. They almost made it. Something stopped them. They sealed it shut and drowned their own work. What your mercenaries find is their remains: infrastructure that still works, warnings that were ignored, and the thing they were trying to contain.

At Layer 30, your mercs reach the Gateway — a sealed passage inscribed with runes matching the god items (Asprika, Sleipnir, Megingjord). Opening it is the narrative climax of The Deep. What lies beyond is a hook for future expansion.

## Part 1: The Expanse Story Event Chain

### Core Mechanic: Rift Resonance

A persistent counter (`rift_resonance: u32` in `DeepPersistent`) that increments by +1 each time the player prestiges after reaching The Expanse in that cycle.

**Increment condition:** On prestige, if the player reached Zone 11 (The Expanse) during the current cycle, and prestige rank >= 15, then `rift_resonance += 1`.

This naturally forces 5-8 prestige cycles to complete the discovery chain, since the player must push to The Expanse each time before prestiging.

### Event Chain (4 story stages)

| Resonance | Prestige Gate | Story Event |
|-----------|--------------|-------------|
| 1 | P15 | **Tremors** — First qualifying prestige. Combat log flavor next cycle: "The ground beneath the Rift remembers you." Sets `deep_story_stage: 1`. |
| 3 | P17 | **The Captain** — Modal: A scarred mercenary captain finds the player. She's been tracking the tremors for years. "Every time you return, the Rift opens a little wider. It knows you now." Sets stage 2. |
| 5 | P19 | **The First Fragment** — After an Expanse boss kill: a Rift Fragment materializes. "It wasn't dropped. It was *given*. Something below is testing whether you'll keep coming back." Sets stage 3, `rift_fragments: 1`. |
| 7 | P21 | **The Entrance** — Four fragments have accumulated (one per resonance at 5, 6, 7, plus a final one at this stage). The captain arranges them. "They snap together — not fitting, but *remembering*. The earth opens. A stairway descends into absolute darkness. 'I'll need soldiers,' the captain says. 'Disposable ones.'" Sets stage 4. |

**Stage 5:** Player presses [D] for the first time → full discovery modal, starter mercs, The Deep begins.

### Fragment Collection

Rift Fragments are collected automatically at resonance 5, 6, 7, and the final one at resonance 7 (4 total). No inventory system needed — just a counter (`rift_fragments: u8`) that increments at each qualifying resonance level.

### Persistent Tracking

New fields in `DeepPersistent` (all `#[serde(default)]`):

```rust
pub rift_resonance: u32,              // +1 per qualifying prestige
pub deep_story_stage: u8,             // 0-5
pub rift_fragments: u8,               // 0-4
```

### Backward Compatibility

Players who already discovered The Deep via the old P15+ random roll retain `discovered: true`, which bypasses the chain entirely. New fields default to 0.

The old discovery roll (`try_discover_deep()`) is replaced by the story chain for new players. Existing saves are unaffected.

## Part 2: Layer Narrative — The Descent

Each layer tier tells a chapter of the same story: someone came before you, went deeper, and failed. Your mercs are walking through the wreckage of that failure.

Narrative is delivered through three existing channels:
1. **Hub atmosphere messages** — rotating text in the mission hub (keyed to frontier layer tier)
2. **Check-in event descriptions** — tier-specific templates already support `{merc_name}` placeholders
3. **Mission result flavor text** — brief narrative additions to completion screens

### The Shallows (L1-3): The Abandoned Works

**Theme:** An old mining operation that clearly became something else. The tunnels are too deliberate, too straight. These weren't dug for ore.

**Narrative beats:**
- Tool marks transition from mining picks to ritual implements
- Carved warnings in a dead language — "Do not answer it"
- The Guardian (L3 boss) is a construct built to keep people *out*, not guard treasure

**Hub atmosphere messages:**
- "The walls here were carved with purpose. This was no mine."
- "Your scouts find a collapsed barracks. Decades of dust. Someone lived down here."
- "The captain traces a finger along a carved warning. She does not translate it."

### The Warrens (L4-7): The Sealed Quarters

**Theme:** Living quarters, archives, laboratories. A civilization that moved underground permanently. They were studying something below.

**Narrative beats:**
- Living spaces with personal effects — families lived here
- A sealed archive with stone tablets describing "the Wellspring" — a source of power beneath everything
- The Overseer (L7 boss) is an automaton left to maintain the seal, not a creature

**Hub atmosphere messages:**
- "Gareth found a child's doll in the rubble. Stone, but carefully carved. Someone loved this."
- "The archive tablets mention 'the Wellspring' seventeen times. Always with reverence. Sometimes with fear."
- "The Overseer's body twitches even in death. Its purpose outlasted its makers."

### The Hollows (L8-12): The Living Dark

**Theme:** Below the civilization's reach, the tunnels become organic. Bioluminescent growth covers everything. The stone breathes. The Wellspring's influence begins here — power leaking upward, making the rock alive.

**Narrative beats:**
- Infrastructure transitions from carved to grown — bridges like bone, walls that pulse
- Echoes appear — imprints of the old civilization's final expeditions, replaying endlessly
- The spore clouds are a defense mechanism — something is trying to keep you from going deeper

**Hub atmosphere messages:**
- "The walls pulse with a slow rhythm. It takes a moment to realize it matches your heartbeat."
- "Your Arcanist says the light here isn't bioluminescence. It's memory. The stone remembers being shaped."
- "An Echo walks past the camp. It doesn't see you. It never will."

### The Sunken Reach (L13-18): The Drowned Seals

**Theme:** The old civilization's last stand. Massive chambers flooded deliberately — water used as a barrier. Seals carved into walls, powered by mechanisms that still function. They sealed it shut and drowned their own work.

**Narrative beats:**
- Flooded corridors with intact seals — rune patterns match the god items
- The seals are cracking — not from age, but from pressure below. The Wellspring pushes back
- The Drowned King (L18 boss) was the civilization's last leader, who stayed behind to hold the seals

**Hub atmosphere messages:**
- "The seals glow brighter when your Arcanist approaches. They recognize power."
- "Water pressure should have crushed these chambers millennia ago. The seals hold more than water."
- "The Drowned King's throne faces downward. Even in death, it watched what lay below."

### The Abyss (L19-25): The Unraveling

**Theme:** Below the seals, reality comes apart. Tunnels don't follow geometry. Gravity shifts. Time stutters. The Wellspring's raw power saturates everything.

**Narrative beats:**
- Mercs lose time — hours feel like minutes. Injuries heal wrong
- The whispers aren't random — the Wellspring is trying to communicate. It's been alone for millennia
- The Void Warden (L25 boss) isn't guarding the gateway — it IS the gateway. A living seal, the last and most desperate measure

**Hub atmosphere messages:**
- "Your Scout returned from a 6-hour Recon convinced only twenty minutes had passed."
- "The whispers have stopped offering power. Now they just say: 'Finally.'"
- "The Void Warden doesn't attack. It simply exists in the way, vast and formless and sad."

### The Void (L26-29): The Approach

**Theme:** No more tunnels, no more stone. Pure void — unformed reality, the same substance as The Expanse above but concentrated, dense, *aware*. Each layer is a step closer to the Wellspring.

**Hub atmosphere messages:**
- "There is no stone here. Your mercs walk on solidified will."
- "The Wellspring pulses. It has been waiting longer than your world has existed."
- "Your Vanguard's wounds close before the Medic reaches them. The power here heals unbidden."

## Part 3: The Gateway (Layer 30)

Layer 30 is not a normal layer. When mercs complete the Layer 29 Breakthrough, Layer 30 is revealed as a special story layer.

### The Gateway Expedition

A unique mission type — `GatewayExpedition`. 24-hour duration, requires minimum 3 mercs, power threshold equal to a Layer 29 Breakthrough.

**5 unique check-in events (sequential):**

1. **The Threshold** — "The void condenses into a corridor. There are walls again — not stone, but light frozen solid. Your mercs can see through them. On the other side, something vast moves."
   - Safe: "Proceed carefully" (+2h)
   - Archetype (Scout): "Map the light-walls" (no delay)

2. **The Memory** — "Echoes of the old civilization's final expedition walk beside your squad. They reached this point. They turned back. Your mercs keep walking."
   - Safe: "Honor their memory" (no delay)
   - Risky: "Follow where they turned back" (65% success, bonus marks on success, +3h on failure)

3. **The Three Seals** — "Three pedestals. Three recesses shaped like artifacts you recognize — a shield, boots, a belt. The runes match Asprika, Sleipnir, Megingjord. The gateway requires divine keys."
   - Safe: "Study the seals" (+1h)
   - Archetype (Arcanist): "Channel prestige energy into the seals" (no delay, 50 mark cost)

4. **The Warden's Plea** — "The Void Warden materializes — not to fight, but to speak. 'Every seal I am was placed by someone who understood what lies beyond. They chose to close it. You are choosing to open it. Be certain.'"
   - Safe: "We're certain." (no delay)
   - Risky: "Break through by force" (70% success, skips time but injures mercs on failure)

5. **The Gate Opens** — "Your mercs place their hands on the gate. It doesn't open — it dissolves. Beyond it: light. Not sunlight. Older. The Wellspring."
   - Single choice: "Step back." (auto-resolves — mercs don't enter, they report what they saw)

### Mission Result

On completion, the result screen shows:

> *"Your company has opened the Gateway. What lies beyond is not for mercenaries. It is for you."*
>
> *The hero must descend personally.*

### Mechanical Effects

- Sets `gateway_opened: true` in `DeepPersistent`
- Unlocks achievement: `GatewayOpened` — "Opened the Gateway beneath the world"
- Hub permanently shows: "The Gateway stands open. The Wellspring waits."
- No new systems launch — this is a clean integration point for future expansion

### New Fields in `DeepPersistent`

```rust
pub gateway_opened: bool,  // #[serde(default)] — false
```

## Part 4: Integration Points

### Existing Lore Connections

- **God items (Asprika, Sleipnir, Megingjord):** Referenced in Gateway event 3 as the shapes of the seals. Implies god items were forged from Wellspring energy that leaked upward through the Rift. No mechanical dependency — the player doesn't need god items to open the Gateway.
- **Stormbreaker / Storm Citadel:** The storm is described as "a machine running since before memory." The Wellspring is the power source for that machine — another piece of the same puzzle, but not mechanically connected.
- **Prestige system:** The narrative frames prestige power as unconsciously drawing from the Wellspring. Ascending doesn't change the loop — it reveals its source.
- **The Expanse:** The Abyssal Rift subzone is the physical entrance. The Expanse's "raw, unformed reality" is the Wellspring's overflow, leaking upward.

### New Files / Modifications

**New or modified source files:**
- `src/deep/types.rs` — Add `rift_resonance`, `deep_story_stage`, `rift_fragments`, `gateway_opened` to `DeepPersistent`
- `src/deep/discovery.rs` — Replace random roll with story chain logic; add `check_rift_resonance()` hook for prestige
- `src/deep/events.rs` — Add Gateway Expedition event templates (5 unique events)
- `src/deep/missions.rs` — Add `GatewayExpedition` mission type handling
- `src/deep/types.rs` — Add `GatewayExpedition` to `MissionType` enum
- `src/ui/deep_missions.rs` — Hub atmosphere messages keyed to frontier tier; Gateway completion display
- `src/ui/deep_scene.rs` — Story event modals (Tremors, Captain, Fragment, Entrance)
- `src/input/prestige_input.rs` — Hook rift_resonance increment on qualifying prestige
- `src/achievements/types.rs` — Add `GatewayOpened` achievement
- `src/achievements/data.rs` — Achievement description and unlock condition

**New narrative content (no new systems):**
- 6 tiers × 3 hub atmosphere messages = 18 new rotating messages
- 5 Gateway Expedition check-in events with choices
- 4 story event modal texts (Tremors, Captain, Fragment, Entrance)
- 1 achievement

**Test files:**
- `tests/deep_story_chain_test.rs` — Rift Resonance incrementing, story stage progression, backward compat
- `tests/deep_gateway_test.rs` — Gateway Expedition mission flow, gateway_opened flag
- Updates to existing `tests/deep_integration_test.rs` and `tests/deep_prestige_persistence_test.rs`

## Part 5: Future Expansion Hook

The `gateway_opened: bool` flag in `DeepPersistent` is the integration point for whatever comes next — Ascension system, new zones, new mechanics. The narrative has planted seeds:

- The Wellspring is the source of prestige power
- The god items were forged from Wellspring energy
- The hero must descend personally (mercs can't go beyond)
- The old civilization tried and failed — implying the hero might succeed where they couldn't

None of these require implementation now. They're narrative threads available to pull when ready.

## 2026-02-25-deep-design-improvements.md

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

## 2026-02-26-deep-designer-workshop.md

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

## 2026-02-26-deep-discovery-redesign.md

# Deep Discovery Redesign

## Problem

The Deep requires 30 Rift Resonance (earned 1 per prestige from Zone 11) across a 10-stage story chain. This is an excessive time gate for an endgame system already behind P15+. The escalating cost curve is invisible to players, making progress feel like a flat 30-prestige grind.

## Design

Replace the Rift Resonance story chain with a single trigger: killing The Endless (Zone 11 subzone 4 boss) for the first time at P15+.

### Trigger

First `BossDefeatResult::ExpanseCycle` where `prestige_rank >= 15` and `!deep.persistent.discovered`.

### Player Experience

```
Kill The Endless (first time, P15+)
  → Single story modal: narrative moment about the earth cracking open
  → [Enter] to dismiss
  → Discovery modal: "The Deep Discovered!" (existing, unchanged)
  → Starter mercs created, First Orders mission queued
  → [D] keybind available
```

### Removals

- `rift_resonance` field on `DeepPersistent` (with `#[serde(default)]` for migration)
- `deep_story_stage` field and `STORY_STAGE_ENTRANCE`, `STORY_STAGE_DISCOVERED` constants
- `STORY_RESONANCE_THRESHOLDS` array
- `advance_deep_story()` in `discovery.rs`
- `check_story_progression()` method on `DeepState`
- `pending_story_stage` on `DeepUiState`
- `render_story_modal()` and `story_modal_content()` (10 story stages) in `deep_scene.rs`
- Rift Resonance display in stats panel (`Rift: X/30 · The Expanse responds to prestige`)
- Rift hint in prestige confirm dialog (`The Rift will remember this.`)
- `maybe_increment_rift_resonance()` and its call sites in `prestige_input.rs`
- `rift_hint` parameter threading through `draw_prestige_confirm`, `draw_stats_panel`, `draw_game_layout`

### Additions

- In `tick_stages.rs`, handle `BossDefeatResult::ExpanseCycle`: check discovery conditions, call `complete_story_discovery()`, emit `TickEvent::DeepDiscovered`
- Single story modal text for the boss-kill moment (replaces 10 stages)
- New field or reuse existing mechanism to show the story modal before the discovery modal

### Unchanged

- `complete_story_discovery()` internals (starter mercs, First Orders mission)
- Discovery modal UI (`render_deep_discovery_modal`)
- Debug menu "Discover The Deep" shortcut
- All post-discovery gameplay (missions, roster, layers, infrastructure)
- `DeepPersistent.discovered` field and save/load

### Migration

Existing saves with `rift_resonance > 0` but `discovered = false` will simply discover The Deep on their next Endless kill. Players who already discovered The Deep are unaffected — `discovered = true` is already set.

Fields removed from the struct use `#[serde(default)]` so old saves deserialize without error.

## 2026-02-26-deep-hub-ui-improvements.md

# Deep Hub UI Improvements — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Overhaul The Deep's Hub screen for better scannability, emotional engagement, and edge case handling based on team review.

**Architecture:** Changes are scoped to UI rendering files (`src/ui/deep_*.rs`), UI state (`src/deep/types.rs`), and minor integration points. No game logic changes. All rendering is read-only against `DeepState`.

**Tech Stack:** Rust, Ratatui, `scene_fx` buffer-based rendering

---

## Task 1: Tab System Improvements (types.rs + deep_scene.rs)

**Files:**
- Modify: `src/deep/types.rs` — Reorder `TABS` const, add `EventResponse` to TABS
- Modify: `src/ui/deep_scene.rs:252-372` — `render_tab_bar()` overflow handling

### Step 1: Reorder TABS in types.rs

In `src/deep/types.rs`, change the `TABS` const (line ~1011):

```rust
pub const TABS: &[DeepView] = &[
    DeepView::Hub,
    DeepView::EventResponse,  // moved to 2nd — most time-critical
    DeepView::NewMission,
    DeepView::Roster,
    DeepView::Recruit,
    DeepView::Infrastructure,
];
```

Update `tab_label()` — rename "Event" to "Events":
```rust
DeepView::EventResponse => "Events",
```

### Step 2: Tab bar overflow handling in deep_scene.rs

In `render_tab_bar()` (line ~252), after computing all tab labels, check if total width exceeds available space. If so, use abbreviated labels:

```rust
// After computing total tab width, check overflow
let total_tab_width: usize = DeepView::TABS.iter().enumerate().map(|(i, tab)| {
    let label = tab.tab_label();
    let badge_len = /* compute badge length */;
    if i > 0 { 1 } else { 0 } + label.len() + badge_len + 2 // brackets
}).sum();

let use_abbrev = total_tab_width + 2 > width;

// Abbreviated labels
fn abbrev_label(view: DeepView) -> &'static str {
    match view {
        DeepView::Hub => "H",
        DeepView::EventResponse => "Evt",
        DeepView::NewMission => "Msn",
        DeepView::Roster => "Rst",
        DeepView::Recruit => "Rec",
        DeepView::Infrastructure => "Lyr",
    }
}
```

When `use_abbrev` is true, use `abbrev_label()` and drop badge counts (keep symbols only).

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): reorder tabs, add overflow abbreviation
```

---

## Task 2: Persistent Status Summary Bar (deep_scene.rs)

**Files:**
- Modify: `src/ui/deep_scene.rs:377-489` — `render_deep_overlay()`

### Step 1: Add status bar rendering

In `render_deep_overlay()`, after painting the backdrop and before rendering the tab bar, render a one-line status summary at row 0. Push the tab bar down to row 1, separator to row 2, and content starts at row 3.

The status bar shows:
```
⬡ {rank_name}   ◆ {marks} Marks   {active}/{max} missions   Next: ~{time}
```

Logic:
- Rank display name from `deep.persistent.guild_rank.display_name()`
- Marks from `deep.prestige.warband_marks`
- Active/max from `deep.prestige.active_mission_count()` / `effective_concurrent_missions()`
- "Next complete" = minimum remaining time across active missions, or "None active"

Colors: Rank in white, Marks in amber RGB(220,180,60), mission count in cyan, time in DarkGray.

### Step 2: Adjust content offset

Change `content_buffer = &mut buffer[2..]` to `&mut buffer[3..]` and update `content_height` accordingly.

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): add persistent status summary bar above tabs
```

---

## Task 3: Hub Mission List Redesign (deep_missions.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs:266-797` — `render_hub()` rewrite

### Step 1: Labeled section separators

Replace flat `─` separators with labeled rules:
```rust
fn render_section_rule(buffer: &mut [Vec<SceneCell>], row: i32, width: usize, label: &str, count: Option<usize>) {
    let count_str = count.map(|c| format!(" ({})", c)).unwrap_or_default();
    let prefix = format!("── {} ", label);
    let suffix_len = count_str.len();
    let rule_len = width.saturating_sub(prefix.len() + suffix_len + 2);
    let rule: String = "─".repeat(rule_len);
    put_text(buffer, row, 1, &prefix, SECTION_LABEL_COLOR);
    put_text(buffer, row, 1 + prefix.len() as i32, &rule, Color::Rgb(40, 60, 80));
    if !count_str.is_empty() {
        put_text(buffer, row, (width as i32 - suffix_len as i32 - 1), &count_str, Color::DarkGray);
    }
}
```

### Step 2: Pre-attentive mission status glyphs

Replace the current mission rendering with glyph-prefixed 2-line cards:

Completed missions (pending_results):
```
[✓] {type} — Layer {n} {tier}      COLLECT → [Enter]
```

Active missions with events:
```
[!] {type} — Layer {n}   {lead_merc} leads    ⚡ EVENT
    ████████▒▒▒▒  {pct}%   ~{time} left
```

Active missions normal:
```
[▶] {type} — Layer {n}   {lead_merc} leads
    ████████▒▒▒▒  {pct}%   ~{time} left
```

Lead merc = first merc in squad by id lookup.

Sort order: completed first, then event-pending, then by time remaining ascending.

### Step 3: Completed vs active separator

Insert `render_section_rule()` between completed and active sections:
```
── COMPLETED ──────────────── (1)
[✓] ...

── ACTIVE ─────────────────── (2)
[▶] ...
```

### Step 4: Progress bar visual update

Change empty bar character from `░` (U+2591) to `▒` (U+2592) for mission progress bars. Keep `░` for familiarity bars elsewhere.

### Step 5: QA fix — "Resolving..." state

When `progress >= 1.0` and status is not `EventPending`:
```rust
if progress >= 1.0 && !matches!(mission.status, MissionStatus::EventPending) {
    // Show "Resolving..." instead of "0h 00m"
    put_text(buffer, row + 2, 3 + bar_width as i32, "  Resolving...", Color::Green);
}
```

### Step 6: QA fix — progress bar pulse at >95%

When progress > 0.95, alternate bar fill color between normal and brighter variant based on `millis`:
```rust
let bar_color = if progress > 0.95 {
    let pulse = (millis / 500) % 2 == 0;
    if pulse { Color::Rgb(120, 220, 160) } else { tc }
} else { tc };
```

### Step 7: Run `cargo test` and `cargo clippy`

### Step 8: Commit

```
feat(deep-ui): redesign Hub mission list with status glyphs and sections
```

---

## Task 4: Hub Empty State & Onboarding (deep_missions.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs:466-587` — empty state in `render_hub()`

### Step 1: Actionable empty state

Replace centered text tips with a structured action panel:

```rust
// When no active missions and no pending results
let mid = missions_top + remaining_space as i32 / 2;

// Show warband log first if available (keep existing)
// Then show action panel below:
put_text(buffer, action_row, 3, "[N] New Mission", Color::Rgb(80, 160, 220));
put_text(buffer, action_row, 20, "— Send your first squad", Color::DarkGray);
action_row += 1;
put_text(buffer, action_row, 3, "[R] Recruit", Color::Rgb(80, 160, 220));
put_text(buffer, action_row, 20, "— Hire mercenaries", Color::DarkGray);
action_row += 1;
put_text(buffer, action_row, 3, "[L] Layers", Color::Rgb(80, 160, 220));
put_text(buffer, action_row, 20, "— View explored territory", Color::DarkGray);
```

Context-sensitive: only show [N] if there are available missions, show "Supply Runs are free" if marks == 0.

### Step 2: QA fix — injured roster deadlock guidance

When all mercs are injured and no missions active:
```rust
if deep.prestige.roster.iter().all(|m| matches!(m.status, MercStatus::Injured { .. }))
    && deep.prestige.active_missions.is_empty()
{
    put_text(buffer, row, 1,
        "Your mercs are recovering. They'll be ready after the next mission resolves.",
        Color::Rgb(80, 80, 120));
}
```

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): actionable empty state with context-sensitive guidance
```

---

## Task 5: Amber Marks Color & Marks-to-Goal Display (deep_missions.rs + deep_roster.rs + deep_layers.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs` — all `Color::Yellow` for marks → `Color::Rgb(220, 180, 60)`
- Modify: `src/ui/deep_roster.rs` — same
- Modify: `src/ui/deep_layers.rs` — same
- Modify: `src/ui/deep_results.rs` — same

### Step 1: Define MARKS_COLOR constant

In `src/ui/deep_missions.rs`, add near the top:
```rust
/// Amber color for Warband Marks currency displays.
const MARKS_COLOR: Color = Color::Rgb(220, 180, 60);
```

Replace all `Color::Yellow` that specifically colors marks/costs with `MARKS_COLOR`. Leave non-marks Yellow uses (event badges, warnings) unchanged.

Export from deep_missions for use in sibling files:
```rust
pub(super) const MARKS_COLOR: Color = Color::Rgb(220, 180, 60);
```

### Step 2: Marks relative to next purchase (Hub only)

In the guild status block of `render_hub()`, replace raw marks display:

```rust
// Find cheapest affordable action
let cheapest_recruit = deep.prestige.recruit_pool.recruit_costs.iter().min().copied().unwrap_or(0);
let cheapest_infra = /* find cheapest unbuilt infra across frontier layers */;
let next_goal = if cheapest_recruit > 0 && marks < cheapest_recruit {
    format!("{} / {} — Next recruit", marks, cheapest_recruit)
} else if cheapest_infra > 0 && marks < cheapest_infra {
    format!("{} / {} — Next infrastructure", marks, cheapest_infra)
} else {
    format!("{}", marks)
};
```

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): amber marks color, marks-to-goal display
```

---

## Task 6: QA Fixes Bundle (types.rs + deep_scene.rs + deep_missions.rs)

**Files:**
- Modify: `src/deep/types.rs` — generation_number default
- Modify: `src/ui/deep_scene.rs` — badge visibility footer
- Modify: `src/ui/deep_missions.rs` — squad name overflow, mission card truncation

### Step 1: Gen.0 fix

In `render_compact_hub()` (deep_missions.rs line ~128):
```rust
let gen_label = format!("Gen.{}", deep.prestige.generation_number.max(1));
```

In `render_hub()` full layout (line ~343), same pattern — already shows conditionally for gen > 1, which is correct.

### Step 2: Squad name overflow

In the mission card rendering, cap squad string:
```rust
let max_squad_w = width.saturating_sub(12);
let squad_display = if squad_str.len() > max_squad_w {
    let first_name = squad_names.first().map(|s| s.as_str()).unwrap_or("");
    format!("{}, +{} others", first_name, squad_names.len() - 1)
} else {
    squad_str
};
```

### Step 3: Mission card truncation

In `render_new_mission_split()` (line ~1258), apply the already-computed `max_name_w`:
```rust
let display_name = &type_name[..type_name.len().min(max_name_w)];
```

### Step 4: Event badge footer reminder

In `render_deep_overlay()` or in each sub-view's footer, when not on Hub/EventResponse tab and events are pending:
```rust
if ui.view != DeepView::Hub && ui.view != DeepView::EventResponse
    && deep.prestige.has_any_pending_event()
{
    put_text(buffer, height - 1, width / 2, "⚡ Event pending — [Tab] to Hub", Color::Yellow);
}
```

### Step 5: Run `cargo test` and `cargo clippy`

### Step 6: Commit

```
fix(deep-ui): Gen.0, squad overflow, card truncation, event badge reminder
```

---

## Task 7: Border Pulse Animation (deep_scene.rs)

**Files:**
- Modify: `src/ui/deep_scene.rs:377-402` — `render_deep_overlay()` border rendering

### Step 1: Slow pulse on border color

Replace the static `DEEP_BORDER_COLOR` with a pulsing variant:

```rust
fn pulsing_border_color(millis: u128) -> Color {
    // 5-second cycle (5000ms), smooth sine wave
    let phase = (millis as f64 / 5000.0 * std::f64::consts::TAU).sin();
    let t = (phase * 0.5 + 0.5) as f64; // 0.0 to 1.0
    let r = (60.0 + t * 40.0) as u8;  // 60-100
    let g = (120.0 + t * 70.0) as u8; // 120-190
    let b = (180.0 + t * 75.0) as u8; // 180-255
    Color::Rgb(r, g, b)
}
```

Apply in `render_deep_overlay()`:
```rust
let border_color = pulsing_border_color(current_millis());
let block = Block::default()
    .title(" THE DEEP ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(super::themed_border_color(border_color)));
```

### Step 2: Run `cargo test` and `cargo clippy`

### Step 3: Commit

```
feat(deep-ui): slow breathing border pulse animation
```

---

## Task 8: Lead Merc Names + Debrief Flavor (deep_missions.rs + deep_results.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs` — add lead merc name to mission cards
- Modify: `src/ui/deep_results.rs` — add flavor text to debrief

### Step 1: Lead merc name helper

In `deep_missions.rs`, add helper:
```rust
/// Get the lead merc's name (first squad member) for display.
fn lead_merc_name(deep: &DeepState, squad: &[u64]) -> String {
    squad.first()
        .and_then(|id| deep.prestige.find_merc(*id))
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}
```

Use in mission card rendering:
```rust
let leader = lead_merc_name(deep, &mission.squad);
// Line 1: [▶] Supply Run — Layer 3   Varek leads
```

### Step 2: Debrief flavor text

In `deep_results.rs`, add tier-based procedural flavor after the rewards section:

```rust
fn debrief_flavor(tier: LayerTier, outcome: MissionOutcome) -> &'static str {
    match (tier, outcome) {
        (LayerTier::Shallows, MissionOutcome::Success) => "The squad reports stable tunnels and breathable air.",
        (LayerTier::Shallows, _) => "The upper passages proved more treacherous than expected.",
        (LayerTier::Warrens, MissionOutcome::Success) => "The Warrens yielded their secrets reluctantly.",
        (LayerTier::Warrens, _) => "Something in the Warrens was waiting for them.",
        (LayerTier::Hollows, MissionOutcome::Success) => "The bioluminescence guided them deeper than planned.",
        (LayerTier::Hollows, _) => "The spore clouds were thicker than the maps suggested.",
        (LayerTier::SunkenReach, MissionOutcome::Success) => "The seals parted for them. Not everyone finds that reassuring.",
        (LayerTier::SunkenReach, _) => "The water pressure claimed equipment. And patience.",
        (LayerTier::Abyss, MissionOutcome::Success) => "They returned with six days of rations consumed. They were gone four hours.",
        (LayerTier::Abyss, _) => "Time moved differently down there. It always does.",
        (LayerTier::Void, MissionOutcome::Success) => "What they found cannot be mapped. Only remembered.",
        (LayerTier::Void, _) => "The Void gives nothing freely. The cost is always personal.",
    }
}
```

Add this flavor line to the mission results modal after the rewards section, styled italic in DarkGray.

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): lead merc names on missions, debrief flavor text
```

---

## Execution Order

Tasks 1-2 must go first (tab system + status bar change the layout offset).
Tasks 3-8 are independent after that and can be parallelized.

Suggested serial order if not parallelizing:
1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

## Verification

After all tasks:
```bash
make check   # Full CI: fmt, clippy, test, build, audit
cargo run    # Visual verification with --debug flag to trigger Deep
```

## 2026-03-02-deep-panel-design.md

# Unified Deep Panel Design

**Date**: 2026-03-02 (updated 2026-03-03)
**Replaces**: Power Cores panel (`draw_power_cores_panel()` in `src/ui/stats_prestige.rs`)

## Overview

Merge the Power Cores panel and Deep status into a single unified panel titled "The Deep" in the stats column. Same 8-row height as the current Power Cores panel (6 content rows + 2 border rows). Features mission progress bar and aggregate core progress bar.

## Layout

```
┌─ The Deep ───────────────────────────────────┐
│ Row 1: Guild rank + Warband Marks            │
│ Row 2: Missions N/M [progress bar] ~ETA  ⚡N  │
│ Row 3: Crew glyphs + Frontier                │
│ Row 4: ─────────── separator ───────────────│
│ Row 5: Cores [aggregate bar] ~ETA    +N PR/d │
│ Row 6: Per-core rate·status pairs            │
└──────────────────────────────────────────────┘
```

## Row Details

### Row 1: Guild Rank + Currency
```
│ ⬡ Company        ◆ 1250 Warband Marks       │
```
- Left: `⬡` hex icon (White) + guild rank name
- Right-aligned: `◆` (Amber) + Warband Marks count

### Row 2: Missions + Progress Bar + Events
```
│ Missions 2/2 [████████░░░░] ~45m     ⚡1     │
```
- Left: `Missions N/M` (Cyan)
- Center: 12-char progress bar showing nearest mission completion
  - Fill: `█` (Amber), empty: `░` (DarkGray), brackets in DarkGray
  - Progress = nearest active mission's elapsed / total duration
  - Only shown when active missions exist
- After bar: `~Xh Ym` time remaining (Yellow <15m, DarkGray otherwise)
- Right-aligned: `⚡N` in Yellow (omit if 0)
- No active missions: `Missions 0/N     ◷ idle` (no bar)

### Row 3: Crew + Frontier
```
│ ♦♦♦ ♢ ✝          Frontier L7                 │
```
- `♦` Green = Available, `♢` Cyan = On Mission, `✝` Red = Injured
- Groups separated by spaces, Lost mercs skipped
- Right-aligned: `Frontier LN` in Gray-blue
- Empty roster: blank left side

### Row 4: Separator
```
│─────────────────────────────────────────────│
```

### Row 5: Aggregate Core Progress Bar
```
│ Cores [████████░░░░] ~2h 45m         +8 PR/d │
```
- `Cores` label in DarkGray
- 12-char progress bar showing the **soonest-to-complete** core's fill ratio
  - Fill: `█` Amber (or Green when all ready), empty: `░` DarkGray
- After bar: time until next PR grant via `format_eta()`
- Right-aligned: `+N PR/d` in Amber
- All unlocked cores ready: bar fully filled Green, `All ready!` in Green+Bold
- No cores unlocked: `Cores: locked    First core at L3`

### Row 6: Per-Core Rate·Status Pairs
```
│ 2·✓  3·✓  5·2h  ◇L18  ◇L25  ◇L30            │
```
- Each unlocked core: `{pr_per_day}·{status}`
  - PR rate number in Amber — this IS the core's identity and conveys speed
  - `·` separator in DarkGray
  - Ready: `✓` in Green
  - Filling: `Xh` or `Xm` in DarkGray
- Locked cores: `◇LN` in DarkGray
- Two spaces between each entry

## State Examples

### Discovery (no mercs, no cores)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 0 Warband Marks          │
│ Missions 0/1     ◷ idle                      │
│                   Frontier L1                │
│─────────────────────────────────────────────│
│ Cores: locked    First core at L3            │
│ ◇L3  ◇L7  ◇L12  ◇L18  ◇L25  ◇L30            │
└──────────────────────────────────────────────┘
```

### Early game (1 core filling, 1 mission)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Freelancers    ◆ 80 Warband Marks         │
│ Missions 1/1 [████░░░░░░░░] ~3h 15m         │
│ ♦♦                Frontier L3                │
│─────────────────────────────────────────────│
│ Cores [██░░░░░░░░░░] ~4h 30m         +2 PR/d │
│ 2·4h  ◇L7  ◇L12  ◇L18  ◇L25  ◇L30           │
└──────────────────────────────────────────────┘
```

### Mid game (3 cores, mixed crew, events)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Company        ◆ 1250 Warband Marks       │
│ Missions 2/2 [████████░░░░] ~45m     ⚡1     │
│ ♦♦♦ ♢ ✝          Frontier L7                 │
│─────────────────────────────────────────────│
│ Cores [████████░░░░] ~2h 45m         +8 PR/d │
│ 2·✓  3·✓  5·2h  ◇L18  ◇L25  ◇L30            │
└──────────────────────────────────────────────┘
```

### Late game (all 6 cores, large roster)
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 8200 Warband Marks       │
│ Missions 2/4 [██████████░░] ~1h 20m          │
│ ♦♦♦♦ ♢♢ ✝✝       Frontier L30               │
│─────────────────────────────────────────────│
│ Cores [████████░░░░] ~1h 05m        +48 PR/d │
│ 2·✓  3·1h  5·3h  8·5h  12·8h  18·11h         │
└──────────────────────────────────────────────┘
```

### All cores ready, no missions
```
┌─ The Deep ───────────────────────────────────┐
│ ⬡ Vanguard ★     ◆ 12000 Warband Marks      │
│ Missions 0/4     ◷ idle                      │
│ ♦♦♦♦♦♦♦♦          Frontier L30               │
│─────────────────────────────────────────────│
│ Cores [████████████] All ready!     +48 PR/d │
│ 2·✓  3·✓  5·✓  8·✓  12·✓  18·✓               │
└──────────────────────────────────────────────┘
```

## Color Reference

| Element | Color | Notes |
|---------|-------|-------|
| `⬡` guild rank icon | White | |
| `◆` Warband Marks | Amber `Rgb(220,180,60)` | |
| `Missions N/M` | Cyan | |
| Mission bar fill `█` | Amber `Rgb(255,165,0)` | |
| Mission bar empty `░` | DarkGray | |
| Mission ETA text | Yellow (<15m), DarkGray (otherwise) | |
| `◷ idle` | DarkGray | |
| `♦` available merc | Green | |
| `♢` deployed merc | Cyan | |
| `✝` injured merc | Red | |
| Frontier label | Gray-blue `Rgb(120,140,170)` | |
| `⚡` events | Yellow | |
| `Cores` label | DarkGray | |
| Core bar fill (filling) `█` | Amber `Rgb(255,165,0)` | |
| Core bar fill (all ready) `█` | Green | |
| Core bar empty `░` | DarkGray | |
| Core ETA text | DarkGray | |
| `All ready!` | Green + Bold | |
| PR rate number | Amber `Rgb(255,165,0)` | |
| `·` separator | DarkGray | |
| `✓` ready | Green | |
| Core time remaining | DarkGray | |
| `◇` locked core | DarkGray | |
| `+N PR/d` | Amber `Rgb(255,165,0)` | |
| Panel border | Amber (themed) | |
| Separator | DarkGray | |

## Implementation Notes

- Panel function: `draw_deep_panel()` in `src/ui/stats_prestige.rs`
- Panel title: `" The Deep "`
- Height: `8` rows (6 content + 2 border)
- Visibility: `deep.persistent.discovered`
- Mission bar: 12 chars, uses nearest active mission's `progress(Utc::now())`
- Core aggregate bar: 12 chars, uses `next_ready_ratio` from `CoreSummary`
- Core rate labels: PR rate as identity (2, 3, 5, 8, 12, 18) — inherently conveys speed
- Crew glyphs: one per merc, grouped by MercStatus, Lost skipped

## deep-balance-design.md

# The Deep — Balance Design

Concrete numbers for layer progression, economy, mercenary stats, and reward scaling. All values are tuned to support the walkthroughs in issue #362 and align with Quest's existing balance constants in `src/core/constants.rs`.

---

## 1. Layer Difficulty Curves

Each layer has a **Power threshold** — the minimum total squad Power for a comfortable Breakthrough. Squads below the threshold risk partial success or failure. Squads above it gain faster completion and better outcomes.

### Power Thresholds by Layer

| Layer | Tier | Power Threshold (Breakthrough) | Expedition Min | Recon Min | Supply Run Min |
|-------|------|-------------------------------|----------------|-----------|----------------|
| 1 | Shallows | 25 | 20 | 15 | 10 |
| 2 | Shallows | 40 | 30 | 20 | 15 |
| 3 | Shallows | 55 | 40 | 30 | 20 |
| 4 | Warrens | 75 | 55 | 40 | 25 |
| 5 | Warrens | 95 | 70 | 50 | 30 |
| 6 | Warrens | 115 | 85 | 60 | 40 |
| 7 | Warrens | 130 | 100 | 75 | 50 |
| 8 | Hollows | 155 | 115 | 85 | 55 |
| 9 | Hollows | 180 | 135 | 100 | 65 |
| 10 | Hollows | 205 | 155 | 115 | 75 |
| 11 | Hollows | 230 | 175 | 130 | 85 |
| 12 | Hollows | 260 | 195 | 145 | 95 |
| 13 | Sunken Reach | 295 | 220 | 165 | 110 |
| 14 | Sunken Reach | 330 | 250 | 185 | 125 |
| 15 | Sunken Reach | 370 | 280 | 210 | 140 |
| 16 | Sunken Reach | 410 | 310 | 230 | 155 |
| 17 | Sunken Reach | 450 | 340 | 255 | 170 |
| 18 | Sunken Reach | 495 | 370 | 280 | 185 |
| 19 | Abyss | 545 | 410 | 310 | 205 |
| 20 | Abyss | 600 | 450 | 340 | 225 |
| 21 | Abyss | 660 | 495 | 370 | 250 |
| 22 | Abyss | 720 | 540 | 405 | 275 |
| 23 | Abyss | 785 | 590 | 440 | 300 |
| 24 | Abyss | 855 | 640 | 480 | 325 |
| 25 | Abyss | 930 | 700 | 525 | 350 |
| 26+ | Void | 930 + 80*(L-25) | 700 + 60*(L-25) | 525 + 45*(L-25) | 350 + 30*(L-25) |

**Scaling rationale**: Shallows layers scale by ~15 Power per layer. Warrens by ~18. Hollows by ~25. Sunken Reach by ~33. Abyss by ~55. Void scales linearly at +80/layer (infinite endgame wall, analogous to Zone 11).

---

## 2. Mission Durations

Base durations before any modifiers (infrastructure, familiarity). All values in hours of real wall-clock time.

### Base Durations by Mission Type and Tier

| Layer Tier | Supply Run | Recon | Expedition | Breakthrough | Construction |
|------------|-----------|-------|------------|--------------|--------------|
| Shallows (1-3) | 2.0h | 4.0h | 8.0h | 18.0h | 4.0h |
| Warrens (4-7) | 2.5h | 5.0h | 10.0h | 20.0h | 5.0h |
| Hollows (8-12) | 3.0h | 6.0h | 12.0h | 22.0h | 6.0h |
| Sunken Reach (13-18) | 3.5h | 7.0h | 14.0h | 24.0h | 7.0h |
| Abyss (19-25) | 4.0h | 8.0h | 16.0h | 24.0h | 8.0h |
| Void (26+) | 4.0h | 8.0h | 16.0h | 24.0h | 8.0h |

### Duration Modifiers (multiplicative, stacking)

| Source | Reduction | Notes |
|--------|-----------|-------|
| Outpost infrastructure | -25% | Per-layer, permanent |
| Familiarity 25-49% (Mapped) | -10% | |
| Familiarity 50-74% (Familiar) | -20% | |
| Familiarity 75-100% (Mastered) | -30% | |
| Saboteur in squad | -10% to -15% | -10% base, -15% at Lv10+ |
| Overpowered squad (>150% threshold) | -10% | Cap at -10% |

**Example**: Supply Run on Layer 4 with Outpost + 75% Familiarity + Saboteur Lv10:
- Base: 2.5h
- After Outpost: 2.5 * 0.75 = 1.875h
- After Familiarity: 1.875 * 0.70 = 1.3125h
- After Saboteur: 1.3125 * 0.85 = 1.116h (~1h 7m)

**Minimum duration floor**: No mission can drop below 30 minutes. This prevents degenerate speed loops.

---

## 3. Warband Marks Economy

### Earning Rates by Layer and Mission Type

Mark rewards scale by layer. Formula: `base_marks * (1 + 0.08 * (layer - 1))` rounded to nearest 5.

| Layer | Supply Run | Recon | Expedition | Breakthrough |
|-------|-----------|-------|------------|--------------|
| 1 | 35 | 50 | 130 | 280 |
| 2 | 40 | 55 | 140 | 300 |
| 3 | 40 | 60 | 155 | 320 |
| 4 | 45 | 65 | 170 | 345 |
| 5 | 50 | 70 | 185 | 370 |
| 6 | 55 | 80 | 200 | 395 |
| 7 | 60 | 85 | 215 | 420 |
| 8 | 65 | 95 | 235 | 450 |
| 9 | 70 | 100 | 255 | 480 |
| 10 | 75 | 110 | 275 | 510 |
| 11 | 80 | 115 | 295 | 540 |
| 12 | 90 | 125 | 315 | 570 |
| 13 | 95 | 135 | 340 | 600 |
| 14 | 100 | 145 | 365 | 635 |
| 15 | 110 | 155 | 390 | 670 |
| 16 | 115 | 165 | 415 | 705 |
| 17 | 125 | 175 | 440 | 740 |
| 18 | 130 | 185 | 470 | 780 |
| 19 | 140 | 200 | 500 | 820 |
| 20 | 150 | 210 | 530 | 860 |
| 21 | 160 | 225 | 560 | 900 |
| 22 | 170 | 235 | 590 | 940 |
| 23 | 180 | 250 | 625 | 985 |
| 24 | 190 | 265 | 660 | 1030 |
| 25 | 200 | 280 | 695 | 1075 |
| 26+ | 200 + 10*(L-25) | 280 + 15*(L-25) | 695 + 35*(L-25) | 1075 + 50*(L-25) |

**Reward variance**: Actual rewards have +/- 15% random variance. A Layer 1 Supply Run pays 30-42 Marks (center 35).

**Partial success**: 60% of full rewards.
**Failure**: 20% of full rewards.

### Earning Modifiers

| Source | Bonus | Notes |
|--------|-------|-------|
| Supply Cache infrastructure | +50% Marks | Per-layer supply runs only |
| Familiarity 75-100% | +15% Marks | Bonus yield on Mastered layers |
| Full Success | 100% | Standard |
| Partial Success | 60% | |
| Failure | 20% | |

### Spending Sinks

| Cost | Marks | Notes |
|------|-------|-------|
| Recruit merc (Common quality) | 30-50 | Random within range per candidate |
| Recruit merc (Uncommon quality) | 50-80 | Better base stats |
| Recruit merc (Rare quality) | 80-120 | Best stats, rarer archetypes |
| Launch Supply Run | 0 (free daily) or 15-25 | One free per day on any cleared layer |
| Launch Recon | 30-50 | Scales slightly with layer: 30 + layer |
| Launch Expedition | 80-150 | Scales: 80 + 3*layer |
| Launch Breakthrough | 150-350 | Scales: 150 + 8*layer |
| Build Outpost | 60 + 4*layer | Layer 1: 64, Layer 10: 100, Layer 20: 140 |
| Build Supply Cache | 80 + 5*layer | Layer 1: 85, Layer 10: 130, Layer 20: 180 |
| Build Watchtower | 70 + 4*layer | Layer 1: 74, Layer 10: 110, Layer 20: 150 |
| Build Bridge | 100 + 5*layer | Layer 1: 105, Layer 10: 150, Layer 20: 200 |
| Guild Rank 2 | 200 | One-time, persists |
| Guild Rank 3 | 500 | One-time, persists |
| Guild Rank 4 | 1,200 | One-time, persists |
| Guild Rank 5 | 3,000 | One-time, persists |

### Economy Flow Validation

**Day 1 (fresh P15, Generation 1):**
- Free Supply Run L1: +35 Marks (2h)
- Recon L1: -30, +50 Marks net +20 (4h)
- End of Day 1: ~55 Marks, 1 cleared layer

**Week 1 end (Generation 1):**
- ~480 Marks accumulated
- Layers 1-2 cleared, Layer 3 frontier
- 1-2 infrastructure buildings
- 4/5 mercs

**Mid-game (Generation 3, Rank 3, Layers 1-8 cleared):**
- Daily passive from supply circuit (2 slots): ~200-300 Marks/day
- Cost to push 1 frontier layer: ~250-400 Marks (Recon + Expedition + Breakthrough)
- Net progress: ~1 layer every 2-3 days

**Endgame (Generation 8+, Rank 5, Layers 1-22 cleared):**
- Daily passive from 4-slot supply circuit: ~600-1000 Marks/day
- Cost to push 1 frontier layer: ~600-900 Marks
- Net progress: ~1 layer every 1-2 days

---

## 4. Mercenary Stat Curves

### Base Stats by Archetype (Level 1)

Each merc has 3 stats: Power, Resilience, Expertise. Base stats at Level 1 depend on archetype and guild rank quality tier.

#### Rank 1 (Freelancers) — Common Quality

| Archetype | Power | Resilience | Expertise | Total |
|-----------|-------|------------|-----------|-------|
| Vanguard | 12 | 14 | 8 | 34 |
| Scout | 8 | 10 | 14 | 32 |
| Arcanist | 10 | 8 | 14 | 32 |
| Medic | 6 | 12 | 12 | 30 |
| Saboteur | 9 | 9 | 14 | 32 |

#### Rank 2 (Sellswords) — Common/Uncommon Quality

Base stats increase by +2 per stat compared to Rank 1 pool averages. Uncommon recruits get an additional +2 to their primary stats.

| Quality | Power Range | Resilience Range | Expertise Range |
|---------|------------|------------------|-----------------|
| Common | +2 over Rank 1 bases | +2 | +2 |
| Uncommon | +4 to primary, +2 to others | +4 to primary, +2 to others | +4 to primary, +2 to others |

#### Rank 3 (Company) — Uncommon/Rare Quality

+4 per stat over Rank 1 bases. Rare recruits get +6 to primary stats.

#### Rank 4 (Battalion) — Rare Quality Standard

+6 per stat over Rank 1 bases. Rare recruits get +8 to primary stats.

#### Rank 5 (Legion) — Rare/Elite Quality

+8 per stat over Rank 1 bases. Elite recruits get +12 to primary stats.

**Summary: Level 1 Stats by Rank (Vanguard archetype as example)**

| Rank | Quality | Power | Resilience | Expertise |
|------|---------|-------|------------|-----------|
| 1 | Common | 12 | 14 | 8 |
| 2 | Common | 14 | 16 | 10 |
| 2 | Uncommon | 16 | 18 | 10 |
| 3 | Uncommon | 16 | 18 | 12 |
| 3 | Rare | 18 | 20 | 12 |
| 4 | Rare | 18 | 20 | 14 |
| 4 | Rare+ | 20 | 22 | 14 |
| 5 | Rare | 20 | 22 | 16 |
| 5 | Elite | 24 | 26 | 16 |

### Level Scaling (1-20)

Stats grow per level based on archetype weights. Growth is **not** linear — early levels grow fast, later levels taper.

**Per-level stat growth formula**: `stat_at_level = base + growth_per_level * (level - 1)`

| Archetype | Power/Lvl | Resilience/Lvl | Expertise/Lvl |
|-----------|-----------|----------------|---------------|
| Vanguard | +4.0 | +3.5 | +2.0 |
| Scout | +3.0 | +3.0 | +3.5 |
| Arcanist | +3.5 | +2.0 | +4.0 |
| Medic | +2.0 | +3.5 | +3.0 |
| Saboteur | +3.0 | +2.5 | +4.0 |

**Example: Rank 1 Vanguard Level Progression**

| Level | Power | Resilience | Expertise | Total |
|-------|-------|------------|-----------|-------|
| 1 | 12 | 14 | 8 | 34 |
| 2 | 16 | 18 | 10 | 44 |
| 3 | 20 | 21 | 12 | 53 |
| 4 | 24 | 25 | 14 | 63 |
| 5 | 28 | 28 | 16 | 72 |
| 8 | 40 | 39 | 22 | 101 |
| 10 | 48 | 46 | 26 | 120 |
| 12 | 56 | 53 | 30 | 139 |
| 15 | 68 | 63 | 36 | 167 |
| 18 | 80 | 74 | 42 | 196 |
| 20 | 88 | 81 | 46 | 215 |

### Merc XP and Leveling

Mercs gain XP from completing missions. XP per mission scales with mission type and layer.

| Mission Type | Base XP | Layer Scaling |
|-------------|---------|---------------|
| Supply Run | 100 | +10 per layer |
| Recon | 200 | +20 per layer |
| Expedition | 400 | +40 per layer |
| Breakthrough | 800 | +80 per layer |
| Construction | 50 | Flat (no scaling) |

**XP to next level**: `200 * level^1.3` (same curve shape as main game, different scale)

| Level | XP Required | Cumulative XP |
|-------|------------|---------------|
| 1->2 | 200 | 200 |
| 2->3 | 492 | 692 |
| 3->4 | 851 | 1,543 |
| 4->5 | 1,262 | 2,805 |
| 5->6 | 1,716 | 4,521 |
| 8->9 | 3,436 | 15,074 |
| 10->11 | 4,689 | 24,226 |
| 14->15 | 8,281 | 56,610 |
| 18->19 | 12,583 | 104,258 |
| 19->20 | 13,869 | 118,127 |

**Leveling pace**: A merc running Supply Runs on Layer 5 (~150 XP per run, ~3h each) takes roughly:
- Level 1->5: ~6-7 supply runs (~20h)
- Level 5->10: ~20 supply runs (~60h, ~3 days)
- Level 10->15: ~50 supply runs (~150h, ~7 days)
- Level 15->20: ~100+ supply runs (~300h+, unlikely before prestige)

This ensures merc leveling is meaningful but not the primary bottleneck. Most mercs reach Level 8-12 in a typical generation. Level 15+ requires intentional investment. Level 20 is exceptional.

### Squad Power Calculation

Total squad Power = sum of individual merc Power stats.

**Comfortable margin**: Squad Power >= 110% of threshold. Good auto-resolve odds.
**Tight margin**: Squad Power 100-110% of threshold. 65-75% full success chance.
**Underpowered**: Squad Power < 100% of threshold. Risk of partial success or failure.

---

## 5. Familiarity System

### Familiarity Gain Per Mission

| Mission Type | Familiarity Gain | Notes |
|-------------|-----------------|-------|
| Supply Run | +5% | Slow but safe |
| Recon | +15% | Primary familiarity builder |
| Expedition | +10% | Secondary gain |
| Breakthrough | +15% | One-time per layer |
| Construction | +5% | Small bonus |

**Watchtower infrastructure**: Grants +25% Familiarity immediately on construction.

### Familiarity Thresholds

| Range | Status | Mission Duration | Auto-Resolve Quality | Mark Bonus |
|-------|--------|-----------------|---------------------|------------|
| 0-24% | Unknown | Base | Poor (65% safe option success) | None |
| 25-49% | Mapped | -10% | Fair (75%) | None |
| 50-74% | Familiar | -20% | Good (85%) | None |
| 75-100% | Mastered | -30% | Excellent (95%) | +15% |

### Familiarity Persistence

Familiarity persists across prestiges. It never decreases. This means:
- Generation 1: Layer 1 reaches ~70% Familiarity through normal play
- Generation 2: Layer 1 starts at 70%, reaches 85%+ with a few more missions
- Generation 3+: Layer 1 is at 95-100%, missions are lightning fast

**Cap**: 100%. Beyond 100% has no additional effect.

---

## 6. Infrastructure ROI

### Cost and Effect Summary

| Infrastructure | Cost Formula | Effect | Persists |
|---------------|-------------|--------|----------|
| Outpost | 60 + 4*layer | -25% mission duration this layer | Yes |
| Supply Cache | 80 + 5*layer | +50% Supply Run Marks this layer | Yes |
| Watchtower | 70 + 4*layer | +25 Familiarity, better auto-resolve | Yes |
| Bridge | 100 + 5*layer | -2h on missions transiting through this layer | Yes |

### Supply Cache ROI Analysis

The Supply Cache is the primary income-generating infrastructure. ROI = cost / extra marks per run.

| Layer | Cache Cost | Base Supply Run Marks | +50% Bonus | Extra/Run | Runs to Break Even |
|-------|-----------|----------------------|-----------|-----------|-------------------|
| 1 | 85 | 35 | 52 | 17 | 5.0 |
| 3 | 95 | 40 | 60 | 20 | 4.8 |
| 5 | 105 | 50 | 75 | 25 | 4.2 |
| 8 | 120 | 65 | 97 | 32 | 3.8 |
| 10 | 130 | 75 | 112 | 37 | 3.5 |
| 12 | 140 | 90 | 135 | 45 | 3.1 |
| 15 | 155 | 110 | 165 | 55 | 2.8 |
| 20 | 180 | 150 | 225 | 75 | 2.4 |

**Key insight**: Supply Caches pay for themselves in 3-5 runs (6-15 hours of wall time depending on layer and speed modifiers). After that, they generate pure profit forever. This is the core of the infrastructure ratchet.

### Outpost ROI Analysis

The Outpost saves time, not Marks directly. Value measured in time saved per mission.

| Layer | Outpost Cost | Supply Run Base Duration | -25% Savings | Missions to Save 1h |
|-------|-------------|-------------------------|-------------|---------------------|
| 1 | 64 | 2.0h | 0.5h | 2 |
| 5 | 80 | 2.5h | 0.625h | 1.6 |
| 10 | 100 | 3.0h | 0.75h | 1.3 |
| 15 | 120 | 3.5h | 0.875h | 1.1 |
| 20 | 140 | 4.0h | 1.0h | 1.0 |

**Practical value**: Time savings compound with Supply Cache. An Outpost + Supply Cache layer generates Marks per hour at ~2x the rate of a raw layer. Both together cost ~145-320 Marks depending on layer but generate permanent compounding returns.

### Optimal Build Orders

**Economy-first (recommended for new players):**
1. Supply Cache on Layer 1 (best ROI early)
2. Outpost on Layer 1 (speed up the earning)
3. Supply Cache on Layer 3-4 (second income source)
4. Push frontier to Layer 7 for Guild Rank 3
5. Supply Cache on highest cleared layer
6. Repeat: cache on best supply layer, push frontier

**Speed-first (for experienced players):**
1. Outpost on Layer 1 (fast runs)
2. Bridge on Layer 2 (shortcut for deep missions)
3. Push frontier aggressively
4. Backfill Supply Caches after reaching Rank 3

**Balance (hybrid):**
1. Supply Cache on Layer 1
2. Push to Layer 3
3. Outpost + Supply Cache on Layer 3
4. Push to Layer 7, buy Rank 3
5. Fill in Supply Caches on Layers 4-7
6. Use 2 slots to run supply + push simultaneously

---

## 7. Guild Rank Costs and Requirements

| Rank | Name | Mark Cost | Layer Requirement | Max Roster | Concurrent Missions | Recruit Quality |
|------|------|-----------|-------------------|------------|---------------------|-----------------|
| 1 | Freelancers | Free | Discovery | 5 | 1 | Common |
| 2 | Sellswords | 200 | Layer 3 cleared | 7 | 1 | Common + Uncommon |
| 3 | Company | 500 | Layer 7 cleared | 9 | 2 | Uncommon + Rare |
| 4 | Battalion | 1,200 | Layer 13 cleared | 12 | 3 | Rare |
| 5 | Legion | 3,000 | Layer 19 cleared | 15 | 4 | Rare + Elite |

### Rank Upgrade Timeline (expected)

| Rank | Earliest Generation | Typical Day in Generation | Cumulative Marks Earned |
|------|--------------------|--------------------------|-----------------------|
| 2 | Gen 1 | Day 5-8 | ~500 |
| 3 | Gen 1-2 | Day 12-18 | ~2,000 |
| 4 | Gen 3-4 | Day 8-14 (of that gen) | ~5,000 |
| 5 | Gen 5-6 | Day 10-15 (of that gen) | ~12,000 |

### Guild Rank Effects on Recruitment Pool

| Rank | Pool Size | Quality Distribution | Archetype Availability |
|------|-----------|---------------------|----------------------|
| 1 | 3 candidates | 100% Common | Vanguard, Scout, Medic only |
| 2 | 4 candidates | 60% Common, 40% Uncommon | + Arcanist |
| 3 | 4 candidates | 30% Common, 50% Uncommon, 20% Rare | + Saboteur |
| 4 | 5 candidates | 40% Uncommon, 50% Rare, 10% Elite | All archetypes |
| 5 | 5 candidates | 20% Uncommon, 50% Rare, 30% Elite | All archetypes |

**Daily pool refresh**: Pool refreshes every 24 hours (wall clock) or on prestige, whichever comes first. Unrecruited candidates are lost.

---

## 8. Injury and Loss Probability Curves

### Risk Levels by Mission Type

| Mission Type | Base Injury Chance | Base Loss Chance | Notes |
|-------------|-------------------|-----------------|-------|
| Supply Run (cleared) | 0% | 0% | Always safe |
| Recon (frontier) | 10% | 0% | Can injure, never lose |
| Expedition (frontier) | 20% | 2% | Moderate risk |
| Breakthrough (frontier) | 35% | 5% | Highest risk |
| Construction (cleared) | 0% | 0% | Always safe |

### Modifiers to Injury/Loss

| Factor | Injury Modifier | Loss Modifier | Notes |
|--------|----------------|---------------|-------|
| Underpowered squad (<100% threshold) | +15% | +5% | Significant penalty |
| Overpowered squad (>120% threshold) | -10% | -2% | Safety from strength |
| Medic in squad | -10% injury | Loss -> Injury downgrade | Medic prevents loss |
| Vanguard in squad | -5% all | -2% | Frontline protection |
| High Resilience (avg >50) | -5% | -1% | Experienced squads |
| Failed event choice | +10% | +3% | Cascading consequences |
| Partial Success outcome | +15% injury | +3% loss | Rough mission |
| Failure outcome | +25% injury | +8% loss | Very rough |

**Medic loss prevention**: When a merc would be lost, if a Medic is in the squad, the loss is downgraded to a severe injury (16h recovery) instead. The Medic ability triggers with probability `50% + Medic_Level * 2.5%` (max 100% at Level 20). This is the core reason Medics are valuable.

### Injury Duration

| Severity | Recovery Time | Trigger |
|----------|-------------|---------|
| Light | 4-8h | Standard injury roll |
| Moderate | 8-12h | Failed event + injury |
| Severe | 12-16h | Medic-prevented loss |

**Mid-mission injury**: A merc injured during a mission (from an event) operates at -20% Power for the remainder of the mission and then enters recovery for 8-12h after the mission completes.

---

## 9. Reward Scaling

### XP Rewards

XP from The Deep feeds into the main game's XP pool. Scaled to be meaningful but not dominant over the combat loop.

| Mission Type | Base XP | Layer Scaling | Layer 1 XP | Layer 10 XP | Layer 20 XP |
|-------------|---------|---------------|-----------|------------|------------|
| Supply Run | 150 | +20/layer | 170 | 350 | 550 |
| Recon | 300 | +35/layer | 335 | 650 | 1000 |
| Expedition | 600 | +60/layer | 660 | 1200 | 1800 |
| Breakthrough | 1200 | +120/layer | 1320 | 2400 | 3600 |

**Context**: The main combat loop awards 200-400 XP per kill (every ~4-5 seconds). A Layer 10 Expedition (12h) awards 1200 XP — equivalent to about 3-6 kills. The Deep's XP contribution is a nice bonus, not a replacement for combat. Its value is in the other rewards (Marks, items, Stormglass, PR fragments).

### Item Rewards

| Mission Type | Item Drop Chance | Max Rarity | Notes |
|-------------|-----------------|-----------|-------|
| Supply Run | 20% | Common (L1-7), Uncommon (L8+) | Low-quality drops |
| Recon | 30% | Uncommon | |
| Expedition | 60% | Rare (L1-12), Epic (L13+) | Primary item source |
| Breakthrough | 100% | Epic (L1-12), Legendary (L13+) | Guaranteed drop |

**Item level**: ilvl = layer * 10 (matching zone ilvl scaling: Zone 1 = ilvl 10, Layer 10 = ilvl 100).

**Abyssal equipment** (Layers 19+): 10% chance on Expedition, 25% chance on Breakthrough. These items have standard combat stats PLUS one Deep-specific affix:

| Abyssal Affix | Effect |
|--------------|--------|
| Expedition Haste | +5-15% mission speed (scales with ilvl) |
| Deep Harvest | +10-25% supply run yield |
| Abyssal Ward | +5-15% squad Resilience |
| Voidtouch | +5-15% squad Power |
| Cartographer's Insight | +5-10% Familiarity gain |

### Stormglass Rewards

Stormglass is earned from Expeditions and Breakthroughs only. Requires P15+ (same gate as Stormglass system).

| Mission Type | Stormglass | Layer Scaling |
|-------------|-----------|---------------|
| Supply Run | 0 | None |
| Recon | 0 | None |
| Expedition | 5 + floor(layer/3) | L1: 5, L10: 8, L20: 11 |
| Breakthrough | 10 + floor(layer/2) | L1: 10, L10: 15, L20: 20 |

### Prestige Rank Fragments

Breakthrough missions on Layers 8+ award fractional prestige ranks. This creates an alternate prestige income path.

| Layer Range | PR Fragment per Breakthrough | Notes |
|-------------|---------------------------|-------|
| 1-7 | 0 | Too shallow for PR |
| 8-12 | 0.25 PR | Modest contribution |
| 13-18 | 0.50 PR | Meaningful supplement |
| 19-25 | 0.75 PR | Significant alternative |
| 26+ | 1.00 PR | Full rank per breakthrough |

**Prestige rank fragments accumulate across Breakthroughs and are awarded as whole ranks when they reach integer values.** (e.g., four Layer 8-12 Breakthroughs = +1 PR)

---

## 10. Auto-Resolve Quality

When events are not manually resolved (player offline or 2h auto-resolve timer expires), the system picks the safest option. Auto-resolve quality determines how good the "safe" option is.

| Familiarity | Auto-Resolve Success Rate | Delay Penalty |
|-------------|--------------------------|---------------|
| 0-24% (Unknown) | 65% | +2h average |
| 25-49% (Mapped) | 75% | +1.5h average |
| 50-74% (Familiar) | 85% | +1h average |
| 75-100% (Mastered) | 95% | +0.5h average |

**Scout bonus**: +10% to auto-resolve success rate when a Scout is in the squad.
**Watchtower infrastructure**: +5% to auto-resolve success rate on that layer.

---

## 11. Discovery Constants

Following existing patterns from Haven and Soulforge:

```
DEEP_MIN_PRESTIGE_RANK: 15
DEEP_DISCOVERY_BASE_CHANCE: 0.000014  // Per tick, same as Haven/Soulforge
DEEP_DISCOVERY_RANK_BONUS: 0.000007  // Per rank above 15
```

At P15: `0.000014` per tick = ~71,400 ticks = ~7,140 seconds = ~2 hours average.
At P20: `0.000014 + 5*0.000007 = 0.000049` per tick = ~1.7 hours average.

---

## 12. Prestige Reset Behavior

### What Resets
- All mercenaries disbanded
- All active missions cancelled (partial rewards at 50% of earned-so-far)
- Warband Marks set to 0
- Merc levels reset (new recruits start at Lv1)

### What Persists
- Guild Rank (and all its benefits)
- All cleared layers (permanently cleared)
- All built infrastructure (2 slots per layer)
- All Familiarity levels (never decrease)

### Generation Startup Package
- 3 free starter mercs (quality based on current Guild Rank)
- 1 free Supply Run immediately available
- All cleared layers accessible for missions immediately

---

## 13. Balance Validation Scenarios

### Scenario A: Fresh Discovery (P15, Gen 1)
- Start: 0 Marks, 3 free mercs (Rank 1 Common), 0 layers cleared
- Day 1: Clear Layer 1, earn ~90 Marks
- Day 3: Layer 2 cleared, ~300 Marks, first infrastructure on L1
- Day 7: Layer 3 cleared, ~500 Marks, Guild Rank 2 purchased, 5-6 mercs
- Day 14: Layer 5-6, ~1000 cumulative Marks earned, approaching Rank 3

### Scenario B: Second Generation (P17, Gen 2, Rank 2, L1-5 cleared)
- Start: 0 Marks, 3 free mercs (Rank 2 quality), Layers 1-5 cleared with infrastructure
- Day 1: 200+ Marks from optimized supply runs, recruit 2 mercs
- Day 3: Back at Layer 5 frontier, 500 Marks
- Day 7: Layer 7 cleared, buy Rank 3 (500 Marks), 2 mission slots unlocked

### Scenario C: Endgame Push (P28, Gen 8, Rank 5, L1-21 cleared)
- Start: 0 Marks, 3 free mercs (Rank 5 Elite quality), deep infrastructure network
- Day 1: 500+ Marks from optimized 2-slot supply circuit
- Day 2: 9+ mercs recruited, 4 slots running, frontier L22 recon begins
- Day 5: Layer 22 Breakthrough attempted, back to pushing frontier
- Steady state: ~1 new layer every 2 days

### Key Balance Checks

1. **Supply Cache never outpaces frontier rewards**: Supply Run income on cleared layers should always be less than Breakthrough/Expedition rewards on frontier. This ensures pushing the frontier remains attractive.

2. **Guild Rank 3 is achievable in Gen 1-2**: The 2nd mission slot is the most impactful upgrade. Players should reach it within 2-3 weeks of real play.

3. **Losing a high-level merc stings but doesn't brick progress**: Even losing a Lv15+ merc, the player can recruit from a Rank 3+ pool with good base stats. Recovery time: 2-3 days to level a replacement to usefulness.

4. **Infrastructure investment always pays off within the same generation**: Supply Cache ROI is 3-5 runs. Even if you prestige soon after building, the infrastructure persists and pays dividends in every future generation.

5. **Marks-per-hour scales sub-linearly with layer depth**: Deeper layers pay more Marks but take longer. Marks/hour stays in the ~15-45 range across all layers, preventing any single layer from being strictly dominant for farming.

| Layer | Supply Run Marks | Duration (no mods) | Marks/Hour |
|-------|-----------------|-------------------|------------|
| 1 | 35 | 2.0h | 17.5 |
| 5 | 50 | 2.5h | 20.0 |
| 10 | 75 | 3.0h | 25.0 |
| 15 | 110 | 3.5h | 31.4 |
| 20 | 150 | 4.0h | 37.5 |
| 25 | 200 | 4.0h | 50.0 |

With Supply Cache (+50%) and infrastructure modifiers, optimized layers can reach 50-60 Marks/hour. This is intentional — it rewards infrastructure investment without breaking the economy.

## deep-events-design.md

# The Deep — Check-In Event System Design

## Overview

Check-in events are the primary interactive element during missions. They fire at scheduled progress milestones, present the player with 2-3 choices (some gated behind squad archetypes), and resolve with consequences that affect mission outcome. When the player doesn't respond within 2 hours, the safest option auto-resolves.

Events are the mechanism that transforms The Deep from a passive timer system into an active strategic game. They reward:
- **Attention**: Players who respond to events get better options than auto-resolve
- **Squad composition**: Having the right archetype unlocks superior choices
- **Risk assessment**: Risky choices can pay off with bonus rewards or cascade into worse outcomes
- **Layer knowledge**: Experienced players learn which archetypes matter for which layer tiers

## Event Data Structures

### EventTemplate

The core template defining a reusable event. Templates are parameterized by layer tier — the same structural event can appear across layers with different flavor text and scaled consequences.

```rust
/// Unique identifier for each event template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTemplateId {
    // Shallows (L1-3)
    FloodedPassage,
    UnstableStairway,
    CrumblingBridge,
    SubterraneanNest,
    // Warrens (L4-7)
    TunnelAmbush,
    PoisonGasVent,
    CollapsingTunnel,
    ForkInThePath,
    // Hollows (L8-12)
    ToxicSporeCloud,
    CrystalFormation,
    HollowStalkers,
    AbyssalEcho,
    // Sunken Reach (L13-18)
    FloodedVault,
    CorruptedArtifact,
    SunkenGuardian,
    TidalSurge,
    // Abyss (L19-25)
    AbyssalRift,
    VoidSwarm,
    RealityFracture,
    TheWhisperingDark,
}

/// A reusable event template. Instances are generated from these at mission creation.
pub struct EventTemplate {
    pub id: EventTemplateId,
    /// Which layer tier this event belongs to
    pub tier: LayerTier,
    /// Category for scheduling balance (avoid 3 combat events in a row)
    pub category: EventCategory,
    /// Display title
    pub title: &'static str,
    /// Flavor text (2-3 lines describing the situation)
    pub description: &'static str,
    /// The available choices
    pub choices: Vec<EventChoice>,
    /// Index into choices for the auto-resolve default (always the safest)
    pub auto_resolve_index: usize,
    /// Optional: if this event was created by a chain trigger, which chain
    pub chain_source: Option<EventChainId>,
    /// Optional: tags for chain triggers (e.g., "took_risk", "explored_side_path")
    pub tags: Vec<EventTag>,
}

/// Category of event, used for scheduling diversity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    /// Environmental hazard (spores, floods, gas, crystals)
    Hazard,
    /// Combat encounter (ambush, stalkers, guardians)
    Combat,
    /// Navigation obstacle (collapse, bridge, fork)
    Obstacle,
    /// Discovery opportunity (artifact, cache, rift)
    Discovery,
    /// Boss pre-fight (only on breakthrough missions, always last event)
    BossApproach,
}
```

### EventChoice

```rust
/// A single choice within an event
pub struct EventChoice {
    /// Display text for the choice
    pub label: String,
    /// Optional archetype gate — if Some, only available when squad has this archetype
    pub archetype_gate: Option<Archetype>,
    /// Whether this choice is the "risky" option
    pub is_risky: bool,
    /// Consequences of picking this choice
    pub outcome: EventOutcome,
    /// Optional: flavor text explaining why this archetype helps
    pub archetype_flavor: Option<&'static str>,
}

/// The consequences of an event choice
pub struct EventOutcome {
    /// Time added to mission duration (can be 0)
    pub time_delay_minutes: u32,
    /// Mark cost (supplies consumed, bribes, etc.)
    pub mark_cost: u32,
    /// Bonus marks earned (from looting, harvesting, etc.)
    pub bonus_marks: u32,
    /// Risk of injury to a squad member (0.0 = safe, 1.0 = guaranteed)
    pub injury_chance: f64,
    /// Risk of merc loss (0.0 = safe, only > 0 on frontier breakthroughs)
    pub loss_chance: f64,
    /// Effective Power modifier for the rest of the mission (e.g., 1.15 = +15%)
    pub power_modifier: f64,
    /// Whether this choice triggers a chain event later in the mission
    pub chain_trigger: Option<EventChainTrigger>,
    /// Flavor text describing the outcome
    pub result_text: &'static str,
    /// For risky choices: success chance (based on squad Power vs threshold)
    pub success_chance: Option<RiskCheck>,
}

/// A risky choice's success/failure resolution
pub struct RiskCheck {
    /// Base success chance (0.0-1.0)
    pub base_chance: f64,
    /// Power threshold — squad Power above this adds bonus chance
    pub power_threshold: u32,
    /// Bonus chance per Power point above threshold (e.g., 0.005 = +0.5% per point)
    pub power_scaling: f64,
    /// Outcome if the risk check succeeds
    pub success: RiskOutcome,
    /// Outcome if the risk check fails
    pub failure: RiskOutcome,
}

pub struct RiskOutcome {
    pub time_delay_minutes: u32,
    pub bonus_marks: u32,
    pub injury_chance: f64,
    pub power_modifier: f64,
    pub result_text: &'static str,
}
```

### EventInstance

```rust
/// A concrete event instance attached to a specific mission
pub struct EventInstance {
    /// Which template this was generated from
    pub template_id: EventTemplateId,
    /// Progress percentage at which this event fires (0.0-1.0)
    pub trigger_progress: f64,
    /// Current state of this event
    pub state: EventState,
    /// Wall-clock time when the event became pending (for auto-resolve countdown)
    pub pending_since: Option<DateTime<Utc>>,
    /// Which choice was made (set on resolution)
    pub chosen_index: Option<usize>,
    /// The outcome that was applied (set on resolution)
    pub applied_outcome: Option<AppliedOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventState {
    /// Not yet triggered (mission hasn't reached this progress point)
    Scheduled,
    /// Triggered and waiting for player response (2h countdown running)
    Pending,
    /// Player made a choice
    Resolved,
    /// Auto-resolved after 2h timeout
    AutoResolved,
}

/// Record of what actually happened when an event resolved
pub struct AppliedOutcome {
    pub choice_label: String,
    pub time_delay_added: u32,
    pub marks_spent: u32,
    pub marks_earned: u32,
    pub injury: Option<MercInjuryRecord>,
    pub was_auto_resolved: bool,
    pub risk_check_result: Option<bool>, // Some(true) = succeeded, Some(false) = failed
}
```

### Event Chains

```rust
/// Identifies a chain relationship between events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventChainId {
    /// Shallows: exploring a side cave in event 1 reveals treasure in event 3
    ShallowsSideCave,
    /// Warrens: taking the risky fork leads to an ambush OR a shortcut
    WarrensForkConsequence,
    /// Hollows: fighting stalkers alerts the boss, changing the boss approach event
    HollowsAlertedBoss,
    /// Sunken Reach: corrupted artifact energy can be channeled later
    SunkenReachCorruption,
    /// Abyss: harvesting void energy has cascading effects
    AbyssVoidHarvest,
}

/// Tags applied when a choice is made, checked by later events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTag {
    TookRisk,
    ExploredSidePath,
    AlertedEnemies,
    HarvestedEnergy,
    ConservedSupplies,
    UsedArcanePower,
    ScouredAhead,
    SetTraps,
}

/// Trigger for injecting or modifying a later event
pub struct EventChainTrigger {
    pub chain_id: EventChainId,
    /// Tags to add to the mission state
    pub tags: Vec<EventTag>,
    /// If Some, replace the event at this position index with a chain variant
    pub modify_event_index: Option<usize>,
}
```

## Event Scheduling

### When Events Fire

Events fire at fixed progress milestones during missions, determined by mission type:

| Mission Type   | Event Count | Trigger Points              |
|---------------|-------------|------------------------------|
| Supply Run     | 0           | (no events — always safe)    |
| Recon          | 1           | 50%                          |
| Expedition     | 2           | 33%, 66%                     |
| Breakthrough   | 3-5         | Varies by layer depth (below)|
| Construction   | 0           | (no events — always safe)    |

**Breakthrough event scheduling by layer tier:**

| Layer Tier     | Layers | Event Count | Trigger Points            |
|---------------|--------|-------------|---------------------------|
| Shallows       | 1-3    | 3           | 25%, 50%, 75%             |
| Warrens        | 4-7    | 3           | 25%, 50%, 75%             |
| Hollows        | 8-12   | 4           | 25%, 50%, 75%, 90%        |
| Sunken Reach   | 13-18  | 4           | 20%, 45%, 70%, 90%        |
| Abyss          | 19-25  | 5           | 15%, 35%, 55%, 75%, 90%   |
| Void           | 26+    | 5           | 15%, 35%, 55%, 75%, 90%   |

The final event on a Breakthrough mission is always a `BossApproach` category event. Earlier events are drawn from the tier's event pool with category diversity (no two consecutive events of the same category).

### Event Generation Algorithm

When a mission is created, its events are pre-generated:

```
fn generate_mission_events(mission: &Mission, layer: &Layer, rng: &mut impl Rng) -> Vec<EventInstance> {
    let event_count = event_count_for_mission_type(mission.mission_type, layer.tier);
    let trigger_points = trigger_points_for(mission.mission_type, layer.tier, event_count);
    let tier_pool = get_event_pool(layer.tier);

    let mut events = Vec::new();
    let mut last_category = None;

    for (i, &progress) in trigger_points.iter().enumerate() {
        let is_last = i == trigger_points.len() - 1;
        let is_breakthrough = mission.mission_type == MissionType::Breakthrough;

        // Last event on breakthrough is always BossApproach
        let template = if is_last && is_breakthrough {
            select_boss_approach_event(layer.tier, rng)
        } else {
            // Filter: different category from last, not BossApproach
            let candidates: Vec<_> = tier_pool.iter()
                .filter(|t| t.category != EventCategory::BossApproach)
                .filter(|t| Some(t.category) != last_category)
                .collect();
            candidates.choose(rng).unwrap()
        };

        last_category = Some(template.category);

        events.push(EventInstance {
            template_id: template.id,
            trigger_progress: progress,
            state: EventState::Scheduled,
            pending_since: None,
            chosen_index: None,
            applied_outcome: None,
        });
    }

    events
}
```

### Progress Tracking and Trigger

During mission ticking (checked periodically and on game load):

```
fn check_event_triggers(mission: &mut Mission, now: DateTime<Utc>) {
    let progress = mission.progress_fraction(now); // 0.0 to 1.0

    for event in &mut mission.events {
        match event.state {
            EventState::Scheduled => {
                if progress >= event.trigger_progress {
                    event.state = EventState::Pending;
                    event.pending_since = Some(now);
                    // Mission pauses progress until event is resolved
                }
            }
            EventState::Pending => {
                let elapsed = now - event.pending_since.unwrap();
                if elapsed >= Duration::hours(2) {
                    // Auto-resolve with safest option
                    auto_resolve_event(event, mission);
                }
            }
            _ => {}
        }
    }
}
```

**Key behavior:** When an event becomes Pending, mission progress pauses. The 2-hour auto-resolve timer runs on wall-clock time. Progress resumes after resolution.

### Auto-Resolve Rules

Auto-resolve always picks the **safest** option, defined as:

1. `injury_chance == 0.0` AND `loss_chance == 0.0` (no risk to mercs)
2. Among safe options, prefer the one with `is_risky == false`
3. Among equally safe options, prefer the one with the lowest `time_delay_minutes`
4. Auto-resolve never picks an archetype-gated option (even if available) — this is a deliberate design choice to reward active play

The `auto_resolve_index` on each template is pre-computed to be the index of the safest non-gated choice.

**Rationale for not using archetype options in auto-resolve:** Auto-resolve should never produce a better outcome than simply being present. If the Scout option is clearly best, the player is rewarded for checking in and choosing it. This maintains the "reward attention, never punish absence" philosophy.

## Event Catalog

### Layer Tier: The Shallows (Layers 1-3)

Themes: Damp tunnels, unstable stonework, minor creatures, basic navigation obstacles. Gentle introduction to the event system.

---

#### 1. Flooded Passage

**Category:** Obstacle
**Title:** FLOODED PASSAGE
**Description:** The path ahead is submerged in dark water. Something moves beneath the surface.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Wade through carefully | — | 2h | Safe | Auto-resolve default |
| 2 | Swim across quickly | — | 0 | Risky (injury 20%) | Risky: no delay but danger |
| 3 | Find higher ground | SCOUT | 1h | Safe | Scout spots ledge path |

**Auto-resolve:** Choice 1 (Wade through carefully)
**Chain:** If Choice 2 succeeds, tags `TookRisk` — may unlock bonus cache in a later Discovery event.

---

#### 2. Unstable Stairway

**Category:** Obstacle
**Title:** CRUMBLING DESCENT
**Description:** A long stone stairway descends into darkness. Cracks spider across every step. One wrong move and the whole thing comes down.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Descend one at a time | — | 2h | Safe | Auto-resolve default |
| 2 | Reinforce and cross | SABOTEUR | 30min | Safe, -20 Marks | Saboteur shores up the stairs |
| 3 | Jump to the landing | — | 0 | Risky (injury 25%) | High risk, no delay |

**Auto-resolve:** Choice 1

---

#### 3. Crumbling Bridge

**Category:** Hazard
**Title:** CRUMBLING BRIDGE
**Description:** A stone bridge spans a chasm filled with stagnant water. The supports are cracked. It might hold — or it might not.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Cross slowly, single-file | — | 1.5h | Safe | Auto-resolve default |
| 2 | Find another way around | — | 3h | Safe | Longer but completely safe |
| 3 | Reinforce with magic | ARCANIST | 0 | Safe, -15 Marks | Arcanist reinforces the stone |

**Auto-resolve:** Choice 1

---

#### 4. Subterranean Nest

**Category:** Combat
**Title:** SUBTERRANEAN NEST
**Description:** Your squad stumbles into a nest of pale, eyeless creatures. They skitter away from the light but there are dozens of them blocking the tunnel ahead.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Wait for them to disperse | — | 2h | Safe | Auto-resolve default |
| 2 | Push through aggressively | — | 0 | Risky (injury 15%) | Fast but risky |
| 3 | Shield wall advance | VANGUARD | 30min | Safe | Vanguard leads safe push |

**Auto-resolve:** Choice 1

---

### Layer Tier: The Warrens (Layers 4-7)

Themes: Branching tunnels, underground fauna, ambush threats, toxic environments. Choices begin to have meaningful consequences — delays are longer, risks higher.

---

#### 5. Tunnel Ambush

**Category:** Combat
**Title:** AMBUSH IN THE WARRENS
**Description:** Shapes materialize from alcoves in the tunnel walls. Your squad is surrounded on three sides.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Defensive formation | VANGUARD | 2h | Safe | Vanguard holds line, orderly retreat |
| 2 | Fight through | — | 0 | Risky (injury 30%, +60 bonus Marks on success) | 65% base success, Power-scaled |
| 3 | Set trap and counter-ambush | SABOTEUR | 0 | Safe, +30 Marks | Saboteur turns the tables |

**Auto-resolve:** Choice 1 (requires Vanguard) or Choice 2's safe fallback if no Vanguard — but see note below.

**Auto-resolve note:** If no Vanguard is in the squad, Choice 1 is unavailable. Auto-resolve falls back to the next safest non-gated option. In this case there is none that is perfectly safe, so the template has a hidden fallback: "Retreat and find another route" (3h delay, safe, no gate). This fallback is always present as a safety valve but not shown to the player if a better safe option exists.

**Design rule:** Every event template MUST have at least one non-gated, non-risky choice. This ensures auto-resolve always has a safe fallback.

---

#### 6. Poison Gas Vent

**Category:** Hazard
**Title:** TOXIC VENT
**Description:** A fissure in the tunnel floor belches noxious green vapor. The gas is heavier than air, pooling in the corridor ahead.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Wait for the vent to subside | — | 3h | Safe | Auto-resolve default |
| 2 | Purify the air | ARCANIST | 0 | Safe, -25 Marks | Arcanist neutralizes toxins |
| 3 | Rush through with held breath | — | 0 | Risky (injury 35%) | Fast but dangerous |

**Auto-resolve:** Choice 1
**Chain:** If player chooses 3 and fails, tags `TookRisk` — next event may have reduced squad Power modifier (0.9x for rest of mission).

---

#### 7. Collapsing Tunnel

**Category:** Obstacle
**Title:** CAVE-IN AHEAD
**Description:** Your squad encounters a collapsed tunnel blocking the main path. Rubble fills the corridor ceiling-to-floor.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Dig through | VANGUARD | 3h | Safe | Vanguard leads excavation |
| 2 | Find alternate route | SABOTEUR | 0 | Safe | Saboteur knows the tunnels |
| 3 | Blast through | ARCANIST | 1h | Safe, -30 Marks | Arcanist clears rubble with magic |
| 4 | Dig through (no Vanguard) | — | 4h | Safe | Slow manual excavation |

**Auto-resolve:** Choice 4 (always available, no gate)

**Note:** This event has 4 choices, 3 of which are archetype-gated. The non-gated fallback (Choice 4) is always available but is the worst option. Having ANY of the three specialists drastically improves the outcome. This teaches players that squad composition matters for The Warrens.

---

#### 8. Fork in the Path

**Category:** Discovery
**Title:** BRANCHING TUNNELS
**Description:** The tunnel splits into three passages. One is wide and well-traveled. One is narrow and dark. One has faint markings on the walls.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Take the wide passage | — | 0 | Safe | Standard route, auto-resolve default |
| 2 | Explore the narrow path | — | 1h | Risky (injury 15%, +40 bonus Marks on success) | Might find a cache |
| 3 | Follow the markings | SCOUT | 0 | Safe, +20 Marks | Scout reads wayfinding markers |

**Auto-resolve:** Choice 1
**Chain:** Choice 2 triggers `WarrensForkConsequence` — if successful, tags `ExploredSidePath`. A later event may offer a bonus treasure room or shortcut.

---

### Layer Tier: The Hollows (Layers 8-12)

Themes: Open caverns, bioluminescence, environmental hazards (spores, crystals, reality distortions), pack predators. Events become more complex — longer delays, higher stakes, more archetype interactions.

---

#### 9. Toxic Spore Cloud

**Category:** Hazard
**Title:** TOXIC SPORE CLOUD
**Description:** Dense clouds of luminous spores fill the cavern. Your squad is choking. The spores corrode metal and burn exposed skin.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Push through with cloth masks | — | 3h | Minor damage (all mercs -5% Power for rest of mission) | Auto-resolve default |
| 2 | Arcane Barrier | ARCANIST | 0 | Safe, -30 Marks | Arcanist conjures protective ward |
| 3 | Find ventilation shaft | SCOUT | 1h | Safe | Scout finds clean air route |

**Auto-resolve:** Choice 1 (safe from injury but squad takes minor Power debuff)

---

#### 10. Crystal Formation

**Category:** Discovery
**Title:** UNSTABLE CRYSTAL FORMATION
**Description:** A cluster of glowing crystals pulses with contained energy. Beautiful — and potentially volatile.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Navigate carefully | — | 2h | Safe | Auto-resolve default |
| 2 | Harvest crystal energy | ARCANIST | 0 | Safe, +50 Marks | Arcanist safely extracts energy |
| 3 | Smash through | — | 0 | Risky (injury 25%, +30 Marks on success) | Might shatter safely, might not |

**Auto-resolve:** Choice 1
**Chain:** If Choice 2 is picked, tags `HarvestedEnergy`. A later event in The Hollows may offer amplified Arcanist powers (extra bonus Marks or reduced delays).

---

#### 11. Hollow Stalkers

**Category:** Combat
**Title:** HOLLOW STALKERS
**Description:** Three eyeless predators emerge from the shadows. They've been tracking your squad for hours.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Defensive formation | VANGUARD | 2h | Safe | Vanguard holds line |
| 2 | Fight through | — | 0 | Risky (65% base success) | Success: +60 Marks, no delay. Failure: 1h delay + injury |
| 3 | Set trap and ambush | SABOTEUR | 0 | Safe, +30 Marks | Saboteur turns predators into prey |
| 4 | Retreat slowly | — | 3h | Safe | No gate fallback |

**Auto-resolve:** Choice 4 (safe, no gate)
**Chain:** Choice 2 (fight) failure triggers `HollowsAlertedBoss` — on breakthrough missions, the final BossApproach event becomes harder (boss is forewarned).

---

#### 12. Abyssal Echo

**Category:** Hazard
**Title:** ECHOES FROM BELOW
**Description:** A deep vibration rolls through the cavern. The walls pulse with faint light. Something vast stirs in the layers below, and its attention has brushed against your squad.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Wait for it to pass | — | 2h | Safe | Auto-resolve default |
| 2 | Ward the squad | ARCANIST | 0 | Safe, -40 Marks | Arcanist shields against psychic echo |
| 3 | Medicate the shaken | MEDIC | 30min | Safe | Medic calms nerves, minor delay |

**Auto-resolve:** Choice 1

---

### Layer Tier: The Sunken Reach (Layers 13-18)

Themes: Flooded chambers, ancient corruption, drowned guardians, tidal mechanics. The environment itself is hostile — Arcanists and Medics are critical. Events test economic decisions (spend Marks vs. lose time).

---

#### 13. Flooded Vault

**Category:** Obstacle
**Title:** THE DROWNED VAULT
**Description:** An ancient vault lies submerged beneath black water. The mission path runs through it. The water is cold, deep, and wrong.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Swim through | — | 0 | Risky (injury 30%, loss 5% on breakthrough) | Fast but the water is treacherous |
| 2 | Drain the vault | SABOTEUR | 2h | Safe, -50 Marks | Saboteur redirects water flow |
| 3 | Ward against the water | ARCANIST | 1h | Safe, -40 Marks | Arcanist creates air pocket |
| 4 | Find a way around | — | 4h | Safe | Long detour, auto-resolve default |

**Auto-resolve:** Choice 4

---

#### 14. Corrupted Artifact

**Category:** Discovery
**Title:** CORRUPTED RELIC
**Description:** A pedestal holds a dark artifact radiating waves of distortion. Your equipment buzzes. The air tastes like copper.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Leave it alone | — | 0 | Safe | Auto-resolve default |
| 2 | Purify and claim | ARCANIST | 1h | Safe, +80 Marks | Arcanist cleanses corruption |
| 3 | Grab and go | — | 0 | Risky (injury 40%, +60 Marks on success) | Corruption might spread |

**Auto-resolve:** Choice 1
**Chain:** Choice 2 triggers `SunkenReachCorruption` — tags `UsedArcanePower`. If a later event involves corruption, the Arcanist gets a bonus option to channel the purified energy.

---

#### 15. Sunken Guardian

**Category:** Combat
**Title:** THE SUNKEN GUARDIAN WAKES
**Description:** A massive stone figure rises from the flooded floor. Ancient runes flare to life across its body. It blocks the only passage forward.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Fight the Guardian | — | 0 | Risky (50% base, Power-scaled, injury 35% on fail) | Direct combat, Power check |
| 2 | Disable its runes | ARCANIST | 1h | Safe, -60 Marks | Arcanist unravels the enchantment |
| 3 | Shield wall and endure | VANGUARD | 3h | Safe | Vanguard absorbs blows while squad slips past |
| 4 | Find the bypass tunnel | SCOUT | 2h | Safe | Scout locates hidden passage |
| 5 | Wait for it to power down | — | 5h | Safe | It eventually runs out of energy |

**Auto-resolve:** Choice 5 (longest delay but always safe)

---

#### 16. Tidal Surge

**Category:** Hazard
**Title:** TIDAL SURGE
**Description:** A rhythmic booming echoes from below. The water level is rising — fast. Your squad has minutes before this section floods completely.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Climb to high ground | — | 2h | Safe | Wait out the surge, auto-resolve default |
| 2 | Sprint through before flood | — | 0 | Risky (injury 25%) | Race the water |
| 3 | Seal the water source | SABOTEUR | 30min | Safe, -30 Marks | Saboteur blocks the flood channel |
| 4 | Stabilize the wounded | MEDIC | 1h | Safe (reduces existing injury severity) | Medic tends to anyone hurt previously |

**Auto-resolve:** Choice 1
**Note:** Choice 4 (Medic) is unique — it doesn't affect this event's obstacle but instead heals previous injuries. It's a recovery opportunity rather than a navigation choice.

---

### Layer Tier: The Abyss (Layers 19-25)

Themes: Reality fractures, void entities, cosmic horror, extreme danger. Every event has high stakes. Multiple archetype options are common. Risky choices can produce extraordinary rewards or devastating losses.

---

#### 17. Abyssal Rift

**Category:** Discovery
**Title:** ABYSSAL RIFT
**Description:** A crack in reality shimmers before the squad. Strange energies seep through. The air hums with power — and danger.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Seal the rift | — | 2h | Safe | Auto-resolve default |
| 2 | Harvest the energy | — | 0 | Risky (injury 30%, +80 bonus Marks on success, 55% base) | High reward gamble |
| 3 | Channel through | ARCANIST | 0 | Safe, +50 Marks | Arcanist stabilizes and draws power |

**Auto-resolve:** Choice 1
**Chain:** Choice 2 success triggers `AbyssVoidHarvest`, tags `HarvestedEnergy`. If a later event involves void entities, the squad may gain advantage (or draw more attention).

---

#### 18. Void Swarm

**Category:** Combat
**Title:** VOID SWARM
**Description:** The darkness ahead isn't empty — it's alive. A swarm of void fragments converges on your position, each one a splinter of unmade reality.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Turtle formation | VANGUARD | 3h | Safe | Vanguard creates mobile fortress |
| 2 | Purging fire | ARCANIST | 0 | Safe, -70 Marks | Arcanist burns the void away |
| 3 | Sprint through the swarm | — | 0 | Risky (injury 40%, loss 5% on breakthrough) | Dangerous but instant |
| 4 | Slow retreat | — | 4h | Safe | Fall back and wait |

**Auto-resolve:** Choice 4 (no gate, safe)

---

#### 19. Reality Fracture

**Category:** Hazard
**Title:** REALITY FRACTURE
**Description:** Space folds. The tunnel ahead exists in two states simultaneously — one path leads forward, the other loops back on itself. Your squad can feel their thoughts splitting.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Anchor with willpower | — | 3h | Safe (minor Power debuff -5%) | Push through the disorientation |
| 2 | Navigate the fracture | SCOUT | 0 | Safe, +30 Marks | Scout's spatial awareness finds true path |
| 3 | Stabilize reality | ARCANIST | 1h | Safe, -50 Marks | Arcanist collapses the fracture |
| 4 | Meditate through it | MEDIC | 2h | Safe | Medic guides mental resilience, auto-resolve default |

**Auto-resolve:** Choice 4 (Medic option would be better, but auto-resolve doesn't use gated options — falls to Choice 1)

**Correction:** Auto-resolve uses Choice 1 (the non-gated safe option, despite the minor debuff), not Choice 4.

---

#### 20. The Whispering Dark

**Category:** Hazard
**Title:** THE WHISPERING DARK
**Description:** In the deepest reaches, the darkness whispers. Not sound — thought. It offers knowledge, power, a way forward. All it asks is a small price.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Ignore the whispers | — | 2h | Safe | Auto-resolve default |
| 2 | Listen and bargain | — | 0 | Risky (50% chance: success = +100 Marks + Power boost 1.1x, failure = injury 50% + Power debuff 0.85x) | The ultimate gamble |
| 3 | Ward against intrusion | ARCANIST | 0 | Safe, -80 Marks | Arcanist blocks the psychic assault |
| 4 | Medic counter-measure | MEDIC | 1h | Safe, squad Power restored if previously debuffed | Medic administers mental shields |

**Auto-resolve:** Choice 1

---

### Boss Approach Events (Breakthrough-only, always last event)

Each layer tier has a Boss Approach event variant. These fire at the final event slot (75% or 90% progress) and determine the conditions of the boss fight.

#### Shallows Boss Approach

**Title:** THE GUARDIAN'S CHAMBER
**Description:** A massive stone guardian blocks the descent. Its eyes glow with cold blue light.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Direct assault | — | 0 | Standard boss fight | Power check against boss |
| 2 | Tactical approach | SCOUT | 0 | Effective Power +15% | Scout finds weak points |
| 3 | Retreat | — | 0 | Safe, abort breakthrough | Partial rewards, layer NOT cleared |

**Auto-resolve:** Choice 1 (direct assault — auto-resolve does not retreat)

#### Warrens Boss Approach

**Title:** THE OVERSEER'S DEN
**Description:** The Warrens Overseer is a massive burrowing creature. Its lair is a tangle of tunnels within tunnels.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Direct assault | — | 0 | Standard boss fight | Power check against boss |
| 2 | Collapse tunnels | SABOTEUR | 0 | Effective Power +20% | Saboteur limits escape routes |
| 3 | Lure and trap | SCOUT | 0 | Effective Power +10%, -30 Marks | Scout baits the Overseer |
| 4 | Retreat | — | 0 | Safe, abort breakthrough | Partial rewards |

**Auto-resolve:** Choice 1
**Chain variant:** If `HollowsAlertedBoss` or `AlertedEnemies` tag is set, the boss starts with +10% effective Power (forewarned).

#### Hollows Boss Approach

**Title:** THE SENTINEL'S CHAMBER
**Description:** A massive stone guardian blocks the descent. Your squad is battered but determined.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Direct assault | — | 0 | Standard boss fight | Power check |
| 2 | Tactical approach | SCOUT | 0 | Effective Power +15% | Scout finds weak points |
| 3 | Arcane disruption | ARCANIST | 0 | Effective Power +20%, -40 Marks | Arcanist weakens the Sentinel's magic |
| 4 | Retreat | — | 0 | Safe, abort breakthrough | Partial rewards |

**Auto-resolve:** Choice 1

#### Sunken Reach Boss Approach

**Title:** THE DROWNED KING
**Description:** The Drowned King sits on a throne of coral and bone in a flooded throne room. The water here obeys its will.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Direct assault | — | 0 | Standard boss fight, -10% Power (water disadvantage) | Fighting in water is hard |
| 2 | Drain the chamber | SABOTEUR | 1h | Standard boss fight (normal Power) | Saboteur redirects water |
| 3 | Arcane counter | ARCANIST | 0 | Effective Power +15%, -60 Marks | Arcanist controls the water |
| 4 | Medical preparation | MEDIC | 30min | Injury severity reduced by 50% on partial success | Medic pre-treats squad |
| 5 | Retreat | — | 0 | Safe, abort breakthrough | Partial rewards |

**Auto-resolve:** Choice 1 (with water disadvantage — incentivizes bringing specialists)

#### Abyss Boss Approach

**Title:** THE VOID WARDEN
**Description:** Something vast and formless guards the threshold between layers. It has no shape, no weakness you can see. But it can be broken — everything down here can be broken, if you push hard enough.

| # | Choice | Gate | Delay | Risk | Notes |
|---|--------|------|-------|------|-------|
| 1 | Direct assault | — | 0 | Standard boss fight, -15% Power (void debuff) | The void saps strength |
| 2 | Void ward | ARCANIST | 0 | Standard boss fight (normal Power), -80 Marks | Arcanist counters void debuff |
| 3 | Find the anchor point | SCOUT | 0 | Effective Power +10% | Scout identifies the Warden's tether |
| 4 | Medical preparation | MEDIC | 30min | Injury/loss severity reduced by 50% | Medic pre-treats squad |
| 5 | All-out assault | — | 0 | Effective Power +25% but injury chance +20% | Glass cannon approach |
| 6 | Retreat | — | 0 | Safe, abort breakthrough | Partial rewards |

**Auto-resolve:** Choice 1 (with void debuff)

---

## Archetype Interaction Table

This table summarizes which archetypes unlock bonus options across all events, organized by layer tier. The table shows the number of events where each archetype provides a gated option.

### Options Per Archetype By Tier

| Archetype | Shallows | Warrens | Hollows | Sunken Reach | Abyss | Total |
|-----------|----------|---------|---------|--------------|-------|-------|
| Vanguard  | 1        | 2       | 1       | 1            | 1     | 6     |
| Scout     | 2        | 1       | 0       | 1            | 2     | 6     |
| Arcanist  | 1        | 2       | 3       | 3            | 4     | 13    |
| Medic     | 0        | 0       | 1       | 2            | 2     | 5     |
| Saboteur  | 1        | 2       | 1       | 2            | 0     | 6     |

### Archetype Specialization By Tier

Each tier has archetypes that are particularly valuable:

| Tier | Primary Archetype | Why |
|------|------------------|-----|
| Shallows | Scout | Navigation expertise in unfamiliar terrain |
| Warrens | Saboteur/Vanguard | Tunnel combat and obstacle clearing |
| Hollows | Arcanist | Environmental hazards (spores, crystals, echoes) |
| Sunken Reach | Arcanist/Saboteur | Water management and corruption purification |
| Abyss | Arcanist | Void/reality hazards dominate, arcane mastery essential |

### What Each Archetype Provides

| Archetype | Typical Bonus | When It Shines |
|-----------|--------------|----------------|
| **Vanguard** | Safe combat resolution with moderate delay | Ambush and combat events; holds the line |
| **Scout** | Faster safe routes, Power boosts for boss fights | Navigation obstacles, boss approaches |
| **Arcanist** | Zero-delay safe options (costs Marks) | Environmental hazards, boss fights, any magic-based obstacle |
| **Medic** | Injury reduction, recovery opportunities | Long missions, after risky choices fail, boss pre-fight |
| **Saboteur** | Fast resolution with bonus rewards | Tunnels, infrastructure, obstacles; turns threats into advantages |

### Squad Composition Recommendations

| Mission Profile | Recommended Squad (4-merc) | Rationale |
|----------------|---------------------------|-----------|
| Shallows Breakthrough | Vanguard, Scout, Medic, flex | Scout for navigation, Vanguard for nest/boss |
| Warrens Breakthrough | Vanguard, Saboteur, flex, flex | Saboteur critical for tunnels, Vanguard for ambushes |
| Hollows Breakthrough | Arcanist, Vanguard, Scout, Medic | Arcanist essential for hazards, full coverage |
| Sunken Reach Breakthrough | Arcanist, Saboteur, Medic, flex | Water management, corruption, injury prevention |
| Abyss Breakthrough | Arcanist, Scout, Medic, Vanguard | Arcanist mandatory, Scout for boss, Medic for survival |

## Auto-Resolve Logic

### Algorithm

```rust
fn auto_resolve_event(event: &mut EventInstance, mission: &Mission) {
    let template = get_template(event.template_id);

    // Find the safest non-gated choice
    let choice_index = template.auto_resolve_index;

    // Apply the outcome
    let choice = &template.choices[choice_index];
    let outcome = resolve_outcome(&choice.outcome, mission);

    event.state = EventState::AutoResolved;
    event.chosen_index = Some(choice_index);
    event.applied_outcome = Some(AppliedOutcome {
        choice_label: choice.label.clone(),
        time_delay_added: outcome.time_delay_minutes,
        marks_spent: outcome.mark_cost,
        marks_earned: outcome.bonus_marks,
        injury: None, // Auto-resolve never picks risky options
        was_auto_resolved: true,
        risk_check_result: None,
    });

    // Add time delay to mission
    mission.add_delay(Duration::minutes(outcome.time_delay_minutes as i64));
}
```

### Auto-Resolve Guarantees

1. **Never picks a risky choice** — `is_risky == true` options are excluded
2. **Never picks a gated choice** — even if the squad has the archetype
3. **Never causes injury or merc loss** — `injury_chance` and `loss_chance` must be 0.0
4. **Always picks the pre-computed `auto_resolve_index`** — deterministic, no randomness
5. **Applies time delays and Mark costs** — the safe option may still add time or cost Marks (this is the "cost of absence")

### Auto-Resolve Display

When the player returns and views a completed mission, auto-resolved events show:

```
Event: "Toxic Spore Cloud" (auto-resolved)
Choice made: Push through with cloth masks (safest option)
Result: +3h delay, squad took minor spore damage
Note: Arcanist option would have avoided the delay entirely.
```

The "Note" line is intentional — it teaches the player which archetype would have helped, encouraging better squad composition next time.

## Event Chaining Mechanics

### How Chains Work

1. When a player picks a choice with a `chain_trigger`, the mission gains `EventTag`s
2. Later events in the same mission check for these tags
3. If tags match, the event may be replaced with a chain variant or gain bonus options

### Chain Examples

#### Shallows — Side Cave Chain (`ShallowsSideCave`)

- **Event 1 (Flooded Passage):** Player picks risky "Swim across quickly" and succeeds
  - Tags applied: `TookRisk`
- **Event 3 (any):** If `TookRisk` is set, a bonus discovery is added:
  - "Your risky swim earlier revealed a hidden alcove. +25 bonus Marks."
  - This is a passive bonus, not a separate choice — it just adds Marks to the event outcome.

#### Warrens — Fork Consequence (`WarrensForkConsequence`)

- **Event 1 (Fork in the Path):** Player picks "Explore the narrow path" and succeeds
  - Tags applied: `ExploredSidePath`
- **Event 2 or 3:** If `ExploredSidePath` is set, an extra choice appears:
  - "The narrow path you found earlier connects to a hidden supply cache here."
  - New choice: "Loot the cache (+50 Marks, no delay, safe)"

#### Hollows — Alerted Boss (`HollowsAlertedBoss`)

- **Event 2 (Hollow Stalkers):** Player picks "Fight through" and fails
  - Tags applied: `AlertedEnemies`
- **Boss Approach event:** If `AlertedEnemies` is set, the boss has +10% effective Power
  - The boss fight is harder because the commotion alerted the Sentinel
  - A note explains: "The sounds of battle reached the Sentinel. It is prepared."

#### Sunken Reach — Corruption Channel (`SunkenReachCorruption`)

- **Event 1 (Corrupted Artifact):** Player picks "Purify and claim" (Arcanist)
  - Tags applied: `UsedArcanePower`
- **Later event with corruption:** If `UsedArcanePower` is set, an Arcanist bonus appears:
  - "The purified relic resonates with this corruption. Vex channels the stored energy."
  - Bonus: Arcanist option costs 0 Marks instead of the normal cost

#### Abyss — Void Harvest (`AbyssVoidHarvest`)

- **Event 1 (Abyssal Rift):** Player picks "Harvest the energy" and succeeds
  - Tags applied: `HarvestedEnergy`
- **Later combat event:** If `HarvestedEnergy` is set:
  - "The void energy your squad harvested earlier crackles to life."
  - Bonus: +20% squad Power for this event's combat resolution
  - But also: if `HarvestedEnergy` AND a Void Swarm event appears, the swarm is attracted (+1h delay to all options)
  - This creates a genuine double-edged sword — harvesting helps in some events, hurts in others

### Chain Design Rules

1. **Chains never make auto-resolve worse** — chain consequences only modify active player choices or add bonus options. Auto-resolve paths remain identical.
2. **Chains reward attention** — the player who saw Event 1 and made a deliberate choice gets a payoff in Event 3. Sleeping through both events gets standard outcomes.
3. **Negative chains are telegraphed** — if fighting the stalkers will alert the boss, the event text hints at this: "The sound of battle echoes through the caverns..."
4. **Chains don't cross missions** — all chain state is local to a single mission. No persistent event tags.
5. **At most 1 chain per mission** — to keep complexity manageable, a mission has at most one active chain. If multiple chain triggers would fire, only the first one activates.

## Implementation Notes

### Event Template Storage

Event templates are defined as static data (similar to `ZONE_ENEMY_STATS` in `constants.rs` or achievement data in `achievements/data.rs`). They live in `src/deep/events.rs` or `src/deep/event_data.rs`.

### Persistence

Event instances are serialized as part of the `Mission` struct in `~/.quest/deep.json`. The event state, choices made, and applied outcomes are all persisted so that:
- Events that fired while offline are correctly auto-resolved on load
- Mission results can show the full event history
- Chain tags are preserved for the mission's duration

### Wall-Clock Timing

Events use `DateTime<Utc>` for timing (via the `chrono` crate, already a dependency). The 2-hour auto-resolve window is checked:
- On game startup (process all missed events)
- Every tick (100ms) while the game is running
- On overlay open (immediate check)

### Scaling to The Void (L26+)

The Void reuses Abyss event templates with scaled consequences:
- Time delays increase by 10% per layer above 25
- Mark costs increase by 15% per layer above 25
- Injury/loss chances increase by 2% per layer above 25
- Boss Power thresholds increase by 5% per layer above 25

This provides infinite scaling without requiring new event content.

## deep-integration-architecture.md

# The Deep — Integration Architecture

**Date**: 2026-02-22
**Author**: sys-arch-2 (integration architecture agent)
**Status**: Draft

This document specifies exactly which existing files change, what new files are created, and how The Deep system integrates with every existing Quest subsystem.

---

## Overview

The Deep is an account-level system (like Haven and Soulforge) that persists across characters and prestiges. Its state is loaded at startup and saved alongside Haven and Enhancement. Missions run on wall-clock time so they progress while the game is closed. The integration surface is designed to be minimal and consistent with existing patterns.

---

## 1. New Module: `src/deep/`

Follow the standard module structure used by `src/haven/` and `src/enhancement/`.

### Files to Create

```
src/deep/
├── mod.rs          — Public re-exports + discovery roll function
├── types.rs        — All data structures (see Task #1 for full types spec)
├── generation.rs   — Merc generation, mission generation, event templates
├── logic.rs        — Mission ticking, event resolution, squad validation
├── persistence.rs  — Save/load from ~/.quest/deep.json
└── discovery.rs    — Discovery roll logic (try_discover_deep)
```

### `src/deep/mod.rs` — Public Re-exports

```rust
pub mod discovery;
pub mod generation;
pub mod logic;
pub mod persistence;
pub mod types;

pub use discovery::try_discover_deep;
pub use persistence::{load_deep, save_deep};
pub use types::DeepState;
```

### `src/deep/persistence.rs` — Save/Load

Mirrors `src/enhancement/persistence.rs` exactly. File: `~/.quest/deep.json`.

```rust
pub fn deep_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir()...;
    Ok(home_dir.join(".quest").join("deep.json"))
}

pub fn load_deep() -> DeepState {
    // Read file, serde_json::from_str, return DeepState::new() on error
}

pub fn save_deep(deep: &DeepState) -> io::Result<()> {
    // create_dir_all, to_string_pretty, write
}
```

### `src/deep/discovery.rs` — Discovery Roll

Mirrors the pattern from `src/haven/bonus.rs::haven_discovery_chance()` and `src/enhancement/logic.rs::try_discover_soulforge()`.

```rust
pub const DEEP_MIN_PRESTIGE_RANK: u32 = 15;
pub const DEEP_DISCOVERY_BASE_CHANCE: f64 = 0.000014; // same as Haven/Soulforge
pub const DEEP_DISCOVERY_RANK_BONUS: f64 = 0.000007;  // same as Haven/Soulforge

pub fn deep_discovery_chance(prestige_rank: u32) -> f64 {
    if prestige_rank < DEEP_MIN_PRESTIGE_RANK {
        return 0.0;
    }
    DEEP_DISCOVERY_BASE_CHANCE
        + (prestige_rank - DEEP_MIN_PRESTIGE_RANK) as f64 * DEEP_DISCOVERY_RANK_BONUS
}

pub fn try_discover_deep<R: Rng>(deep: &mut DeepState, prestige_rank: u32, rng: &mut R) -> bool {
    if deep.discovered {
        return false;
    }
    let chance = deep_discovery_chance(prestige_rank);
    if rng.random::<f64>() < chance {
        deep.discovered = true;
        true
    } else {
        false
    }
}
```

---

## 2. Modified File: `src/core/tick_types.rs`

### Add `deep_changed` Flag to `TickResult`

**File**: `/Users/stphung/workspace/quest3/src/core/tick_types.rs`

Add one field to `TickResult` (after `enhancement_changed`, following established pattern):

```rust
/// True if Deep state was modified (discovery) and should be persisted.
pub deep_changed: bool,
```

### Add `DeepDiscovered` Variant to `TickEvent`

Add to the Discovery section (after `SoulforgeDiscovered`):

```rust
/// The Deep was discovered (P15+ idle roll).
DeepDiscovered,
```

---

## 3. Modified File: `src/core/tick.rs`

### Add Deep Discovery Check (Stage 12 — New Stage)

The existing stages go to 12 (achievement modal accumulation). Add a new Stage 12 for Deep discovery, pushing achievement modal to Stage 13. Alternatively, insert it between Soulforge discovery (Stage 11) and achievement modal (Stage 12). The cleaner approach is to insert it as Stage 11b:

**Import additions at top of file**:

```rust
use crate::deep::DeepState;
```

**Function signature change** — add `deep: &mut DeepState` parameter:

```rust
pub fn game_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    enhancement: &mut crate::enhancement::EnhancementProgress,
    deep: &mut crate::deep::DeepState,          // NEW
    achievements: &mut Achievements,
    debug_mode: bool,
    rng: &mut R,
) -> TickResult
```

**Add discovery stage** (after Soulforge discovery block, before achievement modal):

```rust
// ── 11b. Deep discovery check ────────────────────────────
// Independent roll per tick, only when eligible (P15+, no active content)
if !deep.discovered
    && state.prestige_rank >= crate::deep::discovery::DEEP_MIN_PRESTIGE_RANK
    && state.active_dungeon.is_none()
    && state.active_fishing.is_none()
    && state.active_minigame.is_none()
    && crate::deep::try_discover_deep(deep, state.prestige_rank, rng)
{
    result.events.push(TickEvent::DeepDiscovered);
    result.deep_changed = true;
    if !debug_mode {
        result.achievements_changed = true;
    }
}
```

**Design rationale**: Deep discovery uses the same guard conditions as Haven and Soulforge (no active content). This ensures discoveries don't fire during dungeon/fishing/minigame sessions. The identical prestige threshold (P15+) matches Soulforge and Stormglass, making The Deep the third P15+ system alongside them.

---

## 4. Modified File: `src/main.rs`

### Load Deep State at Startup

After the Haven and Enhancement load calls (around line 178-180):

```rust
// Load account-level Deep state
let mut deep = deep::load_deep();
```

**Declare `mod deep`** at the top of `main.rs` (alongside other module declarations):

```rust
mod deep;
```

### Pass `deep` to `game_tick()`

Every call to `core::tick::game_tick()` in `main.rs` must add `&mut deep` as a parameter. There are three call sites:

1. **Normal game tick** (line ~752): Add `&mut deep,` before `&mut global_achievements`.
2. **Chrono Surge batch tick** (line ~678): Same.
3. **Chrono Surge skip/Esc tick loop** (line ~491): Same.

### Handle `deep_changed` Flag in Tick Result

After the existing flag checks in the normal tick path (around line 778-791), add:

```rust
if tick_result.deep_changed && !debug_mode {
    deep::save_deep(&deep).ok();
}
```

### Handle `DeepDiscovered` TickEvent

In `src/tick_events.rs` (the `apply_tick_events` function), add a case for `TickEvent::DeepDiscovered` that returns a flag, similar to how `HavenDiscovered` and `SoulforgeDiscovered` are handled.

Then in the main tick event processing block in `main.rs` (around line 796-804):

```rust
if tick_flags.deep_discovered {
    overlay = GameOverlay::DeepDiscovery;
}
```

### Save Deep in `save_all()`

**File**: `/Users/stphung/workspace/quest3/src/main_helpers/persistence.rs`

```rust
pub fn save_all(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    deep: &deep::DeepState,                   // NEW parameter
) {
    let _ = character_manager.save_character(state);
    achievements::save_achievements(global_achievements).ok();
    if haven.discovered {
        haven::save_haven(haven).ok();
    }
    if enhancement.discovered {
        enhancement::save_enhancement(enhancement).ok();
    }
    if deep.discovered {                       // NEW block
        deep::save_deep(deep).ok();
    }
}
```

Update all call sites of `save_all()` in `main.rs` to pass `&deep`.

### Resolve Completed Missions on Login

**File**: `/Users/stphung/workspace/quest3/src/main_helpers/offline.rs`

Add a parallel function to `apply_offline_xp()` for Deep offline resolution:

```rust
pub fn resolve_deep_offline(deep: &mut crate::deep::DeepState) -> Option<DeepOfflineReport> {
    crate::deep::logic::resolve_offline_missions(deep)
}
```

`resolve_offline_missions()` in `src/deep/logic.rs` inspects `last_resolved_at` timestamps on each active mission, completes missions whose `end_time` has passed, auto-resolves any check-in events that fired while offline (always picking the safe choice), and queues rewards for collection.

Call this from the character load path in `src/main_helpers/character_screens.rs` after offline XP is applied.

---

## 5. Modified File: `src/tick_events.rs`

**File**: `/Users/stphung/workspace/quest3/src/tick_events.rs`

The `apply_tick_events()` function returns a flags struct. Add a `deep_discovered` field:

```rust
pub struct TickFlags {
    pub haven_discovered: bool,
    pub soulforge_discovered: bool,
    pub stormglass_discovered: bool,
    pub deep_discovered: bool,              // NEW
}
```

Add match arm:

```rust
TickEvent::DeepDiscovered => {
    state.combat_state.add_log_entry(
        "\u{1F30A} A mercenary captain has found you. The Deep awaits...".to_string(),
        false,
        true,
    );
    flags.deep_discovered = true;
}
```

---

## 6. Modified File: `src/input/mod.rs`

### Add `DeepOverlay` Game Overlay State

**File**: `/Users/stphung/workspace/quest3/src/input/types.rs`

Add to the `GameOverlay` enum:

```rust
DeepDiscovery,
DeepOverlay,
```

### Add Deep Input Handler

Create **new file** `/Users/stphung/workspace/quest3/src/input/deep_input.rs`:

```rust
//! Input handling for The Deep overlay.
use crate::deep::DeepState;
use crate::input::types::{GameOverlay, InputResult};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_deep(
    key: KeyEvent,
    deep: &mut DeepState,
    overlay: &mut GameOverlay,
) -> InputResult {
    match key.code {
        KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
            *overlay = GameOverlay::None;
            InputResult::Continue
        }
        // TODO: Navigation, mission selection, event response
        _ => InputResult::Continue,
    }
}
```

Declare in `src/input/mod.rs`:

```rust
mod deep_input;
use deep_input::handle_deep;
```

### Add Deep Discovery Modal Handler

In `handle_game_input()` in `src/input/mod.rs`, add after the Stormglass discovery handler (around line 138-141):

```rust
// 1d. Deep discovery modal (blocks all other input)
if matches!(overlay, GameOverlay::DeepDiscovery) {
    return handle_deep_discovery(key, overlay);
}
```

Add the handler function:

```rust
fn handle_deep_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}
```

### Add Deep Overlay Handler in Input Priority Chain

In `handle_game_input()`, add after the Stormglass Exchange handler (around line 157-161):

```rust
// 2.8. Deep overlay
if matches!(overlay, GameOverlay::DeepOverlay) {
    return handle_deep(key, deep, overlay);
}
```

The function signature of `handle_game_input()` must add the `deep` parameter:

```rust
pub fn handle_game_input(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    exchange_ui: &mut ExchangeUiState,
    enhancement: &mut enhancement::EnhancementProgress,
    deep: &mut crate::deep::DeepState,            // NEW
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
    debug_mode: bool,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult
```

### Add `[D]` Keybind in Base Game Input

In `handle_base_game()` in `src/input/mod.rs`, add after the `[G]` Stormglass keybind (around line 385-390):

```rust
KeyCode::Char('d') | KeyCode::Char('D') => {
    if deep.discovered {
        *overlay = GameOverlay::DeepOverlay;
    }
    InputResult::Continue
}
```

The `handle_base_game()` signature must also receive `deep`:

```rust
fn handle_base_game(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    exchange_ui: &mut ExchangeUiState,
    enhancement: &enhancement::EnhancementProgress,
    deep: &crate::deep::DeepState,                // NEW
    overlay: &mut GameOverlay,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult
```

---

## 7. Modified File: `src/character/prestige_actions.rs`

### On Prestige: Reset Transient Deep State, Preserve Persistent

In `perform_prestige()`, add after the existing resets:

```rust
// Note: DeepState is account-level (not on GameState), so it's passed
// separately and handled in the prestige input handler, not here.
// The UI layer (prestige_input.rs) must call deep.on_prestige() after
// calling perform_prestige().
```

**File to modify**: `/Users/stphung/workspace/quest3/src/input/prestige_input.rs`

In `handle_prestige_confirm()`, after calling `perform_prestige()` or `perform_prestige_with_vault()`:

```rust
// Reset prestige-scoped Deep state (mercs, marks, active missions)
// while preserving guild rank, layer progression, infrastructure
deep.on_prestige();
```

This requires `deep: &mut crate::deep::DeepState` to be threaded through to `handle_prestige_confirm()`. The function signature becomes:

```rust
pub fn handle_prestige_confirm(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    deep: &mut crate::deep::DeepState,    // NEW
    overlay: &mut GameOverlay,
) -> InputResult
```

Update the call site in `handle_game_input()` to pass `deep`.

### `DeepState::on_prestige()` in `src/deep/types.rs`

```rust
impl DeepState {
    pub fn on_prestige(&mut self) {
        // Preserve: guild_rank, layers (cleared status, intel, infrastructure)
        // Reset: mercenaries, warband_marks, active_missions, available_missions
        self.mercenaries.clear();
        self.warband_marks = 0;
        self.active_missions.clear();
        self.available_missions.clear();
        // guild_rank, layers, deepest_layer_reached — untouched
    }
}
```

---

## 8. Modified File: `src/ui/mod.rs` and New `src/ui/deep_scene.rs`

### New UI Scene

Create `/Users/stphung/workspace/quest3/src/ui/deep_scene.rs` following the pattern of `soulforge_scene.rs` and `haven_scene.rs`. Sub-modules for clarity:

```
src/ui/deep_scene.rs        — Main overlay coordinator
src/ui/deep_roster.rs       — Mercenary roster panel
src/ui/deep_missions.rs     — Active/available missions panel
src/ui/deep_infrastructure.rs — Layer infrastructure view
```

### Pending Event Indicator in Stats Panel

**File**: `/Users/stphung/workspace/quest3/src/ui/stats_panel.rs`

Add a subtle indicator when The Deep has a pending check-in event that needs player response. This follows the same pattern as achievement notification counts. The stats panel already reads `state` and a separate `Achievements` — it will also receive `deep: &crate::deep::DeepState` (read-only reference) to check `deep.has_pending_events()`.

Add to the stats panel rendering to show something like:

```
[D] The Deep ⚡ event
```

The indicator appears only when `deep.discovered && deep.has_pending_events()`.

### Discovery Overlay

**File**: `/Users/stphung/workspace/quest3/src/main_helpers/overlay.rs`

Add Deep discovery overlay rendering to `draw_game_overlays()`:

```rust
GameOverlay::DeepDiscovery => {
    ui::deep_scene::render_discovery_modal(frame, area);
}
GameOverlay::DeepOverlay => {
    ui::deep_scene::render_deep_overlay(frame, area, deep, &ctx);
}
```

The `draw_game_overlays()` function must receive `deep: &crate::deep::DeepState` as a new parameter.

---

## 9. Modified File: `src/main_helpers/character_screens.rs`

### Pass `deep` to Character Load Flow

When loading a character, the offline progression runs. Add Deep offline resolution here:

```rust
// Resolve completed Deep missions that finished while offline
if deep.discovered {
    let deep_report = crate::deep::logic::resolve_offline_missions(&mut deep);
    if let Some(report) = deep_report {
        // Queue rewards, show summary in overlay or combat log
    }
}
```

This requires `deep: &mut crate::deep::DeepState` to be threaded through the `handle_select_frame()` call chain to `LoadCharacter`.

---

## 10. Reward Integration: Existing Systems

### XP Flow

Mission rewards include XP. This is applied via `crate::core::xp::apply_tick_xp()` during offline mission resolution and reward collection, exactly as offline XP is applied in `src/core/offline.rs`. No changes to the XP system itself.

### Item Flow

Mission rewards include items. Generated items use the existing `src/items/generation.rs` pipeline. Abyssal equipment uses the same `Item` struct with a special affix type added to `src/items/types.rs`:

**File**: `/Users/stphung/workspace/quest3/src/items/types.rs`

Add affix variants to the existing `AffixType` enum:

```rust
// Abyssal (Deep-exclusive) affixes
AbyssalMissionSpeed,     // % faster mission timers
AbyssalSupplyYield,      // % more Warband Marks from supply runs
AbyssalResilience,       // bonus merc resilience
```

### Stormglass Flow

Expedition and breakthrough missions award Stormglass. This increments `state.stormglass` directly during reward collection, same as salvage and dungeon caches. No changes needed to the Stormglass system.

---

## 11. Mission Timer Architecture

### Wall-Clock Time for Missions

Mission completion is determined by comparing `mission.start_time` (Unix timestamp, `i64`) against the current wall-clock time. This is the same approach used by `offline.rs` for offline XP:

```rust
// src/deep/types.rs
pub struct ActiveMission {
    pub mission_def: MissionDef,
    pub squad: Vec<MercId>,
    pub start_time: i64,           // Unix timestamp (Utc::now().timestamp())
    pub end_time: i64,             // start_time + mission_duration_seconds
    pub events: Vec<CheckInEvent>, // Pre-generated events with scheduled fire times
    pub event_choices: Vec<Option<usize>>, // Player's choices (None = auto-resolve)
    pub last_resolved_at: i64,     // Tracks partial resolution
}
```

### Tick-Based Mission Progress Updates

During the game tick (in `src/core/tick.rs`), mission timers do NOT use game ticks. Instead, missions are checked on each tick by comparing current wall-clock time against `end_time`. This is fast (just timestamp comparison, no simulation).

Add a lightweight mission check to `game_tick()`:

**Stage 13** (after achievement modal): Check for pending Deep check-in events that fired during this tick (events with `fire_time <= Utc::now().timestamp()` that have no choice yet). If found, set a flag in `TickResult`:

```rust
/// If Some, a Deep check-in event is ready for player response.
pub deep_event_ready: bool,
```

This flag is used by `main.rs` to show the event indicator in the UI (not to open the overlay automatically — the player presses `[D]` to respond).

Completed missions (where `end_time <= now`) are not resolved during the tick — they are resolved during offline resolution on load, or when the player opens The Deep overlay. This keeps the tick stage minimal and avoids item generation during the game loop.

---

## 12. Debug Menu Integration

**File**: `/Users/stphung/workspace/quest3/src/utils/debug_menu.rs`

Add a "Deep" category to the debug menu with options:
- "Discover The Deep" (sets `deep.discovered = true`, shows discovery modal)
- "Add 1000 Warband Marks"
- "Complete all missions"
- "Force next layer breakthrough"

This follows the existing pattern in `debug_menu.rs` (tabbed categories: Challenges, World, Resources, Items — add "Deep" as a fifth category).

---

## 13. Complete Change Summary

### New Files

| File | Description |
|------|-------------|
| `src/deep/mod.rs` | Module root, public re-exports |
| `src/deep/types.rs` | DeepState, Mercenary, Mission, Layer, Guild structs |
| `src/deep/generation.rs` | Merc/mission/event generation |
| `src/deep/logic.rs` | Mission ticking, event resolution, reward application |
| `src/deep/persistence.rs` | load_deep() / save_deep() |
| `src/deep/discovery.rs` | try_discover_deep(), discovery constants |
| `src/input/deep_input.rs` | handle_deep() input handler |
| `src/ui/deep_scene.rs` | Deep overlay coordinator |
| `src/ui/deep_roster.rs` | Mercenary roster panel |
| `src/ui/deep_missions.rs` | Missions panel |
| `src/ui/deep_infrastructure.rs` | Infrastructure/layer view |

### Modified Files

| File | Change |
|------|--------|
| `src/main.rs` | Add `mod deep`, load/save Deep, pass to game_tick, handle discovery overlay, resolve offline missions, update save_all calls |
| `src/core/tick.rs` | Add `deep: &mut DeepState` param, add Stage 11b (Deep discovery), add Stage 13 (check event ready) |
| `src/core/tick_types.rs` | Add `TickEvent::DeepDiscovered`, `TickResult::deep_changed`, `TickResult::deep_event_ready` |
| `src/tick_events.rs` | Handle `DeepDiscovered` event, add `deep_discovered` to `TickFlags` |
| `src/input/mod.rs` | Declare `deep_input`, add `deep` parameter, add discovery modal handler, add overlay handler, add `[D]` keybind, add deep to handle_base_game |
| `src/input/types.rs` | Add `GameOverlay::DeepDiscovery` and `GameOverlay::DeepOverlay` variants |
| `src/input/prestige_input.rs` | Add `deep: &mut DeepState` param, call `deep.on_prestige()` on prestige |
| `src/main_helpers/persistence.rs` | Add `deep: &crate::deep::DeepState` param to save_all(), call save_deep() |
| `src/main_helpers/offline.rs` | Add resolve_deep_offline() function |
| `src/main_helpers/overlay.rs` | Add Deep overlay rendering cases to draw_game_overlays() |
| `src/main_helpers/character_screens.rs` | Pass deep through LoadCharacter path, call resolve_offline_missions() |
| `src/character/prestige_actions.rs` | Add doc comment noting DeepState prestige handling is in prestige_input.rs |
| `src/items/types.rs` | Add AbyssalMissionSpeed, AbyssalSupplyYield, AbyssalResilience to AffixType |
| `src/ui/mod.rs` | Declare deep_scene, deep_roster, deep_missions, deep_infrastructure |
| `src/ui/stats_panel.rs` | Add pending event indicator when deep.has_pending_events() |
| `src/utils/debug_menu.rs` | Add "Deep" category with test triggers |

---

## 14. Dependency Graph for Implementation Tasks

The following dependencies exist between implementation tasks:

```
Task #8 (types.rs)
    └── blocks Task #9 (merc system)
    └── blocks Task #10 (mission system)
    └── blocks Task #11 (layer/economy)
    └── blocks Task #12 (persistence/discovery) ← This doc (integration arch) blocks #12
    └── blocks Task #13 (UI)
    └── blocks Task #14 (input)

Task #12 (persistence/discovery)
    └── blocks main.rs integration

Task #9 + #10 + #11
    └── block Task #12 (logic.rs depends on all types being defined)

Task #12 + #13 + #14
    └── block Task #15-21 (tests)
```

The integration document (this file) blocks Task #12 because the persistence integration pattern and discovery mechanism must be defined before implementation begins.

---

## 15. Architectural Decisions

### Why Account-Level (Not Character-Level)?

Deep state is account-level (like Haven and Soulforge) because:
1. Layer progression and infrastructure persist across prestiges by design
2. Guild rank is a permanent unlock
3. Multiple characters would share the same mercenary company narrative

This means `deep: DeepState` lives in `main.rs` alongside `haven` and `enhancement`, not embedded in `GameState`.

### Why Wall-Clock Time for Missions?

Missions are intended to progress while the game is closed. Using `last_save_time` (Unix timestamps) for missions follows exactly the same pattern as offline XP in `src/core/offline.rs`. The game tick does not simulate mission progress — it only checks for pending events and completion on the next frame after the game is already running.

### Why Not Extend `GameState`?

Adding `deep_state: Option<DeepState>` to `GameState` would couple Deep to character saves, causing The Deep to be reset when a character is deleted. As an account-level system, it must be independent. This is consistent with how Haven and Enhancement are handled.

All Deep data stays in the standalone `DeepState`, consistent with Haven and Enhancement.

### Why `[D]` Keybind?

Currently assigned keybinds: `[H]` Haven, `[S]` Soulforge, `[G]` Stormglass, `[A]` Achievements, `[P]` Prestige, `[Tab]` Challenges, `[?]` Help, `[U]` Updates, `[!]` Bug report.

`[D]` for "The Deep" is natural and unoccupied. The existing `[D]` key in the challenge menu is scoped to the challenge menu context only (Decline), so it does not conflict with the base game keybind.

### Discovery: Same Rate as Soulforge

The Deep uses the same discovery rate formula as Soulforge and Haven: `0.000014 + (rank - 15) * 0.000007` per tick. This gives an average discovery time of roughly 2 hours at P15 (same as Soulforge). This is intentional — all three P15+ systems are meant to be discovered at roughly the same progression milestone.

### Mission Resolution: Offline, Not On-Tick

Completed missions are not resolved during the game tick — they are resolved:
1. On game load (in the character load path, same as offline XP)
2. When the player opens The Deep overlay

This keeps the tick function minimal. The tick only checks for pending check-in events (a simple timestamp comparison, no item generation or complex logic).

## deep-quality-standards.md

# The Deep — Quality Standards & Review Criteria

Quality standards, testing requirements, review checklists, acceptance criteria, and performance considerations for The Deep (Mercenary Expedition System).

---

## 1. Code Quality Standards

### Module Structure

Follow the established Quest module pattern:

```
src/deep/
├── mod.rs          # Public re-exports
├── types.rs        # Mercenary, Layer, Mission, Guild, DeepState, events
├── generation.rs   # Merc generation, mission generation, event generation
├── logic.rs        # Mission ticking, event resolution, squad validation
├── persistence.rs  # Save/load from ~/.quest/deep.json (account-level)
└── discovery.rs    # Discovery roll logic (P15+ tick-based)
```

Every file must have a module-level `//!` doc comment explaining its purpose (see any existing module for examples).

### Zero UI Imports in Game Logic

All files in `src/deep/` must have **zero imports from `src/ui/`**. This is the same constraint enforced on `src/core/tick.rs` and all other game logic modules. The separation works as follows:

- Game logic returns data (structs, enums, events) describing what happened.
- The presentation layer (`main.rs`, `src/ui/`) reads that data and renders it.
- No `ratatui`, `crossterm`, or UI type references in game logic.

### Explicit Parameter Injection (Haven Pattern)

Deep bonuses and state must be passed as explicit parameters, not accessed via globals or statics. This is the same pattern used by Haven, Enhancement, and Stormglass:

```rust
// GOOD: explicit parameter
pub fn resolve_mission(mission: &mut Mission, deep_state: &DeepState, rng: &mut R) -> MissionResult { ... }

// BAD: global access
pub fn resolve_mission(mission: &mut Mission) -> MissionResult {
    let state = DEEP_STATE.lock().unwrap(); // NEVER do this
}
```

### Serde Derives for All Persistent Types

Every type that is part of the save file (`~/.quest/deep.json`) must derive `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepState { ... }
```

Transient fields (UI state, animation ticks, cached computations) must be marked `#[serde(skip)]` with sensible defaults via `Default` implementation.

### Generic RNG for Testability

All functions that use randomness must accept `<R: Rng>` (or `&mut impl Rng`) as a parameter. This enables deterministic testing with seeded `ChaCha8Rng`:

```rust
pub fn generate_mercenary<R: Rng>(guild_rank: u32, rng: &mut R) -> Mercenary { ... }
pub fn roll_event_outcome<R: Rng>(event: &CheckInEvent, squad: &Squad, rng: &mut R) -> EventOutcome { ... }
```

Never use `rand::rng()` or `thread_rng()` directly inside game logic functions.

### Clippy and Formatting

- All code must pass `cargo clippy --all-targets -- -D warnings` with zero warnings.
- All code must pass `cargo fmt --check`.
- No `#[allow(clippy::...)]` without a justifying comment.
- No `unwrap()` or `expect()` in production code paths. Use `unwrap_or_default()`, `?`, or explicit error handling. (Test code may use `unwrap()`.)

### Naming Conventions

- Types: `PascalCase` (e.g., `Mercenary`, `MissionType`, `GuildRank`)
- Functions: `snake_case` (e.g., `generate_mercenary`, `resolve_mission`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `DEEP_MIN_PRESTIGE_RANK`, `MAX_ROSTER_SIZE`)
- Enum variants: `PascalCase` (e.g., `MissionType::SupplyRun`, `Archetype::Vanguard`)
- Module files: `snake_case.rs`

### Constants Organization

All balance constants for The Deep must live in `src/deep/types.rs` (following Enhancement's pattern) or in `src/core/constants.rs` if they're referenced by the tick engine. Group constants with doc comments:

```rust
// ── Discovery ──────────────────────────────────────
pub const DEEP_MIN_PRESTIGE_RANK: u32 = 15;
pub const DEEP_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
pub const DEEP_DISCOVERY_RANK_BONUS: f64 = 0.000007;
```

---

## 2. Testing Requirements

### Coverage Target

The Deep game logic modules (`src/deep/`) must maintain **90% line coverage**, consistent with the project-wide CI gate (`--fail-under-lines 90` in `scripts/ci-checks.sh`, which excludes `ui/`, `utils/updater`, `utils/build_info`, and `tick_events`).

### Test Determinism

All tests must use seeded RNG for reproducibility:

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}
```

No `thread_rng()`, no `SystemTime::now()` in tests. For wall-clock time testing, inject timestamps as parameters.

### Unit Tests by Module

#### `types.rs` Tests
- Default/new construction for all structs (DeepState, Mercenary, Mission, Guild, Layer)
- Serde roundtrip: serialize then deserialize all persistent types, assert equality
- Enum variant completeness: all variants of MissionType, Archetype, LayerTier have correct properties
- Boundary values: max roster size, max guild rank, max layer depth

#### `generation.rs` Tests
- Mercenary generation produces valid stats for each archetype
- Mercenary quality scales with guild rank (higher rank = higher base stats)
- Mission generation respects layer difficulty and intel level
- Mission pool size is within bounds (3-5 missions)
- Event generation produces correct number of events per mission type (1-2 for short, 3-5 for breakthroughs)
- Seeded RNG produces identical output across runs

#### `logic.rs` Tests
- **Squad validation**: rejects squads below minimum requirements, accepts valid squads
- **Mission ticking**: wall-clock elapsed time correctly advances mission progress
- **Event scheduling**: events fire at correct progress percentages (25%, 50%, 75%)
- **Auto-resolve**: always picks the safest option (never risks merc loss)
- **Mission resolution**: success/partial/failure outcomes based on squad power vs. layer difficulty
- **Merc injury/loss**: only occurs on frontier missions, never on supply runs or construction
- **Merc XP**: awarded on mission completion, scales with mission difficulty
- **Infrastructure effects**: outpost reduces duration by 25%, watchtower improves auto-resolve
- **Breakthrough**: unlocks next layer on success, does not unlock on failure

#### `discovery.rs` Tests
- Discovery chance is 0 below P15
- Discovery chance increases linearly with prestige rank above 15
- Discovery is blocked when dungeon, fishing, or minigame is active (same as Haven/Soulforge)
- Once discovered, `discovered` flag persists and no further rolls occur

#### `persistence.rs` Tests
- Save/load roundtrip preserves all fields
- Missing file returns default state
- Corrupted JSON returns default state (graceful degradation)
- Save creates `~/.quest/` directory if needed
- Account-level fields survive prestige reset; character-level fields are cleared

### Integration Tests

Integration test files go in `tests/` (following existing patterns like `tests/enhancement_test.rs`, `tests/haven_dungeon_coverage_test.rs`).

#### `tests/deep_integration_test.rs`
- **Discovery flow**: Simulate ticks at P15+ until discovery occurs; verify DeepState transitions from None to Some
- **Mission lifecycle**: Generate mission, assign squad, tick time forward, trigger events, resolve mission, verify rewards
- **Prestige reset**: Verify guild rank, cleared layers, and infrastructure persist; mercs, marks, and active missions reset
- **Offline resolution**: Set mission start time in the past, load game, verify missions resolved with auto-resolve for missed events

#### `tests/deep_economy_test.rs`
- **Mark earning**: Verify all mission types award correct Warband Marks
- **Mark spending**: Recruitment, infrastructure, guild rank upgrades deduct correctly
- **Insufficient marks**: Operations fail gracefully when player cannot afford them
- **Guild rank progression**: Layer breakthrough requirements, cost verification, roster/mission slot scaling

#### `tests/deep_tick_test.rs`
- **game_tick integration**: Verify Deep discovery stage integrates with existing tick pipeline
- **No interference**: Deep system does not affect combat, fishing, dungeon, or challenge systems
- **TickEvent emission**: Deep-related events (discovery, mission complete, event pending) emit correct TickEvent variants

### What NOT to Test

Following the project's testing philosophy:

- Do not test UI rendering (Ratatui widget construction, layout calculations)
- Do not test internal data structure implementation details (cache sizes, internal ordering)
- Do not test timing-dependent behavior with wall-clock assertions (use injected timestamps)
- Do not test framework internals (serde implementation details, rand distribution internals)

---

## 3. PR Review Checklist

Every PR touching `src/deep/` must pass this checklist before merge:

### Architecture
- [ ] No `ui::` imports in game logic modules (`types.rs`, `logic.rs`, `generation.rs`, `discovery.rs`, `persistence.rs`)
- [ ] Bonuses and state passed as explicit parameters (Haven injection pattern)
- [ ] All persistent types derive `Serialize, Deserialize`
- [ ] All random functions accept generic `<R: Rng>` parameter
- [ ] Transient fields marked `#[serde(skip)]` with `Default` impl

### Data Model
- [ ] Types match the design doc (`docs/plans/2026-02-22-the-deep-mercenary-expedition-design.md`)
- [ ] Persistence correctly split: guild/layers/infrastructure persist; mercs/marks/missions reset on prestige
- [ ] Wall-clock timestamps use `chrono::Utc::now().timestamp()` (i64), not game ticks
- [ ] No panics: out-of-bounds access returns defaults, invalid state handled gracefully

### Balance
- [ ] Mission durations match design doc ranges (Supply 2-4h, Recon 4-8h, Expedition 8-16h, Breakthrough 18-24h, Construction 4-8h)
- [ ] Guild rank costs and unlock requirements match design doc
- [ ] Roster size caps match design doc (5/7/9/12/15 by rank)
- [ ] Concurrent mission slots match design doc (1/1/2/3/4 by rank)
- [ ] Infrastructure effects match design doc (outpost -25% duration, etc.)

### Safety
- [ ] Auto-resolve always picks the safest choice (never risks merc loss)
- [ ] Supply runs and construction missions never cause injury or loss
- [ ] Merc loss only possible on frontier missions with explicit risk
- [ ] No `unwrap()` or `expect()` in production code paths
- [ ] Offline resolution handles edge cases (very long absences, clock skew)

### Integration
- [ ] tick.rs changes are minimal (add discovery stage, similar to Haven/Soulforge pattern)
- [ ] DeepState passed to `game_tick()` the same way Haven and Enhancement are
- [ ] TickEvent variants added for Deep events (discovery, mission complete, event pending)
- [ ] TickResult gets `deep_changed: bool` flag for persistence signaling
- [ ] Achievement hooks called for Deep milestones (if applicable)

### Quality Gates
- [ ] `make check` passes (fmt, clippy, tests, build, audit)
- [ ] New tests added for all public functions
- [ ] Existing tests still pass (no regressions)
- [ ] Coverage maintained at 90%+ for game logic

---

## 4. Acceptance Criteria

The Deep is considered complete when all of the following are verified:

### Discovery
- [ ] Not available below P15
- [ ] Discovered via tick-based random roll (same pattern as Haven at P10+, Soulforge at P15+)
- [ ] Discovery blocked when dungeon, fishing, or minigame is active
- [ ] Discovery emits `TickEvent::DeepDiscovered` and sets `deep_changed` flag
- [ ] Once discovered, overlay is accessible via keybind

### Mercenaries
- [ ] 5 archetypes functional (Vanguard, Scout, Arcanist, Medic, Saboteur)
- [ ] Mercenary stats (Power, Resilience, Expertise, Level) calculated correctly per archetype
- [ ] Recruitment from rotating pool, cost in Warband Marks
- [ ] Roster size limited by guild rank
- [ ] Mercs gain XP from completed missions, level up correctly
- [ ] Mercs can be injured (unavailable for 1-2 missions) or lost on failed frontier missions
- [ ] Merc levels and roster reset on prestige

### Missions
- [ ] All 5 mission types functional (Supply Run, Recon, Expedition, Breakthrough, Construction)
- [ ] Mission generation produces 3-5 options matching current frontier/cleared layers
- [ ] Squad assignment validates requirements and recommendations
- [ ] Missions progress on wall-clock time (not game ticks)
- [ ] Missions progress while game is closed (offline resolution on load)
- [ ] Concurrent mission limit enforced per guild rank

### Check-In Events
- [ ] Events fire at scheduled progress percentages
- [ ] Archetype-specific options available when squad includes matching archetype
- [ ] Auto-resolve picks safest option when player does not respond
- [ ] Missed events (offline) auto-resolve correctly
- [ ] Events can chain (risky choice affects later events)

### Layers
- [ ] Layer tiers follow design doc (Shallows 1-3, Warrens 4-7, Hollows 8-12, Sunken Reach 13-18, Abyss 19-25, Void 26+)
- [ ] Breakthrough mission on Layer N unlocks Layer N+1
- [ ] Cleared layers persist across prestiges
- [ ] Infrastructure buildable on cleared layers
- [ ] Intel level accumulates from missions on a layer

### Economy
- [ ] Warband Marks earned from all mission types, scaling with depth and difficulty
- [ ] Marks spent on recruitment, infrastructure, guild rank upgrades
- [ ] Marks reset on prestige
- [ ] Guild rank persists across prestiges
- [ ] Guild rank costs and unlock requirements match design doc

### Persistence
- [ ] Save/load from `~/.quest/deep.json` works correctly
- [ ] Account-level data persists: guild rank, cleared layers, infrastructure, intel
- [ ] Character-level data resets on prestige: mercs, marks, active missions
- [ ] Corrupted save file falls back to default state (no crash)
- [ ] Active missions auto-cancel on prestige

### UI
- [ ] Overlay opens/closes cleanly (like Haven/Soulforge overlays)
- [ ] Active missions panel shows progress, time remaining, pending events
- [ ] New Mission sub-view shows available missions with requirements
- [ ] Roster sub-view shows merc list with stats and status
- [ ] Infrastructure sub-view shows per-layer build status
- [ ] Event Response sub-view shows choices with archetype-gated options
- [ ] Pending event indicator visible in main stats panel

### CI
- [ ] `make check` passes (fmt, clippy, tests, build, audit)
- [ ] No new clippy warnings
- [ ] Coverage maintained at 90%+ for game logic modules
- [ ] All new public APIs have tests

---

## 5. Performance Considerations

### Wall-Clock Time Processing

The Deep uses wall-clock time (real seconds) rather than game ticks for mission progression. This introduces performance constraints not present in tick-based systems.

**On game load (offline resolution):**
- Calculate elapsed time since last save for all active missions.
- Resolve any events that would have fired during offline period.
- This must complete in under **100ms** to avoid visible load delay (the game targets 100ms tick intervals).
- Strategy: iterate active missions (max 4 concurrent), each with at most 5 events. This is O(missions * events), bounded at 20 operations -- trivially fast.

**Per-tick processing:**
- The Deep should add **minimal overhead** to the existing game tick.
- Per tick, only check: (1) has any mission's wall-clock timer advanced past an event threshold? (2) has any mission completed?
- This is O(active_missions), bounded at 4. No expensive computation per tick.
- Discovery roll is a single random check per tick (same as Haven/Soulforge).

**Avoid:**
- Re-computing mission state from scratch every tick. Cache progress percentages and only recalculate when timestamp changes significantly.
- Sorting or searching large collections per tick. Mission and merc counts are small (max 15 mercs, max 4 missions).

### Persistence Overhead

**Save frequency:**
- Save `deep.json` only when `deep_changed` flag is set in TickResult (same pattern as Haven/Enhancement).
- Events that trigger saves: discovery, mission completion, event response, recruitment, construction, guild rank change.
- Do NOT save on every tick (mission progress is derived from timestamps, not stored progress percentages).

**Save file size:**
- Estimated maximum: ~50-100KB for a fully developed Deep state (15 mercs, 26+ layers with infrastructure/intel, guild rank, mission history).
- This is well within acceptable bounds -- Haven and Enhancement saves are under 5KB.
- Use `serde_json::to_string_pretty()` for human-readable saves (consistent with other Quest save files).

**Load performance:**
- Deserialization of ~100KB JSON is sub-millisecond. No concern here.
- Post-load offline resolution (see above) bounded at ~20 operations.

### Memory Usage

- DeepState is an account-level struct, single instance in memory.
- Mercenary roster: max 15 mercs, each ~200 bytes = ~3KB.
- Layer data: 26+ layers, each ~500 bytes with infrastructure and intel = ~15KB.
- Mission data: max 4 active, each ~1KB with events = ~4KB.
- Total in-memory footprint: **under 25KB**. Negligible compared to game state.

### Tick Budget

The game runs at 10 ticks/second (100ms per tick). The Deep's per-tick cost must fit within the existing tick budget alongside combat, fishing, dungeon, challenge AI, and other systems.

- **Target**: Deep tick processing should take under **1ms** per tick.
- **Measurement**: Use `cargo bench` or `std::time::Instant` timing in debug builds to verify.
- **If exceeded**: Profile and optimize the hot path (likely event scheduling or mission state checks).

### Simulator Compatibility

The Deep should integrate with the headless simulator (`src/bin/simulator.rs`):
- The simulator uses `game_tick()` with no UI and no tick delay.
- Wall-clock missions will need a simulated clock (injected timestamps) rather than `Utc::now()`.
- Add a `--deep` CLI flag (similar to `--haven`) for auto-managing Deep state during simulation runs.
- This is not required for initial implementation but should be planned for.

---

## References

- [Design doc](2026-02-22-the-deep-mercenary-expedition-design.md)
- [CI checks script](../../scripts/ci-checks.sh)
- [Enhancement system](../../src/enhancement/CLAUDE.md) -- closest architectural precedent
- [Haven system](../../src/haven/CLAUDE.md) -- bonus injection pattern precedent
- [Core tick engine](../../src/core/CLAUDE.md) -- tick integration pattern

## deep-t1t2-narrative.md

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

## deep-ui-audit.md

# Deep UI Audit — Information Gaps, UX Issues, and Improvement Opportunities

**Date:** 2026-02-23
**Scope:** All Deep UI files (`deep_scene.rs`, `deep_missions.rs`, `deep_roster.rs`, `deep_layers.rs`, `deep_events.rs`, `deep_results.rs`, `deep_input.rs`) plus supporting game logic.

---

## 1. Executive Summary — Top 5 Most Impactful Issues

### Issue 1: No Mission Description Shown (High Impact)
Every `AvailableMission` carries a `description` field populated with thematic, context-rich text (e.g., "Survey the Shallows for intel and entry points."). Neither the compact nor the split-panel New Mission view renders this field. Players see type name, layer number, duration, and risk label — but no flavor or contextual explanation of what the mission actually does.

### Issue 2: Familiarity System Is Nearly Invisible (High Impact)
Familiarity (0–100% intel per layer) is one of the three main progression levers in The Deep — it reduces mission durations by up to 30%, categorizes as Unknown/Mapped/Familiar/Mastered, and unlocks better auto-resolve. The Hub and New Mission views show no familiarity indicator. Only the Layers detail panel shows it, as a raw `%` number and bar labeled "Intel" rather than "Familiarity." The named tiers (Unknown/Mapped/Familiar/Mastered) and their effects are never communicated to the player.

### Issue 3: Success Probability Formula Is Opaque (High Impact)
The power threshold system — which determines success chance — is central to all decision-making. The UI shows "Min Power N" and a success forecast string ("Good odds — 60-90% success") but never explains what drives the outcome. Players cannot see the actual power ratio (squad power / threshold as a percentage), and the thresholds shown in the mission pool were pre-calculated without squad modifiers. Once inside squad assignment, showing `Pwr: 32/25  Good odds — 60-90% success` is correct but misses the ratio context (128% of threshold = in the 60-90% band).

### Issue 4: Guild Rank Upgrade Path Is Completely Hidden (High Impact)
Guild Rank is the primary account-level progression mechanic, gating roster size and concurrent missions. The Hub header shows `Guild: Freelancers (Rank 1)` and nothing more. There is no display of: the current upgrade cost (`guild_upgrade_cost` field exists in `DeepPersistent`), the required breakthrough layer to advance, or any indicator that the player is progressing toward the next rank. The Roster view duplicates this summary without adding upgrade information.

### Issue 5: The Recruit View Has No Rendering (High Impact)
`DeepView::Recruit` appears in the tab bar and has a complete input handler (`handle_recruit` in `deep_input.rs`), but there is no rendering function for it in any `deep_*.rs` UI file. The tab dispatches to `render_roster()` from the scene coordinator:

```rust
DeepView::Roster | DeepView::Recruit => {
    super::deep_roster::render_roster(...)
}
```

Pressing the Recruit tab silently shows the Roster view instead. Recruit candidates (`prestige.recruit_pool.candidates`) with their costs (`prestige.recruit_pool.recruit_costs`) are never displayed.

---

## 2. Information Gaps — Data That Exists But Isn't Shown

### 2.1 Hub View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Concurrent mission cap | `guild_rank.concurrent_missions()` | Player doesn't know how many more missions they can launch |
| Total Warband Marks earned (all-time) | Not tracked, but current balance only shown in header | Minor context loss |
| Guild rank upgrade requirements | `guild_rank.next()`, `guild_upgrade_cost`, `required_breakthrough_layer` | No path to progression visible |
| Merc availability summary | `prestige.available_merc_count()` | Player must tab to Roster to understand bench depth |
| Mission ETA (wall clock time) | `mission.ends_at` | Shown as elapsed/total, not "completes at HH:MM" |

### 2.2 New Mission View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Mission description | `AvailableMission.description` | No flavor; missions feel identical except for type label |
| Power ratio as percentage | `squad_power / min_squad_power * 100` | Success band unclear without ratio context |
| Duration modifiers active on target layer | `layer_record.infrastructure`, `layer_record.familiarity` | Player shown duration already reduced, but doesn't see why |
| Construction target infrastructure detail | `MissionType::Construction(infra)` | Display name shows "Construction" but not what's being built in the list view |
| Event count for mission type | `mission_type.max_events()` | Breakthrough has up to 5 events; player doesn't know to expect interruptions |
| Archetype benefit explanation | Per-archetype effects are implicit | Scout/Arcanist recommended but no tooltip on *why* |

### 2.3 Roster View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| XP-to-next-level progress | `Mercenary::missions_to_next_level(level)`, `merc.missions_completed` | Leveling feels random; no progress bar |
| Injury recovery countdown | `MercStatus::Injured { missions_remaining }` | Shows "Injured (2 missions)" but doesn't clarify what triggers recovery |
| Merc quality tier (Common/Uncommon/Rare/Elite) | Not stored on `Mercenary` struct | Cannot show quality label post-recruit |
| Recruit pool refresh timer | `recruit_pool.refreshed_at + 24h` | Player doesn't know when pool refreshes |
| Recruit candidate stats | `prestige.recruit_pool.candidates` | Entire Recruit tab missing rendering |
| Recruit costs | `prestige.recruit_pool.recruit_costs` | Cannot compare recruit value |

### 2.4 Layers View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Familiarity tier label | `FamiliarityLevel::from_familiarity(pct)` in `layers.rs` | Shows 42% but not "Mapped" |
| Duration reduction total | `layer_record.total_duration_reduction()` | Player cannot see combined modifier effect |
| Infrastructure build cost | `infrastructure_build_cost(infra, layer)` in `layers.rs` | No way to plan without knowing cost |
| Infrastructure build action | No build keybind in `handle_infrastructure` | Players can view but never build from the Layers view |
| Layer difficulty rating | `layer_record` tier + power threshold data | No sense of how hard the next push will be |
| Next guild rank breakthrough requirement | `guild_rank.next()?.required_breakthrough_layer()` | Cannot plan which layer to push toward |

### 2.5 Event Response View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Archetype bonus explanation | `event.archetype_bonus` field | Shows required archetype in `[TAG]` but `archetype_bonus` (a separate "improves outcome" archetype) is never displayed |
| Consequence magnitude | Time delta shown as "delay/faster" but no duration | Player cannot weigh `+3h delay` vs `safe` without numbers |
| Current mission progress bar | Progress is calculated but only shown as `%` in header | Visual progress bar would reinforce urgency |
| Choice outcome probabilities | Whether risky choices show partial odds | `is_risky` flag is used for consequence tag but risk % unclear |

### 2.6 Mission Results Modal

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Familiarity gained | Not shown (applied in `resolve_mission`) | Player misses feedback loop |
| Exact injury recovery duration | `MercStatus::Injured { missions_remaining }` | Shows "injured" but not "back in 2 missions" |
| Marks balance after collection | `prestige.warband_marks` after resolve | No post-result balance shown |
| Layer cleared notification | Breakthrough clearing a layer is only detectable from Layers view | Major milestone has no celebration moment |

---

## 3. UX Issues — Confusing Flows and Unclear Feedback

### 3.1 Recruit Tab Shows Roster Instead of Recruit Candidates
The `DeepView::Recruit` tab exists in the tab bar and pressing Tab cycles to it, but `deep_scene.rs` dispatches it to `render_roster()`. Players pressing Tab to "Recruit" see the Roster view with no indication something went wrong. The input handler for Recruit (`handle_recruit`) correctly navigates the recruit pool, creating an invisible mismatch between state and rendering.

**File:** `src/ui/deep_scene.rs:196` — the `DeepView::Roster | DeepView::Recruit` branch.

### 3.2 Squad Assignment Focus State Is Not Visually Distinguished
In the split-panel New Mission view, focus shifts between the mission list (left) and squad assignment panel (right) when Enter is pressed (`staging_mission_index` toggles). The left panel cursor disappears (becomes `"  "`) to indicate the list is no longer focused, but the right panel doesn't gain a visual "active" indicator. A player skimming the screen cannot immediately identify which panel is interactive.

**File:** `src/ui/deep_missions.rs:458` — `is_sel` condition checks `staging_mission_index.is_none()` for the left panel cursor, but the right panel has no compensating border highlight or header color change.

### 3.3 "No Missions Available" Empty State Is Misleading
When `available_missions` is empty, the message reads "Complete active missions to refresh the pool." However, the pool refresh logic in `missions.rs` is tied to game ticks (not mission completion alone). On a fresh prestige, before any missions are queued, the player sees this message with no actionable path. The empty state should explain discovery state and whether the pool needs a specific trigger.

**File:** `src/ui/deep_missions.rs:303-318`

### 3.4 Construction Mission Type Not Shown in List
In the mission list, `MissionType::Construction(Infrastructure::Outpost)` renders `display_name()` as `"Construction"` — the infrastructure type being built is dropped. Players queuing up a Construction mission have no way to know it will build an Outpost vs. a Bridge until they read the detail panel.

**File:** `src/ui/deep_missions.rs:358` — the `format!` call for the compact list row uses `m.mission_type.display_name()` without a Construction payload branch.

**Suggested fix:** Add a variant: if `MissionType::Construction(infra)`, use `format!("Build {}", infra.display_name())`.

### 3.5 Event Auto-Resolve Timer Uses Inconsistent Countdown Units
The event auto-resolve countdown switches from minutes to seconds at the 5-minute mark (`remaining < 5 * 60`), which is a reasonable UX touch. However, the `AUTO_RESOLVE_SECS = 30 * 60` constant (30 minutes) is hardcoded in the UI file (`deep_events.rs:155`) rather than sourced from a game logic constant. If the logic timeout changes, the UI countdown becomes incorrect.

**File:** `src/ui/deep_events.rs:155`

### 3.6 Hub Enter Key Behavior Is Asymmetric
In the Hub view, `[Enter]` on a completed mission dismisses the results modal. But `[Enter]` on an active mission with a pending event navigates to the EventResponse view. For active missions without events, `[Enter]` silently does nothing. Players pressing Enter on active missions expect some feedback (e.g., mission detail, or a tooltip explaining no action is available).

**File:** `src/input/deep_input.rs:54-106`

### 3.7 Flash Message Positioning Conflict
The flash message in the New Mission view is rendered at `height - 2` (one row above the footer), but the content rendering also uses `content_bottom = height - 2`. Long mission lists can overwrite the flash message area. The message may be obscured by mission list rows that extend to `content_bottom`.

**File:** `src/ui/deep_missions.rs:292-295`

### 3.8 Roster View Compact: Status Color Column Offset Brittle
In compact roster view, the status column offset uses `line.rfind(status_label)` to find the color position — this is fragile because if the merc's name contains a substring matching the status label (e.g., a merc named "Ready" with status "Ready"), the `rfind` will find the wrong position.

**File:** `src/ui/deep_roster.rs:138` — `let status_col = line.rfind(status_label).map(...)`

---

## 4. Visual Issues — Hierarchy, Spacing, Color Usage

### 4.1 Tab Bar Labels Don't Communicate Current State
The tab bar shows `[Hub] [Missions] [Roster] [Layers] [Recruit]` with the active tab in `Color::Rgb(80, 160, 220)` (cyan) and others in `Color::DarkGray`. However, tabs don't carry any state indicators:
- Missions tab with a pending event shows no indicator (the Hub shows `⚡` but you have to navigate there first)
- Roster tab with injured mercs shows no indicator
- Recruit tab when pool is fresh shows no indicator

**Comparison:** The stats panel shows a `[D]` indicator in the main game HUD with state-dependent color (Cyan for running, Yellow for event, Green for done), but the Deep overlay's own tab bar conveys none of this information.

**File:** `src/ui/deep_scene.rs:119-138`

### 4.2 Hub Header Hierarchy: All Text Is Same Color
The Hub header uses `Color::White` for the guild/marks line and `Color::DarkGray` for the subheader. The most actionable piece of information (Warband Marks balance) is embedded in the middle of a white-on-dark string with no visual emphasis. By comparison, Haven's header uses distinct color bands per piece of data.

**File:** `src/ui/deep_missions.rs:117-133`

### 4.3 Layer Tier Colors Are Not Consistently Applied
`layer_tier_color()` in `deep_layers.rs` maps tiers to colors (Shallows=Green, Warrens=Yellow, Hollows=Magenta, SunkenReach=Cyan, Abyss=LightRed, Void=Gold). However, in the compact list view the tier color is computed and then suppressed with `let _ = tc;` (line 155), so the layer number column is rendered in `Color::White` instead of the tier color. The split view correctly applies the tier color to the layer number. This creates an inconsistency between compact and full renderings.

**File:** `src/ui/deep_layers.rs:155` — `let _ = tc;` suppresses intentional color.

### 4.4 Progress Bar in Hub Uses Mission Type Color for Both Fill and Border
The progress bar `render_progress_bar()` uses the mission type color (`tc`) for filled cells and `Color::Rgb(30, 40, 60)` for empty cells. This is correct behavior, but in compact mode (S tier) the bar is only 12 characters wide, making it nearly unreadable against the dark backdrop. The backdrop uses `bottom_rgb = (2, 3, 8)` which makes `Color::Rgb(30, 40, 60)` nearly invisible for the empty portion.

**File:** `src/ui/deep_missions.rs:246`

### 4.5 Event View Title Uses All-Caps Instead of Bold
The event title is rendered with `.to_uppercase()` as a substitute for bold styling:
```rust
put_text_centered(buffer, narrative_top + 1, width, &event.title.to_uppercase(), Color::White);
```
Other overlays (Soulforge, Haven) use `Modifier::BOLD` via Ratatui's `Paragraph` with styled spans for emphasis. The scene buffer approach used here can't apply text modifiers to individual cells in `put_text`, but the event view could use a dedicated Ratatui `Paragraph` widget for the narrative section.

**File:** `src/ui/deep_events.rs:118-124`

### 4.6 Mission Results Modal Is Always 56-Wide Regardless of Terminal Size
The results modal clamps to `56u16.min(area.width - 4)` with a fixed-height layout. On wide terminals (XL tier), it renders as a small centered box while the overlay behind it is full-screen. Haven's details panel and Soulforge's animation panels fill their allocated regions proportionally. The results modal would benefit from adaptive sizing or at minimum a wider cap on XL.

**File:** `src/ui/deep_results.rs:26`

### 4.7 Separation Lines Use Same Color in All Views
Inner panel dividers use `Color::Rgb(40, 60, 80)` uniformly across Hub, New Mission, Roster, and Layers. Haven uses `Color::Rgb(60, 72, 84)` for borders and varies the saturation between inner and outer elements. The uniform separator color makes the Deep UI feel visually flat compared to Haven's subtle layering.

---

## 5. Comparison with Other Overlays

### 5.1 Haven Overlay (Gold Standard)
Haven's implementation provides a strong reference for Deep's missing polish:

| Feature | Haven | The Deep |
|---|---|---|
| Room description (word-wrapped) | Yes, `word_wrap()` in `haven_tree.rs` | Mission description field exists but not rendered |
| Tier progression shown inline | Yes, T1-T4 with next-tier arrow marker | No level/progression shown for mercs or guild |
| Cost shown before action | Yes, PR cost per tier shown in detail panel | Infrastructure costs not shown in Layers view |
| Prestige rank check for affordability | Yes, `can_afford()` with color feedback | No affordability check shown for recruitment |
| Bordered inner panels with titles | Yes, `render_room_detail()` draws its own border | Deep detail panels are borderless text blocks |
| Achievement badge in header | Yes, `highest_haven_badge()` | No achievement integration |

### 5.2 Soulforge Overlay
Soulforge's animation and feedback loop are relevant to mission results:

| Feature | Soulforge | The Deep |
|---|---|---|
| Animated outcome display | Hammering/success/failure particle effects | Static text-only results modal |
| Clear success/failure feedback | Screen-filling effect with color | Color border only |
| Slot state indicators in summary | Enhancement level bars per slot | No persistent state summary |

### 5.3 Stormglass Overlay
Stormglass shows how to handle time-gated elements:

| Feature | Stormglass | The Deep |
|---|---|---|
| Countdown timer display | `Chrono Surge` speed ramp with timer | Mission ETA shown as elapsed/total only |
| Daily reset indicator | Daily rotation label | Recruit pool refresh timer not shown |
| Phase-aware layout | `ExchangePhase` drives rendering | `DeepView` drives rendering (similar pattern, well implemented) |

### 5.4 Overall Assessment
The Deep UI is functionally correct and architecturally consistent with other overlays (scene buffer, tab bar, split panels). The main gaps versus the mature Haven/Soulforge overlays are:
1. **Description/context text** — Haven shows room descriptions; Deep doesn't show mission descriptions
2. **Progression feedback** — Haven shows tier arrows and costs; Deep shows no upgrade paths
3. **State indicators on tabs** — Deep tabs are stateless; event/completion states require navigation to discover
4. **Empty Recruit view** — a tab that renders nothing

---

## 6. Recommended Improvements (Prioritized)

### P0 — Critical Bugs / Missing Features

**P0.1 — Implement Recruit View Rendering**
The Recruit tab dispatches to `render_roster()`. Create `render_recruit()` in `deep_roster.rs` that shows:
- Candidate list with name, archetype, stats (power/resilience/expertise), and cost
- Recruitment cost in Warband Marks with affordability color (Green/Red)
- Pool refresh countdown (`recruit_pool.refreshed_at + 24h - now`)
- Roster capacity indicator
- `[Enter] Recruit` action and flash message for insufficient marks or full roster

**P0.2 — Show Mission Description in Detail Panel**
In `render_new_mission_split()`, add `AvailableMission.description` as the first item in the detail panel after the layer/tier header. Use `word_wrap()` (import from `haven_tree.rs`) to wrap to `detail_inner_w`. This is zero-cost to compute — the data is already available.

**P0.3 — Fix Construction Mission Label**
Change the mission list display to show `"Build Outpost"` / `"Build Bridge"` etc. instead of `"Construction"`:
```rust
// In deep_missions.rs, replace display_name() call with:
let type_label = match m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
```

### P1 — High Value / Low Complexity

**P1.1 — Add Familiarity Tier Label to Layers Detail**
In `render_layers_split()`, replace raw `"Intel:  42%"` with `"Familiarity: 42%  [Mapped]"` where the bracket label is colored per tier:
- Unknown (0-24%) → `Color::DarkGray`
- Mapped (25-49%) → `Color::Cyan`
- Familiar (50-74%) → `Color::Green`
- Mastered (75-100%) → `Color::Rgb(255, 215, 0)` (Gold)

**P1.2 — Show Guild Rank Upgrade Path in Hub**
Add a third header line in the Hub showing:
- Current concurrent mission count and cap: `Concurrent: 0/1`
- Next rank requirements: `Next Rank: Layer 3 Breakthrough  →  Rank 2 (Sellswords)`
Or fold into the existing subheader with color-coded upgrade info.

**P1.3 — Add State Indicators to Tab Bar**
After each tab label, append a state badge when relevant:
- `[Hub]` — show `●N` in Yellow when N events are pending, `✓N` in Green when N results await
- `[Missions]` — show `N` in Cyan for available mission count
- `[Roster]` — show `!N` in Yellow when N mercs are injured or lost
- `[Recruit]` — show `●` in Cyan when pool is fresh / has candidates

**P1.4 — Display Power Ratio as Percentage**
In the squad assignment summary, replace `"Power: 32/25"` with `"Power: 32/25  (128%)"`:
```rust
let ratio_pct = if min == 0 { 999 } else { squad_power * 100 / min };
let power_line = format!("Power: {}/{} ({}%)", squad_power, min, ratio_pct);
```
This directly communicates which success band the player is in.

**P1.5 — Show Infrastructure Build Costs in Layers View**
For unbuilt infrastructure slots in the detail panel, append the Warband Marks cost:
```rust
// In render_layers_split():
let cost = crate::deep::layers::infrastructure_build_cost(*infra, layer.index);
let cost_str = if built { String::new() } else { format!("  {}M", cost) };
put_text(buffer, row, col, &format!("{:12}  {}{}", name, desc, cost_str), color);
```

### P2 — Medium Value / Moderate Complexity

**P2.1 — Show Merc Leveling Progress in Roster**
In the detail panel, add an XP-style missions progress bar:
```
Level 3  [██████░░░░]  6/8 missions  →  Lv4
```
Where `missions_to_next_level(3) = 3 + 3*2 = 9`, and progress is `missions_completed % 9`.

**P2.2 — Show Mission ETA as Wall-Clock Time**
For active missions, add an ETA line: `"ETA: 14:32"` (local time when mission completes). This is especially valuable for multi-hour Breakthrough missions where players return to check progress.

**P2.3 — Show Familiarity Gained in Mission Results**
Add a line to the results modal: `"+ N% Familiarity on Layer X"` after the rewards section. The gain is deterministic from mission type (`familiarity_gain()` in `layers.rs`).

**P2.4 — Add Infrastructure Build Action to Layers View**
Currently the Layers view is read-only. Add `[B]` keybind to open a build sub-menu from the detail panel when the selected layer has buildable infrastructure slots and sufficient Marks.

**P2.5 — Fix Layer Tier Color in Compact Mode**
Remove `let _ = tc;` in `render_layers_compact()` (line 155) and apply `tc` to the layer number column, matching the split view behavior.

**P2.6 — Add Breakthrough Layer Cleared Celebration to Results Modal**
When `mission.mission_type == Breakthrough` and `result.outcome` is Success/PartialSuccess, add a prominent celebration line:
```
LAYER N CLEARED — New Depth Unlocked!
```
Colored in `Color::Rgb(255, 215, 0)` (Gold) centered in the modal.

### P3 — Polish / Low Priority

**P3.1 — Add Archetype Benefit Tooltips to Squad Assignment**
When a recommended archetype is present in the squad, highlight their name with the recommendation color and add a brief tooltip line explaining the benefit (e.g., "Scout reduces mission duration").

**P3.2 — Show Event Consequence Time Delta as Explicit Duration**
In the event choices, replace `"— delay"` / `"— faster"` with `"— +2h"` / `"— -1h"` using `format_hours(choice.time_delta_secs.abs() as u64)`.

**P3.3 — Extract Auto-Resolve Timer Constant to Game Logic**
Move `AUTO_RESOLVE_SECS = 30 * 60` from `deep_events.rs` to `src/deep/types.rs` or `missions.rs` so UI and logic agree on the timeout value.

**P3.4 — Fix Roster Compact Status Color Offset**
Replace `line.rfind(status_label)` with a fixed column offset calculated from the format string field widths to avoid fragile string searching.

**P3.5 — Adaptive Results Modal Width**
Change the modal width cap from a fixed 56 to a responsive value based on `ctx.tier`:
```rust
let modal_width = match ctx.tier {
    SizeTier::S => 50u16,
    SizeTier::M => 60u16,
    _ => 72u16,
}.min(area.width.saturating_sub(4));
```

---

## Appendix A — File-Level Summary

| File | Status | Key Issues |
|---|---|---|
| `deep_scene.rs` | Functional, solid | Tab bar lacks state indicators; opening animation is good |
| `deep_missions.rs` | Functional, incomplete | Missing: description rendering, construction label, power ratio % |
| `deep_roster.rs` | Functional, missing Recruit | No `render_recruit()` function; status offset fragile |
| `deep_layers.rs` | Functional, read-only | Missing: familiarity tier label, build costs, build action, tier color in compact |
| `deep_events.rs` | Functional | Auto-resolve const hardcoded; consequence magnitudes implicit |
| `deep_results.rs` | Functional | Missing: familiarity gain, layer cleared celebration, adaptive sizing |
| `deep_input.rs` | Correct | Hub Enter on non-event missions is a no-op with no feedback |

## deep-ui-design.md

# The Deep — UI Design

## Overview

The Deep overlay follows the same modal architecture as Haven, Soulforge, and Stormglass: a full-screen Clear + border block with an animated backdrop, sub-views navigated by keybind, and a footer help bar. The overlay opens over the combat scene and closes with Esc.

File structure mirrors Haven:
- `src/ui/deep_scene.rs` — main overlay coordinator
- `src/ui/deep_missions.rs` — active missions panel
- `src/ui/deep_roster.rs` — mercenary roster sub-view
- `src/ui/deep_layers.rs` — layer/infrastructure sub-view
- `src/ui/deep_event.rs` — event response sub-view
- `src/ui/deep_results.rs` — mission complete modal

---

## Color Conventions

Follows Quest's existing `rarity_color()` pattern in `src/ui/mod.rs`.

### Layer Tier Colors

| Tier | Layers | Color | RGB |
|------|--------|-------|-----|
| The Shallows | 1-3 | `Color::Green` | — |
| The Warrens | 4-7 | `Color::Yellow` | — |
| The Hollows | 8-12 | `Color::Magenta` | — |
| The Sunken Reach | 13-18 | `Color::Cyan` | — |
| The Abyss | 19-25 | `Color::LightRed` | — |
| The Void | 26+ | `Color::Rgb(255, 215, 0)` | Gold |

### Mission Type Colors

| Type | Color | Rationale |
|------|-------|-----------|
| Supply Run | `Color::Green` | Safe, reliable |
| Recon | `Color::Cyan` | Information-gathering |
| Expedition | `Color::Yellow` | Core progression |
| Breakthrough | `Color::LightRed` | High stakes |
| Construction | `Color::Blue` | Infrastructure |

### Merc Archetype Colors

| Archetype | Color | Rationale |
|-----------|-------|-----------|
| Vanguard | `Color::LightRed` | Frontline aggression |
| Scout | `Color::Cyan` | Mobility and awareness |
| Arcanist | `Color::Magenta` | Magic/elemental |
| Medic | `Color::Green` | Healing |
| Saboteur | `Color::Yellow` | Trickery |

### Event Urgency Colors

| State | Color |
|-------|-------|
| No pending events | `Color::DarkGray` |
| Event waiting (auto-resolve soon) | `Color::Yellow` |
| Event auto-resolving in <5 min | `Color::LightRed` |
| Mission complete, rewards pending | `Color::Green` |

### Backdrop

Deep blue-black gradient, darker than Stormglass. Top `(5, 8, 20)`, bottom `(2, 3, 8)`. Drifting particles in pale blue-white `(60, 80, 140)` simulate dust motes in cave air. Themed cyan border: `Color::Rgb(80, 160, 220)`.

---

## Backdrop Theme

```rust
// paint_deep_backdrop parameters
top_rgb:    (5, 8, 20)     // near-black deep blue
bottom_rgb: (2, 3, 8)      // void black
particle_count: 10
particle_chars: ['·', '•', '∘']
particle_color_hot: (60, 80, 140)   // cave dust
particle_color_cool: (20, 30, 60)
```

Opening flourish (600ms): brief blue-white sheen sweeping top-to-bottom on overlay open, simulating descent into The Deep.

---

## View 1: Main Overlay

The default view on open. Shows guild status, active missions, and navigation footer.

### Layout (L/XL — 80x30+)

```
┌─ THE DEEP ──────────────────────────────────────────────────────────┐
│                                                                     │
│  Guild: Sellswords (Rank 2)          Warband Marks: 1,240          │
│  Deepest Layer: 8 [The Hollows]      Mercs: 6/7                    │
│                                                                     │
│ ┌─ ACTIVE MISSIONS ─────────────────────────────────────────────┐  │
│ │ ► [Expedition]  Layer 8   12h elapsed / 16h                   │  │
│ │   [████████████████████████░░░░░░░] 78%                       │  │
│ │   Squad: Aldric (Vanguard), Sera (Scout), Thorne (Arcanist)   │  │
│ │   ⚡ Event pending! Press [Enter] to respond.                 │  │
│ │                                                               │  │
│ │   [Supply Run]  Layer 4    3h elapsed / 3h                    │  │
│ │   [████████████████████████████████] Done!                    │  │
│ │   Squad: Mira (Medic)                                         │  │
│ │   ✓ Complete — [Enter] to collect rewards.                    │  │
│ └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  No other missions running. [N] New Mission to deploy a squad.     │
│                                                                     │
│  [N] New Mission  [R] Roster  [L] Layers  [Esc] Close              │
└─────────────────────────────────────────────────────────────────────┘
```

**Cursor**: `►` marks the focused mission row. Up/Down navigate between missions.
**Enter on event**: opens Event Response sub-view.
**Enter on Done**: opens Mission Complete modal.

### Layout (M — 60x24)

```
┌─ THE DEEP ────────────────────────────────────────────┐
│ Sellswords Rank 2   Marks: 1,240   Mercs: 6/7        │
│ Layer 8 [The Hollows]                                 │
├───────────────────────────────────────────────────────┤
│ ► [Expedition] L8  ████████████░░░ 78%  ⚡ Event!   │
│   Aldric, Sera, Thorne                                │
│                                                       │
│   [Supply Run] L4  ████████████████ Done!            │
│   Mira                                                │
├───────────────────────────────────────────────────────┤
│ [N] New  [R] Roster  [L] Layers  [Esc] Close         │
└───────────────────────────────────────────────────────┘
```

### Layout (S — 40x16)

```
┌─ THE DEEP ──────────────────────────┐
│ Rank 2  Marks: 1,240  Mercs: 6/7   │
│                                     │
│ ► Expedition L8  78%  ⚡ Event!    │
│   Supply Run L4  Done!              │
│                                     │
│ [N]New [R]Roster [L]Layers [Esc]   │
└─────────────────────────────────────┘
```

**TooSmall (<40x16)**: Show "Terminal too small" per `render_too_small()` pattern.

---

## View 2: New Mission

Shown when player presses [N]. Left panel lists available missions; right panel shows details and squad assignment for the selected one.

### Layout (L/XL)

```
┌─ THE DEEP — New Mission ────────────────────────────────────────────┐
│                                                                     │
│  ┌─ AVAILABLE ───────────────────┐  ┌─ MISSION DETAIL ───────────┐ │
│  │ ► [Expedition]  Layer 8  12h │  │ Layer 8 — The Hollows      │ │
│  │   [Recon]       Layer 9   6h │  │                            │ │
│  │   [Supply Run]  Layer 4   2h │  │ Duration: 12-16h           │ │
│  │   [Construction] Layer 6  4h │  │ Risk:     Medium           │ │
│  │   [Recon]       Layer 8   5h │  │ Reward:   Marks + items    │ │
│  │                              │  │                            │ │
│  │                              │  │ Requires:                  │ │
│  │                              │  │  Min Power 40              │ │
│  │                              │  │  1+ Vanguard recommended   │ │
│  │                              │  │                            │ │
│  │                              │  │ ─ Assign Squad ──────────  │ │
│  │                              │  │  [✓] Aldric  Vanguard L4   │ │
│  │                              │  │  [✓] Sera    Scout    L3   │ │
│  │                              │  │  [ ] Mira    Medic    L2   │ │
│  │                              │  │        (on mission)         │ │
│  │                              │  │  [ ] Thorne  Arcanist L5   │ │
│  │                              │  │                            │ │
│  │                              │  │ Power: 72  ✓ Requirements  │ │
│  └──────────────────────────────┘  └────────────────────────────┘ │
│                                                                     │
│  [↑/↓] Select Mission  [Tab] Switch Panel  [Enter] Launch  [Esc]   │
└─────────────────────────────────────────────────────────────────────┘
```

**Panel focus**: [Tab] switches focus between mission list (left) and squad picker (right).
**Mission list**: Up/Down selects mission; detail panel updates immediately.
**Squad picker**: Up/Down navigates mercs; [Space] toggles assignment.
**Greyed rows**: Mercs on active missions shown with "(on mission)" label, unselectable.
**Power display**: Updates live as mercs are toggled. Green when requirements met, red when not.
**Launch**: [Enter] launches the mission with assigned squad. Requires at least 1 merc.

### Layout (M)

```
┌─ New Mission ─────────────────────────────────────────┐
│ [↑/↓] Mission  [Tab] Switch  [Space] Toggle Merc     │
├──────────────────────────┬────────────────────────────┤
│ ► Expedition  L8  12h   │ Layer 8 — The Hollows      │
│   Recon       L9   6h   │ Risk: Medium   Marks +items │
│   Supply Run  L4   2h   │                            │
│   Construction L6   4h  │ Assign:                    │
│   Recon       L8   5h   │ [✓] Aldric Vanguard L4     │
│                          │ [✓] Sera   Scout   L3     │
│                          │ [ ] Mira   (on mission)   │
│                          │ Power: 72  ✓ OK           │
├──────────────────────────┴────────────────────────────┤
│             [Enter] Launch  [Esc] Back                │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

Single panel, toggle between list and squad views with [Tab]:

```
┌─ New Mission ───────────────────────┐
│ [Tab] List/Squad  [Esc] Back        │
│                                     │
│ ► Expedition  L8  12h  Medium      │
│   Recon       L9   6h  Low         │
│   Supply Run  L4   2h  Safe        │
│                                     │
│ Power: 72  ✓  [Enter] Launch       │
└─────────────────────────────────────┘
```

---

## View 3: Roster

Shown when player presses [R]. Merc list with stats, archetype, level, status.

### Layout (L/XL)

```
┌─ THE DEEP — Roster ─────────────────────────────────────────────────┐
│                                                                     │
│  Mercs: 6/7          Guild Rank: 2 (Sellswords)                    │
│                                                                     │
│  ┌─ MERCENARIES ─────────────────────────────────────────────────┐ │
│  │  Name          Archetype   Lvl   Power  Resilience  Status    │ │
│  │ ─────────────────────────────────────────────────────────────│ │
│  │ ► Aldric        Vanguard    4     52     High        On L8    │ │
│  │   Sera          Scout       3     38     Med         On L8    │ │
│  │   Thorne        Arcanist    5     61     Low         On L8    │ │
│  │   Mira          Medic       2     24     High        On L4    │ │
│  │   Brennan       Saboteur    1     18     Med         Ready    │ │
│  │   Lys           Vanguard    2     27     High        Ready    │ │
│  │                                                               │ │
│  │   [Recruit slot open — 240 Marks]                            │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─ DETAIL: Aldric ──────────────────────────────────────────────┐ │
│  │  Archetype: Vanguard   Level: 4   Power: 52   Resilience: 72  │ │
│  │  Missions completed: 8   Bonus: Reduces squad casualties       │ │
│  │  Current: Expedition Layer 8 — 12h elapsed / 16h (78%)        │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  [↑/↓] Navigate  [Enter] Recruit (on empty slot)  [Esc] Back      │
└─────────────────────────────────────────────────────────────────────┘
```

**Status column colors**: "On L8" in `Color::DarkGray`, "Ready" in `Color::Green`, "Injured" in `Color::Yellow`, "Lost" in `Color::Red`.
**Archetype column**: Colored per archetype conventions above.
**Detail panel**: Updates live as cursor moves. Shows merc bio, stats, current assignment.
**Recruit slot**: Available if roster not full. Shows cost. [Enter] on empty slot opens recruit confirm modal.
**Injured mercs**: Shown with injury timer: "Injured (2 missions)".

### Layout (M)

```
┌─ Roster ──────────────────────────────────────────────┐
│ Mercs: 6/7   Guild Rank 2                            │
├───────────────────────────────────────────────────────┤
│ ► Aldric      Vanguard L4  P:52  On L8              │
│   Sera        Scout    L3  P:38  On L8              │
│   Thorne      Arcanist L5  P:61  On L8              │
│   Mira        Medic    L2  P:24  On L4              │
│   Brennan     Saboteur L1  P:18  Ready              │
│   Lys         Vanguard L2  P:27  Ready              │
│   [Recruit slot — 240 Marks]                        │
├───────────────────────────────────────────────────────┤
│ Aldric · Vanguard · 8 missions · On L8 78%          │
│ [↑/↓] Navigate  [Enter] Recruit  [Esc] Back         │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

```
┌─ Roster ────────────────────────────────┐
│ Mercs: 6/7                             │
│ ► Aldric   Vanguard L4   On L8        │
│   Sera     Scout    L3   On L8        │
│   Thorne   Arcanist L5   On L8        │
│   Mira     Medic    L2   On L4        │
│   Brennan  Saboteur L1   Ready        │
│   Lys      Vanguard L2   Ready        │
│ [Esc] Back                             │
└─────────────────────────────────────────┘
```

---

## View 4: Layers

Shown when player presses [L]. Layer-by-layer infrastructure status and progression.

### Layout (L/XL)

```
┌─ THE DEEP — Layers ─────────────────────────────────────────────────┐
│                                                                     │
│  Frontier: Layer 8 (The Hollows)     Deepest ever: Layer 8         │
│                                                                     │
│  ┌─ LAYER MAP ──────────────────┐  ┌─ LAYER DETAIL ──────────────┐ │
│  │  L1  The Shallows  [CLEAR]  │  │ Layer 4 — The Warrens       │ │
│  │  L2  The Shallows  [CLEAR]  │  │ Tier: The Warrens           │ │
│  │  L3  The Shallows  [CLEAR]  │  │ Status: Cleared             │ │
│  │  L4  The Warrens   [CLEAR]  │  │ Intel: 100%                 │ │
│  │  L5  The Warrens   [CLEAR]  │  │                             │ │
│  │  L6  The Warrens   [CLEAR]  │  │ Infrastructure:             │ │
│  │  L7  The Warrens   [CLEAR]  │  │  [✓] Outpost      (-25% t)  │ │
│  │ ►L8  The Hollows  [FRNTIR] │  │  [✓] Supply Cache (+yield)  │ │
│  │  L9  The Hollows  [??????] │  │  [ ] Watchtower   (locked)  │ │
│  │  L10 The Hollows  [??????] │  │  [ ] Bridge       (locked)  │ │
│  │  ...                       │  │                             │ │
│  │                             │  │ Available: Construction     │ │
│  │                             │  │  Watchtower — 300 Marks     │ │
│  └─────────────────────────────┘  └─────────────────────────────┘ │
│                                                                     │
│  [↑/↓] Navigate Layers  [Enter] Send Construction Mission  [Esc]   │
└─────────────────────────────────────────────────────────────────────┘
```

**Layer list colors**: Cleared layers use `Color::DarkGray` for number, `Color::Green` for `[CLEAR]`. Frontier layer uses `Color::Yellow` for `[FRNTIR]`. Unknown layers use `Color::DarkGray` for `[??????]`.
**Familiarity/Intel bar**: Rendered as `████░░░ 65%` where filled portion is `Color::Cyan`.
**Infrastructure checkboxes**: Built items in `Color::Green`, unbuilt in `Color::DarkGray`.
**Enter on cleared layer**: If a construction option is available and affordable, queues a construction mission selection flow.

### Layout (M)

```
┌─ Layers ──────────────────────────────────────────────┐
│ Frontier: Layer 8   Deepest: Layer 8                 │
├──────────────────────────┬────────────────────────────┤
│  L1  Shallows  [CLEAR]  │ Layer 4 — The Warrens      │
│  L2  Shallows  [CLEAR]  │ Cleared   Intel: 100%       │
│  L3  Shallows  [CLEAR]  │                            │
│  L4  Warrens   [CLEAR]  │ [✓] Outpost  [✓] Supply    │
│  L5  Warrens   [CLEAR]  │ [ ] Watchtower             │
│  L6  Warrens   [CLEAR]  │ Watchtower — 300 Marks     │
│  L7  Warrens   [CLEAR]  │                            │
│ ►L8  Hollows  [FRNTIR] │                            │
│  L9  Hollows  [??????] │                            │
├──────────────────────────┴────────────────────────────┤
│     [↑/↓] Navigate  [Enter] Build  [Esc] Back         │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

```
┌─ Layers ────────────────────────────────┐
│  L1  Shallows  CLEAR                   │
│  L4  Warrens   CLEAR                   │
│  L7  Warrens   CLEAR                   │
│ ►L8  Hollows   FRONTIER               │
│  L9  Hollows   ???                     │
│ [↑/↓] Navigate  [Esc] Back             │
└─────────────────────────────────────────┘
```

---

## View 5: Event Response

Shown when a mission has a pending check-in event. Accessible from Main view by pressing [Enter] on a mission with `⚡ Event pending!`. Can also be reached by pressing [E] from the main overlay.

### Layout (L/XL)

```
┌─ THE DEEP — Event ──────────────────────────────────────────────────┐
│                                                                     │
│  Mission: Expedition Layer 8   Progress: 78% (12h / 16h)           │
│  Squad: Aldric (Vanguard), Sera (Scout), Thorne (Arcanist)         │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │                                                                 │ │
│ │                     CAVE-IN AHEAD                               │ │
│ │                                                                 │ │
│ │   Your squad encounters a collapsed tunnel blocking the         │ │
│ │   main path. The ceiling groans under the weight above.         │ │
│ │   Dust and pebbles rain down from the passage ahead.           │ │
│ │                                                                 │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ► [Vanguard] Dig through       — 3h delay, safe                   │
│    [Saboteur] Find alternate    — no delay, moderate risk           │
│    [Arcanist] Blast through     — 1h delay, costs supplies         │
│    [Auto]     Let them decide   — always safe, worst outcome        │
│                                                                     │
│  Auto-resolve in: 23m   (safe choice will be selected)             │
│                                                                     │
│  [↑/↓] Choose  [Enter] Confirm  [Esc] Back (auto-resolve later)   │
└─────────────────────────────────────────────────────────────────────┘
```

**Choice rows**: Archetype tag colored per archetype (Vanguard = `Color::LightRed`, etc.). Options not available due to missing archetype shown in `Color::DarkGray` with `[--]` instead of archetype label.
**Auto-resolve timer**: `Color::Yellow` when >10 min remaining, `Color::LightRed` when <5 min.
**Esc from event**: Returns to main view. Event remains pending (timer continues).
**Consequence preview**: Shown inline after the choice text (delay, risk level, resource cost).

### Layout (M)

```
┌─ Event — Layer 8 ─────────────────────────────────────┐
│ Expedition 78%   Squad: Aldric, Sera, Thorne          │
├───────────────────────────────────────────────────────┤
│                                                       │
│               CAVE-IN AHEAD                           │
│                                                       │
│   Your squad encounters a collapsed tunnel            │
│   blocking the main path.                             │
│                                                       │
├───────────────────────────────────────────────────────┤
│ ► [Vanguard] Dig through    3h delay, safe           │
│   [Saboteur] Alternate route  no delay, risk         │
│   [Arcanist] Blast through  1h delay, supply cost    │
│   [Auto]     Let them decide  always safe            │
│                                                       │
│ Auto-resolve in: 23m                                  │
├───────────────────────────────────────────────────────┤
│     [↑/↓] Choose  [Enter] Confirm  [Esc] Back         │
└───────────────────────────────────────────────────────┘
```

### Layout (S)

```
┌─ Event ─────────────────────────────────┐
│ L8 Expedition  78%  Auto: 23m          │
│                                         │
│   CAVE-IN AHEAD                        │
│                                         │
│ ► [Vanguard] Dig through  safe         │
│   [Saboteur] Alternate    risk         │
│   [Arcanist] Blast        supply       │
│   [Auto]     Decide                    │
│                                         │
│ [↑/↓] [Enter] Confirm  [Esc] Back     │
└─────────────────────────────────────────┘
```

---

## View 6: Mission Complete

Modal overlay displayed when entering the main view with a completed mission (Done! state). Pressing [Enter] collects rewards and dismisses.

### Layout (L/XL — centered modal, ~55x18)

```
┌─ Mission Complete ──────────────────────────────────────┐
│                                                        │
│  Supply Run — Layer 4 — The Warrens                   │
│  Duration: 3h   Result: SUCCESS                        │
│                                                        │
│  Rewards:                                              │
│   + 380 Warband Marks                                  │
│   + 1 Rare item (Iron Raider's Chestguard)            │
│   + 240 XP (Mira +1 mission)                          │
│                                                        │
│  Squad:                                                │
│   ✓ Mira (Medic L2) — returned safely                 │
│                                                        │
│                                                        │
│           [Enter] Collect and Close                    │
└────────────────────────────────────────────────────────┘
```

**Result line color**: SUCCESS in `Color::Green`, PARTIAL SUCCESS in `Color::Yellow`, FAILURE in `Color::Red`.
**Item rarity**: Colored via `rarity_color()`.
**Merc status**: `✓` in `Color::Green` for safe return, `!` in `Color::Yellow` for injured, `✗` in `Color::Red` for lost.

### Breakthrough variant (larger rewards)

```
┌─ Mission Complete ──────────────────────────────────────┐
│                                                        │
│  Breakthrough — Layer 8 — The Hollows                 │
│  Duration: 18h   Result: SUCCESS                       │
│                                                        │
│  Layer 9 unlocked! (The Hollows continue deeper)       │
│                                                        │
│  Rewards:                                              │
│   + 1,840 Warband Marks                               │
│   + 1 Legendary item (Abyssal Voidmantle)             │
│   + 0.5 Prestige Rank                                 │
│   + 1,200 XP (all squad members)                      │
│                                                        │
│  Squad:                                                │
│   ✓ Aldric (Vanguard L5) — returned safely            │
│   ✓ Sera (Scout L4) — returned safely                 │
│   ! Thorne (Arcanist L5) — injured (2 missions)       │
│                                                        │
│           [Enter] Collect and Close                    │
└────────────────────────────────────────────────────────┘
```

---

## Stats Panel Integration

The existing stats panel in `src/ui/stats_panel.rs` needs a small indicator when The Deep is active.

### [D] Indicator Placement

Added to the header section (`draw_header()`) alongside existing system indicators. Appears after the character name/level line:

```
┌──────────────────────────────────┐
│ Aldric the Bold  Lv.142  P23    │
│ ████████████████████░░ 87% XP   │
│ [H] Haven  [J] Forge  [D] Deep  │  ← indicator row
└──────────────────────────────────┘
```

**States for [D] Deep indicator**:

| State | Display | Color |
|-------|---------|-------|
| Not discovered | (hidden) | — |
| Discovered, no missions | `[D] Deep` | `Color::DarkGray` |
| Mission running | `[D] Deep ●` | `Color::Cyan` |
| Event pending | `[D] Deep ⚡` | `Color::Yellow` |
| Mission complete | `[D] Deep ✓` | `Color::Green` |

The indicator uses the most urgent state if multiple missions are running (event pending > mission complete > running > idle).

### M-tier stats bar

On M-tier, the compact stats bar adds a single character to the activity line:

```
P23 | H✓ | J+5 | D⚡ | Zone 8 | Lv.142 ...
```

Where `D⚡` collapses to the most urgent Deep state character. Colors follow the same table above.

### S-tier

On S-tier, add `[D]` to the footer keybind line only. No inline indicator (too little space).

---

## Interaction Flow Diagrams

### Flow 1: Discovery to First Mission

```
[Idle — P15+]
      │
      ▼ (tick-based discovery roll fires)
[Combat log: "A scarred mercenary captain approaches..."]
[Discovery modal — "The Deep Unlocked! Press [D] to open."]
      │ [Enter] or [D]
      ▼
[The Deep Main View — first open]
  Guild: Freelancers (Rank 1)   Marks: 0
  Mercs: 4/5 (starter roster)
  No active missions.
  [N] New Mission ...
      │ [N]
      ▼
[New Mission View]
  Available missions:
  ► [Supply Run] Layer 1  2h  Safe    ← only safe options shown first
    [Recon]     Layer 1  4h  Low
      │ select mission, assign mercs
      │ [Enter]
      ▼
[Main View — mission launched]
  ► Supply Run L1  ░░░░░░░░░░░░ 0%
      │ (real time passes)
      ▼
[Main View — mission complete]
  ► Supply Run L1  ████████████ Done! ✓
      │ [Enter]
      ▼
[Mission Complete Modal]
  + 120 Marks  + Common item  + XP
      │ [Enter]
      ▼
[Main View — rewards collected]
```

### Flow 2: Mission Launch

```
[Main View]
      │ [N]
      ▼
[New Mission View — left panel focused]
      │ [↑/↓] select mission
      ▼
[Mission detail updates in right panel]
      │ [Tab] switch to squad picker
      ▼
[Squad picker focused]
      │ [↑/↓] navigate mercs
      │ [Space] toggle assignment
      ▼
[Power display updates, requirement met/not]
      │ [Enter] (requirements met)
      ▼
[Confirmation: "Launch Expedition L8 with Aldric, Sera, Thorne? [Enter]/[Esc]"]
      │ [Enter]
      ▼
[Main View — new mission appears in active list]
```

### Flow 3: Event Response

```
[Main View — event pending indicator ⚡]
      │ [↑/↓] cursor on mission
      │ [Enter]
      ▼
[Event Response View]
  Event text + 4 choices displayed
      │ [↑/↓] select choice
      │ [Enter] confirm
      ▼
[Main View — event resolved]
  Mission continues (timer adjusted by choice outcome)
      │ OR player presses [Esc] without responding
      ▼
[Main View — event still pending, timer ticking]
      │ (auto-resolve timer expires)
      ▼
[Auto-resolve applies safe default choice silently]
```

### Flow 4: Prestige Transition

When player prestiges while Deep missions are active:

```
[Prestige Confirm Screen — standard]
      │ If Deep missions are active:
      ▼
[Warning line added to prestige confirm dialog]
  "Active Deep missions will be cancelled."
  "Guild rank, infrastructure, and layer progress persist."
      │ [Enter] confirm prestige
      ▼
[Prestige executes]
  - All active missions cancelled (no rewards)
  - Warband Marks reset to 0
  - Mercenaries dismissed (fresh roster on next open)
  - Guild rank preserved
  - Cleared layers preserved
  - Infrastructure preserved
      │ (new prestige begins)
      ▼
[The Deep Main View — first open after prestige]
  Guild: [same rank]   Marks: 0
  Mercs: 0/[max] — recruit fresh mercenaries
  [N] New Mission (re-enabled immediately)
```

---

## Module Structure

```
src/ui/
├── deep_scene.rs       — Main overlay coordinator, backdrop, tab routing
├── deep_missions.rs    — Active missions panel and mission list rendering
├── deep_roster.rs      — Roster sub-view
├── deep_layers.rs      — Layer map sub-view
├── deep_event.rs       — Event response sub-view
└── deep_results.rs     — Mission complete modal
```

### deep_scene.rs responsibilities

- `render_deep_overlay(frame, area, deep_state, ctx)` — top-level entry point
- `paint_deep_backdrop(buffer, millis)` — cave-blue gradient + dust particles
- `paint_opening_deep_fx(buffer, millis, elapsed)` — 600ms descent sheen
- Routes to sub-views based on `DeepUiState` (Main, NewMission, Roster, Layers, EventResponse)
- Renders border with title " THE DEEP "
- Renders footer help bar for current sub-view

### Color helper (add to src/ui/mod.rs)

```rust
pub fn layer_tier_color(layer: u32) -> Color {
    match layer {
        1..=3   => Color::Green,
        4..=7   => Color::Yellow,
        8..=12  => Color::Magenta,
        13..=18 => Color::Cyan,
        19..=25 => Color::LightRed,
        _       => Color::Rgb(255, 215, 0),
    }
}

pub fn merc_archetype_color(archetype: MercArchetype) -> Color {
    match archetype {
        MercArchetype::Vanguard  => Color::LightRed,
        MercArchetype::Scout     => Color::Cyan,
        MercArchetype::Arcanist  => Color::Magenta,
        MercArchetype::Medic     => Color::Green,
        MercArchetype::Saboteur  => Color::Yellow,
    }
}

pub fn mission_type_color(mission_type: MissionType) -> Color {
    match mission_type {
        MissionType::SupplyRun    => Color::Green,
        MissionType::Recon        => Color::Cyan,
        MissionType::Expedition   => Color::Yellow,
        MissionType::Breakthrough => Color::LightRed,
        MissionType::Construction => Color::Blue,
    }
}
```

---

## Keybind Conventions

| Key | Action |
|-----|--------|
| `d` | Toggle The Deep overlay (from main game) |
| Esc | Close overlay / back to parent view |
| N | New Mission sub-view |
| R | Roster sub-view |
| L | Layers sub-view |
| E | Event (if pending) sub-view |
| Enter | Confirm selection / collect rewards |
| Space | Toggle merc in squad picker |
| Tab | Switch panel focus (New Mission view) |
| Up/Down | Navigate list items |

The `d` keybind follows the same convention as `h` (Haven), `j` (Soulforge) — single lowercase letter toggles the overlay.

---

## Implementation Notes for Developers

### Scene buffer pattern

All sub-views render into a `Vec<Vec<SceneCell>>` buffer using `put_text()` and `put_cell()` from `scene_fx.rs`, then flush via `render_buffer()`. This matches Haven, Soulforge, and Stormglass.

### Sub-view switching

`DeepUiState` (defined in `src/deep/types.rs` or `src/input/types.rs`) tracks which sub-view is active. The input handler routes keys based on the current state. Pattern from `StormglassUiState` and `SoulforgeUiState`.

### Progress bar rendering

Mission progress bars use the same `Gauge` widget as the XP bar in `stats_panel.rs`:
```rust
let ratio = elapsed_secs as f64 / total_secs as f64;
let gauge = Gauge::default()
    .gauge_style(Style::default().fg(mission_type_color(mission.mission_type)))
    .ratio(ratio.clamp(0.0, 1.0));
```

Or inline via `put_text()` with block characters `████░░░` when rendering inside a scene buffer.

### Wall-clock time display

Format elapsed/remaining time using the existing `format_eta()` helper from `stats_prestige.rs`:
- `format_eta(remaining_secs)` → "~3h 20m"
- For elapsed: same function with elapsed_secs

### Responsive layout dispatch

```rust
match ctx.tier {
    SizeTier::TooSmall => render_too_small(frame, ctx),
    SizeTier::S        => render_deep_s(frame, area, state, ctx),
    SizeTier::M        => render_deep_m(frame, area, state, ctx),
    SizeTier::L | SizeTier::XL => render_deep_lxl(frame, area, state, ctx),
}
```

### Discovery modal

Follows `render_haven_discovery_modal()` pattern. Centered ~52x10 modal with Yellow border, flavor text, keybind hint. Trigger phrase in combat log: "A scarred mercenary captain approaches, maps of underground passages spilling from worn satchels."

## deep-ui-hub-missions-design.md

# The Deep — Hub View and Mission Assignment UI Design

**Author:** UX Designer Agent
**Date:** 2026-02-23
**Updated:** 2026-02-23 (incorporating findings from `docs/plans/deep-ui-audit.md`)
**Scope:** Tasks #2 and #3 — Hub view information hierarchy and Mission assignment flow clarity
**Status:** Final — ready for implementation

---

## Executive Summary

After auditing all Deep UI files (`deep_missions.rs`, `deep_scene.rs`, `deep_roster.rs`, `deep_layers.rs`, `deep_events.rs`, `deep_input.rs`, `types.rs`, `CLAUDE.md`) plus cross-referencing the system audit (`docs/plans/deep-ui-audit.md`), I identified concrete gaps in information hierarchy, progressive disclosure, and player feedback. This document covers the Hub view and Mission assignment flow with ASCII mockups, specific color values, and exact function changes needed.

**Audit findings incorporated (P0 and P1 that touch Hub/Mission scope):**
- P0.2: Mission descriptions exist but are never shown — addressed in Part 2
- P0.3: Construction missions drop infrastructure type in labels — addressed in Part 2 Change 7
- P1.2: Guild rank upgrade path completely hidden — addressed in Part 1 Change 1
- P1.3: Tab bar has no state indicators — addressed in Part 1 Change 6 (new)
- P1.4: Power ratio should show percentage — addressed in Part 2 Change 2
- Bug 3.3: "No missions available" empty state misleading — addressed in Part 2 Change 8
- Bug 3.7: Flash message positioning conflict with content — addressed in Part 2 Change 9

---

## Part 1: Hub View Redesign

### 1.1 Current State Problems

**Identified issues in `render_hub()` (`deep_missions.rs:98-260`):**

1. **Flat information hierarchy** — Guild status header (row 0) shows rank, marks, roster, and frontier in a single dense string with no visual weight differentiation. The player cannot scan quickly.

2. **Missing guild progression context** — No indication of what's needed to advance guild rank. A new player cannot understand they need a Layer 3 breakthrough to reach Rank 2.

3. **Mission cards lack urgency signaling** — Completed missions and event-pending missions look nearly identical to active missions. The `⚡ Event pending!` suffix is correct in intent but gets buried in the line.

4. **Progress bar placement** — The bar is on row+1, which means you read the mission label, then look down for context. The opposite order from what's natural.

5. **Empty state misses opportunity** — "No active missions." + "[Tab] to Missions view" works but does nothing to orient a new player. No mention of what the Deep *is*, what Warband Marks are, or what to do first.

6. **Marks balance not prominent** — Marks appear in the header string. Before launching missions (which cost Marks), players need to see balance as a first-class status.

7. **Roster availability not surfaced** — No indication of how many mercs are free vs. on missions. Players can't tell if they have capacity to launch another mission.

8. **No mission slot capacity display** — Concurrent mission limit (from guild rank) is invisible. Players don't know if they've hit the cap.

### 1.2 Redesigned Hub View

#### Layout Structure

```
┌─ THE DEEP ─────────────────────────────────────────────────────────────┐
│ [Hub] [Missions] [Roster] [Layers] [Recruit]                           │  ← tab bar (existing)
│────────────────────────────────────────────────────────────────────────│
│ GUILD STATUS                                                           │  ← section A: guild block
│ Rank 2 — Sellswords        Mercs: 3/7   Missions: 1/1   ◆ 240 Marks   │
│ Frontier: Layer 3 (The Warrens)   Deepest: Layer 3                    │
│ Next rank needs: Layer 3 Breakthrough                                  │  ← only shown if can advance
│────────────────────────────────────────────────────────────────────────│
│ ACTIVE MISSIONS                                                        │  ← section B
│ ▶ [Expedition]  Layer 3 — The Warrens          ⚡ Event pending!       │  ← urgent state first
│   Squad: Gareth (Vanguard), Lyra (Scout)                              │
│   ██████████░░░░░░░░░░░░░░░░░  65%   2h 10m remaining                 │  ← bar then time
│ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │  ← soft divider
│   [Supply Run]  Layer 1 — The Shallows          43%   3h 45m left     │
│   Squad: Aldric (Medic)                                               │
│   ██████░░░░░░░░░░░░░░░░░░░░░░  43%                                    │
│────────────────────────────────────────────────────────────────────────│
│  [Tab] Switch view   [Enter] Select mission   [Esc] Close              │
└────────────────────────────────────────────────────────────────────────┘
```

#### Empty State (first-time player)

```
┌─ THE DEEP ─────────────────────────────────────────────────────────────┐
│ [Hub] [Missions] [Roster] [Layers] [Recruit]                           │
│────────────────────────────────────────────────────────────────────────│
│ GUILD STATUS                                                           │
│ Rank 1 — Freelancers        Mercs: 3/5   Missions: 0/1   ◆ 0 Marks    │
│ Frontier: Layer 1 (The Shallows)                                       │
│ Next rank needs: Layer 3 Breakthrough                                  │
│────────────────────────────────────────────────────────────────────────│
│                                                                        │
│                     No active missions.                                │
│                                                                        │
│        Your mercenary company is ready to descend.                     │
│        [Missions] tab → pick a mission and assign your squad.          │
│        Earn Warband Marks to grow your roster and buy infrastructure.  │
│                                                                        │
│────────────────────────────────────────────────────────────────────────│
│  [Tab] Switch view   [Enter] Select mission   [Esc] Close              │
└────────────────────────────────────────────────────────────────────────┘
```

#### Completed Mission (result waiting)

```
│ ✓ [Expedition]  Layer 2 — The Shallows         COMPLETE — collect!    │
│   Squad: Gareth, Lyra    ██████████████████████████████  100%         │
│   [Enter] to collect rewards                                           │
```

### 1.3 Information Hierarchy Principles Applied

**Visual weight hierarchy (lightest to heaviest):**
- Section labels (`GUILD STATUS`, `ACTIVE MISSIONS`): `Color::Rgb(80, 160, 220)` — deep blue
- Supporting context (frontier, deepest): `Color::DarkGray`
- Guild name: `Color::White`
- Marks balance: `Color::Yellow` with `◆` prefix for scannability
- Mission type: mission type color (existing `mission_type_color()`)
- Event pending badge: `Color::Yellow` with `⚡` prefix — rightmost, conspicuous
- Completed badge: `Color::Green` with `✓` prefix

**Ordering within mission card (top to bottom):**
1. Mission type + layer + tier + status badge (scan line)
2. Squad names (who is deployed)
3. Progress bar + percentage + time remaining (how far along)

The previous ordering had the scan line then bar then squad. Time remaining is more actionable than squad identity at a glance, so bar+time before squad.

### 1.4 Specific Code Changes — Hub View

**File:** `src/ui/deep_missions.rs`

#### Change 1: Guild status block (`render_hub()` lines 117-137)

Replace the two-line flat header with a structured 4-row guild block:

```rust
// ── Guild Status Block (rows 0-3) ──
let rank = deep.persistent.guild_rank;
let marks = deep.prestige.warband_marks;
let roster_count = deep.prestige.roster.len();
let available_mercs = deep.prestige.available_merc_count();
let max_roster = rank.max_roster() as usize;
let active_count = deep.prestige.active_mission_count() as u32;
let max_concurrent = rank.concurrent_missions();
let frontier = deep.persistent.frontier_layer();
let deepest = deep.persistent.deepest_layer_reached;

// Row 0: Section label
put_text(buffer, 0, 1, "GUILD STATUS", SECTION_LABEL_COLOR);

// Row 1: Rank name + key numeric stats
let marks_str = format!("\u{25c6} {} Marks", marks); // ◆ symbol
let rank_line = format!(
    "Rank {} \u{2014} {:12}  Mercs: {}/{}   Missions: {}/{}   {}",
    rank.0,
    rank.display_name(),
    roster_count, max_roster,
    active_count, max_concurrent,
    marks_str,
);
put_text(buffer, 1, 1, &rank_line, Color::White);
// Recolor marks ◆ in Yellow
let marks_col = rank_line.find('\u{25c6}').unwrap_or(0) as i32 + 1;
put_text(buffer, 1, marks_col, &marks_str, Color::Yellow);

// Row 2: Frontier info
let frontier_tier = crate::deep::LayerTier::from_layer(frontier);
let frontier_str = format!(
    "Frontier: Layer {} ({})   Deepest ever: Layer {}",
    frontier,
    frontier_tier.display_name(),
    deepest.max(1),
);
put_text(buffer, 2, 1, &frontier_str, Color::DarkGray);

// Row 3 (optional): Next rank requirement
if rank.can_advance() {
    if let Some(next) = rank.next() {
        if let Some(needed_layer) = next.required_breakthrough_layer() {
            let progress_text = format!(
                "Advance to {}: complete Layer {} Breakthrough",
                next.display_name(), needed_layer
            );
            put_text(buffer, 3, 1, &progress_text, Color::Rgb(50, 100, 60));
        }
    }
}

// ── Separator ──
let sep_row = 4i32;
let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
put_text(buffer, sep_row, 1, &sep, Color::Rgb(40, 60, 80));
```

#### Change 2: Mission card order and content (`render_hub()` active mission loop)

Reorder card to: status line → squad line → bar line. Add time remaining to bar line.

```rust
// Line 1: Mission type + layer + tier + status badge
let total_secs = (mission.ends_at - mission.started_at).num_seconds().max(1) as u64;
let elapsed_secs = (now - mission.started_at).num_seconds().max(0) as u64;
let remaining_secs = total_secs.saturating_sub(elapsed_secs);

let status_badge = match &mission.status {
    MissionStatus::EventPending => "  \u{26a1} EVENT PENDING",   // ⚡
    MissionStatus::Completed    => "  \u{2713} COMPLETE \u{2014} [Enter]", // ✓
    _                           => "",
};
let badge_color = match &mission.status {
    MissionStatus::EventPending => Color::Yellow,
    MissionStatus::Completed    => Color::Green,
    _                           => tc,
};

let tier_name = crate::deep::LayerTier::from_layer(mission.layer).display_name();
let line1 = format!(
    "{}[{}]  Layer {} \u{2014} {}{}",
    cursor,
    type_name,
    mission.layer,
    tier_name,
    status_badge,
);
put_text(buffer, row, 1, &line1, tc);
put_text(buffer, row, 1, cursor, if is_selected { Color::Cyan } else { Color::DarkGray });
// Recolor badge
if !status_badge.is_empty() {
    let badge_col = line1.find('\u{26a1}').or_else(|| line1.find('\u{2713}')).unwrap_or(0) as i32 + 1;
    put_text(buffer, row, badge_col, status_badge.trim(), badge_color);
}

// Line 2: Squad names
let squad_label = format!("  Squad: {}", squad_str);
put_text(buffer, row + 1, 1, &squad_label, Color::DarkGray);

// Line 3: Progress bar + % + time remaining
let pct = (progress * 100.0) as u32;
let bar_width = (width.saturating_sub(20)).min(28);
render_progress_bar(buffer, row + 2, 3, bar_width, progress, tc);
let time_str = if remaining_secs > 0 {
    format!("  {}%   {} left", pct, format_hours(remaining_secs))
} else {
    format!("  {}%   done", pct)
};
put_text(buffer, row + 2, 3 + bar_width as i32, &time_str, Color::DarkGray);

row += 4; // 3 content rows + 1 blank gap between missions
```

#### Change 3: Empty state copy

```rust
// Three-line empty state with action guidance
let mid = missions_top + content_height as i32 / 2;
put_text_centered(buffer, mid - 1, width, "No active missions.", Color::DarkGray);
put_text_centered(buffer, mid, width, "Your company is ready to descend.", Color::Rgb(60, 80, 120));
put_text_centered(buffer, mid + 1, width, "[Missions] tab \u{2192} pick a mission and assign squad.", Color::Rgb(50, 70, 100));
if deep.prestige.warband_marks == 0 {
    put_text_centered(buffer, mid + 2, width, "Supply Runs are free \u{2014} start there.", Color::Rgb(40, 80, 50));
}
```

#### Change 4: Add `SECTION_LABEL_COLOR` constant

At the top of `deep_missions.rs`:

```rust
const SECTION_LABEL_COLOR: Color = Color::Rgb(80, 160, 220);
```

#### Change 5: Compact mode adjustments

For S-tier, condense guild block to 2 rows:
- Row 0: `Rank 1 Freelancers  3/5 Mercs  0/1 Missions  ◆240M`
- Row 1: `Frontier: L3 Warrens` (no next-rank progression text)
- Skip separator, go straight to missions

#### Change 6: Tab bar state indicators (NEW — from audit P1.3)

**File:** `src/ui/deep_scene.rs` — `render_tab_bar()` function (lines 119-138)

The current tab bar shows only the active tab highlighted. Add compact state badges after each label:

| Tab | Badge condition | Badge | Color |
|-----|----------------|-------|-------|
| Hub | N results await | `✓N` | `Green` |
| Hub | N events pending | `⚡N` | `Yellow` |
| Missions | N missions available | `·N` | `Cyan` |
| Roster | N mercs injured/lost | `!N` | `Yellow` |
| Recruit | Pool has candidates | `·` | `Cyan` |

The badge is appended inside the `[Label]` brackets: `[Hub⚡2]` or `[Roster!1]`.

```rust
fn render_tab_bar(buffer: &mut [Vec<SceneCell>], width: usize, active: DeepView, deep: &DeepState) {
    let mut col = 1i32;
    for (i, &tab) in DeepView::TABS.iter().enumerate() {
        if i > 0 {
            put_text(buffer, 0, col, " ", Color::DarkGray);
            col += 1;
        }

        // Compute badge for this tab
        let (badge, badge_color) = match tab {
            DeepView::Hub => {
                let events = deep.prestige.active_missions.iter()
                    .filter(|m| m.has_pending_event()).count();
                let results = deep.prestige.pending_results.len();
                if events > 0 {
                    (format!("\u{26a1}{}", events), Color::Yellow)
                } else if results > 0 {
                    (format!("\u{2713}{}", results), Color::Green)
                } else {
                    (String::new(), Color::DarkGray)
                }
            }
            DeepView::NewMission => {
                let n = deep.prestige.available_missions.len();
                if n > 0 { (format!("\u{00b7}{}", n), Color::Cyan) }
                else { (String::new(), Color::DarkGray) }
            }
            DeepView::Roster => {
                let injured = deep.prestige.roster.iter()
                    .filter(|m| matches!(m.status, MercStatus::Injured { .. } | MercStatus::Lost))
                    .count();
                if injured > 0 { (format!("!{}", injured), Color::Yellow) }
                else { (String::new(), Color::DarkGray) }
            }
            DeepView::Recruit => {
                let n = deep.prestige.recruit_pool.candidates.len();
                if n > 0 { ("\u{00b7}".to_string(), Color::Cyan) }
                else { (String::new(), Color::DarkGray) }
            }
            _ => (String::new(), Color::DarkGray),
        };

        let label = tab.tab_label();
        let full = if badge.is_empty() {
            format!("[{}]", label)
        } else {
            format!("[{}{}]", label, badge)
        };
        let tab_color = if tab == active {
            Color::Rgb(80, 160, 220)
        } else {
            Color::DarkGray
        };
        put_text(buffer, 0, col, &full, tab_color);
        // Overcolor badge portion only when not active
        if !badge.is_empty() && tab != active {
            let badge_col = col + 1 + label.len() as i32;
            put_text(buffer, 0, badge_col, &badge, badge_color);
        }
        col += full.len() as i32;
    }

    // Separator (existing logic, unchanged)
    let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
    let remaining = (width as i32 - col - 1).max(0) as usize;
    put_text(buffer, 0, col + 1, &sep[..remaining.min(sep.len())], Color::Rgb(40, 60, 80));
}
```

Note: `render_tab_bar()` signature gains a `deep: &DeepState` parameter. Update the call site in `render_deep_overlay()` accordingly.

---

## Part 2: Mission Assignment Flow Redesign

### 2.1 Current State Problems

**Identified issues in `render_new_mission()` and `render_new_mission_split()` (`deep_missions.rs:264-598`):**

1. **Mission type mystery** — Each mission in the list shows `[Supply Run]  L1  2h  Safe` but never explains what a Supply Run *is*. Players unfamiliar with the system must Tab to Roster, look around, and guess. The `description` field on `AvailableMission` exists but is never rendered.

2. **Power requirement ambiguity** — `Requires: Min Power 25` tells players a number but not whether their current squad meets it. The ratio feedback exists in the summary row (at the bottom) but the two pieces of information are visually separated, forcing vertical scanning.

3. **Cost balance not co-located** — Mission cost appears in the list line as `25M` but current Marks balance only appears in the Hub view. Players leave the Missions tab, note their balance, come back, and verify — unnecessary cognitive round-trip.

4. **Archetype requirements buried** — "Arcanist required" appears below duration and cost in the detail panel. Required archetypes gate entire missions and should be prominently flagged, especially when the player lacks that archetype.

5. **Squad assignment flow is two-phase but feels one-phase** — The list → squad picker flow is correct but poorly communicated. The split-panel design shows the squad picker immediately in the right panel without indicating that [Enter] activates it and changes the cursor target. New players think the list is the whole experience.

6. **No archetype squad summary** — When building a squad, players see names and levels but don't see a quick archetype summary. If a mission wants a Saboteur, players need to remember which merc is which archetype.

7. **Success probability labels are inconsistent** — "60-90% success" and "Good odds" say the same thing redundantly. "Overpowered — 95% success, faster" is the most useful line but feels like a bonus, not a feature.

8. **No feedback on duration modifiers** — The displayed duration (`2h`) is the base duration. Players don't know that their Outpost on Layer 1 is already factored in, or that adding a Saboteur would reduce it further.

9. **Available Marks not shown in Missions view** — If cost > 0, the number shows in the list. But current balance is invisible. A player with 20 Marks looking at a 25-Mark mission has no direct affordability signal.

10. **Construction mission type is opaque** (audit P0.3) — `MissionType::Construction(Infrastructure::Outpost)` renders as `"Construction"` in the list. Players cannot see what infrastructure they are building without opening the detail panel.

11. **Power ratio lacks percentage context** (audit P1.4) — `Power: 32/25` is shown but `128%` — the number that maps to a success band — is never computed or displayed. Players must do math to understand whether 32 is well over or barely over threshold.

12. **"No missions available" message is misleading** (audit bug 3.3) — On a fresh prestige with nothing queued, "Complete active missions to refresh the pool" gives wrong guidance. The pool refresh is time-based and independent of mission completion.

13. **Flash message overlaps content** (audit bug 3.7) — `flash_message` is rendered at `height - 2` but `content_bottom` is also `height - 2`. Long mission lists can write over the flash message row before it can be read.

### 2.2 Redesigned Mission Assignment Flow

#### Phase 1: Mission List (before Enter)

```
┌ AVAILABLE MISSIONS ──────────────────┬─ MISSION DETAIL ────────────────┐
│ AVAILABLE                            │ Layer 3 — The Warrens           │
│                                      │                                 │
│ ▶ [Expedition]  L3  10h  Medium  20M │ Expedition                      │
│   [Supply Run]  L1  2h   Safe    —   │ Primary progression mission.    │
│   [Recon]       L3  6h   Low     5M  │ Longer, riskier than Recon,     │
│                                      │ earns more Marks and XP.        │
│                                      │                                 │
│                                      │ Duration:  10h (base)           │
│                                      │ Risk:      Medium               │
│                                      │ Cost:      20 Marks  (have 240) │
│                                      │ Reward:    Marks + XP + items   │
│                                      │                                 │
│                                      │ Requires:                       │
│                                      │   Min Power  100                │
│                                      │ ⚠ Arcanist required (none in    │
│                                      │   roster!) — can still attempt  │
│                                      │ ★ Scout recommended             │
│                                      │                                 │
│                                      │ [Enter] to assign squad         │
└──────────────────────────────────────┴─────────────────────────────────┘
  [↑/↓] Select Mission   [Enter] Assign Squad   [Esc] Back         ◆ 240 M
```

#### Phase 2: Squad Assignment (after Enter)

```
┌ ASSIGN SQUAD ────────────────────────┬─ SQUAD SUMMARY ─────────────────┐
│ Expedition  Layer 3  10h  20M        │ Cost: 20 Marks   Balance: 240   │
│                                      │                                 │
│ [↑/↓] Select   [Space] Toggle        │ Squad Power: 68 / 100           │
│                                      │ ████████░░░░░░░░░░░░░░░░░░░░   │
│ [ ] Gareth      Vanguard  L3  P:20   │ Risky — ~30% success            │
│ [✓] Lyra        Scout     L2  P:14   │                                 │
│ [ ] Aldric      Medic     L1  P: 8   │ Archetypes in squad:            │
│ [ ] Theron      Arcanist  L4  P:16 ← │   Scout (Lyra)                 │
│     Vex         Saboteur  L2  P:12   │   (!) Arcanist required         │
│     (injured: 2 missions)            │   ★  Scout recommended — present│
│                                      │                                 │
│                                      │ Add Theron (Arcanist) to meet   │
│                                      │ the requirement. Power: 84/100  │
│                                      │ Risky — ~30% (Arcanist present) │
│                                      │                                 │
│                                      │ [Enter] Launch Mission          │
└──────────────────────────────────────┴─────────────────────────────────┘
  [Space] Toggle   [Enter] Launch Mission   [Esc] Cancel         ◆ 220 M
```

### 2.3 Phase 1 Design Specifics

#### Mission list line format

Current: `▶ [Supply Run]  L1  2h  Safe`
Proposed: `▶ [Supply Run]  L1  2h   Safe   —`

For compactness the list stays the same width. The innovation is in the **detail panel**.

#### Detail panel improvements

**New fields to render:**

1. **Mission description** — Render `available_mission.description` field (already exists in `AvailableMission` struct, currently unused in UI). Wrap at 2 lines max with `detail_inner_w` constraint.

2. **Marks balance co-located with cost:**
   ```
   Cost:     20 Marks   (have 240)
   ```
   - If `marks >= cost`: render `(have N)` in `Color::Rgb(60, 180, 80)` (soft green)
   - If `marks < cost`: render `(have N — INSUFFICIENT)` in `Color::LightRed`
   - If `marks_cost == 0`: render `Cost:     Free`

3. **Archetype requirement prominence:**
   - Required archetype: Show with `⚠` prefix in `Color::Yellow` if archetype missing from roster, `Color::Green` if present
   - Recommended archetype: Show with `★` prefix in `Color::Cyan` if present, `Color::DarkGray` if absent
   - Check the full roster (not just available mercs) since players may want to know if they have the archetype at all

4. **"[Enter] to assign squad" hint** at bottom of detail panel, `Color::Rgb(50, 100, 50)`

5. **Duration modifier hint** (when infrastructure active):
   ```
   Duration:  10h   (Outpost: -25% → 7.5h effective)
   ```
   Query `deep.persistent.layer_record(m.layer)` for infrastructure and familiarity; show effective time if modifiers exist.

### 2.4 Phase 2 Design Specifics

#### Left panel changes (merc list during squad staging)

Current: Shows all roster mercs including unavailable.
Proposed: Separate available vs. unavailable with a visual group break.

```
Available:
  [✓] Gareth    Vanguard  L3  Pwr:20
  [ ] Lyra      Scout     L2  Pwr:14
  [ ] Theron    Arcanist  L4  Pwr:16
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
Unavailable:
      Vex       Saboteur  (on mission)
      Aldric    Medic     (injured: 2)
```

The divider `─ ─ ─` uses `Color::Rgb(40, 60, 80)`. Unavailable mercs are not selectable (cursor skips them).

#### Right panel (squad summary — new)

Replace the current bottom-of-panel power summary with a dedicated right panel that updates reactively as mercs are toggled.

**Sections:**

1. **Cost and balance header:**
   ```
   Cost: 20 Marks     Balance: 240
   ```
   Color cost red if insufficient, green if sufficient.

2. **Squad power meter:**
   ```
   Squad Power:  68 / 100
   ████████████████░░░░░░░░░░░░░░░   68%
   ```
   Bar color matches forecast: Green (good), Yellow (risky), Red (fail).

3. **Success forecast (prominent):**
   ```
   Risky — ~30% success
   ```
   Use large labels. For safe missions: `Always succeeds — no power check`.

4. **Archetype summary:**
   List archetypes present in staged squad. Highlight required/recommended.
   ```
   Archetypes in squad:
     Scout (Lyra)
   (!) Arcanist required — not present
   ★  Scout recommended — present
   ```
   - `(!)` prefix: `Color::Yellow`
   - `★` prefix: `Color::Cyan`

5. **Smart hint line** (contextual tip):
   - If required archetype missing and is in roster: `Add [Name] ([Arch]) to meet requirement.`
   - If squad empty: `Select mercs with [Space]`
   - If squad overpowered: `Overpowered — mission will be faster!`
   - If insufficient marks: `Not enough Marks — earn more via Supply Runs`

6. **Launch action hint:**
   ```
   [Enter] Launch Mission
   ```
   `Color::Rgb(60, 180, 80)` when squad is non-empty, `Color::DarkGray` when empty.

### 2.5 Success Probability Display Improvements

Current labels are inconsistent across compact and split views. Standardize to:

| Power Ratio | Label | Color |
|---|---|---|
| Squad empty | `Select mercs with [Space]` | `DarkGray` |
| Safe mission type | `Always succeeds` | `Green` |
| `>= 150%` of min | `Overpowered — 95% + faster` | `Rgb(80, 220, 120)` |
| `>= 100%` of min | `Good — 60-90% success` | `Green` |
| `>= 75%` of min | `Risky — ~30% success` | `Yellow` |
| `< 75%` of min | `Likely to fail` | `LightRed` |

The label should always be on its own line, never appended as a suffix.

### 2.6 Specific Code Changes — Mission Assignment

**File:** `src/ui/deep_missions.rs`

#### Change 1: Add `render_mission_detail_phase1()` helper

Extract the right panel of `render_new_mission_split()` into a dedicated function that renders the full Phase 1 detail:

```rust
fn render_mission_detail_phase1(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    ui: &DeepUiState,
    mission: &AvailableMission,
    detail_inner_left: i32,
    detail_inner_w: i32,
    content_top: i32,
    content_bottom: i32,
) {
    let mut row = content_top;

    // Layer + tier heading
    let tier_name = crate::deep::LayerTier::from_layer(mission.layer).display_name();
    put_text(buffer, row, detail_inner_left, &format!("Layer {} \u{2014} {}", mission.layer, tier_name), Color::White);
    row += 1;

    // Mission type name (colored)
    let tc = mission_type_color(mission.mission_type);
    put_text(buffer, row, detail_inner_left, mission.mission_type.display_name(), tc);
    row += 1;

    // Description (word-wrapped, 2 lines max)
    if !mission.description.is_empty() {
        let max_w = (detail_inner_w - 1).max(10) as usize;
        let words: Vec<&str> = mission.description.split_whitespace().collect();
        let mut line_buf = String::new();
        let mut lines_rendered = 0;
        for word in &words {
            if lines_rendered >= 2 { break; }
            if line_buf.len() + word.len() + 1 > max_w && !line_buf.is_empty() {
                put_text(buffer, row, detail_inner_left, &line_buf, Color::DarkGray);
                row += 1;
                lines_rendered += 1;
                line_buf.clear();
            }
            if !line_buf.is_empty() { line_buf.push(' '); }
            line_buf.push_str(word);
        }
        if !line_buf.is_empty() && lines_rendered < 2 {
            put_text(buffer, row, detail_inner_left, &line_buf, Color::DarkGray);
            row += 1;
        }
    }
    row += 1;

    // Duration — show effective if modifiers apply
    let layer_record = deep.persistent.layer_record(mission.layer);
    let duration_reduction = layer_record.map(|l| l.total_duration_reduction()).unwrap_or(0.0);
    let effective_secs = (mission.duration_secs as f64 * (1.0 - duration_reduction)) as u64;
    let dur_str = if duration_reduction > 0.01 {
        format!("Duration:  {}   (\u{2192} {} effective)", format_hours(mission.duration_secs), format_hours(effective_secs))
    } else {
        format!("Duration:  {}", format_hours(mission.duration_secs))
    };
    put_text(buffer, row, detail_inner_left, &dur_str, Color::DarkGray);
    row += 1;

    // Risk
    let risk_str = format!("Risk:      {}", risk_label(mission.mission_type.risk_tier()));
    put_text(buffer, row, detail_inner_left, &risk_str, risk_color(mission.mission_type.risk_tier()));
    row += 1;

    // Cost with affordability
    let marks = deep.prestige.warband_marks;
    if mission.marks_cost > 0 {
        let (afford_str, afford_color) = if marks >= mission.marks_cost {
            (format!("   (have {})", marks), Color::Rgb(60, 180, 80))
        } else {
            (format!("   (have {} \u{2014} INSUFFICIENT)", marks), Color::LightRed)
        };
        put_text(buffer, row, detail_inner_left, &format!("Cost:      {} Marks", mission.marks_cost), Color::Yellow);
        put_text(buffer, row, detail_inner_left + format!("Cost:      {} Marks", mission.marks_cost).len() as i32, &afford_str, afford_color);
    } else {
        put_text(buffer, row, detail_inner_left, "Cost:      Free", Color::Rgb(60, 180, 80));
    }
    row += 1;

    put_text(buffer, row, detail_inner_left, "Reward:    Marks + XP + items", Color::DarkGray);
    row += 2;

    // Requirements section
    put_text(buffer, row, detail_inner_left, "Requires:", Color::Cyan);
    row += 1;

    // Power (always shown)
    put_text(buffer, row, detail_inner_left, &format!("  Min Power  {}", mission.min_squad_power), Color::White);
    row += 1;

    // Required archetype — check against full roster
    if let Some(req_arch) = mission.required_archetype {
        let in_roster = deep.prestige.roster.iter().any(|m| m.archetype == req_arch);
        let (prefix, label_color) = if in_roster {
            ("\u{2713} ", Color::Green)
        } else {
            ("\u{26a0} ", Color::Yellow)
        };
        let suffix = if !in_roster { " (not in roster!)" } else { " (required)" };
        put_text(
            buffer, row, detail_inner_left,
            &format!("  {}{}{}", prefix, req_arch.display_name(), suffix),
            label_color,
        );
        row += 1;
    }

    // Recommended archetype
    if let Some(rec_arch) = mission.recommended_archetype {
        let in_roster = deep.prestige.roster.iter().any(|m| m.archetype == rec_arch);
        let (prefix, color) = if in_roster { ("\u{2605} ", Color::Cyan) } else { ("  ", Color::DarkGray) };
        put_text(
            buffer, row, detail_inner_left,
            &format!("  {}{} recommended", prefix, rec_arch.display_name()),
            color,
        );
        row += 1;
    }
    row += 1;

    // Action hint at bottom
    let hint_row = (content_bottom - 2).max(row);
    put_text(buffer, hint_row, detail_inner_left, "[Enter] Assign squad \u{2192}", Color::Rgb(50, 120, 60));
}
```

#### Change 2: Add `render_squad_summary_panel()` helper

New right panel for Phase 2:

```rust
fn render_squad_summary_panel(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    ui: &DeepUiState,
    mission: &AvailableMission,
    detail_inner_left: i32,
    detail_inner_w: i32,
    content_top: i32,
    content_bottom: i32,
) {
    let squad_power: u32 = ui.staged_squad.iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.effective_power())
        .sum();
    let min = mission.min_squad_power;
    let marks = deep.prestige.warband_marks;
    let is_safe = matches!(mission.mission_type, MissionType::SupplyRun | MissionType::Construction(_));
    let can_afford = marks >= mission.marks_cost;

    let mut row = content_top;

    // Cost + balance
    if mission.marks_cost > 0 {
        let cost_color = if can_afford { Color::Green } else { Color::LightRed };
        put_text(buffer, row, detail_inner_left,
            &format!("Cost: {} Marks     Balance: {}", mission.marks_cost, marks),
            Color::White);
        // Recolor cost portion
        put_text(buffer, row, detail_inner_left + 6, &format!("{}", mission.marks_cost), cost_color);
    }
    row += 2;

    // Power meter
    let power_str = format!("Squad Power:  {} / {}", squad_power, min);
    let ratio = if min == 0 { 1.0 } else { squad_power as f64 / min as f64 };
    let (bar_color, forecast_label, forecast_color) = if ui.staged_squad.is_empty() {
        (Color::DarkGray, "Select mercs with [Space]", Color::DarkGray)
    } else if is_safe {
        (Color::Green, "Always succeeds", Color::Green)
    } else if ratio >= 1.5 {
        (Color::Rgb(80, 220, 120), "Overpowered \u{2014} 95% + faster!", Color::Rgb(80, 220, 120))
    } else if ratio >= 1.0 {
        (Color::Green, "Good \u{2014} 60-90% success", Color::Green)
    } else if ratio >= 0.75 {
        (Color::Yellow, "Risky \u{2014} ~30% success", Color::Yellow)
    } else {
        (Color::LightRed, "Likely to fail", Color::LightRed)
    };

    put_text(buffer, row, detail_inner_left, &power_str, Color::White);
    row += 1;
    let bar_w = (detail_inner_w as usize).saturating_sub(2).min(24);
    render_progress_bar(buffer, row, detail_inner_left, bar_w, ratio.min(1.0), bar_color);
    row += 1;
    put_text(buffer, row, detail_inner_left, forecast_label, forecast_color);
    row += 2;

    // Archetype summary
    put_text(buffer, row, detail_inner_left, "Archetypes in squad:", Color::Cyan);
    row += 1;

    let squad_archetypes: Vec<crate::deep::MercArchetype> = ui.staged_squad.iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.archetype)
        .collect();

    if squad_archetypes.is_empty() {
        put_text(buffer, row, detail_inner_left, "  (none selected)", Color::DarkGray);
        row += 1;
    } else {
        // Deduplicate and list present archetypes
        let mut seen = std::collections::HashSet::new();
        for &arch in &squad_archetypes {
            if seen.insert(arch) {
                if row >= content_bottom - 3 { break; }
                let name = deep.prestige.roster.iter()
                    .find(|m| m.archetype == arch && ui.staged_squad.contains(&m.id))
                    .map(|m| m.name.as_str())
                    .unwrap_or("");
                put_text(buffer, row, detail_inner_left,
                    &format!("  {} ({})", arch.display_name(), name),
                    archetype_color(arch));
                row += 1;
            }
        }
    }

    // Required archetype check
    if let Some(req_arch) = mission.required_archetype {
        let req_present = squad_archetypes.contains(&req_arch);
        let (prefix, color, suffix) = if req_present {
            ("\u{2713} ", Color::Green, " required \u{2014} present")
        } else {
            ("(!) ", Color::Yellow, " required \u{2014} missing!")
        };
        if row < content_bottom - 3 {
            put_text(buffer, row, detail_inner_left,
                &format!("{}{}{}", prefix, req_arch.display_name(), suffix),
                color);
            row += 1;
        }
    }

    // Recommended archetype check
    if let Some(rec_arch) = mission.recommended_archetype {
        let rec_present = squad_archetypes.contains(&rec_arch);
        let (prefix, color, suffix) = if rec_present {
            ("\u{2605} ", Color::Cyan, " recommended \u{2014} present")
        } else {
            ("  ", Color::DarkGray, " recommended")
        };
        if row < content_bottom - 3 {
            put_text(buffer, row, detail_inner_left,
                &format!("{}{}{}", prefix, rec_arch.display_name(), suffix),
                color);
            row += 1;
        }
    }

    // Smart contextual hint
    row += 1;
    let hint = if !can_afford && mission.marks_cost > 0 {
        Some(("Earn Marks via Supply Runs (free)", Color::DarkGray))
    } else if ui.staged_squad.is_empty() {
        Some(("Select mercs with [Space]", Color::DarkGray))
    } else if ratio >= 1.5 {
        Some(("Overpowered \u{2014} mission will complete faster!", Color::Rgb(80, 220, 120)))
    } else if let Some(req_arch) = mission.required_archetype {
        let req_present = squad_archetypes.contains(&req_arch);
        if !req_present {
            // Find the archetype in roster
            let merc_with_arch = deep.prestige.roster.iter()
                .find(|m| m.archetype == req_arch && m.is_available());
            if let Some(m) = merc_with_arch {
                Some((format!("Add {} ({}) to meet requirement", m.name, req_arch.display_name()).into(), Color::Yellow))
            } else {
                Some(("Recruit a {} — check [Recruit] tab".to_string().into(), Color::Yellow))
            }
        } else { None }
    } else { None };

    if let Some((hint_text, hint_color)) = hint {
        let hint_row = (content_bottom - 3).max(row);
        if hint_row < content_bottom {
            put_text(buffer, hint_row, detail_inner_left, &hint_text.to_string(), hint_color);
        }
    }

    // Launch action at bottom
    let launch_row = content_bottom - 1;
    let (launch_color, launch_label) = if ui.staged_squad.is_empty() {
        (Color::DarkGray, "[Enter] Launch Mission")
    } else {
        (Color::Rgb(60, 180, 80), "[Enter] Launch Mission")
    };
    put_text(buffer, launch_row, detail_inner_left, launch_label, launch_color);
}
```

#### Change 3: Update `render_new_mission_split()` to use phase-specific panels

```rust
// In render_new_mission_split(), replace the detail rendering block:
let staging = ui.staging_mission_index.is_some();
let detail_idx = ui.staging_mission_index.unwrap_or(ui.selected_index);
let Some(m) = available.get(detail_idx) else { return; };

if !staging {
    render_mission_detail_phase1(buffer, deep, ui, m, detail_inner_left, detail_inner_w, content_top, content_bottom);
} else {
    render_squad_summary_panel(buffer, deep, ui, m, detail_inner_left, detail_inner_w, content_top, content_bottom);
    // Also update left panel heading
    put_text(buffer, content_top, detail_inner_left - list_width as i32 + 1, "ASSIGN SQUAD", SECTION_LABEL_COLOR);
}
```

#### Change 4: Left panel merc list with group separator (Phase 2)

Replace the current single-pass merc list in Phase 2 with grouped rendering:

```rust
// In render_new_mission_split() merc list section (during squad staging):
let available_roster: Vec<_> = deep.prestige.roster.iter().enumerate()
    .filter(|(_, m)| m.is_available()).collect();
let unavailable_roster: Vec<_> = deep.prestige.roster.iter().enumerate()
    .filter(|(_, m)| !m.is_available()).collect();

// Render available mercs
let mut row = squad_label_row + 1;
for (ri, merc) in &available_roster {
    if row >= content_bottom - 2 { break; }
    let is_sel = *ri == ui.selected_index;
    // ... render with cursor and checkbox ...
    row += 1;
}

// Group separator
if !unavailable_roster.is_empty() && row < content_bottom - 2 {
    let sep = "\u{2500} \u{2500} \u{2500}".repeat(list_width / 6);
    put_text(buffer, row, 1, &sep[..list_width.min(sep.len())], Color::Rgb(40, 60, 80));
    row += 1;
    // Render unavailable (not selectable, cursor does not stop here)
    for (_, merc) in &unavailable_roster {
        if row >= content_bottom { break; }
        let avail_str = match &merc.status {
            MercStatus::OnMission(_) => "on mission".to_string(),
            MercStatus::Injured { missions_remaining } => format!("injured: {}", missions_remaining),
            MercStatus::Lost => "lost".to_string(),
            _ => String::new(),
        };
        put_text(buffer, row, 3,
            &format!("  {:14} {:8} ({})", &merc.name[..merc.name.len().min(14)], merc.archetype.display_name(), avail_str),
            Color::Rgb(50, 60, 70));
        row += 1;
    }
}
```

#### Change 5: Footer — show Marks balance

Add Marks balance to footer in both Phase 1 and Phase 2:

```rust
// At start of render_new_mission(), compute marks display
let marks_display = format!("\u{25c6} {} M", deep.prestige.warband_marks);
// Render in footer right-aligned
let footer_col = (width as i32 - marks_display.len() as i32 - 2).max(1);
put_text(buffer, height as i32 - 1, footer_col, &marks_display, Color::Yellow);
```

#### Change 6: Input — skip unavailable mercs during squad assignment

In `handle_squad_assignment()` (`deep_input.rs:157-281`), filter mercs so cursor only lands on available ones:

```rust
// Replace available_mercs computation to use only available mercs for navigation
let available_mercs: Vec<(usize, u64)> = deep_state.prestige.roster.iter()
    .enumerate()
    .filter(|(_, m)| m.is_available())
    .map(|(i, m)| (i, m.id))
    .collect();
// Navigation and Space toggle operate on this filtered list
// selected_index refers to position within available_mercs, not full roster
```

#### Change 7: Construction mission type label (P0.3 from audit)

**File:** `src/ui/deep_missions.rs` — compact list line 358, split list line 468

In both compact and split mission list renderers, replace:
```rust
let type_name = m.mission_type.display_name();
```
with:
```rust
let type_label: String = match m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
// Use type_label wherever type_name was previously used in the format! call
```

This makes `"Build Outpost"`, `"Build Watchtower"` etc. appear in the list, letting players immediately know what they are queueing without opening the detail panel.

#### Change 8: Empty mission pool message (audit bug 3.3)

**File:** `src/ui/deep_missions.rs` — `render_new_mission()` lines 303-318

Replace the misleading "Complete active missions to refresh the pool" with accurate guidance:

```rust
if available.is_empty() {
    let mid = content_top + content_height as i32 / 2;
    put_text_centered(buffer, mid - 1, width, "No missions available.", Color::DarkGray);

    // Accurate guidance based on actual state
    let active_count = deep.prestige.active_mission_count();
    if active_count == 0 && deep.prestige.roster.is_empty() {
        put_text_centered(buffer, mid, width, "Recruit mercenaries in [Recruit] tab first.", Color::Rgb(50, 70, 100));
    } else if active_count > 0 {
        put_text_centered(buffer, mid, width, "Mission pool refreshes over time.", Color::Rgb(50, 70, 100));
        put_text_centered(buffer, mid + 1, width, "Check back after your current missions complete.", Color::Rgb(40, 55, 80));
    } else {
        put_text_centered(buffer, mid, width, "Mission pool refreshes periodically.", Color::Rgb(50, 70, 100));
        put_text_centered(buffer, mid + 1, width, "Return in a few minutes.", Color::Rgb(40, 55, 80));
    }
    return;
}
```

#### Change 9: Flash message positioning fix (audit bug 3.7)

**File:** `src/ui/deep_missions.rs` — `render_new_mission()` lines 292-299

Reserve one extra row so flash messages are never overwritten by content:

```rust
// Flash row is height - 2; content must stop at height - 3
if let Some(msg) = &ui.flash_message {
    put_text(buffer, height as i32 - 2, 1, msg, Color::LightRed);
}
put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);

let content_top = 0i32;
let content_bottom = height as i32 - 3; // was height - 2; now leaves room for flash + footer
```

#### Change 10: Power ratio percentage display (P1.4 from audit)

In `render_squad_summary_panel()` (new helper) and in the compact power summary row, show the power percentage inline:

```rust
let ratio_pct = if min == 0 { 999u32 } else { squad_power * 100 / min };
let power_str = if min == 0 || is_safe {
    format!("Squad Power:  {}", squad_power)
} else {
    format!("Squad Power:  {} / {}  ({}%)", squad_power, min, ratio_pct)
};
put_text(buffer, row, detail_inner_left, &power_str, Color::White);
// Recolor the percentage based on success band
let pct_color = if ratio_pct >= 150 { Color::Rgb(80, 220, 120) }
    else if ratio_pct >= 100 { Color::Green }
    else if ratio_pct >= 75 { Color::Yellow }
    else { Color::LightRed };
// Find position of "(N%)" and recolor that segment
let pct_str = format!("({}%)", ratio_pct);
if let Some(pos) = power_str.find('(') {
    put_text(buffer, row, detail_inner_left + pos as i32, &pct_str, pct_color);
}
```

---

## Part 3: Compact Mode (S-tier) Adaptations

For S-tier (width < 60 or height tier S), the designs simplify to single-column:

### Hub compact

```
Rank 1 Freelancers  3/5  0/1 concurrent  ◆240M
Frontier: L3 Warrens
─────────────────────────────────────────
MISSIONS
▶ [Expedition] L3  65%  ⚡ Event!
  Gareth, Lyra  ████████░░  2h 10m left
  [Supply Run] L1  43%
  Aldric  █████░░░░░  3h 45m left
─────────────────────────────────────────
[Tab]Switch [Enter]Select [Esc]Close
```

### Mission compact (Phase 1)

Single column mission list with description on selection:

```
▶ [Expedition] L3 10h Medium 20M
  [Supply Run] L1 2h  Safe  Free
  [Recon]      L3 6h  Low   5M

Selected: Expedition — primary progression mission.
Min Power: 100   ⚠ Arcanist required (missing!)
Cost: 20M  (have 240)  [Enter] Assign squad
```

### Mission compact (Phase 2)

```
[Expedition] Layer 3  10h  20M

[✓] Gareth   Vanguard  L3
[ ] Lyra     Scout     L2
[ ] Theron   Arcanist  L4
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
    Vex      (on mission)
    Aldric   (injured: 2)

Power: 20/100  Risky 30%
(!) Arcanist required — add Theron
[Space] Toggle  [Enter] Launch  [Esc] Cancel
```

---

## Part 4: Summary of All File Changes

### `src/ui/deep_missions.rs`

| Change | Function | Lines | Description |
|--------|----------|-------|-------------|
| Guild status block | `render_hub()` | 117-137 | 4-row structured block with marks, capacity, next-rank hint |
| Mission card order | `render_hub()` | 175-251 | Reorder to: type+status → squad → bar+time |
| Empty state copy (Hub) | `render_hub()` | 147-151 | 4-line actionable empty state |
| `SECTION_LABEL_COLOR` constant | module level | — | `Color::Rgb(80, 160, 220)` |
| Footer Marks display | `render_new_mission()` | 291-295 | Right-aligned `◆ N M` in footer |
| Phase 1 detail panel | New fn `render_mission_detail_phase1()` | — | Full detail with description, affordability, archetype warning |
| Phase 2 squad summary | New fn `render_squad_summary_panel()` | — | Reactive squad summary with power meter and archetype checks |
| Phase dispatch in split | `render_new_mission_split()` | 489-597 | Route to phase1 or phase2 helper |
| Left panel grouped mercs | `render_new_mission_split()` | 538-562 | Available/unavailable groups with separator |
| Compact Phase 1 detail | `render_new_mission_compact()` | 342-426 | Show description + affordability under selected mission |
| Construction label fix (P0.3) | `render_new_mission_compact()`, `render_new_mission_split()` | 358, 468 | Use `"Build {infra}"` instead of `"Construction"` |
| Power ratio percentage (P1.4) | `render_squad_summary_panel()`, compact power line | new fn | Show `(128%)` after `Power: N/M` |
| Empty pool message fix (bug 3.3) | `render_new_mission()` | 303-318 | Replace misleading "complete missions" hint with accurate guidance |
| Flash message positioning fix (bug 3.7) | `render_new_mission()` | 292-295 | Reserve flash row at `height - 2`, set `content_bottom = height - 3` |

### `src/ui/deep_scene.rs`

| Change | Function | Lines | Description |
|--------|----------|-------|-------------|
| Tab bar state badges (P1.3) | `render_tab_bar()` | 119-138 | Add ⚡/✓/·/! badges to tab labels; add `deep: &DeepState` parameter |

### `src/input/deep_input.rs`

| Change | Function | Lines | Description |
|--------|----------|-------|-------------|
| Available-only cursor | `handle_squad_assignment()` | 162-168 | Filter roster to only available mercs for navigation |
| Warn on missing arch | `handle_squad_assignment()` | Enter branch | Flash message if required archetype missing on launch |

### No changes needed to:
- `deep_events.rs` — Event view is solid (bug 3.5 auto-resolve constant is low priority)
- `deep_roster.rs` — Addressed in Task #4
- `deep_layers.rs` — Addressed in Task #5
- `types.rs` — All required data structures already exist

---

## Part 5: Design Rationale

### Why group-separated merc list during squad staging?

Players making squad decisions need to answer two questions: "Who is available?" and "Who can I add?" The current flat list mixes available and unavailable mercs. Grouping eliminates scanning overhead and reduces the chance of accidentally fixating on mercs that cannot be selected.

### Why co-locate Marks balance with mission cost?

Affordability is a binary go/no-go decision made immediately when viewing a mission. Requiring players to navigate to Hub → check balance → navigate back → assess mission breaks flow. The pattern of showing balance inline (similar to how shops show "You have X gold") is universal in RPG UIs.

### Why show required archetype in mission list vs. buried in details?

Required archetypes are hard constraints, not preferences. A player without a Saboteur who is browsing missions should immediately see which missions are gated. The `⚠` prefix on the list line (for missing required archetypes) lets players filter by eye without reading each detail panel.

### Why reorder Hub mission cards (bar before squad)?

Progress bar + time remaining answers "when does this come back?" — the most frequent scan question for idle players checking in. Squad identity answers "who is deployed?" — important context but not the first question. The reordering matches the mental model: players glance at The Deep to see progress, not to remember roster assignments.

### Why a Phase 1 / Phase 2 split concept?

The current `staging_mission_index` approach already encodes two phases but renders them ambiguously. Making the phase explicit in rendering (different right panel content, different heading) reduces the cognitive overhead of understanding "am I selecting a mission or selecting a squad right now?"

### Why tab bar state badges?

The Deep is an asynchronous system — events fire while players are away. The stats panel `[D]` indicator tells players something needs attention, but once inside the overlay the tab bar gives no guidance on *where* to go. A player with a pending event and a completed mission who opens The Deep should immediately see `[Hub⚡1✓1]` and navigate directly, not discover the state by cycling tabs.

### Why fix the "no missions available" empty state message?

"Complete active missions to refresh the pool" implies a causal relationship that doesn't exist. The mission pool refreshes on a timer, not on mission completion. Giving players false mental models leads to confusion when the pool stays empty after missions complete. The fix branches on actual state (no mercs, missions active, or just waiting) to give accurate, actionable guidance.

### Why show power as a percentage?

The success bands are defined by power ratios: `<75%` = fail, `75-99%` = risky (30%), `100-149%` = good (60-90%), `150%+` = overpowered (95%). Showing `Power: 32/25` requires players to compute `32/25 = 128%` mentally to know they are in the "good" band. Showing `(128%)` colored green eliminates the math and makes the band membership immediately scannable.

---

## Appendix: Color Palette Reference for The Deep UI

| Element | Color | Hex approx |
|---------|-------|-----------|
| Section labels | `Rgb(80, 160, 220)` | Deep blue |
| Marks/currency | `Yellow` | Bright yellow |
| Marks ◆ icon | `Yellow` | — |
| Good/affordable | `Rgb(60, 180, 80)` | Soft green |
| Warning/missing | `Yellow` | — |
| Error/insufficient | `LightRed` | — |
| Overpowered | `Rgb(80, 220, 120)` | Bright green |
| Risky | `Yellow` | — |
| Fail | `LightRed` | — |
| Background separators | `Rgb(40, 60, 80)` | Dark slate |
| Inactive/unavailable | `DarkGray` | — |
| Required archetype present | `Green` | — |
| Recommended archetype present | `Cyan` | — |
| Event pending badge | `Yellow` | — |
| Completed badge | `Green` | — |

## deep-ui-onboarding-design.md

# The Deep — Onboarding and Contextual Help System Design

**Updated:** 2026-02-24 — Incorporates findings from `deep-ui-audit.md`

## Overview

The Deep is the most complex endgame system in Quest3, requiring players to understand
wall-clock mission timing, squad composition, layer progression, infrastructure investment,
and the prestige reset cycle — all before seeing meaningful payoff. This document defines
a contextual help strategy that communicates these mechanics inline, non-intrusively, and
progressively as players engage with the system.

**Design principle**: experienced players should never be slowed down. New players get
exactly enough information at the moment of need — no more.

This document covers onboarding design only. Companion documents define the broader
UX improvements (view hierarchy, visual polish) to which this onboarding layer attaches.

---

## 1. Audit Findings — Summary

The sys-architect audit (`deep-ui-audit.md`) identified the following issues that directly
affect new-player comprehension. This document addresses the onboarding dimension of each.

### Critical (P0) — Blocking Onboarding

| Issue | Description | Onboarding Impact |
|-------|-------------|------------------|
| P0.1 | Recruit tab has no rendering — shows Roster instead | Players cannot discover mercenary hiring |
| P0.2 | Mission `description` field never shown | Missions feel identical; purpose is opaque |
| P0.3 | Construction missions show "Construction" not "Build Outpost" etc. | Players cannot tell what they are building |

### High Impact (P1) — Core Comprehension Gaps

| Issue | Description | Onboarding Impact |
|-------|-------------|------------------|
| P1.1 | Familiarity tier labels never shown (Unknown/Mapped/Familiar/Mastered) | Players don't understand the progression system |
| P1.2 | Guild rank upgrade path completely hidden | No visible path to account-level advancement |
| P1.3 | Tab bar carries no state indicators | Events and injuries require navigation to discover |
| P1.4 | Power ratio shown as N/N but not as percentage | Success bands are unclear |
| P1.5 | Infrastructure build costs not shown in Layers view | Cannot plan without costs |

### Other Gaps Not in Audit

- The prestige reset mechanic (mercs reset, infrastructure persists) is never communicated
- Stat meanings (Power/Resilience/Expertise) have no in-panel descriptions
- Risk tier labels have no consequence explanation
- The auto-resolve event mechanic is the one well-explained mechanism; serve as reference

---

## 2. Onboarding Strategy

### Approach: Three-Layer Progressive Disclosure

Rather than a separate tutorial screen or help modal, information is embedded at the
point of need across three layers:

**Layer 1 — Structural fixes (P0/P1)**
These are not "help text" — they are missing information the UI should always show.
Mission descriptions, construction labels, familiarity tiers, build costs, and power
percentages are data the player needs to make decisions. They belong in the primary UI.

**Layer 2 — First-visit contextual hints**
For mechanics that are harder to discover (prestige cycle, stat meanings, risk consequences),
show one-line hints for the first N visits to a view. These fade out automatically — never
returning once the player has seen them. No toggle needed; no memory needed.

**Layer 3 — On-demand reference ([?] key)**
A per-view reference panel toggled with [?]. Experienced players ignore it. New players
who want depth can access a condensed reference at any time without leaving the overlay.

This approach is consistent with how Haven uses room descriptions and tier cost displays
to teach the system without a separate tutorial.

---

## 3. Discovery Modal — The First Impression

The discovery modal is the player's first contact with The Deep. Currently:

```
  The Deep Discovered!

  A scarred mercenary captain approaches,
  maps of underground passages in hand.
  "The Deep goes further than you know."

  Press [D] to visit.  [Enter] to dismiss.
```

**Problem**: No explanation of what The Deep is, what makes it different from other
systems, or that its infrastructure survives prestige (the core hook).

**Revised modal content**:

```
  The Deep Discovered!

  A scarred mercenary captain approaches,
  maps of underground passages in hand.
  "The Deep goes further than you know."

  Send mercenaries on hour-long expeditions.
  Earn Marks, items, and Prestige Rank fragments.
  Infrastructure you build here survives prestige.

  Press [D] to visit.  [Enter] to dismiss.
```

**Rationale for each added line**:
- "Send mercenaries on hour-long expeditions" — establishes the real-time mechanic
- "Earn Marks, items, and Prestige Rank fragments" — establishes the reward structure
- "Infrastructure you build here survives prestige" — the generational hook; the key
  reason to invest in The Deep even near prestige time

**Implementation**: `src/ui/deep_scene.rs: render_deep_discovery_modal()`.
Add three `Line::from(Span::styled(..., Color::Cyan))` entries before the footer hint.
Modal height increases from 11 to 14 to accommodate; cap at `area.height.saturating_sub(4)`.

---

## 4. Hub View — First-Visit Onboarding

### 4.1 Guild Rank Upgrade Path (P1.2 — structural fix)

The audit confirmed guild rank upgrade requirements are completely hidden. This is a
structural data gap, not a tooltip problem. The hub header should always show:

**Current header (two lines)**:
```
  Guild: Freelancers (Rank 1)    Marks: 1,240
  Frontier: Layer 3 (The Shallows)    Mercs: 3/5
```

**Revised header (three lines)**:
```
  Guild: Freelancers (Rank 1)    Marks: 1,240    Concurrent: 1/1
  Frontier: Layer 3 (The Shallows)    Mercs: 3/5    Available: 2
  Next rank: Layer 3 Breakthrough  →  Rank 2 (Sellswords, 7 mercs, 1 concurrent)
```

Third line uses `Color::Rgb(60, 100, 140)` — distinct from the header rows but not
alarming. When already at max rank (5/Legion), replace with "Max guild rank reached."

**Data sources**:
- `guild_rank.concurrent_missions()` — concurrent cap
- `prestige.available_merc_count()` — ready mercs
- `guild_rank.next()` — next rank name
- `guild_rank.next()?.required_breakthrough_layer()` — layer requirement
- Next rank max roster size from `GUILD_RANK_STATS`

### 4.2 Empty State Messaging

The empty state message when no missions are active is the primary teaching moment
for new players. Current text is too terse.

**Current**:
```
  No active missions.
  [Tab] to Missions view to deploy a squad.
```

**Revised** (first visit only, `hub_visit_count == 0`):
```
  No active missions running.

  Start with a Supply Run — safe income, no risk, 2-4 hours.
  Missions continue while the game is closed.

  [Tab] Switch View  to deploy your first squad.
```

After `hub_visit_count >= 1`, the hint collapses back to:
```
  No active missions.
  [Tab] Switch View  to deploy a squad.
```

Color: hint lines in `Color::Rgb(50, 80, 110)`.

### 4.3 Prestige Cycle Hint (First Visit Only)

When `hub_visit_count == 0`, show a one-line persistent reminder between the header
and the mission list area:

```
  ──────────────────────────────────────────────────────────────
  Tip: Mercs and Marks reset on prestige. Infrastructure persists.
  ──────────────────────────────────────────────────────────────
```

Color: `Color::Rgb(50, 80, 110)`. Shown for exactly one session (first visit to Hub).
Clears permanently when `hub_visit_count` increments past 1.

---

## 5. New Mission View — Onboarding

### 5.1 Mission Description (P0.2 — structural fix, always shown)

The `AvailableMission.description` field contains thematic context for every mission.
It is never rendered. This should always be visible in the detail panel — not a hint,
not toggled, always present.

**Revised detail panel layout (split view)**:
```
  Layer 3 — The Shallows
  Recon  ·  Low risk

  Survey the Shallows for entry points    ← description (word-wrapped, 2 lines max)
  and note hazards for future squads.

  Duration: 4h 0m
  Reward:   Marks + Familiarity (intel)

  Requires:
    Min Power 15
    Scout recommended
```

**Implementation**: In `render_new_mission_split()` in `deep_missions.rs`, after the
layer/tier header row, word-wrap `m.description` to `detail_inner_w` characters and
render in `Color::DarkGray`. Use the existing word-wrap pattern from the event view
(split on whitespace, accumulate to line width limit). Maximum 2 lines to preserve space.

### 5.2 Construction Mission Label (P0.3 — structural fix, always shown)

Replace `m.mission_type.display_name()` in mission list rows with:

```rust
let type_label = match &m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
```

Apply to both compact and split list panels. This is a one-line fix with zero complexity.

### 5.3 Mission Type Descriptions (First-Visit Contextual Hints)

When `mission_visit_count < 5`, show a one-line description after the risk line in
the detail panel. These describe what the mission type accomplishes, not just its label:

| Mission Type | One-Line Description |
|-------------|---------------------|
| Supply Run | Safe income — always returns, earns Marks reliably |
| Recon | Raises layer familiarity — cuts future mission times |
| Expedition | Core rewards — items, Marks, and merc XP |
| Breakthrough | Clears the frontier — unlocks the next layer |
| Construction | Builds permanent infrastructure — survives prestige |

Color: `Color::Rgb(50, 80, 110)` on first 5 visits, then hidden.

### 5.4 Power Ratio Percentage (P1.4 — structural fix, always shown)

Replace `Power: 32/25` with `Power: 32/25  (128%)` in both compact and split views.
Color the percentage based on success band:

| Ratio | Color |
|-------|-------|
| >= 150% | `Color::Rgb(80, 220, 120)` (bright green — overpowered) |
| >= 100% | `Color::Green` |
| >= 75% | `Color::Yellow` |
| < 75% | `Color::LightRed` |

Safe missions (Supply Run, Construction) show `Always succeeds` instead of a percentage.

### 5.5 Risk Consequence Descriptions (First-Visit Hints)

When `mission_visit_count < 5`, enhance the risk display with a consequence note:

| Risk | Current | Enhanced |
|------|---------|---------|
| Safe | `Safe` | `Safe  — no injuries, guaranteed return` |
| Low | `Low` | `Low   — rare injuries, Marks lost on failure` |
| Medium | `Medium` | `Medium — injuries likely on failure` |
| High | `High` | `High  — injuries or death possible on failure` |

Consequence text in `Color::DarkGray`. Collapse to just the label after 5 visits.

### 5.6 Panel Focus Indicator (UX Fix — always shown)

The audit noted (3.2) that the squad assignment panel has no visual "active" indicator
when focus moves to it. Add a header row color change: when `staging_mission_index.is_some()`,
render `"Assign Squad:"` in `Color::Rgb(80, 160, 220)` (DEEP_BORDER_COLOR — active).
When unfocused, render in `Color::DarkGray`. This is a low-effort signal with immediate clarity.

---

## 6. Roster View — Onboarding

### 6.1 Stat Descriptions (First-Visit Hints)

In the split-panel detail view, add inline descriptions after each stat value.
Show when `roster_visit_count < 3`:

```
  Stats:
    Power:      52      — drives mission success
    Resilience: 48      — reduces injury risk
    Expertise:  24      — unlocks archetype event choices
```

The `— description` text in `Color::DarkGray`. After 3 visits, collapse to stat values only.

**Implementation**: In `render_roster_split()` in `deep_roster.rs`, conditionally append
the description string to each `put_text` stat line based on visit count.

### 6.2 Injury Recovery Explanation

The audit noted (2.3) that injury shows "Injured (2 missions)" with no explanation of what
"missions" means in the context of wall-clock recovery.

**Current**: `"Injured (3 missions)"`
**Proposed**: `"Injured — recovers after 3 missions complete (approx. 18h)"`

The approximate hour estimate is `missions_remaining * 6` (average mission duration).
Show in compact form on the list row, full form in the detail panel.

### 6.3 Merc Leveling Progress (P2.1 from audit)

In the detail panel, add a leveling progress indicator after the level/missions line:

```
  Level: 3    Missions: 6

  Progress to Lv4:  [███████░░] 7/9 missions
```

Where `missions_to_next_level(3) = 3 + 3*2 = 9` and `missions_completed % 9 = 6`.
Rendered as a compact block bar using the existing `render_progress_bar` pattern.

Bar color: `Color::Cyan`. Empty cells: `Color::Rgb(30, 40, 60)`.

---

## 7. Recruit View — Must Be Implemented (P0.1)

The Recruit tab currently renders the Roster view silently. The audit identified this as
a critical bug. From an onboarding perspective, the Recruit view is also the primary
teaching moment for the guild economics (Marks → mercs → stronger squads).

### 7.1 Recruit View Layout (L/XL)

```
  Recruits: 4 candidates    Pool refreshes in: 16h 20m
  Roster: 3/5 open slots    Marks: 1,240
  ─────────────────────────────────────────────────────────
  ┌─ CANDIDATES ──────────────────┐  ┌─ CANDIDATE DETAIL ──────────────┐
  │ ► Gareth   Vanguard  Common  │  │ Gareth the Ironclad             │
  │   Mira     Medic     Common  │  │ Archetype: Vanguard             │
  │   Thorne   Arcanist  Uncommon│  │ Quality:   Common               │
  │   Kira     Scout     Common  │  │                                 │
  │                              │  │ Stats:                          │
  │   [Recruit slot: 2 open]     │  │   Power:      14  — high        │
  │                              │  │   Resilience: 12  — medium      │
  │                              │  │   Expertise:   4  — low         │
  │                              │  │                                 │
  │                              │  │ Cost: 60 Marks  [affordable]    │
  │                              │  │                                 │
  │                              │  │ [Enter] Recruit  [Esc] Back     │
  └──────────────────────────────┘  └─────────────────────────────────┘
  [↑/↓] Navigate  [Enter] Recruit  [Esc] Back
```

### 7.2 Recruit View Layout (S — compact)

```
  Recruits: 4    Marks: 1,240    Roster: 3/5
  ► Gareth   Vanguard  Pwr:14  60M  [Recruit]
    Mira     Medic     Pwr:6   45M
    Thorne   Arcanist  Pwr:10  80M  [Uncommon]
    Kira     Scout     Pwr:8   55M
  Pool refreshes in: 16h 20m
  [↑/↓] Navigate  [Enter] Recruit  [Esc] Back
```

### 7.3 First-Visit Onboarding Hint for Recruit View

When `recruit_visit_count == 0`, show above the candidate list:

```
  Tip: Stronger archetypes unlock at higher Guild Ranks.
       Pool refreshes every 24 hours.
```

Color: `Color::Rgb(50, 80, 110)`. Shown once.

### 7.4 Affordability Feedback

Cost display uses color-coded affordability (matching Haven's pattern):
- `Color::Green` when marks >= cost
- `Color::Red` when marks < cost
- Error flash message "Insufficient Warband Marks" when [Enter] pressed on unaffordable recruit

### 7.5 Quality Color

Candidate quality is shown with rarity-adjacent colors:

| Quality | Color |
|---------|-------|
| Common | `Color::White` |
| Uncommon | `Color::Green` |
| Rare | `Color::Yellow` |
| Elite | `Color::Magenta` |

---

## 8. Layers View — Onboarding

### 8.1 Familiarity Tier Labels (P1.1 — structural fix, always shown)

Replace `"Intel:  42%  [bar]"` with `"Familiarity: 42%  [Mapped -10%]  [bar]"`.

Tier label in brackets, colored by tier:

| Level | % Range | Label | Duration Reduction | Color |
|-------|---------|-------|-------------------|-------|
| Unknown | 0–24% | `[Unknown]` | none | `Color::DarkGray` |
| Mapped | 25–49% | `[Mapped]` | -10% | `Color::Cyan` |
| Familiar | 50–74% | `[Familiar]` | -20% | `Color::Green` |
| Mastered | 75–100% | `[Mastered]` | -30% | `Color::Rgb(255, 215, 0)` |

Include the reduction percentage in the label. This communicates both the tier name
and its mechanical effect without requiring a tooltip.

### 8.2 Total Duration Reduction Display

After the familiarity bar, add a "Combined reduction" line that shows the total modifier
from all sources (Outpost + familiarity + Saboteur):

```
  Familiarity: 75%  [Mastered -30%]  ████████████████████████░░░░░
  Duration reduction: -55%  (Outpost -25%  Mastered -30%)
```

The summary line uses `Color::Cyan` for the total and `Color::DarkGray` for the breakdown.
This is particularly valuable for players deciding whether to build an Outpost on a
layer that already has high familiarity.

**Data source**: `layer.total_duration_reduction()` method exists; breakdown comes from
checking `layer.has_infrastructure(Infrastructure::Outpost)` and `familiarity_level`.

### 8.3 Infrastructure Build Costs (P1.5 — structural fix, always shown)

For unbuilt infrastructure in the detail panel, show the Warband Marks cost:

**Current**:
```
  [ ] Watchtower   +intel
```

**Revised**:
```
  [ ] Watchtower   +intel, +25 familiarity on build    280M
```

Cost in `Color::Yellow` when affordable, `Color::Red` when not.
Call `infrastructure_build_cost(infra, layer.index)` for each unbuilt slot.

### 8.4 First-Visit Infrastructure Context Hint

When `layer_visit_count < 3`, show a one-line hint at the bottom of the infrastructure
list in the detail panel:

```
  Build via Construction missions (safe, 4-8h). Permanent.
```

Color: `Color::Rgb(50, 80, 110)`. Disappears after 3 visits.

### 8.5 Next Guild Rank Breakthrough Target

In the layer list, add a visual marker for the layer required for the next guild rank
upgrade. For example, if Layer 7 Breakthrough unlocks Rank 3, annotate Layer 7 in the
list:

```
  L7  The Warrens  [CLEAR]  ★ Rank 3 unlock
```

The `★` marker in `Color::Rgb(255, 215, 0)` (gold) draws attention to the strategic
layer milestone. Only show when the player hasn't yet reached that rank.

---

## 9. Event Response View — Onboarding

### 9.1 First-Event Hint

When `event_visit_count == 0`, show a two-line explanation above the choices:

```
  Your choice affects outcome and timing. Events auto-resolve safely if ignored.
```

Color: `Color::Rgb(50, 80, 110)`. Collapsed after first visit — the existing auto-resolve
countdown already serves as the ongoing reminder.

### 9.2 Unavailable Choice Labels (Audit Finding 3.x)

When an archetype-gated choice cannot be selected because the required archetype is not
in the squad, add a parenthetical explanation:

**Current**: `[VANGUARD]  Break through the rubble` (in DarkGray, no explanation)
**Revised**: `[VANGUARD]  Break through the rubble  (Vanguard not in squad)`

The parenthetical in `Color::Rgb(80, 80, 80)` — visibly different from DarkGray content.

### 9.3 Consequence Time Delta (P3.2 from audit — always shown)

Replace `"— delay"` / `"— faster"` with explicit durations:

```rust
let consequence = match (choice.is_risky, choice.time_delta_secs) {
    (true, _) => "— risky".to_string(),
    (false, d) if d > 0 => format!("— +{}", format_hours(d as u64)),
    (false, d) if d < 0 => format!("— -{}", format_hours(d.unsigned_abs())),
    _ => "— safe".to_string(),
};
```

This is a one-line change to `deep_events.rs` with high player value.

---

## 10. Mission Results Modal — Onboarding

### 10.1 Familiarity Gained (P2.3 — always shown)

Add a familiarity line to the rewards section:

```
  Rewards:
    + 380 Warband Marks
    + Familiarity on Layer 3: +5%  (now 47%, Mapped)
```

`familiarity_gain(mission_type)` is deterministic. Show the new level label if it
crossed a tier boundary: `+5% → Mapped!` in `Color::Cyan`.

### 10.2 Breakthrough Layer Cleared Celebration (P2.6 — always shown)

When a Breakthrough mission succeeds, add a prominent centered line:

```
  ★  LAYER 3 CLEARED — Layer 4 Unlocked!  ★
```

Color: `Color::Rgb(255, 215, 0)` (Gold). Centered. This is the major milestone
players push toward; it deserves a moment of recognition.

### 10.3 Post-Collection Balance Preview

After collecting rewards, flash the updated mark balance in the modal before dismissal:

```
  [Enter] Collect and Close
  Marks after: 1,620
```

The "Marks after" line appears immediately when [Enter] is pressed (before the modal
closes), giving players confirmation of the economic transaction before the view clears.

---

## 11. Tab Bar State Indicators (P1.3)

The tab bar is the navigation hub for the entire overlay. Adding state indicators
eliminates the need to navigate to each view to discover pending events or fresh recruits.

### 11.1 Indicator Design

Append compact badges after each tab label:

| Tab | Condition | Badge | Color |
|-----|-----------|-------|-------|
| `[Hub]` | Mission complete, awaiting collect | `✓` | `Color::Green` |
| `[Hub]` | Mission event pending | `⚡` | `Color::Yellow` |
| `[Missions]` | Missions available in pool | count `·N` | `Color::Cyan` |
| `[Roster]` | Mercs injured or lost | `!N` | `Color::Yellow` |
| `[Recruit]` | Pool has candidates | `●` | `Color::Cyan` |

**Priority**: When multiple conditions apply, show the most urgent (event pending >
mission complete > injured mercs > mission count > recruit available).

**Examples**:
```
  [Hub ⚡] [Missions] [Roster !2] [Layers] [Recruit ●]
```

### 11.2 Implementation

In `render_tab_bar()` in `deep_scene.rs`, pass `&DeepState` and append indicators
after the tab label before closing `]`. The function currently takes only `active: DeepView`.

```rust
fn render_tab_bar(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    active: DeepView,
    deep: &DeepState,  // add this
) {
    // ... for each tab:
    let badge = match tab {
        DeepView::Hub => {
            if deep.prestige.has_any_pending_event() { " ⚡" }
            else if !deep.prestige.pending_results.is_empty() { " ✓" }
            else { "" }
        }
        DeepView::Roster => {
            let injured = deep.prestige.roster.iter()
                .filter(|m| matches!(m.status, MercStatus::Injured { .. } | MercStatus::Lost))
                .count();
            if injured > 0 { &format!(" !{}", injured) } else { "" }
        }
        DeepView::Recruit => {
            if !deep.prestige.recruit_pool.candidates.is_empty() { " ●" } else { "" }
        }
        _ => "",
    };
    let label = format!("[{}{}]", tab.tab_label(), badge);
```

---

## 12. [?] Help Key — On-Demand Reference

Pressing [?] at any point in the Deep overlay toggles a compact reference panel.
This is the explicit escape hatch — for players who want depth, not the default experience.

### 12.1 UI Behavior

- [?] toggles `ui.show_help: bool`
- When `show_help` is true, render a 36-column reference panel on the right side (L/XL)
  or a full-width overlay (S/M), over the current view content
- Content is specific to `ui.view` (different text per view)
- No border on the reference panel — it floats visually over the scene buffer
- Background cells: `Color::Rgb(5, 8, 15)` (darker than backdrop, creates depth)
- A single `[?] Help` label in the footer row indicates the toggle is available

### 12.2 Hub Reference Content

```
  THE DEEP — Quick Reference

  Warband Marks
  Earned from missions. Resets on prestige.
  Spend on recruits and infrastructure.

  Guild Rank
  Rank 1 Freelancers  5 mercs  1 mission
  Rank 2 Sellswords   7 mercs  1 mission
  Rank 3 Company      9 mercs  2 missions
  Rank 4 Battalion   12 mercs  3 missions
  Rank 5 Legion      15 mercs  4 missions
  Advance by clearing breakthrough layers.

  Prestige Cycle
  Resets: mercs, Marks, active missions
  Survives: guild rank, cleared layers,
            infrastructure, familiarity
```

### 12.3 Mission Reference Content

```
  MISSION TYPES

  Supply Run   2-4h  Safe
  Safe income. Always returns.
  Use on cleared layers.

  Recon        4-8h  Low
  Raises layer familiarity.
  Cuts future mission times.

  Expedition   8-16h  Medium
  Main rewards: items + Marks.
  Use on frontier layers.

  Breakthrough 18-24h  High
  Clears frontier. Unlocks next.
  Earns 0.5 Prestige Rank.

  Construction 4-8h  Safe
  Builds infrastructure.
  Permanent — survives prestige.

  POWER GUIDE
  >= 150% threshold: 95% success
  >= 100% threshold: 60-90% success
  >= 75% threshold:  ~30% success
  < 75% threshold:   likely fail
```

### 12.4 Roster Reference Content

```
  MERC STATS

  Power      — mission success driver
  Resilience — injury resistance
  Expertise  — enables special choices

  ARCHETYPES
  Vanguard   high power, durable
  Scout      recon duration bonus
  Arcanist   expedition bonuses
  Medic      reduces squad injuries
  Saboteur   cuts mission time -10-15%

  LEVELING
  Complete missions to gain XP.
  Missions to level = 3 + level * 2
  Max level: 20

  INJURIES
  Light:     4-8h  (≈1 mission)
  Moderate:  8-12h (≈2 missions)
  Severe:   12-16h (≈3 missions)
  Injured mercs cannot be assigned.
```

### 12.5 Layer Reference Content

```
  FAMILIARITY (Intel %)
  Unknown  0-24%   no reduction
  Mapped   25-49%  -10% durations
  Familiar 50-74%  -20% durations
  Mastered 75-100% -30% durations

  Gain familiarity by running missions.
  Recon gives +15%, most others +5-10%.

  INFRASTRUCTURE (permanent)
  Outpost      -25% all mission times
  Supply Cache +50% Marks on Supply Runs
  Watchtower   +25 familiarity on build
  Bridge       -2h on deeper missions

  BUILD VIA Construction missions (safe).
  Costs scale with layer depth.
  Max 4 infrastructure per layer.
```

---

## 13. Visit Count Implementation

### 13.1 Data Structure Changes

Add to `DeepUiState` in `src/deep/types.rs`:

```rust
pub struct DeepUiState {
    // existing fields...
    pub hub_visit_count: u8,
    pub mission_visit_count: u8,
    pub roster_visit_count: u8,
    pub layer_visit_count: u8,
    pub event_visit_count: u8,
    pub recruit_visit_count: u8,
    pub show_help: bool,
}
```

No persistence needed — visit counts are per-session. Players who restart the game
after a long absence benefit from seeing contextual hints again.

### 13.2 Counter Increment Logic

In `src/input/deep_input.rs`, when `ui.view` changes to a new sub-view:

```rust
fn switch_view(ui: &mut DeepUiState, target: DeepView) {
    ui.view = target;
    ui.selected_index = 0;
    // Increment visit counter for the new view (saturating at 255)
    match target {
        DeepView::Hub => ui.hub_visit_count = ui.hub_visit_count.saturating_add(1),
        DeepView::NewMission => ui.mission_visit_count = ui.mission_visit_count.saturating_add(1),
        DeepView::Roster => ui.roster_visit_count = ui.roster_visit_count.saturating_add(1),
        DeepView::Infrastructure => ui.layer_visit_count = ui.layer_visit_count.saturating_add(1),
        DeepView::EventResponse => ui.event_visit_count = ui.event_visit_count.saturating_add(1),
        DeepView::Recruit => ui.recruit_visit_count = ui.recruit_visit_count.saturating_add(1),
    }
}
```

### 13.3 Rendering Pattern

```rust
// Standard conditional hint pattern
if ui.mission_visit_count < 5 {
    put_text(buffer, row, detail_col, hint_text, Color::Rgb(50, 80, 110));
    row += 1;
}
```

The threshold `< 5` for mission hints and `< 3` for stat descriptions is a design
judgment — mission type hints need more repetitions to be internalized since players
return to New Mission view frequently; stat descriptions are needed fewer times.

---

## 14. Size Tier Handling

All onboarding elements respect the existing responsive size tiers:

| Tier | Hub hints | Detail descriptions | [?] help | Tab badges |
|------|-----------|--------------------|-----------|----|
| TooSmall | No rendering | No rendering | No | No |
| S | 1-line hints only | None (no detail panel) | Full-width modal | Abbreviated |
| M | 1-line hints | Short descriptions | Full-width modal | Yes |
| L/XL | Full hints | Full descriptions with legend | Right-panel | Yes |

For S tier, the tab badges use single characters only: `⚡` `✓` `!` `●`.

---

## 15. Files Requiring Changes

| File | Change | Priority |
|------|--------|----------|
| `src/deep/types.rs` | Add visit counters + `show_help` to `DeepUiState` | P0 |
| `src/input/deep_input.rs` | Increment visit counters on view switch; [?] toggle | P0 |
| `src/ui/deep_scene.rs` | Update discovery modal; add `DeepState` param to `render_tab_bar`; tab badges | P0 |
| `src/ui/deep_missions.rs` | Mission description; construction label; power %; risk hints; mission type hints | P0 |
| `src/ui/deep_roster.rs` | Stat descriptions; injury detail; leveling progress; add `render_recruit()` | P0 |
| `src/ui/deep_layers.rs` | Familiarity tier labels; total reduction; infra costs; infra hint; guild rank milestone | P1 |
| `src/ui/deep_events.rs` | First-event hint; unavailable choice labels; explicit time deltas | P1 |
| `src/ui/deep_results.rs` | Familiarity gained; breakthrough celebration; post-collect balance | P2 |

---

## 16. Acceptance Criteria

A new player encountering The Deep for the first time should, after their first session:

1. **Understand the time scale** — know missions take hours, the game can be closed
2. **Know what Supply Run vs Breakthrough are for** — mission descriptions make this clear
3. **Know what failure means** — risk consequence descriptions explain injury probability
4. **Know how power affects success** — power percentage communicates success bands
5. **Know familiarity builds over time** — tier labels show the progression path
6. **Know infrastructure is permanent** — discovery modal and infra hint surface this
7. **Know the prestige cycle** — hub tip and discovery modal both mention it
8. **Be able to hire mercs** — Recruit view is now functional with full candidate details

An experienced player should:

1. **Never be slowed by hints** — hints disappear after 3-5 visits to each view
2. **Access help on demand** — [?] key provides per-view reference at any time
3. **See actionable tab state at a glance** — tab badges show events/injuries/recruits
4. **Not lose workflow** — all new content fits in existing layout without new modals

---

## 17. Design Decisions and Rationale

**Why not a separate tutorial modal?**
The Deep has six sub-views, each with distinct mechanics. A single tutorial modal would
either be impossibly long or incomplete. Inline progressive disclosure ensures players
receive information when they can immediately act on it.

**Why visit counts instead of persistent flags?**
Resetting on restart is a feature, not a bug. Players who haven't played in weeks
benefit from seeing hints again. The cost of showing a hint to an experienced player
for one session is negligible.

**Why [?] instead of inline expandable sections?**
The scene buffer rendering model (`put_text` into `Vec<Vec<SceneCell>>`) doesn't
support interactive expandable sections without significant complexity. A toggle-key
approach fits the existing interaction model (similar to how Stormglass uses keybinds
for phase transitions) and gives experienced players a clear escape hatch.

**Why show power percentage alongside raw numbers?**
The success forecast strings ("60-90% success", "Overpowered — 95%") are useful but
require players to mentally map the ratio to the forecast. Showing `(128%)` alongside
the threshold makes this mapping explicit and teaches the mechanic rather than hiding it.

## deep-ui-roster-layers-design.md

# The Deep UI: Roster, Layers, and Recruit View — Design Specification

**Date:** 2026-02-23 (updated post-audit)
**Scope:** Tasks #4 and #5 — Roster stat clarity, Layer map visual improvements
**Also covers:** Recruit view (P0.1 critical bug), tab bar state indicators (P1.3)
**Author:** UX Designer (agent)
**Based on audit:** `docs/plans/deep-ui-audit.md`

---

## Executive Summary

The audit confirmed and extended the problems identified in the initial draft. Critical additions:

1. **The Recruit tab renders nothing** (dispatches to Roster instead) — P0.1 critical
2. **Construction missions drop their payload** — shows "Construction" not "Build Outpost" — P0.3
3. **Familiarity tier labels never shown** — raw % only — P1.1
4. **Tab bar carries no state indicators** — injuries, events, and completions require manual tab-switching to discover — P1.3
5. **Infrastructure costs absent** — players cannot plan builds — P1.5
6. **Build action missing from Layers view** — read-only when it should be actionable — P2.4
7. **Tier color suppressed in compact mode** — `let _ = tc` bug — P2.5
8. **Status offset fragile in compact Roster** — `rfind` pattern breaks on name matches — P3.4

This document covers all of these issues in implementation-ready detail.

---

## Part 1: Roster View — Stat Clarity and Merc Progression

### 1.1 Current Problems (Confirmed by Audit)

**Stats have no semantic meaning:**
- `Power: 14`, `Resilience: 12`, `Expertise: 8` — numbers with no context
- No indication that Power = combat effectiveness, Resilience = injury resistance, Expertise = event bonuses

**Status display is opaque:**
- `"Injured (2 missions)"` — measured in missions; players think in hours
- `"On mission #7"` — mission ID is meaningless
- No urgency gradient between Light/Moderate/Severe injuries

**Archetype identity is invisible:**
- Color only; role description never shown
- No indication which mission types the archetype benefits

**Progression is hidden:**
- Level shown, missions-to-next-level never shown
- No progress bar toward next level
- Audit correction: `missions_to_next_level(level) = 3 + level * 2`, cumulative from level 1

**Quality tier is not stored on Mercenary:**
- Audit confirmed: `quality` is not on the `Mercenary` struct; infer from stat delta vs archetype baseline, or add as a field (recommended)

**Status column offset is fragile (P3.4):**
- `line.rfind(status_label)` will misidentify position if merc name contains "Ready", "Injured", etc.
- Must use fixed column offsets derived from format string field widths

### 1.2 Redesigned Roster List — Compact Mode

**Header:**
```
  Name              Role  Lv  Pwr  Res  Status
```

**Row format** — fixed column offsets, no rfind:
```
▶ Gareth Ironwall  [VAN]   8   47   38   Ready
  Lyra Shadowfoot  [SCT]   5   28   22   ● Mission
  Aldric Mender    [MED]   3   18   20   ✖ 1 miss
  Finn the Cunning [SAB]   2   15   12   ⚕ Injured
```

Fixed column positions (to avoid fragile rfind):
- Col 1-2: cursor `▶ ` or `  `
- Col 3-16: name, max 14 chars
- Col 18-22: archetype abbreviation `[VAN]` in archetype color
- Col 24-25: level, right-aligned 2 chars
- Col 27-29: effective_power, right-aligned 3 chars
- Col 31-33: effective_resilience, right-aligned 3 chars
- Col 36+: status with glyph prefix

Status display format (use fixed status column, not rfind):
- `Ready` → `Color::Green`
- `● Mission` → `Color::Cyan` (U+25CF filled circle)
- `⚕ N miss` → `Color::Yellow` (N = missions_remaining)
- `✖ Lost` → `Color::Red` (U+2716 heavy X)

Archetype abbreviations and colors:
- `[VAN]` Vanguard → `Color::Red`
- `[SCT]` Scout → `Color::Green`
- `[ARC]` Arcanist → `Color::Magenta`
- `[MED]` Medic → `Color::Cyan`
- `[SAB]` Saboteur → `Color::Yellow`

Remove the blank-row gap between mercs — the status glyph (`●`/`⚕`/`✖`) provides visual rhythm. This allows more mercs on screen, which matters when guild rank cap reaches 15.

### 1.3 Redesigned Roster List — Split Mode (Left Panel)

**Header:**
```
  Name              Archetype   Lv  Pwr  Res   Status
```

**Row format** (single row per merc):
```
▶ Gareth Ironwall  Vanguard      8   47   38   Ready
  Lyra Shadowfoot  Scout         5   28   22   ● Active
  Aldric Mender    Medic         3   18   20   ⚕ 2 miss
  Finn the Cunning Saboteur      2   15   12   ✖ Lost
  [ Recruit slot — 60 Marks ]
```

Recruit slot hint shown when roster is below guild rank cap.

### 1.4 Redesigned Roster Detail Panel (Right Panel)

**Available merc example:**
```
Gareth Ironwall
Vanguard  ·  Level 8  ·  Missions: 14

Role: Frontline tank
  High Power + Resilience. Reduces squad
  casualties. Best on high-risk missions.

Stats
  Power:       47  (combat effectiveness)
  Resilience:  38  (reduces injury chance)
  Expertise:   10  (event bonuses and unlocks)

Progression
  Missions completed: 14
  To Level 9:  ██████░░░  6 / 9 missions
  At Lv9:  Pwr 50  Res 40  Exp 11

Status
  Ready for assignment
```

**Injured merc example:**
```
Aldric the Mender
Medic  ·  Level 3  ·  Missions: 4

Role: Squad healer
  Highest Resilience. Prevents permanent
  loss and reduces injury severity.

Stats
  Power:       18  (combat effectiveness)
  Resilience:  20  (reduces injury chance)
  Expertise:   16  (event bonuses and unlocks)

Progression
  Missions completed: 4
  To Level 4:  ████░░░░░  4 / 9 missions
  At Lv4:  Pwr 20  Res 22  Exp 18

Status
  Injured — Moderate
  Recovery: ~2 missions remaining (~12h)
  ████████░░░░  Returns after 2 missions
```

**On-mission merc example:**
```
Lyra Shadowfoot
Scout  ·  Level 5  ·  Missions: 9

Role: Recon specialist
  High Expertise. Better auto-resolve and
  early event reveals. Faster missions.

Stats
  Power:       28  (combat effectiveness)
  Resilience:  22  (reduces injury chance)
  Expertise:   22  (event bonuses and unlocks)

Progression
  Missions completed: 9
  To Level 6:  █████░░░░  5 / 11 missions
  At Lv6:  Pwr 31  Res 25  Exp 26

Status
  On Mission — Layer 2 Recon
  ██████████░░░░  68% — returns in ~3h 20m
  ETA: 14:32
```

### 1.5 Progress Bar Calculation

The XP formula from `mercenaries.rs`:
- `missions_to_next_level(level) = 3 + level * 2`
- Cumulative missions to reach level N = `sum(3 + i*2 for i in 1..N)`

Since `Mercenary` only stores `missions_completed` (total all-time), we compute:

```rust
fn missions_to_reach_level(target_level: u32) -> u32 {
    (1..target_level).map(|l| Mercenary::missions_to_next_level(l)).sum()
}

fn level_progress(merc: &Mercenary) -> (u32, u32) {
    let missions_at_current_level = missions_to_reach_level(merc.level);
    let missions_for_this_level = Mercenary::missions_to_next_level(merc.level);
    let progress = merc.missions_completed.saturating_sub(missions_at_current_level);
    (progress.min(missions_for_this_level), missions_for_this_level)
}
```

Bar format: `██████░░░  6 / 9 missions`
- Filled: `Color::Cyan` `█`
- Empty: `Color::Rgb(30, 40, 60)` `░`
- Bar width: `detail_inner_w.saturating_sub(16).min(20)`

### 1.6 Injury Display

Severity display based on `missions_remaining`:
- 1 mission: `Color::Yellow` — "Light injury"
- 2 missions: `Color::LightRed` — "Moderate injury"
- 3+ missions: `Color::Red` — "Severe injury"

Hour estimate: `missions_remaining * 6` (from `HOURS_PER_MISSION_EQUIVALENT = 6` in `mercenaries.rs`)

### 1.7 Level-Up Stat Preview

At Lv N+1, stats calculated with `stats_at_level()` from `mercenaries.rs`. Show the delta from current effective stats:

```
At Lv9:  Pwr +3→50  Res +2→40  Exp +2→12
```

Or simplified:
```
At Lv9:  Pwr 50  Res 40  Exp 12
```

Use `Color::DarkGray` for preview values (not yet earned).

### 1.8 Archetype Role Descriptions

```rust
fn archetype_role(archetype: MercArchetype) -> (&'static str, &'static str) {
    match archetype {
        MercArchetype::Vanguard  => (
            "Frontline tank",
            "High Power + Resilience. Reduces squad\ncasualties. Best on high-risk missions."
        ),
        MercArchetype::Scout     => (
            "Recon specialist",
            "High Expertise. Better auto-resolve and\nearly event reveals. Faster missions."
        ),
        MercArchetype::Arcanist  => (
            "Elemental expert",
            "Highest Expertise. Counters hazards and\nenvironmental dangers. Fragile."
        ),
        MercArchetype::Medic     => (
            "Squad healer",
            "Highest Resilience. Prevents permanent\nloss and reduces injury severity."
        ),
        MercArchetype::Saboteur  => (
            "Trap specialist",
            "High Expertise. Speeds missions and\nunlocks alternate routes."
        ),
    }
}
```

### 1.9 Fixed Status Column Implementation

Replace fragile `rfind` with a calculated fixed column:

```rust
// In render_roster_compact():
// Format string: "{cursor}{name:14} {abbrev:5} {lv:2} {pwr:3} {res:3}   {status}"
// cursor=2, name=14, space=1, abbrev=5, space=1, lv=2, space=2, pwr=3, space=2, res=3, gap=3
const STATUS_COL: i32 = 2 + 14 + 1 + 5 + 1 + 2 + 2 + 3 + 2 + 3 + 3; // = 40
put_text(buffer, row, STATUS_COL, status_glyph_and_label, status_color);
```

---

## Part 2: Recruit View — New Implementation (P0.1 Critical)

### 2.1 The Bug

`deep_scene.rs:196` dispatches `DeepView::Recruit` to `render_roster()`. The Recruit tab silently shows the Roster. This must be fixed.

### 2.2 New `render_recruit()` Function

Add to `src/ui/deep_roster.rs` (alongside existing Roster functions):

**ASCII Mockup — Compact mode:**
```
RECRUIT POOL        Roster: 3/5    Marks: 240

  Name                Role   Lv  Pwr  Res  Exp  Cost
  ─────────────────────────────────────────────────────
  Bram Ironwall       [VAN]   1   15   13    5    50M
► Kira Shadowfoot     [SCT]   1    9   11   13    35M
  Njord the Mender    [MED]   1    7   15   11    40M

  Pool refreshes in: 14h 32m
  [ Select with ↑/↓, recruit with Enter ]
```

**ASCII Mockup — Split mode (left panel — candidate list):**
```
RECRUIT POOL             Marks: 240 M

  Name              Archetype  Pwr  Res  Exp  Cost
  ─────────────────────────────────────────────────
  Bram Ironwall     Vanguard    15   13    5   50M
► Kira Shadowfoot   Scout        9   11   13   35M
  Njord the Mender  Medic        7   15   11   40M

  Pool refresh: 14h 32m
```

**ASCII Mockup — Split mode (right panel — candidate detail):**
```
Kira Shadowfoot
Scout  ·  Level 1  ·  Common

Role: Recon specialist
  High Expertise. Better auto-resolve and
  early event reveals. Faster missions.

Stats at Level 1
  Power:       9   (combat effectiveness)
  Resilience:  11  (reduces injury chance)
  Expertise:   13  (event bonuses and unlocks)

Cost: 35 Warband Marks
Balance: 240 Marks  →  205 after recruit

Roster: 3 / 5 slots used
  [ Enter ] Recruit    [ Esc ] Cancel
```

**Affordability feedback:**
- If `marks >= cost`: cost in `Color::Green`, action available
- If `marks < cost`: cost in `Color::Red`, "Insufficient Marks" flash message on Enter
- If `roster >= max_roster`: "Roster full" flash message, action blocked

**Pool refresh timer:**
- `refreshed_at + 24h - now` as `"Xh Ym"` countdown
- If `needs_refresh(now)` is true: `"Pool ready for refresh"` in `Color::Yellow`

### 2.3 Required Code Change in `deep_scene.rs`

```rust
// Before (bug):
DeepView::Roster | DeepView::Recruit => {
    super::deep_roster::render_roster(buffer, width, height, deep, ui, ctx);
}

// After (fix):
DeepView::Roster => {
    super::deep_roster::render_roster(buffer, width, height, deep, ui, ctx);
}
DeepView::Recruit => {
    super::deep_roster::render_recruit(buffer, width, height, deep, ui, ctx);
}
```

---

## Part 3: Layer Map — Depth Progression and Infrastructure Clarity

### 3.1 Current Problems (Confirmed by Audit)

**Familiarity tier labels never shown (P1.1):**
- Shows `"Intel: 42%"` — not the named level (Mapped), not its effect (-10% duration)
- The audit calls this a high-value/low-complexity fix

**Tier color suppressed in compact mode (P2.5):**
- `let _ = tc;` on line 155 of `deep_layers.rs` — intentionally ignores the tier color
- Layer numbers in compact mode render in `Color::White` instead of tier color

**Infrastructure costs hidden (P1.5):**
- Unbuilt infrastructure shows description but no Warband Marks cost
- Players cannot plan builds without knowing cost

**Build action missing (P2.4):**
- The Layers view is read-only; no keybind to build infrastructure
- Players must know to use the mission system for Construction

**Flat list with no depth metaphor:**
- Tier boundaries are not visible in the list
- All layers look the same regardless of depth

**Familiarity bar has no threshold markers:**
- Bar is filled 0-100% but thresholds at 25/50/75 are not marked
- No way to know how far away the next named tier is

### 3.2 Redesigned Layer List — Compact Mode

**Before (buggy, all White):**
```
  L 1   The Shallows    CLEAR    [2/4]
► L 4   Brackwater      FRONT
  L 5   ???
```

**After (tier colors, status glyphs, tier headers):**
```
════ The Shallows (L1-3) ═══════════
✓  L 1  The Mirefall   ▓▓▓▓▓▓▓▓▓▓  [OC  ]
✓  L 2  Dustbone       ▓▓▓▓▓▓▓░░░  [O   ]
✓  L 3  Ashcroft       ▓▓▓▓░░░░░░  [    ]
════ The Warrens (L4-7) ════════════
►  L 4  Brackwater     ▓░░░░░░░░░  [    ]  FRONTIER
?  L 5  ???
```

- Layer number colored in tier color (not suppressed)
- Status glyphs: `✓` cleared (`Color::Green`), `►` frontier+selected (`Color::Cyan`), `?` unknown (`Color::DarkGray`)
- 8-char familiarity mini-bar: `▓` filled `Color::Cyan`, `░` empty `Color::Rgb(30,40,60)`
- 6-char infra slot: `[OCWB]` where each letter is its initial or space
- Tier section headers in tier color with `════` dividers in `Color::Rgb(40,60,80)`

### 3.3 Redesigned Layer List — Split Mode (Left Panel)

**Header:**
```
  #    Name               Fam     Infra   Status
```

**Rows:**
```
════ The Shallows ════════════════════════════════════
✓  L 1  The Mirefall   ▓▓▓▓▓▓▓▓░░  [OC  ]  Cleared
✓  L 2  Dustbone       ▓▓▓▓▓▓░░░░  [O   ]  Cleared
✓  L 3  Ashcroft       ▓▓▓░░░░░░░  [    ]  Cleared
════ The Warrens ════════════════════════════════════
►  L 4  Brackwater     ▓░░░░░░░░░  [    ]  FRONTIER
   L 5  ???
```

Tier headers appear as rows at every tier boundary, using `═` character in `Color::Rgb(40,60,80)` with tier name in tier color.

### 3.4 Redesigned Layer Detail Panel — Cleared Layer

**Full mockup:**
```
Layer 2 — Dustbone
The Shallows  ·  Cleared

Familiarity: Mapped (45%)
  ▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░
  [0%]·········[25%]·····[50%]·····[75%]·[100%]
  Effect: -10% mission duration

Infrastructure  [2/4 built]
  [✓] Outpost      -25% duration on this layer
  [✓] Supply Cache +50% Marks from supply runs
  [ ] Watchtower   +25 intel instantly           74M
  [ ] Bridge       Skip this layer on deep push  85M

Duration after all bonuses:
  Supply Run:  2.0h → 1.35h
  Recon:       4.0h → 2.70h
  Expedition:  8.0h → 5.40h
```

**Key design decisions:**
- Familiarity label: `"Mapped (45%)"` not `"Intel: 45%"` — uses the game term from `FamiliarityLevel::display_name()`
- Familiarity color by tier: Unknown=`DarkGray`, Mapped=`Cyan`, Familiar=`Green`, Mastered=`Rgb(255,215,0)`
- Threshold markers on the bar row (below the fill bar)
- Effect text: concrete description of the named tier's bonus
- Infrastructure: `[✓]` built in `Color::Green`, `[ ]` unbuilt in `Color::DarkGray`
- Costs shown on right for unbuilt slots only, in `Color::DarkGray`
- Duration section: base hours × combined modifier factor, shown as `X.Xh → Y.Yh`

**Familiarity threshold marker implementation:**
```
Row 0:  ▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░  (fill bar, 21 chars wide)
Row 1:  ▲        ▲        ▲    (tick marks at 25%, 50%, 75%)
```

Tick position calculation: `tick_col = col + (threshold * bar_width / 100)`

### 3.5 Redesigned Layer Detail Panel — Frontier Layer

```
Layer 4 — Brackwater
The Warrens  ·  FRONTIER

Familiarity: Unknown (8%)
  ▓░░░░░░░░░░░░░░░░░░░░
  [0%]·········[25%]·····[50%]·····[75%]·[100%]
  Effect: No duration bonus yet. Explore to unlock.

Power Required
  Supply Run:    25   (safe farming)
  Recon:         40   (low risk, build intel)
  Expedition:    55   (medium risk, primary XP)
  Breakthrough:  75   (one-time — clears this layer)

Infrastructure: Available after Breakthrough
  [ ] Outpost      -25% duration        Cost: 76M
  [ ] Supply Cache +50% Marks yield     Cost: 100M
  [ ] Watchtower   +25 intel instantly  Cost: 86M
  [ ] Bridge       Skip this layer      Cost: 120M

Next step: Send a Breakthrough mission to clear this layer.
```

**Power threshold section:** Uses `layer_power_thresholds(layer.index)` from `layers.rs`. Shows all four mission types with concise parenthetical explaining the mission.

**Infrastructure for uncleared layers:** Shown dimmed (`Color::DarkGray`) with costs visible but labeled "Available after Breakthrough."

### 3.6 Redesigned Layer Detail Panel — Unknown Layer

```
Layer 5 — ???
The Warrens  ·  UNKNOWN

 Nothing is known about this layer yet.
 Clear Layer 4 (Brackwater) to reveal it.

 Estimated power required:
  Breakthrough: ~95  (based on tier progression)
```

### 3.7 Infrastructure Build Action (P2.4)

Add a `[B]` keybind to the Layers detail view for uncleared→cleared layers with buildable slots:

**In `deep_layers.rs` detail panel footer:**
```
[↑/↓] Navigate   [B] Build Infrastructure   [Esc] Back
```

When `[B]` is pressed on a cleared layer with available slots and sufficient Marks, open a sub-menu:

```
BUILD INFRASTRUCTURE — Layer 2

  [O] Outpost       -25% duration            Cost: 72M
  [C] Supply Cache  +50% Mark yield          Cost: 90M
  [W] Watchtower    +25 intel on build        Cost: 82M
  [B] Bridge        Skip layer on deep push   Cost: 105M

  Balance: 240M
  [Enter] Build   [Esc] Cancel
```

The build action triggers `build_infrastructure()` from `layers.rs` and deducts Warband Marks. This requires adding a build sub-view state to `DeepUiState` (e.g., a `building_layer: Option<u32>` field) and a new input handler branch.

**Implementation note:** The input handler in `deep_input.rs` / `handle_infrastructure` already exists. Add a `KeyCode::Char('b') | KeyCode::Char('B')` branch that sets a new `ui.build_mode = true` flag, and render the sub-menu when that flag is set.

### 3.8 Tier Color Bug Fix (P2.5)

In `src/ui/deep_layers.rs` line 155, change:
```rust
// Before (bug — tier color computed then discarded):
let _ = tc;
row += 1;

// After (fix — apply tier color to layer number):
put_text(buffer, row, 3, &format!("L{:2}", layer.index), tc);
row += 1;
```

This matches the behavior already present in the split view.

### 3.9 Familiarity Tier Colors

Consistent color mapping for familiarity tier labels:
- Unknown (0-24%): `Color::DarkGray`
- Mapped (25-49%): `Color::Cyan`
- Familiar (50-74%): `Color::Green`
- Mastered (75-100%): `Color::Rgb(255, 215, 0)` (gold)

---

## Part 4: Tab Bar State Indicators (P1.3)

### 4.1 Current Tab Bar

```
[Hub] [Missions] [Roster] [Layers] [Recruit]
```

Active tab in `Color::Rgb(80, 160, 220)`, others in `Color::DarkGray`.

### 4.2 Proposed Tab Bar with State Badges

```
[Hub] [Missions 3] [Roster ⚡2] [Layers] [Recruit ●]
```

Badge rules (badges appear only when their tab is not active):

| Tab | Badge | Condition | Color |
|-----|-------|-----------|-------|
| Hub | `⚡N` | N events pending | `Color::Yellow` |
| Hub | `✓N` | N results awaiting collection | `Color::Green` |
| Missions | `N` | N missions in pool | `Color::Cyan` |
| Roster | `⚕N` | N mercs injured or lost | `Color::Yellow` |
| Recruit | `●` | Pool has candidates and is fresh | `Color::Cyan` |
| Recruit | `!` | Pool refresh due in < 1h | `Color::Yellow` |

**Implementation in `deep_scene.rs`:**
```rust
fn tab_badge(view: DeepView, deep: &DeepState, now: DateTime<Utc>) -> Option<(String, Color)> {
    match view {
        DeepView::Hub => {
            let events = deep.prestige.active_missions.iter()
                .filter(|m| m.has_pending_event()).count();
            let results = deep.prestige.pending_results.len();
            if events > 0 {
                Some((format!("⚡{}", events), Color::Yellow))
            } else if results > 0 {
                Some((format!("✓{}", results), Color::Green))
            } else {
                None
            }
        }
        DeepView::NewMission => {
            let n = deep.prestige.available_missions.len();
            if n > 0 { Some((n.to_string(), Color::Cyan)) } else { None }
        }
        DeepView::Roster => {
            let injured = deep.prestige.roster.iter()
                .filter(|m| !matches!(m.status, MercStatus::Available | MercStatus::OnMission(_)))
                .count();
            if injured > 0 { Some((format!("⚕{}", injured), Color::Yellow)) } else { None }
        }
        DeepView::Recruit => {
            if deep.prestige.recruit_pool.candidates.is_empty() {
                None
            } else if deep.prestige.recruit_pool.needs_refresh(now) {
                Some(("!".to_string(), Color::Yellow))
            } else {
                Some(("●".to_string(), Color::Cyan))
            }
        }
        DeepView::Infrastructure | DeepView::EventResponse => None,
    }
}
```

Tab label rendering with badge:
```rust
let label = format!("[{}{}]",
    tab.tab_label(),
    badge.as_ref().map(|(s, _)| format!(" {}", s)).unwrap_or_default()
);
put_text(buffer, tab_row, col, &label, if is_active { active_color } else { inactive_color });
if let Some((badge_str, badge_color)) = &badge {
    // Re-render just the badge portion in badge color
    let badge_col = col + tab.tab_label().len() as i32 + 2; // "[label "
    put_text(buffer, tab_row, badge_col, badge_str, *badge_color);
}
```

---

## Part 5: Construction Mission Label Fix (P0.3)

**In `src/ui/deep_missions.rs`**, wherever `m.mission_type.display_name()` is used in the mission list:

```rust
// Before:
let type_label = m.mission_type.display_name().to_string();

// After:
let type_label = match m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
```

This is a two-line change that immediately makes Construction missions self-describing. "Build Outpost", "Build Bridge", "Build Supply Cache", "Build Watchtower" — no ambiguity.

---

## Part 6: Implementation Notes and Helper Functions

### 6.1 New Helper Functions for `deep_roster.rs`

```rust
/// Archetype 3-letter abbreviation in brackets.
fn archetype_abbrev(archetype: MercArchetype) -> &'static str {
    match archetype {
        MercArchetype::Vanguard  => "[VAN]",
        MercArchetype::Scout     => "[SCT]",
        MercArchetype::Arcanist  => "[ARC]",
        MercArchetype::Medic     => "[MED]",
        MercArchetype::Saboteur  => "[SAB]",
    }
}

/// Role tag and description for the detail panel.
fn archetype_role_desc(archetype: MercArchetype) -> (&'static str, &'static str) {
    match archetype {
        MercArchetype::Vanguard  => ("Frontline tank",
            "High Power + Resilience. Reduces squad casualties."),
        MercArchetype::Scout     => ("Recon specialist",
            "High Expertise. Better auto-resolve, faster missions."),
        MercArchetype::Arcanist  => ("Elemental expert",
            "Highest Expertise. Counters hazards. Fragile."),
        MercArchetype::Medic     => ("Squad healer",
            "Highest Resilience. Prevents permanent loss."),
        MercArchetype::Saboteur  => ("Trap specialist",
            "High Expertise. Speeds missions, alternate routes."),
    }
}

/// Cumulative missions completed to reach a given level (level 1 = 0).
fn missions_to_reach_level(level: u32) -> u32 {
    (1..level).map(|l| Mercenary::missions_to_next_level(l)).sum()
}

/// (progress_within_level, missions_needed_for_this_level)
fn level_progress(merc: &Mercenary) -> (u32, u32) {
    let base = missions_to_reach_level(merc.level);
    let needed = Mercenary::missions_to_next_level(merc.level);
    let progress = merc.missions_completed.saturating_sub(base).min(needed);
    (progress, needed)
}

/// Injury severity label and color from missions_remaining.
fn injury_severity_display(missions_remaining: u32) -> (&'static str, Color) {
    match missions_remaining {
        1 => ("Light injury", Color::Yellow),
        2 => ("Moderate injury", Color::LightRed),
        _ => ("Severe injury", Color::Red),
    }
}

/// Hour estimate from missions_remaining (1 mission ≈ 6h).
fn injury_hours_estimate(missions_remaining: u32) -> u32 {
    missions_remaining * 6
}
```

### 6.2 New Helper Functions for `deep_layers.rs`

```rust
/// Familiarity level label and color.
fn familiarity_label_color(familiarity: u8) -> (&'static str, Color) {
    match familiarity {
        0..=24  => ("Unknown",  Color::DarkGray),
        25..=49 => ("Mapped",   Color::Cyan),
        50..=74 => ("Familiar", Color::Green),
        _       => ("Mastered", Color::Rgb(255, 215, 0)),
    }
}

/// Effect text for a familiarity level.
fn familiarity_effect_text(familiarity: u8) -> &'static str {
    match familiarity {
        0..=24  => "No duration bonus yet",
        25..=49 => "-10% mission duration",
        50..=74 => "-20% mission duration",
        _       => "-30% duration, +15% Mark yield",
    }
}

/// Infrastructure 4-slot display string: "[OCWB]".
fn infra_slots_str(layer: &LayerRecord) -> String {
    let o = if layer.has_infrastructure(Infrastructure::Outpost)     { 'O' } else { ' ' };
    let c = if layer.has_infrastructure(Infrastructure::SupplyCache) { 'C' } else { ' ' };
    let w = if layer.has_infrastructure(Infrastructure::Watchtower)  { 'W' } else { ' ' };
    let b = if layer.has_infrastructure(Infrastructure::Bridge)      { 'B' } else { ' ' };
    format!("[{}{}{}{}]", o, c, w, b)
}

/// Familiarity bar with threshold markers, rendered into scene buffer.
/// Renders two rows: fill bar at `row`, tick marks at `row+1`.
fn render_familiarity_bar_with_thresholds(
    buffer: &mut [Vec<SceneCell>],
    row: i32, col: i32, bar_width: usize, familiarity: u8,
) {
    let ratio = familiarity as f64 / 100.0;
    let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
    for i in 0..filled {
        put_cell(buffer, row, col + i as i32, '▓', Color::Cyan);
    }
    for i in filled..bar_width {
        put_cell(buffer, row, col + i as i32, '░', Color::Rgb(30, 40, 60));
    }
    // Threshold tick marks below
    for threshold in [25usize, 50, 75] {
        let tick_col = col + (threshold * bar_width / 100) as i32;
        put_cell(buffer, row + 1, tick_col, '▲', Color::DarkGray);
    }
}

/// Duration in seconds as "X.Xh".
fn format_hours(secs: u64) -> String {
    format!("{:.1}h", secs as f64 / 3600.0)
}

/// Compute duration after infrastructure and familiarity bonuses.
fn effective_duration_hours(
    tier: LayerTier,
    mission_type: MissionType,
    layer: &LayerRecord,
) -> f64 {
    let base = crate::deep::layers::base_mission_duration_secs(tier, mission_type) as f64;
    let outpost = if layer.has_infrastructure(Infrastructure::Outpost) { 0.75 } else { 1.0 };
    let fam = crate::deep::layers::FamiliarityLevel::from_familiarity(layer.familiarity)
        .duration_factor();
    (base * outpost * fam) / 3600.0
}
```

### 6.3 Import Additions

In `src/ui/deep_layers.rs`, add:
```rust
use crate::deep::layers::{
    FamiliarityLevel, infrastructure_build_cost, layer_power_thresholds,
    base_mission_duration_secs,
};
```

In `src/ui/deep_roster.rs`, add:
```rust
use crate::deep::mercenaries::stats_at_level;
```

### 6.4 Recommended Type Change

Add `pub quality: MercQuality` to `Mercenary` in `src/deep/types.rs` and move `MercQuality` into `types.rs` (or re-export from `mercenaries.rs`). Update `generate_mercenary` to store it. Without this, quality display requires stat inference which is imprecise.

---

## Part 7: Responsive Degradation

### XL/L (width >= 80)
- Full split layout
- Tier section headers, full detail panels
- Infrastructure detail cards with costs
- Duration-after-bonuses calculations
- Tab bar with state badges

### M (60-79)
- Abbreviated split (50/50)
- Tier section headers condensed to `═ Shallows ═`
- Detail panel: role description 1 line only, no duration math
- Infrastructure: name + main effect, no costs

### S (< 60)
- Compact single-column list
- Tier section headers as 1-char indicators (tier color on layer number)
- No detail panel; status shown below selected item inline
- Tab bar badges still rendered (they fit in < 5 chars)

---

## Part 8: Summary of Changes Required

### `src/ui/deep_scene.rs`
- Fix `DeepView::Recruit` dispatch to call `render_recruit()` instead of `render_roster()`
- Add `tab_badge()` helper and update tab bar rendering to show badges

### `src/ui/deep_roster.rs`
- Add `render_recruit()` function (entire new function)
- Rewrite compact roster header and rows with fixed column offsets (no rfind)
- Rewrite split roster detail panel with role description, stat hints, progress bar, injury hours estimate
- Add all helper functions from section 6.1

### `src/ui/deep_layers.rs`
- Fix compact mode tier color bug (remove `let _ = tc`)
- Add tier section headers to both compact and split list
- Expand familiarity display to named tier + effect text
- Add familiarity bar with threshold tick marks
- Expand infrastructure display to include costs and concrete descriptions
- Add duration-after-bonuses section for cleared layers
- Add power threshold section for frontier layers
- Add build sub-menu state and `[B]` keybind handling
- Add all helper functions from section 6.2

### `src/ui/deep_missions.rs`
- Fix Construction mission label (P0.3): replace `display_name()` with match on Construction payload

### `src/deep/types.rs` (optional but recommended)
- Add `quality: MercQuality` field to `Mercenary`

### `src/deep/mercenaries.rs` (if quality field added)
- Re-export `MercQuality` or move it to `types.rs`
- Set `quality` field in `generate_mercenary()`
