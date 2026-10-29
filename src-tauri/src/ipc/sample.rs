//! IPC for the Sample area. See `src/areas/sample/api.ts` for the callers.

use std::path::PathBuf;

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::sample::{self, Sample, SampleFit, SampleId, SampleOutcome, SamplePhoto};
use crate::tech_pack::{PointOfMeasureId, SizeId};
use crate::style::StyleId;
use crate::vendor::VendorId;

#[tauri::command]
pub fn request_sample(state: State<AppState>, style_id: StyleId, vendor_id: VendorId) -> CoreResult<Sample> {
    state.with_library(|lib| sample::request_sample(lib, style_id, vendor_id))
}

#[tauri::command]
pub fn style_samples(state: State<AppState>, style_id: StyleId) -> CoreResult<Vec<Sample>> {
    state.with_library(|lib| sample::style_samples(lib, style_id))
}

#[tauri::command]
pub fn set_sample_feedback(state: State<AppState>, id: SampleId, feedback: String) -> CoreResult<Sample> {
    state.with_library(|lib| sample::set_sample_feedback(lib, id, &feedback))
}

#[tauri::command]
pub fn attach_sample_photos(state: State<AppState>, id: SampleId, sources: Vec<PathBuf>) -> CoreResult<Vec<SamplePhoto>> {
    state.with_library(|lib| sample::attach_sample_photos(lib, id, &sources))
}

#[tauri::command]
pub fn decide_sample(state: State<AppState>, id: SampleId, outcome: SampleOutcome) -> CoreResult<Sample> {
    state.with_library(|lib| sample::decide_sample(lib, id, outcome))
}

#[tauri::command]
pub fn sample_fit(state: State<AppState>, id: SampleId) -> CoreResult<Option<SampleFit>> {
    state.with_library(|lib| sample::sample_fit(lib, id))
}

#[tauri::command]
pub fn set_fit_note(
    state: State<AppState>,
    id: SampleId,
    point_of_measure_id: PointOfMeasureId,
    size_id: SizeId,
    measured: Option<f64>,
) -> CoreResult<SampleFit> {
    state.with_library(|lib| sample::set_fit_note(lib, id, point_of_measure_id, size_id, measured))
}
