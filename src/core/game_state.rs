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
use crate::god_items::CachedGodItemBonuses;
use crate::haven::HavenBonuses;
use crate::items::equipment::Equipment;
use crate::items::types::Rarity;
use crate::stormglass::sigils::{SigilBonuses, StormSigils};
use crate::zones::ZoneProgression;
use std::collections::VecDeque;

// Re-export ticker types for backward compatibility
pub use super::ticker::{Ticker, TickerEntry, TickerSegment, TICKER_SCROLL_SPEED};

// Re-export RecentDrop types for backward compatibility
pub use super::recent_drops::{RecentDrop, MAX_RECENT_DROPS};

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
        super::recent_drops::add_recent_drop(
            &mut self.recent_drops,
            name,
            rarity,
            equipped,
            icon,
            slot,
            stats,
        );
    }
}

/// Main game state containing all player progress
#[derive(Debug, Clone)]
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
    pub active_dungeon: Option<Dungeon>,
    /// Persistent fishing progression state
    pub fishing: FishingState,
    /// Active fishing session (transient, not saved)
    #[allow(dead_code)]
    pub active_fishing: Option<FishingSession>,
    /// Zone progression state
    pub zone_progression: ZoneProgression,
    /// Generic challenge menu (transient, not saved)
    pub challenge_menu: ChallengeMenu,
    /// Chess stats (transient, not saved to disk)
    pub chess_stats: ChessStats,
    /// Stormglass currency balance (character-level, saved to disk)
    pub stormglass: u64,
    /// Whether the player has discovered Stormglass (first gear salvage)
    pub stormglass_discovered: bool,
    /// Storm Sigils — persistent sigil slots (character-level, survives prestige)
    pub storm_sigils: StormSigils,
    /// Ascension level — per-character combat power multiplier (0 = no ascension)
    pub ascension_level: u32,
    /// True after the Zone 50 final boss first falls — enables Vessel ticker
    /// whispers and the [V] overlay (persistent, survives prestige)
    pub vessel_signal_discovered: bool,
    /// True after the player confirms the Vessel launch and burns 100,000 PR
    pub vessel_launched: bool,
    /// True once the Vessel reaches the Tree — the crossing is over and
    /// Act 3, whenever it exists, keys off this the way Act 2 keyed off
    /// `vessel_launched` (persistent, survives everything)
    pub vessel_arrived: bool,
    /// Play-time seconds when the last Vessel whisper was pushed (transient)
    pub vessel_last_whisper_at: u64,
    /// Active challenge minigame (transient, not saved)
    pub active_minigame: Option<ActiveMinigame>,
    /// Session kill count (transient, not saved)
    pub session_kills: u64,
    /// Consecutive deaths to regular mobs without a kill (transient, for death loop detection)
    pub consecutive_deaths: u32,
    /// When true, suppresses challenge discovery during Chrono Surge
    pub chrono_surge_active: bool,
    /// Debug: force next Chrono Surge to be overcharged
    pub debug_force_overcharge: bool,
    /// Recent item drops for display (transient, not saved)
    pub recent_drops: VecDeque<RecentDrop>,
    /// Scrolling loot ticker state (transient, not saved)
    pub ticker: Ticker,
    /// Last minigame win info for achievement tracking (transient, not saved)
    pub last_minigame_win: Option<MinigameWinInfo>,
    /// Cached derived stats — recalculated when attributes, equipment, or enhancement change
    pub cached_derived_stats: DerivedStats,
    /// Cached prestige combat bonuses — recalculated when prestige_rank changes
    pub cached_prestige_bonuses: PrestigeCombatBonuses,
    /// Dirty flag: set when attributes, equipment, or enhancement levels change
    pub derived_stats_dirty: bool,
    /// Rolling XP rate: XP gained per second over the last 15 minutes of combat time
    pub xp_rate_samples: VecDeque<u64>,
    /// XP accumulated during the current second (rotated into xp_rate_samples each second)
    pub xp_this_second: u64,
    /// True if any combat XP was earned during the current second (controls rate sampling)
    pub combat_seconds_this_tick: bool,
    /// When the game-over screen was first shown (for dismiss cooldown)
    pub game_over_shown_at: Option<std::time::Instant>,
    /// Cached power rating — computed each tick from DPS × eHP formula
    pub cached_power_rating: f64,
    /// Cached fracture zone cap from Deep — used by UI for zone track rendering
    pub cached_fracture_zone_cap: u32,
    /// Cached Loom zone cap from completed patterns — used by UI for zone track rendering
    pub cached_loom_zone_cap: u32,
    /// Cached merged Haven bonuses — recomputed only when bonuses_dirty is true
    pub cached_haven_bonuses: HavenBonuses,
    /// Cached merged Sigil bonuses — recomputed only when bonuses_dirty is true
    pub cached_sigil_bonuses: SigilBonuses,
    /// Dirty flag: set when Haven rooms, Storm Sigils, or prestige rank change
    pub bonuses_dirty: bool,
    /// Cached god item bonuses — recomputed when equipment changes (derived_stats_dirty)
    pub cached_god_item_bonuses: CachedGodItemBonuses,
}

