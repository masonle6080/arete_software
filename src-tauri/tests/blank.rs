//! Library core: the Blank library, and Styles that use a Blank as their base.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::blank::{self, BlankInput};
use arete_lib::drop;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::style::{self, Style, StyleBase};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorId, VendorInput, VendorRole};
use common::TestEnv;

fn new_vendor(lib: &Library, name: &str) -> VendorId {
    vendor::create_vendor(lib, VendorInput { name: name.into(), roles: vec![VendorRole::BlankSupplier], ..Default::default() })
        .unwrap()
        .id
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn tee_input(vendor_id: VendorId) -> BlankInput {
    BlankInput {
        brand: "Los Angeles Apparel".into(),
        model: "1801GD".into(),
        vendor_id,
        colors: strings(&["Bone", "Black"]),
        sizes: strings(&["S", "M", "L", "XL"]),
        cost_cents: Some(1250),
        currency: "USD".into(),
        notes: "6.5 oz garment dyed".into(),
    }
}

fn new_style(lib: &Library) -> Style {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap()
}

#[test]
fn founder_can_create_a_blank_and_find_it_in_the_list() {
    let env = TestEnv::new();
    let lib = env.library();
    let v = new_vendor(&lib, "LA Apparel");

    let b = blank::create_blank(&lib, tee_input(v)).unwrap();

    assert_eq!((b.brand.as_str(), b.model.as_str()), ("Los Angeles Apparel", "1801GD"));
    assert_eq!(b.vendor_id, v);
    assert_eq!(b.vendor_name, "LA Apparel");
    assert_eq!(b.colors, strings(&["Bone", "Black"]));
    assert_eq!(b.sizes, strings(&["S", "M", "L", "XL"]));
    assert_eq!(b.cost_cents, Some(1250));
    assert_eq!(b.currency, "USD");
    assert_eq!(b.notes, "6.5 oz garment dyed");
    assert_eq!(blank::get_blank(&lib, b.id).unwrap(), b);
    assert_eq!(blank::list_blanks(&lib).unwrap(), vec![b]);
}

#[test]
fn founder_can_edit_a_blank() {
    let env = TestEnv::new();
    let lib = env.library();
    let v = new_vendor(&lib, "LA Apparel");
    let other = new_vendor(&lib, "Shaka Wear");
    let b = blank::create_blank(&lib, tee_input(v)).unwrap();

    let edited = blank::update_blank(
        &lib,
        b.id,
        BlankInput {
            brand: "Shaka Wear".into(),
            model: "SHMHSS".into(),
            vendor_id: other,
            colors: strings(&["White"]),
            sizes: strings(&["S", "M", "L", "XL", "2XL"]),
            cost_cents: None,
            currency: String::new(),
            notes: String::new(),
        },
    )
    .unwrap();

    assert_eq!((edited.brand.as_str(), edited.model.as_str(), edited.vendor_id), ("Shaka Wear", "SHMHSS", other));
    assert_eq!(edited.colors, strings(&["White"]));
    assert_eq!(edited.sizes, strings(&["S", "M", "L", "XL", "2XL"]));
    assert_eq!(edited.cost_cents, None);
    assert_eq!(blank::get_blank(&lib, b.id).unwrap(), edited);
}

#[test]
fn blank_lists_are_trimmed_and_hold_each_item_once() {
    let env = TestEnv::new();
    let lib = env.library();
    let v = new_vendor(&lib, "LA Apparel");

    let b = blank::create_blank(
        &lib,
        BlankInput { colors: strings(&[" Bone ", "Bone", ""]), sizes: strings(&["M", " M", "L", " "]), ..tee_input(v) },
    )
    .unwrap();

    assert_eq!(b.colors, strings(&["Bone"]));
    assert_eq!(b.sizes, strings(&["M", "L"]));
}

#[test]
fn a_blank_needs_a_brand_a_model_a_known_vendor_and_a_size() {
    let env = TestEnv::new();
    let lib = env.library();
    let v = new_vendor(&lib, "LA Apparel");

    for bad in [
        BlankInput { brand: "  ".into(), ..tee_input(v) },
        BlankInput { model: "".into(), ..tee_input(v) },
        BlankInput { sizes: vec![], ..tee_input(v) },
        BlankInput { cost_cents: Some(-1), ..tee_input(v) },
    ] {
        let err = blank::create_blank(&lib, bad).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    }
    let err = blank::create_blank(&lib, tee_input(9999)).unwrap_err();
    assert!(matches!(err, CoreError::NotFound { record: "Vendor", id: 9999 }), "{err:?}");
    assert!(blank::list_blanks(&lib).unwrap().is_empty());
}

#[test]
fn a_new_style_has_no_base_and_no_sizes() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);

    assert_eq!(style::style_base(&lib, s.id).unwrap(), None);
    assert!(style::style_sizes(&lib, s.id).unwrap().is_empty());
}

#[test]
fn founder_can_set_a_styles_base_to_a_blank() {
    let env = TestEnv::new();
    let lib = env.library();
    let b = blank::create_blank(&lib, tee_input(new_vendor(&lib, "LA Apparel"))).unwrap();
    let s = new_style(&lib);

    let base = style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: b.id }).unwrap();

    assert_eq!(base, StyleBase::Blank { blank_id: b.id });
    assert_eq!(style::style_base(&lib, s.id).unwrap(), Some(base));
}

