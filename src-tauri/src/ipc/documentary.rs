//! IPC for the Documentary area. See `src/areas/documentary/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::clip::Clip;
use crate::documentary::{self, Documentary, DocumentaryId, DocumentaryStatus};
use crate::error::CoreResult;
use crate::thesis::ThesisId;

#[tauri::command]
pub fn create_documentary(state: State<AppState>, thesis_id: ThesisId, title: String) -> CoreResult<Documentary> {
    state.with_library(|lib| documentary::create_documentary(lib, thesis_id, &title))
}

#[tauri::command]
pub fn move_documentary(state: State<AppState>, id: DocumentaryId, to: DocumentaryStatus) -> CoreResult<Documentary> {
    state.with_library(|lib| documentary::move_documentary(lib, id, to))
}

#[tauri::command]
pub fn thesis_documentary(state: State<AppState>, thesis_id: ThesisId) -> CoreResult<Option<Documentary>> {
    state.with_library(|lib| documentary::thesis_documentary(lib, thesis_id))
}

#[tauri::command]
pub fn list_documentaries(state: State<AppState>) -> CoreResult<Vec<Documentary>> {
    state.with_library(documentary::list_documentaries)
}

#[tauri::command]
pub fn add_clip_to_documentary(state: State<AppState>, id: DocumentaryId, clip_id: i64) -> CoreResult<()> {
    state.with_library(|lib| documentary::add_clip(lib, id, clip_id))
}

#[tauri::command]
pub fn remove_clip_from_documentary(state: State<AppState>, id: DocumentaryId, clip_id: i64) -> CoreResult<()> {
    state.with_library(|lib| documentary::remove_clip(lib, id, clip_id))
}

#[tauri::command]
pub fn documentary_clips(state: State<AppState>, id: DocumentaryId) -> CoreResult<Vec<Clip>> {
    state.with_library(|lib| documentary::documentary_clips(lib, id))
}

#[tauri::command]
pub fn clip_documentaries(state: State<AppState>, clip_id: i64) -> CoreResult<Vec<Documentary>> {
    state.with_library(|lib| documentary::clip_documentaries(lib, clip_id))
}
