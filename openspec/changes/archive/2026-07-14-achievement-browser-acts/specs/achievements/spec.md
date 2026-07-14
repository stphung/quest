# Delta: achievements — Act-Sectioned Browser (Option A)

## MODIFIED Requirements

### Requirement: Achievement Categories, Count, and Scoring

The system SHALL define 247 achievements, each assigned to one of twelve browsing categories grouped under two acts — Act I · The Ascent (Combat, Level, Prestige, Progression, Challenges, Exploration, The Deep, Loom, Stats) and Act II · The Crossing (The Voyage, The Ferry, The Era) — and one point value drawn from a tiered scale of 5, 10, 25, 50, 100, 250, and 500 points. The achievement browser SHALL present an act selector above the category tabs, showing per-act unlocked/total and points summaries, with tab cycling scoped to the selected act and a single key toggling acts; the Act II label SHALL render dimmed while the Act 2 kill-switch is off (its rows remain visible and locked, per the visible-but-unearnable ruling). The achievement score SHALL be computed as the sum of the point values of all currently unlocked achievements, and the system SHALL also expose the unlocked/total count overall and per category and an overall unlock percentage. The score SHALL rise only as new achievements unlock and SHALL equal zero when none are unlocked.

#### Scenario: Score is the sum of unlocked points

- **WHEN** two achievements worth 5 and 25 points are unlocked and no others
- **THEN** the achievement score is 30

#### Scenario: Category counts reflect unlocks

- **WHEN** two Combat achievements are unlocked
- **THEN** the Combat category reports two unlocked out of its total, and categories with no unlocks report zero

#### Scenario: Acts partition the categories

- **WHEN** the browser's act selector is toggled
- **THEN** only the selected act's category tabs are shown, every category belongs to exactly one act, and cycling wraps within the act

### Requirement: Vessel Act Milestones

The system SHALL track seven account-level Vessel achievements in Act II's subsections — The Voyage (The Burn: launch the Vessel; The Roots of Light: first arrival at the Tree), The Ferry (Ferryman I/II/III: 1,000 / 10,000 / 50,000 lifetime souls delivered, driven by a persisted `total_souls_delivered` aggregate), and The Era (The Last Crossing: complete the ferry era; The Covenant Kept: complete the era with no crew soul ever lost, driven by a persisted lifetime lost-souls counter that only authored loss scenes can increment). These achievements SHALL remain visible in the browser and its counts even while Act 2 is dark-shipped — the kill-switch gates entry into the act, not the existence of its milestones (a deliberate teaser) — but SHALL be unearnable while dark, since every unlock path lives behind `act2_enabled()`-gated code.

#### Scenario: The launch unlocks The Burn

- **WHEN** the player performs the all-or-nothing launch burn
- **THEN** The Burn unlocks once, account-wide

#### Scenario: Souls tiers follow the lifetime counter

- **WHEN** a crossing's delivery raises lifetime souls delivered across 1,000, 10,000, or 50,000
- **THEN** the corresponding Ferryman tier unlocks

#### Scenario: The covenant is judged at era end

- **WHEN** the Last Crossing completes
- **THEN** The Last Crossing unlocks, and The Covenant Kept unlocks only if no crew soul was ever lost across the era

#### Scenario: Visible but unearnable while dark

- **WHEN** the achievement browser lists Act II's subsections with the Act 2 kill-switch off
- **THEN** the Vessel achievements appear (locked), and no gameplay path can unlock them
