//! Library core: Moodboard items linked to Passages (spec #1, ticket #12).
//!
//! The canvas puts the link on the element (`customData.passageId` in the
//! scene JSON, ADR-0003). Every scene save updates the links the core can
//! query, so the Passage detail view can list the Moodboard items linked to
//! it. These tests call the domain API directly and check results through it.

mod common;

use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::library::Library;
use arete_lib::moodboard::{self, Moodboard, MoodboardOwner};
use arete_lib::thesis;
use common::TestEnv;

/// An Essay with one Passage `p-brave` ("brave new").
fn essay_with_passage(lib: &Library) -> i64 {
    let essay = document::create_document(lib, DocumentKind::Essay, "On courage").unwrap();
    let md = "Hello <!--passage:p-brave-->brave new<!--/passage:p-brave--> world.\n";
    document::save_document_with_passages(
        lib,
        essay.id,
        &DocumentEdit { title: "On courage".into(), markdown: md.into() },
        &[PassageText { id: "p-brave".into(), text: "brave new".into() }],
    )
    .unwrap();
    essay.id
}

fn board(lib: &Library) -> Moodboard {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    moodboard::create_moodboard(lib, MoodboardOwner::Thesis(t.id), "Textures").unwrap()
}

fn scene(elements: &str) -> String {
    format!(r#"{{"type":"excalidraw","version":2,"elements":[{elements}]}}"#)
}

const LINKED_IMAGE: &str =
    r#"{"id":"img-1","type":"image","fileId":"abc.png","isDeleted":false,"customData":{"passageId":"p-brave"}}"#;

#[test]
fn a_canvas_item_linked_in_the_scene_is_listed_on_its_passage() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passage(&lib);
    let b = board(&lib);

    moodboard::save_moodboard_scene(&lib, b.id, &scene(LINKED_IMAGE)).unwrap();

    let items = moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].moodboard, b);
    assert_eq!(items[0].element_id, "img-1");
    assert_eq!(items[0].element_type, "image");
    assert_eq!(items[0].media.as_deref(), Some("abc.png"));
}

const PLAIN_IMAGE: &str = r#"{"id":"img-1","type":"image","fileId":"abc.png","isDeleted":false}"#;

#[test]
fn removing_the_link_from_the_scene_unlinks_the_item() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passage(&lib);
    let b = board(&lib);
    moodboard::save_moodboard_scene(&lib, b.id, &scene(LINKED_IMAGE)).unwrap();

    moodboard::save_moodboard_scene(&lib, b.id, &scene(PLAIN_IMAGE)).unwrap();

    assert!(moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap().is_empty());
}

#[test]
fn a_deleted_or_removed_canvas_item_is_no_longer_listed() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passage(&lib);
    let b = board(&lib);
    moodboard::save_moodboard_scene(&lib, b.id, &scene(LINKED_IMAGE)).unwrap();

    let deleted = r#"{"id":"img-1","type":"image","isDeleted":true,"customData":{"passageId":"p-brave"}}"#;
    moodboard::save_moodboard_scene(&lib, b.id, &scene(deleted)).unwrap();
    assert!(moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap().is_empty());

    moodboard::save_moodboard_scene(&lib, b.id, &scene(LINKED_IMAGE)).unwrap();
    moodboard::save_moodboard_scene(&lib, b.id, &scene("")).unwrap();
    assert!(moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap().is_empty());
}

#[test]
fn the_link_survives_when_the_passage_becomes_orphaned() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = essay_with_passage(&lib);
    let b = board(&lib);
    moodboard::save_moodboard_scene(&lib, b.id, &scene(LINKED_IMAGE)).unwrap();

    let edit = DocumentEdit { title: "On courage".into(), markdown: "Hello world.\n".into() };
    document::save_document_with_passages(&lib, essay, &edit, &[]).unwrap();

    assert!(document::get_passage(&lib, "p-brave").unwrap().orphaned);
    let items = moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap();
    assert_eq!(items.iter().map(|i| i.element_id.as_str()).collect::<Vec<_>>(), vec!["img-1"]);
}

#[test]
fn items_from_several_moodboards_are_listed_with_their_text_or_color() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passage(&lib);
    let a = board(&lib);
    let b = board(&lib);
    let text = r#"{"id":"t-1","type":"text","text":"Courage","backgroundColor":"transparent","customData":{"passageId":"p-brave"}}"#;
    let swatch = r##"{"id":"s-1","type":"rectangle","backgroundColor":"#c0392b","customData":{"passageId":"p-brave"}}"##;
    let other = r#"{"id":"x-1","type":"text","text":"Unlinked"}"#;

    moodboard::save_moodboard_scene(&lib, a.id, &scene(&format!("{text},{other}"))).unwrap();
    moodboard::save_moodboard_scene(&lib, b.id, &scene(swatch)).unwrap();

    let items = moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!((items[0].moodboard.id, items[0].element_type.as_str()), (a.id, "text"));
    assert_eq!(items[0].text.as_deref(), Some("Courage"));
    assert_eq!(items[0].color, None);
    assert_eq!((items[1].moodboard.id, items[1].element_type.as_str()), (b.id, "rectangle"));
    assert_eq!(items[1].color.as_deref(), Some("#c0392b"));
}

#[test]
fn a_link_to_an_unknown_passage_is_not_listed_and_does_not_block_the_save() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passage(&lib);
    let b = board(&lib);
    let dangling = r#"{"id":"img-2","type":"image","customData":{"passageId":"p-gone"}}"#;
    let both = scene(&format!("{LINKED_IMAGE},{dangling}"));

    moodboard::save_moodboard_scene(&lib, b.id, &both).unwrap();

    assert_eq!(moodboard::load_moodboard_scene(&lib, b.id).unwrap(), Some(both));
    assert_eq!(moodboard::moodboard_items_of_passage(&lib, "p-brave").unwrap().len(), 1);
}

#[test]
fn listing_items_of_a_missing_passage_fails() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = moodboard::moodboard_items_of_passage(&lib, "p-none").unwrap_err();

    assert!(matches!(err, arete_lib::error::CoreError::PassageNotFound { .. }), "{err:?}");
}
