//! Library core: fit notes on Samples of Tech Pack Styles. The founder
//! records the measured value of each Point of Measure, per size, and sees
//! it beside the Tech Pack value and tolerance.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::blank::{self, BlankInput};
use arete_lib::drop;
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::sample::{self, FitCell, Sample};
use arete_lib::style::{self, StyleBase, StyleId};
use arete_lib::tech_pack::{self, PointOfMeasureId, SizeId};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorInput, VendorRole};
use common::TestEnv;

/// A Tech Pack Style with sizes M and L and one Point of Measure, chest
/// width (M 56, L 58, tolerance 1), and a Sample of it.
struct Fixture {
    style_id: StyleId,
    sample: Sample,
    chest: PointOfMeasureId,
    m: SizeId,
    l: SizeId,
}

fn a_style(lib: &Library) -> StyleId {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy hoodie", "Black").unwrap().id
}

fn a_vendor(lib: &Library) -> vendor::VendorId {
    vendor::create_vendor(
        lib,
        VendorInput { name: "Cut Co".into(), roles: vec![VendorRole::Manufacturer], ..Default::default() },
    )
    .unwrap()
    .id
}

fn fixture(lib: &Library) -> Fixture {
    let style_id = a_style(lib);
    style::set_style_base(lib, style_id, StyleBase::TechPack).unwrap();
    tech_pack::add_size(lib, style_id, "M").unwrap();
    let tp = tech_pack::add_size(lib, style_id, "L").unwrap();
    let (m, l) = (tp.sizes[0].id, tp.sizes[1].id);
    let tp = tech_pack::add_point_of_measure(lib, style_id, "Chest width", 1.0).unwrap();
    let chest = tp.points_of_measure[0].id;
    tech_pack::set_point_of_measure_value(lib, chest, m, Some(56.0)).unwrap();
    tech_pack::set_point_of_measure_value(lib, chest, l, Some(58.0)).unwrap();
    let sample = sample::request_sample(lib, style_id, a_vendor(lib)).unwrap();
    Fixture { style_id, sample, chest, m, l }
}

fn cell(lib: &Library, sample: &Sample, row: usize, col: usize) -> FitCell {
    let fit = sample::sample_fit(lib, sample.id).unwrap().expect("a Tech Pack Style offers fit notes");
    fit.rows[row].cells[col].clone()
}

#[test]
fn a_new_sample_shows_the_tech_pack_table_with_nothing_measured() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);

    let fit = sample::sample_fit(&lib, f.sample.id).unwrap().unwrap();

    assert_eq!(fit.sample_id, f.sample.id);
    assert_eq!(fit.unit, "cm");
    assert_eq!(fit.sizes.iter().map(|z| z.name.as_str()).collect::<Vec<_>>(), vec!["M", "L"]);
    assert_eq!(fit.rows.len(), 1);
    let row = &fit.rows[0];
    assert_eq!((row.point_of_measure_id, row.name.as_str(), row.tolerance), (f.chest, "Chest width", 1.0));
    assert_eq!(
        row.cells,
        vec![
            FitCell { size_id: f.m, spec: Some(56.0), measured: None, deviation: None, within_tolerance: None },
            FitCell { size_id: f.l, spec: Some(58.0), measured: None, deviation: None, within_tolerance: None },
        ]
    );
}

#[test]
fn a_measured_value_shows_its_deviation_from_the_tech_pack_and_the_tolerance() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);

    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(57.0)).unwrap();
    let fit = sample::set_fit_note(&lib, f.sample.id, f.chest, f.l, Some(60.5)).unwrap();

    assert_eq!(
        fit.rows[0].cells,
        vec![
            FitCell { size_id: f.m, spec: Some(56.0), measured: Some(57.0), deviation: Some(1.0), within_tolerance: Some(true) },
            FitCell { size_id: f.l, spec: Some(58.0), measured: Some(60.5), deviation: Some(2.5), within_tolerance: Some(false) },
        ]
    );
    assert_eq!(sample::sample_fit(&lib, f.sample.id).unwrap(), Some(fit));
}

#[test]
fn a_measured_value_can_be_changed_and_cleared() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(57.0)).unwrap();

    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(55.5)).unwrap();
    let changed = cell(&lib, &f.sample, 0, 0);
    assert_eq!((changed.measured, changed.deviation, changed.within_tolerance), (Some(55.5), Some(-0.5), Some(true)));

    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, None).unwrap();
    let cleared = cell(&lib, &f.sample, 0, 0);
    assert_eq!((cleared.measured, cleared.deviation, cleared.within_tolerance), (None, None, None));
}

#[test]
fn a_measured_value_without_a_tech_pack_value_has_no_deviation() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    tech_pack::set_point_of_measure_value(&lib, f.chest, f.l, None).unwrap();

    sample::set_fit_note(&lib, f.sample.id, f.chest, f.l, Some(59.0)).unwrap();

    let c = cell(&lib, &f.sample, 0, 1);
    assert_eq!((c.spec, c.measured, c.deviation, c.within_tolerance), (None, Some(59.0), None, None));
}

