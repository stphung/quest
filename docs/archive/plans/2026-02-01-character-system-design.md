# Character System Design

**Date:** 2026-02-01
**Status:** Approved
**Author:** Brainstorming session with user

## Overview

A character management system with character creation, selection, and full character lifecycle management. Characters are stored as individual JSON files in `~/.quest/` for maximum portability and simplicity.

## Design Decisions

### Save Location & Format

**Directory:** `~/.quest/` (user's home directory)

**File format:** One JSON file per character (max 3 characters)

```
~/.quest/
├── hero.json
├── warrior.json
└── mage_the_great.json
```

**Why JSON:**
- Human-readable and debuggable
- Easy to backup/share (just text files)
- Portable across systems
- Can manually inspect/edit if needed
- Still checksummed for integrity protection

**Why single file per character:**
- Maximum simplicity - no metadata sync issues
- Each file is fully self-contained and portable
- Copy a .json file = copy a character
- No separate index file needed

### Character Limits & Rules

**Maximum characters:** 3 (classic RPG style)

**Character naming rules:**
- Length: 1-16 characters
- Allowed: letters, numbers, spaces, hyphens, underscores
- Leading/trailing whitespace trimmed
- Must be unique (case-insensitive)
- Sanitized for filename safety (lowercase, spaces→underscores)

**Examples:**
- "Hero" → `hero.json`
- "Mage the Great" → `mage_the_great.json`
- "Warrior-2" → `warrior-2.json`

### JSON File Format

```json
{
  "version": 2,
  "character_id": "uuid-1234-5678-90ab",
  "character_name": "Hero",
  "character_level": 25,
  "character_xp": 125000,
  "attributes": {
    "str": 18,
    "dex": 14,
    "con": 16,
    "int": 10,
    "wis": 12,
    "cha": 8
  },
  "prestige_rank": 3,
  "total_prestige_count": 3,
  "last_save_time": 1234567890,
  "play_time_seconds": 86400,
  "combat_state": {
    "current_hp": 180,
    "max_hp": 180,
    "in_combat": false,
    "current_enemy": null,
    "log_entries": [],
    "attack_timer": 0.0,
    "hp_regen_timer": 0.0
  },
  "equipment": {
    "weapon": { "slot": "Weapon", "rarity": "Legendary", ... },
    "armor": { "slot": "Armor", "rarity": "Rare", ... },
    "helmet": null,
    "gloves": { "slot": "Gloves", "rarity": "Magic", ... },
    "boots": { "slot": "Boots", "rarity": "Magic", ... },
    "amulet": { "slot": "Amulet", "rarity": "Rare", ... },
    "ring": null
  },
  "checksum": "sha256_hash_of_all_fields_above"
}
```

**Checksum calculation:**
- Serialize all fields except `checksum` (deterministic order)
- Compute SHA256 hash of serialized data
- Store as hex string in `checksum` field
- On load, verify checksum matches - reject if tampered

### UI Flow & Screens

**Startup Flow:**

```
main() starts
    ↓
Check ~/.quest/*.json files
    ↓
┌─────────────────┬──────────────────┐
│ No characters   │ Has characters   │
│ found           │ (1-3 files)      │
└────────┬────────┴────────┬─────────┘
         ↓                 ↓
   Character          Character
   Creation           Select
   Screen             Screen
         ↓                 ↓
         └────────┬────────┘
                  ↓
            Main Game Loop
            (existing UI)
```

**Smart detection:**
- If no characters exist → Character Creation screen
- If characters exist → Character Select screen
- After creating first character → Load directly into game
- After deleting last character → Return to Character Creation

### Character Select Screen

**Layout (80x24 minimum):**

```
┌─ QUEST - Character Select ──────────────────────────────────────┐
│                                                                  │
│  ┌─ Characters ─────────┐  ┌─ Character Details ──────────────┐ │
│  │                      │  │                                   │ │
│  │ ❯ Hero               │  │ Name: Hero                        │ │
│  │   Level 25           │  │ Level: 25                         │ │
│  │   Prestige: Gold III │  │ Prestige: Gold III (Rank 5)       │ │
│  │                      │  │ Playtime: 12h 34m                 │ │
│  │   Warrior            │  │                                   │ │
│  │   Level 15           │  │ Attributes:                       │ │
│  │   Prestige: Silver I │  │   STR 23  DEX 15  CON 18         │ │
│  │                      │  │   INT 10  WIS 14  CHA 12         │ │
│  │   Mage               │  │                                   │ │
│  │   Level 8            │  │ Equipment: 5/7 slots filled       │ │
│  │   Prestige: Bronze V │  │   ⚔️  Cruel Greatsword           │ │
│  │                      │  │   🛡  Plate Mail of Valor        │ │
│  │                      │  │   🪖  [Empty]                     │ │
│  └──────────────────────┘  │   🧤  Swift Gauntlets            │ │
│                            │   👢  Boots of Haste              │ │
│                            │   📿  Amulet of Vitality          │ │
│                            │   💍  [Empty]                     │ │
│                            └───────────────────────────────────┘ │
│                                                                  │
│  [Enter] Play  [R] Rename  [D] Delete  [N] New  [Q] Quit        │
└──────────────────────────────────────────────────────────────────┘
```

**Features:**
- Left panel: Character list with name, level, prestige
- Right panel: Detailed stats for highlighted character
- Equipment preview shows item names (truncated if needed)
- "New" action disabled (grayed out) when 3 characters exist

**Actions:**
- **Play** (Enter): Load selected character into game
- **Rename** (R): Rename selected character
- **Delete** (D): Delete selected character (requires confirmation)
- **New** (N): Create new character (disabled if 3 exist)
- **Quit** (Q/Esc): Exit game

**Navigation:**
- ↑↓ or j/k: Move selection up/down
- Enter: Play selected character
- R: Rename
- D: Delete
- N: New character
- Q/Esc: Quit

### Character Creation Screen

**Layout:**

```
┌─ QUEST - Create New Character ──────────────────────────────────┐
│                                                                  │
│                                                                  │
│                     Create Your Hero                             │
│                                                                  │
│                                                                  │
│  Character Name:                                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Hero_                                                       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  • 1-16 characters                                              │
│  • Letters, numbers, spaces, hyphens, underscores               │
│  • Must be unique                                               │
│                                                                  │
│  ✓ Name is valid                                                │
│                                                                  │
│                                                                  │
│                                                                  │
│                                                                  │
│  [Enter] Create Character    [Esc] Cancel                       │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Input Behavior:**
- Text input with blinking cursor
- Real-time validation as user types
- Character count displayed: `(4/16)`
- Validation feedback changes color:
  - Red: `✗ Too short (minimum 1 character)`
  - Red: `✗ Too long (maximum 16 characters)`
  - Red: `✗ Invalid characters`
  - Red: `✗ Name already exists`
  - Green: `✓ Name is valid`
- Enter button disabled until name is valid
- Backspace/Delete to edit input

**Cancel Behavior:**
- If characters exist: Return to character select screen
- If no characters exist: Exit game (can't play without a character)

**On Success:**
- Generate UUID for character_id
- Create GameState with character_name and character_id
- Sanitize name for filename
- Save to `~/.quest/{sanitized_name}.json`
- Load character directly into game

### Delete Confirmation Screen

**Layout:**

```
┌─ QUEST - Delete Character ───────────────────────────────────────┐
│                                                                   │
│                                                                   │
│                     ⚠️  DELETE CHARACTER                          │
│                                                                   │
│  You are about to permanently delete:                            │
│                                                                   │
│  Name: Hero                                                       │
│  Level: 25                                                        │
│  Prestige: Gold III (Rank 5)                                      │
│  Playtime: 12h 34m                                               │
│                                                                   │
│  This action CANNOT be undone.                                   │
│  All progress and equipment will be lost.                        │
│                                                                   │
│  Type the character name to confirm:                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ _                                                            │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  Must type exactly: Hero                                         │
│                                                                   │
│  [Enter] Delete    [Esc] Cancel                                  │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

**Confirmation Flow:**
1. User presses 'D' on character in select screen
2. Show delete confirmation screen with character details
3. User must type exact character name (case-sensitive)
4. Name must match exactly (no extra spaces, correct case)
5. On match: Delete .json file from filesystem
6. Return to character select screen
7. If last character deleted: Go to character creation screen

**Safety Features:**
- Must type exact name (case-sensitive)
- Shows character details as reminder of what's being lost
- Clear warning about permanent deletion
- Easy to cancel with Esc

### Rename Screen

**Layout:**

```
┌─ QUEST - Rename Character ───────────────────────────────────────┐
│                                                                   │
│  Renaming: Hero (Level 25)                                       │
│                                                                   │
│  New Name:                                                        │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ Hero the Great_                                              │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  • 1-16 characters                                               │
│  • Must be unique                                                │
│                                                                   │
│  ✓ Name is valid                                                 │
│                                                                   │
│  [Enter] Rename    [Esc] Cancel                                  │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

**Rename Flow:**
1. User presses 'R' on character in select screen
2. Show rename screen with current name pre-filled in input
3. User edits name (same validation as creation)
4. Validate new name (unique, length, characters)
5. On success:
   - Update `character_name` field in GameState
   - Sanitize new name for filename
   - Rename .json file: `hero.json` → `hero_the_great.json`
   - Save GameState to new file
   - Delete old file
   - Return to character select
6. File rename is atomic where possible

**Validation:**
- Same rules as character creation
- Must not conflict with other existing characters
- Shows real-time validation feedback

## Data Structures

### GameState Additions

```rust
pub struct GameState {
    // NEW FIELDS
    pub character_id: String,      // UUID, survives renames
    pub character_name: String,    // Display name (user-provided)

    // EXISTING FIELDS
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub combat_state: CombatState,
    pub equipment: Equipment,
}
```

### CharacterManager

```rust
pub struct CharacterManager {
    quest_dir: PathBuf,  // ~/.quest/
}

impl CharacterManager {
    // Initialization
    pub fn new() -> io::Result<Self>

    // Character operations
    pub fn list_characters() -> io::Result<Vec<CharacterInfo>>
    pub fn load_character(filename: &str) -> io::Result<GameState>
    pub fn save_character(state: &GameState) -> io::Result<()>
    pub fn create_character(name: String) -> io::Result<GameState>
    pub fn delete_character(filename: &str) -> io::Result<()>
    pub fn rename_character(old_filename: &str, new_name: String) -> io::Result<()>

    // Name operations
    pub fn validate_name(name: &str) -> Result<(), String>
    pub fn sanitize_name(name: &str) -> String
    pub fn check_name_unique(name: &str, except: Option<&str>) -> bool

    // Filesystem operations
    fn compute_checksum(state: &GameState) -> String
    fn verify_checksum(json: &str) -> Result<GameState, io::Error>
}
```

### CharacterInfo

```rust
pub struct CharacterInfo {
    pub character_id: String,
    pub character_name: String,
    pub filename: String,           // e.g., "hero.json"
    pub character_level: u32,
    pub prestige_rank: u32,
    pub play_time_seconds: u64,
    pub last_save_time: i64,
    pub attributes: Attributes,
    pub equipment: Equipment,
    pub is_corrupted: bool,        // Checksum failed
}
```

## Implementation Flow

### Character Listing

```rust
pub fn list_characters() -> io::Result<Vec<CharacterInfo>> {
    // 1. Scan ~/.quest/*.json files
    // 2. For each file:
    //    - Load JSON
    //    - Verify checksum
    //    - Extract metadata (name, level, prestige, etc.)
    //    - If checksum fails, mark as corrupted
    // 3. Sort by last_played (most recent first)
    // 4. Return Vec<CharacterInfo>
}
```

### Character Creation

```rust
pub fn create_character(name: String) -> io::Result<GameState> {
    // 1. Validate name (length, characters, uniqueness)
    // 2. Check character count (max 3)
    // 3. Generate UUID for character_id
    // 4. Create new GameState with name and ID
    // 5. Sanitize name for filename
    // 6. Compute checksum
    // 7. Serialize to JSON
    // 8. Write to ~/.quest/{sanitized_name}.json
    // 9. Return GameState
}
```

### Character Deletion

```rust
pub fn delete_character(filename: &str) -> io::Result<()> {
    // 1. Verify filename exists in ~/.quest/
    // 2. Delete file from filesystem
    // 3. Return success/error
}
```

### Character Rename

```rust
pub fn rename_character(old_filename: &str, new_name: String) -> io::Result<()> {
    // 1. Validate new name (length, characters, uniqueness)
    // 2. Load GameState from old_filename
    // 3. Update character_name field
    // 4. Sanitize new name for filename
    // 5. Compute new checksum
    // 6. Write to new filename
    // 7. Delete old file
    // 8. Return success/error
}
```

### Name Sanitization

```rust
pub fn sanitize_name(name: &str) -> String {
    // 1. Trim leading/trailing whitespace
    // 2. Convert to lowercase
    // 3. Replace spaces with underscores
    // 4. Remove any characters not in [a-z0-9_-]
    // 5. If empty after sanitization, use UUID fallback
    // 6. Return sanitized name

    // Examples:
    // "Hero" → "hero"
    // "Mage the Great" → "mage_the_great"
    // "Warrior-2" → "warrior-2"
    // "Test!!!" → "test"
    // "!!!" → "{uuid}" (fallback)
}
```

### Name Validation

```rust
pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    // Check length
    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if trimmed.len() > 16 {
        return Err("Name must be 16 characters or less".to_string());
    }

    // Check characters
    let valid_chars = trimmed.chars().all(|c| {
        c.is_alphanumeric() || c == ' ' || c == '-' || c == '_'
    });
    if !valid_chars {
        return Err("Name can only contain letters, numbers, spaces, hyphens, and underscores".to_string());
    }

    // Check uniqueness (done separately by caller)

    Ok(())
}
```

## Error Handling & Edge Cases

### Corrupted Save Files

**Checksum failure:**
- Load file, verify checksum
- If checksum fails: Mark character as `⚠️ [CORRUPTED]` in list
- Show warning in character select
- Offer options: "Delete" or "Keep" (doesn't count toward limit)
- If user selects corrupted character: Show error, refuse to load

**Parse failure:**
- If JSON parsing fails: Mark as corrupted
- Same handling as checksum failure
- Offer to delete file

### Filesystem Issues

**~/.quest/ doesn't exist:**
- Create directory automatically on first run
- Set appropriate permissions (user read/write only)

**Permission errors:**
- Show clear error message: "Cannot access ~/.quest/ - check permissions"
- Exit gracefully (don't crash)

**Disk full during save:**
- Show error: "Cannot save - disk full"
- Don't delete old file (keep most recent good save)
- Retry on next autosave

**Invalid filename characters:**
- Sanitize aggressively (remove all invalid chars)
- If name becomes empty after sanitization: Use UUID as filename
- Example: "!!!" → `{uuid}.json`

### Race Conditions

**Multiple game instances:**
- Not prevented (rare edge case)
- Both instances can load same character
- Last write wins (acceptable risk for single-player game)
- Future enhancement: Lock files if needed

**File being written during scan:**
- Skip file if locked/inaccessible
- Retry on next character list refresh
- Show warning if file repeatedly fails to load

### Migration from Old System

**Old save file detected:**
- On first launch, check for old save at:
  - macOS: `~/Library/Application Support/idle-rpg/save.dat`
  - Linux: `~/.config/idle-rpg/save.dat`
  - Windows: `%APPDATA%\idle-rpg\save.dat`
- If found:
  - Offer to import as "Imported Character"
  - Load old GameState
  - Add character_name = "Imported Character"
  - Generate character_id (UUID)
  - Convert to JSON format with checksum
  - Save to `~/.quest/imported_character.json`
  - Leave old save in place (don't delete)
- Show success message with details
- Offer to rename imported character immediately

### Character Name Conflicts

**Duplicate names:**
- "Hero" and "hero" are considered duplicates (case-insensitive check)
- "Mage the Great" and "mage_the_great" conflict (same sanitized name)
- Validation rejects duplicates during creation/rename
- Show error: "A character with this name already exists"

**Sanitization conflicts:**
- "Test!!!" and "Test" both sanitize to "test"
- Check sanitized names for conflicts
- If conflict detected after sanitization: Append number
  - "test" → `test_2.json`
  - "test_2" → `test_3.json`

**Rename to same name:**
- If renaming character to exact same name: Allow (no-op)
- File doesn't change, just update last_save_time

### File Operation Safety

**Character save (autosave in game):**
1. Write to temp file: `~/.quest/.{name}.tmp`
2. Compute checksum
3. Verify file is valid
4. Atomic rename: `.{name}.tmp` → `{name}.json`
5. If rename fails: Keep old file, show error

**Character creation:**
1. Check for existing file first
2. Write to new file
3. Verify checksum
4. If write fails: Delete partial file, show error

**Character deletion:**
1. Verify file exists
2. Delete file
3. If delete fails: Show error, keep file

**Character rename:**
1. Load old file
2. Update character_name
3. Write to new filename
4. Verify new file
5. Delete old file (only after new file verified)
6. If any step fails: Rollback, keep old file

## Testing Strategy

### Unit Tests

**CharacterManager:**
- Name validation (valid, too short, too long, invalid chars)
- Name sanitization (spaces, special chars, empty)
- Checksum calculation and verification
- Character creation with all valid names
- Character deletion
- Character rename (same name, new name, conflicts)
- List characters (0, 1, 2, 3 characters)
- Character limit enforcement (can't create 4th)

**File operations:**
- Create character saves to correct path
- Load character reads correct data
- Rename updates filename and character_name
- Delete removes file
- Corrupted files marked correctly

### Integration Tests

**End-to-end flows:**
- Create 3 characters → List shows 3
- Create character → Load → Save → Load again (data persists)
- Rename character → File renamed, data intact
- Delete character → File removed, count decreases
- Import old save → New format created correctly

**UI flows:**
- No characters → Creation screen shown
- Has characters → Select screen shown
- Create at limit → Button disabled
- Delete last character → Creation screen shown
- Rename with conflict → Error shown

### Edge Cases to Test

**Filesystem:**
- ~/.quest/ doesn't exist (auto-create)
- No permission to write (show error)
- Disk full (show error, keep old file)
- File locked by another process (show error)

**Names:**
- Single character: "A"
- Maximum length: "1234567890123456"
- Special chars: "Test!!!" → "test.json"
- Unicode: "Héro" → sanitize to "hro" or reject
- Spaces: "   Test   " → "test.json"
- All invalid chars: "!!!" → UUID fallback

**Corruption:**
- Invalid JSON (parse error)
- Checksum mismatch (tampered file)
- Missing fields (old version)
- Wrong version number

**Migration:**
- Old save exists (import offered)
- Old save corrupted (show error)
- Old save missing character fields (add defaults)

## Success Criteria

1. ✅ Character creation works with name validation
2. ✅ Character select screen shows up to 3 characters
3. ✅ Detailed preview panel shows level, prestige, playtime, attributes, equipment
4. ✅ Delete requires typing exact name to confirm
5. ✅ Rename updates filename and character_name
6. ✅ Create button disabled when 3 characters exist
7. ✅ Checksum protects against tampering
8. ✅ JSON files are portable (copy file = copy character)
9. ✅ Old save migration works correctly
10. ✅ Corrupted files handled gracefully
11. ✅ All characters save to ~/.quest/
12. ✅ Smart detection: no chars → creation, has chars → select
13. ✅ Navigation works with both arrow keys and vim keys
14. ✅ Character limit enforced (max 3)
15. ✅ All file operations are safe (atomic where possible)

## Files to Create/Modify

### New Files
- `src/character_manager.rs` - Character CRUD operations
- `src/ui/character_select.rs` - Character select screen
- `src/ui/character_creation.rs` - Character creation screen
- `src/ui/character_delete.rs` - Delete confirmation screen
- `src/ui/character_rename.rs` - Rename screen

### Modified Files
- `src/main.rs` - Add character select flow before game loop
- `src/game_state.rs` - Add character_name and character_id fields
- `src/save_manager.rs` - Update to use CharacterManager, support JSON format
- `src/ui/mod.rs` - Export new UI modules
- `CLAUDE.md` - Document character system architecture

## Migration Path

### From Current System

**Current state:**
- Single save file at platform-specific location
- Binary format with bincode + SHA256 checksum
- No character name or ID

**Migration steps:**
1. On first launch with new code:
   - Check for old save at old location
   - If found, offer import
2. Import process:
   - Load old GameState (binary format)
   - Add character_name = "Imported Character"
   - Generate character_id (UUID)
   - Convert to JSON with checksum
   - Save to `~/.quest/imported_character.json`
   - Leave old save in place (user can delete manually)
3. User can rename imported character immediately
4. All new characters use new system

**Backward compatibility:**
- Old save remains usable until imported
- No automatic deletion (user decides)
- Import can be done multiple times (creates duplicates with different names)

## Future Enhancements

**Not in scope for v1, but possible later:**
- Character export/import (share characters)
- Character comparison view (side-by-side stats)
- Character achievements/milestones
- Character notes/descriptions
- Character portraits/icons
- Cloud save sync
- Increase character limit to 5 or 10
- Character class/build templates
- Lock files to prevent multi-instance corruption
- Undo delete (trash/recycle bin integration)
