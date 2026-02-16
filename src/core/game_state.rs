use crate::challenges::chess::ChessStats;
use crate::challenges::menu::ChallengeMenu;
use crate::challenges::ActiveMinigame;
use crate::challenges::MinigameWinInfo;
use crate::character::attributes::Attributes;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::combat::types::CombatState;
use crate::dungeon::types::Dungeon;
use crate::fishing::types::{FishingSession, FishingState};
use crate::items::equipment::Equipment;
use crate::items::types::Rarity;
use crate::zones::ZoneProgression;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A recently gained item or fish for display in the Loot panel
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecentDrop {
    pub name: String,
    pub rarity: Rarity,
    pub equipped: bool,
    pub icon: &'static str,
    /// Equipment slot name (e.g. "Weapon", "Armor"), empty for non-equipment
    pub slot: String,
    /// Stat summary (e.g. "+8 STR +3 DEX +Crit"), empty for non-equipment
    pub stats: String,
}

/// Max number of recent drops to track
const MAX_RECENT_DROPS: usize = 10;

/// A single entry in the scrolling loot ticker.
#[derive(Debug, Clone)]
pub struct TickerEntry {
    /// Icon prefix (e.g., "\u{2694}" for sword, "\u{1F41F}" for fish)
    pub icon: &'static str,
    /// Pre-formatted display text (e.g., "[E] Shadowfang +8STR")
    pub text: String,
    /// Display color (rarity or event-type color)
    pub color: ratatui::style::Color,
    /// Whether to render bold
    pub bold: bool,
}

/// Internal entry with its scroll birth time for independent positioning.
#[derive(Debug, Clone)]
struct TimedEntry {
    entry: TickerEntry,
    /// The scroll_offset value when this entry was pushed.
    born_at: f64,
}

/// Scrolling loot ticker state. Transient (not serialized).
///
/// Each entry independently enters from the right edge of the viewport and
/// scrolls left. Speed adapts smoothly: when entries queue far ahead (debt),
/// scroll accelerates to keep the ticker responsive; when the queue drains,
/// it decelerates back to the base speed.
#[derive(Debug, Clone)]
pub struct LootTicker {
    entries: VecDeque<TimedEntry>,
    /// Fractional scroll offset (integer part = character position)
    pub scroll_offset: f64,
    /// Current scroll speed (chars per tick), smoothly interpolated
    current_speed: f64,
    /// Last known viewport width for cleanup calculations
    pub viewport_width: usize,
}

/// Max entries in the ticker before oldest are evicted
const TICKER_MAX_ENTRIES: usize = 30;

/// Base scroll speed in chars per tick (0.4 = ~4 chars/sec at 100ms ticks)
pub const TICKER_SCROLL_SPEED: f64 = 0.4;

/// Maximum scroll speed multiplier when catching up to queued entries
const TICKER_MAX_SPEED_MULT: f64 = 4.0;

/// How quickly the scroll speed adjusts toward the target (0.0–1.0).
/// Lower = smoother but slower to react. 0.08 gives ~1s ramp time.
const TICKER_SPEED_LERP: f64 = 0.08;

/// Gap (in chars) between consecutive entries on screen
const ENTRY_GAP: usize = 3;

impl LootTicker {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(TICKER_MAX_ENTRIES),
            scroll_offset: 0.0,
            current_speed: TICKER_SCROLL_SPEED,
            viewport_width: 80,
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Add a new entry to the ticker. Evicts oldest if at capacity.
    ///
    /// Each entry gets a `born_at` timestamp so it independently enters from
    /// the right edge. If entries arrive in rapid succession, they are spaced
    /// apart so they don't overlap on screen.
    pub fn push(&mut self, entry: TickerEntry) {
        // Space new entry after the previous one to avoid overlap
        let born_at = if let Some(prev) = self.entries.front() {
            let min_gap = Self::entry_char_len(&prev.entry) + ENTRY_GAP;
            self.scroll_offset.max(prev.born_at + min_gap as f64)
        } else {
            self.scroll_offset
        };

        if self.entries.len() >= TICKER_MAX_ENTRIES {
            self.entries.pop_back();
        }
        self.entries.push_front(TimedEntry { entry, born_at });
    }

