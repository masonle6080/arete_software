//! Library core: the Documentary of a Thesis, its lifecycle, and its Clips.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::Path;

use arete_lib::clip::{self, Clip, VideoFacts, VideoProbe};
use arete_lib::documentary::{self, DocumentaryStatus};
use arete_lib::error::{CoreError, CoreResult};
use arete_lib::library::Library;
use arete_lib::thesis;
use common::TestEnv;

#[test]
fn a_new_documentary_starts_in_planning() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    let d = documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();

    assert_eq!(d.thesis_id, t.id);
    assert_eq!(d.title, "Amor fati");
    assert_eq!(d.status, DocumentaryStatus::Planning);
    assert_eq!(documentary::thesis_documentary(&lib, t.id).unwrap(), Some(d));
}

#[test]
fn a_second_documentary_on_one_thesis_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let first = documentary::create_documentary(&lib, t.id, "First").unwrap();

    let err = documentary::create_documentary(&lib, t.id, "Second").unwrap_err();

    assert!(matches!(err, CoreError::DocumentaryExists { thesis_id } if thesis_id == t.id), "{err:?}");
    assert_eq!(documentary::thesis_documentary(&lib, t.id).unwrap(), Some(first));
}

#[test]
fn a_documentary_moves_from_planning_to_released() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();
    assert_eq!(d.moves, vec![DocumentaryStatus::Filming, DocumentaryStatus::Abandoned]);

    documentary::move_documentary(&lib, d.id, DocumentaryStatus::Filming).unwrap();
    documentary::move_documentary(&lib, d.id, DocumentaryStatus::Editing).unwrap();
    let d = documentary::move_documentary(&lib, d.id, DocumentaryStatus::Released).unwrap();

    assert_eq!(d.status, DocumentaryStatus::Released);
    assert_eq!(d.moves, vec![]);
    assert_eq!(documentary::get_documentary(&lib, d.id).unwrap().status, DocumentaryStatus::Released);
}

#[test]
fn a_documentary_can_be_abandoned_before_it_is_released() {
    let env = TestEnv::new();
    let lib = env.library();
    for stops in [0, 1, 2] {
        let t = thesis::create_thesis(&lib, &format!("Thesis {stops}"), "").unwrap();
        let d = documentary::create_documentary(&lib, t.id, "Film").unwrap();
        for to in [DocumentaryStatus::Filming, DocumentaryStatus::Editing].into_iter().take(stops) {
            documentary::move_documentary(&lib, d.id, to).unwrap();
        }

        let d = documentary::move_documentary(&lib, d.id, DocumentaryStatus::Abandoned).unwrap();

        assert_eq!(d.status, DocumentaryStatus::Abandoned);
        assert_eq!(d.moves, vec![]);
    }
}

#[test]
fn documentary_statuses_cannot_be_skipped_reversed_or_left_after_release() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = documentary::create_documentary(&lib, t.id, "Amor fati").unwrap();

    let err = documentary::move_documentary(&lib, d.id, DocumentaryStatus::Editing).unwrap_err();
    assert!(
        matches!(err, CoreError::StatusMoveRefused { what: "Documentary", from: "Planning", to: "Editing" }),
        "{err:?}"
    );
    documentary::move_documentary(&lib, d.id, DocumentaryStatus::Filming).unwrap();
    let err = documentary::move_documentary(&lib, d.id, DocumentaryStatus::Planning).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Filming", to: "Planning", .. }), "{err:?}");
    documentary::move_documentary(&lib, d.id, DocumentaryStatus::Editing).unwrap();
    documentary::move_documentary(&lib, d.id, DocumentaryStatus::Released).unwrap();
    let err = documentary::move_documentary(&lib, d.id, DocumentaryStatus::Abandoned).unwrap_err();
    assert!(matches!(err, CoreError::StatusMoveRefused { from: "Released", to: "Abandoned", .. }), "{err:?}");

    assert_eq!(documentary::get_documentary(&lib, d.id).unwrap().status, DocumentaryStatus::Released);
}

#[test]
fn an_abandoned_documentary_still_blocks_a_second_one() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();
    let d = documentary::create_documentary(&lib, t.id, "First").unwrap();
    documentary::move_documentary(&lib, d.id, DocumentaryStatus::Abandoned).unwrap();

    let err = documentary::create_documentary(&lib, t.id, "Second").unwrap_err();

    assert!(matches!(err, CoreError::DocumentaryExists { .. }), "{err:?}");
}

