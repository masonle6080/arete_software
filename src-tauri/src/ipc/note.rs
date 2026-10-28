//! IPC for the shared Notes record. See `src/areas/note/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::note::{self, Note, NoteInput, NoteOwner};

#[tauri::command]
pub fn list_notes(state: State<AppState>, owner: NoteOwner) -> CoreResult<Vec<Note>> {
    state.with_library(|lib| note::list_notes(lib, owner))
}

#[tauri::command]
pub fn add_note(state: State<AppState>, owner: NoteOwner, input: NoteInput) -> CoreResult<Note> {
    state.with_library(|lib| note::add_note(lib, owner, input))
}
