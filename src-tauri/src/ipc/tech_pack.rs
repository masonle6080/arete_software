//! IPC for the Tech Pack area. See `src/areas/techPack/api.ts` for the
//! callers.

use std::path::PathBuf;

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::style::StyleId;
use crate::tech_pack::{self, PointOfMeasureId, SizeId, TechPack, TechPackDetails};

#[tauri::command]
pub fn get_tech_pack(state: State<AppState>, style_id: StyleId) -> CoreResult<Option<TechPack>> {
    state.with_library(|lib| tech_pack::get_tech_pack(lib, style_id))
}

#[tauri::command]
pub fn add_tech_pack_size(state: State<AppState>, style_id: StyleId, name: String) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::add_size(lib, style_id, &name))
}

#[tauri::command]
pub fn rename_tech_pack_size(state: State<AppState>, size_id: SizeId, name: String) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::rename_size(lib, size_id, &name))
}

#[tauri::command]
pub fn remove_tech_pack_size(state: State<AppState>, size_id: SizeId) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::remove_size(lib, size_id))
}

#[tauri::command]
pub fn add_point_of_measure(state: State<AppState>, style_id: StyleId, name: String, tolerance: f64) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::add_point_of_measure(lib, style_id, &name, tolerance))
}

#[tauri::command]
pub fn update_point_of_measure(
    state: State<AppState>,
    id: PointOfMeasureId,
    name: String,
    tolerance: f64,
) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::update_point_of_measure(lib, id, &name, tolerance))
}

#[tauri::command]
pub fn set_point_of_measure_value(
    state: State<AppState>,
    id: PointOfMeasureId,
    size_id: SizeId,
    value: Option<f64>,
) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::set_point_of_measure_value(lib, id, size_id, value))
}

#[tauri::command]
pub fn remove_point_of_measure(state: State<AppState>, id: PointOfMeasureId) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::remove_point_of_measure(lib, id))
}

#[tauri::command]
pub fn update_tech_pack_details(
    state: State<AppState>,
    style_id: StyleId,
    details: TechPackDetails,
) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::update_tech_pack_details(lib, style_id, details))
}

#[tauri::command]
pub fn attach_tech_pack_flats(state: State<AppState>, style_id: StyleId, sources: Vec<PathBuf>) -> CoreResult<TechPack> {
    state.with_library(|lib| tech_pack::attach_flats(lib, style_id, &sources))
}
