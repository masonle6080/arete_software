//! Library core: the Drop lifecycle and its cascades onto the Thesis.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::drop::{self, DropStage};
use arete_lib::error::CoreError;
use arete_lib::thesis::{self, ThesisStatus};
use common::TestEnv;

#[test]
fn creating_a_drop_commits_its_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    thesis::move_thesis(&lib, t.id, ThesisStatus::Developing).unwrap();

    let d = drop::create_drop(&lib, t.id, "Amor fati SS27").unwrap();

    assert_eq!(d.name, "Amor fati SS27");
    assert_eq!(d.thesis_id, t.id);
    assert_eq!(d.stage, DropStage::Design);
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Committed);
}

#[test]
fn a_second_active_drop_on_one_thesis_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let first = drop::create_drop(&lib, t.id, "First").unwrap();

    let err = drop::create_drop(&lib, t.id, "Second").unwrap_err();

    assert!(matches!(err, CoreError::ActiveDropExists { thesis_id } if thesis_id == t.id), "{err:?}");
    let drops = drop::thesis_drops(&lib, t.id).unwrap();
    assert_eq!(drops.drops, vec![first]);
    assert!(!drops.can_create_drop);
}

/// Moves a Drop without confirmation and expects the move to go through.
fn move_to(lib: &arete_lib::library::Library, id: drop::DropId, to: DropStage) -> drop::Drop {
    match drop::move_drop(lib, id, to, false).unwrap() {
        drop::DropMove::Moved(d) => d,
        other => panic!("→ {to}: {other:?}"),
    }
}

#[test]
fn a_drop_moves_through_its_stages_in_order() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(&lib, t.id, "Amor fati").unwrap();
    assert_eq!(d.moves, vec![DropStage::Sampling, DropStage::Cancelled]);

    assert_eq!(move_to(&lib, d.id, DropStage::Sampling).stage, DropStage::Sampling);
    let d = move_to(&lib, d.id, DropStage::Production);

    assert_eq!(d.stage, DropStage::Production);
    assert_eq!(drop::get_drop(&lib, d.id).unwrap().stage, DropStage::Production);
    assert_eq!(d.moves, vec![DropStage::Released, DropStage::Cancelled]);
    // The Thesis stays Committed until the Drop is Released.
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Committed);
}

#[test]
fn releasing_the_drop_releases_its_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(&lib, t.id, "Amor fati").unwrap();
    move_to(&lib, d.id, DropStage::Sampling);
    move_to(&lib, d.id, DropStage::Production);

    let d = move_to(&lib, d.id, DropStage::Released);

    assert_eq!(d.stage, DropStage::Released);
    assert_eq!(d.moves, vec![]);
    let t = thesis::get_thesis(&lib, t.id).unwrap();
    assert_eq!(t.status, ThesisStatus::Released);
    assert_eq!(t.manual_moves, vec![ThesisStatus::Archived]);
}

#[test]
fn cancelling_keeps_the_drop_as_cancelled_and_returns_the_thesis_to_developing() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(&lib, t.id, "First try").unwrap();
    move_to(&lib, d.id, DropStage::Sampling);

    let cancelled = move_to(&lib, d.id, DropStage::Cancelled);

    assert_eq!(cancelled.stage, DropStage::Cancelled);
    assert_eq!(cancelled.moves, vec![]);
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Developing);
    let drops = drop::thesis_drops(&lib, t.id).unwrap();
    assert_eq!(drops.drops, vec![cancelled.clone()]);
    assert!(drops.can_create_drop);

    // The Thesis can now get a new Drop; the cancelled one stays listed.
    let second = drop::create_drop(&lib, t.id, "Second try").unwrap();
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Committed);
    assert_eq!(drop::thesis_drops(&lib, t.id).unwrap().drops, vec![cancelled, second]);
}

#[test]
fn a_released_drop_cannot_be_cancelled() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(&lib, t.id, "Amor fati").unwrap();
    for to in [DropStage::Sampling, DropStage::Production, DropStage::Released] {
        move_to(&lib, d.id, to);
    }

    let err = drop::move_drop(&lib, d.id, DropStage::Cancelled, false).unwrap_err();

    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Released", to: "Cancelled", .. }), "{err:?}");
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Released);
}

#[test]
fn an_abandoned_thesis_cannot_get_a_drop() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Dead end", "").unwrap();
    thesis::move_thesis(&lib, t.id, ThesisStatus::Abandoned).unwrap();

    let err = drop::create_drop(&lib, t.id, "Dead end").unwrap_err();

    assert!(matches!(err, CoreError::StatusMoveRefused { what: "Thesis", from: "Abandoned", .. }), "{err:?}");
    let drops = drop::thesis_drops(&lib, t.id).unwrap();
    assert_eq!(drops.drops, vec![]);
    assert!(!drops.can_create_drop);
}

#[test]
fn a_drop_needs_a_name_and_an_existing_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    let err = drop::create_drop(&lib, t.id, "  ").unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Seed);

    let err = drop::create_drop(&lib, 999, "Ghost").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Thesis", id: 999 }), "{err:?}");
}

#[test]
fn drop_stages_cannot_be_skipped_or_reversed() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(&lib, t.id, "Amor fati").unwrap();

    for to in [DropStage::Production, DropStage::Released, DropStage::Design] {
        let err = drop::move_drop(&lib, d.id, to, false).unwrap_err();
        assert!(matches!(err, CoreError::StatusMoveRefused { what: "Drop", from: "Design", .. }), "→ {to}: {err:?}");
    }
    move_to(&lib, d.id, DropStage::Sampling);
    let err = drop::move_drop(&lib, d.id, DropStage::Design, false).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Sampling", to: "Design", .. }), "{err:?}");

    assert_eq!(drop::get_drop(&lib, d.id).unwrap().stage, DropStage::Sampling);
}
