//! Library core: a library whose folder is unreachable refuses writes, and
//! reconnects when the folder returns (ADR-0002).
//!
//! "Unplugging the drive" is simulated by moving the library folder away.
//! The open connection still points at the moved file, so these tests also
//! prove that the guard, not the file system, stops the write. The write
//! under test is a new Thesis, made through the domain API.

mod common;

use std::path::Path;

use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::thesis;
use common::TestEnv;

/// The titles of every Thesis in the library at `folder`, oldest first.
fn thesis_titles(folder: &Path) -> Vec<String> {
    let lib = Library::open(folder).unwrap();
    let mut titles: Vec<(i64, String)> = thesis::list_theses(&lib)
        .unwrap()
        .into_iter()
        .flat_map(|g| g.theses)
        .map(|t| (t.id, t.title))
        .collect();
    titles.sort();
    titles.into_iter().map(|(_, title)| title).collect()
}

#[test]
fn writing_to_a_disconnected_library_is_refused_and_changes_nothing() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let unplugged = env.path("unplugged");
    let lib = Library::create(&folder).unwrap();
    thesis::create_thesis(&lib, "Before", "").unwrap();

    std::fs::rename(&folder, &unplugged).unwrap();
    let err = thesis::create_thesis(&lib, "While unplugged", "").unwrap_err();
    std::fs::rename(&unplugged, &folder).unwrap();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert_eq!(err.kind(), "LibraryDisconnected");
    drop(lib);
    assert_eq!(thesis_titles(&folder), ["Before"]);
}

#[test]
fn reconnecting_while_the_folder_is_still_gone_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let mut lib = Library::create(&folder).unwrap();

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
    let mut lib = Library::create(&folder).unwrap();
    std::fs::rename(&folder, &unplugged).unwrap();
    assert!(thesis::create_thesis(&lib, "Refused", "").is_err());

    std::fs::rename(&unplugged, &folder).unwrap();
    lib.reconnect().unwrap();
    thesis::create_thesis(&lib, "After reconnect", "").unwrap();

    assert!(lib.is_reachable());
    assert_eq!(lib.root(), folder.as_path());
    drop(lib);
    assert_eq!(thesis_titles(&folder), ["After reconnect"]);
}
