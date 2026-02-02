//! Compile-time build information.

include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_not_empty() {
        assert!(!BUILD_COMMIT.is_empty());
        assert!(!BUILD_DATE.is_empty());
    }

    #[test]
    fn test_build_commit_format() {
        // Should be 7 chars or "unknown"
        assert!(BUILD_COMMIT == "unknown" || BUILD_COMMIT.len() == 7);
    }

    #[test]
    fn test_build_date_format() {
        // Should be YYYY-MM-DD format
        assert!(BUILD_DATE.len() == 10 || BUILD_DATE == "unknown");
    }
}
