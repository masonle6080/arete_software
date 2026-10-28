//! Library core: documents (Essays now, Scripts later), stored as Markdown.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results through the same API.

mod common;

use arete_lib::document::{self, DocumentEdit, DocumentKind};
use arete_lib::error::CoreError;
use common::TestEnv;

#[test]
fn founder_can_create_an_essay_with_no_thesis_and_open_it() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = document::create_document(&lib, DocumentKind::Essay, "On patience").unwrap();
    let opened = document::open_document(&lib, created.id).unwrap();

    assert_eq!(opened.title, "On patience");
    assert_eq!(opened.kind, DocumentKind::Essay);
    assert_eq!(opened.markdown, "");
}

const ESSAY: &str = "# Patience\n\nIt *takes* time.  Two spaces stay.\n\n- one\n- two\n\n\u{00e9}t\u{00e9} \u{2014} unicode\n";

#[test]
fn a_saved_essay_reloads_unchanged_after_the_library_is_reopened() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Draft").unwrap();

    document::save_document(
        &lib,
        essay.id,
        &DocumentEdit { title: "Patience".into(), markdown: ESSAY.into() },
    )
    .unwrap();
    let root = lib.root().to_path_buf();
    drop(lib);
    let lib = arete_lib::library::open_library(&env.config, &root).unwrap();
    let opened = document::open_document(&lib, essay.id).unwrap();

    assert_eq!(opened.title, "Patience");
    assert_eq!(opened.markdown, ESSAY);
}

#[test]
fn an_essay_is_a_markdown_file_in_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Draft").unwrap();

    document::save_document(
        &lib,
        essay.id,
        &DocumentEdit { title: "Draft".into(), markdown: ESSAY.into() },
    )
    .unwrap();

    let md_files: Vec<_> = std::fs::read_dir(lib.documents_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    assert_eq!(md_files.len(), 1, "{md_files:?}");
    assert_eq!(std::fs::read_to_string(&md_files[0]).unwrap(), ESSAY);
}

#[test]
fn saving_a_document_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = document::save_document(
        &lib,
        999,
        &DocumentEdit { title: "X".into(), markdown: "x\n".into() },
    )
    .unwrap_err();

    assert!(matches!(err, CoreError::NotFound { .. }), "{err:?}");
}

#[test]
fn the_essay_list_shows_essays_most_recently_edited_first() {
    let env = TestEnv::new();
    let lib = env.library();
    let first = document::create_document(&lib, DocumentKind::Essay, "First").unwrap();
    let second = document::create_document(&lib, DocumentKind::Essay, "Second").unwrap();
    document::create_document(&lib, DocumentKind::Script, "A script").unwrap();

    document::save_document(
        &lib,
        first.id,
        &DocumentEdit { title: "First, edited".into(), markdown: "More.\n".into() },
    )
    .unwrap();
    let essays = document::list_documents(&lib, DocumentKind::Essay).unwrap();

    let titles: Vec<_> = essays.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["First, edited", "Second"]);
    assert_eq!(essays[1].id, second.id);
}

#[test]
fn a_new_library_has_no_essays() {
    let env = TestEnv::new();
    let lib = env.library();

    assert!(document::list_documents(&lib, DocumentKind::Essay).unwrap().is_empty());
}

#[test]
fn saving_an_essay_while_the_library_is_disconnected_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let lib = arete_lib::library::create_library(&env.config, &folder).unwrap();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Draft").unwrap();
    document::save_document(&lib, essay.id, &DocumentEdit { title: "Draft".into(), markdown: "Kept.\n".into() })
        .unwrap();

    std::fs::rename(&folder, env.path("unplugged")).unwrap();
    let err = document::save_document(
        &lib,
        essay.id,
        &DocumentEdit { title: "Draft".into(), markdown: "Lost.\n".into() },
    )
    .unwrap_err();
    std::fs::rename(env.path("unplugged"), &folder).unwrap();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert_eq!(document::open_document(&lib, essay.id).unwrap().markdown, "Kept.\n");
}
