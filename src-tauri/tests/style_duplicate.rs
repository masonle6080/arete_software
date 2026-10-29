//! Library core: duplicating a Style in another color (spec #1, ticket #22).
//!
//! The copy gets the base, sizes, Decorations, Design Decisions with their
//! Passage links, Tech Pack, Moodboard, and design files. It does not get
//! Samples, and it starts as Draft. These tests call the domain API directly
//! against a fresh temporary library, and check results only through it.

mod common;

use std::path::PathBuf;

use arete_lib::blank::{self, BlankInput};
use arete_lib::decoration::{self, DecorationInput};
use arete_lib::design_decision::{self, DesignAspect};
use arete_lib::document::{self, DocumentEdit, DocumentKind, PassageText};
use arete_lib::drop;
use arete_lib::moodboard::{self, MoodboardOwner};
use arete_lib::sample;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::style::{self, StyleBase, StyleId, StyleStatus};
use arete_lib::tech_pack::{self, Fabric, TechPackDetails};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorId, VendorInput, VendorRole};
use common::TestEnv;

fn a_style(lib: &Library) -> StyleId {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap().id
}

fn a_vendor(lib: &Library, name: &str, role: VendorRole) -> VendorId {
    vendor::create_vendor(lib, VendorInput { name: name.into(), roles: vec![role], ..Default::default() }).unwrap().id
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn a_blank(lib: &Library) -> i64 {
    let vendor_id = a_vendor(lib, "LAA", VendorRole::BlankSupplier);
    blank::create_blank(
        lib,
        BlankInput {
            brand: "Los Angeles Apparel".into(),
            model: "1801GD".into(),
            vendor_id,
            colors: strings(&["Bone", "Black"]),
            sizes: strings(&["S", "M", "L", "XL"]),
            cost_cents: Some(1250),
            currency: "USD".into(),
            notes: String::new(),
        },
    )
    .unwrap()
    .id
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn a_duplicate_is_a_new_draft_style_in_the_new_color_on_the_same_drop() {
    let env = TestEnv::new();
    let lib = env.library();
    let original = style::get_style(&lib, a_style(&lib)).unwrap();
    style::move_style(&lib, original.id, StyleStatus::Sampling).unwrap();

    let copy = style::duplicate_style(&lib, original.id, "  Black ").unwrap();

    assert_ne!(copy.id, original.id);
    assert_eq!(copy.drop_id, original.drop_id);
    assert_eq!(copy.name, "Heavy tee");
    assert_eq!(copy.color, "Black");
    assert_eq!(copy.status, StyleStatus::Draft);
    assert_eq!(style::get_style(&lib, original.id).unwrap().status, StyleStatus::Sampling);
    let ids: Vec<StyleId> = style::drop_styles(&lib, original.drop_id).unwrap().iter().map(|s| s.id).collect();
    assert_eq!(ids, vec![original.id, copy.id]);
}

#[test]
fn a_duplicate_needs_a_color_and_an_existing_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);

    assert!(matches!(style::duplicate_style(&lib, id, "  "), Err(CoreError::InvalidInput { .. })));
    assert!(matches!(style::duplicate_style(&lib, 999, "Black"), Err(CoreError::NotFound { .. })));
    assert_eq!(style::drop_styles(&lib, style::get_style(&lib, id).unwrap().drop_id).unwrap().len(), 1);
}

#[test]
fn a_duplicate_keeps_the_blank_base_and_chosen_sizes() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    let blank_id = a_blank(&lib);
    style::set_style_base(&lib, id, StyleBase::Blank { blank_id }).unwrap();
    style::set_style_sizes(&lib, id, &strings(&["M", "L"])).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert_eq!(style::style_base(&lib, copy.id).unwrap(), Some(StyleBase::Blank { blank_id }));
    assert_eq!(style::style_sizes(&lib, copy.id).unwrap(), strings(&["M", "L"]));

    style::set_style_sizes(&lib, copy.id, &strings(&["S"])).unwrap();
    assert_eq!(style::style_sizes(&lib, id).unwrap(), strings(&["M", "L"]));
}

