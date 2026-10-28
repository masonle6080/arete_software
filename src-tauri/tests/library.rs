//! Library core: create, open, and reopen a library.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary folder, and check results only through the same API.

mod common;

use arete_lib::error::CoreError;
use arete_lib::library::{self, AppConfig, Library};
use common::TestEnv;

#[test]
fn founder_can_create_a_library_in_a_chosen_folder() {
    let env = TestEnv::new();
    let folder = env.path("My Brand");

    let lib = library::create_library(&env.config, &folder).unwrap();

    assert_eq!(lib.root(), folder.as_path());
    assert!(lib.documents_dir().is_dir());
    assert!(lib.media_dir().is_dir());
}

#[test]
fn creating_a_library_where_one_already_exists_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("lib");
    library::create_library(&env.config, &folder).unwrap();

    let err = library::create_library(&env.config, &folder).unwrap_err();

    assert!(matches!(err, CoreError::AlreadyALibrary { .. }), "{err:?}");
}

#[test]
fn creating_a_library_in_a_non_empty_folder_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("busy");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("notes.txt"), "hello").unwrap();

    let err = library::create_library(&env.config, &folder).unwrap_err();

    assert!(matches!(err, CoreError::FolderNotEmpty { .. }), "{err:?}");
}

#[test]
fn founder_can_open_an_existing_library_from_any_path() {
    let env = TestEnv::new();
    let folder = env.path("drive/Volumes/External/Arete Library");
    library::create_library(&env.config, &folder).unwrap();

    let other_app = TestEnv::new();
    let lib = library::open_library(&other_app.config, &folder).unwrap();

    assert_eq!(lib.root(), folder.as_path());
    assert_eq!(lib.schema_version().unwrap(), library::latest_schema_version());
}

#[test]
fn opening_a_folder_that_is_not_a_library_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("random");
    std::fs::create_dir_all(&folder).unwrap();

    let err = library::open_library(&env.config, &folder).unwrap_err();

    assert!(matches!(err, CoreError::NotALibrary { .. }), "{err:?}");
}

#[test]
fn app_reopens_the_last_library_on_launch() {
    let env = TestEnv::new();
    let first = env.path("first");
    let second = env.path("second");
    library::create_library(&env.config, &first).unwrap();
    library::create_library(&env.config, &second).unwrap();
    library::open_library(&env.config, &first).unwrap();

    // A new launch reads the same app config folder.
    let relaunch = AppConfig::new(env.config.dir());
    let lib = library::reopen_last_library(&relaunch).unwrap().unwrap();

    assert_eq!(lib.root(), first.as_path());
}

#[test]
fn first_launch_has_no_library_to_reopen() {
    let env = TestEnv::new();

    let reopened = library::reopen_last_library(&env.config).unwrap();

    assert!(reopened.is_none());
}

#[test]
fn reopened_library_keeps_its_data_folders() {
    let env = TestEnv::new();
    let folder = env.path("lib");
    let created = library::create_library(&env.config, &folder).unwrap();
    std::fs::write(created.documents_dir().join("essay.md"), "# Hi").unwrap();
    drop(created);

    let lib = library::reopen_last_library(&env.config).unwrap().unwrap();

    assert!(lib.documents_dir().join("essay.md").is_file());
}

#[test]
fn a_new_library_is_at_the_latest_schema_version() {
    let env = TestEnv::new();

    let lib = library::create_library(&env.config, &env.path("lib")).unwrap();

    assert_eq!(lib.schema_version().unwrap(), library::latest_schema_version());
}

#[test]
fn opening_a_library_with_an_older_schema_runs_the_missing_migrations() {
    use arete_lib::migrations::Migration;
    let v1 = Migration { name: "0001_test_a", sql: "CREATE TABLE a (id INTEGER);" };
    let v2 = Migration { name: "0002_test_b", sql: "CREATE TABLE b (id INTEGER);" };
    let env = TestEnv::new();
    let folder = env.path("lib");
    let old = Library::create_with_migrations(&folder, &[v1]).unwrap();
    assert_eq!(old.schema_version().unwrap(), 1);
    drop(old);

    let lib = Library::open_with_migrations(&folder, &[v1, v2]).unwrap();

    assert_eq!(lib.schema_version().unwrap(), 2);
    // Opening again does not re-run anything (v2 would fail: table exists).
    drop(lib);
    let again = Library::open_with_migrations(&folder, &[v1, v2]).unwrap();
    assert_eq!(again.schema_version().unwrap(), 2);
}

#[test]
fn a_library_at_version_zero_is_migrated_to_the_latest_schema_on_open() {
    let env = TestEnv::new();
    let folder = env.path("lib");
    drop(Library::create_with_migrations(&folder, &[]).unwrap());

    let lib = library::open_library(&env.config, &folder).unwrap();

    assert_eq!(lib.schema_version().unwrap(), library::latest_schema_version());
}

#[test]
fn a_library_from_a_newer_app_version_is_refused() {
    use arete_lib::migrations::Migration;
    let v1 = Migration { name: "0001_test_a", sql: "CREATE TABLE a (id INTEGER);" };
    let v2 = Migration { name: "0002_test_b", sql: "CREATE TABLE b (id INTEGER);" };
    let env = TestEnv::new();
    let folder = env.path("lib");
    drop(Library::create_with_migrations(&folder, &[v1, v2]).unwrap());

    let err = Library::open_with_migrations(&folder, &[v1]).unwrap_err();

    assert!(matches!(err, CoreError::NewerSchema { .. }), "{err:?}");
}
