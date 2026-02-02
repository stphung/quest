use std::fs;
use std::io;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct CharacterManager {
    quest_dir: PathBuf,
}

#[allow(dead_code)]
impl CharacterManager {
    pub fn new() -> io::Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine home directory",
            )
        })?;

        let quest_dir = home_dir.join(".quest");
        fs::create_dir_all(&quest_dir)?;

        Ok(Self { quest_dir })
    }
}

#[allow(dead_code)]
pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if trimmed.len() > 16 {
        return Err("Name must be 16 characters or less".to_string());
    }

    let valid_chars = trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_');

    if !valid_chars {
        return Err(
            "Name can only contain letters, numbers, spaces, hyphens, and underscores".to_string(),
        );
    }

    Ok(())
}

#[allow(dead_code)]
pub fn sanitize_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

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

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Hero"), "hero");
        assert_eq!(sanitize_name("Mage the Great"), "mage_the_great");
        assert_eq!(sanitize_name("Warrior-2"), "warrior-2");
        assert_eq!(sanitize_name("Test!!!"), "test");
        assert_eq!(sanitize_name("   Spaces   "), "spaces");
        assert_eq!(sanitize_name("MixedCase"), "mixedcase");
    }

    #[test]
    fn test_character_manager_new() {
        let manager = CharacterManager::new().expect("Failed to create CharacterManager");
        assert!(manager.quest_dir.ends_with(".quest"));
        assert!(manager.quest_dir.exists());
    }
}
