# Save Persistence & Account Model Specification

## Purpose

Define how Quest persists game state to disk as human-readable JSON, split between per-character run state and shared account-level state, and how the save directory is resolved (honoring the `QUEST_DIR` environment variable for isolated runs, otherwise the default OS home directory). This capability owns the durable on-disk file contract, the autosave and save-on-event flow, and — most importantly — the backward-compatibility guarantee that existing player saves keep loading across code changes, together with the silent-wipe hazard that makes the committed save corpus the only signal a compatibility break has occurred.

## Requirements

### Requirement: Save Directory Resolution

The system SHALL resolve a single save directory for all persistence, preferring the `QUEST_DIR` environment variable when it is set to a non-empty (non-whitespace) value, and otherwise defaulting to the `.quest` directory inside the operating-system home directory (i.e. `~/.quest`). Every persistence module SHALL resolve its file paths through this shared resolver rather than constructing the home path independently. If neither `QUEST_DIR` is usable nor the home directory can be determined, directory resolution SHALL fail rather than silently pick an arbitrary location.

#### Scenario: QUEST_DIR override

- **WHEN** `QUEST_DIR` is set to a non-empty path
- **THEN** all save and load operations use that path as the save directory, isolating the run from the player's real `~/.quest` saves

#### Scenario: Empty override falls back to default

- **WHEN** `QUEST_DIR` is unset, empty, or only whitespace
- **THEN** the save directory resolves to `~/.quest` under the home directory

#### Scenario: Home directory undeterminable

- **WHEN** `QUEST_DIR` is not usable and the home directory cannot be determined
- **THEN** directory resolution returns an error instead of a fabricated path

### Requirement: Per-Character Run State File

The system SHALL persist each character's run state to its own JSON file named after the sanitized character name (`{sanitized_name}.json`) within the save directory. This file SHALL carry the character's identity, level, experience, attributes, prestige rank, play time, combat/equipment/dungeon/fishing/zone state, Stormglass currency and Storm Sigils, ascension level, and Act 2 vessel flags. The file SHALL include a save-format version field (currently 2). On every save the stored last-save timestamp SHALL be set to the current wall-clock time (not a value carried in memory), so that active play cannot be replayed as offline progression.

#### Scenario: Character save round-trips

- **WHEN** a character is saved and then loaded again
- **THEN** its level, experience, prestige rank, ascension level, attributes, zone progression, and equipped items are identical to what was saved

#### Scenario: Last-save time is refreshed on write

- **WHEN** a character is saved during an active session
- **THEN** the persisted last-save timestamp is the current time at the moment of writing, preventing accrual of offline experience for time spent actively playing

#### Scenario: Transient state resets on load

- **WHEN** a character file is loaded into game state
- **THEN** run-only fields (active fishing session, challenge menu, session kills, consecutive deaths, recent-drop ticker, cached derived stats) are reconstructed to their defaults rather than read from disk

### Requirement: Account-Level State Files

The system SHALL persist account-level progression — shared across all characters and surviving every prestige reset — in separate JSON files in the same save directory, one file per system: `haven.json`, `enhancement.json`, `achievements.json`, `deep.json`, and `loom.json`. The Deep file SHALL also carry Power Core state. Account files that gate on discovery (Haven, Soulforge/enhancement, the Deep, the Loom) SHALL be written only once their system has been discovered; achievements SHALL always be written.

#### Scenario: Account state survives prestige and character switches

- **WHEN** a player prestiges or switches to a different character
- **THEN** Haven rooms, Soulforge enhancement levels, achievements, Deep progress, and Loom progress persist unchanged because they live in account-level files, not in the character save

#### Scenario: Undiscovered systems are not written

- **WHEN** a save occurs while the Deep (or Haven, Soulforge, or Loom) has not yet been discovered
- **THEN** that system's account file is not written

### Requirement: Backward-Compatibility Contract

The system SHALL keep loading save files written by earlier versions of the game. New or changed serialized fields SHALL be made load-compatible with existing saves through serde defaults, field aliases, or explicit migration logic — never by editing or regenerating committed save files to match new code. A committed save corpus SHALL exist and SHALL be loaded through the game's real deserialization paths as a regression gate; a failure of that gate means existing player saves would break and MUST be fixed in code, not by touching the corpus. After an intentional, migration-backed format change, a NEW corpus generation SHALL be added while all older generations continue to load.

#### Scenario: New field added with a default

- **WHEN** a new field is added to a serialized save structure and marked with a serde default
- **THEN** an older save that lacks the field still loads, taking the default for the missing field

#### Scenario: Committed corpus stays frozen

- **WHEN** a code change causes a committed corpus file to fail deserialization
- **THEN** the fix restores compatibility in code (default, alias, or migration) and the committed corpus file is left unchanged

#### Scenario: Real load paths gate the corpus

