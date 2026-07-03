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
//! Every preset accepts override flags, and the `custom` scenario starts from
//! a fresh character, so arbitrary states compose from the command line:
//!
//! ```sh
//! mkstate custom --level 60 --prestige 12 --zone 9 --gear epic --boss-ready
//! mkstate midgame --zone 3 --subzone 2 --attrs 25
//! ```
//!
//! Scenarios only write the character save file. Account-level state
//! (haven.json, deep.json, loom, enhancement) is not generated yet — the game
//! treats those systems as undiscovered, which is a valid state.

use quest::character::CharacterManager;
use quest::core::GameState;
use quest::core::KILLS_FOR_BOSS;
use quest::fixtures;
use quest::items::Rarity;
use quest::zones::get_zone;

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
    Scenario {
        name: "custom",
        description: "Fresh character shaped entirely by override flags",
        build: build_fresh,
    },
];

#[derive(Default)]
struct Overrides {
    name: Option<String>,
    level: Option<u32>,
    prestige: Option<u32>,
    ascension: Option<u32>,
    attrs: Option<u32>,
    zone: Option<u32>,
    subzone: Option<u32>,
    gear: Option<Rarity>,
    ilvl: Option<u32>,
    stormglass: Option<u64>,
    stormbreaker: bool,
    boss_ready: bool,
    vessel_signal: bool,
    vessel_launched: bool,
    loom_patterns: Option<usize>,
    voyage: Option<u16>,
    voyage_day: Option<u64>,
    voyage_provisions: Option<f64>,
    voyage_arrived: bool,
}

fn usage() -> ! {
    eprintln!("Usage: mkstate <scenario> [overrides]\n");
    eprintln!("Scenarios:");
    for s in SCENARIOS {
        eprintln!("  {:<10} {}", s.name, s.description);
    }
    eprintln!(
        "\nOverrides (apply on top of any scenario):\n\
         \x20 --name <name>        Character name (defaults to the scenario name)\n\
         \x20 --level <n>          Character level\n\
         \x20 --prestige <n>       Prestige rank\n\
         \x20 --ascension <n>      Ascension level\n\
         \x20 --attrs <n>          All six attributes (clamped to the prestige cap)\n\
         \x20 --zone <n>           Current zone (1-11; 12+ needs Deep account state)\n\
         \x20 --subzone <n>        Current subzone (default 1)\n\
         \x20 --gear <rarity>      Equip all slots: common|magic|rare|epic|legendary|mythic\n\
         \x20 --ilvl <n>           Item level for --gear (default zone * 10)\n\
         \x20 --stormglass <n>     Stormglass balance (also marks it discovered)\n\
         \x20 --stormbreaker       Grant Stormbreaker (Zone 10 boss gate)\n\
         \x20 --boss-ready         Subzone boss spawns on the first tick\n\
         \x20 --vessel-signal      Vessel signal discovered (Z50 boss fallen)\n\
         \x20 --vessel-launched    Vessel already launched (100k PR burned)\n\
         \x20 --loom-patterns <n>  Write loom.json with n completed patterns\n\
         \x20 --voyage <waypoint>  Write voyage.json holding at waypoint 0-37\n\
         \x20                      (implies --vessel-launched; needs QUEST_ACT2=1 to see)\n\
         \x20 --voyage-day <n>     Days since launch (default 2x path length)\n\
         \x20 --voyage-provisions <n>  Provisions in the hold (default 80)\n\
         \x20 --voyage-arrived     Write voyage.json moored at the Tree (spec 7\n\
         \x20                      harbor: manifest, keepsake chart, record)\n\
         \nSet QUEST_DIR to write into an isolated directory."
    );
    std::process::exit(1);
}

