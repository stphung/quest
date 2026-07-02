//! Shared game-state fixture builders for named scenarios.
//!
//! Single source of truth for the states used by the `mkstate` binary
//! (on-disk save fixtures for `drive-game` sessions) and by the UI snapshot
//! tests (in-memory states rendered against a `TestBackend`).
//!
//! Builders take the creation timestamp and RNG as parameters so callers
//! control determinism: `mkstate` passes `Utc::now()` and `rand::rng()`,
//! tests pass a fixed timestamp and a seeded `ChaCha8Rng` so every generated
//! item (tier, attributes, affixes, name) is reproducible.
#![allow(dead_code)]

use crate::achievements::{Achievements, UnlockedAchievement};
use crate::challenges::{ActiveMinigame, ChessDifficulty, ChessGame};
use crate::character::{AttributeType, Attributes};
use crate::combat::{CombatState, Enemy};
use crate::core::GameState;
use crate::deep::{DeepState, Mission, MissionStatus, MissionType};
use crate::items::{generate_item_with_rng, EquipmentSlot, Rarity};
use crate::power_cores::ALL_POWER_CORES;
use crate::zones::{get_zone, ZoneProgression};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;

/// Level 1 character at Zone 1, nothing discovered.
pub fn fresh(name: &str, created_at: i64) -> GameState {
    GameState::new(name.to_string(), created_at)
}

/// Level 45, P5, Zone 8, rare/epic gear, stormglass discovered.
pub fn midgame(name: &str, created_at: i64, rng: &mut impl Rng) -> GameState {
    let mut state = fresh(name, created_at);
    state.character_level = 45;
    state.prestige_rank = 5;
    state.total_prestige_count = 5;
    state.play_time_seconds = 60 * 60 * 30;
    set_attributes(&mut state, 40); // cap at P5 is 45
    advance_to_zone(&mut state, 8, 2);
    equip_all(&mut state, Rarity::Rare, Rarity::Epic, 80, rng);
    state.stormglass_discovered = true;
    state.stormglass = 750;
    sync_hp(&mut state);
    state
}

/// Level 80, P25, Ascension III, Zone 11 (The Expanse), epic/legendary gear.
pub fn endgame(name: &str, created_at: i64, rng: &mut impl Rng) -> GameState {
    let mut state = fresh(name, created_at);
    state.character_level = 80;
    state.prestige_rank = 25;
    state.total_prestige_count = 32;
    state.ascension_level = 3;
    state.play_time_seconds = 60 * 60 * 400;
    set_attributes(&mut state, 100); // cap at P25 is 145
    advance_to_zone(&mut state, 11, 1);
    equip_all(&mut state, Rarity::Epic, Rarity::Legendary, 110, rng);
    state.zone_progression.has_stormbreaker = true;
    state.stormglass_discovered = true;
    state.stormglass = 25_000;
    sync_hp(&mut state);
    state
}

/// Midgame state with the subzone boss ready to spawn on the first tick.
pub fn boss(name: &str, created_at: i64, rng: &mut impl Rng) -> GameState {
    let mut state = midgame(name, created_at, rng);
    // should_spawn_boss() becomes true, so the first tick spawns the boss.
    state.zone_progression.kills_in_subzone = crate::core::KILLS_FOR_BOSS;
    state
}

/// Sets all six attributes to `value`.
pub fn set_attributes(state: &mut GameState, value: u32) {
    let mut attrs = Attributes::new();
    for attr in AttributeType::all() {
        attrs.set(attr, value);
    }
    state.attributes = attrs;
}

/// Unlocks zones 1..=target, marks every subzone boss below the target
/// position as defeated, and places the character at (target, subzone).
pub fn advance_to_zone(state: &mut GameState, zone_id: u32, subzone_id: u32) {
    let mut prog = ZoneProgression::new();
    for z in 1..=zone_id {
        prog.unlock_zone(z);
        let Some(zone) = get_zone(z) else { continue };
        for sub in &zone.subzones {
            if z < zone_id || sub.id < subzone_id {
                prog.defeat_boss(z, sub.id);
            }
        }
    }
    prog.current_zone_id = zone_id;
    prog.current_subzone_id = subzone_id;
    state.zone_progression = prog;
}

/// Equips every slot with a generated item: `weapon_rarity` for the weapon,
/// `base` for the other six slots.
pub fn equip_all(
    state: &mut GameState,
    base: Rarity,
    weapon_rarity: Rarity,
    ilvl: u32,
    rng: &mut impl Rng,
) {
    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];
    for slot in slots {
        let rarity = if slot == EquipmentSlot::Weapon {
            weapon_rarity
        } else {
            base
        };
        state
            .equipment
            .set(slot, Some(generate_item_with_rng(slot, rarity, ilvl, rng)));
    }
}

/// Gives the fixture a sane starting HP pool. The real max HP (with
/// prestige/ascension bonuses) is recalculated by the first game tick.
pub fn sync_hp(state: &mut GameState) {
    let hp = 50 + state.character_level as u64 * 10;
    state.combat_state = CombatState::new(hp);
}

