#![allow(dead_code)]
use crate::dungeon::types::Dungeon;

/// Explicit inputs for the dungeon tick facade.
pub struct DungeonInput<'a> {
    pub dungeon: &'a mut Option<Dungeon>,
    pub zone_id: u32,
    pub prestige_rank: u32,
    pub player_level: u32,
}

/// Facade: tick dungeon exploration with explicit inputs.
pub fn tick_dungeon_facade(_input: &mut DungeonInput) -> Option<()> {
    todo!("Wire to existing update_dungeon")
}
