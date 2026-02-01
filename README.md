# Idle RPG

A terminal-based idle RPG game built with Rust.

## Prerequisites

- Rust toolchain (rustc + cargo)
- Install from: https://rustup.rs/

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Project Structure

- `src/main.rs` - Entry point
- `src/constants.rs` - Game configuration constants
- `Cargo.toml` - Project dependencies

## Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **serde** - Serialization framework
- **bincode** - Binary serialization
- **sha2** - Hashing for save integrity
- **chrono** - Time handling for offline progression
- **rand** - Random number generation
- **directories** - Cross-platform directory paths

## Status

Project is in initial development phase.
