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
