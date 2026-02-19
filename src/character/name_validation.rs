//! Character name validation and sanitization.

use crate::core::constants::CHARACTER_NAME_MAX_LENGTH;

/// Reserved names that cannot be used for characters (would conflict with system files)
pub const RESERVED_NAMES: &[&str] = &["haven", "achievements"];

pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if trimmed.len() > CHARACTER_NAME_MAX_LENGTH {
        return Err(format!(
            "Name must be {} characters or less",
            CHARACTER_NAME_MAX_LENGTH
        ));
    }

    let valid_chars = trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_');

    if !valid_chars {
        return Err(
            "Name can only contain letters, numbers, spaces, hyphens, and underscores".to_string(),
        );
    }

    // Check for reserved names (case-insensitive, matches sanitized filename)
    let sanitized = sanitize_name(trimmed);
    if RESERVED_NAMES.contains(&sanitized.as_str()) {
        return Err(format!("Name '{}' is reserved", trimmed));
    }

    Ok(())
}

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
    fn test_sanitize_name_special_cases() {
        // Unicode alphanumeric characters are preserved
        assert_eq!(sanitize_name("Hérö"), "hérö");

        // Multiple spaces become underscores
        assert_eq!(sanitize_name("My   Hero"), "my___hero");

        // Empty after sanitization (only special chars)
        assert_eq!(sanitize_name("!!!"), "");

        // Numbers preserved
        assert_eq!(sanitize_name("Hero123"), "hero123");
    }

    #[test]
    fn test_validate_name_boundary_lengths() {
        // Exactly 16 characters should be valid
        assert!(validate_name("1234567890123456").is_ok());

        // 17 characters should fail
        assert!(validate_name("12345678901234567").is_err());

        // 1 character should be valid
        assert!(validate_name("A").is_ok());
    }

    // =========================================================================
    // CHARACTER NAME VALIDATION - EXTENDED EDGE CASES
    // =========================================================================

    #[test]
    fn test_validate_name_extended_invalid_chars() {
        // Various special characters that should be rejected
        assert!(validate_name("Name#1").is_err());
        assert!(validate_name("Hero$").is_err());
        assert!(validate_name("Test%").is_err());
        assert!(validate_name("Name&Name").is_err());
        assert!(validate_name("Hero*").is_err());
        assert!(validate_name("<script>").is_err());
        assert!(validate_name("Name\nNewline").is_err());
        assert!(validate_name("Name\tTab").is_err());
        assert!(validate_name("test;drop").is_err());
        assert!(validate_name("name'quote").is_err());
        assert!(validate_name("name\"quote").is_err());
    }

    #[test]
    fn test_validate_name_trims_whitespace() {
        // Leading/trailing whitespace should be trimmed, then validated
        assert!(validate_name("  Hero  ").is_ok());
        assert!(validate_name("\tHero\t").is_ok());
    }

    #[test]
    fn test_validate_name_unicode_letters() {
        // Unicode letters should work (alphanumeric includes unicode)
        assert!(validate_name("Héro").is_ok());
        assert!(validate_name("日本語").is_ok()); // Japanese
        assert!(validate_name("Müller").is_ok()); // German umlaut
        assert!(validate_name("Ωmega").is_ok()); // Greek
    }

    #[test]
    fn test_validate_name_reserved_names() {
        // "haven" is reserved (conflicts with haven.json)
        assert!(validate_name("haven").is_err());
        assert!(validate_name("Haven").is_err()); // Case-insensitive
        assert!(validate_name("HAVEN").is_err());
        assert!(validate_name("  haven  ").is_err()); // With whitespace

        // "achievements" is reserved (conflicts with achievements.json)
        assert!(validate_name("achievements").is_err());
        assert!(validate_name("Achievements").is_err()); // Case-insensitive
        assert!(validate_name("ACHIEVEMENTS").is_err());
        assert!(validate_name("  achievements  ").is_err()); // With whitespace

        // Similar names that don't conflict should be fine
        assert!(validate_name("haven2").is_ok());
        assert!(validate_name("myhaven").is_ok());
        assert!(validate_name("the-haven").is_ok());
        assert!(validate_name("achievements2").is_ok());
        assert!(validate_name("myachievements").is_ok());
    }
}
