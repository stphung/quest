//! Fishing system: types, generation, and logic.

#![allow(unused_imports)]

pub mod discovery;
pub mod drops;
pub mod generation;
pub mod logic;
pub mod rank;
pub mod types;

pub use discovery::*;
pub use generation::*;
pub use logic::*;
pub use rank::*;
pub use types::*;
