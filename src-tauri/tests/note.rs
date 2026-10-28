//! Library core: the shared Notes record (dated notes with attachments), used
//! here on Vendors.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::media;
use arete_lib::note::{self, NoteInput, NoteOwner};
use arete_lib::vendor::{self, VendorId, VendorInput, VendorRole};
use common::TestEnv;

fn a_vendor(lib: &Library, name: &str) -> VendorId {
    vendor::create_vendor(
        lib,
        VendorInput { name: name.into(), roles: vec![VendorRole::Manufacturer], ..Default::default() },
    )
    .unwrap()
    .id
}

fn text(body: &str) -> NoteInput {
    NoteInput { body: body.into(), attachments: vec![] }
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

/// `YYYY-MM-DDTHH:MM:SS.sssZ`, UTC.
fn is_utc_timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 24
        && s.ends_with('Z')
        && [4, 7].iter().all(|&i| b[i] == b'-')
        && b[10] == b'T'
        && [13, 16].iter().all(|&i| b[i] == b':')
        && b[19] == b'.'
        && b.iter().enumerate().all(|(i, c)| [4, 7, 10, 13, 16, 19, 23].contains(&i) || c.is_ascii_digit())
}

#[test]
fn founder_can_add_a_note_to_a_vendor_and_see_when_it_was_created() {
    let env = TestEnv::new();
    let lib = env.library();
    let vendor_id = a_vendor(&lib, "Cut Co");
    let owner = NoteOwner::Vendor(vendor_id);

    let added = note::add_note(&lib, owner, text("Call: they can do 150 pieces per color.")).unwrap();

    assert_eq!(added.owner, owner);
    assert_eq!(added.body, "Call: they can do 150 pieces per color.");
    assert!(is_utc_timestamp(&added.created_at), "{}", added.created_at);
    assert_eq!(note::list_notes(&lib, owner).unwrap(), vec![added]);
}

#[test]
fn a_note_can_carry_attachments_stored_in_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let owner = NoteOwner::Vendor(a_vendor(&lib, "Mill"));
    let quote = source_file(&env, "Quote March.pdf", b"quote bytes");
    let swatch = source_file(&env, "swatch.JPG", b"swatch bytes");

    let added = note::add_note(
        &lib,
        owner,
        NoteInput { body: "Quote and swatch".into(), attachments: vec![quote.clone(), swatch] },
    )
    .unwrap();

    let names: Vec<&str> = added.attachments.iter().map(|a| a.file_name.as_str()).collect();
    assert_eq!(names, ["Quote March.pdf", "swatch.JPG"]);
    let stored = media::media_path(&lib, &added.attachments[0].media);
    assert!(stored.starts_with(lib.media_dir()), "{stored:?}");
    std::fs::remove_file(&quote).unwrap();
    assert_eq!(std::fs::read(&stored).unwrap(), b"quote bytes");
    assert_eq!(note::list_notes(&lib, owner).unwrap(), vec![added]);
}

#[test]
fn a_note_may_be_only_attachments() {
    let env = TestEnv::new();
    let lib = env.library();
    let owner = NoteOwner::Vendor(a_vendor(&lib, "Printer"));
    let file = source_file(&env, "proof.png", b"proof");

    let added = note::add_note(&lib, owner, NoteInput { body: "  ".into(), attachments: vec![file] }).unwrap();

    assert_eq!(added.body, "");
    assert_eq!(added.attachments.len(), 1);
}

#[test]
fn an_empty_note_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let owner = NoteOwner::Vendor(a_vendor(&lib, "Quiet"));

    let err = note::add_note(&lib, owner, text("   \n ")).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(note::list_notes(&lib, owner).unwrap(), vec![]);
}

#[test]
fn note_text_is_saved_without_surrounding_blank_space() {
    let env = TestEnv::new();
    let lib = env.library();
    let owner = NoteOwner::Vendor(a_vendor(&lib, "Trim"));

    let added = note::add_note(&lib, owner, text("\n  line one\n  line two  \n")).unwrap();

    assert_eq!(added.body, "line one\n  line two");
}

#[test]
fn notes_are_listed_newest_first_and_only_for_their_owner() {
    let env = TestEnv::new();
    let lib = env.library();
    let a = NoteOwner::Vendor(a_vendor(&lib, "A"));
    let b = NoteOwner::Vendor(a_vendor(&lib, "B"));

    let first = note::add_note(&lib, a, text("first")).unwrap();
    note::add_note(&lib, b, text("other vendor")).unwrap();
    let second = note::add_note(&lib, a, text("second")).unwrap();

    let bodies: Vec<String> = note::list_notes(&lib, a).unwrap().into_iter().map(|n| n.body).collect();
    assert_eq!(bodies, ["second", "first"]);
    assert!(second.created_at >= first.created_at);
}

#[test]
fn a_note_on_a_vendor_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = note::add_note(&lib, NoteOwner::Vendor(42), text("hello")).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Vendor", id: 42 }), "{err:?}");
}

#[test]
fn a_note_on_a_sample_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = note::add_note(&lib, NoteOwner::Sample(7), text("fit is off")).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Sample", id: 7 }), "{err:?}");
}

#[test]
fn notes_are_kept_when_the_library_is_reopened() {
    let env = TestEnv::new();
    let lib = env.library();
    let owner = NoteOwner::Vendor(a_vendor(&lib, "Durable"));
    let file = source_file(&env, "deck.pdf", b"deck");
    let added = note::add_note(&lib, owner, NoteInput { body: "kept".into(), attachments: vec![file] }).unwrap();
    let root = lib.root().to_path_buf();
    drop(lib);

    let reopened = arete_lib::library::open_library(&env.config, &root).unwrap();

    assert_eq!(note::list_notes(&reopened, owner).unwrap(), vec![added]);
}

#[test]
fn adding_a_note_is_refused_when_the_library_is_disconnected() {
    let env = TestEnv::new();
    let lib = env.library();
    let owner = NoteOwner::Vendor(a_vendor(&lib, "Offline"));
    let file = source_file(&env, "look.png", b"bytes");
    std::fs::rename(lib.root(), env.path("unplugged")).unwrap();

    let err =
        note::add_note(&lib, owner, NoteInput { body: "lost?".into(), attachments: vec![file] }).unwrap_err();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert!(!lib.root().exists(), "adding a note must not recreate the library folder");
}