- **WHEN** the save-compatibility test suite runs
- **THEN** every committed corpus file is deserialized through the same code path the running game uses, so a break is caught before shipping

### Requirement: Silent Progress Wipe on Account-File Parse Failure

The system's account-level loaders (Haven, enhancement, achievements, the Deep, the Loom) SHALL fall back to a fresh default state when their file is missing, unreadable, or fails to parse — they do NOT surface an error or halt. Because this fallback silently discards the player's existing account progress on a parse failure, the committed save corpus SHALL be treated as the only reliable signal that an account-file format change has broken compatibility.

#### Scenario: Corrupt account file resets silently

- **WHEN** an account-level file such as `haven.json` or `deep.json` contains malformed JSON
- **THEN** the loader returns a default (empty) state and the game continues without an error, silently discarding prior progress for that system

#### Scenario: Missing account file starts fresh

- **WHEN** an account-level file does not exist yet
- **THEN** the loader returns a default state so a first-time system starts cleanly

### Requirement: Character-File Corruption Is Surfaced, Not Discarded

Unlike account files, the system SHALL NOT silently overwrite a character save that fails to load. A character load that cannot parse SHALL return an error, and the character-listing screen SHALL present such a file as a corrupted entry (using any header fields it could still read, or a placeholder name when even the header is unreadable) so the player can see and choose to delete it rather than losing it unknowingly.

#### Scenario: Corrupt character file listed as corrupted

- **WHEN** the character list is built and one `.json` character file cannot be fully deserialized
- **THEN** it appears in the list flagged as corrupted rather than being hidden or auto-reset

#### Scenario: Load of corrupt character errors

- **WHEN** the game attempts to load a character file whose JSON is invalid
- **THEN** the load returns an error instead of substituting a default character

### Requirement: Account-File Isolation in Character Listing

When enumerating characters, the system SHALL scan `.json` files in the save directory and SHALL skip any file that lacks a string `character_name` field, treating it as an account-level file rather than a character. The resulting character list SHALL be sorted by last-save time, most recent first.

#### Scenario: Account files excluded from character list

- **WHEN** the save directory contains both character files and account files (e.g. `haven.json`)
- **THEN** only the character files appear in the character list and the account files are silently skipped

#### Scenario: Listing ordered by recency

- **WHEN** multiple characters exist
- **THEN** they are listed with the most recently saved character first

### Requirement: Loom Version-Gated Reset and Load Sanitization

The Loom account loader SHALL compare the loaded file's stored version against the current Loom save version and, when the stored version is older, SHALL discard the file and return a fresh Loom state rather than loading incompatible data — performing this reset cleanly without erroring. On loading a current-version Loom file, the loader SHALL also sanitize floating-point fields (replacing NaN or infinite values with safe finite defaults) and clamp out-of-range indices and stale node references.

#### Scenario: Legacy Loom save resets cleanly

- **WHEN** a Loom file with a version below the current Loom save version is loaded
- **THEN** the loader returns a fresh, undiscovered Loom state with no shuttles or patterns, and does not error

#### Scenario: Current Loom save is sanitized

- **WHEN** a current-version Loom file is loaded with a non-finite buffer value or an out-of-range active-pattern index
- **THEN** the non-finite value is replaced with a finite default and the index is clamped into valid range

### Requirement: Autosave and Save-on-Event Flow

The system SHALL autosave every 30 seconds during play and SHALL additionally save on significant events (such as prestige, a minigame win, a character switch, and quitting). All saving SHALL flow through a single save entry point that writes the character file plus the applicable account files, and MAY create a git history commit afterward. To avoid blocking the game loop, a save MAY run on a background thread over a cloned snapshot of the state.

#### Scenario: Periodic autosave

- **WHEN** 30 seconds have elapsed since the last save during active play
- **THEN** the game persists the current character and applicable account state to disk

#### Scenario: Save on consequential action

- **WHEN** a consequential action such as a prestige or quit occurs
- **THEN** state is saved through the shared save entry point rather than waiting for the next autosave tick

### Requirement: Act 2 Voyage and Colony Files Are Character-Keyed

The system SHALL persist Act 2 crossing state in `voyage.json` and colony state in `colony.json`, each stamped with the owning character's id. A load SHALL return the stored state only when its character id matches the requesting character; a mismatched id, a missing file, or unreadable JSON SHALL all resolve to "no crossing / found no colony" so that a different character never inherits another character's Act 2 progress.

#### Scenario: Foreign voyage is not inherited

- **WHEN** a character requests its voyage but `voyage.json` was written by a different character id
- **THEN** the load returns nothing and the requesting character does not pick up the foreign crossing

#### Scenario: Missing or corrupt voyage means no crossing

- **WHEN** `voyage.json` is absent or contains invalid JSON
- **THEN** the load returns nothing and the game treats it as no crossing in progress
