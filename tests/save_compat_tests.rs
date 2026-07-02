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

/// Character saves survive a load → save → load cycle without losing data.
#[test]
fn character_corpus_round_trips() {
    let manager = CharacterManager::with_dir(corpus_dir()).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let scratch = CharacterManager::with_dir(tmp.path().to_path_buf()).unwrap();

    for name in ["fresh.json", "midgame.json", "endgame.json", "boss.json"] {
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
}