fn parse_overrides(args: &[String]) -> Overrides {
    fn value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> &'a str {
        *i += 1;
        args.get(*i).unwrap_or_else(|| {
            eprintln!("error: {flag} requires a value");
            std::process::exit(1);
        })
    }
    fn num<T: std::str::FromStr>(args: &[String], i: &mut usize, flag: &str) -> T {
        let v = value(args, i, flag);
        v.parse().unwrap_or_else(|_| {
            eprintln!("error: invalid value for {flag}: {v}");
            std::process::exit(1);
        })
    }

    let mut o = Overrides::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => o.name = Some(value(args, &mut i, "--name").to_string()),
            "--level" => o.level = Some(num(args, &mut i, "--level")),
            "--prestige" => o.prestige = Some(num(args, &mut i, "--prestige")),
            "--ascension" => o.ascension = Some(num(args, &mut i, "--ascension")),
            "--attrs" => o.attrs = Some(num(args, &mut i, "--attrs")),
            "--zone" => o.zone = Some(num(args, &mut i, "--zone")),
            "--subzone" => o.subzone = Some(num(args, &mut i, "--subzone")),
            "--ilvl" => o.ilvl = Some(num(args, &mut i, "--ilvl")),
            "--stormglass" => o.stormglass = Some(num(args, &mut i, "--stormglass")),
            "--stormbreaker" => o.stormbreaker = true,
            "--boss-ready" => o.boss_ready = true,
            "--vessel-signal" => o.vessel_signal = true,
            "--vessel-launched" => o.vessel_launched = true,
            "--loom-patterns" => o.loom_patterns = Some(num(args, &mut i, "--loom-patterns")),
            "--voyage" => o.voyage = Some(num(args, &mut i, "--voyage")),
            "--voyage-arrived" => o.voyage_arrived = true,
            "--voyage-day" => o.voyage_day = Some(num(args, &mut i, "--voyage-day")),
            "--voyage-provisions" => {
                o.voyage_provisions = Some(num(args, &mut i, "--voyage-provisions"))
            }
            "--gear" => {
                o.gear = Some(match value(args, &mut i, "--gear") {
                    "common" => Rarity::Common,
                    "magic" => Rarity::Magic,
                    "rare" => Rarity::Rare,
                    "epic" => Rarity::Epic,
                    "legendary" => Rarity::Legendary,
                    "mythic" => Rarity::Mythic,
                    other => {
                        eprintln!("error: unknown rarity: {other}");
                        std::process::exit(1);
                    }
                })
            }
            other => {
                eprintln!("error: unknown flag: {other}");
                usage();
            }
        }
        i += 1;
    }
    o
}

fn apply_overrides(state: &mut GameState, o: &Overrides) {
    if let Some(p) = o.prestige {
        state.prestige_rank = p;
        state.total_prestige_count = state.total_prestige_count.max(p as u64);
    }
    if let Some(level) = o.level {
        state.character_level = level;
        fixtures::sync_hp(state);
    }
    if let Some(asc) = o.ascension {
        state.ascension_level = asc;
    }
    if let Some(attrs) = o.attrs {
        fixtures::set_attributes(state, attrs.min(state.get_attribute_cap()));
    }
    if o.zone.is_some() || o.subzone.is_some() {
        let zone = o.zone.unwrap_or(state.zone_progression.current_zone_id);
        if get_zone(zone).is_none() {
            eprintln!("error: zone {zone} does not exist");
            std::process::exit(1);
        }
        if zone > 11 {
            eprintln!(
                "warning: zones 12+ are gated by Deep account state, which \
                 mkstate does not generate; the game may not honor this placement"
            );
        }
        fixtures::advance_to_zone(state, zone, o.subzone.unwrap_or(1));
    }
    if let Some(rarity) = o.gear {
        let ilvl = o
            .ilvl
            .unwrap_or(state.zone_progression.current_zone_id * 10);
        fixtures::equip_all(state, rarity, rarity, ilvl, &mut rand::rng());
    }
    if let Some(sg) = o.stormglass {
        state.stormglass = sg;
        state.stormglass_discovered = true;
    }
    if o.stormbreaker {
        state.zone_progression.has_stormbreaker = true;
    }
    if o.boss_ready {
        state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;
    }
    if o.vessel_signal {
        state.vessel_signal_discovered = true;
    }
    if o.vessel_launched || o.voyage.is_some() || o.voyage_arrived {
        state.vessel_signal_discovered = true;
        state.vessel_launched = true;
    }
}

