//! Library: the one folder that holds all brand data (ADR-0002).
//!
//! ```text
//! <library>/
//!   arete.sqlite   structured records
//!   documents/     Essays and Scripts (Markdown)
//!   media/         imported files, named by content hash
//! ```
//!
//! Domain command functions for this area: [`create_library`],
//! [`open_library`], [`reopen_last_library`], and on [`Library`]:
//! [`Library::is_reachable`] and [`Library::reconnect`] for a library whose
//! drive was disconnected.

mod app_config;

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::{CoreError, CoreResult};
use crate::migrations::{self, Migration};

pub use app_config::AppConfig;

const DATABASE_FILE: &str = "arete.sqlite";
const DOCUMENTS_DIR: &str = "documents";
const MEDIA_DIR: &str = "media";

/// An open library. Other areas take `&Library` and reach the database
/// through [`Library::db`] (reads) and [`Library::writer`] (writes).
#[derive(Debug)]
pub struct Library {
    root: PathBuf,
    conn: Connection,
}

impl Library {
    /// Creates a library at the latest schema version.
    pub fn create(root: &Path) -> CoreResult<Library> {
        Self::create_with_migrations(root, migrations::ALL)
    }

    /// Opens a library and migrates it to the latest schema version.
    pub fn open(root: &Path) -> CoreResult<Library> {
        Self::open_with_migrations(root, migrations::ALL)
    }

    /// Like [`Library::create`] with an explicit migration list. Tests use it
    /// to build libraries at older schema versions.
    pub fn create_with_migrations(root: &Path, migrations: &[Migration]) -> CoreResult<Library> {
        if root.join(DATABASE_FILE).exists() {
            return Err(CoreError::AlreadyALibrary { path: root.to_path_buf() });
        }
        if root.is_dir() && std::fs::read_dir(root)?.next().is_some() {
            return Err(CoreError::FolderNotEmpty { path: root.to_path_buf() });
        }
        std::fs::create_dir_all(root.join(DOCUMENTS_DIR))?;
        std::fs::create_dir_all(root.join(MEDIA_DIR))?;
        Self::connect(root, migrations)
    }

    /// Like [`Library::open`] with an explicit migration list.
    pub fn open_with_migrations(root: &Path, migrations: &[Migration]) -> CoreResult<Library> {
        if !root.join(DATABASE_FILE).is_file() {
            return Err(CoreError::NotALibrary { path: root.to_path_buf() });
        }
        // Repair folders a user may have deleted by hand.
        std::fs::create_dir_all(root.join(DOCUMENTS_DIR))?;
        std::fs::create_dir_all(root.join(MEDIA_DIR))?;
        Self::connect(root, migrations)
    }

    fn connect(root: &Path, migrations: &[Migration]) -> CoreResult<Library> {
        let mut conn = Self::open_connection(root)?;
        migrations::apply(&mut conn, migrations)?;
        Ok(Library { root: root.to_path_buf(), conn })
    }

    fn open_connection(root: &Path) -> CoreResult<Connection> {
        let conn = Connection::open(root.join(DATABASE_FILE))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        Ok(conn)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn documents_dir(&self) -> PathBuf {
        self.root.join(DOCUMENTS_DIR)
    }

    pub fn media_dir(&self) -> PathBuf {
        self.root.join(MEDIA_DIR)
    }

    /// The number of migrations applied to this library.
    pub fn schema_version(&self) -> CoreResult<u32> {
        migrations::schema_version(&self.conn)
    }

    /// The database, for reads by domain modules.
    #[allow(dead_code)]
    pub(crate) fn db(&self) -> &Connection {
        &self.conn
    }

    /// True when the library folder can be reached (for example, its
    /// external drive is plugged in).
    pub fn is_reachable(&self) -> bool {
        self.root.join(DATABASE_FILE).is_file()
    }

    /// Fails with `LibraryDisconnected` when the library folder cannot be
    /// reached. [`Library::writer`] calls it; file writes into
    /// `documents/` or `media/` call it too, before they touch the disk.
    pub fn ensure_reachable(&self) -> CoreResult<()> {
        if self.is_reachable() {
            Ok(())
        } else {
            Err(CoreError::LibraryDisconnected { path: self.root.clone() })
        }
    }

    /// Reopens the database after the folder comes back (for example, the
    /// drive is plugged in again). The old connection may hold a dead file
    /// handle, so this always opens a new one. It does not run migrations:
    /// the schema was current when the library was opened. Fails with
    /// `LibraryDisconnected` while the folder is still unreachable.
    pub fn reconnect(&mut self) -> CoreResult<()> {
        self.ensure_reachable()?;
        self.conn = Self::open_connection(&self.root)?;
        Ok(())
    }

    /// The database, for writes by domain modules. Every write must go
    /// through here, so library-wide write guards live in one place. It
    /// refuses with `LibraryDisconnected` when the folder is unreachable.
    pub fn writer(&self) -> CoreResult<&Connection> {
        self.ensure_reachable()?;
        Ok(&self.conn)
    }
}

/// What the UI shows about the open library.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub path: PathBuf,
    pub schema_version: u32,
}

impl Library {
    pub fn info(&self) -> CoreResult<LibraryInfo> {
        Ok(LibraryInfo { path: self.root.clone(), schema_version: self.schema_version()? })
    }
}

/// The schema version that [`Library::create`] and [`Library::open`] produce.
pub fn latest_schema_version() -> u32 {
    migrations::ALL.len() as u32
}

/// Creates a library in `path` and remembers it as the last library.
pub fn create_library(config: &AppConfig, path: &Path) -> CoreResult<Library> {
    let lib = Library::create(path)?;
    config.remember_last_library(lib.root())?;
    Ok(lib)
}

/// Opens the library in `path`, runs migrations, and remembers it.
pub fn open_library(config: &AppConfig, path: &Path) -> CoreResult<Library> {
    let lib = Library::open(path)?;
    config.remember_last_library(lib.root())?;
    Ok(lib)
}

/// Opens the last library, if the app has one on record.
pub fn reopen_last_library(config: &AppConfig) -> CoreResult<Option<Library>> {
    match config.last_library()? {
        Some(path) => Library::open(&path).map(Some),
        None => Ok(None),
    }
}