#[test]
fn a_duplicate_of_a_style_with_no_base_has_no_base() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert_eq!(style::style_base(&lib, copy.id).unwrap(), None);
    assert!(style::style_sizes(&lib, copy.id).unwrap().is_empty());
}

#[test]
fn a_duplicate_has_the_same_design_files() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    let sketch = source_file(&env, "flat.ai", b"vector");
    style::attach_style_files(&lib, id, &[sketch]).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    let files = style::style_files(&lib, copy.id).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name, "flat.ai");
    assert_eq!(std::fs::read(&files[0].path).unwrap(), b"vector");
    let more = source_file(&env, "back.ai", b"other");
    style::attach_style_files(&lib, copy.id, &[more]).unwrap();
    assert_eq!(style::style_files(&lib, id).unwrap().len(), 1);
}

#[test]
fn a_duplicate_gets_its_own_copy_of_the_tech_pack() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    let mill = a_vendor(&lib, "Mill", VendorRole::Mill);
    style::set_style_base(&lib, id, StyleBase::TechPack).unwrap();
    tech_pack::add_size(&lib, id, "M").unwrap();
    tech_pack::add_size(&lib, id, "L").unwrap();
    let tp = tech_pack::add_point_of_measure(&lib, id, "Chest width", 1.0).unwrap();
    let chest = tp.points_of_measure[0].id;
    tech_pack::set_point_of_measure_value(&lib, chest, tp.sizes[0].id, Some(56.0)).unwrap();
    tech_pack::set_point_of_measure_value(&lib, chest, tp.sizes[1].id, Some(59.5)).unwrap();
    let details = TechPackDetails {
        unit: "cm".into(),
        fabric: Fabric {
            composition: "100% cotton".into(),
            weight: "450 gsm".into(),
            color: "Bone".into(),
            vendor_id: Some(mill),
        },
        trims: "Woven label".into(),
        construction_notes: "Flatlock seams".into(),
        label_placement: "Back neck".into(),
    };
    tech_pack::update_tech_pack_details(&lib, id, details).unwrap();
    tech_pack::attach_flats(&lib, id, &[source_file(&env, "front.pdf", b"flat")]).unwrap();
    let original = tech_pack::get_tech_pack(&lib, id).unwrap().unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert_eq!(style::style_base(&lib, copy.id).unwrap(), Some(StyleBase::TechPack));
    assert_eq!(style::style_sizes(&lib, copy.id).unwrap(), strings(&["M", "L"]));
    let copied = tech_pack::get_tech_pack(&lib, copy.id).unwrap().unwrap();
    assert_eq!(copied.style_id, copy.id);
    assert_eq!(copied.details, original.details);
    let names: Vec<&str> = copied.sizes.iter().map(|z| z.name.as_str()).collect();
    assert_eq!(names, ["M", "L"]);
    assert!(copied.sizes.iter().all(|z| original.sizes.iter().all(|o| o.id != z.id)));
    assert_eq!(copied.points_of_measure.len(), 1);
    let pom = &copied.points_of_measure[0];
    assert_ne!(pom.id, chest);
    assert_eq!((pom.name.as_str(), pom.tolerance), ("Chest width", 1.0));
    assert_eq!(pom.values, vec![Some(56.0), Some(59.5)]);
    assert_eq!(copied.flats.len(), 1);
    assert_eq!(copied.flats[0].file_name, "front.pdf");

    // Edits to the copy's table leave the original alone.
    tech_pack::set_point_of_measure_value(&lib, pom.id, copied.sizes[0].id, Some(60.0)).unwrap();
    tech_pack::rename_size(&lib, copied.sizes[1].id, "XL").unwrap();
    tech_pack::add_point_of_measure(&lib, copy.id, "Body length", 1.0).unwrap();
    assert_eq!(tech_pack::get_tech_pack(&lib, id).unwrap().unwrap(), original);
}

