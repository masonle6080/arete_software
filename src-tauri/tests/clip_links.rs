//! Library core: Clip tags, notes, Passage links, and filters (ticket #25).

mod common;

use std::path::Path;

use arete_lib::clip::{self, Clip, ClipFilter, VideoFacts, VideoProbe};
use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::error::{CoreError, CoreResult};
use arete_lib::library::Library;
use common::TestEnv;

/// A probe that reports no facts and makes no thumbnail.
struct NoFacts;

impl VideoProbe for NoFacts {
    fn probe(&self, _video: &Path, _thumbnail: &Path) -> CoreResult<VideoFacts> {
        Ok(VideoFacts::default())
    }
}

fn import(env: &TestEnv, lib: &Library, name: &str) -> Clip {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, name.as_bytes()).unwrap();
    clip::import_clip(lib, &path, &NoFacts).unwrap()
}

#[test]
fn tags_can_be_added_and_removed() {
    let env = TestEnv::new();
    let lib = env.library();
    let c = import(&env, &lib, "a.mov");

    clip::add_tag(&lib, c.id, " beach ").unwrap();
    let tagged = clip::add_tag(&lib, c.id, "Dawn").unwrap();
    assert_eq!(tagged.tags, ["beach", "Dawn"]);

    let untagged = clip::remove_tag(&lib, c.id, "beach").unwrap();
    assert_eq!(untagged.tags, ["Dawn"]);
    assert_eq!(clip::get_clip(&lib, c.id).unwrap().tags, ["Dawn"]);
}

#[test]
fn a_tag_differing_only_in_case_is_the_same_tag_and_an_empty_tag_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let c = import(&env, &lib, "a.mov");

    clip::add_tag(&lib, c.id, "Beach").unwrap();
    assert_eq!(clip::add_tag(&lib, c.id, "beach").unwrap().tags, ["Beach"]);
    assert!(matches!(clip::add_tag(&lib, c.id, "  "), Err(CoreError::InvalidInput { .. })));
    assert!(clip::remove_tag(&lib, c.id, "BEACH").unwrap().tags.is_empty());
}

#[test]
fn notes_can_be_written_and_cleared() {
    let env = TestEnv::new();
    let lib = env.library();
    let c = import(&env, &lib, "a.mov");
    assert_eq!(c.notes, "");

    let noted = clip::set_notes(&lib, c.id, "Golden hour, use the second half.").unwrap();
    assert_eq!(noted.notes, "Golden hour, use the second half.");
    assert_eq!(clip::get_clip(&lib, c.id).unwrap().notes, "Golden hour, use the second half.");
    assert_eq!(clip::set_notes(&lib, c.id, "").unwrap().notes, "");
}

/// An Essay with two Passages, `p-sea` and `p-sky`.
fn essay_with_passages(lib: &Library) -> i64 {
    let essay = document::create_document(lib, DocumentKind::Essay, "Essay").unwrap();
    let md = "<!--passage:p-sea-->The sea.<!--/passage:p-sea--> <!--passage:p-sky-->The sky.<!--/passage:p-sky-->\n";
    let report = [
        PassageText { id: "p-sea".into(), text: "The sea.".into() },
        PassageText { id: "p-sky".into(), text: "The sky.".into() },
    ];
    let edit = DocumentEdit { title: "Essay".into(), markdown: md.into() };
    document::save_document_with_passages(lib, essay.id, &edit, &report).unwrap();
    essay.id
}

fn ids(clips: &[Clip]) -> Vec<i64> {
    clips.iter().map(|c| c.id).collect()
}

