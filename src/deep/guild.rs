//! Guild upgrade system for The Deep.
//!
//! Provides validation and mutation functions for upgrading the guild rank,
//! which persists across all prestiges and unlocks larger rosters and
//! additional concurrent mission slots.

use crate::deep::types::{GuildRank, TheDeepState};

/// Progress information for the next guild rank upgrade.
#[derive(Debug, Clone)]
pub struct GuildUpgradeProgress {
    /// The player's current guild rank.
    pub current_rank: GuildRank,
    /// The next guild rank, or `None` if already at Legion (max rank).
    pub next_rank: Option<GuildRank>,
    /// Warband Marks cost to reach the next rank, or `None` if at max.
    pub marks_needed: Option<u32>,
    /// Layer number that must be cleared to unlock the upgrade, or `None` if at max.
    pub layer_needed: Option<u8>,
    /// Whether the required layer has been cleared.
    pub layer_cleared: bool,
    /// Marks currently held by the player.
    pub marks_available: u32,
}

/// Validates whether the guild can be upgraded to the next rank.
///
/// Checks (in order):
/// 1. Not already at Legion (max rank).
/// 2. The player has enough Warband Marks.
/// 3. The required breakthrough layer has been cleared.
pub fn can_upgrade_guild(state: &TheDeepState) -> Result<(), &'static str> {
    let current = state.account.guild_rank;

    let cost = current
        .upgrade_cost()
        .ok_or("Already at maximum guild rank (Legion)")?;

    if state.run.warband_marks < cost {
        return Err("Insufficient Warband Marks for guild upgrade");
    }

    let next = current.next().expect("upgrade_cost is Some, so next is Some");
    if let Some(required_layer) = next.required_layer() {
        let cleared = state
            .account
            .layers
            .iter()
            .any(|l| l.layer_number == required_layer && l.cleared);

        if !cleared {
            return Err("Required layer has not been cleared");
        }
    }

    Ok(())
}

/// Upgrades the guild to the next rank, deducting the required Marks.
///
/// Calls [`can_upgrade_guild`] for all validation before mutating state.
/// Returns the new [`GuildRank`] on success.
pub fn upgrade_guild(state: &mut TheDeepState) -> Result<GuildRank, &'static str> {
    can_upgrade_guild(state)?;

    let current = state.account.guild_rank;
    let cost = current.upgrade_cost().expect("validated above");
    let next = current.next().expect("validated above");

    state.run.warband_marks -= cost;
    state.account.guild_rank = next;

    Ok(next)
}

/// Returns a snapshot of the current guild upgrade progress.
pub fn guild_upgrade_progress(state: &TheDeepState) -> GuildUpgradeProgress {
    let current_rank = state.account.guild_rank;
    let next_rank = current_rank.next();
    let marks_needed = current_rank.upgrade_cost();
    let layer_needed = next_rank.and_then(|r| r.required_layer());

    let layer_cleared = match layer_needed {
        Some(layer_id) => state
            .account
            .layers
            .iter()
            .any(|l| l.layer_number == layer_id && l.cleared),
        None => false,
    };

    GuildUpgradeProgress {
        current_rank,
        next_rank,
        marks_needed,
        layer_needed,
        layer_cleared,
        marks_available: state.run.warband_marks,
    }
}
