//! IPC for the Clip pool. See `src/areas/clips/api.ts` for the callers.

use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use super::AppState;
use crate::clip::{self, Clip, ClipFilter, FfmpegProbe};
use crate::document::Passage;
use crate::error::{CoreError, CoreResult};
use crate::library::Library;

/// Lets the webview load thumbnails from the media folder (asset protocol).
fn allow_media(app: &AppHandle, lib: &Library) {
    let _ = app.asset_protocol_scope().allow_directory(lib.media_dir(), false);
}

#[tauri::command]
pub fn list_clips(app: AppHandle, state: State<AppState>) -> CoreResult<Vec<Clip>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        clip::list_clips(lib)
    })
}

/// Async so a long copy or ffmpeg run does not block the main thread.
#[tauri::command]
pub async fn import_clip(app: AppHandle, state: State<'_, AppState>, path: PathBuf) -> CoreResult<Clip> {
    let probe = FfmpegProbe::locate()?;
    state.with_library(|lib| {
        allow_media(&app, lib);
        clip::import_clip(lib, &path, &probe)
    })
}

#[tauri::command]
pub fn set_clip_date_shot(state: State<AppState>, id: i64, date_shot: Option<String>) -> CoreResult<Clip> {
    state.with_library(|lib| clip::set_date_shot(lib, id, date_shot.as_deref()))
}

#[tauri::command]
pub fn add_clip_tag(state: State<AppState>, id: i64, tag: String) -> CoreResult<Clip> {
    state.with_library(|lib| clip::add_tag(lib, id, &tag))
}

#[tauri::command]
pub fn remove_clip_tag(state: State<AppState>, id: i64, tag: String) -> CoreResult<Clip> {
    state.with_library(|lib| clip::remove_tag(lib, id, &tag))
}

#[tauri::command]
pub fn set_clip_notes(state: State<AppState>, id: i64, notes: String) -> CoreResult<Clip> {
    state.with_library(|lib| clip::set_notes(lib, id, &notes))
}

#[tauri::command]
pub fn all_clip_tags(state: State<AppState>) -> CoreResult<Vec<String>> {
    state.with_library(clip::all_tags)
}

#[tauri::command]
pub fn filter_clips(app: AppHandle, state: State<AppState>, filter: ClipFilter) -> CoreResult<Vec<Clip>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        clip::filter_clips(lib, &filter)
    })
}

#[tauri::command]
pub fn link_clip_to_passage(state: State<AppState>, id: i64, passage_id: String) -> CoreResult<()> {
    state.with_library(|lib| clip::link_passage(lib, id, &passage_id))
}

#[tauri::command]
pub fn unlink_clip_from_passage(state: State<AppState>, id: i64, passage_id: String) -> CoreResult<()> {
    state.with_library(|lib| clip::unlink_passage(lib, id, &passage_id))
}

#[tauri::command]
pub fn clip_passages(state: State<AppState>, id: i64) -> CoreResult<Vec<Passage>> {
    state.with_library(|lib| clip::clip_passages(lib, id))
}

#[tauri::command]
pub fn passage_clips(app: AppHandle, state: State<AppState>, passage_id: String) -> CoreResult<Vec<Clip>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        clip::passage_clips(lib, &passage_id)
    })
}

/// Shows the Clip's file in Finder, so the founder can drag it into an editor.
#[tauri::command]
pub fn reveal_clip(state: State<AppState>, id: i64) -> CoreResult<()> {
    let path = state.with_library(|lib| clip::clip_file_to_reveal(lib, id))?;
    tauri_plugin_opener::reveal_item_in_dir(&path)
        .map_err(|e| CoreError::Io(std::io::Error::other(e.to_string())))
}
