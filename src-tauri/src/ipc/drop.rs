//! IPC for the Drop area. See `src/areas/drop/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::drop::{self, Drop, DropId, DropMove, DropStage, ThesisDrops};
use crate::error::CoreResult;
use crate::thesis::ThesisId;

#[tauri::command]
pub fn create_drop(state: State<AppState>, thesis_id: ThesisId, name: String) -> CoreResult<Drop> {
    state.with_library(|lib| drop::create_drop(lib, thesis_id, &name))
}

#[tauri::command]
pub fn move_drop(state: State<AppState>, id: DropId, to: DropStage, confirmed: bool) -> CoreResult<DropMove> {
    state.with_library(|lib| drop::move_drop(lib, id, to, confirmed))
}

#[tauri::command]
pub fn thesis_drops(state: State<AppState>, thesis_id: ThesisId) -> CoreResult<ThesisDrops> {
    state.with_library(|lib| drop::thesis_drops(lib, thesis_id))
}
