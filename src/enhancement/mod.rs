// Allow dead code: many functions are defined now but will be used by later tasks
// (blacksmith UI, combat integration, item display).
#[allow(dead_code)]
pub mod logic;
pub mod persistence;
#[allow(dead_code)]
pub mod types;

pub use logic::*;
pub use persistence::*;
pub use types::*;
