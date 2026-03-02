//! Tick event and result types for the game tick engine.
//!
//! These types are produced by [`super::tick::game_tick()`] and consumed by
//! the presentation layer (main.rs) to update the UI without game logic
//! depending on any UI types.

use crate::achievements::AchievementId;
use crate::challenges::menu::ChallengeType;
use crate::dungeon::types::RoomType;
use crate::items::types::Rarity;
use crate::zones::BossDefeatResult;

/// A single event produced by a game tick.
///
/// The presentation layer (main.rs) maps these to combat log entries,
/// visual effects, and UI state changes. The game logic layer never
/// touches UI types directly.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of the public API contract; main.rs matches with `..`
pub enum TickEvent {
    // ── Combat ──────────────────────────────────────────────────
    /// Player attacked an enemy.
    PlayerAttack { damage: u32, was_crit: bool },

    /// Player's attack was blocked because the boss requires a specific weapon.
    PlayerAttackBlocked { weapon_needed: String },

    /// Enemy attacked the player.
    EnemyAttack { damage: u32, enemy_name: String },

    /// Damage reflected back to the enemy.
    DamageReflected { damage: u32 },

    /// HP regen completed after a kill.
    RegenComplete { healed: u32 },

    /// Normal enemy or dungeon combat-room enemy was defeated.
    EnemyDefeated { xp_gained: u64, enemy_name: String },

    /// Boss enraged after fight timer expired — instant kill.
    BossEnrage { message: String },

    /// Player died in overworld combat (boss encounter reset).
    PlayerDied,

    /// Player died in a dungeon (safe exit, no prestige loss).
    PlayerDiedInDungeon,

    /// Player was overwhelmed and retreated to a safer zone.
    CombatRetreat { zone_name: String },

    // ── Item Drops ──────────────────────────────────────────────
    /// An item was dropped and auto-equip was evaluated.
    ItemDropped {
        item_name: String,
        rarity: Rarity,
        tier: u8,
        ilvl: u32,
        power: u32,
        equipped: bool,
        slot: String,
        stats: String,
        from_boss: bool,
    },

    // ── Zone Progression ────────────────────────────────────────
    /// A subzone boss was defeated and zone progression updated.
    SubzoneBossDefeated {
        xp_gained: u64,
        result: BossDefeatResult,
    },

    // ── Dungeon ─────────────────────────────────────────────────
    /// Player entered a dungeon room during auto-exploration.
    DungeonRoomEntered {
        room_type: RoomType,
        message: String,
    },

    /// Treasure found in a dungeon treasure room.
    DungeonTreasureFound {
        item_name: String,
        rarity: Rarity,
        tier: u8,
        ilvl: u32,
        power: u32,
        equipped: bool,
    },

    /// Dungeon key found (from defeating the elite guardian).
    DungeonKeyFound,

    /// Boss room is now unlocked.
    DungeonBossUnlocked,

    /// Dungeon boss defeated — dungeon completed with rewards.
    DungeonBossDefeated {
        xp_gained: u64,
        bonus_xp: u64,
        total_xp: u64,
        items_collected: usize,
        enemy_name: String,
    },

    /// Dungeon elite enemy defeated.
    DungeonEliteDefeated { xp_gained: u64, enemy_name: String },

    /// Player died or was removed from dungeon.
    DungeonFailed,

    /// Dungeon completed event from auto-exploration (update_dungeon).
    DungeonCompleted {
        xp_earned: u64,
        items_collected: usize,
    },

    // ── Fishing ─────────────────────────────────────────────────
    /// A generic fishing phase/event message.
    FishingMessage { message: String },

    /// A fish was caught (for recent-drops tracking in the UI).
    FishCaught {
        fish_name: String,
        rarity: Rarity,
        message: String,
    },

    /// An item was found while fishing.
    FishingItemFound { item_name: String, message: String },

    /// Fishing rank increased.
    FishingRankUp { message: String },

