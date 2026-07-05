# Game Loop & Idle Progression Specification

## Purpose

The game loop is the heartbeat that advances every gameplay system on a fixed cadence while the hero battles automatically, so an idle player keeps making progress without input. It defines how often the world updates, what a single update does, how experience and levels accrue from combat, how the game persists itself and checks for updates in the background, and how time spent away from the game is credited. It also fixes the determinism contract that lets the same inputs and random seed reproduce the same outcome, which the headless simulators and snapshot tests rely on.

## Requirements

### Requirement: Fixed-Cadence Tick Loop

The system SHALL advance gameplay in discrete ticks at a fixed interval of 100 milliseconds, giving a rate of 10 ticks per second, and each tick SHALL treat elapsed game time as exactly 0.1 seconds regardless of real wall-clock drift. A new tick SHALL only be processed once at least 100 milliseconds have passed since the previous tick.

#### Scenario: Steady progression at ten ticks per second

- **WHEN** the game is running normally and 100 milliseconds have elapsed since the last tick
- **THEN** exactly one tick is processed, advancing all active systems by 0.1 seconds of game time
- **AND** roughly 10 ticks are processed over one real second of steady play

#### Scenario: No tick before the interval elapses

- **WHEN** fewer than 100 milliseconds have passed since the previous tick
- **THEN** no new tick is processed and game state is left unchanged until the interval is reached

### Requirement: Single-Tick System Advancement

The system SHALL, on each tick, advance the full set of active systems in a fixed order: recompute active bonuses, progress challenge minigame AI and roll for new challenge discovery, synchronize the hero's derived stats and maximum HP, advance any active dungeon exploration, advance resource-production chains, run either a fishing session or automatic combat (never both in the same tick), spawn a new enemy when none is present and the hero is not regenerating, account play time, and roll for the discovery of new endgame systems. Fishing and combat SHALL be mutually exclusive within a single tick.

#### Scenario: Idle combat tick with no enemy present

- **WHEN** a tick runs while no fishing session is active and the hero has no current enemy and is not regenerating
- **THEN** automatic combat is advanced and a new enemy is spawned for the current zone or dungeon so battling can continue

#### Scenario: Bonuses stay current

- **WHEN** a tick runs
- **THEN** the hero's derived stats and maximum HP are recomputed from current equipment, attributes, and active bonuses before combat is resolved

### Requirement: Play-Time Accounting

The system SHALL count ticks and credit one second of accumulated play time for every 10 ticks processed, resetting the running tick count to zero each time a second is credited.

#### Scenario: One second credited every ten ticks

- **WHEN** 10 ticks have been processed since the last second was credited
- **THEN** total play time increases by exactly 1 second and the tick counter resets to zero

### Requirement: Experience Is Earned Only From Kills

The system SHALL grant experience only when the hero defeats an enemy, and SHALL NOT grant any passive experience on ticks where no enemy dies. Each kill SHALL award experience equal to a uniformly random value between 200 and 400 (inclusive) multiplied by the hero's passive per-tick experience rate, then increased by the Training Yard bonus when present. The passive per-tick rate SHALL be 1.0 at base and SHALL scale up with prestige rank, Wisdom, and Charisma, so a base hero earns 200 to 400 experience per kill while a stronger hero earns proportionally more.

#### Scenario: Defeating an enemy grants experience

- **WHEN** an enemy is defeated during a combat tick
- **THEN** the hero gains between 200 and 400 units of experience times the passive rate (plus any Training Yard bonus), and the kill is recorded

#### Scenario: Idle tick without a kill grants nothing

- **WHEN** a tick resolves combat but no enemy dies
- **THEN** the hero's experience total is unchanged

### Requirement: Leveling And Attribute Distribution

The system SHALL level the hero up whenever accumulated experience reaches the requirement for the next level, computed as 100 times the current level raised to the power 1.5. On each level gained the system SHALL award 3 attribute points distributed at random among the six attributes that are below their cap, and SHALL continue leveling within the same processing pass while enough experience remains for further level-ups. Attributes SHALL never be raised above the current cap of 20 plus 5 per prestige rank.

#### Scenario: Crossing the experience threshold levels up

- **WHEN** the hero's accumulated experience reaches or exceeds 100 × level^1.5
- **THEN** the level increases by one, the experience cost is subtracted, and 3 attribute points are distributed among non-capped attributes

#### Scenario: A single large gain can grant multiple levels

