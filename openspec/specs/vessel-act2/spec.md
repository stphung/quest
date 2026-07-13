# The Vessel — Act 2 Launch Gate & Voyage Specification

## Purpose

Define Act 2 of Quest: after Zone 50 falls, a signal from a dying branch of Yggdrasil is discovered, and burning 250,000 Prestige Ranks in a single all-or-nothing action launches the Vessel into a wall-clock crossing (the Voyage) toward the Tree. The entire feature ships dark behind a compile-time kill-switch with a runtime override, so it is invisible to players by default while remaining fully implemented. This capability owns the kill-switch contract, the launch gate and its prerequisites, the launch transition, and the systemic shape of the Voyage and its persistent ferry loop; it references Prestige Rank, Ascension, Woven Patterns, and Zone 50 clearance only as gates or inputs.
## Requirements
### Requirement: Act 2 Kill-Switch

The system SHALL keep all Act 2 presentation dark by default via a compile-time flag `ACT2_ENABLED` that is `false` in every released build. A runtime check SHALL treat Act 2 as enabled when `ACT2_ENABLED` is `true` OR the environment variable `QUEST_ACT2` equals `1`, and this decision SHALL be read exactly once (cached), so changing the environment variable after the process starts SHALL have no effect. While Act 2 is disabled the system SHALL surface no discovery modal, no ticker whispers, no stats-panel row, no launch hotkey, and no path to the launch burn.

#### Scenario: Dark by default

- **WHEN** the game runs in a released build with `ACT2_ENABLED = false` and `QUEST_ACT2` unset
- **THEN** the Vessel is fully invisible — no whispers, discovery modal, stats row, hotkey, or launch path appears

#### Scenario: Runtime preview override

- **WHEN** the process starts with the environment variable `QUEST_ACT2=1`
- **THEN** Act 2 presentation is enabled for that process even though `ACT2_ENABLED` is still `false`

#### Scenario: Override is read once

- **WHEN** `QUEST_ACT2` is changed after the runtime check has already been evaluated once
- **THEN** the enabled/disabled decision does not change for the remainder of the process

### Requirement: Signal Discovery On Zone 50 Clear

The system SHALL record that the Vessel signal is discovered the first time the player defeats the Zone 50 final boss, and this record SHALL be persistent and survive prestige. This detection SHALL occur unconditionally — independent of the kill-switch — so that already-qualified players light up the instant Act 2 is enabled later; only the presentation of the discovery (log line, ticker, modal, hotkey, stats row) SHALL be gated on Act 2 being enabled.

#### Scenario: First Zone 50 kill records the signal

- **WHEN** the player defeats the Zone 50 final boss for the first time
- **THEN** the persistent "signal discovered" state is set, regardless of whether Act 2 is enabled

#### Scenario: Detection is not gated by the kill-switch

- **WHEN** the Zone 50 final boss falls while Act 2 is disabled
- **THEN** the signal is still recorded in the save, but no discovery modal, log line, or ticker entry is shown until Act 2 is enabled

### Requirement: Pre-Launch Whispers

The system SHALL, only while Act 2 is enabled, emit an atmospheric ticker whisper roughly every 60 seconds of accumulated play time once the signal is discovered and until the Vessel is launched. Whispers SHALL rotate deterministically (no RNG) through a fixed list of 5 messages, wrapping by index. No whisper SHALL fire before discovery or after launch.

#### Scenario: Whisper fires on the interval

- **WHEN** Act 2 is enabled, the signal is discovered, the Vessel is not launched, and at least 60 seconds of play time have elapsed since the last whisper
- **THEN** the next whisper in the rotating sequence is emitted to the ticker

#### Scenario: Whispers stop at launch

- **WHEN** the Vessel has been launched
- **THEN** no further whispers are emitted regardless of elapsed play time

### Requirement: Launch Gate Prerequisites

The system SHALL permit launching the Vessel only when ALL of the following hold: the signal has been discovered (which implies Zone 50 was cleared), the Vessel is not already launched, the player's Ascension level is at least 10 (Ascension X), the player has completed at least 28 Woven Patterns, and the player currently holds at least 250,000 Prestige Ranks. If any one condition is unmet, launch SHALL be refused.

