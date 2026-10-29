//! Library core: the Script of a Documentary (spec #1, ticket #27).
//!
//! Each Documentary has one Script. It is a document of kind Script, saved
//! as Markdown like an Essay, and it holds Passages that behave exactly like
//! Essay Passages. These tests call the domain API directly and check results
//! through it.

mod common;

use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::documentary::{self, Documentary};
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::thesis;
use common::TestEnv;

fn new_documentary(lib: &Library, title: &str) -> Documentary {
    let t = thesis::create_thesis(lib, title, "").unwrap();
    documentary::create_documentary(lib, t.id, title).unwrap()
}

fn report(pairs: &[(&str, &str)]) -> Vec<PassageText> {
    pairs.iter().map(|(id, text)| PassageText { id: (*id).into(), text: (*text).into() }).collect()
}

fn edit(markdown: &str) -> DocumentEdit {
    DocumentEdit { title: "Script".into(), markdown: markdown.into() }
}

#[test]
fn a_documentary_has_an_empty_script_titled_like_the_documentary() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_documentary(&lib, "Amor fati");

    let script = documentary::documentary_script(&lib, d.id).unwrap();

    assert_eq!(script.kind, DocumentKind::Script);
    assert_eq!(script.title, "Amor fati");
    assert_eq!(script.markdown, "");
}

#[test]
fn a_documentary_keeps_one_script_across_opens() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_documentary(&lib, "Amor fati");
    let first = documentary::documentary_script(&lib, d.id).unwrap();

    let again = documentary::documentary_script(&lib, d.id).unwrap();

    assert_eq!(again.id, first.id);
    assert_eq!(document::list_documents(&lib, DocumentKind::Script).unwrap().len(), 1);
}

#[test]
fn each_documentary_has_its_own_script() {
    let env = TestEnv::new();
    let lib = env.library();
    let a = new_documentary(&lib, "Amor fati");
    let b = new_documentary(&lib, "Memento mori");

    let sa = documentary::documentary_script(&lib, a.id).unwrap();
    let sb = documentary::documentary_script(&lib, b.id).unwrap();

    assert_ne!(sa.id, sb.id);
    assert_eq!(sb.title, "Memento mori");
}

#[test]
fn the_script_of_a_missing_documentary_is_not_found() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = documentary::documentary_script(&lib, 999).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Documentary", id: 999 }), "{err:?}");
}

#[test]
fn a_script_saves_as_markdown_and_reopens_unchanged() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_documentary(&lib, "Amor fati");
    let script = documentary::documentary_script(&lib, d.id).unwrap();
    let md = "# Cold open\n\nThe *river* at dawn.\n";

    document::save_document(&lib, script.id, &edit(md)).unwrap();

    let reopened = documentary::documentary_script(&lib, d.id).unwrap();
    assert_eq!(reopened.markdown, md);
    assert_eq!(reopened.title, "Script");
}

#[test]
fn script_passages_are_recorded_orphaned_and_restored_like_essay_passages() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_documentary(&lib, "Amor fati");
    let script = documentary::documentary_script(&lib, d.id).unwrap();
    let with = "Open on <!--passage:s-river-->the river<!--/passage:s-river-->.\n";

    document::save_document_with_passages(&lib, script.id, &edit(with), &report(&[("s-river", "the river")]))
        .unwrap();
    let p = document::get_passage(&lib, "s-river").unwrap();
    assert_eq!(p.document_id, script.id);
    assert_eq!(p.document_kind, DocumentKind::Script);
    assert_eq!(p.last_known_text, "the river");
    assert!(!p.orphaned);

    document::save_document_with_passages(&lib, script.id, &edit("Open on.\n"), &[]).unwrap();
    let orphan = document::get_passage(&lib, "s-river").unwrap();
    assert!(orphan.orphaned);
    assert_eq!(orphan.last_known_text, "the river");
    let orphans: Vec<_> =
        document::passages_of_document(&lib, script.id).unwrap().into_iter().filter(|p| p.orphaned).collect();
    assert_eq!(orphans.len(), 1);

    document::save_document_with_passages(&lib, script.id, &edit(with), &report(&[("s-river", "the river")]))
        .unwrap();
    assert!(!document::get_passage(&lib, "s-river").unwrap().orphaned);
}
