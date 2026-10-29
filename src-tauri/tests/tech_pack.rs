//! Library core: a Style built on a Tech Pack, its Point of Measure table,
//! its fabric and other details, and the Style sizes it gives.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use std::path::PathBuf;

use arete_lib::blank::{self, BlankInput};
use arete_lib::drop;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::media;
use arete_lib::style::{self, Style, StyleBase};
use arete_lib::tech_pack::{self, Fabric, TechPack, TechPackDetails};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorInput, VendorRole};
use common::TestEnv;

/// A new Style on a new Drop.
fn new_style(lib: &Library) -> Style {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy hoodie", "Black").unwrap()
}

/// Sets a Style's base to a Tech Pack and returns the Tech Pack.
fn use_tech_pack(lib: &Library, style_id: style::StyleId) -> TechPack {
    let base = style::set_style_base(lib, style_id, StyleBase::TechPack).unwrap();
    assert_eq!(base, StyleBase::TechPack);
    tech_pack::get_tech_pack(lib, style_id).unwrap().expect("the base is a Tech Pack")
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn size_names(lib: &Library, style_id: style::StyleId) -> Vec<String> {
    style::style_sizes(lib, style_id).unwrap()
}

/// A Blank in sizes M, L, XL.
fn new_blank(lib: &Library) -> blank::Blank {
    let v = vendor::create_vendor(
        lib,
        VendorInput { name: "LA Apparel".into(), roles: vec![VendorRole::BlankSupplier], ..Default::default() },
    )
    .unwrap();
    blank::create_blank(
        lib,
        BlankInput {
            brand: "Los Angeles Apparel".into(),
            model: "HF-09".into(),
            vendor_id: v.id,
            colors: strings(&["Black"]),
            sizes: strings(&["M", "L", "XL"]),
            cost_cents: None,
            currency: String::new(),
            notes: String::new(),
        },
    )
    .unwrap()
}

#[test]
fn a_style_has_no_tech_pack_until_its_base_is_set_to_one() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), None);

    let tp = use_tech_pack(&lib, s.id);

    assert_eq!(style::style_base(&lib, s.id).unwrap(), Some(StyleBase::TechPack));
    assert_eq!(tp.style_id, s.id);
    assert!(tp.sizes.is_empty());
    assert!(tp.points_of_measure.is_empty());
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), Some(tp));
}

#[test]
fn setting_the_base_to_a_tech_pack_again_keeps_the_tech_pack() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    use_tech_pack(&lib, s.id);
    let tp = tech_pack::add_size(&lib, s.id, "M").unwrap();

    assert_eq!(use_tech_pack(&lib, s.id), tp);
    let err = style::set_style_base(&lib, 999, StyleBase::TechPack).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Style", id: 999 }), "{err:?}");
}

#[test]
fn a_style_base_is_either_a_blank_or_a_tech_pack_never_both() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    let b = new_blank(&lib);
    use_tech_pack(&lib, s.id);
    for size in ["S", "M", "L"] {
        tech_pack::add_size(&lib, s.id, size).unwrap();
    }

    // To a Blank: the Tech Pack is no longer the base, and the Style keeps
    // only the sizes the Blank also offers.
    style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: b.id }).unwrap();
    assert_eq!(style::style_base(&lib, s.id).unwrap(), Some(StyleBase::Blank { blank_id: b.id }));
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), None);
    assert_eq!(size_names(&lib, s.id), ["M", "L"]);
    let err = tech_pack::add_size(&lib, s.id, "XL").unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");

    // Back to the Tech Pack: its table is still there and gives the sizes.
    let tp = use_tech_pack(&lib, s.id);
    assert_eq!(tp.sizes.iter().map(|z| z.name.as_str()).collect::<Vec<_>>(), ["S", "M", "L"]);
    assert_eq!(size_names(&lib, s.id), ["S", "M", "L"]);
}

#[test]
fn a_tech_pack_style_cannot_pick_sizes_outside_its_table() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    use_tech_pack(&lib, s.id);
    tech_pack::add_size(&lib, s.id, "M").unwrap();

    let err = style::set_style_sizes(&lib, s.id, &strings(&["M"])).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(size_names(&lib, s.id), ["M"]);
}

#[test]
fn a_tech_pack_style_takes_its_sizes_from_the_point_of_measure_table() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    use_tech_pack(&lib, s.id);

    tech_pack::add_size(&lib, s.id, "S").unwrap();
    tech_pack::add_size(&lib, s.id, " M ").unwrap();
    let tp = tech_pack::add_size(&lib, s.id, "L").unwrap();

    let names: Vec<&str> = tp.sizes.iter().map(|z| z.name.as_str()).collect();
    assert_eq!(names, ["S", "M", "L"]);
    assert_eq!(size_names(&lib, s.id), ["S", "M", "L"]);

    let m = tp.sizes[1].id;
    let tp = tech_pack::rename_size(&lib, m, "Medium").unwrap();
    assert_eq!(tp.sizes[1].id, m);
    let tp = tech_pack::remove_size(&lib, tp.sizes[0].id).unwrap();
    assert_eq!(tp.sizes.len(), 2);
    assert_eq!(size_names(&lib, s.id), ["Medium", "L"]);
}

