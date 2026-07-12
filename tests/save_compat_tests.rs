//! Save-format compatibility corpus.
//!
//! `tests/fixtures/saves/v1/` holds save files committed in the format the
//! game wrote at the time the corpus was generated. These tests load every
//! file through the same deserialization paths the game uses; if a change
//! to a serialized type breaks loading of an existing save, a test here
//! fails.
//!
//! Why this matters more than it looks: the account-state loaders
//! (`load_deep`, `load_haven`, `load_achievements`, …) fall back to a
//! default state when parsing fails — in the real game a serde break does
//! not crash, it SILENTLY WIPES the player's progress. This corpus is what
//! turns that silent wipe into a red test.
//!
//! Rules for the corpus (see also tests/fixtures/saves/README.md):
//! - Committed files are frozen. Never edit or regenerate them to make a
//!   failing test pass — a failure means you broke compatibility with
//!   existing player saves; fix it with `#[serde(default)]`, `alias`, or a
//!   migration instead.
//! - After an intentional, migration-backed format change, add a NEW
//!   corpus generation (v2/, v3/, …) via `regenerate_save_corpus` pointed
//!   at a new directory, and keep the old ones loading.

use quest::character::CharacterManager;
use quest::core::KILLS_FOR_BOSS;
use quest::items::Rarity;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::PathBuf;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("saves")
        .join("v1")
}

fn read(name: &str) -> String {
    let path = corpus_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("corpus file {} missing: {e}", path.display()))
}

/// Deserializes a corpus file, failing loudly (the game would fall back to
/// a default state here and silently discard the player's progress).
fn load<T: DeserializeOwned>(name: &str) -> T {
    serde_json::from_str(&read(name)).unwrap_or_else(|e| {
        panic!("corpus file {name} no longer deserializes — this breaks existing saves: {e}")
    })
}

// ── Character saves (CharacterSaveData via CharacterManager) ─────────────

#[test]
fn character_corpus_loads() {
    let manager = CharacterManager::with_dir(corpus_dir()).unwrap();

    let fresh = manager.load_character("fresh.json").unwrap();
    assert_eq!(fresh.character_level, 1);
    assert_eq!(fresh.prestige_rank, 0);
    assert_eq!(fresh.zone_progression.current_zone_id, 1);
    assert_eq!(fresh.equipment.iter_equipped().count(), 0);

    let midgame = manager.load_character("midgame.json").unwrap();
    assert_eq!(midgame.character_level, 45);
    assert_eq!(midgame.prestige_rank, 5);
    assert_eq!(midgame.zone_progression.current_zone_id, 8);
    assert_eq!(midgame.zone_progression.current_subzone_id, 2);
    assert_eq!(midgame.equipment.iter_equipped().count(), 7);
    assert!(midgame.stormglass_discovered);
    assert_eq!(midgame.stormglass, 750);
    let weapon = midgame.equipment.weapon.as_ref().expect("weapon equipped");
    assert_eq!(weapon.rarity, Rarity::Epic);
    assert_eq!(weapon.ilvl, 80);

    let endgame = manager.load_character("endgame.json").unwrap();
    assert_eq!(endgame.character_level, 80);
    assert_eq!(endgame.prestige_rank, 25);
    assert_eq!(endgame.ascension_level, 3);
    assert_eq!(endgame.zone_progression.current_zone_id, 11);
    assert!(endgame.zone_progression.has_stormbreaker);
    assert_eq!(endgame.stormglass, 25_000);

    let boss = manager.load_character("boss.json").unwrap();
    assert_eq!(boss.zone_progression.kills_in_subzone, KILLS_FOR_BOSS);
    assert!(boss.zone_progression.should_spawn_boss());
}

/// An endgame character carrying an in-progress dungeon, etched Storm
/// Sigils, and an equipped god item — none of which any other corpus file
/// exercises.
#[test]
fn act1_systems_corpus_loads() {
    let manager = CharacterManager::with_dir(corpus_dir()).unwrap();
    let state = manager.load_character("act1systems.json").unwrap();

    let dungeon = state.active_dungeon.as_ref().expect("dungeon in progress");
    assert_eq!(dungeon.player_position, (1, 0));
    assert_eq!(dungeon.rooms_cleared, 1);
    assert!(!dungeon.current_room_cleared);

    assert_eq!(state.storm_sigils.slots_unlocked, 3);
    assert_eq!(state.storm_sigils.etched_count(), 2);

    let ring = state
        .equipment
        .ring
        .as_ref()
        .expect("Megingjord equipped in the ring slot");
    assert_eq!(
        ring.god_item_id,
        Some(quest::god_items::GodItemId::Megingjord)
    );
    assert_eq!(ring.rarity, Rarity::Mythic);
}

