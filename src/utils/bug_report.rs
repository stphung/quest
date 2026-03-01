//! Bug report generation: builds a compact game state summary, constructs a
//! pre-filled GitHub issue URL, copies full save data to clipboard, and opens
//! the default browser.

use crate::achievements::Achievements;
use crate::core::game_state::GameState;
use crate::enhancement::EnhancementProgress;
use crate::haven::Haven;
use crate::zones::get_zone;

/// Data produced by `generate_bug_report`.
pub struct BugReportData {
    /// Compact game-state summary shown in the overlay and embedded in the issue body.
    pub summary: String,
    /// Markdown-formatted save data for pasting into the issue (clipboard content).
    pub clipboard_content: String,
}

/// Build all report data from the current game state.
pub fn generate_bug_report(
    state: &GameState,
    haven: &Haven,
    enhancement: &EnhancementProgress,
    achievements: &Achievements,
) -> BugReportData {
    let summary = build_summary(state, haven, enhancement, achievements);
    let clipboard_content = build_clipboard_content(state, haven, enhancement, achievements);
    BugReportData {
        summary,
        clipboard_content,
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

fn build_summary(
    state: &GameState,
    haven: &Haven,
    enhancement: &EnhancementProgress,
    achievements: &Achievements,
) -> String {
    let zone_name = get_zone(state.zone_progression.current_zone_id)
        .map(|z| z.name)
        .unwrap_or("Unknown");

    let equip_summary = equipment_one_liner(&state.equipment);

    let haven_status = if haven.discovered {
        let built: usize = haven.rooms.values().filter(|&&t| t > 0).count();
        format!("Yes ({built} rooms)")
    } else {
        "No".to_string()
    };

    let soulforge_status = if enhancement.discovered {
        let max = enhancement.highest_level_reached;
        format!("Yes (max +{max})")
    } else {
        "No".to_string()
    };

    let play_time = format_play_time(state.play_time_seconds);

    let unlock_pct = achievements.unlock_percentage();

    format!(
        "Build: {} ({})\n\
         Character: {} | Lv {} | P{}\n\
         Zone: {} {}-{} | Boss: {}\n\
         Equipment: {}\n\
         Fishing Rank: {} | Fish: {}\n\
         Haven: {} | Soulforge: {}\n\
         Achievements: {:.0}% | Kills: {}\n\
         Play Time: {}",
        crate::utils::build_info::BUILD_COMMIT,
        crate::utils::build_info::BUILD_DATE,
        state.character_name,
        state.character_level,
        state.prestige_rank,
        zone_name,
        state.zone_progression.current_zone_id,
        state.zone_progression.current_subzone_id,
        state.zone_progression.fighting_boss,
        equip_summary,
        state.fishing.rank,
        state.fishing.total_fish_caught,
        haven_status,
        soulforge_status,
        unlock_pct,
        achievements.total_kills,
        play_time,
    )
}

fn equipment_one_liner(equip: &crate::items::Equipment) -> String {
    let slots = [
        ("W", &equip.weapon),
        ("A", &equip.armor),
        ("H", &equip.helmet),
        ("G", &equip.gloves),
        ("B", &equip.boots),
        ("Am", &equip.amulet),
        ("R", &equip.ring),
    ];
    let parts: Vec<String> = slots
        .iter()
        .map(|(label, item)| match item {
            Some(i) => format!("{}:{}", label, i.rarity.name()),
            None => format!("{}: -", label),
        })
        .collect();
    parts.join(" ")
}

fn format_play_time(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    format!("{h}h {m}m")
}

// ---------------------------------------------------------------------------
// Clipboard content — full save data wrapped in collapsible markdown blocks
// ---------------------------------------------------------------------------

fn build_clipboard_content(
    state: &GameState,
    haven: &Haven,
    enhancement: &EnhancementProgress,
    achievements: &Achievements,
) -> String {
    let mut out = String::with_capacity(4096);

    out.push_str("## Save Data\n\n");
    out.push_str("Paste this into the issue body (it will be collapsed).\n\n");

    append_details_block(
        &mut out,
        "Character Save",
        &serde_json::to_string_pretty(state).unwrap_or_else(|e| format!("Error: {e}")),
    );
    append_details_block(
        &mut out,
        "Haven",
        &serde_json::to_string_pretty(haven).unwrap_or_else(|e| format!("Error: {e}")),
    );
    append_details_block(
        &mut out,
        "Enhancement",
        &serde_json::to_string_pretty(enhancement).unwrap_or_else(|e| format!("Error: {e}")),
    );
    append_details_block(
        &mut out,
        "Achievements",
        &serde_json::to_string_pretty(achievements).unwrap_or_else(|e| format!("Error: {e}")),
    );

    out
}

fn append_details_block(out: &mut String, title: &str, json: &str) {
    out.push_str("<details><summary>");
    out.push_str(title);
    out.push_str("</summary>\n\n```json\n");
    out.push_str(json);
    out.push_str("\n```\n</details>\n\n");
}

// ---------------------------------------------------------------------------
// Issue URL
// ---------------------------------------------------------------------------

/// Construct a GitHub new-issue URL with the summary pre-filled in the body.
pub fn build_issue_url(summary: &str) -> String {
    let title = url_encode("Bug Report");
    let body = url_encode(&format!(
        "## Bug Description\n\n\
         <!-- Describe what happened and what you expected -->\n\n\
         ## Game State\n\n\
         ```\n{summary}\n```\n\n\
         ## Save Data\n\n\
         <!-- Paste clipboard contents below this line -->\n\n"
    ));
    format!("https://github.com/stphung/quest/issues/new?title={title}&body={body}")
}

// ---------------------------------------------------------------------------
// Platform helpers
// ---------------------------------------------------------------------------

/// Open `url` in the default browser. Uses platform-specific commands.
pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .or_else(|_| std::process::Command::new("wslview").arg(url).spawn());

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let result: Result<std::process::Child, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported platform",
    ));

    result.map(|_| ()).map_err(|e| e.to_string())
}

