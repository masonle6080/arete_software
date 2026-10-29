//! IPC for the Decoration area. See `src/areas/decoration/api.ts` for the
//! callers.

use std::path::PathBuf;

use tauri::State;

use super::AppState;
use crate::decoration::{self, Decoration, DecorationId, DecorationInput};
use crate::error::CoreResult;
use crate::style::StyleId;

#[tauri::command]
pub fn add_decoration(state: State<AppState>, style_id: StyleId, input: DecorationInput) -> CoreResult<Decoration> {
    state.with_library(|lib| decoration::add_decoration(lib, style_id, input))
}

#[tauri::command]
pub fn update_decoration(state: State<AppState>, id: DecorationId, input: DecorationInput) -> CoreResult<Decoration> {
    state.with_library(|lib| decoration::update_decoration(lib, id, input))
}

#[tauri::command]
pub fn set_decoration_artwork(state: State<AppState>, id: DecorationId, source: PathBuf) -> CoreResult<Decoration> {
    state.with_library(|lib| decoration::set_decoration_artwork(lib, id, &source))
}

#[tauri::command]
pub fn clear_decoration_artwork(state: State<AppState>, id: DecorationId) -> CoreResult<Decoration> {
    state.with_library(|lib| decoration::clear_decoration_artwork(lib, id))
}

#[tauri::command]
pub fn remove_decoration(state: State<AppState>, id: DecorationId) -> CoreResult<()> {
    state.with_library(|lib| decoration::remove_decoration(lib, id))
}

#[tauri::command]
pub fn style_decorations(state: State<AppState>, style_id: StyleId) -> CoreResult<Vec<Decoration>> {
    state.with_library(|lib| decoration::style_decorations(lib, style_id))
}
