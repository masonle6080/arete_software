//! Library core: linking Essays to Theses (many-to-many).
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::document::{self, DocumentEdit, DocumentKind};
use arete_lib::error::CoreError;
use arete_lib::thesis;
use common::TestEnv;

fn ids<T>(items: &[T], id: impl Fn(&T) -> i64) -> Vec<i64> {
    items.iter().map(id).collect()
}

#[test]
fn founder_can_link_an_essay_to_several_theses() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "On patience").unwrap();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let time = thesis::create_thesis(&lib, "Slow time", "").unwrap();

    document::link_essay_to_thesis(&lib, essay.id, fate.id).unwrap();
    document::link_essay_to_thesis(&lib, essay.id, time.id).unwrap();

    let theses = document::theses_of_essay(&lib, essay.id).unwrap();
    assert_eq!(ids(&theses, |t| t.id), [fate.id, time.id]);
    let fate_essays = document::essays_of_thesis(&lib, fate.id).unwrap();
    assert_eq!(ids(&fate_essays, |e| e.id), [essay.id]);
    let time_essays = document::essays_of_thesis(&lib, time.id).unwrap();
    assert_eq!(ids(&time_essays, |e| e.id), [essay.id]);
}

#[test]
fn unlinking_an_essay_from_one_thesis_keeps_its_other_links() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "On patience").unwrap();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let time = thesis::create_thesis(&lib, "Slow time", "").unwrap();
    document::link_essay_to_thesis(&lib, essay.id, fate.id).unwrap();
    document::link_essay_to_thesis(&lib, essay.id, time.id).unwrap();

    document::unlink_essay_from_thesis(&lib, essay.id, fate.id).unwrap();

    let theses = document::theses_of_essay(&lib, essay.id).unwrap();
    assert_eq!(ids(&theses, |t| t.id), [time.id]);
    assert!(document::essays_of_thesis(&lib, fate.id).unwrap().is_empty());
}

#[test]
fn an_essay_unlinked_from_every_thesis_stays_a_valid_essay() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "On patience").unwrap();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    document::link_essay_to_thesis(&lib, essay.id, fate.id).unwrap();

    document::unlink_essay_from_thesis(&lib, essay.id, fate.id).unwrap();
    document::save_document(&lib, essay.id, &DocumentEdit { title: "Patience".into(), markdown: "Still here.\n".into() })
        .unwrap();

    assert!(document::theses_of_essay(&lib, essay.id).unwrap().is_empty());
    assert_eq!(document::open_document(&lib, essay.id).unwrap().markdown, "Still here.\n");
    let essays = document::list_documents(&lib, DocumentKind::Essay).unwrap();
    assert_eq!(ids(&essays, |e| e.id), [essay.id]);
    assert_eq!(thesis::get_thesis(&lib, fate.id).unwrap().title, "Amor fati");
}

#[test]
fn a_new_essay_supports_no_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Draft").unwrap();

    assert!(document::theses_of_essay(&lib, essay.id).unwrap().is_empty());
}

#[test]
fn linking_the_same_essay_and_thesis_twice_keeps_one_link() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "On patience").unwrap();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    document::link_essay_to_thesis(&lib, essay.id, fate.id).unwrap();
    document::link_essay_to_thesis(&lib, essay.id, fate.id).unwrap();

    assert_eq!(document::essays_of_thesis(&lib, fate.id).unwrap().len(), 1);
    assert_eq!(document::theses_of_essay(&lib, essay.id).unwrap().len(), 1);
}

#[test]
fn a_thesis_lists_its_essays_most_recently_edited_first_and_no_others() {
    let env = TestEnv::new();
    let lib = env.library();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let older = document::create_document(&lib, DocumentKind::Essay, "Older").unwrap();
    let newer = document::create_document(&lib, DocumentKind::Essay, "Newer").unwrap();
    document::create_document(&lib, DocumentKind::Essay, "Unlinked").unwrap();
    document::link_essay_to_thesis(&lib, older.id, fate.id).unwrap();
    document::link_essay_to_thesis(&lib, newer.id, fate.id).unwrap();

    document::save_document(&lib, older.id, &DocumentEdit { title: "Older, edited".into(), markdown: "x\n".into() })
        .unwrap();

    let essays = document::essays_of_thesis(&lib, fate.id).unwrap();
    let titles: Vec<_> = essays.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["Older, edited", "Newer"]);
}

#[test]
fn a_script_cannot_be_linked_to_a_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let script = document::create_document(&lib, DocumentKind::Script, "Film").unwrap();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    let err = document::link_essay_to_thesis(&lib, script.id, fate.id).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert!(document::essays_of_thesis(&lib, fate.id).unwrap().is_empty());
}

#[test]
fn linking_to_a_thesis_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Draft").unwrap();

    let err = document::link_essay_to_thesis(&lib, essay.id, 999).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Thesis", .. }), "{err:?}");
    assert!(document::theses_of_essay(&lib, essay.id).unwrap().is_empty());
}

#[test]
fn linking_while_the_library_is_disconnected_is_refused() {
    let env = TestEnv::new();
    let folder = env.path("Volumes/External/Arete");
    let lib = arete_lib::library::create_library(&env.config, &folder).unwrap();
    let essay = document::create_document(&lib, DocumentKind::Essay, "Draft").unwrap();
    let fate = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    std::fs::rename(&folder, env.path("unplugged")).unwrap();
    let err = document::link_essay_to_thesis(&lib, essay.id, fate.id).unwrap_err();
    std::fs::rename(env.path("unplugged"), &folder).unwrap();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert!(document::theses_of_essay(&lib, essay.id).unwrap().is_empty());
}
