//! IPC for media. See `src/areas/moodboard/api.ts` for the callers.
//!
//! Bytes cross IPC as a raw body, not a JSON number array, because images
//! can be several MB.

use tauri::ipc::{InvokeBody, Request, Response};
use tauri::State;

use super::AppState;
use crate::error::{CoreError, CoreResult};
use crate::media::{self, MediaRef};

/// Body: the raw file bytes. Header `media-ext`: the file extension
/// (ASCII, for example `png`), or absent.
#[tauri::command]
pub fn import_media_bytes(state: State<AppState>, request: Request) -> CoreResult<MediaRef> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(CoreError::InvalidInput { reason: "media import needs raw bytes".into() });
    };
    let ext = request.headers().get("media-ext").and_then(|v| v.to_str().ok()).unwrap_or("");
    let name = if ext.is_empty() { "drop".to_string() } else { format!("drop.{ext}") };
    state.with_library(|lib| media::import_media_bytes(lib, bytes, &name))
}

/// Returns the raw bytes of a stored media file.
#[tauri::command]
pub fn read_media(state: State<AppState>, media: MediaRef) -> CoreResult<Response> {
    state.with_library(|lib| media::read_media(lib, &media)).map(Response::new)
}