#[test]
fn a_style_without_a_base_has_no_sizes() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);

    assert!(size_names(&lib, s.id).is_empty());
}

#[test]
fn a_size_name_must_be_unique_and_not_blank() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    use_tech_pack(&lib, s.id);
    let tp = tech_pack::add_size(&lib, s.id, "S").unwrap();
    tech_pack::add_size(&lib, s.id, "M").unwrap();

    for bad in ["", "  ", "S", " M "] {
        let err = tech_pack::add_size(&lib, s.id, bad).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{bad:?}: {err:?}");
    }
    let err = tech_pack::rename_size(&lib, tp.sizes[0].id, "M").unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(size_names(&lib, s.id), ["S", "M"]);
}

#[test]
fn sizes_need_a_tech_pack_base_first() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);

    let err = tech_pack::add_size(&lib, s.id, "S").unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), None);
}

/// A Tech Pack Style with sizes S, M, L.
fn sized_style(lib: &Library) -> (Style, TechPack) {
    let s = new_style(lib);
    use_tech_pack(lib, s.id);
    for size in ["S", "M", "L"] {
        tech_pack::add_size(lib, s.id, size).unwrap();
    }
    let tp = tech_pack::get_tech_pack(lib, s.id).unwrap().unwrap();
    (s, tp)
}

#[test]
fn a_point_of_measure_has_a_value_per_size_and_a_tolerance() {
    let env = TestEnv::new();
    let lib = env.library();
    let (s, tp) = sized_style(&lib);
    let (small, large) = (tp.sizes[0].id, tp.sizes[2].id);

    let tp = tech_pack::add_point_of_measure(&lib, s.id, " Chest width ", 1.0).unwrap();
    let chest = tp.points_of_measure[0].id;
    tech_pack::set_point_of_measure_value(&lib, chest, small, Some(54.0)).unwrap();
    let tp = tech_pack::set_point_of_measure_value(&lib, chest, large, Some(60.5)).unwrap();

    let pom = &tp.points_of_measure[0];
    assert_eq!(pom.name, "Chest width");
    assert_eq!(pom.tolerance, 1.0);
    // One value per size, in the order of `sizes`; M is still empty.
    assert_eq!(pom.values, vec![Some(54.0), None, Some(60.5)]);
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), Some(tp));
}

#[test]
fn points_of_measure_keep_their_ids_and_order_through_edits() {
    let env = TestEnv::new();
    let lib = env.library();
    let (s, tp) = sized_style(&lib);
    let small = tp.sizes[0].id;
    tech_pack::add_point_of_measure(&lib, s.id, "Chest width", 1.0).unwrap();
    let tp = tech_pack::add_point_of_measure(&lib, s.id, "Body length", 1.5).unwrap();
    let (chest, length) = (tp.points_of_measure[0].id, tp.points_of_measure[1].id);
    tech_pack::set_point_of_measure_value(&lib, chest, small, Some(54.0)).unwrap();

    let tp = tech_pack::update_point_of_measure(&lib, chest, "Chest width (1 cm below armhole)", 0.5).unwrap();

    let ids: Vec<i64> = tp.points_of_measure.iter().map(|p| p.id).collect();
    assert_eq!(ids, [chest, length]);
    assert_eq!(tp.points_of_measure[0].name, "Chest width (1 cm below armhole)");
    assert_eq!(tp.points_of_measure[0].tolerance, 0.5);
    assert_eq!(tp.points_of_measure[0].values[0], Some(54.0));

    let tp = tech_pack::set_point_of_measure_value(&lib, chest, small, None).unwrap();
    assert_eq!(tp.points_of_measure[0].values, vec![None, None, None]);

    let tp = tech_pack::remove_point_of_measure(&lib, chest).unwrap();
    let ids: Vec<i64> = tp.points_of_measure.iter().map(|p| p.id).collect();
    assert_eq!(ids, [length]);
}

#[test]
fn removing_a_size_removes_its_values() {
    let env = TestEnv::new();
    let lib = env.library();
    let (s, tp) = sized_style(&lib);
    let chest = tech_pack::add_point_of_measure(&lib, s.id, "Chest width", 1.0).unwrap().points_of_measure[0].id;
    for (size, value) in tp.sizes.iter().zip([54.0, 57.0, 60.0]) {
        tech_pack::set_point_of_measure_value(&lib, chest, size.id, Some(value)).unwrap();
    }

    let tp = tech_pack::remove_size(&lib, tp.sizes[1].id).unwrap();

    assert_eq!(tp.points_of_measure[0].values, vec![Some(54.0), Some(60.0)]);
    assert_eq!(size_names(&lib, s.id), ["S", "L"]);
}