#### Scenario: All gates met

- **WHEN** the signal is discovered, the Vessel is unlaunched, Ascension level ≥ 10, completed Woven Patterns ≥ 28, and Prestige Rank ≥ 250,000
- **THEN** the launch is permitted

#### Scenario: A single unmet gate refuses launch

- **WHEN** every gate is met except one — for example Ascension level 9, or 27 completed patterns, or a Prestige Rank of 249,999
- **THEN** the launch is refused

### Requirement: All-Or-Nothing Launch Burn

The system SHALL, when a launch is performed and every gate is met, subtract exactly 250,000 Prestige Ranks in a single action, recalculate prestige-derived bonuses, mark derived stats dirty, and set the persistent "launched" state. The spend SHALL be all-or-nothing: if any gate is unmet the launch SHALL change nothing (no partial spend), and a second launch after the first SHALL be refused and leave Prestige Rank unchanged.

#### Scenario: Exact burn on success

- **WHEN** a player holding 253,218 Prestige Ranks with all gates met performs the launch
- **THEN** their Prestige Rank becomes 3,218, prestige bonuses are recalculated, and the Vessel is marked launched

#### Scenario: Below-cost launch spends nothing

- **WHEN** a launch is attempted while holding fewer than 250,000 Prestige Ranks
- **THEN** no Prestige Ranks are deducted and the Vessel remains unlaunched

#### Scenario: No double launch

- **WHEN** a launch is attempted after the Vessel has already been launched
- **THEN** the launch is refused and Prestige Rank is unchanged

### Requirement: Launch Transition

The system SHALL, after the launch burn and before the first Voyage frame, play a fixed 5-beat authored narrative sequence (Farewell, Unweaving, Construction, Launch, Void) once, advanced only by Enter with no cancel path. The sequence SHALL always present the same five beats regardless of how the player reached launch. Completion SHALL set a persistent "transition played" record; while launched but not yet transition-played, the system SHALL show the transition instead of the Voyage. An interrupted transition (the game closed mid-sequence) SHALL restart at the first beat on the next run, since only the persistent completion record is durable.

#### Scenario: Transition plays once through five beats

- **WHEN** the Vessel is launched but the transition has not yet played and the player presses Enter through the sequence
- **THEN** the five beats advance in order, and after the final beat the persistent "transition played" record is set and control passes to the Voyage

#### Scenario: Interruption restarts the beats

- **WHEN** the game is closed during the transition before the final beat is read
- **THEN** on the next run the transition restarts at the first beat, because the transient beat counter is not saved

### Requirement: Voyage Wall-Clock Simulation And Offline Equivalence

The system SHALL advance the Voyage in simulated wall-clock time, computing elapsed whole game-minutes since launch and stepping the state one game-minute at a time until caught up, so that resolving a long absence in one step produces state bitwise-identical to resolving it in many small steps (offline equivalence / chunking invariance). The production clock SHALL run at 2.64 game-minutes per real-minute (a sea-day of 1,440 game-minutes passing in roughly 9 real hours), making the maiden voyage's ~37 sea-days take roughly two real weeks. A development override environment variable (`QUEST_VOYAGE_TIME_SCALE`) MAY replace the production scale but SHALL NOT change the offline-equivalence property.

#### Scenario: Long gap equals many short ticks

- **WHEN** the Voyage is ticked once after a long real-time absence versus ticked repeatedly across many short intervals covering the same span
- **THEN** the resulting Voyage state is identical

#### Scenario: Production clock scale

- **WHEN** the Voyage runs at the production time scale
- **THEN** each game-minute passes at 2.64 game-minutes per real-minute, so a full 1,440-minute sea-day elapses in about 9 real hours

### Requirement: Voyage Route And Phase Progression

