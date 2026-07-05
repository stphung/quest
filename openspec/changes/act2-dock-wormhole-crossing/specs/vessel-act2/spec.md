## MODIFIED Requirements

### Requirement: Maiden Voyage And Ferry Run Automation

The system SHALL distinguish the maiden voyage (crossing number 1) from ferry runs (crossing number greater than 1). On the maiden voyage, decisions — junctions with more than one road, recruit asks, refit doors, and the pier — SHALL hold the ship until the player acts, while a plain mid-crossing port with no decision SHALL wait 360 game-minutes and then auto-sail. On a ferry run the crossing itself SHALL remain hands-off: the ship SHALL auto-navigate junctions (taking the first road), skip refit doors, and launch herself from the pier once a wormhole jump has been committed, so the crossing completes autonomously in Drive-scaled time; on ferry runs the passenger headcount SHALL NOT deepen provisions burn. Unlike prior behavior, arrival at the Colony (every arrival from the maiden voyage's onward, i.e. before crossing 2 and every crossing after) SHALL NOT auto-transition into the next crossing — it SHALL enter the Dock phase (see the new Dock/Riftglass/Wormhole Jump requirements below), and the next crossing SHALL begin only once the player commits a wormhole jump from Dock.

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

The system SHALL persist a Colony above individual crossings, tracking souls delivered (a number that only rises) against souls remaining in a dying world of 100,000 initial souls. Each completed crossing SHALL deliver its carried passengers into the Colony, pay out Salvage, and let the dark take a per-day share of whoever still waits — so a slower crossing bleeds the waiting world longer. Salvage SHALL be spent across three yards whose levels persist between crossings: Drive (shortens every future crossing's sail time), Shipwright (grows the hold), and Ward (softens the dark's per-day toll, floored so a residual bite always remains). Voyage state SHALL be saved per character so a different character never inherits a crossing in progress. In addition, immediately after a crossing's delivery the Colony SHALL enter a Dock phase during which a new resource, Riftglass, accrues purely from elapsed real time docked at a rate scaled by the Drive yard's level; the player MAY commit a one-way, no-undo wormhole jump at any time once docked, ending the Dock phase and beginning the next crossing via the existing return-crossing machinery. A jump committed at full Riftglass charge SHALL begin the next crossing exactly as crossings begin today (no penalty); a jump committed at a partial charge SHALL apply a deterministic penalty to the new crossing that increases as the charge decreases, with no randomness involved in determining the penalty's presence or magnitude.

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

- **WHEN** the Colony has been in the Dock phase for a span of real time
- **THEN** its Riftglass charge, queried at any later moment, is a pure function of that elapsed real time and the Drive yard's level, identical whether queried once after a long absence or repeatedly across many shorter intervals

#### Scenario: Drive level speeds Riftglass accrual

- **WHEN** two Colonies differ only in Drive level and have been docked for the same elapsed real time
- **THEN** the Colony with the higher Drive level shows a higher (or equal, at the accrual cap) Riftglass charge

#### Scenario: Full-charge jump has no penalty

- **WHEN** the player commits a wormhole jump with Riftglass charge at its maximum
- **THEN** the next crossing begins with the same starting conditions a crossing begins with today — no deficit applied

#### Scenario: Partial-charge jump applies a deterministic penalty

- **WHEN** the player commits a wormhole jump with Riftglass charge below its maximum
- **THEN** the next crossing begins with a penalty whose presence and magnitude are a deterministic function of the charge level only — never randomized — and a lower charge never yields a smaller penalty than a higher charge

## ADDED Requirements

### Requirement: Dock Phase Entry And Exit

The system SHALL enter the Dock phase the moment any crossing's arrival finale delivers its souls and Salvage to the Colony, and SHALL remain in the Dock phase — during which the existing Reckoning (Drive/Shipwright/Ward purchases) and Record views remain reachable exactly as before — until the player commits a wormhole jump. Only one crossing MAY be in progress and only one Dock phase MAY be active at a time; committing a jump SHALL end the Dock phase and begin the next crossing in the same action.

#### Scenario: Arrival enters Dock

- **WHEN** a crossing's arrival finale finishes delivering to the Colony
- **THEN** the Colony's Dock phase becomes active and the player is shown the Dock view

#### Scenario: Yard purchases remain available while docked

- **WHEN** the Colony is in the Dock phase
- **THEN** the player can still spend Salvage on the Drive, Shipwright, and Ward yards exactly as when not docked

#### Scenario: Committing a jump ends the Dock phase

- **WHEN** the player commits a wormhole jump while docked
- **THEN** the Dock phase ends and the next crossing begins immediately in the same action, with no way to return to the ended Dock phase
