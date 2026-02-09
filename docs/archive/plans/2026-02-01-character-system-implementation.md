# Character System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement character management with JSON saves in ~/.quest/, character select/creation screens, and full character lifecycle (create, delete, rename).

**Architecture:** CharacterManager handles file operations and metadata, new UI screens for character select/creation/rename/delete, GameState gains character_name and character_id fields, migration from old binary format to new JSON format.

**Tech Stack:** Rust, Ratatui 0.26, Serde JSON, SHA2 (checksums), UUID (character IDs), Crossterm (input handling)

---

## Task 1: Add character_id and character_name to GameState

**Files:**
- Modify: `src/game_state.rs`
- Test: Unit tests in `src/game_state.rs`

**Step 1: Add uuid dependency**

Add to `Cargo.toml`:
```toml
uuid = { version = "1.6", features = ["v4", "serde"] }
```

Run: `cargo build`
Expected: Dependency added successfully

**Step 2: Add fields to GameState**

In `src/game_state.rs`, add after line 7 (`pub struct GameState {`):
```rust
pub character_id: String,
pub character_name: String,
```

**Step 3: Update GameState::new() to accept name**

Change signature from:
```rust
pub fn new(current_time: i64) -> Self {
```

To:
```rust
pub fn new(character_name: String, current_time: i64) -> Self {
```

Add character fields to initialization:
```rust
use uuid::Uuid;

Self {
    character_id: Uuid::new_v4().to_string(),
    character_name,
    character_level: 1,
    // ... rest of fields
}
```

**Step 4: Run tests to see what breaks**

Run: `cargo test`
Expected: Compilation errors where `GameState::new()` is called

**Step 5: Fix compilation errors**

Update calls in:
- `src/main.rs`: `GameState::new("New Character".to_string(), Utc::now().timestamp())`
- `src/save_manager.rs` (migrate_old_save): `character_id: Uuid::new_v4().to_string(), character_name: "Imported Character".to_string(),`
- Any test files that call `GameState::new()`

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add Cargo.toml src/game_state.rs src/main.rs src/save_manager.rs
git commit -m "feat(game_state): add character_id and character_name fields

- Add uuid dependency for unique character IDs
- Update GameState::new() to accept character_name
- Generate UUID on character creation
- Fix all call sites

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Create CharacterManager with name validation

**Files:**
- Create: `src/character_manager.rs`
- Modify: `src/main.rs` (add module declaration)

**Step 1: Write test for name validation**

Create `src/character_manager.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("Hero").is_ok());
        assert!(validate_name("Test 123").is_ok());
        assert!(validate_name("Warrior-2").is_ok());
        assert!(validate_name("under_score").is_ok());
    }

    #[test]
    fn test_validate_name_too_short() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        assert!(validate_name("12345678901234567").is_err()); // 17 chars
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(validate_name("test@123").is_err());
        assert!(validate_name("hello!world").is_err());
    }
}

pub fn validate_name(name: &str) -> Result<(), String> {
    todo!()
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test validate_name`
Expected: FAIL with "not yet implemented"

**Step 3: Implement name validation**

```rust
pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if trimmed.len() > 16 {
        return Err("Name must be 16 characters or less".to_string());
    }

    let valid_chars = trimmed.chars().all(|c| {
        c.is_alphanumeric() || c == ' ' || c == '-' || c == '_'
    });

    if !valid_chars {
        return Err("Name can only contain letters, numbers, spaces, hyphens, and underscores".to_string());
    }

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test validate_name`
Expected: PASS (all 4 tests)

**Step 5: Add module declaration**

In `src/main.rs`, after line 15 (`mod save_manager;`):
```rust
mod character_manager;
```

**Step 6: Commit**

