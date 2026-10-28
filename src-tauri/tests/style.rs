//! Library core: Styles on a Drop, the Style lifecycle, attached design
//! files, and the Drop's Production guard.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::media;
use arete_lib::drop::{self, Drop, DropStage};
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::style::{self, StyleStatus};
use arete_lib::thesis;
use common::TestEnv;

/// A new Drop in Design, on a new Thesis.
fn new_drop(lib: &Library) -> Drop {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    drop::create_drop(lib, t.id, "Amor fati SS27").unwrap()
}

#[test]
fn styles_are_added_to_a_drop_each_in_one_color() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_drop(&lib);

    let tee = style::create_style(&lib, d.id, "Heavy tee", "Bone").unwrap();
    let hoodie = style::create_style(&lib, d.id, " Hoodie ", " Black ").unwrap();

    assert_eq!(tee.drop_id, d.id);
    assert_eq!((tee.name.as_str(), tee.color.as_str()), ("Heavy tee", "Bone"));
    assert_eq!((hoodie.name.as_str(), hoodie.color.as_str()), ("Hoodie", "Black"));
    assert_eq!(tee.status, StyleStatus::Draft);
    assert_eq!(style::drop_styles(&lib, d.id).unwrap(), vec![tee.clone(), hoodie]);
    assert_eq!(style::get_style(&lib, tee.id).unwrap(), tee);
}

#[test]
fn a_style_moves_from_draft_through_sampling_to_approved() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_drop(&lib);
    let s = style::create_style(&lib, d.id, "Heavy tee", "Bone").unwrap();
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
    let d = new_drop(&lib);
    let s = style::create_style(&lib, d.id, "Heavy tee", "Bone").unwrap();

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
    let d = new_drop(&lib);
    let draft = style::create_style(&lib, d.id, "Tee", "Bone").unwrap();
    let sampling = style::create_style(&lib, d.id, "Hoodie", "Black").unwrap();
    style::move_style(&lib, sampling.id, StyleStatus::Sampling).unwrap();
    let approved = style::create_style(&lib, d.id, "Cap", "Navy").unwrap();
    style::move_style(&lib, approved.id, StyleStatus::Sampling).unwrap();
    style::move_style(&lib, approved.id, StyleStatus::Approved).unwrap();

    for s in [&draft, &sampling, &approved] {
        let cut = style::move_style(&lib, s.id, StyleStatus::Cut).unwrap();
        assert_eq!(cut.status, StyleStatus::Cut);
        assert_eq!(cut.moves, vec![]);
    }

    let styles = style::drop_styles(&lib, d.id).unwrap();
    assert_eq!(styles.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Tee", "Hoodie", "Cap"]);
    assert!(styles.iter().all(|s| s.status == StyleStatus::Cut));
    let err = style::move_style(&lib, draft.id, StyleStatus::Draft).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Cut", .. }), "{err:?}");
}

/// Moves a Style through Sampling to Approved.
fn approve(lib: &Library, id: style::StyleId) {
    style::move_style(lib, id, StyleStatus::Sampling).unwrap();
    style::move_style(lib, id, StyleStatus::Approved).unwrap();
}

#[test]
fn production_is_refused_while_a_style_is_not_approved_or_cut() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_drop(&lib);
    let tee = style::create_style(&lib, d.id, "Tee", "Bone").unwrap();
    let hoodie = style::create_style(&lib, d.id, "Hoodie", "Black").unwrap();
    approve(&lib, tee.id);
    style::move_style(&lib, hoodie.id, StyleStatus::Sampling).unwrap();
    drop::move_drop(&lib, d.id, DropStage::Sampling, false).unwrap();

    let err = drop::move_drop(&lib, d.id, DropStage::Production, false).unwrap_err();
    assert!(matches!(&err, CoreError::StylesNotReady { styles } if styles == &["Hoodie (Black)"]), "{err:?}");
    // Confirming does not get past a hard rule.
    let err = drop::move_drop(&lib, d.id, DropStage::Production, true).unwrap_err();
    assert!(matches!(err, CoreError::StylesNotReady { .. }), "{err:?}");
    assert_eq!(drop::get_drop(&lib, d.id).unwrap().stage, DropStage::Sampling);
}

#[test]
fn production_goes_through_once_every_style_is_approved_or_cut() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_drop(&lib);
    let tee = style::create_style(&lib, d.id, "Tee", "Bone").unwrap();
    let hoodie = style::create_style(&lib, d.id, "Hoodie", "Black").unwrap();
    approve(&lib, tee.id);
    style::move_style(&lib, hoodie.id, StyleStatus::Cut).unwrap();
    drop::move_drop(&lib, d.id, DropStage::Sampling, false).unwrap();

    let moved = drop::move_drop(&lib, d.id, DropStage::Production, false).unwrap();

    assert!(matches!(&moved, drop::DropMove::Moved(d) if d.stage == DropStage::Production), "{moved:?}");
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
    let d = new_drop(&lib);
    let s = style::create_style(&lib, d.id, "Tee", "Bone").unwrap();
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
    let d = new_drop(&lib);
    let s = style::create_style(&lib, d.id, "Tee", "Bone").unwrap();
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
fn a_style_needs_a_name_a_color_and_an_existing_drop() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = new_drop(&lib);

    for (name, color) in [("  ", "Bone"), ("Tee", " ")] {
        let err = style::create_style(&lib, d.id, name, color).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{name:?}/{color:?}: {err:?}");
    }
    let err = style::create_style(&lib, 999, "Tee", "Bone").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Drop", id: 999 }), "{err:?}");
    let err = style::move_style(&lib, 999, StyleStatus::Cut).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", id: 999 }), "{err:?}");
    assert_eq!(style::drop_styles(&lib, d.id).unwrap(), vec![]);
}