#[test]
fn a_duplicate_of_a_blank_style_keeps_its_stored_tech_pack_for_later() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    style::set_style_base(&lib, id, StyleBase::TechPack).unwrap();
    tech_pack::add_size(&lib, id, "M").unwrap();
    let blank_id = a_blank(&lib);
    style::set_style_base(&lib, id, StyleBase::Blank { blank_id }).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert_eq!(style::style_base(&lib, copy.id).unwrap(), Some(StyleBase::Blank { blank_id }));
    assert_eq!(tech_pack::get_tech_pack(&lib, copy.id).unwrap(), None);
    style::set_style_base(&lib, copy.id, StyleBase::TechPack).unwrap();
    assert_eq!(style::style_sizes(&lib, copy.id).unwrap(), strings(&["M"]));
}

fn screen_print(vendor_id: Option<VendorId>) -> DecorationInput {
    DecorationInput {
        technique: "Screen print".into(),
        placement: "Center chest".into(),
        size: "10 x 12 cm".into(),
        colors: strings(&["Black", "Oxblood"]),
        vendor_id,
    }
}

#[test]
fn a_duplicate_gets_its_own_copy_of_each_decoration() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    let printer = a_vendor(&lib, "Printer", VendorRole::Decorator);
    let print = decoration::add_decoration(&lib, id, screen_print(Some(printer))).unwrap();
    decoration::set_decoration_artwork(&lib, print.id, &source_file(&env, "art.png", b"art")).unwrap();
    decoration::add_decoration(&lib, id, DecorationInput { technique: "Woven label".into(), ..Default::default() })
        .unwrap();
    let originals = decoration::style_decorations(&lib, id).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    let copied = decoration::style_decorations(&lib, copy.id).unwrap();
    assert_eq!(copied.len(), 2);
    for (c, o) in copied.iter().zip(&originals) {
        assert_ne!(c.id, o.id);
        assert_eq!(c.style_id, copy.id);
        assert_eq!(
            (&c.technique, &c.placement, &c.size, &c.colors, &c.artwork, c.vendor_id),
            (&o.technique, &o.placement, &o.size, &o.colors, &o.artwork, o.vendor_id)
        );
    }
    assert_eq!(copied[0].artwork.as_ref().unwrap().file_name, "art.png");

    let recolored = DecorationInput { colors: strings(&["White"]), ..screen_print(Some(printer)) };
    decoration::update_decoration(&lib, copied[0].id, recolored).unwrap();
    decoration::remove_decoration(&lib, copied[1].id).unwrap();
    assert_eq!(decoration::style_decorations(&lib, id).unwrap(), originals);
}

/// An Essay with one Passage per (id, text).
fn an_essay_with_passages(lib: &Library, passages: &[(&str, &str)]) {
    let essay = document::create_document(lib, DocumentKind::Essay, "On courage").unwrap();
    let md: String = passages.iter().map(|(id, text)| format!("<!--passage:{id}-->{text}<!--/passage:{id}-->\n\n")).collect();
    let report: Vec<PassageText> =
        passages.iter().map(|(id, text)| PassageText { id: (*id).into(), text: (*text).into() }).collect();
    let edit = DocumentEdit { title: "On courage".into(), markdown: md };
    document::save_document_with_passages(lib, essay.id, &edit, &report).unwrap();
}

#[test]
fn a_duplicate_gets_the_design_decisions_with_their_passages_and_decorations() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    an_essay_with_passages(&lib, &[("p-one", "weight"), ("p-two", "color")]);
    let print = decoration::add_decoration(&lib, id, screen_print(None)).unwrap();
    let why_print = design_decision::add_design_decision(&lib, id, DesignAspect::Decoration, "Loud.").unwrap();
    design_decision::set_decoration(&lib, why_print.id, Some(print.id)).unwrap();
    design_decision::link_passage(&lib, why_print.id, "p-two").unwrap();
    design_decision::link_passage(&lib, why_print.id, "p-one").unwrap();
    design_decision::add_design_decision(&lib, id, DesignAspect::Fabric, "Heavy.").unwrap();
    let originals = design_decision::style_design_decisions(&lib, id).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    let copied = design_decision::style_design_decisions(&lib, copy.id).unwrap();
    let copied_print = decoration::style_decorations(&lib, copy.id).unwrap()[0].id;
    assert_eq!(copied.len(), 2);
    assert_ne!(copied[0].id, why_print.id);
    assert_eq!(copied[0].style_id, copy.id);
    assert_eq!((copied[0].aspect, copied[0].rationale.as_str()), (DesignAspect::Decoration, "Loud."));
    assert_eq!(copied[0].decoration_id, Some(copied_print));
    let linked: Vec<&str> = copied[0].passages.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(linked, ["p-two", "p-one"]);
    assert_eq!((copied[1].aspect, copied[1].decoration_id), (DesignAspect::Fabric, None));
    assert!(copied[1].passages.is_empty());

    // A Passage now lists the Design Decision on both Styles.
    let on_passage: Vec<StyleId> =
        design_decision::passage_design_decisions(&lib, "p-one").unwrap().iter().map(|d| d.style_id).collect();
    assert_eq!(on_passage, vec![id, copy.id]);

    design_decision::unlink_passage(&lib, copied[0].id, "p-two").unwrap();
    design_decision::set_decoration(&lib, copied[0].id, None).unwrap();
    assert_eq!(design_decision::style_design_decisions(&lib, id).unwrap(), originals);
}