/// Character saves survive a load → save → load cycle without losing data.
#[test]
fn character_corpus_round_trips() {
    let manager = CharacterManager::with_dir(corpus_dir()).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let scratch = CharacterManager::with_dir(tmp.path().to_path_buf()).unwrap();

    for name in [
        "fresh.json",
        "midgame.json",
        "endgame.json",
        "boss.json",
        "act1systems.json",
    ] {
        let original = manager.load_character(name).unwrap();
        scratch.save_character(&original).unwrap();
        let reloaded = scratch.load_character(name).unwrap();

        assert_eq!(reloaded.character_level, original.character_level, "{name}");
        assert_eq!(reloaded.character_xp, original.character_xp, "{name}");
        assert_eq!(reloaded.prestige_rank, original.prestige_rank, "{name}");
        assert_eq!(reloaded.ascension_level, original.ascension_level, "{name}");
        assert_eq!(reloaded.attributes, original.attributes, "{name}");
        assert_eq!(
            reloaded.zone_progression.current_zone_id, original.zone_progression.current_zone_id,
            "{name}"
        );
        assert_eq!(
            reloaded.zone_progression.kills_in_subzone, original.zone_progression.kills_in_subzone,
            "{name}"
        );
        let orig_items: Vec<_> = original
            .equipment
            .iter_equipped()
            .map(|i| (i.display_name.clone(), i.rarity, i.ilvl, i.tier))
            .collect();
        let new_items: Vec<_> = reloaded
            .equipment
            .iter_equipped()
            .map(|i| (i.display_name.clone(), i.rarity, i.ilvl, i.tier))
            .collect();
        assert_eq!(new_items, orig_items, "{name}");
    }
}

// ── Account-level saves ───────────────────────────────────────────────────

#[test]
fn deep_corpus_loads() {
    let deep: quest::deep::DeepState = load("deep.json");
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.deepest_layer_reached, 12);
    assert_eq!(deep.prestige.active_missions.len(), 1);
    assert_eq!(
        deep.prestige.active_missions[0].status,
        quest::deep::MissionStatus::Active
    );
    assert_eq!(deep.prestige.warband_marks, 45);
}

#[test]
fn haven_corpus_loads() {
    let haven: quest::haven::Haven = load("haven.json");
    assert!(haven.discovered);
    assert_eq!(
        haven.rooms.get(&quest::haven::HavenRoomId::Hearthstone),
        Some(&4)
    );
}

#[test]
fn loom_corpus_loads() {
    // Through the real loader, which also applies version gating and
    // float sanitization.
    let loom = quest::loom::persistence::load_loom_from_path(&corpus_dir().join("loom.json"));
    assert!(
        loom.persistent.discovered,
        "current-version loom save was reset on load"
    );
    assert_eq!(loom.persistent.shuttles.len(), 1);
    assert_eq!(
        loom.persistent.nodes.iter().filter(|n| n.unlocked).count(),
        2
    );
}

/// Pre-version-2 loom saves are intentionally reset (pattern definitions
/// changed). The loader must do that cleanly rather than erroring.
#[test]
fn loom_legacy_save_resets_cleanly() {
    let loom =
        quest::loom::persistence::load_loom_from_path(&corpus_dir().join("loom_legacy_v1.json"));
    assert!(
        !loom.persistent.discovered,
        "legacy loom save should reset to defaults"
    );
    assert!(loom.persistent.shuttles.is_empty());
}

#[test]
fn enhancement_corpus_loads() {
    let enhancement: quest::enhancement::EnhancementProgress = load("enhancement.json");
    assert!(enhancement.discovered);
    assert_eq!(enhancement.levels, [7, 5, 4, 3, 3, 2, 1]);
    assert_eq!(enhancement.highest_level_reached, 7);
}

#[test]
fn achievements_corpus_loads() {
    let achievements: quest::achievements::Achievements = load("achievements.json");
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::PowerCoreI));
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::PowerCoreII));
    assert_eq!(achievements.total_kills, 12_345);
    assert_eq!(achievements.highest_prestige_rank, 25);
}

