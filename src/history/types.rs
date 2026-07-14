//! Core types for the Time Vault save versioning system.
//!
//! `SaveEvent` describes *why* a commit was made (milestone, action, or manual).
//! `CommitInfo` and `TimelineInfo` carry read-back metadata for the Time Vault UI.

// ── SaveEvent ────────────────────────────────────────────────────────────

/// Describes the reason a save-state commit was created.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SaveEvent {
    // Milestone progression
    LevelUp(u32),
    PrestigeRank(u32),
    ZoneBossDefeated(String),
    ZoneUnlocked(String),
    DungeonCompleted(String),
    FishingRankUp(u32),
    StormLeviathanCaught,
    AchievementUnlocked(String),
    // Act 2 (the Vessel) milestones
    VesselLaunched,
    VesselArrived,
    LastCrossing,

    // State-changing actions
    HavenRoomBuilt(String),
    HavenRoomUpgraded(String, u8),
    SoulforgeEnhanced(String, u8),
    SoulforgeFailed(String, u8),
    ChallengeWon(String, String),
    GodItemForged(String),
    CharacterCreated(String),
    CharacterDeleted(String),
    EquipmentUpgrade(String),
    StormSigilActivated(String),

    // Chrono Surge
    ChronoSurge {
        levels_gained: u32,
        kills: u64,
        ticks: u64,
    },

    // Manual / automatic
    ManualSave,
    AutoSave,
}

impl SaveEvent {
    /// Human-readable description of this event, used as the first part of
    /// a git commit message.
    pub fn description(&self) -> String {
        match self {
            SaveEvent::LevelUp(lvl) => format!("Level up to {lvl}"),
            SaveEvent::PrestigeRank(rank) => format!("Prestige to rank {rank}"),
            SaveEvent::ZoneBossDefeated(zone) => format!("Defeated {zone} boss"),
            SaveEvent::ZoneUnlocked(zone) => format!("Unlocked {zone}"),
            SaveEvent::DungeonCompleted(size) => format!("Completed {size} dungeon"),
            SaveEvent::FishingRankUp(rank) => format!("Fishing rank up to {rank}"),
            SaveEvent::StormLeviathanCaught => "Caught the Storm Leviathan".to_string(),
            SaveEvent::AchievementUnlocked(name) => format!("Achievement: {name}"),
            SaveEvent::VesselLaunched => {
                "The Vessel launches — 250,000 Prestige Ranks burn".to_string()
            }
            SaveEvent::VesselArrived => "The Vessel reaches the Tree".to_string(),
            SaveEvent::LastCrossing => "The Last Crossing — the old world is empty".to_string(),
            SaveEvent::HavenRoomBuilt(room) => format!("Built {room} in Haven"),
            SaveEvent::HavenRoomUpgraded(room, tier) => format!("Upgraded {room} to T{tier}"),
            SaveEvent::SoulforgeEnhanced(slot, level) => {
                format!("Enhanced {slot} to +{level}")
            }
            SaveEvent::SoulforgeFailed(slot, level) => {
                format!("Enhancement failed on {slot} (dropped to +{level})")
            }
            SaveEvent::ChallengeWon(name, diff) => format!("Won {name} at {diff}"),
            SaveEvent::GodItemForged(name) => format!("Forged {name}"),
            SaveEvent::CharacterCreated(name) => format!("Created character {name}"),
            SaveEvent::CharacterDeleted(name) => format!("Deleted character {name}"),
            SaveEvent::EquipmentUpgrade(name) => format!("Equipped {name}"),
            SaveEvent::StormSigilActivated(name) => {
                format!("Activated Storm Sigil: {name}")
            }
            SaveEvent::ChronoSurge {
                levels_gained,
                kills,
                ticks,
            } => {
                let total_seconds = ticks / 10;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let duration = if hours > 0 {
                    format!("{hours}h{minutes:02}m")
                } else {
                    format!("{minutes}m")
                };
                format!("Chrono Surge {duration} (+{levels_gained} levels, {kills} kills)")
            }
            SaveEvent::ManualSave => "Manual save".to_string(),
            SaveEvent::AutoSave => "Auto-save".to_string(),
        }
    }