/// Puts a deterministic mid-fight enemy into the combat state: a Zone 8
/// mob at partial HP, with the player also below full HP so both HP bars
/// render partially filled. Stats are fixed (the real spawner rolls
/// variance from a thread RNG).
pub fn engage_enemy(state: &mut GameState) {
    let mut enemy = Enemy::new_with_defense("Tidal Kraken".to_string(), 820, 96, 48);
    enemy.current_hp = 512;
    state.combat_state.current_enemy = Some(enemy);
    state.combat_state.player_current_hp = (state.combat_state.player_max_hp * 2) / 3;
    state.combat_state.is_regenerating = false;
}

/// One freshly started instance of every challenge minigame, keyed by a
/// short snake_case name. Games whose setup rolls randomness (mine layouts,
/// puzzles, spawn positions) draw from the caller's RNG, so a seeded RNG
/// yields identical boards every time. Construction order is fixed —
/// reordering entries changes every game after the reordered one.
pub fn all_challenges(rng: &mut impl Rng) -> Vec<(&'static str, ActiveMinigame)> {
    use crate::challenges::{
        flappy, go, gomoku, jezzball, minesweeper, morris, rune, runic_lights, runic_shift,
        shard_fusion, snake, sudoku, vault_warden,
    };

    let mut rune_game = rune::RuneGame::new(rune::types::RuneDifficulty::Journeyman);
    rune::generate_code(&mut rune_game, rng);

    let mut shift_game =
        runic_shift::RunicShiftGame::new(runic_shift::RunicShiftDifficulty::Journeyman);
    // The constructor fills the board from a thread RNG; re-roll it from
    // the caller's RNG so the starting grid is reproducible.
    shift_game.fill_starting_rows(rng);
    shift_game.next_row = shift_game.roll_next_row(rng);

    vec![
        (
            "chess",
            ActiveMinigame::Chess(Box::new(ChessGame::new(ChessDifficulty::Journeyman))),
        ),
        (
            "flappy",
            ActiveMinigame::FlappyBird(flappy::FlappyBirdGame::new(
                flappy::FlappyBirdDifficulty::Journeyman,
            )),
        ),
        (
            "morris",
            ActiveMinigame::Morris(morris::MorrisGame::new(
                morris::MorrisDifficulty::Journeyman,
            )),
        ),
        (
            "gomoku",
            ActiveMinigame::Gomoku(gomoku::GomokuGame::new(
                gomoku::GomokuDifficulty::Journeyman,
            )),
        ),
        (
            "minesweeper",
            ActiveMinigame::Minesweeper(minesweeper::MinesweeperGame::new(
                minesweeper::MinesweeperDifficulty::Journeyman,
            )),
        ),
        ("rune", ActiveMinigame::Rune(rune_game)),
        (
            "runic_lights",
            runic_lights::start_runic_lights_game(
                runic_lights::RunicLightsDifficulty::Journeyman,
                rng,
            ),
        ),
        ("runic_shift", ActiveMinigame::RunicShift(shift_game)),
        (
            "shard_fusion",
            shard_fusion::start_shard_fusion_game(
                shard_fusion::ShardFusionDifficulty::Journeyman,
                rng,
            ),
        ),
        (
            "go",
            ActiveMinigame::Go(go::GoGame::new(go::GoDifficulty::Journeyman)),
        ),
        (
            "jezzball",
            ActiveMinigame::Jezzball(jezzball::JezzballGame::new(
                jezzball::JezzballDifficulty::Journeyman,
                rng,
            )),
        ),
        (
            "snake",
            ActiveMinigame::Snake(snake::SnakeGame::new(
                snake::SnakeDifficulty::Journeyman,
                rng,
            )),
        ),
        (
            "sudoku",
            ActiveMinigame::Sudoku(sudoku::generate_puzzle(
                sudoku::SudokuDifficulty::Journeyman,
                rng,
            )),
        ),
        (
            "vault_warden",
            vault_warden::start_vault_warden_game(
                vault_warden::VaultWardenDifficulty::Journeyman,
                rng,
            ),
        ),
    ]
}

/// A discovered Deep with mid-game progress and one running expedition.
/// `now` must be the frozen UI-clock instant in tests so the mission ETA
/// and progress gauge render deterministically.
pub fn deep_state_active(now: DateTime<Utc>) -> DeepState {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.deepest_layer_reached = 12;
    // Power Core I half-filled (2 PR/day = 12h fill), Core II left at 0 so
    // it renders as ready — one gauge of each kind.
    deep.persistent.power_core_last_granted.insert(
        crate::achievements::AchievementId::PowerCoreI,
        now.timestamp() - 6 * 3600,
    );
    deep.prestige.warband_marks = 45;
    deep.prestige.active_missions.push(Mission {
        id: 1,
        mission_type: MissionType::Expedition,
        layer: 12,
        squad: Vec::new(),
        started_at: now - Duration::hours(3),
        ends_at: now + Duration::hours(2),
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    });
    deep
}

/// Unlocks the first `count` Power Cores directly (bypassing the unlock
/// queue so no pending-notification state leaks into the fixture), with a
/// fixed unlock timestamp.
pub fn unlock_power_cores(achievements: &mut Achievements, count: usize, unlocked_at: i64) {
    for core in ALL_POWER_CORES.iter().take(count) {
        achievements.unlocked.insert(
            core.achievement_id,
            UnlockedAchievement {
                unlocked_at,
                character_name: None,
            },
        );
    }
}
