//! Library core: Sample rounds for a Style from a Vendor, with photos,
//! feedback, Notes, and an outcome. A revised Sample leads to the next round.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::drop;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::media;
use arete_lib::note::{self, NoteInput, NoteOwner};
use arete_lib::style::{self, StyleId};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorId, VendorInput, VendorRole};
use arete_lib::sample::{self, SampleOutcome};
use common::TestEnv;

fn a_style(lib: &Library) -> StyleId {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap().id
}

fn a_vendor(lib: &Library, name: &str) -> VendorId {
    vendor::create_vendor(
        lib,
        VendorInput { name: name.into(), roles: vec![VendorRole::Manufacturer], ..Default::default() },
    )
    .unwrap()
    .id
}

#[test]
fn a_requested_sample_starts_at_round_one_pending() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let vendor_id = a_vendor(&lib, "Cut Co");

    let s = sample::request_sample(&lib, style_id, vendor_id).unwrap();

    assert_eq!((s.style_id, s.vendor_id, s.round), (style_id, vendor_id, 1));
    assert_eq!(s.outcome, SampleOutcome::Pending);
    assert_eq!(s.feedback, "");
    assert!(s.photos.is_empty());
    assert_eq!(sample::style_samples(&lib, style_id).unwrap(), vec![s.clone()]);
    assert_eq!(sample::get_sample(&lib, s.id).unwrap(), s);
}

#[test]
fn a_sample_needs_a_style_and_a_vendor_that_exist() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let vendor_id = a_vendor(&lib, "Cut Co");

    let err = sample::request_sample(&lib, 99, vendor_id).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", id: 99 }), "{err:?}");
    let err = sample::request_sample(&lib, style_id, 77).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Vendor", id: 77 }), "{err:?}");
    assert!(sample::style_samples(&lib, style_id).unwrap().is_empty());
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn photos_attach_to_a_sample_and_are_stored_in_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = sample::request_sample(&lib, a_style(&lib), a_vendor(&lib, "Cut Co")).unwrap();
    let front = source_file(&env, "front.jpg", b"front");
    let back = source_file(&env, "back.jpg", b"back");

    let photos = sample::attach_sample_photos(&lib, s.id, &[front.clone(), back]).unwrap();

    assert_eq!(photos.iter().map(|p| p.file_name.as_str()).collect::<Vec<_>>(), ["front.jpg", "back.jpg"]);
    assert!(photos[0].path.starts_with(lib.root()));
    assert_eq!(media::read_media(&lib, &photos[0].media).unwrap(), b"front");
    std::fs::remove_file(&front).unwrap();
    assert_eq!(std::fs::read(&photos[0].path).unwrap(), b"front");
    assert_eq!(sample::get_sample(&lib, s.id).unwrap().photos, photos);
}

#[test]
fn feedback_is_recorded_on_a_sample() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = sample::request_sample(&lib, a_style(&lib), a_vendor(&lib, "Cut Co")).unwrap();

    let s = sample::set_sample_feedback(&lib, s.id, "  Sleeves 2 cm too long.  ").unwrap();

    assert_eq!(s.feedback, "Sleeves 2 cm too long.");
    assert_eq!(sample::get_sample(&lib, s.id).unwrap(), s);
}

#[test]
fn notes_attach_to_a_sample() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = sample::request_sample(&lib, a_style(&lib), a_vendor(&lib, "Cut Co")).unwrap();
    let owner = NoteOwner::Sample(s.id);

    let added = note::add_note(&lib, owner, NoteInput { body: "Shipped today".into(), attachments: vec![] }).unwrap();

    assert_eq!(note::list_notes(&lib, owner).unwrap(), vec![added]);
}

#[test]
fn changing_a_missing_sample_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = sample::set_sample_feedback(&lib, 5, "x").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Sample", id: 5 }), "{err:?}");
    let err = sample::attach_sample_photos(&lib, 5, &[]).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Sample", id: 5 }), "{err:?}");
}

#[test]
fn a_pending_sample_can_be_approved_or_rejected() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let first = sample::request_sample(&lib, style_id, a_vendor(&lib, "Cut Co")).unwrap();
    let other = sample::request_sample(&lib, style_id, a_vendor(&lib, "Sew Co")).unwrap();

    let first = sample::decide_sample(&lib, first.id, SampleOutcome::Approved).unwrap();
    let other = sample::decide_sample(&lib, other.id, SampleOutcome::Rejected).unwrap();

    assert_eq!(first.outcome, SampleOutcome::Approved);
    assert_eq!(other.outcome, SampleOutcome::Rejected);
    assert_eq!(sample::style_samples(&lib, style_id).unwrap(), vec![first, other]);
}

#[test]
fn marking_a_sample_revised_creates_the_next_round_with_the_feedback_carried_forward() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let vendor_id = a_vendor(&lib, "Cut Co");
    let round1 = sample::request_sample(&lib, style_id, vendor_id).unwrap();
    sample::set_sample_feedback(&lib, round1.id, "Sleeves 2 cm too long.").unwrap();

    let round1 = sample::decide_sample(&lib, round1.id, SampleOutcome::Revised).unwrap();

    assert_eq!(round1.outcome, SampleOutcome::Revised);
    let samples = sample::style_samples(&lib, style_id).unwrap();
    assert_eq!(samples.len(), 2);
    let round2 = &samples[1];
    assert_eq!((round2.style_id, round2.vendor_id, round2.round), (style_id, vendor_id, 2));
    assert_eq!(round2.outcome, SampleOutcome::Pending);
    assert_eq!(round2.feedback, "Sleeves 2 cm too long.");
    assert!(round2.photos.is_empty());

    sample::set_sample_feedback(&lib, round2.id, "Sleeves fixed; hem puckers.").unwrap();
    sample::decide_sample(&lib, round2.id, SampleOutcome::Revised).unwrap();
    let round3 = sample::style_samples(&lib, style_id).unwrap().pop().unwrap();
    assert_eq!((round3.round, round3.outcome), (3, SampleOutcome::Pending));
    assert_eq!(round3.feedback, "Sleeves fixed; hem puckers.");
    assert_eq!(sample::get_sample(&lib, round1.id).unwrap().feedback, "Sleeves 2 cm too long.");
}

#[test]
fn rounds_are_counted_per_vendor() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let cut = sample::request_sample(&lib, style_id, a_vendor(&lib, "Cut Co")).unwrap();
    sample::decide_sample(&lib, cut.id, SampleOutcome::Revised).unwrap();

    let sew = sample::request_sample(&lib, style_id, a_vendor(&lib, "Sew Co")).unwrap();

    assert_eq!(sew.round, 1);
}

#[test]
fn a_decided_sample_cannot_be_decided_again() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let s = sample::request_sample(&lib, style_id, a_vendor(&lib, "Cut Co")).unwrap();
    sample::decide_sample(&lib, s.id, SampleOutcome::Revised).unwrap();

    let err = sample::decide_sample(&lib, s.id, SampleOutcome::Revised).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { what: "Sample", from: "Revised", to: "Revised" }), "{err:?}");
    let err = sample::decide_sample(&lib, s.id, SampleOutcome::Approved).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Revised", to: "Approved", .. }), "{err:?}");
    assert_eq!(sample::style_samples(&lib, style_id).unwrap().len(), 2);
}

#[test]
fn a_sample_cannot_be_decided_back_to_pending() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = sample::request_sample(&lib, a_style(&lib), a_vendor(&lib, "Cut Co")).unwrap();

    let err = sample::decide_sample(&lib, s.id, SampleOutcome::Pending).unwrap_err();

    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Pending", to: "Pending", .. }), "{err:?}");
}
