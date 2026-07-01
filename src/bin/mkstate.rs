//! Fixture generator: writes character save files for named game scenarios.
//!
//! Used by test harnesses (and Claude-driven verification sessions) to set up
//! game state without playing through it. Pair with the `QUEST_DIR` env var to
//! write into an isolated directory instead of the real `~/.quest`:
//!
//! ```sh
//! QUEST_DIR=/tmp/fx cargo run --bin mkstate -- midgame
//! QUEST_DIR=/tmp/fx cargo run --bin mkstate -- endgame --name Ragnar
//! QUEST_DIR=/tmp/fx cargo run  # then pick the character on the select screen
//! ```
//!
//! Scenarios only write the character save file. Account-level state
//! (haven.json, deep.json, loom, enhancement) is not generated yet — the game
//! treats those systems as undiscovered, which is a valid state.

use quest::character::{AttributeType, Attributes, CharacterManager};
use quest::combat::CombatState;
use quest::core::GameState;
use quest::core::KILLS_FOR_BOSS;
use quest::items::{generate_item, EquipmentSlot, Rarity};
use quest::zones::{get_zone, ZoneProgression};

use chrono::Utc;

struct Scenario {
    name: &'static str,
    description: &'static str,
    build: fn(String) -> GameState,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "fresh",
        description: "Level 1 character at Zone 1, nothing discovered",
        build: build_fresh,
    },
    Scenario {
        name: "midgame",
        description: "Level 45, P5, Zone 8, rare/epic gear, stormglass discovered",
        build: build_midgame,
    },
    Scenario {
        name: "endgame",
        description: "Level 80, P25, Ascension III, Zone 11 (The Expanse), epic/legendary gear",
        build: build_endgame,
    },
    Scenario {
        name: "boss",
        description: "Midgame state with the subzone boss ready to spawn on the first tick",
        build: build_boss,
    },
];

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--list" || args[1] == "-l" {
        eprintln!("Usage: mkstate <scenario> [--name <character-name>]\n");
        eprintln!("Scenarios:");
        for s in SCENARIOS {
            eprintln!("  {:<10} {}", s.name, s.description);
        }
        eprintln!("\nSet QUEST_DIR to write into an isolated directory.");
        std::process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let scenario_name = args[1].as_str();
    let char_name = args
        .iter()
        .position(|a| a == "--name")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| default_name(scenario_name));

    let Some(scenario) = SCENARIOS.iter().find(|s| s.name == scenario_name) else {
        eprintln!("Unknown scenario: {scenario_name}");
        eprintln!("Run 'mkstate --list' to see available scenarios.");
        std::process::exit(1);
    };

    let state = (scenario.build)(char_name.clone());

    let manager = CharacterManager::new().expect("failed to open quest dir");
    manager
        .save_character(&state)
        .expect("failed to write save file");

    let dir = quest::core::paths::get_quest_dir().expect("failed to resolve quest dir");
    println!(
        "Wrote '{}' ({}) to {}",
        char_name,
        scenario.name,
        dir.display()
    );
}

fn default_name(scenario: &str) -> String {
    // Capitalize the scenario name so each fixture is recognizable
    // on the character select screen.
    let mut c = scenario.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => scenario.to_string(),
    }
}

fn build_fresh(name: String) -> GameState {
    GameState::new(name, Utc::now().timestamp())
}

fn build_midgame(name: String) -> GameState {
    let mut state = GameState::new(name, Utc::now().timestamp());
    state.character_level = 45;
    state.prestige_rank = 5;
    state.total_prestige_count = 5;
    state.play_time_seconds = 60 * 60 * 30;
    set_attributes(&mut state, 40); // cap at P5 is 45
    advance_to_zone(&mut state, 8, 2);
    equip_all(&mut state, Rarity::Rare, Rarity::Epic, 80);
    state.stormglass_discovered = true;
    state.stormglass = 750;
    sync_hp(&mut state);
    state
}

fn build_endgame(name: String) -> GameState {
    let mut state = GameState::new(name, Utc::now().timestamp());
    state.character_level = 80;
    state.prestige_rank = 25;
    state.total_prestige_count = 32;
    state.ascension_level = 3;
    state.play_time_seconds = 60 * 60 * 400;
    set_attributes(&mut state, 100); // cap at P25 is 145
    advance_to_zone(&mut state, 11, 1);
    equip_all(&mut state, Rarity::Epic, Rarity::Legendary, 110);
    state.zone_progression.has_stormbreaker = true;
    state.stormglass_discovered = true;
    state.stormglass = 25_000;
    sync_hp(&mut state);
    state
}

fn build_boss(name: String) -> GameState {
    let mut state = build_midgame(name);
    // should_spawn_boss() becomes true, so the first tick spawns the boss.
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;
    state
}

fn set_attributes(state: &mut GameState, value: u32) {
    let mut attrs = Attributes::new();
    for attr in AttributeType::all() {
        attrs.set(attr, value);
    }
    state.attributes = attrs;
}

/// Unlocks zones 1..=target, marks every subzone boss below the target
/// position as defeated, and places the character at (target, subzone).
fn advance_to_zone(state: &mut GameState, zone_id: u32, subzone_id: u32) {
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

fn equip_all(state: &mut GameState, base: Rarity, weapon_rarity: Rarity, ilvl: u32) {
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
            .set(slot, Some(generate_item(slot, rarity, ilvl)));
    }
}

/// Gives the fixture a sane starting HP pool. The real max HP (with
/// prestige/ascension bonuses) is recalculated by the first game tick.
fn sync_hp(state: &mut GameState) {
    let hp = 50 + state.character_level * 10;
    state.combat_state = CombatState::new(hp);
}
