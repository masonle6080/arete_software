//! IPC for the Production Order area. See `src/areas/productionOrder/api.ts`
//! for the callers.

use tauri::State;

use super::AppState;
use crate::drop::DropId;
use crate::error::CoreResult;
use crate::production_order::{self, ProductionOrder, ProductionOrderId, ProductionOrderStatus};
use crate::style::StyleId;
use crate::vendor::VendorId;

#[tauri::command]
pub fn create_production_order(state: State<AppState>, drop_id: DropId, vendor_id: VendorId) -> CoreResult<ProductionOrder> {
    state.with_library(|lib| production_order::create_production_order(lib, drop_id, vendor_id))
}

#[tauri::command]
pub fn get_production_order(state: State<AppState>, id: ProductionOrderId) -> CoreResult<ProductionOrder> {
    state.with_library(|lib| production_order::get_production_order(lib, id))
}

#[tauri::command]
pub fn drop_production_orders(state: State<AppState>, drop_id: DropId) -> CoreResult<Vec<ProductionOrder>> {
    state.with_library(|lib| production_order::drop_production_orders(lib, drop_id))
}

#[tauri::command]
pub fn set_order_quantity(
    state: State<AppState>,
    id: ProductionOrderId,
    style_id: StyleId,
    size: String,
    quantity: u32,
) -> CoreResult<ProductionOrder> {
    state.with_library(|lib| production_order::set_order_quantity(lib, id, style_id, &size, quantity))
}

#[tauri::command]
pub fn set_delivery_date(state: State<AppState>, id: ProductionOrderId, date: Option<String>) -> CoreResult<ProductionOrder> {
    state.with_library(|lib| production_order::set_delivery_date(lib, id, date.as_deref()))
}

#[tauri::command]
pub fn move_production_order(
    state: State<AppState>,
    id: ProductionOrderId,
    to: ProductionOrderStatus,
) -> CoreResult<ProductionOrder> {
    state.with_library(|lib| production_order::move_production_order(lib, id, to))
}
