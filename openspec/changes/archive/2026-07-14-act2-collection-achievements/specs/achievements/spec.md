# Achievements — Delta

## MODIFIED Requirements

### Requirement: Achievement Categories, Count, and Scoring

The system SHALL define 254 achievements, each assigned to one of twelve browsing categories grouped under two acts — Act I · The Ascent (Combat, Level, Prestige, Progression, Challenges, Exploration, The Deep, Loom, Stats) and Act II · The Crossing (The Voyage, The Ferry, The Era) — and one point value drawn from a tiered scale of 5, 10, 25, 50, 100, 250, and 500 points. The achievement browser SHALL present an act selector above the category tabs, showing per-act unlocked/total and points summaries, with tab cycling scoped to the selected act and a single key toggling acts; the Act II label SHALL render dimmed while the Act 2 kill-switch is off (its rows remain visible and locked, per the visible-but-unearnable ruling). The achievement score SHALL be computed as the sum of the point values of all currently unlocked achievements, and the system SHALL also expose the unlocked/total count overall and per category and an overall unlock percentage. The score SHALL rise only as new achievements unlock and SHALL equal zero when none are unlocked.

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

The system SHALL track fourteen account-level Vessel achievements in Act II's subsections:

- **The Voyage** — The Burn (launch the Vessel, 50 pts); The Roots of Light (first arrival at the Tree); Every Star a Harbor (dock at all 38 charted waypoints, cumulative across crossings via a persisted account-level waypoint union, 100 pts, grants the **Wayfarer** title); Company on the Road (hail all 5 pilgrim ships, cumulative across crossings via a persisted account-level hail union, 50 pts); Ear to the Water (know all 8 rumors at once within a single crossing, 50 pts); Three Doors Opened (take a refit at all three shipyard doors in one crossing — never spending a door on hull mending, 25 pts).
- **The Ferry** — Ferryman I/II/III (1,000 / 10,000 / 50,000 lifetime souls delivered, driven by the persisted `total_souls_delivered` aggregate); The Full Table (make landfall with all seven crew berths filled, 50 pts); Heavy Lading (deliver 2,500 or more souls in a single crossing, 25 pts).
- **The Era** — The Last Crossing (complete the ferry era); The Covenant Kept (complete the era with no crew soul ever lost, driven by a persisted lifetime lost-souls counter that only authored loss scenes can increment); The Swift Passage (complete a crossing in 8 sea-days or fewer — inclusive, the drive-heavy floor is exactly 8, 25 pts).

Collection completion targets SHALL derive from the authored content constants (waypoint, pilgrim, rumor, refit-door, and crew-berth counts) rather than duplicated literals, so authored growth widens the target instead of completing it early. The two tuned thresholds (2,500 souls; 8 sea-days) SHALL be named constants whose reachability inside the ferry-era balance envelope is pinned by the era-harness tests. These achievements SHALL remain visible in the browser and its counts even while Act 2 is dark-shipped — the kill-switch gates entry into the act, not the existence of its milestones (a deliberate teaser) — but SHALL be unearnable while dark, since every unlock path lives behind `act2_enabled()`-gated code.

#### Scenario: The launch unlocks The Burn

- **WHEN** the player performs the all-or-nothing launch burn
- **THEN** The Burn unlocks once, account-wide

#### Scenario: Souls tiers follow the lifetime counter

- **WHEN** a crossing's delivery raises lifetime souls delivered across 1,000, 10,000, or 50,000
- **THEN** the corresponding Ferryman tier unlocks

#### Scenario: The covenant is judged at era end

- **WHEN** the Last Crossing completes
- **THEN** The Last Crossing unlocks, and The Covenant Kept unlocks only if no crew soul was ever lost across the era

#### Scenario: Waypoint and pilgrim unions accumulate across crossings

- **WHEN** the vessel docks at waypoints or hails pilgrim ships across several crossings whose per-crossing logs reset at each departure
- **THEN** the persisted unions retain every waypoint docked and ship hailed, and Every Star a Harbor / Company on the Road unlock when their unions cover all 38 waypoints / all 5 ships

#### Scenario: Single-crossing collections judge the live crossing

- **WHEN** a single crossing holds all 8 rumors at once, or has taken a refit at all three shipyard doors
- **THEN** Ear to the Water / Three Doors Opened unlock at that moment, and a later crossing's reset state does not revoke them

#### Scenario: Landfall records judge the delivery

- **WHEN** a crossing makes landfall carrying 2,500 or more souls, in 8 sea-days or fewer, or with all seven berths filled
- **THEN** Heavy Lading / The Swift Passage / The Full Table unlock respectively, each once

#### Scenario: Visible but unearnable while dark

- **WHEN** the achievement browser lists Act II's subsections with the Act 2 kill-switch off
- **THEN** the Vessel achievements appear (locked), and no gameplay path can unlock them
