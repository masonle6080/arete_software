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
mod lifecycle;

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
pub mod style;
pub mod sample;
pub mod design_decision;
pub mod decoration;
pub mod blank;
pub mod documentary;
pub mod export;
pub mod tech_pack;
pub mod cost_sheet;
pub mod production_order;

pub fn run() {
    ipc::run()
}
