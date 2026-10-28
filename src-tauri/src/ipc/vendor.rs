//! IPC for the Vendor area. See `src/areas/vendor/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::vendor::{self, Vendor, VendorId, VendorInput};

#[tauri::command]
pub fn list_vendors(state: State<AppState>) -> CoreResult<Vec<Vendor>> {
    state.with_library(vendor::list_vendors)
}

#[tauri::command]
pub fn create_vendor(state: State<AppState>, input: VendorInput) -> CoreResult<Vendor> {
    state.with_library(|lib| vendor::create_vendor(lib, input))
}

#[tauri::command]
pub fn update_vendor(state: State<AppState>, id: VendorId, input: VendorInput) -> CoreResult<Vendor> {
    state.with_library(|lib| vendor::update_vendor(lib, id, input))
}
