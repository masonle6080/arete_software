//! IPC for the Style area. See `src/areas/style/api.ts` for the callers.

use std::path::PathBuf;

use tauri::{AppHandle, State};

use super::{allow_media, AppState};
use crate::thesis::ThesisId;
use crate::error::CoreResult;
use crate::style::{self, Style, StyleBase, StyleFile, StyleId, StyleStatus};

#[tauri::command]
pub fn create_style(state: State<AppState>, thesis_id: ThesisId, name: String, color: String) -> CoreResult<Style> {
    state.with_library(|lib| style::create_style(lib, thesis_id, &name, &color))
}

#[tauri::command]
pub fn duplicate_style(app: AppHandle, state: State<AppState>, id: StyleId, color: String) -> CoreResult<Style> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::duplicate_style(lib, id, &color)
    })
}

#[tauri::command]
pub fn move_style(app: AppHandle, state: State<AppState>, id: StyleId, to: StyleStatus) -> CoreResult<Style> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::move_style(lib, id, to)
    })
}

#[tauri::command]
pub fn get_style(app: AppHandle, state: State<AppState>, id: StyleId) -> CoreResult<Style> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::get_style(lib, id)
    })
}

#[tauri::command]
pub fn thesis_styles(app: AppHandle, state: State<AppState>, thesis_id: ThesisId) -> CoreResult<Vec<Style>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::thesis_styles(lib, thesis_id)
    })
}

#[tauri::command]
pub fn attach_style_files(
    app: AppHandle,
    state: State<AppState>,
    id: StyleId,
    sources: Vec<PathBuf>,
) -> CoreResult<Vec<StyleFile>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::attach_style_files(lib, id, &sources)
    })
}

#[tauri::command]
pub fn style_files(app: AppHandle, state: State<AppState>, id: StyleId) -> CoreResult<Vec<StyleFile>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::style_files(lib, id)
    })
}

#[tauri::command]
pub fn remove_style_file(app: AppHandle, state: State<AppState>, id: StyleId, file_id: i64) -> CoreResult<Vec<StyleFile>> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::remove_style_file(lib, id, file_id)
    })
}

#[tauri::command]
pub fn set_style_cover(app: AppHandle, state: State<AppState>, id: StyleId, file_id: i64) -> CoreResult<Style> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::set_style_cover(lib, id, file_id)
    })
}

#[tauri::command]
pub fn clear_style_cover(app: AppHandle, state: State<AppState>, id: StyleId) -> CoreResult<Style> {
    state.with_library(|lib| {
        allow_media(&app, lib);
        style::clear_style_cover(lib, id)
    })
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
