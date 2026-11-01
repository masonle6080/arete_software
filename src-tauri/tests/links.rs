//! Library core: the links of an item that its screen header shows as chips
//! (spec #34, ticket #38). Counts come from the core, so the UI never loads
//! full lists to count them.
//!
//! These tests call the domain API directly against a fresh temporary
//! library, and check results only through the same API.

mod common;

use arete_lib::design_decision::{self, DesignAspect};
use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::documentary;
use arete_lib::drop::{self, DropStage};
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::blank::{self, BlankInput};
use arete_lib::links::{self, BaseLink};
use arete_lib::style::{self, StyleBase, StyleId};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorInput, VendorRole};
use common::TestEnv;

/// Saves a document with one Passage per `(id, text)`, in that order.
fn save_with_passages(lib: &Library, id: i64, title: &str, passages: &[(&str, &str)]) {
    let markdown: String =
        passages.iter().map(|(p, text)| format!("<!--passage:{p}-->{text}<!--/passage:{p}--> ")).collect();
    let report: Vec<PassageText> =
        passages.iter().map(|(p, text)| PassageText { id: (*p).into(), text: (*text).into() }).collect();
    let edit = DocumentEdit { title: title.into(), markdown };
    document::save_document_with_passages(lib, id, &edit, &report).unwrap();
}

/// A new Essay that holds the Passages. Returns its ID.
fn an_essay(lib: &Library, title: &str, passages: &[(&str, &str)]) -> i64 {
    let essay = document::create_document(lib, DocumentKind::Essay, title).unwrap();
    save_with_passages(lib, essay.id, title, passages);
    essay.id
}

#[test]
fn an_essay_counts_the_passages_in_its_text_and_not_the_orphaned_ones() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = an_essay(&lib, "On Ash", &[("p-a", "ash"), ("p-b", "bone"), ("p-c", "salt")]);
    an_essay(&lib, "Other", &[("p-x", "other")]);

    save_with_passages(&lib, essay, "On Ash", &[("p-a", "ash"), ("p-b", "bone")]);

    assert_eq!(links::essay_links(&lib, essay).unwrap().passage_count, 2);
}

fn a_style(lib: &Library, thesis_title: &str) -> StyleId {
    let t = thesis::create_thesis(lib, thesis_title, "").unwrap();
    let d = drop::create_drop(lib, t.id, &format!("{thesis_title} SS27")).unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap().id
}

/// A new Design Decision on `style` motivated by the Passages.
fn a_decision(lib: &Library, style: StyleId, passages: &[&str]) {
    let dd = design_decision::add_design_decision(lib, style, DesignAspect::Color, "Why.").unwrap();
    for p in passages {
        design_decision::link_passage(lib, dd.id, p).unwrap();
    }
}

#[test]
fn an_essay_counts_each_design_decision_its_passages_motivate_once() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = an_essay(&lib, "On Ash", &[("p-a", "ash"), ("p-b", "bone"), ("p-c", "salt")]);
    an_essay(&lib, "Other", &[("p-x", "other")]);
    let tee = a_style(&lib, "Impermanence");
    let coat = a_style(&lib, "Amor fati");
    a_decision(&lib, tee, &["p-a", "p-b"]); // two Passages, one Decision
    a_decision(&lib, coat, &["p-b"]);
    a_decision(&lib, coat, &["p-x"]); // another Essay's Passage
    a_decision(&lib, coat, &[]);

    assert_eq!(links::essay_links(&lib, essay).unwrap().design_decision_count, 2);
}

#[test]
fn an_essay_does_not_count_decisions_that_only_its_orphaned_passages_motivate() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = an_essay(&lib, "On Ash", &[("p-a", "ash"), ("p-b", "bone")]);
    let tee = a_style(&lib, "Impermanence");
    a_decision(&lib, tee, &["p-a"]);
    a_decision(&lib, tee, &["p-b"]);

    save_with_passages(&lib, essay, "On Ash", &[("p-a", "ash")]);

    assert_eq!(links::essay_links(&lib, essay).unwrap().design_decision_count, 1);
}

#[test]
fn an_essay_lists_the_theses_it_supports() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = an_essay(&lib, "On Ash", &[]);
    let one = thesis::create_thesis(&lib, "Impermanence", "").unwrap();
    thesis::create_thesis(&lib, "Unlinked", "").unwrap();
    let two = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    document::link_essay_to_thesis(&lib, essay, two.id).unwrap();
    document::link_essay_to_thesis(&lib, essay, one.id).unwrap();

    let titles: Vec<String> = links::essay_links(&lib, essay).unwrap().theses.into_iter().map(|t| t.title).collect();
    assert_eq!(titles, ["Impermanence", "Amor fati"]);
}

#[test]
fn a_script_or_a_missing_document_has_no_essay_links() {
    let env = TestEnv::new();
    let lib = env.library();
    let script = document::create_document(&lib, DocumentKind::Script, "Ash film").unwrap();

    let err = links::essay_links(&lib, script.id).unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    let err = links::essay_links(&lib, 999).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { .. }), "{err:?}");
}

