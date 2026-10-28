//! Library core: Vendor records (basic). Name and one or more roles.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::error::CoreError;
use arete_lib::vendor::{self, VendorInput, VendorRole};
use common::TestEnv;

fn input(name: &str, roles: &[VendorRole]) -> VendorInput {
    VendorInput { name: name.to_string(), roles: roles.to_vec() }
}

#[test]
fn founder_can_create_a_vendor_and_find_it_in_the_list() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = vendor::create_vendor(&lib, input("Los Angeles Apparel", &[VendorRole::BlankSupplier])).unwrap();

    assert_eq!(created.name, "Los Angeles Apparel");
    assert_eq!(created.roles, vec![VendorRole::BlankSupplier]);
    assert_eq!(vendor::list_vendors(&lib).unwrap(), vec![created.clone()]);
    assert_eq!(vendor::get_vendor(&lib, created.id).unwrap(), created);
}

#[test]
fn a_vendor_can_hold_several_roles() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = vendor::create_vendor(
        &lib,
        input("Full-service factory", &[VendorRole::Mill, VendorRole::Manufacturer, VendorRole::Decorator]),
    )
    .unwrap();

    let read = vendor::get_vendor(&lib, created.id).unwrap();
    assert_eq!(read.roles, vec![VendorRole::Decorator, VendorRole::Manufacturer, VendorRole::Mill]);
}

#[test]
fn a_role_given_twice_is_held_once() {
    let env = TestEnv::new();
    let lib = env.library();

    let created =
        vendor::create_vendor(&lib, input("Printer", &[VendorRole::Decorator, VendorRole::Decorator])).unwrap();

    assert_eq!(vendor::get_vendor(&lib, created.id).unwrap().roles, vec![VendorRole::Decorator]);
}

#[test]
fn a_vendor_without_a_name_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = vendor::create_vendor(&lib, input("   ", &[VendorRole::Mill])).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(vendor::list_vendors(&lib).unwrap(), vec![]);
}

#[test]
fn a_vendor_without_a_role_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = vendor::create_vendor(&lib, input("Mystery Co", &[])).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(vendor::list_vendors(&lib).unwrap(), vec![]);
}

#[test]
fn a_vendor_name_is_saved_without_surrounding_spaces() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = vendor::create_vendor(&lib, input("  Shaka Wear ", &[VendorRole::BlankSupplier])).unwrap();

    assert_eq!(vendor::get_vendor(&lib, created.id).unwrap().name, "Shaka Wear");
}

#[test]
fn founder_can_rename_a_vendor_and_change_its_roles() {
    let env = TestEnv::new();
    let lib = env.library();
    let created = vendor::create_vendor(&lib, input("Old Name", &[VendorRole::BlankSupplier, VendorRole::Mill])).unwrap();

    let edited =
        vendor::update_vendor(&lib, created.id, input("New Name", &[VendorRole::Mill, VendorRole::Decorator])).unwrap();

    let read = vendor::get_vendor(&lib, created.id).unwrap();
    assert_eq!(read, edited);
    assert_eq!(read.name, "New Name");
    assert_eq!(read.roles, vec![VendorRole::Decorator, VendorRole::Mill]);
}

#[test]
fn an_edit_that_removes_every_role_is_refused_and_changes_nothing() {
    let env = TestEnv::new();
    let lib = env.library();
    let created = vendor::create_vendor(&lib, input("Mill Co", &[VendorRole::Mill])).unwrap();

    let err = vendor::update_vendor(&lib, created.id, input("Mill Co", &[])).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(vendor::get_vendor(&lib, created.id).unwrap(), created);
}

#[test]
fn editing_a_vendor_that_does_not_exist_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = vendor::update_vendor(&lib, 42, input("Ghost", &[VendorRole::Mill])).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { .. }), "{err:?}");
    assert_eq!(vendor::list_vendors(&lib).unwrap(), vec![]);
}

#[test]
fn vendors_are_listed_by_name() {
    let env = TestEnv::new();
    let lib = env.library();
    for name in ["mill b", "Decorator A", "Cut and Sew"] {
        vendor::create_vendor(&lib, input(name, &[VendorRole::Manufacturer])).unwrap();
    }

    let names: Vec<String> = vendor::list_vendors(&lib).unwrap().into_iter().map(|v| v.name).collect();

    assert_eq!(names, ["Cut and Sew", "Decorator A", "mill b"]);
}

#[test]
fn vendors_are_kept_when_the_library_is_reopened() {
    let env = TestEnv::new();
    let lib = env.library();
    let created = vendor::create_vendor(&lib, input("Durable", &[VendorRole::Decorator])).unwrap();
    let root = lib.root().to_path_buf();
    drop(lib);

    let reopened = arete_lib::library::open_library(&env.config, &root).unwrap();

    assert_eq!(vendor::list_vendors(&reopened).unwrap(), vec![created]);
}
