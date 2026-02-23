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
