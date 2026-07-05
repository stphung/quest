# Fishing Specification

## Purpose

Fishing is a parallel idle-progression track in which the hero discovers fishing spots, runs timed catch sessions, and advances through 40 fishing ranks that raise catch rarity. It grants character experience and occasional equipment drops, and it is the sole path to the Storm Leviathan — the climactic catch required before the Stormbreaker can be forged. Fishing runs on the same fixed tick loop as combat but is mutually exclusive with it: on any given tick the hero is either fighting or fishing, never both.

## Requirements

### Requirement: Fishing Spot Discovery

The system SHALL roll for the discovery of a new fishing spot after the hero defeats an enemy, with a 5 percent chance per kill. A discovery SHALL be suppressed when a fishing session is already active, when the hero is inside a dungeon, or when a dungeon was discovered on the same kill. On discovery the system SHALL immediately begin a new fishing session at a randomly chosen spot and announce the spot by name. There SHALL be no prestige-rank requirement to discover fishing.

#### Scenario: Spot found after a kill

- **WHEN** the hero defeats an enemy while not fishing and not in a dungeon, and the 5 percent discovery roll succeeds
- **THEN** a new fishing session begins at a randomly selected spot and a discovery message naming that spot is reported

#### Scenario: No discovery while already fishing

- **WHEN** a fishing-spot discovery is rolled while a fishing session is already active
- **THEN** no new spot is discovered and the existing session continues undisturbed

#### Scenario: No discovery inside a dungeon

- **WHEN** a fishing-spot discovery is rolled while the hero is inside a dungeon
- **THEN** no fishing spot is discovered

### Requirement: Fishing Session Structure

The system SHALL create each fishing session at one of 8 named spots with a randomly chosen quota of 3 to 8 fish (inclusive) to catch. A session SHALL cycle each fish through three phases in order — Casting, then Waiting for a bite, then Reeling in — with each phase running a randomly rolled duration: Casting 5 to 15 ticks (0.5 to 1.5 seconds), Waiting 10 to 80 ticks (1.0 to 8.0 seconds), and Reeling 5 to 30 ticks (0.5 to 3.0 seconds). A fish SHALL be caught at the end of the Reeling phase, after which the session SHALL either start a new Casting phase for the next fish or, once the quota is met, end with the spot reported as depleted.

#### Scenario: Phase progression for one fish

- **WHEN** a session is in the Casting phase and the phase timer reaches zero
- **THEN** the session advances to the Waiting phase with a freshly rolled duration, then to Reeling, and catches a fish when the Reeling timer expires

#### Scenario: Session ends when quota is met

- **WHEN** a catch brings the number of fish caught up to the session's quota of 3 to 8 fish
- **THEN** the session ends, the spot is reported as depleted, and no further catches occur at that spot

### Requirement: Fishing And Combat Mutual Exclusivity

The system SHALL, when a fishing session is active during a tick, advance the fishing session and skip automatic combat for that tick, so the hero is never both fighting and fishing in the same tick. While fishing is active the system SHALL still account play time for the tick.

#### Scenario: Fishing suspends combat

- **WHEN** a tick runs while a fishing session is active
- **THEN** the fishing session is advanced and no combat is resolved for that tick

#### Scenario: Play time still credited while fishing

- **WHEN** ten ticks elapse during an active fishing session
- **THEN** one second of play time is credited even though no combat occurred

### Requirement: Catch Rarity Rolling

The system SHALL assign every catch a rarity of Common, Uncommon, Rare, Epic, or Legendary using base probabilities of 60, 25, 10, 4, and 1 percent respectively. For every 5 fishing ranks the system SHALL shift these odds by −2 percent Common, +1 percent Uncommon, +0.5 percent Rare, +0.3 percent Epic, and +0.2 percent Legendary, cumulatively, so higher ranks yield rarer fish. The Common probability SHALL never fall below a floor of 10 percent.

#### Scenario: Base odds at rank 1

