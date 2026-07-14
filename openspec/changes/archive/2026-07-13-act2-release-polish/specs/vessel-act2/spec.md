# Delta: vessel-act2 — Release Polish

## ADDED Requirements

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