#[test]
fn a_point_of_measure_needs_a_name_and_a_tolerance_of_zero_or_more() {
    let env = TestEnv::new();
    let lib = env.library();
    let (s, _) = sized_style(&lib);

    for (name, tolerance) in [("", 1.0), ("  ", 1.0), ("Chest width", -0.5), ("Chest width", f64::NAN)] {
        let err = tech_pack::add_point_of_measure(&lib, s.id, name, tolerance).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{name:?} {tolerance}: {err:?}");
    }
    let tp = tech_pack::add_point_of_measure(&lib, s.id, "Chest width", 0.0).unwrap();
    let err = tech_pack::update_point_of_measure(&lib, tp.points_of_measure[0].id, "", 1.0).unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
}

#[test]
fn a_value_must_be_a_number_in_a_size_of_the_same_tech_pack() {
    let env = TestEnv::new();
    let lib = env.library();
    let (s, _) = sized_style(&lib);
    let (other, other_tp) = sized_style(&lib);
    let chest = tech_pack::add_point_of_measure(&lib, s.id, "Chest width", 1.0).unwrap().points_of_measure[0].id;

    let err = tech_pack::set_point_of_measure_value(&lib, chest, other_tp.sizes[0].id, Some(54.0)).unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    let own_size = tech_pack::get_tech_pack(&lib, s.id).unwrap().unwrap().sizes[0].id;
    let err = tech_pack::set_point_of_measure_value(&lib, chest, own_size, Some(f64::INFINITY)).unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    let err = tech_pack::set_point_of_measure_value(&lib, 999, own_size, Some(54.0)).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Point of Measure", id: 999 }), "{err:?}");

    let tp = tech_pack::get_tech_pack(&lib, s.id).unwrap().unwrap();
    assert_eq!(tp.points_of_measure[0].values, vec![None, None, None]);
    assert!(tech_pack::get_tech_pack(&lib, other.id).unwrap().unwrap().points_of_measure.is_empty());
}

#[test]
fn a_tech_pack_records_fabric_trims_construction_and_label_placement() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    let mill = vendor::create_vendor(
        &lib,
        VendorInput { name: "Shibaura Mill".into(), roles: vec![VendorRole::Mill], ..Default::default() },
    )
    .unwrap();
    let tp = use_tech_pack(&lib, s.id);
    assert_eq!(tp.details.unit, "cm");
    assert_eq!(tp.details.fabric, Fabric::default());

    let details = TechPackDetails {
        unit: "in".into(),
        fabric: Fabric {
            composition: "100% cotton loopback".into(),
            weight: "450 gsm".into(),
            color: "Black".into(),
            vendor_id: Some(mill.id),
        },
        trims: "YKK #5 metal zip, flat drawcord".into(),
        construction_notes: "Coverstitch all seams. Double-needle hem.".into(),
        label_placement: "Woven label centre back neck, 2 cm below seam".into(),
    };
    let tp = tech_pack::update_tech_pack_details(&lib, s.id, details.clone()).unwrap();

    assert_eq!(tp.details, details);
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), Some(tp));
}

#[test]
fn the_fabric_vendor_must_exist_and_the_unit_must_not_be_blank() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    let tp = use_tech_pack(&lib, s.id);

    let with_vendor = TechPackDetails { fabric: Fabric { vendor_id: Some(999), ..Fabric::default() }, ..tp.details.clone() };
    let err = tech_pack::update_tech_pack_details(&lib, s.id, with_vendor).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Vendor", id: 999 }), "{err:?}");

    let blank_unit = TechPackDetails { unit: " ".into(), ..tp.details.clone() };
    let err = tech_pack::update_tech_pack_details(&lib, s.id, blank_unit).unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");

    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), Some(tp));
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn flats_attach_to_a_tech_pack_and_are_stored_in_the_library() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);
    use_tech_pack(&lib, s.id);
    let front = source_file(&env, "front flat.ai", b"front");
    let back = source_file(&env, "back flat.pdf", b"back");

    tech_pack::attach_flats(&lib, s.id, &[front.clone()]).unwrap();
    let tp = tech_pack::attach_flats(&lib, s.id, &[back]).unwrap();

    let names: Vec<&str> = tp.flats.iter().map(|f| f.file_name.as_str()).collect();
    assert_eq!(names, ["front flat.ai", "back flat.pdf"]);
    std::fs::remove_file(&front).unwrap();
    assert_eq!(media::read_media(&lib, &tp.flats[0].media).unwrap(), b"front");
    assert_eq!(tp.flats[0].path, media::media_path(&lib, &tp.flats[0].media));
    assert_eq!(tech_pack::get_tech_pack(&lib, s.id).unwrap(), Some(tp));
}
