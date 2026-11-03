//! Library core: Styles on a Thesis, the Style lifecycle, and attached
//! design files. The Thesis's Production guard and the statuses that take
//! new Styles are tested in `thesis.rs`.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::media;
use arete_lib::style::{self, StyleStatus};
use arete_lib::thesis::Thesis;
use common::TestEnv;

/// A new Thesis in Design.
fn new_thesis(lib: &Library) -> Thesis {
    common::design_thesis(lib, "Amor fati")
}

#[test]
fn styles_are_added_to_a_thesis_each_in_one_color() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);

    let tee = style::create_style(&lib, t.id, "Heavy tee", "Bone").unwrap();
    let hoodie = style::create_style(&lib, t.id, " Hoodie ", " Black ").unwrap();

    assert_eq!(tee.thesis_id, t.id);
    assert_eq!((tee.name.as_str(), tee.color.as_str()), ("Heavy tee", "Bone"));
    assert_eq!((hoodie.name.as_str(), hoodie.color.as_str()), ("Hoodie", "Black"));
    assert_eq!(tee.status, StyleStatus::Draft);
    assert_eq!(style::thesis_styles(&lib, t.id).unwrap(), vec![tee.clone(), hoodie]);
    assert_eq!(style::get_style(&lib, tee.id).unwrap(), tee);
}

#[test]
fn a_style_moves_from_draft_through_sampling_to_approved() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);
    let s = style::create_style(&lib, t.id, "Heavy tee", "Bone").unwrap();
    assert_eq!(s.moves, vec![StyleStatus::Sampling, StyleStatus::Cut]);

    let s = style::move_style(&lib, s.id, StyleStatus::Sampling).unwrap();
    assert_eq!(s.status, StyleStatus::Sampling);
    assert_eq!(s.moves, vec![StyleStatus::Approved, StyleStatus::Cut]);

    let s = style::move_style(&lib, s.id, StyleStatus::Approved).unwrap();
    assert_eq!(s.status, StyleStatus::Approved);
    assert_eq!(s.moves, vec![StyleStatus::Cut]);
    assert_eq!(style::get_style(&lib, s.id).unwrap(), s);
}

#[test]
fn style_statuses_cannot_be_skipped_or_reversed() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);
    let s = style::create_style(&lib, t.id, "Heavy tee", "Bone").unwrap();

    let err = style::move_style(&lib, s.id, StyleStatus::Approved).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { what: "Style", from: "Draft", to: "Approved" }), "{err:?}");
    style::move_style(&lib, s.id, StyleStatus::Sampling).unwrap();
    let err = style::move_style(&lib, s.id, StyleStatus::Draft).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Sampling", to: "Draft", .. }), "{err:?}");

    assert_eq!(style::get_style(&lib, s.id).unwrap().status, StyleStatus::Sampling);
}

#[test]
fn a_style_can_be_cut_from_any_status_and_stays_visible() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);
    let draft = style::create_style(&lib, t.id, "Tee", "Bone").unwrap();
    let sampling = style::create_style(&lib, t.id, "Hoodie", "Black").unwrap();
    style::move_style(&lib, sampling.id, StyleStatus::Sampling).unwrap();
    let approved = style::create_style(&lib, t.id, "Cap", "Navy").unwrap();
    style::move_style(&lib, approved.id, StyleStatus::Sampling).unwrap();
    style::move_style(&lib, approved.id, StyleStatus::Approved).unwrap();

    for s in [&draft, &sampling, &approved] {
        let cut = style::move_style(&lib, s.id, StyleStatus::Cut).unwrap();
        assert_eq!(cut.status, StyleStatus::Cut);
        assert_eq!(cut.moves, vec![]);
    }

    let styles = style::thesis_styles(&lib, t.id).unwrap();
    assert_eq!(styles.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Tee", "Hoodie", "Cap"]);
    assert!(styles.iter().all(|s| s.status == StyleStatus::Cut));
    let err = style::move_style(&lib, draft.id, StyleStatus::Draft).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Cut", .. }), "{err:?}");
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn design_files_attach_to_a_style_and_are_stored_in_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);
    let s = style::create_style(&lib, t.id, "Tee", "Bone").unwrap();
    let flat = source_file(&env, "Tee flat.ai", b"flat sketch");
    let back = source_file(&env, "back.PROCREATE", b"back sketch");

    let files = style::attach_style_files(&lib, s.id, &[flat.clone()]).unwrap();
    assert_eq!(files.len(), 1);
    let files = style::attach_style_files(&lib, s.id, &[back]).unwrap();

    let names: Vec<&str> = files.iter().map(|f| f.file_name.as_str()).collect();
    assert_eq!(names, ["Tee flat.ai", "back.PROCREATE"]);
    assert!(files[0].path.starts_with(lib.media_dir()), "{:?}", files[0].path);
    assert_eq!(files[0].path, media::media_path(&lib, &files[0].media));
    std::fs::remove_file(&flat).unwrap();
    assert_eq!(std::fs::read(&files[0].path).unwrap(), b"flat sketch");
    assert_eq!(style::style_files(&lib, s.id).unwrap(), files);
}

#[test]
fn a_cut_style_keeps_its_design_files() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);
    let s = style::create_style(&lib, t.id, "Tee", "Bone").unwrap();
    let files = style::attach_style_files(&lib, s.id, &[source_file(&env, "flat.pdf", b"flat")]).unwrap();

    style::move_style(&lib, s.id, StyleStatus::Cut).unwrap();

    assert_eq!(style::style_files(&lib, s.id).unwrap(), files);
}

#[test]
fn attaching_files_to_a_missing_style_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let file = source_file(&env, "flat.pdf", b"flat");

    let err = style::attach_style_files(&lib, 999, &[file]).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Style", id: 999 }), "{err:?}");
}

#[test]
fn a_style_needs_a_name_a_color_and_an_existing_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = new_thesis(&lib);

    for (name, color) in [("  ", "Bone"), ("Tee", " ")] {
        let err = style::create_style(&lib, t.id, name, color).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{name:?}/{color:?}: {err:?}");
    }
    let err = style::create_style(&lib, 999, "Tee", "Bone").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Thesis", id: 999 }), "{err:?}");
    let err = style::move_style(&lib, 999, StyleStatus::Cut).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", id: 999 }), "{err:?}");
    assert_eq!(style::thesis_styles(&lib, t.id).unwrap(), vec![]);
}