```bash
git add src/character_manager.rs src/main.rs
git commit -m "feat(character_manager): add name validation

- Validate length (1-16 chars after trim)
- Validate characters (alphanumeric, space, hyphen, underscore)
- Return descriptive error messages

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Add name sanitization for filenames

**Files:**
- Modify: `src/character_manager.rs`

**Step 1: Write test for name sanitization**

Add to test module in `src/character_manager.rs`:
```rust
#[test]
fn test_sanitize_name() {
    assert_eq!(sanitize_name("Hero"), "hero");
    assert_eq!(sanitize_name("Mage the Great"), "mage_the_great");
    assert_eq!(sanitize_name("Warrior-2"), "warrior-2");
    assert_eq!(sanitize_name("Test!!!"), "test");
    assert_eq!(sanitize_name("   Spaces   "), "spaces");
    assert_eq!(sanitize_name("MixedCase"), "mixedcase");
}
```

Add function signature:
```rust
pub fn sanitize_name(name: &str) -> String {
    todo!()
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test sanitize_name`
Expected: FAIL with "not yet implemented"

**Step 3: Implement sanitization**

```rust
pub fn sanitize_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test sanitize_name`
Expected: PASS

**Step 5: Commit**

```bash
git add src/character_manager.rs
git commit -m "feat(character_manager): add name sanitization for filenames

- Convert to lowercase
- Replace spaces with underscores
- Remove invalid filename characters
- Trim whitespace

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create CharacterManager struct with quest_dir

**Files:**
- Modify: `src/character_manager.rs`

**Step 1: Write test for CharacterManager::new()**

Add to test module:
```rust
#[test]
fn test_character_manager_new() {
    let manager = CharacterManager::new().expect("Failed to create CharacterManager");
    assert!(manager.quest_dir.ends_with(".quest"));
    assert!(manager.quest_dir.exists());
}
```

Add struct and impl:
```rust
use std::path::PathBuf;
use std::io;
use std::fs;

pub struct CharacterManager {
    quest_dir: PathBuf,
}

impl CharacterManager {
    pub fn new() -> io::Result<Self> {
        todo!()
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test character_manager_new`
Expected: FAIL with "not yet implemented"

**Step 3: Implement CharacterManager::new()**

```rust
impl CharacterManager {
    pub fn new() -> io::Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not determine home directory")
        })?;

        let quest_dir = home_dir.join(".quest");
        fs::create_dir_all(&quest_dir)?;

        Ok(Self { quest_dir })
    }
}
```

Add `dirs` dependency to `Cargo.toml`:
```toml
dirs = "5.0"
```

**Step 4: Run test to verify it passes**

Run: `cargo test character_manager_new`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/character_manager.rs
git commit -m "feat(character_manager): create CharacterManager with ~/.quest directory

- Add dirs dependency for home directory
- Create ~/.quest/ on initialization
- Store quest_dir path in struct

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Implement JSON serialization with checksum

**Files:**
- Modify: `src/character_manager.rs`

**Step 1: Add dependencies**

Add to `Cargo.toml`:
```toml
serde_json = "1.0"
```

Note: `sha2` and `serde` already in dependencies from save_manager

**Step 2: Write test for save_character**

Add to test module:
```rust
use crate::game_state::GameState;
use crate::attributes::Attributes;
use crate::combat::CombatState;
use crate::equipment::Equipment;
use chrono::Utc;

#[test]
fn test_save_and_load_character() {
    let manager = CharacterManager::new().unwrap();

    let state = GameState {
        character_id: "test-id".to_string(),
        character_name: "TestHero".to_string(),
        character_level: 10,
        character_xp: 5000,
        attributes: Attributes::new(),
        prestige_rank: 2,
        total_prestige_count: 2,
        last_save_time: Utc::now().timestamp(),
        play_time_seconds: 3600,
        combat_state: CombatState::new(100),
        equipment: Equipment::new(),
    };

    // Save character
    manager.save_character(&state).expect("Failed to save");

    // Verify file exists
    let filename = format!("{}.json", sanitize_name(&state.character_name));
    let filepath = manager.quest_dir.join(&filename);
    assert!(filepath.exists());

    // Load character
    let loaded = manager.load_character(&filename).expect("Failed to load");
    assert_eq!(loaded.character_name, "TestHero");
    assert_eq!(loaded.character_level, 10);

    // Cleanup
    fs::remove_file(filepath).ok();
}
```

Add function signatures:
```rust
impl CharacterManager {
    pub fn save_character(&self, state: &GameState) -> io::Result<()> {
        todo!()
    }

    pub fn load_character(&self, filename: &str) -> io::Result<GameState> {
        todo!()
    }
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test save_and_load_character`
Expected: FAIL with "not yet implemented"

**Step 4: Implement save_character with checksum**

```rust
use serde_json;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CharacterSaveData {
    version: u32,
    character_id: String,
    character_name: String,
    character_level: u32,
    character_xp: u64,
    attributes: crate::attributes::Attributes,
    prestige_rank: u32,
    total_prestige_count: u64,
    last_save_time: i64,
    play_time_seconds: u64,
    combat_state: crate::combat::CombatState,
    equipment: crate::equipment::Equipment,
    checksum: String,
}

impl CharacterManager {
    pub fn save_character(&self, state: &crate::game_state::GameState) -> io::Result<()> {
        // Create save data without checksum
        let mut save_data = CharacterSaveData {
            version: 2,
            character_id: state.character_id.clone(),
            character_name: state.character_name.clone(),
            character_level: state.character_level,
            character_xp: state.character_xp,
            attributes: state.attributes.clone(),
            prestige_rank: state.prestige_rank,
            total_prestige_count: state.total_prestige_count,
            last_save_time: state.last_save_time,
            play_time_seconds: state.play_time_seconds,
            combat_state: state.combat_state.clone(),
            equipment: state.equipment.clone(),
            checksum: String::new(),
        };

        // Serialize without checksum to compute hash
        let json_without_checksum = serde_json::to_string_pretty(&save_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Compute checksum
        let mut hasher = Sha256::new();
        hasher.update(json_without_checksum.as_bytes());
        let checksum = format!("{:x}", hasher.finalize());

        // Add checksum and serialize final version
        save_data.checksum = checksum;
        let json_with_checksum = serde_json::to_string_pretty(&save_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write to file
        let filename = format!("{}.json", sanitize_name(&state.character_name));
        let filepath = self.quest_dir.join(filename);
        fs::write(filepath, json_with_checksum)?;

        Ok(())
    }
}
```

**Step 5: Implement load_character with checksum verification**

```rust
impl CharacterManager {
    pub fn load_character(&self, filename: &str) -> io::Result<crate::game_state::GameState> {
        let filepath = self.quest_dir.join(filename);
        let json_content = fs::read_to_string(filepath)?;

        // Parse JSON
        let save_data: CharacterSaveData = serde_json::from_str(&json_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Store checksum, then zero it out for verification
        let stored_checksum = save_data.checksum.clone();
        let mut save_data_for_check = save_data.clone();
        save_data_for_check.checksum = String::new();

        // Recompute checksum
        let json_without_checksum = serde_json::to_string_pretty(&save_data_for_check)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut hasher = Sha256::new();
        hasher.update(json_without_checksum.as_bytes());
        let computed_checksum = format!("{:x}", hasher.finalize());

        // Verify checksum
        if stored_checksum != computed_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum verification failed - file may be corrupted or tampered",
            ));
        }

        // Convert to GameState
        Ok(crate::game_state::GameState {
            character_id: save_data.character_id,
            character_name: save_data.character_name,
            character_level: save_data.character_level,
            character_xp: save_data.character_xp,
            attributes: save_data.attributes,
            prestige_rank: save_data.prestige_rank,
            total_prestige_count: save_data.total_prestige_count,
            last_save_time: save_data.last_save_time,
            play_time_seconds: save_data.play_time_seconds,
            combat_state: save_data.combat_state,
            equipment: save_data.equipment,
        })
    }
}
```

Add Clone derive to CharacterSaveData:
```rust
#[derive(Clone, Serialize, Deserialize)]
```

**Step 6: Run test to verify it passes**

Run: `cargo test save_and_load_character`
Expected: PASS

**Step 7: Commit**

```bash
git add Cargo.toml src/character_manager.rs
git commit -m "feat(character_manager): add JSON save/load with checksum verification

- Serialize GameState to JSON format
- Compute SHA256 checksum of data
- Verify checksum on load (detect tampering)
- Save to ~/.quest/{sanitized_name}.json

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Implement list_characters

**Files:**
- Modify: `src/character_manager.rs`

**Step 1: Write test for list_characters**

Add to test module:
```rust
#[test]
fn test_list_characters() {
    let manager = CharacterManager::new().unwrap();

    // Create test characters
    let char1 = GameState {
        character_id: "id1".to_string(),
        character_name: "Hero".to_string(),
        character_level: 10,
        character_xp: 5000,
        attributes: Attributes::new(),
        prestige_rank: 2,
        total_prestige_count: 2,
        last_save_time: 1000,
        play_time_seconds: 3600,
        combat_state: CombatState::new(100),
        equipment: Equipment::new(),
    };

    let char2 = GameState {
        character_id: "id2".to_string(),
        character_name: "Warrior".to_string(),
        character_level: 15,
        character_xp: 8000,
        attributes: Attributes::new(),
        prestige_rank: 3,
        total_prestige_count: 3,
        last_save_time: 2000,
        play_time_seconds: 7200,
        combat_state: CombatState::new(100),
        equipment: Equipment::new(),
    };

    manager.save_character(&char1).unwrap();
    manager.save_character(&char2).unwrap();

    // List characters
    let list = manager.list_characters().expect("Failed to list");
    assert_eq!(list.len(), 2);

    // Verify sorted by last_played (most recent first)
    assert_eq!(list[0].character_name, "Warrior"); // last_save_time = 2000
    assert_eq!(list[1].character_name, "Hero");    // last_save_time = 1000

    // Cleanup
    fs::remove_file(manager.quest_dir.join("hero.json")).ok();
    fs::remove_file(manager.quest_dir.join("warrior.json")).ok();
}
```

Add CharacterInfo struct and function signature:
```rust
#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub character_id: String,
    pub character_name: String,
    pub filename: String,
    pub character_level: u32,
    pub prestige_rank: u32,
    pub play_time_seconds: u64,
    pub last_save_time: i64,
    pub attributes: crate::attributes::Attributes,
    pub equipment: crate::equipment::Equipment,
    pub is_corrupted: bool,
}

