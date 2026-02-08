//! Web streaming module for Quest.
//!
//! Allows viewing and controlling the game from a web browser via WebSocket.
//! The game runs natively with full file access, while the browser acts as
//! a remote terminal view.
//!
//! ## Usage
//!
//! Build with web feature:
//! ```sh
//! cargo build --features web
//! ```
//!
//! Run with web server:
//! ```sh
//! ./target/debug/quest --serve        # Default port 3000
//! ./target/debug/quest --serve=8080   # Custom port
//! ```
//!
//! Then open http://localhost:3000 in your browser.

#[cfg(feature = "web")]
mod server;

#[cfg(feature = "web")]
mod backend;

#[cfg(feature = "web")]
pub use backend::TeeWriter;

#[cfg(feature = "web")]
pub use server::{start_web_server, WebServer};
