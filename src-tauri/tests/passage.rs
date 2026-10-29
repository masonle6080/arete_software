//! Library core: Passage save reports (spec #1, ticket #9).
//!
//! On every save, the document module reports the Passages present and their
//! text. The core keeps each Passage's last known text, orphans a Passage that
//! is missing (never deletes it), and restores it when its markers return.
//! These tests call the domain API directly and check results through it.

mod common;

use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::error::CoreError;
use common::TestEnv;

fn report(pairs: &[(&str, &str)]) -> Vec<PassageText> {
    pairs.iter().map(|(id, text)| PassageText { id: (*id).into(), text: (*text).into() }).collect()
}

fn edit(markdown: &str) -> DocumentEdit {
    DocumentEdit { title: "Essay".into(), markdown: markdown.into() }
}

const BRAVE: &str = "Hello <!--passage:p-brave-->brave new<!--/passage:p-brave--> world.\n";

#[test]
fn a_save_report_records_each_passage_with_its_text() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Essay").unwrap();

    document::save_document_with_passages(&lib, essay.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();

    let p = document::get_passage(&lib, "p-brave").unwrap();
    assert_eq!(p.document_id, essay.id);
    assert_eq!(p.last_known_text, "brave new");
    assert!(!p.orphaned);
    assert_eq!(document::open_document(&lib, essay.id).unwrap().markdown, BRAVE);
}

#[test]
fn a_later_save_report_updates_last_known_text() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Essay").unwrap();
    document::save_document_with_passages(&lib, essay.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();

    let md = "Hello <!--passage:p-brave-->very brave new<!--/passage:p-brave--> world.\n";
    document::save_document_with_passages(&lib, essay.id, &edit(md), &report(&[("p-brave", "very brave new")]))
        .unwrap();

    assert_eq!(document::get_passage(&lib, "p-brave").unwrap().last_known_text, "very brave new");
}

#[test]
fn a_passage_missing_from_a_save_report_is_orphaned_with_its_last_known_text() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Essay").unwrap();
    let both = "<!--passage:p-a-->One.<!--/passage:p-a--> <!--passage:p-b-->Two.<!--/passage:p-b-->\n";
    document::save_document_with_passages(&lib, essay.id, &edit(both), &report(&[("p-a", "One."), ("p-b", "Two.")]))
        .unwrap();

    let only_a = "<!--passage:p-a-->One.<!--/passage:p-a-->\n";
    document::save_document_with_passages(&lib, essay.id, &edit(only_a), &report(&[("p-a", "One.")])).unwrap();

    let b = document::get_passage(&lib, "p-b").unwrap();
    assert!(b.orphaned);
    assert_eq!(b.last_known_text, "Two.");
    assert!(!document::get_passage(&lib, "p-a").unwrap().orphaned);
    let all = document::passages_of_document(&lib, essay.id).unwrap();
    let states: Vec<_> = all.iter().map(|p| (p.id.as_str(), p.orphaned)).collect();
    assert_eq!(states, [("p-a", false), ("p-b", true)]);
}

#[test]
fn an_orphaned_passage_stays_orphaned_and_unchanged_over_later_saves() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Essay").unwrap();
    document::save_document_with_passages(&lib, essay.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();

    document::save_document_with_passages(&lib, essay.id, &edit("Hello world.\n"), &[]).unwrap();
    document::save_document_with_passages(&lib, essay.id, &edit("Hello world!\n"), &[]).unwrap();

    let p = document::get_passage(&lib, "p-brave").unwrap();
    assert!(p.orphaned);
    assert_eq!(p.last_known_text, "brave new");
}

#[test]
fn re_adding_an_orphaned_passages_markers_restores_it() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Essay").unwrap();
    document::save_document_with_passages(&lib, essay.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();
    document::save_document_with_passages(&lib, essay.id, &edit("Hello world.\n"), &[]).unwrap();

    // For example, an undo brings the text and its markers back.
    document::save_document_with_passages(&lib, essay.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();

    let p = document::get_passage(&lib, "p-brave").unwrap();
    assert!(!p.orphaned);
    assert_eq!(p.last_known_text, "brave new");
    assert_eq!(document::passages_of_document(&lib, essay.id).unwrap().len(), 1);
}

#[test]
fn passages_live_in_scripts_too() {
    let env = TestEnv::new();
    let lib = env.library();
    let script = document::create_document(&lib, DocumentKind::Script, "Script").unwrap();

    document::save_document_with_passages(&lib, script.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();
    document::save_document_with_passages(&lib, script.id, &edit("Hello world.\n"), &[]).unwrap();

    let p = document::get_passage(&lib, "p-brave").unwrap();
    assert_eq!(p.document_kind, DocumentKind::Script);
    assert!(p.orphaned);
}

#[test]
fn a_save_report_cannot_take_a_passage_from_another_document() {
    let env = TestEnv::new();
    let lib = env.library();
    let first = document::create_document(&lib, DocumentKind::Essay, "First").unwrap();
    let second = document::create_document(&lib, DocumentKind::Essay, "Second").unwrap();
    document::save_document_with_passages(&lib, first.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap();

    let err = document::save_document_with_passages(&lib, second.id, &edit(BRAVE), &report(&[("p-brave", "brave new")]))
        .unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(document::get_passage(&lib, "p-brave").unwrap().document_id, first.id);
    assert_eq!(document::open_document(&lib, second.id).unwrap().markdown, "", "nothing was written");
}

#[test]
fn a_save_report_with_a_malformed_or_repeated_id_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Essay").unwrap();

    for bad in [report(&[("not valid-->", "x")]), report(&[("p-a", "x"), ("p-a", "y")])] {
        let err = document::save_document_with_passages(&lib, essay.id, &edit(BRAVE), &bad).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    }
    assert!(document::passages_of_document(&lib, essay.id).unwrap().is_empty());
}

#[test]
fn an_unknown_passage_is_not_found() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = document::get_passage(&lib, "nope").unwrap_err();

    assert_eq!(err.kind(), "NotFound");
}
