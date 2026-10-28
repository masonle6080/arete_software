//! Arete Library core.
//!
//! Layout:
//! - one module per domain area (`library`, later `thesis`, `essay`, ...).
//!   Each area exposes plain domain command functions. Integration tests in
//!   `tests/` call them directly.
//! - `ipc` is a thin Tauri wrapper over those functions. It holds no rules.
//! - `migrations` applies the ordered SQL files in `migrations/`.

pub mod error;
pub mod ipc;
pub mod migrations;

// Domain areas. One module per area; add yours on its own line.
pub mod library;
pub mod thesis;
pub mod document;
pub mod media;
pub mod note;
pub mod vendor;
pub mod drop;
pub mod moodboard;
pub mod clip;

pub fn run() {
    ipc::run()
}
