//! Library core: Decorations on a Style, and the link from a Design Decision
//! to one Decoration on the same Style (spec #1, ticket #20).
//!
//! These tests call the domain API directly against a fresh temporary
//! library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::decoration::{self, DecorationInput};
use arete_lib::design_decision::{self, DesignAspect};
use arete_lib::drop;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::media;
use arete_lib::style::{self, StyleId};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorId, VendorInput, VendorRole};
use common::TestEnv;

/// A Thesis with a Drop, and one Style on it. Returns the Drop's ID too.
fn a_drop_with_style(lib: &Library) -> (i64, StyleId) {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    (d.id, style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap().id)
}

fn a_style(lib: &Library) -> StyleId {
    a_drop_with_style(lib).1
}

fn a_vendor(lib: &Library, name: &str) -> VendorId {
    vendor::create_vendor(lib, VendorInput { name: name.into(), roles: vec![VendorRole::Decorator], ..Default::default() })
        .unwrap()
        .id
}

fn screen_print(vendor_id: Option<VendorId>) -> DecorationInput {
    DecorationInput {
        technique: "  Screen print ".into(),
        placement: "Center chest".into(),
        size: "10 x 12 cm".into(),
        colors: vec!["Black".into(), " Oxblood ".into()],
        vendor_id,
    }
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn a_decoration_records_its_specification_on_the_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let vendor_id = a_vendor(&lib, "Ink House");

    let added = decoration::add_decoration(&lib, style_id, screen_print(Some(vendor_id))).unwrap();

    assert_eq!(added.style_id, style_id);
    assert_eq!(added.technique, "Screen print");
    assert_eq!(added.placement, "Center chest");
    assert_eq!(added.size, "10 x 12 cm");
    assert_eq!(added.colors, ["Black", "Oxblood"]);
    assert_eq!(added.vendor_id, Some(vendor_id));
    assert_eq!(added.vendor_name.as_deref(), Some("Ink House"));
    assert_eq!(added.artwork, None);
    assert_eq!(decoration::style_decorations(&lib, style_id).unwrap(), vec![added.clone()]);
    assert_eq!(decoration::get_decoration(&lib, added.id).unwrap(), added);
}

#[test]
fn a_decoration_needs_a_technique_a_style_and_an_existing_vendor() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);

    let blank = DecorationInput { technique: "  ".into(), ..Default::default() };
    let err = decoration::add_decoration(&lib, style_id, blank).unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    let err = decoration::add_decoration(&lib, 999, screen_print(None)).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", id: 999 }), "{err:?}");
    let err = decoration::add_decoration(&lib, style_id, screen_print(Some(77))).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Vendor", id: 77 }), "{err:?}");
    assert!(decoration::style_decorations(&lib, style_id).unwrap().is_empty());
}

#[test]
fn editing_a_decoration_replaces_its_fields() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let first = a_vendor(&lib, "Ink House");
    let second = a_vendor(&lib, "Thread Co");
    let added = decoration::add_decoration(&lib, style_id, screen_print(Some(first))).unwrap();

    let edit = DecorationInput {
        technique: "Embroidery".into(),
        placement: "Left sleeve".into(),
        size: "4 cm".into(),
        colors: vec!["Gold".into()],
        vendor_id: Some(second),
    };
    let edited = decoration::update_decoration(&lib, added.id, edit).unwrap();

    assert_eq!(edited.id, added.id);
    assert_eq!(
        (edited.technique.as_str(), edited.placement.as_str(), edited.size.as_str()),
        ("Embroidery", "Left sleeve", "4 cm")
    );
    assert_eq!(edited.colors, ["Gold"]);
    assert_eq!(edited.vendor_name.as_deref(), Some("Thread Co"));
    assert_eq!(decoration::style_decorations(&lib, style_id).unwrap(), vec![edited.clone()]);

    let no_vendor = DecorationInput { technique: "Embroidery".into(), ..Default::default() };
    let cleared = decoration::update_decoration(&lib, added.id, no_vendor).unwrap();
    assert_eq!((cleared.vendor_id, cleared.vendor_name, cleared.colors.len()), (None, None, 0));
}

