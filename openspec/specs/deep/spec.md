# The Deep — Mercenary Expeditions Specification

## Purpose

The Deep is an endgame (prestige rank 15+) meta-system in which the player recruits and manages a mercenary company that runs long, real-time (wall-clock) expeditions down through numbered Layers of a vast underground structure. It runs on a fundamentally different timescale from moment-to-moment combat: missions take hours or days, progress while the game is closed, and never punish absence. Clearing Layers through Breakthrough missions is the sole engine that unlocks Fracture zones and Power Cores for the main game, upgrades the mercenary guild, and ultimately opens the Gateway that leads to later endgame content. This capability governs discovery, Layer progression, the mission lifecycle, rewards, guild rank, and the Layer-depth → unlock mappings.

## Requirements

### Requirement: Discovery Gate and Starter Setup

The system SHALL keep The Deep undiscovered and inaccessible until the player defeats The Endless (the Zone 11 "Expanse cycle" final boss) for the first time while at prestige rank 15 or higher. Discovery SHALL be a deterministic consequence of that boss kill, with no per-tick random roll. On discovery the system SHALL mark The Deep discovered, create a starter roster of 3 Common-quality mercenaries, generate the initial available-mission pool, generate the daily recruit pool, queue the guaranteed-success "First Orders" tutorial mission, and grant starting Warband Marks scaled by guild rank: 50 at Rank 1, 100 at Rank 2, 200 at Rank 3, 350 at Rank 4, and 500 at Rank 5.

#### Scenario: Endless killed below the prestige gate

- **WHEN** the player defeats The Endless (Expanse cycle boss) while below prestige rank 15
- **THEN** The Deep remains undiscovered and no roster, missions, or marks are created

#### Scenario: First qualifying Endless kill

- **WHEN** the player defeats The Endless for the first time at prestige rank 15 or higher and The Deep is not yet discovered
- **THEN** The Deep is marked discovered, 3 starter mercenaries and the mission and recruit pools are created, the First Orders mission is queued, and starting Warband Marks are granted according to the current guild rank

#### Scenario: Already discovered

- **WHEN** The Endless is defeated again after The Deep is already discovered
- **THEN** discovery does not run a second time and the existing roster, marks, and pools are left unchanged

### Requirement: Cross-Prestige Persistence and Generations

The system SHALL persist all Deep state — mercenaries, active missions, Warband Marks, guild rank, Layer clears, familiarity, and infrastructure — across prestige resets; none of it is wiped when the player prestiges. Account-level state (guild rank, Layer records, gateway status, fracture cap) SHALL live in a separate account-level store, while operational state (roster, missions, marks) SHALL live with the character. On each prestige the system SHALL only advance the generation counter, snapshot the finished generation's summary statistics into a capped history, and set the new generation number.

#### Scenario: Prestige preserves operational state

- **WHEN** the player prestiges after building up mercenaries, marks, and cleared Layers
- **THEN** the roster, Warband Marks, cleared Layers, familiarity, and infrastructure all carry forward unchanged

#### Scenario: Generation advance on prestige

- **WHEN** a prestige occurs
- **THEN** the generation counter increments, a generation record (marks earned, missions completed, mercs lost, deepest Layer reached, gateway status) is appended, and the record history retains at most the 10 most recent generations

### Requirement: Layer Frontier, Clearing, and Tiers

The system SHALL model The Deep as 1-based numbered Layers grouped into tiers: The Shallows (1–3), The Warrens (4–7), The Hollows (8–12), The Sunken Reach (13–18), The Abyss (19–25), and The Void (26+). A Layer SHALL be cleared only by completing a successful Breakthrough mission on it, which also updates the deepest-Layer-reached marker. The frontier SHALL be the first uncleared Layer, or deepest-reached + 1 when all reached Layers are cleared. Clearing Layer 18 SHALL additionally grant Layer 19 at least Mapped (25) familiarity without lowering any higher existing value.

#### Scenario: Breakthrough clears a Layer

