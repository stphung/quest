# Delta: vessel-act2 — Era Epilogue

## MODIFIED Requirements

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
