//! Thin Tauri IPC layer.
//!
//! Each area has one file here (`ipc/<area>.rs`). Its commands only unpack
//! arguments, call the area's domain functions, and return the result. No
//! rules live here. To add an area: add `mod <area>;` below and list its
//! commands in `generate_handler!` under a `// <area>` comment, one command
//! per line.

mod clip;
mod document;
mod library;
mod note;
mod vendor;
mod thesis;
mod drop;
mod documentary;

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

    /// Like [`AppState::with_library`], for commands that change the library.
    pub fn with_library_mut<T>(&self, f: impl FnOnce(&mut Library) -> CoreResult<T>) -> CoreResult<T> {
        let mut guard = self.library.lock().expect("library lock poisoned");
        f(guard.as_mut().ok_or(CoreError::NoLibraryOpen)?)
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
            library::library_connected,
            library::reconnect_library,
            // vendor
            vendor::list_vendors,
            vendor::create_vendor,
            vendor::update_vendor,
            // note
            note::list_notes,
            note::add_note,
            // thesis
            thesis::create_thesis,
            thesis::move_thesis,
            thesis::list_theses,
            thesis::get_thesis,
            // drop
            drop::create_drop,
            drop::move_drop,
            drop::thesis_drops,
            // document
            document::create_document,
            document::open_document,
            document::save_document,
            document::list_documents,
            // clip
            clip::list_clips,
            clip::import_clip,
            clip::set_clip_date_shot,
            clip::reveal_clip,
            // documentary
            documentary::create_documentary,
            documentary::move_documentary,
            documentary::thesis_documentary,
            documentary::list_documentaries,
            documentary::add_clip_to_documentary,
            documentary::remove_clip_from_documentary,
            documentary::documentary_clips,
            documentary::clip_documentaries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Arete");
}