#[test]
fn a_style_cannot_use_a_blank_that_does_not_exist() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);

    let err = style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: 42 }).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Blank", id: 42 }), "{err:?}");
    assert_eq!(style::style_base(&lib, s.id).unwrap(), None);
}

#[test]
fn a_blank_based_style_picks_its_sizes_from_the_blanks_sizes() {
    let env = TestEnv::new();
    let lib = env.library();
    let b = blank::create_blank(&lib, tee_input(new_vendor(&lib, "LA Apparel"))).unwrap();
    let s = new_style(&lib);
    style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: b.id }).unwrap();

    let sizes = style::set_style_sizes(&lib, s.id, &strings(&["XL", "M", "L"])).unwrap();

    assert_eq!(sizes, strings(&["M", "L", "XL"]), "sizes come back in the Blank's order");
    assert_eq!(style::style_sizes(&lib, s.id).unwrap(), sizes);
}

#[test]
fn a_size_the_blank_does_not_offer_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let b = blank::create_blank(&lib, tee_input(new_vendor(&lib, "LA Apparel"))).unwrap();
    let s = new_style(&lib);
    style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: b.id }).unwrap();
    style::set_style_sizes(&lib, s.id, &strings(&["M"])).unwrap();

    let err = style::set_style_sizes(&lib, s.id, &strings(&["M", "3XL"])).unwrap_err();

    assert!(matches!(&err, CoreError::InvalidInput { reason } if reason.contains("3XL")), "{err:?}");
    assert_eq!(style::style_sizes(&lib, s.id).unwrap(), strings(&["M"]));
}

#[test]
fn a_style_without_a_base_cannot_have_sizes() {
    let env = TestEnv::new();
    let lib = env.library();
    let s = new_style(&lib);

    let err = style::set_style_sizes(&lib, s.id, &strings(&["M"])).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert!(style::style_sizes(&lib, s.id).unwrap().is_empty());
}

#[test]
fn moving_a_style_to_another_blank_keeps_only_the_sizes_it_offers() {
    let env = TestEnv::new();
    let lib = env.library();
    let v = new_vendor(&lib, "LA Apparel");
    let tee = blank::create_blank(&lib, tee_input(v)).unwrap();
    let crop =
        blank::create_blank(&lib, BlankInput { model: "Crop".into(), sizes: strings(&["XS", "S", "M"]), ..tee_input(v) })
            .unwrap();
    let s = new_style(&lib);
    style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: tee.id }).unwrap();
    style::set_style_sizes(&lib, s.id, &strings(&["S", "M", "L"])).unwrap();

    style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: crop.id }).unwrap();

    assert_eq!(style::style_base(&lib, s.id).unwrap(), Some(StyleBase::Blank { blank_id: crop.id }));
    assert_eq!(style::style_sizes(&lib, s.id).unwrap(), strings(&["S", "M"]));
}

#[test]
fn a_blank_edit_cannot_remove_a_size_a_style_uses() {
    let env = TestEnv::new();
    let lib = env.library();
    let v = new_vendor(&lib, "LA Apparel");
    let b = blank::create_blank(&lib, tee_input(v)).unwrap();
    let s = new_style(&lib);
    style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: b.id }).unwrap();
    style::set_style_sizes(&lib, s.id, &strings(&["M", "XL"])).unwrap();

    let err =
        blank::update_blank(&lib, b.id, BlankInput { sizes: strings(&["S", "M", "L"]), ..tee_input(v) }).unwrap_err();
    assert!(
        matches!(&err, CoreError::InvalidInput { reason } if reason.contains("XL") && reason.contains("Heavy tee")),
        "{err:?}"
    );
    assert_eq!(blank::get_blank(&lib, b.id).unwrap().sizes, strings(&["S", "M", "L", "XL"]));

    let ok = blank::update_blank(&lib, b.id, BlankInput { sizes: strings(&["M", "XL"]), ..tee_input(v) }).unwrap();
    assert_eq!(ok.sizes, strings(&["M", "XL"]));
    assert_eq!(style::style_sizes(&lib, s.id).unwrap(), strings(&["M", "XL"]));
}

#[test]
fn one_blank_can_be_the_base_of_styles_on_different_drops() {
    let env = TestEnv::new();
    let lib = env.library();
    let b = blank::create_blank(&lib, tee_input(new_vendor(&lib, "LA Apparel"))).unwrap();
    let first = new_style(&lib);
    let t = thesis::create_thesis(&lib, "Memento mori", "").unwrap();
    let d = drop::create_drop(&lib, t.id, "Memento mori FW27").unwrap();
    let second = style::create_style(&lib, d.id, "Heavy tee", "Black").unwrap();

    for s in [&first, &second] {
        style::set_style_base(&lib, s.id, StyleBase::Blank { blank_id: b.id }).unwrap();
    }
    style::set_style_sizes(&lib, first.id, &strings(&["S"])).unwrap();
    style::set_style_sizes(&lib, second.id, &strings(&["L", "XL"])).unwrap();

    assert_eq!(style::style_sizes(&lib, first.id).unwrap(), strings(&["S"]));
    assert_eq!(style::style_sizes(&lib, second.id).unwrap(), strings(&["L", "XL"]));
}
