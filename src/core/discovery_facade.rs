#![allow(dead_code)]
/// Explicit inputs for discovery rolls.
pub struct DiscoveryInput {
    pub prestige_rank: u32,
    pub character_level: u32,
    pub current_zone_id: u32,
    pub has_active_dungeon: bool,
    pub has_active_fishing: bool,
    pub has_active_minigame: bool,
}

/// Facade: roll for discoveries with explicit inputs.
pub fn roll_discoveries_facade(_input: &DiscoveryInput) -> Option<()> {
    todo!("Wire to existing discovery roll functions")
}
