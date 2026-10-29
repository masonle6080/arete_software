//! Exports: files the founder saves outside the library, and print jobs for
//! PDF exports. These tests call the domain API directly (no IPC).

mod common;

use arete_lib::error::CoreError;
use arete_lib::export::{self, PrintJob, PrintJobs};
use common::TestEnv;

#[test]
fn an_export_writes_the_file_outside_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let dest = env.path("Desktop/Patience.md");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();

    export::write_export(Some(lib.root()), &dest, b"# Patience\n").unwrap();

    assert_eq!(std::fs::read(&dest).unwrap(), b"# Patience\n");
}

#[test]
fn an_export_replaces_an_existing_file() {
    let env = TestEnv::new();
    let dest = env.path("Patience.md");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::write(&dest, b"old").unwrap();

    export::write_export(None, &dest, b"new").unwrap();

    assert_eq!(std::fs::read(&dest).unwrap(), b"new");
}

#[test]
fn an_export_into_the_library_folder_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let dest = lib.documents_dir().join("export.md");

    let err = export::write_export(Some(lib.root()), &dest, b"x").unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert!(!dest.exists());
}

#[test]
fn an_export_works_while_the_library_is_disconnected() {
    let env = TestEnv::new();
    let lib = env.library();
    let root = lib.root().to_path_buf();
    std::fs::rename(&root, env.path("moved")).unwrap();
    assert!(!lib.is_reachable());
    let dest = env.path("Patience.md");

    export::write_export(Some(&root), &dest, b"text").unwrap();

    assert_eq!(std::fs::read(&dest).unwrap(), b"text");
}

#[test]
fn a_print_job_is_taken_once() {
    let jobs = PrintJobs::default();
    let job = PrintJob { title: "Patience".into(), html: "<h1>Patience</h1>".into(), css: None };

    let id = jobs.put(job.clone());

    assert_eq!(jobs.take(&id), Some(job));
    assert_eq!(jobs.take(&id), None);
}

#[test]
fn each_print_job_gets_its_own_id() {
    let jobs = PrintJobs::default();
    let job = PrintJob { title: "A".into(), html: String::new(), css: None };

    let a = jobs.put(job.clone());
    let b = jobs.put(job);

    assert_ne!(a, b);
    assert_eq!(jobs.take("unknown"), None);
}
