//! IPC for the Style area. See `src/areas/style/api.ts` for the callers.

use std::path::PathBuf;

use tauri::State;

use super::AppState;
use crate::drop::DropId;
use crate::error::CoreResult;
use crate::style::{self, Style, StyleBase, StyleFile, StyleId, StyleStatus};

#[tauri::command]
pub fn create_style(state: State<AppState>, drop_id: DropId, name: String, color: String) -> CoreResult<Style> {
    state.with_library(|lib| style::create_style(lib, drop_id, &name, &color))
}

#[tauri::command]
pub fn duplicate_style(state: State<AppState>, id: StyleId, color: String) -> CoreResult<Style> {
    state.with_library(|lib| style::duplicate_style(lib, id, &color))
}

#[tauri::command]
pub fn move_style(state: State<AppState>, id: StyleId, to: StyleStatus) -> CoreResult<Style> {
    state.with_library(|lib| style::move_style(lib, id, to))
}

#[tauri::command]
pub fn get_style(state: State<AppState>, id: StyleId) -> CoreResult<Style> {
    state.with_library(|lib| style::get_style(lib, id))
}

#[tauri::command]
pub fn drop_styles(state: State<AppState>, drop_id: DropId) -> CoreResult<Vec<Style>> {
    state.with_library(|lib| style::drop_styles(lib, drop_id))
}

#[tauri::command]
pub fn attach_style_files(state: State<AppState>, id: StyleId, sources: Vec<PathBuf>) -> CoreResult<Vec<StyleFile>> {
    state.with_library(|lib| style::attach_style_files(lib, id, &sources))
}

#[tauri::command]
pub fn style_files(state: State<AppState>, id: StyleId) -> CoreResult<Vec<StyleFile>> {
    state.with_library(|lib| style::style_files(lib, id))
}

#[tauri::command]
pub fn style_base(state: State<AppState>, id: StyleId) -> CoreResult<Option<StyleBase>> {
    state.with_library(|lib| style::style_base(lib, id))
}

#[tauri::command]
pub fn set_style_base(state: State<AppState>, id: StyleId, base: StyleBase) -> CoreResult<StyleBase> {
    state.with_library(|lib| style::set_style_base(lib, id, base))
}

#[tauri::command]
pub fn style_sizes(state: State<AppState>, id: StyleId) -> CoreResult<Vec<String>> {
    state.with_library(|lib| style::style_sizes(lib, id))
}

#[tauri::command]
pub fn set_style_sizes(state: State<AppState>, id: StyleId, sizes: Vec<String>) -> CoreResult<Vec<String>> {
    state.with_library(|lib| style::set_style_sizes(lib, id, &sizes))
}
