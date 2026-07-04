# Time Vault — Save History Specification

## Purpose

Define the Time Vault: a save-versioning system that keeps a full history of the player's save state so meaningful moments can be revisited or undone. Each notable game event records a versioned snapshot of the entire save; the player browses these snapshots through the Time Vault overlay and can restore an earlier one, branch alternate timelines from any snapshot, or discard timelines they no longer want. This capability owns snapshot triggers, the progress metadata carried by each snapshot, the browse/restore/fork/switch/delete flows, and retention behavior.

## Requirements

### Requirement: Initialize the Save History Vault

The system SHALL maintain the save history as a versioned repository stored alongside the player's save files. On first use, when no repository yet exists, the system SHALL create one, capture every existing save file into a single initial snapshot labeled "Initialize save history", and name the primary timeline "main". The system SHALL keep the cloud-configuration file out of the tracked history by excluding it via an ignore rule, and SHALL remove it from tracking if it was previously captured.

#### Scenario: First run creates the vault

- **WHEN** the game starts and the save directory has no existing history repository
- **THEN** a repository is created, all current save files are captured into an initial snapshot named "Initialize save history"
- **AND** the primary timeline is named "main"
- **AND** the cloud-configuration file is excluded from tracked history

#### Scenario: Existing vault is reused

- **WHEN** the game starts and a history repository already exists in the save directory
- **THEN** the existing repository is opened and no additional initial snapshot is created

### Requirement: Capture Snapshots on Milestone Events

The system SHALL record a new snapshot of the full save state when a meaningful milestone occurs, including: defeating a zone boss (including the Storm Citadel final boss), completing a dungeon, catching the Storm Leviathan, unlocking an achievement, prestiging, winning a challenge minigame, building or upgrading a Haven room, a Soulforge enhancement succeeding or failing, and completing a Chrono Surge. A snapshot SHALL be recorded only when the save contents actually changed since the previous snapshot; if nothing changed, no snapshot is added.

#### Scenario: Milestone records a snapshot

- **WHEN** the player defeats a zone boss
- **THEN** a new snapshot is added to the active timeline describing that milestone

#### Scenario: No change means no snapshot

- **WHEN** a snapshot is requested but the save contents are identical to the most recent snapshot
- **THEN** no new snapshot is added

### Requirement: Snapshot Cadence Is Event-Driven, Not Time-Based

The system SHALL trigger snapshots from milestone events and player actions rather than on a fixed timer. The routine 30-second autosave SHALL write the save files to disk without adding a snapshot to the history, and ordinary combat or experience gains SHALL NOT add snapshots.

#### Scenario: Autosave does not snapshot

- **WHEN** the 30-second autosave fires
- **THEN** the save files are written to disk
- **AND** no new snapshot is added to the history

#### Scenario: Routine combat does not snapshot

- **WHEN** the hero defeats an ordinary enemy or gains experience without hitting a milestone
- **THEN** no new snapshot is added

### Requirement: Embed Progress Metadata in Each Snapshot

Each snapshot SHALL carry a human-readable label followed by an encoded progress summary in the form `<description> | Lv<level> P<prestige> Z<zone>-<subzone> <H>h<MM>m @<character name>`, where the playtime is formatted as hours and two-digit minutes. The Time Vault SHALL read this summary back to display, for each snapshot, its short identifier (the first 7 characters of the version hash), timestamp, character level, prestige rank, zone, and total playtime — without opening the save file itself.

#### Scenario: Snapshot encodes progress

- **WHEN** a level-18 hero at prestige 0 in zone 2 subzone 3 with 2h15m playtime named "Hero" records a snapshot
- **THEN** the snapshot label ends with `| Lv18 P0 Z2-3 2h15m @Hero`

#### Scenario: Metadata read back for browsing

- **WHEN** the Time Vault lists a snapshot
- **THEN** it shows that snapshot's short identifier, timestamp, level, prestige, zone, and playtime parsed from the encoded summary

### Requirement: Browse Timelines and Snapshots

The Time Vault overlay SHALL present the player's timelines and, for a selected timeline, its snapshots. Timelines SHALL be listed with the currently active timeline first and the remainder in alphabetical order. Snapshots within a timeline SHALL be listed newest first.

#### Scenario: Open the Time Vault

- **WHEN** the player opens the Time Vault
- **THEN** the timelines are listed with the active timeline first, then the others alphabetically
- **AND** the selected timeline's snapshots are shown newest first

### Requirement: Restore a Past Snapshot

The system SHALL let the player select an earlier snapshot on the active timeline and, after an explicit confirmation, restore the save to that snapshot's state. Before restoring, the system SHALL save and snapshot the current state so no progress is silently lost. Restoring SHALL move the active timeline back to the chosen snapshot, discarding snapshots taken after it from that timeline, then reload the game from the restored save and suppress offline-progress accrual for the reload.

#### Scenario: Restore reloads earlier state

- **WHEN** the player selects an earlier snapshot on the active timeline and confirms the restore
- **THEN** the current state is first saved and snapshotted as a safety point
- **AND** the game reloads at the chosen snapshot's state
- **AND** offline progress is not accrued for the reload

#### Scenario: Restore requires confirmation

- **WHEN** the player chooses to restore a snapshot but does not confirm
- **THEN** no restore occurs and the save state is unchanged

### Requirement: Fork and Switch Timelines

The system SHALL let the player branch a new timeline from any snapshot under a chosen name and switch between existing timelines. Forking or switching SHALL first save and snapshot the current state, then check out the target timeline and reload the game. A new timeline name SHALL contain only lowercase letters, digits, hyphens, and underscores, SHALL be at most 16 characters, SHALL NOT be empty, SHALL NOT be "main", and SHALL NOT start with a hyphen; a name that is invalid or already in use SHALL be rejected without creating a timeline.

#### Scenario: Fork a new timeline

- **WHEN** the player branches from a snapshot with a valid, unused name
- **THEN** a new timeline is created at that snapshot and becomes the active timeline
- **AND** the current state is saved and snapshotted beforehand

#### Scenario: Invalid timeline name is rejected

- **WHEN** the player tries to create a timeline named "main", an empty name, a name over 16 characters, a name starting with a hyphen, or a name already in use
- **THEN** the timeline is not created

#### Scenario: Switch to another timeline

- **WHEN** the player switches to an existing timeline other than the active one
- **THEN** the current state is saved and snapshotted, the target timeline is checked out, and the game reloads from it

### Requirement: Delete Timelines With Guardrails

The system SHALL let the player delete a timeline only after typing its exact name to confirm, and SHALL refuse to delete the "main" timeline or the currently active timeline.

#### Scenario: Delete a disposable timeline

- **WHEN** the player selects a non-main, non-active timeline and types its exact name to confirm deletion
- **THEN** that timeline is removed from the vault

#### Scenario: Protected timelines cannot be deleted

- **WHEN** the player attempts to delete the "main" timeline or the currently active timeline
- **THEN** the deletion is refused

### Requirement: Retain Snapshots Without Automatic Pruning

The system SHALL retain snapshots indefinitely and SHALL NOT automatically prune or cap the number of snapshots on a timeline. Snapshots discarded from a timeline by a restore SHALL remain recoverable through the underlying version history's reflog rather than being permanently deleted.

#### Scenario: History is not auto-trimmed

- **WHEN** many milestone snapshots accumulate over a long play session
- **THEN** older snapshots remain in the timeline's history and are not automatically removed

#### Scenario: Restored-past snapshots remain recoverable

- **WHEN** a restore discards snapshots taken after the chosen point
- **THEN** those snapshots are still recoverable from the underlying version history and are not permanently erased
