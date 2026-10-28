//! Library core: Moodboards on an owner (a Thesis or a Style) and
//! their saved canvas scene.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::error::CoreError;
use arete_lib::library;
use arete_lib::moodboard::{self, MoodboardOwner};
use arete_lib::drop;
use arete_lib::library::Library;
use arete_lib::style::{self, Style};
use arete_lib::thesis;
use common::TestEnv;

#[test]
fn founder_can_create_several_moodboards_on_a_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let owner = MoodboardOwner::Thesis(t.id);

    moodboard::create_moodboard(&lib, owner, "Textures").unwrap();
    moodboard::create_moodboard(&lib, owner, "Palette").unwrap();

    let titles: Vec<String> =
        moodboard::list_moodboards(&lib, owner).unwrap().into_iter().map(|m| m.title).collect();
    assert_eq!(titles, vec!["Textures", "Palette"]);
}

#[test]
fn a_thesis_lists_only_its_own_moodboards() {
    let env = TestEnv::new();
    let lib = env.library();
    let a = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let b = thesis::create_thesis(&lib, "Memento mori", "").unwrap();

    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(a.id), "Textures").unwrap();

    assert_eq!(moodboard::list_moodboards(&lib, MoodboardOwner::Thesis(a.id)).unwrap(), vec![board]);
    assert!(moodboard::list_moodboards(&lib, MoodboardOwner::Thesis(b.id)).unwrap().is_empty());
}

#[test]
fn a_moodboard_needs_an_existing_owner_and_a_title() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    let missing = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id + 1), "Textures").unwrap_err();
    let blank = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id), "   ").unwrap_err();

    assert!(matches!(missing, CoreError::NotFound { record: "Thesis", .. }), "{missing:?}");
    assert!(matches!(blank, CoreError::InvalidInput { .. }), "{blank:?}");
    assert!(moodboard::list_moodboards(&lib, MoodboardOwner::Thesis(t.id)).unwrap().is_empty());
}

#[test]
fn the_owner_crosses_ipc_as_kind_and_id() {
    let json = serde_json::to_value(MoodboardOwner::Thesis(7)).unwrap();

    assert_eq!(json, serde_json::json!({ "kind": "thesis", "id": 7 }));
    let back: MoodboardOwner = serde_json::from_value(json).unwrap();
    assert_eq!(back, MoodboardOwner::Thesis(7));
}

#[test]
fn a_new_moodboard_has_no_saved_scene() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id), "Textures").unwrap();

    assert_eq!(moodboard::load_moodboard_scene(&lib, board.id).unwrap(), None);
}

const SCENE: &str = r#"{"type":"excalidraw","elements":[{"id":"a","type":"image","fileId":"abc.png"}]}"#;

#[test]
fn the_canvas_scene_persists_across_app_restarts() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id), "Textures").unwrap();

    moodboard::save_moodboard_scene(&lib, board.id, SCENE).unwrap();
    let folder = lib.root().to_path_buf();
    drop(lib);
    let lib = library::open_library(&env.config, &folder).unwrap();

    assert_eq!(moodboard::load_moodboard_scene(&lib, board.id).unwrap().as_deref(), Some(SCENE));
    assert_eq!(moodboard::list_moodboards(&lib, MoodboardOwner::Thesis(t.id)).unwrap(), vec![board]);
}

#[test]
fn a_later_save_replaces_the_scene() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id), "Textures").unwrap();

    moodboard::save_moodboard_scene(&lib, board.id, SCENE).unwrap();
    moodboard::save_moodboard_scene(&lib, board.id, r#"{"elements":[]}"#).unwrap();

    assert_eq!(moodboard::load_moodboard_scene(&lib, board.id).unwrap().as_deref(), Some(r#"{"elements":[]}"#));
}

#[test]
fn a_scene_that_is_not_json_is_refused_and_the_old_scene_stays() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id), "Textures").unwrap();
    moodboard::save_moodboard_scene(&lib, board.id, SCENE).unwrap();

    let err = moodboard::save_moodboard_scene(&lib, board.id, "{not json").unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(moodboard::load_moodboard_scene(&lib, board.id).unwrap().as_deref(), Some(SCENE));
}

#[test]
fn saving_a_scene_on_a_missing_moodboard_is_not_found() {
    let env = TestEnv::new();
    let lib = env.library();

    let save = moodboard::save_moodboard_scene(&lib, 42, SCENE).unwrap_err();
    let load = moodboard::load_moodboard_scene(&lib, 42).unwrap_err();

    assert!(matches!(save, CoreError::NotFound { record: "Moodboard", id: 42 }), "{save:?}");
    assert!(matches!(load, CoreError::NotFound { record: "Moodboard", id: 42 }), "{load:?}");
}

#[test]
fn saving_a_scene_is_refused_when_the_library_is_disconnected() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Thesis(t.id), "Textures").unwrap();
    std::fs::rename(lib.root(), env.path("unplugged")).unwrap();

    let err = moodboard::save_moodboard_scene(&lib, board.id, SCENE).unwrap_err();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
}

/// A new Style on a new Drop on a new Thesis.
fn new_style(lib: &Library) -> Style {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap()
}

#[test]
fn founder_can_create_a_moodboard_on_a_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    let owner = MoodboardOwner::Style(s.id);

    let board = moodboard::create_moodboard(&lib, owner, "Tee references").unwrap();

    assert_eq!(board.owner, owner);
    assert_eq!(moodboard::list_moodboards(&lib, owner).unwrap(), vec![board]);
}

#[test]
fn a_style_moodboard_needs_an_existing_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);

    let err = moodboard::create_moodboard(&lib, MoodboardOwner::Style(s.id + 1), "Tee references").unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Style", .. }), "{err:?}");
}

#[test]
fn a_style_owner_crosses_ipc_as_kind_and_id() {
    let json = serde_json::to_value(MoodboardOwner::Style(3)).unwrap();

    assert_eq!(json, serde_json::json!({ "kind": "style", "id": 3 }));
    let back: MoodboardOwner = serde_json::from_value(json).unwrap();
    assert_eq!(back, MoodboardOwner::Style(3));
}

#[test]
fn a_style_moodboard_scene_persists_across_app_restarts() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    let owner = MoodboardOwner::Style(s.id);
    let board = moodboard::create_moodboard(&lib, owner, "Tee references").unwrap();

    moodboard::save_moodboard_scene(&lib, board.id, SCENE).unwrap();
    let folder = lib.root().to_path_buf();
    drop(lib);
    let lib = library::open_library(&env.config, &folder).unwrap();

    assert_eq!(moodboard::load_moodboard_scene(&lib, board.id).unwrap().as_deref(), Some(SCENE));
    assert_eq!(moodboard::list_moodboards(&lib, owner).unwrap(), vec![board]);
}

#[test]
fn a_style_has_at_most_one_moodboard() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    let owner = MoodboardOwner::Style(s.id);
    let board = moodboard::create_moodboard(&lib, owner, "Tee references").unwrap();

    let err = moodboard::create_moodboard(&lib, owner, "Second board").unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(moodboard::list_moodboards(&lib, owner).unwrap(), vec![board]);
}
