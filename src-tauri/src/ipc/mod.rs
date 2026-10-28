//! Thin Tauri IPC layer.
//!
//! Each area has one file here (`ipc/<area>.rs`). Its commands only unpack
//! arguments, call the area's domain functions, and return the result. No
//! rules live here. To add an area: add `mod <area>;` below and list its
//! commands in `generate_handler!` under a `// <area>` comment, one command
//! per line.

mod library;
mod vendor;

use std::sync::Mutex;

use tauri::Manager;

use crate::error::{CoreError, CoreResult};
use crate::library::{AppConfig, Library};

/// Shared app state behind every command.
pub struct AppState {
    pub config: AppConfig,
    library: Mutex<Option<Library>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        AppState { config, library: Mutex::new(None) }
    }

    /// Runs `f` with the open library, or fails with `NoLibraryOpen`.
    pub fn with_library<T>(&self, f: impl FnOnce(&Library) -> CoreResult<T>) -> CoreResult<T> {
        let guard = self.library.lock().expect("library lock poisoned");
        f(guard.as_ref().ok_or(CoreError::NoLibraryOpen)?)
    }

    /// Replaces the open library.
    pub fn set_library(&self, lib: Library) {
        *self.library.lock().expect("library lock poisoned") = Some(lib);
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            app.manage(AppState::new(AppConfig::new(dir)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // library
            library::create_library,
            library::open_library,
            library::reopen_last_library,
            library::current_library,
            // vendor
            vendor::list_vendors,
            vendor::create_vendor,
            vendor::update_vendor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Arete");
}
