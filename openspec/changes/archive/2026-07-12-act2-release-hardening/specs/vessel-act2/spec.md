# Delta: vessel-act2 — Act 2 Release Hardening

## ADDED Requirements

### Requirement: The Last Crossing Ends The Era

The system SHALL end the ferry era when the dying world's remaining souls reach zero. The arrival that empties the world SHALL be the Last Crossing: it delivers its souls and Salvage as any crossing does, an authored era-end scene SHALL play once, and the persistent `last_crossing_complete` record SHALL be set on the character — the durable gate a future Act 3 keys off, alongside the `vessel_arrived` record set by the first arrival. After the Last Crossing the Colony SHALL NOT enter the Dock phase and no further wormhole jump SHALL be offered; the arrived-harbor views (Manifest, Keepsake, Record) SHALL remain reachable.

#### Scenario: Emptying the world completes the era

- **WHEN** a crossing's delivery reduces souls-remaining to zero
- **THEN** the era-end scene plays, `last_crossing_complete` is set and persists across save/load, and the era is over

#### Scenario: No dock after the Last Crossing

- **WHEN** the era is over and the ship stands at the Tree
- **THEN** no Dock phase is active, a jump request does nothing, and the Manifest, Keepsake, and Record views remain reachable

#### Scenario: The gate defaults closed

- **WHEN** a character save predating the ferry era is loaded
- **THEN** `last_crossing_complete` deserializes to false

### Requirement: Ferry-Era Balance Envelope

The ferry era's pacing SHALL stay inside coarse, CI-asserted bands (deterministic simulation, headroom deliberately wide so only structural regressions trip them). Under a balanced yard spend with full-charge jumps, an era SHALL complete in 20–40 crossings spanning 3–6 real months and deliver at least 82% of the world's 100,000 souls. The naive extremes SHALL remain traps: a Drive-only spend SHALL save no more than 74%. A Ward-leaning spend SHALL save at least 90%, trading a longer era for it. Jumping at full Riftglass charge SHALL never save fewer souls than always jumping at 0% charge.

#### Scenario: The balanced line holds the campaign shape

- **WHEN** a full era is simulated with the balanced spend policy and full-charge jumps
- **THEN** it completes in 20–40 crossings within 3–6 real months with ≥82% of souls delivered

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

## MODIFIED Requirements

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
