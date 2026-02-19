//! Combat system types and logic.

pub(crate) mod damage;
pub mod events;
pub mod logic;
pub(crate) mod regen;
pub mod types;

pub use events::*;
pub use types::*;