#[test]
fn a_clip_can_be_linked_to_several_passages_and_unlinked() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passages(&lib);
    let a = import(&env, &lib, "a.mov");
    let b = import(&env, &lib, "b.mov");

    clip::link_passage(&lib, a.id, "p-sea").unwrap();
    clip::link_passage(&lib, a.id, "p-sky").unwrap();
    clip::link_passage(&lib, a.id, "p-sky").unwrap();
    clip::link_passage(&lib, b.id, "p-sea").unwrap();

    let linked: Vec<_> = clip::clip_passages(&lib, a.id).unwrap().into_iter().map(|p| p.id).collect();
    assert_eq!(linked, ["p-sea", "p-sky"]);
    // Pool order: undated Clips newest import first.
    assert_eq!(ids(&clip::passage_clips(&lib, "p-sea").unwrap()), [b.id, a.id]);

    clip::unlink_passage(&lib, a.id, "p-sea").unwrap();
    assert_eq!(ids(&clip::passage_clips(&lib, "p-sea").unwrap()), [b.id]);
    let linked: Vec<_> = clip::clip_passages(&lib, a.id).unwrap().into_iter().map(|p| p.id).collect();
    assert_eq!(linked, ["p-sky"]);
}

#[test]
fn linking_to_an_unknown_passage_or_clip_fails() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passages(&lib);
    let a = import(&env, &lib, "a.mov");

    assert!(matches!(clip::link_passage(&lib, a.id, "p-none"), Err(CoreError::PassageNotFound { .. })));
    assert!(matches!(clip::link_passage(&lib, 99, "p-sea"), Err(CoreError::NotFound { .. })));
    assert!(matches!(clip::passage_clips(&lib, "p-none"), Err(CoreError::PassageNotFound { .. })));
}

#[test]
fn a_link_survives_when_its_passage_is_orphaned() {
    let env = TestEnv::new();
    let lib = env.library();
    let essay = essay_with_passages(&lib);
    let a = import(&env, &lib, "a.mov");
    clip::link_passage(&lib, a.id, "p-sky").unwrap();

    let only_sea = DocumentEdit { title: "Essay".into(), markdown: "<!--passage:p-sea-->The sea.<!--/passage:p-sea-->\n".into() };
    let report = [PassageText { id: "p-sea".into(), text: "The sea.".into() }];
    document::save_document_with_passages(&lib, essay, &only_sea, &report).unwrap();

    assert_eq!(ids(&clip::passage_clips(&lib, "p-sky").unwrap()), [a.id]);
    let linked = clip::clip_passages(&lib, a.id).unwrap();
    assert!(linked[0].orphaned);
}

#[test]
fn the_pool_filters_by_tag_and_by_linked_passage() {
    let env = TestEnv::new();
    let lib = env.library();
    essay_with_passages(&lib);
    let a = import(&env, &lib, "a.mov");
    let b = import(&env, &lib, "b.mov");
    let c = import(&env, &lib, "c.mov");
    clip::add_tag(&lib, a.id, "beach").unwrap();
    clip::add_tag(&lib, b.id, "Beach").unwrap();
    clip::add_tag(&lib, c.id, "city").unwrap();
    clip::link_passage(&lib, a.id, "p-sea").unwrap();
    clip::link_passage(&lib, c.id, "p-sea").unwrap();

    let by = |tag: Option<&str>, passage: Option<&str>| {
        let filter = ClipFilter { tag: tag.map(String::from), passage_id: passage.map(String::from) };
        ids(&clip::filter_clips(&lib, &filter).unwrap())
    };
    assert_eq!(by(Some("BEACH"), None), [b.id, a.id]);
    assert_eq!(by(None, Some("p-sea")), [c.id, a.id]);
    assert_eq!(by(Some("beach"), Some("p-sea")), [a.id]);
    assert_eq!(by(None, Some("p-sky")), Vec::<i64>::new());
    assert_eq!(by(None, None), [c.id, b.id, a.id]);
    assert_eq!(clip::all_tags(&lib).unwrap(), ["beach", "city"]);
}

#[test]
fn editing_an_unknown_clip_fails() {
    let env = TestEnv::new();
    let lib = env.library();
    assert!(matches!(clip::add_tag(&lib, 99, "x"), Err(CoreError::NotFound { .. })));
    assert!(matches!(clip::set_notes(&lib, 99, "x"), Err(CoreError::NotFound { .. })));
}
