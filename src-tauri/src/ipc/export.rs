//! IPC for exports. See `src/export/` for the callers.
//!
//! - `save_export_file`: a save dialog, then a file write outside the library.
//!   The dialog runs here, so the webview can only write where the founder
//!   chose.
//! - `print_export`: opens a print window for an HTML fragment. The window
//!   takes its job with `take_print_job` and opens the print dialog, which
//!   saves the PDF.

use tauri::{AppHandle, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;

use super::AppState;
use crate::error::{CoreError, CoreResult};
use crate::export::{self, PrintJob, PrintJobs};

/// Asks where to save `contents` and writes it there. Returns the saved path,
/// or `None` when the founder cancels.
#[tauri::command]
pub async fn save_export_file(
    app: AppHandle,
    state: State<'_, AppState>,
    default_name: String,
    contents: String,
    filter_name: String,
    extensions: Vec<String>,
) -> CoreResult<Option<String>> {
    let exts: Vec<&str> = extensions.iter().map(String::as_str).collect();
    let Some(chosen) = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter(&filter_name, &exts)
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let dest = chosen
        .into_path()
        .map_err(|e| CoreError::InvalidInput { reason: format!("cannot save there: {e}") })?;
    let root = state.with_library(|lib| Ok(lib.root().to_path_buf())).ok();
    export::write_export(root.as_deref(), &dest, contents.as_bytes())?;
    Ok(Some(dest.to_string_lossy().into_owned()))
}

/// Opens a print window for `job`. Async: window creation must not block
/// the main thread.
#[tauri::command]
pub async fn print_export(app: AppHandle, jobs: State<'_, PrintJobs>, job: PrintJob) -> CoreResult<()> {
    let title = job.title.clone();
    let id = jobs.put(job);
    WebviewWindowBuilder::new(&app, format!("print-{id}"), WebviewUrl::App(format!("index.html#print={id}").into()))
        .title(if title.is_empty() { "Export".to_string() } else { title })
        .inner_size(820.0, 1000.0)
        .build()
        .map_err(|e| {
            jobs.take(&id);
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;
    Ok(())
}

/// The print window's job. `None` when it was already taken.
#[tauri::command]
pub fn take_print_job(jobs: State<PrintJobs>, id: String) -> Option<PrintJob> {
    jobs.take(&id)
}