    /// The Storm Leviathan was caught (triggers achievement).
    StormLeviathanCaught,

    // ── Discovery ───────────────────────────────────────────────
    /// A challenge minigame was discovered.
    ChallengeDiscovered {
        challenge_type: ChallengeType,
        message: String,
        follow_up: String,
    },

    /// A dungeon entrance was discovered after killing an enemy.
    DungeonDiscovered { message: String },

    /// A fishing spot was discovered after killing an enemy.
    FishingSpotDiscovered { message: String },

    /// The Haven was discovered (P10+ idle roll).
    HavenDiscovered,

    /// The Soulforge was discovered (P15+ idle roll).
    SoulforgeDiscovered,

    /// The Deep was discovered (first Endless kill at P15+).
    DeepDiscovered,

    // ── The Deep ──────────────────────────────────────────────────
    /// A Deep mission was completed.
    DeepMissionComplete { message: String },

    /// A Deep event is pending and needs player response.
    DeepEventPending { message: String },

    /// A mercenary was injured during a Deep expedition.
    DeepMercInjured { message: String },

    /// A mercenary was permanently lost in The Deep.
    DeepMercLost { message: String },

    /// A breakthrough mission was completed, unlocking a new layer.
    DeepBreakthrough { message: String },

    /// The guild rank increased in The Deep.
    DeepGuildRankUp { message: String },

    /// Stormglass was discovered for the first time (first gear salvage).
    StormglassDiscovered,

    /// An item was salvaged into Stormglass.
    StormglassSalvaged {
        item_name: String,
        rarity: Rarity,
        amount: u64,
    },

    /// Stormglass cache found in a dungeon treasure room.
    StormglassDungeonCache { amount: u64 },

    // ── Achievements ────────────────────────────────────────────
    /// An achievement was unlocked during this tick.
    AchievementUnlocked { name: String },

    // ── Fracture Zones ───────────────────────────────────────────
    /// A fracture region was unlocked by a Deep breakthrough.
    FractureRegionUnlocked {
        region: crate::zones::FractureRegion,
        message: String,
    },

    /// Player has Ascended to a new level.
    Ascended { level: u32 },

    // ── Level Up ────────────────────────────────────────────────
    /// Player leveled up (may occur multiple times per tick from large XP gains).
    LeveledUp { new_level: u32 },

    // ── Power Cores ─────────────────────────────────────────────
    /// A Power Core completed a cycle and granted +1 prestige rank.
    PowerCoreGranted {
        /// Display name of the core that fired (e.g. "Red Fault").
        core_name: &'static str,
    },
}

/// Result of processing a single game tick.
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    /// Events produced during this tick, in chronological order.
    pub events: Vec<TickEvent>,

    /// If set, a Storm Leviathan encounter occurred during fishing.
    /// The value is the encounter number (1-10). The presentation layer
    /// uses this to show the Leviathan modal overlay.
    pub leviathan_encounter: Option<u8>,

    /// True if the Storm Lure was consumed during this tick's Leviathan encounter.
    pub leviathan_lure_consumed: bool,

    /// True if a Leviathan catch-miss occurred (appeared but wasn't caught).
    pub leviathan_catch_miss: bool,

    /// True if achievements were modified and should be persisted to disk.
    /// The presentation layer is responsible for the actual IO.
    pub achievements_changed: bool,

    /// True if Haven state was modified (discovery) and should be persisted.
    pub haven_changed: bool,

    /// True if Enhancement state was modified (discovery) and should be persisted.
    pub enhancement_changed: bool,

    /// True if God Item progress was modified and should be persisted.
    pub god_items_changed: bool,

    /// True if Deep state was modified (discovery, power cores) and should be persisted.
    pub deep_changed: bool,

    /// Achievement IDs ready to be shown in a modal overlay.
    /// Populated when the 500ms accumulation window has elapsed.
    /// Empty if no modal is ready or another overlay is already active.
    pub achievement_modal_ready: Vec<AchievementId>,
}
