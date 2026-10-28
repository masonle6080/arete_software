//! Library core: a library whose folder is unreachable refuses writes, and
//! reconnects when the folder returns (ADR-0002).
//!
//! "Unplugging the drive" is simulated by moving the library folder away.
//! The open connection still points at the moved file, so these tests also
//! prove that the guard, not the file system, stops the write.

mod common;

use std::path::Path;

use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::migrations::Migration;
use common::TestEnv;

const NOTES: Migration = Migration {
    name: "0001_test_note",
    sql: "CREATE TABLE note (text TEXT NOT NULL);",
};

fn write_note(lib: &Library, text: &str) -> Result<(), CoreError> {
    lib.writer()?.execute("INSERT INTO note (text) VALUES (?1)", [text])?;
    Ok(())
}

fn note_count(folder: &Path) -> u32 {
    let lib = Library::open_with_migrations(folder, &[NOTES]).unwrap();
    lib.writer()
        .unwrap()
        .query_row("SELECT COUNT(*) FROM note", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn writing_to_a_disconnected_library_is_refused_and_changes_nothing() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let unplugged = env.path("unplugged");
    let lib = Library::create_with_migrations(&folder, &[NOTES]).unwrap();
    write_note(&lib, "before").unwrap();

    std::fs::rename(&folder, &unplugged).unwrap();
    let err = write_note(&lib, "while unplugged").unwrap_err();
    std::fs::rename(&unplugged, &folder).unwrap();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert_eq!(err.kind(), "LibraryDisconnected");
    drop(lib);
    assert_eq!(note_count(&folder), 1);
}

#[test]
fn reconnecting_while_the_folder_is_still_gone_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let mut lib = Library::create_with_migrations(&folder, &[NOTES]).unwrap();

    std::fs::rename(&folder, env.path("unplugged")).unwrap();
    let err = lib.reconnect().unwrap_err();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert!(!lib.is_reachable());
}

#[test]
fn library_reconnects_when_the_folder_returns_and_accepts_writes_again() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let unplugged = env.path("unplugged");
    let mut lib = Library::create_with_migrations(&folder, &[NOTES]).unwrap();
    std::fs::rename(&folder, &unplugged).unwrap();
    assert!(write_note(&lib, "refused").is_err());

    std::fs::rename(&unplugged, &folder).unwrap();
    lib.reconnect().unwrap();
    write_note(&lib, "after reconnect").unwrap();

    assert!(lib.is_reachable());
    assert_eq!(lib.root(), folder.as_path());
    drop(lib);
    assert_eq!(note_count(&folder), 1);
}
