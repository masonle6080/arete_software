//! Library core: Thesis capture, manual status moves, and the grouped list.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::error::CoreError;
use arete_lib::thesis::{self, ThesisStatus};
use common::TestEnv;

#[test]
fn a_new_thesis_starts_as_seed_with_its_title_and_statement() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = thesis::create_thesis(&lib, "Amor fati", "Love what happens to you.").unwrap();

    let read = thesis::get_thesis(&lib, created.id).unwrap();
    assert_eq!(read.title, "Amor fati");
    assert_eq!(read.statement, "Love what happens to you.");
    assert_eq!(read.status, ThesisStatus::Seed);
}

#[test]
fn founder_can_move_a_seed_thesis_to_developing() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    thesis::move_thesis(&lib, t.id, ThesisStatus::Developing).unwrap();

    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Developing);
}

#[test]
fn moves_that_are_not_manual_are_refused_and_change_nothing() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    // Committed and Released come only from the Drop. Archived needs Released.
    for to in [ThesisStatus::Committed, ThesisStatus::Released, ThesisStatus::Archived, ThesisStatus::Seed] {
        let err = thesis::move_thesis(&lib, t.id, to).unwrap_err();
        assert!(
            matches!(err, CoreError::StatusMoveRefused { from: "Seed", .. }),
            "Seed → {to}: {err:?}"
        );
    }

    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Seed);
}

#[test]
fn an_abandoned_thesis_stays_visible_in_the_list() {
    let env = TestEnv::new();
    let lib = env.library();
    let seed = thesis::create_thesis(&lib, "Dead seed", "").unwrap();
    let dev = thesis::create_thesis(&lib, "Dead end", "").unwrap();
    thesis::move_thesis(&lib, dev.id, ThesisStatus::Developing).unwrap();

    thesis::move_thesis(&lib, seed.id, ThesisStatus::Abandoned).unwrap();
    thesis::move_thesis(&lib, dev.id, ThesisStatus::Abandoned).unwrap();

    let abandoned = titles_by_status(&lib)
        .into_iter()
        .find(|(s, _)| *s == ThesisStatus::Abandoned)
        .unwrap()
        .1;
    assert_eq!(abandoned, vec!["Dead seed".to_string(), "Dead end".to_string()]);
    assert_eq!(thesis::get_thesis(&lib, seed.id).unwrap().title, "Dead seed");
}

#[test]
fn an_abandoned_or_developing_thesis_cannot_go_back_by_hand() {
    let env = TestEnv::new();
    let lib = env.library();
    let dev = thesis::create_thesis(&lib, "In work", "").unwrap();
    thesis::move_thesis(&lib, dev.id, ThesisStatus::Developing).unwrap();
    let gone = thesis::create_thesis(&lib, "Gone", "").unwrap();
    thesis::move_thesis(&lib, gone.id, ThesisStatus::Abandoned).unwrap();

    for (id, to) in [
        (dev.id, ThesisStatus::Seed),
        (dev.id, ThesisStatus::Committed),
        (gone.id, ThesisStatus::Seed),
        (gone.id, ThesisStatus::Developing),
    ] {
        let err = thesis::move_thesis(&lib, id, to).unwrap_err();
        assert!(matches!(err, CoreError::StatusMoveRefused { .. }), "→ {to}: {err:?}");
    }
}

/// Released is reached only through the Drop, which a later ticket adds, so
/// the manual-move rule for Released and later states is checked directly.
#[test]
fn the_manual_moves_are_exactly_the_four_the_founder_makes_by_hand() {
    use ThesisStatus::*;
    let allowed: Vec<(ThesisStatus, ThesisStatus)> = ThesisStatus::ALL
        .into_iter()
        .flat_map(|from| ThesisStatus::ALL.into_iter().map(move |to| (from, to)))
        .filter(|(from, to)| from.manual_move_allowed(*to))
        .collect();

    assert_eq!(
        allowed,
        vec![(Seed, Developing), (Seed, Abandoned), (Developing, Abandoned), (Released, Archived)]
    );
}

#[test]
fn each_thesis_reports_the_moves_the_founder_can_make_now() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    assert_eq!(t.manual_moves, vec![ThesisStatus::Developing, ThesisStatus::Abandoned]);

    let t = thesis::move_thesis(&lib, t.id, ThesisStatus::Developing).unwrap();
    assert_eq!(t.manual_moves, vec![ThesisStatus::Abandoned]);

    let t = thesis::move_thesis(&lib, t.id, ThesisStatus::Abandoned).unwrap();
    assert_eq!(t.manual_moves, vec![]);
}

#[test]
fn a_thesis_needs_a_title() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = thesis::create_thesis(&lib, "   ", "A statement").unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert!(titles_by_status(&lib).iter().all(|(_, titles)| titles.is_empty()));
}

#[test]
fn moving_a_thesis_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = thesis::move_thesis(&lib, 999, ThesisStatus::Developing).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Thesis", id: 999 }), "{err:?}");
}

/// The titles in each group of the Thesis list, in list order.
fn titles_by_status(lib: &arete_lib::library::Library) -> Vec<(ThesisStatus, Vec<String>)> {
    thesis::list_theses(lib)
        .unwrap()
        .into_iter()
        .map(|g| (g.status, g.theses.into_iter().map(|t| t.title).collect()))
        .collect()
}

#[test]
fn the_thesis_list_groups_theses_by_status_in_lifecycle_order() {
    let env = TestEnv::new();
    let lib = env.library();
    thesis::create_thesis(&lib, "First seed", "").unwrap();
    let dev = thesis::create_thesis(&lib, "In work", "").unwrap();
    thesis::create_thesis(&lib, "Second seed", "").unwrap();
    thesis::move_thesis(&lib, dev.id, ThesisStatus::Developing).unwrap();

    assert_eq!(
        titles_by_status(&lib),
        vec![
            (ThesisStatus::Seed, vec!["First seed".to_string(), "Second seed".to_string()]),
            (ThesisStatus::Developing, vec!["In work".to_string()]),
            (ThesisStatus::Committed, vec![]),
            (ThesisStatus::Released, vec![]),
            (ThesisStatus::Archived, vec![]),
            (ThesisStatus::Abandoned, vec![]),
        ]
    );
}