/// Serialize→deserialize→serialize is a fixed point for every account
/// type: nothing is lost or reshaped after the first write in the current
/// format.
#[test]
fn account_corpus_serde_is_stable() {
    fn check<T: DeserializeOwned + Serialize>(name: &str) {
        let typed: T = load(name);
        let v1 = serde_json::to_value(&typed).unwrap();
        let typed2: T = serde_json::from_value(v1.clone()).unwrap();
        let v2 = serde_json::to_value(&typed2).unwrap();
        assert_eq!(v1, v2, "{name}: value changed across a serde round trip");
    }
    check::<quest::deep::DeepState>("deep.json");
    check::<quest::haven::Haven>("haven.json");
    check::<quest::loom::types::LoomState>("loom.json");
    check::<quest::enhancement::EnhancementProgress>("enhancement.json");
    check::<quest::achievements::Achievements>("achievements.json");
}

// ── Act 2 account files (voyage.json / colony.json, vessel::persistence) ──

/// Vessel saves load through the REAL load paths — including the
/// `character_id` keying — not bare serde. Added to the v1 corpus
/// 2026-07-12 (additive coverage of previously-uncovered state, not a
/// format migration); generated by `generate_vessel_corpus_v1_addition`.
#[test]
fn vessel_corpus_loads_through_real_paths() {
    use quest::vessel::persistence::{load_colony_from_path, load_voyage_from_path};
    use quest::vessel::voyage::{Trim, VoyagePhase};

    let vpath = corpus_dir().join("voyage.json");
    let v = load_voyage_from_path(&vpath, "corpus-fixture").expect(
        "voyage.json no longer loads for its character — this breaks a player's \
         mid-crossing save (the game would silently restart the crossing)",
    );
    assert!(
        matches!(v.phase, VoyagePhase::Traveling { .. }),
        "mid-leg fixture is underway"
    );
    assert_eq!(v.crossing_number, 1, "the maiden voyage");
    assert_eq!(v.trim, Trim::Quiet);
    assert!(
        v.provisions > 0.0 && v.provisions < v.provisions_cap,
        "partial hold survived ({} of {})",
        v.provisions,
        v.provisions_cap
    );
    assert!(!v.rumors.is_empty(), "the learned rumor survived");
    assert!(!v.refits.is_empty(), "the chosen refit survived");
    assert!(
        v.souls.iter().filter(|s| s.station.is_some()).count() >= 2,
        "staffed stations survived"
    );
    assert!(v.letters_received >= 1, "delivered mail survived");
    // The keying is part of the load path: another character never
    // inherits this crossing.
    assert!(load_voyage_from_path(&vpath, "someone-else").is_none());

    let cpath = corpus_dir().join("colony.json");
    let c = load_colony_from_path(&cpath, "corpus-fixture").expect(
        "colony.json no longer loads for its character — this breaks a player's \
         ferry-era save (the game would silently refound the colony)",
    );
    assert_eq!(c.drive_level, 10);
    assert_eq!(c.cap_level, 10);
    assert_eq!(c.ward_level, 3);
    assert_eq!(c.crossings_completed, 16);
    assert_eq!(c.souls_delivered, 30_000);
    assert!(
        c.has_district(quest::vessel::colony::District::Hearth),
        "founded districts survived"
    );
    assert!(
        c.dock.is_some(),
        "the Dock phase (Riftglass charging) survived"
    );
    assert!(load_colony_from_path(&cpath, "someone-else").is_none());
}

// ── Corpus generator ──────────────────────────────────────────────────────