- **WHEN** a fish rarity is rolled at fishing rank 1
- **THEN** the odds are 60 percent Common, 25 percent Uncommon, 10 percent Rare, 4 percent Epic, and 1 percent Legendary

#### Scenario: Rarer fish at high rank

- **WHEN** a fish rarity is rolled at a high fishing rank
- **THEN** Common becomes less likely (never below 10 percent) and the combined chance of Uncommon through Legendary is higher than at rank 1

### Requirement: Catch Rewards And Progress

The system SHALL award character experience for each caught fish based on its rarity — Common 50 to 100, Uncommon 150 to 250, Rare 400 to 600, Epic 1,000 to 1,500, and Legendary 3,000 to 5,000 — multiplied by the hero's prestige multiplier before being added to character experience, resolving any resulting level-ups. Each catch SHALL increment the total fish caught count, and Legendary catches SHALL additionally increment the legendary catch count. Progress toward the next fishing rank SHALL increment only while the current rank is below the effective maximum rank; total fish caught SHALL always increment regardless of rank cap.

#### Scenario: Experience scaled by prestige

- **WHEN** the hero catches a fish while at a prestige rank above zero
- **THEN** the fish's base experience is multiplied by the prestige multiplier and added to character experience, triggering level-ups if the threshold is crossed

#### Scenario: Progress halts at the rank cap but the tally continues

- **WHEN** the hero catches a fish while already at the effective maximum fishing rank
- **THEN** the total fish caught count increases but progress toward the next rank does not

### Requirement: Fishing Rank Progression

The system SHALL provide 40 fishing ranks grouped into 8 named tiers, advancing the rank whenever accumulated progress reaches the fish requirement for the current rank, with any excess carried over to the next rank. Fish required per rank SHALL be 100 for ranks 1–5, 200 for ranks 6–10, 400 for ranks 11–15, 800 for ranks 16–20, 1,500 for ranks 21–25, and 2,000 for ranks 26–30, then escalate through the Mythic tier (4,000; 6,000; 10,000; 16,000; 25,000 for ranks 31–35) and Transcendent tier (40,000; 65,000; 100,000; 160,000; 250,000 for ranks 36–40). The effective maximum rank SHALL be 30 by default, raised to 40 only when the Haven Fishing Dock tier 4 bonus grants +10, and the rank SHALL never exceed this effective maximum.

#### Scenario: Rank up with carryover

- **WHEN** progress toward the next rank reaches or exceeds the current rank's fish requirement and the hero is below the effective maximum rank
- **THEN** the rank increases by one, the requirement is subtracted, any surplus carries into the next rank, and a rank-up message is reported

#### Scenario: Capped at 30 without the Haven bonus

- **WHEN** the hero reaches rank 30 without the Fishing Dock tier 4 bonus
- **THEN** the rank cannot advance to 31 no matter how many more fish are caught

#### Scenario: Extended to 40 with Fishing Dock tier 4

- **WHEN** the Fishing Dock tier 4 bonus of +10 maximum rank is active
- **THEN** the effective maximum fishing rank becomes 40 and progression may continue past rank 30

### Requirement: Item Drops From Catches

The system SHALL roll for an equipment drop on every catch, with a drop chance determined by the fish's rarity: 5 percent for Common and Uncommon, 15 percent for Rare, 35 percent for Epic, and 75 percent for Legendary. A dropped item's rarity SHALL match the fish rarity, mapping Common to Common, Uncommon to Magic, Rare to Rare, Epic to Epic, and Legendary to Legendary, and its item level SHALL be derived from the hero's current zone (10 times the zone id).

#### Scenario: Legendary catch usually drops loot

- **WHEN** the hero catches a Legendary fish
- **THEN** an item drop is rolled at 75 percent, and any dropped item is Legendary rarity at the current zone's item level

#### Scenario: Common catch rarely drops loot

- **WHEN** the hero catches a Common fish
- **THEN** an item drop is rolled at only 5 percent, and any dropped item is Common rarity

