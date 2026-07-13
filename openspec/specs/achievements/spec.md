# Achievements, Titles & Milestones Specification

## Purpose

The achievement system tracks account-level milestones a player reaches while playing any character — combat kills, leveling, prestige, zone and fracture-zone clears, challenges, fishing, dungeons, enhancement, Haven, the Deep, Ascension, Power Cores, and the Loom. Each achievement is a permanent, once-only unlock earned when its condition is met. Unlocking achievements accumulates an achievement score and can grant selectable character titles displayed alongside the hero's name. This capability defines how achievements are unlocked, how progress and milestones are tracked, how scoring works, how titles are earned and shown, and how all of it persists across characters and prestige.
## Requirements
### Requirement: Account-Level Persistence Across Characters and Prestige

The system SHALL store all achievement state — unlocked achievements, progress, aggregate counters, the selected title, and the global UI border style — in a single account-level record that is separate from any per-character save. Aggregate counters (total kills, bosses, fish, dungeons, minigame wins, highest prestige rank, highest level, highest fishing rank, zones cleared, Expanse cycles, Deep missions, deepest Deep layer, highest guild rank) SHALL accumulate across every character on the account, and unlocked achievements SHALL never be cleared by prestiging, switching characters, or creating a new character. If the account record is missing or cannot be parsed, the system SHALL start from a fresh empty state rather than failing.

#### Scenario: Progress survives prestige and character switches

- **WHEN** a character prestiges or the player switches to a different character
- **THEN** previously unlocked achievements remain unlocked and the aggregate counters retain their accumulated values

#### Scenario: Missing or corrupt record loads fresh

- **WHEN** the achievement record file is absent or contains unreadable data
- **THEN** the system loads a default state with no unlocked achievements and all counters at zero

### Requirement: Permanent One-Time Unlock Contract

The system SHALL treat each achievement as a binary latch: it transitions from locked to unlocked exactly once, the first time its condition is satisfied, and stays unlocked permanently thereafter. On the transition the system SHALL record the unlock timestamp and the name of the character that triggered it. Re-satisfying the same condition later SHALL have no effect and SHALL NOT produce a duplicate unlock or a repeat notification.

#### Scenario: First satisfaction unlocks

- **WHEN** an achievement's condition is met while it is still locked
- **THEN** it becomes unlocked, its unlock time and triggering character name are recorded, and it is queued for notification

#### Scenario: Repeat satisfaction is a no-op

- **WHEN** the condition for an already-unlocked achievement is met again
- **THEN** the achievement stays unlocked with its original timestamp and no new notification is generated

### Requirement: Cumulative-Counter Milestone Unlocks

The system SHALL unlock tiered achievement series by comparing running counters against ordered thresholds, unlocking every threshold at or below the current counter value. The kill series (Slayer I–XV) SHALL trigger at 100; 500; 1,000; 5,000; 10,000; 50,000; 100,000; 500,000; 1,000,000; 2,500,000; 10,000,000; 50,000,000; 100,000,000; 500,000,000; and 1,000,000,000 kills. The boss series (Boss Hunter I–XV) SHALL trigger at 1; 10; 50; 100; 500; 1,000; 5,000; 10,000; 25,000; 75,000; 250,000; 750,000; 2,500,000; 5,000,000; and 10,000,000 bosses. Level milestones SHALL span level 10 through level 100,000, fish-catch milestones 1 fish through 100,000,000 fish, dungeon-completion milestones 1 through 1,000,000 dungeons, fishing-rank milestones ranks 10/20/30/40, and a Grand Champion milestone SHALL trigger at 100 total minigame wins. For any tier not yet unlocked, the system SHALL record current-versus-target progress toward its threshold.

#### Scenario: Crossing the first kill threshold

- **WHEN** the account's cumulative kill count reaches 100
- **THEN** the first-tier kill achievement unlocks and higher tiers remain locked with progress tracked toward their thresholds

#### Scenario: Progress recorded before unlock

- **WHEN** a counter is below a tier's threshold
- **THEN** the system stores the counter as current progress against that threshold without unlocking the tier

### Requirement: Single-Event Discovery and Completion Unlocks