The system SHALL move a single ship along a static route graph shaped as a spine with diamond branches that split at junctions and rejoin within the same chapter, ending at a single terminal waypoint (the Tree). The Voyage state SHALL always be in exactly one phase: Traveling a road, Drifting in place (a road's hold ran dry), Holding Station at a waypoint, or Arrived at the Tree. Reaching the terminal waypoint SHALL enter the Arrived phase, and the arrival finale SHALL fire exactly once, setting the persistent "arrived" record — the durable hook a future Act 3 keys off, the way Act 2 keys off "launched".

#### Scenario: The Tree is the only sink

- **WHEN** the ship advances through the route graph
- **THEN** every maximal path terminates at the single Tree waypoint, and no other waypoint is a dead end

#### Scenario: Arrival fires the finale once

- **WHEN** the ship reaches the Tree and the finale has not yet been shown
- **THEN** the finale plays, the persistent "arrived" record is set, and a repeat request returns nothing

### Requirement: Pace And Provisions Hold

The system SHALL expose a single pace posture with four settings — Grueling (0.80× leg time, 1.30× provisions burn), Steady (default; 1.00× / 1.00×), Easy (1.20× / 0.90×), and Restful (slowest; 1.40× time, 0.80× burn, the thriftiest hold). The provisions hold SHALL start full at 100 (cap 100, or 150 with the Long Hold refit) and burn while traveling, composed from pace, crew station, and weather. When the hold runs dry the ship SHALL Drift in place, recovering after 36 hours and restoring provisions to 25. At every junction the cheapest outgoing road SHALL cost no more than 25 provisions, so running the hold dry always means drifting in place rather than becoming stuck.

#### Scenario: Pace trades speed for provisions

- **WHEN** the pace is set to Grueling versus Restful
- **THEN** Grueling covers a road in 0.80× the base time while burning 1.30× provisions, and Restful takes 1.40× the time while burning 0.80×

#### Scenario: Empty hold drifts and recovers

- **WHEN** provisions reach zero mid-road
- **THEN** the ship enters the Drifting phase and, after 36 hours, recovers with provisions restored to 25 and resumes

#### Scenario: Affordability floor keeps the ship unstuck

- **WHEN** the ship arrives at any junction
- **THEN** at least one outgoing road costs no more than the 25-provisions drift-recovery amount, so a fresh recovery can always afford to sail on

### Requirement: Crew, Stations, And Refits

The system SHALL support a crew of up to 7 aboard drawn from an authored roster of 8 souls (3 present at launch plus 5 recruited at waypoints), where a recruit ask SHALL block departure until answered. Souls SHALL be assignable to stations (Helm, Tender, Watch) that grant Voyage multipliers, SHALL advance authored arcs on a rest-day timer, and SHALL be farewelled to free a crew seat (remembered, not lost). Permanent loss SHALL occur only through authored scenes, never through any time-driven path. The first 3 distinct shipyards visited SHALL each offer one permanent A/B refit door, and choosing one option SHALL close the other forever.

#### Scenario: Recruit ask blocks departure

- **WHEN** the ship arrives where an unmet soul waits and an ask is pending
- **THEN** departure is blocked until the ask is accepted or declined

#### Scenario: Refit doors are one-way

- **WHEN** the player picks one option of a shipyard's A/B refit door
- **THEN** that refit is applied permanently and the alternate option is closed for the rest of the crossing

#### Scenario: Loss is authored-only

- **WHEN** the Voyage advances on its wall-clock timer with no authored loss scene triggered
- **THEN** no soul is ever marked permanently lost by a time-driven path

### Requirement: Maiden Voyage And Ferry Run Automation

The system SHALL distinguish the maiden voyage (crossing number 1) from ferry runs (crossing number greater than 1). On the maiden voyage, decisions — junctions with more than one road, recruit asks, refit doors, and the pier — SHALL hold the ship until the player acts, while a plain mid-crossing port with no decision SHALL wait 360 game-minutes and then auto-sail. On a ferry run the crossing itself SHALL remain hands-off: the ship SHALL auto-navigate junctions (taking the first road), skip refit doors, and launch herself from the pier once a wormhole jump has been committed, so the crossing completes autonomously in Drive-scaled time; on ferry runs the passenger headcount SHALL NOT deepen provisions burn. Unlike prior behavior, arrival at the Colony (every arrival from the maiden voyage's onward, i.e. before crossing 2 and every crossing after) SHALL NOT auto-transition into the next crossing — it SHALL enter the Dock phase (see the Dock Phase Entry And Exit requirement below), and the next crossing SHALL begin only once the player commits a wormhole jump from Dock.

#### Scenario: Maiden voyage holds for decisions

- **WHEN** the maiden voyage reaches a junction offering more than one road
- **THEN** the ship holds station until the player chooses a road

#### Scenario: Ferry run navigates itself once underway

- **WHEN** a ferry run (crossing number > 1) is underway and reaches a junction, a refit door, or the pier
- **THEN** the ship takes the first road / skips the refit / launches herself without waiting, and the crossing completes on its own

#### Scenario: Arrival no longer auto-starts the next crossing

- **WHEN** a crossing's arrival finale plays and the souls/Salvage are delivered to the Colony
- **THEN** the Voyage does not begin a new crossing automatically; the Colony enters the Dock phase and waits for the player to commit a wormhole jump

### Requirement: Colony Ferry Loop Persistence

The system SHALL persist a Colony above individual crossings, tracking souls delivered (a number that only rises) against souls remaining in a dying world of 100,000 initial souls. Each completed crossing SHALL deliver its carried passengers into the Colony, pay out Salvage, and let the dark take a per-day share of whoever still waits — so a slower crossing bleeds the waiting world longer. Salvage SHALL be spent across three yards whose levels persist between crossings: Drive (shortens every future crossing's sail time), Shipwright (grows the hold), and Ward (softens the dark's per-day toll, floored so a residual bite always remains). Voyage state SHALL be saved per character so a different character never inherits a crossing in progress. In addition, immediately after a crossing's delivery the Colony SHALL enter a Dock phase during which a new resource, Riftglass, accrues purely from elapsed real time docked, reaching full charge in `RIFTGLASS_BASE_HOURS_TO_FULL` (24.0) real hours at Drive level 0 and proportionally faster at higher Drive levels (rate multiplier = `1.0 / drive_time_mult()`), capped at 100% with no decay past full; the player MAY commit a one-way, no-undo wormhole jump at any time once docked, ending the Dock phase and beginning the next crossing via the existing return-crossing machinery. A jump committed at full Riftglass charge SHALL begin the next crossing exactly as crossings begin today (no penalty); a jump committed at a partial charge `c` (0.0–1.0) SHALL deduct `MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT (40.0) × (1.0 - c)` from the new crossing's starting provisions (floored at 0) and set its starting hull wear to `round(MAX_PARTIAL_CHARGE_HULL_WEAR (3) × (1.0 - c))`, with no randomness involved in either deduction.

#### Scenario: A crossing delivers souls and takes a toll

- **WHEN** a crossing completes carrying passengers
- **THEN** the carried souls are added to souls-delivered and removed from souls-remaining, Salvage is paid out, and the dark removes a per-day share of the still-waiting world scaled by the crossing's duration

#### Scenario: Yard levels persist and shape future crossings

- **WHEN** Salvage is spent to raise the Drive, Shipwright, or Ward yard
- **THEN** the new level persists into subsequent crossings, shortening sail time, widening the hold, or softening the daily toll respectively

#### Scenario: Voyage is keyed to the character

- **WHEN** a different character's save is loaded
- **THEN** it does not pick up another character's in-progress crossing

#### Scenario: Riftglass accrues from time spent docked

- **WHEN** the Colony has been docked at Drive level 0 for 24 real hours
- **THEN** its Riftglass charge, queried at that moment, is 100% (full), identical whether queried once after a long absence or repeatedly across many shorter intervals

#### Scenario: Drive level speeds Riftglass accrual

- **WHEN** two Colonies differ only in Drive level and have been docked for the same elapsed real time
- **THEN** the Colony with the higher Drive level shows a higher (or equal, at the 100% cap) Riftglass charge, scaled by `1.0 / drive_time_mult()`

#### Scenario: Full-charge jump has no penalty

- **WHEN** the player commits a wormhole jump with Riftglass charge at 100%
- **THEN** the next crossing begins with the same starting conditions a crossing begins with today — no provisions deducted, hull wear starts at 0

#### Scenario: Partial-charge jump applies a deterministic provisions and hull-wear penalty

- **WHEN** the player commits a wormhole jump with Riftglass charge at 0% (an immediate jump with no Dock time)
- **THEN** the next crossing begins with 40 provisions deducted from its starting hold and hull wear set to 3 (of the 6-point scale), and any charge between 0% and 100% deducts and sets these two values proportionally, with a lower charge never yielding a smaller penalty than a higher charge

### Requirement: Dock Phase Entry And Exit

The system SHALL enter the Dock phase the moment any crossing's arrival finale delivers its souls and Salvage to the Colony — unless that delivery emptied the world (the Last Crossing, which ends the era with no further Dock phase) — and SHALL remain in the Dock phase — during which the existing Reckoning (Drive/Shipwright/Ward purchases) and Record views remain reachable exactly as before — until the player commits a wormhole jump. Only one crossing MAY be in progress and only one Dock phase MAY be active at a time; committing a jump SHALL end the Dock phase and begin the next crossing in the same action.

#### Scenario: Arrival enters Dock

- **WHEN** a crossing's arrival finale finishes delivering to the Colony and souls remain in the dying world
- **THEN** the Colony's Dock phase becomes active and the player is shown the Dock view

#### Scenario: Yard purchases remain available while docked

- **WHEN** the Colony is in the Dock phase
- **THEN** the player can still spend Salvage on the Drive, Shipwright, and Ward yards exactly as when not docked

#### Scenario: Committing a jump ends the Dock phase

- **WHEN** the player commits a wormhole jump while docked
- **THEN** the Dock phase ends and the next crossing begins immediately in the same action, with no way to return to the ended Dock phase

#### Scenario: The Last Crossing never docks

- **WHEN** a crossing's delivery empties the world
- **THEN** the Dock phase is not entered and no jump is offered

### Requirement: The Last Crossing Ends The Era

The system SHALL end the ferry era when the dying world's remaining souls reach zero. The arrival that empties the world SHALL be the Last Crossing: it delivers its souls and Salvage as any crossing does, and the persistent `last_crossing_complete` record SHALL be set on the character — the durable gate a future Act 3 keys off, alongside the `vessel_arrived` record set by the first arrival. An authored, multi-beat era-end epilogue SHALL then play exactly once — its account conditioned on the era's own state (souls delivered, souls the dark took, crossings sailed, days at sea, districts standing) and closing on the door in the root-wall standing ajar — recorded by a persistent flag so an interrupted or reloaded era end still receives it, and never twice. After the Last Crossing the Colony SHALL NOT enter the Dock phase and no further wormhole jump SHALL be offered; the Dock view SHALL show an authored quiet-harbor resting state with no charge or jump affordances; the arrived-harbor views (Manifest, Keepsake, Record) SHALL remain reachable, and the Record view SHALL carry a permanent era summary (the same settled account the epilogue reads).

#### Scenario: Emptying the world completes the era

- **WHEN** a crossing's delivery reduces souls-remaining to zero
- **THEN** `last_crossing_complete` is set and persists across save/load, and the era is over

#### Scenario: The epilogue plays exactly once, reload included

- **WHEN** the era is over and the epilogue has not yet been shown — whether in the same session as the Last Crossing or on a later load
- **THEN** the multi-beat epilogue plays once, its account matching the colony's own numbers, and a repeat request returns nothing

#### Scenario: No dock after the Last Crossing

- **WHEN** the era is over and the ship stands at the Tree
- **THEN** no Dock phase is active, a jump request does nothing, the Dock view shows the quiet harbor with no charge bar or jump preview, and the Manifest, Keepsake, and Record views remain reachable

#### Scenario: The Record keeps the era's account

- **WHEN** the era is over and the player opens the Record view
- **THEN** a permanent era summary is shown (crossings, souls delivered, souls taken by the dark, days at sea, districts standing)

#### Scenario: The gate defaults closed

- **WHEN** a character save predating the ferry era is loaded
- **THEN** `last_crossing_complete` deserializes to false

### Requirement: Ferry-Era Balance Envelope

The ferry era's pacing SHALL stay inside coarse, CI-asserted bands (deterministic simulation, headroom deliberately wide so only structural regressions trip them). Under a balanced yard spend with full-charge jumps, an era SHALL complete in 15–30 crossings spanning 2.5–4.5 real months and deliver at least 84% of the world's 100,000 souls. The naive extremes SHALL remain traps: a Drive-only spend SHALL save no more than 74%. A Ward-leaning spend SHALL save at least 90%, trading a longer era for it. Jumping at full Riftglass charge SHALL never save fewer souls than always jumping at 0% charge.

#### Scenario: The balanced line holds the campaign shape

- **WHEN** a full era is simulated with the balanced spend policy and full-charge jumps
- **THEN** it completes in 15–30 crossings within 2.5–4.5 real months with ≥84% of souls delivered

#### Scenario: Skill is rewarded, not marginal

- **WHEN** full eras are simulated with a Ward-leaning spend and with a Drive-only spend
- **THEN** the Ward-leaning line delivers ≥90% of souls and the Drive-only line delivers ≤74%

#### Scenario: Patience at the Dock pays

- **WHEN** full eras are simulated jumping always at 100% charge and always at 0% charge
- **THEN** the full-charge era delivers at least as many souls as the 0%-charge era

### Requirement: Pilgrim Ships Have Authored Fates

The system SHALL show five authored pilgrim ships sailing cyclic scripted routes — their fates authored, not simulated; the player's choices SHALL NOT save or doom them. Exactly one ship, the Grief of Alden, SHALL go dark after her authored day (day 40) and stop appearing; the other four, including the Sister Verity (a face staged for Act 3), SHALL sail on indefinitely. Each ship MAY be hailed at most once per crossing's acquaintance (hailing is once per ship).

#### Scenario: One authored darkening

- **WHEN** the voyage passes the Grief of Alden's authored final day
- **THEN** she no longer appears on any road, and the other four pilgrim ships still sail their scripts

#### Scenario: Fates are weather, not consequence

- **WHEN** the player makes any in-voyage choice (routes, pace, stations, refits)
- **THEN** no pilgrim ship's fate changes

### Requirement: Act Milestones Are Recorded As Save Events

The system SHALL record the Vessel's three act milestones as Time Vault save events: the launch burn (`VesselLaunched`, committed via the launch action's save-with-event result), the first arrival at the Tree (`VesselArrived`), and the Last Crossing (`LastCrossing`) — the latter two committed directly from the voyage loop when their state transitions fire.

#### Scenario: The burn is a vault moment

- **WHEN** the player confirms the launch and the burn succeeds
- **THEN** the save is committed to history with the `VesselLaunched` event

#### Scenario: Arrival and era end are vault moments

- **WHEN** the first arrival sets `vessel_arrived`, or the Last Crossing sets `last_crossing_complete`
- **THEN** the corresponding save event is committed to history exactly once each

### Requirement: Chapter Gateways Close Their Chapters

Each chapter's single gateway waypoint SHALL append one authored chapter-close beat to its arrival scene, so the act's four chapter transitions are felt ceremonies rather than ordinary ports. Every maximal route SHALL see exactly the gateway beats of the chapters it crosses.

#### Scenario: A gateway arrival plays its chapter-close beat

- **WHEN** the ship's arrival scene plays at a chapter gateway waypoint
- **THEN** the scene ends with that chapter's authored close beat

### Requirement: Act 2 State Participates In Time Vault Timelines

The Act 2 account files (`voyage.json`, `colony.json`) SHALL be included in Time Vault snapshots and SHALL rewind with the timeline on restore — the vault rewinds the hero and the era together, like every other account file in the quest directory. Restoring to a commit from before the launch SHALL remove the in-progress crossing files (a later launch begins a fresh crossing); restoring back to a post-launch commit SHALL return them intact and loadable. Outside the vault, the keyed-by-character load behavior is unchanged: a different character never inherits a crossing.

#### Scenario: A pre-launch restore rewinds the era

- **WHEN** the timeline is restored to a commit that predates the launch
- **THEN** `voyage.json` and `colony.json` from the later timeline are no longer present, and a subsequent launch begins a fresh crossing

#### Scenario: Restoring forward returns the crossing

- **WHEN** the timeline is restored to a post-launch commit (recoverable via its commit id)
- **THEN** the voyage and colony files are restored intact and load through the real load paths