/// A probe that reports a fixed date shot and makes no thumbnail.
struct FakeProbe(&'static str);

impl VideoProbe for FakeProbe {
    fn probe(&self, _video: &Path, _thumbnail: &Path) -> CoreResult<VideoFacts> {
        Ok(VideoFacts { duration_ms: Some(1000), date_shot: Some(self.0.into()) })
    }
}

/// Imports a Clip with distinct content and the given date shot.
fn import(env: &TestEnv, lib: &Library, name: &str, date_shot: &'static str) -> Clip {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, name.as_bytes()).unwrap();
    clip::import_clip(lib, &path, &FakeProbe(date_shot)).unwrap()
}

fn ids(clips: &[Clip]) -> Vec<i64> {
    clips.iter().map(|c| c.id).collect()
}

#[test]
fn one_clip_can_be_in_several_documentaries_and_the_pool_filters_by_documentary() {
    let env = TestEnv::new();
    let lib = env.library();
    let a = documentary::create_documentary(&lib, thesis::create_thesis(&lib, "A", "").unwrap().id, "A").unwrap();
    let b = documentary::create_documentary(&lib, thesis::create_thesis(&lib, "B", "").unwrap().id, "B").unwrap();
    let shared = import(&env, &lib, "shared.mov", "2026-03-01");
    let only_a = import(&env, &lib, "only_a.mov", "2026-05-01");
    let loose = import(&env, &lib, "loose.mov", "2026-04-01");

    documentary::add_clip(&lib, a.id, shared.id).unwrap();
    documentary::add_clip(&lib, a.id, only_a.id).unwrap();
    documentary::add_clip(&lib, b.id, shared.id).unwrap();

    // Newest date shot first, like the whole pool.
    assert_eq!(ids(&documentary::documentary_clips(&lib, a.id).unwrap()), vec![only_a.id, shared.id]);
    assert_eq!(ids(&documentary::documentary_clips(&lib, b.id).unwrap()), vec![shared.id]);
    assert_eq!(documentary::clip_documentaries(&lib, shared.id).unwrap(), vec![a.clone(), b.clone()]);
    assert_eq!(documentary::clip_documentaries(&lib, loose.id).unwrap(), vec![]);
    // The pool still holds every Clip.
    assert_eq!(clip::list_clips(&lib).unwrap().len(), 3);
}

#[test]
fn adding_a_clip_twice_keeps_one_link_and_removing_it_leaves_the_clip_in_the_pool() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = documentary::create_documentary(&lib, thesis::create_thesis(&lib, "A", "").unwrap().id, "A").unwrap();
    let c = import(&env, &lib, "a.mov", "2026-03-01");

    documentary::add_clip(&lib, d.id, c.id).unwrap();
    documentary::add_clip(&lib, d.id, c.id).unwrap();
    assert_eq!(ids(&documentary::documentary_clips(&lib, d.id).unwrap()), vec![c.id]);

    documentary::remove_clip(&lib, d.id, c.id).unwrap();

    assert_eq!(documentary::documentary_clips(&lib, d.id).unwrap(), vec![]);
    assert_eq!(ids(&clip::list_clips(&lib).unwrap()), vec![c.id]);
}

#[test]
fn adding_a_missing_clip_or_to_a_missing_documentary_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let d = documentary::create_documentary(&lib, thesis::create_thesis(&lib, "A", "").unwrap().id, "A").unwrap();
    let c = import(&env, &lib, "a.mov", "2026-03-01");

    let err = documentary::add_clip(&lib, d.id, 999).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Clip", id: 999 }), "{err:?}");
    let err = documentary::add_clip(&lib, 999, c.id).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Documentary", id: 999 }), "{err:?}");
}

#[test]
fn a_documentary_needs_a_title_and_an_existing_thesis() {
    let env = TestEnv::new();
    let lib = env.library();
    let t = thesis::create_thesis(&lib, "Amor fati", "").unwrap();

    let err = documentary::create_documentary(&lib, t.id, "  ").unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(documentary::thesis_documentary(&lib, t.id).unwrap(), None);

    let err = documentary::create_documentary(&lib, 999, "Ghost").unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Thesis", id: 999 }), "{err:?}");
}