- **WHEN** a Breakthrough mission on the frontier Layer resolves as a Success
- **THEN** that Layer is marked cleared and, if it is deeper than any before, the deepest-Layer-reached marker is raised to it

#### Scenario: Frontier is the first uncleared Layer

- **WHEN** Layers 1 and 2 are cleared and Layer 3 is not
- **THEN** the frontier is Layer 3, and safe (cleared) missions are only offered on Layers 1 and 2

#### Scenario: Abyss entry bonus

- **WHEN** Layer 18 is cleared
- **THEN** Layer 19's familiarity is raised to at least 25 (Mapped) unless it was already higher

### Requirement: Guild Rank Progression and Capacity

The system SHALL define five guild ranks, each fixing the roster capacity, the base concurrent-mission cap, and the Layer breakthrough required to unlock it: Rank 1 Freelancers (roster 5, 1 concurrent, no requirement), Rank 2 Company (7, 2, Layer 3), Rank 3 Battalion (9, 2, Layer 7), Rank 4 Legion (12, 3, Layer 13), Rank 5 Vanguard (15, 4, Layer 19). Upgrading SHALL cost Warband Marks — 200 (1→2), 500 (2→3), 1200 (3→4), 3000 (4→5) — and SHALL be rejected (with marks untouched) when at max rank, when the required Layer is not cleared, or when marks are insufficient. As an early bonus, a Rank 1 guild that has reached Layer 3 SHALL gain one extra concurrent-mission slot (effective cap 2) before formally reaching Rank 2.

#### Scenario: Valid guild upgrade

- **WHEN** a Rank 1 guild has cleared Layer 3 and holds at least 200 Warband Marks and requests an upgrade
- **THEN** the rank becomes Rank 2, 200 marks are deducted, and roster capacity and concurrent-mission cap rise to their Rank 2 values

#### Scenario: Upgrade blocked by unmet Layer requirement

- **WHEN** a Rank 1 guild attempts to upgrade without Layer 3 cleared
- **THEN** the upgrade is rejected with a Layer-requirement error and no marks are spent

#### Scenario: Early concurrency bonus at Rank 1

- **WHEN** a Rank 1 guild has reached Layer 3
- **THEN** its effective concurrent-mission limit is 2 instead of the base 1

### Requirement: Mercenary Roster, Archetypes, and Injuries

The system SHALL populate the roster with mercenaries of five archetypes whose base (Power, Resilience, Expertise) stats are Vanguard (14,12,4), Scout (8,10,12), Arcanist (10,6,14), Medic (6,14,10), and Saboteur (10,8,12). Recruit-pool archetype availability SHALL widen with rank: Rank 1 offers Vanguard/Scout/Medic, Rank 2 adds Arcanist, and Rank 3 and above add Saboteur. Recruit quality SHALL be rolled from a rank-dependent distribution (Rank 1: 100% Common; Rank 5: 20% Uncommon, 50% Rare, 30% Elite) and priced in Warband Marks by tier — Common 50–80, Uncommon 80–130, Rare 130–200, Elite 200–300 — rounded to the nearest 5. A mercenary's effective Power SHALL be base + (level−1)×3 and effective Resilience base + (level−1)×2; mercenaries SHALL cap at level 20. Injuries SHALL set a wall-clock recovery time (Light 4–8h, Moderate 8–12h, Severe 12–16h) that heals automatically each tick and on load, so an injured roster can never soft-lock.

#### Scenario: Effective power scales with level

- **WHEN** a Vanguard (base Power 14) reaches level 3
- **THEN** its effective Power is 14 + (3−1)×3 = 20

#### Scenario: Injured mercenary recovers on wall-clock time

- **WHEN** a mercenary is injured with a recovery time and that wall-clock time later passes
- **THEN** the mercenary returns to Available status without needing any mission to run

#### Scenario: Rank-gated archetype availability

- **WHEN** the guild is Rank 1
- **THEN** the recruit pool contains only Vanguard, Scout, and Medic candidates

### Requirement: Mission Types, Power Thresholds, Durations, and Familiarity

