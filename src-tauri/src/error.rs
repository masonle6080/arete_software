//! The one error type of the Library core.
//!
//! It crosses IPC as `{ "kind": "<Variant>", "message": "<text>" }`. The UI
//! switches on `kind` (see `src/shell/coreError.ts`). Add new variants here as
//! areas need them; keep each variant something the founder could notice.

use std::path::PathBuf;

use serde::ser::SerializeStruct;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{path} is not an Arete library")]
    NotALibrary { path: PathBuf },

    #[error("{path} already holds an Arete library")]
    AlreadyALibrary { path: PathBuf },

    #[error("{path} is not empty; choose an empty or new folder")]
    FolderNotEmpty { path: PathBuf },

    #[error("this library was made by a newer version of Arete (unknown migration {migration})")]
    NewerSchema { migration: String },

    #[error("no library is open")]
    NoLibraryOpen,

    /// A lifecycle rule refuses this status move. `what` names the record
    /// (for example "Thesis"); `from` and `to` are status labels.
    #[error("a {what} cannot move from {from} to {to}")]
    StatusMoveRefused { what: &'static str, from: &'static str, to: &'static str },

    /// A Thesis has at most one Drop that is not Cancelled.
    #[error("this Thesis already has a Drop; cancel it before you make a new one")]
    ActiveDropExists { thesis_id: i64 },

    #[error("the library at {path} is disconnected; reconnect its drive")]
    LibraryDisconnected { path: PathBuf },

    #[error("{record} {id} does not exist")]
    NotFound { record: &'static str, id: i64 },

    /// The founder gave a value the core cannot accept, for example an
    /// empty title. `reason` says what to fix.
    #[error("{reason}")]
    InvalidInput { reason: String },

    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl CoreError {
    /// The stable name the UI switches on.
    pub fn kind(&self) -> &'static str {
        match self {
            CoreError::NotALibrary { .. } => "NotALibrary",
            CoreError::AlreadyALibrary { .. } => "AlreadyALibrary",
            CoreError::FolderNotEmpty { .. } => "FolderNotEmpty",
            CoreError::NewerSchema { .. } => "NewerSchema",
            CoreError::NoLibraryOpen => "NoLibraryOpen",
            CoreError::StatusMoveRefused { .. } => "StatusMoveRefused",
            CoreError::ActiveDropExists { .. } => "ActiveDropExists",
            CoreError::LibraryDisconnected { .. } => "LibraryDisconnected",
            CoreError::NotFound { .. } => "NotFound",
            CoreError::InvalidInput { .. } => "InvalidInput",
            CoreError::Io(_) => "Io",
            CoreError::Database(_) => "Database",
        }
    }
}

impl serde::Serialize for CoreError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("CoreError", 2)?;
        st.serialize_field("kind", self.kind())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
