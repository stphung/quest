# Delta: achievements — Vessel / Act 2 Achievements

## ADDED Requirements

### Requirement: Vessel Act Milestones

The system SHALL track seven account-level Vessel achievements in the Progression category: The Burn (launch the Vessel), The Roots of Light (first arrival at the Tree), Ferryman I/II/III (1,000 / 10,000 / 50,000 lifetime souls delivered, driven by a persisted `total_souls_delivered` aggregate), The Last Crossing (complete the ferry era), and The Covenant Kept (complete the era with no crew soul ever lost, driven by a persisted lifetime lost-souls counter that only authored loss scenes can increment). While Act 2 is dark-shipped (`act2_enabled()` false), all seven SHALL be invisible in every player-facing surface — category lists, totals, per-category counts, and the maximum score — so the kill-switch's "fully invisible" promise holds; they appear the moment the act is enabled.

#### Scenario: The launch unlocks The Burn

- **WHEN** the player performs the all-or-nothing launch burn
- **THEN** The Burn unlocks once, account-wide

#### Scenario: Souls tiers follow the lifetime counter

- **WHEN** a crossing's delivery raises lifetime souls delivered across 1,000, 10,000, or 50,000
- **THEN** the corresponding Ferryman tier unlocks

#### Scenario: The covenant is judged at era end

- **WHEN** the Last Crossing completes
- **THEN** The Last Crossing unlocks, and The Covenant Kept unlocks only if no crew soul was ever lost across the era

#### Scenario: Hidden while the act is dark

- **WHEN** the achievement browser lists the Progression category with the Act 2 kill-switch off
- **THEN** no Vessel achievement appears, and totals/percentages/max score exclude all seven
