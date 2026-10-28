//! IPC for the Document area (Essays, later Scripts). See
//! `src/areas/essays/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::document::{self, Document, DocumentEdit, DocumentKind, DocumentSummary};
use crate::error::CoreResult;

#[tauri::command]
pub fn create_document(state: State<AppState>, kind: DocumentKind, title: String) -> CoreResult<Document> {
    state.with_library(|lib| document::create_document(lib, kind, &title))
}

#[tauri::command]
pub fn open_document(state: State<AppState>, id: i64) -> CoreResult<Document> {
    state.with_library(|lib| document::open_document(lib, id))
}

#[tauri::command]
pub fn save_document(state: State<AppState>, id: i64, edit: DocumentEdit) -> CoreResult<DocumentSummary> {
    state.with_library(|lib| document::save_document(lib, id, &edit))
}

#[tauri::command]
pub fn list_documents(state: State<AppState>, kind: DocumentKind) -> CoreResult<Vec<DocumentSummary>> {
    state.with_library(|lib| document::list_documents(lib, kind))
}