#[test]
fn each_sample_round_has_its_own_fit_notes() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(57.5)).unwrap();

    sample::decide_sample(&lib, f.sample.id, sample::SampleOutcome::Revised).unwrap();
    let round_two = sample::style_samples(&lib, f.style_id).unwrap().pop().unwrap();

    assert_eq!(round_two.round, 2);
    assert_eq!(cell(&lib, &round_two, 0, 0).measured, None);
    assert_eq!(cell(&lib, &f.sample, 0, 0).measured, Some(57.5));
}

#[test]
fn a_fit_note_needs_a_point_of_measure_and_size_of_the_samples_own_tech_pack() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    let other = fixture(&lib);

    let wrong_point_of_measure = sample::set_fit_note(&lib, f.sample.id, other.chest, f.m, Some(1.0));
    let wrong_size = sample::set_fit_note(&lib, f.sample.id, f.chest, other.m, Some(1.0));
    let not_a_number = sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(f64::NAN));
    let no_sample = sample::set_fit_note(&lib, 9999, f.chest, f.m, Some(1.0));

    assert!(matches!(wrong_point_of_measure, Err(CoreError::InvalidInput { .. })), "{wrong_point_of_measure:?}");
    assert!(matches!(wrong_size, Err(CoreError::InvalidInput { .. })), "{wrong_size:?}");
    assert!(matches!(not_a_number, Err(CoreError::InvalidInput { .. })), "{not_a_number:?}");
    assert!(matches!(no_sample, Err(CoreError::NotFound { record: "Sample", .. })), "{no_sample:?}");
    assert_eq!(cell(&lib, &f.sample, 0, 0).measured, None);
}

/// A Blank in size M from a new Vendor.
fn a_blank(lib: &Library) -> blank::BlankId {
    blank::create_blank(
        lib,
        BlankInput {
            brand: "Los Angeles Apparel".into(),
            model: "HF-09".into(),
            vendor_id: a_vendor(lib),
            colors: vec!["Black".into()],
            sizes: vec!["M".into()],
            cost_cents: None,
            currency: String::new(),
            notes: String::new(),
        },
    )
    .unwrap()
    .id
}

#[test]
fn fit_notes_are_not_offered_on_a_blank_based_style() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let blank_id = a_blank(&lib);
    style::set_style_base(&lib, style_id, StyleBase::Blank { blank_id }).unwrap();
    let s = sample::request_sample(&lib, style_id, a_vendor(&lib)).unwrap();

    assert_eq!(sample::sample_fit(&lib, s.id).unwrap(), None);

    let no_base = a_style(&lib);
    let s2 = sample::request_sample(&lib, no_base, a_vendor(&lib)).unwrap();
    assert_eq!(sample::sample_fit(&lib, s2.id).unwrap(), None);
}

#[test]
fn fit_notes_are_kept_while_the_style_is_on_a_blank_and_come_back_with_the_tech_pack() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(57.0)).unwrap();
    let blank_id = a_blank(&lib);

    style::set_style_base(&lib, f.style_id, StyleBase::Blank { blank_id }).unwrap();
    assert_eq!(sample::sample_fit(&lib, f.sample.id).unwrap(), None);
    let refused = sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(58.0));
    assert!(matches!(refused, Err(CoreError::InvalidInput { .. })), "{refused:?}");

    style::set_style_base(&lib, f.style_id, StyleBase::TechPack).unwrap();
    assert_eq!(cell(&lib, &f.sample, 0, 0).measured, Some(57.0));
}

#[test]
fn a_point_of_measure_with_fit_notes_cannot_be_removed_until_they_are_cleared() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, Some(57.0)).unwrap();

    let refused = tech_pack::remove_point_of_measure(&lib, f.chest);

    match refused {
        Err(CoreError::InvalidInput { reason }) => assert!(reason.contains("round 1"), "{reason}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
    let tp = tech_pack::get_tech_pack(&lib, f.style_id).unwrap().unwrap();
    assert_eq!(tp.points_of_measure.len(), 1);
    assert_eq!(cell(&lib, &f.sample, 0, 0).measured, Some(57.0));

    sample::set_fit_note(&lib, f.sample.id, f.chest, f.m, None).unwrap();
    let tp = tech_pack::remove_point_of_measure(&lib, f.chest).unwrap();
    assert!(tp.points_of_measure.is_empty());
    assert!(sample::sample_fit(&lib, f.sample.id).unwrap().unwrap().rows.is_empty());
}

#[test]
fn a_size_with_fit_notes_cannot_be_removed_until_they_are_cleared() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = fixture(&lib);
    sample::set_fit_note(&lib, f.sample.id, f.chest, f.l, Some(58.0)).unwrap();

    let refused = tech_pack::remove_size(&lib, f.l);

    match refused {
        Err(CoreError::InvalidInput { reason }) => assert!(reason.contains("round 1"), "{reason}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(style::style_sizes(&lib, f.style_id).unwrap(), vec!["M".to_string(), "L".to_string()]);
    assert_eq!(cell(&lib, &f.sample, 0, 1).measured, Some(58.0));

    // A size with no fit notes is removed as before.
    let tp = tech_pack::remove_size(&lib, f.m).unwrap();
    assert_eq!(tp.sizes.len(), 1);

    sample::set_fit_note(&lib, f.sample.id, f.chest, f.l, None).unwrap();
    let tp = tech_pack::remove_size(&lib, f.l).unwrap();
    assert!(tp.sizes.is_empty());
}
