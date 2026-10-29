//! IPC for the Document area (Essays, later Scripts). See
//! `src/areas/essays/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::document::{self, Document, DocumentEdit, DocumentKind, DocumentSummary, Passage, PassageText};
use crate::error::CoreResult;
use crate::thesis::{Thesis, ThesisId};

#[tauri::command]
pub fn create_document(state: State<AppState>, kind: DocumentKind, title: String) -> CoreResult<Document> {
    state.with_library(|lib| document::create_document(lib, kind, &title))
}

#[tauri::command]
pub fn open_document(state: State<AppState>, id: i64) -> CoreResult<Document> {
    state.with_library(|lib| document::open_document(lib, id))
}

/// `passages` is the save report. The editor always sends it; without it the
/// core saves the text and leaves the Passage records as they are.
#[tauri::command]
pub fn save_document(
    state: State<AppState>,
    id: i64,
    edit: DocumentEdit,
    passages: Option<Vec<PassageText>>,
) -> CoreResult<DocumentSummary> {
    state.with_library(|lib| match &passages {
        Some(report) => document::save_document_with_passages(lib, id, &edit, report),
        None => document::save_document(lib, id, &edit),
    })
}

#[tauri::command]
pub fn get_passage(state: State<AppState>, id: String) -> CoreResult<Passage> {
    state.with_library(|lib| document::get_passage(lib, &id))
}

#[tauri::command]
pub fn passages_of_document(state: State<AppState>, document_id: i64) -> CoreResult<Vec<Passage>> {
    state.with_library(|lib| document::passages_of_document(lib, document_id))
}

#[tauri::command]
pub fn list_documents(state: State<AppState>, kind: DocumentKind) -> CoreResult<Vec<DocumentSummary>> {
    state.with_library(|lib| document::list_documents(lib, kind))
}

#[tauri::command]
pub fn link_essay_to_thesis(state: State<AppState>, essay_id: i64, thesis_id: ThesisId) -> CoreResult<()> {
    state.with_library(|lib| document::link_essay_to_thesis(lib, essay_id, thesis_id))
}

#[tauri::command]
pub fn unlink_essay_from_thesis(state: State<AppState>, essay_id: i64, thesis_id: ThesisId) -> CoreResult<()> {
    state.with_library(|lib| document::unlink_essay_from_thesis(lib, essay_id, thesis_id))
}

#[tauri::command]
pub fn theses_of_essay(state: State<AppState>, essay_id: i64) -> CoreResult<Vec<Thesis>> {
    state.with_library(|lib| document::theses_of_essay(lib, essay_id))
}

#[tauri::command]
pub fn essays_of_thesis(state: State<AppState>, thesis_id: ThesisId) -> CoreResult<Vec<DocumentSummary>> {
    state.with_library(|lib| document::essays_of_thesis(lib, thesis_id))
}