#[test]
fn a_removed_decoration_is_gone_from_its_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let keep = decoration::add_decoration(&lib, style_id, screen_print(None)).unwrap();
    let gone = decoration::add_decoration(&lib, style_id, screen_print(None)).unwrap();

    decoration::remove_decoration(&lib, gone.id).unwrap();

    assert_eq!(decoration::style_decorations(&lib, style_id).unwrap(), vec![keep]);
    let err = decoration::get_decoration(&lib, gone.id).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Decoration", .. }), "{err:?}");
    let err = decoration::remove_decoration(&lib, gone.id).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Decoration", .. }), "{err:?}");
}

#[test]
fn artwork_imports_into_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let added = decoration::add_decoration(&lib, a_style(&lib), screen_print(None)).unwrap();
    let source = source_file(&env, "sun.ai", b"artwork bytes");

    let with_art = decoration::set_decoration_artwork(&lib, added.id, &source).unwrap();

    let art = with_art.artwork.clone().unwrap();
    assert_eq!(art.file_name, "sun.ai");
    assert!(art.path.starts_with(lib.root()));
    std::fs::remove_file(&source).unwrap();
    assert_eq!(media::read_media(&lib, &art.media).unwrap(), b"artwork bytes");
    assert_eq!(decoration::get_decoration(&lib, added.id).unwrap(), with_art);

    let without = decoration::clear_decoration_artwork(&lib, added.id).unwrap();
    assert_eq!(without.artwork, None);
}

#[test]
fn a_design_decision_links_to_one_decoration_on_the_same_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let print = decoration::add_decoration(&lib, style_id, screen_print(None)).unwrap();
    let label = decoration::add_decoration(&lib, style_id, DecorationInput { technique: "Woven label".into(), ..Default::default() }).unwrap();
    let decision =
        design_decision::add_design_decision(&lib, style_id, DesignAspect::Decoration, "The sun, from the Essay.").unwrap();
    assert_eq!(decision.decoration_id, None);

    let linked = design_decision::set_decoration(&lib, decision.id, Some(print.id)).unwrap();
    assert_eq!(linked.decoration_id, Some(print.id));

    let relinked = design_decision::set_decoration(&lib, decision.id, Some(label.id)).unwrap();
    assert_eq!(relinked.decoration_id, Some(label.id));
    assert_eq!(design_decision::style_design_decisions(&lib, style_id).unwrap(), vec![relinked]);

    let unlinked = design_decision::set_decoration(&lib, decision.id, None).unwrap();
    assert_eq!(unlinked.decoration_id, None);
}

#[test]
fn a_decoration_on_another_style_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let (drop_id, style_id) = a_drop_with_style(&lib);
    let other_style = style::create_style(&lib, drop_id, "Heavy tee", "Black").unwrap().id;
    let other_print = decoration::add_decoration(&lib, other_style, screen_print(None)).unwrap();
    let own_print = decoration::add_decoration(&lib, style_id, screen_print(None)).unwrap();
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Decoration, "Why").unwrap();
    design_decision::set_decoration(&lib, decision.id, Some(own_print.id)).unwrap();

    let err = design_decision::set_decoration(&lib, decision.id, Some(other_print.id)).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(design_decision::get_design_decision(&lib, decision.id).unwrap().decoration_id, Some(own_print.id));
}

#[test]
fn linking_a_missing_decoration_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Decoration, "Why").unwrap();

    let err = design_decision::set_decoration(&lib, decision.id, Some(404)).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Decoration", id: 404 }), "{err:?}");
    assert_eq!(design_decision::get_design_decision(&lib, decision.id).unwrap().decoration_id, None);
}

#[test]
fn removing_a_linked_decoration_keeps_the_design_decision_without_a_link() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let print = decoration::add_decoration(&lib, style_id, screen_print(None)).unwrap();
    let decision = design_decision::add_design_decision(&lib, style_id, DesignAspect::Decoration, "Why").unwrap();
    design_decision::set_decoration(&lib, decision.id, Some(print.id)).unwrap();

    decoration::remove_decoration(&lib, print.id).unwrap();

    let kept = design_decision::get_design_decision(&lib, decision.id).unwrap();
    assert_eq!((kept.decoration_id, kept.rationale.as_str()), (None, "Why"));
}
