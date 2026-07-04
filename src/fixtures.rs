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
use crate::haven::{Haven, HavenRoomId};
use crate::items::{generate_item_with_rng, EquipmentSlot, Rarity};
use crate::loom::types::{LoomNodeRef, LoomState, NodeId, NodeNature, Resource, Shuttle};
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

/// A discovered Loom with two unlocked extractors feeding one built T1
/// shuttle (Ember + VoidEssence -> ForgedLight), so the graph renderer has
/// nodes and edges to draw instead of the empty-loom placeholder.
pub fn loom_state_with_shuttle() -> LoomState {
    let mut loom = LoomState::new();
    loom.persistent.discovered = true;
    for node in loom.persistent.nodes.iter_mut() {
        if matches!(node.id, NodeId::EmberSpindle | NodeId::VoidCondenser) {
            node.unlocked = true;
            node.buffer = 120.0;
        }
    }
    let mut shuttle = Shuttle::new(
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        Resource::ForgedLight,
        1.0,
        1,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    shuttle.buffer = 42.0;
    loom.persistent.shuttles.push(shuttle);
    loom
}

/// A discovered Haven with a spread of room tiers (assigned along the
/// fixed `HavenRoomId::ALL` order, so the layout is deterministic).
pub fn haven_built() -> Haven {
    let mut haven = Haven::new();
    haven.discovered = true;
    let tiers: [u8; 12] = [4, 3, 2, 2, 1, 1, 0, 3, 1, 2, 4, 1];
    for (room, tier) in HavenRoomId::ALL.iter().zip(tiers) {
        haven.rooms.insert(*room, tier);
    }
    haven
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

/// A voyage six days in, holding station at the Shoal Markets (the first
/// junction), scene played, ready to chart a course. Launch time is
/// `now - 6 days` so wall-clock ticks are deterministic for tests.
pub fn voyage_at_first_junction(now: DateTime<Utc>) -> crate::vessel::voyage::VoyageState {
    use crate::vessel::route::{roads_from, WaypointId, ROUTE_START};
    let launched = now - Duration::days(6);
    let mut v =
        crate::vessel::voyage::VoyageState::begin("fixture-voyager".to_string(), 42, launched);
    v.intro_pending = false;
    v.play_arrival_scene();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(launched + Duration::days(2));
    v.play_arrival_scene();
    // Maren asks to board at the Lightship Vigil; the fixture takes her.
    assert!(v.accept_ask(), "Maren's ask expected at W1");
    v.depart(roads_from(WaypointId(1)).next().unwrap().id)
        .unwrap();
    v.tick(launched + Duration::days(4));
    v.play_arrival_scene();
    v.tick(now);
    // A rumor aboard so the junction card shows an annotation and a
    // rumor-revealed stop.
    v.rumors.push(crate::vessel::voyage::LearnedRumor {
        rumor: crate::vessel::route::RumorId(1),
        learned_at: WaypointId(1),
    });
    v
}

/// A voyage mid-leg on the Drowned Choir road (out of the first junction),
/// roughly half way along, at Quiet trim.
pub fn voyage_mid_leg(now: DateTime<Utc>) -> crate::vessel::voyage::VoyageState {
    use crate::vessel::route::roads_from;
    let mut v = voyage_at_first_junction(now - Duration::hours(22));
    v.set_trim(crate::vessel::voyage::Trim::Quiet);
    let junction = v.current_waypoint().unwrap();
    v.depart(roads_from(junction).next().unwrap().id).unwrap();
    v.tick(now);
    v
}

/// A voyage holding station at an arbitrary waypoint (scene not yet played),
/// with a plausible visited spine behind it and siblings of every taken road
/// marked untaken — exactly as real play would leave them. Used by
/// `mkstate --voyage` for drive-game sessions.
pub fn voyage_holding_at(
    character_id: String,
    waypoint: crate::vessel::route::WaypointId,
    day: u64,
    provisions: f64,
    now: DateTime<Utc>,
) -> crate::vessel::voyage::VoyageState {
    use crate::vessel::route::{roads_from, ROUTE_START};
    use crate::vessel::voyage::{SceneState, VoyagePhase, VoyageState, MINUTES_PER_DAY};

    // BFS for a real path from the start to the target waypoint.
    let mut queue = std::collections::VecDeque::from([vec![ROUTE_START]]);
    let path = loop {
        let Some(path) = queue.pop_front() else {
            panic!("waypoint {waypoint:?} is not reachable from the start");
        };
        let last = *path.last().unwrap();
        if last == waypoint {
            break path;
        }
        for road in roads_from(last) {
            let mut next = path.clone();
            next.push(road.to);
            queue.push_back(next);
        }
    };

    let launched = now - Duration::days(day as i64);
    let mut v = VoyageState::begin(character_id, 42, launched);
    v.intro_pending = false;
    v.processed_minutes = day * MINUTES_PER_DAY;
    v.provisions = provisions.clamp(0.0, v.provisions_cap);
    for pair in path.windows(2) {
        for sibling in roads_from(pair[0]) {
            if sibling.to != pair[1] {
                v.untaken.push(sibling.id);
            }
        }
    }
    v.visited = path;
    v.phase = VoyagePhase::HoldingStation {
        waypoint,
        arrived_at_min: v.processed_minutes,
        scene_state: SceneState::Waiting,
        arrived_by: None,
    };
    // The doors a real arrival would have opened.
    if let Some(def) = crate::vessel::souls::recruit_at(waypoint) {
        if !v.met(def.id) {
            v.pending_ask = Some(def.id);
        }
    }
    if crate::vessel::route::waypoint(waypoint).has_feature(crate::vessel::route::Feature::Shipyard)
    {
        v.pending_refit = Some(0);
    }
    v
}

/// A colony mid-era for the Reckoning pane: twenty crossings into a long
/// campaign, half the districts founded, the headline into five figures,
/// drive compounding. Deterministic; matches the big-world curve.
pub fn colony_midera() -> crate::vessel::colony::ColonyState {
    use crate::vessel::colony::{ColonyState, CrossingRecords};
    let mut c = ColonyState::found("fixture-voyager".to_string());
    c.souls_delivered = 11_000;
    c.souls_remaining = 76_000;
    c.drive = 11_000;
    c.crossings_completed = 20;
    c.records = CrossingRecords {
        fastest_days: 27,
        most_carried: 1_146,
        total_lightyears: 6_400,
        total_nights: 610,
    };
    // Well into the era: most of the old world has gone dark, a lit path
    // still runs toward the Tree.
    c.dimmed_ports = c.dark_ports();
    c
}

/// A finished crossing, moored at the Tree with the finale already read:
/// the harbor screen's home state. The roster carries one of everything
/// the manifest can show — souls ashore, a decline, a farewell, and a
/// carved name — plus keepsakes, letters, a refit, and the Going-Dark.
pub fn voyage_arrived(
    character_id: String,
    now: DateTime<Utc>,
) -> crate::vessel::voyage::VoyageState {
    use crate::vessel::refits::RefitId;
    use crate::vessel::route::{waypoint, RumorId, ROUTE_SINK};
    use crate::vessel::souls::{SoulId, Station};
    use crate::vessel::voyage::{
        LearnedRumor, LogEntry, SoulState, SoulStatus, VoyagePhase, MINUTES_PER_DAY,
    };

    let mut v = voyage_holding_at(character_id, ROUTE_SINK, 60, 40.0, now);
    v.phase = VoyagePhase::Arrived {
        at_min: 60 * MINUTES_PER_DAY,
    };
    v.finale_shown = true;
    v.hope = 9;

    // The cast: the launch trio crossed (Torvald last stood the helm),
    // Maren came aboard and crossed, Sefa was declined, Ysolt stepped
    // ashore in a farewell, and Cormac's name is carved into the hull.
    let met = |soul, status, station, left_at| SoulState {
        soul,
        status,
        station,
        arc_beat: 4,
        rest_minutes: 0,
        left_at,
        strain: 0,
        consecutive_watches: 0,
    };
    v.souls = vec![
        met(SoulId(0), SoulStatus::Aboard, Some(Station::Helm), None),
        met(SoulId(1), SoulStatus::Aboard, Some(Station::Tender), None),
        met(SoulId(2), SoulStatus::Aboard, None, None),
        met(SoulId(3), SoulStatus::Aboard, None, None),
        met(
            SoulId(4),
            SoulStatus::Declined,
            None,
            v.visited.get(2).copied(),
        ),
        met(
            SoulId(5),
            SoulStatus::Ashore,
            None,
            v.visited.get(4).copied(),
        ),
        met(SoulId(6), SoulStatus::Lost, None, None),
    ];

    v.keepsakes = vec![
        "the Warden's token, white and warm".to_string(),
        "a thorn spar, cut clean at the tip".to_string(),
    ];
    v.refits = vec![RefitId::StormSail];
    v.refit_doors_seen = 1;
    v.letters_received = 13;
    v.gone_dark = true;
    v.rumors = vec![LearnedRumor {
        rumor: RumorId(1),
        learned_at: v.visited.get(1).copied().unwrap_or(ROUTE_SINK),
    }];
    // ~60 days over the visited spine, a little over two days per port.
    let ports = v.visited.len().max(1) as u64;
    v.log = v
        .visited
        .iter()
        .enumerate()
        .map(|(i, w)| LogEntry {
            day: i as u64 * 60 / ports,
            text: waypoint(*w).name.to_string(),
        })
        .collect();
    for (day, text) in [
        (18, "Refit: Storm Sail"),
        (42, "No letters. There will be no more letters."),
    ] {
        let at = v
            .log
            .iter()
            .position(|e| e.day > day)
            .unwrap_or(v.log.len());
        v.log.insert(
            at,
            LogEntry {
                day,
                text: text.to_string(),
            },
        );
    }
    v
}