    /// Build a full commit message: "{description} | {suffix}".
    ///
    /// The suffix encodes snapshot metadata so it can be parsed back later
    /// without reading the save file.
    pub fn commit_message(
        &self,
        level: u32,
        prestige: u32,
        zone_id: u32,
        subzone_id: u32,
        play_time_seconds: u64,
        character_name: &str,
    ) -> String {
        let suffix = format_suffix(
            level,
            prestige,
            zone_id,
            subzone_id,
            play_time_seconds,
            character_name,
        );
        format!("{} | {}", self.description(), suffix)
    }
}

// ── Suffix formatting ────────────────────────────────────────────────────

/// Format the snapshot suffix: "Lv{level} P{prestige} Z{zone}-{subzone} {time} @{name}".
///
/// Time is formatted as `{h}h{mm}m` (e.g. "2h15m", "45h10m").
pub fn format_suffix(
    level: u32,
    prestige: u32,
    zone_id: u32,
    subzone_id: u32,
    play_time_seconds: u64,
    character_name: &str,
) -> String {
    let total_minutes = play_time_seconds / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("Lv{level} P{prestige} Z{zone_id}-{subzone_id} {hours}h{minutes:02}m @{character_name}")
}

// ── CommitMetadata ───────────────────────────────────────────────────────

/// Snapshot metadata passed to `HistoryRepo::commit()`.
///
/// Bundles all per-commit fields so the caller doesn't need to pass
/// individual positional arguments and the signature stays stable as
/// new fields are added.
#[derive(Debug, Clone)]
pub struct CommitMetadata {
    /// Character level at the time of the save.
    pub level: u32,
    /// Prestige rank at the time of the save.
    pub prestige: u32,
    /// Current zone id at the time of the save.
    pub zone_id: u32,
    /// Current subzone id at the time of the save.
    pub subzone_id: u32,
    /// Total play time in seconds at the time of the save.
    pub play_time_seconds: u64,
    /// Character name at the time of the save.
    pub character_name: String,
}

// ── CommitInfo ───────────────────────────────────────────────────────────

/// Metadata extracted from a single history commit.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Short hex commit id (first 7 characters of SHA).
    pub id: String,
    /// Full commit message.
    pub message: String,
    /// Unix timestamp of the commit.
    pub timestamp: i64,
    /// Character level at time of commit.
    pub level: u32,
    /// Prestige rank at time of commit.
    pub prestige: u32,
    /// Zone id at time of commit.
    pub zone: u32,
    /// Total play time in seconds at time of commit.
    pub playtime: u64,
}

// ── TimelineInfo ─────────────────────────────────────────────────────────

/// Summary of a single timeline (git branch).
#[derive(Debug, Clone)]
pub struct TimelineInfo {
    /// Branch / timeline display name.
    pub name: String,
    /// Whether this is the currently checked-out branch.
    pub is_active: bool,
    /// Most recent commit on this branch, if any.
    pub head_commit: Option<CommitInfo>,
}

#[cfg(test)]
mod tests {
    use super::SaveEvent;

    #[test]
    fn vessel_save_events_describe_their_moments() {
        assert_eq!(
            SaveEvent::VesselLaunched.description(),
            "The Vessel launches — 250,000 Prestige Ranks burn"
        );
        assert_eq!(
            SaveEvent::VesselArrived.description(),
            "The Vessel reaches the Tree"
        );
        assert_eq!(
            SaveEvent::LastCrossing.description(),
            "The Last Crossing — the old world is empty"
        );
        // The commit-message suffix machinery composes with them unchanged.
        let msg = SaveEvent::VesselLaunched.commit_message(80, 250_000, 50, 5, 3_600, "Ferry");
        assert!(msg.starts_with("The Vessel launches"));
        assert!(msg.contains("@Ferry"));
    }
}
