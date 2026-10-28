//! IPC for the Thesis area. See `src/areas/thesis/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::thesis::{self, Thesis, ThesisGroup, ThesisId, ThesisStatus};

#[tauri::command]
pub fn create_thesis(state: State<AppState>, title: String, statement: String) -> CoreResult<Thesis> {
    state.with_library(|lib| thesis::create_thesis(lib, &title, &statement))
}

#[tauri::command]
pub fn move_thesis(state: State<AppState>, id: ThesisId, to: ThesisStatus) -> CoreResult<Thesis> {
    state.with_library(|lib| thesis::move_thesis(lib, id, to))
}

#[tauri::command]
pub fn list_theses(state: State<AppState>) -> CoreResult<Vec<ThesisGroup>> {
    state.with_library(thesis::list_theses)
}

#[tauri::command]
pub fn get_thesis(state: State<AppState>, id: ThesisId) -> CoreResult<Thesis> {
    state.with_library(|lib| thesis::get_thesis(lib, id))
}