/// Writes voyage.json holding station at the given waypoint.
fn write_voyage_state(state: &GameState, o: &Overrides) {
    let waypoint = o.voyage.expect("checked by caller");
    if waypoint as usize >= quest::vessel::route::WAYPOINTS.len() {
        eprintln!(
            "error: waypoint {waypoint} does not exist (0-{})",
            quest::vessel::route::WAYPOINTS.len() - 1
        );
        std::process::exit(1);
    }
    let waypoint = quest::vessel::route::WaypointId(waypoint);
    // A loose default: roughly two days per waypoint passed.
    let day = o.voyage_day.unwrap_or(2 * waypoint.0 as u64 + 2);
    let voyage = fixtures::voyage_holding_at(
        state.character_id.clone(),
        waypoint,
        day,
        o.voyage_provisions.unwrap_or(80.0),
        Utc::now(),
    );
    quest::vessel::persistence::save_voyage(&voyage).expect("failed to write voyage.json");
    println!(
        "Wrote voyage.json (holding at {}, day {}, {} provisions)",
        quest::vessel::route::waypoint(waypoint).name,
        day,
        voyage.provisions_display()
    );
}

/// Writes loom.json with the Loom discovered and `completed` patterns marked
/// complete (0-28). Needed for Vessel launch-gate states.
fn write_loom_state(completed: usize) {
    use quest::loom;
    let mut loom_state = loom::LoomState::new();
    loom::initialize_loom(&mut loom_state);
    loom::complete_discovery(&mut loom_state);
    for p in loom_state.persistent.patterns.iter_mut().take(completed) {
        p.completed = true;
    }
    loom::save_loom(&loom_state).expect("failed to write loom.json");
    println!(
        "Wrote loom.json ({} / {} patterns complete)",
        completed.min(loom_state.persistent.patterns.len()),
        loom_state.persistent.patterns.len()
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--list" || args[1] == "-l" || args[1] == "--help" {
        usage();
    }

    let scenario_name = args[1].as_str();
    let Some(scenario) = SCENARIOS.iter().find(|s| s.name == scenario_name) else {
        eprintln!("Unknown scenario: {scenario_name}\n");
        usage();
    };

    let overrides = parse_overrides(&args[2..]);
    let char_name = overrides
        .name
        .clone()
        .unwrap_or_else(|| default_name(scenario_name));

    let mut state = (scenario.build)(char_name.clone());
    apply_overrides(&mut state, &overrides);

    let manager = CharacterManager::new().expect("failed to open quest dir");
    manager
        .save_character(&state)
        .expect("failed to write save file");

    if let Some(patterns) = overrides.loom_patterns {
        write_loom_state(patterns);
    }

    if overrides.voyage_arrived {
        let voyage = fixtures::voyage_arrived(state.character_id.clone(), Utc::now());
        quest::vessel::persistence::save_voyage(&voyage).expect("failed to write voyage.json");
        println!("Wrote voyage.json (arrived at the Tree, finale read)");
    } else if overrides.voyage.is_some() {
        write_voyage_state(&state, &overrides);
    }

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

// Scenario builders live in quest::fixtures (shared with the UI snapshot
// tests); these wrappers just supply the wall clock and thread RNG.
fn build_fresh(name: String) -> GameState {
    fixtures::fresh(&name, Utc::now().timestamp())
}

fn build_midgame(name: String) -> GameState {
    fixtures::midgame(&name, Utc::now().timestamp(), &mut rand::rng())
}

fn build_endgame(name: String) -> GameState {
    fixtures::endgame(&name, Utc::now().timestamp(), &mut rand::rng())
}

fn build_boss(name: String) -> GameState {
    fixtures::boss(&name, Utc::now().timestamp(), &mut rand::rng())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quest::character::AttributeType;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_overrides_all_flags() {
        let o = parse_overrides(&args(&[
            "--name",
            "Hero",
            "--level",
            "60",
            "--prestige",
            "12",
            "--ascension",
            "3",
            "--attrs",
            "70",
            "--zone",
            "9",
            "--subzone",
            "3",
            "--gear",
            "epic",
            "--ilvl",
            "95",
            "--stormglass",
            "3000",
            "--stormbreaker",
            "--boss-ready",
        ]));
        assert_eq!(o.name.as_deref(), Some("Hero"));
        assert_eq!(o.level, Some(60));
        assert_eq!(o.prestige, Some(12));
        assert_eq!(o.ascension, Some(3));
        assert_eq!(o.attrs, Some(70));
        assert_eq!(o.zone, Some(9));
        assert_eq!(o.subzone, Some(3));
        assert_eq!(o.gear, Some(Rarity::Epic));
        assert_eq!(o.ilvl, Some(95));
        assert_eq!(o.stormglass, Some(3000));
        assert!(o.stormbreaker);
        assert!(o.boss_ready);
    }

    #[test]
    fn test_apply_overrides_composes_state() {
        let mut state = build_fresh("T".into());
        apply_overrides(
            &mut state,
            &parse_overrides(&args(&[
                "--level",
                "60",
                "--prestige",
                "12",
                "--zone",
                "9",
                "--subzone",
                "3",
                "--gear",
                "epic",
                "--stormglass",
                "500",
                "--boss-ready",
            ])),
        );
        assert_eq!(state.character_level, 60);
        assert_eq!(state.prestige_rank, 12);
        assert_eq!(state.zone_progression.current_zone_id, 9);
        assert_eq!(state.zone_progression.current_subzone_id, 3);
        assert!(state.zone_progression.unlocked_zones.contains(&9));
        assert_eq!(state.zone_progression.kills_in_subzone, KILLS_FOR_BOSS);
        assert!(state.zone_progression.should_spawn_boss());
        assert_eq!(state.stormglass, 500);
        assert!(state.stormglass_discovered);
        assert_eq!(state.equipment.iter_equipped().count(), 7);
        // --ilvl omitted: gear defaults to zone * 10.
        for item in state.equipment.iter_equipped() {
            assert_eq!(item.ilvl, 90);
        }
    }

    #[test]
    fn test_attrs_clamped_to_prestige_cap() {
        let mut state = build_fresh("T".into());
        apply_overrides(
            &mut state,
            &parse_overrides(&args(&["--prestige", "2", "--attrs", "999"])),
        );
        // Cap at P2 is 20 + 2*5 = 30.
        let cap = state.get_attribute_cap();
        assert_eq!(cap, 30);
        for attr in AttributeType::all() {
            assert_eq!(state.attributes.get(attr), cap);
        }
    }

    #[test]
    fn test_advance_to_zone_defeats_prior_bosses() {
        let mut state = build_fresh("T".into());
        fixtures::advance_to_zone(&mut state, 3, 2);
        // Every subzone boss in zones 1-2 and subzone 1 of zone 3 is defeated.
        for z in 1..=2 {
            for sub in &get_zone(z).unwrap().subzones {
                assert!(state
                    .zone_progression
                    .defeated_bosses
                    .contains(&(z, sub.id)));
            }
        }
        assert!(state.zone_progression.defeated_bosses.contains(&(3, 1)));
        assert!(!state.zone_progression.defeated_bosses.contains(&(3, 2)));
    }

    #[test]
    fn test_presets_roundtrip_through_save_json() {
        // Fixtures must serialize cleanly through the same serde path the
        // game's persistence uses.
        for scenario in SCENARIOS {
            let state = (scenario.build)("Roundtrip".into());
            let json = serde_json::to_string(&state).expect(scenario.name);
            let back: GameState = serde_json::from_str(&json).expect(scenario.name);
            assert_eq!(back.character_level, state.character_level);
            assert_eq!(back.prestige_rank, state.prestige_rank);
            assert_eq!(
                back.zone_progression.current_zone_id,
                state.zone_progression.current_zone_id
            );
        }
    }
}
