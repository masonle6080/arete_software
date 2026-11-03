//! Library core: Thesis capture, its one lifecycle (the idea and its Drop),
//! the guards on its moves, and the grouped list.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::documentary::{self, DocumentaryStatus};
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::style::{self, StyleStatus};
use arete_lib::thesis::{self, Thesis, ThesisMove, ThesisStatus, ThesisWarning};
use common::{move_to, TestEnv};

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
fn a_thesis_needs_a_title() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = thesis::create_thesis(&lib, "   ", "A statement").unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert!(titles_by_status(&lib).iter().all(|(_, titles)| titles.is_empty()));
}

#[test]
fn a_thesis_moves_through_every_status_in_order() {
    use ThesisStatus::*;
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    assert_eq!(t.manual_moves, vec![Developing, Abandoned]);

    let expected_moves = [
        (Developing, vec![Design, Abandoned]),
        (Design, vec![Sampling, Abandoned]),
        (Sampling, vec![Production, Abandoned]),
        (Production, vec![Released, Abandoned]),
        (Released, vec![Archived]),
        (Archived, vec![]),
    ];
    for (to, moves) in expected_moves {
        let moved = move_to(&lib, t.id, to);
        assert_eq!(moved.status, to);
        assert_eq!(moved.manual_moves, moves, "moves from {to}");
        assert_eq!(thesis::get_thesis(&lib, t.id).unwrap(), moved);
    }
}

#[test]
fn the_lifecycle_allows_exactly_the_forward_steps_and_abandoning_before_release() {
    use ThesisStatus::*;
    let allowed: Vec<(ThesisStatus, ThesisStatus)> = ThesisStatus::ALL
        .into_iter()
        .flat_map(|from| ThesisStatus::ALL.into_iter().map(move |to| (from, to)))
        .filter(|(from, to)| from.move_allowed(*to))
        .collect();

    assert_eq!(
        allowed,
        vec![
            (Seed, Developing),
            (Seed, Abandoned),
            (Developing, Design),
            (Developing, Abandoned),
            (Design, Sampling),
            (Design, Abandoned),
            (Sampling, Production),
            (Sampling, Abandoned),
            (Production, Released),
            (Production, Abandoned),
            (Released, Archived),
        ]
    );
}

#[test]
fn moves_the_lifecycle_does_not_allow_are_refused_and_change_nothing() {
    use ThesisStatus::*;
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    for to in [Seed, Design, Sampling, Production, Released, Archived] {
        let err = thesis::move_thesis(&lib, t.id, to, true).unwrap_err();
        assert!(matches!(err, CoreError::StatusMoveRefused { what: "Thesis", from: "Seed", .. }), "Seed → {to}: {err:?}");
    }
    move_to(&lib, t.id, Developing);
    move_to(&lib, t.id, Design);
    for to in [Seed, Developing, Production, Released] {
        let err = thesis::move_thesis(&lib, t.id, to, true).unwrap_err();
        assert!(matches!(err, CoreError::StatusMoveRefused { from: "Design", .. }), "Design → {to}: {err:?}");
    }

    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, Design);
}

#[test]
fn a_thesis_can_be_abandoned_from_any_status_before_released() {
    use ThesisStatus::*;
    let env = TestEnv::new();
    let lib = env.library();

    for (i, at) in [Seed, Developing, Design, Sampling, Production].into_iter().enumerate() {
        let t = thesis::create_thesis(&lib, &format!("Idea {i}"), "").unwrap();
        walk_to(&lib, t.id, at);

        let gone = move_to(&lib, t.id, Abandoned);

        assert_eq!(gone.status, Abandoned, "from {at}");
        assert_eq!(gone.manual_moves, vec![], "from {at}");
    }
}

#[test]
fn a_released_or_archived_thesis_cannot_be_abandoned() {
    use ThesisStatus::*;
    let env = TestEnv::new();
    let lib = env.library();

    for at in [Released, Archived] {
        let t = thesis::create_thesis(&lib, at.label(), "").unwrap();
        walk_to(&lib, t.id, at);

        let err = thesis::move_thesis(&lib, t.id, Abandoned, true).unwrap_err();

        assert!(matches!(err, CoreError::StatusMoveRefused { to: "Abandoned", .. }), "from {at}: {err:?}");
        assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, at);
    }
}

#[test]
fn moving_a_thesis_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = thesis::move_thesis(&lib, 999, ThesisStatus::Developing, false).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Thesis", id: 999 }), "{err:?}");
}

#[test]
fn production_is_refused_while_a_style_is_not_approved_or_cut_and_names_those_styles() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = common::design_thesis(&lib, "Amor fati");
    let tee = style::create_style(&lib, t.id, "Heavy tee", "Bone").unwrap();
    let hoodie = style::create_style(&lib, t.id, "Hoodie", "Black").unwrap();
    let cap = style::create_style(&lib, t.id, "Cap", "Ink").unwrap();
    approve(&lib, tee.id);
    style::move_style(&lib, cap.id, StyleStatus::Cut).unwrap();
    style::move_style(&lib, hoodie.id, StyleStatus::Sampling).unwrap();
    move_to(&lib, t.id, ThesisStatus::Sampling);

    // Confirming does not go past a hard rule.
    let err = thesis::move_thesis(&lib, t.id, ThesisStatus::Production, true).unwrap_err();

    assert!(
        matches!(&err, CoreError::StylesNotReady { styles } if styles == &vec!["Hoodie (Black)".to_string()]),
        "{err:?}"
    );
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Sampling);

    style::move_style(&lib, hoodie.id, StyleStatus::Approved).unwrap();
    assert_eq!(move_to(&lib, t.id, ThesisStatus::Production).status, ThesisStatus::Production);
}

