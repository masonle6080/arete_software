//! Export: files the founder saves outside the library (Markdown, later
//! Style traces), and print jobs for PDF exports (Essays, Tech Packs, Style
//! traces).
//!
//! An export is not a library write. It does not go through
//! `Library::writer()` and works while the library is disconnected. It must
//! not land inside the library folder, where it could replace library files.
//!
//! PDF: the webview renders HTML in a print window and the founder saves it
//! with the system print dialog ("Save as PDF"). See `ipc/export.rs` and
//! `src/export/`. [`PrintJobs`] hands each print window its HTML.

use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::error::{CoreError, CoreResult};

/// Writes `bytes` to `dest`, which the founder chose in a save dialog.
/// Replaces an existing file. Refuses a destination inside `library_root`.
pub fn write_export(library_root: Option<&Path>, dest: &Path, bytes: &[u8]) -> CoreResult<()> {
    let dir = dest.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = dest
        .file_name()
        .ok_or_else(|| CoreError::InvalidInput { reason: "choose a file name for the export".into() })?;
    let dir = dir.canonicalize()?;
    if let Some(root) = library_root {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if dir.starts_with(&root) {
            return Err(CoreError::InvalidInput {
                reason: "save the export outside the library folder".into(),
            });
        }
    }
    // Write a temporary file next to the destination, then rename it, so a
    // failed write never leaves half a file.
    let tmp: PathBuf = dir.join(format!(".{}.arete-export", name.to_string_lossy()));
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&tmp, dir.join(name))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    Ok(result?)
}

/// What a print window shows: an HTML fragment, its title (the default PDF
/// file name), and optional CSS on top of the base print style.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintJob {
    pub title: String,
    pub html: String,
    pub css: Option<String>,
}

/// Print jobs waiting for their window. Each job is taken once.
#[derive(Default)]
pub struct PrintJobs {
    next: AtomicU64,
    jobs: Mutex<HashMap<String, PrintJob>>,
}

impl PrintJobs {
    /// Stores `job` and returns its ID.
    pub fn put(&self, job: PrintJob) -> String {
        let id = (self.next.fetch_add(1, Ordering::Relaxed) + 1).to_string();
        self.jobs.lock().expect("print jobs lock poisoned").insert(id.clone(), job);
        id
    }

    /// Removes and returns the job with `id`, if it is still waiting.
    pub fn take(&self, id: &str) -> Option<PrintJob> {
        self.jobs.lock().expect("print jobs lock poisoned").remove(id)
    }
}