impl CharacterManager {
    pub fn list_characters(&self) -> io::Result<Vec<CharacterInfo>> {
        todo!()
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test list_characters`
Expected: FAIL with "not yet implemented"

**Step 3: Implement list_characters**

```rust
impl CharacterManager {
    pub fn list_characters(&self) -> io::Result<Vec<CharacterInfo>> {
        let mut characters = Vec::new();

        // Read directory entries
        let entries = fs::read_dir(&self.quest_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let filename = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Try to load character
            match self.load_character(&filename) {
                Ok(state) => {
                    characters.push(CharacterInfo {
                        character_id: state.character_id,
                        character_name: state.character_name,
                        filename,
                        character_level: state.character_level,
                        prestige_rank: state.prestige_rank,
                        play_time_seconds: state.play_time_seconds,
                        last_save_time: state.last_save_time,
                        attributes: state.attributes,
                        equipment: state.equipment,
                        is_corrupted: false,
                    });
                }
                Err(_) => {
                    // Mark as corrupted but include in list
                    characters.push(CharacterInfo {
                        character_id: String::new(),
                        character_name: "[CORRUPTED]".to_string(),
                        filename,
                        character_level: 0,
                        prestige_rank: 0,
                        play_time_seconds: 0,
                        last_save_time: 0,
                        attributes: crate::attributes::Attributes::new(),
                        equipment: crate::equipment::Equipment::new(),
                        is_corrupted: true,
                    });
                }
            }
        }

        // Sort by last_save_time (most recent first)
        characters.sort_by(|a, b| b.last_save_time.cmp(&a.last_save_time));

        Ok(characters)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test list_characters`
Expected: PASS

**Step 5: Commit**

```bash
git add src/character_manager.rs
git commit -m "feat(character_manager): add list_characters

- Scan ~/.quest/*.json files
- Load each character file
- Handle corrupted files gracefully
- Sort by last_played (most recent first)
- Return CharacterInfo vec

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Implement delete_character and rename_character

**Files:**
- Modify: `src/character_manager.rs`

**Step 1: Write tests for delete and rename**

Add to test module:
```rust
#[test]
fn test_delete_character() {
    let manager = CharacterManager::new().unwrap();

    let state = GameState {
        character_id: "test-id".to_string(),
        character_name: "ToDelete".to_string(),
        character_level: 5,
        character_xp: 1000,
        attributes: Attributes::new(),
        prestige_rank: 0,
        total_prestige_count: 0,
        last_save_time: Utc::now().timestamp(),
        play_time_seconds: 100,
        combat_state: CombatState::new(50),
        equipment: Equipment::new(),
    };

    manager.save_character(&state).unwrap();

    let filename = "todelete.json";
    assert!(manager.quest_dir.join(filename).exists());

    manager.delete_character(filename).expect("Delete failed");
    assert!(!manager.quest_dir.join(filename).exists());
}

#[test]
fn test_rename_character() {
    let manager = CharacterManager::new().unwrap();

    let state = GameState {
        character_id: "test-id".to_string(),
        character_name: "OldName".to_string(),
        character_level: 8,
        character_xp: 3000,
        attributes: Attributes::new(),
        prestige_rank: 1,
        total_prestige_count: 1,
        last_save_time: Utc::now().timestamp(),
        play_time_seconds: 500,
        combat_state: CombatState::new(75),
        equipment: Equipment::new(),
    };

    manager.save_character(&state).unwrap();

    manager.rename_character("oldname.json", "NewName".to_string())
        .expect("Rename failed");

    // Old file should not exist
    assert!(!manager.quest_dir.join("oldname.json").exists());

    // New file should exist
    assert!(manager.quest_dir.join("newname.json").exists());

    // Load and verify name updated
    let loaded = manager.load_character("newname.json").unwrap();
    assert_eq!(loaded.character_name, "NewName");

    // Cleanup
    fs::remove_file(manager.quest_dir.join("newname.json")).ok();
}
```

Add function signatures:
```rust
impl CharacterManager {
    pub fn delete_character(&self, filename: &str) -> io::Result<()> {
        todo!()
    }

    pub fn rename_character(&self, old_filename: &str, new_name: String) -> io::Result<()> {
        todo!()
    }
}
```

**Step 2: Run test to verify they fail**

Run: `cargo test delete_character rename_character`
Expected: FAIL with "not yet implemented"

**Step 3: Implement delete_character**

```rust
impl CharacterManager {
    pub fn delete_character(&self, filename: &str) -> io::Result<()> {
        let filepath = self.quest_dir.join(filename);
        fs::remove_file(filepath)?;
        Ok(())
    }
}
```

**Step 4: Implement rename_character**

```rust
impl CharacterManager {
    pub fn rename_character(&self, old_filename: &str, new_name: String) -> io::Result<()> {
        // Validate new name
        validate_name(&new_name)?;

        // Load existing character
        let mut state = self.load_character(old_filename)?;

        // Update character name
        state.character_name = new_name.clone();

        // Save with new name
        self.save_character(&state)?;

        // Delete old file
        let old_filepath = self.quest_dir.join(old_filename);
        fs::remove_file(old_filepath)?;

        Ok(())
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test delete_character rename_character`
Expected: PASS (both tests)

**Step 6: Commit**

```bash
git add src/character_manager.rs
git commit -m "feat(character_manager): add delete and rename operations

- delete_character: Remove character file
- rename_character: Update name, save to new file, delete old
- Validate new name before renaming

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Create CharacterCreation UI screen

**Files:**
- Create: `src/ui/character_creation.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create file and add module declaration**

Create `src/ui/character_creation.rs`:
```rust
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CharacterCreationScreen {
    pub name_input: String,
    pub cursor_position: usize,
    pub validation_error: Option<String>,
}

impl CharacterCreationScreen {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            cursor_position: 0,
            validation_error: None,
        }
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // TODO: Implement
    }

    pub fn handle_char_input(&mut self, c: char) {
        self.name_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.validate();
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.name_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.validate();
        }
    }

    pub fn validate(&mut self) {
        self.validation_error = match crate::character_manager::validate_name(&self.name_input) {
            Ok(_) => None,
            Err(e) => Some(e),
        };
    }

    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none() && !self.name_input.trim().is_empty()
    }

    pub fn get_name(&self) -> String {
        self.name_input.trim().to_string()
    }
}
```

In `src/ui/mod.rs`, add after line 5:
```rust
pub mod character_creation;
```

**Step 2: Implement draw method**

In `src/ui/character_creation.rs`, replace `draw` method:
```rust
pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(1),  // Spacer
            Constraint::Length(3),  // Input label + field
            Constraint::Length(1),  // Spacer
            Constraint::Length(4),  // Rules
            Constraint::Length(2),  // Validation
            Constraint::Min(0),     // Filler
            Constraint::Length(3),  // Controls
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Create Your Hero")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Input label
    let label = Paragraph::new("Character Name:");
    f.render_widget(label, chunks[2]);

    // Input field with cursor
    let input_area = Rect {
        x: chunks[2].x,
        y: chunks[2].y + 1,
        width: chunks[2].width,
        height: 1,
    };

    let input_text = if self.cursor_position < self.name_input.len() {
        format!(
            "{}{}{}",
            &self.name_input[..self.cursor_position],
            "_",
            &self.name_input[self.cursor_position..]
        )
    } else {
        format!("{}_", self.name_input)
    };

    let input_widget = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(input_widget, input_area);

    // Rules
    let rules = vec![
        Line::from("• 1-16 characters"),
        Line::from("• Letters, numbers, spaces, hyphens, underscores"),
        Line::from("• Must be unique"),
    ];
    let rules_widget = Paragraph::new(rules).style(Style::default().fg(Color::Gray));
    f.render_widget(rules_widget, chunks[4]);

    // Validation feedback
    let validation_text = if let Some(error) = &self.validation_error {
        Line::from(Span::styled(
            format!("✗ {}", error),
            Style::default().fg(Color::Red),
        ))
    } else if !self.name_input.trim().is_empty() {
        Line::from(Span::styled(
            "✓ Name is valid",
            Style::default().fg(Color::Green),
        ))
    } else {
        Line::from("")
    };
    let validation_widget = Paragraph::new(validation_text);
    f.render_widget(validation_widget, chunks[5]);

    // Controls
    let controls = Paragraph::new("[Enter] Create Character    [Esc] Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(controls, chunks[7]);
}
```

**Step 3: Run build to check for errors**

Run: `cargo build`
Expected: SUCCESS (compiles cleanly)

**Step 4: Commit**

```bash
git add src/ui/character_creation.rs src/ui/mod.rs
git commit -m "feat(ui): add character creation screen

- Input field with cursor
- Real-time name validation
- Visual feedback (green checkmark, red error)
- Character input and backspace handling
- Rules display

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Create CharacterSelect UI screen

**Files:**
- Create: `src/ui/character_select.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create file and basic structure**

Create `src/ui/character_select.rs`:
```rust
use crate::character_manager::CharacterInfo;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct CharacterSelectScreen {
    pub selected_index: usize,
}

impl CharacterSelectScreen {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }

    pub fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        characters: &[CharacterInfo],
    ) {
        // TODO: Implement
    }

    pub fn move_up(&mut self, count: usize) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    pub fn move_down(&mut self, count: usize) {
        self.selected_index = (self.selected_index + 1).min(count.saturating_sub(1));
    }

    pub fn get_selected_character<'a>(
        &self,
        characters: &'a [CharacterInfo],
    ) -> Option<&'a CharacterInfo> {
        characters.get(self.selected_index)
    }
}
```

In `src/ui/mod.rs`, add after character_creation:
```rust
pub mod character_select;
```

**Step 2: Implement draw method**

Replace `draw` method in `src/ui/character_select.rs`:
```rust
pub fn draw<B: Backend>(
    &self,
    f: &mut Frame<B>,
    area: Rect,
    characters: &[CharacterInfo],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Controls
        ])
        .split(area);

    // Title
    let title = Paragraph::new("QUEST - Character Select")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content: character list + details panel
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Character list (left panel)
    self.draw_character_list(f, main_chunks[0], characters);

    // Character details (right panel)
    if let Some(selected) = self.get_selected_character(characters) {
        self.draw_character_details(f, main_chunks[1], selected);
    }

    // Controls
    let can_create = characters.len() < 3;
    let new_control = if can_create {
        "[N] New"
    } else {
        "New (Max 3)"
    };

    let controls = Paragraph::new(format!(
        "[Enter] Play  [R] Rename  [D] Delete  {}  [Q] Quit",
        new_control
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[2]);
}

fn draw_character_list<B: Backend>(
    &self,
    f: &mut Frame<B>,
    area: Rect,
    characters: &[CharacterInfo],
) {
    let items: Vec<ListItem> = characters
        .iter()
        .enumerate()
        .map(|(i, char_info)| {
            let prestige_name = crate::prestige::get_prestige_name(char_info.prestige_rank);
            let content = vec![
                Line::from(Span::styled(
                    &char_info.character_name,
                    Style::default()
                        .fg(if i == self.selected_index {
                            Color::Yellow
                        } else {
                            Color::White
                        })
                        .add_modifier(if i == self.selected_index {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                )),
                Line::from(format!("  Level {}", char_info.character_level)),
                Line::from(format!("  Prestige: {}", prestige_name)),
            ];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Characters"))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn draw_character_details<B: Backend>(
    &self,
    f: &mut Frame<B>,
    area: Rect,
    char_info: &CharacterInfo,
) {
    let prestige_name = crate::prestige::get_prestige_name(char_info.prestige_rank);
    let hours = char_info.play_time_seconds / 3600;
    let minutes = (char_info.play_time_seconds % 3600) / 60;

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Name: {}", char_info.character_name),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Level: {}", char_info.character_level)),
        Line::from(format!(
            "Prestige: {} (Rank {})",
            prestige_name, char_info.prestige_rank
        )),
        Line::from(format!("Playtime: {}h {}m", hours, minutes)),
        Line::from(""),
        Line::from("Attributes:"),
        Line::from(format!(
            "  STR {}  DEX {}  CON {}",
            char_info.attributes.str, char_info.attributes.dex, char_info.attributes.con
        )),
        Line::from(format!(
            "  INT {}  WIS {}  CHA {}",
            char_info.attributes.int, char_info.attributes.wis, char_info.attributes.cha
        )),
        Line::from(""),
    ];

    // Equipment summary
    let equipped_count = [
        &char_info.equipment.weapon,
        &char_info.equipment.armor,
        &char_info.equipment.helmet,
        &char_info.equipment.gloves,
        &char_info.equipment.boots,
        &char_info.equipment.amulet,
        &char_info.equipment.ring,
    ]
    .iter()
    .filter(|item| item.is_some())
    .count();

    lines.push(Line::from(format!(
        "Equipment: {}/7 slots filled",
        equipped_count
    )));

    // Show equipped items
    if let Some(weapon) = &char_info.equipment.weapon {
        lines.push(Line::from(format!("  ⚔️  {}", weapon.display_name)));
    }
    if let Some(armor) = &char_info.equipment.armor {
        lines.push(Line::from(format!("  🛡  {}", armor.display_name)));
    }
    if let Some(helmet) = &char_info.equipment.helmet {
        lines.push(Line::from(format!("  🪖  {}", helmet.display_name)));
    }
    if let Some(gloves) = &char_info.equipment.gloves {
        lines.push(Line::from(format!("  🧤  {}", gloves.display_name)));
    }
    if let Some(boots) = &char_info.equipment.boots {
        lines.push(Line::from(format!("  👢  {}", boots.display_name)));
    }
    if let Some(amulet) = &char_info.equipment.amulet {
        lines.push(Line::from(format!("  📿  {}", amulet.display_name)));
    }
    if let Some(ring) = &char_info.equipment.ring {
        lines.push(Line::from(format!("  💍  {}", ring.display_name)));
    }

    let details = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Character Details"))
        .style(Style::default().fg(Color::White));

    f.render_widget(details, area);
}
```

**Step 3: Run build to check for errors**

Run: `cargo build`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add src/ui/character_select.rs src/ui/mod.rs
git commit -m "feat(ui): add character select screen

- Character list with selection cursor
- Detailed preview panel (level, prestige, playtime, attributes, equipment)
- Navigation controls display
- Up/down movement support

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Create delete confirmation screen

**Files:**
- Create: `src/ui/character_delete.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create file with basic structure**

Create `src/ui/character_delete.rs`:
```rust
use crate::character_manager::CharacterInfo;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CharacterDeleteScreen {
    pub confirmation_input: String,
    pub cursor_position: usize,
}

impl CharacterDeleteScreen {
    pub fn new() -> Self {
        Self {
            confirmation_input: String::new(),
            cursor_position: 0,
        }
    }

    pub fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        character: &CharacterInfo,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(3)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(7),  // Character info
                Constraint::Length(3),  // Warning
                Constraint::Length(3),  // Input
                Constraint::Length(2),  // Instructions
                Constraint::Min(0),     // Filler
                Constraint::Length(3),  // Controls
            ])
            .split(area);

        // Title
        let title = Paragraph::new("⚠️  DELETE CHARACTER")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Character info
        let prestige_name = crate::prestige::get_prestige_name(character.prestige_rank);
        let hours = character.play_time_seconds / 3600;
        let minutes = (character.play_time_seconds % 3600) / 60;

        let info = vec![
            Line::from("You are about to permanently delete:"),
            Line::from(""),
            Line::from(format!("Name: {}", character.character_name)),
            Line::from(format!("Level: {}", character.character_level)),
            Line::from(format!("Prestige: {} (Rank {})", prestige_name, character.prestige_rank)),
            Line::from(format!("Playtime: {}h {}m", hours, minutes)),
        ];
        let info_widget = Paragraph::new(info).style(Style::default().fg(Color::White));
        f.render_widget(info_widget, chunks[1]);

        // Warning
        let warning = vec![
            Line::from(Span::styled(
                "This action CANNOT be undone.",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from("All progress and equipment will be lost."),
        ];
        let warning_widget = Paragraph::new(warning);
        f.render_widget(warning_widget, chunks[2]);

        // Input prompt
        let prompt = Paragraph::new("Type the character name to confirm:");
        f.render_widget(prompt, chunks[3]);

        // Input field
        let input_area = Rect {
            x: chunks[3].x,
            y: chunks[3].y + 1,
            width: chunks[3].width,
            height: 1,
        };

        let input_text = if self.cursor_position < self.confirmation_input.len() {
            format!(
                "{}{}{}",
                &self.confirmation_input[..self.cursor_position],
                "_",
                &self.confirmation_input[self.cursor_position..]
            )
        } else {
            format!("{}_", self.confirmation_input)
        };

        let input_widget = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        f.render_widget(input_widget, input_area);

        // Required text
        let required = Paragraph::new(format!("Must type exactly: {}", character.character_name))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(required, chunks[4]);

        // Controls
        let controls = Paragraph::new("[Enter] Delete    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[6]);
    }

    pub fn handle_char_input(&mut self, c: char) {
        self.confirmation_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.confirmation_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn is_confirmed(&self, character_name: &str) -> bool {
        self.confirmation_input == character_name
    }

    pub fn reset(&mut self) {
        self.confirmation_input.clear();
        self.cursor_position = 0;
    }
}
```

In `src/ui/mod.rs`, add:
```rust
pub mod character_delete;
```

**Step 2: Run build**

Run: `cargo build`
Expected: SUCCESS

**Step 3: Commit**

```bash
git add src/ui/character_delete.rs src/ui/mod.rs
git commit -m "feat(ui): add delete confirmation screen

- Show character details before deletion
- Require typing exact name to confirm
- Clear warning about permanent deletion
- Input handling for confirmation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Create rename screen

**Files:**
- Create: `src/ui/character_rename.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create file**

Create `src/ui/character_rename.rs`:
```rust
use crate::character_manager::CharacterInfo;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CharacterRenameScreen {
    pub name_input: String,
    pub cursor_position: usize,
    pub validation_error: Option<String>,
}

impl CharacterRenameScreen {
    pub fn new(current_name: &str) -> Self {
        Self {
            name_input: current_name.to_string(),
            cursor_position: current_name.len(),
            validation_error: None,
        }
    }

    pub fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        character: &CharacterInfo,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(3)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(3),  // New name input
                Constraint::Length(3),  // Rules
                Constraint::Length(2),  // Validation
                Constraint::Min(0),     // Filler
                Constraint::Length(3),  // Controls
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!(
            "Renaming: {} (Level {})",
            character.character_name, character.character_level
        ))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Input label
        let label = Paragraph::new("New Name:");
        f.render_widget(label, chunks[1]);

        // Input field
        let input_area = Rect {
            x: chunks[1].x,
            y: chunks[1].y + 1,
            width: chunks[1].width,
            height: 1,
        };

        let input_text = if self.cursor_position < self.name_input.len() {
            format!(
                "{}{}{}",
                &self.name_input[..self.cursor_position],
                "_",
                &self.name_input[self.cursor_position..]
            )
        } else {
            format!("{}_", self.name_input)
        };

        let input_widget = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        f.render_widget(input_widget, input_area);

        // Rules
        let rules = vec![
            Line::from("• 1-16 characters"),
            Line::from("• Must be unique"),
        ];
        let rules_widget = Paragraph::new(rules).style(Style::default().fg(Color::Gray));
        f.render_widget(rules_widget, chunks[2]);

        // Validation
        let validation_text = if let Some(error) = &self.validation_error {
            Line::from(Span::styled(
                format!("✗ {}", error),
                Style::default().fg(Color::Red),
            ))
        } else if !self.name_input.trim().is_empty() {
            Line::from(Span::styled(
                "✓ Name is valid",
                Style::default().fg(Color::Green),
            ))
        } else {
            Line::from("")
        };
        let validation_widget = Paragraph::new(validation_text);
        f.render_widget(validation_widget, chunks[3]);

        // Controls
        let controls = Paragraph::new("[Enter] Rename    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[5]);
    }

    pub fn handle_char_input(&mut self, c: char) {
        self.name_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.validate();
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.name_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.validate();
        }
    }

    pub fn validate(&mut self) {
        self.validation_error = match crate::character_manager::validate_name(&self.name_input) {
            Ok(_) => None,
            Err(e) => Some(e),
        };
    }

    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none() && !self.name_input.trim().is_empty()
    }

    pub fn get_name(&self) -> String {
        self.name_input.trim().to_string()
    }
}
```

In `src/ui/mod.rs`, add:
```rust
pub mod character_rename;
```

**Step 2: Run build**

Run: `cargo build`
Expected: SUCCESS

**Step 3: Commit**

```bash
git add src/ui/character_rename.rs src/ui/mod.rs
git commit -m "feat(ui): add character rename screen

- Pre-fill with current name
- Real-time validation
- Show character level in title
- Input handling with cursor

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Wire up character system in main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Add screen state enum**

After imports in `src/main.rs`, add:
```rust
use character_manager::{CharacterManager, CharacterInfo};
use ui::character_creation::CharacterCreationScreen;
use ui::character_select::CharacterSelectScreen;
use ui::character_delete::CharacterDeleteScreen;
use ui::character_rename::CharacterRenameScreen;

enum Screen {
    CharacterSelect,
    CharacterCreation,
    CharacterDelete,
    CharacterRename,
    Game,
}
```

**Step 2: Replace main() startup logic**

Replace the current save loading logic (lines 34-47) with:
```rust
fn main() -> io::Result<()> {
    // Initialize CharacterManager
    let character_manager = CharacterManager::new()?;

    // List existing characters
    let characters = character_manager.list_characters()?;

    // Determine initial screen
    let mut current_screen = if characters.is_empty() {
        Screen::CharacterCreation
    } else {
        Screen::CharacterSelect
    };

    // Screen state
    let mut creation_screen = CharacterCreationScreen::new();
    let mut select_screen = CharacterSelectScreen::new();
    let mut delete_screen = CharacterDeleteScreen::new();
    let mut rename_screen: Option<CharacterRenameScreen> = None;
    let mut characters_list = characters;

    // Game state (loaded when character selected)
    let mut game_state: Option<GameState> = None;

    // TODO: Main loop will go here

    Ok(())
}
```

**Step 3: Run build to verify**

Run: `cargo build`
Expected: Compilation errors about unused variables (expected)

**Step 4: Add main loop skeleton**

After the variable declarations, add:
```rust
// Setup terminal
enable_raw_mode()?;
let mut stdout = io::stdout();
stdout.execute(EnterAlternateScreen)?;
let backend = CrosstermBackend::new(stdout);
let mut terminal = Terminal::new(backend)?;

// Main loop
loop {
    match current_screen {
        Screen::CharacterCreation => {
            terminal.draw(|f| {
                creation_screen.draw(f, f.size());
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => creation_screen.handle_char_input(c),
                        KeyCode::Backspace => creation_screen.handle_backspace(),
                        KeyCode::Enter => {
                            if creation_screen.is_valid() {
                                let name = creation_screen.get_name();
                                game_state = Some(GameState::new(name, Utc::now().timestamp()));
                                current_screen = Screen::Game;
                            }
                        }
                        KeyCode::Esc => {
                            if !characters_list.is_empty() {
                                current_screen = Screen::CharacterSelect;
                            } else {
                                break; // Quit if no characters
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Screen::CharacterSelect => {
            terminal.draw(|f| {
                select_screen.draw(f, f.size(), &characters_list);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            select_screen.move_up(characters_list.len());
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            select_screen.move_down(characters_list.len());
                        }
                        KeyCode::Enter => {
                            if let Some(char_info) = select_screen.get_selected_character(&characters_list) {
                                let loaded = character_manager.load_character(&char_info.filename)?;
                                game_state = Some(loaded);
                                current_screen = Screen::Game;
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            if characters_list.len() < 3 {
                                creation_screen = CharacterCreationScreen::new();
                                current_screen = Screen::CharacterCreation;
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            delete_screen.reset();
                            current_screen = Screen::CharacterDelete;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            if let Some(char_info) = select_screen.get_selected_character(&characters_list) {
                                rename_screen = Some(CharacterRenameScreen::new(&char_info.character_name));
                                current_screen = Screen::CharacterRename;
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Screen::CharacterDelete => {
            if let Some(char_info) = select_screen.get_selected_character(&characters_list) {
                terminal.draw(|f| {
                    delete_screen.draw(f, f.size(), char_info);
                })?;

                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char(c) => delete_screen.handle_char_input(c),
                            KeyCode::Backspace => delete_screen.handle_backspace(),
                            KeyCode::Enter => {
                                if delete_screen.is_confirmed(&char_info.character_name) {
                                    character_manager.delete_character(&char_info.filename)?;
                                    characters_list = character_manager.list_characters()?;

                                    if characters_list.is_empty() {
                                        creation_screen = CharacterCreationScreen::new();
                                        current_screen = Screen::CharacterCreation;
                                    } else {
                                        select_screen.selected_index = 0;
                                        current_screen = Screen::CharacterSelect;
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                current_screen = Screen::CharacterSelect;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Screen::CharacterRename => {
            if let (Some(rename), Some(char_info)) = (
                rename_screen.as_mut(),
                select_screen.get_selected_character(&characters_list),
            ) {
                terminal.draw(|f| {
                    rename.draw(f, f.size(), char_info);
                })?;

                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char(c) => rename.handle_char_input(c),
                            KeyCode::Backspace => rename.handle_backspace(),
                            KeyCode::Enter => {
                                if rename.is_valid() {
                                    let new_name = rename.get_name();
                                    character_manager.rename_character(&char_info.filename, new_name)?;
                                    characters_list = character_manager.list_characters()?;
                                    current_screen = Screen::CharacterSelect;
                                }
                            }
                            KeyCode::Esc => {
                                current_screen = Screen::CharacterSelect;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Screen::Game => {
            // Existing game loop goes here
            // For now, just break to test character screens
            break;
        }
    }
}

// Cleanup terminal
disable_raw_mode()?;
terminal.backend_mut().execute(LeaveAlternateScreen)?;

Ok(())
```

**Step 5: Run to test character screens**

Run: `cargo run`
Expected: Character creation or select screen appears, navigation works

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): wire up character system screens

- Add Screen enum for state machine
- Implement character creation flow
- Implement character select with navigation
- Implement delete with confirmation
- Implement rename with validation
- Smart detection: no chars → creation, has chars → select

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Integrate game loop with character saves

**Files:**
- Modify: `src/main.rs`

**Step 1: Move existing game loop into Screen::Game branch**

Replace the `Screen::Game => { break; }` placeholder with the existing game loop code.

Find the old game loop starting after terminal setup (around line 67 in old code) through the end, and move it into the Game screen branch.

Update to use `game_state.unwrap()` at the start:
```rust
Screen::Game => {
    let mut game_state = game_state.take().expect("Game state not initialized");

    // Process offline progression
    let current_time = Utc::now().timestamp();
    let elapsed_seconds = current_time - game_state.last_save_time;

    // ... rest of existing game loop code
}
```

**Step 2: Update autosave to use CharacterManager**

Find the autosave logic (around line 105 in old code):
```rust
if last_save.elapsed() >= autosave_interval {
    save_manager.save(&game_state)?;
    last_save = Instant::now();
}
```

Replace with:
```rust
if last_save.elapsed() >= autosave_interval {
    character_manager.save_character(&game_state)?;
    last_save = Instant::now();
}
```

**Step 3: Handle return to character select on quit**

At the end of the game loop (where it breaks), add option to return to character select:

Replace the break at the end of game loop with:
```rust
// Save before exiting
character_manager.save_character(&game_state)?;

// Return to character select
characters_list = character_manager.list_characters()?;
current_screen = Screen::CharacterSelect;
```

**Step 4: Test full flow**

Run: `cargo run`
Expected:
1. Character creation screen appears
2. Create character, game starts
3. Game runs normally
4. Press Q to quit, returns to character select
5. Character appears in list with stats

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): integrate game loop with character saves

- Move game loop into Screen::Game branch
- Use CharacterManager for autosave
- Return to character select on quit
- Save character before returning to select

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Add migration from old save format

**Files:**
- Modify: `src/main.rs`

**Step 1: Add old save detection**

At the start of `main()`, after `CharacterManager::new()`, add:
```rust
// Check for old save file to migrate
let old_save_manager = SaveManager::new()?;
if old_save_manager.save_exists() {
    println!("Old save file detected. Importing as 'Imported Character'...");

    match old_save_manager.load() {
        Ok(old_state) => {
            // Save as new character
            character_manager.save_character(&old_state)?;
            println!("Import successful! Character available in character select.");
            println!("Old save file left at original location (you can delete it manually).");
        }
        Err(e) => {
            println!("Warning: Could not import old save: {}", e);
            println!("You can still create new characters.");
        }
    }
}
```

**Step 2: Test migration**

Create an old save file (or restore the backup from earlier):
```bash
cp ~/.quest/characters.dat.backup ~/Library/Application\ Support/idle-rpg/save.dat
```

Run: `cargo run`
Expected:
- "Old save file detected" message
- Import successful
- Character appears in select screen as "Imported Character"

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): add migration from old save format

- Detect old save file at platform location
- Import as 'Imported Character'
- Leave old save in place
- Handle import errors gracefully

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Run full test suite and update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`
- All code complete

**Step 1: Run all tests**

Run: `cargo test --quiet`
Expected: All tests pass

**Step 2: Run format and clippy**

Run: `cargo fmt && cargo clippy --quiet -- -D warnings`
Expected: No warnings

**Step 3: Update CLAUDE.md**

Add character system documentation after the Item System section:

```markdown
### Character System

- `character_manager.rs` — Character CRUD operations (create, delete, rename), JSON save/load with SHA256 checksums, name validation and sanitization
- `ui/character_select.rs` — Character selection screen with detailed preview panel
- `ui/character_creation.rs` — Character creation with real-time name validation
- `ui/character_delete.rs` — Delete confirmation requiring exact name typing
- `ui/character_rename.rs` — Character renaming with validation
```

Update save location in description:
```markdown
- `save_manager.rs` — Legacy binary save/load (deprecated, used for migration)
- `character_manager.rs` — JSON save/load in ~/.quest/ directory
```

**Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with character system

- Document CharacterManager and character UI screens
- Update save location (now ~/.quest/)
- Note legacy SaveManager used for migration

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

**Step 5: Final verification**

Run: `make check`
Expected: All CI checks pass

---

## Success Criteria

After completing all tasks:

1. ✅ Characters saved to `~/.quest/*.json` with checksums
2. ✅ Character creation with name validation (1-16 chars, alphanumeric + space/hyphen/underscore)
3. ✅ Character select shows up to 3 characters with details
4. ✅ Delete requires typing exact name
5. ✅ Rename updates name and filename
6. ✅ Smart detection: no chars → creation, has chars → select
7. ✅ Game integrates with character system (load/save)
8. ✅ Migration from old save format works
9. ✅ All tests pass
10. ✅ CLAUDE.md updated

## Execution Notes

- Each task is independent and can be tested
- Frequent commits with meaningful messages
- TDD approach: write tests first where applicable
- All new code passes clippy and rustfmt
- Character system fully functional before integration with game loop