impl GameState {
    /// Creates a new game state with default values
    pub fn new(character_name: String, current_time: i64) -> Self {
        use uuid::Uuid;

        let character_id = Uuid::new_v4().to_string();
        let attributes = Attributes::new();
        let combat_state = CombatState::new(crate::core::constants::BASE_HP as u64);
        let equipment = Equipment::new();

        Self {
            character_id,
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
            stormglass: 0,
            stormglass_discovered: false,
            storm_sigils: StormSigils::new(),
            ascension_level: 0,
            vessel_signal_discovered: false,
            vessel_launched: false,
            vessel_arrived: false,
            vessel_last_whisper_at: 0,
            active_minigame: None,
            session_kills: 0,
            consecutive_deaths: 0,
            recent_drops: VecDeque::with_capacity(5),
            ticker: Ticker::new(),
            last_minigame_win: None,
            cached_derived_stats: DerivedStats::default(),
            cached_prestige_bonuses: PrestigeCombatBonuses::default(),
            derived_stats_dirty: true,
            xp_rate_samples: VecDeque::new(),
            xp_this_second: 0,
            combat_seconds_this_tick: false,
            game_over_shown_at: None,
            cached_power_rating: 0.0,
            cached_fracture_zone_cap: 0,
            cached_loom_zone_cap: 0,
            cached_haven_bonuses: HavenBonuses::default(),
            cached_sigil_bonuses: SigilBonuses::default(),
            bonuses_dirty: true,
            cached_god_item_bonuses: CachedGodItemBonuses::default(),
            chrono_surge_active: false,
            debug_force_overcharge: false,
        }
    }

    /// Returns true if the player is currently in a dungeon
    #[allow(dead_code)]
    pub fn is_in_dungeon(&self) -> bool {
        self.active_dungeon.is_some()
    }

    /// Returns XP per hour based on rolling 15-minute combat-only window, or None if < 10s of data.
    pub fn xp_per_hour(&self) -> Option<u64> {
        if self.xp_rate_samples.len() < 10 {
            return None;
        }
        let sum: u64 = self.xp_rate_samples.iter().sum();
        Some((sum as f64 / self.xp_rate_samples.len() as f64 * 3600.0) as u64)
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

    /// Mark Haven/Sigil bonuses as needing recalculation.
    pub fn invalidate_bonuses(&mut self) {
        self.bonuses_dirty = true;
    }

    /// Recalculate and cache prestige combat bonuses from current prestige rank.
    pub fn recalculate_prestige_bonuses(&mut self) {
        self.cached_prestige_bonuses = PrestigeCombatBonuses::from_rank(self.prestige_rank);
    }
}

/// Grouped accessors — these provide the same data organized by domain.
/// New code should prefer these over direct field access.
#[allow(dead_code)]
impl GameState {
    // --- Player Identity ---
    pub fn player_id(&self) -> &str {
        &self.character_id
    }
    pub fn player_level(&self) -> u32 {
        self.character_level
    }
    pub fn player_xp(&self) -> u64 {
        self.character_xp
    }
    pub fn player_name(&self) -> &str {
        &self.character_name
    }
    pub fn player_prestige_rank(&self) -> u32 {
        self.prestige_rank
    }
    pub fn player_attributes(&self) -> &Attributes {
        &self.attributes
    }
    pub fn player_attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }
    pub fn total_prestige_count(&self) -> u64 {
        self.total_prestige_count
    }

    // --- Combat Context ---
    pub fn current_zone_id(&self) -> u32 {
        self.zone_progression.current_zone_id
    }
    pub fn is_fighting(&self) -> bool {
        self.combat_state.current_enemy.is_some() && !self.combat_state.is_regenerating
    }
    pub fn is_regenerating(&self) -> bool {
        self.combat_state.is_regenerating
    }
    pub fn current_subzone_id(&self) -> u32 {
        self.zone_progression.current_subzone_id
    }