The system SHALL unlock certain achievements on a single discrete event rather than a cumulative count, including: catching the Storm Leviathan; forging the Stormbreaker; defeating the final zone (Storm's End); completing individual zone clears for zones 1–10; completing fracture zones 12–30; completing a cycle of the Expanse (Beyond Infinity); discovering Haven, the Soulforge, the Deep, and the Loom; losing a mercenary for the first time; completing the first breakthrough; and opening the Gateway. Each such achievement SHALL unlock the first time its event fires and SHALL be independent of the others.

#### Scenario: Discovery unlocks on first occurrence

- **WHEN** the Loom of Worlds is discovered for the first time
- **THEN** the Loom discovery achievement unlocks

#### Scenario: Zone clear unlocks its specific achievement

- **WHEN** all subzones of a given zone (1–10 or fracture zone 12–30) are cleared
- **THEN** that zone's completion achievement unlocks and unrelated zone achievements stay locked

### Requirement: Cross-System Milestone Achievements

The system SHALL grant milestone achievements tied to endgame systems using their native progression values. Ascension milestones (I–X) SHALL unlock at ascension levels 1 through 10. Deep layer milestones SHALL unlock at layers 5, 10, 15, 20, 25, and 26 (the Void). Deep mission milestones SHALL unlock at 1, 10, 25, 50, and 100 completed missions. Deep guild-rank milestones SHALL unlock at guild ranks 2, 3, 4, and 5. Power Core milestones (I–VI) SHALL unlock when the deepest Deep layer reached passes 3, 7, 12, 18, 25, and 30. Loom pattern milestones SHALL unlock at 1, 4, 8, 16, 22, and 28 completed Woven Patterns. Enhancement milestones SHALL unlock at reaching +1, +5, +6, +7, +8, +9, and +10 on any equipment slot, at +4 on all seven slots, at +7 on all seven slots, and at 100 total enhancement attempts.

#### Scenario: Ascension level grants its milestone

- **WHEN** a character ascends to ascension level 3
- **THEN** the third Ascension milestone unlocks

#### Scenario: Power Core milestone tied to Deep depth

- **WHEN** the deepest Deep layer reached passes layer 7
- **THEN** the second Power Core milestone unlocks

### Requirement: Prestige Milestones Use an Account High-Water Mark

The system SHALL evaluate prestige milestone achievements against the account's highest prestige rank ever reached, not the current spendable rank. Because prestige rank is also a currency that can be spent (on Ascension, Haven, enhancement, and other sinks), the current rank may drop below a milestone already earned; the system SHALL keep such milestones unlocked and SHALL NOT regress recorded progress toward higher milestones when the current rank falls. Prestige milestones SHALL span rank 1 (First Prestige) through rank 10,000, with rank 100 granting the Eternal milestone.

#### Scenario: Spending prestige does not revoke milestones

- **WHEN** a player reaches a high prestige rank, then spends prestige rank down below a previously earned milestone
- **THEN** the earned milestones stay unlocked and progress toward higher milestones reflects the highest rank ever reached

#### Scenario: Milestone unlocks at the account peak

- **WHEN** the account's highest prestige rank reaches 100
- **THEN** the Eternal prestige milestone unlocks

### Requirement: Retroactive Synchronization on Character Load

The system SHALL retroactively unlock all achievements whose milestones a loaded character has already passed, covering level, prestige, fishing rank, fish count, zone completions (derived from defeated subzone bosses), ascension level, Deep discovery/guild-rank/layer, Haven discovery and per-tier room completion, and Loom discovery and pattern completion. When reconciling counters, the system SHALL take the maximum of the stored value and the incoming value so a lower character-side count never decreases an already-higher account counter.

#### Scenario: Loading a progressed character backfills milestones

- **WHEN** a character at level 120 and prestige 17 is loaded
- **THEN** all level milestones up to 100 and all prestige milestones up to rank 15 are unlocked, while higher, unreached milestones remain locked

#### Scenario: Sync never lowers a higher counter

- **WHEN** the account already records a higher fish count than the loaded character's save reports
- **THEN** the account counter is left unchanged and the corresponding high-count achievements remain unlocked

### Requirement: Achievement Categories, Count, and Scoring

The system SHALL define 240 achievements, each assigned to one of nine browsing categories (Combat, Level, Prestige, Progression, Challenges, Exploration, The Deep, Loom, Stats) and one point value drawn from a tiered scale of 5, 10, 25, 50, 100, 250, and 500 points. The achievement score SHALL be computed as the sum of the point values of all currently unlocked achievements, and the system SHALL also expose the unlocked/total count overall and per category and an overall unlock percentage. The score SHALL rise only as new achievements unlock and SHALL equal zero when none are unlocked.

#### Scenario: Score is the sum of unlocked points

- **WHEN** two achievements worth 5 and 25 points are unlocked and no others
- **THEN** the achievement score is 30

#### Scenario: Category counts reflect unlocks

- **WHEN** two Combat achievements are unlocked
- **THEN** the Combat category reports two unlocked out of its total, and categories with no unlocks report zero

### Requirement: Titles Earned From Achievements

The system SHALL let a player earn a title from each of a curated subset of achievements, where unlocking the achievement makes its title available. The player MAY select at most one active title, stored account-wide, which is displayed as a comma-separated suffix after the character's name (for example, "Hero, Eternal") in the stats panel and character-select views. Only titles from unlocked achievements SHALL be selectable, and on load the system SHALL clear the selected title if its achievement is not unlocked or no longer grants a title.

#### Scenario: Unlocked achievement grants a selectable title

- **WHEN** an achievement that maps to a title is unlocked
- **THEN** that title becomes available for selection and, if chosen, appears after the character name

#### Scenario: Invalid selected title is cleared on load

- **WHEN** the account loads with a selected title whose achievement is not unlocked
- **THEN** the selected title is cleared and no title is displayed

### Requirement: Batched Unlock Notifications

The system SHALL batch achievements unlocked in close succession into a single notification burst using a 500-millisecond accumulation window that begins at the first unlock in a batch; the batched set becomes ready to display as a modal only after the window elapses. The system SHALL additionally track unviewed unlocks as a pending-notification badge count that is cleared when the player opens the achievement browser, and SHALL surface each newly unlocked achievement once for event logging. Notification-tracking state SHALL be transient and SHALL NOT be persisted to disk.

#### Scenario: Rapid unlocks batch into one modal

- **WHEN** several achievements unlock within 500 milliseconds of each other
- **THEN** they accumulate together and are presented as a single modal burst once the window elapses

#### Scenario: Opening the browser clears the pending badge

- **WHEN** the player opens the achievement browser with unviewed unlocks pending
- **THEN** the pending-notification count is cleared

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