    /// Advance the scroll offset by one tick. Call once per 100ms tick.
    ///
    /// Speed adapts smoothly based on "debt" — how far ahead the newest
    /// entry's born_at is from the current scroll position. When entries
    /// pile up faster than they scroll, speed ramps up gradually; when the
    /// queue drains, speed eases back to the base rate.
    pub fn tick(&mut self) {
        let target_speed = if let Some(newest) = self.entries.front() {
            let debt = (newest.born_at - self.scroll_offset).max(0.0);
            let half_vw = (self.viewport_width as f64 / 2.0).max(1.0);
            let multiplier = (1.0 + debt / half_vw).min(TICKER_MAX_SPEED_MULT);
            TICKER_SCROLL_SPEED * multiplier
        } else {
            TICKER_SCROLL_SPEED
        };

        // Smooth interpolation toward target speed
        self.current_speed += (target_speed - self.current_speed) * TICKER_SPEED_LERP;
        self.scroll_offset += self.current_speed;
        self.cleanup_scrolled_entries();
    }

    /// Returns entries with their viewport column position (oldest first).
    /// Position is the left-edge column of the entry in the viewport.
    pub fn visible_entries(&self, viewport_width: usize) -> Vec<(&TickerEntry, isize)> {
        let vw = viewport_width as f64;
        self.entries
            .iter()
            .rev()
            .filter_map(|te| {
                let pos = (vw - (self.scroll_offset - te.born_at)) as isize;
                let entry_len = Self::entry_char_len(&te.entry) as isize;
                // Visible if any part is on screen
                if pos + entry_len > 0 && pos < viewport_width as isize {
                    Some((&te.entry, pos))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Remove entries that have fully scrolled off the left edge.
    fn cleanup_scrolled_entries(&mut self) {
        while let Some(oldest) = self.entries.back() {
            let entry_chars = Self::entry_char_len(&oldest.entry);
            let age = self.scroll_offset - oldest.born_at;
            // Off-screen when it has scrolled past the viewport + its own width
            if age > (self.viewport_width + entry_chars) as f64 {
                self.entries.pop_back();
            } else {
                break;
            }
        }
    }

    fn entry_char_len(entry: &TickerEntry) -> usize {
        let icon_len = if entry.icon.is_empty() {
            0
        } else {
            entry.icon.chars().count() + 1 // icon + space
        };
        icon_len + entry.text.chars().count()
    }
}

impl Default for LootTicker {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Record a recent gain (item drop, fish catch, etc.)
    pub fn add_recent_drop(
        &mut self,
        name: String,
        rarity: Rarity,
        equipped: bool,
        icon: &'static str,
        slot: String,
        stats: String,
    ) {
        if self.recent_drops.len() >= MAX_RECENT_DROPS {
            self.recent_drops.pop_back();
        }
        self.recent_drops.push_front(RecentDrop {
            name,
            rarity,
            equipped,
            icon,
            slot,
            stats,
        });
    }
}

/// Main game state containing all player progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub combat_state: CombatState,
    pub equipment: Equipment,
    /// Active dungeon exploration (None when not in a dungeon)
    #[serde(default)]
    pub active_dungeon: Option<Dungeon>,
    /// Persistent fishing progression state
    #[serde(default)]
    pub fishing: FishingState,
    /// Active fishing session (transient, not saved)
    #[serde(skip)]
    #[allow(dead_code)]
    pub active_fishing: Option<FishingSession>,
    /// Zone progression state
    #[serde(default)]
    pub zone_progression: ZoneProgression,
    /// Generic challenge menu (transient, not saved)
    #[serde(skip)]
    pub challenge_menu: ChallengeMenu,
    /// Persistent chess stats (survives prestige, saved to disk)
    #[serde(default)]
    pub chess_stats: ChessStats,
    /// Active challenge minigame (transient, not saved)
    #[serde(skip)]
    pub active_minigame: Option<ActiveMinigame>,
    /// Session kill count (transient, not saved)
    #[serde(skip)]
    pub session_kills: u64,
    /// Recent item drops for display (transient, not saved)
    #[serde(skip)]
    pub recent_drops: VecDeque<RecentDrop>,
    /// Scrolling loot ticker state (transient, not saved)
    #[serde(skip)]
    pub loot_ticker: LootTicker,
    /// Last minigame win info for achievement tracking (transient, not saved)
    #[serde(skip)]
    pub last_minigame_win: Option<MinigameWinInfo>,
    /// Cached derived stats — recalculated when attributes, equipment, or enhancement change
    #[serde(skip)]
    pub cached_derived_stats: DerivedStats,
    /// Cached prestige combat bonuses — recalculated when prestige_rank changes
    #[serde(skip)]
    pub cached_prestige_bonuses: PrestigeCombatBonuses,
    /// Dirty flag: set when attributes, equipment, or enhancement levels change
    #[serde(skip)]
    pub derived_stats_dirty: bool,
}

impl GameState {
    /// Creates a new game state with default values
    pub fn new(character_name: String, current_time: i64) -> Self {
        use uuid::Uuid;

        let attributes = Attributes::new();
        let combat_state = CombatState::new(crate::core::constants::BASE_HP as u32);
        let equipment = Equipment::new();

        Self {
            character_id: Uuid::new_v4().to_string(),
            character_name,
            character_level: 1,
            character_xp: 0,
            attributes,
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: current_time,
            play_time_seconds: 0,
            combat_state,
            equipment,
            active_dungeon: None,
            fishing: FishingState::default(),
            active_fishing: None,
            zone_progression: ZoneProgression::new(),
            challenge_menu: ChallengeMenu::new(),
            chess_stats: ChessStats::default(),
            active_minigame: None,
            session_kills: 0,
            recent_drops: VecDeque::with_capacity(5),
            loot_ticker: LootTicker::new(),
            last_minigame_win: None,
            cached_derived_stats: DerivedStats::default(),
            cached_prestige_bonuses: PrestigeCombatBonuses::default(),
            derived_stats_dirty: true,
        }
    }

    /// Returns true if the player is currently in a dungeon
    #[allow(dead_code)]
    pub fn is_in_dungeon(&self) -> bool {
        self.active_dungeon.is_some()
    }

    pub fn get_attribute_cap(&self) -> u32 {
        crate::core::constants::BASE_ATTRIBUTE_CAP
            + (self.prestige_rank * crate::core::constants::ATTRIBUTE_CAP_PER_PRESTIGE)
    }

    /// Recalculate and cache derived stats from current attributes, equipment, and enhancement levels.
    pub fn recalculate_derived_stats(&mut self, enhancement_levels: &[u8; 7]) {
        self.cached_derived_stats = DerivedStats::calculate_derived_stats(
            &self.attributes,
            &self.equipment,
            enhancement_levels,
        );
        self.derived_stats_dirty = false;
    }

    /// Mark derived stats as needing recalculation (e.g., after equipment or attribute change).
    pub fn invalidate_derived_stats(&mut self) {
        self.derived_stats_dirty = true;
    }

    /// Recalculate and cache prestige combat bonuses from current prestige rank.
    pub fn recalculate_prestige_bonuses(&mut self) {
        self.cached_prestige_bonuses = PrestigeCombatBonuses::from_rank(self.prestige_rank);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::attributes::AttributeType;

    #[test]
    fn test_new_game_state() {
        let current_time = 1234567890;
        let game_state = GameState::new("Test Hero".to_string(), current_time);

        assert_eq!(game_state.character_level, 1);
        assert_eq!(game_state.character_xp, 0);
        assert_eq!(game_state.prestige_rank, 0);
        assert_eq!(game_state.total_prestige_count, 0);
        assert_eq!(game_state.last_save_time, current_time);
        assert_eq!(game_state.play_time_seconds, 0);

        // Verify all attributes start at 10
        for attr in AttributeType::all() {
            assert_eq!(game_state.attributes.get(attr), 10);
        }
    }

    #[test]
    fn test_attribute_cap() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Prestige 0: cap 20
        assert_eq!(game_state.get_attribute_cap(), 20);

        // Prestige 1: cap 25
        game_state.prestige_rank = 1;
        assert_eq!(game_state.get_attribute_cap(), 25);

        // Prestige 2: cap 30
        game_state.prestige_rank = 2;
        assert_eq!(game_state.get_attribute_cap(), 30);
    }

    #[test]
    fn test_character_id_uniqueness() {
        let state1 = GameState::new("Hero1".to_string(), 0);
        let state2 = GameState::new("Hero2".to_string(), 0);

        // Each character should have a unique ID
        assert_ne!(state1.character_id, state2.character_id);
        // IDs should be valid UUIDs (36 chars with hyphens)
        assert_eq!(state1.character_id.len(), 36);
        assert_eq!(state2.character_id.len(), 36);
    }

    #[test]
    fn test_is_in_dungeon() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Initially not in a dungeon
        assert!(!game_state.is_in_dungeon());

        // Set an active dungeon
        game_state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0, 1));

        assert!(game_state.is_in_dungeon());
    }

