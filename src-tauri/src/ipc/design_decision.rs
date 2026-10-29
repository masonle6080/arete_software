//! IPC for the Design Decision area. See `src/areas/designDecision/api.ts` for
//! the callers.

use tauri::State;

use super::AppState;
use crate::decoration::DecorationId;
use crate::design_decision::{self, DesignAspect, DesignDecision, DesignDecisionId, PassageDesignDecision};
use crate::error::CoreResult;
use crate::style::StyleId;

#[tauri::command]
pub fn add_design_decision(
    state: State<AppState>,
    style_id: StyleId,
    aspect: DesignAspect,
    rationale: String,
) -> CoreResult<DesignDecision> {
    state.with_library(|lib| design_decision::add_design_decision(lib, style_id, aspect, &rationale))
}

#[tauri::command]
pub fn style_design_decisions(state: State<AppState>, style_id: StyleId) -> CoreResult<Vec<DesignDecision>> {
    state.with_library(|lib| design_decision::style_design_decisions(lib, style_id))
}

#[tauri::command]
pub fn link_design_decision_passage(
    state: State<AppState>,
    id: DesignDecisionId,
    passage_id: String,
) -> CoreResult<DesignDecision> {
    state.with_library(|lib| design_decision::link_passage(lib, id, &passage_id))
}

#[tauri::command]
pub fn unlink_design_decision_passage(
    state: State<AppState>,
    id: DesignDecisionId,
    passage_id: String,
) -> CoreResult<DesignDecision> {
    state.with_library(|lib| design_decision::unlink_passage(lib, id, &passage_id))
}

#[tauri::command]
pub fn passage_design_decisions(state: State<AppState>, passage_id: String) -> CoreResult<Vec<PassageDesignDecision>> {
    state.with_library(|lib| design_decision::passage_design_decisions(lib, &passage_id))
}

#[tauri::command]
pub fn set_design_decision_decoration(
    state: State<AppState>,
    id: DesignDecisionId,
    decoration_id: Option<DecorationId>,
) -> CoreResult<DesignDecision> {
    state.with_library(|lib| design_decision::set_decoration(lib, id, decoration_id))
}
