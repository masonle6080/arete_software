//! IPC for the Blank area. See `src/areas/blank/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::blank::{self, Blank, BlankId, BlankInput};
use crate::error::CoreResult;

#[tauri::command]
pub fn list_blanks(state: State<AppState>) -> CoreResult<Vec<Blank>> {
    state.with_library(blank::list_blanks)
}

#[tauri::command]
pub fn create_blank(state: State<AppState>, input: BlankInput) -> CoreResult<Blank> {
    state.with_library(|lib| blank::create_blank(lib, input))
}

#[tauri::command]
pub fn update_blank(state: State<AppState>, id: BlankId, input: BlankInput) -> CoreResult<Blank> {
    state.with_library(|lib| blank::update_blank(lib, id, input))
}