### Requirement: Haven And God-Item Fishing Bonuses

The system SHALL apply Haven bonuses to fishing when present: the Garden bonus SHALL reduce Casting, Waiting, and Reeling phase durations by a percentage; the Fishing Dock bonus SHALL grant a percentage chance to catch two fish instead of one on a single reel; and the Fishing Dock tier 4 bonus SHALL raise the maximum fishing rank by 10. God-item fishing timer reduction SHALL stack multiplicatively with the Garden reduction, and every reduced phase duration SHALL be at least 1 tick. When a double catch occurs, each of the two fish SHALL independently roll its own rarity, experience, and item drop.

#### Scenario: Timer reductions stack multiplicatively

- **WHEN** a phase duration is reduced by both a Garden bonus and a god-item bonus
- **THEN** the reductions apply one after the other (multiplicatively) and the final duration is never below 1 tick

#### Scenario: Double catch on one reel

- **WHEN** the Fishing Dock double-fish chance succeeds on a reel
- **THEN** two fish are caught in that reel, each independently rolled for rarity, experience, and item drop

### Requirement: Storm Leviathan Hunt

The system SHALL make the Storm Leviathan available only at fishing rank 40 or higher and only on Legendary catches. The hunt SHALL require 10 progressive encounters before the beast can be caught, each Legendary catch rolling against the encounter's fixed chance in sequence: 5, 3, 4, 5, 4, 3, 2, 1.5, 1, and 0.8 percent for encounters 1 through 10. Each successful roll SHALL count as an encounter in which the Leviathan appears and escapes, incrementing the encounter counter. After all 10 encounters are complete, each subsequent Legendary catch SHALL have a 25 percent chance to catch the Storm Leviathan, awarding 10,000 to 15,000 experience, marking the Leviathan as caught, and unlocking the ability to forge the Stormbreaker at the Storm Forge.

#### Scenario: No Leviathan below rank 40

- **WHEN** a Legendary fish is caught while the fishing rank is below 40
- **THEN** no Storm Leviathan encounter or catch can occur

#### Scenario: Progressive escape

- **WHEN** a Legendary fish is caught at rank 40 with fewer than 10 encounters recorded and the current encounter's chance roll succeeds
- **THEN** the Leviathan appears and escapes, and the encounter counter increases by one

#### Scenario: Catch after ten encounters

- **WHEN** a Legendary fish is caught at rank 40 with all 10 encounters complete and the 25 percent catch roll succeeds
- **THEN** the Storm Leviathan is caught, 10,000 to 15,000 experience is awarded, and Stormbreaker forging becomes available

### Requirement: Storm Lure Assistance

The system SHALL support an optional Storm Lure — a Stormglass consumable — that improves the Storm Leviathan hunt while active. When the lure is active the system SHALL add a combined tracking-plus-miss-ramp bonus to both the encounter chance and the post-encounter catch chance. The tracking bonus SHALL increase by 1.5 percent per completed encounter and persist across lure purchases; the miss-ramp bonus SHALL increase by 0.5 percent each time a Legendary catch fails to produce an encounter or catch, capping at 10 percent and resetting to zero whenever an encounter fires or the Leviathan is caught. The lure SHALL be consumed when it produces an encounter, a catch miss, or a catch, and lure purchases SHALL be disabled once the Storm Leviathan has been caught.

#### Scenario: Lure raises the odds and is consumed on an encounter

- **WHEN** the Storm Lure is active and a Legendary catch produces a Leviathan encounter
- **THEN** the encounter fires with the boosted chance, the lure is consumed, the tracking bonus grows by 1.5 percent, and the miss ramp resets to zero

#### Scenario: Miss ramp accumulates on failed legendaries

- **WHEN** the Storm Lure is active and a Legendary catch yields no encounter
- **THEN** the miss-ramp bonus increases by 0.5 percent, up to a cap of 10 percent, improving the odds of the next attempt
