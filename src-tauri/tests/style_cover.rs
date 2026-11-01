//! Library core: a Style's Cover, the one image picked from its design files
//! (spec #34, ticket #40).
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::drop::{self, DropId};
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::style::{self, StyleFile, StyleId};
use arete_lib::thesis;
use common::TestEnv;

fn a_drop(lib: &Library) -> DropId {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    drop::create_drop(lib, t.id, "Amor fati SS27").unwrap().id
}

fn a_style(lib: &Library, drop_id: DropId) -> StyleId {
    style::create_style(lib, drop_id, "Heavy tee", "Bone").unwrap().id
}

/// A file outside the library. Each file gets distinct bytes: media is
/// stored by content.
fn source_file(env: &TestEnv, name: &str) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, format!("bytes of {name}")).unwrap();
    path
}

/// Attaches files by name and returns them in the order given.
fn attach(env: &TestEnv, lib: &Library, id: StyleId, names: &[&str]) -> Vec<StyleFile> {
    let before = style::style_files(lib, id).unwrap().len();
    let sources: Vec<PathBuf> = names.iter().map(|n| source_file(env, n)).collect();
    style::attach_style_files(lib, id, &sources).unwrap().split_off(before)
}

#[test]
fn a_new_style_has_no_cover() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));

    assert_eq!(style::get_style(&lib, id).unwrap().cover, None);
}

#[test]
fn an_image_file_of_the_style_becomes_its_cover() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = a_drop(&lib);
    let id = a_style(&lib, d);
    let files = attach(&env, &lib, id, &["flat.ai", "front.png"]);

    let s = style::set_style_cover(&lib, id, files[1].id).unwrap();

    assert_eq!(s.cover, Some(files[1].clone()));
    assert_eq!(style::get_style(&lib, id).unwrap(), s);
    assert_eq!(style::drop_styles(&lib, d).unwrap(), vec![s]);
}

#[test]
fn png_jpeg_webp_and_heic_files_can_be_a_cover_in_any_letter_case() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let names = ["a.png", "b.jpg", "c.jpeg", "d.webp", "e.heic", "f.PNG", "g.JPG", "h.HeIc"];
    for file in attach(&env, &lib, id, &names) {
        let s = style::set_style_cover(&lib, id, file.id).unwrap();
        assert_eq!(s.cover.map(|c| c.file_name), Some(file.file_name));
    }
}

#[test]
fn a_file_that_is_not_a_png_jpeg_webp_or_heic_image_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let files = attach(&env, &lib, id, &["front.png", "flat.ai", "spec.pdf", "art.procreate", "anim.gif", "noext"]);
    style::set_style_cover(&lib, id, files[0].id).unwrap();

    for file in &files[1..] {
        let err = style::set_style_cover(&lib, id, file.id).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{}: {err:?}", file.file_name);
    }
    assert_eq!(style::get_style(&lib, id).unwrap().cover, Some(files[0].clone()));
}

#[test]
fn an_image_of_another_style_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = a_drop(&lib);
    let tee = a_style(&lib, d);
    let hoodie = style::create_style(&lib, d, "Hoodie", "Black").unwrap().id;
    let hoodie_front = attach(&env, &lib, hoodie, &["hoodie.png"]).remove(0);

    let err = style::set_style_cover(&lib, tee, hoodie_front.id).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(style::get_style(&lib, tee).unwrap().cover, None);
}

#[test]
fn a_file_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));

    let err = style::set_style_cover(&lib, id, 999).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
}

#[test]
fn setting_a_cover_on_a_missing_style_is_not_found() {
    let env = TestEnv::new();
    let lib = env.library();

    assert!(matches!(style::set_style_cover(&lib, 999, 1), Err(CoreError::NotFound { .. })));
    assert!(matches!(style::clear_style_cover(&lib, 999), Err(CoreError::NotFound { .. })));
}

#[test]
fn clearing_the_cover_leaves_the_style_with_no_cover_and_keeps_the_file() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let files = attach(&env, &lib, id, &["front.png"]);
    style::set_style_cover(&lib, id, files[0].id).unwrap();

    let s = style::clear_style_cover(&lib, id).unwrap();

    assert_eq!(s.cover, None);
    assert_eq!(style::get_style(&lib, id).unwrap().cover, None);
    assert_eq!(style::style_files(&lib, id).unwrap(), files);
}

#[test]
fn removing_the_cover_file_clears_the_cover() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let files = attach(&env, &lib, id, &["front.png", "back.png"]);
    style::set_style_cover(&lib, id, files[0].id).unwrap();

    let left = style::remove_style_file(&lib, id, files[0].id).unwrap();

    assert_eq!(left, vec![files[1].clone()]);
    assert_eq!(style::style_files(&lib, id).unwrap(), left);
    assert_eq!(style::get_style(&lib, id).unwrap().cover, None);
}

#[test]
fn removing_another_file_keeps_the_cover() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let files = attach(&env, &lib, id, &["front.png", "flat.ai"]);
    style::set_style_cover(&lib, id, files[0].id).unwrap();

    style::remove_style_file(&lib, id, files[1].id).unwrap();

    assert_eq!(style::get_style(&lib, id).unwrap().cover, Some(files[0].clone()));
}

#[test]
fn removing_a_file_of_another_style_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = a_drop(&lib);
    let tee = a_style(&lib, d);
    let hoodie = style::create_style(&lib, d, "Hoodie", "Black").unwrap().id;
    let hoodie_front = attach(&env, &lib, hoodie, &["hoodie.png"]).remove(0);

    let err = style::remove_style_file(&lib, tee, hoodie_front.id).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { .. }), "{err:?}");
    assert_eq!(style::style_files(&lib, hoodie).unwrap(), vec![hoodie_front]);
}

#[test]
fn a_duplicate_keeps_the_cover_on_its_own_copy_of_the_file() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let files = attach(&env, &lib, id, &["flat.ai", "front.png", "back.png"]);
    style::set_style_cover(&lib, id, files[2].id).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    let copy_files = style::style_files(&lib, copy.id).unwrap();
    let cover = copy.cover.clone().expect("the copy has a Cover");
    assert_eq!(cover.file_name, "back.png");
    assert_eq!(cover, copy_files[2]);
    assert_ne!(cover.id, files[2].id);
    assert_eq!(style::get_style(&lib, copy.id).unwrap(), copy);

    // The two Covers are separate: removing the copy's file leaves the original's.
    style::remove_style_file(&lib, copy.id, cover.id).unwrap();
    assert_eq!(style::get_style(&lib, copy.id).unwrap().cover, None);
    assert_eq!(style::get_style(&lib, id).unwrap().cover, Some(files[2].clone()));
}

#[test]
fn a_duplicate_of_a_style_with_no_cover_has_no_cover() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    attach(&env, &lib, id, &["front.png"]);

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert_eq!(copy.cover, None);
}

#[test]
fn a_removed_file_stays_for_a_duplicate_that_shares_it() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib, a_drop(&lib));
    let files = attach(&env, &lib, id, &["front.png"]);
    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    style::remove_style_file(&lib, id, files[0].id).unwrap();

    let copy_files = style::style_files(&lib, copy.id).unwrap();
    assert_eq!(copy_files.len(), 1);
    assert!(copy_files[0].path.exists());
}
