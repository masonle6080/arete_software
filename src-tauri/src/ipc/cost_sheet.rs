//! IPC for the Cost Sheet area and the currency settings. See
//! `src/areas/costSheet/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::cost_sheet::{self, CostLineId, CostLineInput, CostSheet, CurrencySettings, QuantityTierId};
use crate::error::CoreResult;
use crate::style::StyleId;

#[tauri::command]
pub fn cost_sheet(state: State<AppState>, style_id: StyleId) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::cost_sheet(lib, style_id))
}

#[tauri::command]
pub fn add_quantity_tier(state: State<AppState>, style_id: StyleId, quantity: u32) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::add_quantity_tier(lib, style_id, quantity))
}

#[tauri::command]
pub fn set_quantity_tier(state: State<AppState>, tier_id: QuantityTierId, quantity: u32) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::set_quantity_tier(lib, tier_id, quantity))
}

#[tauri::command]
pub fn remove_quantity_tier(state: State<AppState>, tier_id: QuantityTierId) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::remove_quantity_tier(lib, tier_id))
}

#[tauri::command]
pub fn add_cost_line(state: State<AppState>, style_id: StyleId, input: CostLineInput) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::add_cost_line(lib, style_id, input))
}

#[tauri::command]
pub fn update_cost_line(state: State<AppState>, line_id: CostLineId, input: CostLineInput) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::update_cost_line(lib, line_id, input))
}

#[tauri::command]
pub fn remove_cost_line(state: State<AppState>, line_id: CostLineId) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::remove_cost_line(lib, line_id))
}

#[tauri::command]
pub fn set_unit_cost(
    state: State<AppState>,
    line_id: CostLineId,
    tier_id: QuantityTierId,
    unit_cost: Option<String>,
) -> CoreResult<CostSheet> {
    state.with_library(|lib| cost_sheet::set_unit_cost(lib, line_id, tier_id, unit_cost.as_deref()))
}

#[tauri::command]
pub fn currency_settings(state: State<AppState>) -> CoreResult<CurrencySettings> {
    state.with_library(cost_sheet::currency_settings)
}

#[tauri::command]
pub fn set_home_currency(state: State<AppState>, currency: String) -> CoreResult<CurrencySettings> {
    state.with_library(|lib| cost_sheet::set_home_currency(lib, &currency))
}

#[tauri::command]
pub fn set_exchange_rate(state: State<AppState>, currency: String, rate: String) -> CoreResult<CurrencySettings> {
    state.with_library(|lib| cost_sheet::set_exchange_rate(lib, &currency, &rate))
}

#[tauri::command]
pub fn remove_exchange_rate(state: State<AppState>, currency: String) -> CoreResult<CurrencySettings> {
    state.with_library(|lib| cost_sheet::remove_exchange_rate(lib, &currency))
}
