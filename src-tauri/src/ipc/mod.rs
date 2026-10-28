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
mod style;
mod sample;
mod media;
mod moodboard;

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
            // style
            style::create_style,
            style::move_style,
            style::get_style,
            style::drop_styles,
            style::attach_style_files,
            style::style_files,
            // sample
            sample::request_sample,
            sample::style_samples,
            sample::set_sample_feedback,
            sample::attach_sample_photos,
            sample::decide_sample,
            // media
            media::import_media_bytes,
            media::read_media,
            // moodboard
            moodboard::create_moodboard,
            moodboard::list_moodboards,
            moodboard::load_moodboard_scene,
            moodboard::save_moodboard_scene,
            // document
            document::create_document,
            document::open_document,
            document::save_document,
            document::list_documents,
            document::link_essay_to_thesis,
            document::unlink_essay_from_thesis,
            document::theses_of_essay,
            document::essays_of_thesis,
            // clip
            clip::list_clips,
            clip::import_clip,
            clip::set_clip_date_shot,
            clip::reveal_clip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Arete");
}
