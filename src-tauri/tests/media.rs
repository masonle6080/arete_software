//! Library core: media import with content-addressed dedup.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library.

mod common;

use arete_lib::error::CoreError;
use arete_lib::media;
use common::TestEnv;

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn import_copies_the_file_into_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "look.png", b"fake png bytes");

    let media = media::import_media(&lib, &src).unwrap();

    let stored = media::media_path(&lib, &media);
    assert!(stored.starts_with(lib.media_dir()), "{stored:?}");
    assert_eq!(std::fs::read(&stored).unwrap(), b"fake png bytes");
    // A copy, not a reference: the library keeps it when the source goes.
    std::fs::remove_file(&src).unwrap();
    assert_eq!(std::fs::read(&stored).unwrap(), b"fake png bytes");
}

fn stored_files(lib: &arete_lib::library::Library) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(lib.media_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn importing_identical_content_twice_stores_one_file() {
    let env = TestEnv::new();
    let lib = env.library();
    let first = source_file(&env, "a/clip.mov", b"same video bytes");
    let second = source_file(&env, "b/clip copy.MOV", b"same video bytes");

    let a = media::import_media(&lib, &first).unwrap();
    let b = media::import_media(&lib, &second).unwrap();

    assert_eq!(a, b);
    assert_eq!(stored_files(&lib), vec![a.file_name().to_string()]);
}

#[test]
fn identical_content_under_another_extension_reuses_the_first_copy() {
    let env = TestEnv::new();
    let lib = env.library();
    let png = source_file(&env, "ref.png", b"identical bytes");
    let jpg = source_file(&env, "ref.jpg", b"identical bytes");

    let a = media::import_media(&lib, &png).unwrap();
    let b = media::import_media(&lib, &jpg).unwrap();

    assert_eq!(a, b);
    assert_eq!(stored_files(&lib).len(), 1);
}

#[test]
fn different_content_is_stored_separately() {
    let env = TestEnv::new();
    let lib = env.library();
    let one = source_file(&env, "one.pdf", b"first document");
    let two = source_file(&env, "two.pdf", b"second document");

    let a = media::import_media(&lib, &one).unwrap();
    let b = media::import_media(&lib, &two).unwrap();

    assert_ne!(a, b);
    assert_eq!(stored_files(&lib).len(), 2);
    assert_eq!(std::fs::read(media::media_path(&lib, &b)).unwrap(), b"second document");
}

#[test]
fn the_media_name_is_the_content_hash() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "Note.TXT", b"abc");

    let media = media::import_media(&lib, &src).unwrap();

    // Known BLAKE3 digest of "abc".
    assert_eq!(
        media.file_name(),
        "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85.txt"
    );
}

#[test]
fn import_is_refused_when_the_library_is_disconnected() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "look.png", b"fake png bytes");
    // Unplugging the drive: the library folder is no longer at its path.
    let unplugged = env.path("unplugged");
    std::fs::rename(lib.root(), &unplugged).unwrap();

    let err = media::import_media(&lib, &src).unwrap_err();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert!(!lib.root().exists(), "import must not recreate the library folder");
    let media_dir = unplugged.join("media");
    assert_eq!(std::fs::read_dir(media_dir).unwrap().count(), 0, "nothing was copied");
}
