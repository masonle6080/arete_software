//! IPC for the Library area. See `src/areas/library/api.ts` for the callers.

use std::path::PathBuf;

use tauri::State;

use super::AppState;
use crate::error::{CoreError, CoreResult};
use crate::library::{self, LibraryInfo};

#[tauri::command]
pub fn create_library(state: State<AppState>, path: PathBuf) -> CoreResult<LibraryInfo> {
    let lib = library::create_library(&state.config, &path)?;
    let info = lib.info()?;
    state.set_library(lib);
    Ok(info)
}

#[tauri::command]
pub fn open_library(state: State<AppState>, path: PathBuf) -> CoreResult<LibraryInfo> {
    let lib = library::open_library(&state.config, &path)?;
    let info = lib.info()?;
    state.set_library(lib);
    Ok(info)
}

#[tauri::command]
pub fn reopen_last_library(state: State<AppState>) -> CoreResult<Option<LibraryInfo>> {
    let Some(lib) = library::reopen_last_library(&state.config)? else {
        return Ok(None);
    };
    let info = lib.info()?;
    state.set_library(lib);
    Ok(Some(info))
}

#[tauri::command]
pub fn current_library(state: State<AppState>) -> CoreResult<Option<LibraryInfo>> {
    match state.with_library(|lib| lib.info()) {
        Ok(info) => Ok(Some(info)),
        Err(CoreError::NoLibraryOpen) => Ok(None),
        Err(e) => Err(e),
    }
}

/// True while the open library's folder is reachable. The UI polls it.
#[tauri::command]
pub fn library_connected(state: State<AppState>) -> CoreResult<bool> {
    state.with_library(|lib| Ok(lib.is_reachable()))
}

#[tauri::command]
pub fn reconnect_library(state: State<AppState>) -> CoreResult<LibraryInfo> {
    state.with_library_mut(|lib| {
        lib.reconnect()?;
        lib.info()
    })
}
