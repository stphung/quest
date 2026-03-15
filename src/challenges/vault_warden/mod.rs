//! Vault Warden challenge — Sokoban puzzle.

pub mod levels;
pub mod logic;
pub mod types;

pub use logic::{process_input, start_vault_warden_game};
pub use types::{VaultWardenDifficulty, VaultWardenGame, VaultWardenInput, VaultWardenResult};

impl_apply_game_result! {
    fn apply_vault_warden_result;
    variant: VaultWarden;
    result_body: |result, _state, _reward| {
        use VaultWardenResult::*;
        match result {
            Win => (true, ""),
            Loss => (false, "The relics remain scattered."),
        }
    }
    game_type: crate::achievements::MinigameType::VaultWarden;
    icon: "\u{1F512}";
    win_message: "All relics placed!";
}
