use crate::character::attributes::Attributes;
use serde::{Deserialize, Serialize};

/// Character identity and progression fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerIdentity {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
}

impl Default for PlayerIdentity {
    fn default() -> Self {
        Self {
            character_id: String::new(),
            character_name: String::new(),
            character_level: 1,
            character_xp: 0,
            attributes: Attributes::new(),
            prestige_rank: 0,
            total_prestige_count: 0,
        }
    }
}
