//! resolve-merge-conflicts core library.
//!
//! Everything that decides *what* to do lives here and performs no terminal I/O, so the same
//! implementation serves the interactive CLI (`src/main.rs`) and the tests.

pub mod agent;
pub mod config;
pub mod conflicts;
pub mod git;
pub mod prompt;
pub mod verify;

/// The version reported by `--version`, baked in from Cargo.toml at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