/// Regenerates `tests/fixtures/saves/v1/` in the CURRENT save format.
///
/// Run manually with:
/// `cargo test --test save_compat_tests regenerate_save_corpus -- --ignored`
///
/// Only do this when creating a corpus generation for a NEW format era —
/// never to silence a failing compatibility test. `loom_legacy_v1.json` is
/// hand-written and deliberately not regenerated.
#[test]
#[ignore]
fn regenerate_save_corpus() {
    use quest::fixtures;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    const CREATED_AT: i64 = 1_749_000_000;
    const FROZEN_MILLIS: i64 = 1_750_000_000_123;
    let now = chrono::DateTime::from_timestamp_millis(FROZEN_MILLIS).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let dir = corpus_dir();
    std::fs::create_dir_all(&dir).unwrap();

    // Character saves through the real save path (CharacterSaveData).
    let manager = CharacterManager::with_dir(dir.clone()).unwrap();
    let mut states = [
        fixtures::fresh("Fresh", CREATED_AT),
        fixtures::midgame("Midgame", CREATED_AT, &mut rng),
        fixtures::endgame("Endgame", CREATED_AT, &mut rng),
        fixtures::boss("Boss", CREATED_AT, &mut rng),
    ];
    for (i, state) in states.iter_mut().enumerate() {
        // Fixed ids keep regeneration diffs minimal.
        state.character_id = format!("00000000-0000-0000-0000-00000000000{i}");
        manager.save_character(state).unwrap();
    }

    // Endgame character with an in-progress dungeon, etched Storm Sigils,
    // and an equipped god item layered on — the combination of Act 1
    // systems that have no other save-compat coverage.
    let mut act1_systems = fixtures::endgame("Act1Systems", CREATED_AT, &mut rng);
    act1_systems.character_id = "00000000-0000-0000-0000-000000000004".to_string();
    act1_systems.active_dungeon = Some(fixtures::dungeon_in_progress(11));
    fixtures::unlock_storm_sigils(&mut act1_systems);
    fixtures::equip_god_item(&mut act1_systems, quest::god_items::GodItemId::Megingjord);
    manager.save_character(&act1_systems).unwrap();

    // Account-level saves. Serializing via Value sorts map keys, keeping
    // the committed files diff-stable across regenerations.
    fn write<T: Serialize>(dir: &std::path::Path, name: &str, value: &T) {
        let v = serde_json::to_value(value).unwrap();
        std::fs::write(dir.join(name), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    }

    write(&dir, "deep.json", &fixtures::deep_state_active(now));
    write(&dir, "haven.json", &fixtures::haven_built());
    write(&dir, "loom.json", &fixtures::loom_state_with_shuttle());

    let mut enhancement = quest::enhancement::EnhancementProgress::new();
    enhancement.discovered = true;
    enhancement.levels = [7, 5, 4, 3, 3, 2, 1];
    enhancement.highest_level_reached = 7;
    enhancement.total_attempts = 40;
    enhancement.total_successes = 25;
    enhancement.total_failures = 15;
    write(&dir, "enhancement.json", &enhancement);

    let mut achievements = quest::achievements::Achievements::default();
    fixtures::unlock_power_cores(&mut achievements, 2, now.timestamp());
    achievements.total_kills = 12_345;
    achievements.highest_prestige_rank = 25;
    achievements.highest_level = 80;
    write(&dir, "achievements.json", &achievements);

    write_vessel_corpus(&dir, now);
}

/// The Act 2 account files: a mid-crossing voyage and a mid-era, docked
/// colony, both keyed to the synthetic `corpus-fixture` character. Shared
/// by `regenerate_save_corpus` (future format eras) and the one-shot v1
/// addition below.
fn write_vessel_corpus(dir: &std::path::Path, now: chrono::DateTime<chrono::Utc>) {
    use quest::fixtures;
    use quest::vessel::refits::RefitId;
    use quest::vessel::souls::{SoulId, Station};

    fn write<T: Serialize>(dir: &std::path::Path, name: &str, value: &T) {
        let v = serde_json::to_value(value).unwrap();
        std::fs::write(dir.join(name), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    }

    // Mid-leg on the Drowned Choir road: underway, Quiet trim, a rumor
    // aboard, stations staffed, a refit chosen, mail delivered, hold partly
    // eaten — one of everything a mid-crossing save carries.
    let mut voyage = fixtures::voyage_mid_leg(now);
    voyage.character_id = "corpus-fixture".to_string();
    voyage.set_station(SoulId(0), Some(Station::Helm));
    voyage.set_station(SoulId(1), Some(Station::Tender));
    voyage.refits.push(RefitId::StormSail);
    voyage.letters_received = voyage.letters_received.max(1);
    write(dir, "voyage.json", &voyage);

    // Mid-era, all three yards raised, districts founded, docked with
    // Riftglass charging.
    let mut colony = fixtures::colony_midera();
    colony.character_id = "corpus-fixture".to_string();
    colony.ward_level = 3;
    colony.dock(now);
    write(dir, "colony.json", &colony);
}

/// One-shot generator for the 2026-07-12 v1 corpus ADDITION (voyage.json,
/// colony.json). Run manually with:
/// `cargo test --test save_compat_tests generate_vessel_corpus_v1_addition -- --ignored`
///
/// Unlike `regenerate_save_corpus` this writes ONLY the two vessel files —
/// the rest of v1 stays frozen. Kept for provenance; do not re-run to
/// silence a failing `vessel_corpus_loads_through_real_paths`.
#[test]
#[ignore]
fn generate_vessel_corpus_v1_addition() {
    const FROZEN_MILLIS: i64 = 1_750_000_000_123;
    let now = chrono::DateTime::from_timestamp_millis(FROZEN_MILLIS).unwrap();
    write_vessel_corpus(&corpus_dir(), now);
}
