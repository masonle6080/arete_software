//! Library core: Vendor records (basic). Name and one or more roles.
//!
//! These tests call the domain API directly (no IPC) against a fresh
//! temporary library, and check results only through the same API.

mod common;

use arete_lib::error::CoreError;
use arete_lib::vendor::{self, VendorContact, VendorInput, VendorRole};
use common::TestEnv;

fn input(name: &str, roles: &[VendorRole]) -> VendorInput {
    VendorInput { name: name.to_string(), roles: roles.to_vec(), ..Default::default() }
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

fn contact(name: &str, role: &str, email: &str, phone: &str) -> VendorContact {
    VendorContact { name: name.into(), role: role.into(), email: email.into(), phone: phone.into() }
}

#[test]
fn a_vendor_holds_contacts_location_minimum_order_quantity_and_lead_time() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = vendor::create_vendor(
        &lib,
        VendorInput {
            name: "Cut Co".into(),
            roles: vec![VendorRole::Manufacturer],
            contacts: vec![
                contact("Ana Ruiz", "Sales", "ana@cut.co", "+1 555 0100"),
                contact("Ben Ito", "Production", "", ""),
            ],
            location: "Los Angeles, CA".into(),
            minimum_order_quantity: Some(150),
            lead_time: "4-6 weeks after Sample approval".into(),
        },
    )
    .unwrap();

    let read = vendor::get_vendor(&lib, created.id).unwrap();
    assert_eq!(read, created);
    assert_eq!(
        read.contacts,
        vec![contact("Ana Ruiz", "Sales", "ana@cut.co", "+1 555 0100"), contact("Ben Ito", "Production", "", "")]
    );
    assert_eq!(read.location, "Los Angeles, CA");
    assert_eq!(read.minimum_order_quantity, Some(150));
    assert_eq!(read.lead_time, "4-6 weeks after Sample approval");
    assert_eq!(vendor::list_vendors(&lib).unwrap(), vec![read]);
}

#[test]
fn a_new_vendor_has_no_details_until_the_founder_adds_them() {
    let env = TestEnv::new();
    let lib = env.library();

    let created = vendor::create_vendor(&lib, input("Bare", &[VendorRole::Mill])).unwrap();

    assert_eq!(created.contacts, vec![]);
    assert_eq!(created.location, "");
    assert_eq!(created.minimum_order_quantity, None);
    assert_eq!(created.lead_time, "");
}

#[test]
fn editing_a_vendor_replaces_its_contacts_and_details() {
    let env = TestEnv::new();
    let lib = env.library();
    let mut first = input("Dye House", &[VendorRole::Decorator]);
    first.contacts = vec![contact("Old Contact", "", "", "")];
    first.minimum_order_quantity = Some(50);
    first.location = "Portland".into();
    let created = vendor::create_vendor(&lib, first).unwrap();

    let mut second = input("Dye House", &[VendorRole::Decorator]);
    second.contacts = vec![contact("New Contact", "Owner", "new@dye.house", "")];
    second.lead_time = "2 weeks".into();
    vendor::update_vendor(&lib, created.id, second).unwrap();

    let read = vendor::get_vendor(&lib, created.id).unwrap();
    assert_eq!(read.contacts, vec![contact("New Contact", "Owner", "new@dye.house", "")]);
    assert_eq!(read.location, "");
    assert_eq!(read.minimum_order_quantity, None);
    assert_eq!(read.lead_time, "2 weeks");
}

#[test]
fn contact_and_detail_text_is_saved_without_surrounding_spaces() {
    let env = TestEnv::new();
    let lib = env.library();
    let mut i = input("Trim", &[VendorRole::Mill]);
    i.contacts = vec![contact("  Cleo  ", " Sales ", " c@mill.co ", " 123 ")];
    i.location = "  Osaka ".into();
    i.lead_time = " 3 weeks ".into();

    let created = vendor::create_vendor(&lib, i).unwrap();

    assert_eq!(created.contacts, vec![contact("Cleo", "Sales", "c@mill.co", "123")]);
    assert_eq!(created.location, "Osaka");
    assert_eq!(created.lead_time, "3 weeks");
}

#[test]
fn a_contact_without_a_name_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let mut i = input("Nameless contact", &[VendorRole::Mill]);
    i.contacts = vec![contact("  ", "Sales", "x@y.z", "")];

    let err = vendor::create_vendor(&lib, i).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
    assert_eq!(vendor::list_vendors(&lib).unwrap(), vec![]);
}

#[test]
fn a_minimum_order_quantity_of_zero_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let mut i = input("Zero MOQ", &[VendorRole::Mill]);
    i.minimum_order_quantity = Some(0);

    let err = vendor::create_vendor(&lib, i).unwrap_err();

    assert!(matches!(err, CoreError::InvalidInput { .. }), "{err:?}");
}
