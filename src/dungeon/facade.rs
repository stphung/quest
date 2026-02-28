use crate::dungeon::types::Dungeon;

/// Explicit inputs for the dungeon tick facade.
pub struct DungeonInput<'a> {
    pub dungeon: &'a mut Option<Dungeon>,
    pub zone_id: u32,
    pub prestige_rank: u32,
    pub player_level: u32,
}