    // --- Progression State ---
    pub fn is_fishing(&self) -> bool {
        self.active_fishing.is_some()
    }
    pub fn fishing_rank(&self) -> u32 {
        self.fishing.rank
    }
    pub fn stormglass_balance(&self) -> u64 {
        self.stormglass
    }
    pub fn has_active_minigame(&self) -> bool {
        self.active_minigame.is_some()
    }

    // --- Session State ---
    pub fn save_time(&self) -> i64 {
        self.last_save_time
    }
    pub fn play_time(&self) -> u64 {
        self.play_time_seconds
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
            "\u{2694}",
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
        // storm_sigils should default to 0 slots unlocked, no sigils etched
        assert_eq!(loaded.storm_sigils.slots_unlocked, 0);
        assert_eq!(loaded.storm_sigils.etched_count(), 0);
    }

    #[test]
    fn test_storm_sigils_initialized_in_new() {
        let gs = GameState::new("Sigil Hero".to_string(), 0);
        assert_eq!(gs.storm_sigils.slots_unlocked, 0);
        assert_eq!(gs.storm_sigils.sigils.len(), 5);
        assert_eq!(gs.storm_sigils.etched_count(), 0);
    }

    #[test]
    fn test_storm_sigils_serde_round_trip() {
        use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

        let mut gs = GameState::new("Sigil Hero".to_string(), 0);
        gs.storm_sigils.slots_unlocked = 3;
        gs.storm_sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 18.5,
            grade: SigilGrade::A,
        });
        gs.storm_sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 7.2,
            grade: SigilGrade::C,
        });

        let json = serde_json::to_string(&gs).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.storm_sigils.slots_unlocked, 3);
        assert_eq!(loaded.storm_sigils.etched_count(), 2);
        let s0 = loaded.storm_sigils.sigils[0].as_ref().unwrap();
        assert_eq!(s0.effect, SigilEffectType::XpPercent);
        assert!((s0.value - 18.5).abs() < 1e-10);
        assert_eq!(s0.grade, SigilGrade::A);
        let s1 = loaded.storm_sigils.sigils[1].as_ref().unwrap();
        assert_eq!(s1.effect, SigilEffectType::DamagePercent);
        assert!((s1.value - 7.2).abs() < 1e-10);
        assert!(loaded.storm_sigils.sigils[2].is_none());
    }

    #[test]
    fn test_storm_sigils_preserved_through_prestige() {
        use crate::character::prestige_actions::perform_prestige;
        use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

        let mut gs = GameState::new("Prestige Hero".to_string(), 0);
        // Make character eligible for prestige (level 10 required for first prestige)
        gs.character_level = 10;
        gs.prestige_rank = 0;

        // Etch a sigil before prestige
        gs.storm_sigils.slots_unlocked = 2;
        gs.storm_sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 5.5,
            grade: SigilGrade::APlus,
        });

        perform_prestige(&mut gs);

        // Verify prestige happened
        assert_eq!(gs.prestige_rank, 1);
        assert_eq!(gs.character_level, 1);

        // Verify sigils survived
        assert_eq!(gs.storm_sigils.slots_unlocked, 2);
        assert_eq!(gs.storm_sigils.etched_count(), 1);
        let sigil = gs.storm_sigils.sigils[0].as_ref().unwrap();
        assert_eq!(sigil.effect, SigilEffectType::CritChancePercent);
        assert!((sigil.value - 5.5).abs() < 1e-10);
        assert_eq!(sigil.grade, SigilGrade::APlus);
    }

    #[test]
    fn test_consecutive_deaths_transient() {
        let mut gs = GameState::new("Hero".to_string(), 0);
        assert_eq!(gs.consecutive_deaths, 0);

        gs.consecutive_deaths = 5;
        let json = serde_json::to_string(&gs).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.consecutive_deaths, 0); // transient, not saved
    }

    #[test]
    fn test_accessor_methods() {
        let state = GameState::new("AccessorTest".to_string(), 1000);
        assert_eq!(state.player_id(), state.character_id);
        assert_eq!(state.total_prestige_count(), 0);
        assert!(!state.is_fighting());
        assert!(!state.is_regenerating());
        assert!(!state.is_fishing());
        assert_eq!(state.fishing_rank(), 1);
        assert_eq!(state.stormglass_balance(), 0);
        assert!(!state.has_active_minigame());
        assert_eq!(state.save_time(), 1000);
        assert_eq!(state.play_time(), 0);
    }
}
