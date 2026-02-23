//! Time Vault — git-based save versioning system.
//!
//! Every meaningful game event (prestige, boss defeat, etc.) creates a git
//! commit containing the full save state. Players can browse, restore, and
//! fork save branches through the Time Vault overlay.

pub mod git;
pub mod types;

pub use git::validate_branch_name;
#[allow(unused_imports)]
pub use git::HistoryError;
pub use git::HistoryRepo;
pub use types::*;
