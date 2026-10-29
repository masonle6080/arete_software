//! Library core: Design Decisions on a Style, linked to Passages (spec #1,
//! ticket #16). A Design Decision has an aspect and a required rationale,
//! and links to zero or more Passages from any Essay.
//!
//! These tests call the domain API directly against a fresh temporary
//! library, and check results only through the same API.

mod common;

use arete_lib::design_decision::{self, DesignAspect};
use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::drop;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::style::{self, StyleId, StyleStatus};
use arete_lib::thesis;
use common::TestEnv;

fn a_style(lib: &Library) -> StyleId {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap().id
}

/// A new Essay titled `title` that holds one Passage. Returns the Essay's ID.
fn an_essay_with_passage(lib: &Library, title: &str, passage_id: &str, text: &str) -> i64 {
    let essay = document::create_document(lib, DocumentKind::Essay, title).unwrap();
    let markdown = format!("Before <!--passage:{passage_id}-->{text}<!--/passage:{passage_id}--> after.\n");
    let edit = DocumentEdit { title: title.into(), markdown };
    let report = [PassageText { id: passage_id.into(), text: text.into() }];
    document::save_document_with_passages(lib, essay.id, &edit, &report).unwrap();
    essay.id
}

#[test]
fn a_design_decision_records_its_aspect_and_rationale_on_the_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);

    let added =
        design_decision::add_design_decision(&lib, style_id, DesignAspect::Fabric, "  Heavy cotton, to last.  ").unwrap();

    assert_eq!(added.style_id, style_id);
    assert_eq!(added.aspect, DesignAspect::Fabric);
    assert_eq!(added.rationale, "Heavy cotton, to last.");
    assert!(added.passages.is_empty());
    assert_eq!(design_decision::style_design_decisions(&lib, style_id).unwrap(), vec![added]);
}

#[test]
fn an_empty_rationale_is_refused_and_nothing_is_added() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);

    for blank in ["", "   \n\t"] {
        let err = design_decision::add_design_decision(&lib, style_id, DesignAspect::Color, blank).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    }
    assert!(design_decision::style_design_decisions(&lib, style_id).unwrap().is_empty());
}

#[test]
fn a_design_decision_needs_an_existing_style() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = design_decision::add_design_decision(&lib, 999, DesignAspect::Other, "Why").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", .. }), "{err:?}");
}

#[test]
fn a_design_decision_links_passages_from_several_essays_in_link_order() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let one = an_essay_with_passage(&lib, "One", "p-one", "the weight of a day");
    let two = an_essay_with_passage(&lib, "Two", "p-two", "bone white");
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Color, "Bone.").unwrap();

    design_decision::link_passage(&lib, decision.id, "p-two").unwrap();
    let linked = design_decision::link_passage(&lib, decision.id, "p-one").unwrap();

    let passages: Vec<_> = linked.passages.iter().map(|p| (p.id.as_str(), p.document_id)).collect();
    assert_eq!(passages, [("p-two", two), ("p-one", one)]);
    assert_eq!(linked.passages[1].last_known_text, "the weight of a day");
    assert_eq!(design_decision::get_design_decision(&lib, decision.id).unwrap(), linked);
}

#[test]
fn linking_a_passage_twice_keeps_one_link() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    an_essay_with_passage(&lib, "One", "p-one", "words");
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Fabric, "Why").unwrap();

    design_decision::link_passage(&lib, decision.id, "p-one").unwrap();
    let again = design_decision::link_passage(&lib, decision.id, "p-one").unwrap();

    assert_eq!(again.passages.len(), 1);
}

#[test]
fn unlinking_the_last_passage_leaves_a_valid_design_decision() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    an_essay_with_passage(&lib, "One", "p-one", "words");
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Silhouette, "Boxy.").unwrap();
    design_decision::link_passage(&lib, decision.id, "p-one").unwrap();

    let unlinked = design_decision::unlink_passage(&lib, decision.id, "p-one").unwrap();

    assert!(unlinked.passages.is_empty());
    assert_eq!(unlinked.rationale, "Boxy.");
    assert_eq!(design_decision::style_design_decisions(&lib, style_id).unwrap(), vec![unlinked]);
}

#[test]
fn linking_an_unknown_passage_or_decision_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    an_essay_with_passage(&lib, "One", "p-one", "words");
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Other, "Why").unwrap();

    let err = design_decision::link_passage(&lib, decision.id, "p-nowhere").unwrap_err();
    assert!(matches!(err, CoreError::PassageNotFound { .. }), "{err:?}");
    let err = design_decision::link_passage(&lib, 999, "p-one").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Design Decision", .. }), "{err:?}");
    assert!(design_decision::get_design_decision(&lib, decision.id).unwrap().passages.is_empty());
}

#[test]
fn a_link_to_an_orphaned_passage_stays() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let essay = an_essay_with_passage(&lib, "One", "p-one", "words");
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Other, "Why").unwrap();
    design_decision::link_passage(&lib, decision.id, "p-one").unwrap();

    let edit = DocumentEdit { title: "One".into(), markdown: "No more words.\n".into() };
    document::save_document_with_passages(&lib, essay, &edit, &[]).unwrap();

    let kept = design_decision::get_design_decision(&lib, decision.id).unwrap();
    assert_eq!(kept.passages.len(), 1);
    assert!(kept.passages[0].orphaned);
    assert_eq!(kept.passages[0].last_known_text, "words");
}

#[test]
fn a_passage_lists_its_design_decisions_with_their_styles() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    an_essay_with_passage(&lib, "One", "p-one", "words");
    an_essay_with_passage(&lib, "Two", "p-two", "other words");
    let fabric = design_decision::add_design_decision(&lib, style_id, DesignAspect::Fabric, "Heavy.").unwrap();
    let color = design_decision::add_design_decision(&lib, style_id, DesignAspect::Color, "Bone.").unwrap();
    let other = design_decision::add_design_decision(&lib, style_id, DesignAspect::Other, "Unlinked.").unwrap();
    design_decision::link_passage(&lib, fabric.id, "p-one").unwrap();
    design_decision::link_passage(&lib, color.id, "p-one").unwrap();
    design_decision::link_passage(&lib, other.id, "p-two").unwrap();

    let listed = design_decision::passage_design_decisions(&lib, "p-one").unwrap();

    let rows: Vec<_> = listed.iter().map(|d| (d.id, d.rationale.as_str(), d.style_name.as_str())).collect();
    assert_eq!(rows, [(fabric.id, "Heavy.", "Heavy tee"), (color.id, "Bone.", "Heavy tee")]);
    assert_eq!(listed[0].style_id, style_id);
    assert_eq!(listed[0].aspect, DesignAspect::Fabric);

    let err = design_decision::passage_design_decisions(&lib, "p-nowhere").unwrap_err();
    assert!(matches!(err, CoreError::PassageNotFound { .. }), "{err:?}");
}

#[test]
fn a_cut_style_still_shows_its_design_decisions() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    an_essay_with_passage(&lib, "One", "p-one", "words");
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Decoration, "A print.").unwrap();
    let decision = design_decision::link_passage(&lib, decision.id, "p-one").unwrap();

    style::move_style(&lib, style_id, StyleStatus::Cut).unwrap();

    assert_eq!(design_decision::style_design_decisions(&lib, style_id).unwrap(), vec![decision]);
    let listed = design_decision::passage_design_decisions(&lib, "p-one").unwrap();
    assert_eq!(listed[0].style_status, StyleStatus::Cut);
}