#[test]
fn a_style_counts_each_passage_that_motivates_its_decisions_once() {
    let env = TestEnv::new();
    let lib = env.library();
    an_essay(&lib, "On Ash", &[("p-a", "ash"), ("p-b", "bone"), ("p-c", "salt")]);
    let script = document::create_document(&lib, DocumentKind::Script, "Ash film").unwrap();
    save_with_passages(&lib, script.id, "Ash film", &[("p-s", "smoke")]);
    let tee = a_style(&lib, "Impermanence");
    let coat = a_style(&lib, "Amor fati");
    a_decision(&lib, tee, &["p-a", "p-b"]);
    a_decision(&lib, tee, &["p-b", "p-s"]); // p-b again, and a Script's Passage
    a_decision(&lib, tee, &[]);
    a_decision(&lib, coat, &["p-c"]); // another Style

    assert_eq!(links::style_links(&lib, tee).unwrap().passage_count, 3);
    assert_eq!(links::style_links(&lib, coat).unwrap().passage_count, 1);
}

#[test]
fn a_style_still_counts_an_orphaned_passage_that_motivates_it() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = an_essay(&lib, "On Ash", &[("p-a", "ash"), ("p-b", "bone")]);
    let tee = a_style(&lib, "Impermanence");
    a_decision(&lib, tee, &["p-a", "p-b"]);

    save_with_passages(&lib, essay, "On Ash", &[("p-a", "ash")]);

    assert_eq!(links::style_links(&lib, tee).unwrap().passage_count, 2);
}

#[test]
fn a_style_names_its_drop_and_the_drops_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let tee = a_style(&lib, "Impermanence");

    let found = links::style_links(&lib, tee).unwrap();
    assert_eq!(found.drop.name, "Impermanence SS27");
    assert_eq!(found.thesis.title, "Impermanence");
    assert_eq!(found.drop.thesis_id, found.thesis.id);
}

#[test]
fn a_style_names_its_base_blank_or_tech_pack() {
    let env = TestEnv::new();
    let lib = env.library();
    let tee = a_style(&lib, "Impermanence");
    let vendor_id = vendor::create_vendor(
        &lib,
        VendorInput { name: "LAA".into(), roles: vec![VendorRole::BlankSupplier], ..Default::default() },
    )
    .unwrap()
    .id;
    let blank = blank::create_blank(
        &lib,
        BlankInput {
            brand: "Los Angeles Apparel".into(),
            model: "1801GD".into(),
            vendor_id,
            colors: vec!["Bone".into()],
            sizes: vec!["M".into()],
            cost_cents: None,
            currency: String::new(),
            notes: String::new(),
        },
    )
    .unwrap();

    assert_eq!(links::style_links(&lib, tee).unwrap().base, None);

    style::set_style_base(&lib, tee, StyleBase::Blank { blank_id: blank.id }).unwrap();
    assert_eq!(links::style_links(&lib, tee).unwrap().base, Some(BaseLink::Blank { blank }));

    style::set_style_base(&lib, tee, StyleBase::TechPack).unwrap();
    assert_eq!(links::style_links(&lib, tee).unwrap().base, Some(BaseLink::TechPack));
}

#[test]
fn a_thesis_counts_the_essays_that_support_it() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Impermanence", "").unwrap();
    let other = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    for title in ["On Ash", "On Salt"] {
        document::link_essay_to_thesis(&lib, an_essay(&lib, title, &[]), t.id).unwrap();
    }
    document::link_essay_to_thesis(&lib, an_essay(&lib, "Elsewhere", &[]), other.id).unwrap();
    an_essay(&lib, "Unlinked", &[]);

    assert_eq!(links::thesis_links(&lib, t.id).unwrap().essay_count, 2);
}

#[test]
fn a_thesis_shows_its_drop_that_is_not_cancelled() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Impermanence", "").unwrap();
    assert_eq!(links::thesis_links(&lib, t.id).unwrap().drop, None);

    let first = drop::create_drop(&lib, t.id, "First").unwrap();
    drop::move_drop(&lib, first.id, DropStage::Cancelled, true).unwrap();
    assert_eq!(links::thesis_links(&lib, t.id).unwrap().drop, None);

    let second = drop::create_drop(&lib, t.id, "Second").unwrap();
    assert_eq!(links::thesis_links(&lib, t.id).unwrap().drop, Some(second));
}

#[test]
fn a_thesis_shows_its_documentary() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Impermanence", "").unwrap();
    assert_eq!(links::thesis_links(&lib, t.id).unwrap().documentary, None);

    let film = documentary::create_documentary(&lib, t.id, "Ash").unwrap();
    assert_eq!(links::thesis_links(&lib, t.id).unwrap().documentary, Some(film));
}

#[test]
fn a_missing_thesis_has_no_links() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = links::thesis_links(&lib, 999).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Thesis", .. }), "{err:?}");
}

#[test]
fn a_missing_style_has_no_links() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = links::style_links(&lib, 999).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", .. }), "{err:?}");
}