const LINKED_SWATCH: &str = r##"{"id":"sw-1","type":"rectangle","backgroundColor":"#e8e0d0","isDeleted":false,"customData":{"passageId":"p-one"}}"##;

#[test]
fn a_duplicate_gets_its_own_copy_of_the_moodboard_with_its_passage_links() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    an_essay_with_passages(&lib, &[("p-one", "weight")]);
    let board = moodboard::create_moodboard(&lib, MoodboardOwner::Style(id), "Bone refs").unwrap();
    let scene = format!(r#"{{"type":"excalidraw","version":2,"elements":[{LINKED_SWATCH}]}}"#);
    moodboard::save_moodboard_scene(&lib, board.id, &scene).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    let boards = moodboard::list_moodboards(&lib, MoodboardOwner::Style(copy.id)).unwrap();
    assert_eq!(boards.len(), 1);
    assert_ne!(boards[0].id, board.id);
    assert_eq!(boards[0].title, "Bone refs");
    assert_eq!(moodboard::load_moodboard_scene(&lib, boards[0].id).unwrap().as_deref(), Some(scene.as_str()));
    let items = moodboard::moodboard_items_of_passage(&lib, "p-one").unwrap();
    let on: Vec<i64> = items.iter().map(|i| i.moodboard.id).collect();
    assert_eq!(on, vec![board.id, boards[0].id]);
    assert_eq!(items[1].color.as_deref(), Some("#e8e0d0"));

    moodboard::save_moodboard_scene(&lib, boards[0].id, r#"{"elements":[]}"#).unwrap();
    assert_eq!(moodboard::load_moodboard_scene(&lib, board.id).unwrap().as_deref(), Some(scene.as_str()));
    assert_eq!(moodboard::moodboard_items_of_passage(&lib, "p-one").unwrap().len(), 1);
}

#[test]
fn a_duplicate_has_no_moodboard_or_an_unsaved_one_when_the_original_does() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    moodboard::create_moodboard(&lib, MoodboardOwner::Style(id), "Never saved").unwrap();
    let other = style::duplicate_style(&lib, a_style(&lib), "Black").unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert!(moodboard::list_moodboards(&lib, MoodboardOwner::Style(other.id)).unwrap().is_empty());
    let boards = moodboard::list_moodboards(&lib, MoodboardOwner::Style(copy.id)).unwrap();
    assert_eq!(boards.len(), 1);
    assert_eq!(moodboard::load_moodboard_scene(&lib, boards[0].id).unwrap(), None);
}

#[test]
fn a_duplicate_does_not_get_the_samples() {
    let env = TestEnv::new();
    let lib = env.library();
    let id = a_style(&lib);
    let maker = a_vendor(&lib, "Maker", VendorRole::Manufacturer);
    sample::request_sample(&lib, id, maker).unwrap();

    let copy = style::duplicate_style(&lib, id, "Black").unwrap();

    assert!(sample::style_samples(&lib, copy.id).unwrap().is_empty());
    assert_eq!(sample::style_samples(&lib, id).unwrap().len(), 1);
}