#[test]
fn releasing_before_the_documentary_is_released_warns_and_changes_nothing() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis_in_production(&lib);
    let film = documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();
    documentary::move_documentary(&lib, film.id, DocumentaryStatus::Filming).unwrap();

    let result = thesis::move_thesis(&lib, t.id, ThesisStatus::Released, false).unwrap();

    let ThesisMove::NeedsConfirmation { thesis: unchanged, warnings } = result else {
        panic!("expected a warning: {result:?}")
    };
    assert_eq!(unchanged.status, ThesisStatus::Production);
    assert_eq!(
        warnings,
        vec![ThesisWarning::DocumentaryNotReleased {
            documentary_id: film.id,
            title: "Amor fati".into(),
            status: DocumentaryStatus::Filming,
        }]
    );
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Production);
}

#[test]
fn a_confirmed_release_goes_past_the_documentary_warning() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis_in_production(&lib);
    documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();

    let result = thesis::move_thesis(&lib, t.id, ThesisStatus::Released, true).unwrap();

    let ThesisMove::Moved(released) = result else { panic!("expected the move: {result:?}") };
    assert_eq!(released.status, ThesisStatus::Released);
    assert_eq!(thesis::get_thesis(&lib, t.id).unwrap().status, ThesisStatus::Released);
}

#[test]
fn releasing_after_the_documentary_is_released_needs_no_confirmation() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis_in_production(&lib);
    let film = documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();
    for to in [DocumentaryStatus::Filming, DocumentaryStatus::Editing, DocumentaryStatus::Released] {
        documentary::move_documentary(&lib, film.id, to).unwrap();
    }

    assert_eq!(move_to(&lib, t.id, ThesisStatus::Released).status, ThesisStatus::Released);
}

#[test]
fn the_documentary_warning_applies_only_to_release() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis_in_production(&lib);
    documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();

    assert_eq!(move_to(&lib, t.id, ThesisStatus::Abandoned).status, ThesisStatus::Abandoned);
}

#[test]
fn only_developing_design_and_sampling_take_new_styles() {
    use ThesisStatus::*;
    let takes: Vec<ThesisStatus> = ThesisStatus::ALL.into_iter().filter(|s| s.takes_new_styles()).collect();
    assert_eq!(takes, vec![Developing, Design, Sampling]);

    let env = TestEnv::new();
    let lib = env.library();
    for (i, at) in ThesisStatus::ALL.into_iter().enumerate() {
        let t = thesis::create_thesis(&lib, &format!("Idea {i}"), "").unwrap();
        walk_to(&lib, t.id, at);
        let t = thesis::get_thesis(&lib, t.id).unwrap();
        assert_eq!(t.can_add_styles, at.takes_new_styles(), "{at}");

        let added = style::create_style(&lib, t.id, "Heavy tee", "Bone");

        if at.takes_new_styles() {
            assert!(added.is_ok(), "{at}: {added:?}");
        } else {
            assert!(matches!(added, Err(CoreError::InvalidInput { .. })), "{at}: {added:?}");
            assert_eq!(style::thesis_styles(&lib, t.id).unwrap(), vec![], "{at}");
        }
    }
}

#[test]
fn styles_stay_on_their_thesis_after_it_stops_taking_new_ones() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis_in_production(&lib);

    assert_eq!(style::thesis_styles(&lib, t.id).unwrap().len(), 1);
    let err = style::create_style(&lib, t.id, "Late tee", "Bone").unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
}

/// The titles in each group of the Thesis list, in list order.
fn titles_by_status(lib: &Library) -> Vec<(ThesisStatus, Vec<String>)> {
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
    let design = thesis::create_thesis(&lib, "On paper", "").unwrap();
    let gone = thesis::create_thesis(&lib, "Dead end", "").unwrap();
    walk_to(&lib, dev.id, ThesisStatus::Developing);
    walk_to(&lib, design.id, ThesisStatus::Design);
    move_to(&lib, gone.id, ThesisStatus::Abandoned);

    let s = |v: &[&str]| v.iter().map(|t| t.to_string()).collect::<Vec<_>>();
    assert_eq!(
        titles_by_status(&lib),
        vec![
            (ThesisStatus::Seed, s(&["First seed", "Second seed"])),
            (ThesisStatus::Developing, s(&["In work"])),
            (ThesisStatus::Design, s(&["On paper"])),
            (ThesisStatus::Sampling, vec![]),
            (ThesisStatus::Production, vec![]),
            (ThesisStatus::Released, vec![]),
            (ThesisStatus::Archived, vec![]),
            (ThesisStatus::Abandoned, s(&["Dead end"])),
        ]
    );
}

/// Moves Thesis `id` forward from Seed, one step at a time, until it is
/// `at`. Abandoned is reached from Seed.
fn walk_to(lib: &Library, id: i64, at: ThesisStatus) {
    use ThesisStatus::*;
    if at == Abandoned {
        move_to(lib, id, Abandoned);
        return;
    }
    for step in [Developing, Design, Sampling, Production, Released, Archived] {
        if thesis::get_thesis(lib, id).unwrap().status == at {
            return;
        }
        move_to(lib, id, step);
    }
}

fn approve(lib: &Library, style_id: i64) {
    style::move_style(lib, style_id, StyleStatus::Sampling).unwrap();
    style::move_style(lib, style_id, StyleStatus::Approved).unwrap();
}

/// A fresh Thesis with one Approved Style, moved to Production.
fn thesis_in_production(lib: &Library) -> Thesis {
    let t = common::design_thesis(lib, "Amor fati");
    let tee = style::create_style(lib, t.id, "Heavy tee", "Bone").unwrap();
    approve(lib, tee.id);
    move_to(lib, t.id, ThesisStatus::Sampling);
    move_to(lib, t.id, ThesisStatus::Production)
}