- **WHEN** a single experience award exceeds the combined cost of several levels
- **THEN** the hero levels up repeatedly in the same pass, gaining 3 attribute points per level

### Requirement: HP Regeneration After A Kill

The system SHALL, after the hero defeats an enemy, hold a regeneration window of 2.5 seconds before the hero's HP is restored, and SHALL emit a regeneration-complete signal when it finishes. The window MAY be shortened by regeneration-delay-reduction bonuses.

#### Scenario: Recovery follows a victory

- **WHEN** the hero defeats an enemy with no delay-reduction bonuses
- **THEN** the hero's HP is restored 2.5 seconds later and a regeneration-complete outcome is reported

### Requirement: Fishing And Combat Are Mutually Exclusive

The system SHALL, when a fishing session is active during a tick, advance the fishing session and skip automatic combat for that tick, still accounting play time and crediting any passive rewards produced earlier in the tick.

#### Scenario: Active fishing suspends combat

- **WHEN** a tick runs while a fishing session is active
- **THEN** the fishing session is advanced and combat is not processed for that tick
- **AND** play time is still accounted for the tick

### Requirement: Periodic Autosave

The system SHALL persist all game and account state to disk on a recurring interval of 30 seconds. On each autosave the system SHALL synchronize the recorded last-save timestamp to the current time so that active play is not later mistaken for offline time, and SHALL avoid starting a new background save while a previous one is still running. Autosave file writes SHALL be suppressed in debug mode.

#### Scenario: Autosave fires every thirty seconds

- **WHEN** 30 seconds have elapsed since the last autosave and the game is not in debug mode
- **THEN** state is written to disk in the background and the recorded last-save timestamp is updated to the current time

#### Scenario: Overlapping saves are avoided

- **WHEN** an autosave interval elapses but the previous background save has not finished
- **THEN** no new background save is started until the prior one completes

### Requirement: Periodic Update Check

The system SHALL check for a newer release in the background on a jittered recurring interval centered on 15 minutes with a plus-or-minus 5 minute jitter, producing an effective interval uniformly distributed between 10 and 20 minutes. The system SHALL re-randomize the interval after each check, and SHALL NOT start a new check while one is already in flight or after an update has already been found.

#### Scenario: Jittered background check

- **WHEN** the current jittered interval (between 10 and 20 minutes) has elapsed, no check is currently running, and no update has yet been found
- **THEN** a new background update check is started and the next interval is re-randomized within the 10 to 20 minute range

### Requirement: Offline Progression

The system SHALL, when the game is reopened, credit progress for the time elapsed since the recorded last-save timestamp. Elapsed time SHALL be capped at 7 days (604800 seconds). The system SHALL simulate kills at an estimated rate of one kill every 5 seconds reduced to 25 percent of the online rate, award the average per-kill experience (equivalent to 300 ticks) scaled by the hero's passive rate for each simulated kill, and multiply the result by any offline-experience bonus from account systems. It SHALL then apply the resulting experience with normal leveling and reset the last-save timestamp to the current time. If the elapsed time is zero or negative, no experience SHALL be granted.

#### Scenario: Away for an hour

- **WHEN** the game is reopened after being closed for 1 hour
- **THEN** offline experience is credited as (3600 ÷ 5) × 0.25 simulated kills times the average per-kill experience times the passive rate and any offline bonus, and the hero levels up accordingly

#### Scenario: Long absence is capped at seven days

- **WHEN** the game is reopened after being closed for 14 days
- **THEN** offline progress is credited as though only 7 days had elapsed

#### Scenario: Reopening immediately grants nothing

- **WHEN** the elapsed time since the last save is zero or negative
- **THEN** no offline experience is granted and no levels are gained

### Requirement: Deterministic Seeded RNG Contract

The system SHALL draw every random outcome within a tick from a single caller-supplied random number generator, so that given the same seed and the same starting state and inputs, a tick produces identical events and identical resulting state. Production play SHALL use a system-seeded generator, while the headless simulators and snapshot tests SHALL supply a fixed seed to obtain reproducible runs.

#### Scenario: Same seed and state reproduce the same tick

- **WHEN** two ticks are run from identical starting state with generators initialized to the same seed
- **THEN** both ticks produce the same events and leave state in the same condition

#### Scenario: Fixed seed enables reproducible simulation

- **WHEN** a headless simulator or test runs the loop with a fixed seed
- **THEN** the run is reproducible and yields the same outcome on every execution
