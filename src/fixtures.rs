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

use crate::character::{AttributeType, Attributes};
use crate::combat::CombatState;
use crate::core::GameState;
use crate::items::{generate_item_with_rng, EquipmentSlot, Rarity};
use crate::zones::{get_zone, ZoneProgression};
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
