//! IPC for the links that screen headers show as chips. See
//! `src/areas/links/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::links::{self, EssayLinks, StyleLinks, ThesisLinks};
use crate::style::StyleId;
use crate::thesis::ThesisId;

#[tauri::command]
pub fn thesis_links(state: State<AppState>, thesis_id: ThesisId) -> CoreResult<ThesisLinks> {
    state.with_library(|lib| links::thesis_links(lib, thesis_id))
}

#[tauri::command]
pub fn essay_links(state: State<AppState>, essay_id: i64) -> CoreResult<EssayLinks> {
    state.with_library(|lib| links::essay_links(lib, essay_id))
}

#[tauri::command]
pub fn style_links(state: State<AppState>, style_id: StyleId) -> CoreResult<StyleLinks> {
    state.with_library(|lib| links::style_links(lib, style_id))
}