/// Copy `text` to the system clipboard. Uses platform-specific commands.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut child = std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    let mut child = std::process::Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .or_else(|_| {
            std::process::Command::new("clip.exe")
                .stdin(std::process::Stdio::piped())
                .spawn()
        })
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    let mut child = std::process::Command::new("clip")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err("unsupported platform".to_string());

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        child.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// URL encoding (RFC 3986)
// ---------------------------------------------------------------------------

/// Percent-encode a string per RFC 3986 (unreserved chars pass through).
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(HEX_UPPER[(byte >> 4) as usize] as char);
                out.push(HEX_UPPER[(byte & 0x0F) as usize] as char);
            }
        }
    }
    out
}

const HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_ascii() {
        assert_eq!(url_encode("hello"), "hello");
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_url_encode_unicode() {
        let encoded = url_encode("\u{2694}"); // crossed swords
        assert!(!encoded.contains('\u{2694}'));
        assert!(encoded.starts_with('%'));
    }

    #[test]
    fn test_url_encode_unreserved_passthrough() {
        assert_eq!(url_encode("A-Z_a-z.0~9"), "A-Z_a-z.0~9");
    }

    #[test]
    fn test_summary_size_bounds() {
        let state = GameState::new("TestHero".to_string(), 0);
        let haven = Haven::default();
        let enhancement = EnhancementProgress::new();
        let achievements = Achievements::default();

        let summary = build_summary(&state, &haven, &enhancement, &achievements);
        // Summary should be compact — well under 2KB
        assert!(
            summary.len() < 2048,
            "Summary too large: {} bytes",
            summary.len()
        );
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_summary_contains_key_fields() {
        let state = GameState::new("MyHero".to_string(), 0);
        let haven = Haven::default();
        let enhancement = EnhancementProgress::new();
        let achievements = Achievements::default();

        let summary = build_summary(&state, &haven, &enhancement, &achievements);
        assert!(summary.contains("MyHero"), "Missing character name");
        assert!(summary.contains("Lv 1"), "Missing level");
        assert!(summary.contains("P0"), "Missing prestige");
        assert!(summary.contains("Meadow"), "Missing zone name");
        assert!(summary.contains("Build:"), "Missing build info");
        assert!(summary.contains("Fishing Rank:"), "Missing fishing");
        assert!(summary.contains("Haven:"), "Missing haven");
        assert!(summary.contains("Soulforge:"), "Missing soulforge");
    }

    #[test]
    fn test_issue_url_size_bounds() {
        let state = GameState::new("TestHero".to_string(), 0);
        let haven = Haven::default();
        let enhancement = EnhancementProgress::new();
        let achievements = Achievements::default();

        let summary = build_summary(&state, &haven, &enhancement, &achievements);
        let url = build_issue_url(&summary);
        // URL should fit within browser limits — well under 8KB
        assert!(url.len() < 8192, "URL too large: {} bytes", url.len());
        assert!(url.starts_with("https://github.com/stphung/quest/issues/new"));
    }

    #[test]
    fn test_clipboard_content_has_json_sections() {
        let state = GameState::new("TestHero".to_string(), 0);
        let haven = Haven::default();
        let enhancement = EnhancementProgress::new();
        let achievements = Achievements::default();

        let content = build_clipboard_content(&state, &haven, &enhancement, &achievements);
        assert!(
            content.contains("Character Save"),
            "Missing character section"
        );
        assert!(content.contains("Haven"), "Missing haven section");
        assert!(
            content.contains("Enhancement"),
            "Missing enhancement section"
        );
        assert!(
            content.contains("Achievements"),
            "Missing achievements section"
        );
        assert!(content.contains("```json"), "Missing JSON code blocks");
    }

    #[test]
    fn test_format_play_time() {
        assert_eq!(format_play_time(0), "0h 0m");
        assert_eq!(format_play_time(3661), "1h 1m");
        assert_eq!(format_play_time(7200), "2h 0m");
        assert_eq!(format_play_time(86400), "24h 0m");
    }

    #[test]
    fn test_equipment_one_liner_empty() {
        let equip = crate::items::Equipment::new();
        let s = equipment_one_liner(&equip);
        assert!(s.contains("W: -"));
        assert!(s.contains("R: -"));
    }
}
