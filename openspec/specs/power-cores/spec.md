# Power Cores — Passive PR Generation Specification

## Purpose

Define the Power Cores system: six passive generators that award Prestige Rank (PR) over real wall-clock time once unlocked. Each core unlocks at a specific Deep layer milestone (the same layers that gate Fracture zones), produces PR at a fixed daily rate by completing discrete fill cycles, and accrues while the game is closed as well as while it runs. This capability owns the per-core rates, the unlock gates, the fill/grant timing, and the offline catch-up rules; The Deep is referenced only as the unlock source and Prestige Rank only as the output.

## Requirements

### Requirement: Six Power Cores With Fixed Generation Rates

The system SHALL define exactly six Power Cores, each with a fixed display name and a fixed Prestige-Rank-per-day (PR/day) generation rate, ordered by ascending rate. The rates SHALL be exactly:

| Power Core | Name | PR/day |
|------------|--------------|--------|
| I | Red Fault | 2 |
| II | Mirror Scar | 3 |
| III | Black Mouth | 5 |
| IV | Hollow Throne | 8 |
| V | Wailing Reach | 12 |
| VI | Origin Wound | 18 |

Each successive core's PR/day rate SHALL be greater than or equal to the preceding core's rate.

#### Scenario: Core roster and rates

- **WHEN** the set of Power Cores is enumerated
- **THEN** there are exactly six cores named Red Fault, Mirror Scar, Black Mouth, Hollow Throne, Wailing Reach, and Origin Wound
- **AND** their PR/day rates are 2, 3, 5, 8, 12, and 18 respectively
- **AND** the rates are non-decreasing from core I through core VI

### Requirement: Deep-Layer Unlock Gating

The system SHALL unlock each Power Core through a Deep layer milestone, and only unlocked cores SHALL generate Prestige Rank. The mapping of Deep layer to core SHALL be exactly: layer 3 → core I, layer 7 → core II, layer 12 → core III, layer 18 → core IV, layer 25 → core V, layer 30 → core VI. A core whose milestone has not been reached SHALL never grant PR, regardless of elapsed time.

#### Scenario: Core activates at its Deep layer

- **WHEN** the player reaches the Deep layer that unlocks a given Power Core
- **THEN** that Power Core becomes active and begins generating Prestige Rank passively

#### Scenario: Locked cores never generate

- **WHEN** a Power Core's unlock milestone has not been reached
- **THEN** no Prestige Rank is granted for that core even if a full fill duration or more has elapsed

### Requirement: Combined Maximum Passive Output

The system SHALL cap combined passive Power Core output at 48 PR/day, which is the sum of all six per-core rates (2 + 3 + 5 + 8 + 12 + 18) reached only when all six cores are unlocked. Fewer unlocked cores SHALL produce the sum of only their individual rates.

#### Scenario: All six cores active

- **WHEN** all six Power Cores are unlocked and generating
- **THEN** the total passive output is exactly 48 PR/day

#### Scenario: Partial unlock output

- **WHEN** only cores I and II are unlocked (2 PR/day and 3 PR/day)
- **THEN** the combined passive output is 5 PR/day

### Requirement: Fill-Cycle Accrual Model

The system SHALL grant Prestige Rank in whole units of one PR per completed fill cycle, where a core's fill duration in seconds equals 86400 divided by its PR/day rate. PR SHALL NOT accrue continuously or fractionally; a core grants nothing until at least one full fill duration has elapsed since its last grant. The per-core fill durations SHALL be:

| Power Core | PR/day | Fill duration |
|------------|--------|-----------------------|
| I | 2 | 43200 s (12 h) |
| II | 3 | 28800 s (8 h) |
| III | 5 | 17280 s (4.8 h) |
| IV | 8 | 10800 s (3 h) |
| V | 12 | 7200 s (2 h) |
| VI | 18 | 4800 s (~1.33 h) |

#### Scenario: Grant after a full fill cycle

- **WHEN** the wall-clock time elapsed since a core's last grant reaches or exceeds its fill duration
- **THEN** exactly 1 Prestige Rank is granted per completed cycle

#### Scenario: No grant before a full cycle

- **WHEN** the elapsed time since the last grant is less than the core's fill duration
- **THEN** no Prestige Rank is granted and the core's grant timestamp is unchanged

### Requirement: First-Cycle Initialization On Unlock

The system SHALL treat a newly unlocked core (one with no recorded grant timestamp) by initializing its timer to the current time without granting Prestige Rank, so that the core's first PR is earned one full fill duration after activation rather than immediately. A missing or zero grant timestamp SHALL be interpreted as "never granted."

#### Scenario: First processing after unlock

- **WHEN** a core is processed for the first time after unlocking and has no prior grant timestamp
- **THEN** its grant timestamp is set to the current time
- **AND** no Prestige Rank is granted on that first processing
- **AND** the change is persisted so the first cycle counts from the unlock moment

### Requirement: Multi-Cycle Batching And Remainder Carryover

The system SHALL award one Prestige Rank for each fill cycle that completed since a core's last grant, so that when several cycles elapse between processings they are all counted. After granting, the core's grant timestamp SHALL advance by exactly the duration of the completed cycles (fill duration multiplied by the number of completed cycles), preserving any leftover partial-cycle time toward the next grant.

#### Scenario: Multiple elapsed cycles counted individually

- **WHEN** three full fill durations have elapsed for an active core since its last grant
- **THEN** 3 Prestige Rank is granted in that processing
- **AND** the grant timestamp advances by three fill durations, carrying the remaining partial time forward

### Requirement: Offline Passive Accrual

The system SHALL accrue Power Core Prestige Rank while the game is closed and apply the accumulated whole-cycle grants when the character is next loaded, using the same fill-cycle math as live ticks. The offline catch-up SHALL return or report the total Prestige Rank granted and SHALL NOT emit per-grant in-game events (unlike live processing, which emits one grant event per completed cycle).

#### Scenario: Grants applied on load after downtime

- **WHEN** a character with an active core is loaded after enough real time has passed for two fill cycles
- **THEN** 2 Prestige Rank is added on load
- **AND** the total granted is reported for an offline summary without emitting per-cycle events

#### Scenario: Offline processing initializes an uninitialized core

- **WHEN** offline catch-up encounters an active core that has no prior grant timestamp
- **THEN** it sets the timestamp to the current time and grants no Prestige Rank for that core

### Requirement: Passive PR Applied To Prestige Rank

The system SHALL add each Power Core grant to the character's Prestige Rank using saturating addition (so the count never overflows), and after any grant occurs it SHALL recalculate prestige-derived bonuses and mark derived stats for refresh so subsequent combat reflects the higher Prestige Rank.

#### Scenario: Prestige Rank increases and bonuses refresh

- **WHEN** one or more Power Cores grant Prestige Rank in a processing pass
- **THEN** the character's Prestige Rank increases by the total granted amount
- **AND** prestige bonuses are recalculated and derived stats are flagged for refresh