    #[test]
    fn test_character_name_stored() {
        let game_state = GameState::new("My Hero Name".to_string(), 0);
        assert_eq!(game_state.character_name, "My Hero Name");
    }

    #[test]
    fn test_combat_state_initialized() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        // Combat state should be initialized with base HP
        assert_eq!(game_state.combat_state.player_max_hp, 50);
        assert_eq!(game_state.combat_state.player_current_hp, 50);
        assert!(game_state.combat_state.current_enemy.is_none());
        assert!(!game_state.combat_state.is_regenerating);
    }

    #[test]
    fn test_equipment_starts_empty() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        assert!(game_state.equipment.weapon.is_none());
        assert!(game_state.equipment.armor.is_none());
        assert!(game_state.equipment.helmet.is_none());
        assert!(game_state.equipment.gloves.is_none());
        assert!(game_state.equipment.boots.is_none());
        assert!(game_state.equipment.amulet.is_none());
        assert!(game_state.equipment.ring.is_none());
    }

    #[test]
    fn test_zone_progression_starts_at_zone_1() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        assert_eq!(game_state.zone_progression.current_zone_id, 1);
        assert_eq!(game_state.zone_progression.current_subzone_id, 1);
        assert!(!game_state.zone_progression.fighting_boss);
    }

    #[test]
    fn test_fishing_state_initialized() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        assert_eq!(game_state.fishing.rank, 1); // Fishing starts at rank 1
        assert_eq!(game_state.fishing.total_fish_caught, 0);
        assert!(game_state.active_fishing.is_none());
    }

    #[test]
    fn test_attribute_cap_high_prestige() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Prestige 10: cap should be 20 + (10 * 5) = 70
        game_state.prestige_rank = 10;
        assert_eq!(game_state.get_attribute_cap(), 70);

        // Prestige 20: cap should be 20 + (20 * 5) = 120
        game_state.prestige_rank = 20;
        assert_eq!(game_state.get_attribute_cap(), 120);
    }

    #[test]
    fn test_add_recent_drop_single() {
        let mut gs = GameState::new("Hero".to_string(), 0);
        assert!(gs.recent_drops.is_empty());

        gs.add_recent_drop(
            "Iron Sword".to_string(),
            Rarity::Common,
            true,
            "⚔",
            "Weapon".to_string(),
            "+2 STR".to_string(),
        );

        assert_eq!(gs.recent_drops.len(), 1);
        assert_eq!(gs.recent_drops[0].name, "Iron Sword");
        assert_eq!(gs.recent_drops[0].rarity, Rarity::Common);
        assert!(gs.recent_drops[0].equipped);
        assert_eq!(gs.recent_drops[0].slot, "Weapon");
        assert_eq!(gs.recent_drops[0].stats, "+2 STR");
    }

    #[test]
    fn test_add_recent_drop_fifo_order() {
        let mut gs = GameState::new("Hero".to_string(), 0);

        gs.add_recent_drop(
            "First".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        gs.add_recent_drop(
            "Second".to_string(),
            Rarity::Rare,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        gs.add_recent_drop(
            "Third".to_string(),
            Rarity::Epic,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );

        // Most recent should be at front
        assert_eq!(gs.recent_drops.len(), 3);
        assert_eq!(gs.recent_drops[0].name, "Third");
        assert_eq!(gs.recent_drops[1].name, "Second");
        assert_eq!(gs.recent_drops[2].name, "First");
    }

    #[test]
    fn test_add_recent_drop_caps_at_max() {
        let mut gs = GameState::new("Hero".to_string(), 0);

        // Fill to the cap (MAX_RECENT_DROPS = 10)
        for i in 0..10 {
            gs.add_recent_drop(
                format!("Item {i}"),
                Rarity::Common,
                false,
                "",
                "".to_string(),
                "".to_string(),
            );
        }
        assert_eq!(gs.recent_drops.len(), 10);

        // Adding one more should evict the oldest
        gs.add_recent_drop(
            "Overflow".to_string(),
            Rarity::Legendary,
            true,
            "",
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(gs.recent_drops.len(), 10);
        assert_eq!(gs.recent_drops[0].name, "Overflow");
        // "Item 0" (the oldest) should have been evicted
        assert!(gs.recent_drops.iter().all(|d| d.name != "Item 0"));
        // "Item 1" should still be present as the last element
        assert_eq!(gs.recent_drops[9].name, "Item 1");
    }

    #[test]
    fn test_add_recent_drop_at_exact_cap_boundary() {
        let mut gs = GameState::new("Hero".to_string(), 0);

        // Add exactly MAX_RECENT_DROPS items
        for i in 0..10 {
            gs.add_recent_drop(
                format!("Item {i}"),
                Rarity::Common,
                false,
                "",
                "".to_string(),
                "".to_string(),
            );
        }
        assert_eq!(gs.recent_drops.len(), 10);

        // Add two more, should still be capped at 10
        gs.add_recent_drop(
            "Extra1".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        gs.add_recent_drop(
            "Extra2".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(gs.recent_drops.len(), 10);
        assert_eq!(gs.recent_drops[0].name, "Extra2");
        assert_eq!(gs.recent_drops[1].name, "Extra1");
    }

    #[test]
    fn test_serialization_round_trip_preserves_persistent_fields() {
        let mut gs = GameState::new("Serde Hero".to_string(), 42);
        gs.character_level = 15;
        gs.character_xp = 5000;
        gs.prestige_rank = 3;
        gs.total_prestige_count = 5;
        gs.play_time_seconds = 3600;
        gs.attributes.set(AttributeType::Strength, 18);

        let json = serde_json::to_string(&gs).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.character_name, "Serde Hero");
        assert_eq!(loaded.character_level, 15);
        assert_eq!(loaded.character_xp, 5000);
        assert_eq!(loaded.prestige_rank, 3);
        assert_eq!(loaded.total_prestige_count, 5);
        assert_eq!(loaded.play_time_seconds, 3600);
        assert_eq!(loaded.last_save_time, 42);
        assert_eq!(loaded.attributes.get(AttributeType::Strength), 18);
    }

    #[test]
    fn test_serialization_skips_transient_fields() {
        let mut gs = GameState::new("Hero".to_string(), 0);
        gs.session_kills = 999;
        gs.add_recent_drop(
            "Sword".to_string(),
            Rarity::Rare,
            true,
            "",
            "".to_string(),
            "".to_string(),
        );

        let json = serde_json::to_string(&gs).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();

        // Transient fields should be at default values after deserialization
        assert_eq!(loaded.session_kills, 0);
        assert!(loaded.recent_drops.is_empty());
        assert!(loaded.active_fishing.is_none());
        assert!(loaded.active_minigame.is_none());
        assert!(loaded.last_minigame_win.is_none());
    }

    #[test]
    fn test_serialization_default_fields_from_old_json() {
        // Simulate loading from an older save that lacks optional fields
        let minimal_json = serde_json::json!({
            "character_id": "test-id",
            "character_name": "Old Hero",
            "character_level": 5,
            "character_xp": 100,
            "attributes": { "values": [10, 10, 10, 10, 10, 10] },
            "prestige_rank": 0,
            "total_prestige_count": 0,
            "last_save_time": 0,
            "play_time_seconds": 0,
            "combat_state": {
                "player_max_hp": 50,
                "player_current_hp": 50,
                "current_enemy": null,
                "is_regenerating": false,
                "regen_timer": 0.0,
                "attack_timer": 0.0,
                "kills_in_subzone": 0,
                "fighting_boss": false,
                "total_kills": 0,
                "combat_log": []
            },
            "equipment": {
                "weapon": null,
                "armor": null,
                "helmet": null,
                "gloves": null,
                "boots": null,
                "amulet": null,
                "ring": null
            }
        });

        let loaded: GameState = serde_json::from_value(minimal_json).unwrap();

        // #[serde(default)] fields should get their defaults
        assert!(loaded.active_dungeon.is_none());
        assert_eq!(loaded.fishing.rank, 1);
        assert_eq!(loaded.zone_progression.current_zone_id, 1);
        assert_eq!(loaded.chess_stats.games_played, 0);
    }

    #[test]
    fn test_loot_ticker_new_is_empty() {
        let ticker = LootTicker::new();
        assert!(ticker.is_empty());
        assert_eq!(ticker.scroll_offset, 0.0);
    }

    #[test]
    fn test_loot_ticker_push_adds_entry() {
        let mut ticker = LootTicker::new();
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "[R] Flamebrand".to_string(),
            color: ratatui::style::Color::Yellow,
            bold: false,
        });
        assert_eq!(ticker.len(), 1);
    }

    #[test]
    fn test_loot_ticker_push_evicts_oldest() {
        let mut ticker = LootTicker::new();
        for i in 0..TICKER_MAX_ENTRIES + 5 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Item {i}"),
                color: ratatui::style::Color::White,
                bold: false,
            });
        }
        assert_eq!(ticker.len(), TICKER_MAX_ENTRIES);
    }

    #[test]
    fn test_loot_ticker_tick_advances_offset() {
        let mut ticker = LootTicker::new();
        assert_eq!(ticker.scroll_offset, 0.0);
        ticker.tick();
        assert!((ticker.scroll_offset - TICKER_SCROLL_SPEED).abs() < f64::EPSILON);
        ticker.tick();
        assert!((ticker.scroll_offset - 2.0 * TICKER_SCROLL_SPEED).abs() < f64::EPSILON);
    }

    #[test]
    fn test_loot_ticker_push_does_not_change_offset() {
        let mut ticker = LootTicker::new();
        // Add an initial entry and advance scroll
        ticker.push(TickerEntry {
            icon: "",
            text: "First".to_string(),
            color: ratatui::style::Color::White,
            bold: false,
        });
        for _ in 0..10 {
            ticker.tick();
        }
        let offset_before = ticker.scroll_offset;
        assert!(offset_before > 0.0);

        // Push a new entry — offset should NOT change at all
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "Sword".to_string(),
            color: ratatui::style::Color::Yellow,
            bold: false,
        });
        assert_eq!(ticker.scroll_offset, offset_before);
    }

    #[test]
    fn test_loot_ticker_cleanup_removes_scrolled_entries() {
        let mut ticker = LootTicker::new();
        ticker.viewport_width = 10;
        ticker.push(TickerEntry {
            icon: "",
            text: "Old".to_string(), // 3 chars, born_at=0
            color: ratatui::style::Color::White,
            bold: false,
        });
        ticker.push(TickerEntry {
            icon: "",
            text: "New".to_string(), // born_at spaced after "Old"
            color: ratatui::style::Color::White,
            bold: false,
        });
        assert_eq!(ticker.len(), 2);

        // Old entry: born_at=0, off-screen when scroll > viewport(10) + len(3) = 13
        // At 0.4/tick, need 32.5 ticks → ~35 ticks
        for _ in 0..35 {
            ticker.tick();
        }

        // Oldest entry should have been cleaned up
        assert_eq!(ticker.len(), 1);
    }

    #[test]
    fn test_loot_ticker_adaptive_speed_prevents_empty() {
        // Simulate rapid entry arrivals (every 15 ticks, like combat kills).
        // Without adaptive scroll speed, born_at debt grows unboundedly
        // and entries become permanently invisible after ~1 minute.
        let mut ticker = LootTicker::new();
        ticker.viewport_width = 80;

        // Push 100 entries with 15 ticks between each (simulates ~2.5 min of play)
        for i in 0..100 {
            ticker.push(TickerEntry {
                icon: "\u{2728}",
                text: format!("+{} XP", 200 + i),
                color: ratatui::style::Color::Green,
                bold: false,
            });
            for _ in 0..15 {
                ticker.tick();
            }
        }

        // After 100 entries, visible_entries should still return something.
        // Without adaptive speed, this would return empty.
        let visible = ticker.visible_entries(80);
        assert!(
            !visible.is_empty(),
            "Ticker should still show entries after extended play"
        );
    }

    #[test]
    fn test_loot_ticker_speed_ramps_up_with_debt() {
        let mut ticker = LootTicker::new();
        ticker.viewport_width = 80;

        // Push several entries rapidly to build up debt
        for i in 0..10 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Entry {i}"),
                color: ratatui::style::Color::White,
                bold: false,
            });
        }

        // Tick several times to let speed ramp up
        let initial_speed = ticker.current_speed;
        for _ in 0..20 {
            ticker.tick();
        }
        assert!(
            ticker.current_speed > initial_speed,
            "Speed should increase when entries are queued ahead"
        );
    }

    #[test]
    fn test_loot_ticker_speed_decelerates_when_caught_up() {
        let mut ticker = LootTicker::new();
        ticker.viewport_width = 80;

        // Push entries to build debt
        for i in 0..10 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Entry {i}"),
                color: ratatui::style::Color::White,
                bold: false,
            });
        }

        // Let speed ramp up
        for _ in 0..50 {
            ticker.tick();
        }
        let peak_speed = ticker.current_speed;

        // Now tick without pushing — speed should decelerate as debt clears
        for _ in 0..200 {
            ticker.tick();
        }
        assert!(
            ticker.current_speed < peak_speed,
            "Speed should decrease when debt is paid off"
        );
    }

    #[test]
    fn test_loot_ticker_scrolls_continuously() {
        let mut ticker = LootTicker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "Test".to_string(),
            color: ratatui::style::Color::White,
            bold: false,
        });
        // Scrolling should advance every tick, no pauses
        let offset_before = ticker.scroll_offset;
        ticker.tick();
        assert!(ticker.scroll_offset > offset_before);
        let offset_after_one = ticker.scroll_offset;
        ticker.tick();
        assert!(ticker.scroll_offset > offset_after_one);
    }
}