The system SHALL offer mission types with distinct risk, event counts, and placement: Supply Run (safe, cleared Layers, 0 events), Recon (low risk, frontier, 1 event), Expedition (medium risk, frontier, 2 events), Breakthrough (high risk, frontier, once per Layer, up to 5 events, clears the Layer), Construction (safe, cleared Layers, builds infrastructure), and Gateway Expedition (Layer 30 only, fixed 72 hours, up to 5 events). Each mission SHALL carry a recommended squad Power threshold read from the per-Layer table (Layer 1 Breakthrough 25 / Expedition 20 / Recon 15 / Supply Run 10; Layer 25 700 / 525 / 395 / 265; Void Layers 26+ scale as 700+60n / 525+45n / 395+35n / 265+25n where n = Layer−25; Construction uses the Supply Run threshold). Base durations SHALL scale by tier (Shallows Supply Run 1h through Void Breakthrough 40h). Familiarity (0–100) SHALL band into Unknown (0–24), Mapped (25–49), Familiar (50–74), Mastered (75–100), reducing mission duration by 0%, 15%, 30%, and 45% respectively, improving auto-resolve success (0.65 / 0.75 / 0.85 / 0.95), and increasing Mark yield (Familiar ×1.10, Mastered ×1.25). Completing a mission SHALL grant familiarity — Supply Run 2, Recon 5, Expedition 15, Breakthrough 15, Construction 5 — capped at 100.

#### Scenario: Effective duration combines modifiers

- **WHEN** a mission runs on a Layer with an Outpost (−25%) built and Familiar (−30%) familiarity
- **THEN** its base tier duration is multiplied by both reductions, with Bridge reductions (−2% per bridged Layer below, capped at −30%) also applied and the combined effect bounded, while a Gateway Expedition stays fixed at 72 hours regardless of any modifier

#### Scenario: Familiarity accrues and caps

- **WHEN** a Layer at familiarity 95 completes an Expedition (+15)
- **THEN** its familiarity becomes 100 (capped), not 110

#### Scenario: Void power threshold scaling

- **WHEN** a Breakthrough is offered on Layer 26
- **THEN** its recommended squad Power threshold is 700 + 60×1 = 760

### Requirement: Infrastructure Construction

The system SHALL allow up to four distinct infrastructure types to be built, one of each, on any cleared Layer, persisting across prestiges: Outpost (−25% mission duration on that Layer), Supply Cache (Supply Runs on that Layer yield +75% Marks), Watchtower (grants +40 familiarity immediately on build and improves auto-resolve), and Bridge (−2% mission duration per bridged Layer below, capped at −30%). Build costs in Warband Marks SHALL scale with Layer depth: Outpost 85+6×Layer, Supply Cache 110+7×Layer, Watchtower 100+6×Layer, Bridge 140+7×Layer. Building SHALL be rejected if the Layer is not cleared or if that infrastructure type is already present.

#### Scenario: Watchtower grants familiarity on build

- **WHEN** a Watchtower is built on a cleared Layer at familiarity 30
- **THEN** the infrastructure is recorded and that Layer's familiarity rises to 70 (capped at 100)

#### Scenario: Build rejected on uncleared Layer

- **WHEN** a Construction is attempted on a Layer that has not been cleared
- **THEN** the build fails with a not-cleared error and no infrastructure is added

#### Scenario: No duplicate infrastructure

- **WHEN** a second Outpost is attempted on a Layer that already has one
- **THEN** the build fails with an already-built error

### Requirement: Squad Assignment and Mission Launch

The system SHALL validate a squad before launching a mission and reject it when the squad is empty, when the effective concurrent-mission limit is already reached, when any chosen mercenary is not Available, when total effective squad Power is below the mission's minimum, when a required archetype is absent from the squad, or when Warband Marks are insufficient for the launch cost. Launch costs SHALL be Supply Run 5+Layer, Recon 30+Layer, Expedition 80+3×Layer, Breakthrough/Gateway 300+25×Layer, and Construction equal to the infrastructure build cost. On launch the system SHALL deduct the cost, set each assigned mercenary to on-mission, schedule the mission's check-in events, and compute the mission's wall-clock end time. One free daily Supply Run per UTC calendar day SHALL be available that skips the Marks cost but is forced to a minimum duration of 3 hours; it resets at the next UTC midnight.

