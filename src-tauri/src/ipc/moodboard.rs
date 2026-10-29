//! IPC for the Moodboard area. See `src/areas/moodboard/api.ts` for the callers.

use tauri::State;

use super::AppState;
use crate::error::CoreResult;
use crate::moodboard::{self, LinkedMoodboardItem, Moodboard, MoodboardId, MoodboardOwner};

#[tauri::command]
pub fn create_moodboard(state: State<AppState>, owner: MoodboardOwner, title: String) -> CoreResult<Moodboard> {
    state.with_library(|lib| moodboard::create_moodboard(lib, owner, &title))
}

#[tauri::command]
pub fn list_moodboards(state: State<AppState>, owner: MoodboardOwner) -> CoreResult<Vec<Moodboard>> {
    state.with_library(|lib| moodboard::list_moodboards(lib, owner))
}

#[tauri::command]
pub fn load_moodboard_scene(state: State<AppState>, id: MoodboardId) -> CoreResult<Option<String>> {
    state.with_library(|lib| moodboard::load_moodboard_scene(lib, id))
}

#[tauri::command]
pub fn save_moodboard_scene(state: State<AppState>, id: MoodboardId, scene: String) -> CoreResult<()> {
    state.with_library(|lib| moodboard::save_moodboard_scene(lib, id, &scene))
}

#[tauri::command]
pub fn moodboard_items_of_passage(state: State<AppState>, passage_id: String) -> CoreResult<Vec<LinkedMoodboardItem>> {
    state.with_library(|lib| moodboard::moodboard_items_of_passage(lib, &passage_id))
}
