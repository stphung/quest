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