#### Scenario: Launch rejected for insufficient power

- **WHEN** the summed effective Power of the selected squad is below the mission's minimum squad Power
- **THEN** the launch is rejected with an insufficient-power error and no marks are spent

#### Scenario: Concurrent-mission limit enforced

- **WHEN** the number of active missions already equals the effective concurrent limit
- **THEN** any further launch is rejected with a concurrent-limit error

#### Scenario: Free daily Supply Run

- **WHEN** the free daily Supply Run has not been used this UTC day and the player launches one
- **THEN** no Warband Marks are deducted, the mission runs for at least 3 hours, and the free slot becomes unavailable until the next UTC midnight

### Requirement: Wall-Clock Progression and Offline Resolution

The system SHALL run missions on real wall-clock time using UTC timestamps, so missions advance even while the game is closed. The live tick SHALL NOT simulate mission progress; it SHALL only detect check-in events whose scheduled time has arrived and detect completed missions by comparing the current time to each mission's end time. On load, offline resolution SHALL resolve every mission that finished while the game was closed and auto-resolve any check-in events that were missed. Completed missions SHALL move into a pending-results queue for the player to review, and the warband log SHALL retain at most the 10 most recent mission outcomes.

#### Scenario: Mission completes while game is closed

- **WHEN** a mission's end time passed while the game was not running and the game is reopened
- **THEN** offline resolution resolves the mission, applies its outcome and rewards, and places it in the pending-results queue

#### Scenario: Tick detects a completed mission

- **WHEN** the current wall-clock time reaches or passes an active mission's end time with no unresolved pending event
- **THEN** the mission is resolved that tick and its result is queued for review

### Requirement: Check-In Events and Optional Auto-Resolve

The system SHALL schedule check-in events at progress milestones during risk-bearing missions (Supply Run and Construction have none; Recon up to 1, Expedition up to 2, Breakthrough and Gateway Expedition up to 5), each offering the player a choice with a designated safe (non-risky) auto-resolve fallback. If the player does not respond within 2 hours of an event firing, or is offline, the system SHALL auto-resolve it using the safe fallback so a mission can never stall on an unanswered event. Should the mission timer elapse while events remain unresolved, the system SHALL force-auto-resolve them without applying their time deltas.

#### Scenario: Unanswered event auto-resolves after timeout

- **WHEN** a check-in event has been pending for at least 2 hours without a player response
- **THEN** the event resolves to its safe auto-resolve choice and the mission continues

#### Scenario: Safe missions raise no events

- **WHEN** a Supply Run or Construction mission runs
- **THEN** no check-in events fire during it

### Requirement: Mission Outcome Determination

The system SHALL treat Supply Run and Construction missions as always Success. For risk-bearing missions it SHALL compute a power ratio of total effective squad Power to the mission's Power threshold (treating a zero threshold as ratio 2.0), optionally reduced on Breakthroughs by a per-auto-resolved-event penalty (base 10% per fully auto-resolved event fraction, softened 5% by a Watchtower on that Layer), then roll the outcome: at effective ratio ≥ 1.5, 95% Success else Partial Success; at ratio ≥ 1.0, Success with probability clamp(0.60 + ratio×0.25, 0.60, 0.90) else Partial Success; at ratio ≥ 0.75, 30% Success / 50% Partial Success / 20% Failure; below 0.75, 50% Partial Success / 50% Failure.

#### Scenario: Overpowered squad

- **WHEN** a risk-bearing mission's effective power ratio is 1.5 or greater
- **THEN** the outcome is Success roughly 95% of the time and otherwise Partial Success (never Failure)

#### Scenario: Underpowered squad

- **WHEN** the effective power ratio is below 0.75
- **THEN** the outcome is only ever Partial Success or Failure, never a full Success

#### Scenario: Safe mission auto-succeeds

- **WHEN** a Supply Run or Construction mission resolves
- **THEN** its outcome is Success regardless of squad power

### Requirement: Warband Marks Rewards

The system SHALL award Warband Marks on mission completion by taking the per-type, per-Layer base value (Layer 1: Supply Run 35, Recon 50, Expedition 130, Breakthrough 70; Layer 25: 200 / 280 / 695 / 269; Construction always 0), then applying in order: a ×1.75 Supply Cache bonus (Supply Runs only), the familiarity Mark-yield multiplier (Familiar ×1.10, Mastered ×1.25), the outcome multiplier (Success ×1.0, Partial Success ×0.60, Failure ×0.20), and a ±15% random variance, rounding the result to a whole number. Earned Marks SHALL be added to the operational balance and counted toward lifetime totals.

#### Scenario: Outcome scales the reward

- **WHEN** two otherwise-identical Expeditions resolve, one Success and one Failure
- **THEN** the Failure awards about 20% of the Success's Marks

#### Scenario: Supply Cache boosts only Supply Runs

- **WHEN** a Supply Cache exists on a Layer
- **THEN** Supply Run Mark rewards there are multiplied by 1.75 while Expedition and other rewards are unaffected

#### Scenario: Construction yields no Marks

- **WHEN** a Construction mission completes
- **THEN** it awards 0 Warband Marks (its value is the infrastructure it builds)

### Requirement: Layer Breakthroughs Unlock Fracture Zones and Power Cores

The system SHALL, when a Breakthrough clears one of the milestone Layers 3, 7, 12, 18, 25, or 30, raise the account-level Fracture zone cap and queue a world-event unlock for the corresponding Fracture region, and unlock the corresponding Power Core, per this exact mapping: Layer 3 → Red Fault, Zones 12–14 (cap 14), Power Core 2 PR/day; Layer 7 → Mirror Scar, Zones 15–17 (cap 17), 3 PR/day; Layer 12 → Black Mouth, Zones 18–20 (cap 20), 5 PR/day; Layer 18 → Hollow Throne, Zones 21–23 (cap 23), 8 PR/day; Layer 25 → Wailing Reach, Zones 24–26 (cap 26), 12 PR/day; Layer 30 → Origin Wound, Zones 27–30 (cap 30), 18 PR/day. The default Fracture cap before any breakthrough SHALL be 11 (the Expanse only), the cap SHALL only ever increase, and all six Power Cores active SHALL total 48 PR/day.

#### Scenario: First fracture unlock

- **WHEN** Layer 3 is cleared and the current Fracture cap is 11
- **THEN** the cap rises to 14, the Red Fault region unlock is queued for its world-event modal, and the corresponding 2 PR/day Power Core becomes active

#### Scenario: Non-milestone breakthrough

- **WHEN** a Layer that is not 3, 7, 12, 18, 25, or 30 is cleared
- **THEN** no Fracture zone unlock and no Power Core unlock occur

#### Scenario: Cap never regresses

- **WHEN** a breakthrough would set a cap no higher than the current one
- **THEN** the Fracture zone cap is left unchanged

### Requirement: The Gateway at Layer 30

The system SHALL make a one-time Gateway Expedition available once the frontier reaches Layer 30 (the Gateway Layer) and the Gateway is not yet opened. The Gateway Expedition SHALL last a fixed 72 hours regardless of infrastructure or familiarity. A successful Gateway Expedition SHALL mark the Gateway opened, which is a prerequisite (together with The Deep being discovered) for unlocking downstream endgame content.

#### Scenario: Gateway Expedition offered at Layer 30

- **WHEN** the frontier reaches Layer 30 and the Gateway has not been opened
- **THEN** a Gateway Expedition mission becomes available and runs for a fixed 72 hours

#### Scenario: Gateway opens on success

- **WHEN** a Gateway Expedition resolves as a Success
- **THEN** the Gateway is marked opened and remains open thereafter across prestiges
